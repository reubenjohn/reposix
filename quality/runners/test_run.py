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
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path
from unittest import mock

# run.py uses script-style absolute imports (`from _freshness import ...`), so
# the runners dir itself must be on sys.path (mirrors test_realbackend.py).
sys.path.insert(0, str(Path(__file__).resolve().parent))

import run  # noqa: E402
import _env_load  # noqa: E402  (P123 SC1 / DRAIN-03: ./.env self-sourcing)
import _persist_guard  # noqa: E402  (P123 SC2 / DRAIN-04: committed-GREEN downgrade guard)


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


class TestEnvSelfSourcing(unittest.TestCase):
    """P123 SC1 / DRAIN-03: run.py self-sources ./.env — present-only,
    non-clobbering, no value leak. Backs catalog row
    structure/quality-runner-sources-dotenv.

    Closes the false-green-preflight / silent-skip gap: preflight sourced .env
    but run.py did not, so a pre-release-real-backend cadence silently skipped
    every real-backend row to NOT-VERIFIED unless the caller pre-sourced .env.

    Hermetic: every fixture .env is rooted under tempfile.TemporaryDirectory
    and os.environ is patched with clear=True so no real .env / real cred is
    ever read or mutated (mirrors _drive()'s isolation style above).
    """

    _ENV_BODY = "FOO=bar\n# comment\n\nBAZ=qux\n"

    def test_unset_keys_are_loaded(self):
        # Test 1: unset keys from .env land in os.environ.
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            (root / ".env").write_text(self._ENV_BODY, encoding="utf-8")
            with mock.patch.dict(os.environ, {}, clear=True):
                _env_load.load_dotenv_if_present(root)
                self.assertEqual(os.environ.get("FOO"), "bar",
                                 "an unset key from .env was not loaded")
                self.assertEqual(os.environ.get("BAZ"), "qux",
                                 "a key after a comment/blank line was not loaded")

    def test_export_prefixed_line_loads_bare_key(self):
        # Test: a `.env` line written with shell export syntax
        # (`export KEY=value`, as scripts/preflight-real-backends.sh honors when
        # it source-includes .env) must load KEY, not "export KEY" — else run.py
        # skips the row to NOT-VERIFIED while preflight sees the cred, the exact
        # false-green-preflight divergence SC1/DRAIN-03 closes. A bare
        # `KEY=value` line must still load too, and `exportFOO=x` (no whitespace
        # after `export`) must be left as the literal key `exportFOO`.
        body = "export GITHUB_TOKEN=ghp_x\nBARE_KEY=plain\nexportFOO=y\n"
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            (root / ".env").write_text(body, encoding="utf-8")
            with mock.patch.dict(os.environ, {}, clear=True):
                _env_load.load_dotenv_if_present(root)
                self.assertEqual(os.environ.get("GITHUB_TOKEN"), "ghp_x",
                                 "`export KEY=value` loaded the wrong key "
                                 "(export prefix not stripped)")
                self.assertNotIn("export GITHUB_TOKEN", os.environ,
                                 "the literal 'export KEY' leaked in as a key")
                self.assertEqual(os.environ.get("BARE_KEY"), "plain",
                                 "a bare KEY=value line stopped loading after the "
                                 "export-strip change")
                self.assertEqual(os.environ.get("exportFOO"), "y",
                                 "`exportFOO=` (no space) must stay key 'exportFOO' "
                                 "— `export` is a whole-token prefix only")

    def test_already_set_env_wins(self):
        # Test 2: an explicitly-set var must NOT be clobbered by .env.
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            (root / ".env").write_text(self._ENV_BODY, encoding="utf-8")
            with mock.patch.dict(os.environ, {"FOO": "keep-me"}, clear=True):
                _env_load.load_dotenv_if_present(root)
                self.assertEqual(os.environ.get("FOO"), "keep-me",
                                 ".env clobbered an already-exported env var")
                self.assertEqual(os.environ.get("BAZ"), "qux",
                                 "an unset key from .env should still load")

    def test_missing_env_is_silent_noop(self):
        # Test 3: no .env present -> silent no-op (no exception, env unchanged).
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)  # deliberately no .env written
            with mock.patch.dict(os.environ, {"ONLY": "me"}, clear=True):
                _env_load.load_dotenv_if_present(root)  # must not raise
                self.assertEqual(dict(os.environ), {"ONLY": "me"},
                                 "a missing .env changed os.environ")

    def test_no_secret_value_reaches_stderr(self):
        # Test 4: the diagnostic names loaded KEY names but NEVER their values.
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            (root / ".env").write_text(self._ENV_BODY, encoding="utf-8")
            buf = io.StringIO()
            with mock.patch.dict(os.environ, {}, clear=True), \
                 contextlib.redirect_stderr(buf):
                _env_load.load_dotenv_if_present(root)
            captured = buf.getvalue()
            self.assertIn("FOO", captured, "loaded KEY name not reported")
            self.assertIn("BAZ", captured, "loaded KEY name not reported")
            self.assertNotIn("bar", captured, "secret VALUE 'bar' leaked to stderr")
            self.assertNotIn("qux", captured, "secret VALUE 'qux' leaked to stderr")


