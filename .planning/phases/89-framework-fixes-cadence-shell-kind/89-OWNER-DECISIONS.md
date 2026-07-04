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

## DECISION OD-3 — Full-autonomy drive to v1.0

**Date:** 2026-07-03
**Source:** owner (Reuben), interactive session with the coordinating agent,
ratified at execution kick-off after the 8-week idle gap (2026-05-16 →
2026-07-03). Post-dates OD-1/OD-2. AMENDS OD-1's sign-off mechanics (see
"Autonomy" below); OD-2 remains in force UNCHANGED.

### Scope — drive to v1.0

Complete v0.13.0-ext (P89–P97, tag v0.13.0), then v0.13.2 (P98–P107, tag
v0.13.2) **STRICTLY SERIALLY** — workstream A then workstream B. P98's
"Depends on: v0.13.0 milestone GREEN" is taken literally; the
parallel-worktree model in STATE.md is retired. After both tags, formalize
the research-only ladder (v0.14.0 observability/multi-repo → plugin
ecosystem/launch readiness → v1.0.0 + ADR-009 semver activation) as real
GSD milestones via `/gsd-new-milestone` and execute them.

### Branching

`main` becomes the working branch: `workstream/v0.13.0-ext` is
fast-forwarded into `main` on 2026-07-03 and retired; the per-phase push
cadence (CLAUDE.md § "Push cadence — per-phase") targets `origin/main`
directly from here on.

### Autonomy — full, including former hard gates

1. **OD-1 amendment.** OD-1's cross-AI review of the *implemented*
   framework at P89 close REMAINS REQUIRED, but the "explicit owner
   sign-off" leg (OD-1 item 2, including its "Absent owner sign-off ⇒ P89
   does NOT close" clause) is delegated to the orchestrator — the owner is
   notified in the session summary instead of blocking P89 close.
2. **Tag pushes delegated.** Tag pushes at P97 (v0.13.0) and P107
   (v0.13.2) are delegated to the orchestrator, contingent on GREEN
   milestone verdicts.
3. **Spend pre-authorization.** Real-money spend for v0.13.2 P106 L3
   dogfood is pre-authorized up to ~$50.
4. **Gates unchanged.** OD-2 remains in force UNCHANGED, and litmus REOPEN
   gates remain in force — autonomy does not weaken gates; on RED the
   orchestrator loops back, never waives to keep moving.

### North star

Owner's words, verbatim:

> "The project is not yet used by anyone, and I want it to go as far as
> possible and reach a state where it is a highly polished tool for
> thousands of devs to adopt like wildfire."

All subsequent scope/priority judgment calls resolve toward
polish-for-adoption.
