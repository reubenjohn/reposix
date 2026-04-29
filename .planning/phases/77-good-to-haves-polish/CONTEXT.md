# Phase 77: Good-to-haves polish — the +2 reservation, slot 2 (Context)

**Gathered:** 2026-04-29 (autonomous-run prep)
**Status:** Ready for execution (intake populated during P72-P76)
**Mode:** ROI-aware time-box. `--auto` if intake is short.
**Milestone:** v0.12.1
**Estimated effort:** Whatever fits before 5pm. Floor: 30 min (XS items only).

<domain>
## Phase Boundary

Drain `.planning/milestones/v0.12.1-phases/GOOD-TO-HAVES.md` for items that will fit in the time remaining at 5pm. Distinct from P76 (surprises) — good-to-haves are improvements that make the next maintainer's life easier (better error messages, clearer doc cross-refs, redundant test consolidation, helper extractions, naming polish). Not fixing something broken; making something better.

**Explicitly NOT in scope:**
- Surprises (P76 absorbs those).
- New features.
- Refactors that touch > 5 files (file as v0.13.0 instead).

</domain>

<decisions>
## Implementation Decisions

### D-01: Size labels and the 5pm budget
Each intake entry carries a size label (XS, S, M):
- **XS** — 5-15 min: typo fix, error message clarification, single-file cross-ref, comment-only update.
- **S** — 15-60 min: helper extraction, test consolidation, single-file refactor, doc cross-ref sweep on one page.
- **M** — 1-3 hours: multi-file refactor, naming sweep, doc reorganization.

P77 closes ALL XS items. Closes S items if they fit. Defers M items to v0.13.0.

### D-02: Atomic commit per item
Commit message: `polish(p77): <one-line-summary> (was: discovered-by P<N>)`. Body quotes the GOOD-TO-HAVES.md entry verbatim.

### D-03: Time-box check at half-budget
At 4:00 PT, check remaining items against time-to-5pm. If remaining work > remaining time, STOP picking up new items and start drafting the verdict + CLAUDE.md update + final commit.

### D-04: M-items deferred to v0.13.0 backlog
Same pattern as P76's DEFERRED rule: file under `.planning/milestones/v0.13.0-phases/REQUIREMENTS.md` (create if needed) with ID `POLISH-DEFERRED-<short-name>`.

### D-05: Empty intake handling
If GOOD-TO-HAVES.md is empty at P77 start, ship the phase as: "Intake was empty. No good-to-haves observed. P77 ran. Verdict: GREEN."

### D-06: NO new good-to-haves discovered during P77 itself
P77 doesn't grow its own intake. If observation in flight, file as v0.13.0 directly.

### D-07: Verifier subagent dispatch — Path A
Verdict at `quality/reports/verdicts/p77/VERDICT.md`. Verdict checks each closed item's commit message references the GOOD-TO-HAVES entry and that all XS items are closed (or rationale provided).

### D-08: CLAUDE.md update
P77 H3 subsection ≤30 lines listing closures + deferrals. Final commit of the autonomous run also bumps `.planning/STATE.md` cursor to "v0.12.1 in-flight (P67-P71 follow-up session pending)".

### D-09: This is the LAST phase of the autonomous run
After P77 verdict GREEN, the run is complete. The orchestrator writes a session-end summary (ANYTHING shipped, anything deferred, what the human owes — primarily HANDOVER step 1 retire-confirms + the v0.12.0 tag push).

### D-10: ROI awareness over completeness
The 5pm deadline is a real budget. Better to ship 5 XS polish items + a clean verdict than to chase an M-sized item and run over. Per CLAUDE.md global OP-3 (ROI awareness): "if implementation seems very complex for the value it brings, something is wrong."

</decisions>

<canonical_refs>
## Canonical References

- `.planning/milestones/v0.12.1-phases/GOOD-TO-HAVES.md` — the intake file (populated by P72-P76).
- `.planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md` — sibling intake (P76 absorbs).
- CLAUDE.md OP-8 — the +2 phase practice.
- CLAUDE.md global OP-3 (`~/.claude/CLAUDE.md`) — ROI awareness.

</canonical_refs>

<specifics>
## Specific Ideas

- Read `GOOD-TO-HAVES.md` first thing; group by size; sort each group by impact.
- For XS items, batch atomic commits but keep them separate (don't squash; the git log readability is the point).
- For S items, dispatch `gsd-executor` per item if scope-isolated; otherwise inline in the orchestrator.
- The session-end summary commit should be the final commit before the run ends; it's small and read by the human in the morning.

</specifics>

<deferred>
## Deferred Ideas

- An explicit "polish budget" tracking script (count items shipped, items remaining, ETA): nice but adds infra; consider for v0.13.0 if the +2 practice proves valuable across milestones.

</deferred>

---

*Phase: 77-good-to-haves-polish*
*Context gathered: 2026-04-29*
*Source: User directive (autonomous-run +2 phase practice).*