class TestPersistDowngradeGuard(unittest.TestCase):
    """P123 SC2 / DRAIN-04: --persist refuses to silently downgrade a
    committed-GREEN (PASS/WAIVED) catalog row to an EXPLICIT regression
    (FAIL/PARTIAL) without --allow-downgrade. Backs catalog row
    structure/persist-refuses-downgrade.

    Load-bearing distinction (Test 6): a demotion to NOT-VERIFIED (freshness-TTL
    expiry, missing verifier, env-skip, exit-75) is NOT a regression and must
    NEVER need the flag — otherwise the phase's own freshness-invariant mints
    (which produce NOT-VERIFIED) would deadlock against this guard.

    Reality-faithful: each test builds a THROWAWAY /tmp git repo, commits a
    "before" catalog so the guard has a real `git show HEAD:` baseline to read,
    then drives the REAL run.main() persist path over that repo. All git setup +
    the run happen within one test method — no shared-repo writes (the git init /
    config / commit target a disposable tempdir, never the project repo).
    """

    # A committer identity + config isolation applied to every throwaway repo.
    # Local-only (never --global); global/system config are pinned to os.devnull
    # so the fixture never reads or writes the developer's ~/.gitconfig.
    _GIT_ENV_EXTRA = {
        "GIT_CONFIG_GLOBAL": os.devnull,
        "GIT_CONFIG_SYSTEM": os.devnull,
    }

    def _git(self, td: Path, *args: str) -> None:
        env = {**os.environ, "HOME": str(td), **self._GIT_ENV_EXTRA}
        subprocess.run(
            ["git", *args], cwd=str(td), env=env,
            check=True, capture_output=True, text=True,
        )

    def _seed_repo(self, td: Path, *, head_rows: list[dict],
                   working_status: str, working_blast: str, verifier_exit: int) -> Path:
        """Init a throwaway git repo, commit `head_rows` as the HEAD baseline,
        then overwrite the working copy with a single row at `working_status`
        (which may differ from HEAD) and drop a verifier that exits
        `verifier_exit`. Returns the catalog path."""
        cat_dir = td / "quality" / "catalogs"
        cat_dir.mkdir(parents=True)
        cat_path = cat_dir / "synthetic.json"
        _write_verifier(td, "quality/gates/synthetic-verifier.sh", verifier_exit)

        # 1) Commit the HEAD baseline (what `git show HEAD:` will return).
        run.save_catalog(cat_path, {"dimension": "structure", "rows": head_rows})
        self._git(td, "init", "-q")
        self._git(td, "config", "user.email", "gate@test.invalid")
        self._git(td, "config", "user.name", "downgrade-guard-test")
        self._git(td, "config", "commit.gpgsign", "false")
        self._git(td, "add", "quality/catalogs/synthetic.json")
        self._git(td, "commit", "-q", "-m", "seed baseline")

        # 2) Overwrite the WORKING copy (may differ from HEAD — this is what
        #    run.py loads + re-grades). The guard still reads HEAD, not this.
        run.save_catalog(cat_path, {"dimension": "structure",
                                    "rows": [_synthetic_row(status=working_status,
                                                            blast=working_blast)]})
        return cat_path

    def _run_persist(self, td: Path, cat_path: Path, *, allow_downgrade: bool
                     ) -> tuple[int, bytes, bytes, dict, str]:
        """Drive run.main() --persist over the throwaway repo. Returns
        (exit_code, catalog_bytes_before, catalog_bytes_after, row_on_disk_after,
        stderr_text)."""
        cat_dir = cat_path.parent
        argv = ["--cadence", "pre-push", "--persist"]
        if allow_downgrade:
            argv.append("--allow-downgrade")
        # Preserve PATH (git + bash lookup) + a clean HOME; pin git config to
        # devnull; clear all other env (incl. any real-backend creds).
        penv = {
            "PATH": os.environ.get("PATH", ""),
            "HOME": str(td),
            **self._GIT_ENV_EXTRA,
        }
        before = cat_path.read_bytes()
        err = io.StringIO()
        with mock.patch.object(run, "REPO_ROOT", td), \
             mock.patch.object(run, "CATALOG_DIR", cat_dir), \
             mock.patch.object(run, "REPORTS_DIR", td / "quality" / "reports"), \
             mock.patch.dict(os.environ, penv, clear=True), \
             contextlib.redirect_stdout(io.StringIO()), \
             contextlib.redirect_stderr(err):
            exit_code = run.main(argv)
        after = cat_path.read_bytes()
        row_after = json.loads(after)["rows"][0]
        return exit_code, before, after, row_after, err.getvalue()

    def test_1_committed_pass_downgraded_to_fail_is_refused(self):
        # Committed PASS, fresh grade FAIL, NO --allow-downgrade -> save_catalog
        # NOT called (disk stays PASS) and the refusal names row id + PASS + FAIL
        # + the literal --allow-downgrade recovery command.
        with tempfile.TemporaryDirectory() as td:
            td = Path(td)
            cat_path = self._seed_repo(
                td, head_rows=[_synthetic_row(status="PASS")],
                working_status="PASS", working_blast="P1", verifier_exit=1)
            exit_code, before, after, row, err = self._run_persist(
                td, cat_path, allow_downgrade=False)
            self.assertEqual(before, after,
                             "REFUSED downgrade still wrote the catalog (silent overwrite)")
            self.assertEqual(row["status"], "PASS",
                             "committed PASS was overwritten with FAIL without --allow-downgrade")
            self.assertIn("test/synthetic-immutable", err, "refusal did not name the row id")
            self.assertIn("PASS", err, "refusal did not name the old status")
            self.assertIn("FAIL", err, "refusal did not name the new status")
            self.assertIn("--allow-downgrade", err,
                          "refusal did not teach the --allow-downgrade recovery command")
            self.assertEqual(exit_code, 1, "a blocked downgrade must surface as a failing run")

    def test_2_allow_downgrade_flag_persists_the_regression(self):
        # Same setup, but --allow-downgrade -> save_catalog IS called (FAIL
        # persisted) and a loud per-row notice is still printed (never silent).
        with tempfile.TemporaryDirectory() as td:
            td = Path(td)
            cat_path = self._seed_repo(
                td, head_rows=[_synthetic_row(status="PASS")],
                working_status="PASS", working_blast="P1", verifier_exit=1)
            _exit, before, after, row, err = self._run_persist(
                td, cat_path, allow_downgrade=True)
            self.assertNotEqual(before, after,
                                "--allow-downgrade did not persist the regression")
            self.assertEqual(row["status"], "FAIL",
                             "--allow-downgrade did not write the FAIL grade")
            self.assertIn("test/synthetic-immutable", err,
                          "the allowed-downgrade notice did not name the row")
            self.assertTrue(
                any(tok in err for tok in ("ALLOWED", "allow-downgrade")),
                "an explicitly-permitted downgrade was persisted SILENTLY (no notice)")

    def test_3_committed_waived_downgraded_to_fail_is_refused(self):
        # WAIVED counts as green: WAIVED -> FAIL is also blocked without the flag.
        with tempfile.TemporaryDirectory() as td:
            td = Path(td)
            cat_path = self._seed_repo(
                td, head_rows=[_synthetic_row(status="WAIVED")],
                working_status="WAIVED", working_blast="P1", verifier_exit=1)
            _exit, before, after, row, err = self._run_persist(
                td, cat_path, allow_downgrade=False)
            self.assertEqual(before, after, "WAIVED->FAIL downgrade was silently written")
            self.assertEqual(row["status"], "WAIVED",
                             "committed WAIVED was overwritten with FAIL without the flag")
            self.assertIn("WAIVED", err, "refusal did not name the old WAIVED status")
            self.assertIn("FAIL", err, "refusal did not name the new FAIL status")

    def test_4_brand_new_row_absent_from_head_mints_freely(self):
        # A row NOT present in the git-HEAD catalog (e.g. straight out of a
        # catalog-first commit) has no committed baseline -> never blocked,
        # regardless of its fresh grade.
        with tempfile.TemporaryDirectory() as td:
            td = Path(td)
            cat_path = self._seed_repo(
                td, head_rows=[],  # HEAD has NO rows -> no baseline for our id
                working_status="PASS", working_blast="P1", verifier_exit=1)
            _exit, before, after, row, err = self._run_persist(
                td, cat_path, allow_downgrade=False)
            self.assertNotEqual(before, after,
                                "a brand-new row (absent from HEAD) failed to mint")
            self.assertEqual(row["status"], "FAIL",
                             "a brand-new row did not mint its fresh grade")
            self.assertNotIn("REFUSED", err,
                             "a brand-new row was wrongly blocked by the downgrade guard")

    def test_5_unchanged_pass_is_not_blocked(self):
        # Committed PASS, fresh grade STILL PASS -> not a downgrade; nothing
        # changes and nothing is refused.
        with tempfile.TemporaryDirectory() as td:
            td = Path(td)
            cat_path = self._seed_repo(
                td, head_rows=[_synthetic_row(status="PASS")],
                working_status="PASS", working_blast="P1", verifier_exit=0)
            _exit, before, after, row, err = self._run_persist(
                td, cat_path, allow_downgrade=False)
            self.assertEqual(before, after, "an unchanged PASS row dirtied the catalog")
            self.assertEqual(row["status"], "PASS")
            self.assertNotIn("REFUSED", err, "an unchanged PASS was wrongly refused")
        # Direct-function reinforcement: PASS->PASS is never a violation.
        self.assertEqual(
            _persist_guard.refuse_downgrade_without_flag(
                {"x": "PASS"}, [{"id": "x", "status": "PASS"}]),
            [], "PASS->PASS must not be reported as a downgrade")

    def test_6_downgrade_to_not_verified_is_always_allowed(self):
        # SC2 regression-vs-TTL semantics / deadlock prevention: committed PASS,
        # fresh grade NOT-VERIFIED (here via an exit-75 verifier — the same shape
        # as a freshness-TTL expiry / missing-verifier / env-skip demotion) is
        # NOT blocked and needs NO --allow-downgrade. This is the exact case that
        # must never deadlock the phase's own freshness-invariant mints.
        with tempfile.TemporaryDirectory() as td:
            td = Path(td)
            cat_path = self._seed_repo(
                td, head_rows=[_synthetic_row(status="PASS")],
                working_status="PASS", working_blast="P1", verifier_exit=75)
            _exit, before, after, row, err = self._run_persist(
                td, cat_path, allow_downgrade=False)
            self.assertNotEqual(before, after,
                                "a legitimate NOT-VERIFIED demotion failed to persist")
            self.assertEqual(row["status"], "NOT-VERIFIED",
                             "PASS->NOT-VERIFIED demotion was not written")
            self.assertNotIn("REFUSED", err,
                             "PASS->NOT-VERIFIED was wrongly treated as a downgrade "
                             "(would deadlock freshness-TTL mints)")
        # Direct-function reinforcement across EVERY NOT-VERIFIED cause: the
        # guard consults status only, so any committed-green -> NOT-VERIFIED is [].
        for old in ("PASS", "WAIVED"):
            self.assertEqual(
                _persist_guard.refuse_downgrade_without_flag(
                    {"x": old}, [{"id": "x", "status": "NOT-VERIFIED"}]),
                [], f"{old}->NOT-VERIFIED must never be a downgrade")

    def test_7_blocked_downgrade_forces_nonzero_exit_even_for_p2(self):
        # The blocked-downgrade -> exit 1 contract is INDEPENDENT of
        # compute_exit_code: a P2 FAIL does not itself trip compute_exit_code
        # (only P0/P1 do), so a green-looking run must STILL exit non-zero when a
        # downgrade was refused — a block is never swallowed into a green exit.
        with tempfile.TemporaryDirectory() as td:
            td = Path(td)
            cat_path = self._seed_repo(
                td, head_rows=[_synthetic_row(status="PASS", blast="P2")],
                working_status="PASS", working_blast="P2", verifier_exit=1)
            exit_code, before, after, _row, err = self._run_persist(
                td, cat_path, allow_downgrade=False)
            self.assertEqual(exit_code, 1,
                             "a refused P2 downgrade was swallowed into a green exit code")
            self.assertEqual(before, after, "refused P2 downgrade still wrote the catalog")
            self.assertIn("REFUSED", err)

    def test_allow_downgrade_default_is_off(self):
        # Mirror test_persist_default_is_off: the whole refuse-by-default contract
        # hinges on --allow-downgrade parsing to False when absent.
        parser = run._build_arg_parser()
        self.assertFalse(
            parser.parse_args(["--cadence", "pre-push"]).allow_downgrade,
            "--allow-downgrade must default to False (refuse-by-default)")
        self.assertTrue(
            parser.parse_args(
                ["--cadence", "pre-push", "--persist", "--allow-downgrade"]
            ).allow_downgrade)

    def test_committed_head_statuses_none_when_no_baseline(self):
        # No git repo / untracked catalog -> None (not an error): the caller
        # treats None as "no baseline", exempting every row. Guards the exact
        # fail-open path the existing TestPersistGate --persist test relies on.
        with tempfile.TemporaryDirectory() as td:
            td = Path(td)
            cat_dir = td / "quality" / "catalogs"
            cat_dir.mkdir(parents=True)
            cat_path = cat_dir / "synthetic.json"
            run.save_catalog(cat_path, {"dimension": "structure", "rows": []})
            # td is NOT a git repo -> git show fails -> None.
            self.assertIsNone(
                _persist_guard.committed_head_statuses(td, cat_path),
                "a non-git / untracked catalog must yield None, not raise or fabricate")


