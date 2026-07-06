---
gsd_state_version: 1.0
mode: serial-workstreams
status: v0.13.0-closed-green-awaiting-owner-pre-tag
last_updated: "2026-07-05"
last_activity: "2026-07-05 — v0.13.0 milestone CLOSED GREEN (with owner-gated caveats). Milestone verdict quality/reports/verdicts/milestone-v0.13.0/VERDICT.md (verdict commit 390ce31, graded HEAD 3c6d72f) — 117 PASS / 0 FAIL / 15 WAIVED / 13 ratified-honest NOT-VERIFIED; OP-9 RETROSPECTIVE distilled (ratifier exit 0); 9th probe honestly NOT-VERIFIED (real-backend env-gate); release-plz git-state blocker fixed (3c6d72f). P97 (milestone-close phase) closed via this milestone verdict; workstream A / v0.13.0-extension COMPLETE (20/20 phases P78–P97). NEXT: owner pre-tag actions + L0 v0.13.0 tag push, then workstream B (v0.13.2, P98+). Durable OWNER PRE-TAG ACTIONS + queued post-tag /gsd-quick meta-infra items live in § Workstream A."
workstreams:
  workstream_a:
    milestone: v0.13.0
    milestone_name: DVCS over REST (extended)
    status: closed-green  # v0.13.0-extension CLOSED GREEN with owner-gated caveats 2026-07-05 (milestone verdict quality/reports/verdicts/milestone-v0.13.0/VERDICT.md, verdict commit 390ce31, graded HEAD 3c6d72f)
    phases_total: 20  # P78-P97 (P78-P88 shipped + P89-P97 extension)
    phases_completed: 20  # P78-P97 (workstream A / v0.13.0-extension COMPLETE)
    next_phase: P98  # v0.13.0 milestone CLOSED GREEN; awaiting owner pre-tag actions + L0 tag push; then workstream B (v0.13.2, P98+)
    blocks_tag: true  # P97 GREEN so the tag is phase-ready, BUT the v0.13.0 tag push is L0/owner's (NOT the coordinator's) and gated on the OWNER PRE-TAG ACTIONS in § Workstream A
  workstream_b:
    milestone: v0.13.2
    milestone_name: Cross-link fidelity
    status: queued  # RESEQUENCED per OD-4 item 3 (2026-07-04): queued BEHIND the new launch-readiness milestone, which itself follows P97/v0.13.0 tag
    phases_total: 10  # P98-P107
    phases_completed: 0
    next_phase: P98
    blocks_tag: false  # v0.13.2 tag ships at P107, sequenced after v0.13.0 tag AND the launch-readiness milestone per OD-4; tag push delegated to orchestrator
---

# Project State

## Current Position

**Mode:** serial-workstreams per OD-3 (workstream A → then B; the parallel-worktree model is RETIRED).

**OD-3 mandate (2026-07-03)** — see `.planning/phases/89-framework-fixes-cadence-shell-kind/89-OWNER-DECISIONS.md` § "DECISION OD-3": drive to v1.0. Complete v0.13.0-ext (P89–P97, tag v0.13.0), then v0.13.2 (P98–P107, tag v0.13.2) STRICTLY SERIALLY; after both tags, formalize the research-only ladder (v0.14.0 observability/multi-repo → plugin ecosystem/launch readiness → v1.0.0 + ADR-009 semver activation) as real GSD milestones via `/gsd-new-milestone`. `main` is the working branch (workstream/v0.13.0-ext fast-forwarded into main 2026-07-03 and retired; per-phase push cadence targets origin/main). Full autonomy incl. former hard gates: OD-1's owner sign-off delegated to orchestrator (owner notified, not blocking), tag pushes at P97/P107 delegated contingent on GREEN verdicts, ~$50 pre-authorized for P106 L3 dogfood. OD-2 + litmus REOPEN gates remain in force UNCHANGED — on RED the orchestrator loops back, never waives.

### Workstream A — v0.13.0 (extended) — CLOSED GREEN

Phase: **v0.13.0-extension (P78–P97) CLOSED GREEN with owner-gated caveats, 2026-07-05.** Milestone verdict `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md` (verdict commit `390ce31`, graded HEAD `3c6d72f`, on origin/main). Disk tally: **117 PASS / 0 FAIL / 15 WAIVED / 13 ratified-honest NOT-VERIFIED**. P97 (the OP-8 Slot 2 + milestone-close phase) is **closed via this milestone verdict** — there is no separate `p97/VERDICT.md`; the milestone-close verdict IS P97's close. OP-9 RETROSPECTIVE distilled (ratifier exit 0); the non-skippable **9th probe** graded honestly **NOT-VERIFIED** (real-backend env-gate unset — never skip-as-pass); the release-plz git-state blocker (5 tracked+ignored P93 evidence JSONs) was fixed at `3c6d72f`.
Status: **20/20 phases complete (P78–P97)** — workstream A / v0.13.0-extension COMPLETE. The v0.13.0 tag is phase-ready; the **tag push itself is L0/owner's, NOT the coordinator's** (`.planning/milestones/v0.13.0-phases/tag-v0.13.0.sh.disabled` stays disabled — do NOT run it) and is gated on the owner pre-tag actions below.

