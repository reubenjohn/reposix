"""Synthetic STALE regression for the runner. P61 SUBJ-03.

Exercises the runner end-to-end: backdates the headline-numbers row's
last_verified to far in the past, strips its waiver, runs the weekly
runner subprocess, and asserts the [STALE] label appears in stdout.
Restores the original catalogs via a fixture so re-runs are idempotent.

**Hermeticity (Cycle-2 task (d), 2026-07-19).** `--cadence weekly` iterates
EVERY weekly-tagged row across EVERY catalog file, not just the target row
below. Two classes of noise this test never intended to exercise live
under that cadence:

1. `quality/catalogs/release-assets.json` alone carries 14 weekly rows
   (8x `release/crates-io-max-version/*`, `install/homebrew`,
   `release/brew-formula-current`, `release/gh-assets-present`,
   `install/curl-installer-sh`, `install/powershell-installer-ps1`,
   `install/build-from-source`) that all shell out to live HTTP
   (crates.io / GitHub API) via `urllib.request`. A flaky or sandboxed-
   offline network turns any of these P1 rows FAIL, which makes
   `compute_exit_code` return 1 -- breaking this test's
   `returncode == 0` assertion nondeterministically (the "stale-P2
   flake" from the PR#77 family).
2. `quality/catalogs/docs-reproducible.json` carries 2 weekly rows
   (`benchmark-claim/8ms-cached-read`,
   `benchmark-claim/89.1-percent-token-reduction`) with a deliberate
   catalog-first `verifier.script: null` stub. Because they carry no
   waiver, `run_row` always emits a `verifier not found at None` NOT-
   VERIFIED artifact and prints it to stdout on every weekly run --
   unrelated leakage this test's stdout scraping had no way to filter
   out.

`_neutralize_other_weekly_rows` gives every OTHER weekly-cadence row
(across every catalog) a temporary near-future waiver -- the runner's own
WAIVED short-circuit in `run_row`, which returns BEFORE the network-call
branch and BEFORE the verifier-missing branch (see `run.py::run_row`).
That makes the subprocess invocation below deterministic and fully
offline: only the one row this test backdates ever reaches a verifier (it
has none -- `subjective/headline-numbers-sanity` short-circuits via the
STALE freshness-TTL branch, also before any subprocess is spawned). Every
catalog touched is backed up before mutation and restored after, so
nothing survives past this test (see `backup_catalogs`).

Verified network-denied (2026-07-19, Cycle-2 task (d)):
`unshare -rn -- python3 -m pytest quality/runners/test_freshness_synth.py -v`
passes deterministically with the loopback-only network namespace up --
see `quality/CLAUDE.md` "Hermetic test convention" for the full
network-mock convention this test now follows.

Run:
    pytest quality/runners/test_freshness_synth.py -v
"""

from __future__ import annotations

import json
import shutil
import subprocess
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
CATALOG_DIR = REPO_ROOT / "quality" / "catalogs"
CATALOG = CATALOG_DIR / "subjective-rubrics.json"
TARGET_ROW_ID = "subjective/headline-numbers-sanity"


def _neutralize_other_weekly_rows(catalog_path: Path, now: datetime) -> bool:
    """Give every weekly-cadence row OTHER than TARGET_ROW_ID a temporary
    waiver so `run_row` short-circuits to WAIVED before it can reach a live
    network call or hit a null `verifier.script`. Returns True iff the file
    was modified (all callers back up every catalog regardless, so the
    return value is informational only)."""
    data = json.loads(catalog_path.read_text(encoding="utf-8"))
    until = (now + timedelta(hours=1)).strftime("%Y-%m-%dT%H:%M:%SZ")
    changed = False
    for row in data.get("rows", []):
        if row.get("id") == TARGET_ROW_ID:
            continue
        if "weekly" not in row.get("cadences", []):
            continue
        row["waiver"] = {
            "until": until,
            "reason": (
                "test_freshness_synth.py: neutralized for the duration of "
                "the hermetic single-row STALE-P2 assertion (no live "
                "network call, no null-verifier leakage) -- see module "
                "docstring."
            ),
            "dimension_owner": "quality/runners/test_freshness_synth.py",
            "tracked_in": "quality/CLAUDE.md#hermetic-test-convention",
        }
        changed = True
    if changed:
        catalog_path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    return changed


@pytest.fixture
def backup_catalogs(tmp_path):
    """Back up every catalog file under quality/catalogs/ before this test
    mutates any of them (target-row backdate + neutralization waivers on
    every other weekly row), and restore all of them afterward -- even on
    assertion failure -- so the mutation never survives the test."""
    backup_dir = tmp_path / "catalogs-backup"
    backup_dir.mkdir()
    originals = sorted(CATALOG_DIR.glob("*.json"))
    for path in originals:
        shutil.copy2(path, backup_dir / path.name)
    yield backup_dir
    for path in originals:
        shutil.copy2(backup_dir / path.name, path)


def test_stale_label_appears_in_runner_output(backup_catalogs) -> None:
    now = datetime.now(timezone.utc)

    # Backdate the headline-numbers row + strip its waiver.
    data = json.loads(CATALOG.read_text(encoding="utf-8"))
    for r in data["rows"]:
        if r["id"] == TARGET_ROW_ID:
            r["last_verified"] = "2026-01-01T00:00:00Z"
            r["waiver"] = None
    CATALOG.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")

    # Hermeticity: neutralize every OTHER weekly row across every catalog so
    # `--cadence weekly` below cannot reach a live network endpoint or a
    # null-verifier.script row. See module docstring.
    for catalog_path in sorted(CATALOG_DIR.glob("*.json")):
        _neutralize_other_weekly_rows(catalog_path, now)

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
    assert "verifier not found at None" not in combined, (
        f"null-verifier leakage should be neutralized by the weekly waiver; got:\n{combined}"
    )
    # P2 row -> NOT-VERIFIED -> exit 0 (per existing compute_exit_code logic).
    assert result.returncode == 0, (
        f"weekly runner should exit 0 for a STALE P2 row; got {result.returncode}\n{combined}"
    )
