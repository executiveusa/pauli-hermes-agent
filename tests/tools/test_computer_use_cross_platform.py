from __future__ import annotations

from types import SimpleNamespace

import pytest

from tools.computer_use import permissions, vision_routing


def test_explicit_aux_vision_override_routes_capture_to_aux(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(
        vision_routing,
        "_lookup_user_declared_supports_vision",
        lambda *_args, **_kwargs: None,
    )
    assert vision_routing.should_route_capture_to_aux_vision(
        "openai",
        "gpt-test",
        {"auxiliary": {"vision": {"provider": "openrouter", "model": "vision-model"}}},
    ) is True


def test_declared_vision_capability_keeps_capture_on_main_model(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(
        vision_routing,
        "_lookup_user_declared_supports_vision",
        lambda *_args, **_kwargs: True,
    )
    assert vision_routing.should_route_capture_to_aux_vision(
        "custom",
        "local-vlm",
        {"model": {"supports_vision": True}},
    ) is False


def test_declared_text_only_capability_routes_capture_to_aux(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(
        vision_routing,
        "_lookup_user_declared_supports_vision",
        lambda *_args, **_kwargs: False,
    )
    assert vision_routing.should_route_capture_to_aux_vision(
        "custom",
        "text-model",
        {"model": {"supports_vision": False}},
    ) is True


def test_provider_without_multimodal_tool_results_routes_to_aux(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(
        vision_routing,
        "_lookup_user_declared_supports_vision",
        lambda *_args, **_kwargs: None,
    )
    monkeypatch.setattr(
        vision_routing,
        "_provider_accepts_multimodal_tool_result",
        lambda *_args, **_kwargs: False,
    )
    assert vision_routing.should_route_capture_to_aux_vision(
        "text-provider",
        "model",
        {},
    ) is True


def test_native_vision_model_with_supported_tool_results_stays_multimodal(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(
        vision_routing,
        "_lookup_user_declared_supports_vision",
        lambda *_args, **_kwargs: None,
    )
    monkeypatch.setattr(
        vision_routing,
        "_provider_accepts_multimodal_tool_result",
        lambda *_args, **_kwargs: True,
    )
    monkeypatch.setattr(
        vision_routing,
        "_lookup_supports_vision",
        lambda *_args, **_kwargs: True,
    )
    assert vision_routing.should_route_capture_to_aux_vision(
        "openai",
        "vision-model",
        {},
    ) is False


def test_computer_use_status_without_driver_is_not_installed(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(permissions, "_resolve_driver_cmd", lambda _override: None)
    monkeypatch.setattr(permissions.sys, "platform", "win32")

    status = permissions.computer_use_status()

    assert status["platform"] == "win32"
    assert status["platform_supported"] is True
    assert status["installed"] is False
    assert status["ready"] is None
    assert status["can_grant"] is False


@pytest.mark.parametrize("platform", ["win32", "linux"])
def test_windows_and_linux_readiness_follows_driver_doctor(
    monkeypatch: pytest.MonkeyPatch,
    platform: str,
) -> None:
    monkeypatch.setattr(permissions.sys, "platform", platform)
    monkeypatch.setattr(permissions, "_resolve_driver_cmd", lambda _override: "cua-driver")
    monkeypatch.setattr(
        permissions,
        "_run",
        lambda *_args, **_kwargs: SimpleNamespace(stdout="cua-driver 1.2.3\n"),
    )
    monkeypatch.setattr(
        permissions,
        "_doctor",
        lambda _binary: {
            "ok": True,
            "checks": [{"label": "driver", "status": "ok", "message": "ready"}],
        },
    )

    status = permissions.computer_use_status()

    assert status["installed"] is True
    assert status["ready"] is True
    assert status["can_grant"] is False
    assert status["checks"][0]["status"] == "ok"


def test_macos_readiness_requires_accessibility_and_screen_recording(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(permissions.sys, "platform", "darwin")
    monkeypatch.setattr(permissions, "_resolve_driver_cmd", lambda _override: "cua-driver")
    monkeypatch.setattr(
        permissions,
        "_run",
        lambda *_args, **_kwargs: SimpleNamespace(stdout="cua-driver 1.2.3\n"),
    )
    monkeypatch.setattr(permissions, "_doctor", lambda _binary: {"ok": True, "checks": []})

    def fake_mac_permissions(_binary: str, out: dict) -> None:
        out["accessibility"] = True
        out["screen_recording"] = False

    monkeypatch.setattr(permissions, "_mac_permissions", fake_mac_permissions)

    status = permissions.computer_use_status()

    assert status["can_grant"] is True
    assert status["ready"] is False


def test_non_macos_permission_grant_is_rejected(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(permissions.sys, "platform", "win32")
    assert permissions.request_permissions_grant() == 64
