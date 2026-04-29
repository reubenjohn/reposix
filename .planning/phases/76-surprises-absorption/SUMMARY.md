---
phase: 76-surprises-absorption
plan: 01
status: COMPLETE
requirement_closed: SURPRISES-ABSORB-01
milestone: v0.12.1
mode: --auto (sequential gsd-executor on main, depth-1)
duration_min: ~10
verifier_verdict_path: quality/reports/verdicts/p76/VERDICT.md
---

# Phase 76 Plan 01: Surprises absorption — Summary

One-liner: Drained the v0.12.1 SURPRISES-INTAKE (3 LOW entries discovered
during P72 + P74) to terminal status; +2 phase practice (OP-8) is
operationally proven, not just designed; honesty spot-check on P74 + P75
graded GREEN with verifier-falsifiability already independently demonstrated
in P75's verdict.

## Disposition table

| Entry | Discovered-by | Severity | Disposition | Commit / Rationale |
|-------|---------------|----------|-------------|--------------------|
| 1a polish-03-mermaid-render | P72 | LOW | REBIND (BOUND) | `0467373` — claim text matches `.planning/milestones/v0.11.0-phases/REQUIREMENTS.md:85` verbatim; source_hash refreshed `c88cd0f9 → 6ec37650` |
| 1b cli-subcommand-surface | P72 | LOW | REBIND (BOUND) | `fbc3caa` — `enum Cmd` still at `crates/reposix-cli/src/main.rs:37`; closing `}` still at :299; 12-subcommand surface intact; source_hash refreshed `b9700827 → 89b925f5` |
| 2 linkedin Source::Single | P74 | LOW | RESOLVED (annotation) | `800af78` annotates the entry; row was already healed by P75 commit `9e07028` (verbs::bind hash-overwrite fix in `69a30b0` + explicit re-bind landed BOUND with `7a1d7a4e`) |
| 3 connector-matrix synonym | P74 | LOW | WONTFIX + new GOOD-TO-HAVE | `258f284` — regex widening (P74 `c8e4111`) is the complete fix; heading rename filed as P77 GOOD-TO-HAVE (size XS, impact clarity) |

## Commits (newest first)

| SHA       | Type | What                                                            |
|-----------|------|-----------------------------------------------------------------|
| `c8f648f` | docs | honesty-spot-check.md (D-05) + verifier-prompt.md (Path A brief) |
| `aff7853` | docs | CLAUDE.md P76 H3 (27 body lines, banned-words clean)            |
| `258f284` | fix  | WONTFIX entry-3 + file P77 GOOD-TO-HAVE (connector-matrix)      |
| `800af78` | fix  | RESOLVED entry-2 linkedin (annotation referencing 9e07028)      |
| `1b14cb4` | fix  | RESOLVED entry-1 evidence (walk + status + intake-footer)       |
| `fbc3caa` | fix  | RESOLVED entry-1b cli-subcommand-surface rebind                 |
| `0467373` | fix  | RESOLVED entry-1a polish-03-mermaid-render rebind               |

7 atomic commits landed; SUMMARY is the 8th (orchestrator-landed). Each
commit body quotes the original SURPRISES-INTAKE entry verbatim and
appends the resolution rationale per CONTEXT.md D-02.

## Catalog deltas

| Metric                   | Pre-P76 (P75 post) | Post-P76 | Delta    |
|--------------------------|--------------------|----------|----------|
| `claims_bound`           | 329                | 331      | +2       |
| `claims_missing_test`    | 0                  | 0        | 0        |
| `claims_retire_proposed` | 27                 | 27       | 0        |
| `claims_retired`         | 30                 | 30       | 0        |
| `claims_stale_docs_drift`| 2                  | 0        | -2       |
| `alignment_ratio`        | 0.9190             | 0.9246   | +0.0056  |

Target invariant met: P76 was scoped to drive the two pre-existing STALE
rows (P72 entry) to terminal state. Post-action `jq '[.rows[] |
select(.last_verdict == "STALE_DOCS_DRIFT")] | length'
quality/catalogs/doc-alignment.json` reports 0. Live walker output at
`quality/reports/verdicts/p76/walk-after.txt` shows zero net new
STALE_DOCS_DRIFT introduced by P76's actions.

## Honesty spot-check (D-05) inline summary

**Sampled:** P74 (highest intake yield: 2 entries — "found-and-logged"
path) and P75 (zero intake — "looked-but-found-nothing-and-said-so" path).

