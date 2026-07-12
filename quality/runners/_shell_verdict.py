#!/usr/bin/env python3
"""Canonical serializer for kind: shell-subprocess committed verdicts.

SINGLE SOURCE OF TRUTH for the DETERMINISTIC verdict schema written to
quality/reports/verifications/agent-ux/<slug>.json by BOTH producers:

  1. quality/gates/agent-ux/lib/transcript.sh — the gate's own emitter (run
     standalone by an agent, dark-factory.sh, or the runner), and
  2. quality/runners/run.py — the pre-push / CI grader (run_row).

Why this module exists (D-P96-01, extended): P96 split the runner's GRADE from
PERSIST for the *catalog* (save_catalog gated behind --persist) but the per-row
verdict artifact was still rewritten on EVERY grade run. For kind:
shell-subprocess rows that artifact carried a per-run `ts` and an RFC3339-
stamped `transcript_path`, so a read-only pre-push GATE run self-mutated the
tracked JSON and repeatedly tripped stop-on-dirty in the shared tree.

The fix: the committed verdict records ONLY the graded RESULT — exit_code plus
the asserts the scenario actually evaluated — and a STABLE transcript_path. All
volatile detail (the per-run timestamp, captured stdout/stderr, env_keys, cwd)
lives in the gitignored transcript at transcript_path, NOT the committed JSON.
Both producers serialize through THIS module so their bytes are identical and a
re-run changes the committed file ONLY when the graded result changes.

Formatting note: `dumps()` mirrors run.py's write_artifact (json.dumps indent=2
+ trailing newline) so the two producers are byte-for-byte interchangeable.
"""
from __future__ import annotations

import json
import sys
from typing import Iterable


# Canonical key order — both producers construct the dict in THIS order so
# json.dumps (which preserves insertion order) yields identical bytes.
def canonical_verdict(
    row_id: str,
    exit_code: int | None,
    transcript_path: str | None,
    asserts_passed: Iterable[str],
    asserts_failed: Iterable[str],
) -> dict:
    return {
        "row_id": row_id,
        "exit_code": exit_code,
        "transcript_path": transcript_path,
        "asserts_passed": list(asserts_passed),
        "asserts_failed": list(asserts_failed),
    }


def dumps(verdict: dict) -> str:
    """Serialize identically to run.py's write_artifact (indent=2 + newline)."""
    return json.dumps(verdict, indent=2) + "\n"


def _main(argv: list[str]) -> int:
    """CLI for transcript.sh: emit the canonical verdict JSON to a file.

    argv: <artifact_path> <row_id> <exit_code> <transcript_path>
    stdin: one assert per line, TAB-separated as "PASS\\t<label>" / "FAIL\\t<label>".
    Blank / malformed lines are ignored so an empty scenario report is a clean
    (asserts_passed=[], asserts_failed=[]) verdict, not a crash.
    """
    if len(argv) < 5:
        sys.stderr.write(
            "usage: _shell_verdict.py <artifact_path> <row_id> <exit_code> "
            "<transcript_path>  (assert lines on stdin)\n"
        )
        return 2
    artifact_path, row_id, exit_code_s, transcript_path = argv[1:5]
    passed: list[str] = []
    failed: list[str] = []
    for raw in sys.stdin:
        line = raw.rstrip("\n")
        if not line:
            continue
        verdict, tab, label = line.partition("\t")
        if not tab:
            continue
        if verdict == "PASS":
            passed.append(label)
        elif verdict == "FAIL":
            failed.append(label)
    try:
        exit_code: int | None = int(exit_code_s)
    except ValueError:
        exit_code = None
    verdict_obj = canonical_verdict(
        row_id, exit_code, transcript_path or None, passed, failed
    )
    with open(artifact_path, "w", encoding="utf-8") as fh:
        fh.write(dumps(verdict_obj))
    return 0


if __name__ == "__main__":
    raise SystemExit(_main(sys.argv))
