//! Child-process wrapper around the `reposix-fuse` binary, with a 3-second
//! unmount watchdog on Drop.

use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};

use crate::binpath::resolve_bin;

/// A running `reposix-fuse` child with its mount path bookkeept so Drop
/// can try `fusermount3 -u` bounded by a 3-second watchdog.
#[derive(Debug)]
pub struct MountProcess {
    child: Child,
    mount: PathBuf,
}

impl MountProcess {
    /// Spawn the FUSE daemon in the foreground-of-its-own-process-group.
    ///
    /// # Errors
    /// Returns an error if the child fails to spawn, or if the mount does
    /// not expose entries within 3s.
    pub fn spawn(mount_point: &Path, backend: &str, project: &str) -> Result<Self> {
        let mut cmd = resolve_bin("reposix-fuse");
        cmd.arg(mount_point)
            .arg("--backend")
            .arg(backend)
            .arg("--project")
            .arg(project);
        // Discard stdin (the child has no input). Inherit stderr so the
        // child's tracing output surfaces during normal `reposix demo`
        // / `reposix mount` runs. The integration test now uses
        // `std::process::Command` with inherited stdio instead of
        // `assert_cmd`, so pipe-buffer deadlock is not a concern.
        cmd.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit());
        // `process_group(0)` on Unix means "new session leader" — ensures
        // a SIGTERM to this process doesn't propagate automatically to
        // the grandchild fuse process; we manage its lifecycle ourselves.
        // Stable since Rust 1.64 on Unix, requires no unsafe.
        cmd.process_group(0);
        let child = cmd.spawn().context("spawn reposix-fuse")?;
        let me = Self {
            child,
            mount: mount_point.to_path_buf(),
        };
        me.wait_ready()?;
        Ok(me)
    }

    /// Poll the mount point until `read_dir` returns Ok with ≥1 entry, or
    /// 5s elapse. (Bumped from 3s for slower dev hosts / cold starts;
    /// the demo's overall 30s budget still holds.)
    fn wait_ready(&self) -> Result<()> {
        let t0 = Instant::now();
        loop {
            match std::fs::read_dir(&self.mount) {
                Ok(it) => {
                    // Counting entries triggers readdir on the FUSE FS,
                    // which fetches from the backend and populates the
                    // cache — any non-empty result proves the mount is
                    // serving reads.
                    if it.flatten().count() >= 1 {
                        return Ok(());
                    }
                }
                Err(e) => {
                    // EIO from the kernel means FUSE is mounted but the
                    // backend transiently failed; keep polling until
                    // either the backend becomes reachable or we time
                    // out. ENOTCONN / ENOENT would mean the mount
                    // hasn't come up yet — also retry.
                    tracing::debug!("wait_ready: read_dir err (retrying): {e}");
                }
            }
            if t0.elapsed() >= Duration::from_secs(5) {
                anyhow::bail!(
                    "reposix-fuse at {} did not become ready within 5s",
                    self.mount.display()
                );
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    /// Block until the FUSE daemon exits (used by `reposix mount`
    /// foreground).
    ///
    /// # Errors
    /// Returns any [`std::io::Error`] from `wait`.
    pub fn wait(mut self) -> Result<std::process::ExitStatus> {
        let s = self.child.wait().context("wait reposix-fuse")?;
        std::mem::forget(self);
        Ok(s)
    }

    fn watchdog_unmount(&mut self) {
        // 1. SIGTERM the fuse child so it drops its BackgroundSession
        //    (which triggers fuser's UmountOnDrop).
        let pid_raw = self.child.id();
        if let Ok(pid_i32) = i32::try_from(pid_raw) {
            if let Some(pid) = rustix::process::Pid::from_raw(pid_i32) {
                let _ = rustix::process::kill_process(pid, rustix::process::Signal::Term);
            }
        }
        // 2. Spawn `fusermount3 -u <mount>` as belt-and-suspenders. If
        //    fuser's own UmountOnDrop already unmounted, this is a no-op
        //    that exits non-zero; that's fine.
        let Ok(mut um) = std::process::Command::new("fusermount3")
            .arg("-u")
            .arg(&self.mount)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        else {
            let _ = self.child.wait();
            return;
        };
        let t0 = Instant::now();
        #[allow(clippy::while_let_loop)]
        // the explicit loop mirrors the watchdog structure and keeps the timeout/poll split readable
        loop {
            match um.try_wait() {
                Ok(None) => {
                    if t0.elapsed() >= Duration::from_secs(3) {
                        let _ = um.kill();
                        let _ = um.wait();
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                Ok(Some(_)) | Err(_) => break,
            }
        }
        // 3. Reap the fuse child so it doesn't zombie.
        let _ = self.child.wait();
    }
}

impl Drop for MountProcess {
    fn drop(&mut self) {
        self.watchdog_unmount();
    }
}

/// Inline wrapper for `reposix mount` (foreground). Spawns fuse and
/// blocks until it exits.
///
/// # Errors
/// Returns any spawn error or a non-zero exit from the child.
pub fn run(mount_point: PathBuf, backend: String, project: String) -> Result<()> {
    let mount = MountProcess::spawn(&mount_point, &backend, &project)?;
    let status = mount.wait()?;
    if !status.success() {
        anyhow::bail!("reposix-fuse exited with {status}");
    }
    Ok(())
}
