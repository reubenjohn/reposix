//! Integration tests for `reposix attach <backend>::<project>`
//! (DVCS-ATTACH-01..04, v0.13.0 P79-03).
//!
//! Coverage matrix:
//! - DVCS-ATTACH-01: post-attach `extensions.partialClone=reposix` and
//!   `remote.reposix.url` shape (`reposix::` prefix + SoT URL).
//! - DVCS-ATTACH-02: 5 reconciliation cases (match / no-id /
//!   backend-deleted / duplicate-id / mirror-lag).
//! - DVCS-ATTACH-03: re-attach idempotency (Q1.3) + multi-SoT reject (Q1.2).
//! - DVCS-ATTACH-04 reframed part 2: forces ONE materialization via
//!   `Cache::read_blob` after attach and pins the byte stream as
//!   `Tainted<Vec<u8>>` at runtime.
//! - OP-3 unconditional: asserts an `audit_events_cache` row with
//!   `op = 'attach_walk'` lands per attach.
//!
//! Tests shell out to the workspace `reposix` and `reposix-sim`
//! binaries (built via `cargo test`'s `CARGO_BIN_EXE_*` env vars). The
//! sim binds on a free port per test (no shared 7878 collision); the
//! `REPOSIX_SIM_ORIGIN` env var threads the per-test port into the
//! attach subprocess so its `SimBackend` REST round-trip lands on the
//! right sim. Each test gets its own `REPOSIX_CACHE_DIR` tempdir so
//! cache state doesn't bleed across tests.
//!
//! Tests run serially under the `single_threaded_attach_tests` mutex to
//! keep cargo's parallel test runner from racing on shared filesystem
//! artefacts (XDG cache fallback under unset `REPOSIX_CACHE_DIR`).

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::too_many_lines,
    // Test prose freely names CLI literals (`reposix attach`, `Tainted<Vec<u8>>`,
    // `cache_reconciliation`, `Cache::read_blob`, `Q1.2/Q1.3`); backticking
    // every occurrence buys nothing and noise-pollutes the failure messages.
    clippy::doc_markdown,
    // `unwrap_or_else(|e| e.into_inner())` is the canonical PoisonError handler
    // in #[forbid(unsafe_code)] test code; `unwrap_or_else(PoisonError::into_inner)`
    // would shave a closure but reads strictly worse for human reviewers.
    clippy::redundant_closure_for_method_calls,
    // The Tainted<Vec<u8>> sink uses an underscore prefix to signal "type-only
    // discharge"; clippy's used_underscore_items lint flags any consumption of
    // such names. Documentation pattern overrides the lint here.
    clippy::used_underscore_items,
    // The std::env::set_var inside the Tainted-materialization test runs once,
    // serialized by SERIAL; clippy still flags it as held-across-await
    // because the test is async. Acceptable for a single-shot test.
    clippy::await_holding_lock,
    clippy::items_after_statements,
)]

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tempfile::TempDir;

// --- shared serialization ---------------------------------------------------

/// Tests serialize on this mutex. Not strictly required (each test now
/// uses a distinct `REPOSIX_CACHE_DIR`), but keeps subprocess + sim
/// bring-up tidy and predictable on busy CI.
static SERIAL: Mutex<()> = Mutex::new(());

// --- helpers ---------------------------------------------------------------

/// Resolve the workspace root from `CARGO_MANIFEST_DIR`.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

/// Path to a built workspace binary in `target/debug/`. The cargo test
/// harness compiles `bin = "reposix"` (this crate) but not sibling
/// binaries; tests using `reposix-sim` depend on a prior
/// `cargo build --workspace --bins`. The CI integration job runs that
/// build before invoking `cargo test`.
fn target_bin(name: &str) -> PathBuf {
    workspace_root().join("target").join("debug").join(name)
}

/// Pick a free TCP port by binding to 0 and letting the OS assign. The
/// listener drops before we hand the port back, so a sim spawned
/// against the port should bind successfully (modulo the rare TIME_WAIT
/// race; tests retry briefly if that fires).
fn pick_free_port() -> u16 {
    use std::net::TcpListener;
    TcpListener::bind("127.0.0.1:0")
        .expect("bind 127.0.0.1:0")
        .local_addr()
        .expect("local_addr")
        .port()
}

/// Spawn `reposix-sim --bind 127.0.0.1:<port> --ephemeral` and wait
/// until it accepts a TCP connection on the bound port. Returns the
/// child handle; caller owns teardown via `kill_child`.
fn spawn_sim(port: u16) -> Child {
    let bin = target_bin("reposix-sim");
    assert!(
        bin.exists(),
        "reposix-sim missing at {}; run `cargo build --workspace --bins` first",
        bin.display(),
    );
    let mut child = Command::new(&bin)
        .args([
            "--bind",
            &format!("127.0.0.1:{port}"),
            "--ephemeral",
            "--no-seed",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn reposix-sim");

    let t0 = Instant::now();
    while t0.elapsed() < Duration::from_secs(5) {
        if std::net::TcpStream::connect(format!("127.0.0.1:{port}")).is_ok() {
            return child;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let _ = child.kill();
    let _ = child.wait();
    panic!("sim did not bind on port {port} within 5s");
}

/// Spawn a sim AND seed it with one issue id=1 via the simulator's
/// POST /projects/{slug}/issues endpoint. Returns the (child, port)
/// pair. Failures panic with the request body for diagnostics.
fn spawn_sim_with_issue(slug: &str, id_hint_only_for_doc: u64) -> (Child, u16) {
    let _ = id_hint_only_for_doc; // sim assigns IDs server-side; the seed maps to id=1.
    let port = pick_free_port();
    let child = spawn_sim(port);
    seed_issue(port, slug, "fixture issue");
    (child, port)
}

/// Seed one issue into the sim via its public REST API. Returns the
/// assigned id (the sim allocates AUTOINCREMENT ids; first POST →
/// id=1, second → id=2, etc.).
fn seed_issue(port: u16, slug: &str, title: &str) -> u64 {
    let url = format!("http://127.0.0.1:{port}/projects/{slug}/issues");
    let body = format!(r#"{{"title":"{title}","status":"open","body":"seed"}}"#);
    // Use curl rather than reqwest to keep this test crate's deps tiny.
    let out = Command::new("curl")
        .args([
            "-fsS",
            "-X",
            "POST",
            "-H",
            "content-type: application/json",
            "-d",
            &body,
            &url,
        ])
        .output()
        .expect("invoke curl");
    assert!(
        out.status.success(),
        "seed POST failed: stdout={:?} stderr={:?}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    // Response is the created issue JSON; extract `id` via grep+sed.
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    let id_str = text
        .split("\"id\":")
        .nth(1)
        .and_then(|s| s.split(|c: char| !c.is_ascii_digit()).next())
        .unwrap_or("");
    id_str.parse::<u64>().unwrap_or_else(|_| {
        panic!("seed_issue could not extract id from response body: {text}");
    })
}

fn kill_child(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn git_init(path: &Path) {
    let out = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["init", "-q"])
        .output()
        .expect("git init");
    assert!(out.status.success(), "git init failed");
    // Ensure committer identity for any operations that need it.
    let _ = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["config", "user.email", "test@reposix.invalid"])
        .status();
    let _ = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["config", "user.name", "reposix-test"])
        .status();
}

fn git_config(path: &Path, key: &str) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["config", "--get", key])
        .output()
        .expect("git config");
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        None
    }
}

/// Write a markdown file with the given frontmatter id.
fn write_record_md(path: &Path, id: u64, title: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("mkdir parent");
    }
    let body = format!(
        "---\nid: {id}\ntitle: {title}\nstatus: open\n\
         created_at: 2026-04-15T00:00:00Z\nupdated_at: 2026-04-15T00:00:00Z\n\
         version: 1\n---\n{title} body\n"
    );
    std::fs::write(path, body).expect("write md");
}

