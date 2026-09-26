from __future__ import annotations

import importlib.util
from pathlib import Path

MODULE_PATH = Path(__file__).resolve().parents[1] / "scripts" / "doctor.py"
spec = importlib.util.spec_from_file_location("doctor_21st", MODULE_PATH)
module = importlib.util.module_from_spec(spec)
assert spec and spec.loader
spec.loader.exec_module(module)


def test_missing_config_is_safe(tmp_path: Path) -> None:
    result = module.inspect_config(tmp_path / ".mcp.json")
    assert result["exists"] is False
    assert result["mentions_21st"] is False


def test_env_reference_is_detected_without_secret_risk(tmp_path: Path) -> None:
    config = tmp_path / ".mcp.json"
    config.write_text(
        '{"mcpServers":{"21st":{"url":"https://21st.dev/api/mcp",'
        '"headers":{"x-api-key":"${API_KEY_21ST}"}}}}',
        encoding="utf-8",
    )
    result = module.inspect_config(config)
    assert result["mentions_21st"] is True
    assert result["contains_literal_key_pattern"] is False


def test_literal_key_pattern_is_flagged(tmp_path: Path) -> None:
    config = tmp_path / ".mcp.json"
    config.write_text('{"x-api-key":"21st_sk_example-do-not-use"}', encoding="utf-8")
    result = module.inspect_config(config)
    assert result["contains_literal_key_pattern"] is True
