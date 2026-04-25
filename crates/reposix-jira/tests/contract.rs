//! Contract test — the same 5 invariants hold for SimBackend,
//! a wiremock-backed [`JiraBackend`], and (when `#[ignore]`-unlocked +
//! env configured) a live Atlassian JIRA tenant.
//!
//! ## The 5 invariants
//!
//! 1. `list_issues(project)` returns `Ok(vec)` for a known-good project.
//! 2. The list is non-empty (≥1 issue).
//! 3. `get_issue(project, known_issue_id)` returns `Ok(issue)` with matching id.
//! 4. `get_issue(project, RecordId(u64::MAX))` returns `Err` (404 path).
//! 5. Every listed issue's status is a valid `IssueStatus` variant.
//!
//! ## Test arms
//!
//! - `contract_sim` — boots `reposix-sim` on an ephemeral port. Always runs.
//! - `contract_jira_wiremock` — mounts JIRA v3 endpoints on MockServer. Always runs.
//! - `contract_jira_live` — hits a real JIRA tenant. `#[ignore]`-gated +
//!   `skip_if_no_env!`-guarded.

use std::path::PathBuf;

use reposix_core::backend::sim::SimBackend;
use reposix_core::backend::{BackendConnector, DeleteReason};
use reposix_core::{RecordId, IssueStatus};
use reposix_jira::{JiraBackend, JiraCreds};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Skip the enclosing test if any listed env var is unset or empty.
///
/// Prints `SKIP: env vars unset: ...` to stderr and returns early.
/// Only variable *names* are printed — never values.
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

    // (5) Every listed issue has a valid IssueStatus variant.
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

// ─── Sim fixture ─────────────────────────────────────────────────────────────

fn sim_seed_fixture() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("..");
    p.push("reposix-sim");
    p.push("fixtures");
    p.push("seed.json");
    p
}

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

// ─── Minimal JIRA issue fixture ───────────────────────────────────────────────

fn jira_issue_json(id: u64, key: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id.to_string(),
        "key": key,
        "fields": {
            "summary": "Fix login bug",
            "description": serde_json::Value::Null,
            "status": {
                "name": "In Progress",
                "statusCategory": {"key": "indeterminate"}
            },
            "resolution": serde_json::Value::Null,
            "assignee": {"displayName": "Alice"},
            "labels": [],
            "created": "2025-01-01T00:00:00.000+0000",
            "updated": "2025-11-15T10:30:00.000+0000",
            "parent": serde_json::Value::Null,
            "issuetype": {"name": "Story", "hierarchyLevel": 0},
            "priority": {"name": "Medium"}
        }
    })
}

// ─── Test: contract_sim ───────────────────────────────────────────────────────

/// Always runs — no external deps needed. Proves the shared `assert_contract`
/// helper is exercisable within the jira crate's own CI footprint.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_sim() {
    let (origin, _db, handle) = spawn_sim().await;
    let backend = SimBackend::new(origin).expect("SimBackend::new");
    assert_contract(&backend, "demo", RecordId(1)).await;
    handle.abort();
}

// ─── Test: contract_jira_wiremock ─────────────────────────────────────────────

/// Always runs. Exercises the full `list_issues → get_issue(10001) →
/// get_issue(u64::MAX)` sequence through the [`BackendConnector`] trait seam.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_jira_wiremock() {
    let server = MockServer::start().await;

    // POST /rest/api/3/search/jql → 1 issue, isLast: true
    Mock::given(method("POST"))
        .and(path("/rest/api/3/search/jql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "issues": [jira_issue_json(10001, "PROJ-1")],
            "isLast": true
        })))
        .mount(&server)
        .await;

    // GET /rest/api/3/issue/10001 → 200 known issue
    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/10001"))
        .respond_with(ResponseTemplate::new(200).set_body_json(jira_issue_json(10001, "PROJ-1")))
        .mount(&server)
        .await;

    // GET /rest/api/3/issue/18446744073709551615 (u64::MAX) → 404
    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/18446744073709551615"))
        .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
            "errorMessages": ["Issue Does Not Exist"],
            "errors": {}
        })))
        .mount(&server)
        .await;

    let creds = JiraCreds {
        email: "test@example.com".into(),
        api_token: "token".into(),
    };
    let backend = JiraBackend::new_with_base_url(creds, server.uri())
        .expect("JiraBackend::new_with_base_url");

    assert_contract(&backend, "PROJ", RecordId(10001)).await;
}

