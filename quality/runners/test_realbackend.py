"""Tests for quality/runners/_realbackend.py — RBF-FW-01 (P89 89-03).

Covers env-gating (is_skipped / skip_reason), the exit-code -> status map
(incl. exit-75 -> NOT-VERIFIED and the preserved exit-2 -> PARTIAL), and
runner-level integration through run.run_row().

Run: python3 -m unittest quality.runners.test_realbackend -v
"""

from __future__ import annotations

import contextlib
import io
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
        # RBF-FW-07b: skip_reason is now the machine marker; the human recovery
        # text moved to skip_detail so the churn is explained without ambiguity.
        self.assertEqual(artifact["skip_reason"], "env-missing")
        self.assertIn("REPOSIX_ALLOWED_ORIGINS unset", artifact["skip_detail"])


class TestLoopbackSpellings(unittest.TestCase):
    """P89 cross-AI review H1: every loopback spelling must skip, not just 127.0.0.1."""

    ROW = {"id": "x/y", "cadences": ["pre-release-real-backend"]}
    CREDS = {"GITHUB_TOKEN": "x"}

    def _env(self, origins):
        return {"REPOSIX_ALLOWED_ORIGINS": origins, **self.CREDS}

    def test_localhost_skips(self):
        self.assertTrue(_realbackend.is_skipped(self.ROW, self._env("http://localhost:7878")))

    def test_ipv6_loopback_skips(self):
        self.assertTrue(_realbackend.is_skipped(self.ROW, self._env("http://[::1]:7878")))

    def test_unspecified_addr_skips(self):
        self.assertTrue(_realbackend.is_skipped(self.ROW, self._env("http://0.0.0.0:7878")))

    def test_other_127_addr_skips(self):
        self.assertTrue(_realbackend.is_skipped(self.ROW, self._env("http://127.0.0.2:7878")))

    def test_uppercase_localhost_skips(self):
        self.assertTrue(_realbackend.is_skipped(self.ROW, self._env("HTTP://LOCALHOST:7878")))

    def test_multi_origin_all_local_skips(self):
        env = self._env("http://127.0.0.1:7878,http://localhost:9999")
        self.assertTrue(_realbackend.is_skipped(self.ROW, env))

    def test_multi_origin_one_real_runs(self):
        env = self._env("http://127.0.0.1:7878,https://api.github.com")
        self.assertFalse(_realbackend.is_skipped(self.ROW, env))

    def test_real_origin_still_runs(self):
        self.assertFalse(_realbackend.is_skipped(self.ROW, self._env("https://api.github.com")))

    def test_local_skip_reason_names_loopback(self):
        reason = _realbackend.skip_reason({"REPOSIX_ALLOWED_ORIGINS": "http://localhost:7878"})
        self.assertIn("local-only", reason)


class TestVerifierMissing(unittest.TestCase):
    """RBF-FW-07a (cross-AI H4): a missing verifier script demotes to
    NOT-VERIFIED unconditionally, with a distinct `error` marker."""

    def _run(self, status):
        with tempfile.TemporaryDirectory() as td:
            row = {
                "id": "test/missing", "cadences": ["pre-push"], "blast_radius": "P1",
                "status": status, "artifact": "artifact.json",
                "verifier": {"script": "does-not-exist.sh"},
            }
            with mock.patch.dict(os.environ, {}, clear=True):
                updated, _ = run.run_row(row, Path(td), datetime.now(timezone.utc))
            artifact = json.loads((Path(td) / "artifact.json").read_text())
        return updated, artifact

    def test_prior_pass_demotes_to_not_verified_with_error(self):
        updated, artifact = self._run("PASS")
        self.assertEqual(updated["status"], "NOT-VERIFIED")
        self.assertIn("verifier not found", artifact["error"])
        self.assertTrue(updated["_verifier_missing"])

    def test_prior_not_verified_stays(self):
        updated, _ = self._run("NOT-VERIFIED")
        self.assertEqual(updated["status"], "NOT-VERIFIED")

    def test_verifier_missing_flag_stripped_by_main(self):
        # End-to-end: main() must strip the transient flag before persisting.
        persisted = _run_main_over_synthetic_catalog(
            dimension="structure",
            row={
                "id": "test/missing", "cadences": ["pre-push"], "blast_radius": "P1",
                "status": "PASS", "last_verified": "2026-04-01T00:00:00Z",
                "artifact": "quality/reports/verifications/test-missing.json",
                "verifier": {"script": "quality/gates/structure/does-not-exist.sh"},
            },
        )
        self.assertEqual(persisted["status"], "NOT-VERIFIED")
        self.assertNotIn("_verifier_missing", persisted)


