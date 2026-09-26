#!/usr/bin/env python3
"""Read-only validator for the Vibe Client Factory Hermes skill."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

REQUIRED_FILES = (
    "SKILL.md",
    "references/WORKFLOW.md",
    "templates/decision-card.schema.json",
    "templates/authority-policy.json",
    "templates/run-receipt.schema.json",
)

REQUIRED_SKILL_HEADINGS = (
    "## Purpose",
    "## Final outcome",
    "## ICM operating model",
    "## One production line",
    "## Stage 0 — Outcome contract",
    "## Stage 2 — Client decision cards",
    "## Stage 5 — Verification and Council",
    "## Stage 6 — Judge",
    "## Stage 7 — Release",
    "## Client experience laws",
    "## Authority classes",
    "## Completion record",
)

EXPECTED_AUTHORITY_CLASSES = (
    "READ",
    "ANALYZE",
    "DRAFT",
    "REVERSIBLE_INTERNAL",
    "CLIENT_FACING",
    "RELEASE",
    "SENSITIVE",
    "PROHIBITED",
)

EXPECTED_OVERALL_STATES = (
    "PASS",
    "PASS_WITH_DISPOSITIONS",
    "HOLD",
    "BLOCKED",
    "NOT_RUN",
)


def _load_json(path: Path, errors: list[str]) -> dict[str, Any] | None:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        errors.append(f"Missing required file: {path.name}")
        return None
    except json.JSONDecodeError as exc:
        errors.append(f"Invalid JSON in {path.name}: {exc}")
        return None
    if not isinstance(value, dict):
        errors.append(f"Expected JSON object in {path.name}")
        return None
    return value


def _frontmatter_value(content: str, key: str) -> str | None:
    match = re.search(rf"^{re.escape(key)}:\s*(.+?)\s*$", content, re.MULTILINE)
    return match.group(1).strip() if match else None


def inspect_skill(skill_dir: Path, registry_path: Path | None = None) -> dict[str, Any]:
    skill_dir = skill_dir.resolve()
    errors: list[str] = []
    warnings: list[str] = []

    for relative in REQUIRED_FILES:
        path = skill_dir / relative
        if not path.is_file():
            errors.append(f"Missing required file: {relative}")
            continue
        if not path.read_text(encoding="utf-8").strip():
            errors.append(f"Required file is empty: {relative}")

    skill_path = skill_dir / "SKILL.md"
    if skill_path.is_file():
        content = skill_path.read_text(encoding="utf-8")
        if _frontmatter_value(content, "name") != "vibe-client-factory":
            errors.append("SKILL.md frontmatter name must be vibe-client-factory")
        description = _frontmatter_value(content, "description")
        if not description or len(description) < 40:
            errors.append("SKILL.md must include a useful lazy-load description")
        if "triggers:" not in content:
            errors.append("SKILL.md must declare triggers for lazy loading")
        for heading in REQUIRED_SKILL_HEADINGS:
            if not re.search(rf"^{re.escape(heading)}\s*$", content, re.MULTILINE):
                errors.append(f"SKILL.md is missing heading: {heading}")

    decision = _load_json(skill_dir / "templates" / "decision-card.schema.json", errors)
    if decision:
        properties = decision.get("properties")
        if not isinstance(properties, dict):
            errors.append("Decision schema is missing properties")
        else:
            options = properties.get("options")
            if not isinstance(options, dict):
                errors.append("Decision schema is missing options")
            else:
                if options.get("minItems") != 2 or options.get("maxItems") != 2:
                    errors.append("Decision cards must contain exactly two options")
            recommendation = properties.get("recommendation")
            if not isinstance(recommendation, dict) or recommendation.get("enum") != ["A", "B"]:
                errors.append("Decision recommendation must be A or B")
            not_authorized = properties.get("not_authorized")
            if not isinstance(not_authorized, dict) or not_authorized.get("minItems") != 1:
                errors.append("Decision cards must state at least one non-authorized action")

    authority = _load_json(skill_dir / "templates" / "authority-policy.json", errors)
    if authority:
        classes = authority.get("classes")
        if not isinstance(classes, list):
            errors.append("Authority policy must contain a classes list")
        else:
            actual = [item.get("id") for item in classes if isinstance(item, dict)]
            if actual != list(EXPECTED_AUTHORITY_CLASSES):
                errors.append(
                    "Authority classes must be ordered exactly: "
                    + ", ".join(EXPECTED_AUTHORITY_CLASSES)
                )
            by_id = {
                item.get("id"): item for item in classes if isinstance(item, dict)
            }
            prohibited = by_id.get("PROHIBITED", {})
            if prohibited.get("approval") != "NEVER":
                errors.append("PROHIBITED authority must use approval NEVER")
            release = by_id.get("RELEASE", {})
            if release.get("automatic") is not False:
                errors.append("RELEASE authority must not be automatic")
            sensitive = by_id.get("SENSITIVE", {})
            if sensitive.get("approval") != "DUAL_CONFIRMATION":
                errors.append("SENSITIVE authority must require dual confirmation")

        boundary = authority.get("client_decision_boundary")
        never_direct = boundary.get("never_directly_authorizes") if isinstance(boundary, dict) else None
        if not isinstance(never_direct, list) or "production deployment" not in never_direct:
            errors.append("Client decisions must not directly authorize production deployment")

    receipt = _load_json(skill_dir / "templates" / "run-receipt.schema.json", errors)
    if receipt:
        properties = receipt.get("properties")
        if not isinstance(properties, dict):
            errors.append("Run receipt schema is missing properties")
        else:
            judge = properties.get("judge_verdict")
            if not isinstance(judge, dict) or judge.get("enum") != ["SHIP", "HOLD"]:
                errors.append("Judge verdict must be exactly SHIP or HOLD")
            overall = properties.get("overall_status")
            if not isinstance(overall, dict) or overall.get("enum") != list(EXPECTED_OVERALL_STATES):
                errors.append("Run receipt overall states do not match the workflow contract")

    if registry_path is not None:
        registry = _load_json(registry_path, errors)
        if registry is not None:
            entry = registry.get("vibe-client-factory")
            if not isinstance(entry, dict):
                errors.append("Skill registry is missing vibe-client-factory")
            else:
                if entry.get("path") != "skills/studio/vibe-client-factory":
                    errors.append("Skill registry path is incorrect")
                if entry.get("enabled") is not True:
                    errors.append("vibe-client-factory must be enabled")
                if entry.get("required") is True:
                    errors.append("vibe-client-factory must remain lazy-loaded and optional")

    return {
        "skill": "vibe-client-factory",
        "skill_dir": str(skill_dir),
        "status": "PASS" if not errors else "BLOCKED",
        "errors": errors,
        "warnings": warnings,
        "checked_files": len(REQUIRED_FILES),
    }


def build_parser() -> argparse.ArgumentParser:
    default_skill = Path(__file__).resolve().parents[1]
    default_registry = default_skill.parents[1] / "SKILL_REGISTRY.json"
    parser = argparse.ArgumentParser(
        description="Validate the Vibe Client Factory skill without changing state."
    )
    parser.add_argument("--skill-dir", type=Path, default=default_skill)
    parser.add_argument("--registry", type=Path, default=default_registry)
    parser.add_argument("--json", action="store_true", dest="as_json")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    report = inspect_skill(args.skill_dir, args.registry)
    if args.as_json:
        json.dump(report, sys.stdout, indent=2)
        sys.stdout.write("\n")
    else:
        print(f"Vibe Client Factory Doctor: {report['status']}")
        print(f"Skill directory: {report['skill_dir']}")
        print(f"Files checked: {report['checked_files']}")
        for warning in report["warnings"]:
            print(f"WARNING: {warning}")
        for error in report["errors"]:
            print(f"ERROR: {error}", file=sys.stderr)
    return 0 if report["status"] == "PASS" else 1


if __name__ == "__main__":
    raise SystemExit(main())