/// Run `reposix attach <args...>` against the given working tree. Sets
/// `REPOSIX_CACHE_DIR` and `REPOSIX_SIM_ORIGIN` env vars so the
/// subprocess's cache and sim REST target are both isolated to this
/// test invocation. Returns the captured `Output`.
fn run_attach(
    work: &Path,
    cache_dir: &Path,
    sim_port: u16,
    extra_args: &[&str],
) -> std::process::Output {
    let bin = target_bin("reposix");
    assert!(
        bin.exists(),
        "reposix missing at {}; run `cargo build --workspace --bins` first",
        bin.display(),
    );
    let mut cmd = Command::new(&bin);
    cmd.arg("attach")
        .args(extra_args)
        .current_dir(work)
        .env("REPOSIX_CACHE_DIR", cache_dir)
        .env(
            "REPOSIX_SIM_ORIGIN",
            format!("http://127.0.0.1:{sim_port}"),
        )
        // The default allowlist permits 127.0.0.1:* so we don't need to
        // override REPOSIX_ALLOWED_ORIGINS here.
        .stdin(Stdio::null());
    cmd.output().expect("spawn reposix attach")
}

/// Open a `rusqlite::Connection` to the cache DB for `(backend, project)`
/// rooted at `cache_dir`. Mirrors `reposix_cache::path::resolve_cache_path`'s
/// layout — `<root>/reposix/<backend>-<project>.git/cache.db`.
fn open_cache_connection(cache_dir: &Path, backend: &str, project: &str) -> rusqlite::Connection {
    let db_path = cache_dir
        .join("reposix")
        .join(format!("{backend}-{project}.git"))
        .join("cache.db");
    rusqlite::Connection::open(&db_path)
        .unwrap_or_else(|e| panic!("open cache db at {}: {e}", db_path.display()))
}

// --- Tests: T01 — DVCS-ATTACH-01 + DVCS-ATTACH-02 (6 tests) -----------------

/// DVCS-ATTACH-01 — post-conditions: `extensions.partialClone == reposix`
/// and `remote.reposix.url` starts with `reposix::` and contains the
/// translated SoT URL (with `?mirror=` since `--no-bus` is not set and
/// the working tree has an origin remote pointing at a vanilla mirror).
#[test]
#[ignore = "spawns reposix-sim child; requires `cargo build --workspace --bins` first"]
fn attach_against_vanilla_clone_sets_partial_clone() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let port = pick_free_port();
    let mut sim = spawn_sim(port);

    let work_tmp = TempDir::new().expect("work tempdir");
    let cache_tmp = TempDir::new().expect("cache tempdir");
    git_init(work_tmp.path());
    let _ = Command::new("git")
        .arg("-C")
        .arg(work_tmp.path())
        .args([
            "remote",
            "add",
            "origin",
            "https://example.invalid/mirror.git",
        ])
        .status();

    let out = run_attach(
        work_tmp.path(),
        cache_tmp.path(),
        port,
        &["sim::demo", "--remote-name", "reposix"],
    );
    assert!(
        out.status.success(),
        "attach failed: stdout={:?} stderr={:?}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );

    let pclone = git_config(work_tmp.path(), "extensions.partialClone");
    assert_eq!(
        pclone.as_deref(),
        Some("reposix"),
        "extensions.partialClone must be 'reposix' (NOT origin)"
    );
    let url = git_config(work_tmp.path(), "remote.reposix.url")
        .expect("remote.reposix.url should be set");
    assert!(
        url.starts_with("reposix::"),
        "remote.reposix.url must start with reposix::, got {url}"
    );
    // Confirm origin remote (vanilla mirror) is unchanged.
    let origin_url = git_config(work_tmp.path(), "remote.origin.url");
    assert_eq!(
        origin_url.as_deref(),
        Some("https://example.invalid/mirror.git"),
        "origin remote URL must be unchanged"
    );

    kill_child(&mut sim);
}

/// Wave-5.5 credential-leak fix (MEDIUM intake, 91-05 litmus finding):
/// a token-in-URL origin (`https://user:TOKEN@host/...`) must NOT have its
/// credentials folded into `remote.reposix.url` — the bus URL lands in
/// plaintext `.git/config` and is echoed by git on every push. The token
/// stays only in origin's own config; attach warns and strips.
#[test]
#[ignore = "spawns reposix-sim child; requires `cargo build --workspace --bins` first"]
fn attach_strips_credentials_from_mirror_url() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let port = pick_free_port();
    let mut sim = spawn_sim(port);

    let work_tmp = TempDir::new().expect("work tempdir");
    let cache_tmp = TempDir::new().expect("cache tempdir");
    git_init(work_tmp.path());
    let cred_url = "https://x-access-token:ghp_FAKESECRET@example.invalid/mirror.git";
    let _ = Command::new("git")
        .arg("-C")
        .arg(work_tmp.path())
        .args(["remote", "add", "origin", cred_url])
        .status();

    let out = run_attach(
        work_tmp.path(),
        cache_tmp.path(),
        port,
        &["sim::demo", "--remote-name", "reposix"],
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "attach failed: stderr={stderr:?}");

    let url = git_config(work_tmp.path(), "remote.reposix.url")
        .expect("remote.reposix.url should be set");
    assert!(
        !url.contains("ghp_FAKESECRET") && !url.contains("x-access-token"),
        "remote.reposix.url must NOT embed origin's credentials; got {url}"
    );
    assert!(
        url.contains("?mirror=https://example.invalid/mirror.git"),
        "cred-stripped mirror URL must still be folded; got {url}"
    );
    assert!(
        stderr.contains("embeds credentials"),
        "attach must warn about the stripped credentials; stderr={stderr}"
    );
    // Origin keeps its own (credential-bearing) URL — auth is unaffected.
    assert_eq!(
        git_config(work_tmp.path(), "remote.origin.url").as_deref(),
        Some(cred_url),
        "origin remote URL must be unchanged"
    );

    kill_child(&mut sim);
}

