← [back to index](./index.md)

# Task 03-T05 — Catalog-first row mint via `reposix-quality bind` + workspace gates

<read_first>
- `quality/PROTOCOL.md` § "Principle A — Subagents propose with citations; tools validate and mint".
- `crates/reposix-quality/src/main.rs` (entire — confirms the `bind` CLI verb invocation form).
- `quality/catalogs/doc-alignment.json` — the catalog target.
</read_first>

<action>
**Catalog-first ordering:** the catalog row tracking
MULTI-SOURCE-WATCH-01 lands BEFORE the schema-migration commits per
`quality/PROTOCOL.md` Principle A. But the `bind` verb requires the test it
binds to ALREADY EXISTS (it hashes the test fn body via
`crates/reposix-quality/src/bin/hash_test_fn.rs`). Resolution: this T05
runs AFTER T01-T04 land (the test exists post-T04), but the `bind` call
+ the catalog row addition + the CLAUDE.md update + the workspace gates +
the push all happen in ONE TERMINAL COMMIT for this plan. The
"catalog-first" constraint is satisfied at the **plan boundary**: this
plan's first impl commit will include the catalog row mint as part of the
same atomic series.

Strategy: stage the catalog row mint via `bind` as a separate bash step,
verify the catalog file now has the row, then stage the migration commits
together. Final commit-graph for this plan:
- Commit N: schema migration (T01) + walker migration (T02) + bind/merge
  migration (T03) + tests (T04) + catalog row + CLAUDE.md update.

Single commit is OK because the catalog row IS the spec of the GREEN
contract; it must NOT land before the test exists (the test fn body hash
is part of the bind invocation). The atomic single-commit form satisfies
both Principle A (verifier exists at commit time) and the parallel-array
invariant (catalog row's `tests` array binds to a real fn).

```bash
# Build the binary first (once; serial cargo per CLAUDE.md).
cargo build -p reposix-quality --release

# Mint the row via bind. Cite the CARRY-FORWARD.md lines as the source.
./target/release/reposix-quality bind \
  --catalog quality/catalogs/doc-alignment.json \
  --row-id "doc-alignment/multi-source-watch-01-non-first-drift" \
  --claim "Walker hashes every source citation in a Source::Multi row; row enters STALE_DOCS_DRIFT on any index drift (path-b per CARRY-FORWARD.md MULTI-SOURCE-WATCH-01)" \
  --source ".planning/milestones/v0.13.0-phases/CARRY-FORWARD.md:19-35" \
  --test "crates/reposix-quality/tests/walk.rs::walk_multi_source_non_first_drift_fires_stale" \
  --grade GREEN \
  --rationale "P78-03 closes the v0.12.1 P75 carry-forward false-negative window before P85 DVCS docs add multi-source rows."
```

If `bind` rejects with "test does not exist" or "source range does not exist",
the test name or line range is wrong — diagnose, fix, re-bind. If `bind`
succeeds, `quality/catalogs/doc-alignment.json` now has the new row +
recomputed summary block.

Run workspace gates SERIALLY:

```bash
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace
```

(Per-crate fallback if memory pressure manifests, per CLAUDE.md "Build
memory budget".)

If any test outside `reposix-quality` regresses (unlikely — the schema
migration is local to `reposix-quality`), the failure surfaces here.
Diagnose, fix, re-run.

Run the walker against the live catalog as a smoke (no migration drift):

```bash
./target/release/reposix-quality walk --catalog quality/catalogs/doc-alignment.json
```

This MUST exit 0. The 388-row catalog post-backfill carries
`source_hashes` for every row; the walker AND-compares per-source hashes;
no row should drift if the catalog was clean before the migration.

If a row drifts unexpectedly, diagnose. The most likely cause is a row
where `Source::Multi` had a stale non-first hash (i.e., the path-(a)
false-negative was hiding actual drift). This is the CORRECT behavior of
the migration — drift surfaces; fix the row via
`/reposix-quality-refresh <doc>` per CLAUDE.md slash-command hint. If
multiple rows surface drift, append to SURPRISES-INTAKE.md (severity
LOW: "P78-03 walker migration surfaced N rows with non-first-source
drift; refreshing inline / deferring to P87"). Choose Eager-resolution
if < 5 rows; defer to P87 if >5.

CLAUDE.md update — edit § "v0.12.1 — in flight" → "P75 — bind-verb
hash-overwrite fix" → "Path-(a) tradeoff" paragraph. Replace:

> Path (b) (parallel `source_hashes: Vec<String>` + per-source walker
> compare) is filed as v0.13.0 carry-forward `MULTI-SOURCE-WATCH-01`.

with:

> Path (b) — parallel `source_hashes: Vec<String>` + per-source walker
> AND-compare — closed in v0.13.0 P78-03 (commit `<SHA>`); non-first-source
> drift now fires `STALE_DOCS_DRIFT` per the regression test
> `crates/reposix-quality/tests/walk.rs::walk_multi_source_non_first_drift_fires_stale`.

The commit SHA gets filled in AFTER `git commit`; substitute via a sed
edit on the staged file or a follow-up edit on a post-commit pass. (A
common pattern: commit with placeholder `<P78-03 commit>`, then a
follow-up amending edit. But CLAUDE.md "git safety: NEVER amend" applies.
Instead, commit with the placeholder, then a SECOND commit "docs: cite
P78-03 SHA in CLAUDE.md" with the actual SHA. Two commits total; clean.)
</action>

<acceptance_criteria>
- `quality/catalogs/doc-alignment.json` contains a row with id `doc-alignment/multi-source-watch-01-non-first-drift` and `last_verdict: BOUND`.
- The summary block re-computed (`claims_total` incremented; `alignment_ratio` recomputed within epsilon).
- `cargo check --workspace` exits 0.
- `cargo clippy --workspace --all-targets -- -D warnings` exits 0.
- `cargo nextest run --workspace` exits 0.
- `./target/release/reposix-quality walk --catalog quality/catalogs/doc-alignment.json` exits 0 (or the few drifted rows are documented in SURPRISES-INTAKE.md).
- CLAUDE.md § "v0.12.1 — in flight" → "P75" paragraph cites P78-03 commit SHA.
- All cargo runs were SERIAL (no parallel cargo per CLAUDE.md).
</acceptance_criteria>
