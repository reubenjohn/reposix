//! Contract test — the same 5 invariants hold for SimBackend,
//! a wiremock-backed [`ConfluenceBackend`], and (when
//! `#[ignore]`-unlocked + env configured) a live Atlassian tenant.
//!
//! **Why this file exists.** The whole point of the [`BackendConnector`] seam
//! (Phase 8 spec) is that the FUSE daemon and CLI orchestrator don't care
//! which concrete backend they're talking to. Plan 11-A shipped 17 wiremock
//! unit tests for `ConfluenceBackend`, but those exercise *private*
//! helpers through module-internal access; they never drive the adapter
//! through the [`BackendConnector`] trait seam the rest of the codebase
//! actually consumes. This file is that proof.
//!
//! Mirrors `crates/reposix-github/tests/contract.rs` exactly in spirit —
//! the `assert_contract` helper is identical, only the fixture-plumbing
//! differs (wiremock mounts for Confluence, real ephemeral sim for sim).
//!
//! ## The 5 invariants (shared with reposix-github)
//!
//! 1. `list_records(project)` returns `Ok(vec)` for a known-good project.
//! 2. The list is non-empty (≥1 issue).
//! 3. `get_record(project, known_issue_id)` returns `Ok(issue)` with matching id.
//! 4. `get_record(project, RecordId(u64::MAX))` returns `Err` (the 404 path).
//! 5. Every listed issue's status is a valid [`RecordStatus`] variant — the
//!    adapter didn't leave a raw backend-specific string dangling.
//!
//! ## Test arms
//!
//! - `contract_sim` — boots a local `reposix-sim` on an ephemeral port,
//!   runs invariants against [`SimBackend`]. Always runs.
//! - `contract_confluence_wiremock` — mounts the three Confluence v2
//!   endpoints the contract hits (spaces-by-key resolver, list pages, get
//!   page) on a [`MockServer`], drives assert_contract through
//!   [`ConfluenceBackend`]. Always runs.
//! - `contract_confluence_live` — hits a real Atlassian tenant. `#[ignore]`-
//!   gated + `skip_if_no_env!`-guarded so a fresh clone's CI stays green
//!   without any secrets. Opt-in via
//!   `cargo test -p reposix-confluence -- --ignored`.

use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::Mutex;
use reposix_confluence::{ConfluenceBackend, ConfluenceCreds};
use reposix_core::backend::sim::SimBackend;
use reposix_core::backend::{BackendConnector, DeleteReason};
use reposix_core::{sanitize, Record, RecordId, RecordStatus, ServerMetadata, Tainted, Untainted};
use rusqlite::Connection;
use serde_json::json;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Skip the enclosing test if any listed env var is unset or empty.
///
/// Prints a `SKIP: env vars unset: ...` line to stderr (visible with
/// `cargo test -- --nocapture`) and returns from the enclosing fn. Only
/// variable *names* are printed, never values — the whole point of the
/// live test is to exercise secret material, so the names themselves
/// are useful diagnostic output but the values must never be logged.
///
/// Intended for use at the top of `#[ignore]`-gated live-wire tests so
/// that a fresh clone's `cargo test -- --ignored` invocation either
/// runs to completion (all vars set) or skips cleanly (at least one
/// unset), rather than panicking with an opaque "env var not found".
macro_rules! skip_if_no_env {
    ($($var:literal),+ $(,)?) => {{
        let mut missing: Vec<&'static str> = Vec::new();
        $(
            if std::env::var($var).map_or(true, |v| v.is_empty()) {
                missing.push($var);
            }
        )+
        if !missing.is_empty() {
            eprintln!("SKIP: env vars unset: {}", missing.join(", "));
            return;
        }
    }};
}

/// The 5 invariants that hold for any well-behaved [`BackendConnector`].
///
/// Every assertion writes its expectation inline so a failing run points
/// directly at the rule that broke, not a distant line of driver code.
/// Shared verbatim with `reposix-github/tests/contract.rs` by intent —
/// the trait's value *is* this shared contract.
async fn assert_contract<B: BackendConnector>(
    backend: &B,
    project: &str,
    known_issue_id: RecordId,
) {
    // (1) list_records returns Ok(vec).
    let issues = backend.list_records(project).await.unwrap_or_else(|e| {
        panic!(
            "[{}] list_records({project}) should be Ok, got {e:?}",
            backend.name()
        )
    });

    // (2) list is non-empty.
    assert!(
        !issues.is_empty(),
        "[{}] list_records({project}) returned empty — seed/fixture missing?",
        backend.name()
    );

    // (3) get_record for a known id returns matching id.
    let issue = backend
        .get_record(project, known_issue_id)
        .await
        .unwrap_or_else(|e| {
            panic!(
                "[{}] get_record({project}, {known_issue_id}) should be Ok, got {e:?}",
                backend.name()
            )
        });
    assert_eq!(
        issue.id,
        known_issue_id,
        "[{}] round-tripped id mismatch",
        backend.name()
    );

    // (4) u64::MAX is expected to be absent — this is the 404 path.
    let missing = backend.get_record(project, RecordId(u64::MAX)).await;
    assert!(
        missing.is_err(),
        "[{}] get_record({project}, u64::MAX) should be Err, got {missing:?}",
        backend.name()
    );

    // (5) Every listed issue has a valid RecordStatus variant. `match` on
    // the enum proves exhaustiveness at compile time; the explicit arms
    // guard against a future `non_exhaustive` attribute that might weaken
    // the check.
    for i in &issues {
        match i.status {
            RecordStatus::Open
            | RecordStatus::InProgress
            | RecordStatus::InReview
            | RecordStatus::Done
            | RecordStatus::WontFix => {}
        }
    }
}

