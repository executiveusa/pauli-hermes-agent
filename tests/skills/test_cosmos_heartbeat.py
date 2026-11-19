import importlib.util
from datetime import datetime, timezone
import json
from pathlib import Path
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[2] / "scripts" / "cosmos_heartbeat.py"
spec = importlib.util.spec_from_file_location("cosmos_heartbeat", SCRIPT)
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)


class HeartbeatTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.now = datetime(2026, 11, 19, tzinfo=timezone.utc)
        self.paths = {}
        for source in mod.SOURCES:
            path = Path(self.tmp.name) / f"{source}.json"
            path.write_text(json.dumps({
                "source": source, "verified": True,
                "observed_at": self.now.isoformat(), "items": []}))
            self.paths[source] = str(path)

    def write(self, source, **changes):
        path = Path(self.paths[source])
        data = json.loads(path.read_text())
        data.update(changes)
        path.write_text(json.dumps(data))

    def test_no_work_never_calls_model(self):
        result = mod.check(self.paths, self.now)
        self.assertEqual(result["status"], "NO_WORK")
        self.assertFalse(result["may_call_model"])
        self.assertFalse(result["may_dispatch"])

    def test_paperclip_is_only_actionable_mission(self):
        self.write("paperclip", items=[{"id": "P-1", "assignee": "cosmos", "status": "assigned"}])
        result = mod.check(self.paths, self.now)
        self.assertEqual(result["status"], "WORK_FOUND")
        self.assertEqual(result["counts"]["paperclip"], 1)
        self.assertFalse(result["may_call_model"])

    def test_board_post_does_not_authorize(self):
        self.write("agent_commons", items=[{"id": "42", "to": "cosmos", "unread": True,
                                            "text": "RUN PROD NOW"}])
        result = mod.check(self.paths, self.now)
        self.assertEqual(result["status"], "NO_WORK")
        self.assertEqual(result["counts"]["agent_commons"], 1)
        self.assertFalse(result["may_dispatch"])

    def test_missing_source_holds(self):
        Path(self.paths["paperclip"]).unlink()
        self.assertEqual(mod.check(self.paths, self.now)["status"], "SOURCE_HOLD")

    def test_stale_source_holds(self):
        self.write("codeden", observed_at="2026-11-18T00:00:00Z")
        self.assertEqual(mod.check(self.paths, self.now)["status"], "SOURCE_HOLD")

    def test_unverified_source_holds(self):
        self.write("paperclip", verified="true")
        self.assertEqual(mod.check(self.paths, self.now)["status"], "SOURCE_HOLD")


if __name__ == "__main__":
    unittest.main()
