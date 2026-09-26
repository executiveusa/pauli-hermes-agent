#!/usr/bin/env python3
"""Idempotently register the Faceless YouTube Channel OS in Hermes cron."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from cron.jobs import create_job, list_jobs


JOB_NAME = "faceless-youtube-channel-os"
PROMPT = """Run one autonomous heartbeat of the faceless-youtube-channel-operator.
Read its ICM operating contract, authority policy, channel-os state machine, and current benchmark evidence first.
Continue the highest-priority non-blocked stage for the active channel. Research, package, script, QA, production preparation, derivatives, measurement, and learning may proceed automatically within configured budgets. Reuse existing verified artifacts and idempotency receipts instead of regenerating expensive work.
Do NOT publish, schedule a public upload, spend outside a preapproved budget, change account settings, or bypass login/MFA/CAPTCHA. If one of those gates is reached, write a compact owner approval receipt and stop at that gate. If there is no active approved channel thesis yet, perform evidence-backed discovery/validation and stop at the initial thesis approval gate.
Use the executable runtime under tools/faceless_youtube_runtime for state/receipts and provider doctor checks. Sollo is an execution provider, never the strategy authority. End with DECISION, WORKFLOW STATE, CHANGES, PROOF, COST, BLOCKER, NEXT."""


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", default=str(Path.cwd()), help="Absolute Hermes checkout path on the server")
    parser.add_argument("--schedule", default="0 */6 * * *")
    args = parser.parse_args()
    repo = Path(args.repo).expanduser().resolve()
    if not repo.is_dir():
        raise SystemExit(f"repo does not exist: {repo}")
    existing = [job for job in list_jobs() if job.get("name") == JOB_NAME]
    if existing:
        print(json.dumps({"status": "exists", "job": existing[0]}, indent=2))
        return 0
    job = create_job(
        prompt=PROMPT,
        schedule=args.schedule,
        name=JOB_NAME,
        deliver="local",
        skills=["faceless-youtube-channel-operator"],
        workdir=str(repo),
    )
    print(json.dumps({"status": "created", "job": job}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
