#!/usr/bin/env python3
"""Read-only local machine audit for Hermes deployment planning.

The script gathers hardware and filesystem metadata only. It does not delete,
move, upload, install, or change system configuration.
"""
from __future__ import annotations

import argparse
import json
import os
import platform
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat()


def gib(value: int | float) -> float:
    return round(float(value) / (1024 ** 3), 2)


def run_text(command: list[str], timeout: int = 8) -> str | None:
    try:
        completed = subprocess.run(command, check=False, capture_output=True, text=True, timeout=timeout)
    except (OSError, subprocess.TimeoutExpired):
        return None
    text = (completed.stdout or completed.stderr or "").strip()
    return text or None


def total_memory_bytes() -> int | None:
    if sys.platform == "win32":
        output = run_text([
            "powershell", "-NoProfile", "-Command",
            "(Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory",
        ])
        if output:
            try:
                return int(output.splitlines()[-1].strip())
            except ValueError:
                pass
    if hasattr(os, "sysconf"):
        try:
            pages = os.sysconf("SC_PHYS_PAGES")
            page_size = os.sysconf("SC_PAGE_SIZE")
            return int(pages * page_size)
        except (ValueError, OSError, AttributeError):
            pass
    return None


def gpu_inventory() -> list[dict[str, Any]]:
    devices: list[dict[str, Any]] = []
    if sys.platform == "win32":
        command = (
            "Get-CimInstance Win32_VideoController | "
            "Select-Object Name,AdapterRAM,DriverVersion | ConvertTo-Json -Compress"
        )
        raw = run_text(["powershell", "-NoProfile", "-Command", command])
        if raw:
            try:
                value = json.loads(raw)
                rows = value if isinstance(value, list) else [value]
                for row in rows:
                    ram = row.get("AdapterRAM") if isinstance(row, dict) else None
                    devices.append({
                        "name": row.get("Name"),
                        "adapter_ram_gib": gib(ram) if isinstance(ram, int) and ram > 0 else None,
                        "driver": row.get("DriverVersion"),
                        "source": "windows-cim",
                    })
            except json.JSONDecodeError:
                pass
    nvidia = run_text([
        "nvidia-smi", "--query-gpu=name,memory.total,driver_version", "--format=csv,noheader,nounits"
    ])
    if nvidia:
        for line in nvidia.splitlines():
            parts = [part.strip() for part in line.split(",")]
            if len(parts) >= 3:
                try:
                    memory_gib = round(float(parts[1]) / 1024, 2)
                except ValueError:
                    memory_gib = None
                devices.append({
                    "name": parts[0], "adapter_ram_gib": memory_gib,
                    "driver": parts[2], "source": "nvidia-smi",
                })
    return devices


def drive_inventory() -> list[dict[str, Any]]:
    roots: list[Path] = []
    if sys.platform == "win32":
        for letter in "ABCDEFGHIJKLMNOPQRSTUVWXYZ":
            root = Path(f"{letter}:\\")
            if root.exists():
                roots.append(root)
    else:
        roots.append(Path("/"))
        for base in (Path("/mnt"), Path("/media"), Path("/Volumes")):
            if base.exists():
                roots.extend(path for path in base.iterdir() if path.is_dir())
    result = []
    seen: set[str] = set()
    for root in roots:
        key = str(root.resolve()) if root.exists() else str(root)
        if key in seen:
            continue
        seen.add(key)
        try:
            usage = shutil.disk_usage(root)
        except OSError:
            continue
        result.append({
            "root": str(root),
            "total_gib": gib(usage.total),
            "used_gib": gib(usage.used),
            "free_gib": gib(usage.free),
            "free_percent": round(usage.free / usage.total * 100, 1) if usage.total else None,
        })
    return result


def directory_size(path: Path, max_depth: int, depth: int = 0) -> int:
    total = 0
    try:
        entries = list(os.scandir(path))
    except (OSError, PermissionError):
        return 0
    for entry in entries:
        try:
            if entry.is_symlink():
                continue
            if entry.is_file(follow_symlinks=False):
                total += entry.stat(follow_symlinks=False).st_size
            elif entry.is_dir(follow_symlinks=False) and depth < max_depth:
                total += directory_size(Path(entry.path), max_depth, depth + 1)
        except (OSError, PermissionError):
            continue
    return total


