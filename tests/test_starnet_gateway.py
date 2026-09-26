"""STARNET gateway: own credential only, truthful city status, async task receipts."""
import importlib
import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
fastapi = pytest.importorskip("fastapi")
from fastapi import FastAPI  # noqa: E402
from fastapi.testclient import TestClient  # noqa: E402

TOKEN = "starnet-gateway-test-token-123456"


@pytest.fixture()
def gw(tmp_path, monkeypatch):
    monkeypatch.setenv("STARNET_GATEWAY_STATE_DIR", str(tmp_path / "tasks"))
    monkeypatch.setenv("STARNET_ENV_FILE", str(tmp_path / "missing.env"))
    monkeypatch.setenv("STARNET_GATEWAY_TOKEN", TOKEN)
    monkeypatch.setenv("STARNET_API_KEY", "local-starnet-key-1234567890")
    monkeypatch.delenv("TERABITHIA_API_KEY", raising=False)
    monkeypatch.delenv("HERMES_API_KEY", raising=False)
    mod = importlib.reload(importlib.import_module("starnet_gateway"))
    app = FastAPI()
    app.include_router(mod.router, prefix="/starnet")
    return mod, TestClient(app)


def _auth(token=TOKEN):
    return {"Authorization": f"Bearer {token}"}


def test_other_planes_keys_do_not_open_this_gateway(gw, monkeypatch):
    mod, client = gw
    monkeypatch.delenv("STARNET_GATEWAY_TOKEN")
    monkeypatch.setenv("TERABITHIA_API_KEY", "terabithia-key-abcdefghijklmnop")
    monkeypatch.setenv("HERMES_API_KEY", "hermes-key-abcdefghijklmnopqrst")
    for key in ("terabithia-key-abcdefghijklmnop", "hermes-key-abcdefghijklmnopqrst"):
        assert client.get("/starnet/v1/city/status", headers=_auth(key)).status_code == 503


def test_city_status_reports_runtime_health_not_hardcoded_online(gw, monkeypatch):
    mod, client = gw

    async def fake(method, path, **kw):
        if path == "/health":
            return {"status": "degraded", "version": "0.12"}
        return {"data": [{"id": "starnet-agent"}, {"id": "scout"}]}

    monkeypatch.setattr(mod, "_starnet_json", fake)
    body = client.get("/starnet/v1/city/status", headers=_auth()).json()
    assert body["city"]["status"] == "degraded"
    assert body["degraded"] is True
    assert "districts" in body["unreported"]
    assert [c["id"] for c in body["citizens"]] == ["scout"]


def test_task_create_returns_202_and_settles(gw, monkeypatch):
    mod, client = gw

    async def fake(method, path, **kw):
        return {"choices": [{"message": {"content": "done"}}]}

    monkeypatch.setattr(mod, "_starnet_json", fake)
    created = client.post("/starnet/v1/heisenberg/tasks", headers=_auth(), json={"task": "ping"})
    assert created.status_code == 202
    task_id = created.json()["id"]
    for _ in range(50):
        got = client.get(f"/starnet/v1/heisenberg/tasks/{task_id}", headers=_auth()).json()
        if got["status"] != "running":
            break
    assert got["status"] == "completed"
    assert got["receipt"]["completed"] is True


def test_task_failure_is_recorded_as_failed(gw, monkeypatch):
    mod, client = gw

    async def boom(method, path, **kw):
        raise RuntimeError("down")

    monkeypatch.setattr(mod, "_starnet_json", boom)
    task_id = client.post("/starnet/v1/heisenberg/tasks", headers=_auth(), json={"task": "ping"}).json()["id"]
    for _ in range(50):
        got = client.get(f"/starnet/v1/heisenberg/tasks/{task_id}", headers=_auth()).json()
        if got["status"] != "running":
            break
    assert got["status"] == "failed"
    assert got["receipt"]["completed"] is False


def test_running_receipts_are_settled_after_restart(tmp_path, monkeypatch):
    import json

    tasks = tmp_path / "tasks"
    tasks.mkdir()
    (tasks / "heis-abc.json").write_text(json.dumps({"id": "heis-abc", "status": "running", "logs": []}))
    (tasks / "heis-done.json").write_text(json.dumps({"id": "heis-done", "status": "completed", "logs": []}))
    monkeypatch.setenv("STARNET_GATEWAY_STATE_DIR", str(tasks))
    importlib.reload(importlib.import_module("starnet_gateway"))  # simulates a process start
    interrupted = json.loads((tasks / "heis-abc.json").read_text())
    assert interrupted["status"] == "failed"
    assert interrupted["receipt"]["completed"] is False
    assert json.loads((tasks / "heis-done.json").read_text())["status"] == "completed"
