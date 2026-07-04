# P89 — Owner Decisions (post-replan, override plan defaults)

**Date:** 2026-05-16
**Source:** owner conversation (status-catch-up reflective Q: "how do we prevent
falling short of the milestone end state again?"). These post-date
`89-PLAN-OVERVIEW.md`'s 2026-05-08 replan revision and OVERRIDE its defaults
where they conflict. The executor + the planner (if a surgical replan is
triggered) MUST fold these in verbatim before P89 closes.

## Context: the failure mode being prevented

v0.13.0 (P78–P88) shipped GREEN — every per-phase verifier passed, the
milestone-close verdict passed 8 probes — yet an owner-run real-Confluence
dark-factory session (May 2) + 12-subagent audit (May 8) surfaced 37 frictions
/ 16 HIGH + ~51 audit HIGH that the green-graded milestone never caught. Root
cause is structural, not a missed bug: the quality framework structurally
exempted real-backend flows (sim/wiremock graded as "feature ships"), success
criteria were silently dropped under "eager-resolution pivot" with no gate
noticing, and P87's honesty spot-check was authored by the milestone
orchestrator and excluded the two phases (P79, P86) with the biggest scope
cuts. P89/P90 (F-K1…K8) are the designed countermeasures; these two decisions
close residual holes in that design.

## DECISION OD-1 — P89 close is verified by Cross-AI + owner gate

P89/P90 build the gates that grade every other v0.13.0-extension phase. They
**cannot** be credibly graded by the framework they fix, nor by the
orchestrator that builds them (that is the P87 auditor-capture failure,
repeated one level up). Therefore P89 does NOT close on the standard
independent-verifier-subagent path alone.

P89 close requires BOTH:

1. **Cross-AI review** (Claude + Codex + Gemini) of the framework-fix
   deliverables — same shape as the planning-time cross-AI review that
   converged this phase's plan, but run against the *implemented* framework
   changes, not the plan.
2. **Explicit owner sign-off** on the framework-fix close, recorded as a
   committed artifact (not a session statement). Absent owner sign-off ⇒
   P89 does NOT close regardless of catalog-row colour.

Rationale: breaks the recursive-trust loop. A framework fix graded only by the
framework it fixes is the P87 capture failure with extra steps.

## DECISION OD-2 — Unrunnable real-backend gate at milestone-close = hard RED

`RBF-FW-01` (`cadence: pre-release-real-backend`) default-skips in CI when
creds/sanctioned-targets are absent. The current plan wording ("required at
milestone-close") left the skip-at-close case semantically soft. Owner ruling:

> If the `pre-release-real-backend` cadence cannot EXECUTE against the
> sanctioned target at milestone-close, the milestone-close verdict is **RED**.
> Milestone does NOT close. **No owner-waiver. No `until_date`. No
> PASS-with-comment. No skip-counts-as-pass.**

This is strictly stronger than the option-set originally offered (the
"RED unless owner-waived w/ until_date" option was explicitly retracted by the
owner in favour of hard-block). It extends the existing anti-self-licensing
SLOT stance (D-03c: verifier EXISTS, returns NOT-VERIFIED, NO waiver
mechanism) to the cred/target-missing case.

**Distinction the executor must preserve:** the P89 9th-probe SLOT itself is
legitimately `NOT-VERIFIED` (not RED) during P89–P95 because the substrate it
probes isn't built until P91–P95. OD-2 governs a *different* state: the gate
is runnable-in-principle but cannot run at milestone-close because
creds/targets are missing/unreachable. That state is RED, full stop.

## Plan touch-points requiring amendment before P89 executes

- **`89-PLAN-OVERVIEW.md` SC-set + per-task PLAN for RBF-FW-01 (89-03):**
  add the OD-2 hard-RED skip-semantics to PROTOCOL.md's documented contract
  + the runner behaviour spec. The exit-75→NOT-VERIFIED mapping must NOT be
  reachable for the cred-missing-at-close path; that path is exit→RED.
- **`89-PLAN-OVERVIEW.md` P89 close criteria:** add OD-1 (Cross-AI + owner
  sign-off) as an explicit close gate distinct from the standard verifier
  subagent. The standard verifier subagent still runs; it is necessary but
  not sufficient for P89.
- **`quality/PROTOCOL.md` milestone-close verdict template:** the 9th-probe
  entry's failure semantics must encode OD-2 (no waiver column for this
  probe; skip ⇒ RED).

These are surgical contract tightenings, not a scope change. Wave
decomposition + task count are unchanged.
