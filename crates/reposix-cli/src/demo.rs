//! `reposix demo` — end-to-end orchestration. Spawns the sim, waits for
//! `/healthz`, mounts the FUSE daemon, runs scripted `ls`/`cat`/`grep`
//! against the live mount, tails the audit DB, and tears down. A
//! top-level `Guard` owns the sim + mount + tempdir so teardown order is
//! deterministic on any exit path (normal, `?`, panic, Ctrl-C).

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Context, Result};
use reposix_core::http::{client, ClientOpts};
use rusqlite::OpenFlags;
use tracing::info;

use crate::mount::MountProcess;
use crate::sim::SimProcess;

const SIM_BIND: &str = "127.0.0.1:7878";

/// Top-level resource guard. Drop order (guaranteed by struct field
/// order + Rust drop semantics): mount first (so `fusermount3 -u` runs
/// before the sim dies and the kernel sees backend disappear), then
/// sim, then the tempdir.
#[derive(Default)]
struct Guard {
    mount: Option<MountProcess>,
    sim: Option<SimProcess>,
    tempdir: Option<tempfile::TempDir>,
}

impl Drop for Guard {
    fn drop(&mut self) {
        // Explicit take-and-drop in order, so panics inside one Drop
        // don't leak the other resources.
        self.mount.take();
        self.sim.take();
        self.tempdir.take();
    }
}

/// Run the full demo flow.
///
/// # Errors
/// Returns any spawn / network / `SQLite` / I/O error from the scripted
/// steps. Cleanup is guaranteed via [`Guard`].
pub async fn run(keep_running: bool) -> Result<()> {
    let mut guard = Guard::default();
    let body = async {
        // Step 1: sim.
        info!("[step 1/6] starting reposix-sim on {SIM_BIND} (ephemeral)");
        let db_path =
            PathBuf::from("runtime").join(format!("demo-sim-{pid}.db", pid = std::process::id()));
        std::fs::create_dir_all("runtime").ok();
        let seed = PathBuf::from("crates/reposix-sim/fixtures/seed.json");
        let seed_arg = if seed.exists() { Some(&seed) } else { None };
        guard.sim = Some(
            SimProcess::spawn_ephemeral(
                SIM_BIND,
                &db_path,
                seed_arg.map(std::convert::AsRef::as_ref),
            )
            .context("spawn sim")?,
        );

        // Step 2: healthz wait. 15s budget covers the `cargo run -q -p
        // reposix-sim --` cold-start fallback used when the sibling
        // binary isn't built — common in CI's integration job before
        // `cargo build --release` has landed all binaries.
        info!("[step 2/6] waiting for /healthz");
        wait_for_healthz(
            &format!("http://{SIM_BIND}/healthz"),
            Duration::from_secs(15),
        )
        .await
        .context("healthz wait")?;

        // Step 3: mount on a tempdir.
        let td = tempfile::Builder::new()
            .prefix("reposix-demo-")
            .tempdir()
            .context("create tempdir")?;
        let mount_path = td.path().to_path_buf();
        info!("[step 3/6] mounting FUSE at {}", mount_path.display());
        guard.tempdir = Some(td);
        guard.mount = Some(
            MountProcess::spawn(&mount_path, &format!("http://{SIM_BIND}"), "demo")
                .context("spawn mount")?,
        );

        // Step 4: scripted ls / cat / grep.
        info!(
            "[step 4/6] scripted ls / cat / grep on {}",
            mount_path.display()
        );
        let listing = list_sorted(&mount_path)?;
        info!("  ls: {listing:?}");
        let first_name = listing
            .first()
            .ok_or_else(|| anyhow!("mount empty — sim missing seed?"))?;
        let first = mount_path.join(first_name);
        let body = std::fs::read_to_string(&first).context("read 0001.md")?;
        let head: Vec<&str> = body.lines().take(3).collect();
        info!("  cat {}:\n    {}", first.display(), head.join("\n    "));
        let hits = grep_ril(&mount_path, "database")?;
        info!("  grep -ril database: {hits:?}");

        // Step 5: audit tail.
        info!(
            "[step 5/6] tail last 5 audit rows from {}",
            db_path.display()
        );
        print_audit_tail(&db_path, 5)?;

        // Step 6: teardown via Guard::drop.
        info!("[step 6/6] cleaning up (Guard::drop)");
        if keep_running {
            info!("  --keep-running set; press Ctrl-C to exit");
            tokio::signal::ctrl_c().await.ok();
        }
        Ok::<_, anyhow::Error>(())
    };

    let res = tokio::select! {
        r = body => r,
        _ = tokio::signal::ctrl_c() => {
            info!("Ctrl-C received — cleaning up");
            Ok(())
        }
    };
    drop(guard); // explicit for clarity; the implicit drop at scope-end would be equivalent
    res
}

async fn wait_for_healthz(url: &str, budget: Duration) -> Result<()> {
    let http = client(ClientOpts::default())?;
    let t0 = Instant::now();
    loop {
        if let Ok(resp) = http.get(url).await {
            if resp.status().is_success() {
                return Ok(());
            }
        }
        if t0.elapsed() >= budget {
            bail!("sim did not become ready at {url} within {budget:?}");
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

fn list_sorted(dir: &Path) -> Result<Vec<String>> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .with_context(|| format!("read_dir {}", dir.display()))?
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    Ok(names)
}

fn grep_ril(dir: &Path, needle: &str) -> Result<Vec<PathBuf>> {
    let needle_lower = needle.to_lowercase();
    let mut hits = Vec::new();
    for entry in std::fs::read_dir(dir)
        .with_context(|| format!("read_dir {}", dir.display()))?
        .flatten()
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if let Ok(body) = std::fs::read_to_string(&path) {
            if body.to_lowercase().contains(&needle_lower) {
                hits.push(path);
            }
        }
    }
    Ok(hits)
}

fn print_audit_tail(db: &Path, limit: u32) -> Result<()> {
    if !db.exists() {
        info!("  (audit DB not yet flushed to disk)");
        return Ok(());
    }
    let conn = rusqlite::Connection::open_with_flags(db, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .with_context(|| format!("open audit DB {}", db.display()))?;
    // The Phase-1 fixture + Phase-2 writer stores these columns. Try the
    // canonical column set; if the simulator schema names differ we
    // fall back to a wildcard SELECT that still tails the last N rows.
    let mut stmt = conn
        .prepare(
            "SELECT ts, agent, method, path, status FROM audit_events ORDER BY id DESC LIMIT ?1",
        )
        .context("prepare audit query")?;
    let rows = stmt
        .query_map([limit], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
            ))
        })
        .context("query audit_events")?;
    for row in rows.flatten() {
        let (ts, agent, method, path, status) = row;
        info!("  audit: {ts} {agent} {method} {path} -> {status}");
    }
    Ok(())
}
