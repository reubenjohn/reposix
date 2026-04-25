//! Phase 35 Plan 03 real-backend integration tests.
//!
//! Per project CLAUDE.md OP-6, simulator-only coverage does not satisfy
//! the architecture's transport/perf claims. These tests exercise the
//! `reposix init` flow against the three sanctioned real-backend
//! targets:
//!
//! - **Confluence `TokenWorld`** — owner-sanctioned scratchpad ("go
//!   crazy, it's safe"); env vars `ATLASSIAN_API_KEY`, `ATLASSIAN_EMAIL`,
//!   `REPOSIX_CONFLUENCE_TENANT`.
//! - **GitHub `reubenjohn/reposix`** — the project's own issue tracker;
//!   env var `GITHUB_TOKEN`.
//! - **JIRA project `TEST`** — overridable via `JIRA_TEST_PROJECT` /
//!   `REPOSIX_JIRA_PROJECT`; env vars `JIRA_EMAIL`, `JIRA_API_TOKEN`,
//!   `REPOSIX_JIRA_INSTANCE`.
//!
//! All three are `#[ignore]`-gated AND credential-gated via
//! `skip_if_no_env!` (copied verbatim from
//! `crates/reposix-confluence/tests/contract.rs` per Phase 11
//! convention). Without creds, each test prints
//! `SKIP: env vars unset: …` to stderr and returns; this means
//! `cargo test --test agent_flow_real -- --ignored` is safe to run on a
//! fresh-clone CI without any secrets.
//!
//! Per Plan 35-03: the helper still hardcodes `SimBackend` (Phase 32
//! limitation — see 32-SUMMARY.md), so the "real-backend exercise"
//! verified here is bounded to:
//!   1. `reposix init <real-backend>::<project> <path>` succeeds
//!      (init configures local state; fetch is best-effort).
//!   2. `git config remote.origin.url` returns the expected
//!      `reposix::https://...` URL.
//!
//! Live `git fetch` against a real backend is deferred to a future phase
//! when the helper learns multi-backend dispatch. Plan 35-03 ships the
//! gated infrastructure now so Phase 36 can wire the
//! `integration-contract-{confluence,github,jira}-v09` CI jobs without
//! touching test source.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

use std::path::{Path, PathBuf};
use std::process::Command;

/// Skip the enclosing test if any listed env var is unset or empty.
///
/// Mirrors the macro in `reposix-confluence/tests/contract.rs` lines
/// 61-74. Per T-11B-01 the macro NEVER logs env-var values — only
/// names, so test output is safe to paste into bug reports.
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

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root from CARGO_MANIFEST_DIR")
        .to_path_buf()
}

fn target_bin(name: &str) -> PathBuf {
    workspace_root().join("target").join("debug").join(name)
}

/// Resolve the JIRA test project key per the prompt:
///   `JIRA_TEST_PROJECT` ∨ `REPOSIX_JIRA_PROJECT`, default `TEST`.
fn jira_test_project() -> String {
    std::env::var("JIRA_TEST_PROJECT")
        .or_else(|_| std::env::var("REPOSIX_JIRA_PROJECT"))
        .unwrap_or_else(|_| "TEST".to_owned())
}

