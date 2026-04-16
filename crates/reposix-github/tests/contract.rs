//! Contract test — the same 5 invariants hold for any well-behaved
//! [`IssueBackend`] implementation.
//!
//! **Why this file exists.** The whole point of the `IssueBackend` seam
//! (Phase 8 spec) is that the FUSE daemon and CLI orchestrator don't care
//! which concrete backend they're talking to. But that promise is vacuous
//! unless we can *demonstrate* two implementations pass the same test. This
//! file is that demonstration.
//!
//! The shared `assert_contract` helper codifies what "well-behaved" means:
//!
//! 1. `list_issues(project)` returns `Ok(vec)` for a known-good project.
//! 2. The list is non-empty (≥1 issue).
//! 3. `get_issue(project, known_issue_id)` returns `Ok(issue)` with matching
//!    id.
//! 4. `get_issue(project, IssueId(u64::MAX))` returns `Err` (the 404 path).
//! 5. Every listed issue's status is a valid [`IssueStatus`] variant — the
//!    adapter didn't leave a raw backend-specific string dangling.
//!
//! Two concrete tests run the helper:
//!
//! - `contract_sim` — boots a local `reposix-sim` on an ephemeral port,
//!   seeds 3 issues, runs the invariants against a [`SimBackend`]. Runs in
//!   every CI invocation.
//! - `contract_github` — hits real `octocat/Hello-World` via
//!   [`GithubReadOnlyBackend`]. `#[ignore]`-gated because (a)
//!   unauthenticated GitHub allows 60 req/hr per IP and CI can exhaust
//!   that under concurrent job scheduling, and (b) running against real
//!   GitHub requires the allowlist env var. The opt-in
//!   `cargo test -- --ignored` invocation is the signal the caller has
//!   set both.

use std::path::PathBuf;

use reposix_core::backend::sim::SimBackend;
use reposix_core::backend::IssueBackend;
use reposix_core::{IssueId, IssueStatus};
use reposix_github::GithubReadOnlyBackend;
use serde_json::json;
use wiremock::matchers::{any, header_exists, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// The 5 invariants that hold for any well-behaved [`IssueBackend`].
///
/// Every assertion writes its expectation inline so a failing run points
/// directly at the rule that broke, not a distant line of driver code.
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
/// (127.0.0.1 is allowed by default).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_sim() {
    let (origin, _db, handle) = spawn_sim().await;
    let backend = SimBackend::new(origin).expect("SimBackend::new");

    // The seed fixture at crates/reposix-sim/fixtures/seed.json guarantees
    // id=1 exists in the "demo" project.
    assert_contract(&backend, "demo", IssueId(1)).await;

    handle.abort();
}

// --------------------------------------------------- wiremock fixtures

/// Build a minimal GitHub Issues v3 JSON shape. Mirrors the helper in
/// `src/lib.rs#tests::gh_issue_json` but lives in this integration-test
/// crate so the file is self-contained.
fn gh_issue(
    number: u64,
    state: &str,
    state_reason: Option<&str>,
    assignee_login: Option<&str>,
) -> serde_json::Value {
    let assignee = match assignee_login {
        Some(login) => json!({
            "login": login,
            // Adversarial: GitHub REST emits these URL fields. They MUST NOT
            // trigger outbound calls. The SSRF-regression test below mounts
            // a decoy at the host these values point to and asserts zero
            // hits, but every fixture seeds the same shape so the round-trip
            // mapping covers them too.
            "id": 1,
            "url": "http://decoy.invalid/users/x",
            "html_url": "http://decoy.invalid/x",
            "avatar_url": "http://decoy.invalid/avatar.png",
        }),
        None => serde_json::Value::Null,
    };
    json!({
        "number": number,
        "title": format!("issue {number}"),
        "state": state,
        "state_reason": state_reason,
        "body": "some body",
        "labels": [],
        "assignee": assignee,
        "created_at": "2026-04-13T00:00:00Z",
        "updated_at": "2026-04-13T00:00:00Z",
        // Adversarial URL fields GitHub puts on every issue — same SSRF
        // tripwire as the per-assignee fields above.
        "url": "http://decoy.invalid/repos/o/r/issues/1",
        "html_url": "http://decoy.invalid/o/r/issues/1",
        "comments_url": "http://decoy.invalid/repos/o/r/issues/1/comments",
        "events_url": "http://decoy.invalid/repos/o/r/issues/1/events",
    })
}

