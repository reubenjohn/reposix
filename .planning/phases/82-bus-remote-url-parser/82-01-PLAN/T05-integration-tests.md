← [back to index](./index.md) · phase 82 plan 01

## Task 82-01-T05 — 4 integration tests (bus_url, bus_capabilities, bus_precheck_a, bus_precheck_b)

<read_first>
- `crates/reposix-remote/tests/perf_l1.rs` (P81 wiremock fixture
  donor pattern; ~250-300 lines including helpers).
- `crates/reposix-remote/tests/mirror_refs.rs` (P80 helper-driver
  donor pattern: `drive_helper_export`, `render_with_overrides`,
  `sample_issue`, `one_file_export`).
- `scripts/dark-factory-test.sh` (file:// bare-repo fixture donor
  pattern — RESEARCH.md Test Fixture Strategy option (a)).
- `crates/reposix-remote/Cargo.toml` `[dev-dependencies]` —
  `wiremock`, `assert_cmd`, `tempfile` already present.
- `crates/reposix-cache/tests/common/mod.rs` — authoritative wiremock
  helper (`sample_issues`, `seed_mock`, `sim_backend`, `CacheDirGuard`).
  Cargo's test harness does NOT share `mod common;` across crates;
  step 5a-prime copies this file unconditionally to
  `crates/reposix-remote/tests/common.rs` BEFORE step 5d's import
  reaches `cargo check`. The copy is the literal first sub-step of T05.
- `crates/reposix-remote/src/main.rs::handle_export` (post-T04 state
  — confirm bus dispatch is wired and capability branching is in
  place).
- `.planning/phases/82-bus-remote-url-parser/82-RESEARCH.md`
  § Test Fixture Strategy (option a — two local bare repos for
  PRECHECK A; wiremock for PRECHECK B).
</read_first>

<action>
Four concerns: write four test files. Order: bus_url → bus_capabilities
→ bus_precheck_a → bus_precheck_b → cargo nextest + commit.

The four test files share ONE common helper module
(`crates/reposix-remote/tests/common.rs`). Step 5a-prime below copies
it unconditionally from `crates/reposix-cache/tests/common/mod.rs` as
the literal first sub-step (M3 hard-block from P81 plan-check —
cargo's test harness does NOT share `mod common;` across crates).

### 5a-prime. Copy `tests/common.rs` from `reposix-cache` (HARD-BLOCK)

Per P81 plan-check M3: cargo's test harness does NOT share `mod common;`
across crates. The wiremock helpers (`sample_issues`, `seed_mock`,
`sim_backend`, `CacheDirGuard`) live ONLY in
`crates/reposix-cache/tests/common/mod.rs`. Step 5d below imports
`common::{sample_issues, seed_mock, sim_backend, CacheDirGuard}` — if
this copy is skipped, `cargo nextest run --test bus_precheck_b` fails
to compile.

This step is UNCONDITIONAL — confirm-then-copy, do NOT short-circuit
on "maybe P81 already did this". P81's M3 is a documented gap (the
copy was scoped out of P81 plan-time) and v0.13.0 has not yet added
it to `reposix-remote/tests/`. Run the copy verbatim:

```bash
# Sanity: confirm the source exists and the destination does NOT.
test -f crates/reposix-cache/tests/common/mod.rs || {
    echo "FATAL: source common/mod.rs missing; cannot proceed with P82 T05"
    exit 1
}
if test -f crates/reposix-remote/tests/common.rs; then
    echo "WARN: crates/reposix-remote/tests/common.rs already exists; skipping copy"
else
    cp crates/reposix-cache/tests/common/mod.rs crates/reposix-remote/tests/common.rs
fi

cargo check -p reposix-remote --tests
```

The `cargo check -p reposix-remote --tests` MUST exit 0 — it confirms
(a) the copied file compiles in the new crate, (b) no `pub use` shape
broke the import graph, (c) `reposix-cache` and `reposix-core` are
already in `reposix-remote`'s `[dev-dependencies]` (verified during
P81). If any of these fail, fix BEFORE proceeding to step 5a.

```bash
git add crates/reposix-remote/tests/common.rs
git commit -m "$(cat <<'EOF'
test(remote): copy tests/common.rs from reposix-cache (P81 M3 gap)

Cargo's test harness does NOT share `mod common;` across crates. The
wiremock helpers (`sample_issues`, `seed_mock`, `sim_backend`,
`CacheDirGuard`) lived only in `crates/reposix-cache/tests/common/mod.rs`
post-P81; P82 T05's `tests/bus_precheck_b.rs` imports them via
`mod common; use common::{...};`. Without this copy the integration
test would fail to compile (P81 plan-check M3 hard-block, carried
into P82 T05).

Phase 82 / Plan 01 / Task 05 / step 5a-prime / DVCS-BUS-PRECHECK-02 (substrate).
EOF
)"
```