// ------------------------------------------------------------ sim fixture

fn sim_seed_fixture() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("..");
    p.push("reposix-sim");
    p.push("fixtures");
    p.push("seed.json");
    p
}

/// Spawn a simulator on `127.0.0.1:0` (ephemeral port), return `(origin,
/// db_file, join_handle)`. Dropping the handle aborts the task.
async fn spawn_sim() -> (String, tempfile::NamedTempFile, tokio::task::JoinHandle<()>) {
    let db = tempfile::NamedTempFile::new().expect("tempfile");
    let db_path = db.path().to_owned();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("local_addr");
    let origin = format!("http://{addr}");
    let cfg = reposix_sim::SimConfig {
        bind: addr,
        db_path,
        seed: true,
        seed_file: Some(sim_seed_fixture()),
        ephemeral: false,
        rate_limit_rps: 100,
    };
    let handle = tokio::spawn(async move {
        let _ = reposix_sim::run_with_listener(listener, cfg).await;
    });
    // Poll /healthz until the server is serving.
    let http =
        reposix_core::http::client(reposix_core::http::ClientOpts::default()).expect("http client");
    for _ in 0..40 {
        if http
            .get(format!("{origin}/healthz"))
            .await
            .is_ok_and(|r| r.status().is_success())
        {
            return (origin, db, handle);
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }
    panic!("sim failed to come up at {origin}");
}

// ------------------------------------------------------------ SimBackend test

/// Always runs — no external dependencies, no allowlist env var needed
/// (127.0.0.1 is allowed by default). Included in THIS crate's test
/// suite (not only reposix-github's) because it proves the shared
/// `assert_contract` helper is reusable and asserts the sim half of
/// "both sides of the BackendConnector seam are contract-testable" within
/// the confluence crate's own CI footprint.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_sim() {
    let (origin, _db, handle) = spawn_sim().await;
    let backend = SimBackend::new(origin).expect("SimBackend::new");

    // The seed fixture at crates/reposix-sim/fixtures/seed.json guarantees
    // id=1 exists in the "demo" project.
    assert_contract(&backend, "demo", RecordId(1)).await;

    handle.abort();
}

// ----------------------------------------------- wiremock-Confluence test

