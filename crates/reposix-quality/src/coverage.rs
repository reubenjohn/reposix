//! Coverage metric: how thoroughly the extractor mined the prose.
//!
//! Distinct from alignment (claim -> test). A row contributes to coverage
//! regardless of `last_verdict` -- even `MISSING_TEST` and `RETIRE_PROPOSED`
//! rows count as "covered prose" for this metric. The two axes together yield
//! the agent's mental model:
//!
//! |                  | high alignment                      | low alignment                            |
//! |------------------|-------------------------------------|------------------------------------------|
//! | high coverage    | ideal                                | extracted everything; most claims unbound |
//! | low coverage     | tested what we found; missed prose   | haven't started                          |
//!
//! Source-of-truth: `quality/catalogs/README.md` § "docs-alignment dimension"
//! and the P66 plan at `.planning/phases/66-coverage-ratio/66-01-PLAN.md`.

use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::catalog::Row;

/// Per-file coverage record. Sorted ascending by `ratio` in
/// [`compute_per_file`] so the worst-covered files surface first -- this is
/// the agent's gap-target view.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PerFileCoverage {
    pub path: PathBuf,
    pub total_lines: u64,
    pub covered_lines: u64,
    pub ratio: f64,
    pub row_count: u64,
}

/// Eligible file set = the coverage DENOMINATOR (which files' lines count and
/// which citations are "in bounds"). A row whose `source.file` is NOT in this
/// set emits a `cites out-of-eligible file` warning at attribution time
/// ([`compute_per_file`]).
///
/// SHARED BASE with `collect_backfill_inputs` in `commands/doc_alignment.rs`
/// (the backfill miner's input universe): `docs/**/*.md`, `README.md`, and
/// archived milestone `REQUIREMENTS.md` (v0.6.0 -- v0.15.0, extended from
/// v0.11.0 in DRAIN-21). Keep that base in LOCKSTEP in both functions.
///
/// SUPERSET (coverage-only, DRAIN-21): this set ADDITIONALLY includes specific
/// doc-of-record surfaces that individual BOUND rows cite -- a handful of prose
/// files (`benchmarks/README.md` + three archived `.planning/` docs) plus the
/// two SOURCE files the connector-protocol / CLI-surface rows verify against
/// (`crates/reposix-core/src/backend.rs`, `crates/reposix-cli/src/main.rs`).
/// These are NARROW, explicit paths -- NEVER a glob. A blanket `crates/**/*.rs`
/// would pull hundreds of un-rowed source lines into the denominator, tank the
/// coverage ratio below its floor (docs-alignment gate RED), and flip this
/// metric from doc-coverage into code-coverage. The two source files stay OUT
/// of `collect_backfill_inputs` because the backfill miner extracts PROSE
/// claims, never Rust source -- the one intentional divergence between the two
/// lists (they are hand-maintained parallel copies; a DRY refactor is a tracked
/// GOOD-TO-HAVE, deliberately not done here).
///
/// Files absent on disk are skipped silently (warnings fire only for a row
/// whose citation names a file outside this set).
#[must_use]
pub fn eligible_files() -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();

    if Path::new("README.md").exists() {
        out.push(PathBuf::from("README.md"));
    }

    if Path::new("docs").is_dir() {
        let mut acc = Vec::new();
        if walk_md(Path::new("docs"), &mut acc).is_ok() {
            out.extend(acc.into_iter().map(PathBuf::from));
        }
    }

    // Archived milestone REQUIREMENTS. Cutoff extended v0.11.0 -> v0.15.0
    // (DRAIN-21): closed milestones accrued since the original range; keeping
    // it current keeps their archived requirement docs in the denominator.
    // v0.15.0 is the active milestone whose file lands here only at close
    // (no-op until then). Kept in LOCKSTEP with `collect_backfill_inputs`.
    for v in &[
        "v0.6.0", "v0.7.0", "v0.8.0", "v0.9.0", "v0.10.0", "v0.11.0", "v0.12.0", "v0.13.0",
        "v0.14.0", "v0.15.0",
    ] {
        let p = format!(".planning/milestones/{v}-phases/REQUIREMENTS.md");
        if Path::new(&p).exists() {
            out.push(PathBuf::from(p));
        }
    }

    // DRAIN-21 SUPERSET (coverage-only): specific PROSE doc-of-record surfaces
    // that BOUND rows cite but the base globs above miss. NARROW explicit paths
    // (see the fn doc comment). These are legitimate prose and could also be
    // mined, but are kept coverage-only here to avoid widening the backfill
    // universe as a side effect of a coverage-warning fix.
    for p in &[
        "benchmarks/README.md",
        ".planning/milestones/v0.13.0-phases/CARRY-FORWARD.md",
        ".planning/research/v0.13.0-dvcs/architecture-sketch/innovations.md",
        ".planning/milestones/v0.8.0-phases/ARCHIVE.md",
    ] {
        if Path::new(p).exists() {
            out.push(PathBuf::from(*p));
        }
    }

    // DRAIN-21 SUPERSET (coverage-only, DELIBERATELY absent from the backfill
    // miner): the two SOURCE files that 11 already-BOUND rows verify doc claims
    // against -- the BackendConnector protocol and the CLI subcommand surface.
    // NEVER broaden to a `crates/**/*.rs` glob (see the fn doc comment).
    for p in &[
        "crates/reposix-core/src/backend.rs",
        "crates/reposix-cli/src/main.rs",
    ] {
        if Path::new(p).exists() {
            out.push(PathBuf::from(*p));
        }
    }

    out.sort();
    out.dedup();
    out
}

