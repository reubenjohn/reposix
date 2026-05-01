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
    // a no-network constructor (`crates/reposix-remote/src/backend_dispatch.rs`),
    // so the helper reaches the dispatch loop and the capabilities
    // arm fires regardless of SoT availability.
    let out = Command::cargo_bin("git-remote-reposix")
        .expect("cargo bin")
        .args([
            "origin",
            "reposix::http://127.0.0.1:9/projects/demo?mirror=file:///tmp/m.git",
        ])
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
        .args([
            "origin",
            "reposix::http://127.0.0.1:9/projects/demo+file:///tmp/m.git",
        ])
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
        .args([
            "origin",
            "reposix::http://127.0.0.1:9/projects/demo?priority=high",
        ])
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
