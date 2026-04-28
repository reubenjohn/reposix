//! `reposix-quality doc-alignment <verb>` -- 9 verbs.
//!
//! Verb shapes are byte-equivalent to the skill prompts at
//! `.claude/skills/reposix-quality-doc-alignment/prompts/{extractor,grader}.md`.
//!
//! NOTE (P64-02 Commit A): every verb is wired into the dispatch tree so
//! the `--help` smoke test passes. Subcommand bodies marked `todo:` are
//! filled in Commit B.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use clap::Subcommand;

/// All 9 docs-alignment verbs.
#[derive(Debug, Subcommand)]
pub enum Verb {
    /// Bind a row: validate citations, compute hashes, persist with `last_verdict=BOUND`.
    Bind {
        #[arg(long = "row-id")]
        row_id: String,
        #[arg(long)]
        claim: String,
        /// Source citation: `<file>:<line_start>-<line_end>`.
        #[arg(long)]
        source: String,
        /// Test citation: `<file>::<fn>`.
        #[arg(long)]
        test: String,
        /// Grade verdict (must be `GREEN`).
        #[arg(long)]
        grade: String,
        #[arg(long)]
        rationale: String,
    },

    /// Propose retiring a row: persist with `last_verdict=RETIRE_PROPOSED`.
    #[command(name = "propose-retire")]
    ProposeRetire {
        #[arg(long = "row-id")]
        row_id: String,
        #[arg(long)]
        claim: String,
        #[arg(long)]
        source: String,
        #[arg(long)]
        rationale: String,
    },

    /// Confirm a retirement -- HUMAN ONLY (env-guarded).
    #[command(name = "confirm-retire")]
    ConfirmRetire {
        #[arg(long = "row-id")]
        row_id: String,
    },

    /// Mark a row's binding as missing/misaligned (`last_verdict=MISSING_TEST`).
    #[command(name = "mark-missing-test")]
    MarkMissingTest {
        #[arg(long = "row-id")]
        row_id: String,
        #[arg(long)]
        claim: String,
        #[arg(long)]
        source: String,
        #[arg(long)]
        rationale: Option<String>,
    },

    /// Emit stale-row JSON manifest for one doc to stdout.
    #[command(name = "plan-refresh")]
    PlanRefresh {
        /// Doc file whose stale rows should be planned.
        doc_file: PathBuf,
    },

    /// Write a deterministic backfill MANIFEST.json under quality/reports/doc-alignment/.
    #[command(name = "plan-backfill")]
    PlanBackfill,

    /// Merge per-shard JSONs from a backfill run-dir into the catalog.
    #[command(name = "merge-shards")]
    MergeShards {
        /// Run directory with `shards/*.json` produced by the backfill subagents.
        run_dir: PathBuf,
    },

    /// Hash drift walker -- updates `last_verdict` only.
    Walk,

    /// Print summary block (table by default; --json for machine-readable).
    Status {
        #[arg(long)]
        json: bool,
    },
}

/// Dispatch a doc-alignment verb. Returns the process exit code on success.
pub fn dispatch(catalog: &Path, verb: Verb) -> Result<i32> {
    match verb {
        Verb::Bind {
            row_id,
            claim,
            source,
            test,
            grade,
            rationale,
        } => verbs::bind(catalog, &row_id, &claim, &source, &test, &grade, &rationale),
        Verb::ProposeRetire {
            row_id,
            claim,
            source,
            rationale,
        } => verbs::propose_retire(catalog, &row_id, &claim, &source, &rationale),
        Verb::ConfirmRetire { row_id } => verbs::confirm_retire(catalog, &row_id),
        Verb::MarkMissingTest {
            row_id,
            claim,
            source,
            rationale,
        } => verbs::mark_missing_test(catalog, &row_id, &claim, &source, rationale.as_deref()),
        Verb::PlanRefresh { doc_file } => verbs::plan_refresh(catalog, &doc_file),
        Verb::PlanBackfill => verbs::plan_backfill(catalog),
        Verb::MergeShards { run_dir } => verbs::merge_shards(catalog, &run_dir),
        Verb::Walk => verbs::walk(catalog),
        Verb::Status { json } => verbs::status(catalog, json),
    }
}

/// Read-only `verify --row-id <id>` -- prints row state to stdout.
pub fn verify(catalog: &Path, row_id: &str) -> Result<i32> {
    use crate::catalog::Catalog;
    let cat = Catalog::load(catalog)?;
    if let Some(r) = cat.row(row_id) {
        let body = serde_json::to_string_pretty(r)?;
        println!("{body}");
        Ok(0)
    } else {
        eprintln!(
            "verify: row-id `{row_id}` not found in {}",
            catalog.display()
        );
        Ok(1)
    }
}

/// Verbs implementation namespace. Bodies live here.
pub(crate) mod verbs {
    use std::path::Path;

