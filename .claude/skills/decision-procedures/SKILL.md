---
name: decision-procedures
description: Use as a reposix orchestrator/coordinator facing one of five concrete
  situations, unsure whether to act, wait, or escalate — (1) a dispatched child hasn't
  committed in a while, stuck vs slow?; (2) about to dispatch a fix for a BLOCKER/HIGH
  whose only evidence is a code reading, not something executed; (3) an intake/ledger
  row has a fix design already sketched, about to implement as written; (4) the next
  roadmap item no longer seems right given new information; (5) found out-of-charter
  work (bug, missing tool, doc lie) and unsure whether to fix now, file, or ask.
  Also for the fable-consult template or the CONSULT-DECISIONS.md ledger format. NOT
  for routine dispatch (use coordinator-dispatch) or anything a hook already resolves.
---

# Decision procedures + escalation valve

Doctrine home: `.planning/ORCHESTRATION.md` §11. Each DP: **trigger → evidence (delegate
to an L4 lane) → decide → what escalates**. No match → the Valve below.

## DP-1 — Coordinator-rot diagnosis from behavioral signals

Trigger: ≥2 of {stop/watch cycles; repeated <5-tool-call bookkeeping turns; arming
watchers/sleeps/polling (violates ORCHESTRATION §2 rule 4); re-asking already-answered
questions; report latency up while commit rate falls; self-contradiction on wave state}
in a child's reports + `git log` — OR 1 signal + child past ~50% context. (1 signal +
child under ~50% context → don't rotate yet: send one corrective message naming the
signal; rotate only if it recurs.) Decide: above the threshold, rotate at the next wave
boundary via **pre-notified** handover (ORCHESTRATION §3 template) — the
child is told IN ADVANCE, finishes its atomic unit, `relief-handover-writer` commits the
handover, successor spawned with it as charter. Escalates when the SUCCESSOR rots within
one wave — mis-scoped charter (10x rule violated), not rot: split and re-charter; two
failed re-charters → valve E4.

## DP-2 — Prove-before-fix on BLOCKERs

Trigger: any BLOCKER/HIGH finding (ledger row, verifier RED, intake entry, audit-fleet
report) whose evidence is a static trace, a code reading, or a reviewer's assertion —
anything not EXECUTED. Decide: an L4/L3 `gsd-executor` lane builds a minimal repro (a
failing test, a script, or a `shell-subprocess`-shaped transcript) BEFORE any fixer is
dispatched, committed. Repro executes and fails → CONFIRMED, becomes the regression test,
dispatch the fix with the repro path in its charter. Repro can't be built within one lane
budget → DOWNGRADE to suspicion, route to intake with the static trace attached, do not
fix blind. Repro executes and passes → finding is FALSE, record `[SELF]` in the ledger
with the transcript path, close the row. No fix lane without a committed failing
artifact; a fix merges only when the repro flips green AND a fresh `gsd-code-reviewer`
confirms the fix addresses the mechanism, not the symptom. Escalates when the repro
proves a design flaw — fixing it would change a Load-bearing behavior or public
contract → valve E2.

## DP-3 — Intake-design inversion

Trigger: before executing ANY intake entry or ledger row that carries a sketched design,
or any plan wave implementing such a sketch — sketches are problem statements, not specs.
Decide: an L4 digest answers, with file:line citations: (1) what PROBLEM does the sketch
solve? (2) what is the simplest mechanism that solves that problem? (3) does the sketch
add a new artifact/dependency/concept where an existing one could absorb the job? A
simpler mechanism covering the problem with at most trivial capability loss → INVERT:
implement the simpler design, recorded in the decision ledger (`[SELF]`, with the
3-question evidence) BEFORE implementation — never silently deviate from a sketch; the
entry's disposition notes the inversion. Capability loss that's real but arguably
acceptable → weigh against the north star; when in doubt keep capability, file the
simplification instead. Escalates when the inversion would alter a ratified architecture
surface (Load-bearing behaviors, wire formats, ref layouts) → valve E2.

## DP-4 — Executive resequencing (mission over plan)

Trigger: evidence that executing the next planned item no longer serves the ratified
mission — a dependency inverted, new information invalidated the ordering rationale, an
owner remark implies a different priority, or a coordinator is tracking past its own 10%
budget (ORCHESTRATION §11) with scope still outstanding. Decide — resequence yourself
ONLY when all three hold: (1) no ratified owner decision is contradicted (reordering
WITHIN an owner-ratified sequence is yours; reordering AGAINST it is not); (2) work is
REORDERED, never deleted or de-scoped; (3) the change is recorded BEFORE execution
(roadmap edit + state note + `[SELF]` ledger entry with a one-page mission-delta memo:
what the plan says next / what the mission implies / the delta / cost + reversibility).
Prefer the standing order — resequencing is the exception that requires the memo.
Escalates when the move deletes/defers owner-ratified scope, changes milestone
boundaries, or trades mission priorities at portfolio level → valve E3; spending beyond
pre-authorized budget → owner.

