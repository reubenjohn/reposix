---
phase: 88
plan: 01
subsystem: planning / quality-gates / milestone-close
tags: [good-to-haves-polish, +2-reservation, op-8, op-9, v0.13.0, milestone-close, ready-to-tag]
dependency_graph:
  requires: [P78, P79, P80, P81, P82, P83, P84, P85, P86, P87]
  provides:
    - .planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md (drained; 1 entry DEFERRED to v0.14.0)
    - .planning/milestones/v0.13.0-phases/tag-v0.13.0.sh (8 guards; executable; owner-runnable)
    - CHANGELOG.md `## [v0.13.0]` section (substantive — 30+ non-blank lines, every shipped REQ-ID category named, v0.14.0 carry-forward bundle named)
    - .planning/RETROSPECTIVE.md v0.13.0 milestone section (OP-9 template — What Was Built / What Worked / What Was Inefficient / Patterns Established / Key Lessons + 6 distilled cross-phase lessons + Carry-forward to v0.14.0)
    - quality/gates/agent-ux/p88-good-to-haves-drained.sh (mechanical drain assertion)
    - quality/gates/agent-ux/v0.13.0-changelog-entry-present.sh (mechanical CHANGELOG presence + substantiveness)
    - quality/gates/agent-ux/v0.13.0-tag-script-present.sh (mechanical tag-script structure check)
    - quality/gates/agent-ux/v0.13.0-retrospective-distilled.sh (mechanical OP-9 subheading check)
    - quality/catalogs/agent-ux.json + 4 rows (agent-ux/p88-good-to-haves-drained, agent-ux/v0.13.0-changelog-entry-present, agent-ux/v0.13.0-tag-script-present, agent-ux/v0.13.0-retrospective-distilled — all PASS, kind: mechanical, cadence: on-demand, blast_radius P3)
  affects:
    - CLAUDE.md (v0.13.0 SHIPPED historical-milestone subsection inserted before "## Quick links")
    - .planning/STATE.md (cursor flipped to ready-to-tag; progress 11/11; 100%)
    - .planning/REQUIREMENTS.md (DVCS-SURPRISES-01 + DVCS-GOOD-TO-HAVES-01 checkboxes flipped to [x])
tech_stack:
  added: []
  patterns:
    - "Catalog-first commit with eventual-pass verifiers — 4 P88 milestone-close rows minted at T01 with verifiers that FAIL at commit time; T02-T04 land artifacts that flip the rows to PASS. Same shape as P86 dvcs-third-arm and P87 p87-surprises-absorption."
    - "GOOD-TO-HAVES STATUS regex: `^[*[:space:]]*\\**[[:space:]]*STATUS[[:space:]]*\\**:?\\**[[:space:]]+(RESOLVED|DEFERRED|WONTFIX)` — single regex covers both bare and bold-wrapped variants without aliasing-double-count. Caught at first verifier run."
    - "8-guard tag-script (exceeds the >=6 floor) — clean tree, on main, version match, CHANGELOG entry, full test suite GREEN, pre-push runner GREEN, P88 verdict GREEN, milestone-close verdict GREEN. The verdict guards are unique to the milestone-close tag-script (v0.12.0 used 6 guards, no verdict-gate guards)."
    - "OP-9 milestone-close ritual: distill BEFORE archive. Raw intake files (SURPRISES-INTAKE, GOOD-TO-HAVES) travel with the milestone archive into `*-phases/`; distilled lessons live permanently in `.planning/RETROSPECTIVE.md` so future planners can skim cross-milestone."
    - "Pure-docs-envelope phase boundary: P88 charter is docs+catalog+shell only, no Rust. The boundary is enforced at planning time (Hard constraints in the prompt) AND at execution time (every commit reviewed against the boundary). GOOD-TO-HAVES-01's S-sized Rust+test+schema work crossed the boundary and DEFERRED."
key_files:
  created:
    - .planning/phases/88-good-to-haves-polish/88-01-PLAN.md
    - .planning/phases/88-good-to-haves-polish/88-01-SUMMARY.md
    - .planning/milestones/v0.13.0-phases/tag-v0.13.0.sh
    - quality/gates/agent-ux/p88-good-to-haves-drained.sh
    - quality/gates/agent-ux/v0.13.0-changelog-entry-present.sh
    - quality/gates/agent-ux/v0.13.0-tag-script-present.sh
    - quality/gates/agent-ux/v0.13.0-retrospective-distilled.sh
  modified:
    - .planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md (entry-01 STATUS flipped to DEFERRED to v0.14.0 with rationale)
    - quality/catalogs/agent-ux.json (+4 rows)
    - CHANGELOG.md (`## [v0.13.0] -- DVCS over REST -- 2026-05-01` section appended above v0.12.0)
    - .planning/RETROSPECTIVE.md (v0.13.0 section replaced with OP-9 template + 6 distilled lessons + v0.14.0 carry-forward)
    - CLAUDE.md (v0.13.0 SHIPPED subsection inserted before "## Quick links")
    - .planning/STATE.md (cursor flipped to ready-to-tag; progress 11/11)
    - .planning/REQUIREMENTS.md (DVCS-SURPRISES-01 + DVCS-GOOD-TO-HAVES-01 checkboxes)
