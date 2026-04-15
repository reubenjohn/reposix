//! Chaos audit-log test (HARD-03): kill -9 the sim mid-swarm, verify WAL
//! durability (no torn rows).
//!
//! This test is `#[ignore]` by default and requires `REPOSIX_CHAOS_TEST=1`
//! to run. It spawns the `reposix-sim` binary as a real child process so
//! `Child::kill()` sends SIGKILL (not cooperative cancellation).
//!
//! Purpose: An in-process `JoinHandle::abort()` unwinds too cleanly — we need
//! a real child process and a real SIGKILL to prove WAL durability. Per
//! RESEARCH.md §Pattern 4 and Pitfall 2.
//!
//! ## Binary path resolution
//!
//! `CARGO_BIN_EXE_reposix-sim` is only set by Cargo when the binary lives in
//! the same package as the integration test. Since `reposix-sim` is a sibling
//! crate, we resolve the path at runtime from `CARGO_MANIFEST_DIR`:
//!
//! ```text
//! <manifest_dir>/../../../target/{profile}/reposix-sim
//! ```
//!
//! The `REPOSIX_SIM_BIN` env var overrides this (useful in CI where the
//! binary was built separately with `cargo build --release`).

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc, clippy::too_many_lines)]

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use reposix_core::http::{client, ClientOpts};
use rusqlite::Connection;
use tempfile::NamedTempFile;

/// Fixed bind address for the chaos sim. Port 7979 is chosen to avoid
/// colliding with the default sim port (7878) or the contention test port.
/// Operators running this test manually should ensure no stale sim process
/// holds this port (a best-effort `pkill -f reposix-sim` runs at test start).
const SIM_BIND: &str = "127.0.0.1:7979";
const HEALTHZ_URL: &str = "http://127.0.0.1:7979/healthz";

/// Resolve the path to the `reposix-sim` binary.
///
/// Resolution order:
/// 1. `REPOSIX_SIM_BIN` env var (explicit override — useful in CI).
/// 2. `<workspace_root>/target/release/reposix-sim` (if exists — prefer
///    release for chaos since it's more representative of production).
/// 3. `<workspace_root>/target/debug/reposix-sim` (fallback for dev).
///
/// Panics if neither path exists, with a helpful message.
fn resolve_sim_bin() -> PathBuf {
    if let Ok(explicit) = std::env::var("REPOSIX_SIM_BIN") {
        return PathBuf::from(explicit);
    }

    // CARGO_MANIFEST_DIR = .../crates/reposix-swarm
    // workspace root = ../../.. from there
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent() // crates/
        .and_then(|p| p.parent()) // workspace root
        .expect("CARGO_MANIFEST_DIR has unexpected structure");

    let release_bin = workspace_root.join("target/release/reposix-sim");
    let debug_bin = workspace_root.join("target/debug/reposix-sim");

    if release_bin.exists() {
        return release_bin;
    }
    if debug_bin.exists() {
        return debug_bin;
    }

    panic!(
        "reposix-sim binary not found at {} or {}.\n\
         Build it first with: cargo build -p reposix-sim --release\n\
         Or set REPOSIX_SIM_BIN=/path/to/reposix-sim",
        release_bin.display(),
        debug_bin.display(),
    );
}

