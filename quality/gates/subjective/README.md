# quality/gates/subjective/

Rubric definitions backing `quality/catalogs/subjective-rubrics.json` (3 seed rows; cross-dimension; `subagent-graded` kind). Verifier implementation lives at `.claude/skills/reposix-quality-review/` (P61 Waves C-E ship the skill scaffold + 3 rubric impls).

| Rubric ID | Implementation skill | Cadence | freshness_ttl |
|---|---|---|---|
| `subjective/cold-reader-hero-clarity` | `doc-clarity-review` (existing global skill at `$HOME/.claude/skills/doc-clarity-review/SKILL.md`) | pre-release | 30d |
| `subjective/install-positioning` | `reposix-quality-review` (P61 ships at `.claude/skills/reposix-quality-review/`) | pre-release | 30d |
| `subjective/headline-numbers-sanity` | `reposix-quality-review` | weekly | 30d |

## Conventions

- **`subagent-graded` kind** = JSON artifact at `quality/reports/verifications/subjective/<rubric-id>.json` with shape `{ts, score, verdict, rationale, evidence_files}`. The runner re-grades a row PASS when `score >= 7`.
- **Numeric score 1-10.** Some rubrics map categorical CLEAR/NEEDS-WORK/CONFUSING to 10/5/2 internally; the catalog asserts cite the resulting numeric score so the threshold is uniform.
- **PASS threshold:** `score >= 7` unless a rubric overrides (none currently do).
- **Status semantics for missing artifact:** within freshness window -> NOT-VERIFIED; outside freshness window -> STALE (counts as NOT-VERIFIED but flagged in the verdict). P61 Wave B extends the runner with the freshness check.
- **Parallel dispatch.** The skill dispatches one subagent per stale row IN PARALLEL per OP-2 aggressive subagent delegation. Catalog rows are independent; the cargo-memory-budget rule does NOT apply (subjective rubrics are pure-prose Claude reads, no cargo).
- **`doc-clarity-review` integration.** The cold-reader rubric's implementation IS `doc-clarity-review`. The reposix-quality-review skill calls `doc-clarity-review` with the rubric's prompt + the source files (lines 1-50 of README.md + docs/index.md) and parses the CLEAR/NEEDS-WORK/CONFUSING verdict into the numeric score.
- **Cross-dimension namespace.** `subjective` is not one of the 8 official dimensions in CLAUDE.md "Quality Gates -- dimension/cadence/kind taxonomy"; it is a cross-dimension namespace per `quality/catalogs/README.md`. The 8-dimension list stays as-is; subjective is the rubric-shared catalog file.

## Pivot rules

- **Subagent score variance > 2pp run-to-run** on the same artifact: re-grade twice; flag PARTIAL if delta > tolerance. Reference: `.planning/research/v0.12.0/open-questions-and-deferrals.md` Q2.
- **Rubric instability** (5+ run swings): rewrite the rubric format from holistic-score to assertion-checklist (deterministic per-assert pass/fail).
- **P0/P1 finding flagged by rubric in Wave G**: fix in-phase per the broaden-and-deepen directive (61-07-PLAN.md). Scope creep beyond Wave G: file v0.12.1 carry-forward in MIGRATE-03 + waive the row with documented expiry.

## Cross-references

- `quality/PROTOCOL.md` -- runtime contract; do NOT duplicate runtime detail here.
- `quality/catalogs/subjective-rubrics.json` -- 3-row catalog (P61 Wave A).
- `CLAUDE.md` § "Cold-reader pass on user-facing surfaces" -- positioning rationale.
- `$HOME/.claude/skills/doc-clarity-review/SKILL.md` -- the existing skill that implements the cold-reader rubric.
