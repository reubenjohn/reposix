//! `SotPartialFail` recovery integration test (ADR-010 / RBF-LR-03).
//!
//! Proves the ADR's ratified partial-fail recovery semantics end-to-end
//! against the default (sim) backend, driving the real `git-remote-reposix`
//! export path twice through a stateful wiremock that models the sim's HTTP
//! contract:
//!
//! **Push 1 (partial fail):** the agent edits issues 1 AND 2. `execute_action`
//! PATCHes issue 1 (200 — lands on the SoT) then issue 2 (500 — fails). The
//! helper emits `error refs/heads/main some-actions-failed`, exits non-zero,
//! writes the `helper_push_partial_fail_sot` audit row (OP-3), and does NOT
//! advance the `last_fetched_at` cursor (those writes live on the `SotOk`
//! branch, which the partial-fail return skips).
//!
//! **Push 2 (recovery / convergence):** the agent's working tree is the
//! post-`git pull --rebase` state — issue 1 already reflects the landed write
//! (version 2, same body) and issue 2 still carries the un-landed edit. The
//! helper's PRECHECK B re-reads the current SoT via `list_changed_since`
//! (issue 1 shows as changed), `diff::plan` recomputes against that base, and
//! ONLY issue 2 is replanned — issue 1 is diffed away. Issue 2's PATCH now
//! succeeds and the push converges (`ok refs/heads/main`).
//!
//! The "replans ONLY the still-needed action" claim is asserted by the
//! wiremock `.expect()` counts: issue 1 is PATCHed exactly ONCE total (push 1
//! only — never re-attempted on push 2), issue 2 exactly TWICE (push 1 fail +
//! push 2 success).

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::too_many_lines)] // one narrow end-to-end recovery scenario reads top-to-bottom
#![allow(clippy::doc_markdown)] // test-internal doc comments cite SoT/PRECHECK/refs verbatim

use std::collections::HashMap;
use std::fmt::Write as _;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use assert_cmd::Command as AssertCommand;
use reposix_cache::Cache;
use serde_json::{json, Value};
use wiremock::matchers::{method, path_regex};
use wiremock::{Match, Mock, MockServer, Request, ResponseTemplate};

mod common;
use common::{count_audit_cache_rows, sim_backend, CacheDirGuard};

const PROJECT: &str = "demo";
const CREATED_AT: &str = "2026-04-13T00:00:00Z";

/// Matches a `GET /projects/<p>/issues?since=...` (the `list_changed_since`
/// delta query) so it can be disambiguated from the unfiltered
/// `list_records` on the same path.
struct HasSinceQueryParam;
impl Match for HasSinceQueryParam {
    fn matches(&self, req: &Request) -> bool {
        req.url.query_pairs().any(|(k, _)| k == "since")
    }
}

/// A single issue's mutable state in the modeled SoT.
#[derive(Clone)]
struct IssueState {
    title: String,
    body: String,
    version: u64,
}

/// The stateful backend behind the wiremock closures. `issues` is the SoT;
/// `landed` is the set of ids changed since the cache cursor (drives
/// `list_changed_since`); `patch2_attempts` gates issue 2's transient fault.
struct Sot {
    issues: Mutex<HashMap<u64, IssueState>>,
    landed: Mutex<Vec<u64>>,
    patch2_attempts: AtomicUsize,
}

fn issue_json(id: u64, st: &IssueState) -> Value {
    json!({
        "id": id,
        "title": st.title,
        "status": "open",
        "assignee": Value::Null,
        "labels": [],
        "created_at": CREATED_AT,
        "updated_at": CREATED_AT,
        "version": st.version,
        "body": st.body,
    })
}

/// Render an on-disk issue blob (frontmatter + body). `version` is forged so
/// we can model an agent's local base version explicitly (stale-base v1 on
/// push 1; post-rebase v2 for the already-landed issue 1 on push 2).
fn render_issue_blob(id: u64, version: u64, body: &str) -> String {
    let mut s = String::new();
    s.push_str("---\n");
    writeln!(&mut s, "id: {id}").unwrap();
    writeln!(&mut s, "title: issue {id} in {PROJECT}").unwrap();
    s.push_str("status: open\n");
    writeln!(&mut s, "created_at: {CREATED_AT}").unwrap();
    writeln!(&mut s, "updated_at: {CREATED_AT}").unwrap();
    writeln!(&mut s, "version: {version}").unwrap();
    s.push_str("---\n");
    s.push_str(body);
    if !s.ends_with('\n') {
        s.push('\n');
    }
    s
}

