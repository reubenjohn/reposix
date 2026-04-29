#!/usr/bin/env bash
# P74 UX-BIND-02 (D-04): docs/index.md:78 claims "the audit trail is git log".
# Verifier asserts the claim's premise — the repo has at least one
# commit observable via `git log --oneline`. If `git log` breaks for any
# reason (e.g. shallow clone in CI without history, broken .git/), the
# verifier fires and a maintainer reviews.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "$REPO_ROOT"
FIRST=$(git log --oneline -n 1 2>/dev/null || true)
LINE_COUNT=0
[[ -n "$FIRST" ]] && LINE_COUNT=1
if (( LINE_COUNT < 1 )); then
  echo "FAIL: git log --oneline returned no commits — audit-trail premise broken" >&2
  exit 1
fi
echo "PASS: git log --oneline shows >=1 commit (audit-trail premise holds)"
exit 0
