# RUNBOOK ch.01 — decision procedures + escalation valve

The 7e2a4cf2 session needed fable-tier judgment roughly five times in eleven
hours. Each of those moments belongs to a CLASS; this chapter converts each class
into an explicit procedure an opus (or sonnet) coordinator executes: **trigger →
evidence (always delegated to an L4 lane) → decision criteria → default action →
what makes it escalate**. If a judgment call doesn't match any DP, run it through
the valve criteria (§Valve) — below the bar, decide-and-record.

---

## DP-1 — Coordinator-rot diagnosis from behavioral signals

**Class exemplar:** the P91 Wave-5.5 polling-loop stall — rot was diagnosed from
the OUTSIDE, from behavior alone, before the coordinator itself noticed.

- **Trigger:** any of these observed in a child coordinator (L1/L2), from its
  reports and the agent tree — you never need its internal context:
  1. stop/watch cycles — it stops, gets poked, stops again without a new commit;
  2. repeated turns of <5 tool calls doing bookkeeping instead of dispatching;
  3. arming watchers/sleeps/polling loops (violates ORCHESTRATION §2 rule 4);
  4. re-asking questions its own earlier reports answered;
  5. report latency growing while commit rate falls;
  6. contradicting its own earlier state ("wave 3 done" then "starting wave 3").
- **Evidence (delegate):** one L4 lane: last 3 reports from the child + `git log
  --oneline` over its window + the QUALITY-LEDGER/verdict artifacts it claims.
  ≤300-word digest: signals seen, commits landed, context % if reported.
- **Decide:** ≥2 signals, or 1 signal + child past ~50% context → ROTATE at the
  next wave boundary. 1 signal + <50% context → send one corrective message
  naming the signal; if it recurs → rotate.
- **Default action — pre-notified rotation (never silent):**
  1. tell the child IN ADVANCE it is being relieved (persistence is solicited,
     not automatic — ORCHESTRATION §3);
  2. child finishes its atomic unit, then a `relief-handover-writer` writes AND
     commits `<N>-HANDOVER.md` (template in ORCHESTRATION §3) and reports the SHA;
  3. spawn the successor coordinator with the handover path as charter §1;
  4. old coordinator stands down. Relief is cheap; rot is not.
- **Escalates when:** the SUCCESSOR shows rot within one wave — that's not rot,
  that's a mis-scoped charter (10x rule violated). Split the phase/portion and
  re-charter; two failed re-charters of the same work → valve E4.

---

## DP-2 — Prove-before-fix on BLOCKERs

**Class exemplar:** a P90-window BLOCKER whose "repro" was a static code trace;
fable forced an executable repro first — the trace had missed a guard.

- **Trigger:** any BLOCKER/HIGH finding (ledger row, verifier RED, intake entry,
  audit-fleet report) whose evidence is a static trace, a reading of code, or a
  reviewer's assertion — anything that has not been EXECUTED.
- **Evidence (delegate):** an L4/L3 runner lane builds a minimal executable
  repro BEFORE any fixer is dispatched: a failing `#[test]`, a script, or a
  shell-subprocess transcript (`quality/reports/transcripts/` shape). Committed.
- **Decide:**
  - repro executes and fails → finding CONFIRMED; the repro becomes the
    regression test; dispatch the fix lane with repro path in its charter.
  - repro cannot be built within one lane budget (~100 tool calls) → finding is
    DOWNGRADED to suspicion: route to `SURPRISES-INTAKE.md` with the static
    trace attached; do NOT fix blind, do NOT block dispatch on it.
  - repro executes and PASSES → finding is FALSE; record `[SELF]` in the ledger
    with the transcript path; close the row.
- **Default:** no fix lane without a committed failing artifact. Fix merges only
  when the repro flips green AND an independent reviewer (HCI) confirms the fix
  addresses the mechanism, not the symptom.
- **Escalates when:** the repro proves a design-level flaw — fixing it would
  change a Load-bearing behavior (root CLAUDE.md list) or public contract →
  valve E2 (ADR-class).

