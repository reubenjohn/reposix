#!/usr/bin/env bash
# quality/gates/perf/latency-bench/regen-guard.selftest.sh
#
# Self-test for regen_guard_check (regen-guard.sh) -- the guard that stops
# emit-markdown.sh from clobbering the CI-canonical sections of
# docs/benchmarks/latency.md. Builds throwaway fixtures under /tmp (never
# the shared repo -- leaf isolation) and asserts:
#   (a) no file at the OUT path            -> safe (rc=0), no stderr
#   (b) file exists, no protected marker    -> safe (rc=0)
#   (c) file exists WITH protected marker, no override -> refuse (rc=1),
#       teaching error names what/why/recovery, file left byte-identical
#   (d) same as (c) but with the override env var set  -> safe (rc=0)
#   (e) the REAL docs/benchmarks/latency.md as committed actually trips
#       the guard -- catches accidental marker removal in that file
#   (f) workflow wiring: ci.yml regenerates to a scratch OUT (artifact-only)
#       and the cron workflow carries the sanctioned override env var
#
# Run: bash quality/gates/perf/latency-bench/regen-guard.selftest.sh
# Exit 0 = all assertions pass; exit 1 = a regression.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../../.." && pwd)"
GUARD="${SCRIPT_DIR}/regen-guard.sh"
[[ -f "$GUARD" ]] || { echo "FATAL: guard not found at $GUARD" >&2; exit 1; }

# shellcheck source=regen-guard.sh
source "$GUARD"

WORK="$(mktemp -d "${TMPDIR:-/tmp}/regen-guard-selftest.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT

pass=0; fail=0
check() { # check <label> <actual> <expected>
  if [[ "$2" == "$3" ]]; then echo "  PASS: $1 ($2)"; pass=$((pass+1))
  else echo "  FAIL: $1 (expected $3, got $2)"; fail=$((fail+1)); fi
}

echo "== Case (a): no file at OUT path -> safe =="
UNSET_OUT="${WORK}/does-not-exist.md"
set +e; regen_guard_check "$UNSET_OUT" 2>"$WORK/a.err"; rc=$?; set -e
check "exit code" "$rc" 0
check "no stderr" "$(wc -c <"$WORK/a.err")" 0

echo "== Case (b): file exists, no protected marker -> safe =="
UNMARKED="${WORK}/unmarked.md"
printf '# just a doc\n\nnothing protected here.\n' >"$UNMARKED"
set +e; regen_guard_check "$UNMARKED" 2>"$WORK/b.err"; rc=$?; set -e
check "exit code" "$rc" 0
check "no stderr" "$(wc -c <"$WORK/b.err")" 0

echo "== Case (c): protected marker present, no override -> refuse =="
PROTECTED="${WORK}/protected.md"
printf '# fixture doc\n\n%s -- fixture protected-end -->\n' "$REGEN_GUARD_BEGIN_MARKER" >"$PROTECTED"
before_sum="$(sha256sum "$PROTECTED" | awk '{print $1}')"
unset REPOSIX_LATENCY_BENCH_ALLOW_CANONICAL_OVERWRITE || true
set +e; regen_guard_check "$PROTECTED" 2>"$WORK/c.err"; rc=$?; set -e
after_sum="$(sha256sum "$PROTECTED" | awk '{print $1}')"
check "exit code" "$rc" 1
check "file untouched" "$after_sum" "$before_sum"
grep -q "refusing to regenerate" "$WORK/c.err" && w="yes" || w="no"
check "teaches WHAT was protected" "$w" yes
grep -q "why:" "$WORK/c.err" && w="yes" || w="no"
check "teaches WHY" "$w" yes
grep -q "${REGEN_GUARD_OVERRIDE_VAR}=1 bash quality/gates/perf/latency-bench.sh" "$WORK/c.err" && w="yes" || w="no"
check "teaches copy-paste override recovery" "$w" yes
grep -q "OUT=/tmp/latency-preview.md bash quality/gates/perf/latency-bench.sh" "$WORK/c.err" && w="yes" || w="no"
check "teaches copy-paste preview recovery" "$w" yes
echo "  --- observed stderr ---"; sed 's/^/  /' "$WORK/c.err"

echo "== Case (d): protected marker present, WITH override -> safe =="
set +e
REPOSIX_LATENCY_BENCH_ALLOW_CANONICAL_OVERWRITE=1 regen_guard_check "$PROTECTED" 2>"$WORK/d.err"
rc=$?
set -e
check "exit code" "$rc" 0

echo "== Case (e): the real docs/benchmarks/latency.md trips the guard =="
REAL_LATENCY="${REPO_ROOT}/docs/benchmarks/latency.md"
if [[ -f "$REAL_LATENCY" ]]; then
  set +e; regen_guard_check "$REAL_LATENCY" 2>"$WORK/e.err"; rc=$?; set -e
  check "exit code (real fixture is protected)" "$rc" 1
else
  echo "  SKIP: $REAL_LATENCY not found (unexpected repo layout)"
fi

echo "== Case (f): workflow wiring keeps the guard out of CI =="
CI_YML="${REPO_ROOT}/.github/workflows/ci.yml"
CRON_YML="${REPO_ROOT}/.github/workflows/bench-latency-cron.yml"
check "ci.yml exports scratch OUT" "$(grep -cF 'export OUT="${RUNNER_TEMP}/latency-preview.md"' "$CI_YML")" 1
check "ci.yml uploads scratch artifact" "$(grep -cF 'path: ${{ runner.temp }}/latency-preview.md' "$CI_YML")" 1
check "cron sets canonical-overwrite override" "$(grep -cF 'REPOSIX_LATENCY_BENCH_ALLOW_CANONICAL_OVERWRITE: "1"' "$CRON_YML")" 1

echo
echo "regen-guard.selftest: ${pass} passed, ${fail} failed"
[[ "$fail" -eq 0 ]]
