"""
pauli/flywheel/dispatchers/openclaude_dispatcher.py

Flywheel dispatcher for the OpenClaude isolated coding worker.

Hermes is the orchestrator. OpenClaude is a leaf worker. This module manages
the full lifecycle of dispatching a bead (task unit) to OpenClaude:

1. Load and validate the worker registry config.
2. Reject tasks with denied task types immediately.
3. Check that the worker binary exists (returns clear error if not).
4. Optionally start the worker if it is not running.
5. Select the cheapest available model/provider.
6. Dispatch the task via:
   - gRPC  (preferred, if worker is in headless mode and port is open)
   - CLI   (fallback — subprocess with stdin/stdout capture)
7. Capture output, redact secrets, return structured result.
8. Update bead status.

Usage:
    from pauli.flywheel.dispatchers.openclaude_dispatcher import OpenClaudeDispatcher

    dispatcher = OpenClaudeDispatcher()
    result = dispatcher.dispatch({
        "bead_id": "bead_abc123",
        "task_type": "refactor",
        "description": "Extract auth logic from gateway/run.py into gateway/auth.py",
        "repo_path": "/path/to/repo",
    })
"""

from __future__ import annotations

import json
import logging
import os
import re
import shutil
import socket
import subprocess
import time
import uuid
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

import yaml

logger = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

_REGISTRY_PATH_DEFAULT = Path(__file__).resolve().parents[3] / "config" / "pauli_worker_registry.yaml"

_SECRET_PATTERN = re.compile(
    r"(?:sk-|gsk_|xai-|Bearer |key-)[A-Za-z0-9_\-]{12,}",
    re.IGNORECASE,
)

_REDACTED = "***REDACTED***"


# ---------------------------------------------------------------------------
# Exceptions
# ---------------------------------------------------------------------------


class DeniedTaskTypeError(ValueError):
    """Raised when a bead's task_type is in the deny-list."""


class WorkerNotInstalledError(RuntimeError):
    """Raised when the openclaude binary cannot be found."""


class WorkerDispatchError(RuntimeError):
    """Raised when the worker subprocess fails unexpectedly."""


class ApprovalRequiredError(RuntimeError):
    """Raised when a bead needs human approval before dispatch."""


# ---------------------------------------------------------------------------
# Data classes
# ---------------------------------------------------------------------------


@dataclass
class BeadSpec:
    """Structured representation of a task bead."""

    bead_id: str
    task_type: str
    description: str
    repo_path: str = ""
    allowed_files: list[str] = field(default_factory=list)
    max_tokens: int = 8192
    timeout_seconds: int = 600
    metadata: dict[str, Any] = field(default_factory=dict)

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "BeadSpec":
        return cls(
            bead_id=data.get("bead_id", f"bead_{uuid.uuid4().hex[:8]}"),
            task_type=data["task_type"],
            description=data["description"],
            repo_path=data.get("repo_path", ""),
            allowed_files=data.get("allowed_files", []),
            max_tokens=data.get("max_tokens", 8192),
            timeout_seconds=data.get("timeout_seconds", 600),
            metadata=data.get("metadata", {}),
        )


@dataclass
class DispatchResult:
    """Structured result returned by the dispatcher."""

    status: str                          # "success" | "failed" | "blocked"
    bead_id: str
    model_used: str = ""
    provider: str = ""
    logs: str = ""
    files_changed: list[str] = field(default_factory=list)
    test_results: dict[str, int] | None = None
    duration_seconds: float = 0.0
    error: str | None = None

    def to_dict(self) -> dict[str, Any]:
        return {
            "status": self.status,
            "bead_id": self.bead_id,
            "model_used": self.model_used,
            "provider": self.provider,
            "logs": self.logs,
            "files_changed": self.files_changed,
            "test_results": self.test_results,
            "duration_seconds": round(self.duration_seconds, 2),
            "error": self.error,
        }


# ---------------------------------------------------------------------------
# Registry loader
# ---------------------------------------------------------------------------


def load_worker_registry(path: Path | str | None = None) -> dict[str, Any]:
    """Load and validate config/pauli_worker_registry.yaml."""
    registry_path = Path(path) if path else _REGISTRY_PATH_DEFAULT
    if not registry_path.exists():
        raise FileNotFoundError(
            f"Worker registry not found: {registry_path}. "
            "Expected at config/pauli_worker_registry.yaml"
        )
    with registry_path.open() as fh:
        data = yaml.safe_load(fh)
    if not isinstance(data, dict) or "workers" not in data:
        raise ValueError(f"Invalid worker registry at {registry_path}: missing 'workers' key")
    return data


