from pauli.mcp.registry import inspect_all_modules


def test_module_registry_reports_known_modules():
    modules = inspect_all_modules()
    names = {module["name"] for module in modules}
    assert "browser-harness" in names
    assert "jcodemunch-mcp" in names
    assert "OpenChronicle" in names


def test_openchronicle_is_platform_blocked_on_windows_or_linux():
    modules = {module["name"]: module for module in inspect_all_modules()}
    openchronicle = modules["OpenChronicle"]
    if openchronicle["platform_ok"]:
        assert openchronicle["status"] in {"installed", "blocked"}
    else:
        assert "platform_blocked" in ",".join(openchronicle["blockers"])
