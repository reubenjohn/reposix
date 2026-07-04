"""Tests for quality/runners/_audit_field.py — RBF-FW-11.

Run: python3 -m unittest quality.runners.test_audit_field -v
"""
from __future__ import annotations

import sys
import unittest
from datetime import datetime
from pathlib import Path

# Make the runner modules importable top-level (mirrors test_realbackend.py;
# run.py uses script-style absolute imports, so package-context import of
# `quality.runners.run` breaks on `from _freshness import ...`).
sys.path.insert(0, str(Path(__file__).resolve().parent))

import _audit_field  # noqa: E402


def parse_rfc3339(s):
    """Stub matching the runner's parser signature."""
    return datetime.fromisoformat(s.replace("Z", "+00:00"))


VALID_AUDIT = (
    "The verifier asserts X holds; if the description claim were false, "
    "the assertion would fail because the synthetic counter-example would "
    "not produce the expected output."
)


class TestValidateRow(unittest.TestCase):
    def test_pre_cutoff_row_passes_without_field(self):
        row = {"id": "x/y", "last_verified": "2026-04-01T00:00:00Z"}
        _audit_field.validate_row(row, "test.json", parse_rfc3339)  # no raise

    def test_on_cutoff_row_missing_field_fails(self):
        row = {"id": "x/y", "last_verified": "2026-05-08T00:00:00Z"}
        with self.assertRaises(SystemExit) as ctx:
            _audit_field.validate_row(row, "test.json", parse_rfc3339)
        self.assertIn("missing claim_vs_assertion_audit", str(ctx.exception))

    def test_on_cutoff_row_with_short_field_fails(self):
        row = {
            "id": "x/y",
            "last_verified": "2026-05-08T00:00:00Z",
            "claim_vs_assertion_audit": "too short",
        }
        with self.assertRaises(SystemExit):
            _audit_field.validate_row(row, "test.json", parse_rfc3339)

    def test_on_cutoff_row_with_valid_field_passes(self):
        row = {
            "id": "x/y",
            "last_verified": "2026-05-08T00:00:00Z",
            "claim_vs_assertion_audit": VALID_AUDIT,
        }
        _audit_field.validate_row(row, "test.json", parse_rfc3339)

    def test_null_last_verified_treated_as_new_fails_without_field(self):
        row = {"id": "x/y", "last_verified": None}
        with self.assertRaises(SystemExit):
            _audit_field.validate_row(row, "test.json", parse_rfc3339)

    def test_null_last_verified_with_valid_field_passes(self):
        row = {"id": "x/y", "last_verified": None, "claim_vs_assertion_audit": VALID_AUDIT}
        _audit_field.validate_row(row, "test.json", parse_rfc3339)

    def test_post_cutoff_row_missing_field_fails(self):
        row = {"id": "x/y", "last_verified": "2026-06-01T12:00:00Z"}
        with self.assertRaises(SystemExit):
            _audit_field.validate_row(row, "test.json", parse_rfc3339)

    def test_date_parsing_error_fails_loud(self):
        row = {"id": "x/y", "last_verified": "not-a-date"}
        with self.assertRaises(Exception):
            _audit_field.validate_row(row, "test.json", parse_rfc3339)


