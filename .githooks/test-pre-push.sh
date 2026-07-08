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
#
# Scope (2026-07-07): this is a UNIT test of the credential-hygiene gate
# (hook step 1) + the hook's exit-code plumbing -- NOT a re-run of the
# whole pre-push quality suite. The hook's step 2 (`python3
# quality/runners/run.py --cadence pre-push`: clippy, mkdocs,
# shell-coverage, ...) owns its own CI jobs (`quality gates`,
# `shell-coverage`). Coupling this security test to that suite made the two
# "clean pass" assertions ("clean commit passes", "hook self-scan exclusion
# honored") go RED for unrelated reasons -- a missing kcov binary, a clippy
# warning in another lane -- even though the credential scan was perfect.
# So the pass-path assertions PATH-stub `python3` to succeed (via
# ${pass_stub_dir}), isolating step 2. The P0 cred-hygiene scan (step 1)
# runs FULLY UNSTUBBED in every test; the reject-path tests short-circuit
# at step 1 and never reach the stub; TEST 7 supplies its OWN fail-stub.

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

# D-CONV-6 (2026-07-04): abort BEFORE any git mutation if the working tree is
# dirty. cleanup() below runs `git reset -q --hard "$orig_head"`
# unconditionally on EXIT -- against a dirty tree that would silently
# discard uncommitted work. Runs in CI too (ci.yml's test job invokes this
# script), but a dirty tree there is a checkout anomaly worth failing loud
# on, not a case to special-case around.
if [[ -n "$(git status --porcelain)" ]]; then
  printf '%b\n' "${RED}✖ working tree is dirty -- aborting before any test fixtures are created.${NC}" >&2
  printf '%b\n' "${RED}  This harness's cleanup trap runs \`git reset --hard\` unconditionally on exit, which would discard your uncommitted changes.${NC}" >&2
  printf '%b\n' "${RED}  Commit or stash (git stash -u) your changes, then re-run: bash .githooks/test-pre-push.sh${NC}" >&2
  git status --porcelain >&2
  exit 1
fi

# Save current branch + HEAD so cleanup() can restore. The test
# detaches HEAD during execution; cleanup must return us to the
# original branch if we started on one, not leave us in detached
# HEAD state (which would silently swallow any subsequent commits).
readonly orig_head="$(git rev-parse HEAD)"
readonly orig_branch="$(git symbolic-ref --short -q HEAD || echo '')"
readonly tmp_branch="test-pre-push-$$-$RANDOM"

# Pass-through python3 stub. The hook's step 2 shells out to
# `python3 quality/runners/run.py --cadence pre-push` (clippy, mkdocs,
# shell-coverage, ...) -- gates that own their own CI jobs and are
# irrelevant to a *credential*-hook unit test. The pass-path assertions
# below run the hook with this stub first on PATH so step 2 succeeds and
# the test grades ONLY credential behavior (step 1, run unstubbed) + the
# hook's exit plumbing. See the scope note in the file header. The stub is
# invoked ONLY by the hook's step 2 -- the reject-path tests exit at step 1
# before python3 is ever called, and TEST 7 overlays its own fail-stub.
readonly pass_stub_dir="$(mktemp -d)"
cat > "${pass_stub_dir}/python3" <<'PASS_STUB'
#!/usr/bin/env bash
# Pass-through python3 stub for .githooks/test-pre-push.sh -- pretends the
# pre-push quality runner (hook step 2) passed so the credential-hook unit
# test is isolated from clippy/mkdocs/shell-coverage. Does NOT touch the P0
# cred-hygiene scan (step 1, pure bash, always real).
exit 0
PASS_STUB
chmod +x "${pass_stub_dir}/python3"

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
  rm -rf "${pass_stub_dir:-}" 2>/dev/null || true
}
trap cleanup EXIT

