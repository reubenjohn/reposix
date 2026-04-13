//! `reposix demo` orchestration — Task 2 fills this in.
//!
//! Stub for Task 1 so the subcommand is reachable and the `--help` test
//! passes. Task 2 replaces this with the full sim → mount → scripted
//! ls/cat/grep → audit tail flow, with `Guard`-based cleanup and
//! `tokio::signal::ctrl_c` handling.

use anyhow::Result;

/// Run the demo. Task 1 stub — bails; Task 2 provides the real body.
///
/// # Errors
/// Always returns an error until Task 2 lands.
#[allow(clippy::unused_async)] // Task 2 makes this async in earnest.
pub async fn run(_keep_running: bool) -> Result<()> {
    anyhow::bail!("demo orchestration lands in plan 03-02 Task 2")
}
