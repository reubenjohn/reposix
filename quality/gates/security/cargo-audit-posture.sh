#!/usr/bin/env bash
# quality/gates/security/cargo-audit-posture.sh -- security/cargo-audit-rustsec-posture verifier.
#
# Local/catalogued counterpart to the CI cargo-audit job (.github/workflows/audit.yml).
# Encodes reposix's RUSTSEC advisory posture as a GREEN contract (P107, 2026-07-12):
#
#   1. `cargo audit` exits 0 with ZERO live vulnerabilities against the committed
#      Cargo.lock. A NEW live advisory flips this RED.
#   2. RUSTSEC-2026-0186 (memmap2) is CLEARED by version floor: installed memmap2
#      >= 0.9.11 (the patched floor). memmap2 is transitive via gix 0.83.0
#      (gix-commitgraph/index/odb/pack/ref) -> reposix-cache; the advisory is
#      informational="unsound" (only gated by `-D unsound`), so it never blocks a
#      default `cargo audit` -- the floor assert is the real defense.
#   3. RUSTSEC-2026-0185 (quinn-proto) is CLEARED: either absent from the resolved
#      tree (orphan Cargo.lock entry, never built) OR pinned >= 0.11.15. Both states
#      are non-actionable; we assert "absent OR floor met".
#   4. A committed posture doc (SECURITY.md § Advisory posture) names BOTH advisory
#      ids -- so the honest reachability verdict cannot silently rot away.
#
# Version floors are parsed from Cargo.lock (offline, deterministic) rather than
# `cargo tree` (whose output for an absent orphan is empty and brittle to parse).
# A network-flaky advisory-db fetch is DISTINGUISHED from a real advisory: the
# artifact carries "network_fetch_failure": true and the script exits 2 (PARTIAL),
# never conflating an infra hiccup with a live CVE.
#
# Ground-truth evidence artifact:
#   .planning/milestones/v0.14.0-phases/evidence/p107-cargo-audit-2026-07-12.txt
#
# Honors --row-id <id> (defaults to security/cargo-audit-rustsec-posture).
# Implements catalog row security/cargo-audit-rustsec-posture.
set -euo pipefail

readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
readonly ARTIFACT_DIR="${REPO_ROOT}/quality/reports/verifications/security"
readonly ARTIFACT="${ARTIFACT_DIR}/cargo-audit-posture.json"
readonly LOCKFILE="${REPO_ROOT}/Cargo.lock"
readonly SECURITY_DOC="${REPO_ROOT}/SECURITY.md"
readonly MEMMAP2_FLOOR="0.9.11"
readonly QUINN_PROTO_FLOOR="0.11.15"
readonly RUSTSEC_MEMMAP2="RUSTSEC-2026-0186"
readonly RUSTSEC_QUINN="RUSTSEC-2026-0185"

row_id="security/cargo-audit-rustsec-posture"
if [[ "${1:-}" == "--row-id" && -n "${2:-}" ]]; then
  row_id="$2"
fi

cd "$REPO_ROOT"
mkdir -p "$ARTIFACT_DIR"

# --- helpers ---------------------------------------------------------------

