from __future__ import annotations

import importlib.util
from pathlib import Path

MODULE_PATH = Path(__file__).resolve().parents[1] / "scripts" / "audit_machine.py"
spec = importlib.util.spec_from_file_location("audit_machine", MODULE_PATH)
module = importlib.util.module_from_spec(spec)
assert spec and spec.loader
spec.loader.exec_module(module)


def test_control_only_profile() -> None:
    result = module.classify_machine(4.0, [{"free_gib": 10.0}], [])
    assert result["profile"] == "CONTROL_ONLY"


def test_small_local_profile() -> None:
    result = module.classify_machine(16.0, [{"free_gib": 80.0}], [])
    assert result["profile"] == "SMALL_LOCAL"


def test_gpu_worker_profile() -> None:
    result = module.classify_machine(
        32.0,
        [{"free_gib": 100.0}],
        [{"adapter_ram_gib": 12.0}],
    )
    assert result["profile"] == "GPU_WORKER"


def test_report_declares_read_only(monkeypatch) -> None:
    monkeypatch.setattr(module, "total_memory_bytes", lambda: 16 * 1024**3)
    monkeypatch.setattr(module, "drive_inventory", lambda: [{"free_gib": 50.0}])
    monkeypatch.setattr(module, "gpu_inventory", lambda: [])

    class Args:
        scan_root = None
        max_depth = 2
        top = 25

    report = module.build_report(Args())
    assert report["read_only"] is True
    assert report["mutations_performed"] == []