/// Always runs. Mounts the three Confluence v2 endpoints the contract
/// sequence hits (space-key resolver → list pages → get single page)
/// plus a 404 for `RecordId(u64::MAX)` and drives `assert_contract`
/// through [`ConfluenceBackend`].
///
/// Stronger than the unit tests in `lib.rs` because it exercises the
/// full `list_records → get_record → get_record(u64::MAX)` sequence
/// through the [`BackendConnector`] trait seam, not through private
/// helpers — the same seam the FUSE daemon and CLI consume.
///
/// ## Mock-ordering note
///
/// Wiremock matches most-recently-mounted-first. The `u64::MAX` 404
/// mount is therefore placed BEFORE the `pages/1` 200 mount so that
/// requests for `pages/1` fall through to the 200 handler and only
/// `pages/{u64::MAX}` is intercepted by the 404.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_confluence_wiremock() {
    let server = MockServer::start().await;

    // 1. resolve space key "REPOSIX" → space id "12345". The adapter
    //    calls this once before its first list and caches the result,
    //    but we mount it without a `.up_to_n_times` cap so repeat
    //    contract runs against the same backend instance still work.
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/spaces"))
        .and(query_param("keys", "REPOSIX"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [{"id": "12345", "key": "REPOSIX"}]
        })))
        .mount(&server)
        .await;

    // 2. list pages (single-page response, no _links.next). Two pages
    //    with different statuses to exercise invariant 5 on both a
    //    `current`→Open and an `archived`→Done mapping.
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/spaces/12345/pages"))
        .and(query_param("limit", "100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [
                {
                    "id": "1",
                    "status": "current",
                    "title": "Home",
                    "createdAt": "2024-01-15T10:30:00.000Z",
                    "version": {"number": 1, "createdAt": "2024-01-15T10:30:00.000Z"},
                    "ownerId": null,
                    "body": {}
                },
                {
                    "id": "2",
                    "status": "archived",
                    "title": "Old Page",
                    "createdAt": "2024-01-15T10:30:00.000Z",
                    "version": {"number": 3, "createdAt": "2024-02-20T14:00:00.000Z"},
                    "ownerId": "557058:abc",
                    "body": {}
                }
            ],
            "_links": {}
        })))
        .mount(&server)
        .await;

    // 3. RecordId(u64::MAX) → 404. MOUNTED BEFORE the id=1 success mount
    //    below so wiremock's most-recently-mounted-first matching
    //    doesn't let the broader path matcher swallow `pages/1`.
    Mock::given(method("GET"))
        .and(path(format!("/wiki/api/v2/pages/{}", u64::MAX)))
        .respond_with(
            ResponseTemplate::new(404)
                .set_body_json(json!({"statusCode": 404, "message": "Not found"})),
        )
        .mount(&server)
        .await;

    // 4. get_record(RecordId(1)) — single page with ADF body (C4: atlas_doc_format path).
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/pages/1"))
        .and(query_param("body-format", "atlas_doc_format"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "1",
            "status": "current",
            "title": "Home",
            "createdAt": "2024-01-15T10:30:00.000Z",
            "version": {"number": 1, "createdAt": "2024-01-15T10:30:00.000Z"},
            "ownerId": null,
            "body": {
                "atlas_doc_format": {
                    "value": {
                        "type": "doc",
                        "version": 1,
                        "content": [
                            {
                                "type": "paragraph",
                                "content": [{"type": "text", "text": "home"}]
                            }
                        ]
                    },
                    "representation": "atlas_doc_format"
                }
            }
        })))
        .mount(&server)
        .await;

    let creds = ConfluenceCreds {
        email: "ci@example.com".into(),
        api_token: "dummy".into(),
    };
    let backend = ConfluenceBackend::new_with_base_url(creds, server.uri()).expect("backend");

    assert_contract(&backend, "REPOSIX", RecordId(1)).await;
}

// ----------------------------------------------- SSRF regression tests
//
// These are regression guards for OP-7 bullet 5 (HANDOFF.md lines 401–412).
// WR-02 already validates that `space_id` is server-resolved rather than
// client-trusted. The uncovered surface was attacker-controlled URL fields
// in Confluence v2 response JSON — specifically `_links.base`, `webui_link`,
// and free-form string fields (title, body, ownerId, parentId) that a
// future feature like "follow the webui_link for a page screenshot" would
// reopen as an SSRF vector.
//
// Strategy: for each adversarial field, mount two `MockServer`s:
//   * `legit_server` — the Confluence-shaped backend the adapter talks to.
//     Returns valid responses but EMBEDS adversarial URL fields pointing
//     at `decoy_server`.
//   * `decoy_server`  — a catch-all mock on 127.0.0.1 (so it's reachable
//     through the default SG-01 allowlist) that `.expect(0)`s. Wiremock
//     panics on `MockServer::drop` if the expectation isn't met, so any
//     accidental follow by the adapter surfaces as a test failure.
//
// This is a defense-in-depth test — today the `ConfPage`/`ConfLinks` structs
// don't even deserialize `_links.base` or `webui_link`, so the adapter
// literally cannot follow them. If a future PR adds those fields to the
// deserialized shape without also adding explicit allowlist checks, these
// tests fire.

/// Wiremock: page list response whose `_links.base` points at the decoy
/// server. If the adapter ever starts trusting `_links.base` for
/// subsequent requests, the decoy's `.expect(0)` will fail.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn adversarial_links_base_does_not_trigger_outbound_call() {
    let legit_server = MockServer::start().await;
    let decoy_server = MockServer::start().await;

    // Decoy: any request whatsoever means the adapter followed an
    // attacker-controlled URL. `.expect(0)` panics on drop if hit.
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string("exfiltrated"))
        .expect(0)
        .mount(&decoy_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/spaces"))
        .and(query_param("keys", "REPOSIX"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [{"id": "12345", "key": "REPOSIX"}],
            // Adversarial: `_links.base` at the top level of the space
            // list response. Real Confluence puts a `base` here; the
            // adapter must ignore it.
            "_links": {"base": decoy_server.uri()}
        })))
        .mount(&legit_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/spaces/12345/pages"))
        .and(query_param("limit", "100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [{
                "id": "1",
                "status": "current",
                "title": "Home",
                "createdAt": "2024-01-15T10:30:00.000Z",
                "version": {"number": 1, "createdAt": "2024-01-15T10:30:00.000Z"},
                "ownerId": null,
                "body": {}
            }],
            // Adversarial: both the global `base` and a per-page
            // `_links.base` shape Confluence sometimes emits.
            "_links": {
                "base": decoy_server.uri(),
                "self": format!("{}/self", decoy_server.uri())
            }
        })))
        .mount(&legit_server)
        .await;

    let creds = ConfluenceCreds {
        email: "ci@example.com".into(),
        api_token: "dummy".into(),
    };
    let backend = ConfluenceBackend::new_with_base_url(creds, legit_server.uri()).expect("backend");

    let issues = backend
        .list_records("REPOSIX")
        .await
        .expect("list_records succeeded");
    assert_eq!(issues.len(), 1, "one page in fixture");
    assert_eq!(issues[0].id, RecordId(1));

    // Explicit sanity check — wiremock also verifies on drop, but an
    // inline assertion produces a clearer failure message if the decoy
    // was somehow touched.
    let hits = decoy_server.received_requests().await.unwrap_or_default();
    assert!(
        hits.is_empty(),
        "adapter made {} request(s) to adversarial _links.base host: {:?}",
        hits.len(),
        hits.iter().map(|r| r.url.to_string()).collect::<Vec<_>>()
    );
}