/// DVCS-ATTACH-02 case 1 — match: local file with `id` matching a
/// backend record produces a `cache_reconciliation` row.
#[test]
#[ignore = "spawns reposix-sim child; requires `cargo build --workspace --bins` first"]
fn attach_match_records_by_id() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (mut sim, port) = spawn_sim_with_issue("demo", 1);

    let work_tmp = TempDir::new().expect("work tempdir");
    let cache_tmp = TempDir::new().expect("cache tempdir");
    git_init(work_tmp.path());

    write_record_md(&work_tmp.path().join("issues/0001.md"), 1, "match me");

    let out = run_attach(
        work_tmp.path(),
        cache_tmp.path(),
        port,
        &["sim::demo", "--remote-name", "reposix"],
    );
    assert!(
        out.status.success(),
        "attach failed: stderr={:?}",
        String::from_utf8_lossy(&out.stderr)
    );

    let conn = open_cache_connection(cache_tmp.path(), "sim", "demo");
    let count: i64 = conn
        .query_row(
            "SELECT count(*) FROM cache_reconciliation WHERE record_id = 1",
            [],
            |r| r.get(0),
        )
        .expect("query reconciliation");
    assert_eq!(count, 1, "expected one reconciliation row for record_id=1");

    kill_child(&mut sim);
}

/// DVCS-ATTACH-02 case 2 — backend-deleted: local id=99 with no
/// matching backend record. Default `--orphan-policy=abort` warns to
/// stderr and continues; no row is inserted for id=99.
#[test]
#[ignore = "spawns reposix-sim child; requires `cargo build --workspace --bins` first"]
fn attach_warns_on_backend_deleted() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (mut sim, port) = spawn_sim_with_issue("demo", 1);

    let work_tmp = TempDir::new().expect("work tempdir");
    let cache_tmp = TempDir::new().expect("cache tempdir");
    git_init(work_tmp.path());

    write_record_md(&work_tmp.path().join("issues/0099.md"), 99, "ghost record");

    let out = run_attach(
        work_tmp.path(),
        cache_tmp.path(),
        port,
        &["sim::demo", "--remote-name", "reposix"],
    );
    assert!(
        out.status.success(),
        "attach should succeed under --orphan-policy=abort (default just warns): stderr={:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("BACKEND_DELETED"),
        "expected BACKEND_DELETED token in stderr, got: {stderr}"
    );

    let conn = open_cache_connection(cache_tmp.path(), "sim", "demo");
    let count: i64 = conn
        .query_row(
            "SELECT count(*) FROM cache_reconciliation WHERE record_id = 99",
            [],
            |r| r.get(0),
        )
        .expect("query reconciliation");
    assert_eq!(count, 0, "no row for backend-deleted record_id=99");

    kill_child(&mut sim);
}

/// DVCS-ATTACH-02 case 2 + D91-04 — fork-as-new: an orphan local record
/// (id absent from the backend) under `--orphan-policy=fork-as-new` is
/// KEPT on disk (not deleted, attach not aborted) so the next `git push`
/// creates it as a new backend record. Proves ForkAsNew is neither
/// DeleteLocal nor a hard abort, and that it needs no extra reconciliation
/// state — leaving the file in place IS the mechanism (diff.rs classifies
/// a path with no pushed prior as a Create).
// test-name-honesty: ok — "next push" describes the mechanism under test
// (the orphan file is left in place so a FUTURE git push classifies it as
// a Create), not a claim this test itself issues a push; it drives a real
// attach against a real reposix-sim child + real cache DB.
#[test]
#[ignore = "spawns reposix-sim child; requires `cargo build --workspace --bins` first"]
fn attach_fork_as_new_keeps_orphan_for_next_push() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (mut sim, port) = spawn_sim_with_issue("demo", 1);

    let work_tmp = TempDir::new().expect("work tempdir");
    let cache_tmp = TempDir::new().expect("cache tempdir");
    git_init(work_tmp.path());

    let orphan = work_tmp.path().join("issues/0099.md");
    write_record_md(&orphan, 99, "fork me as new");

    let out = run_attach(
        work_tmp.path(),
        cache_tmp.path(),
        port,
        &[
            "sim::demo",
            "--remote-name",
            "reposix",
            "--orphan-policy",
            "fork-as-new",
        ],
    );
    assert!(
        out.status.success(),
        "attach --orphan-policy=fork-as-new should succeed: stderr={:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("FORK_AS_NEW"),
        "expected FORK_AS_NEW token in stderr, got: {stderr}"
    );
    assert!(
        !stderr.contains("TODO"),
        "fork-as-new must not advertise a TODO stub, got: {stderr}"
    );
    // The orphan file is KEPT (not deleted) — this is what lets the next
    // `git push` classify it as a Create.
    assert!(
        orphan.exists(),
        "fork-as-new must leave the orphan file in place for the next push"
    );
    // It is not a backend match, so no cache_reconciliation row.
    let conn = open_cache_connection(cache_tmp.path(), "sim", "demo");
    let count: i64 = conn
        .query_row(
            "SELECT count(*) FROM cache_reconciliation WHERE record_id = 99",
            [],
            |r| r.get(0),
        )
        .expect("query reconciliation");
    assert_eq!(count, 0, "no reconciliation row for orphan record_id=99");

    kill_child(&mut sim);
}

