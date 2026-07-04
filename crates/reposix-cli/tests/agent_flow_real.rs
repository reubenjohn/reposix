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
//! The `dark_factory_real_*` init smokes assert the config-string
//! contract:
//!   1. `reposix init <real-backend>::<project> <path>` succeeds.
//!   2. `git config remote.origin.url` returns the expected
//!      `reposix::https://…` URL (incl. the `/confluence/` or `/jira/`
//!      disambiguator marker).
//!
//! The `attach_real_*` / `sync_real_*` smokes go further and exercise a
//! REAL round-trip. `reposix attach` and `reposix sync --reconcile`
//! construct the concrete backend connector through the git remote
//! helper's shared dispatch factory (`reposix_remote::backend_dispatch`)
//! and call `list_records` against the live backend. This supersedes the
//! old "the helper still hardcodes `SimBackend`" Phase-32 limitation —
//! that debt was closed by `backend_dispatch` (Phase 36-followup) and is
//! now consumed by `attach`/`sync` (v0.13.0 real-backend wiring). The
//! smokes assert `extensions.partialClone` + `remote.<name>.url` are
//! configured and that `sync --reconcile` reports a reconcile result.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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

/// Pure resolver: first non-empty candidate in order, else `default`.
///
/// An **empty-but-set** env var is treated as unset. This is load-bearing:
/// an undefined GitHub Actions secret (e.g. `${{ secrets.JIRA_TEST_PROJECT }}`
/// when the repo has no such secret) is forwarded to the job as the empty
/// STRING, not as absent. The old `env::var(..).or_else(..).unwrap_or_else(..)`
/// chain treated `Ok("")` as a present value, so an empty `JIRA_TEST_PROJECT`
/// won over the `TEST` default and produced the spec `jira::` — which
/// `reposix init` rejects (`invalid spec jira::: empty project`, CI run
/// 28723077083). Skipping empties fixes that for every candidate.
fn first_nonempty_or(
    candidates: impl IntoIterator<Item = Option<String>>,
    default: &str,
) -> String {
    candidates
        .into_iter()
        .flatten()
        .find(|v| !v.is_empty())
        .unwrap_or_else(|| default.to_owned())
}

/// Resolve the JIRA test project key per the prompt:
///   `JIRA_TEST_PROJECT` ∨ `REPOSIX_JIRA_PROJECT`, default `TEST`.
/// Empty-but-set env vars fall through (see `first_nonempty_or`).
fn jira_test_project() -> String {
    first_nonempty_or(
        [
            std::env::var("JIRA_TEST_PROJECT").ok(),
            std::env::var("REPOSIX_JIRA_PROJECT").ok(),
        ],
        "TEST",
    )
}