/// Wiremock: page list response whose per-page `webui_link` / `_links.webui`
/// fields point at the decoy server. A future "follow webui_link for a
/// thumbnail" feature would reopen SSRF; this test is the tripwire.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn adversarial_webui_link_does_not_trigger_outbound_call() {
    let legit_server = MockServer::start().await;
    let decoy_server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string("exfiltrated"))
        .expect(0)
        .mount(&decoy_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/spaces"))
        .and(query_param("keys", "REPOSIX"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [{"id": "12345", "key": "REPOSIX"}]
        })))
        .mount(&legit_server)
        .await;

    let decoy = decoy_server.uri();

    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/spaces/12345/pages"))
        .and(query_param("limit", "100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [{
                "id": "1",
                "status": "current",
                "title": "Home",
                "createdAt": "2024-01-15T10:30:00.000Z",
                "version": {"number": 1, "createdAt": "2024-01-15T10:30:00.000Z"},
                "ownerId": null,
                "body": {},
                // Adversarial URL fields that Confluence v2 / v1 have
                // emitted at various times. None of these should trigger
                // an outbound call today; the test locks that in.
                "webui_link": format!("{decoy}/exfil/webui"),
                "_links": {
                    "webui": format!("{decoy}/exfil/_links.webui"),
                    "tinyui": format!("{decoy}/exfil/tinyui"),
                    "self": format!("{decoy}/exfil/self"),
                    "edit": format!("{decoy}/exfil/edit")
                }
            }],
            "_links": {}
        })))
        .mount(&legit_server)
        .await;

    // Also arm get_record(1) with ADF body — so a contract-style round trip works.
    // C4: get_record now requests atlas_doc_format first.
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/pages/1"))
        .and(query_param("body-format", "atlas_doc_format"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "1",
            "status": "current",
            "title": "Home",
            "createdAt": "2024-01-15T10:30:00.000Z",
            "version": {"number": 1, "createdAt": "2024-01-15T10:30:00.000Z"},
            "ownerId": null,
            "body": {
                "atlas_doc_format": {
                    "value": {
                        "type": "doc",
                        "version": 1,
                        "content": [
                            {"type": "paragraph", "content": [{"type": "text", "text": "home"}]}
                        ]
                    },
                    "representation": "atlas_doc_format"
                }
            },
            "webui_link": format!("{decoy}/exfil/page1/webui"),
            "_links": {
                "webui": format!("{decoy}/exfil/page1/_links.webui"),
                "self": format!("{decoy}/exfil/page1/self")
            }
        })))
        .mount(&legit_server)
        .await;

    let creds = ConfluenceCreds {
        email: "ci@example.com".into(),
        api_token: "dummy".into(),
    };
    let backend = ConfluenceBackend::new_with_base_url(creds, legit_server.uri()).expect("backend");

    let issues = backend
        .list_records("REPOSIX")
        .await
        .expect("list_records succeeded");
    assert_eq!(issues.len(), 1);

    // Round-trip the single page via get_record too, so the adversarial
    // `webui_link` on the single-page shape gets the same regression
    // coverage as the list shape.
    let single = backend
        .get_record("REPOSIX", RecordId(1))
        .await
        .expect("get_record succeeded");
    assert_eq!(single.id, RecordId(1));

    let hits = decoy_server.received_requests().await.unwrap_or_default();
    assert!(
        hits.is_empty(),
        "adapter made {} request(s) to adversarial webui_link host: {:?}",
        hits.len(),
        hits.iter().map(|r| r.url.to_string()).collect::<Vec<_>>()
    );
}

