#!/usr/bin/env bash
# quality/gates/security/audit-immutability.sh -- security/audit-immutability verifier.
#
# CLAUDE.md OP-3 ("Audit log is non-optional... TWO append-only tables") +
# threat-model P0 cut: neither audit_events (reposix-core) nor
# audit_events_cache (reposix-cache) may be UPDATEd or DELETEd once written.
# This row's verifier script was dangling (P90 90-05 R2 § D/H finding #3) --
# the path named in the catalog row did not exist, even though the schema
# invariant it claims to cover was already real and tested in BOTH crates.
# Landed here as a thin wrapper, mirroring connector-audit-wired.sh's shape:
#
#   1. cargo test -p reposix-core --test audit_schema -- 8 #[test] fns
#      against `audit_events`: column shape, both triggers registered,
#      UPDATE/DELETE rejected with an "append-only" message + row survives,
#      plus the H-02 schema-attack hardening trio (writable_schema bypass,
#      DROP TRIGGER documented limit, rollback-does-not-break-invariant).
#   2. cargo test -p reposix-cache --test audit_is_append_only -- the
#      `audit_events_cache` counterpart (1 #[test] fn covering the same
#      UPDATE/DELETE-both-fail + row-survives shape via the real
#      `open_cache_db` connection).
#   3. A static grep confirming WAL mode ("PRAGMA journal_mode" -> WAL) is
#      actually set in reposix-cache::db::open_cache_db -- the row's
#      "WAL mode confirmed" assert. NOTE (asymmetry, flagged not fixed):
#      reposix-core::audit::open_audit_db does NOT set WAL explicitly (only
#      DEFENSIVE + schema load); the cache-side connection is the one that
#      is WAL-tuned for concurrent-reader friendliness per db.rs's own
#      module doc. This script only claims WAL on the cache side, matching
#      reality instead of assuming symmetry that doesn't exist.
#
# NOTE (honesty caveat, 90-05): this script has NOT been executed by the
# agent that wrote it -- CLAUDE.md's "Build memory budget" section forbids
# ad-hoc cargo invocations during framework-dimension (F-class) dispatches
# on this VM. Correctness was established by manual line-by-line reading of
# crates/reposix-core/tests/audit_schema.rs,
# crates/reposix-cache/tests/audit_is_append_only.rs, and
# crates/reposix-cache/src/db.rs:54-55 (WAL pragma). The catalog row's
# waiver stays renewed (not cleared) until a real `cargo test` run confirms
# this script's exit code -- tracked as an explicit P92 line item
# (RAISE LIST § 5) rather than assumed green.
#
# Honors --row-id <id> (defaults to security/audit-immutability).
# Implements catalog row security/audit-immutability.
set -euo pipefail

readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
readonly ARTIFACT_DIR="${REPO_ROOT}/quality/reports/verifications/security"
readonly ARTIFACT="${ARTIFACT_DIR}/audit-immutability.json"
readonly CACHE_DB_SRC="${REPO_ROOT}/crates/reposix-cache/src/db.rs"
readonly MIN_CORE_TESTS=8
readonly MIN_CACHE_TESTS=1

row_id="security/audit-immutability"
if [[ "${1:-}" == "--row-id" && -n "${2:-}" ]]; then
  row_id="$2"
fi

cd "$REPO_ROOT"
mkdir -p "$ARTIFACT_DIR"

core_stdout=$(cargo test -p reposix-core --test audit_schema 2>&1) && core_exit=0 || core_exit=$?
cache_stdout=$(cargo test -p reposix-cache --test audit_is_append_only 2>&1) && cache_exit=0 || cache_exit=$?
ts=$(date -u +%Y-%m-%dT%H:%M:%SZ)

core_ran=$(printf '%s\n' "$core_stdout" | grep -oE '[0-9]+ passed' | grep -oE '[0-9]+' | awk '{s+=$1} END {print s+0}')
core_failed=$(printf '%s\n' "$core_stdout" | grep -oE '[0-9]+ failed' | grep -oE '[0-9]+' | awk '{s+=$1} END {print s+0}')
cache_ran=$(printf '%s\n' "$cache_stdout" | grep -oE '[0-9]+ passed' | grep -oE '[0-9]+' | awk '{s+=$1} END {print s+0}')
cache_failed=$(printf '%s\n' "$cache_stdout" | grep -oE '[0-9]+ failed' | grep -oE '[0-9]+' | awk '{s+=$1} END {print s+0}')

