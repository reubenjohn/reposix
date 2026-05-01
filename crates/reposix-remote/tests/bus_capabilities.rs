//! Integration test: bus URL omits `stateless-connect` from
//! capabilities (DVCS-BUS-FETCH-01 / Q3.4).

#![allow(clippy::missing_panics_doc)]

use assert_cmd::Command;

#[test]
fn bus_url_omits_stateless_connect() {
    // Bus URL — `stateless-connect` MUST be absent.
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
    assert!(
        stdout.contains("import"),
        "expected `import`; got: {stdout}"
    );
    assert!(
        stdout.contains("export"),
        "expected `export`; got: {stdout}"
    );
    assert!(
        stdout.contains("refspec refs/heads/*:refs/reposix/*"),
        "expected `refspec`; got: {stdout}"
    );
    assert!(
        stdout.contains("object-format=sha1"),
        "expected `object-format=sha1`; got: {stdout}"
    );
    assert!(
        !stdout.contains("stateless-connect"),
        "bus URL MUST NOT advertise stateless-connect (DVCS-BUS-FETCH-01); got: {stdout}"
    );
}

#[test]
fn single_backend_url_advertises_stateless_connect() {
    // Regression check: bare `reposix::<sot>` (no `?mirror=`) DOES
    // advertise `stateless-connect`. Without this guard, an off-by-
    // one in capability branching would silently break single-backend
    // fetch (DVCS-DARKFACTORY-* would catch it eventually but the
    // signal is much faster here).
    let out = Command::cargo_bin("git-remote-reposix")
        .expect("cargo bin")
        .args(["origin", "reposix::http://127.0.0.1:9/projects/demo"])
        .write_stdin("capabilities\n\n")
        .output()
        .expect("run helper");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("stateless-connect"),
        "single-backend URL MUST advertise stateless-connect; got: {stdout}"
    );
}
