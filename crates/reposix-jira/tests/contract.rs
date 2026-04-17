//! Contract test — the same 5 invariants hold for SimBackend,
//! a wiremock-backed [`JiraBackend`], and (when `#[ignore]`-unlocked +
//! env configured) a live Atlassian JIRA tenant.
//!
//! ## The 5 invariants
//!
//! 1. `list_issues(project)` returns `Ok(vec)` for a known-good project.
//! 2. The list is non-empty (≥1 issue).
//! 3. `get_issue(project, known_issue_id)` returns `Ok(issue)` with matching id.
//! 4. `get_issue(project, IssueId(u64::MAX))` returns `Err` (404 path).
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
use reposix_core::backend::BackendConnector;
use reposix_core::{IssueId, IssueStatus};
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
async fn assert_contract<B: BackendConnector>(backend: &B, project: &str, known_issue_id: IssueId) {
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
    assert_contract(&backend, "demo", IssueId(1)).await;
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

    assert_contract(&backend, "PROJ", IssueId(10001)).await;
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
#[ignore]
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
