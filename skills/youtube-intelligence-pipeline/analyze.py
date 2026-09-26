"""
Claude-powered analysis layer.
Per-video insight extraction + cross-channel synthesis.
Routes through ANTHROPIC_BASE_URL if set (NIM proxy, etc.).
"""
import json
import os
from typing import Any

import anthropic

MODEL = os.getenv("PIPELINE_MODEL", "claude-sonnet-4-6")

_client: anthropic.Anthropic | None = None


def _get_client() -> anthropic.Anthropic:
    global _client
    if _client is None:
        _client = anthropic.Anthropic()
    return _client


# ---------------------------------------------------------------------------
# Prompts
# ---------------------------------------------------------------------------

_VIDEO_PROMPT = """\
Analyze this YouTube video and extract structured intelligence.

Video:
  Title: {title}
  Channel: {channel}
  Views: {views}
  Upload date: {date}
  Description: {description}
  Transcript excerpt: {transcript}

User goals:
{goals}

Return ONLY valid JSON (no markdown fences) with these keys:
{{
  "topic": "one-sentence core topic",
  "insights": ["key claim or insight", ...],
  "tools": ["tool or product mentioned", ...],
  "techniques": ["method or technique shown", ...],
  "actions": ["specific action relevant to user goals", ...],
  "content_ideas": ["inspired video/post idea", ...],
  "follow_up_people": ["person or brand worth following", ...]
}}"""

_SYNTHESIS_PROMPT = """\
You are a strategic analyst building an actionable second brain from YouTube research.

Stats: {video_count} videos across {channel_count} channels analyzed.

User goals:
{goals}

Sample of per-video analyses (JSON):
{sample}

Produce a cross-channel synthesis. Return ONLY valid JSON (no markdown fences):
{{
  "executive_summary": "2-3 sentence overview of what you found",
  "trends": ["dominant trend observed across channels", ...],
  "gaps": ["content gap = opportunity not being covered", ...],
  "top_actions": [
    {{"action": "specific action", "reason": "why high impact", "urgency": "now|soon|later"}},
    ...
  ],
  "content_calendar": [
    {{"title": "post or video title", "format": "short|long|thread|reel", "angle": "unique angle", "reasoning": "why this will work"}},
    ...
  ],
  "top_tools": ["tool worth using or reviewing", ...],
  "collaborators": ["person or channel worth reaching out to", ...],
  "skills_to_develop": ["skill that high performers in this space have", ...]
}}

Limit: 10 top_actions, 15 content_calendar items, 10 each for the rest."""


# ---------------------------------------------------------------------------
# Per-video analysis
# ---------------------------------------------------------------------------

def _parse_json_response(text: str) -> dict[str, Any]:
    """Strip markdown fences if present and parse JSON."""
    text = text.strip()
    if text.startswith("```"):
        lines = text.splitlines()
        text = "\n".join(lines[1:-1] if lines[-1].strip() == "```" else lines[1:])
    return json.loads(text)


def analyze_video(video: dict[str, Any], goals: list[str]) -> dict[str, Any]:
    transcript = video.get("transcript", "")
    transcript_excerpt = transcript[:2500] if transcript else "(no transcript available)"

    prompt = _VIDEO_PROMPT.format(
        title=video.get("video_title", ""),
        channel=video.get("channel_name", ""),
        views=video.get("view_count", ""),
        date=video.get("upload_date", ""),
        description=(video.get("description") or "")[:1200],
        transcript=transcript_excerpt,
        goals="\n".join(f"- {g}" for g in goals) if goals else "Not specified",
    )

    client = _get_client()
    response = client.messages.create(
        model=MODEL,
        max_tokens=1024,
        messages=[{"role": "user", "content": prompt}],
    )

    try:
        analysis = _parse_json_response(response.content[0].text)
    except Exception:
        analysis = {"raw": response.content[0].text}

    return {**video, "analysis": analysis}


def analyze_batch(
    videos: list[dict[str, Any]],
    goals: list[str],
    max_videos: int = 100,
) -> list[dict[str, Any]]:
    """Analyze each video with Claude up to max_videos (controls API cost)."""
    sample = videos[:max_videos]
    remainder = videos[max_videos:]
    results: list[dict[str, Any]] = []

    for i, video in enumerate(sample):
        print(f"  [{i + 1}/{len(sample)}] {video.get('video_title', '')[:65]}")
        try:
            results.append(analyze_video(video, goals))
        except Exception as e:
            print(f"         [WARN] Analysis failed: {e}")
            results.append(video)

    results.extend(remainder)
    return results


# ---------------------------------------------------------------------------
# Cross-channel synthesis
# ---------------------------------------------------------------------------

def cross_channel_synthesis(
    analyzed: list[dict[str, Any]],
    goals: list[str],
) -> dict[str, Any]:
    """Synthesize patterns and actions across all analyzed videos."""
    analyzed_videos = [v for v in analyzed if "analysis" in v]
    channels = sorted({v.get("channel_name", "") for v in analyzed if v.get("channel_name")})

    # Build a compact sample that fits in context
    sample_items = []
    for v in analyzed_videos[:40]:
        sample_items.append(
            {
                "title": v.get("video_title", ""),
                "channel": v.get("channel_name", ""),
                "views": v.get("view_count", ""),
                "analysis": v["analysis"],
            }
        )
    sample_text = json.dumps(sample_items, indent=2, ensure_ascii=False)[:22000]

    prompt = _SYNTHESIS_PROMPT.format(
        video_count=len(analyzed),
        channel_count=len(channels),
        goals="\n".join(f"- {g}" for g in goals) if goals else "Not specified",
        sample=sample_text,
    )

    client = _get_client()
    response = client.messages.create(
        model=MODEL,
        max_tokens=4096,
        messages=[{"role": "user", "content": prompt}],
    )

    try:
        synthesis = _parse_json_response(response.content[0].text)
    except Exception:
        synthesis = {"raw": response.content[0].text}

    return {
        "synthesis": synthesis,
        "videos": analyzed,
        "channels": channels,
        "total_videos": len(analyzed),
        "analyzed_videos": len(analyzed_videos),
        "goals": goals,
    }
