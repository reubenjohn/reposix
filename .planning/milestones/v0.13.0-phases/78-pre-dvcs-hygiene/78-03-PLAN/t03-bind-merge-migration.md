← [back to index](./index.md)

# Task 03-T03 — Migrate `verbs::bind` + `merge-shards` to write `source_hashes`

<read_first>
- `crates/reposix-quality/src/commands/doc_alignment.rs:224-349` (`bind` fn —
  4 code paths to update).
- `crates/reposix-quality/src/commands/doc_alignment.rs:296-316` (P75
  rationale comment — update post-migration to cite P78-03).
- `crates/reposix-quality/src/commands/doc_alignment.rs:455-510` (alt `bind`
  surface; second `src_hash` computation site).
- `crates/reposix-quality/src/commands/doc_alignment.rs:770-816`
  (`merge-shards` source builder).
</read_first>

<action>
**Bind path 1 — new row** (around line 330-345):

The current code:
```rust
            let mut new_row = Row {
                id: row_id.to_string(),
                claim: claim.to_string(),
                source: Source::Single(new_source),
                source_hash: Some(src_hash),
                tests: Vec::new(),
                test_body_hashes: Vec::new(),
                ...
```

After: clone `src_hash` so both fields are populated. Use `set_source` for
invariant validation:

```rust
            let mut new_row = Row {
                id: row_id.to_string(),
                claim: claim.to_string(),
                source: Source::Single(new_source.clone()),
                source_hash: Some(src_hash.clone()),
                source_hashes: vec![src_hash.clone()],
                tests: Vec::new(),
                test_body_hashes: Vec::new(),
                ...
```

(Or call `new_row.set_source(Source::Single(new_source), vec![src_hash])`
AFTER struct init — choose whichever keeps the diff minimal; the inline
struct-literal form is simpler.)

**Bind path 2 — existing row, Single result** (around lines 311-321):

Current:
```rust
            let result_is_single = sources.len() == 1;
            row.source = if result_is_single {
                Source::Single(sources.into_iter().next().expect("len==1"))
            } else {
                Source::Multi(sources)
            };
            row.claim = claim.to_string();
            if result_is_single {
                row.source_hash = Some(src_hash);
            }
            // else: preserve existing row.source_hash (first-source invariant).
```

After (path-b is in effect; preserve P75's first-source invariant for
`source_hash` legacy compat, but ALSO update `source_hashes`):

```rust
            // P75 + P78 MULTI-SOURCE-WATCH-01: maintain BOTH the legacy
            // single-hash field (back-compat) and the parallel-array
            // source_hashes (the new invariant). The walker reads
            // source_hashes; downgrade rollback reads source_hash.
            let result_is_single = sources.len() == 1;
            // Find the matching index BEFORE we move `sources`.
            let new_index = sources
                .iter()
                .position(|c| {
                    c.file == new_source.file
                        && c.line_start == new_source.line_start
                        && c.line_end == new_source.line_end
                })
                .expect("new_source must appear in sources after the append/heal logic");
            // Rebuild source_hashes parallel to sources. Reuse existing
            // entries where the cite is unchanged; insert/overwrite at
            // new_index for the freshly-bound source.
            let prior_hashes = std::mem::take(&mut row.source_hashes);
            let prior_cites = row.source.as_slice();
            let mut new_hashes: Vec<String> = Vec::with_capacity(sources.len());
            for (i, c) in sources.iter().enumerate() {
                if i == new_index {
                    new_hashes.push(src_hash.clone());
                } else if let Some(prior_idx) = prior_cites.iter().position(|p| {
                    p.file == c.file && p.line_start == c.line_start && p.line_end == c.line_end
                }) {
                    // Carry forward the prior hash for unchanged cites.
                    new_hashes.push(
                        prior_hashes
                            .get(prior_idx)
                            .cloned()
                            .unwrap_or_else(|| src_hash.clone()),
                    );
                } else {
                    // Cite never seen before; this branch shouldn't fire
                    // under the current bind algorithm (sources is built
                    // from the prior source.as_slice() + the new one)
                    // but defending against future shape changes.
                    new_hashes.push(src_hash.clone());
                }
            }
            row.source = if result_is_single {
                Source::Single(sources.into_iter().next().expect("len==1"))
            } else {
                Source::Multi(sources)
            };
            row.source_hashes = new_hashes;
            row.claim = claim.to_string();
            // Back-compat: maintain source_hash as hash(first source) per
            // P75 first-source-invariant for legacy readers.
            row.source_hash = row.source_hashes.first().cloned();
```

