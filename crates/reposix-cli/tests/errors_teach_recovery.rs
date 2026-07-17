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
