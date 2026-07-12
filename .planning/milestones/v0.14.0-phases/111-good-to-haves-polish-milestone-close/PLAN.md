---
phase: 111
plan: 01
title: "+2 reservation Slot 2 — good-to-haves polish + v0.14.0 milestone-close (OP-9)"
type: milestone-close
autonomous: true
requirements: [D2-GOOD-TO-HAVES-01, D2-MILESTONE-CLOSE-01, D2-RETROSPECTIVE-DISTILL-01]
depends_on: [P110]
provides: [v0.14.0-ready-to-tag, op9-retrospective-v0.14.0]
affects:
  - CHANGELOG.md
  - .planning/RETROSPECTIVE.md
  - .planning/milestones/v0.14.0-phases/ROADMAP.md
  - .planning/CONSULT-DECISIONS.md
  - .planning/milestones/v0.14.0-phases/SURPRISES-INTAKE.md
  - .planning/milestones/v0.14.0-phases/GOOD-TO-HAVES.md
  - crates/CLAUDE.md
  - scripts/ci-wait.sh
  - quality/reports/verifications/agent-ux/p93-*.json
---

# Phase 111: +2 reservation Slot 2 — good-to-haves polish + v0.14.0 milestone-close (OP-9)

> **Boundary note.** This phase drives v0.14.0 to a fully-ratified,
> ready-to-tag state **UP TO (not including) the owner tag-cut**. The
> executor STOPs at the tag boundary — it does NOT author or run
> `tag-v0.14.0.sh`, and it does NOT push the tag. The tag script and the
> non-skippable 9th probe (`pre-release-real-backend`) are **owner
> territory** (see § Out of scope).

## Objective

Close the v0.14.0 milestone per OP-8 (Slot 2 good-to-haves polish) + OP-9
(distill-before-archive), landing the milestone-close hygiene checklist so
the four P111 catalog rows in `quality/catalogs/agent-ux.json`
(`agent-ux/p111-*`) flip PASS under an unbiased verifier and the OP-9
retrospective ratification grades GREEN. Every deliverable is
milestone-bounded (cadence `on-demand`; nothing fires pre-push).

## Context

@quality/catalogs/agent-ux.json (the four `agent-ux/p111-*` GREEN-contract rows)
@.planning/milestones/v0.14.0-phases/ROADMAP.md (Phase 111 block — SC alignment)
@.planning/milestones/v0.14.0-phases/GOOD-TO-HAVES.md (OP-8 Slot 2 drain source)
@.planning/milestones/v0.14.0-phases/SURPRISES-INTAKE.md (P110-drained; preserve 0-OPEN)
@CLAUDE.md (OP-8 / OP-9 rituals; the ownership charter OD-3)

The four catalog rows were minted `status: FAIL` in the catalog-first
commit (`chore(quality): P111 catalog-first ...`). The unbiased verifier
reads those rows — which predate the implementation — and flips them PASS
once the tasks below land.

## Tasks (the 8-item milestone-close hygiene checklist)

### Task 1 — `type="auto"` — ci-wait bounded-poll helper `scripts/ci-wait.sh`

**Landed by the FOUNDATION lane in this phase.** Promote the ad-hoc
background `gh run watch` — which HANGS on already-concluded GREEN runs
(evidence hang IDs `bulqmsyrv`, `biy9yxt33`) — into a committed
bounded-poll helper (CLAUDE.md OP-4: promote ad-hoc bash). Spec: bounded
poll of `gh run view <id> --json status,conclusion` every
`CI_WAIT_INTERVAL` (default 20s) up to `CI_WAIT_TIMEOUT` (default 900s);
immediate return when the first query shows status `completed`; exit 0
only on conclusion `success`, exit 2 on hard-timeout.
**Row:** `agent-ux/p111-ci-wait-helper`.

### Task 2 — `type="auto"` — fix-twice: crates/CLAUDE.md pre-push cargo doctrine

The pre-push hook runs a **workspace** cargo validate (~100s). Two
concurrent pushes racing that hook contend for the single machine-wide
cargo token (OOM history). Add the doctrine to `crates/CLAUDE.md`:
pre-push runs a workspace cargo validate, so **serialize pushes**
machine-wide (never start a push while another push's pre-push is
running). Fix-twice: the fix IS the doc (this is a grounding gap).
**Sentinel (row asserts):** a `seriali[sz]e push` instruction.
**Row:** `agent-ux/p111-milestone-hygiene` (assert D).

### Task 3 — `type="auto"` — Phase 113 reconciliation into ROADMAP

The lost-update HIGH fix was minted at Phase **113** (renumbered from a
106 collision; see `113-lost-update-shared-cursor/PLAN.md`) but
`ROADMAP.md` has no `### Phase 113` heading (filed in CONSULT-DECISIONS +
SURPRISES-INTAKE). Add a Phase 113 entry to the v0.14.0 ROADMAP so the
phase-number scheme is coherent for the tag-cut.
**Row:** `agent-ux/p111-milestone-hygiene` (assert B).