class TestKindShellSubprocessTranscriptRule(unittest.TestCase):
    """kind:shell-subprocess rows MUST have transcript contract."""

    def test_shell_subprocess_without_transcript_fails(self):
        row = {
            "id": "x/y",
            "last_verified": "2026-05-08T00:00:00Z",
            "claim_vs_assertion_audit": VALID_AUDIT,
            "kind": "shell-subprocess",
            "expected": {"asserts": ["exits 0"]},  # no transcript mention
        }
        with self.assertRaises(SystemExit) as ctx:
            _audit_field.validate_row(row, "test.json", parse_rfc3339)
        self.assertIn("kind: shell-subprocess", str(ctx.exception))
        self.assertIn("transcript", str(ctx.exception).lower())

    def test_shell_subprocess_with_expected_artifact_transcript_path_passes(self):
        row = {
            "id": "x/y",
            "last_verified": "2026-05-08T00:00:00Z",
            "claim_vs_assertion_audit": VALID_AUDIT,
            "kind": "shell-subprocess",
            "expected": {
                "asserts": ["exits 0"],
                "artifact": {"transcript_path": "quality/reports/transcripts/x.txt"},
            },
        }
        _audit_field.validate_row(row, "test.json", parse_rfc3339)  # no raise

    def test_shell_subprocess_with_transcript_in_asserts_passes(self):
        row = {
            "id": "x/y",
            "last_verified": "2026-05-08T00:00:00Z",
            "claim_vs_assertion_audit": VALID_AUDIT,
            "kind": "shell-subprocess",
            "expected": {"asserts": ["transcript artifact written"]},
        }
        _audit_field.validate_row(row, "test.json", parse_rfc3339)  # no raise

    def test_non_shell_subprocess_kind_does_not_require_transcript(self):
        row = {
            "id": "x/y",
            "last_verified": "2026-05-08T00:00:00Z",
            "claim_vs_assertion_audit": VALID_AUDIT,
            "kind": "mechanical",
            "expected": {"asserts": ["grep returns zero matches"]},
        }
        _audit_field.validate_row(row, "test.json", parse_rfc3339)  # no raise


class TestComputeHash(unittest.TestCase):
    def test_returns_sha256_hex(self):
        row = {"claim_vs_assertion_audit": VALID_AUDIT}
        h = _audit_field.compute_hash(row)
        self.assertIsNotNone(h)
        self.assertEqual(len(h), 64)
        self.assertTrue(all(c in "0123456789abcdef" for c in h))

    def test_returns_none_if_absent(self):
        self.assertIsNone(_audit_field.compute_hash({}))
        self.assertIsNone(_audit_field.compute_hash({"claim_vs_assertion_audit": None}))

    def test_strips_whitespace(self):
        row1 = {"claim_vs_assertion_audit": VALID_AUDIT}
        row2 = {"claim_vs_assertion_audit": f"  {VALID_AUDIT}  \n"}
        self.assertEqual(_audit_field.compute_hash(row1), _audit_field.compute_hash(row2))


class TestOD2WaiverRejection(unittest.TestCase):
    """P89 cross-AI review H3: waivers forbidden on pre-release-real-backend rows."""

    def _row(self, **overrides):
        row = {
            "id": "x/y",
            "last_verified": None,
            "claim_vs_assertion_audit": VALID_AUDIT,
            "cadences": ["pre-release-real-backend"],
            "waiver": None,
        }
        row.update(overrides)
        return row

    def test_waiver_on_real_backend_row_fails_loud(self):
        row = self._row(waiver={"until": "2099-01-01T00:00:00Z", "reason": "nope"})
        with self.assertRaises(SystemExit) as ctx:
            _audit_field.validate_row(row, "test.json", parse_rfc3339)
        self.assertIn("OD-2 forbids", str(ctx.exception))

    def test_null_waiver_on_real_backend_row_passes(self):
        _audit_field.validate_row(self._row(), "test.json", parse_rfc3339)  # no raise

    def test_waiver_on_other_cadence_row_passes(self):
        row = self._row(
            cadences=["pre-push"],
            waiver={"until": "2099-01-01T00:00:00Z", "reason": "sanctioned elsewhere"},
        )
        _audit_field.validate_row(row, "test.json", parse_rfc3339)  # no raise


