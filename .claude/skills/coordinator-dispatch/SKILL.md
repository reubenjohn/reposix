---
name: coordinator-dispatch
description: Dispatch template for a reposix phase/debt-drain coordinator — charter to
  paste, canonical role→subagent_type mapping, model-tier table, lane-slicing checklist,
  report contract, relief trigger, pause/resume brief. Pull this before dispatching a
  wave of sub-agents.
when_to_use: When you are a coordinator about to dispatch sub-agents for a phase or
  debt-drain window and need the exact `subagent_type` string to paste (§2 below) —
  doctrine talks about "executor"/"runner"/"reviewer" roles, but none of those bare
  words are registered dispatchable types.
---

# Coordinator dispatch template

Full doctrine: `.planning/ORCHESTRATION.md`. This is the paste-ready operational form.
For a specific stalled/BLOCKER/sketch/resequence/tangent judgment call, use the
`decision-procedures` skill (DP-1..5 + escalation valve) instead of improvising.

## 1. Charter block (paste VERBATIM into every dispatch)

> 1. You own what you touch. Acceptance criteria are the floor, not the ceiling.
> 2. Noticing is a deliverable — report lying doc claims, tests that don't assert what
>    their names promise, teaching-free errors, dead code, missing edge cases. An empty
>    noticing section from code-touching work is a red flag.
> 3. Eager-fix or file — never silently skip. <1h + no new dependency → fix in place;
>    else → SURPRISES-INTAKE / GOOD-TO-HAVES with severity + sketch.
> 4. Verify against reality — run it, render it, hit the backend. A claim without an
>    artifact is not done.
> 5. North star: polish for adoption — would a skeptical first-time dev come away impressed?

## 2. Role → subagent_type mapping (canonical — copy the right-hand column verbatim)

ORCHESTRATION.md and this skill describe dispatch targets by ROLE ("executor", "runner",
"reviewer", "verifier", "reader", "relief", "coordinator"). **None of those bare words is
a registered `subagent_type`** — dispatching one fails with `Agent type '<name>' not
found`. This table is the only source of truth for what to actually paste; if a doc
elsewhere says "dispatch a runner," come back here for the real string.

| Doctrine role | Dispatch with `subagent_type:` | Notes |
|---|---|---|
| executor (writes/edits repo files, incl. plans/handovers) | `gsd-executor` | file-backed |
| runner — build/test during phase execution | `gsd-executor` | no dedicated "runner" type is registered; the executor's own Bash loop runs its build/test cycle. See GOOD-TO-HAVES for a proposed read-only runner. |
| runner — phase-close gate/litmus grading | `gsd-verifier` | `quality/PROTOCOL.md` Step 6/7: the verifier subagent is what actually executes `quality/gates/**` scripts |
| reviewer (diff/code review after a phase ships) | `gsd-code-reviewer` | file-backed |
| verifier (goal-backward phase-close verification) | `gsd-verifier` | fallback `general-purpose` if `gsd-verifier` is unavailable (established pattern, `quality/reports/verdicts/p76/verifier-prompt.md`) |
| reader / reader-digester (>100-line read → ≤300-word digest) | `reader-digester` | file-backed |
| relief (writes+commits a handover) | `relief-handover-writer` | file-backed |
| coordinator (owns a phase/debt-drain window by delegating) | `phase-coordinator` | file-backed |

**Never pass a bare role word as `subagent_type`.** If none of the rows above fit,
`general-purpose` is the safe fallback — never invent a new type name.

## 3. Model tier table

| Lane shape | Model | `subagent_type` |
|---|---|---|
| Genuinely complex / security judgment | opus (explicit `model: opus` override) | `gsd-executor` |
| Default implementation | sonnet | `gsd-executor` |
| Mechanical / leaf / >100-line read | haiku | `reader-digester` |
| Coordinate a wave of sub-lanes | fable (inherit); no-fable mode: explicit `model: opus` (ORCHESTRATION §11) | `phase-coordinator` |

**Never fable at a leaf.**

## 4. Lane-slicing checklist (before dispatching)

- [ ] Lane needs <100 tool calls? If not, split into sub-lanes.
- [ ] Single tree-writer this window? (no two executors editing the tree at once)
- [ ] Any cargo work serialized? (ONE cargo machine-wide; the hook backstops this)
- [ ] Charter block pasted into the dispatch?
- [ ] Evidence destination named (SUMMARY/verdict/transcript path), not "report in chat"?
- [ ] `subagent_type` copied verbatim from §2's right-hand column, not a bare role word?

## 5. Report-format contract (tell every sub-agent)

≤400-word structured report: verdict/outcome, commit SHAs, RAISE LIST for downstream,
intake disposition, and a NOTICED section. Full evidence → committed artifacts, never chat.

