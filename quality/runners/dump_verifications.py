#!/usr/bin/env python3
"""Dump quality-gate verification artifacts to stdout for CI diagnosis.

`run.py` captures each verifier's stdout/stderr into the per-row JSON
artifact under ``quality/reports/verifications/**``, NOT the job log. A FAIL
row therefore leaves its root cause in an ephemeral file the runner never
prints, so a CI failure cannot be diagnosed without a local reproduction.

The pre-pr CI job invokes this on failure (``if: failure()``) to surface every
artifact's ``asserts_failed`` / ``stderr`` (and any transcript) in the log.
Read-only, stdlib-only, best-effort: an unreadable artifact is noted, never
fatal. Exit 0 always (a diagnostic dump must not mask the real failure).

Usage: ``python3 quality/runners/dump_verifications.py [repo_root]``
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

# 2026-07-07 (RBF investigation, S-260707-rbf-01): 40 was too small for
# verifiers like p94-git243-fallback-sentinel.sh, which appends a `grep -B2
# -A15 'panicked at|assertion.*failed'` diagnostic block FOLLOWED BY a
# `tail -60` of the raw cargo-test log. With a 40-line tail window, the
# window fell entirely inside the trailing 60-line block, silently
# discarding the earlier grep context that contains the actual panic/
# assertion message -- exactly the failure this dump step exists to
# surface. Raised well past the largest known appended block (60 lines)
# so both the grep diagnostic and the raw tail survive truncation.
_TAIL_LINES = 200


def _dump_artifact(path: Path) -> None:
    print(f">>> {path}")
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        print(f"  (unreadable: {exc})")
        return
    for key in ("row_id", "exit_code", "status", "timed_out", "reason"):
        if key in data:
            print(f"  {key}: {data[key]}")
    for key in ("asserts_failed", "stderr", "stdout"):
        value = data.get(key)
        if not value:
            continue
        print(f"  {key}:")
        text = value if isinstance(value, str) else json.dumps(value, indent=2)
        for line in text.splitlines()[-_TAIL_LINES:]:
            print(f"    {line}")


def main() -> int:
    repo_root = Path(sys.argv[1]) if len(sys.argv) > 1 else Path.cwd()
    verifications = repo_root / "quality" / "reports" / "verifications"
    transcripts = repo_root / "quality" / "reports" / "transcripts"

    print("=== quality verification artifacts (stderr/asserts_failed) ===")
    for artifact in sorted(verifications.rglob("*.json")):
        _dump_artifact(artifact)

    print("=== transcripts ===")
    for transcript in sorted(transcripts.glob("*.txt")):
        print(f">>> {transcript}")
        try:
            lines = transcript.read_text(encoding="utf-8").splitlines()
        except OSError as exc:
            print(f"  (unreadable: {exc})")
            continue
        for line in lines[-60:]:
            print(f"  {line}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