/// Recursive markdown collector for the docs tree.
fn walk_md(dir: &Path, out: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("reading dir {}", dir.display()))? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            walk_md(&p, out)?;
        } else if p.extension().and_then(|e| e.to_str()) == Some("md") {
            out.push(p.to_string_lossy().to_string());
        }
    }
    Ok(())
}

/// Count `\n`-terminated lines in a file. UTF-8 safe via `BufReader`.
///
/// # Errors
/// Propagates `io::Error` from open / read.
pub fn line_count(path: &Path) -> Result<u64> {
    let f = fs::File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let reader = BufReader::new(f);
    let mut n: u64 = 0;
    for line in reader.lines() {
        line.with_context(|| format!("reading line in {}", path.display()))?;
        n += 1;
    }
    Ok(n)
}

/// Merge overlapping AND adjacent inclusive 1-based ranges.
///
/// Adjacent example: `[(5, 10), (11, 15)]` -> `[(5, 15)]` (the gap of 0 lines
/// counts as adjacent for line-coverage purposes; if you cited lines 5-10 and
/// then 11-15, you've covered the contiguous block 5-15).
///
/// Overlapping example: `[(5, 10), (8, 15)]` -> `[(5, 15)]`.
///
/// Disjoint example: `[(5, 10), (20, 25)]` -> `[(5, 10), (20, 25)]`.
#[must_use]
pub fn merge_ranges(ranges: &[(usize, usize)]) -> Vec<(usize, usize)> {
    if ranges.is_empty() {
        return Vec::new();
    }
    let mut sorted: Vec<(usize, usize)> = ranges.to_vec();
    // Normalize swapped pairs (line_end < line_start would be a bug; clamp).
    for r in &mut sorted {
        if r.0 > r.1 {
            std::mem::swap(&mut r.0, &mut r.1);
        }
    }
    sorted.sort_by_key(|r| (r.0, r.1));

    let mut out: Vec<(usize, usize)> = Vec::with_capacity(sorted.len());
    let mut cur = sorted[0];
    for &next in &sorted[1..] {
        // Adjacent: next.start == cur.end + 1; overlapping: next.start <= cur.end.
        // Both fold into cur via the `<= cur.end + 1` test (saturating to avoid
        // overflow at usize::MAX).
        if next.0 <= cur.1.saturating_add(1) {
            if next.1 > cur.1 {
                cur.1 = next.1;
            }
        } else {
            out.push(cur);
            cur = next;
        }
    }
    out.push(cur);
    out
}

/// Sum of merged covered lines for a single file across `rows`.
///
/// Each row contributes every `SourceCite` whose `file == path`; multi-source
/// rows attribute to each cited file independently (a row that cites file A
/// AND file B contributes its A-range only here when called with `path == A`).
///
/// Out-of-bounds ranges (`line_end` > file's line count) clamp to the actual
/// line count. Caller is responsible for emitting the warning -- this fn just
/// computes.
#[must_use]
pub fn covered_lines_for_file(rows: &[Row], path: &Path) -> u64 {
    let path_str = path.to_string_lossy();
    let total = line_count(path).unwrap_or_default();
    let mut ranges: Vec<(usize, usize)> = Vec::new();
    for r in rows {
        for cite in r.source.as_slice() {
            if cite.file == path_str {
                let start = cite.line_start.max(1);
                let mut end = cite.line_end;
                if total > 0 {
                    let total_us = usize::try_from(total).unwrap_or(usize::MAX);
                    if end > total_us {
                        end = total_us;
                    }
                }
                if end >= start {
                    ranges.push((start, end));
                }
            }
        }
    }
    let merged = merge_ranges(&ranges);
    let mut sum: u64 = 0;
    for (s, e) in merged {
        // (e - s + 1) inclusive count; saturate to avoid panics on bad input.
        let span = u64::try_from(e.saturating_sub(s).saturating_add(1)).unwrap_or(0);
        sum = sum.saturating_add(span);
    }
    sum
}

