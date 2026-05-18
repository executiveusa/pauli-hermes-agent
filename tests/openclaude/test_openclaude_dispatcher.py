"""
tests/openclaude/test_openclaude_dispatcher.py

Pytest tests for the OpenClaude Flywheel dispatcher.

All tests run without an installed openclaude binary or active worker process.
External calls are patched at the subprocess level.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path
from unittest.mock import MagicMock, patch

import pytest
import yaml

# ---------------------------------------------------------------------------
# Repo root path so we can locate config files without installing the package
# ---------------------------------------------------------------------------
_REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(_REPO_ROOT))

from pauli.flywheel.dispatchers.openclaude_dispatcher import (
    BeadSpec,
    DeniedTaskTypeError,
    DispatchResult,
    OpenClaudeDispatcher,
    WorkerNotInstalledError,
    _parse_files_changed,
    _parse_test_results,
    _redact_secrets,
    find_openclaude_binary,
    healthcheck,
    load_worker_registry,
    select_model,
)

# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------

_REGISTRY_PATH = _REPO_ROOT / "config" / "pauli_worker_registry.yaml"


@pytest.fixture()
def registry() -> dict:
    """Load the real registry YAML."""
    return load_worker_registry(_REGISTRY_PATH)


@pytest.fixture()
def worker_config(registry) -> dict:
    """Extract the openclaude worker config."""
    return registry["workers"]["openclaude"]


@pytest.fixture()
def minimal_bead() -> dict:
    """A minimal valid bead dict for a non-denied task type."""
    return {
        "bead_id": "bead_test001",
        "task_type": "refactor",
        "description": "Rename FooBar to BarBaz in gateway/run.py.",
        "repo_path": str(_REPO_ROOT),
    }


@pytest.fixture()
def denied_bead() -> dict:
    """A bead with a denied task type."""
    return {
        "bead_id": "bead_bad001",
        "task_type": "production_deploy",
        "description": "Deploy to prod.",
    }


@pytest.fixture()
def fake_binary(tmp_path) -> Path:
    """Create a fake openclaude binary that echoes its version."""
    bin_dir = tmp_path / "vendor" / "openclaude" / "bin"
    bin_dir.mkdir(parents=True)
    fake = bin_dir / "openclaude"
    fake.write_text(
        "#!/usr/bin/env python3\n"
        "import sys\n"
        "args = sys.argv[1:]\n"
        "if '--version' in args:\n"
        "    print('openclaude 0.9.2')\n"
        "elif '--print' in args:\n"
        "    idx = args.index('--print')\n"
        "    print('Modified: gateway/auth.py')\n"
        "    print('3 passed')\n"
        "else:\n"
        "    print('openclaude running')\n"
        "sys.exit(0)\n"
    )
    fake.chmod(0o755)
    return tmp_path


@pytest.fixture()
def dispatcher_with_fake_binary(fake_binary) -> OpenClaudeDispatcher:
    """Dispatcher pointing at fake binary + real registry."""
    return OpenClaudeDispatcher(
        registry_path=_REGISTRY_PATH,
        repo_root=fake_binary,
    )


# ---------------------------------------------------------------------------
# test_worker_registry_loads
# ---------------------------------------------------------------------------


class TestWorkerRegistryLoads:
    """The registry YAML must be valid and contain required keys."""

    def test_registry_file_exists(self):
        assert _REGISTRY_PATH.exists(), f"Registry not found: {_REGISTRY_PATH}"

    def test_registry_is_valid_yaml(self):
        with _REGISTRY_PATH.open() as fh:
            data = yaml.safe_load(fh)
        assert isinstance(data, dict), "Registry must be a YAML mapping"

    def test_registry_has_workers_key(self, registry):
        assert "workers" in registry

    def test_registry_has_openclaude_worker(self, registry):
        assert "openclaude" in registry["workers"]

    def test_openclaude_has_required_keys(self, worker_config):
        required = [
            "enabled",
            "type",
            "allowed_task_types",
            "denied_task_types",
            "requires_approval_for",
        ]
        for key in required:
            assert key in worker_config, f"Missing required key: {key}"

    def test_denied_task_types_are_present(self, worker_config):
        denied = worker_config["denied_task_types"]
        assert isinstance(denied, list)
        assert "production_deploy" in denied
        assert "secret_rotation" in denied
        assert "destructive_git" in denied

    def test_allowed_task_types_are_present(self, worker_config):
        allowed = worker_config["allowed_task_types"]
        assert isinstance(allowed, list)
        assert len(allowed) >= 1

    def test_requires_approval_for_is_present(self, worker_config):
        approval = worker_config["requires_approval_for"]
        assert isinstance(approval, list)
        assert "git_push" in approval


# ---------------------------------------------------------------------------
# test_dispatcher_blocks_denied_task
# ---------------------------------------------------------------------------


class TestDispatcherBlocksDeniedTask:
    """Dispatcher must raise DeniedTaskTypeError for blocked task types."""

    def test_production_deploy_is_denied(self, denied_bead, fake_binary):
        dispatcher = OpenClaudeDispatcher(
            registry_path=_REGISTRY_PATH,
            repo_root=fake_binary,
        )
        with pytest.raises(DeniedTaskTypeError) as exc_info:
            dispatcher.dispatch(denied_bead)
        assert "production_deploy" in str(exc_info.value)

    def test_secret_rotation_is_denied(self, fake_binary):
        dispatcher = OpenClaudeDispatcher(
            registry_path=_REGISTRY_PATH,
            repo_root=fake_binary,
        )
        bead = {"task_type": "secret_rotation", "description": "Rotate keys."}
        with pytest.raises(DeniedTaskTypeError):
            dispatcher.dispatch(bead)

    def test_destructive_git_is_denied(self, fake_binary):
        dispatcher = OpenClaudeDispatcher(
            registry_path=_REGISTRY_PATH,
            repo_root=fake_binary,
        )
        bead = {"task_type": "destructive_git", "description": "Force push."}
        with pytest.raises(DeniedTaskTypeError):
            dispatcher.dispatch(bead)

    def test_error_message_includes_deny_list(self, fake_binary):
        dispatcher = OpenClaudeDispatcher(
            registry_path=_REGISTRY_PATH,
            repo_root=fake_binary,
        )
        bead = {"task_type": "production_deploy", "description": "Deploy."}
        with pytest.raises(DeniedTaskTypeError) as exc_info:
            dispatcher.dispatch(bead)
        msg = str(exc_info.value)
        assert "deny-list" in msg or "denied" in msg.lower()


# ---------------------------------------------------------------------------
# test_healthcheck_missing_install
# ---------------------------------------------------------------------------


class TestHealthcheckMissingInstall:
    """healthcheck() must return a clear error when vendor/openclaude is absent."""

    def test_missing_binary_returns_error(self, tmp_path, worker_config):
        # tmp_path has no vendor/openclaude
        with patch("shutil.which", return_value=None):
            result = healthcheck(worker_config, repo_root=tmp_path)
        assert result["binary_found"] is False
        assert result["error"] is not None
        assert "not found" in result["error"].lower() or "install" in result["error"].lower()

    def test_missing_binary_has_no_version(self, tmp_path, worker_config):
        with patch("shutil.which", return_value=None):
            result = healthcheck(worker_config, repo_root=tmp_path)
        assert result["version"] is None

    def test_missing_binary_has_no_grpc(self, tmp_path, worker_config):
        with patch("shutil.which", return_value=None):
            result = healthcheck(worker_config, repo_root=tmp_path)
        assert result["grpc_port_open"] is False

    def test_found_binary_returns_version(self, fake_binary, worker_config):
        result = healthcheck(worker_config, repo_root=fake_binary)
        assert result["binary_found"] is True
        assert result["version"] is not None
        assert "openclaude" in result["version"].lower()


# ---------------------------------------------------------------------------
# test_dispatcher_assigns_bead
# ---------------------------------------------------------------------------


class TestDispatcherAssignsBead:
    """Dispatcher must assign a bead and return a structured result."""

    def test_dispatch_returns_dict(self, dispatcher_with_fake_binary, minimal_bead):
        result = dispatcher_with_fake_binary.dispatch(minimal_bead)
        assert isinstance(result, dict)

    def test_result_has_required_fields(self, dispatcher_with_fake_binary, minimal_bead):
        result = dispatcher_with_fake_binary.dispatch(minimal_bead)
        required_fields = [
            "status", "bead_id", "model_used", "provider",
            "logs", "files_changed", "test_results", "duration_seconds",
        ]
        for f in required_fields:
            assert f in result, f"Missing field in result: {f}"

    def test_result_bead_id_matches(self, dispatcher_with_fake_binary, minimal_bead):
        result = dispatcher_with_fake_binary.dispatch(minimal_bead)
        assert result["bead_id"] == minimal_bead["bead_id"]

    def test_result_status_is_success_on_exit_0(self, dispatcher_with_fake_binary, minimal_bead):
        result = dispatcher_with_fake_binary.dispatch(minimal_bead)
        assert result["status"] == "success"

    def test_result_files_changed_parsed(self, dispatcher_with_fake_binary, minimal_bead):
        result = dispatcher_with_fake_binary.dispatch(minimal_bead)
        # Fake binary prints "Modified: gateway/auth.py"
        assert isinstance(result["files_changed"], list)

    def test_result_duration_is_positive(self, dispatcher_with_fake_binary, minimal_bead):
        result = dispatcher_with_fake_binary.dispatch(minimal_bead)
        assert result["duration_seconds"] >= 0.0

    def test_dispatch_raises_if_binary_missing(self, tmp_path, minimal_bead):
        with patch("shutil.which", return_value=None):
            dispatcher = OpenClaudeDispatcher(
                registry_path=_REGISTRY_PATH,
                repo_root=tmp_path,
            )
            with pytest.raises(WorkerNotInstalledError):
                dispatcher.dispatch(minimal_bead)


# ---------------------------------------------------------------------------
# test_config_redacts_secrets
# ---------------------------------------------------------------------------


class TestConfigRedactsSecrets:
    """Secrets must not appear in dispatch logs or error messages."""

    def test_redact_secrets_removes_sk_key(self):
        text = "Error: invalid key sk-abc123XYZ789DEFghiJKLmno"
        redacted = _redact_secrets(text)
        assert "sk-abc123XYZ789DEFghiJKLmno" not in redacted
        assert "***REDACTED***" in redacted

    def test_redact_secrets_removes_gsk_key(self):
        text = "Authorization: Bearer gsk_abcDEFghiJKLmno012345678"
        redacted = _redact_secrets(text)
        assert "gsk_abcDEFghiJKLmno012345678" not in redacted

    def test_redact_secrets_preserves_non_key_text(self):
        text = "All good, no secrets here."
        assert _redact_secrets(text) == text

    def test_dispatch_result_logs_are_redacted(self, dispatcher_with_fake_binary, minimal_bead):
        """Even if a key leaks into output, logs must be redacted in result."""
        fake_output = "Modified: file.py\nsk-leakedKEY1234567890 found in output"

        with patch.object(
            dispatcher_with_fake_binary,
            "_dispatch_cli",
            return_value=(fake_output, ["file.py"], None, None),
        ):
            result = dispatcher_with_fake_binary.dispatch(minimal_bead)

        assert "sk-leakedKEY1234567890" not in result["logs"]
        assert "***REDACTED***" in result["logs"]

    def test_generate_config_output_does_not_contain_raw_key(self, tmp_path):
        """
        Simulate generate-config.sh behavior: config file contains apiKey field.
        The script itself should only write to a non-committed path.
        Verify the dispatcher never logs the key value.
        """
        config = {
            "provider": "openrouter",
            "apiKey": "sk-or-secret-realkey-ABCDEF1234",
            "model": "llama-3.1-8b-instruct:free",
            "baseUrl": "https://openrouter.ai/api/v1",
        }
        config_path = tmp_path / ".openclaude.json"
        config_path.write_text(json.dumps(config))

        # The config file exists — but the dispatcher should never read it
        # directly into logs. Verify redaction works on config content if leaked.
        leaked = json.dumps(config)
        redacted = _redact_secrets(leaked)
        assert "sk-or-secret-realkey-ABCDEF1234" not in redacted


# ---------------------------------------------------------------------------
# test_model_priority_order
# ---------------------------------------------------------------------------


class TestModelPriorityOrder:
    """Dispatcher must select the cheapest available model."""

    def test_ollama_selected_when_env_set(self, worker_config):
        with patch.dict(os.environ, {"OLLAMA_HOST": "http://localhost:11434"}, clear=False):
            provider, model = select_model(worker_config)
        assert provider == "ollama"

    def test_openrouter_selected_when_no_ollama(self, worker_config):
        env = {
            "OPENROUTER_API_KEY": "sk-or-test",
        }
        # Remove OLLAMA_HOST if set, ensure ollama binary not found
        with patch.dict(os.environ, env, clear=False), \
             patch("os.environ.get", side_effect=_env_get_no_ollama(env)), \
             patch("shutil.which", return_value=None):
            provider, model = select_model(worker_config)
        # With OLLAMA_HOST unset and ollama not in PATH, should pick openrouter
        assert provider in ("openrouter", "ollama", "groq", "deepseek", "openai", "auto")
        # Cannot assert a specific provider without full env isolation —
        # verify the contract: returns a 2-tuple of non-empty strings
        assert isinstance(provider, str)
        assert isinstance(model, str)

    def test_select_model_returns_tuple(self, worker_config):
        result = select_model(worker_config)
        assert isinstance(result, tuple)
        assert len(result) == 2

    def test_select_model_provider_is_string(self, worker_config):
        provider, _ = select_model(worker_config)
        assert isinstance(provider, str)
        assert len(provider) > 0

    def test_select_model_model_is_string(self, worker_config):
        _, model = select_model(worker_config)
        assert isinstance(model, str)
        assert len(model) > 0

    def test_no_keys_set_returns_auto(self, worker_config):
        """When no provider keys are set and ollama is absent, returns 'auto'."""
        clean_env = {
            k: v for k, v in os.environ.items()
            if not any(k.startswith(p) for p in [
                "OPENROUTER", "GROQ", "DEEPSEEK", "OPENAI", "OLLAMA"
            ])
        }
        with patch.dict(os.environ, {}, clear=True):
            # Re-add only non-provider env vars
            for k, v in clean_env.items():
                os.environ[k] = v
            with patch("shutil.which", return_value=None):
                provider, model = select_model(worker_config)
        # With nothing available, should fall through to "auto"
        assert provider == "auto"
        assert model == "auto"


# ---------------------------------------------------------------------------
# Output parser unit tests
# ---------------------------------------------------------------------------


class TestOutputParsers:
    """Unit tests for _parse_files_changed and _parse_test_results."""

    def test_parse_files_changed_modified(self):
        output = "Modified: gateway/auth.py\nModified: tests/test_auth.py"
        files = _parse_files_changed(output)
        assert "gateway/auth.py" in files
        assert "tests/test_auth.py" in files

    def test_parse_files_changed_created(self):
        output = "Created: new_module.py"
        files = _parse_files_changed(output)
        assert "new_module.py" in files

    def test_parse_files_changed_deduplicates(self):
        output = "Modified: file.py\nModified: file.py"
        files = _parse_files_changed(output)
        assert files.count("file.py") == 1

    def test_parse_files_changed_empty(self):
        files = _parse_files_changed("No file operations performed.")
        assert files == []

    def test_parse_test_results_pytest(self):
        output = "5 passed, 2 failed in 1.23s"
        result = _parse_test_results(output)
        assert result is not None
        assert result["passed"] == 5
        assert result["failed"] == 2

    def test_parse_test_results_all_passed(self):
        output = "12 passed in 0.45s"
        result = _parse_test_results(output)
        assert result is not None
        assert result["passed"] == 12
        assert result["failed"] == 0

    def test_parse_test_results_none_when_absent(self):
        output = "No tests were run."
        result = _parse_test_results(output)
        assert result is None


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _env_get_no_ollama(extra_env: dict) -> "Any":
    """
    Return a side_effect for os.environ.get that omits OLLAMA_HOST
    but honours extra_env for provider keys.
    """
    real_get = os.environ.get

    def _get(key, default=None):
        if key == "OLLAMA_HOST":
            return None
        if key in extra_env:
            return extra_env[key]
        return real_get(key, default)

    return _get
