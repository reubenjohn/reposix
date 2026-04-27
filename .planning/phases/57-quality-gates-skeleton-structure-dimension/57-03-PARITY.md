# 57-03-PARITY — old vs new chain verdicts

Written at the close of P57 Wave C per `.planning/research/v0.12.0-decisions-log.md` D4 (parallel-then-cut migration). Required reading for Wave F's verifier subagent.

## Old chain (pre-Wave-C `scripts/end-state.py verdict`)

Rows in scope at pre-push: **23** (6 freshness + 8 mermaid + 8 crates-io + 1 ci-status). Last GREEN verdict baseline at `.planning/SESSION-END-STATE-VERDICT.md` (pre-P57).

After Wave C's shim shrink, `scripts/end-state.py verdict` no longer invokes the old `bootstrap_claims()` — it delegates to `python3 quality/runners/verdict.py session-end`. The mermaid×8, crates-io×8, ci-status×1 rows fall out of pre-push grading via the shim. Compensating coverage: existing pre-push fatal blocks (cargo fmt + cargo clippy + check-docs-site + check-mermaid-renders) remain unchanged.

## New chain (`python3 quality/runners/run.py --cadence pre-push`)

Rows in scope at pre-push: **9** (6 freshness + 1 banned-words + 1 QG-08 + 1 BADGE-01 (waived)).

First Wave-C verdict (steady state):

```
[PASS] structure/install-leads-with-pkg-mgr-docs-index  (P0)
[PASS] structure/install-leads-with-pkg-mgr-readme  (P0)
[PASS] structure/no-version-pinned-filenames  (P1)
[PASS] structure/benchmarks-in-mkdocs-nav  (P1)
[PASS] structure/no-loose-roadmap-or-requirements  (P1)
[PASS] structure/no-orphan-docs  (P1)
[FAIL] structure/top-level-requirements-roadmap-scope  (P1)  <-- QG-08; Wave D fixes
[PASS] structure/banned-words  (P1)
[WAIVED] structure/badges-resolve  (P2)
summary: 7 PASS, 1 FAIL, 0 PARTIAL, 1 WAIVED, 0 NOT-VERIFIED -> exit=1
```

## Per-row parity (the 6 shared freshness rows)

| row_id | old chain | new chain | match? |
|---|---|---|---|
| structure/no-version-pinned-filenames | PASS | PASS | yes |
| structure/install-leads-with-pkg-mgr-docs-index | PASS | PASS | yes |
| structure/install-leads-with-pkg-mgr-readme | PASS | PASS | yes |
| structure/benchmarks-in-mkdocs-nav | PASS | PASS | yes |
| structure/no-loose-roadmap-or-requirements | PASS | PASS | yes |
| structure/no-orphan-docs | PASS | PASS | yes |

All 6 shared rows PASS in both chains. No divergence; no SURPRISES.md entry needed for parity.

## Coverage delta (new vs old)

**Added by new chain:**
- `structure/banned-words` — was an unrelated pre-push fatal block; now a P1 catalog row (PASS)
- `structure/top-level-requirements-roadmap-scope` (QG-08) — NEW; expected RED today (Wave D fixes)
- `structure/badges-resolve` (BADGE-01 P57 stub) — WAIVED until 2026-07-25; verifier ships P60

**Dropped from new chain pre-push grading** (until P58/P60 catalog them):
- mermaid×8 → migrate to docs-build dim P60
- crates-io×8 → migrate to release dim P58
- ci-status×1 → migrate to code dim P58

**Compensating coverage** (the dropped rows' regression classes are still gated by other pre-push fatal blocks): cargo fmt + cargo clippy + check-docs-site (mkdocs --strict) + check-mermaid-renders (playwright artifact check) — all unchanged in the pre-push hook. The new runner does NOT remove these fatals; it adds itself as a non-fatal warning alongside.

## Decision

Parallel migration acceptable. Per D4, the old shim + new runner co-exist for 2 pre-push cycles before any further migration. Wave D's POLISH-STRUCT addresses the QG-08 RED. P58 catalogs the crates-io + ci-status rows; P60 catalogs mermaid + does the SIMPLIFY-08/09/10 hard-cut.

## SURPRISES.md entries

No parity divergence found in this wave; no SURPRISES.md entry needed.

A separate runner-idempotency issue surfaced during Wave B that did get resolved in commit `dd458bd` (Rule 1 auto-fix for catalog mutation noise on every pre-push run). That fix landed in Wave B and is not a Wave-C parity concern.
