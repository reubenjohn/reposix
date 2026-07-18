#!/usr/bin/env bash
# quality/gates/agent-ux/lib/t4-real-backend-flow.selftest.sh
#
# Hermetic selftest for the SC5b fix (DRAIN-01): _t4_checkout_or_fail must
# surface the REAL captured git stderr as the failure detail, never the
# hardcoded (and, at this call site, always-FALSE) "requires git >= 2.34"
# fallback string that used to be passed to hard_fail_exit inline.
#
# Builds a throwaway /tmp git repo — never the shared repo, so
# leaf-isolation-guard.sh is a no-op here (plain `git init`, not
# `reposix init`/sim-seed) — with NO refs/reposix/origin/main ref, so the
# checkout inside _t4_checkout_or_fail is guaranteed to fail for a REAL git
# reason (unknown ref). This proves the structural fix without needing real
# Confluence credentials (the row this guards stays env-gated NOT-VERIFIED).
#
# Run: bash quality/gates/agent-ux/lib/t4-real-backend-flow.selftest.sh
# Exit 0 = all assertions pass; exit 1 = a regression.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIB="${SCRIPT_DIR}/t4-real-backend-flow.sh"
[[ -f "$LIB" ]] || { echo "FATAL: lib not found at $LIB" >&2; exit 1; }

WORK="$(mktemp -d "${TMPDIR:-/tmp}/t4-real-backend-flow-selftest.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT

git -C "$WORK" init -q
git -C "$WORK" config core.hooksPath /dev/null
git -C "$WORK" config user.name "selftest"
git -C "$WORK" config user.email "selftest@example.invalid"
# One commit so HEAD/the repo is non-empty, but deliberately NO
# refs/reposix/origin/main -- _t4_checkout_or_fail's checkout target is
# guaranteed absent, so the checkout fails for a real (non-version) reason.
echo "seed" > "$WORK/seed.txt"
git -C "$WORK" add seed.txt
git -C "$WORK" commit -qm seed

# Stub hard_fail_exit: RECORD args instead of exiting, so the assertions
# below can inspect exactly what _t4_checkout_or_fail passed it (in
# production this is the caller's real hard_fail_exit, which does exit 1).
HARD_FAIL_CALLED=0
HARD_FAIL_LABEL=""
HARD_FAIL_DETAIL=""
hard_fail_exit() {
  HARD_FAIL_CALLED=1
  HARD_FAIL_LABEL="$1"
  HARD_FAIL_DETAIL="${2:-}"
}

# shellcheck source=quality/gates/agent-ux/lib/t4-real-backend-flow.sh
source "$LIB"

_t4_checkout_or_fail "$WORK" "SELFTEST"

pass=0; fail=0
check() {  # check <label> <0|1> <detail-on-fail>
  if [[ "$2" -eq 1 ]]; then echo "  PASS: $1"; pass=$((pass + 1))
  else echo "  FAIL: $1 -- $3"; fail=$((fail + 1)); fi
}

echo "== _t4_checkout_or_fail against a repo with no refs/reposix/origin/main =="

c1=0; [[ "$HARD_FAIL_CALLED" -eq 1 ]] && c1=1
check "hard_fail_exit was invoked" "$c1" "checkout against a missing ref should have failed"

c2=0; echo "$HARD_FAIL_DETAIL" | grep -qE 'error:|fatal:|pathspec' && c2=1
check "captured detail contains real git stderr" "$c2" "detail was: $HARD_FAIL_DETAIL"

c3=0; [[ "$HARD_FAIL_DETAIL" != *"requires git >= 2.34"* ]] && c3=1
check "captured detail does NOT contain the hardcoded git-version fallback" "$c3" "detail was: $HARD_FAIL_DETAIL"

echo
echo "  observed hard_fail_exit args: label='$HARD_FAIL_LABEL' detail='$HARD_FAIL_DETAIL'"
echo
echo "RESULT: $pass passed, $fail failed"
[[ $fail -eq 0 ]] || exit 1
