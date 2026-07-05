//! QL-001 Assertion-2 regression: a `git push` whose blob differs from the
//! backend ONLY in server-controlled frontmatter (version / updated_at /
//! created_at) must write NOTHING to the backend.
//!
//! This is the shape a routine `git pull --no-rebase` merge produces on the
//! push path that reddened `agent-ux/real-git-push-e2e` Assertion 2 (CI run
//! 28725302159: "no-op push wrote backend mutations: got 2"): after a first
//! push bumps the backend record's `version` (1 → 2) and `updated_at`, a later
//! no-op push carries a merged working blob whose server-controlled fields
//! diverge from the backend's current values (cache-rebuild divergence), while
//! the `list_changed_since` cursor is warm enough that the L1 precheck skips
//! the record (no conflict) and proceeds straight to `diff::plan`. The pre-fix
//! planner compared the FULL frontmatter render and emitted a spurious PATCH —
//! pure backend noise + a needless version bump, on every routine pull/push
//! cycle. The write path already sanitizes those fields
//! (`execute_action::Update` uses the cache-derived `prior_version` for
//! `If-Match` and never sends the blob's own values), so the PATCH was never a
//! real edit.
//!
//! This drives the REAL `git-remote-reposix` binary through the full
//! precheck → plan → execute path against a wiremock backend at version 2 with
//! a WARM cache cursor (`?since=` returns empty → the record is skipped by the
//! precheck), feeding an export stream whose issue-1 blob is at version 1 with
//! byte-identical writable content. It asserts ZERO POST/PATCH/DELETE and a
//! clean `ok` ack. RED against the pre-fix planner (one spurious PATCH).

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::doc_markdown)] // test-internal doc comments cite frontmatter fields verbatim

use std::io::Write;
use std::sync::Arc;

use assert_cmd::Command as AssertCommand;
use chrono::TimeZone;
use reposix_cache::Cache;
use reposix_core::{frontmatter, BackendConnector, Record, RecordId, RecordStatus};
use serde_json::json;
use wiremock::matchers::{any, method, path_regex};
use wiremock::{Match, Mock, MockServer, Request, ResponseTemplate};

mod common;
use common::{sim_backend, CacheDirGuard};

/// Custom matcher: requests that carry a `since` query param (the
/// `list_changed_since` shape). Mirrors `tests/bus_write_happy.rs`.
struct HasSinceQueryParam;
impl Match for HasSinceQueryParam {
    fn matches(&self, req: &Request) -> bool {
        req.url.query_pairs().any(|(k, _)| k == "since")
    }
}

/// Issue 1 with a FIXED writable payload (title/status/labels/body) and the
/// given SERVER-CONTROLLED `version`/`updated_at`. Two records built with
/// different `version`/`updated_at` differ ONLY in server-controlled fields.
fn issue_1(version: u64, updated_at: chrono::DateTime<chrono::Utc>) -> Record {
    let created = chrono::Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
    Record {
        id: RecordId(1),
        title: "database connection drops under load".to_owned(),
        status: RecordStatus::Open,
        assignee: None,
        labels: vec!["bug".to_owned(), "p1".to_owned()],
        created_at: created,
        updated_at,
        version,
        body: "Pool exhaustion under sustained load.\n\ne2e edit 1751700000".to_owned(),
        parent_id: None,
        extensions: std::collections::BTreeMap::new(),
    }
}

fn issue_to_json(issue: &Record) -> serde_json::Value {
    json!({
        "id": issue.id.0,
        "title": issue.title,
        "status": issue.status.as_str(),
        "assignee": issue.assignee,
        "labels": issue.labels,
        "created_at": issue.created_at.to_rfc3339(),
        "updated_at": issue.updated_at.to_rfc3339(),
        "version": issue.version,
        "body": issue.body,
    })
}

/// A single-commit export stream whose `issues/1.md` blob renders `issue`.
fn one_commit_export_stream(issue: &Record) -> Vec<u8> {
    let blob = frontmatter::render(issue).expect("render issue-1 blob");
    let mut out: Vec<u8> = Vec::new();
    writeln!(&mut out, "feature done").unwrap();
    writeln!(&mut out, "blob").unwrap();
    writeln!(&mut out, "mark :100").unwrap();
    writeln!(&mut out, "data {}", blob.len()).unwrap();
    out.extend_from_slice(blob.as_bytes());
    out.push(b'\n');
    writeln!(&mut out, "commit refs/heads/main").unwrap();
    writeln!(&mut out, "mark :1").unwrap();
    writeln!(&mut out, "committer test <t@t> 0 +0000").unwrap();
    let msg = "merge; no writable edits\n";
    writeln!(&mut out, "data {}", msg.len()).unwrap();
    out.extend_from_slice(msg.as_bytes());
    writeln!(&mut out, "M 100644 :100 issues/1.md").unwrap();
    writeln!(&mut out, "done").unwrap();
    out
}

