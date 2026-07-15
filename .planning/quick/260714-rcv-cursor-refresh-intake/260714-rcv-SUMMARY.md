---
quick_id: 260714-rcv
title: "Post-tag cursor refresh + carried-noticing intake filing (L0 rotation #21)"
status: complete
completed: 2026-07-14
---

# Quick Task 260714-rcv — SUMMARY (post-tag cursor refresh + carried-noticing intake filing)

Planning-artifact-only quick, L0 rotation #21. Refreshed the `.planning/STATE.md` cursor
to reflect the post-tag queue closing green + Arc D ratification, and filed two carried
noticings (verified against reality, with scope corrections) into the active v0.15.0
surprises-intake. No code, no cargo, no reposix/sim/git test setup.

## Task 1 — STATE.md cursor refresh

Edited in place:
- **Frontmatter `status`** → `v0.14.0-SHIPPED-public-posttag-queue-0-5-CLOSED-green-ArcD-RATIFIED-6aa734a-pipeline-active-on-new-milestone-prep-v0.15-floor`
- **`last_updated`** → `"2026-07-14"`
- **`last_activity`** → L0 rotation #21 narrative: post-tag queue items 0–5 CLOSED green
  (main green at 6aa734a, CI run 29384458026 success); Arc D RATIFIED at 6aa734a (manager
  under owner delegation; canonical record = ADDENDUM in
  `.planning/milestones/audits/2026-07-12-reality-check.md`, verified present at L582 with
  Arc D ratified at L605); pipeline pause LIFTED, no-new-lanes constraint DISSOLVED;
  pipeline now active on `/gsd-new-milestone` re-anchor (Arc D ratchet-first, v0.15 floor
  first).
- **Prose cursor lines** (next_phase frontmatter comment, Current Focus § Workstream C,
  Session Continuity live-cursor sentence) rewritten from "post-tag queue items 0-5 in
  progress / re-anchor deferred" → "post-tag queue 0-5 CLOSED green; Arc D RATIFIED
  (6aa734a); re-anchor now ACTIVE via /gsd-new-milestone (v0.15 floor first)".

Verification: final `grep -ni "in progress|in-progress" .planning/STATE.md` returns
nothing; the residual `post-tag` matches are historical/factual (STATE-history reference,
"item 1 handled", P112 stub creation-time narrative), none claiming the queue is open.

## Task 2 — Two carried noticings filed as intake rows

Active home located + used: **`.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md`**
(current milestone's live intake; the root `.planning/SURPRISES-INTAKE.md` does not exist
and was deliberately NOT created — the milestone-scoped file is the canonical live home,
alongside the v0.15.0 `GOOD-TO-HAVES.md`). Deduped first (grep confirmed neither present).
File grew 8950 → 13242 bytes (under the 20k `structure/file-size-limits` ceiling).

- **(a) MEDIUM — GOOD-TO-HAVES oversize.** Two milestone-scoped ledgers over the 20k
  `*.md` ceiling: `v0.14.0-phases/GOOD-TO-HAVES.md` = **27629 chars**, `v0.15.0-phases/
  GOOD-TO-HAVES.md` = **23584 chars**. Masked repo-wide by the `structure/file-size-limits`
  `--warn-only` waiver expiring **2026-08-08T00:00:00Z**. v0.14.0 also breaches the 24k
  `agent-ux/p111-milestone-hygiene` ceiling — a gate that genuinely `exit 1`s today
  (catalog `status: FAIL`, verified by running it) but is **on-demand/milestone-bounded**,
  so it does NOT gate main. Fix = progressive-disclosure split preserving row IDs; home =
  Arc D v0.17 bloat-remediation.
- **(b) LOW — v0.13.0 ROADMAP broken plan links.** Six `**Plan:**` links (P79–P84) point at
  `NN-PLAN-OVERVIEW.md` files; the real artifact is a `NN-PLAN-OVERVIEW/` directory. Fix =
  repoint to directory form.

## Files touched

- `.planning/STATE.md` — cursor frontmatter (3 fields) + 3 prose lines + 1 Quick Tasks row
- `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md` — 2 appended intake rows
- `.planning/quick/260714-rcv-cursor-refresh-intake/260714-rcv-PLAN.md` (new)
- `.planning/quick/260714-rcv-cursor-refresh-intake/260714-rcv-SUMMARY.md` (new, this file)

## Noticings (ownership OD-3 — deliverable, not optional)

1. **#20 hand-off imprecision — corrected in the filed rows (verify-against-reality).**
   The tasking named `.planning/GOOD-TO-HAVES.md` (~27.6k) for noticing (a); reality: that
   root file is **4525 chars** (fine). The 27629-char file is `v0.14.0-phases/
   GOOD-TO-HAVES.md`, and `v0.15.0-phases/GOOD-TO-HAVES.md` (23584) is also over 20k. Both
   milestone-scoped files, not the root, are the subjects.
2. **#20 hand-off undercount — corrected in row (b).** The carried note said "P80–82 and
   P84–88"; the true broken-link set is **P79–P84 (six)** — P79 and P83 were omitted, P78's
   link is FINE (real file), and P85–P88 have NO link (they say "TBD not yet authored",
   itself stale since those plans exist as `NN-01-PLAN.md`).
3. **`agent-ux/p111-milestone-hygiene` is a latent RED that never gates main.** It legit
   `exit 1`s on the v0.14.0 GTH oversize (27629 > 24000), carries catalog `status: FAIL` /
   `last_verified: null`, but its cadence is on-demand/milestone-bounded — so a real failing
   invariant sits un-surfaced until a milestone-close re-run. This is exactly the global-
   CLAUDE "metric you generate but don't watch silently decaying" pattern. Folded into
   intake row (a); worth a milestone-close checklist item so the 24k ceiling isn't a
   surprise at v0.15 close.
4. **`.planning/GOOD-TO-HAVES.md` (v0.15.0-phases) L65 row may now be stale.** It documents
   `ORCHESTRATION.md` as "26968 B vs its 20000 B ceiling, WAIVED until 2026-08-08", but
   `ORCHESTRATION.md` is now **19443 B** (under 20k) with `ORCHESTRATION-REFERENCE.md`
   (13163 B) already split out — i.e. that good-to-have appears RESOLVED. NOT touched here
   (that file is oversized + out of this quick's scope); flagged for the next
   GOOD-TO-HAVES sweep to flip the row's status.

## Deferred / not-done

- Both intake rows are FILED, not fixed (out-of-scope per charter).
- Noticing 4's stale GOOD-TO-HAVES row left untouched (editing the oversized ledger is out
  of scope and self-defeating; reported for the next sweep).

## Push + CI verification

Recorded in the follow-up commit / rotation report: pushed-commit SHA, its CI run id +
`success` conclusion, and the `code/ci-green-on-main` (P0) post-push probe result.
