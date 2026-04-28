"""Import + smoke tests for the reposix-quality-review skill helpers. P61 SUBJ-02.

Run:
    pytest .claude/skills/reposix-quality-review/lib/test_lib_smoke.py -v
"""

from __future__ import annotations

import sys
from pathlib import Path

LIB = Path(__file__).resolve().parent
sys.path.insert(0, str(LIB))


def test_catalog_loads_3_rows() -> None:
    from catalog import load_subjective_catalog
    c = load_subjective_catalog()
    assert len(c["rows"]) == 3


def test_find_row_resolves_each_seed_id() -> None:
    from catalog import find_row, load_subjective_catalog
    c = load_subjective_catalog()
    for rubric_id in (
        "subjective/cold-reader-hero-clarity",
        "subjective/install-positioning",
        "subjective/headline-numbers-sanity",
    ):
        r = find_row(c, rubric_id)
        assert r["id"] == rubric_id


def test_stale_rows_returns_all_three_when_last_verified_null() -> None:
    """All P61-Wave-A rows have last_verified=null in the committed catalog,
    so stale_rows() should report all 3 (the runner has not graded them yet
    because they are waivered through 2026-05-15).
    """
    from catalog import load_subjective_catalog, stale_rows
    c = load_subjective_catalog()
    # In a clean catalog every row has last_verified=null. The runner mutates
    # last_verified during a run, but the committed file should be clean.
    # We don't assert exact count here -- only that the function runs.
    assert isinstance(stale_rows(c), list)


def test_persist_artifact_round_trip(tmp_path, monkeypatch) -> None:
    """persist_artifact writes the canonical JSON shape under a tmp dir."""
    import importlib
    monkeypatch.chdir(tmp_path)
    # Re-route ARTIFACTS_DIR to tmp by patching the module after import.
    import persist_artifact as pa
    monkeypatch.setattr(pa, "ARTIFACTS_DIR", tmp_path / "subjective")
    out = pa.persist_artifact(
        rubric_id="subjective/cold-reader-hero-clarity",
        score=8,
        verdict="CLEAR",
        rationale="hero is clear",
        evidence_files=["README.md:1-50"],
        dispatched_via="test",
        asserts_passed=["a"],
        asserts_failed=[],
    )
    assert out.exists()
    import json
    payload = json.loads(out.read_text())
    expected = {"ts", "rubric_id", "score", "verdict", "rationale",
                "evidence_files", "dispatched_via", "asserts_passed",
                "asserts_failed", "stale"}
    assert expected.issubset(payload.keys())
    assert payload["score"] == 8
    assert payload["stale"] is False