/// Broader regression: the adapter treats arbitrary string fields —
/// `title`, `body.storage.value`, `ownerId`, `parentId` — as opaque bytes.
/// Feeding fully-qualified URLs into any of them must not provoke an
/// outbound call. Catches a class of bug where a future "auto-resolve
/// mentions" or "linkify body text" feature silently adds URL-following.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn adversarial_host_in_arbitrary_string_field_is_ignored() {
    let legit_server = MockServer::start().await;
    let decoy_server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string("exfiltrated"))
        .expect(0)
        .mount(&decoy_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/spaces"))
        .and(query_param("keys", "REPOSIX"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [{"id": "12345", "key": "REPOSIX"}]
        })))
        .mount(&legit_server)
        .await;

    let decoy = decoy_server.uri();

    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/spaces/12345/pages"))
        .and(query_param("limit", "100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [{
                "id": "1",
                "status": "current",
                // URL in title — agents rendering issue trees might be
                // tempted to pre-fetch linked resources. Regression guard.
                "title": format!("Home (see {decoy}/title-exfil)"),
                "createdAt": "2024-01-15T10:30:00.000Z",
                "version": {"number": 1, "createdAt": "2024-01-15T10:30:00.000Z"},
                // URL-shaped ownerId; adapter stores it as an opaque
                // string and must not resolve it to a profile URL.
                "ownerId": format!("{decoy}/owner-exfil"),
                // parentId is supposed to be a numeric string. Feed a
                // URL to prove the adapter's numeric parse fails
                // gracefully without attempting to fetch the string.
                "parentId": format!("{decoy}/parent-exfil"),
                "parentType": "page",
                "body": {}
            }],
            "_links": {}
        })))
        .mount(&legit_server)
        .await;

    // C4: get_record now requests atlas_doc_format first. Return ADF body that
    // contains the adversarial URL as plain text so the body-exfil assertion
    // still passes (adapter must not resolve the URL, just pass it through).
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/pages/1"))
        .and(query_param("body-format", "atlas_doc_format"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "1",
            "status": "current",
            "title": format!("Home (see {decoy}/title-exfil)"),
            "createdAt": "2024-01-15T10:30:00.000Z",
            "version": {"number": 1, "createdAt": "2024-01-15T10:30:00.000Z"},
            "ownerId": format!("{decoy}/owner-exfil"),
            "body": {
                "atlas_doc_format": {
                    "value": {
                        "type": "doc",
                        "version": 1,
                        "content": [{
                            "type": "paragraph",
                            "content": [{
                                "type": "text",
                                // Body text contains the adversarial URL as a plain text node.
                                // adf_to_markdown will pass it through verbatim — no fetching.
                                "text": format!("visit {decoy}/body-exfil for details")
                            }]
                        }]
                    },
                    "representation": "atlas_doc_format"
                }
            }
        })))
        .mount(&legit_server)
        .await;

    let creds = ConfluenceCreds {
        email: "ci@example.com".into(),
        api_token: "dummy".into(),
    };
    let backend = ConfluenceBackend::new_with_base_url(creds, legit_server.uri()).expect("backend");

    let issues = backend
        .list_records("REPOSIX")
        .await
        .expect("list_records succeeded");
    assert_eq!(issues.len(), 1);
    // parentId was a URL, not a numeric string → degrades to orphan
    // (see lib.rs `translate` comment on T-13-PB1 graceful degradation).
    assert_eq!(
        issues[0].parent_id, None,
        "non-numeric parentId must degrade to None, not propagate the URL"
    );
    // title and body round-trip as opaque strings — URL-shaped content
    // is preserved verbatim (no linkification, no pre-fetch).
    assert!(issues[0].title.contains("title-exfil"));

    let single = backend
        .get_record("REPOSIX", RecordId(1))
        .await
        .expect("get_record succeeded");
    assert!(single.body.contains("body-exfil"));

    let hits = decoy_server.received_requests().await.unwrap_or_default();
    assert!(
        hits.is_empty(),
        "adapter made {} request(s) to adversarial string-field host: {:?}",
        hits.len(),
        hits.iter().map(|r| r.url.to_string()).collect::<Vec<_>>()
    );
}

// ----------------------------------------------- render-parity (FIX-01)

