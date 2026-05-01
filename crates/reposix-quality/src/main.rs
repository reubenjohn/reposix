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

use reposix_quality::commands::{agent_ux, doc_alignment, run as run_cmd};

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

    /// Mint or refresh a catalog row (Principle A entry point).
    ///
    /// `--dimension docs-alignment` (the default) routes to the existing
    /// `doc-alignment bind` verb shape -- use that subcommand directly.
    /// `--dimension agent-ux` mints a row in `quality/catalogs/agent-ux.json`
    /// (GOOD-TO-HAVES-01 Path A). Other dimensions error pending v0.14.0.
    Bind(BindArgs),
}

/// Top-level `bind` args. Per-dimension flag relevance:
///
/// - **agent-ux:** uses `--row-id`, `--verifier`, `--cadence`, `--kind`,
///   `--source` (>=1), `--blast-radius`, `--asserts` (>=0), and optionally
///   `--args`, `--timeout`, `--freshness-ttl`, `--owner-hint`.
/// - **docs-alignment:** today, route to `reposix-quality doc-alignment
///   bind` directly -- the top-level surface refuses with a message that
///   names the subcommand.
/// - **other:** v0.14.0 will extend per GOOD-TO-HAVES-01 Path B.
#[derive(Debug, clap::Args)]
struct BindArgs {
    /// Catalog dimension. Default `docs-alignment` (preserves existing
    /// behavior; routes to the `doc-alignment bind` subcommand surface).
    #[arg(long, default_value = "docs-alignment")]
    dimension: String,

    /// Catalog row id. For `--dimension agent-ux`, must start with
    /// `agent-ux/`. Other dimensions: see the dimension's per-row schema.
    #[arg(long = "row-id")]
    row_id: String,

    /// Path to the verifier script (agent-ux only).
    #[arg(long)]
    verifier: Option<PathBuf>,

    /// Cadence: one of `pre-push`, `pre-pr`, `weekly`, `pre-release`,
    /// `post-release`, `on-demand` (agent-ux only; runner enforces).
    #[arg(long)]
    cadence: Option<String>,

    /// Kind: one of `mechanical`, `container`, `asset-exists`,
    /// `subagent-graded`, `manual` (agent-ux only).
    #[arg(long)]
    kind: Option<String>,

    /// Source citation -- repeatable. Each is a path that must exist on the
    /// live filesystem (agent-ux only; >=1 required).
    #[arg(long, action = clap::ArgAction::Append)]
    source: Vec<String>,

    /// Verifier arg -- repeatable. Forwarded into `verifier.args[]`
    /// (agent-ux only).
    #[arg(long = "args", action = clap::ArgAction::Append)]
    verifier_args: Vec<String>,

    /// Verifier timeout in seconds (agent-ux only). Default 180s.
    #[arg(long, default_value_t = 180)]
    timeout: u64,

    /// Blast radius: one of `P0`, `P1`, `P2` (agent-ux only).
    #[arg(long = "blast-radius")]
    blast_radius: Option<String>,

    /// Freshness TTL (e.g. `30d`) for subagent-graded rows. Optional;
    /// stored as JSON null when omitted (agent-ux only).
    #[arg(long = "freshness-ttl")]
    freshness_ttl: Option<String>,

    /// Owner hint -- short text shown when the row goes RED (agent-ux only).
    #[arg(long = "owner-hint")]
    owner_hint: Option<String>,

    /// Behavioural assertion -- repeatable. Each becomes one element of
    /// `expected.asserts[]` (agent-ux only).
    #[arg(long = "asserts", action = clap::ArgAction::Append)]
    asserts: Vec<String>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let catalog = cli.catalog;

    let result: anyhow::Result<i32> = match cli.cmd {
        TopCmd::DocAlignment { verb } => doc_alignment::dispatch(&catalog, verb),
        TopCmd::Walk => doc_alignment::dispatch(&catalog, doc_alignment::Verb::Walk),
        TopCmd::Verify { row_id } => doc_alignment::verify(&catalog, &row_id),
        TopCmd::Run(args) => run_cmd::run(args),
        TopCmd::Bind(args) => dispatch_bind(&catalog, args),
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

/// Top-level `bind` dispatcher (GOOD-TO-HAVES-01 Path A).
fn dispatch_bind(default_catalog: &std::path::Path, args: BindArgs) -> anyhow::Result<i32> {
    match args.dimension.as_str() {
        "agent-ux" => {
            // If --catalog was left at the docs-alignment default, route to
            // the agent-ux dimension's default catalog. Otherwise honor the
            // explicit override (tests pass --catalog <tempdir>).
            let cat = if default_catalog == std::path::Path::new(DEFAULT_CATALOG) {
                agent_ux::default_catalog()
            } else {
                default_catalog.to_path_buf()
            };

            let verifier = args
                .verifier
                .ok_or_else(|| anyhow::anyhow!("bind --dimension agent-ux: --verifier <script> is required"))?;
            let cadence = args
                .cadence
                .ok_or_else(|| anyhow::anyhow!("bind --dimension agent-ux: --cadence <value> is required"))?;
            let kind = args
                .kind
                .ok_or_else(|| anyhow::anyhow!("bind --dimension agent-ux: --kind <value> is required"))?;
            let blast_radius = args
                .blast_radius
                .ok_or_else(|| anyhow::anyhow!("bind --dimension agent-ux: --blast-radius <P0|P1|P2> is required"))?;

            agent_ux::bind(
                &cat,
                &args.row_id,
                &verifier,
                &cadence,
                &kind,
                &args.verifier_args,
                args.timeout,
                &blast_radius,
                args.freshness_ttl.as_deref(),
                &args.source,
                args.owner_hint.as_deref(),
                &args.asserts,
            )
        }
        "docs-alignment" => Err(anyhow::anyhow!(
            "bind --dimension docs-alignment: top-level shortcut not implemented; run `reposix-quality doc-alignment bind ...` directly (the existing surface)."
        )),
        other => Err(anyhow::anyhow!(
            "bind --dimension `{other}`: not yet supported; v0.14.0 will extend (GOOD-TO-HAVES-01 Path B). Supported today: agent-ux."
        )),
    }
}