def get_worker_config(registry: dict[str, Any], worker_name: str = "openclaude") -> dict[str, Any]:
    """Extract a specific worker's config from the registry."""
    workers = registry.get("workers", {})
    if worker_name not in workers:
        raise KeyError(f"Worker '{worker_name}' not found in registry")
    return workers[worker_name]


# ---------------------------------------------------------------------------
# Binary discovery
# ---------------------------------------------------------------------------


def find_openclaude_binary(repo_root: Path | None = None) -> str | None:
    """
    Locate the openclaude binary.

    Search order:
    1. vendor/openclaude/bin/openclaude
    2. vendor/openclaude/node_modules/.bin/openclaude
    3. System PATH
    """
    root = repo_root or Path(__file__).resolve().parents[3]
    vendor = root / "vendor" / "openclaude"

    candidates = [
        vendor / "bin" / "openclaude",
        vendor / "node_modules" / ".bin" / "openclaude",
    ]
    for candidate in candidates:
        if candidate.exists() and os.access(str(candidate), os.X_OK):
            return str(candidate)

    # Fall back to system PATH
    system_bin = shutil.which("openclaude")
    if system_bin:
        return system_bin

    return None


# ---------------------------------------------------------------------------
# Model / provider selection
# ---------------------------------------------------------------------------


def select_model(worker_config: dict[str, Any]) -> tuple[str, str]:
    """
    Select the cheapest available model/provider.

    Returns (provider_name, model_name). Iterates the model_priority list
    from worker config and returns the first entry whose env_check variable
    is set (or which doesn't have an env_check).

    Special case: 'ollama' is considered available if OLLAMA_HOST is set
    or if the `ollama` command exists on PATH.
    """
    model_priority = worker_config.get("model_priority", [])

    for entry in model_priority:
        provider = entry.get("provider", "")
        model = entry.get("model", "")
        env_check = entry.get("env_check", "")

        if provider == "ollama":
            if os.environ.get("OLLAMA_HOST") or shutil.which("ollama"):
                return provider, model
            continue

        if env_check:
            if os.environ.get(env_check):
                return provider, model
            continue

        # No env_check means always available
        return provider, model

    # Hard fallback — let openclaude use its own config
    return "auto", "auto"


# ---------------------------------------------------------------------------
# Health check
# ---------------------------------------------------------------------------


def _is_port_open(host: str, port: int, timeout: float = 3.0) -> bool:
    """Return True if a TCP connection can be established to host:port."""
    try:
        with socket.create_connection((host, port), timeout=timeout):
            return True
    except OSError:
        return False


def healthcheck(worker_config: dict[str, Any], repo_root: Path | None = None) -> dict[str, Any]:
    """
    Check worker health.

    Returns a dict:
        {
          "binary_found": bool,
          "binary_path": str | None,
          "version": str | None,
          "grpc_port_open": bool,
          "error": str | None,
        }
    """
    result: dict[str, Any] = {
        "binary_found": False,
        "binary_path": None,
        "version": None,
        "grpc_port_open": False,
        "error": None,
    }

    binary = find_openclaude_binary(repo_root)
    if not binary:
        result["error"] = (
            "openclaude binary not found in vendor/openclaude/bin/, "
            "vendor/openclaude/node_modules/.bin/, or system PATH. "
            "Run: scripts/pauli/openclaude/install.sh"
        )
        return result

    result["binary_found"] = True
    result["binary_path"] = binary

    # Get version
    try:
        proc = subprocess.run(
            [binary, "--version"],
            capture_output=True,
            text=True,
            timeout=10,
        )
        if proc.returncode == 0:
            result["version"] = proc.stdout.strip() or proc.stderr.strip()
        else:
            result["error"] = f"openclaude --version exited {proc.returncode}"
    except (subprocess.TimeoutExpired, FileNotFoundError) as exc:
        result["error"] = f"binary check failed: {exc}"

    # Check gRPC port
    grpc_cfg = worker_config.get("grpc", {})
    port = int(grpc_cfg.get("port", 50051))
    host = grpc_cfg.get("host", "localhost")
    result["grpc_port_open"] = _is_port_open(host, port, timeout=3.0)

    return result


# ---------------------------------------------------------------------------
# Prompt builder
# ---------------------------------------------------------------------------