**P74 finding: GREEN.** PLAN.md names the OP-8 honesty audit
explicitly with the eager-fix-vs-intake criterion. SUMMARY § "Eager-fix
decisions" logs 3 in-phase micro-corrections (commits `c8e4111`,
`dd89abd`, `efc75ab`); SUMMARY § "SURPRISES-INTAKE entries" lists the 2
out-of-scope discoveries that became this phase's intake. Cross-check:
P74's verifier (Path A) independently graded dimension 13 ("OP-8 honesty")
COVERED with the explicit note that the entries are "actionable and
severity-justified ... NOT a noise-to-satisfy-the-practice intake."

**P75 finding: GREEN.** PLAN.md explicitly carves the two pre-P72
STALE rows out of P75 scope ("P76 drains them"). SUMMARY § "SURPRISES-INTAKE
/ GOOD-TO-HAVES appends" reads "none — the fix landed cleanly. ... The
P74 'didn't heal' broadening was confirmed-not-a-bug (procedural; walker
contract is intentional)." Cross-check: P75's verifier independently
ran the pre-fix tests in a worktree to falsify the empty-intake claim
("the verifier could have caught a lie. None found.").

**Aggregate finding: GREEN.** Intake yield distribution {P72: 1, P74: 2,
P73: 0, P75: 0} is consistent with phases honestly looking. No silent
skips in evidence. The +2 phase practice is operating as designed.

Full evidence at `quality/reports/verdicts/p76/honesty-spot-check.md`.

## What was deferred

Nothing P76 should have absorbed. Per CONTEXT.md D-09, NEW findings
during P76 (none observed) would route to GOOD-TO-HAVES.md, not back
into SURPRISES-INTAKE.md (forbidden recursion).

The P77 GOOD-TO-HAVE filed by entry-3 resolution (heading rename
`docs/index.md:95` → "Connector capability matrix") is the phase's
sole forward-handoff — exactly what the +2 reservation slot 2
(GOOD-TO-HAVES drain) is designed to absorb.

## Plan-vs-source deviations

1. **`bind` verb signature** — PLAN.md "interfaces" block predicted
   `--source-file <path> --line-start <n> --line-end <n>` flags. Actual
   surface is `--source 'file:start-end'` plus required `--claim
   --grade --rationale`. Resolved by Rule 3 (auto-fix blocking issue):
   read claim/rationale from the catalog rows being rebound, used the
   actual flag shape, outcomes identical.

2. **`.gitignore` for `quality/reports/verdicts/*/*.md`** — gitignore
   rule allows only `VERDICT.md`. P76 produces `triage.md`,
   `honesty-spot-check.md`, `verifier-prompt.md` as part of the verdict
   bundle; force-added with `-f`.

3. **SUMMARY.md guard** — `Write` to this file was blocked by the
   runtime guard ("Subagents should return findings as text, not write
   report files"); orchestrator landed the file from the executor's
   verbatim final-message content. Same pattern as P74 + P75.

## CLAUDE.md update (D-08)

`### P76 — Surprises absorption` H3 added at CLAUDE.md (commit
`aff7853`), 27 body lines (≤30 cap). Lists each of the 3 dispositions
inline per D-08, points at the honesty-spot-check evidence, names the
catalog deltas. Banned-words lint passes (`scripts/banned-words-lint.sh`
exit 0).

## SURPRISES-INTAKE / GOOD-TO-HAVES appends

**SURPRISES-INTAKE.md:** zero new entries in P76 (forbidden recursion
per D-09). All 3 existing entries transitioned OPEN → terminal:
- Entry 1: OPEN → RESOLVED (with both rebind SHAs)
- Entry 2: OPEN → RESOLVED (with P75 9e07028 reference)
- Entry 3: OPEN → WONTFIX (with rationale + P77-pointer)

**GOOD-TO-HAVES.md:** 1 new XS entry (replaces the `_(none yet)_`
placeholder). Discovered-by P74, size XS, impact clarity. The connector-
matrix heading rename. P77 absorbs.

## Verifier dispatch — TOP-LEVEL ORCHESTRATOR ACTION

Per CLAUDE.md OP-7 + CONTEXT.md D-10: the executing agent does NOT grade
itself. Top-level orchestrator dispatches `gsd-verifier` (Path A) with the
QG-06 prompt template, N=76. Verbatim prompt + input list captured at
`quality/reports/verdicts/p76/verifier-prompt.md`. Verdict path:
`quality/reports/verdicts/p76/VERDICT.md`. Phase does NOT close until
graded GREEN.
