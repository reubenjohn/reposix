//! End-to-end integration tests for the simulator.
//!
//! Boots the sim on `127.0.0.1:0` (ephemeral port), drives a real HTTP
//! client (via `reposix_core::http::client` — direct `reqwest::Client`
//! construction is banned by the workspace clippy lint), and asserts the
//! live behavior that closes ROADMAP Phase-2 success criteria 1-5.
//!
//! Covered criteria (from `.planning/ROADMAP.md` lines 145-165):
//!   1. list >= 3
//!   2. GET /projects/demo/issues/1 returns 200 + id + version
//!   3. PATCH with bogus If-Match returns 409
//!   4. audit rows written for GET/PATCH/DELETE; UPDATE on audit_events
//!      fails with "append-only" literal
//!   5. this file IS the integration test

use std::path::PathBuf;

use reposix_core::http::{client, ClientOpts};
use reposix_sim::{run_with_listener, SimConfig};
use serde_json::Value;
use tempfile::NamedTempFile;

fn seed_fixture() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("fixtures/seed.json");
    p
}

/// Spawn a sim on an ephemeral port, returning `(base_url, db_path,
/// _join_handle)`. Dropping the handle aborts the task.
async fn spawn_sim(rate_limit_rps: u32) -> (String, NamedTempFile, tokio::task::JoinHandle<()>) {
    let db = NamedTempFile::new().expect("tempfile");
    let db_path = db.path().to_owned();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("local_addr");
    let base_url = format!("http://{addr}");

    let cfg = SimConfig {
        bind: addr, // not used by run_with_listener but carried for completeness
        db_path: db_path.clone(),
        seed: true,
        seed_file: Some(seed_fixture()),
        ephemeral: false,
        rate_limit_rps,
    };

    let handle = tokio::spawn(async move {
        let _ = run_with_listener(listener, cfg).await;
    });

    // Wait for /healthz to respond.
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
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }
    panic!("sim failed to come up on {base_url}");
}

