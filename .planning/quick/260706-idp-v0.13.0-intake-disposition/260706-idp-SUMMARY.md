---
phase: 260706-idp-v0.13.0-intake-disposition
plan: 01
subsystem: planning (v0.13.0 intake registries)
tags: [planning, op-8, intake, carry-forward, bound-to-live-state, pre-tag, quick]
requires: []
provides:
  - SURPRISES-INTAKE.md (carry-forward banner + 2 terminal entries deleted; live backlog preserved)
  - GOOD-TO-HAVES.md (carry-forward banner + 4 completed RESOLVING-P97 rows deleted + tally corrected + 1 new MEDIUM filed)
affects: []
tech-stack:
  added: []
  patterns: []
key-files:
  created:
    - .planning/quick/260706-idp-v0.13.0-intake-disposition/260706-idp-PLAN.md
    - .planning/quick/260706-idp-v0.13.0-intake-disposition/260706-idp-SUMMARY.md
  modified:
    - .planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md
    - .planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md
    - .planning/STATE.md
decisions:
  - Bound-to-live-state applied conservatively — only 2 SURPRISES entries proved genuinely terminal (work DONE, not merely deferred) and were deleted; every OPEN/deferred entry kept verbatim (deletion is the risky direction).
  - Chose ONE ≤6-line carry-forward banner per file over per-entry STATUS rewrites — the 18 stale P95/P97 phase refs are re-interpreted by the banner ("deferred past that closed phase, now pending v0.14.0 re-triage") without churning each line.
  - Deleting the phantom-green entry drops its DEFERRED-v0.14.0 load-time-guard residual from the live ledger (git retains it); the residual is a member of the loader-hardening family still represented by 2 live sibling entries — noted for the v0.14.0 scoper.
  - Filed the troubleshooting.md progressive-disclosure split as its own MEDIUM with an explicit dedupe cross-ref to GOOD-TO-HAVES-15 (which tracks the raw file-size overage across 9 files) so it is not read as a duplicate.
metrics:
  duration: ~25 minutes
  completed: 2026-07-06
---

# Quick 260706-idp: v0.13.0 intake OP-8 disposition + bound-to-live-state sweep Summary

One-liner: swept the two v0.13.0 intake registries into a clean pre-tag carry-forward ledger — added one ≤6-line banner per file re-scoping every OPEN entry to the post-tag v0.14.0/v0.13.2 session, deleted 2 terminal SURPRISES entries + 4 completed RESOLVING-P97 drain rows (git is the archive), preserved all deferred carry-forward + 5 live HIGHs, and filed one new MEDIUM (troubleshooting.md 25.5k > 20k). Planning-doc edits only; no code, no docs/**, no cargo.

## SURPRISES-INTAKE.md

- **Banner added** (rule C, top of file): v0.13.0 CLOSED-GREEN/tag-imminent; every OPEN entry is a live carry-forward to the post-tag v0.14.0/v0.13.2 scoping session; a STATUS citing a closed P9x phase means "deferred past that closed phase, now pending v0.14.0 re-triage"; terminal entries DELETED (git is the archive); do NOT create a `v0.14.0-phases/` dir.
- **DELETED 2 terminal entries** (bound-to-live-state, work proven DONE): (1) `2026-07-03 20:16 P89-02` — banned-token scan marker "already applied in-task," sketch "None required beyond the marker already applied"; (2) `2026-07-05 phantom-green (status:WAIVED + waiver:null)` — P97 Wave A reconciliation proved "phantom-green grep is CLEAN, ZERO rows, no active phantom-green rows this milestone."
- **KEPT** every other entry verbatim (no per-entry STATUS churn; the banner re-scopes the stale P95/P97 refs).

## GOOD-TO-HAVES.md

- **Banner added** (rule C) — same carry-forward framing, noting the P97 drain ledger below is historical.
- **DELETED 4 completed `RESOLVING-P97` drain-ledger rows** (terminal, landed at `302e8ec`): GTH-02 (linter regex), GTH-08 (raise-list split), GTH-09 (trust-model WAL doc-note), GTH-13 (grep-vs-rg note). These existed ONLY as table rows (full entries were already removed at 302e8ec).
- **Tally corrected**: `39 topics / 40 lines` → `35 topics / 36 lines`; dropped `RESOLVING-P97 = 4`; kept the still-live GTH-10/GTH-12 DEFERRED-v0.14.0 reversion note.
- **KEPT** all DEFERRED-v0.14.0 / DEFERRED-post-tag / DEFERRED-to-Wave-B-mint (row 23) / OWNER-ACTION (row 18, GTH-18 JIRA secret) rows as carry-forward.
- **FILED 1 new MEDIUM** (rule E): `docs/guides/troubleshooting.md` 25,503 chars > 20k progressive-disclosure limit; split into a child page per DVCS symptom class; >1h (restructures cross-linked anchors); dedupe cross-ref to GOOD-TO-HAVES-15.

## HIGH/BLOCKER dangle check (rule F)

Zero BLOCKER entries exist. All **5 HIGH-severity** SURPRISES entries confirmed live carry-forward (none dangling): (1) RUSTSEC dependabot #64/#65/#66 [OPEN/re-sequenced]; (2) RBF-FW-11 date-cutoff 89-07 [OPEN]; (3) pagination-truncation `prune_oid_map` [OPEN]; (4) quality-convergence swarm write-contention [ROUTED-P95 → carry-forward]; (5) TokenWorld two-writer conflict verifier [OPEN, P97-reconciled]. The task named 3; I confirmed all 5. Not fixed (all cargo/architectural, out of a doc-quick's scope).

## Deviations from Plan

None — plan executed as written. Note: the charter named "the 3 HIGHs"; the file actually carries 5 HIGH-severity entries, all confirmed live (thoroughness, not a deviation).

## Verification (SEEN, not assumed)

- `freshness-invariants.py --row-id structure/no-loose-roadmap-or-requirements` → EXIT=0
- `... structure/no-loose-top-level-planning-audits` → EXIT=0
- `... structure/no-orphan-docs` → EXIT=0
- `... structure/no-version-pinned-filenames` → EXIT=0
- `bash quality/gates/structure/banned-words.sh` → `✓ banned-words-lint passed (all mode).` EXIT=0

## Noticing (OD-3 deliverable)

- Deleting the phantom-green SURPRISES entry drops its one live residual — the DEFERRED-v0.14.0 "load-time guard: assert `status==WAIVED ⟺ well-formed waiver`, else raise NOT-VERIFIED at load" hardening idea — from the live ledger. Git retains the full text (`git show HEAD~1:...SURPRISES-INTAKE.md`). It is a member of the catalog-loader-honesty family still represented by two live sibling entries (the `--persist` load-refusal write-path entry and the `2026-07-06 env-gate/minted_at` load-fragility entry in GOOD-TO-HAVES), so a v0.14.0 quality-framework hardening phase picking up those will naturally re-encounter it. Flagged so the deletion is transparent and reversible.
- The GOOD-TO-HAVES vocabulary blockquote (line ~12) still defines `RESOLVING-P97` even though no table row now carries it. Left intact as historical drain-decision framing (the tally narrative explains the rows were deleted); a future v0.14.0 ingest can trim it. Low value; not churned this pass.
- STATE.md OWNER PRE-TAG ACTION #4 ("Disposition-or-carry-forward the 19 bare-OPEN SURPRISES entries") is materially satisfied by this sweep — the banner + terminal deletions convert the bare-OPEN backlog into an explicitly-scoped carry-forward ledger with zero dangling HIGH/BLOCKER. The owner/L0 may now treat #4 as cleared.

## Self-Check: PASSED
