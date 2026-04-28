//! `reposix-quality run` -- cadence-driven invocation.
//!
//! Cadence mode shells to `python3 quality/runners/run.py --cadence <name>`.
//! Gate mode dispatches to the named gate's verifier (delegated to the
//! same Python runner via `--gate <name>`).

use std::process::Command;

use anyhow::{anyhow, Result};
use clap::Args;

#[derive(Debug, Args)]
#[command(group = clap::ArgGroup::new("target").required(true).args(["gate", "cadence"]))]
pub struct RunArgs {
    /// Run a single named gate (mutually exclusive with --cadence).
    #[arg(long, group = "target")]
    pub gate: Option<String>,

    /// Run every gate at this cadence (mutually exclusive with --gate).
    #[arg(long, group = "target")]
    pub cadence: Option<String>,
}

/// Dispatch to the Python runner (the umbrella does not reimplement
/// orchestration; it delegates to `quality/runners/run.py`).
pub fn run(args: RunArgs) -> Result<i32> {
    let mut cmd = Command::new("python3");
    cmd.arg("quality/runners/run.py");

    if let Some(c) = args.cadence.as_deref() {
        cmd.arg("--cadence").arg(c);
    } else if let Some(g) = args.gate.as_deref() {
        cmd.arg("--gate").arg(g);
    } else {
        return Err(anyhow!("run: one of --gate or --cadence is required"));
    }

    let status = cmd
        .status()
        .map_err(|e| anyhow!("invoking python3 quality/runners/run.py failed: {e}"))?;
    Ok(status.code().unwrap_or(1))
}
