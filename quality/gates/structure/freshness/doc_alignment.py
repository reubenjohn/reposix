"""Doc-alignment catalog schema + floor-monotonicity guards.

Catalog rows:
- structure/doc-alignment-catalog-present
- structure/doc-alignment-summary-block-valid
- structure/doc-alignment-floor-not-decreased
"""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

from ._shared import make_artifact, write_artifact

_DOC_ALIGN_CATALOG_REL = "quality/catalogs/doc-alignment.json"
_DOC_ALIGN_SUMMARY_KEYS = (
    "claims_total",
    "claims_bound",
    "claims_missing_test",
    "claims_retire_proposed",
    "claims_retired",
    "alignment_ratio",
    "floor",
    "trend_30d",
    "last_walked",
)


def verify_doc_alignment_catalog_present(row: dict, repo_root: Path) -> int:
    """Assert quality/catalogs/doc-alignment.json exists, parses, and has required top-level keys."""
    asserts_passed: list[str] = []
    asserts_failed: list[str] = []
    catalog = repo_root / _DOC_ALIGN_CATALOG_REL
    if not catalog.exists():
        asserts_failed.append(f"missing catalog: {_DOC_ALIGN_CATALOG_REL}")
    else:
        try:
            data = json.loads(catalog.read_text(encoding="utf-8"))
        except json.JSONDecodeError as e:
            asserts_failed.append(f"json.loads failed for {_DOC_ALIGN_CATALOG_REL}: {e}")
            data = None
        if data is not None:
            asserts_passed.append(f"{_DOC_ALIGN_CATALOG_REL} exists and parses")
            for key in ("schema_version", "summary", "rows"):
                if key not in data:
                    asserts_failed.append(f"missing top-level key: {key}")
            # Accept "1.0" (P64 baseline) and "2.0" (W7/P71 tests Vec migration).
            # Future migrations bump this list deliberately; the verifier never
            # auto-tracks the live catalog.
            known_versions = ("1.0", "2.0")
            if data.get("schema_version") not in known_versions:
                asserts_failed.append(
                    f"schema_version is {data.get('schema_version')!r}; "
                    f"expected one of {known_versions}"
                )
            else:
                asserts_passed.append(
                    f"schema_version == {data.get('schema_version')!r}"
                )
    artifact = make_artifact(row, 1 if asserts_failed else 0, asserts_passed, asserts_failed)
    write_artifact(repo_root / row["artifact"], artifact)
    return artifact["exit_code"]


def verify_doc_alignment_summary_block_valid(row: dict, repo_root: Path) -> int:
    """Assert summary block has 9 required keys + alignment_ratio is recomputable from row counts."""
    asserts_passed: list[str] = []
    asserts_failed: list[str] = []
    catalog = repo_root / _DOC_ALIGN_CATALOG_REL
    if not catalog.exists():
        asserts_failed.append(f"missing catalog: {_DOC_ALIGN_CATALOG_REL}")
        artifact = make_artifact(row, 1, asserts_passed, asserts_failed)
        write_artifact(repo_root / row["artifact"], artifact)
        return 1
    try:
        data = json.loads(catalog.read_text(encoding="utf-8"))
    except json.JSONDecodeError as e:
        asserts_failed.append(f"json.loads failed: {e}")
        artifact = make_artifact(row, 1, asserts_passed, asserts_failed)
        write_artifact(repo_root / row["artifact"], artifact)
        return 1
    summary = data.get("summary")
    if not isinstance(summary, dict):
        asserts_failed.append("summary block missing or not a JSON object")
        artifact = make_artifact(row, 1, asserts_passed, asserts_failed)
        write_artifact(repo_root / row["artifact"], artifact)
        return 1
    missing_keys = [k for k in _DOC_ALIGN_SUMMARY_KEYS if k not in summary]
    if missing_keys:
        asserts_failed.append(f"summary block missing keys: {missing_keys}")
    else:
        asserts_passed.append(f"summary block has all 9 required keys: {list(_DOC_ALIGN_SUMMARY_KEYS)}")
    # Recompute alignment_ratio from row counts (definitional; protects against runner drift).
    claims_total = summary.get("claims_total", 0)
    claims_bound = summary.get("claims_bound", 0)
    claims_retired = summary.get("claims_retired", 0)
    denom = max(1, claims_total - claims_retired)
    expected_ratio = claims_bound / denom if claims_total > 0 else 1.0
    actual_ratio = summary.get("alignment_ratio")
    if not isinstance(actual_ratio, (int, float)):
        asserts_failed.append(f"alignment_ratio is not a number: {actual_ratio!r}")
    elif abs(float(actual_ratio) - expected_ratio) > 0.001:
        asserts_failed.append(
            f"alignment_ratio drift: stored={actual_ratio} vs recomputed={expected_ratio:.4f} "
            f"(claims_bound={claims_bound} / max(1, claims_total={claims_total} - claims_retired={claims_retired}))"
        )
    else:
        asserts_passed.append(
            f"alignment_ratio={actual_ratio} matches recomputed {expected_ratio:.4f} (within 0.001 epsilon)"
        )
    floor = summary.get("floor")
    if not isinstance(floor, (int, float)):
        asserts_failed.append(f"floor is not a number: {floor!r}")
    elif not (0.0 <= float(floor) <= 1.0):
        asserts_failed.append(f"floor out of [0.0, 1.0] range: {floor}")
    else:
        asserts_passed.append(f"floor={floor} is in [0.0, 1.0]")
    artifact = make_artifact(row, 1 if asserts_failed else 0, asserts_passed, asserts_failed)
    write_artifact(repo_root / row["artifact"], artifact)
    return artifact["exit_code"]


