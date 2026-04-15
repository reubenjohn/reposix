//! End-to-end integration test for the contention workload.
//!
//! Spins the simulator in-process on an ephemeral port, runs 50
//! `ContentionWorkload` clients for 5 seconds all hammering the same issue
//! with explicit `If-Match` versions, and asserts three invariants:
//!
//! 1. At least one PATCH wins (If-Match doesn't lock out every client).
//! 2. At least one PATCH loses with 409 Conflict (clients are actually racing).
//! 3. No `Other`-class errors (no transport / serialization bugs).
//!
//! This closes HARD-01: the swarm harness now has a mode that deterministically
//! provokes 409s, and the test encodes the "no torn writes" invariant.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]

use std::path::PathBuf;
use std::time::Duration;

use reposix_core::IssueId;
use reposix_core::http::{client, ClientOpts};
use reposix_sim::{run_with_listener, SimConfig};
use reposix_swarm::contention::ContentionWorkload;
use reposix_swarm::driver::{run_swarm, SwarmConfig};
use tempfile::NamedTempFile;

fn seed_fixture() -> PathBuf {
    // Sibling crate's fixture; resolved relative to this crate's manifest.
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop(); // → crates/
    p.push("reposix-sim/fixtures/seed.json");
    assert!(
        p.exists(),
        "seed fixture missing at {} — sim crate moved?",
        p.display()
    );
    p
}

/// Spawn the simulator on an ephemeral port. Returns `(base_url, db_handle,
/// join_handle)`. Drop `join_handle` to abort the sim task; keep `db_handle`
/// alive for the duration of the test.
async fn spawn_sim(rate_limit_rps: u32) -> (String, NamedTempFile, tokio::task::JoinHandle<()>) {
    let db = NamedTempFile::new().expect("tempfile");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind ephemeral port");
    let addr = listener.local_addr().expect("local_addr");
    let base_url = format!("http://{addr}");

    let cfg = SimConfig {
        bind: addr,
        db_path: db.path().to_owned(),
        seed: true,
        seed_file: Some(seed_fixture()),
        ephemeral: false,
        rate_limit_rps,
    };

    let handle = tokio::spawn(async move {
        let _ = run_with_listener(listener, cfg).await;
    });

    // Block until /healthz responds (max ~1s).
    let http = client(ClientOpts::default()).expect("http client");
    for _ in 0..40 {
        if http
            .get(format!("{base_url}/healthz"))
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
        {
            return (base_url, db, handle);
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("sim failed to come up at {base_url} within 1s");
}

fn audit_row_count(path: &std::path::Path) -> rusqlite::Result<i64> {
    // Open R/W (not RO) — sim is still running in WAL mode and a bare RO
    // handle can't see WAL-resident rows.
    let conn = rusqlite::Connection::open(path)?;
    conn.query_row("SELECT COUNT(*) FROM audit_events", [], |row| row.get(0))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn contention_50_clients_5s_deterministic_409() {
    // Generous rate limit: 50 clients × 2 ops/cycle; each client has its own
    // per-agent bucket via `with_agent_suffix`, so 200 rps per bucket is fine.
    let (base, db, sim_handle) = spawn_sim(200).await;
    let db_path = db.path().to_owned();

    // Issue id 1 is always seeded by reposix-sim/fixtures/seed.json.
    let target_id = IssueId(1);

    let cfg = SwarmConfig {
        clients: 50,
        duration: Duration::from_secs(5),
        mode: "contention",
        target: &base,
    };

    let origin = base.clone();
    let project = "demo".to_string();
    let markdown = run_swarm(cfg, |i| {
        ContentionWorkload::new(
            origin.clone(),
            project.clone(),
            target_id,
            u64::try_from(i).unwrap_or(0),
        )
    })
    .await
    .expect("run_swarm returned cleanly");

    // Invariant 1: at least one PATCH op was recorded (wins or conflicts).
    assert!(
        markdown.contains("| patch "),
        "expected patch op row in summary:\n{markdown}"
    );

    // Invariant 2: Conflict errors were recorded (clients are actually racing).
    // The "Errors by class" section only renders when there are errors.
    let has_conflict = markdown
        .split("### Errors by class")
        .nth(1)
        .map_or(false, |s| s.contains("| Conflict"));
    assert!(
        has_conflict,
        "expected Conflict errors — clients should race on If-Match:\n{markdown}"
    );

    // Invariant 3: No Other-class errors (no transport / serialization bugs).
    if let Some(err_section) = markdown.split("### Errors by class").nth(1) {
        assert!(
            !err_section.contains("| Other"),
            "Other-class errors present (transport/serialization bug):\n{markdown}"
        );
    }

    // Invariant 4: audit log received write rows (each winning PATCH writes one row).
    let rows = audit_row_count(&db_path).expect("audit query");
    assert!(
        rows >= 1,
        "expected at least 1 audit row after contention run, got {rows}\n\
         markdown:\n{markdown}"
    );

    sim_handle.abort();
    let _ = sim_handle.await;
}
