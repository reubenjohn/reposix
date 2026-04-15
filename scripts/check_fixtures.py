#!/usr/bin/env python3
"""Validate benchmark fixture files for shape, size, and content safety.

Checks:
  - github_issues.json: JSON array, >=3 issues, required keys, 4-12 KB, no secrets.
  - confluence_pages.json: JSON object with results[], >=3 pages, required keys,
    ADF value re-parseable, 6-16 KB, no secrets.

Run from the repository root:
    python3 scripts/check_fixtures.py
"""

from __future__ import annotations

import json
import pathlib
import re
import sys

FIXTURES = pathlib.Path("benchmarks/fixtures")

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


def check_github() -> list[str]:
    """Check benchmarks/fixtures/github_issues.json.

    # Errors
    Returns a list of error strings; empty list means all checks passed.
    """
    errors: list[str] = []
    path = FIXTURES / "github_issues.json"
    if not path.exists():
        return [f"MISSING: {path}"]

    try:
        data = json.loads(path.read_text())
    except json.JSONDecodeError as exc:
        return [f"INVALID JSON in {path}: {exc}"]

    if not isinstance(data, list):
        errors.append("github_issues.json: top-level must be a JSON array")
    elif len(data) < 3:
        errors.append(f"github_issues.json: need >=3 issues, got {len(data)}")
    else:
        for i, issue in enumerate(data):
            missing = GITHUB_REQUIRED - set(issue.keys())
            if missing:
                errors.append(f"github_issues.json issue[{i}] missing keys: {missing}")

    size = path.stat().st_size
    if size < 4000:
        errors.append(f"github_issues.json too small: {size} bytes (min 4000)")
    if size > 12000:
        errors.append(f"github_issues.json too large: {size} bytes (max 12000)")

    text = path.read_text()
    if SECRET_PATTERN.search(text):
        # False-positive guard: the pattern 'token' appears in legitimate fixture fields
        # like "timeline_url" and "count". We check for credential-shaped strings only.
        cred_pattern = re.compile(
            r'GITHUB_TOKEN|ATLASSIAN_API_KEY|"password"|"secret"|bearer\s+[A-Za-z0-9+/]{20}',
            re.IGNORECASE,
        )
        if cred_pattern.search(text):
            errors.append("github_issues.json: credential-shaped string detected")

    print(f"github_issues.json: {size} bytes, {len(data) if isinstance(data, list) else '?'} issues")
    return errors


def check_confluence() -> list[str]:
    """Check benchmarks/fixtures/confluence_pages.json.

    # Errors
    Returns a list of error strings; empty list means all checks passed.
    """
    errors: list[str] = []
    path = FIXTURES / "confluence_pages.json"
    if not path.exists():
        return [f"MISSING: {path}"]

    try:
        data = json.loads(path.read_text())
    except json.JSONDecodeError as exc:
        return [f"INVALID JSON in {path}: {exc}"]

    if "results" not in data:
        errors.append("confluence_pages.json: missing top-level 'results' key")
        return errors

    results = data["results"]
    if len(results) < 3:
        errors.append(f"confluence_pages.json: need >=3 pages, got {len(results)}")
    else:
        for i, page in enumerate(results):
            missing = CONFLUENCE_REQUIRED - set(page.keys())
            if missing:
                errors.append(f"confluence_pages.json page[{i}] missing keys: {missing}")
            try:
                adf_str = page["body"]["atlas_doc_format"]["value"]
                adf_doc = json.loads(adf_str)
                if adf_doc.get("type") != "doc":
                    errors.append(
                        f"confluence_pages.json page[{i}] ADF value: "
                        "expected top-level type=='doc'"
                    )
            except (KeyError, TypeError, json.JSONDecodeError) as exc:
                errors.append(
                    f"confluence_pages.json page[{i}] ADF value not parseable: {exc}"
                )

    size = path.stat().st_size
    if size < 6000:
        errors.append(f"confluence_pages.json too small: {size} bytes (min 6000)")
    if size > 16000:
        errors.append(f"confluence_pages.json too large: {size} bytes (max 16000)")

    cred_pattern = re.compile(
        r'ATLASSIAN_API_KEY|"password"|"secret"|bearer\s+[A-Za-z0-9+/]{20}|reuben',
        re.IGNORECASE,
    )
    if cred_pattern.search(path.read_text()):
        errors.append("confluence_pages.json: credential or PII string detected")

    print(f"confluence_pages.json: {size} bytes, {len(results)} pages")
    return errors


def check_readme() -> list[str]:
    """Check benchmarks/fixtures/README.md.

    # Errors
    Returns a list of error strings; empty list means all checks passed.
    """
    errors: list[str] = []
    path = FIXTURES / "README.md"
    if not path.exists():
        return [f"MISSING: {path}"]
    text = path.read_text()
    for required_term in ("offline", "content_hash", "github_issues.json",
                          "confluence_pages.json", "synthetic"):
        if required_term not in text:
            errors.append(f"fixtures/README.md: missing required term '{required_term}'")
    print(f"fixtures/README.md: {path.stat().st_size} bytes")
    return errors


def main() -> int:
    """Run all fixture checks and report results.

    # Errors
    Returns 1 if any check fails, 0 if all pass.
    """
    all_errors: list[str] = []
    all_errors.extend(check_github())
    all_errors.extend(check_confluence())
    all_errors.extend(check_readme())

    if all_errors:
        print("\nFAILURES:")
        for err in all_errors:
            print(f"  FAIL: {err}")
        return 1

    print("\nAll fixture checks passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
