"""Pre-LLM model-control routing for Telegram voice commands.

Hermes already owns model switching through the gateway ``/model`` command.
This module translates a deliberately narrow set of spoken natural-language
phrases into that existing command before the main LLM is invoked.  The voice
message is transcribed once, promoted to text, and its audio attachment is
removed from downstream media enrichment so normal voice turns are not
transcribed twice.
"""

from __future__ import annotations

import asyncio
import re
from typing import Any, Callable, Optional

_MODEL_ALIASES = (
    r"claude|anthropic|gemini|glm(?:[-\s]?\d+(?:\.\d+)?)?|"
    r"gpt(?:[-\s]?\d+(?:\.\d+)?)?|openai|codex|deepseek|kimi|qwen|"
    r"grok|xai|mercury(?:[-\s]?\d+(?:\.\d+)?)?|big\s+pickle|"
    r"opencode(?:\s+zen|\s+go)?|nvidia"
)

_STATUS_PATTERNS = (
    re.compile(
        r"^(?:what|which)\s+(?:ai\s+)?model(?:\s+(?:is|are)\s+"
        r"(?:running|you\s+(?:using|on)))?\??$",
        re.IGNORECASE,
    ),
    re.compile(
        r"^(?:what(?:'s|\s+is)\s+the\s+(?:current\s+)?model|"
        r"show\s+(?:me\s+)?(?:the\s+)?(?:current\s+)?model|"
        r"model\s+status)\??$",
        re.IGNORECASE,
    ),
)

_SWITCH_PATTERNS = (
    re.compile(
        rf"^(?:switch|change)\s+(?:the\s+)?(?:model\s+)?to\s+"
        rf"(?P<model>{_MODEL_ALIASES}[\w./:+\-\s]*?)"
        rf"(?P<scope>\s+(?:for\s+this\s+session|globally|"
        rf"as\s+(?:the\s+)?(?:global\s+)?default))?[.!?]*$",
        re.IGNORECASE,
    ),
    re.compile(
        rf"^set\s+(?:the\s+)?model\s+to\s+"
        rf"(?P<model>{_MODEL_ALIASES}[\w./:+\-\s]*?)"
        rf"(?P<scope>\s+(?:for\s+this\s+session|globally|"
        rf"as\s+(?:the\s+)?(?:global\s+)?default))?[.!?]*$",
        re.IGNORECASE,
    ),
    re.compile(
        rf"^use\s+(?P<model>{_MODEL_ALIASES}[\w./:+\-\s]*?)"
        rf"(?P<scope>\s+(?:for\s+this\s+session|globally|"
        rf"as\s+(?:the\s+)?(?:global\s+)?default))?[.!?]*$",
        re.IGNORECASE,
    ),
    re.compile(
        rf"^make\s+(?P<model>{_MODEL_ALIASES}[\w./:+\-\s]*?)\s+"
        rf"(?:the\s+)?(?P<scope>global\s+default|default)"
        rf"(?:\s+model)?[.!?]*$",
        re.IGNORECASE,
    ),
)


def natural_language_model_command(text: str) -> Optional[str]:
    """Translate an explicit natural-language model-control phrase.

    Returns a canonical ``/model`` command or ``None``.  The parser is
    intentionally conservative so ordinary prompts mentioning model names do
    not silently change runtime configuration.
    """
    normalized = " ".join((text or "").strip().split())
    if not normalized or normalized.startswith("/"):
        return None

    if any(pattern.match(normalized) for pattern in _STATUS_PATTERNS):
        return "/model"

    for pattern in _SWITCH_PATTERNS:
        match = pattern.match(normalized)
        if not match:
            continue
        model = match.group("model").strip(" .!?")
        scope = (match.groupdict().get("scope") or "").lower()
        persist_global = "global" in scope or "default" in scope
        return f"/model {model}" + (" --global" if persist_global else "")

    return None


def _enum_value(value: Any) -> str:
    return str(getattr(value, "value", value) or "").lower()


async def preprocess_telegram_voice_model_control(
    event: Any,
    *,
    transcriber: Optional[Callable[[str], dict]] = None,
) -> bool:
    """Transcribe a Telegram voice turn before gateway command dispatch.

    On successful transcription the event is promoted to text and the audio
    attachment is removed so downstream media enrichment does not transcribe it
    a second time.  If the transcript is an explicit model-control phrase,
    ``event.text`` becomes the existing canonical ``/model`` command.

    Returns ``True`` only when a model command was recognized.
    """
    source = getattr(event, "source", None)
    if _enum_value(getattr(source, "platform", None)) != "telegram":
        return False
    if _enum_value(getattr(event, "message_type", None)) != "voice":
        return False

    media_urls = list(getattr(event, "media_urls", None) or [])
    if not media_urls:
        return False

    if transcriber is None:
        from tools.transcription_tools import transcribe_audio

        transcriber = transcribe_audio

    try:
        result = await asyncio.to_thread(transcriber, media_urls[0])
    except Exception:
        return False

    if not isinstance(result, dict) or not result.get("success"):
        return False

    transcript = str(result.get("transcript") or "").strip()
    if not transcript:
        return False

    original_text = str(getattr(event, "text", "") or "").strip()
    command = natural_language_model_command(transcript)
    if command:
        event.text = command
    elif original_text:
        event.text = f"{original_text}\n\n{transcript}"
    else:
        event.text = transcript

    # The transcript is now canonical input.  Removing the consumed voice file
    # prevents the normal media-enrichment path from paying for STT twice.
    event.media_urls = []
    event.media_types = []
    return command is not None