// --------------------------------------------------- wiremock contract test

/// Always runs. Mounts the GitHub REST v3 endpoints the contract sequence
/// hits (list issues, get one issue, get-by-`u64::MAX` 404) on a
/// [`MockServer`] and drives `assert_contract` through
/// [`GithubReadOnlyBackend`].
///
/// Stronger than the unit tests in `lib.rs` because it exercises the
/// full `list_issues → get_issue → get_issue(u64::MAX)` sequence
/// through the [`IssueBackend`] trait seam, not through private
/// helpers — the same seam the FUSE daemon and CLI consume.
///
/// Closes OP-6 MEDIUM-13 (HANDOFF.md) — gives `reposix-github`
/// an always-run wiremock contract sibling to the `#[ignore]`-gated
/// live test, mirroring the `contract_confluence_wiremock` shape.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_github_wiremock() {
    let server = MockServer::start().await;

    // u64::MAX 404. MOUNTED FIRST so wiremock's most-recently-mounted-first
    // matching doesn't let the broader `/issues/1` mount swallow it.
    Mock::given(method("GET"))
        .and(path(format!("/repos/o/r/issues/{}", u64::MAX)))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({"message": "Not Found"})))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/repos/o/r/issues"))
        .and(query_param("state", "all"))
        .and(query_param("per_page", "100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            gh_issue(1, "open", None, Some("octocat")),
            gh_issue(2, "closed", Some("completed"), None),
            gh_issue(3, "closed", Some("not_planned"), None),
        ])))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/repos/o/r/issues/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(gh_issue(
            1,
            "open",
            None,
            Some("octocat"),
        )))
        .mount(&server)
        .await;

    let backend = GithubReadOnlyBackend::new_with_base_url(None, server.uri()).expect("backend");
    assert_contract(&backend, "o/r", IssueId(1)).await;
}

// --------------------------------------------------- pagination

/// `list_issues` follows the `Link: <url>; rel="next"` header through
/// multiple pages and concatenates results. Mirrors the inline unit test
/// in `lib.rs` but exercises the full `IssueBackend::list_issues` seam,
/// confirming the contract holds at the trait boundary too.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn pagination_follows_link_header() {
    let server = MockServer::start().await;
    let next_url = format!(
        "{}/repos/o/r/issues?state=all&per_page=100&page=2",
        server.uri()
    );
    let link_val = format!("<{next_url}>; rel=\"next\"");

    // Page 1 — only matches the FIRST request (no `page` query param).
    // Wiremock matches most-recently-mounted-first, so we mount the
    // narrower (`page=2`) matcher AFTER this broader one.
    Mock::given(method("GET"))
        .and(path("/repos/o/r/issues"))
        .and(query_param("state", "all"))
        .and(query_param("per_page", "100"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Link", link_val.as_str())
                .set_body_json(json!([
                    gh_issue(1, "open", None, None),
                    gh_issue(2, "open", None, None),
                ])),
        )
        .up_to_n_times(1)
        .mount(&server)
        .await;

    // Page 2 — narrower match (includes `page=2`). No further `Link` header,
    // so pagination terminates.
    Mock::given(method("GET"))
        .and(path("/repos/o/r/issues"))
        .and(query_param("page", "2"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!([gh_issue(3, "open", None, None)])),
        )
        .mount(&server)
        .await;

    let backend = GithubReadOnlyBackend::new_with_base_url(None, server.uri()).expect("backend");
    let issues = backend.list_issues("o/r").await.expect("list");
    assert_eq!(issues.len(), 3, "expected page1 (2) + page2 (1) = 3 issues");
    assert_eq!(issues[0].id, IssueId(1));
    assert_eq!(issues[2].id, IssueId(3));
}