class TestSkipFailClosedWithHistory(unittest.TestCase):
    """RBF-FW-07b (AMENDED D90-04): env-gated skip is fail-closed NOT-VERIFIED
    for ALL pre-release-real-backend rows, with the prior real grade preserved
    in write-history fields and an explicit env-missing marker."""

    def _skip(self, prior_status, prior_lv="2026-06-01T00:00:00Z", blast="P1"):
        with tempfile.TemporaryDirectory() as td:
            row = {
                "id": "test/rb", "cadences": ["pre-release-real-backend"],
                "blast_radius": blast, "status": prior_status,
                "last_verified": prior_lv, "artifact": "artifact.json",
                "verifier": {"script": "nonexistent.sh"},
            }
            with mock.patch.dict(os.environ, {}, clear=True):
                updated, _ = run.run_row(row, Path(td), datetime.now(timezone.utc))
            artifact = json.loads((Path(td) / "artifact.json").read_text())
        return updated, artifact

    def test_prior_pass_flips_and_preserves_history(self):
        updated, artifact = self._skip("PASS")
        self.assertEqual(updated["status"], "NOT-VERIFIED")
        self.assertEqual(updated["last_real_grade"], "PASS")
        self.assertEqual(updated["last_real_verified"], "2026-06-01T00:00:00Z")
        self.assertEqual(artifact["skip_reason"], "env-missing")
        self.assertIn("REPOSIX_ALLOWED_ORIGINS", artifact["skip_detail"])

    def test_litmus_p0_prior_pass_flips_and_is_red(self):
        # OD-2: a cred-less milestone-close can never ride a stale real PASS.
        updated, _ = self._skip("PASS", blast="P0")
        self.assertEqual(updated["status"], "NOT-VERIFIED")
        self.assertEqual(run.compute_exit_code([updated]), 1)  # RED

    def test_prior_not_verified_writes_no_history(self):
        updated, _ = self._skip("NOT-VERIFIED", prior_lv=None)
        self.assertEqual(updated["status"], "NOT-VERIFIED")
        self.assertNotIn("last_real_grade", updated)

    def test_second_skip_is_idempotent_on_persistent_fields(self):
        # Run 1: prior PASS -> NOT-VERIFIED + last_real_grade=PASS. Feed the
        # persisted shape into run 2 (prior NOT-VERIFIED) -> history untouched.
        r1, _ = self._skip("PASS")
        r1_persist = {k: v for k, v in r1.items() if not k.startswith("_")}
        with tempfile.TemporaryDirectory() as td:
            r1_persist["artifact"] = "artifact.json"
            r1_persist["verifier"] = {"script": "nonexistent.sh"}
            with mock.patch.dict(os.environ, {}, clear=True):
                r2, _ = run.run_row(r1_persist, Path(td), datetime.now(timezone.utc))
        self.assertEqual(r2["status"], "NOT-VERIFIED")
        self.assertEqual(r2["last_real_grade"], "PASS")  # not overwritten
        self.assertEqual(r2["last_real_verified"], "2026-06-01T00:00:00Z")


