#!/usr/bin/env bash
# P111 milestone-close hygiene verifier (composite).
#
# Rolls up the P111 v0.14.0 milestone-close housekeeping items that are
# not covered by the ci-wait / CHANGELOG / RETROSPECTIVE rows. Every
# assert must hold for the row to flip PASS.
#
# Asserts:
#  A. ROADMAP.md carries a bolded TERMINAL completion status line inside
#     each of the Phase 103..109 blocks (item 4). "Terminal" = a
#     `**Status: ...`/`**STATUS: ...` line naming CLOSED|COMPLETE|
#     SHIPPED|DONE|RATIFIED (ambient "... GREEN" prose does NOT count).
#  B. ROADMAP.md references Phase 113 (item 3 — the lost-update renumber
#     reconciled into the roadmap: `Phase 113` or `P113`).
#  C. The p93-*.json under quality/reports/verifications/agent-ux/ are
#     NOT git-tracked (item 7 — `git ls-files ...p93...` is empty; these
#     are per-run verification artifacts, never committed).
#  D. crates/CLAUDE.md carries the pre-push cargo doctrine (item 2,
#     fix-twice) — sentinel: a "serialize push(es)" instruction near the
#     pre-push workspace-cargo note.
#  E. The three live ledgers are each under a no-ballooning size ceiling
#     (item 5 — none of the three headers state a numeric limit, and the
#     project *.md 20k budget is a LIVE WAIVED/deferred item for exactly
#     these files [file-size-limits waiver -> GOOD-TO-HAVES-02, tracked to
#     this milestone], so asserting <20k would contradict an active
#     waiver; per the catalog-first fallback we apply a no-ballooning
#     ceiling recorded here):
#       .planning/CONSULT-DECISIONS.md                       <= 30000 B
#       .../v0.14.0-phases/SURPRISES-INTAKE.md               <= 44000 B
#       .../v0.14.0-phases/GOOD-TO-HAVES.md                  <= 24000 B
#     (SURPRISES ceiling carries headroom for the P111 ci-wait RESOLVED
#     row; item 5's prune shrinks all three well below these.)
#  F. SURPRISES-INTAKE.md has ZERO `**STATUS:** OPEN` entries (fence-aware
#     — preserves the P110 drained invariant; a new OPEN row regresses
#     p110-surprises-absorption).
#
# Owner-hint on RED: the failing assert names the exact item + fix.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

ROADMAP=".planning/milestones/v0.14.0-phases/ROADMAP.md"
CONSULT=".planning/CONSULT-DECISIONS.md"
INTAKE=".planning/milestones/v0.14.0-phases/SURPRISES-INTAKE.md"
GTH=".planning/milestones/v0.14.0-phases/GOOD-TO-HAVES.md"
CRATES_CLAUDE="crates/CLAUDE.md"
REPORTS_DIR="quality/reports/verifications/agent-ux"

fail() { echo "FAIL: $1" >&2; exit 1; }

# --- A: Phase 103..109 terminal completion markers ---------------------
[[ -f "${ROADMAP}" ]] || fail "${ROADMAP} not found"
for N in 103 104 105 106 107 108 109; do
  BLOCK=$(awk -v n="${N}" '
    $0 ~ "^### Phase " n ":" { cap=1; next }
    cap && (/^### Phase / || /^## /) { exit }
    cap { print }
  ' "${ROADMAP}")
  if [[ -z "${BLOCK}" ]]; then
    fail "item 4: ${ROADMAP} has no '### Phase ${N}:' block"
  fi
  if ! printf '%s\n' "${BLOCK}" | grep -qE '\*\*(STATUS|Status)[:*].*(CLOSED|COMPLETE|SHIPPED|DONE|RATIFIED)'; then
    fail "item 4: ${ROADMAP} Phase ${N} block has no bolded terminal status line (**Status: CLOSED|COMPLETE|SHIPPED|DONE|RATIFIED ...)"
  fi
done

# --- B: Phase 113 reconciled into ROADMAP ------------------------------
if ! grep -qE '(Phase 113|P113|### Phase 113)' "${ROADMAP}"; then
  fail "item 3: ${ROADMAP} does not reference Phase 113 (lost-update renumber not reconciled)"
fi

# --- C: p93 verification artifacts untracked ---------------------------
P93_TRACKED=$(git ls-files "${REPORTS_DIR}" | grep -F 'p93' || true)
if [[ -n "${P93_TRACKED}" ]]; then
  fail "item 7: p93-*.json still git-tracked (run: git rm --cached ${REPORTS_DIR}/p93-*.json):
${P93_TRACKED}"
fi

# --- D: crates/CLAUDE.md pre-push cargo doctrine (fix-twice) ------------
[[ -f "${CRATES_CLAUDE}" ]] || fail "${CRATES_CLAUDE} not found"
if ! grep -qiE 'seriali[sz]e[[:space:]]+push' "${CRATES_CLAUDE}"; then
  fail "item 2: ${CRATES_CLAUDE} lacks the pre-push cargo doctrine sentinel ('serialize pushes' — pre-push runs a workspace cargo validate, so pushes must be serialized machine-wide)"
fi

# --- E: live-ledger no-ballooning size ceilings ------------------------
check_size() {
  local path="$1" cap="$2"
  [[ -f "${path}" ]] || fail "item 5: ${path} not found"
  local sz
  sz=$(wc -c < "${path}")
  if [[ "${sz}" -gt "${cap}" ]]; then
    fail "item 5: ${path} is ${sz} bytes, over its ${cap}-byte ceiling (prune closed/terminal entries per the file's policy header)"
  fi
}
check_size "${CONSULT}" 30000
check_size "${INTAKE}" 44000
check_size "${GTH}" 24000

# --- F: SURPRISES-INTAKE zero OPEN (preserve P110 drained invariant) ----
OPEN_COUNT=$(awk '
  /^```/ { in_fence = !in_fence; next }
  !in_fence && /^\*\*STATUS:\*\* OPEN/ { count++ }
  END { print count + 0 }
' "${INTAKE}")
if [[ "${OPEN_COUNT}" != "0" ]]; then
  fail "item 5/F: ${INTAKE} has ${OPEN_COUNT} STATUS: OPEN entr(y/ies) — regresses the P110 drained invariant; flip to a terminal STATUS"
fi

echo "PASS: P111 milestone-close hygiene — 103-109 terminal markers, Phase 113 reconciled, p93 untracked, pre-push doctrine present, ledgers bounded, SURPRISES 0-OPEN"
exit 0
