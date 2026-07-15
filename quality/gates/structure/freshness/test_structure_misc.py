"""quality/gates/structure/freshness/test_structure_misc.py -- structure dim tests.

Covers the historical-H2-heading detection used by
structure/top-level-requirements-roadmap-scope. Regression test for the bug
fixed 2026-07-14: the old regex hardcoded a `v0.(8|9|10|11)` allowlist and
silently missed every shipped/queued milestone from v0.12 onward (the exact
gap that let stale v0.13.0/v0.13.2 H2 blocks sit un-flagged in the live
.planning/ROADMAP.md). The fix reads the active milestone version from
.planning/STATE.md frontmatter and flags anything strictly below it, so no
future milestone bump requires touching this file again.

Run: python3 -m pytest quality/gates/structure/freshness/test_structure_misc.py -x -q
"""
from __future__ import annotations

import pathlib
import sys

import pytest

# `freshness` is a package (has __init__.py); structure_misc.py uses a
# relative `from ._shared import ...` import, so it must be imported through
# the package, not as a bare top-level module. Add the package's PARENT dir
# (quality/gates/structure/) to sys.path, matching freshness-invariants.py's
# own import convention.
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent.parent))
from freshness import structure_misc as sm  # noqa: E402


# ---------------------------------------------------------------------------
# _historical_headings: version-comparison logic (no hardcoded version list)
# ---------------------------------------------------------------------------


def test_flags_stale_block_above_old_hardcoded_ceiling() -> None:
    """v0.13.2 is above the old v0.8-v0.11 allowlist ceiling -- must still be
    flagged now that detection is version-comparison-based, not a fixed list."""
    text = (
        "## v0.15.0 Floor (PLANNING)\n\nactive content\n\n"
        "## v0.13.2 Cross-link fidelity (PLANNING)\n\nstale content\n"
    )
    flagged = sm._historical_headings(text, sm._HISTORICAL_HEADING_VERSION_WITH_SPACE_RE, (0, 15, 0))
    assert flagged == ["## v0.13.2"]


def test_does_not_flag_active_milestone_heading() -> None:
    """Critical safety: the active milestone's own H2 must never self-flag."""
    text = "## v0.15.0 Floor (PLANNING)\n\nactive content only\n"
    flagged = sm._historical_headings(text, sm._HISTORICAL_HEADING_VERSION_WITH_SPACE_RE, (0, 15, 0))
    assert flagged == []


def test_still_flags_old_hardcoded_floor_versions() -> None:
    """No regression: versions that used to be caught by the v0.8-11 allowlist
    are still caught by the version-comparison logic."""
    text = "## v0.11.0 Polish (PLANNING)\n"
    flagged = sm._historical_headings(text, sm._HISTORICAL_HEADING_VERSION_WITH_SPACE_RE, (0, 15, 0))
    assert flagged == ["## v0.11.0"]


def test_no_active_version_fails_closed_to_empty() -> None:
    """If the active version can't be determined, flag nothing rather than guess."""
    text = "## v0.13.2 Cross-link fidelity (PLANNING)\n"
    flagged = sm._historical_headings(text, sm._HISTORICAL_HEADING_VERSION_WITH_SPACE_RE, None)
    assert flagged == []


# ---------------------------------------------------------------------------
# _active_milestone_version: STATE.md frontmatter parsing
# ---------------------------------------------------------------------------


def test_active_milestone_version_parses_frontmatter(tmp_path: pathlib.Path) -> None:
    planning = tmp_path / ".planning"
    planning.mkdir()
    (planning / "STATE.md").write_text(
        "---\ngsd_state_version: 1.0\nmilestone: v0.15.0\nmilestone_name: Floor\n---\n",
        encoding="utf-8",
    )
    assert sm._active_milestone_version(tmp_path) == (0, 15, 0)


def test_active_milestone_version_none_when_state_missing(tmp_path: pathlib.Path) -> None:
    assert sm._active_milestone_version(tmp_path) is None


def test_active_milestone_version_none_when_unparseable(tmp_path: pathlib.Path) -> None:
    planning = tmp_path / ".planning"
    planning.mkdir()
    (planning / "STATE.md").write_text("---\nno_milestone_field: here\n---\n", encoding="utf-8")
    assert sm._active_milestone_version(tmp_path) is None


# ---------------------------------------------------------------------------
# verify_top_level_requirements_roadmap_scope: end-to-end against a fixture repo
# ---------------------------------------------------------------------------


def _make_fixture_repo(tmp_path: pathlib.Path, roadmap_extra: str) -> pathlib.Path:
    planning = tmp_path / ".planning"
    planning.mkdir()
    (planning / "STATE.md").write_text("---\nmilestone: v0.15.0\n---\n", encoding="utf-8")
    (planning / "REQUIREMENTS.md").write_text("## v0.15.0 Requirements\n\ncontent\n", encoding="utf-8")
    (planning / "ROADMAP.md").write_text(
        "## v0.15.0 Floor (PLANNING)\n\ncontent\n" + roadmap_extra, encoding="utf-8"
    )
    reports_dir = tmp_path / "quality" / "reports" / "verifications" / "structure"
    reports_dir.mkdir(parents=True)
    return tmp_path


def _row() -> dict:
    return {
        "id": "structure/top-level-requirements-roadmap-scope",
        "artifact": "quality/reports/verifications/structure/top-level-requirements-roadmap-scope.json",
    }


def test_verifier_passes_on_clean_active_only_roadmap(tmp_path: pathlib.Path) -> None:
    repo = _make_fixture_repo(tmp_path, roadmap_extra="")
    exit_code = sm.verify_top_level_requirements_roadmap_scope(_row(), repo)
    assert exit_code == 0


def test_verifier_fails_on_stale_ge_v012_block(tmp_path: pathlib.Path) -> None:
    """The exact bug this fix closes: a >=v0.12 stale block must BLOCK, not
    silently pass, the way it did before this fix shipped."""
    stale_block = "\n## v0.13.2 Cross-link fidelity (PLANNING — workstream B)\n\nstale\n"
    repo = _make_fixture_repo(tmp_path, roadmap_extra=stale_block)
    exit_code = sm.verify_top_level_requirements_roadmap_scope(_row(), repo)
    assert exit_code == 1