---

## DP-3 — Intake-design inversion

**Class exemplar:** fable inverted an intake entry's sketched design — the sketch
solved the symptom with a new mechanism; the fix used an existing one.

- **Trigger:** before executing ANY intake entry (SURPRISES/GOOD-TO-HAVES) or
  ledger row that carries a sketched design; before any PLAN wave that
  implements such a sketch. Sketches are problem statements, not specs.
- **Evidence (delegate):** an L4 digest lane answers three questions with
  file:line citations: (1) what PROBLEM does the sketch solve? (2) what is the
  simplest mechanism that solves that problem? (3) does the sketch add a new
  artifact/dependency/concept where an existing one could absorb the job?
- **Decide:** if a simpler mechanism covers the problem with at most trivial
  capability loss → INVERT: implement the simpler design. (This is the D-CONV
  doctrine: accept trivial capability loss for major complexity reduction.)
  If capability loss is real but arguably acceptable → weigh against the north
  star (adoption polish); when in doubt, keep capability, file the
  simplification to GOOD-TO-HAVES.
- **Default:** record the inversion in `.planning/CONSULT-DECISIONS.md`
  (`[SELF]`, with the 3-question evidence) BEFORE implementation. Never silently
  deviate from a sketch — the intake author's problem must still be provably
  solved, and the entry's disposition says "RESOLVED (inverted design: <ledger
  ref>)".
- **Escalates when:** the inversion would alter a ratified architecture surface
  (Load-bearing behaviors, wire formats, ref layouts) → valve E2.

---

## DP-4 — Executive resequencing (mission over plan)

**Class exemplar:** OD-4's EXECUTIVE RESEQUENCE — launch-readiness milestone
pulled ahead of the v0.13.2 cross-link work, re-derived from the owner's stated
intention ("puts this project on the global map"), not from phase order.

- **Trigger:** evidence that executing the NEXT PLANNED item no longer serves
  the ratified mission: a dependency inverted, new information invalidated the
  ordering rationale, an owner remark implies a different priority, or the drive
  is tracking >10% of L0 context with portions still outstanding (Amendment 1
  arithmetic broken).
- **Evidence (delegate):** a one-page mission-delta memo from an L4 lane: what
  the plan says next / what the mission (OD-3 north star + OD-4 sequence)
  implies / the delta / cost + reversibility of resequencing.
- **Decide — resequence yourself ONLY when all three hold:**
  1. no ratified owner decision is contradicted (OD-4's sequence itself was
     owner-ratified — reordering WITHIN it is yours; reordering AGAINST it is not);
  2. work is REORDERED, not deleted or de-scoped;
  3. the change is recorded before execution: ROADMAP edit + STATE.md note +
     `[SELF]` ledger entry with the memo attached.
- **Default:** prefer the standing order. Resequencing is the exception that
  requires the memo; "I felt like doing P95 first" is not a memo.
