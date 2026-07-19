#!/usr/bin/env bash
# quality/gates/docs-alignment/walk-block-summary.selftest.sh
#
# DRAIN-17 regression: prove the docs-alignment walk BLOCK output leads with a
# SUMMARY that names the actual blocking row-STATE(s) + the specific row id(s)
# in each state -- NOT just an alignment/coverage RATIO (the pre-DRAIN-17 first
# line said "N below floor", which told a developer nothing about which row to
# fix). Exercises the REAL compiled binary end-to-end through the same
# quality/gates/docs-alignment/walk.sh wrapper the pre-push gate uses.
#
# Leaf isolation: the synthetic catalog lives in a throwaway mktemp file (never
# a shared-repo write); walk.sh's `--catalog <path>` is respected as-is (see the
# wrapper's CALLER_CATALOG branch), so the committed doc-alignment.json is never
# touched. The synthetic rows carry MISSING_TEST / RETIRE_PROPOSED verdicts with
# no test bindings + empty source_hashes, so the walker preserves those blocking
# states deterministically without hashing anything on disk.
#
# The catalog's floors are pinned to 0.0 so ONLY the per-row states block --
# making "the summary names states, not a ratio" a clean assertion (no floor
# ratio line fires to muddy it).
#
# RED-when-ratio / GREEN-when-states: this selftest FAILS against a binary whose
# walk still leads with only the ratio (no "docs-alignment BLOCK:" state summary,
# no per-state row-id enumeration), and PASSES against the DRAIN-17 binary.
#
# Run: bash quality/gates/docs-alignment/walk-block-summary.selftest.sh
# Exit 0 = all assertions pass; exit 1 = a regression.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WALK="${HERE}/walk.sh"
[[ -x "$WALK" ]] || { echo "FATAL: walk.sh not found/executable at $WALK" >&2; exit 1; }

REPO_ROOT="$(git -C "$HERE" rev-parse --show-toplevel)"
# Run from the repo root so any source-file resolution the walker attempts
# (coverage counters) resolves the same way the real gate does.
cd "$REPO_ROOT"

# --- synthetic catalog: 2 MISSING_TEST rows + 1 RETIRE_PROPOSED row -----------
TMP_CATALOG="$(mktemp -t drain17-walk-block-summary.XXXXXX.json)"
TMP_ERR="$(mktemp -t drain17-walk-block-summary.XXXXXX.err)"
trap 'rm -f "$TMP_CATALOG" "$TMP_ERR"' EXIT

cat > "$TMP_CATALOG" <<'JSON'
{
  "schema_version": "1.0",
  "dimension": "docs-alignment",
  "summary": {
    "claims_total": 3,
    "claims_bound": 0,
    "claims_missing_test": 2,
    "claims_retire_proposed": 1,
    "claims_retired": 0,
    "alignment_ratio": 1.0,
    "floor": 0.0,
    "trend_30d": "flat",
    "last_walked": null,
    "coverage_floor": 0.0
  },
  "rows": [
    { "id": "selftest/missing-alpha", "claim": "claim alpha", "source": {"file": "README.md", "line_start": 1, "line_end": 1}, "last_verdict": "MISSING_TEST" },
    { "id": "selftest/missing-beta",  "claim": "claim beta",  "source": {"file": "README.md", "line_start": 2, "line_end": 2}, "last_verdict": "MISSING_TEST" },
    { "id": "selftest/retire-gamma",  "claim": "claim gamma", "source": {"file": "README.md", "line_start": 3, "line_end": 3}, "last_verdict": "RETIRE_PROPOSED" }
  ]
}
JSON

rc=0
bash "$WALK" --catalog "$TMP_CATALOG" >/dev/null 2>"$TMP_ERR" || rc=$?

pass=0; fail=0
check_contains() { # <label> <needle>
  if grep -qF -- "$2" "$TMP_ERR"; then echo "  PASS: $1 (found: $2)"; pass=$((pass+1))
  else echo "  FAIL: $1 (NOT found: $2)"; fail=$((fail+1)); fi
}

echo "== DRAIN-17 walk BLOCK-summary selftest =="
echo "--- captured walk stderr ---"
sed 's/^/  | /' "$TMP_ERR"
echo "----------------------------"

# 1. The walk BLOCKs (a blocking row-state -> exit 1).
if [[ "$rc" -eq 1 ]]; then echo "  PASS: walk exits 1 on blocking rows (rc=$rc)"; pass=$((pass+1))
else echo "  FAIL: expected walk exit 1, got rc=$rc"; fail=$((fail+1)); fi

# 2. A summary header naming the block + row count leads the output.
check_contains "summary header names BLOCK" "docs-alignment BLOCK:"
check_contains "summary header names row count" "3 row(s) blocking"

# 3. Each distinct blocking STATE is named with its count.
check_contains "names MISSING_TEST x2"    "MISSING_TEST x2"
check_contains "names RETIRE_PROPOSED x1" "RETIRE_PROPOSED x1"

# 4. The specific row id(s) appear, grouped under their state.
check_contains "names row id missing-alpha" "selftest/missing-alpha"
check_contains "names row id missing-beta"  "selftest/missing-beta"
check_contains "names row id retire-gamma"  "selftest/retire-gamma"

# 5. North-star three-part teaching (teach fix / alternative / recovery cmd).
check_contains "teaches the fix"          "Fix:"
check_contains "suggests the alternative" "Alternative:"
check_contains "gives a copy-paste recovery command" "reposix-quality doc-alignment mark-missing-test"

# 6. RED-when-ratio guard: the FIRST stderr line must be the state summary, NOT
#    a bare "below floor" ratio (the pre-DRAIN-17 lead). A binary that only
#    printed the ratio line first fails here.
first_line="$(head -n1 "$TMP_ERR")"
if [[ "$first_line" == docs-alignment\ BLOCK:* ]]; then
  echo "  PASS: first line is the state summary, not a ratio (\"$first_line\")"; pass=$((pass+1))
else
  echo "  FAIL: first line must be the state summary, got: \"$first_line\""; fail=$((fail+1))
fi
if grep -q "below floor" "$TMP_ERR"; then
  echo "  FAIL: floors pinned to 0.0 must not emit a ratio 'below floor' line"; fail=$((fail+1))
else
  echo "  PASS: no bare 'below floor' ratio line (states are the block reason)"; pass=$((pass+1))
fi

echo "== summary: ${pass} passed, ${fail} failed =="
[[ "$fail" -eq 0 ]] || exit 1
echo "DRAIN-17 walk BLOCK-summary selftest: OK"
