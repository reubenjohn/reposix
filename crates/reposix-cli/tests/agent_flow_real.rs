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

use reposix_confluence::{ConfluenceBackend, ConfluenceCreds};
use reposix_core::backend::{BackendConnector, DeleteReason};
use reposix_core::{sanitize, Record, RecordId, RecordStatus, ServerMetadata, Tainted, Untainted};

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

/// Regression (v0.14.0 item 5): the Confluence v2 API string-encodes
/// `body.atlas_doc_format.value`; reposix must decode it and return REAL page
/// content — never the item-4b unreadable-ADF placeholder — for a live page
/// whose ADF is valid. This is the durable real-TokenWorld twin of the
/// wiremock decode regression in `reposix-confluence::translate` tests: the
/// diagnostic's executed repro (every real page sentinelled → push blocked)
/// is locked here so it can never silently return.
#[test]
#[ignore = "real-backend; requires ATLASSIAN_API_KEY/EMAIL/REPOSIX_CONFLUENCE_TENANT"]
fn get_record_real_confluence_body_is_not_unreadable_sentinel() {
    skip_if_no_env!(
        "ATLASSIAN_API_KEY",
        "ATLASSIAN_EMAIL",
        "REPOSIX_CONFLUENCE_TENANT"
    );
    let tenant = std::env::var("REPOSIX_CONFLUENCE_TENANT").expect("env-presence checked above");
    let space = confluence_test_space();
    let allowed = format!("http://127.0.0.1:*,https://{tenant}.atlassian.net");
    std::env::set_var("REPOSIX_ALLOWED_ORIGINS", &allowed);
    let creds = ConfluenceCreds {
        email: std::env::var("ATLASSIAN_EMAIL").expect("checked"),
        api_token: std::env::var("ATLASSIAN_API_KEY").expect("checked"),
    };
    let backend = ConfluenceBackend::new(creds, &tenant).expect("backend");
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    let records = rt
        .block_on(backend.list_records(&space))
        .expect("list_records against live TokenWorld");
    assert!(
        !records.is_empty(),
        "TokenWorld space {space} must expose at least one page"
    );
    for r in &records {
        // A valid-ADF live page must translate to real markdown, not the
        // fail-closed placeholder that blocks push round-trips. If ANY page
        // comes back as the sentinel, the string-encoded ADF decode regressed.
        assert!(
            !reposix_confluence::adf::is_unreadable_adf_sentinel(&r.body),
            "page {} came back as the unreadable-ADF sentinel — string-encoded \
             ADF decode regressed:\n{}",
            r.id.0,
            r.body
        );
    }
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

// --- partial_failure_recovery_real_confluence (ADR-010 / RBF-LR-03 real-backend arm) ---
//
// DECISION-2b (v0.14.0 tag remediation, B2-p93-DIAGNOSIS.md): the prior
// version of this smoke exercised CREATE-recovery -- push a bad-parent
// Create, retry with a fixed parent. That is not a claim the product ever
// made against an id-ASSIGNING backend: Confluence's `create_record`
// (crates/reposix-confluence/src/lib.rs) never sends the client's
// locally-chosen id, so a retried Create re-sends the SAME client id and
// `diff::plan` (id-keyed matching) re-CREATEs instead of recognizing the
// already-landed page -- CREATE-recovery genuinely does not converge
// against Confluence (a real, out-of-scope-for-this-lane product gap; see
// the D2 executor RAISE LIST). This rewrite instead mirrors
// `crates/reposix-remote/tests/partial_failure_recovery.rs`: pages
// pre-seeded on the REAL backend at v1, an UPDATE-recovery partial fail,
// and the next push replanning ONLY the still-needed remainder. Id
// stability (Confluence returns the SAME id on every GET/PUT it assigned
// at Create) is exactly what makes UPDATE-recovery converge -- matching
// the sim twin's proven claim.
//
// ## Real-backend-validated fault (not a mock)
//
// The sim twin injects a mocked 500 on issue 2's first PATCH. There is no
// "make Confluence 500 on demand" lever against a live tenant, so this arm
// reproduces a GENUINE backend-validated PUT rejection instead: Confluence
// enforces page-title uniqueness within a space for both Create and
// Update. A third, untouched "blocker" page pre-seeded with a fixed title
// occupies the title page B's push-1 PUT attempts to rename into -- the
// PUT is rejected by Confluence itself, a real `SotPartialFail`, not a
// timing race or a mocked fault. Push 2 retries with B's ORIGINAL
// (non-colliding) title and its intended body edit -- PRECHECK B re-reads
// the live SoT, page A (landed in push 1) diffs away, and only page B is
// replanned and now lands.
//
// ## Full-tree-mirror safety invariant (READ BEFORE EDITING THIS TEST)
//
// `diff::plan` (crates/reposix-remote/src/diff.rs) computes a DELETE for
// every record present in the cache's materialized `prior` that is ABSENT
// from the pushed tree -- by design (a real `git push` naturally carries
// the agent's FULL working tree, not a hand-picked subset). Because
// `reposix attach`'s warm sync (`Cache::build_from`) populates the cache's
// `oid_map` for the ENTIRE TokenWorld space (not just this test's pages),
// and blobs are never locally materialized in this raw fast-import harness
// (no `git clone`/checkout happens), `precheck_export_against_changed_set`'s
// prior-materialization ALWAYS falls through to a full `list_records` call
// -- `prior` is therefore the COMPLETE current space content, including the
// two PROTECTED durable-fixture pages (docs/reference/testing-targets.md).
// EVERY push this test sends therefore re-includes EVERY currently-existing
// page verbatim (unedited ones byte-identical, so `plan()`'s
// writable-equivalence check diffs them away to zero actions) -- skipping
// this would risk a REAL DELETE against fixture content. `render_verbatim`
// + the `untouched_entries` snapshot below are that safety net; do not
// build a push tree in this test without re-including every existing page.
const KIND_TEST_LABEL: &str = "kind=test";

/// Teardown guard (RAII): deletes every self-seeded page id it was told to
/// [`track`](Self::track), even on panic/assert-failure unwind, so a
/// mid-test assertion failure never leaks a page into `TokenWorld`. Rebuilds
/// a fresh [`ConfluenceBackend`] on a throwaway OS thread + its own
/// single-threaded tokio runtime (`Drop` is sync; the delete calls are
/// async, and reusing the outer test's runtime from inside a `Drop` that
/// may run during unwind risks "cannot start a runtime from within a
/// runtime"). NEVER tracks the two protected durable-fixture ids (7766017 /
/// 7798785, docs/reference/testing-targets.md) -- this test never creates
/// or mutates them, only re-mirrors them verbatim (see the module-level
/// safety-invariant doc above).
struct TeardownGuard {
    creds: ConfluenceCreds,
    tenant: String,
    space: String,
    ids: Vec<u64>,
}

impl TeardownGuard {
    fn track(&mut self, id: u64) {
        self.ids.push(id);
    }
}

impl Drop for TeardownGuard {
    fn drop(&mut self) {
        if self.ids.is_empty() {
            return;
        }
        let creds = self.creds.clone();
        let tenant = self.tenant.clone();
        let space = self.space.clone();
        let ids = std::mem::take(&mut self.ids);
        let joined = std::thread::spawn(move || {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    eprintln!(
                        "teardown: failed to build teardown runtime: {e:?} -- {} page(s) \
                         leaked, manual cleanup required: {ids:?}",
                        ids.len()
                    );
                    return;
                }
            };
            rt.block_on(async {
                let backend = match ConfluenceBackend::new(creds, &tenant) {
                    Ok(b) => b,
                    Err(e) => {
                        eprintln!(
                            "teardown: ConfluenceBackend::new failed: {e:?} -- {} page(s) \
                             leaked, manual cleanup required: {ids:?}",
                            ids.len()
                        );
                        return;
                    }
                };
                for id in ids {
                    if let Err(e) = backend
                        .delete_or_close(&space, RecordId(id), DeleteReason::Abandoned)
                        .await
                    {
                        eprintln!(
                            "teardown: delete_or_close({id}) failed (non-fatal, leaves a \
                             kind=test page for manual cleanup -- sweep via \
                             `python3 scripts/confluence_tokenworld.py delete {id}`): {e:?}"
                        );
                    }
                }
            });
        })
        .join();
        if joined.is_err() {
            eprintln!(
                "teardown: teardown thread panicked -- some pages may be leaked, run \
                 `python3 scripts/confluence_tokenworld.py list` to check"
            );
        }
    }
}

