//! `reposix list` — dump issues for a project as JSON or a pretty table.
//!
//! v0.1.5 hard-codes the simulator backend (the only implementor of
//! [`IssueBackend`](reposix_core::IssueBackend) that ships in this version).
//! Once `crates/reposix-github` lands in v0.2, this subcommand will grow a
//! `--backend {sim,github}` flag and `run` will dispatch accordingly.
//!
//! Output formats:
//! - `json` (default): `serde_json::to_string_pretty(&issues)` — machine-readable,
//!   diffable, the format `parity.sh` pipes through `jq`.
//! - `table`: fixed-width columns `ID | STATUS | TITLE` for human reading.

use anyhow::{Context, Result};
use clap::ValueEnum;
use reposix_core::backend::sim::SimBackend;
use reposix_core::{Issue, IssueBackend};

/// Output formats accepted by `reposix list --format`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ListFormat {
    /// Pretty-printed JSON (the default — diffable and scriptable).
    Json,
    /// Human-readable table with fixed-width columns.
    Table,
}

/// Execute `reposix list`. Builds a [`SimBackend`] pointed at `origin`,
/// fetches issues for `project`, and writes formatted output to stdout.
///
/// # Errors
/// Returns a wrapped error if the backend cannot be constructed (e.g. a bad
/// `REPOSIX_ALLOWED_ORIGINS` env var), if the HTTP call fails (sim not
/// running, non-allowlisted origin), or if JSON serialization fails.
pub async fn run(project: String, origin: String, format: ListFormat) -> Result<()> {
    let backend = SimBackend::new(origin).context("build SimBackend")?;
    let issues = backend
        .list_issues(&project)
        .await
        .with_context(|| format!("list_issues project={project}"))?;
    match format {
        ListFormat::Json => {
            let pretty = serde_json::to_string_pretty(&issues).context("serialize json")?;
            println!("{pretty}");
        }
        ListFormat::Table => {
            render_table(&issues);
        }
    }
    Ok(())
}

fn render_table(issues: &[Issue]) {
    // Column widths: ID is never more than 10 digits for any realistic sim
    // seed; STATUS fits in 12 chars (`in_progress` is the longest). Title
    // takes whatever's left. Literal header text is inlined to avoid the
    // clippy::print_literal lint.
    let id_col = "ID";
    let status_col = "STATUS";
    println!("{id_col:<10} {status_col:<12} TITLE");
    let dash10 = "----------";
    let dash12 = "------------";
    let dash40 = "----------------------------------------";
    println!("{dash10} {dash12} {dash40}");
    for issue in issues {
        let id = issue.id.0;
        let status = issue.status.as_str();
        let title = &issue.title;
        println!("{id:<10} {status:<12} {title}");
    }
}