### Task 4 — `type="auto"` — ROADMAP P103–P109 completion status

Each of Phase 103..109 must carry a bolded **terminal** status line
(`**Status: CLOSED|COMPLETE|SHIPPED|DONE|RATIFIED ...`) reflecting its
close state (Phase 108 already has one). Ambient "... GREEN" prose does
NOT satisfy the row — the marker must be an explicit bolded status line.
**Row:** `agent-ux/p111-milestone-hygiene` (assert A).

### Task 5 — `type="auto"` — prune CONSULT-DECISIONS + intakes per policy

Prune `.planning/CONSULT-DECISIONS.md` (delete CLOSED/superseded
decisions — `git log` is the archive), `SURPRISES-INTAKE.md`, and
`GOOD-TO-HAVES.md` per each file's policy header. Keep each under its
no-ballooning ceiling (CONSULT ≤30000 B, SURPRISES ≤44000 B, GTH ≤24000 B
— see the hygiene row's assert E rationale: no header states a numeric
limit and the *.md 20k budget is a LIVE WAIVED item for these three
files). **Preserve the P110 zero-OPEN invariant** in SURPRISES-INTAKE.
**Row:** `agent-ux/p111-milestone-hygiene` (asserts E, F).

### Task 6 — `type="auto"` — OP-9 RETROSPECTIVE v0.14.0 section

Distill SURPRISES + GOOD-TO-HAVES + per-phase verdicts into a new
`## Milestone: v0.14.0` section in `.planning/RETROSPECTIVE.md` **BEFORE
archive**, using the OP-9 template (all 5 subheadings: What Was Built /
What Worked / What Was Inefficient / Patterns Established / Key Lessons).
The section MUST name the **GTH-09 → v0.15.0** deferral explicitly (an
unnamed deferral is the dishonesty OP-9 forbids).
**Row:** `agent-ux/p111-retrospective-v0.14.0-section`.

### Task 7 — `type="auto"` — untrack the 5 p93 verification artifacts

`git rm --cached` the 5 git-tracked `p93-*.json` under
`quality/reports/verifications/agent-ux/` — per-run verification
artifacts are never committed. Confirm `.gitignore` covers the reports
dir so they stay untracked.
**Row:** `agent-ux/p111-milestone-hygiene` (assert C).

### Task 8 — `type="auto"` — CHANGELOG v0.14.0 PENDING section

Append a substantive `## [v0.14.0]` section (≥10 non-blank lines) to
`CHANGELOG.md` above `## [v0.13.0]`, carrying a **PENDING** release-status
header (the executor STOPs at the owner tag boundary, so the section ships
PENDING — mirrors the v0.13.0 precedent).
**Row:** `agent-ux/p111-changelog-v0.14.0-section`.

### Task 9 — `type="checkpoint:human-verify"` — STOP at the owner tag boundary

After Tasks 1–8 land and the four `agent-ux/p111-*` rows PASS under the
unbiased verifier, STOP. Update `STATE.md` cursor to
"v0.14.0 ready-to-tag; owner pushes tag." Do NOT author/run
`tag-v0.14.0.sh`; do NOT run the 9th probe; do NOT push the tag.

## Success criteria

1. The four `agent-ux/p111-*` catalog rows PASS via an **unbiased**
   verifier subagent (grading the rows that predate the implementation):
   `p111-ci-wait-helper`, `p111-changelog-v0.14.0-section`,
   `p111-retrospective-v0.14.0-section`, `p111-milestone-hygiene`.
2. OP-9 retrospective ratification grades **GREEN** (the v0.14.0
   RETROSPECTIVE section exists with all 5 subheadings + the named
   GTH-09 → v0.15.0 deferral; the ratification subagent grades RED if
   missing, per root CLAUDE.md OP-9).
3. SURPRISES-INTAKE.md retains the P110 **zero-OPEN** drained invariant.
4. Phase close: `git push origin main` lands BEFORE the verifier subagent
   (push-cadence rule); verifier subagent GREEN.
5. **STOP at the tag boundary** — the executor does NOT push the tag; the
   owner runs `tag-v0.14.0.sh` and gates the 9th probe.

## Out of scope (owner territory — NOT P111 deliverables)

- **`tag-v0.14.0.sh`** authorship/execution and the tag push — owner only.
- **The non-skippable 9th probe**
  (`agent-ux/milestone-close-vision-litmus-real-backend` /
  `run.py --cadence pre-release-real-backend`) — **OWNER-gated**: it reads
  **NOT-VERIFIED** honestly without `REPOSIX_ALLOWED_ORIGINS` + a complete
  sanctioned-target credential set, never FAIL-as-skip / skip-as-PASS
  (PROTOCOL.md OD-2). It is NOT a P111 deliverable and is NOT one of the
  four P111 catalog rows above.
