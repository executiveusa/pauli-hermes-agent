#!/usr/bin/env python3
"""Nightly conservative sync from upstream Hermes into this fork.

The sync path is intentionally cautious:
- only replay commits after the recorded upstream baseline
- skip commits that conflict or fail validation
- keep `apps/` protected so the control-room build is not overwritten
- open a draft PR for human approval instead of merging directly
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Iterable


@dataclass
class CommitCandidate:
    sha: str
    subject: str
    paths: list[str]


def repo_root() -> Path:
    return Path(__file__).resolve().parents[1]


def run(cmd: list[str], cwd: Path, *, check: bool = True, capture: bool = True) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(
        cmd,
        cwd=cwd,
        check=False,
        text=True,
        capture_output=capture,
    )
    if check and result.returncode != 0:
        stdout = result.stdout.strip() if result.stdout else ""
        stderr = result.stderr.strip() if result.stderr else ""
        message = "\n".join(part for part in [stdout, stderr] if part)
        suffix = f"\n{message}" if message else ""
        raise RuntimeError(f"Command failed ({result.returncode}): {' '.join(cmd)}{suffix}")
    return result


def git(root: Path, *args: str, check: bool = True, capture: bool = True) -> subprocess.CompletedProcess[str]:
    return run(["git", *args], cwd=root, check=check, capture=capture)


def read_json(path: Path) -> dict:
    if not path.exists():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def parse_lines(output: str) -> list[str]:
    return [line.strip() for line in output.splitlines() if line.strip()]


def commit_candidates(root: Path, baseline: str, head: str) -> list[CommitCandidate]:
    rev_list = parse_lines(git(root, "rev-list", "--reverse", f"{baseline}..{head}").stdout or "")
    candidates: list[CommitCandidate] = []
    for sha in rev_list:
        show = git(root, "show", "--no-patch", "--format=%s", sha).stdout or ""
        paths = parse_lines(git(root, "diff-tree", "--no-commit-id", "--name-only", "-r", sha).stdout or "")
        candidates.append(CommitCandidate(sha=sha, subject=show.strip(), paths=paths))
    return candidates


def is_protected(paths: Iterable[str], protected_prefixes: list[str]) -> bool:
    for path in paths:
        normalized = path.replace("\\", "/")
        for prefix in protected_prefixes:
            if normalized == prefix.rstrip("/") or normalized.startswith(prefix):
                return True
    return False


def has_control_room(root: Path) -> bool:
    return (root / "apps" / "control-room" / "package.json").exists()


def validate_branch(root: Path) -> list[str]:
    checks: list[list[str]] = [["python", "-m", "pytest", "tests", "-q"]]
    if has_control_room(root):
        checks.extend(
            [
                ["npm", "run", "control-room:test"],
                ["npm", "run", "control-room:build"],
            ]
        )

    executed: list[str] = []
    for check_cmd in checks:
        executed.append(" ".join(check_cmd))
        run(check_cmd, cwd=root, check=True, capture=False)
    return executed


def create_pr(root: Path, branch: str, title: str, body: str, draft: bool) -> None:
    gh = run(["gh", "--version"], cwd=root, check=True, capture=True)
    if not gh.stdout:
        raise RuntimeError("gh CLI is not available on PATH.")

    args = ["gh", "pr", "create"]
    if draft:
        args.append("--draft")
    args.extend(["--base", "main", "--head", branch, "--title", title, "--body", body])
    run(args, cwd=root, check=True, capture=False)


def main() -> int:
    parser = argparse.ArgumentParser(description="Conservative upstream Hermes sync helper")
    parser.add_argument("--upstream-repo", default="https://github.com/NousResearch/hermes-agent.git")
    parser.add_argument("--upstream-branch", default="main")
    parser.add_argument("--base-branch", default="main")
    parser.add_argument("--state-file", default=".github/upstream-sync-state.json")
    parser.add_argument("--branch-prefix", default="bot/upstream-sync")
    parser.add_argument("--dry-run", action="store_true", help="Print the sync plan without changing files")
    parser.add_argument("--draft-pr", action="store_true", help="Create the PR as a draft")
    args = parser.parse_args()

    root = repo_root()
    state_path = root / args.state_file
    state = read_json(state_path)
    protected_prefixes = state.get("protectedPrefixes") or ["apps/"]

    if not state:
        # Safety valve: seed the baseline to the current upstream head if the file is missing.
        state = {
            "upstreamRepo": args.upstream_repo,
            "upstreamBranch": args.upstream_branch,
            "lastSyncedCommit": "",
            "lastSyncedAt": "",
            "protectedPrefixes": protected_prefixes,
        }

    if not (root / ".git").exists():
        raise RuntimeError(f"{root} is not a git checkout.")

    remote_names = parse_lines(git(root, "remote").stdout or "")
    if "upstream" not in remote_names:
        git(root, "remote", "add", "upstream", args.upstream_repo)
    else:
        git(root, "remote", "set-url", "upstream", args.upstream_repo)

    git(root, "fetch", "upstream", args.upstream_branch)
    head = parse_lines(git(root, "rev-parse", f"upstream/{args.upstream_branch}").stdout or "")[0]
    baseline = state.get("lastSyncedCommit") or ""

    if not baseline:
        state["lastSyncedCommit"] = head
        state["lastSyncedAt"] = datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")
        if not args.dry_run:
            write_json(state_path, state)
        print(f"Initialized upstream sync baseline at {head}. No PR created.")
        return 0

    if baseline == head:
        print(f"No new upstream commits since {baseline}.")
        return 0

    candidates = commit_candidates(root, baseline, head)
    if not candidates:
        print("No upstream commits found in the recorded range.")
        return 0

    branch_name = f"{args.branch_prefix}-{datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S')}-{head[:8]}"
    print(f"Creating sync branch {branch_name} from {args.base_branch}.")

    if args.dry_run:
        print(json.dumps(
            {
                "baseline": baseline,
                "head": head,
                "candidates": [candidate.__dict__ for candidate in candidates],
            },
            indent=2,
        ))
        return 0

    git(root, "checkout", args.base_branch)
    git(root, "checkout", "-b", branch_name)
    git(root, "config", "user.name", "github-actions[bot]")
    git(root, "config", "user.email", "github-actions[bot]@users.noreply.github.com")

    applied: list[CommitCandidate] = []
    skipped: list[dict[str, str]] = []
    validations: list[str] = []

    for candidate in candidates:
        if is_protected(candidate.paths, protected_prefixes):
            skipped.append(
                {
                    "sha": candidate.sha,
                    "subject": candidate.subject,
                    "reason": f"protected path match: {', '.join(candidate.paths)}",
                }
            )
            continue

        print(f"Cherry-picking {candidate.sha[:8]} {candidate.subject}")
        result = run(["git", "cherry-pick", "-x", candidate.sha], cwd=root, check=False, capture=True)
        if result.returncode != 0:
            stderr = (result.stderr or "") + (result.stdout or "")
            if "previous cherry-pick is now empty" in stderr.lower() or "nothing to commit" in stderr.lower():
                run(["git", "cherry-pick", "--abort"], cwd=root, check=False, capture=True)
                skipped.append(
                    {
                        "sha": candidate.sha,
                        "subject": candidate.subject,
                        "reason": "already applied",
                    }
                )
                continue

            run(["git", "cherry-pick", "--abort"], cwd=root, check=False, capture=True)
            skipped.append(
                {
                    "sha": candidate.sha,
                    "subject": candidate.subject,
                    "reason": "merge conflict",
                }
            )
            continue

        try:
            validations = validate_branch(root)
        except Exception as exc:
            print(f"Validation failed for {candidate.sha[:8]}: {exc}")
            run(["git", "reset", "--hard", "HEAD~1"], cwd=root, check=True, capture=True)
            skipped.append(
                {
                    "sha": candidate.sha,
                    "subject": candidate.subject,
                    "reason": f"validation failed: {exc}",
                }
            )
            continue

        applied.append(candidate)

    if not applied:
        print("No safe upstream commits were applied; nothing to push.")
        return 0

    state.update(
        {
            "upstreamRepo": args.upstream_repo,
            "upstreamBranch": args.upstream_branch,
            "lastSyncedCommit": head,
            "lastSyncedAt": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
            "protectedPrefixes": protected_prefixes,
        }
    )
    write_json(state_path, state)
    git(root, "add", state_path.as_posix())
    git(root, "commit", "-m", f"chore(sync): advance upstream Hermes baseline to {head[:8]}")

    body_lines = [
        "## What this sync does",
        "",
        "- Replays upstream Hermes commits one at a time.",
        "- Skips any commit that conflicts, fails validation, or touches protected paths.",
        "- Keeps `apps/` protected so the control-room build is not overwritten.",
        "- Opens a draft PR for human review and approval.",
        "",
        "## Baseline",
        f"- Previous synced upstream commit: `{baseline}`",
        f"- Current upstream commit: `{head}`",
        "",
        "## Applied commits",
    ]
    for candidate in applied:
        body_lines.append(f"- `{candidate.sha[:8]}` {candidate.subject}")
    body_lines.extend(
        [
            "",
            "## Skipped commits",
        ]
    )
    if skipped:
        for item in skipped:
            body_lines.append(f"- `{item['sha'][:8]}` {item['subject']} ({item['reason']})")
    else:
        body_lines.append("- None")

    body_lines.extend(
        [
            "",
            "## Validation",
        ]
    )
    if validations:
        body_lines.extend(f"- `{command}`" for command in validations)
    else:
        body_lines.append("- Validation commands were not run.")

    body_lines.extend(
        [
            "",
            "## Human approval",
            "This PR is draft-only. Review the diff and merge it only if the control-room app and upstream agent surface both still behave correctly.",
        ]
    )

    git(root, "push", "-u", "origin", branch_name, capture=False)
    create_pr(
        root,
        branch=branch_name,
        title=f"chore(sync): upstream Hermes sync {datetime.now(timezone.utc).strftime('%Y-%m-%d')}",
        body="\n".join(body_lines),
        draft=True,
    )

    print(f"Opened draft PR for {len(applied)} applied commit(s).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
