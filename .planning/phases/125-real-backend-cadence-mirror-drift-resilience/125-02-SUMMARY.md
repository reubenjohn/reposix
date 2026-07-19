---
phase: 125-real-backend-cadence-mirror-drift-resilience
plan: 02
subsystem: quality
tags: [agent-ux, vision-litmus, mirror-drift, backend-drift, self-heal, dvcs, tokenworld]

# Dependency graph
requires:
  - phase: 125-01 (SC3 / DRAIN-12)
    provides: the corrected mirror-lag reject hint whose cold-read is cross-checked in this plan's Task 2 doc
  - phase: refresh-tokenworld-mirror.sh (v0.14.0 item 5)
    provides: the proven fetch + git rm + checkout FETCH_HEAD overlay mechanism this plan composes
  - phase: confluence_tokenworld.py
    provides: the idempotent restore/reparent CLI the fixture pre-flight composes
provides:
  - New sourced helper lib/litmus-self-heal.sh with two self-heal steps composed from proven in-repo mechanisms
  - Backend-drift self-heal (_litmus_fixture_preflight) — idempotent restore/reparent of the three known TokenWorld ids before the mirror clone
  - Mirror-drift self-heal (_litmus_mirror_reconcile) — BUS-remote fetch + git rm + checkout FETCH_HEAD overlay of the LOCAL tree before the marker edit, removing the DRAIN-02 second-run stale-base false-negative
  - Documented DRAIN-02 run-twice regression proof + non-destructive backend-drift manual verification in the milestone-close verdict template
affects: [125-03, mirror-drift-resilience, milestone-close-vision-litmus, pre-release-real-backend]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Reconcile-before-edit: overlay backend-current pages/ BEFORE authoring the marker so the push never carries a stale base (vs the reactive post-rejection bounded retry, kept as a backstop)"
    - "Mirror reconcile routes through the BUS remote (git fetch reposix main → checkout FETCH_HEAD) and reconciles the LOCAL tree only; the SoT-changing marker PUSH refreshes the external mirror head via the bus fan-out — NEVER reposix sync --reconcile (local-cache-only)"
    - "git rm -r --ignore-unmatch -- pages/ before the overlay so backend-side deletions propagate (T-125-02b) instead of silently persisting"
    - "Factor sourced helpers out of a near-ceiling .sh file, mirroring dark-factory/reconciliation-fixture.sh, to hold the 10k budget"

key-files:
  created:
    - quality/gates/agent-ux/lib/litmus-self-heal.sh
  modified:
    - quality/gates/agent-ux/lib/litmus-flow.sh
    - quality/dispatch/milestone-close-verdict.md

key-decisions:
  - "Kept the load-bearing distinction sharp in code, comments, and the verdict doc: the local checkout FETCH_HEAD overlay does NOT converge the external mirror repo — it only stops the litmus false-negativing on a stale clone; the downstream marker PUSH refreshes the mirror head. reposix sync --reconcile is deliberately absent (non-comment grep-count == 0)"
  - "Preserved the existing one-shot fetch-rebase-retry backstop (litmus-flow.sh) unchanged — a rejection persisting past it stays a genuine coherence-bug hard RED; no second/looping retry added"
  - "Removed the now-redundant marker-commit git config user.email/user.name (identity is set by _litmus_mirror_reconcile on the same $tree earlier in the flow) — plan-sanctioned, and it reclaimed budget to hold litmus-flow.sh at 9981 bytes (< 10k)"
  - "Fixture pre-flight called unguarded (its internal steps || true, always passes) so a TokenWorld REST hiccup never hard-fails the run before the clone; safe superset over all three known ids per RESEARCH.md Pitfall 6 / A2"

patterns-established:
  - "Litmus self-heal ordering: fixture pre-flight (backend drift) → mirror pre-reconcile (mirror drift, before target selection/edit) → existing GUARD A/B → marker edit → push → existing bounded backstop"

requirements-completed: [DRAIN-12, DRAIN-02]

# Metrics
duration: ~40min
completed: 2026-07-19
---

# Phase 125 Plan 02: Litmus self-heal (backend drift + mirror drift) Summary