/// DVCS-ATTACH-02 case 3 — no-id: local file lacking parseable `id`
/// frontmatter. Walker warns + skips; the file is not in
/// `cache_reconciliation`.
#[test]
#[ignore = "spawns reposix-sim child; requires `cargo build --workspace --bins` first"]
fn attach_skips_no_id_files() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (mut sim, port) = spawn_sim_with_issue("demo", 1);

    let work_tmp = TempDir::new().expect("work tempdir");
    let cache_tmp = TempDir::new().expect("cache tempdir");
    git_init(work_tmp.path());

    let no_id_path = work_tmp.path().join("notes/freeform.md");
    std::fs::create_dir_all(no_id_path.parent().unwrap()).unwrap();
    std::fs::write(
        &no_id_path,
        "# Just a freeform note\n\nNo frontmatter at all.\n",
    )
    .unwrap();

    let out = run_attach(
        work_tmp.path(),
        cache_tmp.path(),
        port,
        &["sim::demo", "--remote-name", "reposix"],
    );
    assert!(
        out.status.success(),
        "attach should succeed (no-id file is warn+skip): stderr={:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("NO_ID"),
        "expected NO_ID token in stderr, got: {stderr}"
    );

    // The freeform.md path is NOT in the reconciliation table.
    let conn = open_cache_connection(cache_tmp.path(), "sim", "demo");
    let rows: Vec<String> = conn
        .prepare("SELECT local_path FROM cache_reconciliation")
        .unwrap()
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .map(std::result::Result::unwrap)
        .collect();
    let has_freeform = rows.iter().any(|p| p.contains("freeform.md"));
    assert!(
        !has_freeform,
        "freeform.md must not appear in cache_reconciliation; rows: {rows:?}"
    );

    kill_child(&mut sim);
}

/// DVCS-ATTACH-02 case 4 — duplicate id: two local files claim id=42.
/// Reconciliation aborts (exit non-zero); zero rows committed for
/// record_id=42 (atomicity).
#[test]
#[ignore = "spawns reposix-sim child; requires `cargo build --workspace --bins` first"]
fn attach_errors_on_duplicate_id() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (mut sim, port) = spawn_sim_with_issue("demo", 1);

    let work_tmp = TempDir::new().expect("work tempdir");
    let cache_tmp = TempDir::new().expect("cache tempdir");
    git_init(work_tmp.path());

    write_record_md(
        &work_tmp.path().join("issues/dup-a.md"),
        42,
        "dup variant a",
    );
    write_record_md(
        &work_tmp.path().join("issues/dup-b.md"),
        42,
        "dup variant b",
    );

    let out = run_attach(
        work_tmp.path(),
        cache_tmp.path(),
        port,
        &["sim::demo", "--remote-name", "reposix"],
    );
    assert!(
        !out.status.success(),
        "attach must fail on duplicate id; stdout={:?} stderr={:?}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("duplicate id"),
        "stderr should name `duplicate id`, got: {stderr}"
    );
    assert!(
        stderr.contains("dup-a.md") && stderr.contains("dup-b.md"),
        "stderr should name both duplicate paths, got: {stderr}"
    );

    let conn = open_cache_connection(cache_tmp.path(), "sim", "demo");
    let count: i64 = conn
        .query_row(
            "SELECT count(*) FROM cache_reconciliation WHERE record_id = 42",
            [],
            |r| r.get(0),
        )
        .expect("query reconciliation");
    assert_eq!(
        count, 0,
        "duplicate-id case must commit zero rows (atomicity)"
    );

    kill_child(&mut sim);
}

/// DVCS-ATTACH-02 case 5 — mirror-lag: backend has id=1 but the
/// working tree has zero matching files. Attach succeeds; the cache's
/// view of records includes id=1 (visible to the next fetch).
#[test]
#[ignore = "spawns reposix-sim child; requires `cargo build --workspace --bins` first"]
// test-name-honesty: ok — real attach subprocess + cache-state assert; sets up mirror-lag state for a subsequent fetch it does not itself drive (thin but honestly named per R2 adjacent-exempt list)
fn attach_marks_mirror_lag_for_next_fetch() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (mut sim, port) = spawn_sim_with_issue("demo", 1);

    let work_tmp = TempDir::new().expect("work tempdir");
    let cache_tmp = TempDir::new().expect("cache tempdir");
    git_init(work_tmp.path());
    // Deliberately NO local file matching id=1 — this is the
    // backend-has-but-local-lacks (mirror-lag) case.

    let out = run_attach(
        work_tmp.path(),
        cache_tmp.path(),
        port,
        &["sim::demo", "--remote-name", "reposix"],
    );
    assert!(
        out.status.success(),
        "attach must succeed in mirror-lag case: stderr={:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("mirror_lag=1"),
        "summary should report mirror_lag=1, got: {stderr}"
    );

    // The backend record is in oid_map (populated by build_from). We
    // verify via the same join the public Cache::list_record_ids API
    // would use — backend-known ids include 1.
    let conn = open_cache_connection(cache_tmp.path(), "sim", "demo");
    let known_ids: Vec<String> = conn
        .prepare(
            "SELECT DISTINCT issue_id FROM oid_map \
             WHERE backend = ?1 AND project = ?2 ORDER BY issue_id",
        )
        .unwrap()
        .query_map(rusqlite::params!["sim", "demo"], |r| r.get::<_, String>(0))
        .unwrap()
        .map(std::result::Result::unwrap)
        .collect();
    assert!(
        known_ids.contains(&"1".to_string()),
        "cache should know id=1 from backend, got: {known_ids:?}"
    );

    kill_child(&mut sim);
}

// --- Tests: T02 — DVCS-ATTACH-03 + 04 part 2 + OP-3 (4 tests) --------------

