---
phase: 116-adr-010-mirror-fanout-decision-packet-slug-id-durable-create
plan: 03
subsystem: planning-ledgers
tags: [adr-01, fix-03, surprises-intake, good-to-haves, status-flip]
requires:
  - "P116 manager rulings (2026-07-16, commit 8212373) in .planning/CONSULT-DECISIONS.md"
  - "Plan 116-02 ADR-010 §2/§3 durable record (commit 1ea51b3)"
provides:
  - "LIVE v0.15.0 litmus-non-idempotency SURPRISES row: terminal RESOLVED status"
  - "GOOD-TO-HAVES-09: SANCTIONED TARGET DESIGN status + boundary-relative TAG"
affects:
  - .planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md
  - .planning/GOOD-TO-HAVES.md
tech-stack:
  added: []
  patterns:
    - "Terminal-status flip in place (never delete the entry) — first in-file precedent for this ledger, shape copied from archived v0.14.0-phases/surprises-intake/part-03.md:313"
key-files:
  created:
    - .planning/phases/116-adr-010-mirror-fanout-decision-packet-slug-id-durable-create/116-03-SUMMARY.md
  modified:
    - .planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md
    - .planning/GOOD-TO-HAVES.md
decisions:
  - "GTH-09 TAG rewritten to 'boundary-relative — propose as a phase at the next milestone boundary' rather than hardcoding v0.16.0 (no v0.16.0-phases/ dir exists yet)"
  - "GOOD-TO-HAVES.md (root) does NOT cite GTH-V15-38 literally — that id lives in the separate v0.15.0-phases/GOOD-TO-HAVES.md ledger; grep count for GTH-V15-38 in the root file stays 0, unchanged from before this plan"
metrics:
  duration: ~10m
  completed: 2026-07-16
---

# Phase 116 Plan 03: Two planning-ledger status flips (ADR-01 SURPRISES retire + FIX-03 GTH-09) Summary

Two mechanical, precisely-pinned status flips durably record the 2026-07-16 rulings where
next-milestone roadmapping reads them: the LIVE litmus-non-idempotency SURPRISES row now
carries a terminal RESOLVED status, and GOOD-TO-HAVES-09 now records Option B as the
SANCTIONED TARGET DESIGN with a boundary-relative (not hardcoded) TAG.

## What shipped

- **T1 — LIVE litmus-non-idempotency row retired.** The `## 2026-07-14 20:42 | …
  litmus non-idempotency …` entry's own `**STATUS:** OPEN` (L116) flipped to RESOLVED,
  citing the 2026-07-16 [MANAGER] ruling, commit `8212373`, and `GTH-V15-38` as the
  elective Option C upgrade tracker. Entry body (What / Why-out-of-scope / Sketched
  resolution) left byte-unchanged — only the STATUS line was replaced. This is the
  ledger's first terminal-status row; shape copied from the archived v0.14.0 ruling-shape
  analog (no in-file precedent existed).
- **T2 — GOOD-TO-HAVES-09 sanctioned + boundary-relative.** STATUS flipped from `DEFERRED
  — owner scope call, 2026-07-12` to `SANCTIONED TARGET DESIGN (Option B) — 2026-07-16
  [MANAGER] ruling …; design recorded in ADR-010 §3; dedicated design+build phase proposed
  at the next milestone boundary. NO v0.15 build.` TAG rewritten from the hardcoded
  `v0.15.0` to boundary-relative phrasing matching `GTH-V15-38`'s "then-current milestone
  boundary" convention. Entry body (Discovered during / Size / Severity / One-line hazard /
  Fix sketch / Pointer / Default disposition) left untouched.

## Verification (all GREEN)

- T1 scoped awk: STATUS line under the `## 2026-07-14 20:42 …` heading reads RESOLVED,
  citing `8212373` and `GTH-V15-38`.
- T2: `grep -F "SANCTIONED TARGET DESIGN"` hits; boundary phrase present within the
  GOOD-TO-HAVES-09 block; `TAG:** v0.15.0` no longer present in that block.
- Archived v0.14.0 twin untouched: `git diff --stat -- .planning/milestones/v0.14.0-phases/
  | wc -l` == 0.
- `grep -c "GTH-V15-38" .planning/GOOD-TO-HAVES.md` == 0, unchanged from before the edit
  (that id is not, and was not, cited in the root ledger).
- `git diff --stat -- crates/ | wc -l` == 0.
- P115 `RETIRE_PROPOSED` gate count unchanged (0 — already closed by a prior phase; not a
  regression from this plan's edits).
- Full diff for both files reviewed by hand: only the two targeted STATUS/TAG lines
  changed, no neighboring row touched.

## Deviations from Plan

None — plan executed exactly as written. Both status flips landed at the exact pinned
locations; no architectural changes, no auto-fixes required.

## Noticed (ownership deliverable)

1. **Confirms plan's own precision guard.** `GTH-V15-38` is NOT defined in the root
   `.planning/GOOD-TO-HAVES.md` — it lives in the separate per-milestone
   `.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md` ledger. The plan's T2 action
   correctly asked only to copy the *phrasing* ("then-current milestone boundary"), not to
   cite the id, so no cross-file id reference was introduced. Flagging for the next reader
   so the two GOOD-TO-HAVES.md files (root vs. per-milestone) aren't conflated.
2. **LOW (carried from 116-02 SUMMARY Noticed #2, still present).** `GTH-V15-38`'s block in
   `.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md` still carries copy-paste-bled
   duplicate lines from `GTH-V15-37` above it. Out of this plan's edit set (root
   GOOD-TO-HAVES.md, not the per-milestone one) — already filed, no new filing needed.

## Self-Check: PASSED

- `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md` — FOUND, litmus row STATUS
  reads RESOLVED, entry body intact.
- `.planning/GOOD-TO-HAVES.md` — FOUND, GOOD-TO-HAVES-09 STATUS/TAG updated, entry body
  intact.
- `.planning/phases/116-…/116-03-SUMMARY.md` — FOUND (this file).
- Commit hash recorded in the final report.