## DP-5 — Tangent-vs-charter classification

Trigger: any discovered work not named in the current charter (a bug nearby, a missing
tool, a doc lie, an ugly seam). Classification test, run in order: (1) **charter test** —
does it change what GREEN means for the CURRENT phase? Yes → it is NOT a tangent; replan
the wave to include it (the 10x rule says the capacity exists). (2) **size test** — <1h
and no new dependency → fix inline in the discovering lane, note it in the noticing
section; larger → file to `SURPRISES-INTAKE.md` / `GOOD-TO-HAVES.md` with severity +
sketch. Never silently skip; never scope-creep to fit. (3) **balloon test** — would it
consume more than one phase-slot of work? → surface as an explicit scope decision to the
owner (a ~10-line memo: what, why now, cross-project value, cost, cost-of-delay) — this
is a proposal, not a confession (ORCHESTRATION §5); the owner gates approval, never the
surfacing. Escalates when the balloon test fires (→ owner), or the tangent reveals a
security-posture gap (→ owner via valve E1/E3 — security tangents are never
self-classified as deferrable).

## The escalation valve (E1–E4)

Everything the DPs don't resolve gets tested against exactly four criteria. **Escalate
ONLY when at least one holds; below the bar, decide-and-record (`[SELF]` ledger entry),
never idle** — waiting for permission you don't need is a rot signal (DP-1).

| Code | Condition | Route |
|---|---|---|
| E1 — Irreversible or destructive | History rewrites, deletions of committed artifacts, external mutations beyond owner-named sanctioned targets (ORCHESTRATION §9), secret/credential handling, anything a `git revert` cannot undo | **OWNER**, always. A permission classifier blocking you here is design feedback, not an obstacle. |
| E2 — Architecture-shaping (ADR-class) | Changes a Load-bearing behavior, a wire/ref/URL format, a public CLI contract, or resolves an ADR | **FABLE CONSULT** first; owner if the consult concludes the options carry irreversible trade-offs |
| E3 — Mission-priority tradeoff | Deleting/deferring owner-ratified scope, moving milestone boundaries, spending beyond pre-auth, trading polish against speed at portfolio level | **OWNER** for scope/spend; **FABLE CONSULT** for the priority analysis presented to the owner |
| E4 — Two failed self-attempts at the same gate | The same row/gate graded RED twice with two genuinely different fixes attempted (not the same fix twice) | **FABLE CONSULT** with both failure transcripts; owner if the consult's recommendation also fails |

## Fable consult-dispatch (single-shot, reusable)

Dispatched by the top-level orchestrator only, via the Agent tool, `model: fable`,
fresh agent (no inherited context — the digest IS the context), ONE bounded
question per dispatch. Fable unavailable → owner, same package.

```
Agent(model: "fable", description: "Fable consult: <topic>"):

SINGLE-SHOT CONSULT. You are a one-question decision consultant. Do not explore the
repo beyond the files listed; do not implement anything.

QUESTION (bounded, one decision):
<one sentence, with enumerated options A/B/C and what "success" means>

EVIDENCE DIGEST (prepared by an L4 reader-digester — you get digests, not dumps):
<≤600 words: the failing artifacts, the constraint set, what was already tried,
file:line citations. Attach paths to committed transcripts/verdicts for spot-checking.>

BINDING CONSTRAINTS (you may not relax these):
<relevant ratified decisions, ADRs, Load-bearing behaviors, litmus rules>

DELIVERABLE — append to .planning/CONSULT-DECISIONS.md and commit:
## <date> [FABLE] <question>
- Decision: <option chosen>
- Rationale: <why, tied to the evidence>
- Risks + what would change this answer: <2-3 lines>
- Spot-checks performed: <which attached artifacts you actually opened>
Then report the commit SHA and the decision in ≤200 words.
```

The orchestrator relays the decision and resumes. A consult that returns "cannot decide
on this evidence" is valid: gather the named missing evidence once, re-consult once;
still undecidable → owner.

## Decision ledger

All valve-adjacent decisions land in `.planning/CONSULT-DECISIONS.md` (append-only;
create on first use), one section per decision:

```
## <YYYY-MM-DD> [SELF|FABLE|OWNER] <one-line question>
- Context: <2-3 lines>
- Decision: <what was decided>
- Rationale: <why; what evidence>
- Reversibility: <how to undo, or IRREVERSIBLE>
- Commit: <sha of the change that implements it, when applicable>
```

`[SELF]` = decide-and-record below the bar. `[FABLE]` = consult verdict. `[OWNER]` =
owner stop + reply. An empty ledger after a multi-phase run is itself a red flag.