/// DVCS-ATTACH-03 / Q1.3 — re-attach against the same SoT is
/// idempotent: cache_reconciliation rows match across both attaches;
/// remote URL unchanged.
#[test]
#[ignore = "spawns reposix-sim child; requires `cargo build --workspace --bins` first"]
fn re_attach_same_sot_is_idempotent() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (mut sim, port) = spawn_sim_with_issue("demo", 1);

    let work_tmp = TempDir::new().expect("work tempdir");
    let cache_tmp = TempDir::new().expect("cache tempdir");
    git_init(work_tmp.path());
    write_record_md(&work_tmp.path().join("issues/0001.md"), 1, "matched record");

    // First attach.
    let out1 = run_attach(
        work_tmp.path(),
        cache_tmp.path(),
        port,
        &["sim::demo", "--remote-name", "reposix"],
    );
    assert!(
        out1.status.success(),
        "first attach failed: stderr={:?}",
        String::from_utf8_lossy(&out1.stderr)
    );
    let url1 =
        git_config(work_tmp.path(), "remote.reposix.url").expect("remote.reposix.url after 1st");

    let snapshot: Vec<(i64, String)> = open_cache_connection(cache_tmp.path(), "sim", "demo")
        .prepare(
            "SELECT record_id, local_path FROM cache_reconciliation \
             ORDER BY record_id",
        )
        .unwrap()
        .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))
        .unwrap()
        .map(std::result::Result::unwrap)
        .collect();

    // Second attach against the SAME SoT.
    let out2 = run_attach(
        work_tmp.path(),
        cache_tmp.path(),
        port,
        &["sim::demo", "--remote-name", "reposix"],
    );
    assert!(
        out2.status.success(),
        "second attach (same SoT) failed: stderr={:?}",
        String::from_utf8_lossy(&out2.stderr)
    );
    let url2 =
        git_config(work_tmp.path(), "remote.reposix.url").expect("remote.reposix.url after 2nd");
    assert_eq!(url1, url2, "remote URL must be unchanged across re-attach");

    let after: Vec<(i64, String)> = open_cache_connection(cache_tmp.path(), "sim", "demo")
        .prepare(
            "SELECT record_id, local_path FROM cache_reconciliation \
             ORDER BY record_id",
        )
        .unwrap()
        .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))
        .unwrap()
        .map(std::result::Result::unwrap)
        .collect();
    assert_eq!(
        snapshot, after,
        "reconciliation rows must match across re-attach (INSERT OR REPLACE leaves no stale rows)"
    );

    kill_child(&mut sim);
}

/// DVCS-ATTACH-03 / Q1.2 — attach with a different SoT is rejected:
/// non-zero exit, message contains `already attached` AND `multi-SoT
/// not supported in v0.13.0`. Remote URL unchanged.
#[test]
#[ignore = "spawns reposix-sim child; requires `cargo build --workspace --bins` first"]
fn re_attach_different_sot_is_rejected() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let port = pick_free_port();
    let mut sim = spawn_sim(port);
    // Seed the first project (project-a).
    let _ = seed_issue(port, "project-a", "first");

    let work_tmp = TempDir::new().expect("work tempdir");
    let cache_tmp_a = TempDir::new().expect("cache-a tempdir");
    let cache_tmp_b = TempDir::new().expect("cache-b tempdir");
    git_init(work_tmp.path());

    // First attach: project-a.
    let out1 = run_attach(
        work_tmp.path(),
        cache_tmp_a.path(),
        port,
        &["sim::project-a", "--remote-name", "reposix"],
    );
    assert!(
        out1.status.success(),
        "first attach failed: stderr={:?}",
        String::from_utf8_lossy(&out1.stderr)
    );
    let url1 = git_config(work_tmp.path(), "remote.reposix.url").expect("url after 1st");
    let pclone1 = git_config(work_tmp.path(), "extensions.partialClone");

    // Second attach: project-b (different SoT). Must reject.
    let out2 = run_attach(
        work_tmp.path(),
        cache_tmp_b.path(),
        port,
        &["sim::project-b", "--remote-name", "reposix"],
    );
    assert!(
        !out2.status.success(),
        "second attach (different SoT) must fail; stderr={:?}",
        String::from_utf8_lossy(&out2.stderr)
    );
    let stderr = String::from_utf8_lossy(&out2.stderr);
    assert!(
        stderr.contains("already attached"),
        "stderr should contain `already attached`, got: {stderr}"
    );
    assert!(
        stderr.contains("multi-SoT not supported in v0.13.0"),
        "stderr should contain `multi-SoT not supported in v0.13.0`, got: {stderr}"
    );

    let url_after = git_config(work_tmp.path(), "remote.reposix.url").expect("url after reject");
    assert_eq!(
        url1, url_after,
        "remote URL must be unchanged after a rejected re-attach"
    );
    let pclone_after = git_config(work_tmp.path(), "extensions.partialClone");
    assert_eq!(
        pclone1, pclone_after,
        "extensions.partialClone must be unchanged after rejected re-attach"
    );

    kill_child(&mut sim);
}

/// DVCS-ATTACH-04 reframed part 2 — runtime evidence: after attach,
/// open the cache directly, force one blob materialization via
/// `Cache::read_blob`, and feed the result into a function that ONLY
/// accepts `Tainted<Vec<u8>>`. Test passes iff the call compiles
/// (type-system pin) AND runtime materialization succeeds (one real
/// lazy load). Closes the vacuity gap noted in checker B2.
#[tokio::test]
#[ignore = "spawns reposix-sim child; requires `cargo build --workspace --bins` first"]
async fn attach_then_read_blob_returns_tainted() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (mut sim, port) = spawn_sim_with_issue("demo", 1);

    let work_tmp = TempDir::new().expect("work tempdir");
    let cache_tmp = TempDir::new().expect("cache tempdir");
    git_init(work_tmp.path());
    write_record_md(&work_tmp.path().join("issues/0001.md"), 1, "matched record");

    let out = run_attach(
        work_tmp.path(),
        cache_tmp.path(),
        port,
        &["sim::demo", "--remote-name", "reposix"],
    );
    assert!(
        out.status.success(),
        "attach failed: stderr={:?}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Open the cache directly (in-process) and force one
    // materialization. The cache's path layout is deterministic given
    // REPOSIX_CACHE_DIR — same env var the subprocess attach used.
    std::env::set_var("REPOSIX_CACHE_DIR", cache_tmp.path());
    let connector = std::sync::Arc::new(
        reposix_core::backend::sim::SimBackend::new(format!("http://127.0.0.1:{port}"))
            .expect("build SimBackend"),
    );
    let cache = reposix_cache::Cache::open(connector, "sim", "demo").expect("open cache");

    let oid = cache
        .find_oid_for_record(reposix_core::RecordId(1))
        .expect("find_oid_for_record")
        .expect("backend record id=1 must have an oid after build_from");

    // Force ONE materialization via the lazy-blob seam. The return
    // type is the type-system contract DVCS-ATTACH-04 reframed part 2
    // grades: it MUST be `Tainted<Vec<u8>>`. Feeding the result into
    // a function that only accepts that type pins it.
    fn _is_tainted(_: reposix_core::Tainted<Vec<u8>>) {}
    let bytes: reposix_core::Tainted<Vec<u8>> = cache.read_blob(oid).await.expect("read_blob");
    _is_tainted(bytes);
    // The runtime side of DVCS-ATTACH-04: we got bytes back (one real
    // lazy-load round-trip through SimBackend) AND the type checker
    // accepted them as `Tainted<Vec<u8>>` at the call site.

    kill_child(&mut sim);
}

