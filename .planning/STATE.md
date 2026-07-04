---
gsd_state_version: 1.0
mode: serial-workstreams
status: executing-p89-serial-workstreams
last_updated: "2026-07-04T18:00:00Z"
last_activity: 2026-07-04 — Quality Convergence window (OD-4 item 2) between P89 close and P90 dispatch. D-CONV-1..8 journaled (quality/SURPRISES.md 5bba77a) + implemented; QL-001..006 BLOCKERs dispositioned (5 fixed, QL-001 filed BLOCKER intake for P90/P91); 5 permanent catalog rows minted; ledger reconstruction at .planning/audits/QUALITY-LEDGER.md. Next: P90 (gated on convergence coordinator GO).
workstreams:
  workstream_a:
    milestone: v0.13.0
    milestone_name: DVCS over REST (extended)
    status: executing  # P89 execution started 2026-07-03 (top-level orchestration mode)
    phases_total: 20  # P78-P97 (P78-P88 shipped + P89-P97 extension)
    phases_completed: 13  # P78-P90
    next_phase: P91  # P90 SHIPPED GREEN 2026-07-04 (verdict at quality/reports/verdicts/p90/VERDICT.md); P91 next — QL-001 ROUTED to P91 per D90-01 (real-git-push-e2e waiver backstop expires 2026-07-31)
    blocks_tag: true  # v0.13.0 tag does NOT push until P97 GREEN; tag push delegated to orchestrator per OD-3
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

### Workstream A — v0.13.0 (extended) — EXECUTING

Phase: P90 SHIPPED 2026-07-04. Verdict GREEN at `quality/reports/verdicts/p90/VERDICT.md` (all 8 SCs PASS; unbiased zero-context verifier re-ran the 9 P90-minted rows independently). Honesty rules landed: `minted_at` write-once anchor, `coverage_kind` hard-new/RAISE-legacy + `transport_claim` tri-state, missing-verifier→NOT-VERIFIED + `error` marker, env-skip fail-closed with `last_real_grade`/`last_real_verified` history, shell-subprocess transcript-at-grade-time, F-K4b per-expected-assert congruence (gated on minted_at per D90-05, p86 F6 fixture REDs), `test-name-vs-asserts.sh` pre-push gate (4-entry baseline), absorption-honesty + milestone-adversarial dispatch templates + verdict.py GREEN-block. RAISE LIST at `quality/reports/raise-list-p90.md` seeds P92/P94/P95. Waiver cliff disposed per-waiver (D90-02): 14 renewed with live `tracked_in`, binstall cleared (already fixed 33dd41f), `agent-ux/real-git-push-e2e` deliberately NOT renewed. **QL-001 ROUTED to P91 per D90-01** — ROADMAP P91 names its 6 sharpened criteria + D90-06 sanctioned-target litmus criterion; the `real-git-push-e2e` waiver (expires 2026-07-31) is the backstop if P91 slips. 5 MISSING_TEST doc-alignment rows closed with real tests (90-06); `git-checkout-branch-command` stays waived (QL-001-blocked). Deviations journaled in `quality/SURPRISES.md` (5 entries 2026-07-04). Tag pushed only after P97 GREEN.
Plan: shipped — 7 plan files + OVERVIEW + DECISIONS at `.planning/phases/90-framework-fixes-honesty-rules/` executed (waves A–E, sequential single-tree-writer); Wave B1 evidence + 4 deviations in `90-PAUSE-HANDOFF.md`.
Status: P90 GREEN — 13/20 phases complete (P78–P90 shipped); next P91. v0.13.0 tag held until P97 GREEN (tag push delegated to orchestrator per OD-3).
Next agent action: plan + execute P91 (real-backend wiring; TokenWorld litmus T2 REOPEN gate; QL-001 fix per the amended ROADMAP P91 entry — 6 criteria + sanctioned-target litmus-body assertion; preflight 3/3 PASS as of 2026-07-03). P91 must land QL-001 before the 2026-07-31 `real-git-push-e2e` waiver expiry.

### Workstream B — v0.13.2 — QUEUED (RESEQUENCED per OD-4)

Phase: P98 (entry-point) — crate skeleton + shared-compute lift + edge model + walker + catalog + tracker schemas. Sourced from `.planning/research/v0.13.2-cross-link-fidelity/`.
Plan: TBD — P98 plan-overview not yet authored.
Status: RESEQUENCED per OD-4 item 3 (2026-07-04, `89-OWNER-DECISIONS.md` § "DECISION OD-4"): a new **launch-readiness milestone** (asciinema hero demo, CI-verified headline numbers, install-path excellence, positioning/Show-HN kit) is scoped and executed AFTER the P97/v0.13.0 tag and BEFORE P98. P98's "Depends on: v0.13.0 milestone GREEN" still holds; it additionally now depends on launch-readiness GREEN. 0/10 phases complete; ROADMAP scaffolded; REQUIREMENTS scaffolded; intakes scaffolded with 2 Q6 deferrals seeded in GOOD-TO-HAVES.
Next agent action: none until P97 GREEN + launch-readiness milestone scoped via `/gsd-new-milestone`; then `/gsd-discuss-phase 98`.

