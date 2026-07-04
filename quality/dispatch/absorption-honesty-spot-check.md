# Absorption Honesty Spot-Check — TEMPLATE (F-K5)

> **Origin:** P90 RBF-FW-10 (D90-08, 2026-07-04), formalizing the meta-rule
> first named in `.planning/research/v0.13.0-real-backend-frictions/03-synthesis/REMEDIATION-PLAN.md:124`
> ("F-K5 — Honesty-spot-check meta-rule (absorption phase)"). This is the
> TEMPLATE; a per-milestone spot-check REPORT is the artifact an absorption
> phase (Slot 1 of the OP-8 "+2 phase practice") produces when it drains
> `SURPRISES-INTAKE.md`. Sibling of `quality/dispatch/milestone-close-verdict.md`.
>
> **Consumed by:** P96 (v0.13.0's Slot-1 absorption phase) and every future
> milestone's absorption phase. Cross-referenced from
> `.planning/PRACTICES.md` § OP-8 and `quality/PROTOCOL.md` so no absorption
> phase can miss it.

## The F-K5 meta-rule (verbatim — four clauses)

An absorption-phase honesty spot-check is not satisfied by "the phase ran
the framework correctly." It is satisfied only when all four of the
following hold:

1. **(a) Sample MUST include every phase that closed without filing intake.**
   The spot-check sample is not a convenience subset — it is the complete
   set of phases in the milestone that closed clean (zero
   `SURPRISES-INTAKE.md` / `GOOD-TO-HAVES.md` entries filed) plus a
   reasoned sample of phases that did file. A phase that reported zero
   findings is exactly the phase most in need of a spot-check, because an
   empty intake is either (i) genuinely clean or (ii) a phase that stopped
   looking — the honesty check exists to tell those two apart, and it
   cannot do so if it skips the empty-intake phases.

2. **(b) Spot-check author ≠ milestone orchestrator.** The spot-check is
   dispatched as a fresh, independent subagent — never the same session
   (or the same top-level Claude instance) that ran the milestone's
   phases. This is a process-enforcement rule, not a property assertion:
   the v0.13.0 P87 finding (F2, cited in
   `.planning/research/v0.13.0-real-backend-frictions/03-synthesis/COMPLETENESS-CHECK.md:102`)
   was that the orchestrator authored its own honesty spot-check — an
   agent cannot honestly audit its own claim of having looked hard enough.

3. **(c) Rubric question = "walk one critical example end-to-end mentally
   — does it work?"** — NOT "did the phase use the framework correctly?".
   The rubric is deliberately outcome-facing, not process-facing. A phase
   can follow every GSD/quality-gates procedural step (catalog-first
   commit, verifier dispatch, CLAUDE.md update) and still ship something
   that does not actually work end-to-end for the critical path it claims
   to cover. The spot-check subagent picks one load-bearing example per
   sampled phase (a real command, a real user flow, a real push/pull
   round-trip — not a synthetic fixture) and mentally traces it start to
   finish, asking only: does this actually work as claimed?

4. **(d) Verifier hash-binds the spot-check content** — not mere file
   existence. A gutted-but-present spot-check report (the file exists but
   its substance has been stripped, e.g. by a later find-and-replace or an
   automated template refresh that forgot to re-fill the body) must FAIL
   the gate exactly as if the file were absent. The verifier computes a
   content hash (sha256) of the report and compares it against a hash
   recorded at grading time — existence alone is not sufficient evidence
   that a real spot-check occurred.

## Per-milestone report shape

The absorption phase produces a report (suggested path:
`quality/reports/verdicts/p<absorption-phase-N>/honesty-spot-check.md`,
mirroring the P96 precedent named in `ROADMAP.md:272`) containing:

```markdown
# Absorption Honesty Spot-Check — v<version>

**Spot-check author:** <fresh subagent id/session, distinct from the
milestone orchestrator per clause (b)>
**Date:** YYYY-MM-DD

## Sample

- Phases closing with ZERO intake entries: <full list — clause (a)>
- Additional sampled phases (filed intake, spot-checked anyway): <list>

## Per-phase walk (clause (c) rubric)

| Phase | Critical example walked | Does it work end-to-end? | Notes |
|---|---|---|---|
| P<n> | <one concrete command/flow, not a synthetic fixture> | YES/NO | |

## Verdict

- [ ] Every zero-intake phase sampled (clause a)
- [ ] Spot-check author is not the milestone orchestrator (clause b)
- [ ] Every sampled phase answers the "does it work end-to-end" rubric,
      not a "did it follow procedure" rubric (clause c)
- [ ] This report's content hash is recorded by the verifying gate (clause d)

**Status:** ⬜ GREEN | ⬜ RED
```

## Why this template exists (not just prose in PRACTICES.md)

The meta-rule is easy to state and easy to silently skip under time
pressure — exactly the failure mode it exists to catch. Committing it as
a standalone, hash-verified file (rather than only prose inside
`.planning/PRACTICES.md`) means a future absorption phase cannot
"forget" the four clauses; the verifier
(`quality/gates/agent-ux/absorption-honesty-template-present.sh`,
row `agent-ux/absorption-honesty-template-present`) fails loud if this
file goes missing, is edited to drop a clause, or drifts from its
recorded content hash.
