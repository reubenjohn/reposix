//! End-to-end integration tests that drive `command=stateless-connect`
//! against the helper binary and exercise the live tunnel runtime in
//! `crates/reposix-remote/src/stateless_connect.rs`.
//!
//! v0.11.x coverage closure: prior tests in `bulk_delete_cap.rs` and
//! `push_conflict.rs` cover the export (push) path; `protocol.rs`
//! covers `capabilities` advertisement; nothing exercised the
//! `stateless-connect` tunnel until now, leaving
//! `handle_stateless_connect`, `send_advertisement`, and
//! `proxy_one_rpc` at 0 hits in the file-level llvm-cov report.
//!
//! Approach: spawn `reposix-sim` in-process on a random port via the
//! library API, point the helper at it via `REPOSIX_CACHE_DIR=tmp` and
//! the URL, then feed protocol-v2 pkt-lines on stdin.

#![forbid(unsafe_code)]

use std::time::Duration;

use assert_cmd::Command;

/// Spawn the simulator on a random loopback port using the library
/// API. Returns `(origin_url, JoinHandle)` — handle is dropped at end
/// of test (axum task quits when the test thread exits).
async fn spawn_sim() -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind 127.0.0.1:0");
    let addr = listener.local_addr().expect("local_addr");
    let origin = format!("http://{addr}");

    let cfg = reposix_sim::SimConfig {
        bind: addr,
        db_path: std::path::PathBuf::from(":memory:"),
        seed: false,
        seed_file: None,
        ephemeral: true,
        rate_limit_rps: 1000,
    };
    let handle = tokio::spawn(async move {
        let _ = reposix_sim::run_with_listener(listener, cfg).await;
    });
    let client =
        reposix_core::http::client(reposix_core::http::ClientOpts::default()).expect("http client");
    for _ in 0..50 {
        if let Ok(r) = client.get(format!("{origin}/healthz")).await {
            if r.status().is_success() {
                return (origin, handle);
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("sim did not become healthy at {origin}");
}

/// Build a pkt-line data frame: `<4-hex-len><payload>` where `len`
/// includes the 4 length bytes themselves.
fn pkt_data(payload: &[u8]) -> Vec<u8> {
    let total = payload.len() + 4;
    let mut buf = format!("{total:04x}").into_bytes();
    buf.extend_from_slice(payload);
    buf
}

/// Scenario 1 — happy advertise + EOF.
///
/// Sends `stateless-connect git-upload-pack\n` followed by an empty
/// terminating line, then closes stdin. The helper must:
///  1. Read the line-oriented service header.
///  2. Run `cache.sync()` against the sim (empty cache → seed path).
///  3. Reply with a blank line ("ready") + flush.
///  4. Spawn `git upload-pack --advertise-refs --stateless-rpc` and
///     forward its output (capability advertisement).
///  5. Enter the proxy RPC loop, read pkt-lines from stdin, observe
///     EOF on the empty request, and exit cleanly.
///
/// Asserts: exit 0, stdout begins with the blank "ready" line, the
/// advertisement contains the protocol-v2 `version 2` line, and
/// stdout terminates in flush `0000`.
#[tokio::test]
async fn stateless_connect_advertises_then_eof() {
    let (origin, _sim) = spawn_sim().await;
    let cache_dir = tempfile::tempdir().expect("tempdir");

    let url = format!("reposix::{origin}/projects/demo");
    // One terminating \n; the helper transitions directly into pkt-line
    // mode after consuming the service line, so a second \n would be
    // read as the next pkt-line header (and `\n` is not valid hex).
    let stdin_data = b"stateless-connect git-upload-pack\n".to_vec();

    let cache_path = cache_dir.path().to_path_buf();
    let assert = tokio::task::spawn_blocking(move || {
        Command::cargo_bin("git-remote-reposix")
            .expect("binary built")
            .args(["origin", &url])
            .env("REPOSIX_CACHE_DIR", &cache_path)
            .write_stdin(stdin_data)
            .timeout(Duration::from_secs(15))
            .assert()
    })
    .await
    .unwrap();

    let out = assert.get_output();
    let stderr = String::from_utf8_lossy(&out.stderr);
    let stdout = &out.stdout;

    assert!(
        out.status.success(),
        "stateless-connect must exit 0; status={:?}; stderr={stderr}",
        out.status
    );
    // (a) "ready" sentinel: helper writes a single LF after reading the
    // service header. send_blank() emits "\n" via the line-oriented
    // protocol writer.
    assert!(
        stdout.starts_with(b"\n"),
        "stdout must start with blank ready line; got first bytes: {:?}",
        &stdout[..stdout.len().min(32)]
    );
    // (b) Advertisement contains protocol-v2 version line.
    assert!(
        stdout
            .windows(b"version 2".len())
            .any(|w| w == b"version 2"),
        "advertisement missing `version 2`; stdout={:?}",
        String::from_utf8_lossy(stdout)
    );
    // (c) Advertisement is flush-terminated.
    assert!(
        stdout.ends_with(b"0000"),
        "stdout must end with flush 0000; tail: {:?}",
        &stdout[stdout.len().saturating_sub(16)..]
    );
}

/// Scenario 2 — blob-limit refusal on `command=fetch` with too many
/// wants.
///
/// Sets `REPOSIX_BLOB_LIMIT=1`, then sends a fetch RPC with two `want`
/// lines after the advertisement turn. The helper must:
///  1. Complete the advertisement (same as scenario 1).
///  2. Read the `command=fetch` request, count wants, see
///     `2 > limit (1)`, log via `log_blob_limit_exceeded`, write the
///     teaching string to stderr, and exit non-zero.
///
/// Asserts: non-zero exit, stderr contains literal `git sparse-checkout`
/// (the recovery teaching string for an LLM agent).
#[tokio::test]
async fn stateless_connect_blob_limit_refuses_excess_wants() {
    let (origin, _sim) = spawn_sim().await;
    let cache_dir = tempfile::tempdir().expect("tempdir");

    let url = format!("reposix::{origin}/projects/demo");

    // Construct stdin: line-oriented header + pkt-line fetch RPC with
    // two wants. The OIDs are arbitrary 40-hex strings — the blob-limit
    // check fires before upload-pack ever sees them.
    let mut stdin_data = b"stateless-connect git-upload-pack\n".to_vec();
    stdin_data.extend_from_slice(&pkt_data(b"command=fetch\n"));
    stdin_data.extend_from_slice(b"0001"); // delim
    stdin_data.extend_from_slice(&pkt_data(
        b"want 0000000000000000000000000000000000000001\n",
    ));
    stdin_data.extend_from_slice(&pkt_data(
        b"want 0000000000000000000000000000000000000002\n",
    ));
    stdin_data.extend_from_slice(b"0000"); // flush

    let cache_path = cache_dir.path().to_path_buf();
    let assert = tokio::task::spawn_blocking(move || {
        Command::cargo_bin("git-remote-reposix")
            .expect("binary built")
            .args(["origin", &url])
            .env("REPOSIX_CACHE_DIR", &cache_path)
            .env("REPOSIX_BLOB_LIMIT", "1")
            .write_stdin(stdin_data)
            .timeout(Duration::from_secs(15))
            .assert()
    })
    .await
    .unwrap();

    let out = assert.get_output();
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert!(
        !out.status.success(),
        "blob-limit refusal must exit non-zero; stderr={stderr}"
    );
    assert!(
        stderr.contains("git sparse-checkout"),
        "stderr must contain `git sparse-checkout` recovery teaching string; stderr={stderr}"
    );
}

/// Scenario 3 — non-upload-pack service answered with the `fallback`
/// sentinel (git-2.43 push-compat regression, P94 D2).
///
/// Drives `stateless-connect git-receive-pack`, the service git >= 2.43
/// probes FIRST for the push direction. reposix serves push through the
/// `export` capability, not protocol-v2, so per git-remote-helpers(7) the
/// helper MUST reply the literal `fallback` line — git then retries the
/// push via `export`. The pre-fix helper wrote a custom `unsupported
/// service: ...` line, which git treats as "unknown response to connect"
/// and aborts the whole push (exit 128, no export attempt).
///
/// Asserts: the helper writes `fallback` to stdout, does NOT emit the old
/// `unsupported service:` rejection line, and exits 0 — `fallback` is a
/// normal negotiated outcome (the helper resumes its verb loop and hits a
/// clean EOF here), NOT an error. Covers the non-upload-pack branch of
/// `handle_stateless_connect` returning `StatelessConnectOutcome::FellBack`.
#[tokio::test]
async fn stateless_connect_replies_fallback_for_non_upload_pack_service() {
    let (origin, _sim) = spawn_sim().await;
    let cache_dir = tempfile::tempdir().expect("tempdir");

    let url = format!("reposix::{origin}/projects/demo");
    let stdin_data = b"stateless-connect git-receive-pack\n".to_vec();

    let cache_path = cache_dir.path().to_path_buf();
    let assert = tokio::task::spawn_blocking(move || {
        Command::cargo_bin("git-remote-reposix")
            .expect("binary built")
            .args(["origin", &url])
            .env("REPOSIX_CACHE_DIR", &cache_path)
            .write_stdin(stdin_data)
            .timeout(Duration::from_secs(15))
            .assert()
    })
    .await
    .unwrap();

    let out = assert.get_output();
    let stderr = String::from_utf8_lossy(&out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);

    // The reply is a single `fallback` line (git-remote-helpers(7)) — git
    // retries via `export`. It is NOT an error: the helper resumes its
    // verb loop, reads EOF, and exits 0.
    assert!(
        out.status.success(),
        "`fallback` is a normal negotiated reply, not an error; must exit 0; \
         status={:?}; stderr={stderr}",
        out.status
    );
    assert!(
        stdout.lines().any(|l| l == "fallback"),
        "stdout must contain the literal `fallback` sentinel line; stdout={stdout:?}"
    );
    // Guard against the pre-fix bug reappearing: the custom rejection line
    // is exactly what made git 2.43 abort the push.
    assert!(
        !stdout.contains("unsupported service"),
        "stdout must NOT carry the bug-preserving `unsupported service` line; stdout={stdout:?}"
    );
}
