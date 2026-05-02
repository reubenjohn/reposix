"""Misc structure-dimension invariants.

Catalog rows:
- structure/no-version-pinned-filenames
- structure/benchmarks-in-mkdocs-nav
- structure/no-loose-roadmap-or-requirements
- structure/no-orphan-docs
- structure/top-level-requirements-roadmap-scope
- structure/badges-resolve  (P57 stub; P60 docs-build dim ships full verifier)
"""

from __future__ import annotations

import re
import subprocess
from pathlib import Path

from ._shared import bash_check, make_artifact, write_artifact

_HISTORICAL_HEADING_RE = re.compile(r"^## v0\.(?:8|9|10|11)\.[0-9]+", re.MULTILINE)
_HISTORICAL_HEADING_WITH_SPACE_RE = re.compile(r"^## v0\.(?:8|9|10|11)\.[0-9]+ ", re.MULTILINE)


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