This handles all three previously-distinct paths in one branch:
- Single result: `sources.len() == 1`, `new_index == 0`, walk fills 1 hash.
- Multi append (new source): `new_index == sources.len() - 1`, walk
  carries prior hashes forward + appends `src_hash` at the end.
- Multi same-source rebind: `new_index < sources.len() - 1` (re-binding
  an existing cite — the `already_present` short-circuit at line 294
  prevents the append, so the existing-cite branch is the relevant
  one; the prior-cite-position lookup finds it; the if-i-==-new_index
  branch overrides the carried-forward hash with the fresh one).

The P75 same-source-rebind heal logic stays intact (the earlier comment
at 296-316 still describes the rationale; UPDATE that comment to cite
P78-03 — see end of T03).

**Bind path 3 — alt site at line 455-510:** read those lines first; the
shape is similar (`src_hash` computed, then row mutation). Apply the same
pattern: write to BOTH `source_hash` AND `source_hashes`. If the alt site
turns out to be a different verb (e.g., a `rebind` or `force-bind`), the
same parallel-array discipline applies.

**`merge-shards` (lines 770-816):**

The current code builds `all_sources: Vec<SourceCite>` by deduplicating
shard prototypes' source citations (lines 770-794), then assigns:
```rust
                prototype.source = if all_sources.len() == 1 {
                    Source::Single(...)
                } else {
                    Source::Multi(all_sources)
                };
```

After: compute `all_source_hashes: Vec<String>` parallel to `all_sources`
by hashing each cite via `hash::source_hash(&PathBuf::from(&cite.file),
cite.line_start, cite.line_end)`. Then:
```rust
                prototype.source_hashes = all_source_hashes;
                prototype.source_hash = prototype.source_hashes.first().cloned();
                prototype.source = if all_sources.len() == 1 {
                    Source::Single(all_sources.into_iter().next().expect("len==1"))
                } else {
                    Source::Multi(all_sources)
                };
```

If a cite's file is missing during merge-shards hashing, surface a clear
error (the merge would otherwise produce a row with a stale or invalid
hash; better to fail early). Use `with_context(|| format!("merge-shards:
hashing cite {} for row {}", cite.file, prototype.id))` to attach
diagnostics.

**Update the P75 path-(a) tradeoff comment** at lines 296-316 (the
inline `// BIND-VERB-FIX-01 (P75)` paragraph). After this migration, the
comment should say:

```rust
            // BIND-VERB-FIX-01 (P75) + MULTI-SOURCE-WATCH-01 (P78):
            // Walker now AND-compares per-source hashes via
            // `source_hashes` (path-b — closed in P78-03). The legacy
            // `source_hash` field tracks `source_hashes[0]` for one
            // release cycle (back-compat for downgrade rollback);
            // post-v0.14.0 `source_hash` can be retired.
            //
            // Heal paths:
            //   - Single result: refresh source_hashes[0] AND source_hash
            //     (heals Single rows whose prose drifted).
            //   - Multi append: push freshly-bound hash; preserve prior
            //     index hashes (Multi rows accumulate; new source =
            //     new index).
            //   - Multi same-source rebind: refresh JUST that index in
            //     source_hashes (heals individual Multi entries without
            //     disturbing siblings).
```

Per-crate compile: `cargo check -p reposix-quality`.
</action>

<acceptance_criteria>
- `grep -n "row.source_hashes" crates/reposix-quality/src/commands/doc_alignment.rs` matches in BOTH bind path AND walk path AND merge-shards path (≥3 distinct sites).
- `grep -n "MULTI-SOURCE-WATCH-01" crates/reposix-quality/src/commands/doc_alignment.rs` matches at least 3 times (one at each major migration site: walker, bind, merge-shards) with explanatory comments.
- The P75 path-(a) tradeoff sentence at line ~307-310 is gone (search `Path (b) -- walker hashes every source from a Multi -- is filed as` → must NOT match).
- `cargo check -p reposix-quality` exits 0.
- `cargo clippy -p reposix-quality -- -D warnings` exits 0.
</acceptance_criteria>