Folds backend-drift and mirror-drift self-healing into the milestone-close vision-litmus by
composing two proven in-repo mechanisms into a new sourced `lib/litmus-self-heal.sh`, so the
litmus self-heals BOTH failure classes before it edits and pushes — removing the DRAIN-02
second-run false-negative where run 1's own bus push re-stales the GitHub mirror.

## What was built

**Task 1 — `lib/litmus-self-heal.sh` + wiring (`feat` `3864e1a6`).**

- `_litmus_fixture_preflight` (backend-drift, DRAIN-12): idempotently `restore`s the three
  known TokenWorld ids (`2818063`, `7766017`, `7798785`) via the existing
  `scripts/confluence_tokenworld.py`, then `reparent`s `7798785 → 7766017` if its `parentId`
  went null. Composed, not re-implemented; `restore` no-ops when a page is already current.
- `_litmus_mirror_reconcile <tree>` (mirror-drift, DRAIN-02): sets git identity, then
  `git fetch reposix main` (BUS remote) → `git rm -r --quiet --ignore-unmatch -- pages/` →
  `git checkout FETCH_HEAD -- pages/` → stage/commit the overlay. Adapts the exact
  `refresh-tokenworld-mirror.sh:111-129` ordering (the `git rm` step preserved so backend
  deletions propagate — T-125-02b).
- Wired into `litmus-flow.sh`: `source` the helper; call `_litmus_fixture_preflight` as the
  first step (before the clone); call `_litmus_mirror_reconcile "$tree" || return 1` after
  the attach config-check, before GUARD A — so GUARD A and the marker edit both base on a
  backend-current `pages/` tree. The existing bounded self-heal backstop is untouched.

**Task 2 — DRAIN-02 run-twice + backend-drift manual verification (`docs` `35c41ac1`).**

Added `### DRAIN-02 second-run mirror-drift regression (P125)` to
`quality/dispatch/milestone-close-verdict.md`: (1) run `pre-release-real-backend` TWICE in
immediate succession, both exit 0 (second run is the regression proof); (2) non-destructive
backend-drift check via `confluence_tokenworld.py list` (do NOT trash the protected pair);
(3) cold-read of the corrected Plan 01 teaching string. Consistent with 125-VALIDATION.md's
Manual-Only table.

## How both drift classes are reconciled (exact commands)

- **Backend drift:** `python3 .../confluence_tokenworld.py restore <id>` (× the three ids) +
  a conditional `reparent 7798785 7766017` gated on `inspect ... | grep -q '"parentId": null'`.
- **Mirror drift:** `git -C "$tree" fetch --quiet reposix main` → `git -C "$tree" rm -r
  --quiet --ignore-unmatch -- pages/` → `git -C "$tree" checkout FETCH_HEAD -- pages/`. This
  is a LOCAL-tree overlay through the SoT-backed bus remote; the external mirror head is
  refreshed by the run's own SoT-changing marker PUSH (bus fan-out), NOT by this overlay.

## Verification (no real-backend hit — within the phase's per-commit boundary)

- `bash -n` clean on both shell files; both source cleanly and define all four functions
  (`_litmus_flow`, `_litmus_fixture_preflight`, `_litmus_mirror_reconcile`,
  `patch_litmus_artifact`) — verified by sourcing with stub `pass`/`fail`.
- `litmus-flow.sh` = **9981 bytes** (< 10000 `.sh` ceiling); the plan's exact automated
  verify line exits `OK`.
- **Mirror-reconcile grep gate:** `grep -v '^#' litmus-self-heal.sh | grep -c "reposix sync
  --reconcile"` == **0** — confirmed. `reposix sync --reconcile` appears only in the header
  comment (explaining why it is NOT used), never on an executable line.
- `checkout FETCH_HEAD -- pages/` and the `git rm -r --quiet --ignore-unmatch -- pages/`
  step both present (git rm confirmed via `grep -F`; see Deviations for the ugrep-BRE note).
- Bounded backstop string intact in `litmus-flow.sh`.
- `banned-words` gate PASS; `milestone-close-verdict.md` = 6994 bytes (< 20000 `.md` ceiling);
  pre-commit `structure/*` gates PASS on both commits.
