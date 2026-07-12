"""Regression guard: the three fleet-safety verification JSONs stay UNTRACKED.

Context (2026-07-12, GSD quick): these per-run verdict artifacts are regenerated
on every agent-ux fleet-safety grade. Their content derives from live guard-scenario
ASSERT outcomes (exit_code / asserts_passed / asserts_failed), which can differ across
git-version / environment. When force-added they re-dirtied the CI checkout mid-run,
causing release-plz to refuse ("dirty working directory") and go persistently RED.

Fix mirrored the P102 precedent (fbe02c8): `git rm --cached` the three paths. They
remain on disk (per-run outputs) but are matched by `.gitignore` line 72
(`quality/reports/verifications/*/*.json`). Nothing reads the committed bytes as a
baseline — run.py treats the catalog `artifact` field as a pure write target.

This test prevents a future `git add -f` re-introducing the failure class: it asserts
each path is (a) NOT tracked (absent from `git ls-files`) and (b) matched by
`git check-ignore`.

Run: python3 -m unittest quality.runners.test_fleet_safety_verdicts_untracked -v
"""
from __future__ import annotations

import subprocess
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]

FLEET_SAFETY_VERDICTS = (
    "quality/reports/verifications/agent-ux/fleet-safety-leaf-isolation-enforce.json",
    "quality/reports/verifications/agent-ux/fleet-safety-shared-config-write-guard.json",
    "quality/reports/verifications/agent-ux/fleet-safety-tat-identity-reject.json",
)


def _git(*args: str) -> subprocess.CompletedProcess:
    return subprocess.run(
        ["git", *args],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        check=False,
    )


class FleetSafetyVerdictsUntracked(unittest.TestCase):
    def setUp(self) -> None:
        # Skip cleanly if we are not inside a git work tree (e.g. an exported tarball).
        rc = _git("rev-parse", "--is-inside-work-tree")
        if rc.returncode != 0 or rc.stdout.strip() != "true":
            self.skipTest("not inside a git work tree")

    def test_paths_not_tracked(self) -> None:
        """None of the three verdict JSONs may appear in `git ls-files`."""
        tracked = set(_git("ls-files").stdout.splitlines())
        for path in FLEET_SAFETY_VERDICTS:
            self.assertNotIn(
                path,
                tracked,
                msg=(
                    f"{path} is TRACKED again. It is a per-run verdict artifact that "
                    "re-dirties the CI checkout and breaks release-plz. Untrack it: "
                    f"`git rm --cached {path}` (see this test's module docstring)."
                ),
            )

    def test_paths_are_gitignored(self) -> None:
        """`git check-ignore` must match all three paths (belt to the ls-files brace)."""
        result = _git("check-ignore", *FLEET_SAFETY_VERDICTS)
        matched = set(result.stdout.splitlines())
        for path in FLEET_SAFETY_VERDICTS:
            self.assertIn(
                path,
                matched,
                msg=(
                    f"{path} is not matched by .gitignore. It must be ignored so grade-time "
                    "regeneration does not dirty the tree. Confirm the "
                    "`quality/reports/verifications/*/*.json` pattern (or an explicit line) "
                    "covers it and that no later `!` negation un-ignores it."
                ),
            )


if __name__ == "__main__":
    unittest.main()
