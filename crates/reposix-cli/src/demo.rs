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
use crate::sim::{SimOptions, SimProcess};

const SIM_BIND: &str = "127.0.0.1:7878";

/// Top-level resource guard. Drop order (guaranteed by struct field
/// order + Rust drop semantics): mount first (so `fusermount3 -u` runs
/// before the sim dies and the kernel sees backend disappear), then
/// sim, then the tempdir.
///
/// `sim_db` is the on-disk path used by step 5 (audit tail). H-01 fix
/// (review 2026-04-13): the demo previously spawned the sim with
/// `--ephemeral` (in-memory DB) but step 5 tried to open
/// `runtime/demo-sim-<pid>.db`, which never existed — the audit tail
/// always early-returned with a misleading "not yet flushed" message.
/// Now the sim runs against a persistent DB so the audit query has rows
/// to read. Drop sweeps the file plus its WAL siblings (`*-wal`,
/// `*-shm`) — `SQLite` WAL mode creates them and `tempfile::TempDir`
/// can't sweep them because we deliberately keep the DB under
/// `runtime/` (not the tempdir) for post-demo inspectability.
#[derive(Default)]
struct Guard {
    mount: Option<MountProcess>,
    sim: Option<SimProcess>,
    tempdir: Option<tempfile::TempDir>,
    sim_db: Option<PathBuf>,
}

impl Drop for Guard {
    fn drop(&mut self) {
        // Explicit take-and-drop in order, so panics inside one Drop
        // don't leak the other resources.
        self.mount.take();
        self.sim.take();
        self.tempdir.take();
        // Best-effort cleanup of the persistent sim DB + WAL siblings.
        // Errors are swallowed: the demo has already exited by the time
        // Drop runs, and a leftover `*-wal` in `runtime/` is a UX wart,
        // not a correctness bug — the next demo run uses a different
        // PID-suffixed path.
        if let Some(db) = self.sim_db.take() {
            let _ = std::fs::remove_file(&db);
            // SQLite WAL siblings are exactly `<db>-wal` / `<db>-shm`,
            // not `<stem>-wal.<ext>`. Append-on-string is correct.
            let mut wal = db.clone().into_os_string();
            wal.push("-wal");
            let _ = std::fs::remove_file(PathBuf::from(wal));
            let mut shm = db.clone().into_os_string();
            shm.push("-shm");
            let _ = std::fs::remove_file(PathBuf::from(shm));
        }
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
        // H-01 fix (review 2026-04-13): use a persistent on-disk DB so
        // step 5's audit tail can actually read rows. The path is
        // PID-suffixed so concurrent demo runs (e.g. `cargo test
        // --ignored` parallelism) don't collide. `Guard::drop` sweeps
        // the file + WAL siblings on exit.
        info!("[step 1/6] starting reposix-sim on {SIM_BIND}");
        let db_path =
            PathBuf::from("runtime").join(format!("demo-sim-{pid}.db", pid = std::process::id()));
        std::fs::create_dir_all("runtime").ok();
        // Defensive: a leftover DB from a prior crashed run with the
        // same PID would carry stale audit rows that confuse step 5.
        // Best-effort remove; ignore NotFound.
        let _ = std::fs::remove_file(&db_path);
        let seed = PathBuf::from("crates/reposix-sim/fixtures/seed.json");
        let seed_arg = if seed.exists() { Some(&seed) } else { None };
        guard.sim_db = Some(db_path.clone());
        guard.sim = Some(
            SimProcess::spawn(
                SIM_BIND,
                &db_path,
                seed_arg.map(std::convert::AsRef::as_ref),
                SimOptions::default(),
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
            MountProcess::spawn(
                &mount_path,
                &format!("http://{SIM_BIND}"),
                "demo",
                crate::list::ListBackend::Sim,
            )
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
    // H-01 fix (review 2026-04-13): with the sim now running on-disk,
    // the DB file should always exist by the time step 5 runs. If it
    // doesn't, that's a real failure — surface it instead of silently
    // returning Ok(()) with a misleading "not yet flushed" message.
    if !db.exists() {
        bail!(
            "audit DB missing at {} — sim failed to create it?",
            db.display()
        );
    }
    let conn = rusqlite::Connection::open_with_flags(db, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .with_context(|| format!("open audit DB {}", db.display()))?;
    // H-01 fix: schema column is `agent_id` (see
    // crates/reposix-core/fixtures/audit.sql:18 and the writer at
    // crates/reposix-sim/src/middleware/audit.rs:119). The previous
    // SELECT used `agent`, which would have hard-failed
    // `prepare` with `no such column: agent` even if the DB existed.
    let mut stmt = conn
        .prepare(
            "SELECT ts, agent_id, method, path, status \
             FROM audit_events ORDER BY id DESC LIMIT ?1",
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
    let mut count = 0_usize;
    for row in rows.flatten() {
        let (ts, agent, method, path, status) = row;
        info!("  audit: {ts} {agent} {method} {path} -> {status}");
        count += 1;
    }
    if count == 0 {
        info!("  (no audit rows yet — sim may not have served any requests)");
    }
    Ok(())
}