def _build_prompt(bead: BeadSpec) -> str:
    """Build the task prompt string from a bead spec."""
    lines = [
        f"TASK ID: {bead.bead_id}",
        f"TASK TYPE: {bead.task_type}",
        "",
        "DESCRIPTION:",
        bead.description,
    ]
    if bead.repo_path:
        lines += ["", f"REPOSITORY: {bead.repo_path}"]
    if bead.allowed_files:
        lines += ["", "ALLOWED FILES (restrict changes to these):"]
        lines += [f"  - {f}" for f in bead.allowed_files]
    lines += [
        "",
        "OUTPUT CONTRACT:",
        "- Make only the changes described.",
        "- Do not modify files outside the ALLOWED FILES list (if specified).",
        "- Do not commit or push — only make local file edits.",
        "- Do not access secrets, environment credentials, or deploy to any service.",
        "- Report all changed files in your final summary.",
        "- If tests exist for the changed files, run them and report the results.",
    ]
    return "\n".join(lines)


# ---------------------------------------------------------------------------
# Secret redaction
# ---------------------------------------------------------------------------


def _redact_secrets(text: str) -> str:
    """Replace recognizable API key patterns with REDACTED."""
    return _SECRET_PATTERN.sub(_REDACTED, text)


# ---------------------------------------------------------------------------
# Core dispatcher
# ---------------------------------------------------------------------------


