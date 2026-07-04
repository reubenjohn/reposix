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


if __name__ == "__main__":
    unittest.main()
