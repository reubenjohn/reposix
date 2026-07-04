#!/usr/bin/env bash
# quality/gates/agent-ux/ql-001-canonical-path.sh — QL-001 canonical record-path
# shape verifier (D91-01/D91-02).
#
# Implements catalog row agent-ux/ql-001-canonical-path-shape.
#
# Box-independent (D91-02): grep-only asserts + fn-name existence checks. No
# git ≥2.34 required — the full-stack e2e proof lives in the sibling
# agent-ux/real-git-push-e2e row (CI, pre-pr). 91-02 landed the D91-01 path
# fix + canonical cargo regression tests, so these asserts are now REAL.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "$REPO_ROOT"

ARTIFACT="${REPO_ROOT}/quality/reports/verifications/agent-ux/ql-001-canonical-path.json"
mkdir -p "$(dirname "$ARTIFACT")"
TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

PASSED=()
FAILED=()

# (1) QL-001 criterion 6: zero zero-padded record-path construction outside
#     reposix-core (fixtures in tests/ legitimately name issues/N.md literals,
#     never format!{:04}/{:011}, so tests/ need not be excluded — but we keep
#     the exclusion for defense-in-depth against a helper-derived fixture).
if grep -rnE 'format!\("\{:04\}\.md"|format!\("\{:011\}\.md"' crates/ \
    | grep -v '^crates/reposix-core/' \
    | grep -v '^crates/[a-z-]*/tests/' >&2; then
  echo "FAIL: zero-padded record-path construction found outside reposix-core" >&2
  FAILED+=("criterion-6: zero-padded record-path construction outside reposix-core")
else
  PASSED+=("criterion-6: no {:04}/{:011} record-path construction outside reposix-core")
fi

# (2) a single shared path-id helper exists in reposix-core.
helper_count="$(grep -rnE '^[[:space:]]*pub fn (issue_id_from_path|record_path)' \
  crates/reposix-core/src/ | wc -l | tr -d ' ')"
if [ "$helper_count" -lt 1 ]; then
  echo "FAIL: no shared issue_id_from_path/record_path helper in reposix-core" >&2
  FAILED+=("no shared helper in reposix-core")
else
  PASSED+=("shared helper present in reposix-core (record_path + issue_id_from_path)")
fi

# (3) the QL-157 duplicate is gone from reposix-remote/src/main.rs.
if grep -q 'issue_id_from_path' crates/reposix-remote/src/main.rs 2>/dev/null; then
  echo "FAIL: QL-157 duplicate issue_id_from_path still in reposix-remote/src/main.rs" >&2
  FAILED+=("QL-157 duplicate still present in main.rs")
else
  PASSED+=("QL-157 duplicate deleted from main.rs")
fi

# (4) the box-independent QL-001 regression tests exist (RED-if-bug-returns).
#     Wave-5.5 added the pages/-bucket family (confluence mass-delete BLOCKER)
#     + the bucket_for_backend helper assert below.
declare -A REGRESSIONS=(
  ["crates/reposix-remote/src/diff.rs"]="full_seeded_tree_push_emits_zero_deletes canonical_single_edit_is_one_update reposix_metadata_paths_are_ignored_not_rejected delete_wins_over_add_for_same_path pages_full_tree_push_emits_zero_deletes pages_single_edit_is_one_update cross_bucket_tree_still_matches_by_id duplicate_record_id_across_buckets_is_refused pages_bulk_delete_still_capped"
  ["crates/reposix-remote/src/fast_import.rs"]="commit_message_without_trailing_lf_does_not_swallow_first_m_line"
  ["crates/reposix-cache/tests/bucket_tree.rs"]="confluence_cache_tree_uses_pages_bucket sim_cache_tree_uses_issues_bucket"
)

# (5) Wave-5.5: the bucket-aware helper exists in reposix-core.
if grep -qE '^[[:space:]]*pub fn bucket_for_backend' crates/reposix-core/src/path.rs; then
  PASSED+=("bucket_for_backend helper present in reposix-core (Wave-5.5)")
else
  echo "FAIL: bucket_for_backend missing from reposix-core/src/path.rs" >&2
  FAILED+=("bucket_for_backend missing from reposix-core")
fi
for file in "${!REGRESSIONS[@]}"; do
  for fn in ${REGRESSIONS[$file]}; do
    if grep -q "fn ${fn}(" "$file" 2>/dev/null; then
      PASSED+=("regression fn present: ${fn}")
    else
      echo "FAIL: missing QL-001 regression fn ${fn} in ${file}" >&2
      FAILED+=("missing regression fn: ${fn}")
    fi
  done
done

# --- Emit artifact + exit ----------------------------------------------------
pass_json="$(printf '%s\n' "${PASSED[@]}" | sed 's/"/\\"/g' | awk 'NR>1{printf ","}{printf "\"%s\"",$0}')"
fail_json="$(printf '%s\n' "${FAILED[@]:-}" | sed 's/"/\\"/g' | awk 'NF{if(n++)printf ",";printf "\"%s\"",$0}')"

if [ "${#FAILED[@]}" -eq 0 ]; then
  cat > "$ARTIFACT" <<EOF
{"ts":"${TS}","row_id":"agent-ux/ql-001-canonical-path-shape","exit_code":0,"status":"PASS","asserts_passed":[${pass_json}],"asserts_failed":[]}
EOF
  echo "PASS: QL-001 canonical path shape (${#PASSED[@]} asserts)" >&2
  echo "  artifact: ${ARTIFACT}" >&2
  exit 0
else
  cat > "$ARTIFACT" <<EOF
{"ts":"${TS}","row_id":"agent-ux/ql-001-canonical-path-shape","exit_code":1,"status":"FAIL","asserts_passed":[${pass_json}],"asserts_failed":[${fail_json}]}
EOF
  echo "FAIL: QL-001 canonical path shape (${#FAILED[@]} failed)" >&2
  echo "  artifact: ${ARTIFACT}" >&2
  exit 1
fi
