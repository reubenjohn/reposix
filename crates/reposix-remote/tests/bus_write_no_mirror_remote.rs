//! Bus write fan-out regression test (DVCS-BUS-WRITE-05 / Q3.5).
//!
//! Asserts the architectural invariant: a bus URL with no local
//! `git remote` for the mirror still emits the verbatim Q3.5 hint
//! AFTER P83's write fan-out lands — P83 must NOT bypass P82's STEP 0
//! check.
//!
//! Donor pattern: `tests/bus_precheck_a.rs::bus_no_remote_configured_emits_q35_hint`.
//!
//! Assertions:
//!
//! 1. Helper exits non-zero.
//! 2. Helper stderr contains the verbatim Q3.5 hint (`configure the
//!    mirror remote first` + `git remote add`).
//! 3. NO auto-mutation of the working tree's `.git/config` —
//!    `.git/config` bytes identical before and after the helper
//!    invocation.
//! 4. NO PATCH calls hit the wiremock SoT — `Mock::expect(0)` on the
//!    PATCH route. Proves stdin was NEVER read past the bus header
//!    (the helper bailed at STEP 0 BEFORE reaching `parse_export_stream`).
//! 5. NO cache opened — STEP 0 bails BEFORE `ensure_cache` is called
//!    in `handle_bus_export`. The architectural invariant: the
//!    no-mirror-remote path is a "fast bail" before any cache work.
//!    Asserted by checking that the cache root contains no `.git`
//!    directory (no bare repo populated).

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::doc_markdown)] // test-internal doc comments cite SoT/refs/audit ops verbatim
#![allow(clippy::unnecessary_debug_formatting)] // stderr/path Debug is intentional in test diagnostics

use std::path::Path;
use std::process::Command;

use assert_cmd::Command as AssertCommand;
use serde_json::json;
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

mod common;
use common::{sample_issues, seed_mock};

fn run_git_in(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
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

#[tokio::test(flavor = "multi_thread")]
async fn bus_write_no_mirror_remote_emits_q35_hint() {
    let server = MockServer::start().await;
    let project = "demo";
    let issues = sample_issues(project, 3);
    seed_mock(&server, project, &issues).await;

    // PATCH backstop: MUST NEVER fire — STEP 0's bail-out is BEFORE
    // any stdin read or REST write. expect(0) tightens the assertion.
    Mock::given(method("PATCH"))
        .and(path_regex(format!(r"^/projects/{project}/issues/\d+$")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": 1, "version": 2})))
        .expect(0)
        .with_priority(1)
        .mount(&server)
        .await;

    // Working tree with NO `remote.<name>.url` for the mirror.
    let wtree = tempfile::tempdir().expect("wtree tempdir");
    run_git_in(wtree.path(), &["init", "."]);
    run_git_in(wtree.path(), &["config", "user.email", "p83@example"]);
    run_git_in(wtree.path(), &["config", "user.name", "P83 Test"]);

    // Capture .git/config bytes BEFORE the helper invocation.
    let config_path = wtree.path().join(".git").join("config");
    let config_before = std::fs::read(&config_path).expect("read .git/config before");

    // Bus URL: wiremock SoT + a file:// mirror that's NOT in
    // `remote.<name>.url` of the working tree.
    let mirror_url = "file:///nonexistent/p83-mirror.git";
    let bus_url = format!(
        "reposix::{}/projects/{project}?mirror={}",
        server.uri(),
        mirror_url
    );

    // Per-test cache dir — must remain unpopulated on the bail-out path.
    let cache_root = tempfile::tempdir().expect("cache_root");
    let cache_path = cache_root.path().to_path_buf();

    let out = AssertCommand::cargo_bin("git-remote-reposix")
        .expect("cargo bin")
        .args(["origin", &bus_url])
        .write_stdin("capabilities\n\nexport\n\n")
        .current_dir(wtree.path())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("REPOSIX_CACHE_DIR", &cache_path)
        .timeout(std::time::Duration::from_secs(15))
        .output()
        .expect("run helper");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    // ASSERTION 1: helper exited non-zero.
    assert!(
        !out.status.success(),
        "helper must exit non-zero on no-mirror-remote bail-out; \
         stdout={stdout}, stderr={stderr}"
    );

    // ASSERTION 2: stderr contains the verbatim Q3.5 hint.
    assert!(
        stderr.contains("configure the mirror remote first"),
        "expected Q3.5 hint; got stderr={stderr}"
    );
    assert!(
        stderr.contains("git remote add"),
        "expected Q3.5 hint to cite `git remote add`; got stderr={stderr}"
    );

    // ASSERTION 3: NO auto-mutation of .git/config (Q3.5 RATIFIED).
    let config_after = std::fs::read(&config_path).expect("read .git/config after");
    assert_eq!(
        config_before, config_after,
        ".git/config was auto-mutated — Q3.5 violated! \
         (helper added a remote it should NOT have)"
    );

    // ASSERTION 4 (implicit): wiremock's Drop checks Mock::expect(0)
    // — Drop will panic if PATCH fired even once.

    // ASSERTION 5: NO cache opened on bail-out path. The architectural
    // invariant — STEP 0 fires BEFORE any `ensure_cache` call in
    // bus_handler::handle_bus_export. Walk the cache root and confirm
    // no `.git` bare repo got populated.
    let any_bare = walkdir::WalkDir::new(cache_root.path())
        .into_iter()
        .filter_map(std::result::Result::ok)
        .any(|e| e.file_type().is_dir() && e.path().extension().is_some_and(|x| x == "git"));
    assert!(
        !any_bare,
        "bus_handler bail-out path opened a cache — STEP 0 must fire BEFORE ensure_cache; \
         cache root walked: {:?}",
        cache_root.path()
    );

    // Suppress unused-warning on tempdir (must outlive scope).
    let _ = wtree;
}
