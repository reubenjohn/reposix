"""P62 repo-org recurrence guards (Python regression nets).

The P78-02 path-forward shipped sibling .sh verifiers
(`no-loose-top-level-planning-audits.sh`, `no-pre-pivot-doc-stubs.sh`,
`repo-org-audit-artifact-present.sh`) as the catalog-bound entry points.
These Python branches remain dispatchable via `--row-id` for cross-check
and historical regression coverage.

Catalog rows:
- structure/no-loose-top-level-planning-audits
- structure/no-pre-pivot-doc-stubs
- structure/repo-org-audit-artifact-present
"""

from __future__ import annotations

import re
import subprocess
from pathlib import Path

from ._shared import make_artifact, write_artifact


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
