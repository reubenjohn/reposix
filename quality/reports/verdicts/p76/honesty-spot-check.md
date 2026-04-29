# P76 Honesty Spot-Check (D-05)

**Sampled:** P74, P75.

**Pair rationale:** P74 produced the highest intake yield in the v0.12.1
cluster (2 entries — the "found-and-logged" path) and P75 produced zero
intake (the "looked-but-found-nothing-and-said-so" path). Sampling both
covers the practice's two failure modes: (a) under-reporting (a phase
finds something but skips it silently), and (b) over-reporting (a phase
fabricates intake to look diligent). A single agent can be biased both
directions; sampling both polarities tightens the check.

---

## P74 — narrative-ux-prose-cleanup (2 intake entries)

**PLAN.md OP-8/D-09 reference:** present and load-bearing. PLAN.md:47 reads
> "OP-8 honesty audit: any out-of-scope discovery during execution (e.g.
> `reposix spaces --help` actually broken, connector matrix actually missing
> from `docs/index.md`) is either eager-fixed (< 1 hour, no new dep, no new
> file outside the planned set per D-12) OR appended to
> `.planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md` with severity +
> sketched-resolution per CLAUDE.md OP-8."

The plan named the eager-fix-vs-intake decision criterion explicitly and
even pre-specified two example surprises that might surface (the
connector-matrix one DID surface, ratifying the plan's anticipation).

**Eager-resolution decisions named in plan/SUMMARY:** 3 — recorded in P74
SUMMARY § "Eager-fix decisions (in-phase, D-09 / OP-8)":
1. connector-matrix regex widened (commit `c8e4111`)
2. audit-trail SIGPIPE fix (commit `dd89abd`)
3. bind sweep promoted to `scripts/p74-bind-ux-rows.sh`

**Intake entries from this phase:** 2 (both 2026-04-29 from
SURPRISES-INTAKE.md timestamp ordering):
- 20:55 / LOW — linkedin Source::Single STALE_DOCS_DRIFT (resolved by P75 9e07028)
- 20:56 / LOW — connector-matrix synonym mismatch (resolved by P76 WONTFIX +
  P77 GOOD-TO-HAVE)

**SUMMARY § naming each intake-discovery:** YES.
- Linkedin row STALE: P74 SUMMARY § "Catalog deltas" line "1 BOUND -> STALE_DOCS_DRIFT
  (unintended side-effect — confirms P75 walker bug for Source::Single)" + § "Row
  transitions" §6 + § "SURPRISES-INTAKE entries (2 LOW)" entry #1.
- Connector-matrix synonym: P74 SUMMARY § "Eager-fix decisions" #1 ("connector-matrix
  regex widened ... heading rename to 'Connector capability matrix' filed as future
  GOOD-TO-HAVE") + § "SURPRISES-INTAKE entries" entry #2.

**VERDICT.md flag (P74 dimension 13: OP-8 honesty):** PASS.
> "SURPRISES-INTAKE.md gained 2 NEW P74 entries: (a) linkedin walker bug
> (P75-fix scope, severity LOW, sketched-resolution names P75 explicitly)
> and (b) connector-matrix synonym mismatch (severity LOW, P77 GOOD-TO-HAVES
> candidate). Both entries match what an unbiased reader would expect P74
> to surface given the scope... This is NOT a noise-to-satisfy-the-practice
> intake — both entries are actionable and severity-justified."

**Finding: GREEN.** Eager-fix decisions logged in plan + SUMMARY; intake
entries trace to SUMMARY paragraphs that name them; the verifier subagent
already independently graded OP-8 honesty PASS. No silent skips. The
3-eager-fix-plus-2-intake split is internally consistent: the eager fixes
are < 1h scope-local micro-corrections that meet D-12; the 2 intake entries
are out-of-scope discoveries (P75 walker code; cosmetic heading rename).

---

## P75 — bind-verb-hash-fix (zero intake entries — the harder honesty case)

**PLAN.md OP-8/D-09 reference:** present. PLAN.md:62 references
SURPRISES-INTAKE.md as the input source for the P74 broadening that
expanded P75's regression-test scope (Source::Single). PLAN.md:328
explicitly carves the two pre-P72 STALE rows out of P75 scope:
> "The pre-existing two STALE_DOCS_DRIFT rows from the P72 entry in
> SURPRISES-INTAKE.md (`polish-03-mermaid-render` + `cli-subcommand-surface`)
> MAY still be STALE — they are out of P75 scope per the SURPRISES-INTAKE
> entry (P76 drains them). The verdict file must explicitly note that those
> two rows are NOT P75's responsibility and that the count of net-new
> STALE_DOCS_DRIFT transitions caused by P75 is zero."

This is exemplary OP-8 plumbing: the plan reads its predecessors' intake,
absorbs the broadening, and pre-declares the carve-out for the descendant
phase (P76) to drain.

**Eager-resolution decisions named in plan/SUMMARY:** zero, AND that's
honest — the fix landed cleanly per the plan with no in-flight
micro-corrections.

**Intake entries from this phase:** zero.

**SUMMARY § honest-empty-intake claim:** YES, explicit. P75 SUMMARY §
"SURPRISES-INTAKE / GOOD-TO-HAVES appends":
> "SURPRISES-INTAKE: none — the fix landed cleanly. No bind/walker bugs
> surfaced beyond the documented scope. The P74 'didn't heal' broadening was
> confirmed-not-a-bug (procedural; walker contract is intentional), so no
> NEW intake entry is warranted."
>
> "GOOD-TO-HAVES: none — no polish opportunities observed during this
> phase."

The empty-intake is explicitly justified, not silent. The "didn't heal"
finding from P74 was investigated (Test C) and reclassified as
confirmed-not-a-bug — a non-trivial honesty move (the easy path would have
been to silently file it).

**VERDICT.md flag (P75 dimension 11: D-09 honesty / OP-8):** PASS.
> "The SUMMARY's claim that 'Test C passed pre-fix' is independently
> verified: pre-fix worktree run shows `walk_single_source_rebind_heals_after_drift
> ... ok`. The P74 broadening was therefore correctly identified as a
> procedural finding (walker contract: walks don't heal, binds do), not
> a second bug. The honesty grade is direct, falsifiable, and falsifiable
> in either direction — the verifier could have caught a lie. None found.
> SURPRISES-INTAKE / GOOD-TO-HAVES 'none' entries are credible: the fix
> landed cleanly."

The verifier explicitly noted falsifiability and tested the falsifiable
claim (ran the pre-fix tests in a worktree). That's the strongest possible
honesty evidence: a verifier with the ability to catch a lie, looking for
one, finding none.

**Finding: GREEN.** The "looked-but-found-nothing-and-said-so" path is
fully exercised. P75 explicitly justifies its empty intake with a
falsifiable claim that the verifier independently re-ran. No silent
skips, no padded findings.

---

## Aggregate

The 3 SURPRISES-INTAKE entries trace to 3 phase boundaries (P72: 1, P74:
2). 4 phases ran in the v0.12.1 cluster (P72-P75); 2 produced intake (P72,
P74) and 2 produced none (P73, P75). The {found-some, found-none-and-said-so}
distribution is consistent with phases honestly looking — neither the
under-reporting failure mode (skipped findings smuggled into a clean
verdict) nor the over-reporting failure mode (cosmetic intake to satisfy
the practice) is in evidence.

Both sampled phases' verifier subagents independently graded OP-8 honesty
PASS / COVERED, with the P75 verifier going so far as to executable-cross-
check the falsifiable empty-intake claim. The +2 phase practice is
operating as designed: each phase's discovering moment is either
eager-fixed in-phase with traceable commits (P74 c8e4111, dd89abd, efc75ab)
or routed to the dedicated absorption slot (P76, this phase).

P76's own resolutions (entry-1 rebinds, entry-2 annotation, entry-3
WONTFIX + GOOD-TO-HAVE filing) close the loop without re-introducing the
practice's failure modes (no recursion into SURPRISES-INTAKE per D-09;
new findings route to GOOD-TO-HAVES.md instead).

**Aggregate finding: GREEN.**

The verifier subagent (Wave 7 dispatch) MUST independently re-run this
spot-check from zero context and may sample a different pair (e.g.,
P72 + P73) to broaden coverage. This pre-grade is offered as evidence
the executor looked, not as a substitute for an unbiased verifier read.