**Then triage what comes back (parent's job):** each lane's NOTICED / RAISE LIST is
routed on receipt — absorb into the current wave (low charter-deviation + 10x capacity),
re-delegate as a new lane, or file to intake; never drop it (ORCHESTRATION §2).

## 6. Relief trigger (ask at every wave boundary)

**Am I past ~100k tokens of my own context?** (ABSOLUTE, not % of the window — quality
degrades past ~150k regardless of a 1M window, so "50% of 1M" = 500k is already rotting;
**hard stop ~150k**.) If yes: dispatch `relief-handover-writer` to write+commit the
handover (ORCHESTRATION.md §3 template), confirm its SHA, then request relief from the top
level. Relief is cheap; rot is not. Do not idle-wait for children — end your turn; they
notify.

**Two tiers absorb the churn (ORCHESTRATION §3).** The ~100k line rotates coordinators
more often than the old ~50% line, so don't run a whole milestone as one C1 reporting
rotations to L0. The top orchestrator dispatches a **coordinator-of-coordinators (C2 — a
milestone-scoped `phase-coordinator`)** that dispatches **one C1 `phase-coordinator` per
phase**; a relieving C1's successor is dispatched by its parent C2, not by L0. Same
`phase-coordinator` type at both tiers — the tier is just charter scope (milestone vs
single phase).

## 6a. Liveness — never self-watch CI (push→CI-in-flight is a stop-and-return point)

**Never background your OWN `gh run watch` and end your turn expecting it to re-wake
you** — background-task re-invocation is reliable ONLY at L0; a coordinator's self-owned
watcher goes dormant and stalls the close (root cause: P122 close-liveness incident,
2026-07-18). At the push→CI-in-flight boundary, STOP and RETURN to your dispatching
parent: pushed SHA + in-flight `ci.yml` run id + "awaiting CI green to run post-push
cadence + close." Direct child-agent completion notifications DO reliably re-invoke you;
bare background-bash watchers do not — your parent relays the run id up to L0 (which
holds the durable watch) and SendMessages you to resume on green. Everything before the
push runs straight through — only this one boundary is a stop-and-return point. Full
doctrine: `.planning/ORCHESTRATION.md` §3.

**Corollary — never `fork` to resume or close.** Resume happens by SendMessage-to-id (the
warm agent's context is intact), NOT by forking it. To drive a phase close, dispatch the
verifier→executor LEAVES directly (the P122-blessed deterministic pattern) — NEVER `fork`
a coordinator to "resume/close" it. A fork clones the parent's context and can confabulate
a no-op close (P123-close: the fork returned ZERO tool uses while claiming "close
executing," STATE.md unchanged, caught only by verify-against-reality).
`.planning/ORCHESTRATION.md` §11.

## 6b. SendMessage tier limitation (WHY §6a's corollary is mandatory, not a style choice)

> **SendMessage tier limitation (STANDING; MANAGER decide-and-disclose ruling, owner veto open, 2026-07-18).** SendMessage is NOT granted at the phase-coordinator (C2) tier or below. A C2 cannot SendMessage/halt/resume its background children; a child cannot resume-by-id back to its parent C2. L0→C2 works; C2→main (upward relay) works; the failure is C2→child and child→C2. Therefore C2-tier-and-below coordinators MUST serialize strictly and drive every phase close via FRESH verifier→executor LEAVES (P122 pattern — dispatch leaves directly; NEVER fork a coordinator to resume/close it, never background-and-resume a child). RATIFIED standing doctrine.

This is the mechanism behind §6a: L0 keeps the durable CI watch and SendMessages a
dormant C2/C1 back to life on green (L0→C2 works); a C2/C1 has no equivalent capability
over ITS OWN children (C2→child and child→C2 both fail). If you are a C2/C1 tempted to
background a child and wait for it to resume-by-id, don't — dispatch a fresh leaf instead
(§6a's corollary) or stop-and-return to your own parent per §6a.

## 7. Pause/resume brief template (owner-invoked pause)

Finish the current atomic unit first, THEN dispatch relief-handover-writer with:
`# <N>-PAUSE-HANDOFF.md` → RESUME PROTOCOL: (1) ground-truth git (HEAD sha, tree state,
per-commit one-liners, deviations); (2) where we are in the cycle (DONE/NOT-STARTED per
wave); (3) binding artifacts to read in order; (4) operating constraints (one tree-writer,
one cargo, no --no-verify, push-at-green, trailers, tiering); (5) close requirements
(verdict path, CI green, RAISE LIST, intake disposition, STATE.md advance, final-report
contents); (6) waiver-clock expiries.
