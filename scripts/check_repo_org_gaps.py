#!/usr/bin/env python3
"""Verify the P62 repo-org gap audit is complete and consistent.

Reads:
- `.planning/research/v0.11.1-repo-organization-gaps.md` (the source rec doc)
- `quality/reports/audits/repo-org-gaps.md` (the P62 Wave 2 audit closure)

Asserts:
- Every numbered rec (1..N) in the source doc's "Top 10 cleanup recommendations"
  section is line-referenced in the audit table (regex `\\| <N> \\|`).
- Every Disposition cell in audit table rows is in the allow-list.
- No row carries an empty/unclassified Disposition.

Exit 0 = PASS; exit 1 + diagnostic stderr = FAIL.

Usage:
  python3 scripts/check_repo_org_gaps.py            # verify, print summary
  python3 scripts/check_repo_org_gaps.py --json     # machine-readable JSON

Stdlib only. CLAUDE.md OP-4: this is the committed audit invariant.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
SOURCE_DOC = REPO_ROOT / ".planning" / "research" / "v0.11.1-repo-organization-gaps.md"
AUDIT_DOC = REPO_ROOT / "quality" / "reports" / "audits" / "repo-org-gaps.md"

ALLOWED_DISPOSITIONS = {
    "closed-by-deletion",
    "closed-by-existing-gate",
    "closed-by-catalog-row",
    "closed-by-relocation",
    "closed-by-Wave-3-fix",
    "waived",
    "out-of-scope",
}


def extract_top10_rec_numbers(text: str) -> list[int]:
    """Extract numbered recs from the 'Top 10 cleanup recommendations' section."""
    match = re.search(
        r"## Top 10 cleanup recommendations\s*\n(.*?)(?=\n## )",
        text,
        re.DOTALL,
    )
    if not match:
        return []
    section = match.group(1)
    nums = []
    for line in section.splitlines():
        m = re.match(r"^(\d+)\. \*\*", line)
        if m:
            nums.append(int(m.group(1)))
    return nums


def extract_audited_rec_numbers(text: str) -> set[int]:
    """Extract item numbers from the audit's table rows: `| <N> | ...`."""
    nums: set[int] = set()
    for line in text.splitlines():
        m = re.match(r"^\|\s*(\d+)\s*\|", line)
        if m:
            nums.add(int(m.group(1)))
    return nums


def extract_dispositions(text: str) -> list[tuple[int, str]]:
    """Return (item_number, disposition) for every audit table row with a numeric first cell."""
    out: list[tuple[int, str]] = []
    for line in text.splitlines():
        if not line.startswith("|"):
            continue
        cells = [c.strip() for c in line.split("|")]
        # markdown table row: ['', col1, col2, col3, col4, '']
        if len(cells) < 5:
            continue
        first = cells[1]
        if not first.isdigit():
            continue
        disposition = cells[3]
        out.append((int(first), disposition))
    return out


def collect_wave3_items(audit_text: str) -> list[str]:
    """Pull the bullet list under '## Items requiring P62 Wave 3 fix'."""
    match = re.search(
        r"## Items requiring P62 Wave 3 fix\s*\n(.*?)(?=\n## |\Z)",
        audit_text,
        re.DOTALL,
    )
    if not match:
        return []
    section = match.group(1)
    items: list[str] = []
    for line in section.splitlines():
        m = re.match(r"^\d+\.\s+\*\*(.+?)\*\*", line)
        if m:
            items.append(m.group(1))
    return items


def main() -> int:
    parser = argparse.ArgumentParser(description="P62 repo-org gap audit verifier")
    parser.add_argument("--json", action="store_true", help="emit machine-readable JSON summary")
    args = parser.parse_args()

    if not SOURCE_DOC.exists():
        print(f"FAIL: source doc missing: {SOURCE_DOC}", file=sys.stderr)
        return 1
    if not AUDIT_DOC.exists():
        print(f"FAIL: audit doc missing: {AUDIT_DOC}", file=sys.stderr)
        return 1

    source_text = SOURCE_DOC.read_text(encoding="utf-8")
    audit_text = AUDIT_DOC.read_text(encoding="utf-8")

    top10_nums = extract_top10_rec_numbers(source_text)
    audited_nums = extract_audited_rec_numbers(audit_text)
    dispositions = extract_dispositions(audit_text)
    wave3_items = collect_wave3_items(audit_text)

    failures: list[str] = []

    # Assert every Top 10 rec is line-referenced in the audit
    missing_recs = sorted(set(top10_nums) - audited_nums)
    if missing_recs:
        failures.append(f"missing recs in audit table: {missing_recs}")

    # Assert every Disposition is in the allow-list
    bad_dispositions: list[tuple[int, str]] = []
    for num, disp in dispositions:
        if disp not in ALLOWED_DISPOSITIONS:
            bad_dispositions.append((num, disp))
    if bad_dispositions:
        failures.append(f"invalid dispositions: {bad_dispositions}")

    # Assert at least 25 audited items per plan must_haves
    if len(dispositions) < 25:
        failures.append(f"audit covers only {len(dispositions)} items; expected >=25")

    # Count by disposition
    counts: dict[str, int] = {d: 0 for d in ALLOWED_DISPOSITIONS}
    for _num, disp in dispositions:
        if disp in counts:
            counts[disp] += 1

    summary = {
        "total_items": len(dispositions),
        "top10_recs_in_source": len(top10_nums),
        "top10_recs_audited": sorted(audited_nums & set(top10_nums)),
        "missing_top10_recs": missing_recs,
        "counts_by_disposition": counts,
        "wave3_items": wave3_items,
        "failures": failures,
        "status": "PASS" if not failures else "FAIL",
    }

    if args.json:
        print(json.dumps(summary, indent=2, sort_keys=True))
    else:
        print(f"P62 repo-org gap audit verifier — {summary['status']}")
        print(f"  total items audited: {summary['total_items']}")
        print(f"  top-10 recs covered: {len(summary['top10_recs_audited'])}/{summary['top10_recs_in_source']}")
        for disp, n in sorted(counts.items()):
            print(f"  {disp}: {n}")
        print(f"  Wave 3 fix items: {len(wave3_items)}")
        for item in wave3_items:
            print(f"    - {item[:80]}")
        if failures:
            print("FAILURES:", file=sys.stderr)
            for f in failures:
                print(f"  - {f}", file=sys.stderr)

    return 0 if not failures else 1


if __name__ == "__main__":
    raise SystemExit(main())