// ─── Write contract helpers ───────────────────────────────────────────────────

fn make_untainted_for_contract(
    title: &str,
    body: &str,
) -> reposix_core::Untainted<reposix_core::Issue> {
    use reposix_core::{sanitize, ServerMetadata};
    let now = chrono::Utc::now();
    let raw = reposix_core::Issue {
        id: reposix_core::RecordId(0),
        title: title.to_owned(),
        body: body.to_owned(),
        status: reposix_core::IssueStatus::Open,
        created_at: now,
        updated_at: now,
        version: 0,
        assignee: None,
        labels: vec![],
        parent_id: None,
        extensions: Default::default(),
    };
    sanitize(
        reposix_core::Tainted::new(raw),
        ServerMetadata {
            id: reposix_core::RecordId(0),
            created_at: now,
            updated_at: now,
            version: 0,
        },
    )
}

/// Write contract: create → update → delete → assert-gone.
async fn assert_write_contract<B: BackendConnector>(backend: &B, project: &str) {
    // Create
    let issue = make_untainted_for_contract("contract-write-test", "initial body");
    let created = backend
        .create_issue(project, issue)
        .await
        .unwrap_or_else(|e| panic!("[{}] create_issue failed: {e:?}", backend.name()));
    assert_eq!(
        created.title,
        "contract-write-test",
        "[{}] created title mismatch",
        backend.name()
    );

    // Update
    let patch = make_untainted_for_contract("contract-write-updated", "updated body");
    let updated = backend
        .update_issue(project, created.id, patch, None)
        .await
        .unwrap_or_else(|e| panic!("[{}] update_issue failed: {e:?}", backend.name()));
    assert_eq!(
        updated.title,
        "contract-write-updated",
        "[{}] updated title mismatch",
        backend.name()
    );

    // Delete
    backend
        .delete_or_close(project, created.id, DeleteReason::Completed)
        .await
        .unwrap_or_else(|e| panic!("[{}] delete_or_close failed: {e:?}", backend.name()));

    // Verify deleted
    let gone = backend.get_issue(project, created.id).await;
    assert!(
        gone.is_err(),
        "[{}] get_issue after delete should be Err, got {:?}",
        backend.name(),
        gone
    );
}

/// Build a wiremock server that handles the full write contract sequence.
async fn build_jira_wiremock_write_server() -> (String, MockServer) {
    let server = MockServer::start().await;

    // GET /rest/api/3/issuetype → ["Task"]
    Mock::given(method("GET"))
        .and(path("/rest/api/3/issuetype"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id": "10001", "name": "Task"}
        ])))
        .mount(&server)
        .await;

    // POST /rest/api/3/issue → 201 {"id":"1001","key":"P-1"}
    Mock::given(method("POST"))
        .and(path("/rest/api/3/issue"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": "1001", "key": "P-1"
        })))
        .mount(&server)
        .await;

    // PUT /rest/api/3/issue/1001 → 204
    Mock::given(method("PUT"))
        .and(path("/rest/api/3/issue/1001"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    // GET /rest/api/3/issue/1001/transitions → done transition
    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/1001/transitions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "transitions": [
                {"id": "31", "name": "Done", "to": {"statusCategory": {"key": "done"}}}
            ]
        })))
        .mount(&server)
        .await;

    // POST /rest/api/3/issue/1001/transitions → 204
    Mock::given(method("POST"))
        .and(path("/rest/api/3/issue/1001/transitions"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    // GET /rest/api/3/issue/1001 — FIFO: first registered fires first.
    // Call order: (1) create hydrate → "contract-write-test",
    //             (2) update hydrate → "contract-write-updated",
    //             (3) assert-gone    → 404.
    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/1001"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(jira_issue_json_with_summary(
                1001,
                "P-1",
                "contract-write-test",
            )),
        )
        .up_to_n_times(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/1001"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(jira_issue_json_with_summary(
                1001,
                "P-1",
                "contract-write-updated",
            )),
        )
        .up_to_n_times(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/1001"))
        .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
            "errorMessages": ["Issue Does Not Exist"], "errors": {}
        })))
        .mount(&server)
        .await;

    (server.uri(), server)
}

