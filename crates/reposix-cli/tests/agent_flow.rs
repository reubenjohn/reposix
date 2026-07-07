//! Phase 35 Plan 02 dark-factory regression tests.
//!
//! Proves the architecture's central thesis ("pure git, zero in-context
//! learning") with three integration scenarios that simulate an
//! stderr-reading agent's recovery moves.
//!
//! All three are `#[ignore]`-gated because they shell out to a real
//! `git` binary plus the workspace `reposix-sim`/`reposix`/`git-remote-reposix`
//! binaries. CI's integration job runs them via
//! `cargo test -p reposix-cli --test agent_flow -- --ignored` against
//! the simulator. The sister file `agent_flow_real.rs` (Plan 35-03)
//! gates real-backend exercise on env-var presence.
//!
//! Scenarios:
//! 1. `dark_factory_sim_happy_path` — `reposix init sim::demo` produces
//!    a partial-clone working tree with the right git config.
//! 2. `dark_factory_blob_limit_teaching_string_present` — the literal
//!    `git sparse-checkout` substring is committed in
//!    `crates/reposix-remote/src/stateless_connect.rs`. Regression-protects
//!    the Phase 34 dark-factory teaching mechanism.
//! 3. `dark_factory_conflict_teaching_string_present` — the literal
//!    `git pull --rebase` substring is committed in
//!    `crates/reposix-remote/src/main.rs`. Regression-protects the
//!    Phase 34 conflict-rebase teaching mechanism.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

/// Resolve the workspace root from `CARGO_MANIFEST_DIR` (which points at
/// `crates/reposix-cli`).
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root from CARGO_MANIFEST_DIR")
        .to_path_buf()
}

/// Path to a built workspace binary in `target/debug/`. The cargo test
/// harness compiles dependencies but not sibling binaries; tests using
/// these binaries depend on a prior `cargo build --workspace --bins`
/// (or a default `cargo test --workspace --no-run` which builds them).
fn target_bin(name: &str) -> PathBuf {
    workspace_root().join("target").join("debug").join(name)
}

/// SIGTERM-then-wait teardown.
fn kill_child(child: &mut std::process::Child) {
    let _ = child.kill();
    let _ = child.wait();
}