class OpenClaudeDispatcher:
    """
    Dispatch coding beads to the OpenClaude worker.

    Instantiate once per process (config is loaded at init time).

    Example:
        dispatcher = OpenClaudeDispatcher()
        result = dispatcher.dispatch(bead_dict)
    """

    def __init__(
        self,
        registry_path: Path | str | None = None,
        worker_name: str = "openclaude",
        repo_root: Path | None = None,
        require_approval_callback: Any | None = None,
    ) -> None:
        """
        Initialise the dispatcher.

        Args:
            registry_path: Override path to pauli_worker_registry.yaml.
            worker_name: Worker key in the registry (default: "openclaude").
            repo_root: Override repo root for binary discovery.
            require_approval_callback: Optional callable(bead, action) -> bool
                that prompts the user for approval. If None, approval-required
                tasks raise ApprovalRequiredError immediately.
        """
        self.registry = load_worker_registry(registry_path)
        self.worker_config = get_worker_config(self.registry, worker_name)
        self.repo_root = repo_root or Path(__file__).resolve().parents[3]
        self.require_approval_callback = require_approval_callback

        self._denied_task_types: set[str] = set(
            self.worker_config.get("denied_task_types", [])
        )
        self._requires_approval_for: set[str] = set(
            self.worker_config.get("requires_approval_for", [])
        )
        self._allowed_task_types: set[str] = set(
            self.worker_config.get("allowed_task_types", [])
        )
        self._timeout: int = int(
            self.worker_config.get("cli", {}).get("timeout_seconds", 600)
        )

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    def dispatch(self, bead: dict[str, Any] | BeadSpec) -> dict[str, Any]:
        """
        Dispatch a bead to OpenClaude.

        Args:
            bead: Either a raw dict with at minimum {task_type, description}
                  or a BeadSpec instance.

        Returns:
            DispatchResult.to_dict() — always a plain dict.

        Raises:
            DeniedTaskTypeError:    task_type is in the deny-list.
            WorkerNotInstalledError: binary not found.
            ApprovalRequiredError:  approval needed but no callback set.
        """
        if isinstance(bead, dict):
            bead = BeadSpec.from_dict(bead)

        start_time = time.monotonic()

        # 1. Gate: deny-list check
        self._check_denied(bead)

        # 2. Gate: approval check
        self._check_approval(bead)

        # 3. Ensure binary exists
        binary = find_openclaude_binary(self.repo_root)
        if not binary:
            raise WorkerNotInstalledError(
                "openclaude binary not found. Run: scripts/pauli/openclaude/install.sh"
            )

        # 4. Select model
        provider, model = select_model(self.worker_config)

        # 5. Dispatch (gRPC preferred, CLI fallback)
        grpc_cfg = self.worker_config.get("grpc", {})
        grpc_host = grpc_cfg.get("host", "localhost")
        grpc_port = int(grpc_cfg.get("port", 50051))

        if _is_port_open(grpc_host, grpc_port, timeout=2.0):
            logs, files_changed, test_results, error = self._dispatch_grpc(
                bead, binary, provider, model, grpc_host, grpc_port
            )
        else:
            logs, files_changed, test_results, error = self._dispatch_cli(
                bead, binary, provider, model
            )

        duration = time.monotonic() - start_time
        status = "failed" if error else "success"

        result = DispatchResult(
            status=status,
            bead_id=bead.bead_id,
            model_used=model,
            provider=provider,
            logs=_redact_secrets(logs),
            files_changed=files_changed,
            test_results=test_results,
            duration_seconds=duration,
            error=error,
        )
        logger.info(
            "Dispatched bead %s: status=%s, provider=%s, model=%s, duration=%.1fs",
            bead.bead_id,
            status,
            provider,
            model,
            duration,
        )
        return result.to_dict()

    # ------------------------------------------------------------------
    # Internal gates
    # ------------------------------------------------------------------

    def _check_denied(self, bead: BeadSpec) -> None:
        if bead.task_type in self._denied_task_types:
            raise DeniedTaskTypeError(
                f"Task type '{bead.task_type}' is in the deny-list for the OpenClaude worker. "
                f"Denied types: {sorted(self._denied_task_types)}. "
                "This bead must be handled by a different mechanism or discarded."
            )

    def _check_approval(self, bead: BeadSpec) -> None:
        """Check if the bead requires approval for any of its metadata flags."""
        requested_actions = set(bead.metadata.get("requires_actions", []))
        flagged = requested_actions & self._requires_approval_for
        if not flagged:
            return

        if self.require_approval_callback is None:
            raise ApprovalRequiredError(
                f"Bead {bead.bead_id} requires human approval for: {sorted(flagged)}. "
                "Set require_approval_callback on the dispatcher or approve manually."
            )

        approved = self.require_approval_callback(bead, sorted(flagged))
        if not approved:
            raise ApprovalRequiredError(
                f"Human approval denied for bead {bead.bead_id} (actions: {sorted(flagged)})"
            )

    # ------------------------------------------------------------------
    # Dispatch implementations
    # ------------------------------------------------------------------

    def _dispatch_cli(
        self,
        bead: BeadSpec,
        binary: str,
        provider: str,
        model: str,
    ) -> tuple[str, list[str], dict[str, int] | None, str | None]:
        """
        Dispatch via CLI subprocess (fallback mode).

        Returns (logs, files_changed, test_results, error_or_None).
        """
        prompt = _build_prompt(bead)
        env = self._build_env(provider, model)

        # Build command — use --print for non-interactive headless mode if supported
        cmd = [binary, "--print", prompt]

        workdir = bead.repo_path if bead.repo_path and Path(bead.repo_path).is_dir() else None

        logger.debug("CLI dispatch: cmd=%s, workdir=%s", cmd, workdir)

        try:
            proc = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=bead.timeout_seconds or self._timeout,
                env=env,
                cwd=workdir,
            )
        except subprocess.TimeoutExpired:
            return "", [], None, f"Worker timed out after {bead.timeout_seconds}s"
        except FileNotFoundError as exc:
            return "", [], None, f"Binary not found: {exc}"
        except Exception as exc:  # noqa: BLE001
            return "", [], None, f"Dispatch failed: {exc}"

        output = proc.stdout + proc.stderr
        error: str | None = None

        if proc.returncode != 0:
            error = f"openclaude exited {proc.returncode}"

        files_changed = _parse_files_changed(output)
        test_results = _parse_test_results(output)

        return output, files_changed, test_results, error

    def _dispatch_grpc(
        self,
        bead: BeadSpec,
        binary: str,
        provider: str,
        model: str,
        host: str,
        port: int,
    ) -> tuple[str, list[str], dict[str, int] | None, str | None]:
        """
        Dispatch via gRPC (preferred, when worker is in headless mode).

        Currently implemented as a CLI call with --grpc-endpoint flag,
        which is supported by OpenClaude's headless mode for task injection.
        Full gRPC protobuf streaming will be wired in a future iteration.
        """
        prompt = _build_prompt(bead)
        env = self._build_env(provider, model)

        cmd = [
            binary,
            "--grpc-endpoint", f"{host}:{port}",
            "--print", prompt,
        ]

        workdir = bead.repo_path if bead.repo_path and Path(bead.repo_path).is_dir() else None

        logger.debug("gRPC dispatch: cmd=%s, workdir=%s", cmd, workdir)

        try:
            proc = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=bead.timeout_seconds or self._timeout,
                env=env,
                cwd=workdir,
            )
        except subprocess.TimeoutExpired:
            return "", [], None, f"Worker timed out after {bead.timeout_seconds}s"
        except Exception as exc:  # noqa: BLE001
            return "", [], None, f"gRPC dispatch failed: {exc}"

        output = proc.stdout + proc.stderr
        error: str | None = None

        if proc.returncode != 0:
            error = f"openclaude gRPC dispatch exited {proc.returncode}"

        files_changed = _parse_files_changed(output)
        test_results = _parse_test_results(output)

        return output, files_changed, test_results, error

    # ------------------------------------------------------------------
    # Environment builder
    # ------------------------------------------------------------------

    def _build_env(self, provider: str, model: str) -> dict[str, str]:
        """
        Build subprocess environment.

        Passes through only the env vars listed in worker_config.cli.env_passthrough,
        plus sets OPENAI_MODEL and OPENAI_BASE_URL for the selected provider.
        Never includes raw keys beyond what the operator has already set.
        """
        passthrough_keys: list[str] = (
            self.worker_config.get("cli", {}).get("env_passthrough", [])
        )

        env: dict[str, str] = {}

        # Always inherit PATH so the binary can find node, npm, etc.
        env["PATH"] = os.environ.get("PATH", "")

        # Inherit HOME and TERM for subprocess sanity
        for k in ("HOME", "USERPROFILE", "TERM", "LANG", "TMPDIR", "TMP", "TEMP"):
            if k in os.environ:
                env[k] = os.environ[k]

        # Pass through explicit allow-listed env vars
        for key in passthrough_keys:
            if key in os.environ:
                env[key] = os.environ[key]

        # Set model hint for openclaude
        if model and model != "auto":
            env["OPENAI_MODEL"] = model

        # Provider-specific base URL (don't override if user set it explicitly)
        if provider == "openrouter" and "OPENAI_BASE_URL" not in env:
            env["OPENAI_BASE_URL"] = "https://openrouter.ai/api/v1"
            if "OPENROUTER_API_KEY" in os.environ and "OPENAI_API_KEY" not in env:
                env["OPENAI_API_KEY"] = os.environ["OPENROUTER_API_KEY"]
        elif provider == "groq" and "OPENAI_BASE_URL" not in env:
            env["OPENAI_BASE_URL"] = "https://api.groq.com/openai/v1"
            if "GROQ_API_KEY" in os.environ and "OPENAI_API_KEY" not in env:
                env["OPENAI_API_KEY"] = os.environ["GROQ_API_KEY"]
        elif provider == "deepseek" and "OPENAI_BASE_URL" not in env:
            env["OPENAI_BASE_URL"] = "https://api.deepseek.com/v1"
            if "DEEPSEEK_API_KEY" in os.environ and "OPENAI_API_KEY" not in env:
                env["OPENAI_API_KEY"] = os.environ["DEEPSEEK_API_KEY"]
        elif provider == "ollama" and "OPENAI_BASE_URL" not in env:
            ollama_host = os.environ.get("OLLAMA_HOST", "http://localhost:11434")
            env["OPENAI_BASE_URL"] = f"{ollama_host.rstrip('/')}/v1"
            env.setdefault("OPENAI_API_KEY", "ollama")

        return env


