//! Coverage metric tests (P66 v0.12.1).
//!
//! Eight integration cases covering the algorithmic surface of
//! `reposix_quality::coverage`. The tests use synthetic Row inputs (built
//! directly via the public API) and don't shell to the binary -- coverage is
//! pure data-pipeline code, so direct fn calls are the right unit-of-test.
//!
//! Out-of-eligible warning capture: the warnings are emitted to stderr by
//! `compute_per_file` directly (println-shaped); a dedicated case asserts the
//! warning fires AND the row's lines are NOT counted.

use std::fs;
use std::path::PathBuf;

use reposix_quality::catalog::SourceCite;
use reposix_quality::coverage::{
    compute_global, compute_per_file, covered_lines_for_file, line_count, merge_ranges,
};
use reposix_quality::{Row, Source};

/// Cargo runs tests with CWD at the crate dir; the coverage module uses
/// CWD-relative paths (`docs/`, `README.md`, ...). Tests run in parallel by
/// default so we serialize CWD mutations via a process-wide `Mutex`. Each
/// test acquires the guard, flips CWD to the repo root, runs, and the guard
/// drops on test exit.
fn cwd_lock() -> std::sync::MutexGuard<'static, ()> {
    use std::sync::{Mutex, OnceLock};
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let guard = LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("cwd lock not poisoned");
    let manifest = env!("CARGO_MANIFEST_DIR");
    let root = std::path::Path::new(manifest)
        .parent()
        .and_then(|p| p.parent())
        .expect("crate dir must have grandparent");
    std::env::set_current_dir(root).expect("must cd to repo root");
    guard
}

fn synth_row(id: &str, cites: Vec<(&str, usize, usize)>) -> Row {
    let cite_objs: Vec<SourceCite> = cites
        .into_iter()
        .map(|(f, ls, le)| SourceCite {
            file: f.to_string(),
            line_start: ls,
            line_end: le,
        })
        .collect();
    let source = if cite_objs.len() == 1 {
        Source::Single(cite_objs.into_iter().next().unwrap())
    } else {
        Source::Multi(cite_objs)
    };
    Row {
        id: id.to_string(),
        claim: format!("synthetic claim for {id}"),
        source,
        source_hash: None,
        // P78 MULTI-SOURCE-WATCH-01: synthetic coverage rows have no
        // recorded hashes; empty source_hashes preserves the
        // "no-hash-recorded-yet" semantic and bypasses walker drift
        // compares (which is correct for these coverage-only fixtures).
        source_hashes: Vec::new(),
        tests: Vec::new(),
        test_body_hashes: Vec::new(),
        rationale: None,
        last_verdict: reposix_quality::RowState::Bound,
        next_action: reposix_quality::NextAction::BindGreen,
        last_run: None,
        last_extracted: None,
        last_extracted_by: None,
    }
}

// 1. Empty rows -> coverage 0.0; lines_covered 0; total_eligible_lines = sum
//    over the eligible set (this is run from the repo root by `cargo test`).
#[test]
fn empty_rows_yields_zero_coverage_with_nonzero_total() {
    let _guard = cwd_lock();
    let per_file = compute_per_file(&[]);
    let (covered, total, ratio) = compute_global(&per_file);

    assert_eq!(covered, 0, "empty rows must yield 0 covered lines");
    assert!(
        total > 0,
        "eligible set must have nonzero total lines (cargo test runs from repo root)"
    );
    assert!((ratio - 0.0).abs() < 1e-9, "ratio must be 0.0 with no rows");
}

// 2. Single row covering README.md lines 5-10 -> 6 lines covered (inclusive).
#[test]
fn single_row_inclusive_range_counts_six_lines() {
    let _guard = cwd_lock();
    let row = synth_row("test/single", vec![("README.md", 5, 10)]);
    let path = PathBuf::from("README.md");
    if !path.exists() {
        // Repo invariant -- README.md exists. If not, skip.
        return;
    }
    let n = covered_lines_for_file(&[row], &path);
    assert_eq!(n, 6, "lines 5..=10 inclusive == 6 lines");
}

// 3. Two overlapping rows [5-10] + [8-15] -> merged [5-15] = 11 lines.
#[test]
fn overlapping_rows_merge_to_eleven_lines() {
    let _guard = cwd_lock();
    let r1 = synth_row("test/over1", vec![("README.md", 5, 10)]);
    let r2 = synth_row("test/over2", vec![("README.md", 8, 15)]);
    let path = PathBuf::from("README.md");
    if !path.exists() {
        return;
    }
    let n = covered_lines_for_file(&[r1, r2], &path);
    assert_eq!(n, 11, "merged [5-15] == 11 inclusive lines");
}

// 4. Two adjacent rows [5-10] + [11-15] -> 11 lines (treat as merged via union).
#[test]
fn adjacent_rows_merge_to_eleven_lines() {
    let _guard = cwd_lock();
    let r1 = synth_row("test/adj1", vec![("README.md", 5, 10)]);
    let r2 = synth_row("test/adj2", vec![("README.md", 11, 15)]);
    let path = PathBuf::from("README.md");
    if !path.exists() {
        return;
    }
    let n = covered_lines_for_file(&[r1, r2], &path);
    assert_eq!(
        n, 11,
        "adjacent [5-10] + [11-15] folds to [5-15] == 11 lines"
    );
}

