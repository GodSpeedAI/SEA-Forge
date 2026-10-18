"""Harness checks, including a real insecure upstream that MUST be caught as RED."""

import json
import os
from pathlib import Path
import sys
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from mcp_probe import probe as mcp_probe
from postgres_probe import probe as postgres_probe
from postgres_fixture import connection_environment
from verify import verify


class HarnessChecks(unittest.TestCase):
    def test_missing_mcp_endpoint_blocks_both_probes(self):
        with patch.dict(os.environ, {}, clear=True):
            rows = mcp_probe()
        self.assertEqual(len(rows), 2)
        self.assertTrue(all(row["classification"] == "INFRASTRUCTURE-BLOCKED" for row in rows))
        self.assertTrue(all(not row["attempted"] for row in rows))

    def test_unrestricted_real_upstream_is_detected_as_red(self):
        command = json.dumps([sys.executable, str(Path(__file__).parent / "mcp_upstream.py")])
        with patch.dict(os.environ, {"SEA_FORGE_SECURITY_MCP_COMMAND": command}):
            rows = mcp_probe()
        self.assertEqual([row["classification"] for row in rows], ["SECURITY-RED", "SECURITY-RED"])
        self.assertTrue(rows[0]["positive_control"], "real allowed handler must run")
        self.assertTrue(rows[0]["attempted"])
        self.assertNotEqual(rows[0]["observations"]["before_sha256"], rows[0]["observations"]["after_sha256"])
        self.assertTrue(any(row.get("upstream_unchanged") is False for row in rows[1]["observations"]))

    def test_missing_postgres_is_blocked_without_skip(self):
        # Mock dependency discovery only. Resource acceptance itself is never mocked.
        with patch("postgres_fixture.binaries", side_effect=RuntimeError("fixture binaries unavailable")):
            row = postgres_probe()
        self.assertEqual(row["classification"], "INFRASTRUCTURE-BLOCKED")
        self.assertFalse(row["attempted"])

    def test_external_postgres_uri_does_not_use_command_arguments(self):
        parsed = connection_environment("postgresql://synthetic_owner:synthetic_password@127.0.0.1:15432/disposable?sslmode=require")
        self.assertEqual(parsed["PGHOST"],"127.0.0.1")
        self.assertEqual(parsed["PGPORT"],"15432")
        self.assertEqual(parsed["PGDATABASE"],"disposable")
        self.assertEqual(parsed["PGUSER"],"synthetic_owner")
        self.assertEqual(parsed["PGSSLMODE"],"require")
        with self.assertRaises(ValueError):
            connection_environment("postgresql://localhost/disposable?user=owner")
        with self.assertRaises(ValueError):
            connection_environment("postgresql://localhost/disposable?sslmode=require&sslmode=disable")

    def test_external_postgres_does_not_require_server_binaries(self):
        with patch.dict(os.environ, {"SEA_FORGE_SECURITY_PG_ADMIN_DSN":"postgresql://synthetic:synthetic@127.0.0.1/disposable"}):
            with patch("postgres_fixture.binaries", side_effect=RuntimeError("psql unavailable")) as discovery:
                row = postgres_probe()
        discovery.assert_called_once_with(local=False)
        self.assertEqual(row["classification"],"INFRASTRUCTURE-BLOCKED")

    def test_inventory_has_all_contracts_and_gates(self):
        inventory = json.loads((Path(__file__).parent / "inventory.json").read_text())
        self.assertEqual({row["id"].split(".")[0] for row in inventory["cases"]},
                         {f"SEC-{number:02}" for number in range(1, 13)})
        self.assertEqual({row["gate"] for row in inventory["cases"]}, {"A", "B", "C"})
        self.assertEqual(len({row["id"] for row in inventory["cases"]}), len(inventory["cases"]))

    def test_verifier_uses_external_commit_and_detects_tampering(self):
        import hashlib
        previous = Path.cwd()
        with tempfile.TemporaryDirectory(prefix="sf-freeze-selfcheck-") as directory:
            root = Path(directory)
            def git(*args):
                return subprocess.check_output(["git", "-c", "user.name=Synthetic Fixture", "-c",
                    "user.email=fixture@example.invalid", "-c", "commit.gpgsign=false", *args],
                    cwd=root, stderr=subprocess.PIPE).decode().strip()
            git("init", "-q")
            (root / "baseline").write_text("baseline\n")
            git("add", "baseline")
            git("commit", "-qm", "fixture baseline")
            baseline = git("rev-parse", "HEAD")
            protected = root / "protected.txt"
            protected.write_text("frozen\n")
            manifest = root / "tests/security-v1/manifest.json"
            manifest.parent.mkdir(parents=True)
            manifest.write_text(json.dumps({"baseline_commit":baseline, "files":{
                "protected.txt":{"sha256":hashlib.sha256(protected.read_bytes()).hexdigest(),"mode":"100644"}}}))
            git("add", ".")
            git("commit", "-qm", "fixture freeze")
            pin = git("rev-parse", "HEAD")
            try:
                os.chdir(root)
                self.assertEqual(verify(pin, root), [])
                with self.assertRaises(ValueError):
                    verify("HEAD", root)
                protected.write_text("tampered\n")
                self.assertTrue(verify(pin, root))
                manifest.write_text("{}")
                self.assertTrue(verify(pin, root), "working manifest cannot override external pin")
                git("add", ".")
                git("commit", "-qm", "fixture tamper")
                replacement = git("rev-parse", "HEAD")
                git("replace", pin, replacement)
                self.assertTrue(verify(pin, root), "Git replace must not reinterpret external pin")
            finally:
                os.chdir(previous)


if __name__ == "__main__":
    unittest.main()
