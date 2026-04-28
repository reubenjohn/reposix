#!/usr/bin/env bash
#
# Unit tests for .githooks/pre-push.
#
# Runs in-place inside the repo. Creates a throw-away temp file with
# adversarial content, stages it, wraps it in a dummy commit on a
# detached HEAD, runs the hook against that commit's range, and tears
# everything down — leaving the working tree exactly as it was.
#
# Usage:
#   bash .githooks/test-pre-push.sh
#
# Exit code: 0 on all green, 1 on any failure.
# CI-safe: no network calls, no modifications to main.

set -euo pipefail

readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly NC='\033[0m'

readonly repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly hook="${repo_root}/.githooks/pre-push"

cd "$repo_root"

if [[ ! -x "$hook" ]]; then
  printf '%b\n' "${RED}✖ hook not executable: ${hook}${NC}" >&2
  exit 1
fi

# Save current branch + HEAD so cleanup() can restore. The test
# detaches HEAD during execution; cleanup must return us to the
# original branch if we started on one, not leave us in detached
# HEAD state (which would silently swallow any subsequent commits).
readonly orig_head="$(git rev-parse HEAD)"
readonly orig_branch="$(git symbolic-ref --short -q HEAD || echo '')"
readonly tmp_branch="test-pre-push-$$-$RANDOM"

cleanup() {
  # Drop any throw-away test-created commits on the detached head.
  git reset -q --hard "$orig_head" 2>/dev/null || true
  if [[ -n "$orig_branch" ]]; then
    git checkout -q "$orig_branch" 2>/dev/null || true
  else
    git checkout -q "$orig_head" 2>/dev/null || true
  fi
  git branch -D "$tmp_branch" 2>/dev/null || true
  rm -f "${repo_root}/.test-pre-push-fixture.txt"
}
trap cleanup EXIT

# Helper: run the hook with a synthesized push-ref-range for HEAD.
# Arg 1: label, Arg 2: expected exit code (0 = pass, 1 = reject).
run_and_check() {
  local label="$1"
  local expected="$2"
  local actual=0
  echo "refs/heads/test HEAD HEAD^{commit}~1 $(git rev-parse HEAD^)" \
    | bash "$hook" > /tmp/test-pre-push.out 2>&1 || actual=$?
  if [[ "$actual" == "$expected" ]]; then
    printf '%b\n' "${GREEN}✓${NC} ${label} (exit=${actual})"
    return 0
  else
    printf '%b\n' "${RED}✖ ${label}: expected exit=${expected}, got ${actual}${NC}" >&2
    sed 's/^/    /' /tmp/test-pre-push.out >&2
    return 1
  fi
}

printf '%b\n' "${YELLOW}→${NC} testing pre-push hook on detached ${tmp_branch}..."

fails=0

# --- TEST 1: clean commit passes. -------------------------------------
# We use the existing HEAD which we've already pushed through the hook
# in other operations; it should not contain any token-prefix strings.
if ! run_and_check "clean commit passes" 0; then
  fails=$((fails + 1))
fi

# --- TEST 2: commit containing ATATT3 token prefix is rejected. -------
git checkout -q --detach HEAD
echo 'ATATT3xFfWELr_FakeTokenForHookTest' > .test-pre-push-fixture.txt
git add .test-pre-push-fixture.txt
git -c user.email=test@test -c user.name=test commit -q -m "test: inject fake ATATT3 token"
if ! run_and_check "ATATT3 token rejected" 1; then
  fails=$((fails + 1))
fi
git reset -q --hard HEAD^  # pop the fake commit

# --- TEST 3: commit with Bearer ATATT3 header is rejected. ------------
# Fixture must have 20+ chars after ATATT3 to match the stricter pattern.
echo 'Authorization: Bearer ATATT3fake_token_abcdefghijklmnopqr' > .test-pre-push-fixture.txt
git add .test-pre-push-fixture.txt
git -c user.email=test@test -c user.name=test commit -q -m "test: inject fake Bearer ATATT3"
if ! run_and_check "Bearer ATATT3 rejected" 1; then
  fails=$((fails + 1))
fi
git reset -q --hard HEAD^

