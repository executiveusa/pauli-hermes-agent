from __future__ import annotations

import json
import os
import unittest
from unittest.mock import patch

from tools.montage_studio_tool import montage_studio_tool


class _Response:
    def __init__(self, payload: dict):
        self.payload = payload

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc, tb):
        return False

    def read(self):
        return json.dumps(self.payload).encode("utf-8")


class MontageStudioToolTests(unittest.TestCase):
    def setUp(self):
        os.environ["MONTAGE_API_URL"] = "http://montage.test"
        os.environ["MONTAGE_TENANT"] = "owner"
        os.environ["MONTAGE_API_TOKEN"] = "super-secret-token"

    def tearDown(self):
        for key in ("MONTAGE_API_URL", "MONTAGE_TENANT", "MONTAGE_API_TOKEN"):
            os.environ.pop(key, None)

    @patch("tools.montage_studio_tool.urllib.request.urlopen")
    def test_health_uses_public_health_contract_without_leaking_secret(self, urlopen):
        urlopen.return_value = _Response({"ok": True, "service": "yappy-clipz-studio-api"})
        result = json.loads(montage_studio_tool("health"))
        self.assertTrue(result["ok"])
        request = urlopen.call_args.args[0]
        self.assertEqual(request.full_url, "http://montage.test/healthz")
        self.assertNotIn("super-secret-token", json.dumps(result))

    @patch("tools.montage_studio_tool.urllib.request.urlopen")
    def test_run_forwards_named_action_input_and_approval(self, urlopen):
        urlopen.return_value = _Response({"ok": True, "result": {"projectId": "project_1"}})
        result = json.loads(
            montage_studio_tool(
                "run",
                action_id="project.inspect",
                action_input={"projectId": "project_1"},
                approved=False,
            )
        )
        self.assertTrue(result["ok"])
        request = urlopen.call_args.args[0]
        body = json.loads(request.data.decode("utf-8"))
        self.assertEqual(request.full_url, "http://montage.test/api/v1/actions/project.inspect")
        self.assertEqual(body["input"], {"projectId": "project_1"})
        self.assertFalse(body["approved"])
        self.assertEqual(request.headers["X-yappy-tenant"], "owner")
        self.assertEqual(request.headers["Authorization"], "Bearer super-secret-token")

    def test_run_requires_action_id(self):
        result = json.loads(montage_studio_tool("run"))
        self.assertEqual(result["error"], "action_id_required")


if __name__ == "__main__":
    unittest.main()
