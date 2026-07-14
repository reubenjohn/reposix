← [back to index](./index.md)

# Task 03-T01 — Add `source_hashes: Vec<String>` to `Row` + load-time backfill

<read_first>
- `crates/reposix-quality/src/catalog.rs` (entire file — 418 lines; covers
  `Source`, `Row`, `Catalog::load`, parallel-array invariant docstring).
- `crates/reposix-quality/tests/walk.rs:1-69` (test setup: `tempdir`,
  `Catalog::load` smoke patterns).
</read_first>

<action>
Edit `crates/reposix-quality/src/catalog.rs`. In the `Row` struct (around
line 126-165), add immediately AFTER the `source_hash` field:

```rust
    /// Per-source hashes parallel to `source.as_slice()` (path-b per
    /// MULTI-SOURCE-WATCH-01 / P78). `source_hashes[i]` is the
    /// `hash::source_hash` of `source.as_slice()[i]`'s byte range.
    /// Empty vec means "no hashes recorded yet" (matches the empty-tests
    /// semantic of `tests` / `test_body_hashes`).
    ///
    /// **Parallel-array invariant** (P78): `source.as_slice().len() ==
    /// source_hashes.len()` post-load (after `Catalog::load`'s
    /// one-time backfill from the legacy `source_hash` field).
    /// Mutation must go through [`Row::set_source`] to preserve the
    /// invariant; readers may rely on it.
    ///
    /// Back-compat: legacy `source_hash` field stays on the struct for
    /// one release cycle. Newer catalogs may have BOTH fields; readers
    /// MUST prefer `source_hashes` post-backfill. Writers SHOULD update
    /// both during the transition (so a downgrade rollback can still
    /// load the catalog).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_hashes: Vec<String>,
```

Update the parallel-array invariant docstring on `Row` (lines 120-124) to
mention the new `source_hashes` invariant alongside the existing `tests` /
`test_body_hashes` invariant.

Add a new method on `Row` (near `set_tests` at lines 175-185):

```rust
    /// Set `source` and `source_hashes` together. Validates the
    /// parallel-array invariant; rejects mismatched lengths so writer
    /// paths cannot accidentally store an inconsistent row.
    ///
    /// # Errors
    ///
    /// Errors if `source.as_slice().len() != hashes.len()`.
    pub fn set_source(&mut self, source: Source, hashes: Vec<String>) -> Result<()> {
        if source.as_slice().len() != hashes.len() {
            return Err(anyhow::anyhow!(
                "Row::set_source: source.as_slice().len ({}) != source_hashes.len ({})",
                source.as_slice().len(),
                hashes.len(),
            ));
        }
        self.source = source;
        self.source_hashes = hashes;
        // Back-compat: keep source_hash in sync with the first element
        // for one release cycle. Drop after v0.14.0 once the field is
        // unused.
        self.source_hash = self.source_hashes.first().cloned();
        Ok(())
    }
```

Find `Catalog::load` (search `pub fn load` in catalog.rs). Immediately after
deserialization succeeds and BEFORE returning, run the one-time backfill:

```rust
    // P78 MULTI-SOURCE-WATCH-01 backfill: legacy catalogs have
    // `source_hash: Option<String>` and lack `source_hashes`. Promote
    // `source_hash` into `source_hashes[0]` so every read path enters
    // the new world. Idempotent: if `source_hashes` is already
    // populated (newer catalog), the backfill is a no-op.
    for row in &mut cat.rows {
        if row.source_hashes.is_empty() {
            if let Some(ref legacy) = row.source_hash {
                row.source_hashes.push(legacy.clone());
            }
            // else: row had no source_hash recorded; leave source_hashes
            // empty (no hash recorded yet — matches existing semantic).
        }
    }
```

If `Catalog::load` is in a different file (not catalog.rs), grep
`pub fn load` across `crates/reposix-quality/src/`. Most likely catalog.rs;
adjust as needed.

Do NOT delete `source_hash`. Do NOT change its `#[serde(skip_serializing_if =
"Option::is_none")]` attribute. Back-compat for a release cycle.

Run `cargo check -p reposix-quality` to confirm the struct + method
compiles. Use `-p reposix-quality` (per-crate) per CLAUDE.md "Build memory
budget" — workspace check runs in T05.
</action>

<acceptance_criteria>
- `grep -n "pub source_hashes" crates/reposix-quality/src/catalog.rs` matches once.
- `grep -n "fn set_source" crates/reposix-quality/src/catalog.rs` matches once.
- `grep -n "MULTI-SOURCE-WATCH-01 backfill" crates/reposix-quality/src/catalog.rs` matches once.
- `cargo check -p reposix-quality` exits 0.
- `cargo clippy -p reposix-quality -- -D warnings` exits 0 (the new field's serde attrs + docstring satisfy missing-doc + clippy::pedantic per CLAUDE.md crate prelude).
</acceptance_criteria>
