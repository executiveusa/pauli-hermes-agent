from __future__ import annotations

import argparse
import json
from pathlib import Path

from .runtime import FacelessYouTubeRuntime


def _load_json(path: str) -> dict:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog="hermes-youtube", description="Hermes Faceless YouTube runtime")
    p.add_argument("--repo-root", default=".")
    sub = p.add_subparsers(dest="command", required=True)
    sub.add_parser("doctor")
    status = sub.add_parser("status")
    status.add_argument("--limit", type=int, default=20)
    enqueue = sub.add_parser("enqueue")
    enqueue.add_argument("--input", required=True)
    enqueue.add_argument("--force", action="store_true")
    job = sub.add_parser("job")
    job.add_argument("job_id")
    transition = sub.add_parser("transition")
    transition.add_argument("job_id")
    transition.add_argument("--stage")
    transition.add_argument("--status")
    transition.add_argument("--error")
    sollo = sub.add_parser("sollo-script")
    sollo.add_argument("--input", required=True)
    sollo.add_argument("--job-id")
    return p


def main() -> int:
    args = build_parser().parse_args()
    runtime = FacelessYouTubeRuntime(repo_root=Path(args.repo_root))
    if args.command == "doctor":
        result = runtime.doctor()
    elif args.command == "status":
        result = {"jobs": runtime.list_jobs(args.limit)}
    elif args.command == "enqueue":
        result = runtime.create_job(_load_json(args.input), force=args.force)
    elif args.command == "job":
        result = runtime.get_job(args.job_id) or {"error": "not_found", "job_id": args.job_id}
    elif args.command == "transition":
        result = runtime.update_job(args.job_id, stage=args.stage, status=args.status, error=args.error)
    elif args.command == "sollo-script":
        brief = _load_json(args.input)
        job = runtime.get_job(args.job_id) if args.job_id else runtime.create_job(brief)
        runtime.update_job(job["id"], stage="script", status="running")
        try:
            output = runtime.sollo_script(brief)
            receipt = runtime.write_receipt(job["id"], "sollo-script", output)
            runtime.update_job(job["id"], stage="script", status="complete")
            result = {"job_id": job["id"], "status": "complete", "receipt": str(receipt), "output": output}
        except Exception as exc:
            runtime.update_job(job["id"], stage="script", status="blocked", error=str(exc))
            result = {"job_id": job["id"], "status": "blocked", "error": str(exc)}
    else:
        raise AssertionError(args.command)
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0 if not result.get("error") else 1


if __name__ == "__main__":
    raise SystemExit(main())
