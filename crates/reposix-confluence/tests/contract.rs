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
//! 1. `list_issues(project)` returns `Ok(vec)` for a known-good project.
//! 2. The list is non-empty (≥1 issue).
//! 3. `get_issue(project, known_issue_id)` returns `Ok(issue)` with matching id.
//! 4. `get_issue(project, RecordId(u64::MAX))` returns `Err` (the 404 path).
//! 5. Every listed issue's status is a valid [`IssueStatus`] variant — the
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

use reposix_confluence::{ConfluenceBackend, ConfluenceCreds};
use reposix_core::backend::sim::SimBackend;
use reposix_core::backend::BackendConnector;
use reposix_core::{RecordId, IssueStatus};
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
async fn assert_contract<B: BackendConnector>(backend: &B, project: &str, known_issue_id: RecordId) {
    // (1) list_issues returns Ok(vec).
    let issues = backend.list_issues(project).await.unwrap_or_else(|e| {
        panic!(
            "[{}] list_issues({project}) should be Ok, got {e:?}",
            backend.name()
        )
    });

    // (2) list is non-empty.
    assert!(
        !issues.is_empty(),
        "[{}] list_issues({project}) returned empty — seed/fixture missing?",
        backend.name()
    );

    // (3) get_issue for a known id returns matching id.
    let issue = backend
        .get_issue(project, known_issue_id)
        .await
        .unwrap_or_else(|e| {
            panic!(
                "[{}] get_issue({project}, {known_issue_id}) should be Ok, got {e:?}",
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
    let missing = backend.get_issue(project, RecordId(u64::MAX)).await;
    assert!(
        missing.is_err(),
        "[{}] get_issue({project}, u64::MAX) should be Err, got {missing:?}",
        backend.name()
    );

    // (5) Every listed issue has a valid IssueStatus variant. `match` on
    // the enum proves exhaustiveness at compile time; the explicit arms
    // guard against a future `non_exhaustive` attribute that might weaken
    // the check.
    for i in &issues {
        match i.status {
            IssueStatus::Open
            | IssueStatus::InProgress
            | IssueStatus::InReview
            | IssueStatus::Done
            | IssueStatus::WontFix => {}
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
/// full `list_issues → get_issue → get_issue(u64::MAX)` sequence
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

    // 4. get_issue(RecordId(1)) — single page with ADF body (C4: atlas_doc_format path).
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
        .list_issues("REPOSIX")
        .await
        .expect("list_issues succeeded");
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

    // Also arm get_issue(1) with ADF body — so a contract-style round trip works.
    // C4: get_issue now requests atlas_doc_format first.
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
        .list_issues("REPOSIX")
        .await
        .expect("list_issues succeeded");
    assert_eq!(issues.len(), 1);

    // Round-trip the single page via get_issue too, so the adversarial
    // `webui_link` on the single-page shape gets the same regression
    // coverage as the list shape.
    let single = backend
        .get_issue("REPOSIX", RecordId(1))
        .await
        .expect("get_issue succeeded");
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

    // C4: get_issue now requests atlas_doc_format first. Return ADF body that
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
        .list_issues("REPOSIX")
        .await
        .expect("list_issues succeeded");
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
        .get_issue("REPOSIX", RecordId(1))
        .await
        .expect("get_issue succeeded");
    assert!(single.body.contains("body-exfil"));

    let hits = decoy_server.received_requests().await.unwrap_or_default();
    assert!(
        hits.is_empty(),
        "adapter made {} request(s) to adversarial string-field host: {:?}",
        hits.len(),
        hits.iter().map(|r| r.url.to_string()).collect::<Vec<_>>()
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
/// don't have a canonical "page id 1". The test calls `list_issues`
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
        .list_issues(&space)
        .await
        .unwrap_or_else(|e| panic!("list_issues({space}) failed: {e:?}"));
    assert!(
        !issues.is_empty(),
        "live Confluence space {space} has zero pages"
    );

    let known_id = issues[0].id;
    assert_contract(&backend, &space, known_id).await;
}

// ----------------------------------------------- live-Atlassian hierarchy test

/// Phase 13 Wave B1 extension: prove the adapter populates `Issue::parent_id`
/// from real REST v2 `parentId`/`parentType` bytes, not just wiremock fixtures.
/// The REPOSIX demo space in the reuben-john tenant has homepage `360556`
/// with three children — so at least one listed page MUST have
/// `parent_id == Some(_)` if hierarchy plumbing is live.
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
    let backend = ConfluenceBackend::new(creds, &tenant).expect("backend");

    let issues = backend
        .list_issues(&space)
        .await
        .unwrap_or_else(|e| panic!("list_issues({space}) failed: {e:?}"));
    assert!(
        !issues.is_empty(),
        "live Confluence space {space} has zero pages"
    );

    // The REPOSIX demo space is seeded with homepage 360556 + 3 direct
    // children; any well-configured Confluence space exercised through this
    // test should have at least one non-root page. If this assertion fails,
    // either (a) the space truly is flat, or (b) `parentId` plumbing
    // regressed — both cases warrant loud failure.
    let with_parent = issues.iter().filter(|i| i.parent_id.is_some()).count();
    assert!(
        with_parent >= 1,
        "live Confluence space {space} must have ≥1 page with parent_id populated \
         (hierarchy plumbing check); found {with_parent} of {} pages with a parent",
        issues.len()
    );
}