class TestPersistCatalogLock(unittest.TestCase):
    """P123 SC3 / DRAIN-05: concurrent --persist runners cannot race-corrupt the
    shared catalog. Backs catalog row structure/persist-catalog-write-locked.

    Three invariants (each an expected.assert on the backing row) + a fourth
    end-to-end reality proof of the closed race:
      1. a second --persist writer BLOCKS on the OS-level flock while a first
         holds it -- proven by a REAL subprocess and a wall-clock >= ~1.8s (a
         mock or in-process-only lock would be ~0s and never stop a real process).
      2. a validate-only (no --persist) run never opens or contends for the lock.
      3. single-writer --persist minting is unchanged with the lock in place.
      4. (reality) two concurrent run.py --persist processes on the SAME catalog
         produce valid/parseable JSON with NO lost update -- both writers'
         intended row flips survive because the whole read-modify-write serializes
         (an unlocked runner would drop one writer's flip via a stale full-file
         overwrite -- the exact GTH-V15-01 hazard this row closes).

    All setup lives under tempfile.TemporaryDirectory (no shared-repo writes); the
    lock file is <td>/quality/reports/.persist.lock, contended by REAL subprocesses
    so the exclusivity proof is genuine OS behavior, not an in-process assertion.
    The concurrency verifiers are trivial sleep/exit bash scripts -- deliberately
    cargo-free (two cargo builds in parallel would violate the one-cargo rule).
    """

    _RUNNERS_DIR = str(Path(__file__).resolve().parent)

    # Holder: acquire the real flock, drop a sentinel so the test knows the lock
    # is genuinely HELD (not merely that the process started), then sleep.
    _HOLDER = (
        "import sys, time\n"
        "from pathlib import Path\n"
        "sys.path.insert(0, sys.argv[1])\n"
        "import _persist_guard\n"
        "td = Path(sys.argv[2]); hold = float(sys.argv[3])\n"
        "with _persist_guard.catalog_persist_lock(td):\n"
        "    (td / '.locked').write_text('held')\n"
        "    time.sleep(hold)\n"
    )

    # Driver: point run's module globals at the throwaway repo and drive the REAL
    # run.main() --persist path in a SEPARATE interpreter, so the OS-level flock
    # actually engages across processes (an in-process call shares one flock owner).
    _DRIVER = (
        "import sys\n"
        "from pathlib import Path\n"
        "sys.path.insert(0, sys.argv[1])\n"
        "import run\n"
        "td = Path(sys.argv[2]); cadence = sys.argv[3]\n"
        "run.REPO_ROOT = td\n"
        "run.CATALOG_DIR = td / 'quality' / 'catalogs'\n"
        "run.REPORTS_DIR = td / 'quality' / 'reports'\n"
        "sys.exit(run.main(['--cadence', cadence, '--persist']))\n"
    )

    def _spawn_holder(self, td: Path, hold: float) -> subprocess.Popen:
        proc = subprocess.Popen(
            [sys.executable, "-c", self._HOLDER,
             self._RUNNERS_DIR, str(td), str(hold)])
        # Wait for the sentinel so we time against a genuinely-HELD lock, never a
        # process that has spawned but not yet acquired.
        deadline = time.monotonic() + 5.0
        while not (td / ".locked").exists():
            if time.monotonic() > deadline:
                proc.kill()
                self.fail("holder subprocess never acquired the lock (no sentinel)")
            time.sleep(0.02)
        return proc

    def test_1_second_persist_blocks_on_held_lock(self):
        with tempfile.TemporaryDirectory() as td:
            td = Path(td)
            holder = self._spawn_holder(td, hold=2.5)
            try:
                t0 = time.monotonic()
                with _persist_guard.catalog_persist_lock(td):
                    elapsed = time.monotonic() - t0
                self.assertGreaterEqual(
                    elapsed, 1.8,
                    f"acquire returned in {elapsed:.2f}s -- it did NOT block on the "
                    f"real held OS-level flock (a mock/in-process lock would be ~0s), "
                    f"so a second real --persist writer could interleave")
            finally:
                holder.wait(timeout=10)

    def test_2_validate_only_never_touches_the_lock(self):
        # Part A: a validate-only run.main() completes promptly even while a
        # separate process holds the persist lock (nullcontext branch -> no wait).
        with tempfile.TemporaryDirectory() as td:
            td = Path(td)
            cat_dir = td / "quality" / "catalogs"
            cat_dir.mkdir(parents=True)
            _write_verifier(td, "quality/gates/synthetic-verifier.sh", 0)
            run.save_catalog(cat_dir / "synthetic.json",
                             {"dimension": "structure",
                              "rows": [_synthetic_row(status="NOT-VERIFIED")]})
            holder = self._spawn_holder(td, hold=8.0)
            try:
                t0 = time.monotonic()
                with mock.patch.object(run, "REPO_ROOT", td), \
                     mock.patch.object(run, "CATALOG_DIR", cat_dir), \
                     mock.patch.object(run, "REPORTS_DIR", td / "quality" / "reports"), \
                     contextlib.redirect_stdout(io.StringIO()):
                    run.main(["--cadence", "pre-push"])
                elapsed = time.monotonic() - t0
                self.assertLess(
                    elapsed, 2.0,
                    f"validate-only run blocked {elapsed:.2f}s while the persist lock "
                    f"was held -- it must never acquire or contend for the lock")
            finally:
                holder.kill()
                holder.wait(timeout=10)
        # Part B: a validate-only run in a clean tree never CREATES the lock file.
        with tempfile.TemporaryDirectory() as td:
            td = Path(td)
            _drive(td, committed_status="NOT-VERIFIED", verifier_exit=0, persist=False)
            self.assertFalse(
                (td / "quality" / "reports" / ".persist.lock").exists(),
                "a validate-only run opened/created the persist lock file")

    def test_3_single_writer_persist_still_mints(self):
        # Uncontended --persist under the always-on lock still writes the flip
        # (the lock must not regress single-writer minting) AND does open the lock
        # file (contrast with Part B above -- the persist path DOES take the lock).
        with tempfile.TemporaryDirectory() as td:
            td = Path(td)
            _exit, before, after, row = _drive(
                td, committed_status="NOT-VERIFIED", verifier_exit=0, persist=True)
            self.assertNotEqual(before, after,
                                "--persist under the lock failed to mint the grade")
            self.assertEqual(row["status"], "PASS",
                             "--persist under the lock did not write the live PASS grade")
            self.assertTrue(
                (td / "quality" / "reports" / ".persist.lock").exists(),
                "the --persist path did not open the lock file")

    @staticmethod
    def _lock_test_row(rid: str, cadence: str, verifier_rel: str) -> dict:
        # New-regime row: carries minted_at + claim_vs_assertion_audit so that
        # AFTER the first writer flips it PASS (which stamps a fresh, >=P90-cutoff
        # last_verified), the SECOND writer's load_catalog -> validate_row does NOT
        # reject it for a missing minted_at anchor. A legacy no-minted_at row would
        # crash the second loader instead of exercising the concurrency path.
        # No expected.asserts -> F-K4b congruence is a no-op; a bare exit-0
        # verifier grades PASS cleanly.
        return {
            "id": rid, "dimension": "structure", "kind": "mechanical",
            "minted_at": "2026-07-18T00:00:00Z",
            "claim_vs_assertion_audit": (
                "Synthetic lock-contention fixture row: two concurrent --persist "
                "writers each flip a different row; the full read-modify-write "
                "lock must let both flips survive without a lost update."
            ),
            "status": "NOT-VERIFIED", "last_verified": "2026-04-01T00:00:00Z",
            "freshness_ttl": None, "blast_radius": "P1", "cadences": [cadence],
            "artifact": f"quality/reports/verifications/{rid.replace('/', '-')}.json",
            "verifier": {"script": verifier_rel},
        }

    def test_4_two_concurrent_persist_no_lost_update(self):
        with tempfile.TemporaryDirectory() as td:
            td = Path(td)
            cat_dir = td / "quality" / "catalogs"
            cat_dir.mkdir(parents=True)
            # Two sleeping verifiers widen the read-modify window so an UNLOCKED
            # runner would lost-update; both exit 0 -> both rows must flip PASS.
            for rel in ("quality/gates/slow-a.sh", "quality/gates/slow-b.sh"):
                p = td / rel
                p.parent.mkdir(parents=True, exist_ok=True)
                p.write_text("#!/usr/bin/env bash\nsleep 0.8\nexit 0\n", encoding="utf-8")
                p.chmod(p.stat().st_mode | stat.S_IEXEC)
            cat_path = cat_dir / "concurrent.json"
            run.save_catalog(cat_path, {"dimension": "structure", "rows": [
                self._lock_test_row("test/lock-row-a", "pre-push", "quality/gates/slow-a.sh"),
                self._lock_test_row("test/lock-row-b", "pre-pr", "quality/gates/slow-b.sh"),
            ]})
            # Two writers on the SAME catalog: A flips only row-a (pre-push), B
            # flips only row-b (pre-pr). Each writes the WHOLE file. With the
            # full-RMW lock, whichever runs second reads the first's committed
            # flip, so BOTH survive; unlocked, the second overwrites from a stale
            # snapshot and one flip is lost.
            procs = [
                subprocess.Popen(
                    [sys.executable, "-c", self._DRIVER,
                     self._RUNNERS_DIR, str(td), cadence],
                    stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                for cadence in ("pre-push", "pre-pr")
            ]
            for p in procs:
                p.wait(timeout=30)
            # (a) valid/parseable JSON -- no torn/interleaved write.
            final = json.loads(cat_path.read_text(encoding="utf-8"))
            by_id = {r["id"]: r.get("status") for r in final["rows"]}
            # (b) NO lost update -- both writers' intended flips survive.
            self.assertEqual(
                by_id.get("test/lock-row-a"), "PASS",
                "row-a's flip was lost to a concurrent --persist writer (lost update)")
            self.assertEqual(
                by_id.get("test/lock-row-b"), "PASS",
                "row-b's flip was lost to a concurrent --persist writer (lost update)")


if __name__ == "__main__":
    unittest.main()