**OWNER PRE-TAG ACTIONS (durable — the next session/owner must clear these before the v0.13.0 tag push):**
1. Run/ratify the real-backend **9th probe** — `python3 quality/runners/run.py --cadence pre-release-real-backend` with ATLASSIAN creds + `REPOSIX_ALLOWED_ORIGINS` set (currently honestly NOT-VERIFIED, env-gated).
2. Fresh **`dvcs-cold-reader`** rubric review (TTL expired; top-level tooling surface) via `/reposix-quality-review --rubric dvcs-cold-reader`.
3. Run/accept the **6 env-gated real-backend** transport / attach / conflict catalog rows (part of the 13 ratified-honest NOT-VERIFIEDs).
4. **Disposition-or-carry-forward the 19 bare-OPEN SURPRISES entries** into the next milestone's intake (zero-loss split already landed at P96, but the per-entry terminal disposition is owed — carry-forward debt, no un-addressed HIGH/BLOCKER).
5. **Renew the `structure/file-size-limits` waiver before 2026-08-08** (active-corpus WAIVED).
6. **PR #61 (release-plz) is stale vs HEAD** — regenerate from `390ce31` → review → merge (owner-gated crates.io publish).
7. **The v0.13.0 tag push is L0/owner's, NOT the coordinator's** — hand back to L0 for it.

**Queued post-tag `/gsd-quick` meta-infra items (run AFTER the tag lands, each via GSD):**
- The doctrine **4-edit** `/gsd-quick` — NOTE its "create DP-5" step is **stale** (DP-5 already exists = tangent-classification); adjust the edit, do NOT apply verbatim.
- The `dispatch-doctrine.sh` **session-guard** addition.
- The **verifications-layout** evidence/transient subtree refactor.
- The `cli.md` **exit-code v0.14 doc fix**.

Next agent action: **none in workstream A** — it is COMPLETE. Clear the OWNER PRE-TAG ACTIONS above (owner/L0), push the v0.13.0 tag (L0), then proceed to the launch-readiness milestone scoping (`/gsd-new-milestone`) and workstream B (v0.13.2, P98+) per OD-3/OD-4.

### Workstream B — v0.13.2 — QUEUED (RESEQUENCED per OD-4)

Phase: P98 (entry-point) — crate skeleton + shared-compute lift + edge model + walker + catalog + tracker schemas. Sourced from `.planning/research/v0.13.2-cross-link-fidelity/`.
Plan: TBD — P98 plan-overview not yet authored.
Status: RESEQUENCED per OD-4 item 3 (2026-07-04, `89-OWNER-DECISIONS.md` § "DECISION OD-4"): a new **launch-readiness milestone** (asciinema hero demo, CI-verified headline numbers, install-path excellence, positioning/Show-HN kit) is scoped and executed AFTER the P97/v0.13.0 tag and BEFORE P98. P98's "Depends on: v0.13.0 milestone GREEN" still holds; it additionally now depends on launch-readiness GREEN. 0/10 phases complete; ROADMAP scaffolded; REQUIREMENTS scaffolded; intakes scaffolded with 2 Q6 deferrals seeded in GOOD-TO-HAVES.
Next agent action: **P97 is now GREEN (v0.13.0 milestone CLOSED).** Still gated behind the OWNER PRE-TAG ACTIONS + L0 v0.13.0 tag push (see § Workstream A), then the launch-readiness milestone scoped via `/gsd-new-milestone`; then `/gsd-discuss-phase 98`.

Last activity: 2026-07-05 — v0.13.0 milestone CLOSED GREEN with owner-gated caveats (milestone verdict `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md`, verdict commit `390ce31`, graded HEAD `3c6d72f`); STATE advanced to 20/20 phases, workstream A COMPLETE, owner pre-tag actions + post-tag `/gsd-quick` queue recorded durably in § Workstream A.

## Current Focus

**Active milestones (SERIAL per OD-3 — A then B):**

- **Workstream A — v0.13.0 extended.** **CLOSED GREEN 2026-07-05 (P78–P97, 20/20 phases).** Shipped P78–P88 2026-05-01; extended 2026-05-08 with P89–P97 (real-backend frictions); milestone-close verdict at `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md`. The v0.13.0 tag is phase-ready; its push is **L0/owner's**, gated on the OWNER PRE-TAG ACTIONS in § Workstream A. ROADMAP at `.planning/milestones/v0.13.0-phases/ROADMAP.md`.
- **Workstream B — v0.13.2 cross-link-fidelity.** QUEUED behind workstream A per OD-3 (serial). Scoped 2026-05-08; P98–P107. ROADMAP at `.planning/milestones/v0.13.2-phases/ROADMAP.md`. **P97 is now GREEN, but P98 still does not start** until the OWNER PRE-TAG ACTIONS clear, the L0 v0.13.0 tag lands, and the launch-readiness milestone is scoped (OD-4).

