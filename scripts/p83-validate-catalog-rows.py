#!/usr/bin/env python3
"""P83-01 T01 / T06 catalog validator.

Verifies the 4 P83-01 bus-write rows are present in
`quality/catalogs/agent-ux.json` and (in T06 mode) have the expected
final statuses.

Modes (argv[1]):
    present
        Asserts all 4 rows exist (status irrelevant).
    fail-init
        Asserts all 4 rows exist AND have status == FAIL (T01 close).
    p83-01-close
        Asserts rows 1, 3, 4 have status == PASS and row 2 has
        status == FAIL (P83-01 T06 close — row 2 flips to PASS only
        at P83-02 T04, see M1 in PLAN-CHECK.md).
    p83-02-close
        Asserts ALL 4 rows have status == PASS (P83-02 T04+).

The 4 rows:
    agent-ux/bus-write-sot-first-success
    agent-ux/bus-write-mirror-fail-returns-ok      (FAIL through 83-01 close)
    agent-ux/bus-write-no-helper-retry
    agent-ux/bus-write-no-mirror-remote-still-fails

Exit code 0 = OK, 1 = missing/wrong-status row(s), 2 = bad mode.
"""
import json
import sys

REQUIRED = [
    "agent-ux/bus-write-sot-first-success",
    "agent-ux/bus-write-mirror-fail-returns-ok",
    "agent-ux/bus-write-no-helper-retry",
    "agent-ux/bus-write-no-mirror-remote-still-fails",
]

MODES = {
    "present": None,  # status irrelevant
    "fail-init": {rid: "FAIL" for rid in REQUIRED},
    "p83-01-close": {
        "agent-ux/bus-write-sot-first-success": "PASS",
        "agent-ux/bus-write-mirror-fail-returns-ok": "FAIL",
        "agent-ux/bus-write-no-helper-retry": "PASS",
        "agent-ux/bus-write-no-mirror-remote-still-fails": "PASS",
    },
    "p83-02-close": {rid: "PASS" for rid in REQUIRED},
}


def main(argv):
    mode = argv[1] if len(argv) > 1 else "present"
    if mode not in MODES:
        print(
            f"unknown mode {mode!r}; expected one of {sorted(MODES)}",
            file=sys.stderr,
        )
        return 2

    with open("quality/catalogs/agent-ux.json") as f:
        data = json.load(f)

    rows_by_id = {r["id"]: r for r in data["rows"]}

    missing = [r for r in REQUIRED if r not in rows_by_id]
    if missing:
        print(f"missing rows: {missing}", file=sys.stderr)
        return 1

    expected = MODES[mode]
    if expected is not None:
        mismatches = [
            (r, rows_by_id[r].get("status"), exp_status)
            for r, exp_status in expected.items()
            if rows_by_id[r].get("status") != exp_status
        ]
        if mismatches:
            print(f"status mismatches: {mismatches}", file=sys.stderr)
            return 1

    # M1 invariant: row 2 must carry the comment field annotation through 83-01 close.
    if mode in ("present", "fail-init", "p83-01-close"):
        row2 = rows_by_id["agent-ux/bus-write-mirror-fail-returns-ok"]
        comment = row2.get("comment", "")
        if "FAIL through 83-01 close" not in comment:
            print(
                "row 2 (bus-write-mirror-fail-returns-ok) missing M1 comment "
                "annotating 'FAIL through 83-01 close'",
                file=sys.stderr,
            )
            return 1

    print(
        f"OK: mode={mode}; all {len(REQUIRED)} P83-01 rows present in "
        f"agent-ux.json (total rows: {len(data['rows'])})"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
