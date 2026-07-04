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


if __name__ == "__main__":
    unittest.main()
