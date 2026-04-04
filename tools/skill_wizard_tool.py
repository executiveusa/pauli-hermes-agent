"""Skill Wizard — AI-powered skill generation tool.

Generates new SKILL.md files from a description, following the skill-creation-law
template. Uses the agent's own LLM to draft the skill content, then writes
it to the skills directory.

This enables the agent to self-improve by creating skills from experience.
"""

import json
import logging
import os
import re
from pathlib import Path

from tools.registry import registry

logger = logging.getLogger(__name__)


def _get_skills_dir() -> Path:
    """Get the user skills directory (~/.hermes/skills/)."""
    home = Path(os.getenv("HERMES_HOME", Path.home() / ".hermes"))
    skills = home / "skills"
    skills.mkdir(parents=True, exist_ok=True)
    return skills


def skill_create(
    name: str,
    description: str,
    rules: str = "",
    checklist: str = "",
    platforms: str = "cli,telegram,discord,slack,api",
    task_id: str = None,
) -> str:
    """Create a new skill from a description."""
    # Sanitize name
    safe_name = re.sub(r'[^a-z0-9-]', '-', name.lower().strip())
    safe_name = re.sub(r'-+', '-', safe_name).strip('-')
    if not safe_name:
        return json.dumps({"success": False, "error": "Invalid skill name"})

    skills_dir = _get_skills_dir()
    skill_dir = skills_dir / safe_name
    skill_file = skill_dir / "SKILL.md"

    if skill_file.exists():
        return json.dumps({"success": False, "error": f"Skill '{safe_name}' already exists at {skill_file}"})

    skill_dir.mkdir(parents=True, exist_ok=True)

    # Build SKILL.md from template
    platforms_list = [p.strip() for p in platforms.split(",") if p.strip()]
    platform_str = ", ".join(platforms_list)

    content = f"""---
name: {safe_name}
description: {description}
platforms: [{platform_str}]
---

# {name}

## Purpose
{description}

## Rules

{rules if rules else "1. [Define the rules for this skill]"}

## Checklist
{checklist if checklist else "- [ ] [Add verification items]"}
"""

    skill_file.write_text(content.strip() + "\n", encoding="utf-8")
    logger.info("Created skill: %s at %s", safe_name, skill_file)

    return json.dumps({
        "success": True,
        "name": safe_name,
        "path": str(skill_file),
        "message": f"Skill '{safe_name}' created. Use skill_manage to edit it later.",
    })


def skill_list_all(task_id: str = None) -> str:
    """List all skills (user + project)."""
    skills = []

    # User skills
    user_dir = _get_skills_dir()
    if user_dir.is_dir():
        for skill_dir in sorted(user_dir.iterdir()):
            skill_file = skill_dir / "SKILL.md"
            if skill_file.is_file():
                skills.append({
                    "name": skill_dir.name,
                    "source": "user",
                    "path": str(skill_file),
                })

    # Project skills (in workspace)
    project_root = Path(__file__).parent.parent
    project_skills = project_root / "skills"
    if project_skills.is_dir():
        for skill_dir in sorted(project_skills.iterdir()):
            if skill_dir.is_dir():
                skill_file = skill_dir / "SKILL.md"
                if skill_file.is_file():
                    skills.append({
                        "name": skill_dir.name,
                        "source": "project",
                        "path": str(skill_file),
                    })

    return json.dumps({
        "success": True,
        "count": len(skills),
        "skills": skills,
    })


# ---------------------------------------------------------------------------
# Registry
# ---------------------------------------------------------------------------

registry.register(
    name="skill_wizard",
    toolset="skills",
    schema={
        "name": "skill_wizard",
        "description": (
            "Create a new reusable skill from experience. After completing a "
            "complex task, fixing a tricky error, or discovering a workflow, "
            "use this to generate a SKILL.md with rules and checklists. "
            "The skill will be available for future sessions."
        ),
        "parameters": {
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Skill name (lowercase, hyphens OK, e.g. 'api-migration')",
                },
                "description": {
                    "type": "string",
                    "description": "What this skill governs (one sentence)",
                },
                "rules": {
                    "type": "string",
                    "description": "Numbered rules in markdown (e.g. '1. Always validate input\\n2. Use typed schemas')",
                },
                "checklist": {
                    "type": "string",
                    "description": "Markdown checklist items (e.g. '- [ ] Input validation\\n- [ ] Error handling')",
                },
                "platforms": {
                    "type": "string",
                    "description": "Comma-separated platforms (default: 'cli,telegram,discord,slack,api')",
                },
            },
            "required": ["name", "description"],
        },
    },
    handler=lambda args, **kw: skill_create(
        name=args["name"],
        description=args["description"],
        rules=args.get("rules", ""),
        checklist=args.get("checklist", ""),
        platforms=args.get("platforms", "cli,telegram,discord,slack,api"),
        task_id=kw.get("task_id"),
    ),
    emoji="🧙",
)
