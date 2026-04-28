"""CLI helper invoked by dispatch.sh. P61 SUBJ-02.

Subcommands:
    list-stale         -- print rubric ids whose row is_stale or never verified
    list-all           -- print every rubric id
    stub <rubric-id>   -- write a Path-B stub artifact (score=0, verdict=NOT-IMPLEMENTED)

Stdlib-only. Keeps dispatch.sh under the 120-LOC anti-bloat cap by removing
the heredoc python blocks.
"""

from __future__ import annotations

import argparse
import sys
from datetime import datetime, timezone
from pathlib import Path

LIB_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(LIB_DIR))

from catalog import all_rows, find_row, load_subjective_catalog, stale_rows  # noqa: E402
from persist_artifact import persist_artifact  # noqa: E402


def cmd_list_stale() -> int:
    catalog = load_subjective_catalog()
    for r in stale_rows(catalog, datetime.now(timezone.utc)):
        print(r["id"])
    return 0


def cmd_list_all() -> int:
    for r in all_rows(load_subjective_catalog()):
        print(r["id"])
    return 0


def cmd_stub(rubric_id: str) -> int:
    catalog = load_subjective_catalog()
    try:
        row = find_row(catalog, rubric_id)
    except KeyError as e:
        print(f"FAIL: {e}", file=sys.stderr)
        return 2
    artifact_path = persist_artifact(
        rubric_id=row["id"],
        score=0,
        verdict="NOT-IMPLEMENTED",
        rationale=(
            "Path B stub: dispatched via runner subprocess without Task tool. "
            "Full Path A dispatch happens from a Claude session via SKILL.md. "
            "Wave G (61-07) drives the end-to-end Path A run."
        ),
        evidence_files=row.get("sources", []),
        dispatched_via="Path-B-runner-subprocess-stub",
        asserts_passed=[],
        asserts_failed=row["expected"]["asserts"],
    )
    print(f"stub artifact written: {artifact_path}")
    return 1  # Stub is FAIL semantics; runner records FAIL until Path A re-runs.


def main() -> int:
    parser = argparse.ArgumentParser(prog="dispatch_cli.py", description=__doc__)
    sub = parser.add_subparsers(dest="cmd", required=True)
    sub.add_parser("list-stale")
    sub.add_parser("list-all")
    p_stub = sub.add_parser("stub")
    p_stub.add_argument("rubric_id")
    args = parser.parse_args()
    if args.cmd == "list-stale":
        return cmd_list_stale()
    if args.cmd == "list-all":
        return cmd_list_all()
    if args.cmd == "stub":
        return cmd_stub(args.rubric_id)
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
