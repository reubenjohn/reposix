//! Integration tests for the `stateless-connect` capability surface.
//!
//! Two groups:
//!   1. Non-gated binary-surface checks — run on every `cargo test`.
//!      Assert that the helper advertises `stateless-connect` and
//!      `object-format=sha1` alongside the existing capabilities.
//!   2. Gated end-to-end clones (feature `integration-git`) — run a
//!      real `git clone --filter=blob:none` against the helper. The
//!      dev host's git is 2.25.1 (pre-v2-filter); CI on alpine:latest
//!      (git 2.52) exercises this. Off by default.

#![forbid(unsafe_code)]

use assert_cmd::Command;

#[test]
fn capabilities_advertises_stateless_connect_and_object_format() {
    let mut cmd = Command::cargo_bin("git-remote-reposix").expect("binary built");
    let assert = cmd
        .args(["origin", "reposix::http://127.0.0.1:7878/projects/demo"])
        .write_stdin("capabilities\n")
        .timeout(std::time::Duration::from_secs(10))
        .assert();
    let out = assert.get_output();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("stateless-connect\n"),
        "stdout missing stateless-connect; got:\n{stdout}"
    );
    assert!(
        stdout.contains("object-format=sha1\n"),
        "stdout missing object-format=sha1; got:\n{stdout}"
    );
}

#[test]
fn capabilities_refspec_namespace_is_reposix_not_heads() {
    // Regression test for POC "Bug 1": advertising
    // refs/heads/*:refs/heads/* collapses fast-export to an empty
    // delta. Namespace MUST be refs/reposix/*.
    let mut cmd = Command::cargo_bin("git-remote-reposix").expect("binary built");
    let assert = cmd
        .args(["origin", "reposix::http://127.0.0.1:7878/projects/demo"])
        .write_stdin("capabilities\n")
        .timeout(std::time::Duration::from_secs(10))
        .assert();
    let out = assert.get_output();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("refspec refs/heads/*:refs/reposix/*\n"),
        "refspec namespace not refs/reposix/*; got:\n{stdout}"
    );
    assert!(
        !stdout.contains("refspec refs/heads/*:refs/heads/*\n"),
        "refspec collapsed into heads→heads (POC Bug 1 regressed); got:\n{stdout}"
    );
}

#[test]
fn capabilities_stateless_connect_follows_object_format_order() {
    // Per gitremote-helpers.adoc the capabilities are order-insensitive,
    // but the advertised order must be DETERMINISTIC for the trace-log
    // ground-truth artifact to stay stable. Lock the order here.
    let mut cmd = Command::cargo_bin("git-remote-reposix").expect("binary built");
    let assert = cmd
        .args(["origin", "reposix::http://127.0.0.1:7878/projects/demo"])
        .write_stdin("capabilities\n")
        .timeout(std::time::Duration::from_secs(10))
        .assert();
    let out = assert.get_output();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let expected_prefix = "import\nexport\nrefspec refs/heads/*:refs/reposix/*\nstateless-connect\nobject-format=sha1\n\n";
    assert!(
        stdout.starts_with(expected_prefix),
        "capability ordering regressed; expected prefix:\n{expected_prefix}\n---\ngot:\n{stdout}"
    );
}

// -----------------------------------------------------------------------
// Gated end-to-end clone test. Requires git >= 2.27.
// -----------------------------------------------------------------------

#[cfg(feature = "integration-git")]
#[test]
fn partial_clone_against_sim_is_lazy() {
    // End-to-end check placeholder — fleshed out when CI adds alpine
    // job with git 2.52 per Phase 32 ROADMAP success criterion 1.
    // Assertion skeleton:
    //   1. Seed SimBackend with N issues.
    //   2. REPOSIX_CACHE_DIR=tempdir.
    //   3. PATH prepends target/debug so `git` finds the helper.
    //   4. `git clone --filter=blob:none --no-checkout <url> <dst>`.
    //   5. Assert exit 0; `git rev-list --objects --missing=print --all`
    //      prints every issue OID with a leading '?'.
    //   6. `git cat-file -p <oid>` materializes exactly one blob;
    //      audit-row count for op='materialize' == 1.
    //   7. Re-run `git cat-file -p <oid>`; audit count stays at 1
    //      (idempotent — local read).
    //
    // For Phase 32 the assertions live in the SUMMARY as a CI action
    // item; the Rust-port trace log will be captured here when the
    // integration runner lands.
    panic!("integration-git smoke not yet wired to runner");
}