class TestShellSubprocessTranscriptRuntime(unittest.TestCase):
    """RBF-FW-08 (M6): the runner refuses a shell-subprocess PASS without real
    transcript evidence (file exists + argv: line)."""

    def _run_ss(self, *, kind="shell-subprocess", transcript_path=None,
                transcript_body=None, exit_code=0):
        with tempfile.TemporaryDirectory() as td:
            tdp = Path(td)
            if transcript_path is not None and transcript_body is not None:
                fp = tdp / transcript_path
                fp.parent.mkdir(parents=True, exist_ok=True)
                fp.write_text(transcript_body)
            artifact_body = {"asserts_passed": [], "asserts_failed": []}
            if transcript_path is not None:
                artifact_body["transcript_path"] = transcript_path
            script = tdp / "v.sh"
            script.write_text(
                "#!/bin/bash\n"
                f"cat > artifact.json <<'JSON'\n{json.dumps(artifact_body)}\nJSON\n"
                f"exit {exit_code}\n"
            )
            script.chmod(0o755)
            row = {
                "id": "test/ss", "cadences": ["on-demand"], "blast_radius": "P2",
                "kind": kind, "artifact": "artifact.json",
                "verifier": {"script": "v.sh", "timeout_s": 10},
            }
            updated, _ = run.run_row(row, tdp, datetime.now(timezone.utc))
            final = json.loads((tdp / "artifact.json").read_text())
        return updated, final

    def test_valid_transcript_passes(self):
        updated, _ = self._run_ss(
            transcript_path="t.txt",
            transcript_body="argv: /usr/bin/reposix --version\nexit_code: 0\n")
        self.assertEqual(updated["status"], "PASS")

    def test_no_transcript_path_fails(self):
        updated, final = self._run_ss(transcript_path=None)
        self.assertEqual(updated["status"], "FAIL")
        self.assertTrue(any("no transcript_path" in a for a in final["asserts_failed"]))

    def test_missing_transcript_file_fails(self):
        updated, final = self._run_ss(transcript_path="gone.txt", transcript_body=None)
        self.assertEqual(updated["status"], "FAIL")
        self.assertTrue(any("missing" in a for a in final["asserts_failed"]))

    def test_transcript_without_argv_fails(self):
        updated, final = self._run_ss(
            transcript_path="t.txt", transcript_body="cwd: /x\nexit_code: 0\n")
        self.assertEqual(updated["status"], "FAIL")
        self.assertTrue(any("argv" in a for a in final["asserts_failed"]))

    def test_non_shell_subprocess_row_not_gated(self):
        updated, _ = self._run_ss(kind="mechanical", transcript_path=None)
        self.assertEqual(updated["status"], "PASS")  # gate does not over-fire


def _run_main_over_synthetic_catalog(*, dimension: str, row: dict) -> dict:
    """Drive run.main() over a one-row synthetic catalog in a temp REPO_ROOT so
    the full grade->strip->persist path is exercised. Returns the persisted row.

    Uses --persist (the D-P96-01 MINT path): this helper's caller asserts a
    persisted status flip, which since the GRADE/PERSIST split only happens on
    an explicit mint. A bare cadence run is validate-only (see test_run.py).
    """
    with tempfile.TemporaryDirectory() as td:
        tdp = Path(td)
        cat_dir = tdp / "quality" / "catalogs"
        cat_dir.mkdir(parents=True)
        catalog = {"dimension": dimension, "rows": [row]}
        cat_path = cat_dir / "synthetic.json"
        cat_path.write_text(json.dumps(catalog))
        with mock.patch.object(run, "REPO_ROOT", tdp), \
             mock.patch.object(run, "CATALOG_DIR", cat_dir), \
             mock.patch.object(run, "REPORTS_DIR", tdp / "quality" / "reports"), \
             mock.patch.object(sys, "argv",
                               ["run.py", "--cadence", "pre-push", "--persist"]), \
             mock.patch.dict(os.environ, {}, clear=True), \
             contextlib.redirect_stdout(io.StringIO()):
            run.main()
        persisted = json.loads(cat_path.read_text())
    return persisted["rows"][0]


if __name__ == "__main__":
    unittest.main()
