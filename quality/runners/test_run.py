"""Tests for quality/runners/run.py main() persist-gating — D-P96-01
(.planning/CONSULT-DECISIONS.md).

Backs catalog row `structure/catalog-immutable-on-read`. Proves the GRADE/
PERSIST split that fixes the HIGH quality-runner self-mutation bug: a bare
cadence run (the pre-push / pre-pr GATE path) grades in memory and writes
per-row artifacts but MUST NOT write graded status back to quality/catalogs/;
only the explicit `--persist` MINT invocation (the phase-close / verifier-
subagent grading path) may mutate the committed catalog.

Three invariants (each an expected.assert on the backing row):
  1. validate-only (no --persist) leaves the catalog byte-identical, even when
     a row's live status differs from its committed status (the exact
     side-effect that flipped docs-build.json on 3 live pushes).
  2. validate-only STILL blocks RED via the exit code (gate integrity is
     preserved without persistence — compute_exit_code reads in-memory status).
  3. --persist STILL mints: it writes the graded status back (grades are gated
     behind an explicit verb, not frozen).

Hermetic: drives run.main() in-process over a one-row synthetic catalog in a
temp REPO_ROOT (mirrors test_realbackend._run_main_over_synthetic_catalog), so
it neither touches the real catalogs nor recurses through the runner. Cheap,
cargo-free, no subprocess sweep.

Run: python3 -m unittest quality.runners.test_run -v
"""

from __future__ import annotations

import contextlib
import io
import json
import os
import stat
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

# run.py uses script-style absolute imports (`from _freshness import ...`), so
# the runners dir itself must be on sys.path (mirrors test_realbackend.py).
sys.path.insert(0, str(Path(__file__).resolve().parent))

import run  # noqa: E402


def _write_verifier(repo_root: Path, rel: str, exit_code: int) -> None:
    """Write a trivial verifier script under the temp repo that exits <exit_code>."""
    p = repo_root / rel
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(f"#!/usr/bin/env bash\nexit {exit_code}\n", encoding="utf-8")
    p.chmod(p.stat().st_mode | stat.S_IEXEC)


def _synthetic_row(*, status: str, blast: str = "P1") -> dict:
    """A legacy-shape structure row (no minted_at; old last_verified so the
    load-time honesty gate does not demand a claim_vs_assertion_audit) whose
    committed <status> can be made to differ from its live grade."""
    return {
        "id": "test/synthetic-immutable",
        "dimension": "structure",
        "kind": "mechanical",
        "status": status,
        "last_verified": "2026-04-01T00:00:00Z",
        "freshness_ttl": None,
        "blast_radius": blast,
        "cadences": ["pre-push"],
        "artifact": "quality/reports/verifications/synthetic-immutable.json",
        "verifier": {"script": "quality/gates/synthetic-verifier.sh"},
    }


def _drive(td: Path, *, committed_status: str, verifier_exit: int, persist: bool,
           blast: str = "P1") -> tuple[int, bytes, bytes, dict]:
    """Build a one-row synthetic catalog, drive run.main(), and return
    (exit_code, catalog_bytes_before, catalog_bytes_after, row_on_disk_after)."""
    cat_dir = td / "quality" / "catalogs"
    cat_dir.mkdir(parents=True)
    _write_verifier(td, "quality/gates/synthetic-verifier.sh", verifier_exit)
    catalog = {"dimension": "structure",
               "rows": [_synthetic_row(status=committed_status, blast=blast)]}
    cat_path = cat_dir / "synthetic.json"
    # Byte-for-byte snapshot BEFORE (matches the save_catalog serialization so a
    # no-op run is provably byte-identical, not merely semantically equal).
    run.save_catalog(cat_path, catalog)
    before = cat_path.read_bytes()

    argv = ["--cadence", "pre-push"] + (["--persist"] if persist else [])
    with mock.patch.object(run, "REPO_ROOT", td), \
         mock.patch.object(run, "CATALOG_DIR", cat_dir), \
         mock.patch.object(run, "REPORTS_DIR", td / "quality" / "reports"), \
         mock.patch.dict(os.environ, {}, clear=True), \
         contextlib.redirect_stdout(io.StringIO()):
        exit_code = run.main(argv)

    after = cat_path.read_bytes()
    row_after = json.loads(after)["rows"][0]
    return exit_code, before, after, row_after


class TestPersistGate(unittest.TestCase):
    """D-P96-01: cadence GATE runs validate-only; only --persist mints."""

    def test_validate_only_does_not_mutate_catalog(self):
        # Committed NOT-VERIFIED, verifier now exits 0 -> live grade is PASS.
        # A bare cadence run must NOT write the flip back to the catalog.
        with tempfile.TemporaryDirectory() as td:
            _exit, before, after, row = _drive(
                Path(td), committed_status="NOT-VERIFIED",
                verifier_exit=0, persist=False)
            self.assertEqual(before, after,
                             "cadence run mutated the catalog (self-mutation bug)")
            self.assertEqual(row["status"], "NOT-VERIFIED",
                             "committed status was overwritten by a validate-only run")

    def test_validate_only_still_blocks_red(self):
        # Committed PASS, verifier now exits 1 -> live grade FAIL on a P1 row.
        # compute_exit_code reads in-memory status, so the gate still blocks (1)
        # WITHOUT persisting the FAIL to the catalog (disk stays PASS).
        with tempfile.TemporaryDirectory() as td:
            exit_code, before, after, row = _drive(
                Path(td), committed_status="PASS",
                verifier_exit=1, persist=False, blast="P1")
            self.assertEqual(exit_code, 1, "validate-only run failed to block a RED P1 row")
            self.assertEqual(before, after, "validate-only run mutated the catalog")
            self.assertEqual(row["status"], "PASS",
                             "committed status was overwritten by a validate-only run")

    def test_persist_flag_mints_the_grade(self):
        # The explicit mint path must still write the graded status back so
        # catalog-first minting / un-waiving keeps working (grades not frozen).
        with tempfile.TemporaryDirectory() as td:
            _exit, before, after, row = _drive(
                Path(td), committed_status="NOT-VERIFIED",
                verifier_exit=0, persist=True)
            self.assertNotEqual(before, after,
                                "--persist did not write the graded status (mint path broke)")
            self.assertEqual(row["status"], "PASS",
                             "--persist did not mint the live PASS grade")

    def test_persist_default_is_off(self):
        # Guard the default: a run.py invocation WITHOUT --persist parses to
        # persist=False (the whole fix hinges on this default).
        parser = run._build_arg_parser()
        self.assertFalse(parser.parse_args(["--cadence", "pre-push"]).persist)
        self.assertTrue(
            parser.parse_args(["--cadence", "pre-push", "--persist"]).persist)


if __name__ == "__main__":
    unittest.main()
