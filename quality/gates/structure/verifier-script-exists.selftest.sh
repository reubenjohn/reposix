#!/usr/bin/env bash
# quality/gates/structure/verifier-script-exists.selftest.sh
#
# Self-test proving verifier-script-exists.sh's GRADED-OUTCOME scope (refined
# 2026-07-18, P123 close) as a FULL truth table. Mirrors file-size-limits.
# selftest.sh's shape: a throwaway /tmp git repo (never the shared repo -- leaf
# isolation), a fixture catalog under quality/catalogs/, the gate invoked via its
# real absolute path against the fixture repo's toplevel. The gate does
# `cd "$(git rev-parse --show-toplevel)"`, so pointing it at the throwaway repo
# scopes its catalog scan to ONLY that fixture -- the real catalogs are untouched.
#
# TRUTH TABLE (every assertion runs):
#   status  | verifier.script   | verdict
#   --------+-------------------+--------------------------------------
#   PASS    | missing file      | VIOLATION (broken script)
#   FAIL    | missing file      | VIOLATION (broken script)
#   PARTIAL | missing file      | VIOLATION (broken script)
#   PASS    | present, non-exec | VIOLATION (no +x bit)
#   PASS    | null              | VIOLATION (unbacked PASS -- no verifier)
#   WAIVED  | missing file      | EXEMPT    (asserts no green)
#   NOT-VER | missing file      | EXEMPT    (asserts no green)
#   FAIL    | null              | EXEMPT    (asserts no green; no unbacked PASS)
#   PASS    | present, +x       | in-scope PASS (all-good)
#
# PASS+null-script is the P123 residual-hole close (was EXEMPT, now VIOLATION --
# an unbacked PASS; rationale in the gate header). FAIL/PARTIAL+null stay EXEMPT.
#
# Cases: (a) all-good fixture (single PASS +x row) -> exit 0; (b) combined
# fixture (the full table above) -> exit 1 with EXACTLY the 5 violation rows named
# (4 broken-script + 1 unbacked-PASS) and every exempt row absent.
#
# Run: bash quality/gates/structure/verifier-script-exists.selftest.sh
# Exit 0 = all assertions pass; exit 1 = a regression.
set -euo pipefail

GATE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/verifier-script-exists.sh"
[[ -x "$GATE" ]] || { echo "FATAL: gate not found/executable at $GATE" >&2; exit 1; }

pass=0; fail=0
check() { # check <label> <cond-desc> <actual> <expected>
  if [[ "$3" == "$4" ]]; then echo "  PASS: $1 ($2=$3)"; pass=$((pass+1))
  else echo "  FAIL: $1 (expected $2=$4, got $3)"; fail=$((fail+1)); fi
}
check_contains() { # check_contains <label> <haystack-file> <needle>
  if grep -qF -- "$3" "$2"; then echo "  PASS: $1 (found: $3)"; pass=$((pass+1))
  else echo "  FAIL: $1 (NOT found: $3)"; fail=$((fail+1)); fi
}
check_not_contains() { # check_not_contains <label> <haystack-file> <needle>
  if grep -qF -- "$3" "$2"; then echo "  FAIL: $1 (unexpectedly found: $3)"; fail=$((fail+1))
  else echo "  PASS: $1 (correctly absent: $3)"; pass=$((pass+1))
  fi
}

init_fixture_repo() { # init_fixture_repo <dir>
  local dir="$1"
  git -C "$dir" init -q
  git -C "$dir" config core.hooksPath /dev/null
  git -C "$dir" config user.name selftest
  git -C "$dir" config user.email selftest@example.invalid
  mkdir -p "$dir/quality/catalogs" "$dir/quality/gates/structure"
}

run() { # run <dir> -- invokes the real gate scoped to <dir>'s toplevel
  local dir="$1"
  ( cd "$dir" && "$GATE" ) 2>"$dir/.err" >"$dir/.out"
}

