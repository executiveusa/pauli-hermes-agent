#!/usr/bin/env python3
"""Generate a traceable QR asset package with Segno.

This script never verifies that the destination is live and never claims that a
QR has been scanned by a real device. It creates deterministic assets and a
receipt that downstream verification stages can inspect.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path
from urllib.parse import urlparse

SEGNO_VERSION = "1.6.6"
SAFE_SLUG = re.compile(r"[^a-z0-9]+")


def _load_segno():
    try:
        import segno  # type: ignore
    except ImportError as exc:
        raise SystemExit(
            "Segno is not installed. Run: "
            "python -m pip install -r skills/studio/campaign-factory/requirements.txt"
        ) from exc
    return segno


def validate_https_url(value: str) -> str:
    parsed = urlparse(value)
    if parsed.scheme != "https" or not parsed.netloc:
        raise argparse.ArgumentTypeError(
            "payload must be an absolute HTTPS URL for production campaign QR codes"
        )
    if parsed.username or parsed.password:
        raise argparse.ArgumentTypeError("payload URL must not contain credentials")
    return value


def slugify(value: str) -> str:
    slug = SAFE_SLUG.sub("-", value.strip().lower()).strip("-")
    if not slug:
        raise ValueError("campaign name must contain letters or numbers")
    return slug


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def generate_package(
    *,
    payload: str,
    campaign: str,
    output_dir: Path,
    error: str = "h",
    border: int = 4,
    scale: int = 12,
) -> dict:
    segno = _load_segno()
    installed_version = getattr(segno, "__version__", "unknown")
    if installed_version != SEGNO_VERSION:
        raise RuntimeError(
            f"Expected segno=={SEGNO_VERSION}, found {installed_version}. "
            "Install the pinned skill requirements before generating production assets."
        )

    slug = slugify(campaign)
    master_dir = output_dir / "master"
    tests_dir = output_dir / "tests"
    master_dir.mkdir(parents=True, exist_ok=True)
    tests_dir.mkdir(parents=True, exist_ok=True)

    qr = segno.make_qr(payload, error=error.upper(), boost_error=False)
    svg_path = master_dir / f"{slug}-qr.svg"
    png_path = master_dir / f"{slug}-qr.png"
    pdf_path = master_dir / f"{slug}-qr.pdf"

    # SVG is the protected master. PNG/PDF are deterministic derivatives.
    qr.save(svg_path, scale=1, border=border, dark="#000000", light="#ffffff")
    qr.save(png_path, scale=scale, border=border, dark="#000000", light="#ffffff")
    qr.save(pdf_path, scale=scale, border=border, dark="#000000", light="#ffffff")

    files = {}
    for path in (svg_path, png_path, pdf_path):
        files[path.name] = {
            "path": str(path.relative_to(output_dir)),
            "bytes": path.stat().st_size,
            "sha256": sha256(path),
        }

    receipt = {
        "schema_version": "1.0",
        "status": "GENERATED_NOT_SCAN_VERIFIED",
        "campaign": campaign,
        "campaign_slug": slug,
        "payload": payload,
        "generator": {
            "name": "Segno",
            "version": installed_version,
            "upstream": "https://github.com/heuer/segno",
            "license": "BSD-3-Clause",
        },
        "qr": {
            "designator": qr.designator,
            "is_micro": bool(qr.is_micro),
            "error_correction": error.upper(),
            "border_modules": border,
            "png_scale": scale,
        },
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "files": files,
        "verification": {
            "payload_validated_as_https": True,
            "destination_reachability": "NOT_TESTED",
            "software_decode": "NOT_TESTED",
            "physical_device_scan": "NOT_TESTED",
            "final_composed_asset_scan": "NOT_TESTED",
        },
    }

    receipt_path = master_dir / "payload.json"
    receipt_path.write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")

    checksum_lines = [
        f"{metadata['sha256']}  {metadata['path']}" for metadata in files.values()
    ]
    (tests_dir / "checksums.txt").write_text(
        "\n".join(checksum_lines) + "\n", encoding="utf-8"
    )
    return receipt


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Generate an SVG-master QR campaign package with Segno."
    )
    parser.add_argument("--url", required=True, type=validate_https_url)
    parser.add_argument("--campaign", required=True)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--error", choices=("l", "m", "q", "h"), default="h")
    parser.add_argument("--border", type=int, default=4)
    parser.add_argument("--scale", type=int, default=12)
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    if args.border < 4:
        raise SystemExit("border must be at least 4 modules")
    if args.scale < 1:
        raise SystemExit("scale must be at least 1")

    receipt = generate_package(
        payload=args.url,
        campaign=args.campaign,
        output_dir=args.output.resolve(),
        error=args.error,
        border=args.border,
        scale=args.scale,
    )
    json.dump(receipt, sys.stdout, indent=2)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
