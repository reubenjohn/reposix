//! DP-2 regression guard for the ghost-`oid_map`-row fix (P93 HIGH,
//! code-reviewer finding). CONFIRMED-RED-then-fixed under Strategy 1 (prune
//! `oid_map` on sync — see `.planning/CONSULT-DECISIONS.md` § D-P93-02); now
//! GREEN and permanently enabled to keep the defect from reappearing.
//!
//! The bug (before the fix): `Cache::sync`/`Cache::build_from` only upserted
//! `oid_map` (`INSERT OR REPLACE`, never `DELETE`), so an upstream-deleted
//! record's `oid_map` row survived forever. `Cache::list_record_ids()`
//! (`SELECT DISTINCT issue_id FROM oid_map` — unfiltered) resurrected the
//! dead id. `precheck.rs`'s steady-state branch (reached once every
//! `oid_map` blob is already materialized — the NORMAL case after an agent
//! has read its issues) used that stale id set as `diff::plan`'s `prior`.
//! `plan()` saw the id absent from the pushed tree and emitted a phantom
//! `PlannedAction::Delete`; `execute_action` -> `delete_or_close` 404'd
//! (already gone) -> `Error::NotFound`; `write_loop`'s `failed_ids` turned
//! that into `SotPartialFail` + a FALSE `helper_push_partial_fail_sot` audit
//! row — on every push, forever, even though the agent did nothing wrong.
//!
//! The fix prunes `oid_map` rows whose `issue_id` left `list_records`, inside
//! `sync`'s Step-5 transaction (and the `build_from`/`--reconcile` rebuild),
//! restoring tree↔`oid_map` coherence in the DELETION direction (the mirror
//! of Lane 1's ADDITION-direction fix). The assertions below intentionally do
//! NOT pin down `oid_map`'s row count (implementation detail) — they pin the
//! OBSERVABLE contract the fix restores, so it stays valid under either
//! ratified strategy: (1) the push succeeds, (2) no phantom DELETE ever
//! reaches the backend for the already-gone id, (3) no false
//! `helper_push_partial_fail_sot` audit row is written.
//!
//! Repro shape (mirrors `tests/partial_failure_recovery.rs`'s stateful
//! wiremock harness):
//! 1. Seed a 2-issue `SoT`; `Cache::sync()` (seed path == `build_from`) writes
//!    `oid_map` rows for both ids, blobs lazy.
//! 2. Materialize BOTH blobs via `cache.read_blob` — models an agent who has
//!    read both issues (the ordinary, common case; this is what puts
//!    `precheck.rs`'s branch into the "fully materialized" steady state
//!    instead of its `list_records` fallback).
//! 3. Delete issue 2 from the modeled `SoT` (upstream deletion — nobody local
//!    did anything).
//! 4. `Cache::sync()` again (delta path) — advances the cursor, rebuilds the
//!    HEAD tree from the now-2-issues-minus-1 `list_records`. LINK (a),
//!    execute-verified (printed, not asserted — implementation detail):
//!    does `list_record_ids()` still resurrect the dead id after this?
//! 5. Run the REAL `git-remote-reposix export` path (only way to exercise
//!    `pub(crate)` `precheck`/`diff`/`write_loop` from an integration test)
//!    pushing a tree that reflects an agent's ordinary post-pull working
//!    copy: `issues/1.md` present and byte-identical to the `SoT` (so zero
//!    Create/Update actions), `issues/2.md` absent (nobody re-added it — it
//!    is just gone, matching the file a `git pull` would have removed).
//! 6. LINK (b), execute-verified via the assertions: does the export planner
//!    actually emit + execute a Delete for the gone id (a phantom DELETE
//!    hitting the sim's `DELETE /projects/demo/issues/2` route, which 404s —
//!    already deleted — exactly as `SimBackend::delete_or_close` does in
//!    production per `delete_or_close_404_maps_to_not_found`), forcing
//!    `SotPartialFail` plus a `helper_push_partial_fail_sot` audit row for a
//!    push that had no real work left to do?

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::too_many_lines)] // one narrow end-to-end repro scenario reads top-to-bottom

use std::collections::HashMap;
use std::fmt::Write as _;
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use assert_cmd::Command as AssertCommand;
use reposix_cache::Cache;
use reposix_core::RecordId;
use serde_json::{json, Value};
use wiremock::matchers::{method, path_regex};
use wiremock::{Match, Mock, MockServer, Request, ResponseTemplate};

mod common;
use common::{count_audit_cache_rows, sim_backend, CacheDirGuard};

const PROJECT: &str = "demo";
const CREATED_AT: &str = "2026-04-13T00:00:00Z";

