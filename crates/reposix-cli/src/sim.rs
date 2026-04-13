//! Child-process wrapper around the `reposix-sim` binary.

use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};

use crate::binpath::resolve_bin;

/// A running `reposix-sim` child. Dropping SIGTERMs (and SIGKILLs after
/// 2s) the child — no orphans.
#[derive(Debug)]
pub struct SimProcess {
    child: Child,
}

impl SimProcess {
    /// Spawn the sim on `bind`, writing audit rows to `db`. If `seed` is
    /// `Some`, the sim loads it at startup.
    ///
    /// # Errors
    /// Returns any [`std::io::Error`] from spawning the child process.
    pub fn spawn(bind: &str, db: &Path, seed: Option<&Path>) -> Result<Self> {
        let mut cmd = resolve_bin("reposix-sim");
        cmd.arg("--bind").arg(bind);
        cmd.arg("--db").arg(db);
        if let Some(s) = seed {
            cmd.arg("--seed-file").arg(s);
        }
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        let child = cmd.spawn().context("spawn reposix-sim")?;
        Ok(Self { child })
    }

    /// Spawn with `--ephemeral` (in-memory DB) — used by `reposix demo`
    /// when it wants a clean-slate sim per run.
    ///
    /// # Errors
    /// Same as [`SimProcess::spawn`].
    pub fn spawn_ephemeral(bind: &str, db: &Path, seed: Option<&Path>) -> Result<Self> {
        let mut cmd = resolve_bin("reposix-sim");
        cmd.arg("--bind").arg(bind);
        cmd.arg("--db").arg(db);
        cmd.arg("--ephemeral");
        if let Some(s) = seed {
            cmd.arg("--seed-file").arg(s);
        }
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        let child = cmd.spawn().context("spawn reposix-sim --ephemeral")?;
        Ok(Self { child })
    }

    /// Wait for the child to exit (blocking). Propagates its status.
    ///
    /// # Errors
    /// Returns any [`std::io::Error`] from `wait`.
    #[allow(dead_code)] // Public API — used by `reposix demo` Task 2 + external callers.
    pub fn wait(mut self) -> Result<std::process::ExitStatus> {
        let s = self.child.wait().context("wait reposix-sim")?;
        // Prevent Drop from re-reaping.
        std::mem::forget(self);
        Ok(s)
    }

    /// Send SIGTERM and reap within 2s; SIGKILL if the child ignores it.
    fn terminate(&mut self) {
        // `Child::kill` on Unix sends SIGKILL, which we want only as a
        // last resort. Use rustix's safe `kill_process` for SIGTERM first
        // so `tokio`/axum graceful-shutdown paths run in the child.
        let pid_raw = self.child.id();
        if let Ok(pid_i32) = i32::try_from(pid_raw) {
            if let Some(pid) = rustix::process::Pid::from_raw(pid_i32) {
                let _ = rustix::process::kill_process(pid, rustix::process::Signal::Term);
            }
        }
        let t0 = Instant::now();
        loop {
            match self.child.try_wait() {
                Ok(Some(_)) => return,
                Ok(None) => {
                    if t0.elapsed() >= Duration::from_secs(2) {
                        let _ = self.child.kill();
                        let _ = self.child.wait();
                        return;
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(_) => {
                    let _ = self.child.kill();
                    let _ = self.child.wait();
                    return;
                }
            }
        }
    }
}

impl Drop for SimProcess {
    fn drop(&mut self) {
        self.terminate();
    }
}

/// Inline wrapper: spawn, wait, propagate exit code. Used by
/// `reposix sim` (foreground).
///
/// # Errors
/// Returns any spawn error or a non-zero exit status from the child.
pub fn run(bind: &str, db: PathBuf, seed: Option<PathBuf>, ephemeral: bool) -> Result<()> {
    let mut sim = if ephemeral {
        SimProcess::spawn_ephemeral(bind, &db, seed.as_deref())?
    } else {
        SimProcess::spawn(bind, &db, seed.as_deref())?
    };
    // Propagate signals: block until child exits or we get Ctrl-C.
    let status = sim.child.wait().context("wait reposix-sim")?;
    std::mem::forget(sim);
    if !status.success() {
        anyhow::bail!("reposix-sim exited with {status}");
    }
    Ok(())
}