Last activity: 2026-07-03 — resumption after 8-week idle gap: OD-3 full-autonomy mandate ratified, P89 pre-execution consolidation landed (5a6a388), main fast-forward from workstream/v0.13.0-ext pending, P89 execution starting.

## Current Focus

**Active milestones (SERIAL per OD-3 — A then B):**

- **Workstream A — v0.13.0 extended.** EXECUTING. P78–P88 shipped 2026-05-01; extended 2026-05-08 with P89–P97 (real-backend frictions); P89 execution started 2026-07-03. Holds v0.13.0 tag until P97 GREEN. ROADMAP at `.planning/milestones/v0.13.0-phases/ROADMAP.md`.
- **Workstream B — v0.13.2 cross-link-fidelity.** QUEUED behind workstream A per OD-3 (serial). Scoped 2026-05-08; P98–P107. ROADMAP at `.planning/milestones/v0.13.2-phases/ROADMAP.md`. P98 does not start until P97 GREEN.

**Last shipped milestone:** v0.12.1 (closed 2026-04-30). Verdict GREEN at `quality/reports/verdicts/milestone-v0.12.1/VERDICT.md` (commit 9ef348e).

**Cargo serialization rule (CLAUDE.md memory budget):** only ONE cargo invocation at a time. The separate-worktrees caveat is moot under OD-3 serial execution (single working branch: main); doc-only / planning-only subagents can still run truly concurrent with one cargo subagent.

**Pre-kickoff checklist (kickoff-recommendations.md § "Pre-kickoff checklist"):**

1. ✅ Open-questions resolved or explicitly deferred — `.planning/research/v0.13.0-dvcs/decisions.md`.
2. ◐ POC scheduled in `research/v0.13.0-dvcs/poc/` — folded into P79 (POC ships first; attach implementation absorbs findings).
3. ✅ Push cadence decided — per-phase, codified in `CLAUDE.md` § GSD workflow (closes backlog 999.4).
4. ◐ `/gsd-review` scheduled — runs after ROADMAP + first PLAN.md drafted, before execution. ROADMAP drafted 2026-04-30; review can run after owner approval.
5. ✅ 3 WAIVED structure rows scheduled — P78 includes verifiers before waiver expires 2026-05-15.
6. ✅ ROADMAP.md drafted — 11 phases (P78–P88) drafted by gsd-roadmapper 2026-04-30; awaiting owner approval.

**Phase decomposition (P78–P88):**

- **P78** — Pre-DVCS hygiene (gix bump + 3 WAIVED-row verifiers + MULTI-SOURCE-WATCH-01 walker schema migration)
- **P79** — POC + `reposix attach` core (POC ships in `research/v0.13.0-dvcs/poc/` first; then attach subcommand)
- **P80** — Mirror-lag refs (`refs/mirrors/confluence-head` + `refs/mirrors/confluence-synced-at`)
- **P81** — L1 perf migration (replace `list_records` walk with `list_changed_since`-based conflict detection; `reposix sync --reconcile` escape hatch)
- **P82** — Bus remote URL parser + cheap prechecks + fetch-not-advertised dispatch
- **P83** — Bus remote write fan-out (SoT-first, mirror-best-effort, fault injection — riskiest phase, may split)
- **P84** — Webhook-driven mirror sync (GH Action workflow + `--force-with-lease` race protection)
- **P85** — DVCS docs (topology, mirror setup, troubleshooting, cold-reader pass)
- **P86** — Dark-factory regression — third arm (vanilla-clone + attach + bus-push)
- **P87** — Surprises absorption (+2 reservation slot 1, OP-8)
- **P88** — Good-to-haves polish + milestone close (+2 reservation slot 2, OP-9 retrospective ritual)

**Coverage:** 36/36 v0.13.0 REQ-IDs mapped to exactly one phase (no orphans, no duplicates).

**Carry-forward bundle:** `.planning/milestones/v0.13.0-phases/CARRY-FORWARD.md` lists `MULTI-SOURCE-WATCH-01` (P78), `GIX-YANKED-PIN-01` (P78), `WAIVED-STRUCTURE-ROWS-03` (P78), `POC-DVCS-01` (P79).

**Next agent action:** execute P89 waves 1–4 per the 8 plan files (`.planning/phases/89-framework-fixes-cadence-shell-kind/89-0*-PLAN.md`), top-level orchestration mode. v0.13.0 tag is HELD per Path A / Option B (ratified 2026-05-08); tag push at P97 is delegated to the orchestrator per OD-3, contingent on GREEN milestone verdict. The original P78–P88 milestone-close verdict (2026-05-01 GREEN) remains on disk but is superseded by the post-extension verdict P97 will write. The tag-script `.planning/milestones/v0.13.0-phases/tag-v0.13.0.sh` is RENAMED `.disabled` until P97 ratifies; do NOT run it. P89 + P90 are framework fixes (RBF-FW-*) that gate every subsequent extension phase.

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

Frontmatter (above) is the machine-readable cursor. Resume via `/gsd-resume-work`; current cursor is "execute P89 waves 1–4 per the 8 plan files, top-level mode" (workstream B queued behind A per OD-3).
