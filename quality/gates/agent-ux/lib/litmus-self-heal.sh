#!/usr/bin/env bash
# quality/gates/agent-ux/lib/litmus-self-heal.sh — sourced-only self-heal helpers for
# litmus-flow.sh (P125 / DRAIN-02+DRAIN-12). Factored out under the 10k .sh file-size
# budget, mirroring dark-factory/reconciliation-fixture.sh.
#
# Provides:
#   _litmus_fixture_preflight   — restore/reparent the three known TokenWorld fixture
#                                 ids (backend-drift self-heal). FIRST step of the flow.
#   _litmus_mirror_reconcile <tree>
#                               — overlay the backend-current pages/ tree onto the
#                                 mirror clone via the reposix BUS remote (mirror-drift
#                                 self-heal). Runs AFTER attach, BEFORE the marker edit,
#                                 so the edit bases on a backend-current tree and the
#                                 second-run stale-base rebase conflict never occurs.
#
# WHAT _litmus_mirror_reconcile DOES *NOT* DO (load-bearing distinction — ADR-010
# RBF-LR-04, root CLAUDE.md § "Mirror-head refresh promise"): the local `git checkout
# FETCH_HEAD -- pages/` overlay reconciles ONLY the litmus's throwaway LOCAL working tree
# to backend-current — it does NOT converge the external GitHub mirror repo. Its sole job
# is to stop the litmus false-negativing on a stale mirror clone (DRAIN-02). What actually
# refreshes the external mirror head is the SUBSEQUENT SoT-changing marker PUSH through
# the bus fan-out, downstream of this helper. This reconcile reaches the backend-current
# tree through the BUS remote (`git fetch reposix main` → the materialized SoT tree),
# NEVER via `reposix sync --reconcile` — that verb rebuilds only the LOCAL reposix cache
# and leaves the external mirror head byte-identical, so it is deliberately not used here.
# The `git rm -r --ignore-unmatch -- pages/` step before the overlay is kept so a
# backend-DELETED page cannot silently persist (an additive `checkout -- pages/` would
# retain it — refresh-tokenworld-mirror.sh header rationale).
#
# Caller MUST export before sourcing: REPO_ROOT (for confluence_tokenworld.py). `pass`/
# `fail` are defined by the caller. Runs with errexit OFF — every step guards explicitly.

# _litmus_fixture_preflight — backend-drift self-heal. Idempotently restore/reparent the
# three known TokenWorld fixture ids (2818063 sacrificial-editable + the 7766017/7798785
# durable protected pair — the safe superset per RESEARCH.md Pitfall 6 / Assumption A2).
# `restore` is a no-op when a page is already current, so this costs nothing when nothing
# drifted. Best-effort by design: the internal steps `|| true`, so a REST hiccup here
# never hard-fails the run before the clone; it always `pass`es.
_litmus_fixture_preflight() {
  local pid
  for pid in 2818063 7766017 7798785; do
    python3 "${REPO_ROOT}/scripts/confluence_tokenworld.py" restore "$pid" >&2 || true
  done
  # 7798785 is the CHILD of the durable protected pair — its parentId can go null even
  # after a status restore (Confluence does not restore the parent link on un-trash).
  if python3 "${REPO_ROOT}/scripts/confluence_tokenworld.py" inspect 7798785 \
       | grep -q '"parentId": null'; then
    python3 "${REPO_ROOT}/scripts/confluence_tokenworld.py" reparent 7798785 7766017 >&2 || true
  fi
  pass "fixture pre-flight: three known TokenWorld ids restored/reparented (idempotent)"
}

# _litmus_mirror_reconcile <tree> — mirror-drift self-heal. Overlay the backend-current
# pages/ tree onto the mirror clone via the BUS remote, adapting the proven
# refresh-tokenworld-mirror.sh:111-129 fetch + git rm + checkout FETCH_HEAD ordering.
# Set git identity first: the overlay may commit and git fails without it — reuse the
# EXACT values litmus-flow.sh uses at the marker commit, do not invent new ones. This
# reconciles the LOCAL tree only (see header): the marker PUSH downstream is what
# refreshes the external mirror head.
_litmus_mirror_reconcile() {
  local tree="$1"
  git -C "$tree" config user.email "litmus@reposix.invalid"
  git -C "$tree" config user.name "reposix-litmus"
  if ! git -C "$tree" fetch --quiet reposix main; then
    fail "mirror pre-reconcile: git fetch reposix main failed (bus remote unreachable)"; return 1
  fi
  git -C "$tree" rm -r --quiet --ignore-unmatch -- pages/ >/dev/null 2>&1 || true
  git -C "$tree" checkout FETCH_HEAD -- pages/ || { fail "mirror pre-reconcile: checkout FETCH_HEAD -- pages/ failed"; return 1; }
  git -C "$tree" add -A pages/
  if ! git -C "$tree" diff --cached --quiet; then
    git -C "$tree" commit --quiet -m "litmus pre-reconcile: sync mirror clone to backend-current" \
      || { fail "mirror pre-reconcile: commit of backend-current overlay failed"; return 1; }
  fi
  pass "mirror pre-reconcile: pages/ overlaid to backend-current via bus remote before marker edit"
}