# Static grep: WAL pragma set in the cache-side open path.
wal_ok=0
if grep -q 'journal_mode' "$CACHE_DB_SRC" && grep -q '"WAL"' "$CACHE_DB_SRC"; then
  wal_ok=1
fi

exit_code=0
asserts_passed=()
asserts_failed=()

if [[ "$core_exit" -ne 0 ]]; then
  exit_code=1
  asserts_failed+=("cargo test -p reposix-core --test audit_schema exited ${core_exit} (nonzero)")
elif [[ "$core_failed" -gt 0 ]]; then
  exit_code=1
  asserts_failed+=("${core_failed} audit_schema test(s) FAILED")
else
  asserts_passed+=("cargo test -p reposix-core --test audit_schema exits 0")
fi

if [[ "$core_ran" -lt "$MIN_CORE_TESTS" ]]; then
  exit_code=1
  asserts_failed+=("only ${core_ran} audit_schema test(s) ran; expected >= ${MIN_CORE_TESTS}")
else
  asserts_passed+=("${core_ran} audit_schema test(s) ran (>= ${MIN_CORE_TESTS} required) -- audit_events UPDATE/DELETE rejected, both triggers present, H-02 hardening")
fi

if [[ "$cache_exit" -ne 0 ]]; then
  exit_code=1
  asserts_failed+=("cargo test -p reposix-cache --test audit_is_append_only exited ${cache_exit} (nonzero)")
elif [[ "$cache_failed" -gt 0 ]]; then
  exit_code=1
  asserts_failed+=("${cache_failed} audit_is_append_only test(s) FAILED")
else
  asserts_passed+=("cargo test -p reposix-cache --test audit_is_append_only exits 0")
fi

if [[ "$cache_ran" -lt "$MIN_CACHE_TESTS" ]]; then
  exit_code=1
  asserts_failed+=("only ${cache_ran} audit_is_append_only test(s) ran; expected >= ${MIN_CACHE_TESTS}")
else
  asserts_passed+=("${cache_ran} audit_is_append_only test(s) ran (>= ${MIN_CACHE_TESTS} required) -- audit_events_cache UPDATE/DELETE rejected, row survives")
fi

if [[ "$wal_ok" -eq 1 ]]; then
  asserts_passed+=("reposix-cache::db::open_cache_db sets PRAGMA journal_mode=WAL (${CACHE_DB_SRC})")
else
  exit_code=1
  asserts_failed+=("could not confirm WAL pragma in ${CACHE_DB_SRC}")
fi

py_json_array() {
  if [[ "$#" -eq 0 ]]; then
    echo "[]"
  else
    python3 -c "import json,sys; print(json.dumps(sys.argv[1:]))" "$@"
  fi
}
asserts_passed_json=$(py_json_array "${asserts_passed[@]}")
asserts_failed_json=$(py_json_array "${asserts_failed[@]}")
core_stdout_json=$(printf '%s' "$core_stdout" | python3 -c "import sys,json; print(json.dumps(sys.stdin.read()))")
cache_stdout_json=$(printf '%s' "$cache_stdout" | python3 -c "import sys,json; print(json.dumps(sys.stdin.read()))")

cat > "$ARTIFACT" <<EOF
{
  "ts": "$ts",
  "row_id": "$row_id",
  "exit_code": $exit_code,
  "core_cargo_exit_code": $core_exit,
  "cache_cargo_exit_code": $cache_exit,
  "core_tests_ran": $core_ran,
  "core_tests_failed": $core_failed,
  "cache_tests_ran": $cache_ran,
  "cache_tests_failed": $cache_failed,
  "wal_mode_confirmed_cache_side": $([ "$wal_ok" -eq 1 ] && echo true || echo false),
  "core_stdout": $core_stdout_json,
  "cache_stdout": $cache_stdout_json,
  "asserts_passed": $asserts_passed_json,
  "asserts_failed": $asserts_failed_json
}
EOF

if [[ "$exit_code" -ne 0 ]]; then
  echo "$core_stdout" >&2
  echo "$cache_stdout" >&2
fi
exit "$exit_code"