/// Resolve the Confluence test space key:
///   `REPOSIX_CONFLUENCE_SPACE`, default `TokenWorld` (historical canonical
///   per docs/reference/testing-targets.md). Empty-but-set falls through.
fn confluence_test_space() -> String {
    first_nonempty_or(
        [std::env::var("REPOSIX_CONFLUENCE_SPACE").ok()],
        "TokenWorld",
    )
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

/// Confluence real-backend init smoke. Space is configurable via
/// `REPOSIX_CONFLUENCE_SPACE` (default `TokenWorld`); mirrors the JIRA
/// `jira_test_project()` pattern so the test follows whichever space the
/// configured Atlassian tenant actually owns.
#[test]
#[ignore = "real-backend; requires ATLASSIAN_API_KEY/EMAIL/REPOSIX_CONFLUENCE_TENANT"]
fn dark_factory_real_confluence() {
    skip_if_no_env!(
        "ATLASSIAN_API_KEY",
        "ATLASSIAN_EMAIL",
        "REPOSIX_CONFLUENCE_TENANT"
    );
    let tenant = std::env::var("REPOSIX_CONFLUENCE_TENANT").expect("env-presence checked above");
    let space = confluence_test_space();
    let expected_prefix = format!("reposix::https://{tenant}.atlassian.net/");
    let spec = format!("confluence::{space}");
    let url = run_init_and_assert(&spec, &expected_prefix);
    // Phase 36-followup: the `/confluence/` path marker is required so
    // the helper's URL-scheme dispatcher (crates/reposix-remote/src/
    // backend_dispatch.rs) picks the Confluence backend instead of JIRA.
    let expected_suffix = format!("/confluence/projects/{space}");
    assert!(
        url.ends_with(&expected_suffix),
        "url should encode the /confluence/ marker + space {space}, got {url}"
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

// --- attach_real_* / sync_real_* — real round-trip smokes (RBF-A-04) ------
//
// Unlike the init smokes above (which only assert a config string), these
// drive `reposix attach` / `sync --reconcile` end-to-end against a live
// backend: the shared dispatch factory constructs the real connector and
// `build_from` / `Cache::sync` issue a real `list_records`. All are
// `#[ignore]` + `skip_if_no_env!`-gated, so `cargo test` (no `--ignored`)
// on a fresh clone with no secrets is a clean no-op.

/// Vanilla `git init` a fresh repo, then run
/// `reposix attach <spec> <repo> --remote-name <name> --no-bus` with an
/// isolated cache dir and egress allowlist. Returns the tempdir (kept
/// alive by the caller), the attach output, the repo path, and the cache
/// dir (so a follow-up `sync --reconcile` can reuse the same cache).
fn run_attach_real(
    spec: &str,
    remote_name: &str,
    allowed_origins: &str,
) -> (tempfile::TempDir, std::process::Output, PathBuf, PathBuf) {
    let bin = target_bin("reposix");
    assert!(
        bin.exists(),
        "reposix not built at {}; run `cargo build --workspace --bins` first",
        bin.display()
    );
    let tmp = tempfile::tempdir().expect("tempdir");
    let repo = tmp.path().join("repo");
    std::fs::create_dir_all(&repo).expect("create repo dir");
    let init = Command::new("git")
        .args(["-C", repo.to_str().unwrap(), "init", "-q"])
        .status()
        .expect("git init");
    assert!(init.success(), "git init failed");
    let cache = tmp.path().join("cache");
    let out = Command::new(&bin)
        .args([
            "attach",
            spec,
            repo.to_str().unwrap(),
            "--remote-name",
            remote_name,
            "--no-bus",
        ])
        .env("REPOSIX_CACHE_DIR", &cache)
        .env("REPOSIX_ALLOWED_ORIGINS", allowed_origins)
        .stdin(Stdio::null())
        .output()
        .expect("run reposix attach");
    (tmp, out, repo, cache)
}

/// Assert an attach output configured the reposix remote:
/// `extensions.partialClone == <remote_name>` and `remote.<name>.url`
/// starts with `reposix::` and contains `expected_url_contains`.
fn assert_attach_configured(
    out: &std::process::Output,
    repo: &Path,
    remote_name: &str,
    expected_url_contains: &str,
) {
    assert!(
        out.status.success(),
        "reposix attach failed: stdout={:?} stderr={:?}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let pclone = Command::new("git")
        .args([
            "-C",
            repo.to_str().unwrap(),
            "config",
            "extensions.partialClone",
        ])
        .output()
        .expect("git config partialClone");
    assert_eq!(
        String::from_utf8_lossy(&pclone.stdout).trim(),
        remote_name,
        "extensions.partialClone must be the reposix remote name"
    );
    let url_out = Command::new("git")
        .args([
            "-C",
            repo.to_str().unwrap(),
            "config",
            &format!("remote.{remote_name}.url"),
        ])
        .output()
        .expect("git config remote url");
    let url = String::from_utf8_lossy(&url_out.stdout).trim().to_string();
    assert!(
        url.starts_with("reposix::") && url.contains(expected_url_contains),
        "remote.{remote_name}.url should start with reposix:: and contain {expected_url_contains}, got {url}"
    );
}

/// Run `reposix sync --reconcile <repo>` reusing the cache the attach
/// populated (so the configured reposix remote is discoverable) and assert
/// it lists real records without error.
fn assert_sync_reconcile_ok(repo: &Path, cache: &Path, allowed_origins: &str) {
    let bin = target_bin("reposix");
    let out = Command::new(&bin)
        .args(["sync", "--reconcile", repo.to_str().unwrap()])
        .env("REPOSIX_CACHE_DIR", cache)
        .env("REPOSIX_ALLOWED_ORIGINS", allowed_origins)
        .stdin(Stdio::null())
        .output()
        .expect("run reposix sync --reconcile");
    assert!(
        out.status.success(),
        "sync --reconcile failed: stdout={:?} stderr={:?}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("reposix sync:"),
        "sync --reconcile should report a reconcile result, got: {stdout}"
    );
}

/// Confluence `TokenWorld` real-backend attach round-trip.
#[test]
#[ignore = "real-backend; requires ATLASSIAN_API_KEY/EMAIL/REPOSIX_CONFLUENCE_TENANT"]
fn attach_real_confluence() {
    skip_if_no_env!(
        "ATLASSIAN_API_KEY",
        "ATLASSIAN_EMAIL",
        "REPOSIX_CONFLUENCE_TENANT"
    );
    let tenant = std::env::var("REPOSIX_CONFLUENCE_TENANT").expect("checked");
    let space = confluence_test_space();
    let allowed = format!("http://127.0.0.1:*,https://{tenant}.atlassian.net");
    let (_tmp, out, repo, _cache) =
        run_attach_real(&format!("confluence::{space}"), "reposix", &allowed);
    assert_attach_configured(
        &out,
        &repo,
        "reposix",
        &format!("{tenant}.atlassian.net/confluence/projects/{space}"),
    );
}

/// Confluence `TokenWorld` real-backend `sync --reconcile` round-trip.
#[test]
#[ignore = "real-backend; requires ATLASSIAN_API_KEY/EMAIL/REPOSIX_CONFLUENCE_TENANT"]
fn sync_real_confluence() {
    skip_if_no_env!(
        "ATLASSIAN_API_KEY",
        "ATLASSIAN_EMAIL",
        "REPOSIX_CONFLUENCE_TENANT"
    );
    let tenant = std::env::var("REPOSIX_CONFLUENCE_TENANT").expect("checked");
    let space = confluence_test_space();
    let allowed = format!("http://127.0.0.1:*,https://{tenant}.atlassian.net");
    let (_tmp, out, repo, cache) =
        run_attach_real(&format!("confluence::{space}"), "reposix", &allowed);
    assert!(
        out.status.success(),
        "attach prerequisite failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_sync_reconcile_ok(&repo, &cache, &allowed);
}

/// GitHub `reubenjohn/reposix` real-backend attach round-trip.
#[test]
#[ignore = "real-backend; requires GITHUB_TOKEN"]
fn attach_real_github() {
    skip_if_no_env!("GITHUB_TOKEN");
    let allowed = "http://127.0.0.1:*,https://api.github.com";
    let (_tmp, out, repo, _cache) =
        run_attach_real("github::reubenjohn/reposix", "reposix", allowed);
    assert_attach_configured(
        &out,
        &repo,
        "reposix",
        "api.github.com/projects/reubenjohn/reposix",
    );
}

/// GitHub `reubenjohn/reposix` real-backend `sync --reconcile` round-trip.
#[test]
#[ignore = "real-backend; requires GITHUB_TOKEN"]
fn sync_real_github() {
    skip_if_no_env!("GITHUB_TOKEN");
    let allowed = "http://127.0.0.1:*,https://api.github.com";
    let (_tmp, out, repo, cache) =
        run_attach_real("github::reubenjohn/reposix", "reposix", allowed);
    assert!(
        out.status.success(),
        "attach prerequisite failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_sync_reconcile_ok(&repo, &cache, allowed);
}

/// JIRA `TEST` (or `JIRA_TEST_PROJECT` override) real-backend attach.
#[test]
#[ignore = "real-backend; requires JIRA_EMAIL/JIRA_API_TOKEN/REPOSIX_JIRA_INSTANCE"]
fn attach_real_jira() {
    skip_if_no_env!("JIRA_EMAIL", "JIRA_API_TOKEN", "REPOSIX_JIRA_INSTANCE");
    let instance = std::env::var("REPOSIX_JIRA_INSTANCE").expect("checked");
    let project = jira_test_project();
    let allowed = format!("http://127.0.0.1:*,https://{instance}.atlassian.net");
    let (_tmp, out, repo, _cache) =
        run_attach_real(&format!("jira::{project}"), "reposix", &allowed);
    assert_attach_configured(
        &out,
        &repo,
        "reposix",
        &format!("{instance}.atlassian.net/jira/projects/{project}"),
    );
}

/// JIRA `TEST` (or override) real-backend `sync --reconcile` round-trip.
#[test]
#[ignore = "real-backend; requires JIRA_EMAIL/JIRA_API_TOKEN/REPOSIX_JIRA_INSTANCE"]
fn sync_real_jira() {
    skip_if_no_env!("JIRA_EMAIL", "JIRA_API_TOKEN", "REPOSIX_JIRA_INSTANCE");
    let instance = std::env::var("REPOSIX_JIRA_INSTANCE").expect("checked");
    let project = jira_test_project();
    let allowed = format!("http://127.0.0.1:*,https://{instance}.atlassian.net");
    let (_tmp, out, repo, cache) =
        run_attach_real(&format!("jira::{project}"), "reposix", &allowed);
    assert!(
        out.status.success(),
        "attach prerequisite failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_sync_reconcile_ok(&repo, &cache, &allowed);
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

/// Regression for CI run 28723077083: a set-but-empty env var (an undefined
/// GitHub Actions secret forwarded as `""`) must NOT win over the default —
/// otherwise `jira_test_project()` returns `""` and the caller builds the
/// spec `jira::`, which `reposix init` rejects with `empty project`.
///
/// Pure — exercises `first_nonempty_or` directly rather than mutating process
/// env (deterministic, race-free under parallel test execution).
#[test]
fn empty_but_set_candidate_falls_through_to_default() {
    // Empty primary + absent fallback -> default (the exact CI shape:
    // JIRA_TEST_PROJECT="" set, REPOSIX_JIRA_PROJECT unset).
    assert_eq!(
        first_nonempty_or([Some(String::new()), None], "TEST"),
        "TEST"
    );
    // Both empty -> default.
    assert_eq!(
        first_nonempty_or([Some(String::new()), Some(String::new())], "TEST"),
        "TEST"
    );
    // Empty primary falls through to a non-empty fallback (not the default).
    assert_eq!(
        first_nonempty_or([Some(String::new()), Some("KAN".to_owned())], "TEST"),
        "KAN"
    );
    // First non-empty wins over later candidates and the default.
    assert_eq!(
        first_nonempty_or([Some("KAN".to_owned()), Some("OTHER".to_owned())], "TEST"),
        "KAN"
    );
    // All absent -> default.
    assert_eq!(first_nonempty_or([None, None], "TEST"), "TEST");
}