# Helper: run the hook with a synthesized push-ref-range for HEAD.
# Arg 1: label, Arg 2: expected exit code (0 = pass, 1 = reject).
run_and_check() {
  local label="$1"
  local expected="$2"
  local actual=0
  # PATH-stub python3 so the hook's step-2 quality runner (clippy, mkdocs,
  # shell-coverage) short-circuits to success -- this unit test grades the
  # credential-hygiene gate (step 1, unstubbed) + hook exit plumbing, not
  # the whole pre-push suite. Reject-path tests exit at step 1 and never
  # reach python3, so the stub is a no-op for them.
  echo "refs/heads/test HEAD HEAD^{commit}~1 $(git rev-parse HEAD^)" \
    | PATH="${pass_stub_dir}:$PATH" bash "$hook" > /tmp/test-pre-push.out 2>&1 || actual=$?
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
# Commit an innocuous fixture on a detached HEAD and scan its range. Uses
# a purpose-built commit (not the checkout tip) so the test works on a
# shallow CI clone (actions/checkout depth 1) -- `git rev-parse HEAD^` of
# the checkout tip is a "fatal: ambiguous argument 'HEAD^'" there, whereas
# the fixture commit's parent is the fetched tip and always resolves. Also
# gives the scanner a real single-file diff to clear, not an empty range.
git checkout -q --detach HEAD
echo 'a perfectly ordinary line with no credentials in it' > .test-pre-push-fixture.txt
git add .test-pre-push-fixture.txt
git -c user.email=test@test -c user.name=test commit -q -m "test: clean fixture commit"
if ! run_and_check "clean commit passes" 0; then
  fails=$((fails + 1))
fi
git reset -q --hard HEAD^

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

# --- TEST 5b: commit with a Google API key (AIza...) is rejected. -----
# Regression coverage for secret-scanning alert #1 (2026-07-04): the
# alert flagged an AIza-shaped string in a bootstrap-seed .playwright-mcp
# log (owner triaged: FALSE POSITIVE — no real key), which exposed that
# the committed cred-hygiene gate had no Google pattern. Fixture is a
# SYNTHETIC AIza-shaped string (39 chars: prefix + 35), never a real key.
# The key is assembled at RUNTIME from two halves so no AIza-shaped
# literal exists in this source file — secret scanners (gitleaks etc.)
# pattern-match any AIza-shaped string, real or fake, and would
# otherwise block the push that ships this harness.
fake_google_prefix='AIza'
fake_google_rest='SyDfakeFAKEfake0123456789abcdefghij'
printf 'GOOGLE_KEY=%s%s\n' "$fake_google_prefix" "$fake_google_rest" > .test-pre-push-fixture.txt
git add .test-pre-push-fixture.txt
git -c user.email=test@test -c user.name=test commit -q -m "test: inject fake Google API key"
if ! run_and_check "Google AIza API key rejected" 1; then
  fails=$((fails + 1))
fi
git reset -q --hard HEAD^

# --- TEST 5c: commit with an AWS access key id (AKIA...) is rejected. --
# D-CONV-4 (2026-07-04): coverage for the AKIA[0-9A-Z]{16} pattern added
# to cred-hygiene.sh. Fixture is a SYNTHETIC AKIA-shaped id assembled at
# RUNTIME from two halves so no AKIA-shaped literal exists in this source
# file — the gitleaks CI backstop (aws-access-token rule) pattern-matches
# any AKIA-shaped string and would otherwise block the push shipping this
# harness. Same invisibility-by-construction trick as TEST 5b.
fake_aws_prefix='AKIA'
fake_aws_rest='IOSFODNN7EXAMPLE0'   # 17 chars -> 16 taken by the {16} quantifier
printf 'AWS_ACCESS_KEY_ID=%s%s\n' "$fake_aws_prefix" "$fake_aws_rest" > .test-pre-push-fixture.txt
git add .test-pre-push-fixture.txt
git -c user.email=test@test -c user.name=test commit -q -m "test: inject fake AWS access key id"
if ! run_and_check "AWS AKIA access key rejected" 1; then
  fails=$((fails + 1))
fi
git reset -q --hard HEAD^

# --- TEST 5d: commit with a PEM private-key header is rejected. --------
# D-CONV-4 (2026-07-04): coverage for the
# -----BEGIN( RSA| EC| OPENSSH)? PRIVATE KEY----- pattern. The header is
# assembled at RUNTIME from two halves so no full PEM header literal
# exists in this source file — gitleaks' private-key rule matches the
# complete header, so a literal would block the push shipping this harness.
pem_head='-----BEGIN'
pem_tail=' RSA PRIVATE KEY-----'
printf '%s%s\n' "$pem_head" "$pem_tail" > .test-pre-push-fixture.txt
git add .test-pre-push-fixture.txt
git -c user.email=test@test -c user.name=test commit -q -m "test: inject fake PEM private-key header"
if ! run_and_check "PEM private-key header rejected" 1; then
  fails=$((fails + 1))
fi
git reset -q --hard HEAD^

# --- TEST 6: hook file itself is excluded from self-scan. -------------
# The scanner (quality/gates/structure/cred-hygiene.sh) lists .githooks/
# in EXCLUDE_DIRS precisely so the hook's own PATTERNS body -- and any
# marker we append here -- does not self-match. To make this assertion
# actually EXERCISE that exclusion (a short non-matching marker would pass
# whether or not .githooks/ were excluded, making the test name a lie), we
# append a fixture that WOULD trip the ATATT3 pattern if the file weren't
# excluded: 30+ chars after the prefix, so exit 0 here proves the exclusion
# fired, not that the fixture was harmless. The token is assembled at
# RUNTIME from two halves so no matching literal lives in this source file
# (same invisibility-by-construction trick as TESTS 5b/5c/5d) -- gitleaks'
# CI backstop would otherwise flag the harness itself.
orig_hook_contents="$(cat "$hook")"
selfscan_prefix='ATATT3'
selfscan_rest='xFfWELrSelfScanExclusionFixture01234'  # 35 chars -> matches {20,}
printf '\n# Test marker — self-scan exclusion fixture (%s%s)\n' \
  "$selfscan_prefix" "$selfscan_rest" >> "$hook"
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
