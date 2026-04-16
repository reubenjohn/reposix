//! End-to-end integration test for the swarm harness.
//!
//! Closes OP-6 MEDIUM-14 from `HANDOFF.md` — prior to this file the crate
//! shipped only 3 metrics unit tests and zero integration coverage. This
//! test spins the simulator in-process on an ephemeral port, runs the swarm
//! driver with a handful of `SimDirectWorkload` clients for ~1.5s, and
//! asserts the four invariants that matter for a swarm run:
//!
//! 1. The driver returns cleanly (no panic, no hang past the deadline).
//! 2. At least a handful of operations complete (`>= 5`).
//! 3. No `Other`-class permanent errors are recorded. Transient `Conflict`
//!    (409) and `RateLimited` (429) are tolerated — they're expected under
//!    concurrency and rate-limiting and the workload is built to absorb
//!    them.
//! 4. The simulator's append-only audit log received rows for the swarm's
//!    work (each successful op writes one row).
//!
//! Pattern mirrors `crates/reposix-sim/tests/api.rs::spawn_sim`: bind a
//! `tokio::net::TcpListener` on `127.0.0.1:0`, hand it to
//! `reposix_sim::run_with_listener`, and wait for `/healthz` before driving
//! load. The default outbound allowlist (`http://127.0.0.1:*`) covers the
//! ephemeral port without any `REPOSIX_ALLOWED_ORIGINS` override.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]

use std::path::PathBuf;
use std::time::Duration;

