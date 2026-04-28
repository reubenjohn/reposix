//! `reposix-quality` umbrella binary: dimension verifier suite for the
//! Quality Gates framework.
//!
//! Subcommand surface defined by:
//!   `.planning/research/v0.12.0-docs-alignment-design/02-architecture.md`
//!   `.claude/skills/reposix-quality-doc-alignment/prompts/extractor.md`
//!   `.claude/skills/reposix-quality-doc-alignment/prompts/grader.md`
//!
//! Verb shapes are byte-equivalent to the skill prompts -- subagents
//! shell out to this binary; the contract is normative.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::needless_pass_by_value)]

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use reposix_quality::commands::{doc_alignment, run as run_cmd};

/// Default catalog path (used when `--catalog` is not provided).
const DEFAULT_CATALOG: &str = "quality/catalogs/doc-alignment.json";

/// reposix-quality -- dimension verifier suite for the Quality Gates framework.
#[derive(Debug, Parser)]
#[command(name = "reposix-quality", version, about, subcommand_required = true)]
struct Cli {
    /// Path to the docs-alignment catalog (override for tests).
    #[arg(long, global = true, default_value = DEFAULT_CATALOG)]
    catalog: PathBuf,

    #[command(subcommand)]
    cmd: TopCmd,
}

#[derive(Debug, Subcommand)]
enum TopCmd {
    /// Docs-alignment dimension verbs.
    #[command(name = "doc-alignment", subcommand_required = true)]
    DocAlignment {
        #[command(subcommand)]
        verb: doc_alignment::Verb,
    },

    /// Hash drift walker (alias for `doc-alignment walk`).
    Walk,

    /// Read-only inspection of a single catalog row.
    Verify {
        #[arg(long = "row-id")]
        row_id: String,
    },

    /// Cadence-driven invocation -- mutually exclusive with --gate.
    Run(run_cmd::RunArgs),
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let catalog = cli.catalog;

    let result: anyhow::Result<i32> = match cli.cmd {
        TopCmd::DocAlignment { verb } => doc_alignment::dispatch(&catalog, verb),
        TopCmd::Walk => doc_alignment::dispatch(&catalog, doc_alignment::Verb::Walk),
        TopCmd::Verify { row_id } => doc_alignment::verify(&catalog, &row_id),
        TopCmd::Run(args) => run_cmd::run(args),
    };

    match result {
        Ok(code) => {
            #[allow(clippy::cast_sign_loss)]
            let raw = u8::try_from(code.clamp(0, 255)).unwrap_or(1);
            ExitCode::from(raw)
        }
        Err(e) => {
            eprintln!("reposix-quality: {e:#}");
            ExitCode::from(1)
        }
    }
}