### 5a. `crates/reposix-remote/tests/bus_url.rs`

Author the new file. Tests the bus URL parser via `assert_cmd`-driven
helper invocation (the parser itself is also unit-tested in T02
inline; this file exercises the helper-end-to-end shape).

```rust
//! Integration tests for bus URL parser via the helper binary
//! (DVCS-BUS-URL-01).
//!
//! Asserts the helper's `parse remote url` failure path emits
//! verbatim error messages for the rejected forms; the success
//! path is tested by bus_capabilities.rs and bus_precheck_*.rs
//! which exercise the helper end-to-end.

#![allow(clippy::missing_panics_doc)]

use assert_cmd::Command;

#[test]
fn parses_query_param_form_round_trip() {
    // POSITIVE capability-advertise assertion (HIGH-1 fix from P82
    // plan-check). The original negative assertion
    // `!stderr.contains("parse remote url")` passed if the helper
    // errored at ANY later stage with a different message — masking
    // bugs. We assert the helper REACHED the capabilities arm and
    // emitted the expected lines to stdout.
    //
    // Pattern matches `tests/bus_capabilities.rs::bus_url_omits_stateless_connect`
    // (and P80's `tests/stateless_connect.rs::capabilities_advertises_*`):
    // write `capabilities\n\n` on stdin → helper advertises
    // capabilities → next read_line returns Some("") (continue) →
    // EOF → helper exits cleanly with code 0.
    //
    // The bus URL points at port 9 (closed) but `instantiate_sim` is
    // a no-network constructor (`crates/reposix-remote/src/backend_dispatch.rs:228-232`),
    // so the helper reaches the dispatch loop and the capabilities
    // arm fires regardless of SoT availability.
    let out = Command::cargo_bin("git-remote-reposix")
        .expect("cargo bin")
        .args(["origin", "reposix::http://127.0.0.1:9/projects/demo?mirror=file:///tmp/m.git"])
        .write_stdin("capabilities\n\n")
        .output()
        .expect("run helper");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stdout.contains("import") && stdout.contains("export"),
        "expected helper to advertise capabilities; stdout={stdout} stderr={stderr}"
    );
}

#[test]
fn rejects_plus_delimited_bus_url() {
    let out = Command::cargo_bin("git-remote-reposix")
        .expect("cargo bin")
        .args(["origin", "reposix::http://127.0.0.1:9/projects/demo+file:///tmp/m.git"])
        .write_stdin("\n")
        .output()
        .expect("run helper");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("`+`-delimited") && stderr.contains("?mirror="),
        "expected stderr to reject `+` form and suggest `?mirror=`; got: {stderr}"
    );
    assert!(!out.status.success(), "expected helper to exit non-zero");
}

#[test]
fn rejects_unknown_query_param() {
    let out = Command::cargo_bin("git-remote-reposix")
        .expect("cargo bin")
        .args(["origin", "reposix::http://127.0.0.1:9/projects/demo?priority=high"])
        .write_stdin("\n")
        .output()
        .expect("run helper");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("unknown query parameter") && stderr.contains("priority"),
        "expected stderr to name the unknown key; got: {stderr}"
    );
    assert!(!out.status.success());
}
```

### 5b. `crates/reposix-remote/tests/bus_capabilities.rs`

```rust
//! Integration test: bus URL omits `stateless-connect` from
//! capabilities (DVCS-BUS-FETCH-01 / Q3.4).

#![allow(clippy::missing_panics_doc)]

use assert_cmd::Command;

#[test]
fn bus_url_omits_stateless_connect() {
    // Bus URL — `stateless-connect` MUST be absent.
    let out = Command::cargo_bin("git-remote-reposix")
        .expect("cargo bin")
        .args(["origin", "reposix::http://127.0.0.1:9/projects/demo?mirror=file:///tmp/m.git"])
        .write_stdin("capabilities\n\n")
        .output()
        .expect("run helper");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("import"), "expected `import`; got: {stdout}");
    assert!(stdout.contains("export"), "expected `export`; got: {stdout}");
    assert!(stdout.contains("refspec refs/heads/*:refs/reposix/*"), "expected `refspec`; got: {stdout}");
    assert!(stdout.contains("object-format=sha1"), "expected `object-format=sha1`; got: {stdout}");
    assert!(
        !stdout.contains("stateless-connect"),
        "bus URL MUST NOT advertise stateless-connect (DVCS-BUS-FETCH-01); got: {stdout}"
    );
}

#[test]
fn single_backend_url_advertises_stateless_connect() {
    // Regression check: bare `reposix::<sot>` (no `?mirror=`) DOES
    // advertise `stateless-connect`. Without this guard, an off-by-
    // one in capability branching would silently break single-backend
    // fetch (DVCS-DARKFACTORY-* would catch it eventually but the
    // signal is much faster here).
    let out = Command::cargo_bin("git-remote-reposix")
        .expect("cargo bin")
        .args(["origin", "reposix::http://127.0.0.1:9/projects/demo"])
        .write_stdin("capabilities\n\n")
        .output()
        .expect("run helper");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("stateless-connect"),
        "single-backend URL MUST advertise stateless-connect; got: {stdout}"
    );
}
```