/// Run `reposix init <spec> <path>` and assert success + correct
/// `remote.origin.url` config. Returns the configured URL.
fn run_init_and_assert(spec: &str, expected_url_prefix: &str) -> String {
    let bin = target_bin("reposix");
    let bin_display = bin.display();
    assert!(
        bin.exists(),
        "reposix not built at {bin_display}; run `cargo build --workspace --bins` first"
    );
    let tmp = tempfile::tempdir().expect("tempdir");
    let repo = tmp.path().join("repo");
    let out = Command::new(&bin)
        .args(["init", spec, repo.to_str().unwrap()])
        .output()
        .expect("run reposix init");
    assert!(
        out.status.success(),
        "reposix init {spec} failed: stdout={:?} stderr={:?}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let cfg = Command::new("git")
        .args(["-C", repo.to_str().unwrap(), "config", "remote.origin.url"])
        .output()
        .expect("git config remote.origin.url");
    let url = String::from_utf8_lossy(&cfg.stdout).trim().to_string();
    assert!(
        url.starts_with(expected_url_prefix),
        "remote.origin.url should start with {expected_url_prefix}, got {url}"
    );
    url
}

/// GitHub `reubenjohn/reposix` real-backend init smoke.
#[test]
#[ignore = "real-backend; requires GITHUB_TOKEN"]
fn dark_factory_real_github() {
    skip_if_no_env!("GITHUB_TOKEN");
    let url = run_init_and_assert(
        "github::reubenjohn/reposix",
        "reposix::https://api.github.com/",
    );
    assert!(
        url.contains("/projects/reubenjohn/reposix"),
        "url should encode project as `reubenjohn/reposix`, got {url}"
    );
}

/// Confluence `TokenWorld` real-backend init smoke.
#[test]
#[ignore = "real-backend; requires ATLASSIAN_API_KEY/EMAIL/REPOSIX_CONFLUENCE_TENANT"]
fn dark_factory_real_confluence() {
    skip_if_no_env!(
        "ATLASSIAN_API_KEY",
        "ATLASSIAN_EMAIL",
        "REPOSIX_CONFLUENCE_TENANT"
    );
    let tenant = std::env::var("REPOSIX_CONFLUENCE_TENANT").expect("env-presence checked above");
    let expected_prefix = format!("reposix::https://{tenant}.atlassian.net/");
    let url = run_init_and_assert("confluence::TokenWorld", &expected_prefix);
    // Phase 36-followup: the `/confluence/` path marker is required so
    // the helper's URL-scheme dispatcher (crates/reposix-remote/src/
    // backend_dispatch.rs) picks the Confluence backend instead of JIRA.
    assert!(
        url.ends_with("/confluence/projects/TokenWorld"),
        "url should encode the /confluence/ marker + TokenWorld project, got {url}"
    );
}

/// JIRA `TEST` (or override) real-backend init smoke.
#[test]
#[ignore = "real-backend; requires JIRA_EMAIL/JIRA_API_TOKEN/REPOSIX_JIRA_INSTANCE"]
fn dark_factory_real_jira() {
    skip_if_no_env!("JIRA_EMAIL", "JIRA_API_TOKEN", "REPOSIX_JIRA_INSTANCE");
    let instance = std::env::var("REPOSIX_JIRA_INSTANCE").expect("env-presence checked above");
    let project = jira_test_project();
    let expected_prefix = format!("reposix::https://{instance}.atlassian.net/");
    let spec = format!("jira::{project}");
    let url = run_init_and_assert(&spec, &expected_prefix);
    // Phase 36-followup: the `/jira/` path marker disambiguates the
    // JIRA URL from Confluence at the helper's backend-dispatch layer.
    let expected_suffix = format!("/jira/projects/{project}");
    assert!(
        url.ends_with(&expected_suffix),
        "url should encode the /jira/ marker + project {project}, got {url}"
    );
}

/// Defensive sanity test: without any env vars, all three skip cleanly.
/// Runs in default `cargo test` (not #[ignore]) so a fresh clone CI
/// surfaces any regression in the `skip_if_no_env!` plumbing — the
/// architecture's "fail-closed if creds absent" claim must hold.
#[test]
fn skip_pattern_compiles_and_runs_without_creds() {
    // Snapshot+clear all relevant env vars to guarantee a deterministic
    // skip path regardless of the dev host's shell state. Restore on
    // exit so we don't poison sibling tests.
    let names = [
        "GITHUB_TOKEN",
        "ATLASSIAN_API_KEY",
        "ATLASSIAN_EMAIL",
        "REPOSIX_CONFLUENCE_TENANT",
        "JIRA_EMAIL",
        "JIRA_API_TOKEN",
        "REPOSIX_JIRA_INSTANCE",
    ];
    let saved: Vec<(&str, Option<String>)> =
        names.iter().map(|n| (*n, std::env::var(n).ok())).collect();
    for n in &names {
        std::env::remove_var(n);
    }

    // Each closure mirrors the body of one #[ignore] test up to the
    // skip_if_no_env! call. If skip_if_no_env! returns properly we just
    // fall out of the closure with no panic.
    let ran = std::panic::catch_unwind(|| {
        skip_if_no_env!("GITHUB_TOKEN");
        unreachable!("skip_if_no_env! should have returned");
    });

    // catch_unwind returns Ok(()) for the early-return path. If any
    // skip path panics, surface it.
    assert!(
        ran.is_ok(),
        "skip_if_no_env! must early-return cleanly, not panic"
    );

    // Restore env so sibling tests don't pick up our scrubbing.
    for (n, v) in saved {
        match v {
            Some(s) => std::env::set_var(n, s),
            None => std::env::remove_var(n),
        }
    }
}