/// FIX-01 render-parity regression (Phase 114). The Confluence `list_records`
/// path and the `get_record` path MUST request the SAME body representation so
/// an unmutated ADF-native page renders byte-identical bytes on both paths.
///
/// ## The defect this locks out
///
/// Before the fix, `list_issues_impl` requested the space's pages with NO
/// `body-format` query param, so Confluence returned every page body EMPTY;
/// `build_from` hashed that empty-body list render into the tree-blob oid, then
/// `read_blob` re-fetched via `get_record` (which DOES send
/// `?body-format=atlas_doc_format`), got the REAL body, and hard-aborted with
/// `Error::OidDrift` — deterministically, on every ADF-native page. This test
/// mounts the LIST mock behind `query_param("body-format", "atlas_doc_format")`
/// so it only matches once the adapter sends that param; pre-fix the LIST
/// request carries only `limit=100`, misses the mock, gets a 404 and
/// `list_records` errors (the RED). Post-fix the param is present, the mock
/// matches, and the list body decodes non-empty AND byte-equal to the get body.
///
/// FIDELITY: both mocks string-encode `atlas_doc_format.value` (a JSON *string*
/// whose contents are JSON) to match the live v2 API — the same decode path
/// production walks, not an object-encoded fixture that would mask a
/// string-decode regression (see `ConfBodyAdf`'s wire-encoding docs).
#[tokio::test]
async fn list_and_get_render_parity() {
    let server = MockServer::start().await;

    // The single ADF document both the LIST and the GET return for page 1.
    // Real content so the decoded Markdown body is non-empty.
    let adf_doc = json!({
        "type": "doc",
        "version": 1,
        "content": [
            {
                "type": "paragraph",
                "content": [{"type": "text", "text": "render parity body"}]
            }
        ]
    });
    // Live-API fidelity: `atlas_doc_format.value` is a JSON *string*, not object.
    let page_body = json!({
        "atlas_doc_format": {
            "value": adf_doc.to_string(),
            "representation": "atlas_doc_format"
        }
    });

    // (1) space-key resolve.
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/spaces"))
        .and(query_param("keys", "REPOSIX"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [{"id": "12345", "key": "REPOSIX"}],
            "_links": {}
        })))
        .mount(&server)
        .await;

    // (2) LIST — gated on body-format=atlas_doc_format. Pre-fix `list_issues_impl`
    // sends only `limit=100`, so this mock does not match → 404 → `list_records`
    // errors (RED). Post-fix the param is present and the list body populates.
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/spaces/12345/pages"))
        .and(query_param("limit", "100"))
        .and(query_param("body-format", "atlas_doc_format"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [{
                "id": "1",
                "status": "current",
                "title": "render parity page",
                "createdAt": "2026-04-13T00:00:00Z",
                "version": {"number": 1, "createdAt": "2026-04-13T00:00:00Z"},
                "ownerId": null,
                "body": page_body.clone(),
            }],
            "_links": {}
        })))
        .mount(&server)
        .await;

    // (3) GET — the SAME ADF body as the LIST page (byte-identical parity target).
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/pages/1"))
        .and(query_param("body-format", "atlas_doc_format"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "1",
            "status": "current",
            "title": "render parity page",
            "createdAt": "2026-04-13T00:00:00Z",
            "version": {"number": 1, "createdAt": "2026-04-13T00:00:00Z"},
            "ownerId": null,
            "body": page_body,
        })))
        .mount(&server)
        .await;

    let creds = ConfluenceCreds {
        email: "ci@example.com".into(),
        api_token: "dummy".into(),
    };
    let backend = ConfluenceBackend::new_with_base_url(creds, server.uri()).expect("backend");

    let recs = backend
        .list_records("REPOSIX")
        .await
        .expect("list_records must succeed once the LIST url carries body-format");
    assert_eq!(recs.len(), 1, "one-page fixture");

    let got = backend
        .get_record("REPOSIX", recs[0].id)
        .await
        .expect("get_record must succeed");

    // The list body must be populated — the defect blanked it.
    assert!(
        !recs[0].body.is_empty(),
        "list body must be populated post-fix; an empty list body IS the OidDrift defect"
    );
    // ... and byte-identical to the get body. This equality is exactly the
    // property that makes the tree-blob oid match on both the `build_from`
    // (list) path and the `read_blob` (get) path — a mismatch is the oid-drift
    // root cause.
    assert_eq!(
        recs[0].body, got.body,
        "list render must equal get render (parity); a mismatch is the oid-drift root cause"
    );
}

// ----------------------------------------------- live-Atlassian test

