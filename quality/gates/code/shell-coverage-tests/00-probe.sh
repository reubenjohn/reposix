#!/usr/bin/env bash
# 00-probe.sh — kcov invocation probe. Drives ONE self-contained target script
# (docs-alignment/install-snippet-shape.sh) via its shebang so kcov captures its
# line coverage. This is the smallest possible proof that the driver's kcov
# invocation works (shebang execution + include/exclude paths + merge).
#
# Harnesses are NOT graded for their own coverage (the dir is --exclude-path'd);
# they exist only to EXECUTE targets under kcov. Each harness must exit 0, so we
# swallow the target's exit code — a non-zero target exit is a real gate failure
# reported by the gate that owns that script, not by this coverage harness.
set -eu

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"

run_target() {
  # run_target <path> [args...] — execute a target via shebang, ignore its exit.
  local target="$1"; shift
  "$REPO_ROOT/$target" "$@" >/dev/null 2>&1 || true
}

run_target quality/gates/docs-alignment/install-snippet-shape.sh

exit 0
