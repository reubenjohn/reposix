---
quick_id: 260714-rcv
title: "Post-tag cursor refresh + carried-noticing intake filing (L0 rotation #21)"
status: ready
created: 2026-07-14
---

# Quick Task 260714-rcv — Post-tag cursor refresh + carried-noticing intake filing

Planning-artifact-only quick (no code, no cargo, no reposix/sim/git test setup — works
directly on the shared main tree). L0 rotation #21. Enter-through-GSD compliance for the
STATE.md cursor refresh that follows the post-tag queue closing green and Arc D being
ratified.

## Task boundary

- **In scope:** `.planning/STATE.md` cursor fields, two carried-noticing intake rows in
  the active v0.15.0 surprises-intake, and this quick's GSD artifacts.
- **Out of scope:** `.planning/MANAGER-HANDOVER.md`, `.planning/SESSION-HANDOVER.md` (do
  NOT touch), any code / gate rework, and the fixes the two intake rows describe (filed,
  not fixed).

## Task 1 — Refresh `.planning/STATE.md` cursor (edit in place)

Set frontmatter `status` / `last_updated` / `last_activity` to reflect: post-tag queue
items 0–5 CLOSED green (main green at 6aa734a, CI run 29384458026 success), Arc D
RATIFIED at 6aa734a (pipeline pause LIFTED, no-new-lanes constraint DISSOLVED), pipeline
now active on `/gsd-new-milestone` re-anchor (Arc D ratchet-first, v0.15 floor first).
Update the prose cursor lines (next_phase, Current Focus Workstream C, Session Continuity
live cursor) so no line still claims the post-tag queue is "in progress"; final grep
`in progress|in-progress|post-tag` must show only CLOSED/active/historical text.

## Task 2 — File two carried noticings as intake rows

Active surprises-intake home: `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md`
(current milestone's intake; the root `.planning/SURPRISES-INTAKE.md` does not exist and
is NOT created — the milestone-scoped file is the live home). Dedupe first, then append:

  (a) **MEDIUM** — milestone-scoped `GOOD-TO-HAVES.md` ledgers over the 20k
      `structure/file-size-limits` ceiling (v0.14.0 = 27629, v0.15.0 = 23584), masked by
      the `--warn-only` waiver expiring 2026-08-08; v0.14.0 also breaches the 24k
      `agent-ux/p111-milestone-hygiene` ceiling (genuine `exit 1`, on-demand cadence).
      Fix = progressive-disclosure split, preserving row IDs; home = Arc D v0.17 bloat
      remediation. Filed in SURPRISES-INTAKE (NOT GOOD-TO-HAVES — that file is the subject).
  (b) **LOW** — `.planning/milestones/v0.13.0-phases/ROADMAP.md` broken `**Plan:**` links
      (P79–P84) point at `NN-PLAN-OVERVIEW.md` files; real artifact is a
      `NN-PLAN-OVERVIEW/` directory. Fix = repoint to directory form.

## Task 3 — GSD artifacts + commit + push + verify

This PLAN + a post-execution SUMMARY; a `### Quick Tasks Completed` row in STATE.md;
atomic commit `docs(quick-260714-rcv): post-tag cursor refresh + carried-noticing intake
filing` (no `--no-verify`); `git push origin main`; then
`python3 quality/runners/run.py --cadence post-push --persist` with the
`code/ci-green-on-main` (P0) probe GREEN (poll the new CI run to `success` first).