### 5c. `crates/reposix-remote/tests/bus_precheck_a.rs`

The fixture creates two bare repos (`tempfile::tempdir()` + `git init
--bare`); seeds one with a commit; the local working tree is mocked
via a third tempdir + `git init` + `git config remote.mirror.url
file:///tmp/.../mirror.git` + `git update-ref refs/remotes/mirror/main
<some-old-sha>`. The bus URL points at `file:///tmp/.../mirror.git`.
The helper's PRECHECK A sees the local ref is BEHIND the bare repo's
HEAD and rejects with `error refs/heads/main fetch first`.

```rust
//! PRECHECK A — mirror drift via git ls-remote (DVCS-BUS-PRECHECK-01).
//!
//! Fixture strategy: two local bare repos via tempfile + git init
//! --bare (RESEARCH.md § Test Fixture Strategy option (a) —
//! mirrors `scripts/dark-factory-test.sh` idiom). NO network. NO
//! SSH agent.

#![allow(clippy::missing_panics_doc)]

use std::path::Path;
use std::process::Command;

use assert_cmd::Command as AssertCommand;

/// Spawn `git` against a directory; assert success.
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

/// Build a fixture: bare mirror repo + working-tree shell with a
/// stale `refs/remotes/mirror/main` ref. Returns
/// `(working_tree_dir, mirror_bare_dir, mirror_url, drifted_sha,
/// stale_local_sha)`.
fn make_drifting_mirror_fixture() -> (tempfile::TempDir, tempfile::TempDir, String, String, String) {
    let mirror = tempfile::tempdir().expect("mirror tempdir");
    let wtree = tempfile::tempdir().expect("wtree tempdir");

    // Bare mirror — initial commit, then a divergent one.
    run_git_in(mirror.path(), &["init", "--bare", "."]);
    // Seed an initial commit by piping a tree+commit object through
    // git. Easier: use a non-bare scratch repo to author commits,
    // then push to the bare repo.
    let scratch = tempfile::tempdir().expect("scratch tempdir");
    run_git_in(scratch.path(), &["init", "."]);
    run_git_in(scratch.path(), &["config", "user.email", "p82@example"]);
    run_git_in(scratch.path(), &["config", "user.name", "P82 Test"]);
    std::fs::write(scratch.path().join("seed.txt"), "seed").unwrap();
    run_git_in(scratch.path(), &["add", "seed.txt"]);
    run_git_in(scratch.path(), &["commit", "-m", "seed"]);
    let stale_local_sha = run_git_in(scratch.path(), &["rev-parse", "HEAD"]);

    // Push initial state to mirror.
    let mirror_url = format!("file://{}", mirror.path().display());
    run_git_in(scratch.path(), &["remote", "add", "mirror", &mirror_url]);
    run_git_in(scratch.path(), &["push", "mirror", "HEAD:refs/heads/main"]);

    // Author a divergent commit and force-push to the mirror — this
    // is the "someone else pushed" scenario.
    std::fs::write(scratch.path().join("seed.txt"), "drift").unwrap();
    run_git_in(scratch.path(), &["add", "seed.txt"]);
    run_git_in(scratch.path(), &["commit", "-m", "drift"]);
    run_git_in(scratch.path(), &["push", "-f", "mirror", "HEAD:refs/heads/main"]);
    let drifted_sha = run_git_in(scratch.path(), &["rev-parse", "HEAD"]);

    // Build the working tree: init + add the mirror remote + write
    // a STALE refs/remotes/mirror/main pointing at the initial seed
    // SHA (pre-divergence).
    run_git_in(wtree.path(), &["init", "."]);
    run_git_in(wtree.path(), &["config", "user.email", "p82@example"]);
    run_git_in(wtree.path(), &["config", "user.name", "P82 Test"]);
    run_git_in(wtree.path(), &["remote", "add", "mirror", &mirror_url]);
    run_git_in(
        wtree.path(),
        &["update-ref", "refs/remotes/mirror/main", &stale_local_sha],
    );

    (wtree, mirror, mirror_url, drifted_sha, stale_local_sha)
}

#[test]
fn bus_precheck_a_emits_fetch_first_on_drift() {
    let (wtree, _mirror, mirror_url, drifted_sha, stale_local_sha) =
        make_drifting_mirror_fixture();
    let bus_url = format!(
        "reposix::http://127.0.0.1:9/projects/demo?mirror={}",
        mirror_url
    );

    let out = AssertCommand::cargo_bin("git-remote-reposix")
        .expect("cargo bin")
        .args(["origin", &bus_url])
        .write_stdin("capabilities\n\nexport\n\n")
        .current_dir(wtree.path())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .expect("run helper");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert!(
        stdout.contains("error refs/heads/main fetch first"),
        "expected fetch-first protocol error on stdout; got stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stderr.contains("your GH mirror has new commits") || stderr.contains("local refs/remotes/mirror/main"),
        "expected stderr to name the drift; got: {stderr}"
    );
    assert!(
        stderr.contains(&drifted_sha[..8]) || stderr.contains(&stale_local_sha[..8]),
        "expected stderr to cite SHA(s); got: {stderr}"
    );
}

#[test]
fn bus_precheck_a_passes_when_mirror_in_sync() {
    // Mirror in sync — local ref equals mirror HEAD. PRECHECK A
    // passes; PRECHECK B runs (no SoT to drift since mirror_url
    // points at a non-existent SoT, but the bus URL's SoT is also
    // non-running so PRECHECK B errors with backend-unreachable).
    // For this test we only assert PRECHECK A did NOT trip — i.e.,
    // the stderr does NOT contain "your GH mirror has new commits".
    // The helper still exits non-zero (PRECHECK B will fail or the
    // deferred-shipped error will fire), but we filter to the
    // PRECHECK A signal only.
    let (wtree, _mirror, mirror_url, drifted_sha, _) = make_drifting_mirror_fixture();
    // Sync the local ref to mirror's HEAD.
    run_git_in(
        wtree.path(),
        &["update-ref", "refs/remotes/mirror/main", &drifted_sha],
    );
    let bus_url = format!(
        "reposix::http://127.0.0.1:9/projects/demo?mirror={}",
        mirror_url
    );

    let out = AssertCommand::cargo_bin("git-remote-reposix")
        .expect("cargo bin")
        .args(["origin", &bus_url])
        .write_stdin("capabilities\n\nexport\n\n")
        .current_dir(wtree.path())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .expect("run helper");

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("your GH mirror has new commits"),
        "PRECHECK A should NOT trip when mirror is in sync; stderr: {stderr}"
    );
}

