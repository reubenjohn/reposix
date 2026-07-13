#!/usr/bin/env bash
# quality/gates/structure/file-size-limits.selftest.sh
#
# Self-test for file-size-limits.sh's two-tier behavior. Builds a throwaway
# git repo under /tmp (never the shared repo — leaf isolation), seeds fixture
# files sized to land in each band, runs the gate against it, and asserts:
#   (a) a 75–99% file → WARN line present AND exit 0 (no over-budget file)
#   (b) a ≥100% file, NO --warn-only → exit 1 (block contract intact)
#   (c) the same ≥100% file WITH --warn-only → exit 0 (waiver path intact)
#   (d) the ≥75% WARN summary is emitted in BOTH flag modes (flag-independent)
#   (e) >12 band files → top-12 + "… and N more at ≥75%" overflow line
#
# Run: bash quality/gates/structure/file-size-limits.selftest.sh
# Exit 0 = all assertions pass; exit 1 = a regression.
set -euo pipefail

GATE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/file-size-limits.sh"
[[ -x "$GATE" ]] || { echo "FATAL: gate not found/executable at $GATE" >&2; exit 1; }

WORK="$(mktemp -d "${TMPDIR:-/tmp}/file-size-selftest.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT

# Throwaway repo — non-fixture identity so the leaf-isolation guard is a no-op.
# core.hooksPath=/dev/null keeps the temp repo hermetic (no inherited shared
# git hooks, which globally point at .githooks/ and would run on these commits).
git -C "$WORK" init -q
git -C "$WORK" config core.hooksPath /dev/null
git -C "$WORK" config user.name "selftest"
git -C "$WORK" config user.email "selftest@example.invalid"

mk() { head -c "$2" /dev/zero | tr '\0' 'a' > "$WORK/$1"; }
run() { ( cd "$WORK" && "$GATE" "$@" ) 2>"$WORK/.err"; }   # returns gate exit; stderr→.err

pass=0; fail=0
check() { # check <label> <cond-desc> <actual> <expected>
  if [[ "$3" == "$4" ]]; then echo "  PASS: $1 ($2=$3)"; pass=$((pass+1))
  else echo "  FAIL: $1 (expected $2=$4, got $3)"; fail=$((fail+1)); fi
}

echo "== Case (a): only 75–99% files → WARN present, exit 0 =="
mk warn_80.md 16000   # 16000/20000 = 80%
mk warn_95.md 19000   # 19000/20000 = 95%
git -C "$WORK" add -A && git -C "$WORK" commit -qm a
set +e; run; rc=$?; set -e
check "exit code" rc "$rc" 0
grep -q "WARN — 2 file(s) approaching size ceiling" "$WORK/.err" && w=yes || w=no
check "WARN header present" present "$w" yes
# highest pct first
head -2 "$WORK/.err" | grep -q "warn_95.md — 19000/20000 chars (95%)" && s=yes || s=no
check "band sorted desc (95% first line)" ok "$s" yes
echo "  --- observed stderr ---"; sed 's/^/  /' "$WORK/.err"

echo "== Case (b): add a ≥100% file, NO --warn-only → exit 1 =="
mk over_125.md 25000  # 25000/20000 = 125%
git -C "$WORK" add -A && git -C "$WORK" commit -qm b
set +e; run; rc=$?; set -e
check "exit code" rc "$rc" 1
grep -q "over_125.md is 25000 chars (limit: 20000)" "$WORK/.err" && b=yes || b=no
check "over-budget block line present" present "$b" yes
grep -q "WARN — 2 file(s) approaching" "$WORK/.err" && w=yes || w=no
check "WARN still emitted alongside block" present "$w" yes
echo "  --- observed stderr ---"; sed 's/^/  /' "$WORK/.err"

echo "== Case (c): same ≥100% file WITH --warn-only → exit 0 =="
set +e; run --warn-only; rc=$?; set -e
check "exit code" rc "$rc" 0
grep -q "(--warn-only mode; exiting 0)" "$WORK/.err" && m=yes || m=no
check "warn-only mode line present" present "$m" yes
grep -q "WARN — 2 file(s) approaching" "$WORK/.err" && w=yes || w=no
check "WARN independent of --warn-only" present "$w" yes
echo "  --- observed stderr ---"; sed 's/^/  /' "$WORK/.err"

echo "== Case (e): >12 band files → overflow line =="
WORK2="$(mktemp -d "${TMPDIR:-/tmp}/file-size-selftest2.XXXXXX")"
git -C "$WORK2" init -q
git -C "$WORK2" config core.hooksPath /dev/null
git -C "$WORK2" config user.name selftest
git -C "$WORK2" config user.email selftest@example.invalid
for i in $(seq -w 1 13); do head -c 16000 /dev/zero | tr '\0' 'a' > "$WORK2/band_$i.md"; done
git -C "$WORK2" add -A && git -C "$WORK2" commit -qm e
set +e; ( cd "$WORK2" && "$GATE" ) 2>"$WORK2/.err"; rc=$?; set -e
check "exit code" rc "$rc" 0
grep -q "… and 1 more at ≥75%" "$WORK2/.err" && o=yes || o=no
check "overflow line present" present "$o" yes
shown=$(grep -cE '^  band_[0-9]+\.md' "$WORK2/.err")
check "exactly 12 band lines shown" count "$shown" 12
rm -rf "$WORK2"

echo
echo "RESULT: $pass passed, $fail failed"
[[ $fail -eq 0 ]] || exit 1
