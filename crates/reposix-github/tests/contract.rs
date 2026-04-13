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