#[test]
fn bus_no_remote_configured_emits_q35_hint() {
    // Working tree with NO `remote.mirror.url` configured (we don't
    // call `git remote add` here). Bus URL points at some `file://`
    // path. STEP 0 finds zero matches → Q3.5 hint, exit before
    // PRECHECK A.
    let wtree = tempfile::tempdir().expect("wtree tempdir");
    run_git_in(wtree.path(), &["init", "."]);
    run_git_in(wtree.path(), &["config", "user.email", "p82@example"]);
    run_git_in(wtree.path(), &["config", "user.name", "P82 Test"]);

    let bus_url =
        "reposix::http://127.0.0.1:9/projects/demo?mirror=file:///nonexistent/m.git";

    let out = AssertCommand::cargo_bin("git-remote-reposix")
        .expect("cargo bin")
        .args(["origin", bus_url])
        .write_stdin("capabilities\n\nexport\n\n")
        .current_dir(wtree.path())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .expect("run helper");

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("configure the mirror remote first"),
        "expected Q3.5 hint; got: {stderr}"
    );
    assert!(
        stderr.contains("git remote add"),
        "expected Q3.5 hint to cite `git remote add`; got: {stderr}"
    );

    // NO auto-mutation: the working tree still has no `mirror`
    // remote configured.
    let out = Command::new("git")
        .args(["remote"])
        .current_dir(wtree.path())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .unwrap();
    let remotes = String::from_utf8_lossy(&out.stdout);
    assert!(
        !remotes.contains("mirror"),
        "helper auto-mutated git config — Q3.5 violated! remotes: {remotes}"
    );
}

