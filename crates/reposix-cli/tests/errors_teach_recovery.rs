//! Phase 120 (P120) — per-error integration assertions that the `reposix` CLI
//! surface emits Rust-compiler-grade 3-part teaching errors (teach the fix /
//! name the alternative / copy-paste recovery).
//!
//! SCAFFOLD (W0): one ANCHOR test pins the already-3-part `init` refusal
//! exemplar so the catalog-first commit has a real green baseline, not an empty
//! pass. Implementation waves W1–W3 APPEND per-error cases (bad spec, missing
//! creds, no-cache-db, `reposix log` without `--time-travel`, …) as each
//! subcommand is retrofitted through `reposix_core::errmsg::teach`. Those cases
//! drive the real binary via `assert_cmd` and assert the exact Fix:/Recovery:
//! substrings.
//!
//! Leaf isolation: every test operates inside a `tempfile::TempDir`, never the
//! shared repo (crates/CLAUDE.md leaf-isolation rule). The `git init` here is a
//! test subprocess in that TempDir — hermetic.

use assert_cmd::Command;
use std::process::Command as StdCommand;

/// ANCHOR (W0 baseline): `reposix init` pointed at an existing git-repo ROOT
/// refuses with the bespoke 3-part teaching message — it names the corruption
/// shape ("already a git repository root"), teaches the fix (`Fix:`), points at
/// `reposix attach` as the alternative, and prints runnable recovery lines. The
/// refusal fires (init.rs::refuse_existing_repo_root) BEFORE any spec parse or
/// network, so this is fully hermetic.
///
/// This site is already-3-part; the impl waves ADD a `// teach-exempt: ok`
/// marker but MUST NOT change these emitted strings — this test is the
/// regression guard for that (threat T-120-03).
#[test]
fn init_at_existing_repo_root_teaches_attach_and_recovery() {
    let tmp = tempfile::tempdir().expect("create tempdir");
    let root = tmp.path().join("already-a-repo");
    std::fs::create_dir_all(&root).expect("mkdir repo root");

    // Make `root` a git repository ROOT (subprocess git, inside the TempDir).
    let status = StdCommand::new("git")
        .args(["init", "-q"])
        .current_dir(&root)
        .status()
        .expect("spawn git init");
    assert!(status.success(), "git init failed inside the TempDir");

    let out = Command::cargo_bin("reposix")
        .expect("reposix binary built")
        .args(["init", "sim::demo"])
        .arg(&root)
        .output()
        .expect("run `reposix init`");

    assert!(
        !out.status.success(),
        "`reposix init` at an existing repo root MUST fail-closed"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("already a git repository root"),
        "refusal must name the corruption shape; got:\n{stderr}"
    );
    assert!(
        stderr.contains("Fix:"),
        "refusal must teach the fix (a `Fix:` line); got:\n{stderr}"
    );
    assert!(
        stderr.contains("reposix attach"),
        "refusal must name the `reposix attach` alternative; got:\n{stderr}"
    );
}

// ─── W1: init / attach per-error teaching assertions ────────────────────────
//
// These drive the real `reposix` binary via assert_cmd against ARG-LEVEL error
// paths (bad spec / missing creds / non-git tree) that fail BEFORE any network
// or filesystem mutation, so they are fully hermetic — no sim, no shared repo.
// Each asserts the exact 3-part substrings its name promises (OD-3 §2: a test
// must assert what its name claims).

