//! Perf regression test for L1 conflict-detection migration
//! (DVCS-PERF-L1-01..03).
//!
//! Asserts the helper's precheck path makes >=1 `list_changed_since`
//! REST calls AND ZERO `list_records` REST calls when the cache cursor
//! is populated AND blobs are materialized (the steady-state hot path).
//! Includes a positive-control sibling that flips `expect(0)` to
//! `expect(1)` and confirms wiremock fails RED when the matcher is
//! unmet — closing RESEARCH.md MEDIUM risk.

#![forbid(unsafe_code)]
#![allow(clippy::missing_panics_doc)]

use std::fmt::Write as _;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

use assert_cmd::Command;
use chrono::TimeZone;
use reposix_cache::Cache;
use reposix_core::backend::sim::SimBackend;
use reposix_core::{BackendConnector, Record, RecordId, RecordStatus};
use serde_json::Value;
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Match, Mock, MockServer, Request, ResponseTemplate};

/// Custom matcher: matches GET requests with NO `since` query param
/// (i.e., the unconditional `list_records` call, not the L1
/// `list_changed_since` delta). wiremock 0.6 supports custom `Match`
/// impls.
struct NoSinceQueryParam;
impl Match for NoSinceQueryParam {
    fn matches(&self, req: &Request) -> bool {
        req.url.query_pairs().all(|(k, _)| k != "since")
    }
}

/// Custom matcher (M4 fix): symmetric to `NoSinceQueryParam` — matches
/// requests that DO have a `since` query param, regardless of value.
/// wiremock 0.6's `query_param(K, V)` is byte-exact (returns
/// `HeaderExactMatcher`-shape); there is no `query_param_exists` or
/// wildcard-value form. A custom `Match` impl is the canonical idiom.
struct HasSinceQueryParam;
impl Match for HasSinceQueryParam {
    fn matches(&self, req: &Request) -> bool {
        req.url.query_pairs().any(|(k, _)| k == "since")
    }
}

/// Build a sample Record with the given id + version. Mirrors the
/// shape `mirror_refs.rs::sample_issue` produces.
fn sample_record(id: u64, version: u64) -> Record {
    let t = chrono::Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
    Record {
        id: RecordId(id),
        title: format!("issue {id}"),
        status: RecordStatus::Open,
        assignee: None,
        labels: vec![],
        created_at: t,
        updated_at: t,
        version,
        body: format!("body of {id}\n"),
        parent_id: None,
        extensions: std::collections::BTreeMap::new(),
    }
}

fn sample_issue_json(id: u64, version: u64) -> Value {
    serde_json::to_value(sample_record(id, version)).unwrap()
}

/// Render a Record's frontmatter+body form (matches the helper's
/// inbound expectation for a clean push). Verbatim shape from
/// `mirror_refs.rs::render_with_overrides`.
fn render_with_overrides(
    id: u64,
    title: &str,
    body: &str,
    version_override: u64,
    id_override: u64,
) -> String {
    let t = chrono::Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
    let ts = t.to_rfc3339();
    let mut s = String::new();
    s.push_str("---\n");
    writeln!(&mut s, "id: {id_override}").unwrap();
    writeln!(&mut s, "title: {title}").unwrap();
    s.push_str("status: open\n");
    writeln!(&mut s, "created_at: {ts}").unwrap();
    writeln!(&mut s, "updated_at: {ts}").unwrap();
    writeln!(&mut s, "version: {version_override}").unwrap();
    s.push_str("---\n");
    s.push_str(body);
    if !s.ends_with('\n') {
        s.push('\n');
    }
    let _ = id;
    s
}

/// Build a fast-export stream containing one updated issue. Verbatim
/// shape from `mirror_refs.rs::one_file_export`. Currently unused —
/// the perf tests use [`no_op_tree_export`] instead — kept here so
/// future perf-test additions (e.g. one-record edit scenarios that
/// exercise the conflict path) don't have to re-derive the shape.
#[allow(dead_code)]
fn one_file_export(path: &str, blob: &str, msg: &str) -> Vec<u8> {
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
    let bytes = msg.as_bytes();
    writeln!(&mut out, "data {}", bytes.len()).unwrap();
    out.extend_from_slice(bytes);
    out.push(b'\n');
    writeln!(&mut out, "M 100644 :100 {path}").unwrap();
    writeln!(&mut out, "done").unwrap();
    out
}