decisions:
  - id: PCT-04
    summary: "DEFERRED over RESOLVED on GOOD-TO-HAVES-01 (extend `reposix-quality bind` to all dimensions). Per OP-8 sizing: S items close in P88 if budget permits, else default-defer. P88's hard-constraint envelope is docs+catalog+shell only (NO Rust). The bind extension requires (a) ~30-50 lines Rust spanning bind verb dispatch + per-dimension validation, (b) cross-dimension schema design (each catalog has its own row shape), (c) new test fixture in crates/reposix-quality/tests/. The work is well-scoped at S size but doesn't fit P88's pure-docs envelope; doing it here would double the phase's scope per the OP-8 'scope-creep-to-fit-the-finding' anti-pattern. v0.14.0 carries the gap forward; provenance flag on hand-edited rows continues to mark Principle A bypass until the verb extension lands."
  - id: PCT-05
    summary: "8 guards on tag-v0.13.0.sh (exceeds the >=6 floor in ROADMAP P88 SC3). Mirrors v0.12.0 base guards (1-6: clean tree, on main, version match, CHANGELOG entry, CI green, P63/P88 verdict GREEN) PLUS adds a 5th cargo-test-workspace guard (sequential per CLAUDE.md memory budget rule) AND a 8th milestone-close-verdict guard. The milestone-close verdict gate is the new cross-phase coherence check (ROADMAP P88 SC5). The cargo-test guard is the safety net for the case where pre-push runner skipped a slow check; tag-cut is the right place to re-confirm full-workspace test green."
  - id: PCT-06
    summary: "RETROSPECTIVE.md v0.13.0 section REPLACES the P87 preview rather than APPENDING. The P87 surprises-only preview (added 2026-05-01 by 8e0bb9b) was scaffolding for the OP-9 ritual; P88 is the official milestone-close distillation per OP-9. Replacement preserves the carry-forward block (now refactored as 'Carry-forward to v0.14.0' subheading) while adding the 5 OP-9 subheadings + 6 distilled lessons + Cost Observations. The verifier asserts 5 OP-9 subheadings as actual markdown headings (not inline mentions), which the P87 preview's narrative paragraph did NOT satisfy — rebuild was the right move."
  - id: PCT-07
    summary: "DVCS-SURPRISES-01 + DVCS-GOOD-TO-HAVES-01 BOTH flipped to [x] in this phase's commit. P87 close (commit 3fb7fd9) shipped GREEN verdict but did not flip the SURPRISES-01 checkbox in REQUIREMENTS.md — the milestone-close phase is the natural single-commit close for both +2 reservation REQ-IDs. Audit trail intact: P87's terminal STATUS in SURPRISES-INTAKE.md + P87's verdict file are the SURPRISES-01 evidence; the checkbox is the milestone-close marker."
metrics:
  duration: ~55 minutes (catalog-first + 4 task-commits + close)
  completed: 2026-05-01T22:50:00Z
  tasks: 5
  files_created: 7
  files_modified: 7
  commits: 6 (e32c20a + 1ecb16b + 8bab313 + dc6e5ab + close + push commit)
---

# Phase 88, Plan 01 — v0.13.0 Good-to-haves polish + milestone close (+2 reservation slot 2) Summary

## One-liner

Drained GOOD-TO-HAVES.md (1 entry, DEFERRED to v0.14.0); finalized milestone-close artifacts (CHANGELOG `[v0.13.0]` entry, tag-v0.13.0.sh with 8 guards, RETROSPECTIVE.md OP-9 distillation with 6 cross-phase lessons); STATE.md flipped to ready-to-tag; orchestrator does NOT push tag (owner runs tag-v0.13.0.sh).

## Tasks shipped

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Catalog-first: 4 milestone-close rows + verifiers | e32c20a | 4 verifiers + agent-ux.json + 88-01-PLAN.md |
| 2 | Drain GOOD-TO-HAVES + tag-v0.13.0.sh | 1ecb16b | GOOD-TO-HAVES.md + tag-v0.13.0.sh + verifier fix |
| 3 | CHANGELOG `[v0.13.0]` entry | 8bab313 | CHANGELOG.md |
| 4 | RETROSPECTIVE.md OP-9 distillation | dc6e5ab | .planning/RETROSPECTIVE.md |
| 5 | CLAUDE.md + STATE.md + REQUIREMENTS.md + SUMMARY + push | (this commit) | CLAUDE.md + STATE.md + REQUIREMENTS.md + 88-01-SUMMARY.md |

## Verifier roll-up

All 4 P88 catalog rows PASS at phase close:
- `agent-ux/p88-good-to-haves-drained` — 1 entry, 1 terminal STATUS, 0 TBD.
- `agent-ux/v0.13.0-changelog-entry-present` — 30 non-blank lines in `## [v0.13.0]` section.
- `agent-ux/v0.13.0-tag-script-present` — 8 guards, executable, signed-tag invocation.
- `agent-ux/v0.13.0-retrospective-distilled` — 5 OP-9 subheadings as markdown headings.

