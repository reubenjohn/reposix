← [back to index](./index.md)

# Task 03-T02 — Migrate `verbs::walk` to AND-compare per-source hashes

<read_first>
- `crates/reposix-quality/src/commands/doc_alignment.rs:820-1000` (the `walk`
  fn — the source-drift loop is at lines 854-868).
- `crates/reposix-quality/src/commands/doc_alignment.rs:870-879`
  (the existing `drifted_indices` pattern for tests — mirror this for
  source).
</read_first>

<action>
Edit `crates/reposix-quality/src/commands/doc_alignment.rs::walk`. Replace
the single-source drift check at lines 854-868 (`let source_drift: Option<bool>
= match (cite, row.source_hash.as_ref()) { ... }`) with a per-index loop:

```rust
            // P78 MULTI-SOURCE-WATCH-01: AND-compare per-source hashes.
            // Walker iterates every cite in source.as_slice() and
            // compares against source_hashes[i]. Any-index drift fires
            // STALE_DOCS_DRIFT; the drifted index is recorded for the
            // diagnostic line.
            let cites = row.source.as_slice();
            let source_drift: Option<bool> = if row.source_hashes.is_empty() {
                None  // no hashes recorded yet (e.g. retire-proposed rows); skip
            } else if cites.len() != row.source_hashes.len() {
                // Parallel-array invariant violated; treat as drift.
                // This should never happen post-Catalog::load backfill,
                // but defend against hand-edited catalogs.
                Some(true)
            } else {
                let mut any_drift = false;
                let mut drifted_source_indices: Vec<usize> = Vec::new();
                for (i, cite) in cites.iter().enumerate() {
                    let stored = &row.source_hashes[i];
                    let p = PathBuf::from(&cite.file);
                    let drifted = if !p.exists() {
                        true
                    } else {
                        match hash::source_hash(&p, cite.line_start, cite.line_end) {
                            Ok(now) => &now != stored,
                            Err(_) => true,
                        }
                    };
                    if drifted {
                        any_drift = true;
                        drifted_source_indices.push(i);
                    }
                }
                // Surface drifted_source_indices in the blocking-line
                // diagnostic for forensic clarity (mirrors the test
                // drifted_indices pattern at lines 877-879). The first
                // drifted source's file path is the operator's
                // primary signal.
                if any_drift {
                    let first_idx = drifted_source_indices[0];
                    let first_file = &cites[first_idx].file;
                    blocking_lines.push(format!(
                        "docs-alignment: STALE_DOCS_DRIFT on source[{}] = {} (sources drifted: {:?}) -- run /reposix-quality-refresh {}",
                        first_idx,
                        first_file,
                        drifted_source_indices,
                        first_file,
                    ));
                }
                Some(any_drift)
            };
```

The verdict aggregation downstream at the original `source_drift` consumer
(grep `source_drift` in the same fn — likely a few lines below the original
match) needs no change: it still feeds the existing aggregation logic that
folds `source_drift`, test drift, and test-gone into the row's
`last_verdict`. The new variable carries the same `Option<bool>` shape so the
consumer remains compatible.

Important: make sure the `blocking_lines.push` for the new walker is
SCOPED — only push when `any_drift` is true (otherwise we'd surface every
walked row as a blocking line). Mirror the existing pattern carefully.

Per-crate compile: `cargo check -p reposix-quality`. Sequential (one cargo
at a time per CLAUDE.md).
</action>

<acceptance_criteria>
- `grep -n "MULTI-SOURCE-WATCH-01" crates/reposix-quality/src/commands/doc_alignment.rs` matches at least once (the new walker comment).
- `grep -n "drifted_source_indices" crates/reposix-quality/src/commands/doc_alignment.rs` matches.
- `grep -n "row.source_hash.as_ref()" crates/reposix-quality/src/commands/doc_alignment.rs` matches at most ONCE (the original single-source compare is gone from the walk fn; one match could remain if `source_hash` is still read for a back-compat check elsewhere, but the new walker MUST iterate `source_hashes`).
- `cargo check -p reposix-quality` exits 0.
- `cargo clippy -p reposix-quality -- -D warnings` exits 0.
</acceptance_criteria>
