# STATE.md issues

← [back to index](./index.md)

The file is 245 lines; the frontmatter and the body have drifted apart over multiple milestones. Issues by line range, priority-ordered.

### S1 — Performance Metrics block is v0.6/v0.7-era ghost data (lines 79–125) [P1]

**Current text (lines 79–125):**
```
**Velocity:**
- Total plans completed: 10
- Average duration: —
- Total execution time: 0.0 hours (of ~7h total budget, ~4.5h budgeted for MVD)

**By Phase:**
| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| — | — | — | — |
| 22 | 3 | - | - |
...
| Phase 27 P01 | 5 | 2 tasks | 3 files |
```

The "Velocity" block lists 10 plans against a 7h MVD budget — that's the v0.1.0 MVD context. The "By Phase" rows list phases 22 / 25 / 24 / 29 — last touched 2026-04-16 (v0.8.0). The denormalized `Phase 11 PD` etc. tail goes back to v0.3. Phases 31–36 (v0.9.0) and 40–45 (v0.10.0) are not represented.

**Recommendation:** replace the entire block (lines 77–125 inclusive) with a one-paragraph velocity note pointing at `MILESTONES.md` for cross-milestone history, or reset the table to v0.10.0+v0.11.0-in-flight. The "Total execution time: 0.0 hours / ~7h budget" line is actively misleading and must go.

### S2 — "Current Position" block is wrong on every field (lines 64–75) [P0]

**Current text:**
```
Phase: — (v0.10.0 not yet planned)
Plan: —
Cursor: **Run /gsd-autonomous to drive v0.10.0 phases 40–45.**
Status: Planning (v0.10.0 scaffolded — DOCS-01..11 + Phases 40-45 mapped)
Last activity: 2026-04-24 -- v0.10.0 scaffolded (REQUIREMENTS.md + ROADMAP.md + PROJECT.md updates)

Historical note — Phase 15 close-out: ...

Progress: [##########] v0.8.0 complete (Phase 29 of 29 closed)
```

Every line is stale: v0.10.0 shipped 2026-04-25; "v0.8.0 complete" is two milestones behind frontmatter (`milestone: v0.11.0`). "Run /gsd-autonomous to drive v0.10.0 phases 40–45" is a closed cursor.

**Recommended replacement:**
```
Phase: — (v0.11.0 phases not yet planned)
Plan: —
Cursor: **/gsd-plan-milestone-gaps or /gsd-discuss-phase to scope first v0.11.0 phase.**
Status: Planning (v0.11.0 — Performance & Sales Assets; planning_started 2026-04-25)
Last activity: 2026-04-25 -- v0.10.0 SHIPPED (audit tech_debt → resolved); helper backend dispatch tech debt closed cd1b0b6; Record rename + doctor + gc + time-travel landed overnight.

Progress: [##########] v0.10.0 complete (Phases 40–45 closed; phase dirs archived to .planning/milestones/v0.10.0-phases/)
```

### S3 — Phase 14 / Phase 15 close-out narrative (lines 36–55) [P2]

These are useful history but live above the v0.9.0 / v0.10.0 entries. They're correct prose; they just create the impression that the project lives in v0.4.1 / v0.5.0. Roadmap Evolution is meant to be top-newest-first, which it currently is for the top three bullets but degenerates after that.

**Recommendation:** wrap lines 36–55 in `<details><summary>v0.4.1–v0.5.0 (historical)</summary>...</details>`. Same treatment for Phases 16–17 close-out lines (lines 42–54). Move all entries older than 2026-04-22 into a single collapsed `<details>` block; keep only v0.9.0 + v0.10.0 + v0.11.0 prose visible.

### S4 — Decisions section: phase-tagged decisions for closed phases (lines 149–195) [P2]

Lines 149–195 contain `[Phase 11]` … `[Phase 27]` decision entries — all from shipped milestones. These have value as historical record but inflate the file.

**Recommendation:** archive lines 149–195 into a one-line "Historical decisions: see `.planning/milestones/<v>/...`-phases/<phase>/CONTEXT.md per phase" pointer. Or, more concretely, move the entire decisions block to `.planning/RETROSPECTIVE.md` (which already plays this role for cross-milestone trends).

### S5 — Pending Todos / Blockers / Concerns sections are v0.1.0 MVD ghosts (lines 197–212) [P0]