## Deviations from plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] p88-good-to-haves-drained.sh regex double-counted bold-wrapped STATUS lines**
- **Found during:** Task 2 (post-flip, first verifier run after drain).
- **Issue:** Two parallel awk regex branches matched the same `**STATUS:** DEFERRED ...` line, returning `TERMINAL_COUNT=2` for `ENTRY_COUNT=1`. Output was a misleading PASS but the count was wrong.
- **Fix:** Single regex `^[*[:space:]]*\**[[:space:]]*STATUS[[:space:]]*\**:?\**[[:space:]]+(RESOLVED|DEFERRED|WONTFIX)` covers both bare (`STATUS: ...`) and bold-wrapped (`**STATUS:** ...`) variants without aliasing.
- **Files modified:** `quality/gates/agent-ux/p88-good-to-haves-drained.sh`.
- **Commit:** 1ecb16b.

**2. [Rule 1 - Bug] v0.13.0-retrospective-distilled.sh false-PASS on inline subheading mentions**
- **Found during:** Task 1 (verifier dry-run after T01 commit-time intentional FAILs).
- **Issue:** Initial regex used `grep -qF` which matched the OP-9 subheadings as inline mentions in the P87 preview paragraph (which literally enumerates "What Was Built / What Worked / What Was Inefficient / Patterns Established / Key Lessons" as a sentence). Verifier returned PASS even though the v0.13.0 section had ZERO actual markdown headings.
- **Fix:** Changed regex to `^#{2,4}[[:space:]]+${HEADING}\$` to match only actual markdown headings (`### What Was Built` form per OP-9 template).
- **Files modified:** `quality/gates/agent-ux/v0.13.0-retrospective-distilled.sh`.
- **Commit:** e32c20a (T01 catalog-first commit, before any artifact lands).

### Decisions made (no auto-fix needed)

- **DVCS-SURPRISES-01 flipped to [x] alongside DVCS-GOOD-TO-HAVES-01.** P87 close commit (3fb7fd9) didn't flip the SURPRISES-01 checkbox in REQUIREMENTS.md; P88 milestone-close commit closes both. P87 verdict GREEN + terminal STATUS in SURPRISES-INTAKE.md are the SURPRISES-01 evidence; the checkbox is the milestone marker.

### No Rust code changes

Per the hard-constraint envelope (P88 charter), no `crates/` files modified. The GOOD-TO-HAVES-01 entry that would have required Rust was DEFERRED to v0.14.0 explicitly to honor this boundary.

## Self-Check: PASSED

**Files created (verified existence):**
- FOUND: `.planning/phases/88-good-to-haves-polish/88-01-PLAN.md`
- FOUND: `.planning/phases/88-good-to-haves-polish/88-01-SUMMARY.md`
- FOUND: `.planning/milestones/v0.13.0-phases/tag-v0.13.0.sh`
- FOUND: `quality/gates/agent-ux/p88-good-to-haves-drained.sh`
- FOUND: `quality/gates/agent-ux/v0.13.0-changelog-entry-present.sh`
- FOUND: `quality/gates/agent-ux/v0.13.0-tag-script-present.sh`
- FOUND: `quality/gates/agent-ux/v0.13.0-retrospective-distilled.sh`

**Commits verified in `git log`:**
- FOUND: e32c20a (T01 catalog-first)
- FOUND: 1ecb16b (T02 drain + tag-script)
- FOUND: 8bab313 (T03 CHANGELOG)
- FOUND: dc6e5ab (T04 RETROSPECTIVE)
- (T05 close commit follows this SUMMARY write)

**All 4 P88 catalog row verifiers exit 0.** All 4 P88 milestone-close artifacts present + substantive.

## Next agent action

Orchestrator dispatches BOTH:

1. **P88 verifier subagent** — grades the 4 P88 milestone-close catalog rows from artifacts (zero session context). Verdict at `quality/reports/verdicts/p88/VERDICT.md`. Asserts: GOOD-TO-HAVES drained, CHANGELOG substantive, tag-script structure, RETROSPECTIVE OP-9 subheadings.
2. **Milestone-close verifier subagent** — grades P78–P88 cross-phase coherence per ROADMAP P88 SC5. Verdict at `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md`. Asserts: all P78–P88 catalog rows GREEN-or-WAIVED, dark-factory three-arm transcript GREEN against sim, no expired waivers without follow-up, RETROSPECTIVE.md v0.13.0 section exists, +2 reservation operational (intakes drained, honesty check signed, GOOD-TO-HAVES sized correctly).

After BOTH verdicts GREEN: owner runs `bash .planning/milestones/v0.13.0-phases/tag-v0.13.0.sh` (8 guards) then `git push origin v0.13.0`. Orchestrator does NOT push the tag (ROADMAP P88 SC6 — STOP at tag boundary).
