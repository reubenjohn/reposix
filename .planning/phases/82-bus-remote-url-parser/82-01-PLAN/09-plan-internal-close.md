← [back to index](./index.md) · phase 82 plan 01

## Plan-internal close protocol

After T06 push lands, the plan transitions out of the executor's
hands. The orchestrator (top-level coordinator) handles the
remaining steps:

1. **Verifier subagent dispatched.** Unbiased subagent per
   `quality/PROTOCOL.md § "Verifier subagent prompt template"`
   (verbatim copy). Grades the six P82 catalog rows from artifacts
   with zero session context.
2. **Verdict at `quality/reports/verdicts/p82/VERDICT.md`.** Format
   per `quality/PROTOCOL.md`. Phase loops back if RED.
3. **STATE.md cursor advanced.** Update `.planning/STATE.md` Current
   Position from "P81 SHIPPED ... next P82" → "P82 SHIPPED 2026-MM-DD"
   (commit SHA cited).
4. **REQUIREMENTS.md DVCS-BUS-URL-01 + DVCS-BUS-PRECHECK-01 +
   DVCS-BUS-PRECHECK-02 + DVCS-BUS-FETCH-01 checkboxes flipped.**
   `[ ]` → `[x]` after verifier GREEN.
5. **+2 reservation.** If T02–T06 surfaced any out-of-scope items,
   the discovering subagent appended them to
   `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` (per
   OP-8). The verifier subagent's honesty spot-check confirms
   whether the intake reflects what was actually observed.

NONE of these steps are plan tasks — they are orchestrator actions
following the per-phase-push contract.