class TestMintedAt(unittest.TestCase):
    """D90-03 (cross-AI H2): minted_at is the write-once, immutable
    audit-cutoff anchor; post-P90 rows must carry it."""

    def test_pre_cutoff_mint_with_backdated_lv_no_audit_passes(self):
        # minted_at < 2026-05-08 -> is_new False -> no audit required (legacy).
        row = {
            "id": "x/y",
            "minted_at": "2026-04-01T00:00:00Z",
            "last_verified": "2026-04-01T00:00:00Z",
        }
        _audit_field.validate_row(row, "test.json", parse_rfc3339)  # no raise

    def test_post_cutoff_mint_defeats_backdated_lv(self):
        # The H2 fix: minted_at >= cutoff makes is_new True even though a
        # hand-edited last_verified is backdated below the cutoff.
        row = {
            "id": "x/y",
            "minted_at": "2026-06-01T00:00:00Z",
            "last_verified": "2026-04-01T00:00:00Z",  # backdated dodge
        }
        with self.assertRaises(SystemExit) as ctx:
            _audit_field.validate_row(row, "test.json", parse_rfc3339)
        self.assertIn("claim_vs_assertion_audit", str(ctx.exception))

    def test_absent_minted_at_with_post_p90_lv_fails(self):
        row = {"id": "x/y", "last_verified": "2026-07-06T00:00:00Z",
               "claim_vs_assertion_audit": VALID_AUDIT}
        with self.assertRaises(SystemExit) as ctx:
            _audit_field.validate_row(row, "test.json", parse_rfc3339)
        self.assertIn("minted_at", str(ctx.exception))

    def test_absent_minted_at_with_null_lv_not_forced(self):
        # 5-legacy-row guard: null last_verified must NOT be forced to add
        # minted_at (the reject's first clause requires lv is not None).
        row = {"id": "x/y", "last_verified": None,
               "claim_vs_assertion_audit": VALID_AUDIT}
        _audit_field.validate_row(row, "test.json", parse_rfc3339)  # no raise

    def test_minted_at_present_is_sole_anchor_ignores_lv(self):
        # minted_at pre-cutoff wins even when last_verified is post-cutoff.
        row = {
            "id": "x/y",
            "minted_at": "2026-04-01T00:00:00Z",
            "last_verified": "2026-06-01T00:00:00Z",
        }
        _audit_field.validate_row(row, "test.json", parse_rfc3339)  # no raise


class TestCoverageKind(unittest.TestCase):
    """RBF-FW-06 / D90-05: transport/perf rows minted >= P90 need
    coverage_kind: real-backend (or a compliant waiver); legacy = RAISE-only."""

    def _row(self, **overrides):
        row = {
            "id": "perf/x", "minted_at": "2026-07-06T00:00:00Z",
            "last_verified": None, "claim_vs_assertion_audit": VALID_AUDIT,
            "comment": "measures push round-trip latency against the backend",
        }
        row.update(overrides)
        return row

    def test_transport_row_minted_post_p90_without_coverage_kind_fails(self):
        with self.assertRaises(SystemExit) as ctx:
            _audit_field.validate_row(self._row(), "test.json", parse_rfc3339, "perf")
        self.assertIn("coverage_kind", str(ctx.exception))

    def test_transport_row_with_real_backend_coverage_kind_passes(self):
        row = self._row(coverage_kind="real-backend")
        _audit_field.validate_row(row, "test.json", parse_rfc3339, "perf")  # no raise

    def test_legacy_transport_row_without_coverage_kind_passes(self):
        # No minted_at + pre-cutoff lv -> legacy -> RAISE-only, not blocked.
        row = self._row(minted_at=None, last_verified="2026-04-01T00:00:00Z")
        _audit_field.validate_row(row, "test.json", parse_rfc3339, "perf")  # no raise

    def test_non_transport_row_minted_post_p90_passes(self):
        row = self._row(comment="mechanical check that a file exists on disk")
        _audit_field.validate_row(row, "test.json", parse_rfc3339, "structure")  # no raise

    def test_explicit_transport_claim_true_requires_coverage_kind(self):
        # transport_claim: true honored even with no regex hit in the corpus.
        row = self._row(comment="does a thing", transport_claim=True)
        with self.assertRaises(SystemExit):
            _audit_field.validate_row(row, "test.json", parse_rfc3339, "agent-ux")

    def test_transport_claim_false_suppresses_regex(self):
        # P90-D1 tri-state: explicit opt-out beats the regex hit on 'push'.
        row = self._row(transport_claim=False)
        _audit_field.validate_row(row, "test.json", parse_rfc3339, "perf")  # no raise

    def test_compliant_waiver_satisfies_coverage_kind(self):
        row = self._row(waiver={"until": "2099-01-01T00:00:00Z", "reason": "tracked"})
        _audit_field.validate_row(row, "test.json", parse_rfc3339, "perf")  # no raise

    def test_invalid_coverage_kind_enum_fails(self):
        row = self._row(coverage_kind="bogus")
        with self.assertRaises(SystemExit) as ctx:
            _audit_field.validate_row(row, "test.json", parse_rfc3339, "perf")
        self.assertIn("coverage_kind", str(ctx.exception))