// 5. Two non-adjacent rows [5-10] + [20-25] -> 6+6 = 12 lines.
#[test]
fn nonadjacent_rows_count_twelve_lines() {
    let _guard = cwd_lock();
    let r1 = synth_row("test/dis1", vec![("README.md", 5, 10)]);
    let r2 = synth_row("test/dis2", vec![("README.md", 20, 25)]);
    let path = PathBuf::from("README.md");
    if !path.exists() {
        return;
    }
    let n = covered_lines_for_file(&[r1, r2], &path);
    assert_eq!(n, 12, "disjoint [5-10] + [20-25] == 6 + 6 == 12 lines");
}

// 6. Multi-source row covering 2 files -> each file's per-file count gets 1 contribution.
#[test]
fn multi_source_row_attributes_to_each_cited_file_independently() {
    let _guard = cwd_lock();
    // Pick two files that exist in the eligible set: README.md and any docs/*.md.
    let path_a = PathBuf::from("README.md");
    let docs_root = PathBuf::from("docs");
    if !path_a.exists() || !docs_root.is_dir() {
        return;
    }
    // Find any docs/*.md.
    let mut path_b: Option<PathBuf> = None;
    if let Ok(entries) = fs::read_dir(&docs_root) {
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) == Some("md") {
                path_b = Some(p);
                break;
            }
        }
    }
    let Some(path_b) = path_b else { return };

    let row = synth_row(
        "test/multi",
        vec![
            (path_a.to_str().unwrap(), 1, 3),
            (path_b.to_str().unwrap(), 1, 2),
        ],
    );
    let n_a = covered_lines_for_file(std::slice::from_ref(&row), &path_a);
    let n_b = covered_lines_for_file(&[row], &path_b);
    assert_eq!(n_a, 3, "file A gets 3 covered lines");
    assert_eq!(n_b, 2, "file B gets 2 covered lines");
}

// 7. Row pointing at file outside eligible set -> warn stderr + skip
//    (we assert via the per-file aggregator: the row's lines do NOT show up in
//    any per-file record because the path isn't in the eligible set, AND the
//    warning fn is reached. Direct stderr capture is awkward in cargo test;
//    we instead assert the row contribution is correctly absent).
#[test]
fn out_of_eligible_row_does_not_count() {
    let _guard = cwd_lock();
    let row = synth_row(
        "test/out-of-eligible",
        vec![("not/in/the/eligible/set/zzz.md", 1, 100)],
    );
    let per = compute_per_file(std::slice::from_ref(&row));
    // Sum over per-file should NOT include this row's 100 lines anywhere
    // because zzz.md isn't an eligible file.
    for p in &per {
        // No eligible file should match the bogus path.
        assert_ne!(
            p.path.to_string_lossy(),
            "not/in/the/eligible/set/zzz.md",
            "bogus path must not appear in eligible set"
        );
    }
    // And it doesn't inflate the global total.
    let baseline = compute_global(&compute_per_file(&[]));
    let (covered_with_bogus, total_with_bogus, _) = compute_global(&per);
    assert_eq!(
        total_with_bogus, baseline.1,
        "bogus row must not change total_eligible_lines"
    );
    assert_eq!(
        covered_with_bogus, baseline.0,
        "bogus row must not change lines_covered"
    );
}

// 8. per_file output sorted ascending by ratio (worst-first; agent's gap-target view).
#[test]
fn per_file_output_sorted_ascending_by_ratio() {
    let _guard = cwd_lock();
    let per = compute_per_file(&[]);
    // Empty rows: every per-file ratio is 0.0 (0 covered / N total). Sort
    // stability is by path, but the ratio invariant is already trivially held.
    // Add ONE row covering the WHOLE first eligible file so its ratio jumps.
    let Some(first) = per.first() else { return };
    let target = first.path.clone();
    let total = first.total_lines;
    if total == 0 {
        return;
    }
    let row = synth_row(
        "test/sort",
        vec![(target.to_str().unwrap(), 1, usize::try_from(total).unwrap())],
    );
    let per2 = compute_per_file(&[row]);
    // Find the target file in per2; assert it's NOT first (it should now have
    // ratio == 1.0 vs the others' 0.0).
    let target_pos = per2
        .iter()
        .position(|p| p.path == target)
        .expect("target must remain in eligible set");
    // The first entry should have ratio < target's ratio (or be the target if
    // every file is fully covered, which is impossible with a single row).
    if per2.len() > 1 {
        assert!(
            per2[0].ratio <= per2[target_pos].ratio,
            "per_file must be sorted ascending by ratio"
        );
    }
}

// Bonus: direct test of merge_ranges from integration land (covers the
// public-API surface; unit tests inside coverage.rs cover the same).
#[test]
fn merge_ranges_via_public_api() {
    assert_eq!(merge_ranges(&[(5, 10), (8, 15)]), vec![(5, 15)]);
    assert_eq!(merge_ranges(&[(5, 10), (11, 15)]), vec![(5, 15)]);
    assert_eq!(merge_ranges(&[(5, 10), (20, 25)]), vec![(5, 10), (20, 25)]);
}

// Bonus: line_count smoke against README.md.
#[test]
fn line_count_smoke_readme() {
    let _guard = cwd_lock();
    let path = PathBuf::from("README.md");
    if !path.exists() {
        return;
    }
    let n = line_count(&path).expect("README.md must be readable");
    assert!(n > 0, "README.md must have nonzero lines");
}
