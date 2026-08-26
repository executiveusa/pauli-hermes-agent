"""Hermes execution backends.

Only the internal diagnostic backend is implemented here. Native sandbox and
Orca are explicit fail-closed seams until their actual runners are wired and
runtime-verified. Configuration alone never counts as implementation.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Mapping

from .router import (
    BackendCapabilities,
    BackendUnavailable,
    CapabilityManifest,
    ExecutionRequest,
    ExecutionResult,
    IsolationClass,
    new_execution_id,
)


@dataclass(frozen=True)
class PreparedInternal:
    request: ExecutionRequest
    execution_id: str


class InternalBackend:
    """Harmless in-process diagnostic backend; no shell or repository mutation."""

    def capabilities(self) -> CapabilityManifest:
        return CapabilityManifest(
            backend="hermes_internal",
            provider="hermes",
            implementation_status="implemented",
            verified=True,
            isolation_class=IsolationClass.PROCESS,
            capabilities=BackendCapabilities(),
        )

    def prepare(self, request: ExecutionRequest) -> PreparedInternal:
        return PreparedInternal(request=request, execution_id=new_execution_id("hexec"))

    def execute(self, prepared: PreparedInternal) -> ExecutionResult:
        request = prepared.request
        return ExecutionResult(
            mission_id=request.mission_id,
            execution_id=prepared.execution_id,
            status="succeeded",
            backend="hermes_internal",
            provider="hermes",
            isolation_class=IsolationClass.PROCESS,
            summary=f"Hermes accepted diagnostic instruction: {request.instruction}",
            cleanup={"required": False, "status": "not_applicable"},
        )

    def status(self, execution_id: str) -> Mapping[str, Any]:
        return {"executionId": execution_id, "status": "completed"}

    def cancel(self, execution_id: str) -> Mapping[str, Any]:
        return {"executionId": execution_id, "cancelled": False, "reason": "not_long_running"}

    def collect_artifacts(self, execution_id: str) -> tuple[Mapping[str, Any], ...]:
        return ()

    def cleanup(self, execution_id: str) -> Mapping[str, Any]:
        return {"executionId": execution_id, "status": "not_applicable"}


class UnavailableBackend:
    """A registered integration seam that cannot be selected until verified."""

    def __init__(self, manifest: CapabilityManifest) -> None:
        if manifest.available:
            raise ValueError("UnavailableBackend requires an unavailable manifest")
        self._manifest = manifest

    def capabilities(self) -> CapabilityManifest:
        return self._manifest

    def _fail(self) -> None:
        raise BackendUnavailable(
            f"backend_unavailable:{self._manifest.backend}:{self._manifest.unavailable_reason or 'unverified'}"
        )

    def prepare(self, request: ExecutionRequest) -> Any:
        self._fail()

    def execute(self, prepared: Any) -> ExecutionResult:
        self._fail()

    def status(self, execution_id: str) -> Mapping[str, Any]:
        self._fail()

    def cancel(self, execution_id: str) -> Mapping[str, Any]:
        self._fail()

    def collect_artifacts(self, execution_id: str) -> tuple[Mapping[str, Any], ...]:
        self._fail()

    def cleanup(self, execution_id: str) -> Mapping[str, Any]:
        return {"executionId": execution_id, "status": "nothing_allocated"}


def native_sandbox_backend() -> UnavailableBackend:
    # .env.sandcastle.example names providers, but canonical main has not yet
    # supplied a verified runner. Docker is truthfully a container boundary.
    return UnavailableBackend(
        CapabilityManifest(
            backend="native_sandbox",
            provider="docker",
            implementation_status="config_only",
            verified=False,
            isolation_class=IsolationClass.CONTAINER,
            capabilities=BackendCapabilities(),
            unavailable_reason="sandcastle_provider_config_exists_but_runner_not_verified",
        )
    )


def orca_backend() -> UnavailableBackend:
    # Orca's SSH worktree foundation is workspace isolation, not VM proof.
    return UnavailableBackend(
        CapabilityManifest(
            backend="orca",
            provider="orca_worktree",
            implementation_status="partial",
            verified=False,
            isolation_class=IsolationClass.PROCESS,
            capabilities=BackendCapabilities(
                code=False,
                shell=False,
                repo_read=False,
                repo_write=False,
                tests=False,
                browser=False,
                internet=False,
                scraping=False,
                computer_use=False,
                long_running=False,
                persistent_workspace=False,
                parallel_agents=False,
                cancellation=False,
            ),
            unavailable_reason="machine_addressable_orca_adapter_not_runtime_verified",
        )
    )


def build_default_router():
    from .router import ExecutionRouter

    router = ExecutionRouter()
    router.register(InternalBackend())
    router.register(native_sandbox_backend())
    router.register(orca_backend())
    return router
