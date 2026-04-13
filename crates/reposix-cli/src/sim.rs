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

/// Optional knobs forwarded to the `reposix-sim` child process. All
/// fields default to "use the sim's default" (i.e. don't pass the flag).
///
/// Wraps `--no-seed` and `--rate-limit` so callers don't have to pass a
/// long argument list when they only care about defaults. The standalone
/// `--seed-file` and `--ephemeral` knobs stay positional in [`SimProcess::spawn`]
/// /[`SimProcess::spawn_ephemeral`] because every demo path sets them
/// explicitly.
#[derive(Debug, Default, Clone, Copy)]
pub struct SimOptions {
    /// If `true`, pass `--no-seed` to the child. Skips loading the seed
    /// JSON even if `seed_file` is `Some`.
    pub no_seed: bool,
    /// Per-agent rate limit (requests per second). `None` => omit the
    /// flag and let the child use its default (currently 100 rps).
    pub rate_limit: Option<u32>,
}

impl SimProcess {
    /// Spawn the sim on `bind`, writing audit rows to `db`. If `seed` is
    /// `Some`, the sim loads it at startup unless `opts.no_seed` is set.
    ///
    /// # Errors
    /// Returns any [`std::io::Error`] from spawning the child process.
    pub fn spawn(bind: &str, db: &Path, seed: Option<&Path>, opts: SimOptions) -> Result<Self> {
        let mut cmd = resolve_bin("reposix-sim");
        cmd.arg("--bind").arg(bind);
        cmd.arg("--db").arg(db);
        if let Some(s) = seed {
            cmd.arg("--seed-file").arg(s);
        }
        Self::apply_opts(&mut cmd, opts);
        // Discard stdin (unused) and stdout (no structured output).
        // Inherit stderr so axum errors surface in `reposix demo`.
        cmd.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit());
        let child = cmd.spawn().context("spawn reposix-sim")?;
        Ok(Self { child })
    }

    /// Spawn with `--ephemeral` (in-memory DB) — used by `reposix demo`
    /// when it wants a clean-slate sim per run.
    ///
    /// # Errors
    /// Same as [`SimProcess::spawn`].
    pub fn spawn_ephemeral(
        bind: &str,
        db: &Path,
        seed: Option<&Path>,
        opts: SimOptions,
    ) -> Result<Self> {
        let mut cmd = resolve_bin("reposix-sim");
        cmd.arg("--bind").arg(bind);
        cmd.arg("--db").arg(db);
        cmd.arg("--ephemeral");
        if let Some(s) = seed {
            cmd.arg("--seed-file").arg(s);
        }
        Self::apply_opts(&mut cmd, opts);
        cmd.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit());
        let child = cmd.spawn().context("spawn reposix-sim --ephemeral")?;
        Ok(Self { child })
    }

    /// Append optional flags (`--no-seed`, `--rate-limit`) to the child
    /// command if the caller asked for non-default behavior.
    fn apply_opts(cmd: &mut std::process::Command, opts: SimOptions) {
        if opts.no_seed {
            cmd.arg("--no-seed");
        }
        if let Some(rps) = opts.rate_limit {
            cmd.arg("--rate-limit").arg(rps.to_string());
        }
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
/// `no_seed` clobbers `seed` to `None` semantically (the child also
/// honors `--no-seed` independently, so we forward both — the CLI flag
/// is what the user typed and is what should appear in `ps`).
/// `rate_limit` is forwarded only when it differs from the sim's own
/// default (100 rps) so `--help` output and `ps` listings stay tidy in
/// the common case.
///
/// # Errors
/// Returns any spawn error or a non-zero exit status from the child.
pub fn run(
    bind: &str,
    db: PathBuf,
    seed: Option<PathBuf>,
    no_seed: bool,
    ephemeral: bool,
    rate_limit: u32,
) -> Result<()> {
    // Only forward --rate-limit when it isn't the sim's own default.
    // Mirrors `reposix-sim`'s `default_value_t = 100` so we don't
    // clutter `ps` output for the default case.
    let rate_limit_opt = if rate_limit == 100 {
        None
    } else {
        Some(rate_limit)
    };
    let opts = SimOptions {
        no_seed,
        rate_limit: rate_limit_opt,
    };
    let mut sim = if ephemeral {
        SimProcess::spawn_ephemeral(bind, &db, seed.as_deref(), opts)?
    } else {
        SimProcess::spawn(bind, &db, seed.as_deref(), opts)?
    };
    // Propagate signals: block until child exits or we get Ctrl-C.
    let status = sim.child.wait().context("wait reposix-sim")?;
    std::mem::forget(sim);
    if !status.success() {
        anyhow::bail!("reposix-sim exited with {status}");
    }
    Ok(())
}