/// Scenario 1 — happy path.
///
/// `reposix init sim::demo <path>` produces a directory where:
/// - `.git/` exists (real git working tree).
/// - `extensions.partialClone == origin`.
/// - `remote.origin.promisor == true`.
/// - `remote.origin.partialclonefilter == blob:none`.
/// - `remote.origin.url` is a `reposix::http://` URL.
#[test]
#[ignore = "spawns reposix-sim child; requires `cargo build --workspace --bins` first"]
fn dark_factory_sim_happy_path() {
    let bind = "127.0.0.1:7779";
    let mut sim = spawn_seeded_sim(bind);

    let tmp = tempfile::tempdir().expect("tempdir");
    let cache_tmp = tempfile::tempdir().expect("cache tempdir");
    let repo = tmp.path().join("repo");

    let reposix = target_bin("reposix");
    // Point init at THIS test's sim via REPOSIX_SIM_ORIGIN so the initial
    // `git fetch` genuinely reaches a live backend. Since v0.13.1 B4,
    // `reposix init` exits NON-ZERO when the initial fetch cannot complete
    // (an unreachable backend is a hard error, not a warning), so the old
    // "best-effort fetch against the wrong port" assumption no longer holds —
    // the sim must be reachable for init to succeed. `REPOSIX_CACHE_DIR`
    // isolates the cache; `PATH` lets the fetch's spawned `git` discover the
    // real `git-remote-reposix` helper.
    let out = Command::new(&reposix)
        .args(["init", "sim::demo", repo.to_str().unwrap()])
        .env("REPOSIX_SIM_ORIGIN", format!("http://{bind}"))
        .env("REPOSIX_CACHE_DIR", cache_tmp.path())
        .env("PATH", path_with_target_debug())
        .output()
        .expect("run reposix init");
    assert!(
        out.status.success(),
        "reposix init failed: stdout={:?} stderr={:?}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let target_url = format!("reposix::http://{bind}/projects/demo");
    let cfg = Command::new("git")
        .args(["-C", repo.to_str().unwrap(), "config", "remote.origin.url"])
        .output()
        .expect("git config");
    let url = String::from_utf8_lossy(&cfg.stdout).trim().to_string();
    assert!(
        url.starts_with("reposix::http://"),
        "url should start with reposix:: prefix, got {url}"
    );
    // Re-point so a follow-up fetch (if added) hits our sim.
    let _ = Command::new("git")
        .args([
            "-C",
            repo.to_str().unwrap(),
            "config",
            "remote.origin.url",
            &target_url,
        ])
        .status();

    for (key, expected) in [
        ("extensions.partialClone", "origin"),
        ("remote.origin.promisor", "true"),
        ("remote.origin.partialclonefilter", "blob:none"),
        // Fetch refspec that populates refs/reposix/origin/* so a real
        // `git checkout refs/reposix/origin/main` resolves after init.
        ("remote.origin.fetch", "+refs/heads/*:refs/reposix/origin/*"),
    ] {
        let v = Command::new("git")
            .args(["-C", repo.to_str().unwrap(), "config", key])
            .output()
            .expect("git config");
        let got = String::from_utf8_lossy(&v.stdout).trim().to_string();
        assert_eq!(
            got, expected,
            "git config {key}: expected {expected}, got {got}"
        );
    }

    kill_child(&mut sim);
}

/// Scenario 2 — blob-limit teaching string is present in the helper.
///
/// The Phase 34 dark-factory mechanism ships an error message containing
/// the literal `git sparse-checkout` so a stderr-reading agent learns the
/// exact recovery command. Regression test: the literal must remain in
/// `crates/reposix-remote/src/stateless_connect.rs`.
#[test]
fn dark_factory_blob_limit_teaching_string_present() {
    let path = workspace_root()
        .join("crates")
        .join("reposix-remote")
        .join("src")
        .join("stateless_connect.rs");
    let src = std::fs::read_to_string(&path).expect("read stateless_connect.rs");
    assert!(
        src.contains("git sparse-checkout"),
        "BLOB_LIMIT_EXCEEDED_FMT must contain `git sparse-checkout` to teach the agent the recovery move"
    );
    assert!(
        src.contains("BLOB_LIMIT_EXCEEDED_FMT"),
        "the named constant should still exist; if you renamed it, update Phase 34/35 docs and 35-02 tests"
    );
}

/// Scenario 3 — conflict-rebase teaching string is present in the helper.
///
/// The Phase 34 push-conflict path emits a stderr diagnostic containing
/// `git pull --rebase` so an agent that observes `! [remote rejected]`
/// learns the exact recovery command. Regression test: the literal must
/// remain SOMEWHERE in `crates/reposix-remote/src/`.
///
/// P83-01 T02 lifted the conflict-detection write loop into `write_loop.rs`,
/// so the teaching string lives there (single-backend path) AND in
/// `bus_handler.rs` (bus-remote hint). Either source counts; if all files
/// lose it, the dark-factory contract breaks.
#[test]
fn dark_factory_conflict_teaching_string_present() {
    let remote_src = workspace_root()
        .join("crates")
        .join("reposix-remote")
        .join("src");
    let candidates = ["main.rs", "write_loop.rs", "bus_handler.rs"];
    let mut any_has_rebase = false;
    let mut any_has_fetch_first = false;
    for name in &candidates {
        let path = remote_src.join(name);
        if let Ok(src) = std::fs::read_to_string(&path) {
            if src.contains("git pull --rebase") {
                any_has_rebase = true;
            }
            if src.contains("error refs/heads/main fetch first") {
                any_has_fetch_first = true;
            }
        }
    }
    assert!(
        any_has_rebase,
        "conflict path must teach `git pull --rebase` so a stderr-reading agent recovers; checked {candidates:?}"
    );
    assert!(
        any_has_fetch_first,
        "canned `fetch first` status must be byte-identical to git's expected reject token; checked {candidates:?}"
    );
}

/// Spawn an ephemeral (in-memory DB, isolated) sim seeded from the committed
/// `crates/reposix-sim/fixtures/seed.json` fixture, so the working tree has
/// real records (project `demo`, issues 1..=6) to check out. Mirrors
/// [`spawn_sim`]'s readiness poll; the plain [`spawn_sim`] seeds 0 issues.
fn spawn_seeded_sim(bind: &str) -> std::process::Child {
    let bin = target_bin("reposix-sim");
    assert!(
        bin.exists(),
        "reposix-sim not built at {}; run `cargo build --workspace --bins` first",
        bin.display()
    );
    let seed = workspace_root()
        .join("crates")
        .join("reposix-sim")
        .join("fixtures")
        .join("seed.json");
    let mut cmd = Command::new(&bin);
    cmd.args(["--bind", bind, "--ephemeral", "--seed-file"])
        .arg(&seed)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::inherit());
    let mut child = cmd.spawn().expect("spawn seeded reposix-sim");
    let url = format!("http://{bind}/projects/demo/issues");
    let t0 = Instant::now();
    while t0.elapsed() < Duration::from_secs(5) {
        let out = Command::new("curl")
            .args(["-fsS", "-o", "/dev/null", "-m", "1", &url])
            .output();
        if matches!(out, Ok(o) if o.status.success()) {
            return child;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let _ = child.kill();
    let _ = child.wait();
    panic!("seeded sim did not become ready at {bind} within 5s");
}

/// Prepend `target/debug/` to a PATH string so a spawned `git` discovers the
/// real `git-remote-reposix` helper (git resolves `git-remote-<transport>`
/// from PATH). Returns the composed value for `.env("PATH", …)`.
fn path_with_target_debug() -> String {
    let dir = workspace_root().join("target").join("debug");
    let existing = std::env::var("PATH").unwrap_or_default();
    format!("{}:{}", dir.display(), existing)
}

/// v0.13.1 CHECKOUT-BREAK — the documented front door actually works.
///
/// End-to-end and fully leaf-isolated: its own sim on an isolated port
/// (reached via `REPOSIX_SIM_ORIGIN`), its own `REPOSIX_CACHE_DIR` tempdir,
/// and its own working-tree tempdir. It never runs `git`/`init` against the
/// shared repo. Regression-protects three breaks a prior lane reproduced
/// against a LIVE sim:
///
/// 1. `reposix init sim::demo` prints the VERIFIED-WORKING onboarding command
///    (`git checkout -B main refs/reposix/origin/main`), NOT the broken
///    pure-git `git checkout origin/main` (which fails "pathspec did not
///    match" because only `refs/reposix/origin/main` is populated, never
///    `refs/remotes/origin/main`).
/// 2. A second `git fetch --filter=blob:none origin` exits 0 — the spurious
///    git-128 `could not read ref refs/reposix/main` (helper advertised
///    `refs/heads/*:refs/reposix/*` while fast-import wrote
///    `refs/reposix/origin/main`) is closed by aligning the advertised
///    refspec to `refs/reposix/origin/*`.
/// 3. The recommended checkout resolves and `issues/1.md` materialises with
///    frontmatter `id: 1` — the pure-git payload is really there.
#[test]
#[ignore = "spawns reposix-sim child + shells out to git; requires `cargo build --workspace --bins` first"]
fn checkout_break_front_door_works_end_to_end() {
    let bind = "127.0.0.1:7801";
    let mut sim = spawn_seeded_sim(bind);

    let work_tmp = tempfile::tempdir().expect("work tempdir");
    let cache_tmp = tempfile::tempdir().expect("cache tempdir");
    let repo = work_tmp.path().join("repo");
    let repo_str = repo.to_str().expect("utf-8 repo path");
    let sim_origin = format!("http://{bind}");
    let reposix = target_bin("reposix");
    let path_env = path_with_target_debug();

    // `reposix init` honours REPOSIX_SIM_ORIGIN so the stored
    // remote.origin.url — and thus the helper the follow-up `git fetch`
    // spawns — targets THIS test's isolated sim, not the default :7878.
    let init = Command::new(&reposix)
        .args(["init", "sim::demo", repo_str])
        .env("REPOSIX_SIM_ORIGIN", &sim_origin)
        .env("REPOSIX_CACHE_DIR", cache_tmp.path())
        .env("PATH", &path_env)
        .output()
        .expect("run reposix init");
    let init_stdout = String::from_utf8_lossy(&init.stdout);
    let init_stderr = String::from_utf8_lossy(&init.stderr);
    assert!(
        init.status.success(),
        "init must succeed against the live sim; stdout={init_stdout:?} stderr={init_stderr:?}"
    );
    // Regression 1 — banner teaches the WORKING command, not the broken one.
    assert!(
        init_stdout.contains("git checkout -B main refs/reposix/origin/main"),
        "init banner must print the verified-working checkout; got:\n{init_stdout}"
    );
    assert!(
        !init_stdout.contains("git checkout origin/main"),
        "init banner must NOT print the broken pure-git `git checkout origin/main` \
         (fails 'pathspec did not match'); got:\n{init_stdout}"
    );

    // Regression 2 — a re-fetch exits 0 (no spurious git-128).
    let second = Command::new("git")
        .args(["-C", repo_str, "fetch", "--filter=blob:none", "origin"])
        .env("REPOSIX_CACHE_DIR", cache_tmp.path())
        .env("PATH", &path_env)
        .output()
        .expect("git fetch");
    assert!(
        second.status.success(),
        "second `git fetch` must exit 0 (git-128 advertised-refspec mismatch closed); stderr={:?}",
        String::from_utf8_lossy(&second.stderr)
    );

    // Regression 3 — the recommended checkout resolves and issue 1 materialises.
    let co = Command::new("git")
        .args([
            "-C",
            repo_str,
            "checkout",
            "-B",
            "main",
            "refs/reposix/origin/main",
        ])
        .env("REPOSIX_CACHE_DIR", cache_tmp.path())
        .env("PATH", &path_env)
        .output()
        .expect("git checkout");
    assert!(
        co.status.success(),
        "recommended checkout must resolve; stderr={:?}",
        String::from_utf8_lossy(&co.stderr)
    );
    let issue = std::fs::read_to_string(repo.join("issues").join("1.md"))
        .expect("issues/1.md must materialise after checkout");
    assert!(
        issue.contains("id: 1"),
        "issues/1.md must carry frontmatter `id: 1`; got:\n{issue}"
    );

    kill_child(&mut sim);
}