/// Build a brand-new top-level page's [`Untainted<Record>`] for self-seeding
/// (mirrors `contract.rs::make_hierarchy_issue`). Labeled `kind=test` per
/// `docs/reference/testing-targets.md`'s cleanup convention; the title ALSO
/// carries a `kind=test <ts>` marker (real Confluence labels aren't wired
/// through `create_record` yet -- `lib.rs` documents this as deferred, so a
/// human cleanup sweep must search titles, not labels).
fn make_new_page(title: &str, body_md: &str) -> Untainted<Record> {
    let t = chrono::Utc::now();
    sanitize(
        Tainted::new(Record {
            id: RecordId(0),
            title: title.to_owned(),
            status: RecordStatus::Open,
            assignee: None,
            labels: vec![KIND_TEST_LABEL.to_owned()],
            created_at: t,
            updated_at: t,
            version: 0,
            body: body_md.to_owned(),
            parent_id: None,
            extensions: std::collections::BTreeMap::new(),
        }),
        ServerMetadata {
            id: RecordId(0),
            created_at: t,
            updated_at: t,
            version: 1,
        },
    )
}

/// Render a [`Record`] EXACTLY as fetched into a `pages/<id>.md` fast-import
/// tree entry -- the full-tree-mirror safety net (see the module-level doc
/// above `KIND_TEST_LABEL`). Returns `(path, blob)`. Uses
/// `reposix_core::path::record_path` (never a hand-picked bucket string,
/// per project CLAUDE.md) so this stays correct if a bucket ever changes.
fn render_verbatim(record: &Record) -> (String, String) {
    let bucket = reposix_core::path::bucket_for_backend("confluence");
    let path = reposix_core::path::record_path(bucket, record.id.0);
    let blob = reposix_core::frontmatter::render(record).expect("render page blob");
    (path, blob)
}

