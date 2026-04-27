#!/usr/bin/env python3
"""check_install_rows_catalog.py — verify the P56 install-row contract.

Per .planning/phases/56-restore-release-artifacts/56-01-PLAN.md Step 5:
the catalog file at .planning/docs_reproducible_catalog.json is the
contract Wave D's verifier subagent reads. This script asserts the
shape invariants the contract requires, so:

1. Wave A's commit gate (Task 56-01-B + Task 56-01-C verify blocks) has
   a named, committed grader rather than a 30-line inline `python3 -c`.
2. Wave D's verifier subagent has a single one-line invocation it can
   trust without rebuilding the assertion list from prose.
3. P57's catalog migration has a regression detector — if migration
   accidentally drops a phase tag or a tightened assert, this script
   yells.

Stdlib only. Exits 0 if every invariant holds, 1 otherwise (with the
specific failure printed to stderr).

Usage:
  python3 scripts/check_install_rows_catalog.py

Per CLAUDE.md OP-4 (self-improving infrastructure): ad-hoc bash that
asserts cross-file invariants gets promoted to a committed artifact so
the next agent finds a named command instead of reconstructing the
pipeline.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
CATALOG = REPO_ROOT / ".planning" / "docs_reproducible_catalog.json"

# Mirror of the 4 broken-as-of-P56-start rows (catalog row IDs whose
# last_verified_at MUST be null at Wave A close — the stale 2026-04-27
# timestamp was misleading; Wave C overwrites with a real verification).
BROKEN_ROW_IDS = (
    "install/curl-installer-sh",
    "install/powershell-installer-ps1",
    "install/cargo-binstall",
    "install/homebrew",
)

ALL_INSTALL_ROW_IDS = BROKEN_ROW_IDS + ("install/build-from-source",)


def fail(msg: str) -> None:
    print(f"FAIL: {msg}", file=sys.stderr)
    sys.exit(1)


def main() -> int:
    if not CATALOG.exists():
        fail(f"catalog missing: {CATALOG}")

    try:
        data = json.loads(CATALOG.read_text())
    except json.JSONDecodeError as exc:
        fail(f"catalog JSON does not parse: {exc}")
        return 1  # unreachable, but keeps mypy happy

    # 1. schema_status flipped to ACTIVE-V0
    schema_status = data.get("schema_status", "")
    if not schema_status.startswith("ACTIVE-V0"):
        fail(
            "schema_status must start with 'ACTIVE-V0'; "
            f"got: {schema_status!r}"
        )

    claims = data.get("claims", [])
    install_rows = [c for c in claims if c.get("id", "").startswith("install/")]

    # 2. exactly 5 install rows
    if len(install_rows) != 5:
        fail(f"expected 5 install rows, got {len(install_rows)}")

    # 3. every install row has the right id and phase tag
    seen_ids = {c["id"] for c in install_rows}
    expected_ids = set(ALL_INSTALL_ROW_IDS)
    if seen_ids != expected_ids:
        missing = expected_ids - seen_ids
        extra = seen_ids - expected_ids
        fail(f"install row IDs drifted; missing={missing} extra={extra}")

    for row in install_rows:
        if row.get("phase") != "p56":
            fail(f"install row {row['id']} missing phase: 'p56' tag")

    # 4. broken rows have last_verified_at: null
    for row_id in BROKEN_ROW_IDS:
        row = next(c for c in install_rows if c["id"] == row_id)
        if row.get("last_verified_at") is not None:
            fail(
                f"{row_id} last_verified_at must be null at Wave A close; "
                f"got: {row.get('last_verified_at')!r}"
            )

    # 5. build-from-source keeps its implicit-via-ci-test note
    bfs = next(c for c in install_rows if c["id"] == "install/build-from-source")
    bfs_lva = bfs.get("last_verified_at") or ""
    if "implicit" not in bfs_lva or "ci.yml" not in bfs_lva:
        fail(
            "install/build-from-source last_verified_at must keep its "
            f"'implicit via ci.yml test job last green run' note; got: {bfs_lva!r}"
        )

    # 6. curl row asserts contain the three contract keywords
    curl = next(c for c in install_rows if c["id"] == "install/curl-installer-sh")
    curl_asserts = " ".join(curl.get("expected_outcome", {}).get("asserts", []))
    for needle in ("container", "command -v reposix", ">= 1024 bytes"):
        if needle not in curl_asserts:
            fail(
                f"install/curl-installer-sh asserts missing required keyword: {needle!r}"
            )

    # 7. broken rows' remediation_hint references Wave B and the diagnosis doc
    for row_id in BROKEN_ROW_IDS:
        row = next(c for c in install_rows if c["id"] == row_id)
        hint = row.get("remediation_hint", "")
        if "P56 Wave B" not in hint or "v0.12.0-install-regression-diagnosis.md" not in hint:
            fail(
                f"{row_id} remediation_hint must cite P56 Wave B and the diagnosis doc; "
                f"got: {hint!r}"
            )

    # 8. non-install rows untouched (negative test: they must NOT have
    # phase: 'p56' — otherwise we polluted P59's domain).
    other_rows = [c for c in claims if not c.get("id", "").startswith("install/")]
    if not other_rows:
        fail("expected non-install rows present (tutorial/quickstart/benchmark/troubleshooting)")
    for row in other_rows:
        if row.get("phase") == "p56":
            fail(
                f"non-install row {row['id']} has phase: 'p56' — only install rows "
                "are tagged in P56; the tutorial/benchmark rows belong to P59 (DOCS-REPRO-04)"
            )

    print(f"OK: {len(install_rows)} install rows tagged phase=p56")
    print(f"OK: schema_status starts with ACTIVE-V0")
    print(f"OK: 4 broken rows have last_verified_at=null")
    print(f"OK: build-from-source kept its implicit-via-ci note")
    print(f"OK: curl asserts contain all 3 required contract keywords")
    print(f"OK: 4 broken rows' remediation_hint cites Wave B + diagnosis doc")
    print(f"OK: {len(other_rows)} non-install rows untouched (no p56 tag pollution)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
