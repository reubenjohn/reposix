# v0.13.0 carry-forward intake

Items deferred from prior milestones that v0.13.0 phases should pick up.
One H2 per item; cite the originating phase + requirement.

## MULTI-SOURCE-WATCH-01 — walker hashes every source from `Source::Multi`

**Source:** v0.12.1 P75 (`BIND-VERB-FIX-01`) shipped path (a) — preserve
first-source hash on `Source::Single → Source::Multi` promotion. Path (b)
(walker iterates every source citation in a `Multi` row, hashes each,
ANDs the results) was deferred to keep P75 single-phase.

**Why deferred:** path (b) requires a schema migration (`source_hash:
Option<String>` → `source_hashes: Vec<String>`) with a parallel-array
invariant on `Multi` rows, plus migration of the populated 388-row
catalog and a walker compare-loop refactor. Out of scope for a
single-phase fix.

**Acceptance:**

- `Row::source_hashes: Vec<String>` parallel-array to `source.as_slice()`.
- `verbs::walk` hashes each source citation against its corresponding
  `source_hashes[i]`; row enters `STALE_DOCS_DRIFT` on ANY index drift.
- `verbs::bind` writes/preserves all entries on the parallel array
  (Single result → 1-element vec; Multi append → push the new hash;
  Multi same-source rebind → refresh that index only).
- Existing single-source-hash field migrates via `serde(default)` +
  a one-time backfill (read `source_hash` if present, push it into
  `source_hashes[0]`).
- Regression tests in `crates/reposix-quality/tests/walk.rs` exercise
  the path-(b) "non-first source drift fires STALE" case.

**Carries from:** v0.12.1 phase 75 (`BIND-VERB-FIX-01`); see
`.planning/phases/75-bind-verb-hash-fix/PLAN.md` and
`.planning/phases/75-bind-verb-hash-fix/SUMMARY.md`.

**Owner:** unassigned. Pick up under v0.13.0 docs-alignment dimension.