/// OP-3 unconditional — after attach, the cache's `audit_events_cache`
/// table must contain exactly one row with `op = 'attach_walk'`. No
/// conditional escape: a missing row is a real OP-3 violation.
#[test]
#[ignore = "spawns reposix-sim child; requires `cargo build --workspace --bins` first"]
fn attach_audit_log_records_walk_event() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (mut sim, port) = spawn_sim_with_issue("demo", 1);

    let work_tmp = TempDir::new().expect("work tempdir");
    let cache_tmp = TempDir::new().expect("cache tempdir");
    git_init(work_tmp.path());

    let out = run_attach(
        work_tmp.path(),
        cache_tmp.path(),
        port,
        &["sim::demo", "--remote-name", "reposix"],
    );
    assert!(
        out.status.success(),
        "attach failed: stderr={:?}",
        String::from_utf8_lossy(&out.stderr)
    );

    let conn = open_cache_connection(cache_tmp.path(), "sim", "demo");
    let count: i64 = conn
        .query_row(
            "SELECT count(*) FROM audit_events_cache WHERE op = 'attach_walk'",
            [],
            |r| r.get(0),
        )
        .expect("query audit_events_cache");
    assert_eq!(
        count, 1,
        "exactly one attach_walk audit row required (OP-3 unconditional)"
    );

    kill_child(&mut sim);
}

// --- Tests: T2-REOPEN H1 — remote.pushDefault routes `git push` to SoT ------

/// H1 (T2-REOPEN) — after attach, `remote.pushDefault` names the reposix
/// remote so a bare `git push` (the closing step in Pattern C) routes
/// through the SoT bus, NOT the vanilla mirror on `origin`. The origin
/// fetch config MUST be left untouched — reads keep coming from the mirror.
// test-name-honesty: ok — genuinely asserts `remote.pushDefault` git config
// after a real attach run (real reposix-sim child), which is exactly what
// routes a bare `git push` to the reposix remote; no network push is
// issued, but the claim is about push ROUTING config, not push execution.
#[test]
#[ignore = "spawns reposix-sim child; requires `cargo build --workspace --bins` first"]
fn attach_sets_push_default_to_reposix_remote() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let port = pick_free_port();
    let mut sim = spawn_sim(port);

    let work_tmp = TempDir::new().expect("work tempdir");
    let cache_tmp = TempDir::new().expect("cache tempdir");
    git_init(work_tmp.path());
    let _ = Command::new("git")
        .arg("-C")
        .arg(work_tmp.path())
        .args([
            "remote",
            "add",
            "origin",
            "https://example.invalid/mirror.git",
        ])
        .status();
    // Snapshot origin's fetch refspec BEFORE attach — it must survive
    // untouched (attach configures push routing only, never fetch).
    let origin_fetch_before = git_config(work_tmp.path(), "remote.origin.fetch");
    assert!(
        origin_fetch_before.is_some(),
        "sanity: `git remote add origin` should set remote.origin.fetch"
    );

    let out = run_attach(
        work_tmp.path(),
        cache_tmp.path(),
        port,
        &["sim::demo", "--remote-name", "reposix"],
    );
    assert!(
        out.status.success(),
        "attach failed: stderr={:?}",
        String::from_utf8_lossy(&out.stderr)
    );

    assert_eq!(
        git_config(work_tmp.path(), "remote.pushDefault").as_deref(),
        Some("reposix"),
        "attach must set remote.pushDefault to the reposix remote so `git push` hits the SoT bus"
    );
    // Origin FETCH config is untouched.
    assert_eq!(
        git_config(work_tmp.path(), "remote.origin.fetch"),
        origin_fetch_before,
        "attach must NOT modify origin's fetch config (reads still come from the mirror)"
    );
    assert_eq!(
        git_config(work_tmp.path(), "remote.origin.url").as_deref(),
        Some("https://example.invalid/mirror.git"),
        "origin remote URL must be unchanged"
    );

    kill_child(&mut sim);
}

/// H1 (T2-REOPEN) — a user-set `remote.pushDefault` is the user's explicit
/// choice: attach must NOT clobber it. It warns to stderr and leaves the
/// value intact.
// test-name-honesty: ok — genuinely asserts attach does NOT clobber a
// pre-set `remote.pushDefault` git config value, against a real reposix-sim
// child; the claim is about push-routing config preservation, not push
// execution.
#[test]
#[ignore = "spawns reposix-sim child; requires `cargo build --workspace --bins` first"]
fn attach_preserves_user_set_push_default() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let port = pick_free_port();
    let mut sim = spawn_sim(port);

    let work_tmp = TempDir::new().expect("work tempdir");
    let cache_tmp = TempDir::new().expect("cache tempdir");
    git_init(work_tmp.path());
    let _ = Command::new("git")
        .arg("-C")
        .arg(work_tmp.path())
        .args([
            "remote",
            "add",
            "origin",
            "https://example.invalid/mirror.git",
        ])
        .status();
    // User has already pinned pushDefault to origin.
    let _ = Command::new("git")
        .arg("-C")
        .arg(work_tmp.path())
        .args(["config", "remote.pushDefault", "origin"])
        .status();

    let out = run_attach(
        work_tmp.path(),
        cache_tmp.path(),
        port,
        &["sim::demo", "--remote-name", "reposix"],
    );
    assert!(
        out.status.success(),
        "attach must still succeed when pushDefault is pre-set: stderr={:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        git_config(work_tmp.path(), "remote.pushDefault").as_deref(),
        Some("origin"),
        "attach must NOT clobber a user-set remote.pushDefault"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("remote.pushDefault is already set to `origin`"),
        "attach must warn that it left the user's pushDefault unchanged; stderr={stderr}"
    );

    kill_child(&mut sim);
}

// --- Tests: T2-REOPEN H2 — helper-on-PATH teaching warning ------------------

/// Resolve a binary's absolute path via the ambient PATH. Used to build a
/// curated PATH for the helper-discovery tests below.
#[cfg(unix)]
fn resolve_bin(name: &str) -> PathBuf {
    let out = Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {name}"))
        .output()
        .expect("command -v");
    assert!(out.status.success(), "{name} not found on ambient PATH");
    PathBuf::from(String::from_utf8_lossy(&out.stdout).trim())
}

