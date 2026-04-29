# P76 Verifier-Dispatch Brief

Per CLAUDE.md OP-7 + CONTEXT.md D-10: the executing agent does NOT grade
itself. The TOP-LEVEL ORCHESTRATOR (not gsd-executor; gsd-executor lacks
the `Task` tool per CLAUDE.md "Subagent delegation rules") dispatches
`gsd-verifier` Path A. This brief captures the verbatim prompt + input
list so the orchestrator can dispatch without re-reading SUMMARY.

## Dispatch invocation

```
Task(subagent_type="gsd-verifier",
     description="P76 verifier dispatch",
     prompt=<verbatim QG-06 prompt template from quality/PROTOCOL.md, with N=76>)
```

Fallback if `gsd-verifier` is unavailable: `subagent_type="general-purpose"`.

## Verifier prompt (verbatim, N=76)

You are an unbiased verifier subagent dispatched per CLAUDE.md OP-7 and
quality/PROTOCOL.md QG-06. You have ZERO session context. You read only
the artifacts listed below and grade Phase 76 against its PLAN.md
must_haves and CONTEXT.md decisions.

**Phase contract:**
- PLAN.md: `.planning/phases/76-surprises-absorption/PLAN.md` (must_haves §)
- CONTEXT.md: `.planning/phases/76-surprises-absorption/CONTEXT.md`
  (decisions D-01 through D-10)
- Requirement: `SURPRISES-ABSORB-01` (drains v0.12.1 SURPRISES-INTAKE.md to
  terminal state)

**You MUST grade these dimensions and write the verdict to
`quality/reports/verdicts/p76/VERDICT.md`:**

1. All 3 SURPRISES-INTAKE.md entries are terminal (RESOLVED|WONTFIX|DEFERRED);
   zero `**STATUS:** OPEN` lines remain.
2. Entry 1 row mutations landed in catalog: both
   `polish-03-mermaid-render` and `cli-subcommand-surface` show
   `last_verdict ∈ {BOUND, RETIRE_PROPOSED}` with refreshed source_hash if
   rebound.
3. Entry 2 STATUS footer references commit `9e07028` with linkedin
   row `last_verdict == BOUND` in live catalog.
4. Entry 3 STATUS footer is WONTFIX; GOOD-TO-HAVES.md gained exactly 1
   new XS entry for the heading rename.
5. Live walker post-action: zero rows with `last_verdict ==
   STALE_DOCS_DRIFT` (run `target/release/reposix-quality doc-alignment
   walk` and `jq '[.rows[] | select(.last_verdict == "STALE_DOCS_DRIFT")] |
   length' quality/catalogs/doc-alignment.json` — both must show empty/0
   for the post-P76 state).
6. CLAUDE.md `### P76 — Surprises absorption` H3 exists under `## v0.12.1
   — in flight`, body ≤30 lines, banned-words lint passes.
7. Atomic commits in plausible order (entry-1a → entry-1b → evidence →
   entry-2 → entry-3 → CLAUDE.md → SUMMARY); each commit cites P76 + the
   resolved entry id.
8. No prohibited actions: no `git push`, no tag, no `cargo publish`, no
   `confirm-retire`, no `--no-verify`.

**You MUST independently execute the D-05 honesty cross-check:**

Sample at least 2 of {P72, P73, P74, P75} plan + verdict pairs (you may
sample a different pair than the executor's pre-grade at
`quality/reports/verdicts/p76/honesty-spot-check.md` to broaden coverage).
For each sampled phase, verify:

- PLAN.md names an OP-8 / D-09 clause and the eager-fix-vs-intake decision
  criterion.
- Eager-resolution decisions logged in PLAN.md or SUMMARY.md trace to
  visible commits.
- Intake entries from that phase trace to SUMMARY paragraphs naming the
  discovery; conversely, SUMMARY paragraphs naming "out-of-scope" findings
  trace to intake entries (no skipped findings smuggled past).
- For phases with empty intake (P73, P75): SUMMARY explicitly states "we
  looked, found nothing out-of-scope" or equivalent. Empty intake without
  the explicit note is a YELLOW signal.

**DO NOT just rubber-stamp the executor's pre-grade at
`quality/reports/verdicts/p76/honesty-spot-check.md`.** Re-execute the
check from zero context. The executor's pre-grade is offered as evidence
the executor looked, not as a substitute for an unbiased read.

**Verdict format:** mirror the P75 VERDICT.md shape — frontmatter with
`overall_verdict`, dimension table with PASS/PARTIAL/RED/NOT_COVERED for
each numbered dimension, findings/observations section, recommendation
to advance to P77 (good-to-haves polish, slot 2 of the +2 reservation).

## Inputs the verifier reads with zero session context

- `.planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md` (3 STATUS
  footers terminal: RESOLVED | RESOLVED | WONTFIX)
- `.planning/milestones/v0.12.1-phases/GOOD-TO-HAVES.md` (1 new XS entry
  for connector-matrix heading rename)
- `quality/catalogs/doc-alignment.json` (entry-1 rows BOUND with
  refreshed hashes; linkedin row BOUND from P75)
- `quality/reports/verdicts/p76/triage.md` (executor's disposition
  table + sed evidence)
- `quality/reports/verdicts/p76/walk-after.txt` (live walker output
  post-action)
- `quality/reports/verdicts/p76/status-after.txt` (status summary
  post-action: claims_bound=331, alignment_ratio=0.9246)
- `quality/reports/verdicts/p76/honesty-spot-check.md` (executor's D-05
  pre-grade — read for cross-check, do NOT rubber-stamp)
- `CLAUDE.md` (P76 H3 confirmed via `git diff main...HEAD -- CLAUDE.md`
  or by reading the file directly)
- `.planning/phases/{72,73,74,75}-*/PLAN.md` and
  `quality/reports/verdicts/p{72,73,74,75}/VERDICT.md` (D-05 honesty
  cross-check inputs)
- `.planning/phases/76-surprises-absorption/SUMMARY.md` (the SUMMARY
  this verdict closes the loop on)
- `.planning/phases/76-surprises-absorption/PLAN.md` (must_haves §)
- `.planning/phases/76-surprises-absorption/CONTEXT.md` (D-01..D-10)

## Verdict path

`quality/reports/verdicts/p76/VERDICT.md`

Phase 76 does NOT close until the verdict reads GREEN.
