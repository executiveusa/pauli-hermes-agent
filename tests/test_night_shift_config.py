from night_shift.config import NightShiftConfig


def test_missing_secrets_shape(monkeypatch):
    monkeypatch.delenv("OPENAI_API_KEY", raising=False)
    cfg = NightShiftConfig()
    missing = cfg.missing_secrets
    assert "OPENAI_API_KEY" in missing
    assert isinstance(missing["OPENAI_API_KEY"], bool)