fn open_audit_conn(db_path: &std::path::Path) -> rusqlite::Connection {
    rusqlite::Connection::open(db_path).expect("open audit db")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn full_crud_flow_with_audit() {
    let (base, db, handle) = spawn_sim(100).await;
    let http = client(ClientOpts::default()).expect("http");

    // SC1: list >= 3.
    let resp = http
        .get(format!("{base}/projects/demo/issues"))
        .await
        .expect("list");
    assert!(resp.status().is_success(), "list status {}", resp.status());
    let list: Vec<Value> = resp.json().await.expect("json");
    assert!(list.len() >= 3, "expected >=3 issues, got {}", list.len());

    // SC2: GET /issues/1 → 200 + id=1 + version.
    let resp = http
        .get(format!("{base}/projects/demo/issues/1"))
        .await
        .expect("get1");
    assert_eq!(resp.status(), 200);
    let issue: Value = resp.json().await.expect("json");
    assert_eq!(issue["id"], 1);
    assert!(
        issue["version"].as_u64().unwrap_or(0) >= 1,
        "version should be >=1"
    );

    // SC3: PATCH with bogus If-Match → 409.
    let resp = reqwest_patch_with_headers(
        &http,
        &format!("{base}/projects/demo/issues/1"),
        r#"{"status":"done"}"#,
        &[("If-Match", "\"bogus\"")],
    )
    .await;
    assert_eq!(resp.0, 409);
    let body: Value = serde_json::from_str(&resp.1).expect("json");
    assert_eq!(body["error"], "version_mismatch");
    assert_eq!(body["current"], 1);
    assert_eq!(body["sent"], "bogus");

    // DELETE /issues/2 → 204.
    let resp = http
        .delete(format!("{base}/projects/demo/issues/2"))
        .await
        .expect("delete");
    assert_eq!(resp.status(), 204);

    // SC4: audit rows exist for GET/PATCH/DELETE under /projects/demo/.
    //     Open a second connection to the file-backed DB.
    // Flush WAL by running a no-op query against a fresh handle first; then
    // open and read.
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let conn = open_audit_conn(db.path());
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM audit_events \
             WHERE method IN ('GET','PATCH','DELETE') \
               AND path LIKE '/projects/demo/%'",
            [],
            |r| r.get(0),
        )
        .expect("count");
    assert!(
        count >= 4,
        "expected >=4 GET/PATCH/DELETE audit rows, got {count}"
    );

    // SC4 trigger: UPDATE on audit_events must fail with 'append-only'.
    let err = conn
        .execute(
            "UPDATE audit_events SET path='x' WHERE id = \
             (SELECT MIN(id) FROM audit_events)",
            [],
        )
        .unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("append-only"),
        "trigger error must contain literal `append-only`; got {msg:?}"
    );

    handle.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rate_limit_returns_429_on_overflow() {
    let (base, _db, handle) = spawn_sim(2).await;
    let http = client(ClientOpts::default()).expect("http");

    let mut saw_429 = false;
    let mut retry_after_header_present = false;
    for _ in 0..10 {
        let resp = http
            .request(reqwest::Method::GET, format!("{base}/projects/demo/issues"))
            .await
            .expect("req");
        if resp.status().as_u16() == 429 {
            saw_429 = true;
            retry_after_header_present = resp.headers().get("Retry-After").is_some();
            break;
        }
    }
    // The HttpClient request method does not let us attach arbitrary
    // headers, but governor checks on every check(); since rps=2 and we
    // fire 10 sequential requests with the default "anonymous" agent,
    // some must 429.
    //
    // If saw_429 is still false, that means the client wasn't hammering
    // under the same agent bucket — retry with raw reqwest via headers.
    if !saw_429 {
        // Fallback: hit /healthz 20x sharing an agent header. governor
        // Quota::per_second(2) allows burst=2 then throttles.
        for _ in 0..20 {
            let resp = fetch_with_agent(&http, &format!("{base}/healthz"), "hammer").await;
            if resp.0 == 429 {
                saw_429 = true;
                retry_after_header_present = resp.2;
                break;
            }
        }
    }

    assert!(saw_429, "expected at least one 429 across 20 requests");
    assert!(
        retry_after_header_present,
        "Retry-After header must be set on 429"
    );

    handle.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rate_limited_request_is_audited() {
    // rps=1, burst=1. Fire two back-to-back requests with the SAME
    // X-Reposix-Agent header and assert: one succeeds, the second 429s AND
    // the 429 itself is recorded in audit_events (invariant: audit sits
    // OUTSIDE rate-limit).
    let (base, db, handle) = spawn_sim(1).await;
    let http = client(ClientOpts::default()).expect("http");

    let agent = "saturate";

    // First: likely pass.
    let first = fetch_with_agent(&http, &format!("{base}/healthz"), agent).await;
    // Second: likely 429 (burst was consumed by the first call OR by an
    // earlier intervening healthz). We need at least one 429 with this
    // agent.
    let mut got_429 = first.0 == 429;
    if !got_429 {
        for _ in 0..5 {
            let r = fetch_with_agent(&http, &format!("{base}/healthz"), agent).await;
            if r.0 == 429 {
                got_429 = true;
                break;
            }
        }
    }
    assert!(got_429, "expected at least one 429 for agent={agent}");

    // Give the audit middleware a moment to flush.
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let conn = open_audit_conn(db.path());
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM audit_events WHERE status = 429 AND agent_id = ?1",
            rusqlite::params![agent],
            |r| r.get(0),
        )
        .expect("count");
    assert!(
        count >= 1,
        "expected >=1 audit row with status=429 and agent_id={agent}; got {count}"
    );

    handle.abort();
}

// --------- helpers -------------------------------------------------------

/// PATCH with both headers and a request body, routed through the
/// allowlist-gated `HttpClient::request_with_headers_and_body`.
///
/// Returns `(status_u16, body_string)`.
async fn reqwest_patch_with_headers(
    http: &reposix_core::http::HttpClient,
    url: &str,
    body: &str,
    headers: &[(&str, &str)],
) -> (u16, String) {
    let body_bytes: Vec<u8> = body.as_bytes().to_vec();
    let resp = http
        .request_with_headers_and_body(reqwest::Method::PATCH, url, headers, Some(body_bytes))
        .await
        .expect("patch");
    let status = resp.status().as_u16();
    let text = resp.text().await.unwrap_or_default();
    (status, text)
}

async fn fetch_with_agent(
    http: &reposix_core::http::HttpClient,
    url: &str,
    agent: &str,
) -> (u16, String, bool) {
    let resp = http
        .request_with_headers(reqwest::Method::GET, url, &[("X-Reposix-Agent", agent)])
        .await
        .expect("req");
    let retry_after_present = resp.headers().get("Retry-After").is_some();
    let status = resp.status().as_u16();
    let text = resp.text().await.unwrap_or_default();
    (status, text, retry_after_present)
}
