#!/usr/bin/env python3
"""Quality Gates structure-dimension verifier — freshness invariants.

Per .planning/research/v0.12.0-naming-and-architecture.md § "Per-dimension catalog files"
+ quality/catalogs/freshness-invariants.json. Stdlib only.

Each verify_<slug>(row, repo_root) function:
1. Performs the check named in row.expected.asserts.
2. Writes the artifact JSON with asserts_passed + asserts_failed populated.
3. Returns exit code: 0 PASS, 1 FAIL, 2 PARTIAL.

Dispatch is via --row-id; the runner invokes
  python3 quality/gates/structure/freshness-invariants.py --row-id <row.id>

Anti-bloat: this file holds the structure dimension's verifiers. New
dimensions get their own quality/gates/<dim>/<verifier>.py.
"""

from __future__ import annotations

import argparse
import json
import pathlib
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parent.parent.parent.parent
CATALOG_PATH = REPO_ROOT / "quality" / "catalogs" / "freshness-invariants.json"


def now_iso() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def write_artifact(path: Path, data: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def load_row(row_id: str) -> dict:
    data = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
    for r in data["rows"]:
        if r["id"] == row_id:
            return r
    raise SystemExit(f"FAIL: row {row_id!r} not found in {CATALOG_PATH}")


def bash_check(cmd: str, repo_root: Path) -> tuple[int, str, str]:
    """Run `bash -c cmd` from repo_root; return (exit_code, stdout, stderr)."""
    result = subprocess.run(
        ["bash", "-c", cmd],
        capture_output=True,
        text=True,
        cwd=str(repo_root),
        check=False,
    )
    return result.returncode, result.stdout, result.stderr


def make_artifact(row: dict, exit_code: int, asserts_passed: list[str], asserts_failed: list[str], **extra: Any) -> dict:
    art: dict[str, Any] = {
        "ts": now_iso(),
        "row_id": row["id"],
        "exit_code": exit_code,
        "stdout": "",
        "stderr": "",
        "asserts_passed": asserts_passed,
        "asserts_failed": asserts_failed,
    }
    art.update(extra)
    return art


# ---- Verifier functions -----------------------------------------------------


def verify_no_version_pinned_filenames(row: dict, repo_root: Path) -> int:
    cmd = (
        "find docs scripts -type f "
        "| grep -E 'v[0-9]+\\.[0-9]+\\.[0-9]+' "
        "| grep -v CHANGELOG || true"
    )
    _ec, stdout, _ = bash_check(cmd, repo_root)
    asserts_passed: list[str] = []
    asserts_failed: list[str] = []
    if stdout.strip():
        asserts_failed.append(
            "find docs scripts -type f | grep version-pinned filenames returned: "
            + stdout.strip()
        )
    else:
        asserts_passed.append(
            "find docs scripts -type f | grep -E 'v[0-9]+\\.[0-9]+\\.[0-9]+' | grep -v CHANGELOG returned empty stdout"
        )
    artifact = make_artifact(row, 1 if asserts_failed else 0, asserts_passed, asserts_failed)
    write_artifact(repo_root / row["artifact"], artifact)
    return artifact["exit_code"]


_PKG_MGR_RE = re.compile(r"(?:brew install|cargo binstall|curl[^\n]*\| ?sh|powershell[^\n]*irm)", re.IGNORECASE | re.MULTILINE)
_SOURCE_COMPILE_RE = re.compile(r"git clone https?://|cargo build --release", re.IGNORECASE | re.MULTILINE)


def _verify_install_leads(row: dict, repo_root: Path, target_rel: str) -> int:
    asserts_passed: list[str] = []
    asserts_failed: list[str] = []
    target = repo_root / target_rel
    if not target.exists():
        asserts_failed.append(f"target file not found: {target_rel}")
    else:
        text = target.read_text(encoding="utf-8")
        pm = _PKG_MGR_RE.search(text)
        src = list(_SOURCE_COMPILE_RE.finditer(text))
        first_src = min((m.start() for m in src), default=None)
        if pm is None:
            asserts_failed.append(
                f"no pkg-mgr command (brew/binstall/curl|sh/powershell-irm) found in {target_rel}"
            )
        elif first_src is not None and pm.start() >= first_src:
            asserts_failed.append(
                f"pkg-mgr command at offset {pm.start()} appears AFTER source-compile snippet at offset {first_src} in {target_rel}"
            )
        else:
            asserts_passed.append(
                f"pkg-mgr command at offset {pm.start()} appears BEFORE source-compile in {target_rel}"
            )
    artifact = make_artifact(row, 1 if asserts_failed else 0, asserts_passed, asserts_failed)
    write_artifact(repo_root / row["artifact"], artifact)
    return artifact["exit_code"]


def verify_install_leads_with_pkg_mgr_docs_index(row: dict, repo_root: Path) -> int:
    return _verify_install_leads(row, repo_root, "docs/index.md")


def verify_install_leads_with_pkg_mgr_readme(row: dict, repo_root: Path) -> int:
    return _verify_install_leads(row, repo_root, "README.md")


def verify_benchmarks_in_mkdocs_nav(row: dict, repo_root: Path) -> int:
    asserts_passed: list[str] = []
    asserts_failed: list[str] = []
    bench_dir = repo_root / "docs" / "benchmarks"
    mkdocs = repo_root / "mkdocs.yml"
    if not bench_dir.is_dir():
        asserts_passed.append("docs/benchmarks/ does not exist; nothing to nav-check")
        artifact = make_artifact(row, 0, asserts_passed, asserts_failed)
        write_artifact(repo_root / row["artifact"], artifact)
        return 0
    nav_text = mkdocs.read_text(encoding="utf-8") if mkdocs.exists() else ""
    missing = [p.name for p in sorted(bench_dir.glob("*.md")) if p.name not in nav_text]
    if missing:
        asserts_failed.append(f"benchmark pages missing from mkdocs.yml: {missing}")
    else:
        asserts_passed.append("every docs/benchmarks/*.md filename appears in mkdocs.yml")
    artifact = make_artifact(row, 1 if asserts_failed else 0, asserts_passed, asserts_failed)
    write_artifact(repo_root / row["artifact"], artifact)
    return artifact["exit_code"]


def verify_no_loose_roadmap_or_requirements(row: dict, repo_root: Path) -> int:
    cmd = (
        "find .planning/milestones -maxdepth 2 "
        "\\( -name '*ROADMAP*' -o -name '*REQUIREMENTS*' \\) "
        "| grep -v phases | grep -v archive || true"
    )
    _ec, stdout, _ = bash_check(cmd, repo_root)
    asserts_passed: list[str] = []
    asserts_failed: list[str] = []
    if stdout.strip():
        asserts_failed.append(
            "loose ROADMAP/REQUIREMENTS files found at .planning/milestones/ top-level: "
            + stdout.strip()
        )
    else:
        asserts_passed.append(
            "find .planning/milestones -maxdepth 2 ( -name '*ROADMAP*' -o -name '*REQUIREMENTS*' ) | grep -v phases | grep -v archive returned empty stdout"
        )
    artifact = make_artifact(row, 1 if asserts_failed else 0, asserts_passed, asserts_failed)
    write_artifact(repo_root / row["artifact"], artifact)
    return artifact["exit_code"]


def verify_no_orphan_docs(row: dict, repo_root: Path) -> int:
    """Run scripts/check-docs-site.sh and grade by exit code."""
    asserts_passed: list[str] = []
    asserts_failed: list[str] = []
    script = repo_root / "scripts" / "check-docs-site.sh"
    if not script.exists():
        asserts_failed.append(f"verifier prereq missing: {script}")
        artifact = make_artifact(row, 1, asserts_passed, asserts_failed)
        write_artifact(repo_root / row["artifact"], artifact)
        return 1
    result = subprocess.run(
        ["bash", str(script)],
        capture_output=True, text=True, cwd=str(repo_root),
        timeout=60, check=False,
    )
    if result.returncode == 0:
        asserts_passed.append("bash scripts/check-docs-site.sh exits 0 (mkdocs --strict + orphan-doc check)")
    else:
        asserts_failed.append(
            f"bash scripts/check-docs-site.sh exited {result.returncode}: "
            + (result.stderr.strip()[:400] or result.stdout.strip()[:400])
        )
    artifact = make_artifact(
        row, 1 if asserts_failed else 0, asserts_passed, asserts_failed,
        stdout=result.stdout[:1000], stderr=result.stderr[:1000],
    )
    write_artifact(repo_root / row["artifact"], artifact)
    return artifact["exit_code"]


_HISTORICAL_HEADING_RE = re.compile(r"^## v0\.(?:8|9|10|11)\.[0-9]+", re.MULTILINE)
_HISTORICAL_HEADING_WITH_SPACE_RE = re.compile(r"^## v0\.(?:8|9|10|11)\.[0-9]+ ", re.MULTILINE)


def verify_top_level_requirements_roadmap_scope(row: dict, repo_root: Path) -> int:
    asserts_passed: list[str] = []
    asserts_failed: list[str] = []
    # Assert 1 — REQUIREMENTS.md has no historical H2 sections
    req_path = repo_root / ".planning" / "REQUIREMENTS.md"
    if not req_path.exists():
        asserts_failed.append(f"missing: {req_path}")
    else:
        req_text = req_path.read_text(encoding="utf-8")
        historical_req = _HISTORICAL_HEADING_RE.findall(req_text)
        if historical_req:
            asserts_failed.append(
                f".planning/REQUIREMENTS.md contains historical milestone sections: {historical_req}. "
                f"Move them to .planning/milestones/v0.X.0-phases/REQUIREMENTS.md per CLAUDE.md §0.5."
            )
        else:
            asserts_passed.append(".planning/REQUIREMENTS.md scope is clean (no v0.8/9/10/11 H2 sections)")
    # Assert 2 — ROADMAP.md has no historical milestone H2 sections
    rm_path = repo_root / ".planning" / "ROADMAP.md"
    if not rm_path.exists():
        asserts_failed.append(f"missing: {rm_path}")
    else:
        rm_text = rm_path.read_text(encoding="utf-8")
        historical_rm = _HISTORICAL_HEADING_WITH_SPACE_RE.findall(rm_text)
        if historical_rm:
            asserts_failed.append(
                f".planning/ROADMAP.md contains historical milestone sections: {historical_rm}. "
                f"Move them to .planning/milestones/v0.X.0-phases/ROADMAP.md per CLAUDE.md §0.5."
            )
        else:
            asserts_passed.append(".planning/ROADMAP.md scope is clean (no historical milestone H2 sections)")
    asserts_passed.append("Historical milestones index paragraph is permitted in both files (informational only)")
    artifact = make_artifact(row, 1 if asserts_failed else 0, asserts_passed, asserts_failed)
    write_artifact(repo_root / row["artifact"], artifact)
    return artifact["exit_code"]


def verify_badges_resolve(row: dict, repo_root: Path) -> int:
    """P57 stub. Full HEAD-URL verifier ships in P60 per BADGE-01."""
    asserts_passed = ["P57 stub: row anchored in catalog; full verifier ships in P60 per BADGE-01 traceability"]
    asserts_failed: list[str] = []
    artifact = make_artifact(
        row, 0, asserts_passed, asserts_failed,
        note="P57 stub; full HEAD-URL verifier ships in P60",
    )
    write_artifact(repo_root / row["artifact"], artifact)
    return 0


# ---- P62 verifier branches --------------------------------------------------


def verify_no_loose_top_level_planning_audits(row: dict, repo_root: Path) -> int:
    """Assert no *MILESTONE-AUDIT*.md or SESSION-END-STATE* file at .planning/ top level."""
    asserts_passed: list[str] = []
    asserts_failed: list[str] = []
    planning_dir = repo_root / ".planning"
    offenders: list[str] = []
    if planning_dir.is_dir():
        for child in planning_dir.iterdir():
            if not child.is_file():
                continue
            name = child.name
            if "archive" in str(child):
                continue
            if re.search(r"MILESTONE-AUDIT.*\.md$", name) or name.startswith("SESSION-END-STATE"):
                offenders.append(str(child.relative_to(repo_root)))
    if offenders:
        asserts_failed.append(
            "loose milestone-audit / session-end-state files at .planning/ top level: "
            + ", ".join(offenders)
        )
    else:
        asserts_passed.append(
            "find .planning -maxdepth 1 -type f \\( -name '*MILESTONE-AUDIT*.md' -o -name 'SESSION-END-STATE*' \\) | grep -v archive returned empty"
        )
    artifact = make_artifact(row, 1 if asserts_failed else 0, asserts_passed, asserts_failed)
    write_artifact(repo_root / row["artifact"], artifact)
    return artifact["exit_code"]


def verify_no_pre_pivot_doc_stubs(row: dict, repo_root: Path) -> int:
    """Assert every docs/*.md stub <500B is in mkdocs.yml nav: OR not_in_nav: OR redirect_maps."""
    asserts_passed: list[str] = []
    asserts_failed: list[str] = []
    docs_dir = repo_root / "docs"
    mkdocs = repo_root / "mkdocs.yml"
    if not docs_dir.is_dir():
        asserts_passed.append("docs/ does not exist; nothing to check")
        artifact = make_artifact(row, 0, asserts_passed, asserts_failed)
        write_artifact(repo_root / row["artifact"], artifact)
        return 0
    mkdocs_text = mkdocs.read_text(encoding="utf-8") if mkdocs.exists() else ""
    unmapped: list[str] = []
    for stub in sorted(docs_dir.glob("*.md")):
        try:
            size = stub.stat().st_size
        except OSError:
            continue
        if size >= 500:
            continue
        # Stub must appear in nav: / not_in_nav: / redirect_maps section of mkdocs.yml
        # Simple substring search of the basename is sufficient for this guard.
        if stub.name not in mkdocs_text:
            unmapped.append(stub.name)
    if unmapped:
        asserts_failed.append(
            f"top-level docs/*.md stubs <500 bytes not in mkdocs.yml (nav / not_in_nav / redirect_maps): {unmapped}"
        )
    else:
        asserts_passed.append("every docs/*.md stub <500 bytes is referenced in mkdocs.yml")
    artifact = make_artifact(row, 1 if asserts_failed else 0, asserts_passed, asserts_failed)
    write_artifact(repo_root / row["artifact"], artifact)
    return artifact["exit_code"]


# ---- P64 verifier branches (docs-alignment dimension catalog) ---------------


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


def verify_repo_org_audit_artifact_present(row: dict, repo_root: Path) -> int:
    """Assert quality/reports/audits/repo-org-gaps.md exists + the consistency verifier passes."""
    asserts_passed: list[str] = []
    asserts_failed: list[str] = []
    audit = repo_root / "quality" / "reports" / "audits" / "repo-org-gaps.md"
    if not audit.exists():
        asserts_failed.append(f"missing audit artifact: {audit.relative_to(repo_root)}")
    else:
        text = audit.read_text(encoding="utf-8")
        if "# Repo organization gaps" not in text:
            asserts_failed.append("audit doc missing '# Repo organization gaps' heading")
        else:
            asserts_passed.append("audit doc exists with expected heading")
        # Run the consistency verifier as evidence
        check_script = repo_root / "scripts" / "check_repo_org_gaps.py"
        if check_script.exists():
            result = subprocess.run(
                ["python3", str(check_script)],
                capture_output=True, text=True, cwd=str(repo_root),
                timeout=30, check=False,
            )
            if result.returncode == 0:
                asserts_passed.append("scripts/check_repo_org_gaps.py exit 0 (audit consistency check passed)")
            else:
                asserts_failed.append(
                    f"scripts/check_repo_org_gaps.py exit {result.returncode}: "
                    + (result.stderr.strip()[:300] or result.stdout.strip()[:300])
                )
    artifact = make_artifact(row, 1 if asserts_failed else 0, asserts_passed, asserts_failed)
    write_artifact(repo_root / row["artifact"], artifact)
    return artifact["exit_code"]


# ---- Dispatch ---------------------------------------------------------------


DISPATCH = {
    "structure/no-version-pinned-filenames": verify_no_version_pinned_filenames,
    "structure/install-leads-with-pkg-mgr-docs-index": verify_install_leads_with_pkg_mgr_docs_index,
    "structure/install-leads-with-pkg-mgr-readme": verify_install_leads_with_pkg_mgr_readme,
    "structure/benchmarks-in-mkdocs-nav": verify_benchmarks_in_mkdocs_nav,
    "structure/no-loose-roadmap-or-requirements": verify_no_loose_roadmap_or_requirements,
    "structure/no-orphan-docs": verify_no_orphan_docs,
    "structure/top-level-requirements-roadmap-scope": verify_top_level_requirements_roadmap_scope,
    "structure/badges-resolve": verify_badges_resolve,
    "structure/no-loose-top-level-planning-audits": verify_no_loose_top_level_planning_audits,
    "structure/no-pre-pivot-doc-stubs": verify_no_pre_pivot_doc_stubs,
    "structure/repo-org-audit-artifact-present": verify_repo_org_audit_artifact_present,
    "structure/doc-alignment-catalog-present": verify_doc_alignment_catalog_present,
    "structure/doc-alignment-summary-block-valid": verify_doc_alignment_summary_block_valid,
    "structure/doc-alignment-floor-not-decreased": verify_doc_alignment_floor_not_decreased,
}


def main() -> int:
    parser = argparse.ArgumentParser(description="structure-dim freshness invariants verifier")
    parser.add_argument("--row-id", required=True)
    args = parser.parse_args()
    row = load_row(args.row_id)
    fn = DISPATCH.get(args.row_id)
    if fn is None:
        print(f"unknown row_id: {args.row_id}", file=sys.stderr)
        return 1
    return fn(row, REPO_ROOT)


if __name__ == "__main__":
    raise SystemExit(main())
