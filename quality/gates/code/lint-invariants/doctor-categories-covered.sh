#!/usr/bin/env bash
# quality/gates/code/lint-invariants/doctor-categories-covered.sh
#
# Asserts the troubleshooting.md claim: "reposix doctor diagnoses git
# config, cache DB, env vars, auth, sparse-checkout patterns, and
# cache freshness."
#
# Verifies each of the 6 categories has at least one corresponding
# `check_*` function in crates/reposix-cli/src/doctor.rs. Source-level
# grep is the right granularity — the integration test
# `doctor_clean_repo_reports_findings` exercises the dispatch but only
# asserts a subset; the claim is about category COVERAGE which is
# determined by which check_* fns the module compiles in.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

readonly DOCTOR=crates/reposix-cli/src/doctor.rs
[[ -f "$DOCTOR" ]] || { echo "FAIL: $DOCTOR not found" >&2; exit 1; }

failed=0

# Each entry: <category-label>|<grep-pattern>
declare -a CATEGORIES=(
  'git config|fn check_git_repo|fn check_partial_clone|fn check_remote_url'
  'cache DB|fn check_cache_db_exists|fn check_cache_db_readable|fn check_audit_table'
  'env vars|fn check_allowed_origins'
  'auth|fn check_remote_url|fn check_backend_registered'
  'sparse-checkout|fn check_sparse_checkout'
  'cache freshness|fn check_outdated_cache|fn check_cache_has_main_commit|fn check_worktree_head_drift'
)

for entry in "${CATEGORIES[@]}"; do
  label="${entry%%|*}"
  patterns="${entry#*|}"
  found=0
  IFS='|' read -ra pats <<< "$patterns"
  for p in "${pats[@]}"; do
    if grep -q -F "$p" "$DOCTOR"; then
      found=1
      break
    fi
  done
  if [[ "$found" -eq 0 ]]; then
    echo "FAIL: doctor.rs missing any check fn for category '$label' (looked for: $patterns)" >&2
    failed=1
  fi
done

if [[ "$failed" -ne 0 ]]; then
  exit 1
fi

echo "PASS: doctor.rs covers all 6 claimed diagnostic categories (git config, cache DB, env vars, auth, sparse-checkout, cache freshness)"
