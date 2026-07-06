---
phase: 260706-idp-v0.13.0-intake-disposition
plan: 01
type: quick
autonomous: true
subsystem: planning (v0.13.0 intake registries)
requirements: []
---

# Quick 260706-idp: v0.13.0 intake OP-8 disposition + bound-to-live-state sweep

## Objective

Last cleanup before the owner cuts the v0.13.0 tag: sweep the two v0.13.0 intake
registries (`SURPRISES-INTAKE.md`, `GOOD-TO-HAVES.md`) so a skeptical reader opening them
post-tag sees a clean, non-dangling, correctly-scoped carry-forward backlog for the
post-tag v0.14.0 / v0.13.2 scoping session. Planning-doc edits only — NO Rust, NO
`docs/**`, NO cargo. Do NOT create a `v0.14.0-phases/` dir (post-tag activity).

## Context

- `.planning/SESSION-HANDOVER.md` §0 (bound-to-live-state: DELETE terminal, git is the archive) + §5 (next-session sweep).
- v0.13.0 CLOSED-GREEN, tag imminent; open entries are the live carry-forward ledger.

## Tasks

1. **[auto] SURPRISES-INTAKE.md** — add a ≤6-line carry-forward banner (rule C); DELETE the
   2 genuinely-terminal entries (P89-02 banned-token-marker-already-applied; phantom-green
   verified-CLEAN P97 Wave A); KEEP all OPEN/deferred entries verbatim. Confirm no un-addressed
   HIGH/BLOCKER dangles.
2. **[auto] GOOD-TO-HAVES.md** — add the carry-forward banner (rule C); DELETE the 4 completed
   `RESOLVING-P97` drain-ledger rows (GTH-02/08/09/13, landed at `302e8ec`); update the tally;
   KEEP all DEFERRED-* / OWNER-ACTION rows; FILE one new MEDIUM item (troubleshooting.md 25.5k
   > 20k progressive-disclosure limit, rule E).
3. **[auto] STATE.md** — record this quick in "Quick Tasks Completed".

## Verification / success criteria

- 3 HIGHs (+ 2 more HIGH-severity entries) confirmed live carry-forward; zero dangling HIGH/BLOCKER.
- Freshness-invariants gate + banned-words gate SEE exit 0.
- Committed atomically, pushed to origin/main before reporting done.
