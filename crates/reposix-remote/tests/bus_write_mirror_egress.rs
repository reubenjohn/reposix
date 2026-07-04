//! Mirror-egress allowlist gate — bus-push regression (QL-006).
//!
//! Asserts the security invariant: a bus push whose `?mirror=<url>` names
//! a host NOT on `REPOSIX_ALLOWED_ORIGINS` is rejected BEFORE any network
//! egress (no `git ls-remote`, no stdin read, no cache open). The mirror
//! push is a tainted-content egress channel and must be allowlist-gated
//! per CLAUDE.md § Threat model.
//!
//! Donor pattern: `tests/bus_write_no_mirror_remote.rs` (fast-bail shape).

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::unnecessary_debug_formatting)]

use std::path::Path;
use std::process::Command;

use assert_cmd::Command as AssertCommand;

fn run_git_in(dir: &Path, args: &[&str]) {
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
}

#[test]
// test-name-honesty: ok — real git-remote fixture asserting denial before egress; negative security test, name is exact
fn bus_push_to_non_allowlisted_mirror_is_denied_before_egress() {
    // Working tree with a `mirror` remote pointing at a NON-allowlisted
    // host. The host is `.invalid` (RFC 6761) so if the gate ever failed
    // to fire, the subsequent `git ls-remote` would try — and fail — DNS,
    // but the gate must bail first.
    let wtree = tempfile::tempdir().expect("wtree tempdir");
    run_git_in(wtree.path(), &["init", "."]);
    run_git_in(wtree.path(), &["config", "user.email", "ql006@example"]);
    run_git_in(wtree.path(), &["config", "user.name", "QL006 Test"]);

    let mirror_url = "https://mirror.example.invalid/space-mirror.git";
    run_git_in(wtree.path(), &["remote", "add", "mirror", mirror_url]);

    // SoT is a loopback sim URL (port 9 — never contacted; the gate bails
    // before any SoT/prechecks work).
    let bus_url = format!("reposix::http://127.0.0.1:9/projects/demo?mirror={mirror_url}");

    let cache_root = tempfile::tempdir().expect("cache_root");

    let out = AssertCommand::cargo_bin("git-remote-reposix")
        .expect("cargo bin")
        .args(["origin", &bus_url])
        .write_stdin("capabilities\n\nexport\n\n")
        .current_dir(wtree.path())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("REPOSIX_CACHE_DIR", cache_root.path())
        // Default allowlist is loopback-only; mirror.example.invalid is
        // NOT on it. (We set it explicitly to make the test hermetic.)
        .env("REPOSIX_ALLOWED_ORIGINS", "http://127.0.0.1:*")
        .timeout(std::time::Duration::from_secs(15))
        .output()
        .expect("run helper");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    // 1. Non-zero exit.
    assert!(
        !out.status.success(),
        "helper must reject a non-allowlisted mirror; stdout={stdout}, stderr={stderr}"
    );

    // 2. Teaching message: names the env var, the rejected origin, and the
    //    export line to fix it.
    assert!(
        stderr.contains("REPOSIX_ALLOWED_ORIGINS"),
        "stderr must name the env var; got: {stderr}"
    );
    assert!(
        stderr.contains("https://mirror.example.invalid"),
        "stderr must show the rejected origin; got: {stderr}"
    );
    assert!(
        stderr.contains("export REPOSIX_ALLOWED_ORIGINS="),
        "stderr must show the export line to fix it; got: {stderr}"
    );

    // 3. git's protocol-level reject line on stdout.
    assert!(
        stdout.contains("error refs/heads/main"),
        "expected a protocol reject line; got stdout={stdout}"
    );

    // 4. Fast bail BEFORE ensure_cache — no bare repo populated in the
    //    cache root (mirrors the no-mirror-remote invariant).
    let any_bare = walkdir::WalkDir::new(cache_root.path())
        .into_iter()
        .filter_map(std::result::Result::ok)
        .any(|e| e.file_type().is_dir() && e.path().extension().is_some_and(|x| x == "git"));
    assert!(
        !any_bare,
        "egress gate must fire BEFORE ensure_cache; cache root: {:?}",
        cache_root.path()
    );

    let _ = wtree;
}
