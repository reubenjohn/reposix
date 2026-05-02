← [back to index](./index.md) · phase 82 plan 01

## Task 82-01-T05 (continued) — `bus_precheck_a.rs` (step 5c)

Continued from [T05 step 1](./T05-step-1.md) (common.rs + bus_url.rs + bus_capabilities.rs).

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

Continue to [T05 step 3](./T05-step-3.md) for `bus_precheck_b.rs` (5d), build sweep (5e), commit (5f), verify, and done.
