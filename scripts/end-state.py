#!/usr/bin/env python3
"""scripts/end-state.py — thin shim. Delegates to quality/runners/verdict.py.

This file does not grow. New gates go under quality/gates/<dim>/.
The session-end framework migrated to quality/runners/{run,verdict}.py
in v0.12.0 P57 (STRUCT-02 + SIMPLIFY-02). Per quality/PROTOCOL.md
anti-bloat rules per surface: ≤30 lines.

Subcommands:
  verdict            delegate to `python3 quality/runners/verdict.py session-end`
  init/list/status/verify/record-artifact   error stub; use the runner
"""
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
RUNNER = REPO_ROOT / "quality" / "runners" / "verdict.py"


def main() -> int:
    cmd = sys.argv[1] if len(sys.argv) > 1 else "verdict"
    if cmd == "verdict":
        return subprocess.run([sys.executable, str(RUNNER), "session-end"] + sys.argv[2:]).returncode
    print("scripts/end-state.py is now a thin shim; use `python3 quality/runners/run.py --cadence pre-push` and `python3 quality/runners/verdict.py` instead.", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
