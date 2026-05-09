# v0.13.0 corrective phases — investigation + ratified plan

Investigation bundle for the v0.13.0 milestone extension (P89–P97). v0.13.0 graded GREEN structurally but failed end-to-end on real backends; this directory holds the audits that surfaced that gap and the ratified plan that closes it.

## Status

- v0.13.0 milestone-graded GREEN on 2026-05-01; **tag NOT pushed**; `.planning/STATE.md:5` still says `ready-to-tag`.
- 4-subagent dark-factory exercise (May 2) + 12-subagent codebase audit (May 8) found **0/3 milestone roles** (SoT-holder / mirror-only consumer / round-tripper) work end-to-end on a real backend.
- Owner ratified **Path A** (Option B from STRATEGIC-REFRAME Q1+Q2): hold the v0.13.0 tag, extend the milestone with corrective phases **P89–P97**.
- Phase shape: **7 work + 2 reservation**. Plan ratified in `03-synthesis/REMEDIATION-PLAN.md`.
- Total estimated effort: **~42–45 days** (was ~36d before Decision 2 added new P93).

## Ratified decisions

- **Decision 1 — mid-stream litmus checkpoints:** APPROVED. After P91/P92/P93/P94 ship GREEN, re-run the relevant dark-factory T-N against the real backend; if ≥1 HIGH friction, the phase REOPENS before the next starts. Phase-gate semantics, not soft success criterion.
- **Decision 2 — don't defer cache-coherence + recovery:** APPROVED. L2/L3 cache-coherence redesign + `SotPartialFail` recovery test promoted out of v0.14.0 deferral into a new **P93** between P92 and P94 (renumbering existing P93–P96 → P94–P97).
- **Decision 3 — P89/P90 patches plus structural check:** BOTH required. F-K1..F-K8 patches PLUS a **claim-vs-assertion congruence** structural check on every catalog row, plus a milestone-close adversarial pass.
- **Decision 4 — existing GREEN verdict files:** Option (b) — overlay each existing verdict file with a 2026-05-08 `EXTENDED-PENDING-P89-P97` banner; P97's milestone-close verdict OVERWRITES `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md`.

(S1 cross-AI peer review via `gsd-review` was approved in the prior session — see `_archive/SESSION-2026-05-08-HANDOFF.md`.)

## Pre-P89 housekeeping (3 edits required BEFORE invoking `/gsd-phase`)

A cold agent who skips these will think the next move is `bash .planning/milestones/v0.13.0-phases/tag-v0.13.0.sh` — silently undoing the strategic decision.

1. Flip `.planning/STATE.md:5` `status:` from `ready-to-tag` to `extending-via-corrective-phases-p89-p97`.
2. Update `CHANGELOG.md:9` (or wherever the "PENDING owner tag-cut" line lives) to `PENDING P89–P97 GREEN before tag-cut`.
3. Add a guard to `.planning/milestones/v0.13.0-phases/tag-v0.13.0.sh` that fails if `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md` is dated before 2026-05-08 (i.e., is the original GREEN verdict, not the post-extension one). Or rename the script to `.disabled` until P97 GREEN.

## First concrete task

Invoke `/gsd-phase` to append P89–P97 to `.planning/milestones/v0.13.0-phases/ROADMAP.md`, sourcing each phase's goals / REQ-IDs / success-criteria / dependencies / execution-mode verbatim from `03-synthesis/REMEDIATION-PLAN.md` § "Proposed v0.13.0 extension phase shape". **DO NOT use `/gsd-new-milestone`** — milestone identity is settled (extend, not new).

## Reading order for cold agents (~30 min)

1. **This README** — orient, ratified decisions, housekeeping. ~5 min.
2. `03-synthesis/REMEDIATION-PLAN.md` § "Proposed v0.13.0 extension phase shape" + "Dependency graph" + "Effort summary". The plan as ratified, with the 4 decisions absorbed. SKIP §1 inventory + §6 deferral table unless drilling into a specific REQ-ID. ~15 min.
3. `03-synthesis/PATTERNS.md` § "What ties them all together" — the meta-pattern. ~3 min.
4. `03-synthesis/STRATEGIC-REFRAME.md` § Q5 (CTO brief) only. SKIP Q1–Q4, Q6 (settled or background). ~5 min.

OPTIONAL drill-in (when a specific REQ-ID needs context): `02-phase-audits-may08/phase-audit-p<n>.md`, `01-dark-factory-may02/T<n>-*.md`, `03-synthesis/COMPLETENESS-CHECK.md` (annotated with disposition banner).

## Hard constraints (the next session inherits)

- **Orchestrator-only / "main agent never executes"** — CLAUDE.md OP-2. **P89, P90, P96, P97 are `Execution mode: top-level`** per REMEDIATION-PLAN; do NOT run inside `/gsd-execute-phase`.
- **Two-channel rule** — subagents write FULL detail to disk; return ≤300-word TLDR to orchestrator.
- **Build memory budget** — never run more than one cargo invocation at a time; per-crate over `--workspace`. CLAUDE.md § "Build memory budget".
- **Catalog-first per phase** — every phase's FIRST commit writes catalog rows defining its GREEN contract. CLAUDE.md § "Quality Gates".
- **Per-phase push BEFORE verifier** — `git push origin main` is part of phase close, not an end-of-session sweep.
- **Simulator default; real-backend tests gate milestone-close** — CLAUDE.md OP-1. Operationalizing this for the milestone-close gate is the whole point of the corrective phases.

## Directory layout

```
01-dark-factory-may02/   # research input — 4-subagent dark-factory exercise
02-phase-audits-may08/   # research input — 12-subagent codebase audit
03-synthesis/            # synthesis + ratified plan (PRIMARY RECORD post-consolidation)
_archive/                # superseded orchestrator docs from this session's evolution
README.md                # this file — single entry point
```

## Trail

The 5 docs in `_archive/` capture how this session iterated: HANDOFF (decisions made) → SYNTHESIS-VERIFICATION (drift audit of synthesis docs) → DECISIONS-NEEDED (4 questions teed up for owner) → READY-TO-EXECUTE (cold-start sketch, superseded by this README) → COMPLETENESS-CHECK-2 (adversarial review of those 3 docs). Drifts have been incorporated into `03-synthesis/`; decisions ratified above.