use reposix_core::http::{client, ClientOpts};
use reposix_sim::{run_with_listener, SimConfig};
use reposix_swarm::driver::{run_swarm, SwarmConfig};
use reposix_swarm::sim_direct::SimDirectWorkload;
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
/// alive for the duration of the test (`NamedTempFile` deletes on drop and
/// the audit DB lives there).
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

    // Block until /healthz responds (max ~1s). Mirrors sim's own integration
    // test poll loop.
    let http = client(ClientOpts::default()).expect("http client");
    for _ in 0..40 {
        if http
            .get(format!("{base_url}/healthz"))
            .await
            .is_ok_and(|r| r.status().is_success())
        {
            return (base_url, db, handle);
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("sim failed to come up at {base_url} within 1s");
}

fn audit_row_count(path: &std::path::Path) -> rusqlite::Result<i64> {
    // Open R/W (not RO) — sim is still running in WAL mode and a bare RO
    // handle can't see WAL-resident rows. Identical rationale to the
    // `audit_row_count` helper in `reposix-swarm/src/main.rs`.
    let conn = rusqlite::Connection::open(path)?;
    conn.query_row("SELECT COUNT(*) FROM audit_events", [], |row| row.get(0))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn swarm_mini_e2e_sim_5_clients_1_5s() {
    // Generous rate limit so 5 clients × ~5 ops/cycle don't immediately
    // saturate the per-agent bucket. Each client gets its own bucket via
    // the `with_agent_suffix` path in `SimDirectWorkload::new`, so 200 rps
    // per client is plenty.
    let (base, db, sim_handle) = spawn_sim(200).await;
    let db_path = db.path().to_owned();

    let cfg = SwarmConfig {
        clients: 5,
        duration: Duration::from_millis(1_500),
        mode: "sim-direct",
        target: &base,
    };

    let origin = base.clone();
    let project = "demo".to_string();
    let markdown = run_swarm(cfg, |i| {
        SimDirectWorkload::new(
            origin.clone(),
            project.clone(),
            u64::try_from(i).unwrap_or(0),
        )
    })
    .await
    .expect("run_swarm returned cleanly");

    // Invariant 1: clean return (we got here without panic/hang).
    // Invariant 2: rendered summary mentions the configured client count
    // and at least one op kind row.
    assert!(
        markdown.contains("Clients: 5"),
        "summary missing client count:\n{markdown}"
    );
    assert!(
        markdown.contains("| list "),
        "summary missing list row:\n{markdown}"
    );

    // Invariant 3: total ops >= 5, no permanent (Other-class) errors.
    // We dig the totals out of the markdown rather than threading the
    // metrics handle out of `run_swarm` — keeps the test honest about the
    // public surface.
    let total_ops = parse_total_ops(&markdown);
    assert!(
        total_ops >= 5,
        "expected >=5 total ops, got {total_ops}:\n{markdown}"
    );

    // The summary's "Errors by class" section only renders when there are
    // errors. If it's present, allow Conflict/RateLimited/NotFound (all
    // expected under contention or due to a swarm client touching an
    // issue another client is mid-patch on) but reject `Other`.
    if let Some(err_section) = markdown.split("### Errors by class").nth(1) {
        assert!(
            !err_section.contains("| Other"),
            "swarm produced Other-class errors (transport/serialization), \
             which indicates a real bug:\n{markdown}"
        );
    }

    // Invariant 4: audit log captured swarm activity. Each successful op
    // writes one row; with 5 clients × ~5 ops/cycle × multiple cycles in
    // 1.5s we expect well over a dozen rows. Use a conservative floor.
    let rows = audit_row_count(&db_path).expect("audit query");
    assert!(
        rows >= 5,
        "expected >=5 audit rows after swarm run, got {rows}\n\
         markdown:\n{markdown}"
    );

    // Tear down the sim explicitly so the test process doesn't hang on
    // the spawned task. The JoinHandle is detached; `abort()` is the
    // documented stop signal for `axum::serve` running under `tokio::spawn`.
    sim_handle.abort();
    let _ = sim_handle.await;
}

/// Pull the `Total ops: N` integer out of the swarm markdown summary.
/// Returns 0 if the line is missing — the assertion in the test will then
/// fail with a helpful diff.
fn parse_total_ops(md: &str) -> u64 {
    md.lines()
        .find_map(|l| l.strip_prefix("Total ops: "))
        .and_then(|tail| tail.split_whitespace().next())
        .and_then(|n| n.parse::<u64>().ok())
        .unwrap_or(0)
}

// ── Confluence-direct wiremock test ──────────────────────────────────────────

use reposix_confluence::ConfluenceCreds;
use reposix_swarm::confluence_direct::ConfluenceDirectWorkload;
use serde_json::json;
use wiremock::matchers::{method, path, path_regex, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn sample_page(id: &str, title: &str) -> serde_json::Value {
    json!({
        "id": id,
        "status": "current",
        "title": title,
        "createdAt": "2026-01-01T00:00:00Z",
        "version": {"number": 1, "createdAt": "2026-01-01T00:00:00Z"},
        "body": {
            "atlas_doc_format": {
                "value": {"type":"doc","version":1,"content":[]}
            }
        }
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn confluence_direct_3_clients_5s() {
    let server = MockServer::start().await;

    // Space resolver — called repeatedly (RESEARCH.md Risk 4); no .expect(N).
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/spaces"))
        .and(query_param("keys", "TESTSPACE"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [{"id": "9001", "key": "TESTSPACE"}]
        })))
        .mount(&server)
        .await;

    // Page list — single page (empty _links) so `list_issues` exits
    // the pagination loop cleanly (RESEARCH.md Risk 3).
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/spaces/9001/pages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [
                sample_page("10001", "Page 1"),
                sample_page("10002", "Page 2"),
                sample_page("10003", "Page 3"),
            ],
            "_links": {}
        })))
        .mount(&server)
        .await;

    // Page get — path_regex so any of the 3 ids match.
    // NOTE: this stub always responds with sample_page("10001", "Page 1")
    // regardless of which page id was requested. This is intentional for
    // load-testing purposes but means id-routing bugs in
    // `ConfluenceBackend::get_issue` (e.g., always requesting id 0) would
    // go undetected. A more precise per-id stub set could be added if
    // id-routing correctness becomes a concern.
    Mock::given(method("GET"))
        .and(path_regex(r"^/wiki/api/v2/pages/\d+$"))
        .and(query_param("body-format", "atlas_doc_format"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_page("10001", "Page 1")))
        .mount(&server)
        .await;

    let base = server.uri();
    let creds = ConfluenceCreds {
        email: "swarm@test".to_string(),
        api_token: "tok".to_string(),
    };
    let space = "TESTSPACE".to_string();

    let cfg = SwarmConfig {
        clients: 3,
        duration: Duration::from_secs(5),
        mode: "confluence-direct",
        target: &base,
    };
    let markdown = run_swarm(cfg, |i| {
        ConfluenceDirectWorkload::new(
            base.clone(),
            creds.clone(),
            space.clone(),
            u64::try_from(i).unwrap_or(0),
        )
    })
    .await
    .expect("run_swarm returned cleanly");

    assert!(
        markdown.contains("Clients: 3"),
        "summary missing client count:\n{markdown}"
    );
    assert!(
        markdown.contains("| list "),
        "summary missing list row:\n{markdown}"
    );
    assert!(
        markdown.contains("| get "),
        "summary missing get row — get_issue calls not being recorded:\n{markdown}"
    );
    let total_ops = parse_total_ops(&markdown);
    assert!(
        total_ops >= 3,
        "expected >=3 total ops, got {total_ops}:\n{markdown}"
    );
    if let Some(err_section) = markdown.split("### Errors by class").nth(1) {
        assert!(
            !err_section.contains("| Other"),
            "confluence-direct produced Other-class errors under wiremock, \
             which indicates transport/deser breakage:\n{markdown}"
        );
    }
    // NOTE: deliberately NO audit-row assertion. Read-only workload
    // writes 0 audit rows; see RESEARCH.md §"Summary" audit caveat.
}