    use anyhow::Result;

    pub fn bind(
        _catalog: &Path,
        _row_id: &str,
        _claim: &str,
        _source: &str,
        _test: &str,
        _grade: &str,
        _rationale: &str,
    ) -> Result<i32> {
        Err(anyhow::anyhow!("bind: not implemented (P64-02 Commit B)"))
    }

    pub fn propose_retire(
        _catalog: &Path,
        _row_id: &str,
        _claim: &str,
        _source: &str,
        _rationale: &str,
    ) -> Result<i32> {
        Err(anyhow::anyhow!(
            "propose-retire: not implemented (P64-02 Commit B)"
        ))
    }

    pub fn confirm_retire(_catalog: &Path, _row_id: &str) -> Result<i32> {
        Err(anyhow::anyhow!(
            "confirm-retire: not implemented (P64-02 Commit B)"
        ))
    }

    pub fn mark_missing_test(
        _catalog: &Path,
        _row_id: &str,
        _claim: &str,
        _source: &str,
        _rationale: Option<&str>,
    ) -> Result<i32> {
        Err(anyhow::anyhow!(
            "mark-missing-test: not implemented (P64-02 Commit B)"
        ))
    }

    pub fn plan_refresh(_catalog: &Path, _doc_file: &Path) -> Result<i32> {
        Err(anyhow::anyhow!(
            "plan-refresh: not implemented (P64-02 Commit B)"
        ))
    }

    pub fn plan_backfill(_catalog: &Path) -> Result<i32> {
        Err(anyhow::anyhow!(
            "plan-backfill: not implemented (P64-02 Commit B)"
        ))
    }

    pub fn merge_shards(_catalog: &Path, _run_dir: &Path) -> Result<i32> {
        Err(anyhow::anyhow!(
            "merge-shards: not implemented (P64-02 Commit B)"
        ))
    }

    pub fn walk(_catalog: &Path) -> Result<i32> {
        Err(anyhow::anyhow!("walk: not implemented (P64-02 Commit B)"))
    }

    pub fn status(_catalog: &Path, _json: bool) -> Result<i32> {
        Err(anyhow::anyhow!("status: not implemented (P64-02 Commit B)"))
    }
}

/// Parse a source citation `<file>:<line_start>-<line_end>` into its parts.
///
/// Public to the crate so `verbs` and tests can share it.
#[allow(
    dead_code,
    reason = "Verb bodies wired in P64-02 Commit B consume this helper."
)]
pub(crate) fn parse_source(s: &str) -> Result<(PathBuf, usize, usize)> {
    let (file, range) = s
        .rsplit_once(':')
        .ok_or_else(|| anyhow!("source `{s}` is not `<file>:<line_start>-<line_end>`"))?;
    let (lstart, lend) = range
        .split_once('-')
        .ok_or_else(|| anyhow!("source range `{range}` is not `<line_start>-<line_end>`"))?;
    let lstart: usize = lstart
        .parse()
        .map_err(|_| anyhow!("line_start `{lstart}` is not a positive integer"))?;
    let lend: usize = lend
        .parse()
        .map_err(|_| anyhow!("line_end `{lend}` is not a positive integer"))?;
    Ok((PathBuf::from(file), lstart, lend))
}

/// Parse a test citation `<file>::<fn>` into its parts.
#[allow(
    dead_code,
    reason = "Verb bodies wired in P64-02 Commit B consume this helper."
)]
pub(crate) fn parse_test(s: &str) -> Result<(PathBuf, String)> {
    let (file, fn_name) = s
        .rsplit_once("::")
        .ok_or_else(|| anyhow!("test `{s}` is not `<file>::<fn>`"))?;
    if fn_name.is_empty() {
        return Err(anyhow!("test fn name is empty in `{s}`"));
    }
    Ok((PathBuf::from(file), fn_name.to_string()))
}

#[cfg(test)]
mod parse_tests {
    use super::*;

    #[test]
    fn source_parses() {
        let (f, a, b) = parse_source("docs/index.md:10-12").unwrap();
        assert_eq!(f.to_string_lossy(), "docs/index.md");
        assert_eq!((a, b), (10, 12));
    }

    #[test]
    fn source_rejects_bad_shape() {
        assert!(parse_source("foo").is_err());
        assert!(parse_source("foo:10").is_err());
        assert!(parse_source("foo:abc-def").is_err());
    }

    #[test]
    fn test_parses() {
        let (f, fn_name) = parse_test("tests/foo.rs::bar").unwrap();
        assert_eq!(f.to_string_lossy(), "tests/foo.rs");
        assert_eq!(fn_name, "bar");
    }

    #[test]
    fn test_rejects_bad_shape() {
        assert!(parse_test("foo.rs").is_err());
        assert!(parse_test("foo.rs::").is_err());
    }
}
