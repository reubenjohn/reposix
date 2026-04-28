"""Artifact-write helper for the reposix-quality-review skill. P61 SUBJ-02.

Writes the subjective-rubric verdict JSON to
quality/reports/verifications/subjective/<slug>.json. Does NOT update the
catalog row's status -- the next runner sweep is the single writer for
status (matches quality/runners/run.py:run_row semantics).

Stdlib-only.
"""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[4]
ARTIFACTS_DIR = REPO_ROOT / "quality" / "reports" / "verifications" / "subjective"


def now_iso() -> str:
    """RFC3339 UTC timestamp with Z suffix (matches runner now_iso)."""
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def slug_for(rubric_id: str) -> str:
    """Convert 'subjective/<slug>' to '<slug>' for the filename."""
    if "/" not in rubric_id:
        return rubric_id
    return rubric_id.split("/", 1)[1]


def persist_artifact(
    rubric_id: str,
    score: int,
    verdict: str,
    rationale: str,
    evidence_files: list,
    dispatched_via: str,
    asserts_passed: list,
    asserts_failed: list,
) -> Path:
    """Write the rubric verdict artifact. Returns the absolute artifact path."""
    artifact_path = ARTIFACTS_DIR / f"{slug_for(rubric_id)}.json"
    artifact_path.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "ts": now_iso(),
        "rubric_id": rubric_id,
        "score": int(score),
        "verdict": verdict,
        "rationale": rationale,
        "evidence_files": list(evidence_files),
        "dispatched_via": dispatched_via,
        "asserts_passed": list(asserts_passed),
        "asserts_failed": list(asserts_failed),
        "stale": False,
    }
    artifact_path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return artifact_path
