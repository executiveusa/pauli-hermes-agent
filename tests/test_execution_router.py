import pytest

from hermes_execution.backends import InternalBackend, build_default_router
from hermes_execution.router import (
    BackendUnavailable,
    CapabilityManifest,
    ExecutionRequest,
    ExecutionResult,
    ExecutionRouter,
    IsolationClass,
    RequiredIsolationUnavailable,
)


def test_internal_backend_preserves_mission_id():
    router = build_default_router()
    request = ExecutionRequest(mission_id="msn_slice2_test", instruction="Report Hermes status")
    result = router.execute(request)
    assert result.mission_id == request.mission_id
    assert result.backend == "hermes_internal"
    assert result.isolation_class == IsolationClass.PROCESS


def test_native_sandbox_config_does_not_count_as_implemented():
    manifests = {m.backend: m for m in build_default_router().manifests()}
    sandbox = manifests["native_sandbox"]
    assert sandbox.provider == "docker"
    assert sandbox.isolation_class == IsolationClass.CONTAINER
    assert sandbox.implementation_status == "config_only"
    assert sandbox.verified is False
    assert sandbox.available is False


def test_orca_worktree_does_not_claim_vm_isolation():
    manifests = {m.backend: m for m in build_default_router().manifests()}
    orca = manifests["orca"]
    assert orca.isolation_class == IsolationClass.PROCESS
    assert orca.implementation_status == "partial"
    assert orca.available is False


def test_remote_vm_requirement_fails_closed():
    router = build_default_router()
    with pytest.raises(RequiredIsolationUnavailable, match="required_isolation_unavailable:remote_vm"):
        router.execute(
            ExecutionRequest(
                mission_id="msn_remote_vm",
                instruction="Run isolated job",
                required_isolation=IsolationClass.REMOTE_VM,
            )
        )


def test_unverified_native_sandbox_cannot_be_selected():
    router = build_default_router()
    with pytest.raises(BackendUnavailable, match="no_verified_backend_supports_request"):
        router.execute(
            ExecutionRequest(
                mission_id="msn_sandbox",
                instruction="Change fixture",
                preferred_backend="native_sandbox",
            )
        )


def test_unknown_capability_fails_closed():
    router = build_default_router()
    with pytest.raises(BackendUnavailable):
        router.execute(
            ExecutionRequest(
                mission_id="msn_caps",
                instruction="Use unsupported capability",
                requested_capabilities=frozenset({"teleport"}),
            )
        )


class _MismatchBackend(InternalBackend):
    def capabilities(self):
        base = super().capabilities()
        return CapabilityManifest(
            backend="mismatch",
            provider=base.provider,
            implementation_status="implemented",
            verified=True,
            isolation_class=base.isolation_class,
            capabilities=base.capabilities,
        )

    def execute(self, prepared):
        return ExecutionResult(
            mission_id="msn_wrong",
            execution_id=prepared.execution_id,
            status="succeeded",
            backend="mismatch",
            provider="test",
            isolation_class=IsolationClass.PROCESS,
        )


def test_mismatched_mission_result_is_rejected():
    router = ExecutionRouter()
    router.register(_MismatchBackend())
    with pytest.raises(BackendUnavailable, match="mission_id_mismatch"):
        router.execute(
            ExecutionRequest(
                mission_id="msn_expected",
                instruction="test",
                preferred_backend="mismatch",
            )
        )


def test_request_validation():
    with pytest.raises(ValueError, match="mission_id"):
        ExecutionRequest(mission_id="", instruction="x")
    with pytest.raises(ValueError, match="instruction"):
        ExecutionRequest(mission_id="msn", instruction="")
    with pytest.raises(ValueError, match="timeout_seconds"):
        ExecutionRequest(mission_id="msn", instruction="x", timeout_seconds=0)