/// Hits a real Atlassian tenant. `#[ignore]`-gated + `skip_if_no_env!`-
/// guarded so a fresh clone's CI can be green without any secrets set.
///
/// ## Required env vars
///
/// - `ATLASSIAN_API_KEY` — API token from id.atlassian.com.
/// - `ATLASSIAN_EMAIL` — account email that issued the token.
/// - `REPOSIX_CONFLUENCE_TENANT` — your `<tenant>.atlassian.net` subdomain.
/// - `REPOSIX_CONFLUENCE_SPACE` — a space key that exists in the tenant.
/// - `REPOSIX_ALLOWED_ORIGINS` — must contain `https://<tenant>.atlassian.net`.
///
/// The test passes trivially (SKIP) if any of the first four are missing.
/// The allowlist one is `HttpClient`-enforced and surfaces as a real
/// test failure if mis-set — that's the correct behavior because it
/// means the invoker EXPECTED a live run (they unlocked `--ignored`)
/// but the env is misconfigured.
///
/// ## Known-id strategy
///
/// Unlike `octocat/Hello-World` on GitHub, real Confluence spaces
/// don't have a canonical "page id 1". The test calls `list_records`
/// first, asserts the list is non-empty, and uses the first returned
/// id as the `known_issue_id` argument to `assert_contract`. That
/// double-lists the space (once as setup, once inside assert_contract),
/// which is acceptable: Atlassian's 1000 req/min soft cap has plenty
/// of headroom, and the extra round-trip makes the test self-
/// configuring across tenants.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore]
async fn contract_confluence_live() {
    skip_if_no_env!(
        "ATLASSIAN_API_KEY",
        "ATLASSIAN_EMAIL",
        "REPOSIX_CONFLUENCE_TENANT",
        "REPOSIX_CONFLUENCE_SPACE",
    );

    let origins = std::env::var("REPOSIX_ALLOWED_ORIGINS").unwrap_or_default();
    let tenant = std::env::var("REPOSIX_CONFLUENCE_TENANT").unwrap();
    let expected = format!("{tenant}.atlassian.net");
    assert!(
        origins.contains(&expected),
        "contract_confluence_live requires REPOSIX_ALLOWED_ORIGINS to include \
         https://{expected}; got {origins:?}"
    );

    let creds = ConfluenceCreds {
        email: std::env::var("ATLASSIAN_EMAIL").unwrap(),
        api_token: std::env::var("ATLASSIAN_API_KEY").unwrap(),
    };
    let space = std::env::var("REPOSIX_CONFLUENCE_SPACE").unwrap();
    let backend = ConfluenceBackend::new(creds, &tenant).expect("backend");

    // Pick a known id by listing first — real spaces don't have a
    // canonical "issue 1" the way octocat/Hello-World does.
    let issues = backend
        .list_records(&space)
        .await
        .unwrap_or_else(|e| panic!("list_records({space}) failed: {e:?}"));
    assert!(
        !issues.is_empty(),
        "live Confluence space {space} has zero pages"
    );

    let known_id = issues[0].id;
    assert_contract(&backend, &space, known_id).await;
}

// ----------------------------------------------- live-Atlassian hierarchy test

/// The D91-08 protected durable fixture pair (space `REPOSIX`, id `360450`,
/// tenant `reuben-john`), labeled `reposix-durable-fixture` — documented in
/// `docs/reference/testing-targets.md` § "Protected durable fixtures". This
/// test NEVER deletes either id (see `contract_confluence_live_hierarchy` below).
const DURABLE_PARENT_ID: RecordId = RecordId(7_766_017);
const DURABLE_CHILD_ID: RecordId = RecordId(7_798_785);

/// Build an [`Untainted<Record>`] for the D91-08 self-seeding hierarchy
/// test. `parent` is `None` for the parent page itself, `Some(parent_id)`
/// for the child. Labeled `kind=test` per `docs/reference/testing-targets.md`
/// so a cleanup sweep can locate it (distinct from the durable fixture's
/// `reposix-durable-fixture` label, which cleanup sweeps must spare).
fn make_hierarchy_issue(title: &str, parent: Option<RecordId>) -> Untainted<Record> {
    let t = chrono::Utc::now();
    sanitize(
        Tainted::new(Record {
            id: RecordId(0),
            title: title.to_owned(),
            status: RecordStatus::Open,
            assignee: None,
            labels: vec!["kind=test".to_owned()],
            created_at: t,
            updated_at: t,
            version: 0,
            body: "D91-08 self-seeded hierarchy fixture \
                   (contract_confluence_live_hierarchy); safe to delete."
                .to_owned(),
            parent_id: parent,
            extensions: std::collections::BTreeMap::new(),
        }),
        ServerMetadata {
            id: RecordId(0),
            created_at: t,
            updated_at: t,
            version: 1,
        },
    )
}

/// Open an in-memory `SQLite` DB with the audit schema loaded, ready for
/// [`ConfluenceBackend::with_audit`] (OP-3: mutation calls below must land
/// `audit_events` rows).
fn open_audit_db() -> Arc<Mutex<Connection>> {
    let conn = Connection::open_in_memory().expect("in-memory db");
    reposix_core::audit::load_schema(&conn).expect("load audit schema");
    Arc::new(Mutex::new(conn))
}

