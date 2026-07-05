//! Bus write fan-out audit-completeness integration test (DVCS-BUS-WRITE-06).
//!
//! RESEARCH.md § "Test (d)" + § "Audit Completeness Contract":
//! happy-path bus push writes ALL expected rows to BOTH audit layers
//! per OP-3:
//!
//! | Layer                      | Implementation                                                     |
//! |----------------------------|--------------------------------------------------------------------|
//! | `audit_events_cache`       | SQLite cache.db; queried directly via `count_audit_cache_rows`.    |
//! | `audit_events` (sim DB)    | REAL `reposix-sim` SQLite file; queried directly (see below).      |
//!
//! ## P92 SC2+SC3 upgrade: real dual-table query, not a wiremock proxy
//!
//! Prior state (PASS through 83-02) asserted the `audit_events` (backend)
//! layer via wiremock's REQUEST LOG as a proxy for a real audit table — a
//! metaphor, not a real dual-table query (P83-02 read_first finding: "the
//! sim's audit_events table location requires a sibling helper ... file as
//! SURPRISES-INTAKE if not directly queryable in test scope"). This upgrade
//! closes that gap by spinning a REAL `reposix-sim` axum process in-process
//! (`reposix_sim::run_with_listener`, the same library entry point
//! `tests/stateless_connect_e2e.rs` already uses) backed by a PERSISTENT
//! (non-ephemeral) `SQLite` file, pre-seeded via the sim's own production
//! `reposix_sim::seed::apply_seed` — not a hand-rolled `INSERT` — so the
//! seeded rows are indistinguishable from a normal `--seed-file` boot. After
//! the push, the test opens that SAME file with `rusqlite::Connection::open`
//! and queries `audit_events` directly (`real-git-push-e2e.sh`'s shell
//! pattern, ported to Rust). No new dependency: `reposix-sim` and
//! `rusqlite` are both already present in this crate's manifest.
//!
//! A useful side effect: because the sim's own routes now serve the
//! list/get/patch requests for real, the wiremock priority-mocks that used
//! to hand-tune "precheck B stable" and the PATCH response body are gone —
//! the real backend's own read-your-writes behavior produces the same
//! shape without any mocking at all.
//!
//! Test name: `bus_write_audit_completeness_happy_path_writes_both_tables`.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::unnecessary_debug_formatting)]

use std::fmt::Write as _;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use std::sync::Arc;

use assert_cmd::Command as AssertCommand;
use chrono::TimeZone;
use reposix_cache::Cache;
use reposix_core::BackendConnector;
use reposix_sim::seed::{apply_seed, SeedFile, SeedIssue, SeedProject};

mod common;
use common::{count_audit_cache_rows, CacheDirGuard};