fn jira_issue_json_with_summary(id: u64, key: &str, summary: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id.to_string(),
        "key": key,
        "fields": {
            "summary": summary,
            "description": serde_json::Value::Null,
            "status": {"name": "Done", "statusCategory": {"key": "done"}},
            "resolution": serde_json::Value::Null,
            "assignee": serde_json::Value::Null,
            "labels": [],
            "created": "2026-04-16T10:00:00.000+0000",
            "updated": "2026-04-16T10:01:00.000+0000",
            "parent": serde_json::Value::Null,
            "issuetype": {"name": "Task", "hierarchyLevel": 0},
            "priority": {"name": "Medium"}
        }
    })
}

// ─── Test: contract_jira_wiremock_write ───────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_jira_wiremock_write() {
    let (base_url, _mock_server) = build_jira_wiremock_write_server().await;
    let backend = JiraBackend::new_with_base_url(
        JiraCreds {
            email: "e@t.com".into(),
            api_token: "tok".into(),
        },
        base_url,
    )
    .expect("backend");
    assert_write_contract(&backend, "P").await;
}

// ─── Test: contract_jira_live ─────────────────────────────────────────────────

/// Hits a real JIRA tenant. `#[ignore]`-gated + `skip_if_no_env!`-guarded.
/// Opt-in via: `cargo test -p reposix-jira -- --ignored`
///
/// Required env vars: `JIRA_EMAIL`, `JIRA_API_TOKEN`, `REPOSIX_JIRA_INSTANCE`
/// (subdomain only, e.g. `mycompany` for `mycompany.atlassian.net`).
/// `REPOSIX_ALLOWED_ORIGINS` must include `https://{instance}.atlassian.net`.
/// Optional: `JIRA_TEST_PROJECT` (project key, defaults to `TEST`).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires JIRA_EMAIL, JIRA_API_TOKEN, REPOSIX_JIRA_INSTANCE env vars"]
async fn contract_jira_live() {
    skip_if_no_env!("JIRA_EMAIL", "JIRA_API_TOKEN", "REPOSIX_JIRA_INSTANCE");

    let email = std::env::var("JIRA_EMAIL").unwrap();
    let token = std::env::var("JIRA_API_TOKEN").unwrap();
    let instance = std::env::var("REPOSIX_JIRA_INSTANCE").unwrap();
    let project = std::env::var("JIRA_TEST_PROJECT").unwrap_or_else(|_| "TEST".to_string());

    let creds = JiraCreds {
        email,
        api_token: token,
    };
    let backend = JiraBackend::new(creds, &instance).expect("build JiraBackend");

    // list first to get a real issue id
    let issues = backend
        .list_issues(&project)
        .await
        .expect("live list_issues");
    assert!(
        !issues.is_empty(),
        "live JIRA project {project} has no issues — set JIRA_TEST_PROJECT to a project with data"
    );
    let known_id = issues[0].id;
    assert_contract(&backend, &project, known_id).await;
}

// ─── Test: contract_jira_live_write ──────────────────────────────────────────

/// Hits a real JIRA tenant with write operations. `#[ignore]`-gated.
/// Opt-in via: `cargo test -p reposix-jira -- --ignored`
///
/// Required env vars: same as `contract_jira_live` plus a project with
/// write permissions. Creates, updates, and deletes a real issue.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires JIRA_EMAIL, JIRA_API_TOKEN, REPOSIX_JIRA_INSTANCE env vars"]
async fn contract_jira_live_write() {
    skip_if_no_env!("JIRA_EMAIL", "JIRA_API_TOKEN", "REPOSIX_JIRA_INSTANCE");

    let email = std::env::var("JIRA_EMAIL").unwrap();
    let token = std::env::var("JIRA_API_TOKEN").unwrap();
    let instance = std::env::var("REPOSIX_JIRA_INSTANCE").unwrap();
    let project = std::env::var("REPOSIX_JIRA_PROJECT").unwrap_or_else(|_| "TEST".to_string());

    let creds = JiraCreds {
        email,
        api_token: token,
    };
    let backend = JiraBackend::new(creds, &instance).expect("build JiraBackend");
    assert_write_contract(&backend, &project).await;
}
