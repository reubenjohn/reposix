//! `git-remote-reposix` — git remote helper.
//!
//! Invoked by git when a remote URL begins with `reposix::`. Speaks the git remote helper
//! protocol on stdin/stdout. Phase 4 fills in the real protocol; this skeleton only handles
//! the `capabilities` discovery line so `git remote -v` doesn't error out.

use std::io::{BufRead, Write};

use anyhow::Result;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .with_writer(std::io::stderr)
        .init();

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    for line in stdin.lock().lines() {
        let line = line?;
        match line.trim() {
            "capabilities" => {
                // Minimum capability set; phase 4 expands this.
                writeln!(out, "import")?;
                writeln!(out, "export")?;
                writeln!(out, "refspec refs/heads/*:refs/reposix/*")?;
                writeln!(out)?;
                out.flush()?;
            }
            "" => break,
            other => {
                tracing::warn!(command = %other, "git-remote-reposix: command not yet implemented");
                writeln!(out)?;
                out.flush()?;
            }
        }
    }
    Ok(())
}