/// Poll /healthz asynchronously until the sim is up or timeout expires.
async fn poll_healthz_async(timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    let http = client(ClientOpts {
        total_timeout: Duration::from_millis(200),
        ..ClientOpts::default()
    })
    .expect("http client build");
    while Instant::now() < deadline {
        if http
            .get(HEALTHZ_URL)
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
        {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    false
}

/// Count rows with NULL in any NOT NULL column of `audit_events`.
///
/// The audit schema marks `ts`, `method`, and `path` as NOT NULL.
/// A torn row (partial WAL write visible after recovery) would have NULL in
/// one of these columns. The absence of torn rows proves `SQLite` WAL atomicity.
///
/// # Errors
/// Returns [`rusqlite::Error`] if the DB cannot be opened or the query fails.
fn count_torn_rows(db: &Path) -> rusqlite::Result<i64> {
    let conn = Connection::open(db)?;
    conn.query_row(
        "SELECT COUNT(*) FROM audit_events \
         WHERE ts IS NULL OR method IS NULL OR path IS NULL",
        [],
        |row| row.get(0),
    )
}

/// Count all rows in `audit_events`.
///
/// # Errors
/// Returns [`rusqlite::Error`] if the DB cannot be opened or the query fails.
fn count_all_rows(db: &Path) -> rusqlite::Result<i64> {
    let conn = Connection::open(db)?;
    conn.query_row("SELECT COUNT(*) FROM audit_events", [], |row| row.get(0))
}

/// Chaos test: SIGKILL `reposix-sim` mid-swarm; assert no torn rows in WAL DB.
///
/// Algorithm (two cycles):
///
/// **Cycle 1** — Fresh DB:
///   1. Spawn `reposix-sim` on a temp DB.
///   2. Poll /healthz up to 5 s.
///   3. Drive async load (GET /issues/* for 500 ms to generate audit rows).
///   4. `child.kill()` (SIGKILL) + `child.wait()` (reap).
///   5. Sleep 200 ms for OS to release the file lock.
///   6. Reopen DB; assert zero torn rows.
///
/// **Cycle 2** — Same DB (WAL replay on reopen):
///   1. Restart sim on the same DB file.
///   2. Poll /healthz (proves WAL replay succeeded — sim can open DB after kill).
///   3. Brief load + SIGKILL again.
///   4. Assert zero torn rows AND `total_2 >= total_1` (no row loss on replay).
#[tokio::test]
#[ignore = "chaos: requires reposix-sim binary + REPOSIX_CHAOS_TEST=1"]
async fn chaos_kill9_no_torn_rows() {
    if std::env::var("REPOSIX_CHAOS_TEST").is_err() {
        eprintln!("SKIP: set REPOSIX_CHAOS_TEST=1 to run chaos tests");
        return;
    }

    let sim_bin = resolve_sim_bin();
    eprintln!("chaos test: using sim binary at {sim_bin:?}");

    // Best-effort: kill any stale sim from a previous interrupted run so
    // port 7979 is free. Ignore exit code (pkill exits 1 if no match).
    let _ = Command::new("pkill").args(["-f", "reposix-sim"]).status();
    tokio::time::sleep(Duration::from_millis(100)).await;

    let db = NamedTempFile::new().expect("tempfile");
    let db_path = db.path().to_owned();

    // ── Cycle 1: spawn → load → SIGKILL → check ──────────────────────────────

    let mut child = Command::new(&sim_bin)
        .args([
            "--bind",
            SIM_BIND,
            "--db",
            db_path.to_str().expect("db path is valid UTF-8"),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn reposix-sim (cycle 1)");

    assert!(
        poll_healthz_async(Duration::from_secs(5)).await,
        "sim did not come up within 5s (cycle 1) — check that port {SIM_BIND} is free"
    );

    // Drive load. We use reqwest::Client directly (not ContentionWorkload) to
    // keep this test self-contained. Many requests will 404 (no seeded data)
    // or be cut short by the kill; that is fine — every HTTP request still
    // writes an audit row, which is what we need for WAL activity.
    let load = tokio::spawn(async {
        let http = client(ClientOpts::default()).expect("http client");
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut ops = 0_u64;
        while Instant::now() < deadline {
            // fire-and-forget — ECONNREFUSED errors after kill are expected
            let _ = http.get(format!("http://{SIM_BIND}/issues/demo")).await;
            ops += 1;
        }
        ops
    });

    // Let some requests hit the WAL before we kill.
    tokio::time::sleep(Duration::from_millis(500)).await;

    child.kill().expect("SIGKILL reposix-sim (cycle 1)");
    child.wait().expect("reap reposix-sim (cycle 1)");

    // Abort the in-flight load; stray ECONNREFUSED errors after kill are fine.
    load.abort();
    let _ = load.await;

    // Give the kernel time to flush and release the file lock.
    tokio::time::sleep(Duration::from_millis(200)).await;

    let bad1 = count_torn_rows(&db_path).expect("query torn rows (cycle 1)");
    assert_eq!(
        bad1, 0,
        "torn rows (NULL in ts/method/path) found after SIGKILL (cycle 1): {bad1}"
    );

    let total1 = count_all_rows(&db_path).expect("row count (cycle 1)");
    eprintln!("cycle 1: total audit rows after SIGKILL = {total1} (0 torn rows — WAL OK)");

    // ── Cycle 2: restart on same DB → WAL replay → load → SIGKILL → check ───

    let mut child2 = Command::new(&sim_bin)
        .args([
            "--bind",
            SIM_BIND,
            "--db",
            db_path.to_str().expect("db path is valid UTF-8"),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn reposix-sim (cycle 2)");

    assert!(
        poll_healthz_async(Duration::from_secs(5)).await,
        "sim did not come up within 5s after WAL replay (cycle 2) — \
         WAL recovery may have failed or port {SIM_BIND} still in use"
    );

    // Brief load to generate a few more WAL writes before the second kill.
    let load2 = tokio::spawn(async {
        let http = client(ClientOpts::default()).expect("http client");
        let deadline = Instant::now() + Duration::from_millis(600);
        while Instant::now() < deadline {
            let _ = http.get(format!("http://{SIM_BIND}/issues/demo")).await;
        }
    });

    tokio::time::sleep(Duration::from_millis(300)).await;

    child2.kill().expect("SIGKILL reposix-sim (cycle 2)");
    child2.wait().expect("reap reposix-sim (cycle 2)");

    load2.abort();
    let _ = load2.await;

    tokio::time::sleep(Duration::from_millis(200)).await;

    let bad2 = count_torn_rows(&db_path).expect("query torn rows (cycle 2)");
    assert_eq!(
        bad2, 0,
        "torn rows (NULL in ts/method/path) found after SIGKILL (cycle 2): {bad2}"
    );

    let total2 = count_all_rows(&db_path).expect("row count (cycle 2)");
    eprintln!("cycle 2: total audit rows after SIGKILL = {total2} (0 torn rows — WAL OK)");

    // WAL checkpoint on reopen must not lose rows committed before the first kill.
    // Note (T-21-D-03): SQLite WAL replay can in principle roll back uncommitted
    // writes, so total2 >= total1 is the invariant (rows in cycle 1 must survive
    // WAL replay in cycle 2). If this assertion flakes because the restart rolled
    // back in-flight writes from cycle 1, investigate whether WAL sync mode is
    // correct before relaxing the assertion.
    assert!(
        total2 >= total1,
        "cycle 2 row count {total2} < cycle 1 {total1} — WAL checkpoint dropped committed rows"
    );
}