#[test]
fn rejects_dash_prefixed_mirror_url() {
    // T-82-01: argument injection via `--upload-pack=evil`-style
    // mirror URL. `bus_handler` rejects BEFORE any shell-out.
    let wtree = tempfile::tempdir().expect("wtree tempdir");
    run_git_in(wtree.path(), &["init", "."]);

    let bus_url = "reposix::http://127.0.0.1:9/projects/demo?mirror=--upload-pack=evil";

    let out = AssertCommand::cargo_bin("git-remote-reposix")
        .expect("cargo bin")
        .args(["origin", bus_url])
        .write_stdin("capabilities\n\nexport\n\n")
        .current_dir(wtree.path())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .expect("run helper");

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("mirror URL cannot start with `-`"),
        "expected `-`-prefix rejection; got: {stderr}"
    );
}
```

### 5d. `crates/reposix-remote/tests/bus_precheck_b.rs`

The fixture uses wiremock per P81's pattern (`tests/perf_l1.rs` is the
authoritative donor). PRECHECK B fires when wiremock returns a non-empty
delta on `?since=`; otherwise the helper proceeds past PRECHECK B and
emits the deferred-shipped error from D-02 / Q-B. The file:// mirror
fixture is reused inline (cargo's test harness does NOT share modules
across files, so the helper-driver pattern is duplicated from
`bus_precheck_a.rs`).

```rust
//! PRECHECK B — SoT drift via list_changed_since (DVCS-BUS-PRECHECK-02).
//!
//! Fixture strategy: wiremock SoT (P81 donor pattern from
//! `tests/perf_l1.rs`) + synced file:// mirror (P82 donor pattern
//! from `tests/bus_precheck_a.rs::make_drifting_mirror_fixture`).
//! Drifted: wiremock returns non-empty `?since=` response → helper
//! emits `error refs/heads/main fetch first`. Stable: wiremock
//! returns `[]` on `?since=` → helper passes PRECHECK B and emits
//! the D-02 deferred-shipped error.

#![allow(clippy::missing_panics_doc)]

use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use assert_cmd::Command as AssertCommand;
use reposix_cache::Cache;
use reposix_core::backend::sim::SimBackend;
use reposix_core::BackendConnector;
use serde_json::json;
use wiremock::matchers::{method, path_regex};
use wiremock::{Match, Mock, MockServer, Request, ResponseTemplate};

mod common;
use common::{sample_issues, seed_mock, sim_backend, CacheDirGuard};

/// Custom matcher (verbatim from `tests/perf_l1.rs:52-57`): matches
/// requests that DO have a `since` query param. wiremock 0.6's
/// `query_param(K, V)` is byte-exact; there is no `query_param_exists`
/// or wildcard-value form. A custom `Match` impl is the canonical idiom.
struct HasSinceQueryParam;
impl Match for HasSinceQueryParam {
    fn matches(&self, req: &Request) -> bool {
        req.url.query_pairs().any(|(k, _)| k == "since")
    }
}

/// Custom matcher (verbatim from `tests/perf_l1.rs:40-45`): matches
/// requests with NO `since` query param (the unconditional list_records
/// path, used by the warm_cache seed).
struct NoSinceQueryParam;
impl Match for NoSinceQueryParam {
    fn matches(&self, req: &Request) -> bool {
        req.url.query_pairs().all(|(k, _)| k != "since")
    }
}

/// Spawn `git` against a directory; assert success. Mirrors the helper
/// from `bus_precheck_a.rs` verbatim.
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

/// Build a SYNCED file:// mirror fixture: bare mirror with one commit;
/// working tree with `refs/remotes/mirror/main` pointing at that same
/// commit (PRECHECK A passes). Returns
/// `(working_tree_dir, mirror_bare_dir, mirror_url)`.
fn make_synced_mirror_fixture() -> (tempfile::TempDir, tempfile::TempDir, String) {
    let mirror = tempfile::tempdir().expect("mirror tempdir");
    let wtree = tempfile::tempdir().expect("wtree tempdir");
    let scratch = tempfile::tempdir().expect("scratch tempdir");

    run_git_in(mirror.path(), &["init", "--bare", "."]);
    run_git_in(scratch.path(), &["init", "."]);
    run_git_in(scratch.path(), &["config", "user.email", "p82@example"]);
    run_git_in(scratch.path(), &["config", "user.name", "P82 Test"]);
    std::fs::write(scratch.path().join("seed.txt"), "seed").unwrap();
    run_git_in(scratch.path(), &["add", "seed.txt"]);
    run_git_in(scratch.path(), &["commit", "-m", "seed"]);
    let synced_sha = run_git_in(scratch.path(), &["rev-parse", "HEAD"]);

    let mirror_url = format!("file://{}", mirror.path().display());
    run_git_in(scratch.path(), &["remote", "add", "mirror", &mirror_url]);
    run_git_in(scratch.path(), &["push", "mirror", "HEAD:refs/heads/main"]);

    // Working tree: init + add the mirror remote + write
    // refs/remotes/mirror/main pointing at the SAME commit (synced).
    run_git_in(wtree.path(), &["init", "."]);
    run_git_in(wtree.path(), &["config", "user.email", "p82@example"]);
    run_git_in(wtree.path(), &["config", "user.name", "P82 Test"]);
    run_git_in(wtree.path(), &["remote", "add", "mirror", &mirror_url]);
    run_git_in(
        wtree.path(),
        &["update-ref", "refs/remotes/mirror/main", &synced_sha],
    );

    (wtree, mirror, mirror_url)
}

