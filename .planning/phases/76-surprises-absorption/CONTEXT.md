# Phase 76: Surprises absorption — the +2 reservation, slot 1 (Context)

**Gathered:** 2026-04-29 (autonomous-run prep)
**Status:** Ready for execution (intake populated during P72-P75)
**Mode:** Triage-then-execute. Read intake; group by severity; resolve serially. `--auto`.
**Milestone:** v0.12.1
**Estimated effort:** Variable (depends on what P72-P75 surfaces). Budget: 1-2 hours.

<domain>
## Phase Boundary

Drain `.planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md`. Each entry transitions to:
- `RESOLVED` (commit SHA in entry footer),
- `DEFERRED` (with new requirement filed under v0.13.0 carry-forward + brief rationale),
- `WONTFIX` (with rationale).

Empty intake is acceptable IF the running phases honestly observed no out-of-scope items. The verifier subagent checks honesty by spot-checking each P72-P75 plan/verdict for skipped findings.

This phase is the operational arm of CLAUDE.md OP-8 (the +2 phase practice). It exists so plans don't have to be perfect at design time — when reality pushes back, the discovering phase appends to intake instead of either silently skipping or expanding scope, and P76 closes the loop.

**Explicitly NOT in scope:**
- Discovering NEW issues; that work is P72-P75.
- Polish items (those go in `GOOD-TO-HAVES.md` and P77).
- The carry-forward bundle (P67-P71) — that's a separate session.

</domain>

<decisions>
## Implementation Decisions

### D-01: Triage by severity, then execute
1. Read `SURPRISES-INTAKE.md` top-to-bottom.
2. Group entries by severity: BLOCKER, HIGH, MEDIUM, LOW.
3. Resolve in severity order. BLOCKER first; LOW last.
4. Within a severity group, resolve in discovery order (oldest first).

### D-02: Each resolution is its own atomic commit
Commit message format: `fix(p76): RESOLVED <intake-entry-title> (was: discovered-by P<N>)`. The commit body MUST quote the original intake entry verbatim and append the resolution rationale. This makes git log readable as a "what surprised us" history.

### D-03: DEFERRED items file v0.13.0 carry-forwards
If an entry is too large for P76 to absorb, defer to v0.13.0 with a new requirement ID (`SURPRISE-DEFERRED-<short-name>`) under the appropriate dimension's REQUIREMENTS section in a new `.planning/milestones/v0.13.0-phases/` skeleton (or append to an existing pending dir if one exists). The deferred entry stays in SURPRISES-INTAKE.md with a footer `STATUS: DEFERRED → v0.13.0/SURPRISE-DEFERRED-<name>`.

### D-04: WONTFIX rationale is one paragraph, max
"We considered X. We chose to leave it because Y. Cost of revisiting later is Z." Footer: `STATUS: WONTFIX | rationale: <paragraph>`.

### D-05: Verifier subagent honesty check
The P76 verifier subagent reads each P72-P75 plan + verdict and asks: "did this phase honestly look for out-of-scope items?" Spot-check by sampling 2 plans and looking for "Eager-resolution" decisions in the plan vs. discovered surprises in the intake. A plan with zero "I considered fixing X but it was out of scope" reasoning when the intake shows X as a discovery is suspicious.

### D-06: Empty intake handling
If SURPRISES-INTAKE.md is empty at P76 start, that's a valid outcome. Phase ships as: "P76 ran. Intake was empty. P72-P75 verifier verdicts spot-checked for honesty. Verdict: no surprises observed; phases honestly looked." Verdict GREEN with empty resolution log.

### D-07: Execution mode flexes by intake size
- Empty OR < 5 LOW-severity entries: `executor` mode (one gsd-executor pass).
- Mixed severities OR ≥ 1 BLOCKER/HIGH: `top-level` mode (orchestrator triages, dispatches per item, possibly fans out subagents).

### D-08: CLAUDE.md update
P76 H3 subsection ≤30 lines under "v0.12.1 — in flight". List the resolutions inline (one line per entry: `<title> → RESOLVED|DEFERRED|WONTFIX | <commit-sha-or-rationale>`).

### D-09: NO new SURPRISES discovered during P76 itself
If P76 discovers new issues while resolving existing ones, append to GOOD-TO-HAVES.md (P77) instead. P76's own out-of-scope items don't go back in P76's own intake — that recursion is forbidden.

### D-10: Verifier subagent dispatch — Path A
Verdict at `quality/reports/verdicts/p76/VERDICT.md`.

</decisions>

<canonical_refs>
## Canonical References

- `.planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md` — the intake file (populated by P72-P75 during execution).
- `.planning/milestones/v0.12.1-phases/GOOD-TO-HAVES.md` — the sibling intake (P77 absorbs).
- CLAUDE.md OP-8 — the +2 phase practice (operating principle).
- `quality/SURPRISES.md` — project-wide pivot journal (DIFFERENT FILE; that's append-only history of mid-execution pivots, not phase-scoped intake).
- v0.12.0 P56 SURPRISES — historical example of the kind of multi-item intake P76 absorbs.

</canonical_refs>

<specifics>
## Specific Ideas

- Read `SURPRISES-INTAKE.md` first thing in the phase to know the shape.
- If intake is large (>10 entries), consider sub-batching into commit groups of 3-5 atomic commits with a brief "P76 batch N" prefix.
- For DEFERRED items, the v0.13.0 stub directory may not exist yet — create `.planning/milestones/v0.13.0-phases/REQUIREMENTS.md` as a thin file with just the deferred items if needed (or accumulate them in a single `pending-v0.13.0.md` if the milestone hasn't been formally scoped).
- Keep individual fix commits SMALL. The P76 verifier reads commits to grade.

</specifics>

<deferred>
## Deferred Ideas

- Automating intake-entry creation via a GSD subcommand (`gsd surprise add`): tempting, but YAGNI until we see this practice used in 2+ milestones.
- Cross-milestone surprises log (queryable by dimension): v0.13.0+.

</deferred>

---

*Phase: 76-surprises-absorption*
*Context gathered: 2026-04-29*
*Source: User directive (autonomous-run +2 phase practice).*