// test-name-honesty: ok — drives the real compiled `git-remote-reposix`
// binary (via assert_cmd) through the full export/push protocol against a
// wiremock HTTP backend (not a real SaaS, hence no `#[ignore]`); the module
// doc comment states this scope explicitly.
#[tokio::test(flavor = "multi_thread")]
async fn noop_push_with_only_server_field_drift_writes_nothing() {
    let server = MockServer::start().await;
    let project = "demo";

    // Backend's CURRENT state: version 2, freshly bumped updated_at.
    let backend_now = issue_1(
        2,
        chrono::Utc.with_ymd_and_hms(2026, 7, 5, 1, 39, 47).unwrap(),
    );

    // Setup-phase (default priority): list_records (no ?since) + per-id GET
    // both return the version-2 record. `Cache::build_from` and the L1
    // precheck's list_records fallback both read this.
    Mock::given(method("GET"))
        .and(path_regex(format!(r"^/projects/{project}/issues$")))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!([issue_to_json(&backend_now)])),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path_regex(format!(r"^/projects/{project}/issues/1$")))
        .respond_with(ResponseTemplate::new(200).set_body_json(issue_to_json(&backend_now)))
        .mount(&server)
        .await;

    // Warm the cache cursor (last_fetched_at) + oid_map from the version-2
    // backend. build_from leaves blobs lazy, so the precheck's Step 5 prior
    // falls through to list_records (also version 2) — the exact CI shape.
    let cache_root = tempfile::tempdir().expect("cache_root");
    let _env = CacheDirGuard::new(cache_root.path());
    let backend: Arc<dyn BackendConnector> = sim_backend(&server);
    let cache = Cache::open(backend, "sim", project).expect("Cache::open");
    cache.sync().await.expect("seed sync (warm cache cursor)");
    drop(cache);

    // Assertion-phase (priority 1): list_changed_since (`?since=`) returns
    // EMPTY, so the precheck skips issue 1 (no conflict) and proceeds to plan.
    Mock::given(method("GET"))
        .and(path_regex(format!(r"^/projects/{project}/issues$")))
        .and(HasSinceQueryParam)
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .with_priority(1)
        .mount(&server)
        .await;

    // The load-bearing assertions: NOT ONE mutating request may be issued.
    Mock::given(method("PATCH"))
        .and(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .with_priority(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(any())
        .respond_with(ResponseTemplate::new(201))
        .expect(0)
        .with_priority(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(any())
        .respond_with(ResponseTemplate::new(204))
        .expect(0)
        .with_priority(1)
        .mount(&server)
        .await;

    // The pushed blob: version 1, old updated_at — SAME writable content as
    // the version-2 backend record.
    let pushed_stale = issue_1(
        1,
        chrono::Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap(),
    );
    let url = format!("reposix::{}/projects/{project}", server.uri());
    let stdin_data = {
        let mut buf = Vec::new();
        writeln!(&mut buf, "export").unwrap();
        buf.extend_from_slice(&one_commit_export_stream(&pushed_stale));
        buf
    };

    let cache_path = cache_root.path().to_path_buf();
    let allowed = server.uri();
    let out = tokio::task::spawn_blocking(move || {
        AssertCommand::cargo_bin("git-remote-reposix")
            .expect("binary built")
            .args(["origin", &url])
            .env("REPOSIX_CACHE_DIR", &cache_path)
            .env("REPOSIX_ALLOWED_ORIGINS", allowed)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_TERMINAL_PROMPT", "0")
            .write_stdin(stdin_data)
            .timeout(std::time::Duration::from_secs(20))
            .output()
            .expect("run helper")
    })
    .await
    .unwrap();

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "server-field-drift-only push must ack cleanly (no-op); \
         stdout={stdout}\nstderr={stderr}"
    );
    assert!(
        stdout.contains("ok refs/heads/main"),
        "must ack `ok refs/heads/main`; stdout={stdout}\nstderr={stderr}"
    );
    // The wiremock `.expect(0)` mounts verify on drop that ZERO
    // POST/PATCH/DELETE calls were made — the QL-001 Assertion-2 regression.
}