#[tokio::test(flavor = "multi_thread")]
async fn bus_precheck_b_emits_fetch_first_on_sot_drift() {
    let server = MockServer::start().await;
    let project = "demo";
    let issues = sample_issues(project, 3);

    // Setup-phase mocks (default priority 5): seed list + per-id GETs
    // so warm_cache populates the last_fetched_at cursor.
    seed_mock(&server, project, &issues).await;

    // Per-test cache dir.
    let cache_root = tempfile::tempdir().expect("cache_root");
    let _env = CacheDirGuard::new(cache_root.path());
    let backend: Arc<dyn BackendConnector> = sim_backend(&server);
    let cache = Cache::open(backend, "sim", project).expect("Cache::open");
    cache.sync().await.expect("seed sync (warm cache cursor)");
    drop(cache);

    // ASSERTION-PHASE mock (priority=1, beats setup): wiremock returns
    // a non-empty `?since=` response — PRECHECK B sees Drifted.
    Mock::given(method("GET"))
        .and(path_regex(format!(r"^/projects/{project}/issues$")))
        .and(HasSinceQueryParam)
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {"id": 5, "title": "drift", "status": "open",
             "assignee": null, "labels": [],
             "created_at": "2026-04-13T00:00:00Z",
             "updated_at": "2026-05-01T00:00:00Z",
             "version": 2, "body": "drift body"}
        ])))
        .with_priority(1)
        .mount(&server)
        .await;

    // Per-id GET for the drifted record (in case the helper requests
    // it during precheck — defensive, currently unused since PRECHECK B
    // bails on count alone).
    Mock::given(method("GET"))
        .and(path_regex(format!(r"^/projects/{project}/issues/5$")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 5, "title": "drift", "status": "open",
            "assignee": null, "labels": [],
            "created_at": "2026-04-13T00:00:00Z",
            "updated_at": "2026-05-01T00:00:00Z",
            "version": 2, "body": "drift body"
        })))
        .with_priority(1)
        .mount(&server)
        .await;

    // Build the synced file:// mirror fixture (PRECHECK A passes).
    let (wtree, _mirror_bare, mirror_url) = make_synced_mirror_fixture();

    // Bus URL: wiremock SoT + file:// mirror.
    let bus_url = format!(
        "reposix::{}/projects/{project}?mirror={}",
        server.uri(),
        mirror_url
    );

    // Drive the helper. write_stdin uses the same shape as
    // `bus_precheck_a.rs`: capabilities + export verb. The helper
    // reaches PRECHECK B (PRECHECK A passes since mirror is synced),
    // sees Drifted, emits the fetch-first reject before reading the
    // export stream.
    let cache_path = cache_root.path().to_path_buf();
    let out = AssertCommand::cargo_bin("git-remote-reposix")
        .expect("cargo bin")
        .args(["origin", &bus_url])
        .write_stdin("capabilities\n\nexport\n\n")
        .current_dir(wtree.path())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("REPOSIX_CACHE_DIR", &cache_path)
        .output()
        .expect("run helper");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    // Assertion 1: fetch-first protocol error on stdout.
    assert!(
        stdout.contains("error refs/heads/main fetch first"),
        "expected fetch-first protocol error on stdout; got stdout={stdout}, stderr={stderr}"
    );

    // Assertion 2: stderr names SoT drift + mentions `git pull --rebase`
    // + (when populated by P80) cites refs/mirrors/<sot>-synced-at.
    // For this test the synced-at ref is NOT populated (no prior P80
    // push happened), so we only assert the always-on hint substrings.
    assert!(
        stderr.contains("git pull --rebase"),
        "expected stderr to suggest `git pull --rebase`; got: {stderr}"
    );
    assert!(
        stderr.contains("PRECHECK B") || stderr.contains("change(s) since"),
        "expected stderr to name SoT drift / PRECHECK B; got: {stderr}"
    );

    // Assertion 3: helper exited non-zero (precheck reject).
    assert!(!out.status.success(), "expected helper to exit non-zero");
}