/// Matches a `GET /projects/<p>/issues?since=...` (`list_changed_since`) so
/// it can be disambiguated from the unfiltered `list_records` on the same
/// path — same trick as `partial_failure_recovery.rs`.
struct HasSinceQueryParam;
impl Match for HasSinceQueryParam {
    fn matches(&self, req: &Request) -> bool {
        req.url.query_pairs().any(|(k, _)| k == "since")
    }
}

/// A single issue's state in the modeled `SoT`.
#[derive(Clone)]
struct IssueState {
    title: String,
    body: String,
    version: u64,
}

/// The stateful backend behind the wiremock closures. `issues` is the `SoT` —
/// removing a key models an upstream delete nobody local initiated.
struct Sot {
    issues: Mutex<HashMap<u64, IssueState>>,
    delete_attempts_on_2: AtomicUsize,
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

/// Render an on-disk issue blob byte-identical to what the `SoT` would report
/// for `(id, version, body)` via the same title convention `issue_json` uses.
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

/// Build a single-backend `export` payload: `export\n` + a fast-import
/// stream touching each `(path, blob)` entry. Only entries actually present
/// in the agent's working tree belong here — a record's file simply being
/// ABSENT (no explicit `D` line needed) is exactly what `diff::plan` treats
/// as a delete candidate when the id is still present in `prior`.
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

fn find_cache_bare(cache_dir: &std::path::Path) -> Option<std::path::PathBuf> {
    walkdir::WalkDir::new(cache_dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .find(|e| e.file_type().is_dir() && e.path().extension().is_some_and(|x| x == "git"))
        .map(|e| e.path().to_path_buf())
}

/// Run the single-backend helper export path once and return `(success, stdout)`.
fn run_helper_export(url: &str, cache_dir: &std::path::Path, stdin: Vec<u8>) -> (bool, String) {
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
// test-name-honesty: ok — drives the real helper export path against a
// stateful wiremock SoT that models an upstream delete; genuine end-to-end
// repro of the false-SotPartialFail-forever bug, not a code-reading proxy.
async fn deleted_record_ghost_oid_map_row_forces_false_partial_fail() {
    // Modeled SoT: issues 1 and 2, both at version 1.
    let sot = Arc::new(Sot {
        issues: Mutex::new(HashMap::from([
            (
                1u64,
                IssueState {
                    title: format!("issue 1 in {PROJECT}"),
                    body: "orig body 1\n".into(),
                    version: 1,
                },
            ),
            (
                2u64,
                IssueState {
                    title: format!("issue 2 in {PROJECT}"),
                    body: "orig body 2\n".into(),
                    version: 1,
                },
            ),
        ])),
        delete_attempts_on_2: AtomicUsize::new(0),
    });

    let server = MockServer::start().await;

    // list_changed_since (?since=...): nothing "changed" in the REST sense —
    // upstream deletes never show up as a changed-id (they simply vanish
    // from list_records). Empty on every call in this repro.
    Mock::given(method("GET"))
        .and(path_regex(format!(r"^/projects/{PROJECT}/issues$")))
        .and(HasSinceQueryParam)
        .respond_with(ResponseTemplate::new(200).set_body_json(Vec::<Value>::new()))
        .with_priority(1)
        .mount(&server)
        .await;

    // list_records (no since): the full current SoT — drives build_from's
    // seed tree, the delta sync's step-4 rebuild, and (deliberately absent
    // here) the first-push fallback.
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

    // GET /issues/<id>: per-record fetch used by Cache::read_blob's
    // materialization. 404s once the id is gone from the SoT map (models a
    // real deleted record), matching SimBackend's actual 404 contract.
    {
        let sot = sot.clone();
        Mock::given(method("GET"))
            .and(path_regex(format!(r"^/projects/{PROJECT}/issues/\d+$")))
            .respond_with(move |req: &Request| {
                let id: u64 = req
                    .url
                    .path_segments()
                    .and_then(Iterator::last)
                    .and_then(|s| s.parse().ok())
                    .expect("issue id in path");
                let issues = sot.issues.lock().unwrap();
                match issues.get(&id) {
                    Some(st) => ResponseTemplate::new(200).set_body_json(issue_json(id, st)),
                    None => ResponseTemplate::new(404),
                }
            })
            .mount(&server)
            .await;
    }

    // DELETE /issues/2 — the phantom action the bug plans. The record is
    // already gone from the modeled SoT, so this always 404s, exactly as
    // `SimBackend::delete_or_close` maps a real double-delete to
    // `Error::NotFound` (see `delete_or_close_404_maps_to_not_found`). No
    // `.expect(n)` pinned here — the exact hit count is what the assertions
    // below check explicitly (desired: 0; currently: 1, the bug).
    {
        let sot = sot.clone();
        Mock::given(method("DELETE"))
            .and(path_regex(format!(r"^/projects/{PROJECT}/issues/2$")))
            .respond_with(move |_req: &Request| {
                sot.delete_attempts_on_2.fetch_add(1, Ordering::SeqCst);
                ResponseTemplate::new(404)
            })
            .mount(&server)
            .await;
    }

    let cache_root = tempfile::tempdir().expect("cache_root");
    let _env = CacheDirGuard::new(cache_root.path());

    // Step 1: seed sync (build_from) — oid_map rows for BOTH ids, blobs lazy.
    let backend = sim_backend(&server);
    let cache = Cache::open(backend, "sim", PROJECT).expect("Cache::open");
    let seed_report = cache.sync().await.expect("seed sync");
    assert!(seed_report.since.is_none(), "seed sync has no cursor");

    // Step 2: materialize BOTH blobs — models an agent who has read both
    // issues (precheck.rs's steady-state branch requires this: it only
    // trusts oid_map wholesale once materialized_count == ids.len()).
    let oid1 = cache
        .find_oid_for_record(RecordId(1))
        .expect("find_oid_for_record(1)")
        .expect("issue 1 has an oid_map row after seed sync");
    let oid2 = cache
        .find_oid_for_record(RecordId(2))
        .expect("find_oid_for_record(2)")
        .expect("issue 2 has an oid_map row after seed sync");
    cache.read_blob(oid1).await.expect("materialize issue 1");
    cache.read_blob(oid2).await.expect("materialize issue 2");

    // Step 3: upstream deletes issue 2. No local agent action involved.
    sot.issues.lock().unwrap().remove(&2);

    // Step 4: delta sync — advances the cursor, rebuilds the HEAD tree from
    // list_records (now issue 1 only). Per the bug, issue 2's oid_map row is
    // NEVER pruned (no DELETE FROM oid_map anywhere in build_from/sync).
    let delta_report = cache
        .sync()
        .await
        .expect("delta sync after upstream delete");
    assert!(
        delta_report.since.is_some(),
        "second sync must take the delta path (cursor present)"
    );

    // LINK (a), execute-verified and reported (NOT asserted — this is an
    // implementation detail a valid fix may or may not change; see module
    // doc). Printed with `--nocapture` for the record.
    let ids_after_delete: Vec<u64> = cache
        .list_record_ids()
        .expect("list_record_ids after delta sync")
        .iter()
        .map(|r| r.0)
        .collect();
    println!(
        "LINK (a): list_record_ids() after upstream-delete + sync = {ids_after_delete:?} \
         (ghost id 2 {} present)",
        if ids_after_delete.contains(&2) {
            "IS"
        } else {
            "is NOT"
        }
    );

    let cache_bare = find_cache_bare(cache_root.path()).expect("cache bare after syncs");
    let cache_db = cache_bare.join("cache.db");
    drop(cache);

    let url = format!("reposix::{}/projects/{PROJECT}", server.uri());

    // Step 5/6: push the agent's ordinary post-pull working tree — issue 1
    // present and byte-identical to the SoT (zero real actions), issue 2's
    // file simply absent (as it would be after a `git pull` merged in the
    // cache's now-issue-2-less tree). No edits, no explicit `D` line.
    let push = export_stdin(
        &[("issues/1.md", render_issue_blob(1, 1, "orig body 1\n"))],
        "routine push after nothing local changed\n",
    );
    let (ok, stdout) = tokio::task::spawn_blocking({
        let url = url.clone();
        let dir = cache_root.path().to_path_buf();
        move || run_helper_export(&url, &dir, push)
    })
    .await
    .unwrap();

    // DESIRED/correct behavior (what a fix must restore) — a push with
    // genuinely nothing left to do (the only prior "difference" is a record
    // the SoT itself already deleted) must succeed cleanly:
    assert!(
        ok,
        "a no-real-work push must succeed, not false-fail on a ghost id; \
         stdout={stdout}"
    );
    assert!(
        stdout.contains("ok refs/heads/main"),
        "expected a clean ok refs/heads/main; stdout={stdout}"
    );
    // LINK (b) as a correctness assertion: no phantom DELETE may ever reach
    // the backend for an id the SoT itself already removed.
    assert_eq!(
        sot.delete_attempts_on_2.load(Ordering::SeqCst),
        0,
        "no phantom DELETE should be issued for an already-upstream-deleted id"
    );
    assert_eq!(
        count_audit_cache_rows(&cache_db, "helper_push_partial_fail_sot"),
        0,
        "no false helper_push_partial_fail_sot audit row should be written"
    );

    drop(server);
}