def top_directories(root: Path, max_depth: int, top: int) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    try:
        children = [Path(entry.path) for entry in os.scandir(root) if entry.is_dir(follow_symlinks=False)]
    except (OSError, PermissionError):
        return rows
    for child in children:
        size = directory_size(child, max_depth=max_depth)
        rows.append({"path": str(child), "size_gib": gib(size)})
    rows.sort(key=lambda item: item["size_gib"], reverse=True)
    return rows[:top]


def classify_machine(memory_gib: float | None, drives: list[dict[str, Any]], gpus: list[dict[str, Any]]) -> dict[str, Any]:
    free = max((item.get("free_gib") or 0 for item in drives), default=0)
    vram = max((item.get("adapter_ram_gib") or 0 for item in gpus), default=0)
    if vram >= 8 and (memory_gib or 0) >= 16:
        profile = "GPU_WORKER"
    elif (memory_gib or 0) >= 32 and free >= 80:
        profile = "MEDIUM_LOCAL"
    elif (memory_gib or 0) >= 16 and free >= 40:
        profile = "SMALL_LOCAL"
    elif (memory_gib or 0) >= 8 and free >= 20:
        profile = "MICRO_LOCAL"
    else:
        profile = "CONTROL_ONLY"
    trials = {
        "CONTROL_ONLY": ["remote/local-network worker", "small embeddings only after proof"],
        "MICRO_LOCAL": ["0.5B-1.5B Q4 text", "small embeddings", "bounded transcription"],
        "SMALL_LOCAL": ["1.5B-4B Q4 text", "Moondream small trial", "single-job transcription"],
        "MEDIUM_LOCAL": ["3B-8B Q4 text", "lightweight vision", "sustained indexing at concurrency 1"],
        "GPU_WORKER": ["7B-14B Q4 subject to benchmark", "GPU vision/transcription", "batch media indexing"],
    }
    return {
        "profile": profile,
        "recommended_first_trials": trials[profile],
        "rules": [
            "keep at least 20 GiB free on system drive",
            "use no more than 70% of measured RAM",
            "concurrency 1 until benchmarked",
            "do not download a model before install-location approval",
        ],
    }


def build_report(args: argparse.Namespace) -> dict[str, Any]:
    memory = total_memory_bytes()
    drives = drive_inventory()
    gpus = gpu_inventory()
    report: dict[str, Any] = {
        "schema_version": 1,
        "created_at": utc_now(),
        "read_only": True,
        "host": {
            "platform": platform.platform(),
            "system": platform.system(),
            "release": platform.release(),
            "machine": platform.machine(),
            "processor": platform.processor(),
            "logical_cpu_count": os.cpu_count(),
            "python": platform.python_version(),
            "hostname": platform.node(),
        },
        "memory": {
            "total_bytes": memory,
            "total_gib": gib(memory) if memory else None,
        },
        "gpus": gpus,
        "drives": drives,
        "tools": {
            name: shutil.which(name) for name in
            ("git", "node", "npm", "python", "ffmpeg", "ffprobe", "nvidia-smi", "qvac", "21st")
        },
        "machine_fit": classify_machine(gib(memory) if memory else None, drives, gpus),
        "mutations_performed": [],
    }
    if args.scan_root:
        root = args.scan_root.expanduser().resolve()
        report["storage_scan"] = {
            "root": str(root),
            "max_depth": args.max_depth,
            "top_directories": top_directories(root, args.max_depth, args.top),
            "content_opened": False,
            "files_deleted": False,
        }
    return report


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser(description="Read-only local machine audit")
    value.add_argument("--output", type=Path)
    value.add_argument("--scan-root", type=Path)
    value.add_argument("--max-depth", type=int, default=2)
    value.add_argument("--top", type=int, default=25)
    return value


def main() -> int:
    args = parser().parse_args()
    if args.max_depth < 0 or args.max_depth > 6:
        print("--max-depth must be between 0 and 6", file=sys.stderr)
        return 2
    if args.top < 1 or args.top > 200:
        print("--top must be between 1 and 200", file=sys.stderr)
        return 2
    report = build_report(args)
    rendered = json.dumps(report, indent=2)
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        temp = args.output.with_suffix(args.output.suffix + ".tmp")
        temp.write_text(rendered, encoding="utf-8")
        temp.replace(args.output)
    print(rendered)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