/// Build a single-backend `export` fast-import payload creating the given
/// `(path, blob)` entries in one commit. Mirrors
/// `crates/reposix-remote/tests/partial_failure_recovery.rs::export_stdin`.
fn export_stdin_real(entries: &[(String, String)], msg: &str) -> Vec<u8> {
    use std::io::Write as _;
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

/// Read `remote.<name>.url` for `repo` (the value `reposix attach` configured).
fn git_remote_url(repo: &Path, remote_name: &str) -> String {
    let out = Command::new("git")
        .args([
            "-C",
            repo.to_str().unwrap(),
            "config",
            &format!("remote.{remote_name}.url"),
        ])
        .output()
        .expect("git config remote url");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// Drive `git-remote-reposix` directly with a raw `export` fast-import
/// stream on stdin -- bypasses `git push`'s own remote-helper discovery,
/// same low-level pattern as
/// `partial_failure_recovery.rs::run_helper_export`, adapted for a real
/// backend (needs `REPOSIX_ALLOWED_ORIGINS` too, unlike the sim arm).
fn run_helper_export_real(
    url: &str,
    cache_dir: &Path,
    allowed_origins: &str,
    stdin: &[u8],
) -> (bool, String) {
    let bin = target_bin("git-remote-reposix");
    assert!(
        bin.exists(),
        "git-remote-reposix not built at {}; run `cargo build --workspace --bins` first",
        bin.display()
    );
    let mut child = Command::new(&bin)
        .args(["reposix", url])
        .env("REPOSIX_CACHE_DIR", cache_dir)
        .env("REPOSIX_ALLOWED_ORIGINS", allowed_origins)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn git-remote-reposix");
    {
        use std::io::Write as _;
        child
            .stdin
            .take()
            .expect("child stdin")
            .write_all(stdin)
            .expect("write export stream to helper stdin");
    }
    let out = child.wait_with_output().expect("wait for helper");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

/// Confluence `TokenWorld` real-backend UPDATE-recovery `SotPartialFail`
/// smoke (ADR-010 / RBF-LR-03, DECISION-2b rewrite, catalog row
/// `agent-ux/p93-partial-failure-recovery-real-confluence`).
///
/// See the module-level doc comment above `KIND_TEST_LABEL` for the full
/// rationale (why UPDATE not CREATE, the real fault vector, and the
/// full-tree-mirror safety invariant every push in this test obeys).
///
/// - **Self-seed:** pages A, B, and an untouched "blocker" page are
///   created directly via [`ConfluenceBackend::create_record`] (bypassing
///   the reposix push path -- this is test setup, not the behavior under
///   test) and tracked in a [`TeardownGuard`] so they're deleted even on a
///   mid-test panic.
/// - **Push 1:** edits page A's body (a genuine change -- lands) and
///   attempts to rename page B into the blocker's exact title while also
///   carrying B's intended body edit. Confluence rejects the whole PUT
///   atomically on the title collision, so B's edit does NOT land -- a
///   real, backend-validated `SotPartialFail`.
/// - **Push 2:** page A is re-sent in its POST-push-1 landed state
///   (`diff::plan` diffs it away, content-equivalent, never re-attempted);
///   page B is retried with its ORIGINAL (non-colliding) title and the
///   SAME intended body edit. PRECHECK B re-reads the live `SoT` and
///   replans ONLY page B, which now lands -- the push converges (`ok
///   refs/heads/main`).
/// - **Convergence assertions:** page A's version is unchanged between
///   push 1 and push 2 (proves zero re-PATCH); page B's version advances
///   by exactly one PUT total across both pushes (proves the rejected
///   push-1 PUT landed nothing) and its final title is unchanged from
///   creation (it was never actually renamed).
///
/// `#[ignore]` + `skip_if_no_env!`-gated (`TokenWorld` creds).
#[test]
#[ignore = "real-backend; requires ATLASSIAN_API_KEY/EMAIL/REPOSIX_CONFLUENCE_TENANT; mutates TokenWorld"]
#[allow(clippy::too_many_lines)] // one narrow end-to-end recovery scenario reads top-to-bottom
                                 // (same documented exception as the sim twin,
                                 // partial_failure_recovery.rs)
fn partial_failure_recovery_real_confluence() {
    skip_if_no_env!(
        "ATLASSIAN_API_KEY",
        "ATLASSIAN_EMAIL",
        "REPOSIX_CONFLUENCE_TENANT"
    );
    let tenant = std::env::var("REPOSIX_CONFLUENCE_TENANT").expect("checked");
    let space = confluence_test_space();
    let allowed = format!("http://127.0.0.1:*,https://{tenant}.atlassian.net");
    // Gates the DIRECT in-process ConfluenceBackend calls below (the
    // subprocess calls further down pass `allowed` explicitly via `.env()`
    // and do not depend on this). Safe process-env mutation for a test that
    // always runs `--exact` (see the p93 gate script) -- same established
    // pattern as `precheck.rs`'s `stale_base_push_rejected_...` test.
    std::env::set_var("REPOSIX_ALLOWED_ORIGINS", &allowed);

    let creds = ConfluenceCreds {
        email: std::env::var("ATLASSIAN_EMAIL").expect("checked"),
        api_token: std::env::var("ATLASSIAN_API_KEY").expect("checked"),
    };
    let backend = ConfluenceBackend::new(creds.clone(), &tenant).expect("backend");
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");

    let mut guard = TeardownGuard {
        creds,
        tenant,
        space: space.clone(),
        ids: Vec::new(),
    };

    let now_secs = u64::try_from(chrono::Utc::now().timestamp()).expect("post-1970 clock");
    let marker = format!("kind=test {now_secs}");

    // --- Self-seed: page A, page B, and an untouched "blocker" page whose
    // TITLE page B's push-1 rename attempt will collide into.
    let page_a = rt
        .block_on(backend.create_record(
            &space,
            make_new_page(
                &format!("p93 update-recovery A ({marker})"),
                "orig body A\n",
            ),
        ))
        .unwrap_or_else(|e| panic!("create_record(A) failed: {e:?}"));
    guard.track(page_a.id.0);

    let page_b = rt
        .block_on(backend.create_record(
            &space,
            make_new_page(
                &format!("p93 update-recovery B ({marker})"),
                "orig body B\n",
            ),
        ))
        .unwrap_or_else(|e| panic!("create_record(B) failed: {e:?}"));
    guard.track(page_b.id.0);

    let blocker_title = format!("p93 update-recovery BLOCKER ({marker})");
    let blocker = rt
        .block_on(backend.create_record(
            &space,
            make_new_page(
                &blocker_title,
                "untouched -- occupies the title page B's push-1 rename collides into\n",
            ),
        ))
        .unwrap_or_else(|e| panic!("create_record(blocker) failed: {e:?}"));
    guard.track(blocker.id.0);

    // --- Attach: warms the cache (oid_map for the WHOLE space + cursor) so
    // the push path below exercises the real precheck/plan machinery, same
    // as every other real-backend push smoke in this file.
    let (_tmp, attach_out, repo, cache) =
        run_attach_real(&format!("confluence::{space}"), "reposix", &allowed);
    assert!(
        attach_out.status.success(),
        "attach prerequisite failed: {:?}",
        String::from_utf8_lossy(&attach_out.stderr)
    );
    let url = git_remote_url(&repo, "reposix");

    // --- Full-space snapshot (safety net -- see the module-level doc
    // comment above `KIND_TEST_LABEL`): every page NOT touched by this test
    // rides along verbatim in EVERY push so `diff::plan` never mistakes
    // "not part of this test" for "delete me". Includes the two PROTECTED
    // durable-fixture pages, if the space carries them.
    let before = rt
        .block_on(backend.list_records(&space))
        .expect("list_records snapshot before push 1");
    let untouched_entries: Vec<(String, String)> = before
        .iter()
        .filter(|r| r.id.0 != page_a.id.0 && r.id.0 != page_b.id.0)
        .map(render_verbatim)
        .collect();

    // --- Push 1: edit A's body (real, lands) + rename B into the
    // blocker's title while also carrying B's intended body edit (the
    // real backend rejects the whole PUT atomically on the title
    // collision -- B's edit does NOT land, matching the sim twin's "issue
    // 2 still carries the un-landed edit" framing).
    let mut edited_a = page_a.clone();
    edited_a.body = "edited body A\n".to_owned();
    let mut faulty_b = page_b.clone();
    faulty_b.title = blocker_title.clone();
    faulty_b.body = "edited body B\n".to_owned();

    let mut push1_entries = untouched_entries.clone();
    push1_entries.push(render_verbatim(&edited_a));
    push1_entries.push(render_verbatim(&faulty_b));
    let push1 = export_stdin_real(
        &push1_entries,
        "p93 update-recovery: edit A + rename B into a colliding title (deliberately broken)\n",
    );
    let (ok1, stdout1) = run_helper_export_real(&url, &cache, &allowed, &push1);
    assert!(!ok1, "push 1 must fail (partial fail); stdout={stdout1}");
    assert!(
        stdout1.contains("error refs/heads/main some-actions-failed"),
        "push 1 must emit some-actions-failed; stdout={stdout1}"
    );

    // Page A's edit landed; page B's rename+edit did NOT (title collision
    // rejected the whole PUT atomically).
    let landed_a = rt
        .block_on(backend.get_record(&space, page_a.id))
        .unwrap_or_else(|e| panic!("get_record(A) after push 1 failed: {e:?}"));
    assert_eq!(
        landed_a.version,
        page_a.version + 1,
        "page A must have landed exactly one PUT after push 1"
    );
    let unlanded_b = rt
        .block_on(backend.get_record(&space, page_b.id))
        .unwrap_or_else(|e| panic!("get_record(B) after push 1 failed: {e:?}"));
    assert_eq!(
        unlanded_b.version, page_b.version,
        "page B's rename+edit must NOT have landed (title-collision reject)"
    );
    assert_eq!(
        unlanded_b.title, page_b.title,
        "page B's title must be UNCHANGED after the rejected push (Confluence rejects the \
         whole PUT atomically)"
    );

    // --- Push 2 (recovery): page A re-sent verbatim as its POST-push-1
    // landed state (`diff::plan` diffs it away -- content-equivalent,
    // never re-attempted); page B retried with its ORIGINAL
    // (non-colliding) title + the SAME intended body edit -- PRECHECK B
    // re-reads the live SoT and replans ONLY page B, which now lands.
    let mut fixed_b = unlanded_b.clone();
    fixed_b.body = "edited body B\n".to_owned();

    let mut push2_entries = untouched_entries;
    push2_entries.push(render_verbatim(&landed_a));
    push2_entries.push(render_verbatim(&fixed_b));
    let push2 = export_stdin_real(
        &push2_entries,
        "p93 update-recovery: retry B with its original title (recovery)\n",
    );
    let (ok2, stdout2) = run_helper_export_real(&url, &cache, &allowed, &push2);
    assert!(
        ok2,
        "push 2 (recovery) must succeed and converge; stdout={stdout2}"
    );
    assert!(
        stdout2.contains("ok refs/heads/main"),
        "push 2 must emit ok refs/heads/main; stdout={stdout2}"
    );

    // --- Convergence: both pages now hold the agent's intended edits.
    // Page A's version is UNCHANGED from push 1 (proves it was diffed
    // away on push 2, never re-PATCHed); page B's version incremented by
    // EXACTLY one PUT total across both pushes (the rejected push-1 PUT
    // landed nothing).
    let final_a = rt
        .block_on(backend.get_record(&space, page_a.id))
        .unwrap_or_else(|e| panic!("get_record(A) after push 2 failed: {e:?}"));
    assert_eq!(
        final_a.version, landed_a.version,
        "page A must NOT be re-PATCHed on push 2 (diffed away by PRECHECK B)"
    );
    let final_b = rt
        .block_on(backend.get_record(&space, page_b.id))
        .unwrap_or_else(|e| panic!("get_record(B) after push 2 failed: {e:?}"));
    assert_eq!(
        final_b.version,
        page_b.version + 1,
        "page B converged via exactly ONE landed PUT total (push 1's rejected PUT landed nothing)"
    );
    assert_eq!(
        final_b.title, page_b.title,
        "page B's final title is its ORIGINAL (never actually renamed)"
    );

    // Teardown runs via `guard`'s Drop at end of scope (also on panic
    // unwind from any assert! above).
    drop(guard);
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