# ---------------------------------------------------------------------------
# Output parsers
# ---------------------------------------------------------------------------


def _parse_files_changed(output: str) -> list[str]:
    """
    Extract file paths from openclaude output.

    OpenClaude typically outputs lines like:
      Modified: path/to/file.py
      Created:  path/to/other.py
      Deleted:  path/to/gone.py
    """
    files: list[str] = []
    pattern = re.compile(
        r"^(?:Modified|Created|Deleted|Updated|Wrote|Saved):\s+(.+)$",
        re.IGNORECASE | re.MULTILINE,
    )
    for match in pattern.finditer(output):
        path = match.group(1).strip()
        if path and path not in files:
            files.append(path)
    return files


def _parse_test_results(output: str) -> dict[str, int] | None:
    """
    Extract test pass/fail counts from openclaude output.

    Handles common test runner formats:
      - pytest: "X passed, Y failed"
      - jest:   "X tests passed, Y failed"
    """
    # pytest-style
    m = re.search(r"(\d+) passed(?:,\s*(\d+) failed)?", output, re.IGNORECASE)
    if m:
        passed = int(m.group(1))
        failed = int(m.group(2)) if m.group(2) else 0
        return {"passed": passed, "failed": failed}

    # jest-style
    m = re.search(r"Tests:\s+(\d+) passed(?:,\s*(\d+) failed)?", output, re.IGNORECASE)
    if m:
        passed = int(m.group(1))
        failed = int(m.group(2)) if m.group(2) else 0
        return {"passed": passed, "failed": failed}

    return None
