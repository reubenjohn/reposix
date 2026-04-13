//! End-to-end binary tests for the git remote helper protocol surface.
//!
//! Drives the compiled `git-remote-reposix` binary via `assert_cmd`,
//! feeding stdin and inspecting stdout / stderr. Verifies the
//! `capabilities`, `option`, and basic dispatch behavior.

#![forbid(unsafe_code)]

use assert_cmd::Command;

#[test]
fn capabilities_advertises_import_export_refspec() {
    let mut cmd = Command::cargo_bin("git-remote-reposix").expect("binary built");
    let assert = cmd
        .args(["origin", "reposix::http://127.0.0.1:7878/projects/demo"])
        .write_stdin("capabilities\n")
        .timeout(std::time::Duration::from_secs(10))
        .assert();
    let out = assert.get_output();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.starts_with("import\nexport\nrefspec refs/heads/*:refs/reposix/*\n"),
        "stdout did not start with capability advertisement; got:\n{stdout}"
    );
}

#[test]
fn option_replies_unsupported() {
    let mut cmd = Command::cargo_bin("git-remote-reposix").expect("binary built");
    let assert = cmd
        .args(["origin", "reposix::http://127.0.0.1:7878/projects/demo"])
        .write_stdin("option dry-run true\n")
        .timeout(std::time::Duration::from_secs(10))
        .assert();
    let out = assert.get_output();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout.trim(), "unsupported", "stdout: {stdout:?}");
}

#[test]
fn unknown_command_writes_to_stderr_not_stdout() {
    let mut cmd = Command::cargo_bin("git-remote-reposix").expect("binary built");
    let assert = cmd
        .args(["origin", "reposix::http://127.0.0.1:7878/projects/demo"])
        .write_stdin("floofle\n")
        .timeout(std::time::Duration::from_secs(10))
        .assert();
    let out = assert.get_output();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stdout.is_empty() || stdout.trim().is_empty(),
        "stdout polluted on unknown command: {stdout:?}"
    );
    assert!(
        stderr.contains("unknown command"),
        "stderr missing diagnostic: {stderr:?}"
    );
}