- **Escalates when:** the move deletes/defers owner-ratified scope, changes
  milestone boundaries, or trades mission priorities (ship-fast vs
  polish-for-adoption) → valve E3. Spending beyond pre-authorized budget
  (OD-3's ~$50 P106 pre-auth is the only standing spend) → owner.

---

## DP-5 — Tangent-vs-charter classification

**Class exemplar:** the quality-gates framework ballooned from a tangent into
the project's backbone — the right call, but it needed explicit classification,
not silent absorption.

- **Trigger:** any discovered work not named in the current charter (a bug
  nearby, a missing tool, a doc lie, an ugly seam).
- **Classification test (run in order):**
  1. **Charter test:** does it change what GREEN means for the CURRENT phase
     (a gate this phase must pass, a claim this phase makes)? YES → it is NOT a
     tangent; replan the wave to include it (10x rule says the capacity exists).
  2. **Size test (OP-8 / ORCHESTRATION §5):** <1h AND no new dependency → fix
     inline in the discovering lane, note it in the lane report's noticing
     section. Larger → file to `SURPRISES-INTAKE.md` / `GOOD-TO-HAVES.md` with
     severity + sketch. Never silently skip; never scope-creep to fit.
  3. **Balloon test:** would it consume more than one phase-slot of work? →
     surface as an explicit SCOPE DECISION to the owner (this specific case
     goes to the owner, not fable — it is a budget question only the owner can
     answer).
- **Evidence:** the discovering lane's own report suffices for tests 1–2; the
  balloon case gets a 10-line scope memo (what, why now, cost, cost-of-delay).
- **Default:** file, don't absorb. The intake files are the pressure valve that
  keeps phases honest; OP-8's absorption slots drain them on schedule.
- **Escalates when:** balloon test fires (→ owner), or the tangent reveals a
  security-posture gap (→ owner via valve E1/E3 — security tangents are never
  self-classified as deferrable).

---

## §Valve — the escalation valve (narrow and named)

Everything the DPs don't resolve gets tested against exactly four criteria.
**Escalate ONLY when at least one holds:**

- **E1 — Irreversible or destructive.** History rewrites, deletions of committed
  artifacts, external mutations beyond owner-named sanctioned targets
  (ORCHESTRATION §9), secret/credential handling, anything a `git revert`
  cannot undo. → **OWNER**, always. A permission classifier blocking you here is
  design feedback, not an obstacle.
- **E2 — Architecture-shaping (ADR-class).** The decision changes a Load-bearing
  behavior, a wire/ref/URL format, a public CLI contract, or resolves an ADR
  (e.g. P93's L2/L3). → **FABLE CONSULT** first; owner if the consult concludes
  the options carry irreversible trade-offs.
- **E3 — Mission-priority tradeoff.** Deleting/deferring owner-ratified scope,
  moving milestone boundaries, spending beyond pre-auth, trading polish against
  speed at portfolio level. → **OWNER** for scope/spend; **FABLE CONSULT** for
  priority analysis you'll present to the owner.
- **E4 — Two failed self-attempts at the same gate.** The same row/gate graded
  RED twice with two genuinely different fixes attempted (not the same fix
  twice). → **FABLE CONSULT** with both failure transcripts; owner if the
  consult's recommendation also fails.

**Below the bar: decide-and-record (`[SELF]` ledger entry), never idle.** Waiting
for permission you don't need is a rot signal (DP-1 #2).

### Fable consult-dispatch template (single-shot, reusable)

Dispatched by L0 only, via the Agent tool, `model: fable`, fresh agent (no
inherited context — the evidence digest IS the context). One bounded question per
dispatch. If fable is unavailable in the environment → owner, with the same
package.

```
Agent(model: "fable", description: "Fable consult: <topic>"):

SINGLE-SHOT CONSULT for reposix (/home/reuben/workspace/reposix). You are a
one-question decision consultant. Do not explore the repo beyond the files
listed; do not implement anything.

QUESTION (bounded, one decision):
<one sentence, with enumerated options A/B/C and what "success" means>

EVIDENCE DIGEST (prepared by an L4 reader-digester — you get digests, not dumps):
<≤600 words: the failing artifacts, the constraint set, what was already tried,
file:line citations. Attach paths to committed transcripts/verdicts for
spot-checking.>

BINDING CONSTRAINTS (you may not relax these):
<relevant ODs, D-CONVs, ADRs, Load-bearing behaviors, litmus/OD-2 rules>

DELIVERABLE — append to .planning/CONSULT-DECISIONS.md and commit:
## <date> [FABLE] <question>
- Decision: <option chosen>
- Rationale: <why, tied to the evidence>
- Risks + what would change this answer: <2-3 lines>
- Spot-checks performed: <which attached artifacts you actually opened>
Then report the commit SHA and the decision in ≤200 words.
```

L0 relays the decision to the requesting coordinator and resumes. A consult that
comes back "cannot decide on this evidence" is a valid outcome: gather the named
missing evidence once, re-consult once; still undecidable → owner.
