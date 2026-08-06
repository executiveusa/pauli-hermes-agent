"""
Output writers for the YouTube Intelligence Pipeline.
Produces Obsidian-compatible Markdown, structured JSON, and a content calendar.
"""
import json
from datetime import datetime
from pathlib import Path
from typing import Any


def _ts() -> str:
    return datetime.now().strftime("%Y-%m-%d %H:%M")


def _list_section(items: list[Any], numbered: bool = False) -> list[str]:
    lines = []
    for i, item in enumerate(items, 1):
        if isinstance(item, dict):
            # Flatten dict to readable string
            parts = [f"**{v}**" if k in ("action", "title") else str(v) for k, v in item.items() if v]
            text = " — ".join(parts)
        else:
            text = str(item)
        prefix = f"{i}." if numbered else "-"
        lines.append(f"{prefix} {text}")
    return lines


def write_second_brain(data: dict[str, Any], out_path: Path) -> None:
    """
    Obsidian-compatible Markdown second brain note.
    Tags, YAML frontmatter, and linked sections so it drops straight into a vault.
    """
    syn = data.get("synthesis", {})
    goals = data.get("goals", [])
    channels = data.get("channels", [])

    tags = ["youtube-intelligence", "research", "content-strategy"]
    lines = [
        "---",
        f"created: {datetime.now().strftime('%Y-%m-%d')}",
        f"tags: [{', '.join(tags)}]",
        f"channels: {len(channels)}",
        f"videos_scraped: {data.get('total_videos', 0)}",
        f"videos_analyzed: {data.get('analyzed_videos', 0)}",
        "---",
        "",
        "# YouTube Intelligence Report",
        f"*Generated {_ts()}*",
        "",
        "## Executive Summary",
        syn.get("executive_summary", "_No summary generated._"),
        "",
        "## Goals",
        *[f"- {g}" for g in goals],
        "",
        "## Trends",
        *_list_section(syn.get("trends", [])),
        "",
        "## Content Gaps (Your Opportunities)",
        *_list_section(syn.get("gaps", [])),
        "",
        "## Top Actions",
        *_list_section(syn.get("top_actions", []), numbered=True),
        "",
        "## Content Calendar Ideas",
        *_list_section(syn.get("content_calendar", []), numbered=True),
        "",
        "## Tools Worth Using",
        *_list_section(syn.get("top_tools", [])),
        "",
        "## Collaborators to Reach Out To",
        *_list_section(syn.get("collaborators", [])),
        "",
        "## Skills to Develop",
        *_list_section(syn.get("skills_to_develop", [])),
        "",
        "## Channels Analyzed",
        *[f"- {ch}" for ch in channels],
    ]

    out_path.write_text("\n".join(lines), encoding="utf-8")
    print(f"  Second brain  → {out_path}")


def write_actions(data: dict[str, Any], out_path: Path) -> None:
    """Structured JSON of all action items — feed into task managers, Notion, etc."""
    syn = data.get("synthesis", {})
    payload = {
        "generated_at": datetime.now().isoformat(),
        "executive_summary": syn.get("executive_summary", ""),
        "top_actions": syn.get("top_actions", []),
        "content_calendar": syn.get("content_calendar", []),
        "collaborators": syn.get("collaborators", []),
        "tools": syn.get("top_tools", []),
        "skills": syn.get("skills_to_develop", []),
        "gaps": syn.get("gaps", []),
        "trends": syn.get("trends", []),
        "channels": data.get("channels", []),
        "stats": {
            "total_videos": data.get("total_videos", 0),
            "analyzed_videos": data.get("analyzed_videos", 0),
        },
    }
    out_path.write_text(json.dumps(payload, indent=2, ensure_ascii=False))
    print(f"  Actions JSON  → {out_path}")


def write_content_calendar(data: dict[str, Any], out_path: Path) -> None:
    """Markdown content calendar with format and angle details."""
    syn = data.get("synthesis", {})
    calendar = syn.get("content_calendar", [])

    lines = [
        "# Content Calendar",
        f"*Generated {_ts()} | {len(calendar)} ideas*",
        "",
    ]

    for i, item in enumerate(calendar, 1):
        if isinstance(item, dict):
            title = item.get("title", f"Idea {i}")
            fmt = item.get("format", "")
            angle = item.get("angle", "")
            reasoning = item.get("reasoning", "")
            lines.extend([
                f"## {i}. {title}",
                f"**Format:** {fmt}" if fmt else "",
                f"**Angle:** {angle}" if angle else "",
                f"**Why it works:** {reasoning}" if reasoning else "",
                "- [ ] Draft outline",
                "- [ ] Record / write",
                "- [ ] Publish",
                "",
            ])
        else:
            lines.extend([f"## {i}. {item}", "- [ ] Draft", "- [ ] Publish", ""])

    out_path.write_text("\n".join(l for l in lines), encoding="utf-8")
    print(f"  Calendar      → {out_path}")


def write_per_video_insights(data: dict[str, Any], out_path: Path) -> None:
    """One section per analyzed video with extracted insights and actions."""
    videos = data.get("videos", [])
    analyzed = [v for v in videos if "analysis" in v]

    lines = [
        "# Per-Video Insights",
        f"*Generated {_ts()} | {len(analyzed)} videos*",
        "",
    ]

    for v in analyzed:
        a = v["analysis"]
        lines.extend([
            f"## {v.get('video_title', 'Untitled')}",
            f"*{v.get('channel_name', '')} | {v.get('view_count', '')} | {v.get('upload_date', '')}*",
            f"[Watch]({v.get('video_url', '')})" if v.get("video_url") else "",
            "",
            f"**Topic:** {a.get('topic', '')}",
            "",
        ])

        insights = a.get("insights") or []
        if insights:
            lines.append("**Key Insights:**")
            lines.extend(f"- {ins}" for ins in insights)
            lines.append("")

        actions = a.get("actions") or []
        if actions:
            lines.append("**Actions:**")
            lines.extend(f"- {act}" for act in actions)
            lines.append("")

        tools = a.get("tools") or []
        if tools:
            lines.append(f"**Tools mentioned:** {', '.join(tools)}")
            lines.append("")

        content_ideas = a.get("content_ideas") or []
        if content_ideas:
            lines.append("**Content ideas inspired:**")
            lines.extend(f"- {idea}" for idea in content_ideas)
            lines.append("")

        lines.extend(["---", ""])

    out_path.write_text("\n".join(lines), encoding="utf-8")
    print(f"  Per-video     → {out_path}")
