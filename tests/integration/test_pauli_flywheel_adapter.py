from pauli.flywheel.adapter import build_flywheel_bootstrap_command, inspect_ralphy


def test_ralphy_adapter_reports_vendor_checkout():
    payload = inspect_ralphy()
    assert payload["name"] == "ralphy"
    assert isinstance(payload["blockers"], list)


def test_ralphy_bootstrap_command_is_non_empty():
    cmd = build_flywheel_bootstrap_command()
    assert cmd
    assert "--init" in cmd
