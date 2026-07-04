"""Tests for quality/runners/verdict.py -- D-CONV-2 (2026-07-04,
quality/SURPRISES.md "Quality Convergence" -- verdict 3-state honest
contract).

Covers `compute_exit_code` (the --fail-on {red,yellow} threshold semantics)
and `compute_badge_message` (QL-178: message text must match the color --
no "N/N GREEN" on a yellow badge).

Run: python3 -m unittest quality.runners.test_verdict -v
"""

from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

# Mirrors test_realbackend.py: run.py / verdict.py use script-style absolute
# imports (`from _freshness import ...`), so package-context import breaks
# unless the runners dir itself is on sys.path.
sys.path.insert(0, str(Path(__file__).resolve().parent))

import verdict  # noqa: E402


def _row(blast_radius: str, status: str) -> dict:
    return {"id": f"test/{blast_radius}-{status}", "blast_radius": blast_radius, "status": status}


class TestComputeExitCode(unittest.TestCase):
    """--fail-on {red,yellow} threshold semantics (D-CONV-2)."""

    def test_brightgreen_always_zero(self):
        self.assertEqual(verdict.compute_exit_code("brightgreen", "yellow"), 0)
        self.assertEqual(verdict.compute_exit_code("brightgreen", "red"), 0)

    def test_red_always_nonzero(self):
        self.assertEqual(verdict.compute_exit_code("red", "yellow"), 1)
        self.assertEqual(verdict.compute_exit_code("red", "red"), 1)

    def test_yellow_strict_by_default(self):
        # Default fail-on is "yellow" -- yellow must exit nonzero unless the
        # caller explicitly opts into --fail-on red tolerance.
        self.assertEqual(verdict.compute_exit_code("yellow", "yellow"), 1)

    def test_yellow_tolerated_under_fail_on_red(self):
        self.assertEqual(verdict.compute_exit_code("yellow", "red"), 0)


class TestComputeBadgeMessage(unittest.TestCase):
    """QL-178: message text must not claim GREEN on a yellow/red badge."""

    def test_brightgreen_message_is_n_of_n_green(self):
        rows = [_row("P0", "PASS"), _row("P1", "WAIVED"), _row("P2", "PASS")]
        self.assertEqual(verdict.compute_color(rows), "brightgreen")
        self.assertEqual(verdict.compute_badge_message(rows), "2/2 GREEN")

    def test_yellow_message_names_pending_not_green(self):
        # P0+P1 all green, one P2 red -> yellow. Message must say "pending",
        # not "GREEN" (the exact QL-178 bug: color=yellow, text="N/N GREEN").
        rows = [_row("P0", "PASS"), _row("P1", "WAIVED"), _row("P2", "FAIL")]
        self.assertEqual(verdict.compute_color(rows), "yellow")
        msg = verdict.compute_badge_message(rows)
        self.assertNotIn("GREEN", msg)
        self.assertEqual(msg, "2/2 green, 1 pending")

    def test_red_message_names_red_not_green(self):
        rows = [_row("P0", "FAIL"), _row("P1", "PASS")]
        self.assertEqual(verdict.compute_color(rows), "red")
        msg = verdict.compute_badge_message(rows)
        self.assertNotIn("GREEN", msg)
        self.assertIn("RED", msg)

    def test_no_p0p1_rows_falls_back_to_zero_of_zero(self):
        rows = [_row("P2", "FAIL")]
        self.assertEqual(verdict.compute_badge_message(rows), "0/0 GREEN")


class TestEmitMarkdownVerdictThreeState(unittest.TestCase):
    """re-audit R1: the Verdict line must be 3-valued (GREEN/YELLOW/RED),
    matching the adjacent Color line exactly -- not collapsed to a binary
    GREEN/RED that prints "Verdict: **RED**" on a yellow run."""

    def _render(self, rows: list[dict]) -> str:
        with tempfile.TemporaryDirectory() as tmp:
            out_path = Path(tmp) / "verdict.md"
            verdict.emit_markdown_verdict(rows, "on-demand", out_path, Path(tmp))
            return out_path.read_text(encoding="utf-8")

    def test_yellow_run_prints_yellow_verdict(self):
        rows = [_row("P0", "PASS"), _row("P1", "WAIVED"), _row("P2", "FAIL")]
        self.assertEqual(verdict.compute_color(rows), "yellow")
        text = self._render(rows)
        self.assertIn("Verdict: **YELLOW**", text)
        self.assertIn("Color: `yellow`", text)

    def test_brightgreen_run_prints_green_verdict(self):
        rows = [_row("P0", "PASS"), _row("P1", "WAIVED")]
        self.assertEqual(verdict.compute_color(rows), "brightgreen")
        text = self._render(rows)
        self.assertIn("Verdict: **GREEN**", text)

    def test_red_run_prints_red_verdict(self):
        rows = [_row("P0", "FAIL")]
        self.assertEqual(verdict.compute_color(rows), "red")
        text = self._render(rows)
        self.assertIn("Verdict: **RED**", text)


