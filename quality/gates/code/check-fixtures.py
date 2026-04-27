#!/usr/bin/env python3
"""Validate benchmark fixture files for shape, size, and content safety.

Migrated from scripts/check_fixtures.py per SIMPLIFY-05 (P58 Wave C, Option A:
code-dimension gate). Backs the code/fixtures-valid catalog row in
quality/catalogs/code.json.

Adjustments versus the original:
  - FIXTURES path is REPO_ROOT-relative (not cwd-relative) so the runner can
    invoke from any working directory.
  - Writes a JSON artifact at quality/reports/verifications/code/fixtures-valid.json
    with asserts_passed / asserts_failed for the runner to consume.
  - Backward-compatible stdout output (PASS / FAIL summary) preserved for
    interactive use.

Checks:
  - github_issues.json: JSON array, >=3 issues, required keys, 4-12 KB, no secrets.
  - confluence_pages.json: JSON object with results[], >=3 pages, required keys,
    ADF value re-parseable, 6-16 KB, no secrets.
  - fixtures/README.md: contains required terms.

Run from anywhere:
    python3 quality/gates/code/check-fixtures.py
"""

from __future__ import annotations

import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]
FIXTURES = REPO_ROOT / "benchmarks" / "fixtures"
ARTIFACT = REPO_ROOT / "quality" / "reports" / "verifications" / "code" / "fixtures-valid.json"

GITHUB_REQUIRED = {
    "id", "number", "title", "body", "state",
    "user", "labels", "created_at", "updated_at",
    "reactions", "author_association",
}

CONFLUENCE_REQUIRED = {
    "id", "status", "title", "spaceId",
    "createdAt", "version", "body",
}

SECRET_PATTERN = re.compile(
    r"token|api[_-]?key|password|secret|bearer|GITHUB_TOKEN|ATLASSIAN_API_KEY",
    re.IGNORECASE,
)


def check_github() -> tuple[list[str], list[str]]:
    """Check github_issues.json. Returns (asserts_passed, asserts_failed)."""
    passed: list[str] = []
    errors: list[str] = []
    path = FIXTURES / "github_issues.json"
    if not path.exists():
        return passed, [f"MISSING: {path}"]

    try:
        data = json.loads(path.read_text())
    except json.JSONDecodeError as exc:
        return passed, [f"INVALID JSON in {path}: {exc}"]

    if not isinstance(data, list):
        errors.append("github_issues.json: top-level must be a JSON array")
    elif len(data) < 3:
        errors.append(f"github_issues.json: need >=3 issues, got {len(data)}")
    else:
        any_missing = False
        for i, issue in enumerate(data):
            missing = GITHUB_REQUIRED - set(issue.keys())
            if missing:
                errors.append(f"github_issues.json issue[{i}] missing keys: {missing}")
                any_missing = True
        if not any_missing:
            passed.append(f"github_issues.json: {len(data)} issues with required keys")

    size = path.stat().st_size
    if size < 4000:
        errors.append(f"github_issues.json too small: {size} bytes (min 4000)")
    elif size > 12000:
        errors.append(f"github_issues.json too large: {size} bytes (max 12000)")
    else:
        passed.append(f"github_issues.json: size {size} bytes within 4000-12000")

    text = path.read_text()
    if SECRET_PATTERN.search(text):
        cred_pattern = re.compile(
            r'GITHUB_TOKEN|ATLASSIAN_API_KEY|"password"|"secret"|bearer\s+[A-Za-z0-9+/]{20}',
            re.IGNORECASE,
        )
        if cred_pattern.search(text):
            errors.append("github_issues.json: credential-shaped string detected")
        else:
            passed.append("github_issues.json: no credential-shaped strings")
    else:
        passed.append("github_issues.json: no credential-shaped strings")

    print(f"github_issues.json: {size} bytes, {len(data) if isinstance(data, list) else '?'} issues")
    return passed, errors