# ---------------------------------------------------------------------------
echo "== Case (a): all-good fixture (single PASS row, real +x script) -> exit 0 =="
WORK_GOOD="$(mktemp -d "${TMPDIR:-/tmp}/verifier-script-exists-selftest-good.XXXXXX")"
trap 'rm -rf "$WORK_GOOD" "${WORK_COMBINED:-}"' EXIT

init_fixture_repo "$WORK_GOOD"
cat > "$WORK_GOOD/quality/gates/structure/fixture-good.sh" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
chmod +x "$WORK_GOOD/quality/gates/structure/fixture-good.sh"

cat > "$WORK_GOOD/quality/catalogs/fixture.json" <<'EOF'
{
  "$schema": "https://json-schema.org/draft-07/schema#",
  "comment": "selftest fixture -- all-good case",
  "dimension": "structure",
  "rows": [
    {
      "id": "structure/fixture-pass-good",
      "dimension": "structure",
      "status": "PASS",
      "verifier": {"script": "quality/gates/structure/fixture-good.sh", "args": [], "timeout_s": 30, "container": null}
    }
  ]
}
EOF
git -C "$WORK_GOOD" add -A && git -C "$WORK_GOOD" commit -qm a

set +e; run "$WORK_GOOD"; rc=$?; set -e
check "exit code" rc "$rc" 0
check_contains "PASS summary line present" "$WORK_GOOD/.out" "PASS: verifier-script-exists — 1 in-scope graded-outcome rows across 1 catalogs"
check_not_contains "no violation line for the good row" "$WORK_GOOD/.err" "structure/fixture-pass-good::"
echo "  --- observed stdout ---"; sed 's/^/  /' "$WORK_GOOD/.out"

# ---------------------------------------------------------------------------
echo "== Case (b): combined fixture (full truth table) -> exit 1, EXACTLY the 4 graded broken-script rows named =="
WORK_COMBINED="$(mktemp -d "${TMPDIR:-/tmp}/verifier-script-exists-selftest-combined.XXXXXX")"

init_fixture_repo "$WORK_COMBINED"

# GOOD -- real, executable (PASS + present +x).
cat > "$WORK_COMBINED/quality/gates/structure/fixture-good.sh" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
chmod +x "$WORK_COMBINED/quality/gates/structure/fixture-good.sh"

# NON-EXECUTABLE -- real file, +x bit deliberately unset (PASS + non-exec).
cat > "$WORK_COMBINED/quality/gates/structure/fixture-nonexec.sh" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
chmod -x "$WORK_COMBINED/quality/gates/structure/fixture-nonexec.sh"

# All *-MISSING.sh scripts are deliberately never created (missing-file class).
# The null-script rows' verifier.script is explicitly null (PASS -> VIOLATION,
# FAIL -> EXEMPT).