/// Compute per-file coverage records for every eligible file.
///
/// Side effect: emits a stderr warning for each row whose `source.file` cite
/// is OUTSIDE the eligible set (forensic signal -- file moved/renamed). The
/// warning message is `coverage: row {id} cites out-of-eligible file {path}`.
///
/// Output is sorted ascending by `ratio` so worst-covered files surface first
/// -- the agent's gap-target view (`status --top 10` reads this list).
#[must_use]
pub fn compute_per_file(rows: &[Row]) -> Vec<PerFileCoverage> {
    let files = eligible_files();
    let eligible_set: std::collections::HashSet<String> = files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();

    // Out-of-eligible warnings (deduped by (row_id, file) pair).
    let mut warned: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();
    for r in rows {
        for cite in r.source.as_slice() {
            if !eligible_set.contains(&cite.file) {
                let key = (r.id.clone(), cite.file.clone());
                if warned.insert(key) {
                    eprintln!(
                        "coverage: row {} cites out-of-eligible file {}",
                        r.id, cite.file,
                    );
                }
            }
        }
    }

    // Per-file row count: count rows whose source has at least one cite for this file.
    let mut row_counts: BTreeMap<String, u64> = BTreeMap::new();
    for r in rows {
        let mut hit_files: std::collections::HashSet<String> = std::collections::HashSet::new();
        for cite in r.source.as_slice() {
            if eligible_set.contains(&cite.file) {
                hit_files.insert(cite.file.clone());
            }
        }
        for f in hit_files {
            *row_counts.entry(f).or_insert(0) += 1;
        }
    }

    let mut per: Vec<PerFileCoverage> = files
        .into_iter()
        .map(|p| {
            let total = line_count(&p).unwrap_or(0);
            let covered = covered_lines_for_file(rows, &p);
            #[allow(clippy::cast_precision_loss)]
            let ratio = if total == 0 {
                0.0
            } else {
                (covered as f64) / (total as f64)
            };
            let key = p.to_string_lossy().into_owned();
            let row_count = row_counts.get(&key).copied().unwrap_or(0);
            PerFileCoverage {
                path: p,
                total_lines: total,
                covered_lines: covered,
                ratio,
                row_count,
            }
        })
        .collect();

    per.sort_by(|a, b| {
        a.ratio
            .partial_cmp(&b.ratio)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.path.cmp(&b.path))
    });
    per
}

/// Compute global `(lines_covered, total_eligible_lines, coverage_ratio)`.
///
/// Empty input -> `(0, 0, 0.0)`. Total==0 -> ratio 0.0 (NOT 1.0; differs from
/// `alignment_ratio`'s empty-state semantics by design -- you can't claim full
/// coverage when there's nothing to cover).
#[must_use]
pub fn compute_global(per_file: &[PerFileCoverage]) -> (u64, u64, f64) {
    let mut covered: u64 = 0;
    let mut total: u64 = 0;
    for p in per_file {
        covered = covered.saturating_add(p.covered_lines);
        total = total.saturating_add(p.total_lines);
    }
    #[allow(clippy::cast_precision_loss)]
    let ratio = if total == 0 {
        0.0
    } else {
        (covered as f64) / (total as f64)
    };
    (covered, total, ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_ranges_empty() {
        assert_eq!(merge_ranges(&[]), Vec::<(usize, usize)>::new());
    }

    #[test]
    fn merge_ranges_single() {
        assert_eq!(merge_ranges(&[(5, 10)]), vec![(5, 10)]);
    }

    #[test]
    fn merge_ranges_overlapping() {
        // [(5, 10), (8, 15), (20, 22)] -> [(5, 15), (20, 22)]
        assert_eq!(
            merge_ranges(&[(5, 10), (8, 15), (20, 22)]),
            vec![(5, 15), (20, 22)],
        );
    }

    #[test]
    fn merge_ranges_adjacent() {
        // [(5, 10), (11, 15)] -> [(5, 15)] (adjacent fold)
        assert_eq!(merge_ranges(&[(5, 10), (11, 15)]), vec![(5, 15)]);
    }

    #[test]
    fn merge_ranges_disjoint() {
        // [(5, 10), (20, 25)] -> unchanged
        assert_eq!(merge_ranges(&[(5, 10), (20, 25)]), vec![(5, 10), (20, 25)]);
    }

    #[test]
    fn merge_ranges_unsorted_input() {
        // Sorts internally before fold.
        assert_eq!(
            merge_ranges(&[(20, 25), (5, 10), (8, 15)]),
            vec![(5, 15), (20, 25)],
        );
    }

    #[test]
    fn merge_ranges_full_overlap() {
        // [(5, 20), (8, 12)] -> [(5, 20)]
        assert_eq!(merge_ranges(&[(5, 20), (8, 12)]), vec![(5, 20)]);
    }

    #[test]
    fn compute_global_empty() {
        assert_eq!(compute_global(&[]), (0, 0, 0.0));
    }
}
