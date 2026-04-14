//! Contract test — the same 5 invariants hold for SimBackend,
//! a wiremock-backed [`ConfluenceReadOnlyBackend`], and (when
//! `#[ignore]`-unlocked + env configured) a live Atlassian tenant.
//!
//! **Why this file exists.** The whole point of the [`IssueBackend`] seam
//! (Phase 8 spec) is that the FUSE daemon and CLI orchestrator don't care
//! which concrete backend they're talking to. Plan 11-A shipped 17 wiremock
//! unit tests for `ConfluenceReadOnlyBackend`, but those exercise *private*
//! helpers through module-internal access; they never drive the adapter
//! through the [`IssueBackend`] trait seam the rest of the codebase
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
//! 4. `get_issue(project, IssueId(u64::MAX))` returns `Err` (the 404 path).
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
//!   [`ConfluenceReadOnlyBackend`]. Always runs.
//! - `contract_confluence_live` — hits a real Atlassian tenant. `#[ignore]`-
//!   gated + `skip_if_no_env!`-guarded so a fresh clone's CI stays green
//!   without any secrets. Opt-in via
//!   `cargo test -p reposix-confluence -- --ignored`.

use std::path::PathBuf;

use reposix_confluence::{ConfluenceCreds, ConfluenceReadOnlyBackend};
use reposix_core::backend::sim::SimBackend;
use reposix_core::backend::IssueBackend;
use reposix_core::{IssueId, IssueStatus};
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

/// The 5 invariants that hold for any well-behaved [`IssueBackend`].
///
/// Every assertion writes its expectation inline so a failing run points
/// directly at the rule that broke, not a distant line of driver code.
/// Shared verbatim with `reposix-github/tests/contract.rs` by intent —
/// the trait's value *is* this shared contract.
async fn assert_contract<B: IssueBackend>(backend: &B, project: &str, known_issue_id: IssueId) {
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
    let missing = backend.get_issue(project, IssueId(u64::MAX)).await;
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
            .map(|r| r.status().is_success())
            .unwrap_or(false)
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
/// "both sides of the IssueBackend seam are contract-testable" within
/// the confluence crate's own CI footprint.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_sim() {
    let (origin, _db, handle) = spawn_sim().await;
    let backend = SimBackend::new(origin).expect("SimBackend::new");

    // The seed fixture at crates/reposix-sim/fixtures/seed.json guarantees
    // id=1 exists in the "demo" project.
    assert_contract(&backend, "demo", IssueId(1)).await;

    handle.abort();
}

// ----------------------------------------------- wiremock-Confluence test

/// Always runs. Mounts the three Confluence v2 endpoints the contract
/// sequence hits (space-key resolver → list pages → get single page)
/// plus a 404 for `IssueId(u64::MAX)` and drives `assert_contract`
/// through [`ConfluenceReadOnlyBackend`].
///
/// Stronger than the unit tests in `lib.rs` because it exercises the
/// full `list_issues → get_issue → get_issue(u64::MAX)` sequence
/// through the [`IssueBackend`] trait seam, not through private
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

    // 3. IssueId(u64::MAX) → 404. MOUNTED BEFORE the id=1 success mount
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

    // 4. get_issue(IssueId(1)) — single page with storage body.
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/pages/1"))
        .and(query_param("body-format", "storage"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "1",
            "status": "current",
            "title": "Home",
            "createdAt": "2024-01-15T10:30:00.000Z",
            "version": {"number": 1, "createdAt": "2024-01-15T10:30:00.000Z"},
            "ownerId": null,
            "body": {"storage": {"value": "<p>home</p>", "representation": "storage"}}
        })))
        .mount(&server)
        .await;

    let creds = ConfluenceCreds {
        email: "ci@example.com".into(),
        api_token: "dummy".into(),
    };
    let backend =
        ConfluenceReadOnlyBackend::new_with_base_url(creds, server.uri()).expect("backend");

    assert_contract(&backend, "REPOSIX", IssueId(1)).await;
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
    let backend = ConfluenceReadOnlyBackend::new(creds, &tenant).expect("backend");

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
