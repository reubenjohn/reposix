"""Synthetic STALE regression for the runner. P61 SUBJ-03.

Exercises the runner end-to-end: backdates the headline-numbers row's
last_verified to far in the past, strips its waiver, runs the weekly
runner subprocess, and asserts the [STALE] label appears in stdout.
Restores the original catalog via a fixture so re-runs are idempotent.

Run:
    pytest quality/runners/test_freshness_synth.py -v
"""

from __future__ import annotations

import json
import shutil
import subprocess
import sys
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
CATALOG = REPO_ROOT / "quality" / "catalogs" / "subjective-rubrics.json"


@pytest.fixture
def backup_catalog(tmp_path):
    backup = tmp_path / "subjective-rubrics.json.backup"
    shutil.copy2(CATALOG, backup)
    yield backup
    shutil.copy2(backup, CATALOG)


def test_stale_label_appears_in_runner_output(backup_catalog) -> None:
    # Backdate the headline-numbers row + strip its waiver.
    data = json.loads(CATALOG.read_text(encoding="utf-8"))
    for r in data["rows"]:
        if r["id"] == "subjective/headline-numbers-sanity":
            r["last_verified"] = "2026-01-01T00:00:00Z"
            r["waiver"] = None
    CATALOG.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")

    # Run the weekly runner; expect [STALE] in stdout.
    result = subprocess.run(
        [sys.executable, str(REPO_ROOT / "quality" / "runners" / "run.py"), "--cadence", "weekly"],
        cwd=str(REPO_ROOT),
        capture_output=True,
        text=True,
        check=False,
        timeout=60,
    )
    combined = result.stdout + result.stderr
    assert "[STALE" in combined, (
        f"expected [STALE...] label in runner output; got:\n{combined}"
    )
    # P2 row -> NOT-VERIFIED -> exit 0 (per existing compute_exit_code logic).
    assert result.returncode == 0, (
        f"weekly runner should exit 0 for a STALE P2 row; got {result.returncode}\n{combined}"
    )
