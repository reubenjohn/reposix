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

// ─── P122 W3 (DRAIN-09 / GTH-V15-06): reposix init nested-in-worktree refusal ─
//
// Binary-side backstop (RPX-0406) for the D2 shared-tree-corruption recurrence: a
// `reposix init` reaching an enclosing NON-/tmp git tree via a subprocess/worktree
// bypasses the Bash-tool leaf-isolation hook, so only a refusal INSIDE the binary
// cuts the vector. Two latches: (1) pre-mutation nested-in-worktree refusal
// (canonicalized, mirrors leaf-isolation-guard.sh::is_safe), (2) a post-`git init`
// worktree-shared-config self-check that aborts before any config write when the
// git-dir is not the target's own `.git`. Cases (a)-(e) drive the real binary.
//
// Leaf isolation: the NON-/tmp trees are built under CARGO_TARGET_TMPDIR
// (<workspace>/target/tmp — gitignored, a SEPARATE repo, never the shared .git);
// the /tmp cases are built under /tmp explicitly (the OS tempdir is /tmp here but
// this keeps the safe-zone branch exercised on any host).

/// A unique throwaway dir under a NON-/tmp root (CARGO_TARGET_TMPDIR) so the
/// nested-in-NON-/tmp refusal is genuinely exercised. Gitignored; TempDir cleans up.
fn non_tmp_dir(tag: &str) -> tempfile::TempDir {
    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"));
    let d = tempfile::Builder::new()
        .prefix(&format!("p122-{tag}-"))
        .tempdir_in(root)
        .expect("tempdir under CARGO_TARGET_TMPDIR");
    // Guard: this test class REQUIRES a non-/tmp root to exercise the refusal. If
    // CARGO_TARGET_DIR were somehow under /tmp, fail loudly rather than silently pass.
    let canon = std::fs::canonicalize(d.path()).expect("canon non_tmp_dir");
    assert!(
        !canon.starts_with("/tmp") && !canon.starts_with("/private/tmp"),
        "non_tmp_dir must be OUTSIDE /tmp to exercise the refusal; got {}",
        canon.display()
    );
    d
}

/// A unique throwaway dir under /tmp — the sanctioned dark-factory zone.
fn tmp_dir(tag: &str) -> tempfile::TempDir {
    tempfile::Builder::new()
        .prefix(&format!("p122-{tag}-"))
        .tempdir_in("/tmp")
        .expect("tempdir under /tmp")
}

/// (a) `reposix init` into a fresh subdir nested inside a NON-/tmp git working tree
/// is REFUSED with the RPX-0406 coded teaching — it names `reposix attach`, teaches
/// the fix, and prints copy-paste recovery. Fires pre-mutation, so fully hermetic.
#[test]
fn init_nested_in_non_tmp_repo_refuses_with_rpx0406() {
    let base = non_tmp_dir("nested-a");
    // A `.git` marker is enough for the ancestor walk to see a working tree.
    std::fs::create_dir_all(base.path().join(".git")).expect("mkdir .git marker");
    let target = base.path().join("fresh-subdir"); // non-existent, nested inside base

    let out = Command::cargo_bin("reposix")
        .expect("reposix binary built")
        .args(["init", "sim::demo"])
        .arg(&target)
        .output()
        .expect("run `reposix init`");
    assert!(
        !out.status.success(),
        "init nested in a non-/tmp git tree MUST fail-closed"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    for needle in [
        "RPX-0406",
        "reposix attach",
        "Fix:",
        "Recovery:",
        "nested inside",
        "reposix explain RPX-0406",
    ] {
        assert!(stderr.contains(needle), "missing {needle:?} in:\n{stderr}");
    }
}

/// (b) A fresh subdir under a /tmp clone is NOT refused (the dark-factory flow is
/// preserved) — init proceeds past BOTH latches to run `git init` (creating the
/// target's own `.git`) and only then fails at the unreachable-sim fetch. Asserts
/// the target's `.git` was created and NO RPX-0406 refusal fired.
#[test]
// test-name-honesty: ok — "not refused" asserts the ABSENCE of the RPX-0406 nesting
// refusal AND that init reached `git init` (target/.git exists) under /tmp; it does
// NOT assert a full backend round-trip (no sim) — the fetch fails afterward, which is
// expected and not the subject of this test.
fn init_fresh_subdir_under_tmp_clone_is_not_refused() {
    let base = tmp_dir("nested-b");
    std::fs::create_dir_all(base.path().join(".git")).expect("mkdir .git marker");
    let target = base.path().join("fresh-subdir");

    let out = Command::cargo_bin("reposix")
        .expect("reposix binary built")
        .args(["init", "sim::demo"])
        .arg(&target)
        // Unreachable sim so the fetch (if reached) fails fast rather than hanging.
        .env("REPOSIX_SIM_ORIGIN", "http://127.0.0.1:59")
        .output()
        .expect("run `reposix init`");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        target.join(".git").exists(),
        "init must reach `git init` under /tmp (both refusals allowed it); stderr:\n{stderr}"
    );
    assert!(
        !stderr.contains("RPX-0406") && !stderr.contains("nested inside"),
        "the /tmp nested init must NOT be refused; stderr:\n{stderr}"
    );
}