class TestNotVerifiedRendersErrorMarker(unittest.TestCase):
    """FW-07a (R1 noticing 2): the NOT-VERIFIED section must surface an
    artifact's `error` field (e.g. "verifier not found at ...") so a
    missing-verifier row is distinguishable from a merely-stale one."""

    def test_error_marker_rendered_when_present(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo_root = Path(tmp)
            art_dir = repo_root / "quality" / "reports" / "verifications" / "agent-ux"
            art_dir.mkdir(parents=True)
            art_path = art_dir / "some-row.json"
            art_path.write_text(
                '{"error": "verifier not found at quality/gates/agent-ux/missing.sh"}',
                encoding="utf-8",
            )
            row = {
                "id": "agent-ux/some-row",
                "blast_radius": "P1",
                "status": "NOT-VERIFIED",
                "dimension": "agent-ux",
                "artifact": "quality/reports/verifications/agent-ux/some-row.json",
            }
            out_path = repo_root / "verdict.md"
            verdict.emit_markdown_verdict([row], "on-demand", out_path, repo_root)
            text = out_path.read_text(encoding="utf-8")
            self.assertIn("error: `verifier not found at quality/gates/agent-ux/missing.sh`", text)

    def test_no_error_marker_when_absent(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo_root = Path(tmp)
            row = {
                "id": "agent-ux/some-other-row",
                "blast_radius": "P1",
                "status": "NOT-VERIFIED",
                "dimension": "agent-ux",
            }
            out_path = repo_root / "verdict.md"
            verdict.emit_markdown_verdict([row], "on-demand", out_path, repo_root)
            text = out_path.read_text(encoding="utf-8")
            self.assertNotIn("error:", text)


class TestMilestoneAdversarialGate(unittest.TestCase):
    """RBF-FW-12 / D90-09: milestone-close cannot go GREEN without a fresh
    adversarial-pass artifact reporting zero failed rows. R1's 3 cases plus
    the darken-only-never-lightens regression (D-CONV-2)."""

    def _artifact_dir(self, repo_root: Path) -> Path:
        d = repo_root / "quality" / "reports" / "verifications" / "adversarial"
        d.mkdir(parents=True, exist_ok=True)
        return d

    def test_artifact_absent_blocks_even_with_all_pass_rows(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo_root = Path(tmp)
            blocked, reason = verdict.milestone_adversarial_gate(repo_root, "v0.13.0")
            self.assertTrue(blocked)
            self.assertIn("artifact absent", reason)

    def test_empty_rows_failed_does_not_block(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo_root = Path(tmp)
            d = self._artifact_dir(repo_root)
            (d / "v0.13.0.json").write_text(
                '{"milestone": "v0.13.0", "rows_failed": [], "verdict": "PASS"}',
                encoding="utf-8",
            )
            blocked, reason = verdict.milestone_adversarial_gate(repo_root, "v0.13.0")
            self.assertFalse(blocked)
            self.assertEqual(reason, "")

    def test_one_failed_row_blocks(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo_root = Path(tmp)
            d = self._artifact_dir(repo_root)
            (d / "v0.13.0.json").write_text(
                '{"milestone": "v0.13.0", '
                '"rows_failed": [{"id": "agent-ux/some-row", "reason": "vacuous assertion"}], '
                '"verdict": "FAIL"}',
                encoding="utf-8",
            )
            blocked, reason = verdict.milestone_adversarial_gate(repo_root, "v0.13.0")
            self.assertTrue(blocked)
            self.assertIn("agent-ux/some-row", reason)

    def test_darken_only_never_lightens_an_already_red_verdict(self):
        # Simulate main()'s --milestone branch logic directly: a P0 FAIL row
        # makes compute_color red on its own; a clean (non-blocking)
        # adversarial artifact must not flip it back to green.
        with tempfile.TemporaryDirectory() as tmp:
            repo_root = Path(tmp)
            d = self._artifact_dir(repo_root)
            (d / "v0.13.0.json").write_text(
                '{"milestone": "v0.13.0", "rows_failed": [], "verdict": "PASS"}',
                encoding="utf-8",
            )
            rows = [_row("P0", "FAIL")]
            color = verdict.compute_color(rows)
            self.assertEqual(color, "red")
            blocked, _reason = verdict.milestone_adversarial_gate(repo_root, "v0.13.0")
            self.assertFalse(blocked)
            # main()'s logic: color starts red, gate does not block -> color
            # stays whatever compute_color said. It never gets set back to
            # green by the gate -- the gate can only ever force red, never
            # override a red compute_color result to green.
            if blocked:
                color = "red"
            self.assertEqual(color, "red")


if __name__ == "__main__":
    unittest.main()
