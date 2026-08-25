from __future__ import annotations

from types import SimpleNamespace

import pytest

from gateway.voice_model_control import (
    natural_language_model_command,
    preprocess_telegram_voice_model_control,
)


@pytest.mark.parametrize(
    ("spoken", "expected"),
    [
        ("What model is running?", "/model"),
        ("Which model are you using?", "/model"),
        ("What's the current model?", "/model"),
        ("switch to GLM", "/model GLM"),
        ("Switch the model to Claude", "/model Claude"),
        ("use Gemini for this session", "/model Gemini"),
        ("make Gemini the global default", "/model Gemini --global"),
        ("set model to deepseek globally", "/model deepseek --global"),
    ],
)
def test_natural_language_model_command(spoken, expected):
    assert natural_language_model_command(spoken) == expected


@pytest.mark.parametrize(
    "spoken",
    [
        "use Claude to summarize this document",
        "tell me about the GPT-5 model",
        "compare Gemini and Claude",
        "/model glm",
    ],
)
def test_natural_language_model_command_rejects_ordinary_prompts(spoken):
    assert natural_language_model_command(spoken) is None


@pytest.mark.asyncio
async def test_telegram_voice_is_transcribed_once_and_promoted_to_model_command():
    event = SimpleNamespace(
        text="",
        message_type=SimpleNamespace(value="voice"),
        source=SimpleNamespace(platform=SimpleNamespace(value="telegram")),
        media_urls=["/tmp/voice.ogg"],
        media_types=["audio/ogg"],
    )
    calls = []

    def transcriber(path):
        calls.append(path)
        return {"success": True, "transcript": "switch to GLM"}

    matched = await preprocess_telegram_voice_model_control(event, transcriber=transcriber)

    assert matched is True
    assert calls == ["/tmp/voice.ogg"]
    assert event.text == "/model GLM"
    assert event.media_urls == []
    assert event.media_types == []


@pytest.mark.asyncio
async def test_normal_telegram_voice_is_promoted_to_text_without_changing_model():
    event = SimpleNamespace(
        text="",
        message_type=SimpleNamespace(value="voice"),
        source=SimpleNamespace(platform=SimpleNamespace(value="telegram")),
        media_urls=["/tmp/voice.ogg"],
        media_types=["audio/ogg"],
    )

    matched = await preprocess_telegram_voice_model_control(
        event,
        transcriber=lambda _: {"success": True, "transcript": "summarize today's projects"},
    )

    assert matched is False
    assert event.text == "summarize today's projects"
    assert event.media_urls == []


@pytest.mark.asyncio
async def test_non_telegram_voice_is_untouched():
    event = SimpleNamespace(
        text="",
        message_type=SimpleNamespace(value="voice"),
        source=SimpleNamespace(platform=SimpleNamespace(value="discord")),
        media_urls=["/tmp/voice.ogg"],
        media_types=["audio/ogg"],
    )

    matched = await preprocess_telegram_voice_model_control(
        event,
        transcriber=lambda _: {"success": True, "transcript": "switch to GLM"},
    )

    assert matched is False
    assert event.text == ""
    assert event.media_urls == ["/tmp/voice.ogg"]