cat > "$WORK_COMBINED/quality/catalogs/fixture.json" <<'EOF'
{
  "$schema": "https://json-schema.org/draft-07/schema#",
  "comment": "selftest fixture -- full graded-outcome truth table",
  "dimension": "structure",
  "rows": [
    {
      "id": "structure/fixture-pass-good",
      "dimension": "structure",
      "status": "PASS",
      "verifier": {"script": "quality/gates/structure/fixture-good.sh", "args": [], "timeout_s": 30, "container": null}
    },
    {
      "id": "structure/fixture-pass-missing",
      "dimension": "structure",
      "status": "PASS",
      "verifier": {"script": "quality/gates/structure/fixture-pass-MISSING.sh", "args": [], "timeout_s": 30, "container": null}
    },
    {
      "id": "structure/fixture-fail-missing",
      "dimension": "structure",
      "status": "FAIL",
      "verifier": {"script": "quality/gates/structure/fixture-fail-MISSING.sh", "args": [], "timeout_s": 30, "container": null}
    },
    {
      "id": "structure/fixture-partial-missing",
      "dimension": "structure",
      "status": "PARTIAL",
      "verifier": {"script": "quality/gates/structure/fixture-partial-MISSING.sh", "args": [], "timeout_s": 30, "container": null}
    },
    {
      "id": "structure/fixture-pass-nonexec",
      "dimension": "structure",
      "status": "PASS",
      "verifier": {"script": "quality/gates/structure/fixture-nonexec.sh", "args": [], "timeout_s": 30, "container": null}
    },
    {
      "id": "structure/fixture-waived-missing",
      "dimension": "structure",
      "status": "WAIVED",
      "verifier": {"script": "quality/gates/structure/fixture-waived-MISSING.sh", "args": [], "timeout_s": 30, "container": null}
    },
    {
      "id": "structure/fixture-notverified-missing",
      "dimension": "structure",
      "status": "NOT-VERIFIED",
      "verifier": {"script": "quality/gates/structure/fixture-notverified-MISSING.sh", "args": [], "timeout_s": 30, "container": null}
    },
    {
      "id": "structure/fixture-pass-nullscript",
      "dimension": "structure",
      "status": "PASS",
      "verifier": {"script": null, "args": [], "timeout_s": 30, "container": null}
    },
    {
      "id": "structure/fixture-fail-nullscript",
      "dimension": "structure",
      "status": "FAIL",
      "verifier": {"script": null, "args": [], "timeout_s": 30, "container": null}
    }
  ]
}
EOF
git -C "$WORK_COMBINED" add -A && git -C "$WORK_COMBINED" commit -qm b

set +e; run "$WORK_COMBINED"; rc=$?; set -e
check "exit code" rc "$rc" 1
# --- broken-script rows: MUST be named (violations) ---
check_contains "PASS+missing named"    "$WORK_COMBINED/.err" "structure/fixture-pass-missing::quality/gates/structure/fixture-pass-MISSING.sh::file does not exist"
check_contains "FAIL+missing named"    "$WORK_COMBINED/.err" "structure/fixture-fail-missing::quality/gates/structure/fixture-fail-MISSING.sh::file does not exist"
check_contains "PARTIAL+missing named" "$WORK_COMBINED/.err" "structure/fixture-partial-missing::quality/gates/structure/fixture-partial-MISSING.sh::file does not exist"
check_contains "PASS+non-exec named"   "$WORK_COMBINED/.err" "structure/fixture-pass-nonexec::quality/gates/structure/fixture-nonexec.sh::not executable::chmod +x"
# --- PASS+null-script: residual-hole close -- MUST be FLAGGED (unbacked PASS) ---
check_contains "PASS+null-script flagged" "$WORK_COMBINED/.err" "structure/fixture-pass-nullscript::(null)::status:PASS but declares no verifier.script -- an unbacked PASS"
# --- rows that assert no green: MUST be EXEMPT (never named) ---
check_not_contains "WAIVED+missing exempt"       "$WORK_COMBINED/.err" "structure/fixture-waived-missing::"
check_not_contains "NOT-VERIFIED+missing exempt" "$WORK_COMBINED/.err" "structure/fixture-notverified-missing::"
check_not_contains "FAIL+null-script exempt"     "$WORK_COMBINED/.err" "structure/fixture-fail-nullscript::"
check_not_contains "good row not named"          "$WORK_COMBINED/.err" "structure/fixture-pass-good::"
# --- exactly 5 violations (4 broken-script + 1 unbacked-PASS), 3 exempt, 9 seen ---
check_contains "violation count header (5)" "$WORK_COMBINED/.err" "FAIL (structure/verifier-script-exists): 5 violation(s)"
check_contains "exempt count in header (3)" "$WORK_COMBINED/.err" "9 rows seen, 3 exempt"
echo "  --- observed stderr ---"; sed 's/^/  /' "$WORK_COMBINED/.err"

echo
echo "RESULT: $pass passed, $fail failed"
[[ $fail -eq 0 ]] || exit 1
