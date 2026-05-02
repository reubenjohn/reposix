← [back to index](./index.md)

# Task 03-T04 — Author 5 regression tests in `walk.rs`

<read_first>
- `crates/reposix-quality/tests/walk.rs:1-200` (test setup conventions:
  `tempdir`, `Catalog::load`, fixture catalog construction).
- `crates/reposix-quality/tests/walk.rs:323-507` (existing
  `walk_multi_source_*` tests + `walk_single_source_rebind_heals_after_drift`).
</read_first>

<action>
Edit `crates/reposix-quality/tests/walk.rs`. The existing test names at
lines 323, 396, 447 OVERLAP with names in the must_haves list. Two paths:

**Path A: rename existing + add new.** Rename the pre-existing
`walk_multi_source_stable_no_false_drift` (line 323) →
`walk_multi_source_stable_no_false_drift_legacy_p75` (or similar) for clarity;
add a new `walk_multi_source_stable_no_false_drift` that exercises the
post-migration walker (it AND-compares per-source hashes).

**Path B: extend existing in place.** The existing tests likely already
exercise the post-migration walker as written; the path-(a) limitation was
that the walker didn't iterate every source on Multi rows, so the existing
tests probably verify "first source drifts → fires STALE" (which path-a
already did). Read each existing test (323, 396, 447) and decide:
- If the test exercises a path-(a) limitation case (e.g., asserts
  first-source drift), KEEP and ensure it still passes post-migration.
- If the test simulates a Multi row but only seeds first-source bytes,
  EXTEND it to seed both sources + verify both are walked.
- If the test name is occupied but its assertion is now obsolete (e.g.,
  asserts the path-(a) limitation as the expected behavior), REPLACE the
  body with the post-migration assertion.

The decision is per-test. Read each first; choose the minimal-diff path.

The LOAD-BEARING NEW TEST is `walk_multi_source_non_first_drift_fires_stale`.
This name does NOT exist pre-migration (search:
`grep walk_multi_source_non_first_drift crates/reposix-quality/tests/walk.rs`).
Add it as a new `#[test] fn`:

```rust
#[test]
fn walk_multi_source_non_first_drift_fires_stale() {
    // P78 MULTI-SOURCE-WATCH-01: prove the path-(a) false-negative
    // window is closed. Build a Multi row with 2 source citations
    // where the SECOND source's bytes drift post-bind. Walker must
    // fire STALE_DOCS_DRIFT and the diagnostic must name the second-
    // source index/file.
    let tmp = tempdir().expect("tmpdir");
    let repo_root = tmp.path();
    // Two source files, each 5 lines.
    let src_a = repo_root.join("src_a.md");
    let src_b = repo_root.join("src_b.md");
    fs::write(&src_a, "a-line-1\na-line-2\na-line-3\na-line-4\na-line-5\n")
        .expect("write src_a");
    fs::write(&src_b, "b-line-1\nb-line-2\nb-line-3\nb-line-4\nb-line-5\n")
        .expect("write src_b");

    // Hand-roll a catalog with one Multi row citing both files.
    // Use reposix-quality's Catalog/Row types (or hand-rolled JSON
    // depending on what existing tests use).
    let catalog_path = repo_root.join("catalog.json");
    let hash_a = hash::source_hash(&src_a, 1, 5).expect("hash src_a");
    let hash_b = hash::source_hash(&src_b, 1, 5).expect("hash src_b");
    let catalog_json = serde_json::json!({
        "schema_version": "1.0",
        "summary": { /* fill the 9 required keys with sensible defaults */ },
        "rows": [
            {
                "id": "test/multi-non-first-drift",
                "claim": "Two-source row exercising P78 walker AND-compare",
                "source": [
                    { "file": src_a.to_string_lossy(), "line_start": 1, "line_end": 5 },
                    { "file": src_b.to_string_lossy(), "line_start": 1, "line_end": 5 },
                ],
                "source_hash": hash_a, // legacy back-compat
                "source_hashes": [hash_a, hash_b],
                "tests": ["fake/dummy.rs::dummy"],
                "test_body_hashes": ["00".repeat(32)], // placeholder
                "last_verdict": "BOUND",
                "next_action": "BIND_GREEN",
            }
        ]
    });
    fs::write(&catalog_path, serde_json::to_string_pretty(&catalog_json).unwrap())
        .expect("write catalog");

    // Drift the SECOND source only.
    fs::write(&src_b, "DRIFTED-line-1\nb-line-2\nb-line-3\nb-line-4\nb-line-5\n")
        .expect("rewrite src_b");

    // Run walker; expect non-zero exit.
    let exit = verbs::walk(&catalog_path).expect("walk runs");
    assert_ne!(exit, 0, "walker must fire STALE_DOCS_DRIFT on non-first source drift");

    // Stronger assertion: the catalog post-walk has last_verdict
    // STALE_DOCS_DRIFT for the row.
    let cat = Catalog::load(&catalog_path).expect("reload catalog");
    let row = cat.rows.iter().find(|r| r.id == "test/multi-non-first-drift").unwrap();
    assert_eq!(row.last_verdict.as_str(), "STALE_DOCS_DRIFT", "row verdict updated");
}
```

The `summary` block in the test fixture needs all 9 required keys per
`structure/doc-alignment-summary-block-valid` (claims_total, claims_bound,
claims_missing_test, claims_retire_proposed, claims_retired,
alignment_ratio, floor, trend_30d, last_walked). If the existing tests use
a `tests::common` helper or a `Catalog::default()` shortcut, reuse it.

Add the other 4 tests (or extend existing ones per Path B above):
- `walk_multi_source_first_drift_fires_stale` (regression of pre-existing).
- `walk_multi_source_stable_no_false_drift` (post-migration).
- `walk_legacy_catalog_backfills_source_hash_to_source_hashes` —
  hand-rolled JSON with `source_hash` only (no `source_hashes` field);
  load via `Catalog::load`; assert
  `cat.rows[0].source_hashes == vec![cat.rows[0].source_hash.unwrap()]`.
- `bind_multi_same_source_rebind_refreshes_just_that_index` — bind
  twice; verify the rebound index updated; the sibling didn't.

The 5 tests can live in `walk.rs` (existing `walk_multi_source_*` neighbors)
OR in `bind_validation.rs` (which has a similar shape — see the test file
list in canonical_refs). Place per the existing convention; don't introduce a
new test file unless needed.

Per-crate test run: `cargo nextest run -p reposix-quality walk_multi_source`
(filter by name pattern). Faster feedback than full workspace run.
</action>

<acceptance_criteria>
- `grep -n "fn walk_multi_source_non_first_drift_fires_stale" crates/reposix-quality/tests/walk.rs` matches once.
- `grep -n "fn walk_multi_source_first_drift_fires_stale\|fn walk_multi_source_stable_no_false_drift\|fn walk_legacy_catalog_backfills_source_hash_to_source_hashes\|fn bind_multi_same_source_rebind_refreshes_just_that_index" crates/reposix-quality/tests/walk.rs` returns >= 4 matches (some may already exist; verify each name resolves).
- `cargo nextest run -p reposix-quality walk_multi_source_non_first_drift_fires_stale` exits 0.
- `cargo nextest run -p reposix-quality walk_multi_source` exits 0 (all multi-source tests pass).
- `cargo nextest run -p reposix-quality bind_multi_same_source_rebind` exits 0.
- The non-first-drift test would FAIL against the pre-migration walker (sanity: temporarily revert T02 and re-run; the test must fire). DO NOT actually do this revert in the final commit; this is a thought-experiment check.
</acceptance_criteria>
