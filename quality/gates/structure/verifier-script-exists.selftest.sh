#!/usr/bin/env bash
# quality/gates/structure/verifier-script-exists.selftest.sh
#
# Self-test proving verifier-script-exists.sh's three violation classes plus
# the all-good pass path. Mirrors file-size-limits.selftest.sh's shape: a
# throwaway /tmp git repo (never the shared repo -- leaf isolation), a
# fixture catalog seeded under quality/catalogs/, the gate invoked via its
# real absolute path against the fixture repo's own toplevel. The gate
# itself does `cd "$(git rev-parse --show-toplevel)"`, so pointing it at a
# throwaway git repo scopes its `quality/catalogs/*.json` scan to ONLY that
# repo's fixture catalog -- the real repo's catalogs are never touched.
#
# Cases:
#   (a) all-good fixture (single row, real +x script) -> exit 0
#   (b) combined fixture (good + 3 violation rows) -> exit 1, AND each of
#       the missing-file / non-executable / missing-field rows is named
#       individually in the violation output (not just a generic count)
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
echo "== Case (a): all-good fixture (single row, real +x script) -> exit 0 =="
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
      "id": "structure/fixture-good",
      "dimension": "structure",
      "verifier": {"script": "quality/gates/structure/fixture-good.sh", "args": [], "timeout_s": 30, "container": null}
    }
  ]
}
EOF
git -C "$WORK_GOOD" add -A && git -C "$WORK_GOOD" commit -qm a

set +e; run "$WORK_GOOD"; rc=$?; set -e
check "exit code" rc "$rc" 0
check_contains "PASS summary line present" "$WORK_GOOD/.out" "PASS: verifier-script-exists — 1 rows across 1 catalogs"
check_not_contains "no violation line for the good row" "$WORK_GOOD/.err" "structure/fixture-good::"
echo "  --- observed stdout ---"; sed 's/^/  /' "$WORK_GOOD/.out"

# ---------------------------------------------------------------------------
echo "== Case (b): combined fixture (good + missing-file + non-exec + missing-field) -> exit 1, each named =="
WORK_COMBINED="$(mktemp -d "${TMPDIR:-/tmp}/verifier-script-exists-selftest-combined.XXXXXX")"

init_fixture_repo "$WORK_COMBINED"

# (a) GOOD -- real, executable.
cat > "$WORK_COMBINED/quality/gates/structure/fixture-good.sh" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
chmod +x "$WORK_COMBINED/quality/gates/structure/fixture-good.sh"

# (c) NON-EXECUTABLE -- real file, +x bit deliberately unset.
cat > "$WORK_COMBINED/quality/gates/structure/fixture-nonexec.sh" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
chmod -x "$WORK_COMBINED/quality/gates/structure/fixture-nonexec.sh"

# (b) MISSING FILE -- fixture-missing.sh is deliberately never created.
# (d) MISSING/NULL FIELD -- verifier.script explicitly null.

cat > "$WORK_COMBINED/quality/catalogs/fixture.json" <<'EOF'
{
  "$schema": "https://json-schema.org/draft-07/schema#",
  "comment": "selftest fixture -- combined violation case",
  "dimension": "structure",
  "rows": [
    {
      "id": "structure/fixture-good",
      "dimension": "structure",
      "verifier": {"script": "quality/gates/structure/fixture-good.sh", "args": [], "timeout_s": 30, "container": null}
    },
    {
      "id": "structure/fixture-missing",
      "dimension": "structure",
      "verifier": {"script": "quality/gates/structure/fixture-missing-DOES-NOT-EXIST.sh", "args": [], "timeout_s": 30, "container": null}
    },
    {
      "id": "structure/fixture-nonexec",
      "dimension": "structure",
      "verifier": {"script": "quality/gates/structure/fixture-nonexec.sh", "args": [], "timeout_s": 30, "container": null}
    },
    {
      "id": "structure/fixture-nullscript",
      "dimension": "structure",
      "verifier": {"script": null, "args": [], "timeout_s": 30, "container": null}
    }
  ]
}
EOF
git -C "$WORK_COMBINED" add -A && git -C "$WORK_COMBINED" commit -qm b

set +e; run "$WORK_COMBINED"; rc=$?; set -e
check "exit code" rc "$rc" 1
check_not_contains "no violation line for the good row" "$WORK_COMBINED/.err" "structure/fixture-good::"
check_contains "(b) missing-file row named" "$WORK_COMBINED/.err" "structure/fixture-missing::quality/gates/structure/fixture-missing-DOES-NOT-EXIST.sh::file does not exist"
check_contains "(c) non-executable row named" "$WORK_COMBINED/.err" "structure/fixture-nonexec::quality/gates/structure/fixture-nonexec.sh::not executable::chmod +x"
check_contains "(d) missing/null-field row named" "$WORK_COMBINED/.err" "structure/fixture-nullscript::(no verifier.script field)"
check_contains "violation count header" "$WORK_COMBINED/.err" "FAIL (structure/verifier-script-exists): 3 violation(s)"
echo "  --- observed stderr ---"; sed 's/^/  /' "$WORK_COMBINED/.err"

echo
echo "RESULT: $pass passed, $fail failed"
[[ $fail -eq 0 ]] || exit 1