/// Build a fast-export stream containing N issues — each with the
/// frontmatter the cache derives from `sample_record(id, 1)`. Used by
/// the perf test to push a tree IDENTICAL to the cache prior so plan()
/// emits zero actions (no creates / no updates / no deletes). The
/// precheck still runs (list_changed_since fires + parse loop iterates
/// each path), so the wiremock counters reflect the L1 hot path
/// regardless.
fn no_op_tree_export(n: u64, msg: &str) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    writeln!(&mut out, "feature done").unwrap();
    let mut blob_marks: Vec<(u64, u64)> = Vec::with_capacity(n as usize);
    for i in 1..=n {
        let mark: u64 = 100 + i;
        let title = format!("issue {i}");
        let body = format!("body of {i}\n");
        let blob = render_with_overrides(i, &title, &body, 1, i);
        writeln!(&mut out, "blob").unwrap();
        writeln!(&mut out, "mark :{mark}").unwrap();
        writeln!(&mut out, "data {}", blob.len()).unwrap();
        out.extend_from_slice(blob.as_bytes());
        out.push(b'\n');
        blob_marks.push((i, mark));
    }
    writeln!(&mut out, "commit refs/heads/main").unwrap();
    writeln!(&mut out, "mark :1").unwrap();
    writeln!(&mut out, "committer test <t@t> 0 +0000").unwrap();
    let bytes = msg.as_bytes();
    writeln!(&mut out, "data {}", bytes.len()).unwrap();
    out.extend_from_slice(bytes);
    out.push(b'\n');
    for (id, mark) in &blob_marks {
        writeln!(&mut out, "M 100644 :{mark} {id:04}.md").unwrap();
    }
    writeln!(&mut out, "done").unwrap();
    out
}

/// Drive the helper through one export turn. Returns the captured
/// stdout/stderr/status from the subprocess. Mirrors
/// `mirror_refs.rs::drive_helper_export` verbatim.
fn drive_helper_export(url: &str, cache_dir: &Path, stream: &[u8]) -> assert_cmd::assert::Assert {
    let stdin_data = {
        let mut buf = Vec::new();
        writeln!(&mut buf, "export").unwrap();
        buf.extend_from_slice(stream);
        buf
    };
    Command::cargo_bin("git-remote-reposix")
        .expect("binary built")
        .args(["origin", url])
        .env("REPOSIX_CACHE_DIR", cache_dir)
        .write_stdin(stdin_data)
        .timeout(std::time::Duration::from_secs(15))
        .assert()
}

