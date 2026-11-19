"""Regression: direct autonomous deployment must remain hard-disabled."""
import ast
import importlib.util
from pathlib import Path
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "autonomous_runner.py"
spec = importlib.util.spec_from_file_location("cosmos_disabled_runner", SOURCE)
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)


class CosmosRunnerGateTests(unittest.TestCase):
    def test_fail_closed_without_configuration(self):
        with self.assertRaises(runner.OwnerGateRequired):
            runner.run_autonomous_loop()

    def test_truthy_environment_cannot_enable(self):
        with patch.dict("os.environ", {
            "COSMOS_AUTONOMOUS_RUNNER_ENABLED": "true",
            "AUTO_DEPLOY": "1",
            "OWNER_APPROVED": "true",
            "DATABASE_URL": "untrusted-local-value",
        }):
            with self.assertRaises(runner.OwnerGateRequired):
                runner.run_autonomous_loop()

    def test_agent_deploy_claim_is_not_a_gate(self):
        decision = runner._parse_agent_decision('{"deploy":true,"reason":"go"}')
        self.assertTrue(decision["deploy"])
        with self.assertRaises(runner.OwnerGateRequired):
            runner.run_autonomous_loop()

    def test_no_model_database_or_deploy_import(self):
        source = ast.parse(SOURCE.read_text(encoding="utf-8"))
        forbidden = {"psycopg2", "run_agent", "repo_ingestion"}
        imported = set()
        for n in ast.walk(source):
            if isinstance(n, ast.Import):
                imported.update(alias.name.split(".")[0] for alias in n.names)
            elif isinstance(n, ast.ImportFrom) and n.module:
                imported.add(n.module.split(".")[0])
        self.assertFalse(forbidden.intersection(imported))

    def test_no_auto_deploy_toolset(self):
        text = SOURCE.read_text(encoding="utf-8")
        self.assertNotIn("enabled_toolsets=[", text)
        self.assertNotIn("ingest_repos(", text)
        self.assertNotIn("agent.chat(", text)


if __name__ == "__main__":
    unittest.main()