class TestIsTransportOrPerfRow(unittest.TestCase):
    def test_regex_gated_to_transport_dims(self):
        row = {"id": "docs/x", "comment": "the push button in the UI docs"}
        self.assertFalse(_audit_field.is_transport_or_perf_row(row, "docs-alignment"))
        self.assertTrue(_audit_field.is_transport_or_perf_row(row, "agent-ux"))

    def test_regex_over_comment_and_id_not_asserts(self):
        # R-3: expected.asserts is NOT part of the corpus (meta-row self-match).
        row = {"id": "structure/x", "comment": "a plain mechanical file check",
               "expected": {"asserts": ["the runner detects push/fetch transport rows"]}}
        self.assertFalse(_audit_field.is_transport_or_perf_row(row, "perf"))


class TestAssertsCongruence(unittest.TestCase):
    """F-K4b / ROADMAP SC2: per-expected-assert congruence."""

    # The p86 F6 shape: 9 expected vs 17 passed, heavy shared DVCS-bus
    # vocabulary. 7 expected are each strongly covered by a single passed
    # entry; 2 ("uncovered") have their significant tokens scattered ONE-each
    # across many passed entries -- present in the union (a zero-global-overlap
    # strawman would grade them PASS) but concentrated in no single entry, so
    # per-pair matching correctly flags them.
    P86_PASSED = [
        "bus url mirror form taught in bus_url reject hints",
        "refs mirrors tag written on successful sync",
        "git remote add mirror configuration hint emitted by helper",
        "blob limit sparse checkout teaching string present in helper",
        "conflict rebase teaching string present in push helper",
        "reposix attach subcommand listed in help output",
        "attach documents orphan policy flag in help",
        "attach documents delete local enum value",
        "attach documents fork as new enum value",
        "attach documents abort enum value",
        "cache built at configured directory for project",
        "extensions partialClone equals reposix after attach",
        "remote reposix url contains canonical bus form",
        "origin remote url unchanged by attach operation",
        "reconciliation report matched counts emitted to stdout",
        "audit events cache contains attach walk row",
        "wire path round trip covered by happy test function",
    ]
    P86_UNCOVERED = [
        "the ref is taught with a size limit and a configuration",
        "the sync must reject an add after checkout",
    ]
    P86_EXPECTED = [
        "bus url mirror form taught in bus_url reject hints",
        "refs mirrors tag written on sync",
        "reposix attach subcommand listed in help",
        "extensions partialClone equals reposix after attach",
        "audit events cache contains attach walk row",
        "wire path round trip covered by happy test function",
        "blob limit sparse checkout teaching string present",
    ] + P86_UNCOVERED

    def test_p86_f6_regression_goes_red(self):
        ok, unmatched = _audit_field.asserts_congruent(self.P86_EXPECTED, self.P86_PASSED)
        self.assertFalse(ok)
        self.assertEqual(len(unmatched), 2)
        self.assertEqual(set(unmatched), set(self.P86_UNCOVERED))

    def test_p86_uncovered_have_high_global_overlap(self):
        # Guards the regression's teeth: the 2 uncovered asserts must have
        # NON-zero global-union overlap, else a zero-global-overlap check would
        # also catch them and the fixture would not demonstrate per-pair value.
        union = set()
        for p in self.P86_PASSED:
            union |= _audit_field._sig_tokens(p)
        for e in self.P86_UNCOVERED:
            et = _audit_field._sig_tokens(e)
            best_single = max(len(et & _audit_field._sig_tokens(p)) for p in self.P86_PASSED)
            self.assertGreaterEqual(len(et & union), 3)  # globally present
            self.assertLessEqual(best_single, 1)          # per-pair uncovered

    def test_every_expected_covered_passes(self):
        expected = self.P86_EXPECTED[:7]  # the 7 covered ones
        ok, unmatched = _audit_field.asserts_congruent(expected, self.P86_PASSED)
        self.assertTrue(ok)
        self.assertEqual(unmatched, [])

    def test_empty_asserts_passed_is_noop(self):
        ok, unmatched = _audit_field.asserts_congruent(["anything at all here"], [])
        self.assertTrue(ok)
        self.assertEqual(unmatched, [])

    def test_empty_expected_is_noop(self):
        ok, unmatched = _audit_field.asserts_congruent([], ["some passed assert"])
        self.assertTrue(ok)

    def test_single_unmatched_named_exactly(self):
        expected = ["cache built at directory for project",
                    "totally unrelated zebra xylophone quasar nebula"]
        passed = ["cache built at configured directory for project",
                  "another passed entry about directories"]
        ok, unmatched = _audit_field.asserts_congruent(expected, passed)
        self.assertFalse(ok)
        self.assertEqual(unmatched, ["totally unrelated zebra xylophone quasar nebula"])


