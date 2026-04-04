#!/usr/bin/env python3
"""Conformance trend accumulator and report generator.

Reads the current conformance_summary.json, appends a timestamped entry to
conformance_trend.jsonl, and generates TREND_REPORT.md with trend analysis.

Usage:
  python3 scripts/conformance_trend.py

Environment variables:
  GIT_SHA       Override git sha (default: read from `git rev-parse HEAD`)
  GIT_REF       Override git ref (default: read from `git rev-parse --abbrev-ref HEAD`)
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parent.parent
REPORTS_DIR = REPO_ROOT / "tests" / "ext_conformance" / "reports"
SUMMARY_PATH = REPORTS_DIR / "conformance_summary.json"
TREND_PATH = REPORTS_DIR / "conformance_trend.jsonl"
TREND_REPORT_PATH = REPORTS_DIR / "TREND_REPORT.md"

SCHEMA_VERSION = "pi.ext.conformance_trend_entry.v1"


def git_info() -> tuple[str, str]:
    """Return (git_sha, git_ref) from env or git commands."""
    sha = os.environ.get("GIT_SHA", "").strip()
    ref = os.environ.get("GIT_REF", "").strip()
    if not sha:
        try:
            sha = subprocess.check_output(
                ["git", "rev-parse", "HEAD"],
                cwd=str(REPO_ROOT),
                text=True,
                stderr=subprocess.DEVNULL,
            ).strip()
        except (subprocess.CalledProcessError, FileNotFoundError):
            sha = "unknown"
    if not ref:
        try:
            ref = subprocess.check_output(
                ["git", "rev-parse", "--abbrev-ref", "HEAD"],
                cwd=str(REPO_ROOT),
                text=True,
                stderr=subprocess.DEVNULL,
            ).strip()
        except (subprocess.CalledProcessError, FileNotFoundError):
            ref = "unknown"
    return sha, ref


def load_summary() -> dict[str, Any]:
    """Load conformance_summary.json."""
    if not SUMMARY_PATH.is_file():
        print(f"ERROR: missing {SUMMARY_PATH}", file=sys.stderr)
        sys.exit(1)
    with SUMMARY_PATH.open(encoding="utf-8") as fh:
        return json.load(fh)


def load_trend() -> list[dict[str, Any]]:
    """Load existing trend entries from conformance_trend.jsonl."""
    entries: list[dict[str, Any]] = []
    if not TREND_PATH.is_file():
        return entries
    with TREND_PATH.open(encoding="utf-8") as fh:
        for line in fh:
            line = line.strip()
            if not line:
                continue
            try:
                entries.append(json.loads(line))
            except json.JSONDecodeError:
                continue
    return entries


def build_entry(summary: dict[str, Any]) -> dict[str, Any]:
    """Build a trend entry from the current summary."""
    sha, ref = git_info()
    counts = summary.get("counts", {})
    return {
        "schema": SCHEMA_VERSION,
        "timestamp": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "git_sha": sha[:12],
        "git_ref": ref,
        "counts": {
            "total": counts.get("total", 0),
            "pass": counts.get("pass", 0),
            "fail": counts.get("fail", 0),
            "na": counts.get("na", 0),
        },
        "pass_rate_pct": summary.get("pass_rate_pct", 0.0),
        "per_tier": summary.get("per_tier", {}),
        "negative": summary.get("negative", {}),
        "evidence": summary.get("evidence", {}),
    }


def append_entry(entry: dict[str, Any]) -> None:
    """Append entry to conformance_trend.jsonl."""
    TREND_PATH.parent.mkdir(parents=True, exist_ok=True)
    with TREND_PATH.open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(entry, separators=(",", ":")) + "\n")


def compute_direction(entries: list[dict[str, Any]], window: int) -> tuple[str, float]:
    """Compute trend direction and delta over last `window` entries.

    Returns (direction, delta_pct) where direction is one of:
    "improving", "degrading", "stable".
    """
    if len(entries) < 2:
        return "stable", 0.0
    recent = entries[-min(window, len(entries)):]
    first_rate = recent[0].get("pass_rate_pct", 0.0)
    last_rate = recent[-1].get("pass_rate_pct", 0.0)
    delta = last_rate - first_rate
    if abs(delta) < 0.1:
        return "stable", delta
    return ("improving" if delta > 0 else "degrading"), delta


def find_regressions(entries: list[dict[str, Any]]) -> list[dict[str, Any]]:
    """Find consecutive entries where pass_rate dropped."""
    regressions: list[dict[str, Any]] = []
    for i in range(1, len(entries)):
        prev_rate = entries[i - 1].get("pass_rate_pct", 0.0)
        curr_rate = entries[i].get("pass_rate_pct", 0.0)
        if curr_rate < prev_rate:
            regressions.append({
                "from_date": entries[i - 1].get("timestamp", "?"),
                "to_date": entries[i].get("timestamp", "?"),
                "from_sha": entries[i - 1].get("git_sha", "?"),
                "to_sha": entries[i].get("git_sha", "?"),
                "from_rate": prev_rate,
                "to_rate": curr_rate,
                "delta": curr_rate - prev_rate,
            })
    return regressions


def sparkline(values: list[float]) -> str:
    """Generate ASCII sparkline for pass rates (0-100 scale)."""
    if not values:
        return ""
    blocks = " _.-=+*#@"
    result = []
    for v in values:
        idx = min(int(v / 100.0 * (len(blocks) - 1)), len(blocks) - 1)
        result.append(blocks[idx])
    return "".join(result)


def generate_report(entries: list[dict[str, Any]]) -> str:
    """Generate TREND_REPORT.md content."""
    lines: list[str] = []
    now = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

    lines.append("# Conformance Trend Report")
    lines.append("")
    lines.append(f"Generated: {now}")
    lines.append("")

    if not entries:
        lines.append("No trend data available yet.")
        return "\n".join(lines)

    latest = entries[-1]

    # Current Status
    lines.append("## Current Status")
    lines.append("")
    counts = latest.get("counts", {})
    lines.append(f"- **Pass rate**: {latest.get('pass_rate_pct', 0.0):.1f}%")
    lines.append(f"- **Pass/Fail/N-A**: {counts.get('pass', 0)}/{counts.get('fail', 0)}/{counts.get('na', 0)}")
    lines.append(f"- **Total extensions**: {counts.get('total', 0)}")
    neg = latest.get("negative", {})
    lines.append(f"- **Negative tests**: {neg.get('pass', 0)} pass, {neg.get('fail', 0)} fail")
    lines.append(f"- **Git**: `{latest.get('git_sha', '?')}` on `{latest.get('git_ref', '?')}`")
    lines.append("")

    # Trend Summary
    lines.append("## Trend Summary")
    lines.append("")
    dir_7, delta_7 = compute_direction(entries, 7)
    dir_30, delta_30 = compute_direction(entries, 30)
    lines.append(f"- **Last 7 runs**: {dir_7} ({delta_7:+.1f}%)")
    lines.append(f"- **Last 30 runs**: {dir_30} ({delta_30:+.1f}%)")
    lines.append(f"- **Total data points**: {len(entries)}")

    rates = [e.get("pass_rate_pct", 0.0) for e in entries[-30:]]
    spark = sparkline(rates)
    if spark:
        lines.append(f"- **Sparkline** (last {len(rates)}): `{spark}`")
    lines.append("")

    # History Table
    lines.append("## History (last 30 runs)")
    lines.append("")
    lines.append("| Date | SHA | Pass | Fail | N/A | Rate |")
    lines.append("|------|-----|------|------|-----|------|")

    display_entries = entries[-30:]
    for entry in reversed(display_entries):
        ts = entry.get("timestamp", "?")
        date_str = ts[:10] if len(ts) >= 10 else ts
        sha = entry.get("git_sha", "?")[:8]
        c = entry.get("counts", {})
        rate = entry.get("pass_rate_pct", 0.0)
        lines.append(
            f"| {date_str} | `{sha}` | {c.get('pass', 0)} | {c.get('fail', 0)} "
            f"| {c.get('na', 0)} | {rate:.1f}% |"
        )
    lines.append("")

    # Regressions
    regressions = find_regressions(entries)
    lines.append("## Regressions")
    lines.append("")
    if not regressions:
        lines.append("No pass-rate regressions detected in the trend history.")
    else:
        lines.append(f"Found {len(regressions)} regression(s):")
        lines.append("")
        for reg in regressions[-10:]:
            lines.append(
                f"- **{reg['from_date'][:10]}** `{reg['from_sha'][:8]}` "
                f"({reg['from_rate']:.1f}%) -> "
                f"**{reg['to_date'][:10]}** `{reg['to_sha'][:8]}` "
                f"({reg['to_rate']:.1f}%) "
                f"[{reg['delta']:+.1f}%]"
            )
    lines.append("")

    return "\n".join(lines)


def main() -> int:
    summary = load_summary()
    entries = load_trend()
    entry = build_entry(summary)
    append_entry(entry)

    # Reload to include the new entry
    entries.append(entry)

    report = generate_report(entries)
    TREND_REPORT_PATH.parent.mkdir(parents=True, exist_ok=True)
    TREND_REPORT_PATH.write_text(report, encoding="utf-8")

    print(f"Trend entry appended: {TREND_PATH}")
    print(f"Trend report written: {TREND_REPORT_PATH}")
    print(f"Total data points: {len(entries)}")
    print(f"Current pass rate: {entry['pass_rate_pct']:.1f}%")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
