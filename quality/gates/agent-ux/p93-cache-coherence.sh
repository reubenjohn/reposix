#!/usr/bin/env bash
# quality/gates/agent-ux/p93-cache-coherence.sh -- agent-ux/p93-cache-coherence-refresh-honest
# verifier (RBF-LR-02, catalog row minted 2026-07-05T10:30:00Z).
#
# Per the row's `expected.asserts`:
#   1. `cargo test -p reposix-cache --test cache_coherence` exits 0 (all
#      cache-coherence tests pass against the ADR-010-chosen architecture)
#   2. a SotPartialFail recovery test exists that simulates SoT-success +
#      mirror-fail and asserts the next push reads new SoT via PRECHECK B
#      and replans correctly -- per ADR-010 "Test co-location" this test
#      lives in reposix-remote (crates/reposix-remote/tests/
#      partial_failure_recovery.rs), run here as a companion command
#   3. `refresh_for_mirror_head` is exercised on the post-write path and its
#      "no-op" behavior is honestly documented (keep+qualify per RBF-LR-04,
#      or the asterisk is removed) -- not silently lying
#
# ONE cargo invocation at a time (crates/CLAUDE.md build-memory budget) --
# the two `cargo test` calls below run SEQUENTIALLY in this single script,
# never concurrently with another cargo user.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

ROW_ID="agent-ux/p93-cache-coherence-refresh-honest"

# --- Assert 1: reposix-cache cache_coherence test binary, exit 0 ----------
if [[ ! -f crates/reposix-cache/tests/cache_coherence.rs ]]; then
  echo "FAIL (${ROW_ID}): crates/reposix-cache/tests/cache_coherence.rs does not exist" >&2
  exit 1
fi
echo "running: cargo test -p reposix-cache --test cache_coherence" >&2
if ! cargo test -p reposix-cache --test cache_coherence; then
  echo "FAIL (${ROW_ID}): cargo test -p reposix-cache --test cache_coherence did not exit 0" >&2
  exit 1
fi

# --- Assert 2: SotPartialFail recovery test exists + passes ---------------
# ADR-010 "Test co-location": lives in reposix-remote, not reposix-cache.
PFR_TEST="crates/reposix-remote/tests/partial_failure_recovery.rs"
if [[ ! -f "${PFR_TEST}" ]]; then
  echo "FAIL (${ROW_ID}): ${PFR_TEST} (SotPartialFail recovery test) does not exist" >&2
  exit 1
fi
if ! grep -qE 'fn +partial_fail.*replan.*converg' "${PFR_TEST}"; then
  echo "FAIL (${ROW_ID}): ${PFR_TEST} does not contain a recognizable partial-fail-then-replan-converges test fn" >&2
  exit 1
fi
echo "running: cargo test -p reposix-remote --test partial_failure_recovery" >&2
if ! cargo test -p reposix-remote --test partial_failure_recovery; then
  echo "FAIL (${ROW_ID}): cargo test -p reposix-remote --test partial_failure_recovery did not exit 0" >&2
  exit 1
fi

# --- Assert 3: refresh_for_mirror_head honesty (keep+qualify per RBF-LR-04) -
# Either (a) the post-write skip is documented as a SEMANTIC no-op (keep +
# qualify, RBF-LR-04's chosen branch) in BOTH root CLAUDE.md and
# docs/concepts/dvcs-topology.md, or (b) the skip was removed entirely
# (unconditional refresh -- no files_touched gate left in write_loop.rs).
GATE_PRESENT=0
if grep -qE 'files_touched *> *0|files_touched *== *0' crates/reposix-remote/src/write_loop.rs; then
  GATE_PRESENT=1
fi
if [[ "${GATE_PRESENT}" -eq 1 ]]; then
  if ! grep -qiE 'RBF-LR-04' crates/reposix-remote/src/write_loop.rs; then
    echo "FAIL (${ROW_ID}): write_loop.rs still gates refresh_for_mirror_head on files_touched but does not cite RBF-LR-04's qualified rationale" >&2
    exit 1
  fi
  if ! grep -qiE 'semantic no-?op' CLAUDE.md; then
    echo "FAIL (${ROW_ID}): root CLAUDE.md does not qualify the mirror-head refresh skip as a semantic (not coherence) no-op" >&2
    exit 1
  fi
  if ! grep -qiE 'semantic no-?op' docs/concepts/dvcs-topology.md; then
    echo "FAIL (${ROW_ID}): docs/concepts/dvcs-topology.md does not qualify the mirror-head refresh skip as a semantic (not coherence) no-op" >&2
    exit 1
  fi
  echo "  refresh_for_mirror_head: files_touched skip RETAINED, honestly qualified as a semantic no-op (RBF-LR-04) in CLAUDE.md + dvcs-topology.md" >&2
else
  echo "  refresh_for_mirror_head: unconditional refresh (files_touched gate removed) -- no asterisk to qualify" >&2
fi

echo "PASS (${ROW_ID}): cache_coherence GREEN, partial_failure_recovery GREEN, refresh_for_mirror_head honesty holds"
exit 0