**Last shipped milestone:** v0.12.1 (closed 2026-04-30). Verdict GREEN at `quality/reports/verdicts/milestone-v0.12.1/VERDICT.md` (commit 9ef348e).

**Cargo serialization rule (CLAUDE.md memory budget):** only ONE cargo invocation at a time. The separate-worktrees caveat is moot under OD-3 serial execution (single working branch: main); doc-only / planning-only subagents can still run truly concurrent with one cargo subagent.

**Phase decomposition + pre-kickoff scaffolding (superseded — see live cursor above):** the milestone-start scaffolding (pre-kickoff checklist, the original P78–P88 decomposition, the 36/36 REQ-ID coverage map, the CARRY-FORWARD bundle, and the long-dead "execute P89 waves 1–4" cursor) is historical. The live cursor is the frontmatter + § "Workstream A — v0.13.0 (extended)" above (**P96 CLOSED GREEN 2026-07-05, next P97**). Archived detail: `.planning/milestones/v0.13.0-phases/ROADMAP.md` (full P78–P97 decomposition) + `.../CARRY-FORWARD.md`. **Still live:** P97 is now GREEN (milestone CLOSED), so the tag is phase-ready — but the tag-script `.planning/milestones/v0.13.0-phases/tag-v0.13.0.sh.disabled` stays disabled (**do NOT run it**); the **v0.13.0 tag push is L0/owner's, NOT the coordinator's**, and is gated on the OWNER PRE-TAG ACTIONS in § Workstream A.

## Per-milestone history (cross-references)

Historical phase-by-phase contribution narrative lives in per-milestone ARCHIVE files:

- `.planning/milestones/v0.12.1-phases/ARCHIVE.md` — most recently shipped (P72–P77 + owner-TTY close-out).
- `.planning/milestones/v0.12.0-phases/archive/` — Quality Gates framework + 8 dimensions (P56–P65).
- `.planning/milestones/v0.11.0-phases/`, `v0.10.0-phases/`, `v0.9.0-phases/`, `v0.8.0-phases/`, etc. — earlier milestones.

## Project Reference

- `.planning/PROJECT.md` — scope and decisions table (Current Milestone now v0.13.0).
- `.planning/ROADMAP.md` — milestone-level roadmap (P78–P88 drafted 2026-04-30 by gsd-roadmapper).
- `.planning/REQUIREMENTS.md` — milestone requirements (36 v0.13.0 REQ-IDs; traceability table mapped to phases).
- `.planning/research/v0.13.0-dvcs/` — full research bundle (vision, architecture, kickoff, decisions, CARRY-FORWARD).
- `CLAUDE.md` — operating principles + per-phase push cadence + Quality Gates protocol.
- `quality/PROTOCOL.md` — Quality Gates runtime contract.
- `quality/SURPRISES.md` — append-only pivot journal.

## Blockers / Concerns

- POC-FINDINGS.md re-engagement checkpoint pending (orchestrator decision; F01 + F04 are REVISE-tagged but small).
- Tag scripts `tag-v0.9.0.sh` / `tag-v0.10.0.sh` RELOCATED to `.planning/milestones/v0.9.0-phases/` + `.planning/milestones/v0.10.0-phases/` (no longer at `scripts/`); the v0.9.0 + v0.10.0 tags themselves remain unpushed — absent both locally and on origin (verified 2026-07-03). Owner gate, pre-existing.
- 3 WAIVED structure rows expired 2026-05-15 — RESOLVED in P78.
- ROADMAP.md top-level v0.12.0 entries — RESOLVED (top-level `.planning/ROADMAP.md` is now ~156 lines; v0.12.0 entries relocated per CLAUDE.md §0.5).

### Quick Tasks Completed

| # | Description | Date | Commit | Directory |
|---|-------------|------|--------|-----------|
| 260501-mgn | Polish 5 cold-reader nits in DVCS docs | 2026-05-01 | 2b9e9c9 | [260501-mgn-polish-5-cold-reader-nits-in-dvcs-docs-b](./quick/260501-mgn-polish-5-cold-reader-nits-in-dvcs-docs-b/) |

## Session Continuity

Frontmatter (above) is the machine-readable cursor. Resume via `/gsd-resume-work`; current cursor is "**v0.13.0 milestone CLOSED GREEN (P78–P97, 20/20) — awaiting OWNER PRE-TAG ACTIONS + L0 v0.13.0 tag push, then workstream B (v0.13.2, P98+)**" (milestone verdict commit `390ce31`, graded HEAD `3c6d72f`). The durable owner pre-tag checklist + queued post-tag `/gsd-quick` meta-infra items live in § Workstream A. Workstream B stays queued behind A per OD-3/OD-4.

Top-level session handover: `.planning/SESSION-HANDOVER.md` (whole-session rotation handover for session 7e2a4cf2, 2026-07-04/05; distinct from per-phase relief handovers under `.planning/phases/`).
