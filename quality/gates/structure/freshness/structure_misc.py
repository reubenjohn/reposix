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

_HISTORICAL_HEADING_VERSION_RE = re.compile(r"^## v(\d+)\.(\d+)\.(\d+)", re.MULTILINE)
_HISTORICAL_HEADING_VERSION_WITH_SPACE_RE = re.compile(r"^## v(\d+)\.(\d+)\.(\d+) ", re.MULTILINE)
_ACTIVE_MILESTONE_RE = re.compile(r"^milestone:\s*v(\d+)\.(\d+)\.(\d+)\s*$", re.MULTILINE)


def _active_milestone_version(repo_root: Path) -> tuple[int, int, int] | None:
    """Parse the active milestone version from `.planning/STATE.md` frontmatter.

    Returns (major, minor, patch), or None if STATE.md is missing or its
    `milestone:` line doesn't parse. Callers must fail closed (flag nothing)
    on None rather than guess — a silent gate beats a false-positive block on
    the active milestone's own H2 heading.
    """
    state_path = repo_root / ".planning" / "STATE.md"
    try:
        text = state_path.read_text(encoding="utf-8")
    except OSError:
        return None
    match = _ACTIVE_MILESTONE_RE.search(text)
    if not match:
        return None
    return (int(match.group(1)), int(match.group(2)), int(match.group(3)))


def _historical_headings(
    text: str, pattern: re.Pattern[str], active: tuple[int, int, int] | None
) -> list[str]:
    """Return H2 heading-start strings whose version is strictly below `active`.

    Self-maintaining across milestones: no hardcoded version list to go stale
    (the bug this replaces — a `v0.(8|9|10|11)` allowlist silently missed
    every shipped/queued milestone from v0.12 onward). If `active` is None
    (STATE.md unreadable/unparseable), returns [] — fail closed, never flag.
    """
    if active is None:
        return []
    out: list[str] = []
    for m in pattern.finditer(text):
        version = (int(m.group(1)), int(m.group(2)), int(m.group(3)))
        if version < active:
            out.append(m.group(0).rstrip())
    return out


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
    """Run quality/gates/docs-build/mkdocs-strict.sh and grade by exit code.

    D-CONV-3 (2026-07-04): scripts/check-docs-site.sh (the shim this
    verifier used to shell out to) was deleted; call the canonical path
    directly.
    """
    asserts_passed: list[str] = []
    asserts_failed: list[str] = []
    script = repo_root / "quality" / "gates" / "docs-build" / "mkdocs-strict.sh"
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
        asserts_passed.append("bash quality/gates/docs-build/mkdocs-strict.sh exits 0 (mkdocs --strict + orphan-doc check)")
    else:
        asserts_failed.append(
            f"bash quality/gates/docs-build/mkdocs-strict.sh exited {result.returncode}: "
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
    active = _active_milestone_version(repo_root)
    if active is None:
        asserts_failed.append(
            "could not determine active milestone version from "
            ".planning/STATE.md frontmatter `milestone:` field — "
            "historical-heading check skipped (fail closed, nothing flagged)"
        )
    else:
        active_str = ".".join(str(p) for p in active)
        # Assert 1 — REQUIREMENTS.md has no historical H2 sections
        req_path = repo_root / ".planning" / "REQUIREMENTS.md"
        if not req_path.exists():
            asserts_failed.append(f"missing: {req_path}")
        else:
            req_text = req_path.read_text(encoding="utf-8")
            historical_req = _historical_headings(req_text, _HISTORICAL_HEADING_VERSION_RE, active)
            if historical_req:
                asserts_failed.append(
                    f".planning/REQUIREMENTS.md contains historical milestone sections: {historical_req}. "
                    f"Move them to .planning/milestones/v0.X.0-phases/REQUIREMENTS.md per CLAUDE.md §0.5."
                )
            else:
                asserts_passed.append(
                    f".planning/REQUIREMENTS.md scope is clean (no H2 section below active milestone v{active_str})"
                )
        # Assert 2 — ROADMAP.md has no historical milestone H2 sections
        rm_path = repo_root / ".planning" / "ROADMAP.md"
        if not rm_path.exists():
            asserts_failed.append(f"missing: {rm_path}")
        else:
            rm_text = rm_path.read_text(encoding="utf-8")
            historical_rm = _historical_headings(rm_text, _HISTORICAL_HEADING_VERSION_WITH_SPACE_RE, active)
            if historical_rm:
                asserts_failed.append(
                    f".planning/ROADMAP.md contains historical milestone sections: {historical_rm}. "
                    f"Move them to .planning/milestones/v0.X.0-phases/ROADMAP.md per CLAUDE.md §0.5."
                )
            else:
                asserts_passed.append(
                    f".planning/ROADMAP.md scope is clean (no H2 section below active milestone v{active_str})"
                )
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