// --------------------------------------------------- rate-limit 429

/// **Documents current behavior** (regression guard, not requirement).
///
/// The adapter currently treats GitHub's `429 Too Many Requests` like any
/// other non-2xx status: it surfaces an `Error::Other("github returned
/// 429 …")` without honoring `Retry-After` or auto-retrying. The
/// `rate_limit_gate` field only arms on `x-ratelimit-remaining: 0` — see
/// `ingest_rate_limit` in `lib.rs`.
///
/// TODO(future): if we add `Retry-After`-aware retry, flip this test to
/// assert that the adapter sleeps and the second response succeeds.
/// For now, locking in "429 → clean error, no retry" prevents an
/// accidental silent-retry loop that could amplify rate-limit pressure.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rate_limit_429_surfaces_clean_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/o/r/issues/42"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "1")
                .set_body_json(json!({"message": "API rate limit exceeded"})),
        )
        .mount(&server)
        .await;

    let backend = GithubReadOnlyBackend::new_with_base_url(None, server.uri()).expect("backend");
    let err = backend
        .get_issue("o/r", IssueId(42))
        .await
        .expect_err("429 must surface as Err with current adapter");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("429"),
        "error must mention the upstream status; got {msg}"
    );
}

// --------------------------------------------------- state_reason mapping

/// `state_reason` mapping per ADR-001 (`docs/decisions/001-github-state-mapping.md`):
///
/// | `state`  | `state_reason` | reposix `IssueStatus` |
/// |----------|----------------|-----------------------|
/// | `closed` | `completed`    | `Done`                |
/// | `closed` | `not_planned`  | `WontFix`             |
/// | `closed` | `reopened`     | `Done` (fallback)     |
/// | `closed` | `null`         | `Done` (fallback)     |
/// | `open`   | (any/null)     | `Open` (no label)     |
///
/// This integration test pins each row through the trait seam — the
/// inline unit tests in `lib.rs` cover individual rows, but this one
/// proves the whole mapping holds end-to-end through `list_issues`.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn state_reason_maps_to_status() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/o/r/issues"))
        .and(query_param("state", "all"))
        .and(query_param("per_page", "100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            gh_issue(1, "closed", Some("completed"), None),
            gh_issue(2, "closed", Some("not_planned"), None),
            gh_issue(3, "closed", Some("reopened"), None),
            gh_issue(4, "closed", None, None),
            gh_issue(5, "open", Some("completed"), None),
            gh_issue(6, "open", None, None),
        ])))
        .mount(&server)
        .await;

    let backend = GithubReadOnlyBackend::new_with_base_url(None, server.uri()).expect("backend");
    let issues = backend.list_issues("o/r").await.expect("list");
    assert_eq!(issues.len(), 6);
    assert_eq!(issues[0].status, IssueStatus::Done, "closed+completed");
    assert_eq!(issues[1].status, IssueStatus::WontFix, "closed+not_planned");
    assert_eq!(
        issues[2].status,
        IssueStatus::Done,
        "closed+reopened (fallback to Done)"
    );
    assert_eq!(issues[3].status, IssueStatus::Done, "closed+null reason");
    assert_eq!(
        issues[4].status,
        IssueStatus::Open,
        "open + state_reason (any) → Open per ADR-001"
    );
    assert_eq!(issues[5].status, IssueStatus::Open, "open+null");
}

// --------------------------------------------------- SSRF regression
//
// Mirrors the SSRF guards in `crates/reposix-confluence/tests/contract.rs`.
// GitHub REST emits `url`, `html_url`, `comments_url`, `events_url`,
// `assignee.url`, `assignee.html_url`, `assignee.avatar_url`. None of these
// should ever trigger an outbound request from the adapter — the
// deserializer in `lib.rs` doesn't even look at them, but a future feature
// like "fetch assignee avatar for the FUSE listing" would silently reopen
// SSRF if the URL fields started getting trusted.
//
// Strategy (verbatim port from the Confluence test): mount a `decoy_server`
// with a catch-all `.expect(0)` mock that panics on `MockServer::drop` if
// hit. The `legit_server` returns valid GitHub-shaped JSON whose URL fields
// point at the decoy's origin.