/// Build a tempdir containing symlinks to `git` and `sh` (the only PATH
/// binaries `reposix attach` shells out to), optionally plus a fake
/// `git-remote-reposix`. Returns the dir; the caller sets it as the
/// subprocess PATH to control helper discoverability deterministically,
/// regardless of what is installed on the host.
#[cfg(unix)]
fn curated_bindir(include_helper: bool) -> TempDir {
    use std::os::unix::fs::symlink;
    let dir = TempDir::new().expect("bindir tempdir");
    let git = resolve_bin("git");
    let sh = resolve_bin("sh");
    symlink(&git, dir.path().join("git")).expect("symlink git");
    symlink(&sh, dir.path().join("sh")).expect("symlink sh");
    if include_helper {
        // Any executable file satisfies discovery via `command -v` — the
        // warning keys off presence on PATH, not on the helper running.
        symlink(&git, dir.path().join("git-remote-reposix")).expect("symlink fake helper");
    }
    dir
}

#[cfg(unix)]
fn run_attach_with_path(
    work: &Path,
    cache_dir: &Path,
    sim_port: u16,
    extra_args: &[&str],
    path_value: &Path,
) -> std::process::Output {
    let bin = target_bin("reposix");
    let mut cmd = Command::new(&bin);
    cmd.arg("attach")
        .args(extra_args)
        .current_dir(work)
        .env("REPOSIX_CACHE_DIR", cache_dir)
        .env("REPOSIX_SIM_ORIGIN", format!("http://127.0.0.1:{sim_port}"))
        .env("PATH", path_value)
        .stdin(Stdio::null());
    cmd.output().expect("spawn reposix attach")
}

/// H2 (T2-REOPEN) — when `git-remote-reposix` is NOT on PATH, attach still
/// succeeds (config is valid) but prints a prominent teaching warning that
/// names the failure git will emit AND the exact install command.
#[cfg(unix)]
#[test]
#[ignore = "spawns reposix-sim child; requires `cargo build --workspace --bins` first"]
fn attach_warns_when_helper_absent_from_path() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let port = pick_free_port();
    let mut sim = spawn_sim(port);

    let work_tmp = TempDir::new().expect("work tempdir");
    let cache_tmp = TempDir::new().expect("cache tempdir");
    git_init(work_tmp.path());
    let bindir = curated_bindir(false); // helper deliberately absent

    let out = run_attach_with_path(
        work_tmp.path(),
        cache_tmp.path(),
        port,
        &["sim::demo", "--remote-name", "reposix"],
        bindir.path(),
    );
    assert!(
        out.status.success(),
        "attach must succeed even without the helper (non-fatal warning): stderr={:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("`git-remote-reposix` is not on PATH"),
        "expected helper-absent warning; stderr={stderr}"
    );
    assert!(
        stderr.contains("cargo binstall reposix-remote")
            && stderr.contains("cargo install reposix-remote"),
        "warning must name both install commands; stderr={stderr}"
    );
    assert!(
        stderr.contains("unable to find remote helper for 'reposix'"),
        "warning must name the exact git failure so the fix is unambiguous; stderr={stderr}"
    );

    kill_child(&mut sim);
}

/// H2 (T2-REOPEN) — when `git-remote-reposix` IS on PATH, attach emits no
/// helper-absent warning (guards against a spurious nag on healthy setups).
#[cfg(unix)]
#[test]
#[ignore = "spawns reposix-sim child; requires `cargo build --workspace --bins` first"]
fn attach_no_helper_warning_when_present_on_path() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let port = pick_free_port();
    let mut sim = spawn_sim(port);

    let work_tmp = TempDir::new().expect("work tempdir");
    let cache_tmp = TempDir::new().expect("cache tempdir");
    git_init(work_tmp.path());
    let bindir = curated_bindir(true); // helper present

    let out = run_attach_with_path(
        work_tmp.path(),
        cache_tmp.path(),
        port,
        &["sim::demo", "--remote-name", "reposix"],
        bindir.path(),
    );
    assert!(
        out.status.success(),
        "attach failed: stderr={:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("is not on PATH"),
        "attach must NOT warn about the helper when it is on PATH; stderr={stderr}"
    );

    kill_child(&mut sim);
}

// --- Tests: item 4a — attach seeds refs/reposix/origin/main (DP-2 repro) -----

/// Prepend `target/debug/` to PATH so a spawned `git fetch`/`git rebase`
/// discovers the real `git-remote-reposix` helper (git resolves
/// `git-remote-<scheme>` from PATH). Mirrors agent_flow.rs::path_with_target_debug.
fn path_with_target_debug() -> String {
    let dir = workspace_root().join("target").join("debug");
    let existing = std::env::var("PATH").unwrap_or_default();
    format!("{}:{}", dir.display(), existing)
}

/// Run `git -C <work> <args>` with a deterministic committer identity. Returns
/// the raw `Output`. No helper on PATH (plain git plumbing only).
fn git_run(work: &Path, args: &[&str]) -> std::process::Output {
    Command::new("git")
        .arg("-C")
        .arg(work)
        .args(args)
        .env("GIT_AUTHOR_NAME", "reposix-test")
        .env("GIT_AUTHOR_EMAIL", "test@reposix.invalid")
        .env("GIT_COMMITTER_NAME", "reposix-test")
        .env("GIT_COMMITTER_EMAIL", "test@reposix.invalid")
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|e| panic!("spawn git {}: {e}", args.join(" ")))
}

/// Run a git op that shells out to the reposix helper (fetch/rebase) with the
/// helper on PATH and an isolated cache dir.
fn git_run_with_helper(
    work: &Path,
    cache: &Path,
    path_env: &str,
    args: &[&str],
) -> std::process::Output {
    Command::new("git")
        .arg("-C")
        .arg(work)
        .args(args)
        .env("PATH", path_env)
        .env("REPOSIX_CACHE_DIR", cache)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_AUTHOR_NAME", "reposix-test")
        .env("GIT_AUTHOR_EMAIL", "test@reposix.invalid")
        .env("GIT_COMMITTER_NAME", "reposix-test")
        .env("GIT_COMMITTER_EMAIL", "test@reposix.invalid")
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|e| panic!("spawn git {}: {e}", args.join(" ")))
}

/// `git rev-parse --verify --quiet <arg>` → `Some(oid)` when it resolves,
/// `None` when the ref/rev is absent.
fn git_rev_parse(work: &Path, arg: &str) -> Option<String> {
    let out = git_run(work, &["rev-parse", "--verify", "--quiet", arg]);
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    (!s.is_empty()).then_some(s)
}

