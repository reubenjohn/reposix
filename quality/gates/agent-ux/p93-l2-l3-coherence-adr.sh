#!/usr/bin/env bash
# quality/gates/agent-ux/p93-l2-l3-coherence-adr.sh -- agent-ux/p93-l2-l3-coherence-adr
# verifier (RBF-LR-01, catalog row minted 2026-07-05T10:30:00Z).
#
# Grades the ADR's EXISTENCE + CONTENT (not the code fix -- that is
# RBF-LR-02 / agent-ux/p93-cache-coherence-refresh-honest). Per the row's
# `expected.asserts`:
#   1. an ADR file matching docs/decisions/*cache-coherence*.md (or
#      *l2-l3*.md) exists
#   2. the ADR names BOTH options: L2 = re-fetch-on-cache-miss AND
#      L3 = transactional-cache-writes
#   3. the ADR states a CHOSEN path plus its trade-off (a decision, not
#      just an enumeration)
#   4. the ADR records the v0.14.0-deferral list explicitly (what is NOT
#      fixed in v0.13.x) with the no-re-deferral-without-owner-signoff rule
#   5. the ADR cross-references the honest-L1 outcome (RBF-LR-02) --
#      asterisk removed or kept+qualified
#
# kind: mechanical (grep-only; no cargo, no cache, no network).
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

ROW_ID="agent-ux/p93-l2-l3-coherence-adr"

# --- Assert 1: ADR file exists --------------------------------------------
ADR="$(find docs/decisions -maxdepth 1 -type f \
  \( -iname '*cache-coherence*.md' -o -iname '*l2-l3*.md' \) | sort | head -1)"
if [[ -z "${ADR}" || ! -f "${ADR}" ]]; then
  echo "FAIL (${ROW_ID}): no docs/decisions/*cache-coherence*.md (or *l2-l3*.md) found" >&2
  exit 1
fi
echo "  found ADR: ${ADR}" >&2

# --- Assert 2: both options named (canonical ids + L2/L3 labels) ---------
if ! grep -qF 're-fetch-on-cache-miss' "${ADR}" || ! grep -qE '\(L2\)' "${ADR}"; then
  echo "FAIL (${ROW_ID}): ADR does not name Option L2 (re-fetch-on-cache-miss)" >&2
  exit 1
fi
if ! grep -qF 'transactional-cache-writes' "${ADR}" || ! grep -qE '\(L3\)' "${ADR}"; then
  echo "FAIL (${ROW_ID}): ADR does not name Option L3 (transactional-cache-writes)" >&2
  exit 1
fi

# --- Assert 3: a CHOSEN path + trade-off, not just an enumeration ---------
if ! grep -qiE '^\|\s*\*\*Status\*\*\s*\|\s*\*\*ACCEPTED\*\*' "${ADR}" \
   && ! grep -qiE 'Status.*ACCEPTED' "${ADR}"; then
  echo "FAIL (${ROW_ID}): ADR Status is not ACCEPTED -- not yet a ratified decision" >&2
  exit 1
fi
if ! grep -qiE '^## Decision' "${ADR}"; then
  echo "FAIL (${ROW_ID}): ADR has no '## Decision' section (enumeration without a decision)" >&2
  exit 1
fi
if ! grep -qE 'Option C' "${ADR}"; then
  echo "FAIL (${ROW_ID}): ADR does not name the chosen path (Option C)" >&2
  exit 1
fi

# --- Assert 4: v0.14.0-deferral list + no-re-deferral-without-signoff -----
if ! grep -qF 'v0.14.0' "${ADR}"; then
  echo "FAIL (${ROW_ID}): ADR does not record a v0.14.0-deferral item" >&2
  exit 1
fi
if ! grep -qiE 'owner sign-?off' "${ADR}"; then
  echo "FAIL (${ROW_ID}): ADR does not state the no-re-deferral-without-owner-signoff rule" >&2
  exit 1
fi

# --- Assert 5: cross-references the honest-L1 outcome (RBF-LR-02) --------
if ! grep -qF 'RBF-LR-02' "${ADR}"; then
  echo "FAIL (${ROW_ID}): ADR does not cross-reference RBF-LR-02 (the honest-L1 refresh outcome)" >&2
  exit 1
fi

echo "PASS (${ROW_ID}): ${ADR} -- ACCEPTED, names L2+L3, chosen path (Option C) + trade-off, v0.14.0 deferral + owner-signoff rule, cross-refs RBF-LR-02"
exit 0
