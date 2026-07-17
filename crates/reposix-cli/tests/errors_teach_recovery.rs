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