**Current text (lines 197–212):**
```
### Pending Todos
None yet. (Capture via `/gsd-add-todo` during execution.)

### Blockers/Concerns

- **T+3h decision gate (03:30 PDT)** — the orchestrator MUST decide STRETCH
  vs read-only at this point. ...
- **FUSE-in-CI is known-yak-shavy** — lives in Phase S for a reason. ...
- **Demo recording must fire guardrails on camera (SG-08)** — Phase 4 is
  not complete if the recording is happy-path only.
```

All three Blockers/Concerns are from the original 2026-04-13 MVD overnight session. Phase S, FUSE-in-CI, Phase 4 demo recording — all from milestones that shipped over a week ago. The "T+3h gate" is an artifact of a 7-hour autonomous-build window that closed 2026-04-13 ~07:30 PDT. **Most dangerous stale entry** because a future agent might try to honor it.

**Recommended replacement (full block 197–212):**
```
### Pending Todos
None yet. (Capture via `/gsd-add-todo` during execution.)

### Blockers/Concerns

- **`scripts/tag-v0.10.0.sh` exists but tag is unpushed** — owner gate. (See PROJECT.md "Carry-forward from v0.10.0 (tech debt)".)
- **Playwright screenshots deferred** — cairo system libs unavailable on dev host; `scripts/take-screenshots.sh` stub names contract.
- **9 major + 17 minor doc-clarity findings** — `.planning/notes/v0.11.0-doc-polish-backlog.md`; promote into a v0.11.0 phase.
```

### S6 — Session Continuity block is stuck on Phase 29 (lines 214–222) [P1]

**Current text (lines 214–222):**
```
Last session: 2026-04-16T00:00:00.000Z
Checkpoint: Phase 29 complete — milestone v0.8.0 complete, all phases done, UAT 9/9 passed
Resume file: None
Cursor next: **v0.8.0 tag push — user gate. Run `bash scripts/tag-v0.8.0.sh`.** ...

Wave-level commit trail on `main` (Phase 29):
`10d24ba` (29-01: ADF write encoder + issuetype cache), ...
```

This is two milestones stale. v0.8.0 tagged. v0.9.0 shipped. v0.10.0 shipped.

**Recommended replacement:**
```
Last session: 2026-04-25T07:30:00.000Z
Checkpoint: v0.10.0 SHIPPED, audit verdict tech_debt→passed (helper dispatch closed); v0.11.0 planning starts.
Resume file: None
Cursor next: **First v0.11.0 phase to scope is the latency benchmark — see `.planning/research/v0.11.0/latency-benchmark-plan.md`.** Owner gates: tag-v0.10.0 + tag-v0.9.0 not yet pushed.

Recent commit trail on `main`: `cd1b0b6` (helper backend dispatch — closes Phase 32 tech debt) · `856b7b9..132c662` (time-travel via git tags + ADR-007) · `b276473..b862c71` (reposix doctor) · `37ae438..d3647ef` (Cache::gc + reposix gc + reposix tokens) · `2dd06a1..4ad8e2a` (Record rename completion) · `9151b86..6131921` (launch screencast script + quickstart fix).
```

### S7 — "Previous session (Phase 14)" + "Earlier session (Phase 13)" blocks (lines 224–245) [P1]

These two blocks document Phase 13/14 wave commits — irrelevant past v0.5.0. They're a pure copy-forward artifact.

**Recommendation:** delete lines 224–245 entirely. Phase 13/14 history is preserved in `.planning/milestones/v0.4.0-phases/` and `v0.5.0-phases/`.

### S8 — Frontmatter `progress` counter is zeroed (lines 8–13) [P1]

**Current text:**
```
progress:
  total_phases: 0
  completed_phases: 0
  total_plans: 0
  completed_plans: 0
  percent: 0
```

Zero-state numbers. v0.10.0 had 6 phases / ~14 plans (per `.planning/v0.10.0-MILESTONE-AUDIT.md` §1). At least the historical totals should not show as `0`.

**Recommendation:** decide convention — does `progress` track only the active milestone? If so, v0.11.0 phases are not yet scoped, so `total_phases: 0` is technically correct, but `completed_plans: 0` is fine. **Leave as-is** if the convention is per-milestone-from-zero. Otherwise: hydrate with cumulative counts from MILESTONES.md.