/// Build a single-backend `export` payload: `export\n` + a fast-import stream
/// updating each `(path, blob)` entry.
fn export_stdin(entries: &[(&str, String)], msg: &str) -> Vec<u8> {
    let mut stream: Vec<u8> = Vec::new();
    writeln!(&mut stream, "feature done").unwrap();
    let base_mark: u64 = 100;
    for (i, (_, blob)) in entries.iter().enumerate() {
        writeln!(&mut stream, "blob").unwrap();
        writeln!(&mut stream, "mark :{}", base_mark + i as u64).unwrap();
        writeln!(&mut stream, "data {}", blob.len()).unwrap();
        stream.extend_from_slice(blob.as_bytes());
        stream.push(b'\n');
    }
    writeln!(&mut stream, "commit refs/heads/main").unwrap();
    writeln!(&mut stream, "mark :1").unwrap();
    writeln!(&mut stream, "committer test <t@t> 0 +0000").unwrap();
    writeln!(&mut stream, "data {}", msg.len()).unwrap();
    stream.extend_from_slice(msg.as_bytes());
    stream.push(b'\n');
    for (i, (path, _)) in entries.iter().enumerate() {
        writeln!(&mut stream, "M 100644 :{} {path}", base_mark + i as u64).unwrap();
    }
    writeln!(&mut stream, "done").unwrap();

    let mut buf = Vec::new();
    writeln!(&mut buf, "export").unwrap();
    buf.extend_from_slice(&stream);
    buf
}

