"""Tests for quality/runners/_realbackend.py — RBF-FW-01 (P89 89-03).

Covers env-gating (is_skipped / skip_reason), the exit-code -> status map
(incl. exit-75 -> NOT-VERIFIED and the preserved exit-2 -> PARTIAL), and
runner-level integration through run.run_row().

Run: python3 -m unittest quality.runners.test_realbackend -v
"""

from __future__ import annotations

import json
import os
import sys
import tempfile
import unittest
from datetime import datetime, timezone
from pathlib import Path
from unittest import mock

# Make the runner modules importable top-level (mirrors test_freshness.py;
# run.py uses script-style absolute imports, so package-context import of
# `quality.runners.run` breaks on `from _freshness import ...`).
sys.path.insert(0, str(Path(__file__).resolve().parent))

import _realbackend  # noqa: E402
import run  # noqa: E402

TAGGED_ROW = {"id": "x/y", "cadences": ["pre-release-real-backend"]}


class TestIsSkipped(unittest.TestCase):
    def test_row_not_tagged_returns_false_regardless_of_env(self):
        row = {"id": "x/y", "cadences": ["pre-push"]}
        self.assertFalse(_realbackend.is_skipped(row, {}))
        self.assertFalse(_realbackend.is_skipped(
            row, {"REPOSIX_ALLOWED_ORIGINS": "https://api.github.com", "GITHUB_TOKEN": "x"}
        ))

    def test_empty_env_skips(self):
        self.assertTrue(_realbackend.is_skipped(TAGGED_ROW, {}))

    def test_only_origin_set_skips(self):
        env = {"REPOSIX_ALLOWED_ORIGINS": "https://api.github.com"}
        self.assertTrue(_realbackend.is_skipped(TAGGED_ROW, env))

    def test_local_origin_skips(self):
        env = {"REPOSIX_ALLOWED_ORIGINS": "http://127.0.0.1:7878", "GITHUB_TOKEN": "x"}
        self.assertTrue(_realbackend.is_skipped(TAGGED_ROW, env))

    def test_origin_plus_confluence_creds_runs(self):
        env = {
            "REPOSIX_ALLOWED_ORIGINS": "https://reuben-john.atlassian.net",
            "ATLASSIAN_API_KEY": "k", "ATLASSIAN_EMAIL": "e", "REPOSIX_CONFLUENCE_TENANT": "t",
        }
        self.assertFalse(_realbackend.is_skipped(TAGGED_ROW, env))

    def test_origin_plus_github_creds_runs(self):
        env = {"REPOSIX_ALLOWED_ORIGINS": "https://api.github.com", "GITHUB_TOKEN": "x"}
        self.assertFalse(_realbackend.is_skipped(TAGGED_ROW, env))

    def test_origin_plus_jira_creds_runs(self):
        env = {
            "REPOSIX_ALLOWED_ORIGINS": "https://acme.atlassian.net",
            "JIRA_EMAIL": "e", "JIRA_API_TOKEN": "t", "REPOSIX_JIRA_INSTANCE": "acme",
        }
        self.assertFalse(_realbackend.is_skipped(TAGGED_ROW, env))


class TestSkipReason(unittest.TestCase):
    def test_unset_origin(self):
        self.assertIn("REPOSIX_ALLOWED_ORIGINS unset", _realbackend.skip_reason({}))

    def test_local_origin(self):
        reason = _realbackend.skip_reason({"REPOSIX_ALLOWED_ORIGINS": "http://127.0.0.1:7878"})
        self.assertIn("127.0.0.1", reason)

    def test_no_creds_names_every_cred_set(self):
        reason = _realbackend.skip_reason({"REPOSIX_ALLOWED_ORIGINS": "https://api.github.com"})
        self.assertIn("no credential set complete", reason)
        for var in ("ATLASSIAN_API_KEY", "GITHUB_TOKEN", "JIRA_API_TOKEN"):
            self.assertIn(var, reason)  # error must teach recovery (Principle B)


class TestMapExitCodeToStatus(unittest.TestCase):
    """exit-code -> status map: 0/2/75/other."""

    def test_exit_zero_is_pass(self):
        self.assertEqual(_realbackend.map_exit_code_to_status(0), "PASS")

    def test_exit_seventyfive_is_not_verified(self):
        self.assertEqual(_realbackend.map_exit_code_to_status(75), "NOT-VERIFIED")

    def test_exit_two_is_partial(self):
        # Preserved pre-existing runner convention (run.py exit-code branch);
        # the plan's wholesale 0/else replacement would have regressed this.
        self.assertEqual(_realbackend.map_exit_code_to_status(2), "PARTIAL")

    def test_exit_other_is_fail(self):
        self.assertEqual(_realbackend.map_exit_code_to_status(1), "FAIL")
        self.assertEqual(_realbackend.map_exit_code_to_status(127), "FAIL")


class TestRunnerExitCodeIntegration(unittest.TestCase):
    """run_row() end-to-end: synthetic shell verifiers exercise the exit map."""

    def _run_synthetic(self, body: str) -> dict:
        with tempfile.TemporaryDirectory() as td:
            script = Path(td) / "synthetic.sh"
            script.write_text(f"#!/bin/bash\n{body}\n", encoding="utf-8")
            script.chmod(0o755)
            row = {
                "id": "test/synthetic", "cadences": ["on-demand"], "blast_radius": "P2",
                "verifier": {"script": "synthetic.sh", "timeout_s": 10},
            }
            updated, _ = run.run_row(row, Path(td), datetime.now(timezone.utc))
        return updated

    def test_synthetic_exit75_verifier_yields_not_verified(self):
        updated = self._run_synthetic('echo "synthetic NOT-VERIFIED"\nexit 75')
        self.assertEqual(updated["status"], "NOT-VERIFIED")

    def test_synthetic_exit2_verifier_yields_partial(self):
        updated = self._run_synthetic("exit 2")
        self.assertEqual(updated["status"], "PARTIAL")


class TestRunRowEnvGateIntegration(unittest.TestCase):
    """run_row() short-circuits BEFORE verifier lookup when env is unset."""

    def test_run_row_short_circuits_to_not_verified_when_env_unset(self):
        row = {
            "id": "test/skip", "cadences": ["pre-release-real-backend"],
            "blast_radius": "P0", "artifact": "artifact.json",
            # Deliberately nonexistent: the env-gate must fire before the
            # verifier-not-found branch ever looks for this script.
            "verifier": {"script": "nonexistent.sh"},
        }
        with tempfile.TemporaryDirectory() as td:
            with mock.patch.dict(os.environ, {}, clear=True):
                updated, _ = run.run_row(row, Path(td), datetime.now(timezone.utc))
            artifact = json.loads((Path(td) / "artifact.json").read_text(encoding="utf-8"))
        self.assertEqual(updated["status"], "NOT-VERIFIED")
        self.assertTrue(updated["_skipped_real_backend"])
        self.assertTrue(artifact["skipped_real_backend"])
        self.assertIn("REPOSIX_ALLOWED_ORIGINS unset", artifact["skip_reason"])


if __name__ == "__main__":
    unittest.main()