/// item 4a headline (§5.1) + binding end-to-end repro. Drives the REAL
/// `reposix attach` binary against a live sim in a Pattern-C topology
/// (vanilla-clone mirror tracking ref `M` + a committed agent edit `M'` BEFORE
/// attach), then the documented round-trip (`git fetch` → `git rebase`).
///
/// RED on today's unfixed code: `reposix attach` never runs
/// `git update-ref refs/reposix/origin/main …`, so the ref the whole Pattern-C
/// recovery anchors on is ABSENT after attach. The follow-up fetch synthesizes
/// a PARENTLESS import root (`resolve_import_parent` finds no ref → `None`),
/// and the agent's rebase onto it hits the cross-root `add/add` wall.
///
/// GREEN once Part A lands: attach seeds `refs/reposix/origin/main` = the
/// mirror merge-base `M` (NOT HEAD `M'` — the §3.1 silent-revert guard), so the
/// snapshot chains onto `M` and the rebase reconciles.
///
/// This is the ONE test that exercises the REAL `reposix attach` path end-to-end
/// (not a manually-seeded ref); the remote-crate `attach_pattern_c_roundtrip_*`
/// tests prove the underlying git mechanism the fix relies on.
#[test]
#[ignore = "spawns reposix-sim child + shells out to git+helper; requires `cargo build --workspace --bins` first"]
// test-name-honesty: ok — runs REAL `reposix attach` then a REAL git fetch/rebase
// and asserts the seeded tracking ref value; no manual seed.
fn attach_seeds_tracking_ref_at_mirror_base() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (mut sim, port) = spawn_sim_with_issue("demo", 1);

    let work_tmp = TempDir::new().expect("work tempdir");
    let cache_tmp = TempDir::new().expect("cache tempdir");
    let work = work_tmp.path();
    let cache = cache_tmp.path();
    git_init(work);

    // Vanilla-clone topology: an `origin` mirror remote, base commit M with
    // issues/1.md (unpadded path, matching the helper's emit), and the mirror
    // tracking ref refs/remotes/origin/main = M (the merge-base).
    assert!(
        git_run(
            work,
            &[
                "remote",
                "add",
                "origin",
                "https://example.invalid/mirror.git",
            ],
        )
        .status
        .success(),
        "git remote add origin"
    );
    write_record_md(&work.join("issues/1.md"), 1, "base record");
    assert!(
        git_run(work, &["add", "-A"]).status.success(),
        "git add base"
    );
    assert!(
        git_run(work, &["commit", "-q", "-m", "base M"])
            .status
            .success(),
        "git commit base"
    );
    let m = git_rev_parse(work, "HEAD").expect("base commit M");
    assert!(
        git_run(work, &["update-ref", "refs/remotes/origin/main", &m])
            .status
            .success(),
        "seed mirror tracking ref refs/remotes/origin/main = M"
    );

    // Pattern C: commit an agent edit BEFORE attach → HEAD = M' (!= M).
    write_record_md(&work.join("issues/1.md"), 1, "AGENT-EDITED record");
    assert!(
        git_run(work, &["add", "-A"]).status.success(),
        "git add edit"
    );
    assert!(
        git_run(work, &["commit", "-q", "-m", "agent edit M'"])
            .status
            .success(),
        "git commit edit"
    );
    let m_prime = git_rev_parse(work, "HEAD").expect("edit commit M'");
    assert_ne!(m, m_prime, "M' must differ from base M");

    // REAL `reposix attach` (per-test sim via REPOSIX_SIM_ORIGIN threaded by
    // run_attach; the baked remote.reposix.url therefore targets THIS sim).
    let out = run_attach(
        work,
        cache,
        port,
        &["sim::demo", "--remote-name", "reposix"],
    );
    assert!(
        out.status.success(),
        "attach must succeed: stdout={:?} stderr={:?}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );

    // What Part A must produce: the seeded anchor at the mirror base.
    let anchor = git_rev_parse(work, "refs/reposix/origin/main");

    // Round-trip symptom capture (attach → edit → fetch → rebase). Best-effort:
    // on today's code the fetch synthesizes a parentless root under the default
    // refspec (refs/remotes/reposix/main); we rebase onto it to expose the wall.
    let path_env = path_with_target_debug();
    let fetch = git_run_with_helper(work, cache, &path_env, &["fetch", "reposix"]);
    let fetch_out = format!(
        "{}\n{}",
        String::from_utf8_lossy(&fetch.stdout),
        String::from_utf8_lossy(&fetch.stderr)
    );
    let fetched = git_rev_parse(work, "refs/remotes/reposix/main");
    let fetched_parent = git_rev_parse(work, "refs/remotes/reposix/main~1");
    let rebase = git_run_with_helper(
        work,
        cache,
        &path_env,
        &["rebase", "refs/remotes/reposix/main"],
    );
    let rebase_out = format!(
        "{}\n{}",
        String::from_utf8_lossy(&rebase.stdout),
        String::from_utf8_lossy(&rebase.stderr)
    );
    // Don't leave a rebase in progress (tempdir is discarded, but be tidy).
    let _ = git_run_with_helper(work, cache, &path_env, &["rebase", "--abort"]);

    kill_child(&mut sim);

    let evidence = format!(
        "REAL `reposix attach` did NOT seed refs/reposix/origin/main (item 4a).\n\
         mirror-base M                          = {m}\n\
         HEAD after edit (M')                   = {m_prime}\n\
         refs/reposix/origin/main after attach  = {anchor:?}   (expected Some({m}))\n\
         --- round-trip symptom (attach -> edit -> git fetch -> git rebase) ---\n\
         git fetch reposix exit_success={fetch_ok}\n{fetch_out}\n\
         fetched refs/remotes/reposix/main      = {fetched:?}\n\
         refs/remotes/reposix/main~1 (parent)   = {fetched_parent:?}   (None => PARENTLESS import root)\n\
         git rebase refs/remotes/reposix/main exit_success={rebase_ok}:\n{rebase_out}",
        fetch_ok = fetch.status.success(),
        rebase_ok = rebase.status.success(),
    );

    // HARD assertion — the Part A contract. RED today (anchor is None).
    assert_eq!(anchor.as_deref(), Some(m.as_str()), "{evidence}");
    // §3.1 silent-revert guard — the seed must be the mirror base M, NEVER HEAD
    // M' (seeding HEAD fast-forwards main over the un-pushed edit = data loss).
    assert_ne!(
        anchor.as_deref(),
        Some(m_prime.as_str()),
        "seed must be the mirror merge-base M, never HEAD M' (silent-revert data-loss guard):\n{evidence}"
    );
}