fn find_cache_bare(cache_dir: &Path) -> Option<PathBuf> {
    walkdir::WalkDir::new(cache_dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .find(|e| e.file_type().is_dir() && e.path().extension().is_some_and(|x| x == "git"))
        .map(|e| e.path().to_path_buf())
}

fn read_cursor(cache_db: &Path) -> Option<String> {
    let conn = rusqlite::Connection::open(cache_db).ok()?;
    conn.query_row(
        "SELECT value FROM meta WHERE key = 'last_fetched_at'",
        [],
        |r| r.get::<_, String>(0),
    )
    .ok()
}

/// Run the single-backend helper export path once and return `(success, stdout)`.
fn run_helper_export(url: &str, cache_dir: &Path, stdin: Vec<u8>) -> (bool, String) {
    let out = AssertCommand::cargo_bin("git-remote-reposix")
        .expect("binary built")
        .args(["origin", url])
        .env("REPOSIX_CACHE_DIR", cache_dir)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .write_stdin(stdin)
        .timeout(std::time::Duration::from_secs(20))
        .output()
        .expect("run helper");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

#[tokio::test(flavor = "multi_thread")]
// test-name-honesty: ok — drives the real helper export path twice against a stateful
// wiremock SoT; genuine partial-fail + PRECHECK-B replan recovery coverage.
async fn partial_fail_then_next_push_replans_only_remainder_and_converges() {
    // Modeled SoT: issues 1 and 2, both at version 1 with their seed bodies.
    let sot = Arc::new(Sot {
        issues: Mutex::new(HashMap::from([
            (
                1u64,
                IssueState {
                    title: "issue 1 in demo".into(),
                    body: "orig body 1\n".into(),
                    version: 1,
                },
            ),
            (
                2u64,
                IssueState {
                    title: "issue 2 in demo".into(),
                    body: "orig body 2\n".into(),
                    version: 1,
                },
            ),
        ])),
        landed: Mutex::new(Vec::new()),
        patch2_attempts: AtomicUsize::new(0),
    });

    let server = MockServer::start().await;

    // list_changed_since (?since=...): return the `landed` set as full
    // records. Higher priority than the unfiltered list route so the query
    // param disambiguates. Empty on push 1 (nothing landed yet), [issue 1]
    // on push 2 (issue 1's write landed in push 1).
    {
        let sot = sot.clone();
        Mock::given(method("GET"))
            .and(path_regex(format!(r"^/projects/{PROJECT}/issues$")))
            .and(HasSinceQueryParam)
            .respond_with(move |_req: &Request| {
                let issues = sot.issues.lock().unwrap();
                let landed = sot.landed.lock().unwrap();
                let body: Vec<Value> = landed
                    .iter()
                    .filter_map(|id| issues.get(id).map(|st| issue_json(*id, st)))
                    .collect();
                ResponseTemplate::new(200).set_body_json(body)
            })
            .with_priority(1)
            .mount(&server)
            .await;
    }

    // list_records (no since): the full current SoT. Used by the warm sync,
    // and by PRECHECK B's plan()-prior materialization fallback.
    {
        let sot = sot.clone();
        Mock::given(method("GET"))
            .and(path_regex(format!(r"^/projects/{PROJECT}/issues$")))
            .respond_with(move |_req: &Request| {
                let issues = sot.issues.lock().unwrap();
                let mut ids: Vec<u64> = issues.keys().copied().collect();
                ids.sort_unstable();
                let body: Vec<Value> = ids.iter().map(|id| issue_json(*id, &issues[id])).collect();
                ResponseTemplate::new(200).set_body_json(body)
            })
            .with_priority(5)
            .mount(&server)
            .await;
    }

    // GET /issues/1 — PRECHECK B's hot-path conflict check on push 2 (issue 1
    // is in the changed set) re-fetches issue 1's current SoT version.
    {
        let sot = sot.clone();
        Mock::given(method("GET"))
            .and(path_regex(format!(r"^/projects/{PROJECT}/issues/1$")))
            .respond_with(move |_req: &Request| {
                let issues = sot.issues.lock().unwrap();
                ResponseTemplate::new(200).set_body_json(issue_json(1, &issues[&1]))
            })
            .mount(&server)
            .await;
    }

    // PATCH /issues/1 → 200. Applies the edit and marks issue 1 as landed.
    // expect(1): issue 1 is written exactly once across BOTH pushes — on
    // push 2 it must be diffed away (already at target), never re-PATCHed.
    {
        let sot = sot.clone();
        Mock::given(method("PATCH"))
            .and(path_regex(format!(r"^/projects/{PROJECT}/issues/1$")))
            .respond_with(move |req: &Request| {
                let patch: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
                let mut issues = sot.issues.lock().unwrap();
                let st = issues.get_mut(&1).unwrap();
                if let Some(b) = patch.get("body").and_then(Value::as_str) {
                    st.body = b.to_owned();
                }
                st.version += 1;
                let resp = issue_json(1, st);
                drop(issues);
                sot.landed.lock().unwrap().push(1);
                ResponseTemplate::new(200).set_body_json(resp)
            })
            .expect(1)
            .mount(&server)
            .await;
    }

    // PATCH /issues/2 → 500 on the first attempt (partial fail), 200 on the
    // second (recovery). expect(2): push 1 fail + push 2 success.
    {
        let sot = sot.clone();
        Mock::given(method("PATCH"))
            .and(path_regex(format!(r"^/projects/{PROJECT}/issues/2$")))
            .respond_with(move |req: &Request| {
                let attempt = sot.patch2_attempts.fetch_add(1, Ordering::SeqCst);
                if attempt == 0 {
                    return ResponseTemplate::new(500)
                        .set_body_json(json!({ "error": "internal_server_error" }));
                }
                let patch: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
                let mut issues = sot.issues.lock().unwrap();
                let st = issues.get_mut(&2).unwrap();
                if let Some(b) = patch.get("body").and_then(Value::as_str) {
                    st.body = b.to_owned();
                }
                st.version += 1;
                let resp = issue_json(2, st);
                ResponseTemplate::new(200).set_body_json(resp)
            })
            .expect(2)
            .mount(&server)
            .await;
    }

    let cache_root = tempfile::tempdir().expect("cache_root");
    let _env = CacheDirGuard::new(cache_root.path());

    // Warm the cache cursor (build_from → oid_map for issues 1,2; blobs lazy;
    // last_fetched_at set) so the push path takes PRECHECK B's cursor-present
    // hot path (list_changed_since), not the first-push fallback.
    let backend = sim_backend(&server);
    let cache = Cache::open(backend, "sim", PROJECT).expect("Cache::open");
    cache.sync().await.expect("warm sync");
    drop(cache);

    let cache_bare = find_cache_bare(cache_root.path()).expect("cache bare after warm sync");
    let cache_db = cache_bare.join("cache.db");
    let cursor_warm = read_cursor(&cache_db).expect("cursor set by warm sync");

    let url = format!("reposix::{}/projects/{PROJECT}", server.uri());

    // ---- Push 1: edit issues 1 and 2 from base v1 → issue 1 lands, issue 2
    // fails (500) → partial fail.
    let push1 = export_stdin(
        &[
            ("issues/1.md", render_issue_blob(1, 1, "new body 1\n")),
            ("issues/2.md", render_issue_blob(2, 1, "new body 2\n")),
        ],
        "edit issues 1 and 2\n",
    );
    let (ok1, stdout1) = tokio::task::spawn_blocking({
        let url = url.clone();
        let dir = cache_root.path().to_path_buf();
        move || run_helper_export(&url, &dir, push1)
    })
    .await
    .unwrap();

    assert!(!ok1, "push 1 must fail (partial fail); stdout={stdout1}");
    assert!(
        stdout1.contains("error refs/heads/main some-actions-failed"),
        "push 1 must emit some-actions-failed; stdout={stdout1}"
    );

    // OP-3: exactly one helper_push_partial_fail_sot row naming the outcome.
    let partial_rows = count_audit_cache_rows(&cache_db, "helper_push_partial_fail_sot");
    assert_eq!(
        partial_rows, 1,
        "expected exactly 1 helper_push_partial_fail_sot audit row after partial fail, got {partial_rows}"
    );
    // The successful convergence branch was NOT reached on push 1.
    assert_eq!(
        count_audit_cache_rows(&cache_db, "helper_push_accepted"),
        0,
        "helper_push_accepted must be 0 after a partial fail"
    );

    // No cursor advance on the failed push (SotOk-branch write skipped).
    assert_eq!(
        read_cursor(&cache_db).as_deref(),
        Some(cursor_warm.as_str()),
        "last_fetched_at must NOT advance on a partial-fail push"
    );

    // ---- Push 2: post-`git pull --rebase` working tree. Issue 1 now reflects
    // the landed write (version 2, same body) so plan() diffs it away; issue 2
    // still carries the un-landed edit (version 1) so it is the ONLY replanned
    // action, and its PATCH now succeeds → convergence.
    let push2 = export_stdin(
        &[
            ("issues/1.md", render_issue_blob(1, 2, "new body 1\n")),
            ("issues/2.md", render_issue_blob(2, 1, "new body 2\n")),
        ],
        "retry after partial fail\n",
    );
    let (ok2, stdout2) = tokio::task::spawn_blocking({
        let url = url.clone();
        let dir = cache_root.path().to_path_buf();
        move || run_helper_export(&url, &dir, push2)
    })
    .await
    .unwrap();

    assert!(
        ok2,
        "push 2 (recovery) must succeed and converge; stdout={stdout2}"
    );
    assert!(
        stdout2.contains("ok refs/heads/main"),
        "push 2 must emit ok refs/heads/main; stdout={stdout2}"
    );

    // Convergence: both issues now hold the agent's edits at the SoT.
    {
        let issues = sot.issues.lock().unwrap();
        assert_eq!(issues[&1].body, "new body 1\n", "issue 1 converged");
        assert_eq!(issues[&2].body, "new body 2\n", "issue 2 converged");
        assert_eq!(issues[&2].version, 2, "issue 2 was written exactly once");
    }

    // wiremock Drop enforces expect(1) on PATCH /issues/1 (issue 1 was NOT
    // re-attempted on push 2 — replanned ONLY the still-needed issue 2) and
    // expect(2) on PATCH /issues/2 (fail + recovery). Keep server in scope
    // until here so those checks run at end-of-test.
    drop(server);
}
