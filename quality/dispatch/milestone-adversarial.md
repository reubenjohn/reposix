# Milestone-Close Adversarial Pass — TEMPLATE (RBF-FW-12 / D90-09)

> **Origin:** P90 RBF-FW-12 (D90-09, 2026-07-04), draining Decision 3 of
> `.planning/research/v0.13.0-real-backend-frictions/03-synthesis/COMPLETENESS-CHECK.md`
> ("milestone-close adversarial pass ... a fresh subagent reads catalog row
> descriptions only and grades whether assertion would falsify description").
> Sibling of `quality/dispatch/milestone-close-verdict.md` and
> `quality/dispatch/absorption-honesty-spot-check.md`.
>
> **Why this exists:** every other quality-gates check answers "did the
> verifier run and pass?" This pass answers a different, adversarial
> question: "if the row's description were FALSE, would the verifier's
> assertion actually catch that?" A row can PASS honestly today and still
> carry a description whose claim the assertion doesn't actually falsify
> (a congruence gap `RBF-FW-11`'s `claim_vs_assertion_audit` field
> documents but does not independently verify — self-grading a claim
> against itself is not adversarial). This pass is the external check.

## Dispatch contract

A **fresh subagent, distinct from the milestone orchestrator** (same
author-independence requirement as F-K5 clause (b) — see
`quality/dispatch/absorption-honesty-spot-check.md`), is given:

- The catalog row **DESCRIPTIONS ONLY** — `comment`, `expected.asserts`,
  `owner_hint`, and `claim_vs_assertion_audit` text. **NOT** the verifier
  script source, NOT implementation code, NOT prior grading history. The
  point is to grade from the same vantage point a skeptical reader of the
  catalog (not the author) would have.
- One instruction: for each row, decide — **if this row's description
  were false, would the named assertion(s) actually catch that?** Grade
  each row PASS (assertion would falsify a false description) or FAIL
  (assertion is vacuous, tautological, or would still pass even if the
  description were false — e.g. an assertion that only checks a file
  exists when the description claims functional behavior).

## Output artifact

Write the result to
**`quality/reports/verifications/adversarial/<version>.json`**
(D90-09 path — this is deliberately **not** the
`quality/reports/verifications/milestone-adversarial/` path sketched in
`90-RESEARCH-runner.md` § RBF-FW-12; the catalog row comment for
`agent-ux/milestone-adversarial-pass` names the D90-09 path as
authoritative):

```json
{
  "milestone": "v0.13.0",
  "dispatched_at": "2026-07-...Z",
  "subagent": "<author distinct from the milestone orchestrator, per F-K5 clause (b)>",
  "rows_audited": 47,
  "rows_failed": [
    { "id": "dimension/row-id", "reason": "assertion would not falsify description — <why>" }
  ],
  "verdict": "PASS"
}
```

`verdict` is `"PASS"` iff `rows_failed` is empty; `"FAIL"` otherwise —
this is a convenience mirror of the `rows_failed` count, not an
independent judgment call.

## Milestone-close GREEN-block

`quality/runners/verdict.py --milestone <version>` reads this artifact
via `milestone_adversarial_gate(repo_root, version)`:

- **Artifact absent** for the closing milestone → blocks GREEN (forces
  the verdict color to `red`), reason: "adversarial-pass artifact absent
  at `<path>`". A milestone cannot close on the strength of an
  adversarial pass that was never run.
- **Artifact present with ≥1 entry in `rows_failed`** → blocks GREEN,
  reason names the failed row ids.
- **Artifact present with empty `rows_failed`** → the gate does not
  block; the verdict follows `compute_color`/`compute_exit_code` as
  normal (this pass can only **darken** a verdict, never lighten one —
  see `quality/PROTOCOL.md` § "Milestone-close adversarial pass").

## Relationship to the F-K5 absorption spot-check

These are two distinct milestone-close checks, not duplicates:

| | Absorption honesty spot-check (F-K5) | Adversarial pass (RBF-FW-12) |
|---|---|---|
| Question | "Does the shipped feature actually work end-to-end?" | "Would this row's assertion catch a false description?" |
| Scope | Phases that closed the milestone (their claimed outcomes) | Catalog rows (their claimed verification contracts) |
| Reads | Phase artifacts, PLAN/VERIFICATION docs | Catalog row descriptions ONLY (no verifier source, no impl) |
| Author independence | Required (clause b) | Required (mirrors clause b) |

Both are required at milestone-close; neither substitutes for the other.