/// Adversarial `html_url` / `url` / `avatar_url` fields in the issue and
/// assignee shape must NOT trigger outbound calls.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn adversarial_html_url_does_not_trigger_outbound_call() {
    let legit_server = MockServer::start().await;
    let decoy_server = MockServer::start().await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200).set_body_string("exfiltrated"))
        .expect(0)
        .mount(&decoy_server)
        .await;

    let decoy = decoy_server.uri();

    // Build an issue body whose every URL-shaped field points at the
    // decoy server (a real, reachable 127.0.0.1 origin so the SG-01
    // allowlist would not block a follow-up — the test's job is to prove
    // the adapter doesn't even attempt one).
    let evil_issue = json!({
        "number": 1,
        "title": "evil",
        "state": "open",
        "state_reason": null,
        "body": format!("see {decoy}/exfil/body for details"),
        "labels": [],
        "assignee": {
            "login": "octocat",
            "url": format!("{decoy}/users/octocat"),
            "html_url": format!("{decoy}/octocat"),
            "avatar_url": format!("{decoy}/avatars/octocat.png"),
        },
        "created_at": "2026-04-13T00:00:00Z",
        "updated_at": "2026-04-13T00:00:00Z",
        "url": format!("{decoy}/repos/o/r/issues/1"),
        "html_url": format!("{decoy}/o/r/issues/1"),
        "comments_url": format!("{decoy}/repos/o/r/issues/1/comments"),
        "events_url": format!("{decoy}/repos/o/r/issues/1/events"),
        "labels_url": format!("{decoy}/repos/o/r/issues/1/labels{{/name}}"),
        "repository_url": format!("{decoy}/repos/o/r"),
    });

    Mock::given(method("GET"))
        .and(path("/repos/o/r/issues"))
        .and(query_param("state", "all"))
        .and(query_param("per_page", "100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([evil_issue])))
        .mount(&legit_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/repos/o/r/issues/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(evil_issue))
        .mount(&legit_server)
        .await;

    let backend =
        GithubReadOnlyBackend::new_with_base_url(None, legit_server.uri()).expect("backend");

    let issues = backend.list_issues("o/r").await.expect("list");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].id, IssueId(1));
    assert_eq!(issues[0].assignee.as_deref(), Some("octocat"));

    let single = backend.get_issue("o/r", IssueId(1)).await.expect("get");
    assert_eq!(single.id, IssueId(1));

    // Inline assertion on top of `.expect(0)`'s drop-panic so failures
    // surface with a useful diagnostic.
    let hits = decoy_server.received_requests().await.unwrap_or_default();
    assert!(
        hits.is_empty(),
        "adapter made {} request(s) to adversarial url/html_url host: {:?}",
        hits.len(),
        hits.iter().map(|r| r.url.to_string()).collect::<Vec<_>>()
    );
}

// --------------------------------------------------- assignee shape