/// D91-08: prove the adapter populates `Record::parent_id` from real REST v2
/// `parentId`/`parentType` bytes against a live tenant, WITHOUT depending on
/// whatever the configured space happens to contain that day (the original
/// v0.13.0-era version of this test asserted `≥1` page with `parent_id.is_some()`
/// across the whole space's listing — read-only against live state it didn't
/// own, and it broke CI run `28692818500` when that ambient state changed;
/// see `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md`).
///
/// Hybrid strategy (avoids Confluence-space clutter across repeated runs):
/// 1. `get_record` both halves of the durable fixture pair
///    (`DURABLE_PARENT_ID`/`DURABLE_CHILD_ID`, see `docs/reference/testing-targets.md`
///    § "Protected durable fixtures"). If BOTH resolve, assert the child's
///    `parent_id` read-only and return — no mutation, no teardown, the
///    durable pair is untouched.
/// 2. If either is missing, self-seed a FRESH `kind=test`-labeled parent+child
///    pair via `create_record`, assert immediately, then delete BOTH in
///    teardown. The durable fixture ids are NEVER created, mutated, or
///    deleted by this path.
///
/// Same `#[ignore]` + `skip_if_no_env!` gating as `contract_confluence_live`.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore]
async fn contract_confluence_live_hierarchy() {
    skip_if_no_env!(
        "ATLASSIAN_API_KEY",
        "ATLASSIAN_EMAIL",
        "REPOSIX_CONFLUENCE_TENANT",
        "REPOSIX_CONFLUENCE_SPACE",
    );

    let origins = std::env::var("REPOSIX_ALLOWED_ORIGINS").unwrap_or_default();
    let tenant = std::env::var("REPOSIX_CONFLUENCE_TENANT").unwrap();
    let expected = format!("{tenant}.atlassian.net");
    assert!(
        origins.contains(&expected),
        "contract_confluence_live_hierarchy requires REPOSIX_ALLOWED_ORIGINS to include \
         https://{expected}; got {origins:?}"
    );

    let creds = ConfluenceCreds {
        email: std::env::var("ATLASSIAN_EMAIL").unwrap(),
        api_token: std::env::var("ATLASSIAN_API_KEY").unwrap(),
    };
    let space = std::env::var("REPOSIX_CONFLUENCE_SPACE").unwrap();
    let audit_conn = open_audit_db();
    let backend = ConfluenceBackend::new(creds, &tenant)
        .expect("backend")
        .with_audit(Arc::clone(&audit_conn));

    // Step 1: verify the durable fixture pair still exists (read-only).
    let durable_parent = backend.get_record(&space, DURABLE_PARENT_ID).await;
    let durable_child = backend.get_record(&space, DURABLE_CHILD_ID).await;
    if let (Ok(_parent), Ok(child)) = (&durable_parent, &durable_child) {
        assert_eq!(
            child.parent_id,
            Some(DURABLE_PARENT_ID),
            "durable fixture child {} must have parent_id == Some({}) \
             (docs/reference/testing-targets.md § Protected durable fixtures); \
             this test NEVER deletes either id",
            DURABLE_CHILD_ID.0,
            DURABLE_PARENT_ID.0,
        );
        eprintln!(
            "contract_confluence_live_hierarchy: durable fixture pair {}/{} present in space {space}; \
             verified read-only, no mutation, no teardown",
            DURABLE_PARENT_ID.0, DURABLE_CHILD_ID.0,
        );
        return;
    }

    // Step 2: durable pair missing (or one half is) — self-seed a fresh pair.
    eprintln!(
        "contract_confluence_live_hierarchy: durable fixture pair {}/{} not fully present in space \
         {space} (parent: {:?}, child: {:?}); self-seeding a fresh kind=test pair",
        DURABLE_PARENT_ID.0,
        DURABLE_CHILD_ID.0,
        durable_parent.is_ok(),
        durable_child.is_ok(),
    );

    let parent = backend
        .create_record(
            &space,
            make_hierarchy_issue("D91-08 self-seed parent", None),
        )
        .await
        .unwrap_or_else(|e| panic!("create_record(parent) failed: {e:?}"));
    let child = backend
        .create_record(
            &space,
            make_hierarchy_issue("D91-08 self-seed child", Some(parent.id)),
        )
        .await
        .unwrap_or_else(|e| panic!("create_record(child) failed: {e:?}"));

    assert_eq!(
        child.parent_id,
        Some(parent.id),
        "self-seeded child {} must have parent_id == Some({}) immediately after create_record",
        child.id.0,
        parent.id.0,
    );

    // Teardown: delete BOTH self-seeded pages. Never DURABLE_PARENT_ID/DURABLE_CHILD_ID.
    for id in [child.id, parent.id] {
        if let Err(e) = backend
            .delete_or_close(&space, id, DeleteReason::Abandoned)
            .await
        {
            eprintln!(
                "contract_confluence_live_hierarchy: teardown delete_or_close({}) failed \
                 (non-fatal, leaves a kind=test page for manual cleanup): {e:?}",
                id.0
            );
        }
    }

    // OP-3: the 2 create_record mutations above must have landed audit_events rows.
    let audit_count: i64 = audit_conn
        .lock()
        .query_row(
            "SELECT COUNT(*) FROM audit_events WHERE method = 'POST'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    assert!(
        audit_count >= 2,
        "expected >=2 audit_events POST rows for the 2 self-seed create_record calls, got {audit_count} \
         (OP-3: mutations must land dual audit rows)"
    );
}