# lockfile_version <crate> -> prints installed version from Cargo.lock, or empty
# if the crate is absent (an absent orphan is a valid "cleared" state).
lockfile_version() {
  local crate="$1"
  awk -v c="$crate" '
    $1=="name" && $3=="\""c"\"" { found=1; next }
    found && $1=="version" { gsub(/"/,"",$3); print $3; exit }
  ' "$LOCKFILE"
}

# version_ge <a> <b> -> exit 0 if a >= b (semver-ish via sort -V)
version_ge() {
  [[ "$1" == "$2" ]] && return 0
  local lowest
  lowest=$(printf '%s\n%s\n' "$1" "$2" | sort -V | head -1)
  [[ "$lowest" == "$2" ]]
}

# --- run cargo audit -------------------------------------------------------

audit_stdout=$(cargo audit --color never 2>&1) && audit_exit=0 || audit_exit=$?
ts=$(date -u +%Y-%m-%dT%H:%M:%SZ)

# Distinguish a network/advisory-db fetch failure from a real advisory hit.
network_fetch_failure=false
if [[ "$audit_exit" -ne 0 ]] && \
   printf '%s\n' "$audit_stdout" | grep -qiE 'couldn.t fetch|failed to fetch|error fetching|network|could not resolve host|unable to (update|fetch)'; then
  network_fetch_failure=true
fi

# --- version floor checks (offline, from Cargo.lock) -----------------------

memmap2_ver=$(lockfile_version memmap2)
quinn_proto_ver=$(lockfile_version quinn-proto)

memmap2_ok=false
if [[ -n "$memmap2_ver" ]] && version_ge "$memmap2_ver" "$MEMMAP2_FLOOR"; then
  memmap2_ok=true
fi

# quinn-proto: absent OR floor-met both count as CLEARED.
quinn_ok=false
if [[ -z "$quinn_proto_ver" ]]; then
  quinn_ok=true
  quinn_detail="absent from Cargo.lock (transitive-and-absent, never built)"
elif version_ge "$quinn_proto_ver" "$QUINN_PROTO_FLOOR"; then
  quinn_ok=true
  quinn_detail="pinned ${quinn_proto_ver} >= ${QUINN_PROTO_FLOOR}"
else
  quinn_detail="pinned ${quinn_proto_ver} < ${QUINN_PROTO_FLOOR} (BELOW patched floor)"
fi

# --- posture doc names both advisory ids -----------------------------------

posture_doc_ok=false
if [[ -f "$SECURITY_DOC" ]] \
   && grep -q "$RUSTSEC_MEMMAP2" "$SECURITY_DOC" \
   && grep -q "$RUSTSEC_QUINN" "$SECURITY_DOC"; then
  posture_doc_ok=true
fi

# --- compose verdict -------------------------------------------------------

exit_code=0
asserts_passed=()
asserts_failed=()

if [[ "$network_fetch_failure" == true ]]; then
  # Infra hiccup, NOT a live CVE. PARTIAL (exit 2) -- do not green, do not
  # falsely claim a live advisory.
  exit_code=2
  asserts_failed+=("cargo audit could not fetch the advisory database (network failure) -- advisory status INDETERMINATE, not a live-advisory verdict")
elif [[ "$audit_exit" -ne 0 ]]; then
  exit_code=1
  asserts_failed+=("cargo audit exited ${audit_exit}: a live RUSTSEC advisory is present -- read the named RUSTSEC id in stderr and bump the affected crate")
else
  asserts_passed+=("cargo audit exits 0 with 0 live vulnerabilities across the workspace Cargo.lock")
fi

if [[ "$memmap2_ok" == true ]]; then
  asserts_passed+=("${RUSTSEC_MEMMAP2} (memmap2) CLEARED: installed memmap2 ${memmap2_ver} >= ${MEMMAP2_FLOOR} patched floor (transitive via gix; informational=unsound)")
else
  exit_code=1
  asserts_failed+=("${RUSTSEC_MEMMAP2} (memmap2): installed version '${memmap2_ver:-<absent>}' does not meet patched floor ${MEMMAP2_FLOOR}")
fi

if [[ "$quinn_ok" == true ]]; then
  asserts_passed+=("${RUSTSEC_QUINN} (quinn-proto) CLEARED: ${quinn_detail}")
else
  exit_code=1
  asserts_failed+=("${RUSTSEC_QUINN} (quinn-proto): ${quinn_detail}")
fi

if [[ "$posture_doc_ok" == true ]]; then
  asserts_passed+=("SECURITY.md posture doc names both ${RUSTSEC_MEMMAP2} and ${RUSTSEC_QUINN}")
else
  exit_code=1
  asserts_failed+=("SECURITY.md missing an Advisory posture section naming both ${RUSTSEC_MEMMAP2} and ${RUSTSEC_QUINN}")
fi

# --- write artifact --------------------------------------------------------

py_json_array() {
  if [[ "$#" -eq 0 ]]; then
    echo "[]"
  else
    python3 -c "import json,sys; print(json.dumps(sys.argv[1:]))" "$@"
  fi
}
asserts_passed_json=$(py_json_array "${asserts_passed[@]}")
asserts_failed_json=$(py_json_array "${asserts_failed[@]}")
stdout_json=$(printf '%s' "$audit_stdout" | python3 -c "import sys,json; print(json.dumps(sys.stdin.read()))")

cat > "$ARTIFACT" <<EOF
{
  "ts": "$ts",
  "row_id": "$row_id",
  "exit_code": $exit_code,
  "cargo_audit_exit_code": $audit_exit,
  "network_fetch_failure": $network_fetch_failure,
  "memmap2_version": "${memmap2_ver:-}",
  "memmap2_floor": "$MEMMAP2_FLOOR",
  "memmap2_ok": $memmap2_ok,
  "quinn_proto_version": "${quinn_proto_ver:-}",
  "quinn_proto_floor": "$QUINN_PROTO_FLOOR",
  "quinn_proto_ok": $quinn_ok,
  "posture_doc_ok": $posture_doc_ok,
  "stdout": $stdout_json,
  "asserts_passed": $asserts_passed_json,
  "asserts_failed": $asserts_failed_json
}
EOF

if [[ "$exit_code" -eq 0 ]]; then
  echo "PASS security/cargo-audit-rustsec-posture: 0 live advisories; ${RUSTSEC_MEMMAP2} + ${RUSTSEC_QUINN} cleared by version floor; posture doc present"
elif [[ "$exit_code" -eq 2 ]]; then
  echo "PARTIAL security/cargo-audit-rustsec-posture: advisory-db fetch failed (network) -- status indeterminate, not a live-advisory verdict" >&2
  printf '%s\n' "$audit_stdout" >&2
else
  echo "FAIL security/cargo-audit-rustsec-posture:" >&2
  printf '  - %s\n' "${asserts_failed[@]}" >&2
  printf '%s\n' "$audit_stdout" >&2
fi
exit "$exit_code"