- The full two-run real-backend litmus is the phase-close/milestone-close manual proof
  (documented in Task 2 + 125-VALIDATION.md); NOT run here per the real-backend boundary.

## Deviations from Plan

### Auto-fixed / in-scope adjustments

**1. [Rule 3 - Size budget] Held litmus-flow.sh under the 10k `.sh` ceiling.**
- **Found during:** Task 1. The file was already 9794 bytes (97.9% of ceiling) before any
  wiring; adding `source` + two call sites + comments pushed it to 10929 (over).
- **Fix:** (a) removed the now-redundant marker-commit `git config user.email/user.name`
  (the plan explicitly sanctions this — identity is set by `_litmus_mirror_reconcile` on the
  same `$tree` earlier); (b) tightened the verbose marker "VISIBLE text" explanatory comment
  to 3 lines while preserving every load-bearing fact (visible-not-comment, sanitizer strips,
  no new version, transcript ref, drop-prior-marker); (c) made the two new wiring comments
  terse pointers (the full local-vs-mirror framing lives in the helper header). Final: 9981
  bytes. The 10k gate is `--warn-only`-WAIVED until 2026-08-08 so this never blocked, but the
  plan's Task 1 AC hard-requires < 10000, which is met.
- **Files:** quality/gates/agent-ux/lib/litmus-flow.sh — **Commit:** 3864e1a6

## Known Stubs

None. Both self-heal helpers compose real, already-tested tooling (`confluence_tokenworld.py`,
the `refresh-tokenworld-mirror.sh` overlay mechanism); no placeholder/empty-data paths added.

## Threat Flags

None. No new egress path or credential echo. `_litmus_fixture_preflight` composes the
already-audited `confluence_tokenworld.py` (egress behind `REPOSIX_ALLOWED_ORIGINS` + tenant
creds; prints status lines, never page bodies). `_litmus_mirror_reconcile` uses the
egress-gated bus remote and writes only inside the caller's `mktemp -d` `$tree`; no
`$MIRROR_URL`/`$rurl` echoed, so no redactor needed (T-125-02 disposition: accept — held).
`git rm` step preserved (T-125-02b: mitigate — held, grep-gated).

## Noticed (owner mandate OD-3)

1. **Plan AC #3 git-rm grep is BRE-fragile under ugrep.** The AC
   `grep -q "git -C \"\$tree\" rm -r --quiet --ignore-unmatch -- pages/"` FAILS on this
   environment's `grep` (which is ugrep 7.5.0): ugrep treats the mid-pattern `$` in `"$tree"`
   as an end-of-line anchor, so the literal-substring match returns non-zero even though the
   line is present verbatim (confirmed via `grep -F`, exit 0). GNU grep treats a non-terminal
   `$` as literal and would pass. Recommend the phase-close verifier use `grep -F` (or
   `grep -qF`) for any AC whose pattern embeds a `$`-bearing shell variable. Substance is
   correct — the `git rm` step is preserved exactly as the plan mandates. Low severity
   (verifier-tooling caveat, not a code defect); not filed to intake as it is a one-line
   grader-flag rather than repo work.
2. **litmus-flow.sh runs permanently in the early-warning size band.** At 9981 bytes it is
   99% of the 10k `.sh` ceiling — every future edit to this file will need the same
   factor-into-a-sourced-helper discipline. The helper split this plan introduced is the
   right lever; a future plan that adds more litmus steps should extend
   `lib/litmus-self-heal.sh` rather than inline into `litmus-flow.sh`.

## Self-Check: PASSED

- Created file present: `quality/gates/agent-ux/lib/litmus-self-heal.sh` — FOUND.
- Modified files present: `quality/gates/agent-ux/lib/litmus-flow.sh`,
  `quality/dispatch/milestone-close-verdict.md` — FOUND.
- Commits present in `git log`: `3864e1a6` (feat, Task 1), `35c41ac1` (docs, Task 2) — FOUND.
- SUMMARY 10912 bytes (< 20000 `.md` ceiling).