# --- TEST 4: commit with GitHub ghp_ prefix is rejected. --------------
echo 'ghp_abcdefghijklmnopqrstuvwxyz0123456789' > .test-pre-push-fixture.txt
git add .test-pre-push-fixture.txt
git -c user.email=test@test -c user.name=test commit -q -m "test: inject fake ghp_ token"
if ! run_and_check "ghp_ GitHub PAT rejected" 1; then
  fails=$((fails + 1))
fi
git reset -q --hard HEAD^

# --- TEST 5: commit with github_pat_ prefix is rejected. --------------
echo 'github_pat_abcdefghijklmnopqrstuvwxyz012' > .test-pre-push-fixture.txt
git add .test-pre-push-fixture.txt
git -c user.email=test@test -c user.name=test commit -q -m "test: inject fake fine-grained GitHub PAT"
if ! run_and_check "github_pat_ rejected" 1; then
  fails=$((fails + 1))
fi
git reset -q --hard HEAD^

# --- TEST 6: hook file itself is excluded from self-scan. -------------
# (Confirm the hook doesn't fail against its own PATTERNS=('ATATT3' ...)
# body. We do this by triggering a fake commit that touches only the
# hook file with a comment appended — no real change — and verifying
# the range-diff is empty from the scanner's perspective.)
orig_hook_contents="$(cat "$hook")"
printf '\n# Test marker — ignore (ATATT3-self-test)\n' >> "$hook"
git add "$hook"
git -c user.email=test@test -c user.name=test commit -q -m "test: touch hook file"
if ! run_and_check "hook self-scan exclusion honored" 0; then
  fails=$((fails + 1))
fi
git reset -q --hard HEAD^
# Restore hook if anything went sideways.
printf '%s' "$orig_hook_contents" > "$hook"
chmod +x "$hook"

# --- TEST 7: runner exit-non-zero MUST propagate through the hook. ----
# Regression coverage for fdb4d24, which fixed `if ! cmd; then exit $?;
# fi` (always exits 0 because $? after `! cmd` is the negation, masking
# real failures). The bug let bad pushes through; the test must observe
# that today the hook exits non-zero when the runner does.
#
# Mechanism: PATH-stub `python3` with a script that exits 1, then drive
# the hook with empty stdin (so the cred-hygiene block short-circuits
# and the runner is the only thing left that can set the exit code).
# Cleanup overlays the existing trap so the stub dir is removed even
# if the hook exits abnormally.
stub_dir="$(mktemp -d)"
runner_fail_cleanup() {
  rm -rf "$stub_dir" 2>/dev/null || true
}
trap 'cleanup; runner_fail_cleanup' EXIT
cat > "${stub_dir}/python3" <<'STUB'
#!/usr/bin/env bash
# Stub python3 used by .githooks/test-pre-push.sh TEST 7. Pretends to
# be the quality-gates runner and exits non-zero so the harness can
# assert exit-code propagation. Anything else invoking python3 during
# this test (e.g. nested helpers) gets the same fail signal — that's
# fine; we only run the hook here, and the hook only invokes python3
# for the runner.
echo "stub python3: forcing runner FAIL for harness regression test" >&2
exit 1
STUB
chmod +x "${stub_dir}/python3"

run_with_stub() {
  local label="$1"
  local expected="$2"
  local actual=0
  PATH="${stub_dir}:$PATH" bash "$hook" \
    < /dev/null > /tmp/test-pre-push-runner-fail.out 2>&1 || actual=$?
  if [[ "$actual" == "$expected" ]]; then
    printf '%b\n' "${GREEN}✓${NC} ${label} (exit=${actual})"
    return 0
  else
    printf '%b\n' "${RED}✖ ${label}: expected exit=${expected}, got ${actual}${NC}" >&2
    sed 's/^/    /' /tmp/test-pre-push-runner-fail.out >&2
    return 1
  fi
}

if ! run_with_stub "runner FAIL exit propagates" 1; then
  fails=$((fails + 1))
fi

# Restore the original trap (drop the stub-cleanup overlay) and clean
# up the stub dir eagerly so it doesn't leak past the test boundary.
trap cleanup EXIT
runner_fail_cleanup

# --- Summary. ---------------------------------------------------------
printf '\n'
if [[ "$fails" -eq 0 ]]; then
  printf '%b\n' "${GREEN}✓ all hook tests passed${NC}"
  exit 0
fi
printf '%b\n' "${RED}✖ ${fails} hook test(s) failed${NC}" >&2
exit 1