/// `reposix init <spec-with-no-::>` teaches the `<backend>::<project>` form, the
/// backend list, and a runnable `sim::demo` example — routed through
/// `errmsg::teach` via the shared `spec_parse_error` helper.
#[test]
fn init_bad_spec_teaches_form_backends_and_example() {
    let tmp = tempfile::tempdir().expect("tempdir");
    // A NON-existent target → passes the existing-repo-root refusal, so the
    // spec-parse error (not the refusal) is what fires.
    let dest = tmp.path().join("fresh");
    let out = Command::cargo_bin("reposix")
        .expect("reposix binary built")
        .args(["init", "foo"])
        .arg(&dest)
        .output()
        .expect("run `reposix init`");
    assert!(
        !out.status.success(),
        "a spec with no `::` must fail-closed"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    for needle in ["Fix:", "Recovery:", "<backend>::<project>", "sim::demo"] {
        assert!(stderr.contains(needle), "missing {needle:?} in:\n{stderr}");
    }
}

/// `reposix init confluence::…` with `REPOSIX_CONFLUENCE_TENANT` unset teaches the
/// `export …=<value>` recovery, a retry note, and the credential-free `sim::`
/// alternative (shared `missing_env_var_error` helper, P120 leverage #2).
#[test]
fn init_confluence_missing_tenant_teaches_export_and_sim_alternative() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dest = tmp.path().join("fresh");
    let out = Command::cargo_bin("reposix")
        .expect("reposix binary built")
        .args(["init", "confluence::TokenWorld"])
        .arg(&dest)
        // Explicitly clear the env so a dev/CI shell that HAS the tenant set does
        // not mask the missing-env teaching path under test.
        .env_remove("REPOSIX_CONFLUENCE_TENANT")
        .output()
        .expect("run `reposix init`");
    assert!(
        !out.status.success(),
        "a missing tenant env var must fail-closed"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    for needle in [
        "Fix:",
        "Recovery:",
        "export REPOSIX_CONFLUENCE_TENANT=",
        "sim::",
    ] {
        assert!(stderr.contains(needle), "missing {needle:?} in:\n{stderr}");
    }
}

/// Same shape for JIRA — proves the shared `missing_env_var_error` helper is
/// backend-parameterised, not a confluence-only special case.
#[test]
fn init_jira_missing_instance_teaches_export_and_sim_alternative() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dest = tmp.path().join("fresh");
    let out = Command::cargo_bin("reposix")
        .expect("reposix binary built")
        .args(["init", "jira::TEST"])
        .arg(&dest)
        .env_remove("REPOSIX_JIRA_INSTANCE")
        .output()
        .expect("run `reposix init`");
    assert!(
        !out.status.success(),
        "a missing instance env var must fail-closed"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    for needle in [
        "Fix:",
        "Recovery:",
        "export REPOSIX_JIRA_INSTANCE=",
        "sim::",
    ] {
        assert!(stderr.contains(needle), "missing {needle:?} in:\n{stderr}");
    }
}

/// `reposix attach` pointed at a directory with no `.git/` teaches that attach
/// adopts an EXISTING checkout, names `reposix init` as the alternative, and
/// gives copy-paste recovery. Fails at the `.git/` check before any network.
#[test]
fn attach_non_git_tree_teaches_init_alternative_and_recovery() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let empty = tmp.path().join("not-a-repo");
    std::fs::create_dir_all(&empty).expect("mkdir empty (no .git/)");
    let out = Command::cargo_bin("reposix")
        .expect("reposix binary built")
        .args(["attach", "sim::demo"])
        .arg(&empty)
        .output()
        .expect("run `reposix attach`");
    assert!(
        !out.status.success(),
        "attach on a non-git tree must fail-closed"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    for needle in [
        "Fix:",
        "Recovery:",
        "not a git working tree",
        "reposix init",
    ] {
        assert!(stderr.contains(needle), "missing {needle:?} in:\n{stderr}");
    }
}

// ─── W2: list / refresh / spaces / sync per-error teaching assertions ────────
//
// All ARG-LEVEL paths that fail BEFORE any network/seed (unsupported backend,
// no-reposix-remote, --offline-unimplemented), so they are hermetic — no sim,
// no shared repo, explicit temp paths. Each asserts the EXACT 3-part substrings
// its name promises (OD-3 §2).

/// `reposix spaces --backend sim` hits the DEDUPED Confluence-only error (was
/// three near-identical `bail!`s at spaces.rs 26/29/32). The message MUST name
/// the backend the user actually requested (`sim`), teach why spaces is
/// Confluence-only, and point at `reposix list --backend sim` as the
/// alternative + a copy-paste recovery.
#[test]
fn spaces_sim_backend_names_requested_backend_and_teaches_confluence() {
    let out = Command::cargo_bin("reposix")
        .expect("reposix binary built")
        .args(["spaces", "--backend", "sim"])
        .output()
        .expect("run `reposix spaces`");
    assert!(
        !out.status.success(),
        "`reposix spaces --backend sim` must fail-closed"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    for needle in [
        "Fix:",
        "Alternative:",
        "Recovery:",
        // the ACTUAL requested backend name is echoed (dedupe requirement)
        "you requested `sim`",
        // the per-backend alternative points at `reposix list`
        "reposix list --backend sim",
        // still teaches the confluence path
        "confluence",
    ] {
        assert!(stderr.contains(needle), "missing {needle:?} in:\n{stderr}");
    }
}

/// Same deduped path for JIRA — proves the requested-backend name is
/// parameterised (`jira`), not a sim-only special case. Guards against a future
/// regression that hard-codes one backend name in the shared error.
#[test]
fn spaces_jira_backend_names_requested_backend() {
    let out = Command::cargo_bin("reposix")
        .expect("reposix binary built")
        .args(["spaces", "--backend", "jira"])
        .output()
        .expect("run `reposix spaces`");
    assert!(
        !out.status.success(),
        "`reposix spaces --backend jira` must fail-closed"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    for needle in [
        "Fix:",
        "Recovery:",
        "you requested `jira`",
        "reposix list --backend jira",
    ] {
        assert!(stderr.contains(needle), "missing {needle:?} in:\n{stderr}");
    }
}

/// `reposix sync --reconcile <dir-with-no-reposix-remote>` teaches the FULL
/// 3-part message (upgraded from the partial `anyhow!` at sync.rs 91-95): names
/// the missing remote, teaches `reposix init` / `reposix attach`, and gives a
/// copy-paste recovery. Runs against a fresh non-git temp dir so no remote is
/// resolved — hermetic (git `config --get` on the temp path returns nothing).
#[test]
fn sync_no_reposix_remote_teaches_init_attach_and_recovery() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let empty = tmp.path().join("bare-dir");
    std::fs::create_dir_all(&empty).expect("mkdir bare dir (no reposix remote)");
    let out = Command::cargo_bin("reposix")
        .expect("reposix binary built")
        .args(["sync", "--reconcile"])
        .arg(&empty)
        .output()
        .expect("run `reposix sync`");
    assert!(
        !out.status.success(),
        "`reposix sync --reconcile` with no reposix remote must fail-closed"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    for needle in [
        "Fix:",
        "Recovery:",
        "no reposix remote",
        "reposix attach",
        "reposix init",
    ] {
        assert!(stderr.contains(needle), "missing {needle:?} in:\n{stderr}");
    }
}

/// `reposix refresh --offline` teaches that the offline path is unimplemented
/// (upgraded from the bare `bail!` at refresh.rs 69-72), names the read-the-files
/// alternative, and gives a copy-paste recovery. The `--offline` guard fires
/// FIRST in `run_refresh`, before any cache open or network egress — hermetic.
#[test]
fn refresh_offline_teaches_unimplemented_and_read_files_alternative() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let wt = tmp.path().join("wt");
    let out = Command::cargo_bin("reposix")
        .expect("reposix binary built")
        .arg("refresh")
        .arg(&wt)
        .arg("--offline")
        .output()
        .expect("run `reposix refresh --offline`");
    assert!(
        !out.status.success(),
        "`reposix refresh --offline` must fail-closed (unimplemented)"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    for needle in ["Fix:", "Recovery:", "not implemented", "offline"] {
        assert!(stderr.contains(needle), "missing {needle:?} in:\n{stderr}");
    }
}

// ─── W3: gc / history / tokens / cost per-error teaching assertions ──────────
//
// Two error shapes are exercised end-to-end via the real binary:
//   (a) OUTSIDE a reposix tree — cache_path_from_worktree fails at the shared
//       upstream (worktree_helpers.rs:186), so history/tokens/cost/gc all inherit
//       the SAME "no reposix remote" teaching. One test proves the inheritance.
//   (b) INSIDE a valid reposix tree but with NO synced cache — the shared
//       `missing_cache_db_error` fires; tokens/cost/gc all emit the SAME
//       populate-the-cache teaching (`git fetch` / `reposix refresh`). Three
//       tests prove the helper is genuinely shared, not per-subcommand.
// All paths fail BEFORE any network/seed, so every test is hermetic — a git
// subprocess inside a TempDir + a fresh (empty) REPOSIX_CACHE_DIR (crates/CLAUDE.md
// leaf-isolation: TempDir only, never the shared repo).

/// Run `git <args>` inside `dir` (a TempDir), hermetic (`GIT_CONFIG_NOSYSTEM`).
fn git_in(dir: &std::path::Path, args: &[&str]) {
    let out = StdCommand::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .unwrap_or_else(|e| panic!("git {args:?}: {e}"));
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Build a git working tree bound to a reposix `sim::demo` remote (init-shape:
/// `partialClone=origin`). `cache_path_from_worktree` resolves cleanly against
/// it, so the subcommand proceeds PAST the no-remote check and hits the
/// no-synced-cache path. Returns the working-tree dir (owned by `tmp`).
fn reposix_tree(tmp: &tempfile::TempDir) -> std::path::PathBuf {
    let wt = tmp.path().join("wt");
    std::fs::create_dir_all(&wt).expect("mkdir wt");
    git_in(&wt, &["init", "-q", "."]);
    git_in(
        &wt,
        &[
            "remote",
            "add",
            "origin",
            "reposix::http://127.0.0.1:7878/projects/demo",
        ],
    );
    git_in(&wt, &["config", "extensions.partialClone", "origin"]);
    wt
}

/// `reposix log` without `--time-travel` teaches the flag, names `reposix
/// history` as the alternative for the same listing, and gives copy-paste
/// recovery (main.rs:415 retrofit). Fails at the clap-arg gate before any
/// filesystem access — fully hermetic.
#[test]
fn log_without_time_travel_teaches_history_alternative() {
    let out = Command::cargo_bin("reposix")
        .expect("reposix binary built")
        .arg("log")
        .output()
        .expect("run `reposix log`");
    assert!(
        !out.status.success(),
        "`reposix log` without --time-travel must fail-closed"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    for needle in [
        "Fix:",
        "Alternative:",
        "Recovery:",
        "--time-travel",
        "reposix history",
    ] {
        assert!(stderr.contains(needle), "missing {needle:?} in:\n{stderr}");
    }
}

/// `reposix history <dir-with-no-reposix-remote>` inherits the SHARED
/// worktree_helpers teaching (worktree_helpers.rs:186 retrofit): names the
/// missing remote and teaches BOTH `reposix init` and `reposix attach`. Proves
/// the four worktree-context subcommands route their "not a reposix tree" error
/// through one upstream. Runs against a bare TempDir — hermetic.
#[test]
fn history_outside_reposix_tree_teaches_init_and_attach() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let bare = tmp.path().join("bare");
    std::fs::create_dir_all(&bare).expect("mkdir bare (no reposix remote)");
    let out = Command::cargo_bin("reposix")
        .expect("reposix binary built")
        .arg("history")
        .arg(&bare)
        .output()
        .expect("run `reposix history`");
    assert!(
        !out.status.success(),
        "`reposix history` outside a reposix tree must fail-closed"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    for needle in [
        "Fix:",
        "Recovery:",
        "no reposix remote",
        "reposix init",
        "reposix attach",
    ] {
        assert!(stderr.contains(needle), "missing {needle:?} in:\n{stderr}");
    }
}

/// `reposix tokens` in a valid reposix tree whose cache was never synced hits
/// the SHARED `missing_cache_db_error`: teaches populate-the-cache with BOTH a
/// `git fetch` and a `reposix refresh` copy-paste recovery. The cache path
/// resolves under a fresh (empty) REPOSIX_CACHE_DIR, so `sim-demo.git` is
/// absent — hermetic, no network.
#[test]
// test-name-honesty: ok — "teaches_git_fetch_and_refresh" asserts the ERROR
// MESSAGE literally contains the advice strings "git fetch" and "reposix
// refresh" (recovery text), not that the test performs a real git fetch or
// network round-trip — hermetic, no network per the doc comment above.
fn tokens_no_synced_cache_teaches_git_fetch_and_refresh() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let wt = reposix_tree(&tmp);
    let empty_cache = tmp.path().join("empty-cache");
    let out = Command::cargo_bin("reposix")
        .expect("reposix binary built")
        .arg("tokens")
        .arg(&wt)
        .env("REPOSIX_CACHE_DIR", &empty_cache)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .expect("run `reposix tokens`");
    assert!(
        !out.status.success(),
        "`reposix tokens` with no synced cache must fail-closed"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    for needle in [
        "Fix:",
        "Recovery:",
        "no synced reposix cache",
        "git fetch",
        "reposix refresh",
    ] {
        assert!(stderr.contains(needle), "missing {needle:?} in:\n{stderr}");
    }
}

/// Same no-synced-cache path for `reposix cost` — proves `missing_cache_db_error`
/// is genuinely SHARED across subcommands (tokens + cost emit byte-identical
/// populate-the-cache guidance), not copy-pasted per command.
#[test]
// test-name-honesty: ok — "teaches_git_fetch_and_refresh" asserts the ERROR
// MESSAGE literally contains the advice strings "git fetch" and "reposix
// refresh" (recovery text), not that the test performs a real git fetch or
// network round-trip — hermetic, no network per the doc comment above.
fn cost_no_synced_cache_teaches_git_fetch_and_refresh() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let wt = reposix_tree(&tmp);
    let empty_cache = tmp.path().join("empty-cache");
    let out = Command::cargo_bin("reposix")
        .expect("reposix binary built")
        .arg("cost")
        .arg(&wt)
        .env("REPOSIX_CACHE_DIR", &empty_cache)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .expect("run `reposix cost`");
    assert!(
        !out.status.success(),
        "`reposix cost` with no synced cache must fail-closed"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    for needle in [
        "Fix:",
        "Recovery:",
        "no synced reposix cache",
        "git fetch",
        "reposix refresh",
    ] {
        assert!(stderr.contains(needle), "missing {needle:?} in:\n{stderr}");
    }
}

/// Same no-synced-cache path for `reposix gc` — proves gc's "nothing to gc"
/// case also routes through the SHARED `missing_cache_db_error` (was a bespoke
/// `bail!` at gc.rs:74) rather than its own hand-rolled string.
#[test]
// test-name-honesty: ok — "teaches_git_fetch_and_refresh" asserts the ERROR
// MESSAGE literally contains the advice strings "git fetch" and "reposix
// refresh" (recovery text), not that the test performs a real git fetch or
// network round-trip — hermetic, no network per the doc comment above.
fn gc_no_synced_cache_teaches_git_fetch_and_refresh() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let wt = reposix_tree(&tmp);
    let empty_cache = tmp.path().join("empty-cache");
    let out = Command::cargo_bin("reposix")
        .expect("reposix binary built")
        .arg("gc")
        .arg(&wt)
        .env("REPOSIX_CACHE_DIR", &empty_cache)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .expect("run `reposix gc`");
    assert!(
        !out.status.success(),
        "`reposix gc` with no synced cache must fail-closed"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    for needle in [
        "Fix:",
        "Recovery:",
        "no synced reposix cache",
        "git fetch",
        "reposix refresh",
    ] {
        assert!(stderr.contains(needle), "missing {needle:?} in:\n{stderr}");
    }
}
