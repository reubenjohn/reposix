#!/usr/bin/env bash
# quality/gates/agent-ux/p93-delta-sync-coherence.sh -- agent-ux/p93-delta-sync-coherence-invariant
# verifier (D-P92-03 acceptance gate, catalog row minted 2026-07-05T10:30:00Z).
#
# Per the row's `expected.asserts`:
#   1. the `#[ignore]` attribute is REMOVED from
#      delta_sync_tree_references_only_resolvable_oids in
#      crates/reposix-cache/tests/delta_sync.rs (grep confirms no #[ignore]
#      on that fn)
#   2. `cargo test -p reposix-cache --test delta_sync` runs
#      delta_sync_tree_references_only_resolvable_oids (NOT skipped) and it
#      PASSES
#   3. the other 3 delta_sync tests stay green (regression: 4 passed, 0
#      ignored)
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

ROW_ID="agent-ux/p93-delta-sync-coherence-invariant"
TEST_FILE="crates/reposix-cache/tests/delta_sync.rs"
TARGET_FN="delta_sync_tree_references_only_resolvable_oids"

if [[ ! -f "${TEST_FILE}" ]]; then
  echo "FAIL (${ROW_ID}): ${TEST_FILE} does not exist" >&2
  exit 1
fi

# --- Assert 1: no #[ignore] attribute directly above the target fn -------
# Find the fn's line, then scan the immediately-preceding contiguous
# attribute/comment lines for a bare #[ignore] or #[ignore = "..."].
FN_LINE="$(grep -nE "^async fn ${TARGET_FN}\(\)" "${TEST_FILE}" | head -1 | cut -d: -f1)"
if [[ -z "${FN_LINE}" ]]; then
  echo "FAIL (${ROW_ID}): fn ${TARGET_FN} not found in ${TEST_FILE}" >&2
  exit 1
fi
# Walk upward from FN_LINE-1 while lines are attributes (#[...]) or blank;
# stop at the first non-attribute line (e.g. a doc comment or code).
i=$((FN_LINE - 1))
IGNORE_FOUND=0
while [[ "${i}" -ge 1 ]]; do
  line="$(sed -n "${i}p" "${TEST_FILE}")"
  if [[ "${line}" =~ ^[[:space:]]*#\[ignore ]]; then
    IGNORE_FOUND=1
    break
  fi
  if [[ "${line}" =~ ^[[:space:]]*#\[ ]]; then
    i=$((i - 1))
    continue
  fi
  break
done
if [[ "${IGNORE_FOUND}" -eq 1 ]]; then
  echo "FAIL (${ROW_ID}): #[ignore] is still attached to ${TARGET_FN} -- RED regression not flipped GREEN" >&2
  exit 1
fi
echo "  confirmed: no #[ignore] attribute on ${TARGET_FN}" >&2

# --- Assert 2 + 3: cargo test runs it (not skipped), all 4 pass, 0 ignored -
echo "running: cargo test -p reposix-cache --test delta_sync" >&2
OUT_LOG="$(mktemp -t p93-delta-sync-coherence.XXXXXX)"
trap 'rm -f "${OUT_LOG}"' EXIT
set +e
cargo test -p reposix-cache --test delta_sync 2>&1 | tee "${OUT_LOG}"
CARGO_EXIT=${PIPESTATUS[0]}
set -e
if [[ "${CARGO_EXIT}" -ne 0 ]]; then
  echo "FAIL (${ROW_ID}): cargo test -p reposix-cache --test delta_sync did not exit 0" >&2
  exit 1
fi

if ! grep -qE "test ${TARGET_FN} \.\.\. ok" "${OUT_LOG}"; then
  echo "FAIL (${ROW_ID}): ${TARGET_FN} was not observed running + passing (test filtered out / skipped?)" >&2
  exit 1
fi

# "test result: ok. 4 passed; 0 failed; 0 ignored; ..."
RESULT_LINE="$(grep -E '^test result: ' "${OUT_LOG}" | tail -1)"
if [[ -z "${RESULT_LINE}" ]]; then
  echo "FAIL (${ROW_ID}): no 'test result:' summary line found in cargo output" >&2
  exit 1
fi
PASSED="$(echo "${RESULT_LINE}" | grep -oE '[0-9]+ passed' | grep -oE '^[0-9]+')"
IGNORED="$(echo "${RESULT_LINE}" | grep -oE '[0-9]+ ignored' | grep -oE '^[0-9]+')"
if [[ "${PASSED:-0}" -lt 4 ]]; then
  echo "FAIL (${ROW_ID}): expected 4 passed, got '${RESULT_LINE}'" >&2
  exit 1
fi
if [[ "${IGNORED:-1}" -ne 0 ]]; then
  echo "FAIL (${ROW_ID}): expected 0 ignored, got '${RESULT_LINE}'" >&2
  exit 1
fi

echo "PASS (${ROW_ID}): ${RESULT_LINE} -- ${TARGET_FN} un-ignored + GREEN, other 3 delta_sync tests stay green"
exit 0
