---
name: coordinator-dispatch
description: Dispatch template for a reposix phase/debt-drain coordinator — charter to
  paste, model-tier table, lane-slicing checklist, report contract, relief trigger,
  pause/resume brief. Pull this before dispatching a wave of sub-agents.
when_to_use: When you are a coordinator about to dispatch executor/reader/reviewer
  sub-agents for a phase or debt-drain window.
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

## 2. Model tier table

| Lane shape | Model | Agent |
|---|---|---|
| Genuinely complex / security judgment | opus | executor (opus) |
| Default implementation | sonnet | gsd-executor / executor |
| Mechanical / leaf / >100-line read | haiku | reader-digester |
| Coordinate a wave of sub-lanes | fable (inherit); no-fable mode: explicit `model: opus` (ORCHESTRATION §11) | phase-coordinator |

**Never fable at a leaf.**

## 3. Lane-slicing checklist (before dispatching)

- [ ] Lane needs <100 tool calls? If not, split into sub-lanes.
- [ ] Single tree-writer this window? (no two executors editing the tree at once)
- [ ] Any cargo work serialized? (ONE cargo machine-wide; the hook backstops this)
- [ ] Charter block pasted into the dispatch?
- [ ] Evidence destination named (SUMMARY/verdict/transcript path), not "report in chat"?

## 4. Report-format contract (tell every sub-agent)

≤400-word structured report: verdict/outcome, commit SHAs, RAISE LIST for downstream,
intake disposition, and a NOTICED section. Full evidence → committed artifacts, never chat.

**Then triage what comes back (parent's job):** each lane's NOTICED / RAISE LIST is
routed on receipt — absorb into the current wave (low charter-deviation + 10x capacity),
re-delegate as a new lane, or file to intake; never drop it (ORCHESTRATION §2).

## 5. Relief trigger (ask at every wave boundary)

**Am I past ~50% of my context?** If yes: dispatch `relief-handover-writer` to
write+commit the handover (ORCHESTRATION.md §3 template), confirm its SHA, then request
relief from the top level. Relief is cheap; rot is not. Do not idle-wait for children —
end your turn; they notify.

## 6. Pause/resume brief template (owner-invoked pause)

Finish the current atomic unit first, THEN dispatch relief-handover-writer with:
`# <N>-PAUSE-HANDOFF.md` → RESUME PROTOCOL: (1) ground-truth git (HEAD sha, tree state,
per-commit one-liners, deviations); (2) where we are in the cycle (DONE/NOT-STARTED per
wave); (3) binding artifacts to read in order; (4) operating constraints (one tree-writer,
one cargo, no --no-verify, push-at-green, trailers, tiering); (5) close requirements
(verdict path, CI green, RAISE LIST, intake disposition, STATE.md advance, final-report
contents); (6) waiver-clock expiries.