#[tokio::test(flavor = "multi_thread")]
async fn bus_precheck_b_passes_when_sot_stable() {
    let server = MockServer::start().await;
    let project = "demo";
    let issues = sample_issues(project, 3);

    seed_mock(&server, project, &issues).await;

    let cache_root = tempfile::tempdir().expect("cache_root");
    let _env = CacheDirGuard::new(cache_root.path());
    let backend: Arc<dyn BackendConnector> = sim_backend(&server);
    let cache = Cache::open(backend, "sim", project).expect("Cache::open");
    cache.sync().await.expect("seed sync (warm cache cursor)");
    drop(cache);

    // ASSERTION-PHASE mock (priority=1): wiremock returns EMPTY on
    // `?since=` — PRECHECK B sees Stable, helper proceeds to D-02 stub.
    Mock::given(method("GET"))
        .and(path_regex(format!(r"^/projects/{project}/issues$")))
        .and(HasSinceQueryParam)
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .with_priority(1)
        .mount(&server)
        .await;

    // PATCH backstop: should NEVER fire (P82's deferred-shipped exit
    // is BEFORE any write fan-out). expect(0) tightens the assertion.
    Mock::given(method("PATCH"))
        .and(path_regex(format!(r"^/projects/{project}/issues/\d+$")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": 1, "version": 2})))
        .expect(0)
        .with_priority(1)
        .mount(&server)
        .await;

    let (wtree, _mirror_bare, mirror_url) = make_synced_mirror_fixture();
    let bus_url = format!(
        "reposix::{}/projects/{project}?mirror={}",
        server.uri(),
        mirror_url
    );

    let cache_path = cache_root.path().to_path_buf();
    let out = AssertCommand::cargo_bin("git-remote-reposix")
        .expect("cargo bin")
        .args(["origin", &bus_url])
        .write_stdin("capabilities\n\nexport\n\n")
        .current_dir(wtree.path())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("REPOSIX_CACHE_DIR", &cache_path)
        .output()
        .expect("run helper");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    // Assertion 1: helper emitted the D-02 deferred-shipped error
    // (proves PRECHECK B passed AND execution reached the write-fan-out
    // emit point, which is the deferred-shipped stub in P82).
    assert!(
        stdout.contains("error refs/heads/main bus-write-not-yet-shipped"),
        "expected D-02 deferred-shipped protocol error; got stdout={stdout}, stderr={stderr}"
    );

    // Assertion 2: stderr cites the deferred-shipped diagnostic
    // (verbatim from D-02: "bus write fan-out (DVCS-BUS-WRITE-01..06)
    // is not yet shipped — lands in P83").
    assert!(
        stderr.contains("bus write fan-out") && stderr.contains("P83"),
        "expected D-02 stderr diagnostic; got: {stderr}"
    );

    // Assertion 3: NO fetch-first signal (PRECHECK B did NOT trip).
    assert!(
        !stdout.contains("fetch first"),
        "PRECHECK B incorrectly tripped on stable SoT; stdout={stdout}, stderr={stderr}"
    );

    // Assertion 4: helper exited non-zero (P82's deferred-shipped is
    // still a reject — P83 will replace this with `ok refs/heads/main`).
    assert!(!out.status.success(), "expected helper to exit non-zero");

    // Assertion 5 (implicit): wiremock's Drop panics if PATCH
    // expect(0) was violated — confirms ZERO write fan-out.
}
```

**Implementation note for T05.** Both tests above import the
`HasSinceQueryParam` matcher inline (verbatim from
`tests/perf_l1.rs:52-57`) rather than reusing it via a shared helper —
cargo's test harness does NOT share modules across test files. The
`make_synced_mirror_fixture` helper is the synced cousin of
`bus_precheck_a.rs::make_drifting_mirror_fixture`; the two helpers
are intentionally duplicated rather than factored into `tests/common.rs`
because `common.rs` is reserved for the wiremock helpers
(`sample_issues`, `seed_mock`, `sim_backend`, `CacheDirGuard`) per
the P81 M3 hard-block (step 5a-prime).

The `seed_mock` + `cache.sync().await` warm path populates the cache's
`last_fetched_at` cursor; without it, `precheck_sot_drift_any` would
take the first-push (no-cursor) path and return `Stable` regardless
of wiremock's `?since=` response — silently masking PRECHECK B's
drift detection. The `_env = CacheDirGuard::new(...)` guard MUST
remain in scope for the entire `Cache::open` + `sync` span (the
helper subprocess uses `.env("REPOSIX_CACHE_DIR", ...)` which is
child-local; the guard is for the in-process `Cache::open`).


### 5e. Build sweep

Step 5a-prime already copied `tests/common.rs` and committed it; no
further work on the helper module is needed here. Run the full T05
build sweep serially:

```bash
cargo check -p reposix-remote --tests
cargo nextest run -p reposix-remote --test bus_url
cargo nextest run -p reposix-remote --test bus_capabilities
cargo nextest run -p reposix-remote --test bus_precheck_a
cargo nextest run -p reposix-remote --test bus_precheck_b
cargo nextest run -p reposix-remote      # full crate sweep
```

### 5f. Stage and commit

```bash
git add crates/reposix-remote/tests/bus_url.rs \
        crates/reposix-remote/tests/bus_capabilities.rs \
        crates/reposix-remote/tests/bus_precheck_a.rs \
        crates/reposix-remote/tests/bus_precheck_b.rs
git commit -m "$(cat <<'EOF'
test(remote): 4 integration tests — bus_url + bus_capabilities + bus_precheck_a + bus_precheck_b (DVCS-BUS-URL-01..02-PRECHECK + DVCS-BUS-FETCH-01)

- crates/reposix-remote/tests/bus_url.rs (new) — 3 tests: parses_query_param_form_round_trip + rejects_plus_delimited_bus_url + rejects_unknown_query_param (helper-end-to-end via assert_cmd)
- crates/reposix-remote/tests/bus_capabilities.rs (new) — 2 tests: bus_url_omits_stateless_connect (DVCS-BUS-FETCH-01 / Q3.4) + single_backend_url_advertises_stateless_connect (regression check)
- crates/reposix-remote/tests/bus_precheck_a.rs (new) — 4 tests: bus_precheck_a_emits_fetch_first_on_drift + bus_precheck_a_passes_when_mirror_in_sync + bus_no_remote_configured_emits_q35_hint + rejects_dash_prefixed_mirror_url (T-82-01); fixture uses two local bare repos + tempfile (file:// — RESEARCH.md Test Fixture Strategy option (a))
- crates/reposix-remote/tests/bus_precheck_b.rs (new) — 2 tests: bus_precheck_b_emits_fetch_first_on_sot_drift + bus_precheck_b_passes_when_sot_stable; fixture uses wiremock SoT + synced file:// mirror; asserts ZERO PATCH/PUT calls hit wiremock
- crates/reposix-remote/tests/common.rs (committed in step 5a-prime above) provides the wiremock helpers (`sample_issues`, `seed_mock`, `sim_backend`, `CacheDirGuard`) consumed by `bus_precheck_b.rs::mod common;`

All four test files exercise the helper end-to-end via `assert_cmd::Command::cargo_bin("git-remote-reposix")`. Stdin is piped via write_stdin; stdout/stderr are read after the helper exits. NO subprocess fork/exec primitives bypassed.

Phase 82 / Plan 01 / Task 05 / DVCS-BUS-URL-01 + DVCS-BUS-PRECHECK-01 + DVCS-BUS-PRECHECK-02 + DVCS-BUS-FETCH-01.
EOF
)"
```
</action>

<verify>
  <automated>cargo nextest run -p reposix-remote --test bus_url && cargo nextest run -p reposix-remote --test bus_capabilities && cargo nextest run -p reposix-remote --test bus_precheck_a && cargo nextest run -p reposix-remote --test bus_precheck_b</automated>
</verify>

<done>
- `crates/reposix-remote/tests/bus_url.rs` exists with at least 3
  tests (positive parse + reject `+` + reject unknown key);
  `cargo nextest run -p reposix-remote --test bus_url` exits 0.
- `crates/reposix-remote/tests/bus_capabilities.rs` exists with at
  least 2 tests (bus URL omits `stateless-connect`; single-backend
  retains it); `cargo nextest run -p reposix-remote --test
  bus_capabilities` exits 0.
- `crates/reposix-remote/tests/bus_precheck_a.rs` exists with at
  least 4 tests (mirror-drift fetch-first + mirror-in-sync passes +
  no-remote Q3.5 hint + `-`-prefix reject); `cargo nextest run -p
  reposix-remote --test bus_precheck_a` exits 0.
- `crates/reposix-remote/tests/bus_precheck_b.rs` exists with at
  least 2 tests (SoT-drift fetch-first + SoT-stable proceeds to
  deferred-shipped error); `cargo nextest run -p reposix-remote
  --test bus_precheck_b` exits 0.
- The helper exits non-zero in all reject paths; stdout / stderr
  match the expected verbatim strings (`error refs/heads/main fetch
  first`, `your GH mirror has new commits`, `configure the mirror
  remote first`, `mirror URL cannot start with`-`, `bus write
  fan-out (DVCS-BUS-WRITE-01..06) is not yet shipped`).
- ZERO PATCH/PUT calls hit wiremock in `bus_precheck_b.rs` reject
  cases (assertable via wiremock's `Mock::expect(0)`).
- `cargo nextest run -p reposix-remote` passes (full crate sweep —
  no regression on existing P79/P80/P81 tests).
- Cargo serialized: T05 cargo invocations run only after T04's
  commit has landed; per-crate fallback used.
</done>

---

