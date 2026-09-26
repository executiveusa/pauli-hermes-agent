#!/usr/bin/env python3
"""Small, dependency-free client for a local CapCutAPI-compatible HTTP service.

This client intentionally does not freeze upstream request schemas. Production
payloads live in JSON files and should be generated only after inspecting the
installed backend's current API documentation.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen

DEFAULT_BASE_URL = "http://127.0.0.1:9001"
KNOWN_ENDPOINTS = {
    "/create_draft",
    "/save_draft",
    "/add_video",
    "/add_audio",
    "/add_image",
    "/add_text",
    "/add_subtitle",
    "/add_effect",
    "/add_sticker",
}


def base_url() -> str:
    return os.environ.get("CAPCUT_API_BASE_URL", DEFAULT_BASE_URL).rstrip("/")


def request_json(method: str, path: str, payload: dict | None = None, timeout: int = 10):
    if not path.startswith("/"):
        path = "/" + path
    body = None if payload is None else json.dumps(payload).encode("utf-8")
    req = Request(
        base_url() + path,
        data=body,
        method=method,
        headers={"Content-Type": "application/json", "Accept": "application/json"},
    )
    try:
        with urlopen(req, timeout=timeout) as response:
            raw = response.read().decode("utf-8", errors="replace")
            try:
                data = json.loads(raw) if raw else None
            except json.JSONDecodeError:
                data = raw
            return {"ok": True, "status": response.status, "data": data}
    except HTTPError as exc:
        raw = exc.read().decode("utf-8", errors="replace")
        return {"ok": False, "status": exc.code, "error": raw or str(exc)}
    except URLError as exc:
        return {"ok": False, "status": None, "error": str(exc.reason)}


def check_connection(timeout: int = 5) -> dict:
    """Check whether the configured HTTP service is reachable without editing a project."""
    # FastAPI/Swagger-capable implementations commonly expose /docs. If this
    # backend does not, fall back to the root path. Neither call mutates drafts.
    for path in ("/docs", "/"):
        try:
            req = Request(base_url() + path, method="GET")
            with urlopen(req, timeout=timeout) as response:
                return {
                    "ok": True,
                    "status": response.status,
                    "base_url": base_url(),
                    "reachable_path": path,
                }
        except (HTTPError, URLError):
            continue
    return {
        "ok": False,
        "status": None,
        "base_url": base_url(),
        "error": "CapCut API service was not reachable at /docs or /",
    }


def load_payload(path: str) -> dict:
    data = json.loads(Path(path).read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise ValueError("Payload JSON must contain an object at the top level")
    return data


def main() -> int:
    parser = argparse.ArgumentParser(description="Hermes CapCutAPI HTTP client")
    sub = parser.add_subparsers(dest="command", required=True)

    check = sub.add_parser("check", help="Non-mutating reachability check")
    check.add_argument("--timeout", type=int, default=5)

    call = sub.add_parser("call", help="POST a JSON payload to an editing endpoint")
    call.add_argument("endpoint", help="e.g. /create_draft or /add_video")
    call.add_argument("payload", help="Path to JSON payload")
    mode = call.add_mutually_exclusive_group(required=True)
    mode.add_argument("--dry-run", action="store_true", help="Print request; do not send it")
    mode.add_argument("--execute", action="store_true", help="Send the request")
    call.add_argument("--timeout", type=int, default=30)
    call.add_argument(
        "--allow-unknown-endpoint",
        action="store_true",
        help="Permit a POST endpoint not in the reviewed CapCutAPI endpoint set",
    )

    args = parser.parse_args()

    if args.command == "check":
        result = check_connection(args.timeout)
        print(json.dumps(result, indent=2, ensure_ascii=False))
        return 0 if result["ok"] else 2

    endpoint = args.endpoint if args.endpoint.startswith("/") else "/" + args.endpoint
    if endpoint not in KNOWN_ENDPOINTS and not args.allow_unknown_endpoint:
        print(
            json.dumps(
                {
                    "ok": False,
                    "error": "Endpoint is outside the reviewed set. Inspect backend docs and pass --allow-unknown-endpoint deliberately.",
                    "endpoint": endpoint,
                },
                indent=2,
            ),
            file=sys.stderr,
        )
        return 2

    try:
        payload = load_payload(args.payload)
    except (OSError, json.JSONDecodeError, ValueError) as exc:
        print(json.dumps({"ok": False, "error": str(exc)}, indent=2), file=sys.stderr)
        return 2

    if args.dry_run:
        print(
            json.dumps(
                {"dry_run": True, "method": "POST", "url": base_url() + endpoint, "payload": payload},
                indent=2,
                ensure_ascii=False,
            )
        )
        return 0

    result = request_json("POST", endpoint, payload, timeout=args.timeout)
    print(json.dumps(result, indent=2, ensure_ascii=False))
    return 0 if result["ok"] else 2


if __name__ == "__main__":
    raise SystemExit(main())