/// Set up the steady-state cache: build_from + materialize all blobs.
/// This makes the L1 hot-path real — list_changed_since fires and the
/// per-record GETs (Step 3) are bounded by changed_set ∩ push_set,
/// while plan()'s prior is materialized entirely from cache.
async fn warm_cache(cache: &Cache, n: u64) {
    cache.sync().await.expect("seed sync");
    for i in 1..=n {
        let id = RecordId(i);
        let oid = cache
            .find_oid_for_record(id)
            .expect("find_oid_for_record")
            .expect("oid present after sync");
        // Materialize blob into the cache's bare repo via the async
        // path. Subsequent read_blob_cached calls will return Some(_)
        // without backend egress.
        cache.read_blob(oid).await.expect("materialize blob");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn l1_precheck_uses_list_changed_since_not_list_records() {
    // Seed N=200 records; large enough to make pagination observable
    // had we taken the legacy list_records walk.
    let server = MockServer::start().await;
    let project = "demo";
    let n: u64 = 200;
    let issues: Vec<Value> = (1..=n).map(|i| sample_issue_json(i, 1)).collect();

    // SETUP MOCKS — the warm-cache phase below makes ONE list_records
    // call (Cache::sync seed path) + N per-id GETs (read_blob
    // materialization). The hot-path counters for the assertion below
    // are mounted AFTER warm_cache so they don't count setup traffic.
    // Setup-phase mock: list_records returns the seed body.
    let setup_list_mock = Mock::given(method("GET"))
        .and(path(format!("/projects/{project}/issues")))
        .and(NoSinceQueryParam)
        .respond_with(ResponseTemplate::new(200).set_body_json(&issues));
    setup_list_mock.mount(&server).await;
    // Per-issue GET handler — needed during cache warmup
    // (read_blob materializes via backend.get_record). We mount one
    // mock per id so each per-issue call returns the matching issue
    // body. Without per-id matching, all GETs would return id=1's
    // body, causing OidDrift errors when issue 2's blob is requested.
    for issue in &issues {
        let id = issue.get("id").and_then(Value::as_u64).expect("issue id");
        Mock::given(method("GET"))
            .and(path(format!("/projects/{project}/issues/{id}")))
            .respond_with(ResponseTemplate::new(200).set_body_json(issue))
            .mount(&server)
            .await;
    }

    // Per-test cache dir (isolated from system cache).
    let cache_dir = tempfile::tempdir().expect("tempdir");
    let cache_root = cache_dir.path().to_path_buf();

    // Pre-warm cache: build_from + materialize all blobs so plan()'s
    // prior is fully populated WITHOUT any list_records fallback. The
    // L1 hot-path contract is "blobs are materialized in steady state."
    let backend: Arc<dyn BackendConnector> =
        Arc::new(SimBackend::new(server.uri()).expect("SimBackend::new"));
    std::env::set_var("REPOSIX_CACHE_DIR", &cache_root);
    let cache = Cache::open(backend, "sim", project).expect("Cache::open");
    warm_cache(&cache, n).await;
    drop(cache);
    std::env::remove_var("REPOSIX_CACHE_DIR");

    // ASSERTION-PHASE MOCKS: wiremock 0.6 priority is "first mounted
    // wins on ties; lower priority number wins overall" (see
    // wiremock-0.6.5/src/mock.rs:283). The setup mock above (default
    // priority 5) was mounted first and would otherwise catch every
    // subsequent list_records call regardless of order. To make the
    // assertion mocks ACTUALLY observe helper-subprocess traffic, we
    // give them priority=1 (higher than the setup mock).
    //
    // NoSinceQueryParam, expect(0): the helper subprocess MUST NOT
    // call list_records on the cursor-present hot path. wiremock's
    // Drop will panic if this mock matches >0 requests.
    Mock::given(method("GET"))
        .and(path(format!("/projects/{project}/issues")))
        .and(NoSinceQueryParam)
        .respond_with(ResponseTemplate::new(200).set_body_json(&issues))
        .expect(0)
        .with_priority(1)
        .mount(&server)
        .await;

    // HasSinceQueryParam, expect(1..): the precheck MUST call
    // list_changed_since at least once. Empty-result is the cheap
    // success path. priority=1 also so this mock catches the helper's
    // calls (no setup mock matches HasSinceQueryParam, but use
    // matching priority for symmetry).
    Mock::given(method("GET"))
        .and(path(format!("/projects/{project}/issues")))
        .and(HasSinceQueryParam)
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .expect(1..)
        .with_priority(1)
        .mount(&server)
        .await;

    // PATCH stub for any executed update (used as a backstop; the
    // no-op tree below should produce zero PATCHes). Returns the
    // bumped version so the helper records a successful push if it
    // does fire.
    Mock::given(method("PATCH"))
        .and(path_regex(format!(r"^/projects/{project}/issues/\d+$")))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_issue_json(1, 2)))
        .mount(&server)
        .await;

    // Drive the export verb via the helper subprocess. The blob is a
    // CLEAN push (version=1, same as backend) so the precheck enters
    // the no-conflict branch and falls through to plan(). plan() sees
    // 0001.md in parsed.tree matching prior; render-compare equates;
    // no Update fired. The helper acks `ok refs/heads/main`.
    let stream = no_op_tree_export(n, "no-op push\n");
    let url = format!("reposix::{}/projects/{project}", server.uri());
    let stream_clone = stream.clone();
    let url_clone = url.clone();
    let cache_path = cache_root.clone();
    let assert = tokio::task::spawn_blocking(move || {
        drive_helper_export(&url_clone, &cache_path, &stream_clone)
    })
    .await
    .unwrap();
    let out = assert.get_output();
    assert!(
        out.status.success(),
        "helper subprocess failed: stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // wiremock asserts via Drop: panics if expectations unmet
    // (>=1 list_changed_since seen + zero list_records seen).
}

/// Positive control: same setup as the OK test BUT mounts an
/// assertion mock with `expect(1)` AFTER warm_cache and asserts
/// wiremock panics on Drop. Closes RESEARCH.md MEDIUM risk
/// "wiremock semantics need confirmation during Task 4". If this
/// test SKIPs or PASSes when it should FAIL, the assertion contract
/// is broken.
#[tokio::test(flavor = "multi_thread")]
#[should_panic(expected = "Verifications failed")]
async fn positive_control_list_records_call_fails_red() {
    let server = MockServer::start().await;
    let project = "demo";
    let n: u64 = 200;
    let issues: Vec<Value> = (1..=n).map(|i| sample_issue_json(i, 1)).collect();

    // SETUP-PHASE mocks (same as the OK test). Default priority 5;
    // catches warm_cache's list_records traffic so the assertion mock
    // mounted later doesn't observe setup noise.
    Mock::given(method("GET"))
        .and(path(format!("/projects/{project}/issues")))
        .and(NoSinceQueryParam)
        .respond_with(ResponseTemplate::new(200).set_body_json(&issues))
        .mount(&server)
        .await;
    for issue in &issues {
        let id = issue.get("id").and_then(Value::as_u64).expect("issue id");
        Mock::given(method("GET"))
            .and(path(format!("/projects/{project}/issues/{id}")))
            .respond_with(ResponseTemplate::new(200).set_body_json(issue))
            .mount(&server)
            .await;
    }

    let cache_dir = tempfile::tempdir().expect("tempdir");
    let cache_root = cache_dir.path().to_path_buf();

    let backend: Arc<dyn BackendConnector> =
        Arc::new(SimBackend::new(server.uri()).expect("SimBackend::new"));
    std::env::set_var("REPOSIX_CACHE_DIR", &cache_root);
    let cache = Cache::open(backend, "sim", project).expect("Cache::open");
    warm_cache(&cache, n).await;
    drop(cache);
    std::env::remove_var("REPOSIX_CACHE_DIR");

    // ASSERTION-PHASE mocks (priority=1 so they catch helper traffic
    // even though setup mocks are mounted earlier).
    //
    // FLIPPED: expect 1 list_records call. Since L1 precheck does
    // NOT call list_records on the cursor-present hot path AND the
    // no-op push skips refresh_for_mirror_head (files_touched=0),
    // wiremock will see ZERO list_records calls from the helper
    // subprocess — and the expectation's Drop panics.
    Mock::given(method("GET"))
        .and(path(format!("/projects/{project}/issues")))
        .and(NoSinceQueryParam)
        .respond_with(ResponseTemplate::new(200).set_body_json(&issues))
        .expect(1) // <-- DELIBERATELY MISMATCHED
        .with_priority(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path(format!("/projects/{project}/issues")))
        .and(HasSinceQueryParam)
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .with_priority(1)
        .mount(&server)
        .await;

    Mock::given(method("PATCH"))
        .and(path_regex(format!(r"^/projects/{project}/issues/\d+$")))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_issue_json(1, 2)))
        .mount(&server)
        .await;

    let stream = no_op_tree_export(n, "no-op push\n");
    let url = format!("reposix::{}/projects/{project}", server.uri());
    let stream_clone = stream.clone();
    let url_clone = url.clone();
    let cache_path = cache_root.clone();
    let _ = tokio::task::spawn_blocking(move || {
        drive_helper_export(&url_clone, &cache_path, &stream_clone)
    })
    .await
    .unwrap();

    // The MockServer's Drop panics with "Verifications failed: ..."
    // because the expect(1) was unmet. The #[should_panic(expected =
    // ...)] attribute confirms the panic message contains the
    // wiremock assertion-fail string.
}
