#!/usr/bin/env bash
# quality/gates/agent-ux/ql-001-canonical-path.sh — QL-001 canonical record-path
# shape verifier (D91-01/D91-02).
#
# Implements catalog row agent-ux/ql-001-canonical-path-shape.
#
# Substrate dependency: 91-02 (the D91-01 path-fix + cargo regression tests
# that make these asserts real). Until 91-02 lands, this verifier legitimately
# returns NOT-VERIFIED (exit 75) — the code is not yet canonicalized, so
# running the real asserts today would either false-PASS against the old
# broken shape or false-FAIL for a reason unrelated to the actual bug.
#
# The 91-02 executor flips this script by deleting the short-circuit block
# below and un-gating the three real asserts already sketched here.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "$REPO_ROOT"

ARTIFACT="${REPO_ROOT}/quality/reports/verifications/agent-ux/ql-001-canonical-path.json"
mkdir -p "$(dirname "$ARTIFACT")"
TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

# --- 91-02 fills these in for real; short-circuited to exit 75 until then. ---
# shellcheck disable=SC2317  # unreachable until the short-circuit below is removed
run_real_asserts() {
  local fail=0

  # (1) zero zero-padded record-path construction outside reposix-core.
  if grep -rnE 'format!\("\{:04\}\.md"|format!\("\{:011\}\.md"' crates/ \
      | grep -v '^crates/reposix-core/' \
      | grep -v '^crates/[a-z-]*/tests/'; then
    echo "FAIL: zero-padded record-path construction found outside reposix-core" >&2
    fail=1
  fi

  # (2) a single shared path-id helper exists in reposix-core (not counting
  #     comment lines).
  local helper_count
  helper_count="$(grep -rn '^[^#]*pub fn \(issue_id_from_path\|record_path\)' \
    crates/reposix-core/src/ | grep -v '^#' | wc -l)"
  if [ "$helper_count" -lt 1 ]; then
    echo "FAIL: no shared issue_id_from_path/record_path helper found in reposix-core" >&2
    fail=1
  fi

  # (3) the QL-157 duplicate is gone from reposix-remote/src/main.rs.
  if grep -q 'issue_id_from_path' crates/reposix-remote/src/main.rs 2>/dev/null; then
    echo "FAIL: QL-157 duplicate issue_id_from_path still present in reposix-remote/src/main.rs" >&2
    fail=1
  fi

  return "$fail"
}

# --- Short-circuit: 91-02 has not landed yet. Remove this block (and call
# run_real_asserts above) once the D91-01 path fix + cargo regression tests ship.
cat > "$ARTIFACT" <<EOF
{"ts":"${TS}","row_id":"agent-ux/ql-001-canonical-path-shape","exit_code":75,"status":"NOT-VERIFIED","reason":"91-02 not yet landed","blocked_on":["91-02"],"asserts_passed":[],"asserts_failed":["substrate not landed: 91-02 (D91-01 path fix + canonical cargo regression tests) required before this verifier's asserts are meaningful"]}
EOF

echo "NOT-VERIFIED: 91-02 not yet landed (D91-01 path fix + canonical cargo regression tests)" >&2
echo "  artifact: ${ARTIFACT}" >&2
echo "  exit 75 (sysexits.h EX_TEMPFAIL repurposed) -> runner preserves NOT-VERIFIED" >&2
exit 75
