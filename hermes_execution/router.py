"""Fail-closed Hermes execution backend registry/router.

This module intentionally contains no Docker/Orca implementation details.
Backends register truthful capabilities; the router refuses unavailable
capabilities and isolation levels rather than silently downgrading them.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Mapping, Protocol
import uuid


class IsolationClass(str, Enum):
    PROCESS = "process"
    CONTAINER = "container"
    REMOTE_VM = "remote_vm"
    MICROVM = "microvm"


@dataclass(frozen=True)
class BackendCapabilities:
    code: bool = False
    shell: bool = False
    repo_read: bool = False
    repo_write: bool = False
    tests: bool = False
    browser: bool = False
    internet: bool = False
    scraping: bool = False
    computer_use: bool = False
    long_running: bool = False
    persistent_workspace: bool = False
    parallel_agents: bool = False
    cancellation: bool = False

    def supports(self, requested: frozenset[str]) -> bool:
        known = self.__dataclass_fields__
        return all(name in known and bool(getattr(self, name)) for name in requested)


@dataclass(frozen=True)
class CapabilityManifest:
    backend: str
    provider: str
    implementation_status: str
    verified: bool
    isolation_class: IsolationClass
    capabilities: BackendCapabilities = field(default_factory=BackendCapabilities)
    unavailable_reason: str | None = None

    @property
    def available(self) -> bool:
        return self.implementation_status == "implemented" and self.verified and not self.unavailable_reason


@dataclass(frozen=True)
class ExecutionRequest:
    mission_id: str
    instruction: str
    requested_capabilities: frozenset[str] = frozenset()
    required_isolation: IsolationClass | None = None
    preferred_backend: str | None = None
    timeout_seconds: int = 300
    context: Mapping[str, Any] = field(default_factory=dict)

    def __post_init__(self) -> None:
        if not self.mission_id.strip():
            raise ValueError("mission_id is required")
        if not self.instruction.strip():
            raise ValueError("instruction is required")
        if self.timeout_seconds <= 0:
            raise ValueError("timeout_seconds must be positive")


@dataclass(frozen=True)
class ExecutionResult:
    mission_id: str
    execution_id: str
    status: str
    backend: str
    provider: str
    isolation_class: IsolationClass
    summary: str = ""
    artifacts: tuple[Mapping[str, Any], ...] = ()
    error: Mapping[str, Any] | None = None
    worker_id: str | None = None
    cleanup: Mapping[str, Any] | None = None


class BackendUnavailable(RuntimeError):
    pass


class RequiredIsolationUnavailable(BackendUnavailable):
    pass


class ExecutionBackend(Protocol):
    def capabilities(self) -> CapabilityManifest: ...
    def prepare(self, request: ExecutionRequest) -> Any: ...
    def execute(self, prepared: Any) -> ExecutionResult: ...
    def status(self, execution_id: str) -> Mapping[str, Any]: ...
    def cancel(self, execution_id: str) -> Mapping[str, Any]: ...
    def collect_artifacts(self, execution_id: str) -> tuple[Mapping[str, Any], ...]: ...
    def cleanup(self, execution_id: str) -> Mapping[str, Any]: ...


class ExecutionRouter:
    """Selects one verified backend without capability/isolation downgrade."""

    def __init__(self) -> None:
        self._backends: dict[str, ExecutionBackend] = {}

    def register(self, backend: ExecutionBackend) -> None:
        manifest = backend.capabilities()
        if not manifest.backend:
            raise ValueError("backend manifest requires backend id")
        self._backends[manifest.backend] = backend

    def manifests(self) -> tuple[CapabilityManifest, ...]:
        return tuple(backend.capabilities() for backend in self._backends.values())

    def select(self, request: ExecutionRequest) -> ExecutionBackend:
        candidates = list(self._backends.values())
        if request.preferred_backend:
            candidates = [b for b in candidates if b.capabilities().backend == request.preferred_backend]
            if not candidates:
                raise BackendUnavailable(f"unknown backend: {request.preferred_backend}")

        isolation_seen = False
        for backend in candidates:
            manifest = backend.capabilities()
            if request.required_isolation is not None:
                if manifest.isolation_class != request.required_isolation:
                    continue
                isolation_seen = True
            if not manifest.available:
                continue
            if not manifest.capabilities.supports(request.requested_capabilities):
                continue
            return backend

        if request.required_isolation is not None and not isolation_seen:
            raise RequiredIsolationUnavailable(
                f"required_isolation_unavailable:{request.required_isolation.value}"
            )
        raise BackendUnavailable("no_verified_backend_supports_request")

    def execute(self, request: ExecutionRequest) -> ExecutionResult:
        backend = self.select(request)
        prepared = backend.prepare(request)
        result = backend.execute(prepared)
        if result.mission_id != request.mission_id:
            # A result may never complete somebody else's mission.
            try:
                backend.cleanup(result.execution_id)
            finally:
                raise BackendUnavailable("mission_id_mismatch")
        return result


def new_execution_id(prefix: str) -> str:
    return f"{prefix}_{uuid.uuid4().hex}"
