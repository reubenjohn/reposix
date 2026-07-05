#!/usr/bin/env bash
# quality/gates/agent-ux/p93-l1-promise-reconciled.sh -- agent-ux/p93-l1-promise-reconciled
# BACKING verifier (RBF-LR-04, catalog row minted 2026-07-05T10:30:00Z,
# kind: manual, freshness_ttl: 90d).
#
# Per the row's owner_hint: "a backing grep verifier can assert which branch
# the docs took, but the [phase-close] subagent decides consistency with the
# shipped behavior." This script is that backing check -- it does NOT itself
# adjudicate "no lying doc"; it asserts the MECHANICAL facts the subagent
# needs: which branch (keep+qualify vs remove-asterisk) the shipped code
# took, and whether the docs carry the matching qualification. A human/
# subagent still grades final consistency at phase close.
#
# Per the row's `expected.asserts`:
#   1. the L1 promise text in CLAUDE.md + docs/concepts/dvcs-topology.md is
#      internally consistent with the shipped refresh_for_mirror_head
#      behavior (RBF-LR-02)
#   2. if RBF-LR-02 landed the honest post-write refresh: the unqualified
#      asterisk/caveat on the L1 promise is REMOVED
#   3. if the skip is architecturally retained: the asterisk is KEPT AND
#      qualified in dvcs-topology.md with the bounded-exposure explanation +
#      a `reposix sync --reconcile` pointer
#   4. the same PR that lands RBF-LR-02 updates this promise (process claim
#      -- not mechanically checkable by grep; left to the subagent)
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

ROW_ID="agent-ux/p93-l1-promise-reconciled"
CLAUDE_MD="CLAUDE.md"
TOPOLOGY_MD="docs/concepts/dvcs-topology.md"
RECONCILE_MD="docs/guides/troubleshooting.md"

for f in "${CLAUDE_MD}" "${TOPOLOGY_MD}" "${RECONCILE_MD}"; do
  if [[ ! -f "${f}" ]]; then
    echo "FAIL (${ROW_ID}): ${f} not found" >&2
    exit 1
  fi
done

# --- Determine which branch the shipped code took --------------------------
GATE_PRESENT=0
if grep -qE 'files_touched *> *0|files_touched *== *0' crates/reposix-remote/src/write_loop.rs 2>/dev/null; then
  GATE_PRESENT=1
fi

if [[ "${GATE_PRESENT}" -eq 1 ]]; then
  echo "  branch: files_touched skip RETAINED -- expecting keep+qualify (RBF-LR-04)" >&2

  # Asterisk must be KEPT and qualified in dvcs-topology.md: bounded-exposure
  # explanation + a reconcile pointer.
  if ! grep -qiE 'semantic no-?op' "${TOPOLOGY_MD}"; then
    echo "FAIL (${ROW_ID}): ${TOPOLOGY_MD} does not qualify the mirror-head refresh skip as a semantic (bounded) no-op" >&2
    exit 1
  fi
  if ! grep -qF 'reposix sync --reconcile' "${TOPOLOGY_MD}"; then
    echo "FAIL (${ROW_ID}): ${TOPOLOGY_MD} does not point at 'reposix sync --reconcile' as the manual catch-up" >&2
    exit 1
  fi
  if ! grep -qiE 'RBF-LR-04' "${TOPOLOGY_MD}"; then
    echo "FAIL (${ROW_ID}): ${TOPOLOGY_MD} does not cite RBF-LR-04 for the qualified skip" >&2
    exit 1
  fi

  # CLAUDE.md's L1 promise line must ALSO carry the qualifier (not an
  # unqualified "refreshes on every push, no exceptions" claim).
  if ! grep -qiE 'RBF-LR-04' "${CLAUDE_MD}"; then
    echo "FAIL (${ROW_ID}): ${CLAUDE_MD} L1 promise does not cite RBF-LR-04's qualified skip" >&2
    exit 1
  fi
  if ! grep -qiE 'semantic no-?op|not a coherence' "${CLAUDE_MD}"; then
    echo "FAIL (${ROW_ID}): ${CLAUDE_MD} does not qualify the mirror-head skip as semantic (not a coherence shortcut)" >&2
    exit 1
  fi
  echo "  keep+qualify branch: dvcs-topology.md + CLAUDE.md both carry the RBF-LR-04-qualified caveat + reconcile pointer" >&2
else
  echo "  branch: files_touched skip REMOVED -- expecting the unqualified asterisk to be gone" >&2
  if grep -qiE 'no-op.*skip.*refresh|skip.*refresh_for_mirror_head' "${CLAUDE_MD}"; then
    echo "FAIL (${ROW_ID}): ${CLAUDE_MD} still advertises a no-op-skips-refresh caveat, but write_loop.rs no longer gates on files_touched (lying doc)" >&2
    exit 1
  fi
  echo "  unconditional-refresh branch: no lingering no-op-skip caveat found in ${CLAUDE_MD}" >&2
fi

# --- troubleshooting.md must not still say L3 defers to v0.14.0 (that was
# the exact stale cross-reference ADR-010's consequences section calls out
# as needing a refresh in the SAME fix wave). L2 deferral is fine and
# expected; L3 must read as SHIPPED. -----------------------------------------
if grep -qiE 'L3[^.]*defers? to v0\.14\.0' "${RECONCILE_MD}"; then
  echo "FAIL (${ROW_ID}): ${RECONCILE_MD} still claims L3 defers to v0.14.0 -- L3 (transactional cache writes) is shipped per ADR-010; this is a lying doc" >&2
  exit 1
fi
if ! grep -qiE 'ADR-010' "${RECONCILE_MD}"; then
  echo "FAIL (${ROW_ID}): ${RECONCILE_MD} does not cross-reference ADR-010 for the cache-coherence recovery mechanism" >&2
  exit 1
fi

echo "PASS (${ROW_ID}): L1 promise branch (files_touched-gate present=${GATE_PRESENT}) is mechanically consistent across CLAUDE.md + dvcs-topology.md + troubleshooting.md; no lying-doc token found. (Process claim #4 -- same-PR landing -- graded by the phase-close verifier subagent, not this mechanical backing check.)"
exit 0
