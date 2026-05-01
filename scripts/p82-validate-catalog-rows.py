#!/usr/bin/env python3
"""P82-01 T01 / T06 catalog validator.

Verifies the 6 P82 bus-remote rows are present in
`quality/catalogs/agent-ux.json`. Used by:
- T01 verify gate (rows present, status FAIL)
- T06 verify gate (rows present, status PASS after runner re-grade)

Mode is selected via argv[1]:
    python3 scripts/p82-validate-catalog-rows.py present
        Asserts the 6 rows exist (status irrelevant).
    python3 scripts/p82-validate-catalog-rows.py pass
        Asserts the 6 rows exist AND have status == PASS.

Exit code 0 = OK, 1 = missing/wrong-status row(s).
"""
import json
import sys

REQUIRED = [
    "agent-ux/bus-url-parses-query-param-form",
    "agent-ux/bus-url-rejects-plus-delimited",
    "agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first",
    "agent-ux/bus-precheck-b-sot-drift-emits-fetch-first",
    "agent-ux/bus-fetch-not-advertised",
    "agent-ux/bus-no-remote-configured-error",
]


def main(argv):
    mode = argv[1] if len(argv) > 1 else "present"
    if mode not in ("present", "pass"):
        print(f"unknown mode {mode!r}; expected 'present' or 'pass'", file=sys.stderr)
        return 2

    with open("quality/catalogs/agent-ux.json") as f:
        data = json.load(f)

    rows_by_id = {r["id"]: r for r in data["rows"]}

    missing = [r for r in REQUIRED if r not in rows_by_id]
    if missing:
        print(f"missing rows: {missing}", file=sys.stderr)
        return 1

    if mode == "pass":
        not_pass = [(r, rows_by_id[r].get("status")) for r in REQUIRED if rows_by_id[r].get("status") != "PASS"]
        if not_pass:
            print(f"rows not PASS: {not_pass}", file=sys.stderr)
            return 1
        print(f"all {len(REQUIRED)} P82 rows have status PASS")
    else:
        print(f"all {len(REQUIRED)} P82 rows present in agent-ux.json (total rows: {len(data['rows'])})")

    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