def check_confluence() -> tuple[list[str], list[str]]:
    """Check confluence_pages.json. Returns (asserts_passed, asserts_failed)."""
    passed: list[str] = []
    errors: list[str] = []
    path = FIXTURES / "confluence_pages.json"
    if not path.exists():
        return passed, [f"MISSING: {path}"]

    try:
        data = json.loads(path.read_text())
    except json.JSONDecodeError as exc:
        return passed, [f"INVALID JSON in {path}: {exc}"]

    if "results" not in data:
        errors.append("confluence_pages.json: missing top-level 'results' key")
        return passed, errors

    results = data["results"]
    if len(results) < 3:
        errors.append(f"confluence_pages.json: need >=3 pages, got {len(results)}")
    else:
        any_failure = False
        for i, page in enumerate(results):
            missing = CONFLUENCE_REQUIRED - set(page.keys())
            if missing:
                errors.append(f"confluence_pages.json page[{i}] missing keys: {missing}")
                any_failure = True
            try:
                adf_str = page["body"]["atlas_doc_format"]["value"]
                adf_doc = json.loads(adf_str)
                if adf_doc.get("type") != "doc":
                    errors.append(
                        f"confluence_pages.json page[{i}] ADF value: "
                        "expected top-level type=='doc'"
                    )
                    any_failure = True
            except (KeyError, TypeError, json.JSONDecodeError) as exc:
                errors.append(
                    f"confluence_pages.json page[{i}] ADF value not parseable: {exc}"
                )
                any_failure = True
        if not any_failure:
            passed.append(f"confluence_pages.json: {len(results)} pages with required keys + ADF re-parseable")

    size = path.stat().st_size
    if size < 6000:
        errors.append(f"confluence_pages.json too small: {size} bytes (min 6000)")
    elif size > 16000:
        errors.append(f"confluence_pages.json too large: {size} bytes (max 16000)")
    else:
        passed.append(f"confluence_pages.json: size {size} bytes within 6000-16000")

    cred_pattern = re.compile(
        r'ATLASSIAN_API_KEY|"password"|"secret"|bearer\s+[A-Za-z0-9+/]{20}|reuben',
        re.IGNORECASE,
    )
    if cred_pattern.search(path.read_text()):
        errors.append("confluence_pages.json: credential or PII string detected")
    else:
        passed.append("confluence_pages.json: no credential/PII strings")

    print(f"confluence_pages.json: {size} bytes, {len(results)} pages")
    return passed, errors


def check_readme() -> tuple[list[str], list[str]]:
    """Check fixtures/README.md. Returns (asserts_passed, asserts_failed)."""
    passed: list[str] = []
    errors: list[str] = []
    path = FIXTURES / "README.md"
    if not path.exists():
        return passed, [f"MISSING: {path}"]
    text = path.read_text()
    missing_terms = []
    for required_term in ("offline", "content_hash", "github_issues.json",
                          "confluence_pages.json", "synthetic"):
        if required_term not in text:
            missing_terms.append(required_term)
            errors.append(f"fixtures/README.md: missing required term '{required_term}'")
    if not missing_terms:
        passed.append("fixtures/README.md: contains all required terms")
    print(f"fixtures/README.md: {path.stat().st_size} bytes")
    return passed, errors


def write_artifact(asserts_passed: list[str], asserts_failed: list[str]) -> None:
    """Write JSON artifact for the runner."""
    ARTIFACT.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "ts": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "row_id": "code/fixtures-valid",
        "exit_code": 0 if not asserts_failed else 1,
        "asserts_passed": asserts_passed,
        "asserts_failed": asserts_failed,
    }
    ARTIFACT.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def main() -> int:
    """Run all fixture checks and report results."""
    all_passed: list[str] = []
    all_errors: list[str] = []

    p1, e1 = check_github()
    all_passed.extend(p1)
    all_errors.extend(e1)

    p2, e2 = check_confluence()
    all_passed.extend(p2)
    all_errors.extend(e2)

    p3, e3 = check_readme()
    all_passed.extend(p3)
    all_errors.extend(e3)

    write_artifact(all_passed, all_errors)

    if all_errors:
        print("\nFAILURES:")
        for err in all_errors:
            print(f"  FAIL: {err}")
        return 1

    print("\nAll fixture checks passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
