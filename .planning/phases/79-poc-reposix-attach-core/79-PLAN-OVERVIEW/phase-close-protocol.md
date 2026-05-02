← [back to index](./index.md)

# Phase-close protocol

Per CLAUDE.md OP-7 + REQUIREMENTS.md § "Recurring success criteria across
every v0.13.0 phase":

1. **All commits pushed.** Each plan terminates with `git push origin main`
   (per CLAUDE.md "Push cadence — per-phase, codified 2026-04-30, closes
   backlog 999.4"). 79-01 pushes after the POC + FINDINGS land; 79-02
   pushes after the scaffold lands; 79-03 pushes after tests + docs +
   catalog-flip land. Pre-push gate-passing is part of each plan's close
   criterion.
2. **Pre-push gate GREEN** for each plan's push. If pre-push BLOCKS:
   treat as plan-internal failure (fix, NEW commit, re-push). NO
   `--no-verify` per CLAUDE.md git safety protocol.
3. **POC-FINDINGS checkpoint** between 79-01 and 79-02 (see above).
4. **Verifier subagent dispatched.** AFTER 79-03 pushes (i.e., after Wave
   3 completes — NOT after each individual plan), the orchestrator
   dispatches an unbiased verifier subagent per `quality/PROTOCOL.md` §
   "Verifier subagent prompt template" (verbatim copy). The subagent
   grades ALL P79 catalog rows from artifacts with zero session context.
5. **Verdict at `quality/reports/verdicts/p79/VERDICT.md`.** Format per
   `quality/PROTOCOL.md`. Phase loops back if verdict is RED.
6. **STATE.md cursor advanced.** Update `.planning/STATE.md` Current
   Position from "Phase 79 in flight" → "Phase 79 SHIPPED 2026-MM-DD"
   (commit SHA cited). Update `progress` block: `completed_phases: 2`,
   `total_plans: 7`, `completed_plans: 6`, `percent: ~17`.
7. **CLAUDE.md updated in 79-03.** 79-03's terminal task updates §
   "Commands you'll actually use" to add `reposix attach <backend>::<project>`
   example alongside the existing `reposix init` example, and adds a brief
   mention of the cache reconciliation table convention. 79-01 does NOT
   update CLAUDE.md (POC is throwaway research). 79-02 does NOT update
   CLAUDE.md (scaffold without behavior coverage isn't ready for the
   commands section).
8. **GOOD-TO-HAVES.md filed.** Orchestrator (top-level) files the
   GOOD-TO-HAVES-01 entry per § "New GOOD-TO-HAVES entry" above. NOT a
   plan task.
9. **REQUIREMENTS.md DVCS-ATTACH-04 reframed.** Orchestrator updates
   the requirement row per § "Reframe of DVCS-ATTACH-04" above BEFORE
   the verifier subagent dispatches. NOT a plan task.