/// Spawn a REAL `reposix-sim` axum process in-process on a random loopback
/// port, backed by a PERSISTENT `SQLite` file at `db_path`. Mirrors
/// `tests/stateless_connect_e2e.rs::spawn_sim`, generalized to accept a
/// caller-supplied persistent path (that test uses `:memory:` +
/// `ephemeral: true` since it never queries the DB after the fact; this
/// test needs the file to survive the push so it can query `audit_events`
/// directly afterward).
async fn spawn_real_sim(db_path: &Path) -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind 127.0.0.1:0");
    let addr = listener.local_addr().expect("local_addr");
    let origin = format!("http://{addr}");

    let cfg = reposix_sim::SimConfig {
        bind: addr,
        db_path: db_path.to_path_buf(),
        seed: false,
        seed_file: None,
        ephemeral: false,
        rate_limit_rps: 1000,
    };
    let handle = tokio::spawn(async move {
        let _ = reposix_sim::run_with_listener(listener, cfg).await;
    });
    let client =
        reposix_core::http::client(reposix_core::http::ClientOpts::default()).expect("http client");
    for _ in 0..50 {
        if let Ok(r) = client.get(format!("{origin}/healthz")).await {
            if r.status().is_success() {
                return (origin, handle);
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    panic!("sim did not become healthy at {origin}");
}

/// Pre-seed `db_path` (not yet opened by any server) with `n` deterministic
/// issues field-matching `render_issue_blob`'s unchanged shape for ids 2..n
/// (and id 1's PRE-edit shape) via the sim's own production seeding code
/// (`reposix_sim::seed::apply_seed`) — not a hand-rolled `INSERT INTO`.
fn seed_sim_db(db_path: &Path, project: &str, n: u64) {
    let conn = reposix_sim::db::open_db(db_path, false).expect("open sim db for seeding");
    let issues = (1..=n)
        .map(|i| SeedIssue {
            id: i,
            title: format!("issue {i} in {project}"),
            status: "open".to_owned(),
            assignee: None,
            labels: vec![],
            body: format!("body of issue {i}"),
        })
        .collect();
    let seed = SeedFile {
        project: SeedProject {
            slug: project.to_owned(),
            name: project.to_owned(),
            description: String::new(),
        },
        issues,
    };
    let inserted = apply_seed(&conn, &seed).expect("apply_seed");
    let want = usize::try_from(n).unwrap_or(usize::MAX);
    assert_eq!(inserted, want, "expected {n} issues seeded, got {inserted}");
    drop(conn);
}

fn run_git_in(dir: &Path, args: &[&str]) -> String {
    let out = StdCommand::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .unwrap_or_else(|e| panic!("spawn git {args:?}: {e}"));
    assert!(
        out.status.success(),
        "git {args:?} in {dir:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_owned()
}

fn render_issue_blob(id: u64, version: u64, body: &str) -> String {
    let t = chrono::Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
    let ts = t.to_rfc3339();
    let mut s = String::new();
    s.push_str("---\n");
    writeln!(&mut s, "id: {id}").unwrap();
    writeln!(&mut s, "title: issue {id} in demo").unwrap();
    s.push_str("status: open\n");
    writeln!(&mut s, "created_at: {ts}").unwrap();
    writeln!(&mut s, "updated_at: {ts}").unwrap();
    writeln!(&mut s, "version: {version}").unwrap();
    s.push_str("---\n");
    s.push_str(body);
    if !s.ends_with('\n') {
        s.push('\n');
    }
    s
}

fn multi_file_export(entries: &[(&str, String)], msg: &str) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    writeln!(&mut out, "feature done").unwrap();
    let base_mark: u64 = 100;
    for (i, (_, blob)) in entries.iter().enumerate() {
        writeln!(&mut out, "blob").unwrap();
        writeln!(&mut out, "mark :{}", base_mark + i as u64).unwrap();
        writeln!(&mut out, "data {}", blob.len()).unwrap();
        out.extend_from_slice(blob.as_bytes());
        out.push(b'\n');
    }
    writeln!(&mut out, "commit refs/heads/main").unwrap();
    writeln!(&mut out, "mark :1").unwrap();
    writeln!(&mut out, "committer test <t@t> 0 +0000").unwrap();
    let bytes = msg.as_bytes();
    writeln!(&mut out, "data {}", bytes.len()).unwrap();
    out.extend_from_slice(bytes);
    out.push(b'\n');
    for (i, (path, _)) in entries.iter().enumerate() {
        writeln!(&mut out, "M 100644 :{} {path}", base_mark + i as u64).unwrap();
    }
    writeln!(&mut out, "done").unwrap();
    out
}

fn find_cache_bare(cache_dir: &Path) -> Option<PathBuf> {
    walkdir::WalkDir::new(cache_dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .find(|e| e.file_type().is_dir() && e.path().extension().is_some_and(|x| x == "git"))
        .map(|e| e.path().to_path_buf())
}

fn make_synced_mirror_fixture() -> (tempfile::TempDir, tempfile::TempDir, String) {
    let mirror = tempfile::tempdir().expect("mirror tempdir");
    let wtree = tempfile::tempdir().expect("wtree tempdir");
    let scratch = tempfile::tempdir().expect("scratch tempdir");

    run_git_in(mirror.path(), &["init", "--bare", "."]);
    run_git_in(scratch.path(), &["init", "."]);
    run_git_in(scratch.path(), &["config", "user.email", "p83-02@example"]);
    run_git_in(scratch.path(), &["config", "user.name", "P83-02 Test"]);
    run_git_in(scratch.path(), &["checkout", "-b", "main"]);
    std::fs::write(scratch.path().join("seed.txt"), "seed").unwrap();
    run_git_in(scratch.path(), &["add", "seed.txt"]);
    run_git_in(scratch.path(), &["commit", "-m", "seed"]);
    let synced_sha = run_git_in(scratch.path(), &["rev-parse", "HEAD"]);

    let mirror_url = format!("file://{}", mirror.path().display());
    run_git_in(scratch.path(), &["remote", "add", "mirror", &mirror_url]);
    run_git_in(scratch.path(), &["push", "mirror", "HEAD:refs/heads/main"]);

    run_git_in(wtree.path(), &["init", "."]);
    run_git_in(wtree.path(), &["config", "user.email", "p83-02@example"]);
    run_git_in(wtree.path(), &["config", "user.name", "P83-02 Test"]);
    run_git_in(wtree.path(), &["remote", "add", "mirror", &mirror_url]);
    run_git_in(wtree.path(), &["fetch", "mirror"]);
    run_git_in(
        wtree.path(),
        &["update-ref", "refs/remotes/mirror/main", &synced_sha],
    );
    run_git_in(
        wtree.path(),
        &["update-ref", "refs/heads/main", &synced_sha],
    );
    run_git_in(wtree.path(), &["symbolic-ref", "HEAD", "refs/heads/main"]);

    (wtree, mirror, mirror_url)
}

#[tokio::test(flavor = "multi_thread")]
async fn bus_write_audit_completeness_happy_path_writes_both_tables() {
    let project = "demo";

    // Real, persistent sim DB — pre-seeded via production seeding code
    // BEFORE the server binds, then handed to the in-process axum server.
    // Kept alive (not dropped) for the rest of the test: assertion 3 below
    // opens this SAME file and queries `audit_events` directly.
    let sim_db_dir = tempfile::tempdir().expect("sim_db_dir");
    let sim_db_path = sim_db_dir.path().join("sim.db");
    seed_sim_db(&sim_db_path, project, 3);
    let (sim_origin, _sim_handle) = spawn_real_sim(&sim_db_path).await;

    let cache_root = tempfile::tempdir().expect("cache_root");
    let _env = CacheDirGuard::new(cache_root.path());
    let backend: Arc<dyn BackendConnector> = Arc::new(
        reposix_core::backend::sim::SimBackend::new(sim_origin.clone()).expect("SimBackend::new"),
    );
    let cache = Cache::open(backend, "sim", project).expect("Cache::open");
    cache.sync().await.expect("seed sync (warm cache cursor)");
    drop(cache);

    // No wiremock priority-mocks needed: the real sim's own `GET
    // .../issues?since=<cursor>` naturally returns `[]` (nothing changed
    // since seeding — PRECHECK B Stable for free), and the real `PATCH
    // .../issues/1` naturally bumps version 1 -> 2 and writes a genuine
    // `audit_events` row (queried directly below).

    let (wtree, mirror_bare, mirror_url) = make_synced_mirror_fixture();
    let bus_url = format!("reposix::{sim_origin}/projects/{project}?mirror={mirror_url}");

    // Fast-export: id=1 changed → 1 PATCH; ids 2+3 unchanged.
    let blob1 = render_issue_blob(1, 1, "edited body for 1\n");
    let blob2 = render_issue_blob(2, 1, "body of issue 2");
    let blob3 = render_issue_blob(3, 1, "body of issue 3");
    let entries: Vec<(&str, String)> = vec![
        ("issues/1.md", blob1),
        ("issues/2.md", blob2),
        ("issues/3.md", blob3),
    ];
    let stream = multi_file_export(&entries, "edit issue 1\n");
    let stdin_data = {
        let mut buf = Vec::new();
        writeln!(&mut buf, "capabilities").unwrap();
        writeln!(&mut buf).unwrap();
        writeln!(&mut buf, "export").unwrap();
        buf.extend_from_slice(&stream);
        buf
    };

    let cache_path = cache_root.path().to_path_buf();
    let out = AssertCommand::cargo_bin("git-remote-reposix")
        .expect("cargo bin")
        .args(["origin", &bus_url])
        .write_stdin(stdin_data)
        .current_dir(wtree.path())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("REPOSIX_CACHE_DIR", &cache_path)
        .timeout(std::time::Duration::from_secs(30))
        .output()
        .expect("run helper");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    // ASSERTION 1: helper exits zero.
    assert!(
        out.status.success(),
        "helper must exit zero on full happy path; \
         stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stdout.contains("ok refs/heads/main"),
        "missing `ok refs/heads/main` ack; stdout={stdout}, stderr={stderr}"
    );

    // ASSERTION 2: audit_events_cache (cache.db) row counts match the
    //  RESEARCH.md "Audit Completeness Contract" row 1 (SoT-ok-mirror-ok).
    let cache_bare =
        find_cache_bare(cache_root.path()).expect("cache bare dir must exist after push");
    let db_path = cache_bare.join("cache.db");

    // helper_backend_instantiated: >= 1 (every cache open writes one).
    let backend_inst = count_audit_cache_rows(&db_path, "helper_backend_instantiated");
    assert!(
        backend_inst >= 1,
        "expected >= 1 helper_backend_instantiated row, got {backend_inst}"
    );

    // helper_push_started: 1.
    let started = count_audit_cache_rows(&db_path, "helper_push_started");
    assert_eq!(
        started, 1,
        "expected 1 helper_push_started row, got {started}"
    );

    // helper_push_accepted: 1.
    let accepted = count_audit_cache_rows(&db_path, "helper_push_accepted");
    assert_eq!(
        accepted, 1,
        "expected 1 helper_push_accepted row, got {accepted}"
    );

    // mirror_sync_written: 1 (full happy path — mirror push landed).
    let synced = count_audit_cache_rows(&db_path, "mirror_sync_written");
    assert_eq!(
        synced, 1,
        "expected 1 mirror_sync_written row, got {synced}"
    );

    // helper_push_partial_fail_mirror_lag: 0 (happy path — no partial-fail).
    let partial = count_audit_cache_rows(&db_path, "helper_push_partial_fail_mirror_lag");
    assert_eq!(
        partial, 0,
        "expected 0 helper_push_partial_fail_mirror_lag rows on happy path, got {partial}"
    );

    // ASSERTION 3: audit_events (backend) — REAL query, not a wiremock
    //  request-log proxy (P92 SC2+SC3). Opens the SAME sqlite file the
    //  in-process sim server is still holding open (WAL mode + 5s
    //  busy_timeout, set by `reposix_sim::db::open_db`, tolerates the
    //  concurrent reader) and queries `audit_events` — the table
    //  `reposix-core::audit` owns and the sim's audit middleware writes
    //  one row per HTTP request into — directly via `rusqlite`.
    let sim_audit_conn =
        rusqlite::Connection::open(&sim_db_path).expect("open sim db for audit_events query");
    let patch_count: i64 = sim_audit_conn
        .query_row(
            "SELECT COUNT(*) FROM audit_events WHERE method = 'PATCH' AND path = ?1",
            rusqlite::params![format!("/projects/{project}/issues/1")],
            |r| r.get(0),
        )
        .expect("count PATCH audit_events rows");
    let post_count: i64 = sim_audit_conn
        .query_row(
            "SELECT COUNT(*) FROM audit_events WHERE method = 'POST' AND path = ?1",
            rusqlite::params![format!("/projects/{project}/issues")],
            |r| r.get(0),
        )
        .expect("count POST audit_events rows");
    let delete_count: i64 = sim_audit_conn
        .query_row(
            "SELECT COUNT(*) FROM audit_events WHERE method = 'DELETE' AND path LIKE ?1",
            rusqlite::params![format!("/projects/{project}/issues/%")],
            |r| r.get(0),
        )
        .expect("count DELETE audit_events rows");
    drop(sim_audit_conn);

    assert_eq!(
        patch_count, 1,
        "expected exactly 1 real audit_events PATCH row for issue 1 \
         (OP-3 backend-mutation table, queried directly — not a wiremock proxy)"
    );
    assert_eq!(
        post_count, 0,
        "expected 0 audit_events POST (create) rows on a single-update happy path, got {post_count}"
    );
    assert_eq!(
        delete_count, 0,
        "expected 0 audit_events DELETE rows on a single-update happy path, got {delete_count}"
    );

    // ASSERTION 4: mirror's main ref points at new SoT SHA (full happy
    //  path — mirror push landed).
    let mirror_main = StdCommand::new("git")
        .arg("-C")
        .arg(mirror_bare.path())
        .args(["rev-parse", "main"])
        .output()
        .expect("rev-parse mirror after");
    assert!(
        mirror_main.status.success(),
        "mirror's main ref should resolve after happy-path push; stderr={}",
        String::from_utf8_lossy(&mirror_main.stderr)
    );

    // ASSERTION 5: refs/mirrors/<sot>-head and -synced-at populated.
    let refs_out = StdCommand::new("git")
        .arg("-C")
        .arg(&cache_bare)
        .args(["for-each-ref", "refs/mirrors/"])
        .output()
        .expect("git for-each-ref");
    let refs_str = String::from_utf8_lossy(&refs_out.stdout);
    assert!(
        refs_str.contains("refs/mirrors/sim-head"),
        "missing refs/mirrors/sim-head; got: {refs_str}"
    );
    assert!(
        refs_str.contains("refs/mirrors/sim-synced-at"),
        "missing refs/mirrors/sim-synced-at; got: {refs_str}"
    );

    let _ = (wtree, &mirror_bare);
}
