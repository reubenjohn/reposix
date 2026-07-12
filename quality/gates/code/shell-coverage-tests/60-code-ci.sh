#!/usr/bin/env bash
# 60-code-ci.sh — drive quality/gates/code/ci-green-on-main.sh through every
# branch under kcov, WITHOUT touching the network. A stub `gh` on PATH returns
# canned `gh run list` JSON so each verdict path (success / failure /
# in-progress / no-runs / auth-fail / gh-missing) executes deterministically.
#
# Doubles as the behavioral test: each invocation asserts the target's exit code
# maps to the documented PASS(0)/FAIL(1)/NOT-VERIFIED(75) contract. A wrong exit
# here is a real gate bug -- but per the harness contract (see 00-probe.sh) this
# harness still exits 0 so the coverage run is not aborted; the assertion failure
# is printed loudly to stderr for the human/CI log.
#
# Harness contract: exit 0 regardless of target exits.
set -eu

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
TARGET="$REPO_ROOT/quality/gates/code/ci-green-on-main.sh"

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

# --- stub gh: prints $GH_STUB_OUTPUT, or exits $GH_STUB_RC if set nonzero ---
STUB_BIN="$WORK/bin"
mkdir -p "$STUB_BIN"
cat > "$STUB_BIN/gh" <<'STUB'
#!/usr/bin/env bash
if [ "${GH_STUB_RC:-0}" -ne 0 ]; then
  exit "$GH_STUB_RC"
fi
printf '%s\n' "${GH_STUB_OUTPUT:-[]}"
STUB
chmod +x "$STUB_BIN/gh"

fails=0
# expect_exit <label> <expected_rc> — run TARGET with current env, check rc.
expect_exit() {
  local label="$1" want="$2" got=0
  "$TARGET" >/dev/null 2>&1 || got=$?
  if [ "$got" -ne "$want" ]; then
    echo "60-code-ci: FAIL [$label]: expected exit $want, got $got" >&2
    fails=$((fails + 1))
  fi
}

# Branches 1-4 + auth-fail run with the stub gh first on PATH.
export PATH="$STUB_BIN:$PATH"

GH_STUB_RC=0 GH_STUB_OUTPUT='[{"databaseId":1,"conclusion":"success","status":"completed"}]' \
  expect_exit "latest-green" 0
GH_STUB_RC=0 GH_STUB_OUTPUT='[{"databaseId":1,"conclusion":"failure","status":"completed"}]' \
  expect_exit "latest-red" 1
GH_STUB_RC=0 GH_STUB_OUTPUT='[{"databaseId":1,"conclusion":null,"status":"in_progress"}]' \
  expect_exit "in-progress" 75
GH_STUB_RC=0 GH_STUB_OUTPUT='[]' \
  expect_exit "no-runs" 75
GH_STUB_RC=7 GH_STUB_OUTPUT='' \
  expect_exit "auth-fail" 75

# gh-missing branch: a minimal PATH that has the tools the target needs
# (git, python3, date, mkdir, bash) but NO gh, so `command -v gh` fails.
MINI_BIN="$WORK/mini"
mkdir -p "$MINI_BIN"
for tool in bash git python3 date mkdir printf cat rm; do
  p="$(command -v "$tool" 2>/dev/null || true)"
  [ -n "$p" ] && ln -sf "$p" "$MINI_BIN/$tool"
done
env -i PATH="$MINI_BIN" HOME="$HOME" bash -c '"$0"' "$TARGET" >/dev/null 2>&1 && missing_rc=0 || missing_rc=$?
if [ "$missing_rc" -ne 75 ]; then
  echo "60-code-ci: FAIL [gh-missing]: expected exit 75, got $missing_rc" >&2
  fails=$((fails + 1))
fi

if [ "$fails" -eq 0 ]; then
  echo "60-code-ci: all 6 ci-green-on-main branches asserted OK"
else
  echo "60-code-ci: $fails branch assertion(s) FAILED (see above)" >&2
fi

# Harness always exits 0 (coverage contract); real gate failures surface via the
# gate's own catalog row, not this coverage harness.
exit 0