/// GitHub Issues can return `assignee: null` (unassigned), an `assignee`
/// object with `login`, or — historically — both `assignee` and the
/// `assignees` array. The adapter only reads `assignee.login`. Both
/// shapes (null + object) MUST round-trip without panicking; missing
/// `assignee` fields MUST degrade to `None`.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn malformed_assignee_object_degrades_to_none() {
    let server = MockServer::start().await;

    // Three-issue fixture: assignee=null, assignee=full object, and
    // assignee field omitted entirely (serde default).
    let no_assignee = json!({
        "number": 99,
        "title": "no assignee field at all",
        "state": "open",
        "state_reason": null,
        "body": "",
        "labels": [],
        // assignee field deliberately omitted — Option<GhUser> + serde(default)
        // must produce None.
        "created_at": "2026-04-13T00:00:00Z",
        "updated_at": "2026-04-13T00:00:00Z",
    });

    Mock::given(method("GET"))
        .and(path("/repos/o/r/issues"))
        .and(query_param("state", "all"))
        .and(query_param("per_page", "100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            gh_issue(1, "open", None, None),          // assignee: null
            gh_issue(2, "open", None, Some("alice")), // assignee: {login: "alice"}
            no_assignee,                              // assignee field absent
        ])))
        .mount(&server)
        .await;

    let backend = GithubReadOnlyBackend::new_with_base_url(None, server.uri()).expect("backend");
    let issues = backend.list_issues("o/r").await.expect("list");
    assert_eq!(issues.len(), 3);
    assert_eq!(issues[0].assignee, None, "explicit null → None");
    assert_eq!(
        issues[1].assignee.as_deref(),
        Some("alice"),
        "object → login"
    );
    assert_eq!(issues[2].assignee, None, "omitted field → None");
}

// --------------------------------------------------- User-Agent header

/// GitHub REST API returns 403 if `User-Agent` is missing
/// (<https://docs.github.com/en/rest/overview/resources-in-the-rest-api#user-agent-required>).
/// `standard_headers()` in `lib.rs` always sets one — this test pins
/// the contract by inspecting what the mock server actually received.
///
/// Uses `header_exists("User-Agent")` to assert presence at request-match
/// time, then re-checks via `received_requests().headers` for a clearer
/// failure diagnostic if the header value was empty or wrong.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn user_agent_header_is_set() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/o/r/issues/1"))
        .and(header_exists("User-Agent"))
        .respond_with(ResponseTemplate::new(200).set_body_json(gh_issue(1, "open", None, None)))
        .mount(&server)
        .await;

    let backend = GithubReadOnlyBackend::new_with_base_url(None, server.uri()).expect("backend");
    backend
        .get_issue("o/r", IssueId(1))
        .await
        .expect("get_issue should succeed (User-Agent present)");

    // Re-check the captured request for a non-empty User-Agent value.
    let requests = server.received_requests().await.unwrap_or_default();
    let req = requests
        .first()
        .expect("at least one request reached the mock");
    let ua = req
        .headers
        .get("User-Agent")
        .expect("User-Agent header missing on captured request")
        .to_str()
        .expect("User-Agent must be valid ASCII");
    assert!(
        !ua.is_empty(),
        "User-Agent must be non-empty per GitHub REST API rules"
    );
    assert!(
        ua.contains("reposix"),
        "User-Agent should identify reposix; got {ua}"
    );
}

// ------------------------------------------------------------ GithubReadOnlyBackend test

/// Hits real GitHub. `#[ignore]`-gated; opt in with
/// `cargo test -p reposix-github -- --ignored`. Requires
/// `REPOSIX_ALLOWED_ORIGINS=https://api.github.com,http://127.0.0.1:*` in
/// the env. Optionally `GITHUB_TOKEN=<pat>` to skip the 60 req/hr
/// anonymous ceiling.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore]
async fn contract_github() {
    // Sanity-check the allowlist so the failure mode reads "you forgot to
    // set REPOSIX_ALLOWED_ORIGINS" instead of "blocked origin:
    // https://api.github.com".
    let origins = std::env::var("REPOSIX_ALLOWED_ORIGINS").unwrap_or_default();
    assert!(
        origins.contains("api.github.com"),
        "contract_github requires REPOSIX_ALLOWED_ORIGINS to include \
         https://api.github.com; got {origins:?}"
    );

    let token = std::env::var("GITHUB_TOKEN").ok();
    let backend = GithubReadOnlyBackend::new(token).expect("backend");

    // octocat/Hello-World is GitHub's canonical stable fixture repo. Issue
    // #1 has existed since 2011 and is unlikely to disappear.
    assert_contract(&backend, "octocat/Hello-World", IssueId(1)).await;
}