def verify_doc_alignment_floor_not_decreased(row: dict, repo_root: Path) -> int:
    """Walk git history of doc-alignment.json; assert the floor field never decreased between commits.

    On a freshly-seeded catalog with only one historical commit (or none), there is nothing
    to compare and the row passes with a note. Floor regressions name the offending SHA in
    asserts_failed for owner triage.
    """
    asserts_passed: list[str] = []
    asserts_failed: list[str] = []
    catalog_rel = _DOC_ALIGN_CATALOG_REL
    # Walk every commit that touched the file, oldest -> newest.
    log_cmd = ["git", "log", "--reverse", "--format=%H", "--", catalog_rel]
    result = subprocess.run(
        log_cmd, capture_output=True, text=True, cwd=str(repo_root), check=False, timeout=20,
    )
    if result.returncode != 0:
        asserts_failed.append(
            f"git log failed (exit {result.returncode}): {result.stderr.strip()[:200]}"
        )
        artifact = make_artifact(row, 1, asserts_passed, asserts_failed)
        write_artifact(repo_root / row["artifact"], artifact)
        return 1
    shas = [s for s in result.stdout.splitlines() if s.strip()]
    if len(shas) < 2:
        asserts_passed.append(
            f"history has {len(shas)} commit(s) touching {catalog_rel}; nothing to compare yet"
        )
        artifact = make_artifact(
            row, 0, asserts_passed, asserts_failed,
            commits_walked=len(shas),
        )
        write_artifact(repo_root / row["artifact"], artifact)
        return 0
    prev_floor: float | None = None
    prev_sha: str | None = None
    regressions: list[str] = []
    for sha in shas:
        show = subprocess.run(
            ["git", "show", f"{sha}:{catalog_rel}"],
            capture_output=True, text=True, cwd=str(repo_root), check=False, timeout=10,
        )
        if show.returncode != 0:
            # File may have been deleted/renamed in this commit; skip.
            continue
        try:
            data = json.loads(show.stdout)
        except json.JSONDecodeError:
            continue
        floor = data.get("summary", {}).get("floor")
        if not isinstance(floor, (int, float)):
            continue
        if prev_floor is not None and float(floor) < float(prev_floor) - 1e-9:
            regressions.append(
                f"commit {sha[:12]}: floor {prev_floor} -> {floor} (decrease from {prev_sha[:12] if prev_sha else '?'})"
            )
        prev_floor = float(floor)
        prev_sha = sha
    if regressions:
        asserts_failed.append("floor regressions detected: " + "; ".join(regressions))
    else:
        asserts_passed.append(
            f"floor monotone non-decreasing across {len(shas)} commits touching {catalog_rel}"
        )
    artifact = make_artifact(
        row, 1 if asserts_failed else 0, asserts_passed, asserts_failed,
        commits_walked=len(shas),
    )
    write_artifact(repo_root / row["artifact"], artifact)
    return artifact["exit_code"]