class TestTranscriptEvidenceOk(unittest.TestCase):
    """RBF-FW-08 runtime half."""

    def test_missing_transcript_path_fails(self):
        ok, why = _audit_field.transcript_evidence_ok({}, {}, Path("/tmp"))
        self.assertFalse(ok)
        self.assertIn("no transcript_path", why)

    def test_missing_file_fails(self):
        import tempfile
        with tempfile.TemporaryDirectory() as td:
            ok, why = _audit_field.transcript_evidence_ok(
                {}, {"transcript_path": "nope.txt"}, Path(td))
            self.assertFalse(ok)
            self.assertIn("missing", why)

    def test_no_argv_line_fails(self):
        import tempfile
        with tempfile.TemporaryDirectory() as td:
            (Path(td) / "t.txt").write_text("cwd: /x\nexit_code: 0\n")
            ok, why = _audit_field.transcript_evidence_ok(
                {}, {"transcript_path": "t.txt"}, Path(td))
            self.assertFalse(ok)
            self.assertIn("argv", why)

    def test_valid_transcript_passes(self):
        import tempfile
        with tempfile.TemporaryDirectory() as td:
            (Path(td) / "t.txt").write_text("argv: /usr/bin/reposix --version\nexit_code: 0\n")
            ok, why = _audit_field.transcript_evidence_ok(
                {}, {"transcript_path": "t.txt"}, Path(td))
            self.assertTrue(ok)
            self.assertEqual(why, "")

    def test_falls_back_to_row_declared_path(self):
        import tempfile
        with tempfile.TemporaryDirectory() as td:
            (Path(td) / "t.txt").write_text("argv: /bin/bash --version\n")
            row = {"expected": {"artifact": {"transcript_path": "t.txt"}}}
            ok, _ = _audit_field.transcript_evidence_ok(row, {}, Path(td))
            self.assertTrue(ok)


if __name__ == "__main__":
    unittest.main()