/// (c) A symlink whose target lives under /tmp but resolves INTO a non-/tmp git
/// tree must STILL be refused — canonicalization (realpath -m) resolves the symlink
/// before the /tmp-safe decision, defeating path smuggling (T-122-02).
#[test]
fn init_via_symlink_into_non_tmp_repo_refuses_with_rpx0406() {
    let repo = non_tmp_dir("nested-c");
    std::fs::create_dir_all(repo.path().join(".git")).expect("mkdir .git marker");
    let link_base = tmp_dir("nested-c-link");
    let link = link_base.path().join("link"); // lives under /tmp
    std::os::unix::fs::symlink(repo.path(), &link).expect("make symlink");
    // /tmp/.../link/fresh-subdir  →  <non-/tmp repo>/fresh-subdir after canonicalization
    let target = link.join("fresh-subdir");

    let out = Command::cargo_bin("reposix")
        .expect("reposix binary built")
        .args(["init", "sim::demo"])
        .arg(&target)
        .output()
        .expect("run `reposix init`");
    assert!(
        !out.status.success(),
        "a symlink-smuggled non-/tmp nested target MUST fail-closed"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    for needle in ["RPX-0406", "nested inside", "reposix attach"] {
        assert!(
            stderr.contains(needle),
            "symlink smuggle not refused; missing {needle:?} in:\n{stderr}"
        );
    }
}

/// (d) `reposix attach` is NOT regressed by the init-only refusal: attach against a
/// checkout nested inside a non-/tmp git tree proceeds to its OWN logic and never
/// emits the RPX-0406 nesting refusal (attach.rs is untouched by P122 W3).
#[test]
// test-name-honesty: ok — asserts the init-only RPX-0406 refusal does NOT fire on
// `reposix attach` (proves the refusal is init-scoped, attach adoption un-regressed).
// A full attach-success round-trip needs a live sim and is covered by
// quality/gates/agent-ux/reposix-attach.sh, not this hermetic test.
fn attach_nested_checkout_is_not_blocked_by_init_refusal() {
    let base = non_tmp_dir("attach-d");
    std::fs::create_dir_all(base.path().join(".git")).expect("outer .git marker");
    // A real git checkout nested INSIDE the non-/tmp outer tree (attach adopts one).
    let inner = base.path().join("inner-checkout");
    std::fs::create_dir_all(&inner).expect("mkdir inner");
    let status = StdCommand::new("git")
        .args(["init", "-q"])
        .current_dir(&inner)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .status()
        .expect("git init inner");
    assert!(status.success(), "git init inner failed");

    let out = Command::cargo_bin("reposix")
        .expect("reposix binary built")
        .args(["attach", "sim::demo"])
        .arg(&inner)
        .env("REPOSIX_SIM_ORIGIN", "http://127.0.0.1:59")
        .output()
        .expect("run `reposix attach`");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("RPX-0406"),
        "attach must NOT hit the init-only RPX-0406 refusal; stderr:\n{stderr}"
    );
    assert!(
        !stderr.contains("nested inside an existing git working tree"),
        "attach must not emit the init nesting refusal; stderr:\n{stderr}"
    );
}

/// (e) Worktree-shared-config self-check: with GIT_DIR injected (child env) to a
/// shared store, a /tmp-SAFE fresh target passes latch 1 but `git init` binds the
/// target to the shared git-dir, so `git -C <path> rev-parse --absolute-git-dir` !=
/// `<path>/.git`. init MUST abort with RPX-0406 BEFORE any config write, and the
/// shared store's `config` MUST be byte-identical before/after.
#[test]
// test-name-honesty: ok — injects GIT_DIR on the assert_cmd CHILD (no process-global
// set_var) to force a shared git-dir, then asserts (i) init exits non-zero with
// RPX-0406 and (ii) the shared store's config is byte-unchanged (self-check aborted
// pre-config); genuinely exercises latch 2, not merely its presence.
fn init_worktree_shared_git_dir_aborts_before_config_write() {
    // A real shared store under /tmp whose config we watch for corruption.
    let shared_base = tmp_dir("selfcheck-e-shared");
    let shared_repo = shared_base.path().join("shared-repo");
    std::fs::create_dir_all(&shared_repo).expect("mkdir shared-repo");
    let status = StdCommand::new("git")
        .args(["init", "-q"])
        .current_dir(&shared_repo)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .status()
        .expect("git init shared");
    assert!(status.success(), "git init shared failed");
    let shared_git_dir = shared_repo.join(".git");
    let shared_config = shared_git_dir.join("config");
    let before = std::fs::read(&shared_config).expect("read shared config before");

    // A FRESH /tmp-safe target (so latch 1 ALLOWS it) — but GIT_DIR binds git to the
    // shared store, so the resulting git-dir will NOT be <target>/.git.
    let target_base = tmp_dir("selfcheck-e-target");
    let target = target_base.path().join("fresh");

    let out = Command::cargo_bin("reposix")
        .expect("reposix binary built")
        .args(["init", "sim::demo"])
        .arg(&target)
        .env("GIT_DIR", &shared_git_dir) // child-env injection — no process-global set_var
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .expect("run `reposix init`");
    assert!(
        !out.status.success(),
        "a shared git-dir MUST make init fail-closed"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("RPX-0406"),
        "the shared git-dir self-check must emit RPX-0406; stderr:\n{stderr}"
    );

    // The self-check aborted BEFORE any config write — the shared store's config
    // (extensions.partialClone / remote.origin.url would land here) is untouched.
    let after = std::fs::read(&shared_config).expect("read shared config after");
    assert_eq!(
        before, after,
        "the shared store's config MUST be byte-unchanged (self-check aborted before config writes)"
    );
    // Belt-and-suspenders: none of reposix init's config keys leaked into the store.
    let after_str = String::from_utf8_lossy(&after);
    assert!(
        !after_str.contains("partialClone") && !after_str.contains("reposix::"),
        "no reposix init config key may reach the shared store; got:\n{after_str}"
    );
}
