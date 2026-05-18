from pauli.coursegen.adapter import inspect_coursegen


def test_coursegen_adapter_reports_vendor_checkout():
    payload = inspect_coursegen()
    assert payload["name"] == "codebase-to-course"
    assert isinstance(payload["blockers"], list)
