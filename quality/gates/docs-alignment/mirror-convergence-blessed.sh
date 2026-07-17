#!/usr/bin/env bash
# ADR-01 Decision 1(i)/(ii) — webhook + 30-minute cron is the AUTHORITATIVE
# external-mirror convergence mechanism, not a workaround. Neither live doc
# said so explicitly before this phase (the literal word "authoritative"
# appeared 0 times in both root CLAUDE.md and docs/concepts/dvcs-topology.md
# — verified 2026-07-16). This verifier discriminates on "authoritative"
# specifically, NOT the bare word "webhook" (already present 3x in CLAUDE.md
# and 11x in dvcs-topology.md before this phase — a `grep webhook` gate would
# be tautological, always-green). Ruling: .planning/CONSULT-DECISIONS.md,
# 2026-07-16, commit 8212373.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
CLAUDE_MD="${REPO_ROOT}/CLAUDE.md"
TOPOLOGY_DOC="${REPO_ROOT}/docs/concepts/dvcs-topology.md"

if [[ ! -f "$CLAUDE_MD" ]]; then
  echo "FAIL: CLAUDE.md does not exist (mirror-convergence-blessed)" >&2
  exit 1
fi
if [[ ! -f "$TOPOLOGY_DOC" ]]; then
  echo "FAIL: docs/concepts/dvcs-topology.md does not exist (mirror-convergence-blessed)" >&2
  exit 1
fi

if ! grep -qF -- "authoritative" "$CLAUDE_MD"; then
  echo "FAIL: CLAUDE.md does not bless webhook+cron as the AUTHORITATIVE external-mirror" >&2
  echo "      convergence mechanism (missing the literal word 'authoritative')." >&2
  echo "      Fix: inside the § \"Mirror-head refresh promise\" bullet, add one clause" >&2
  echo "      naming webhook + 30-minute cron as authoritative, citing" >&2
  echo "      docs/guides/dvcs-mirror-setup.md. See ADR-01 Decision 1(i)." >&2
  exit 1
fi

if ! grep -qF -- "authoritative" "$TOPOLOGY_DOC"; then
  echo "FAIL: docs/concepts/dvcs-topology.md does not bless webhook+cron as the AUTHORITATIVE" >&2
  echo "      external-mirror convergence mechanism (missing the literal word 'authoritative')." >&2
  echo "      Fix: inside § \"Cache coherence: L1/L2/L3 (ADR-010)\", extend the L1 bullet with" >&2
  echo "      the (a) cache-ref / (b) external-mirror-repo split, citing" >&2
  echo "      docs/guides/dvcs-mirror-setup.md. See ADR-01 Decision 1(i)." >&2
  exit 1
fi

if ! grep -qF -- "refresh-tokenworld-mirror.sh" "$TOPOLOGY_DOC"; then
  echo "FAIL: docs/concepts/dvcs-topology.md does not name scripts/refresh-tokenworld-mirror.sh" >&2
  echo "      as manual op-recovery only (not a convergence mechanism)." >&2
  echo "      Fix: name the script's full basename alongside the (a)/(b) mirror-sense split." >&2
  echo "      See ADR-01 Decision 1(ii)." >&2
  exit 1
fi

echo "PASS: webhook + 30-minute cron blessed as AUTHORITATIVE external-mirror convergence in both live docs"
exit 0
