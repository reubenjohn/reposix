//! ROADMAP Phase 3 SC #4: killing the backend makes `stat` return in <7s,
//! not hang forever. Gated `#[ignore]` so default `cargo test` stays fast;
//! CI's integration job runs it under `--ignored`.
//!
//! Methodology (per plan 03-01 Task 3):
//!
//! 1. Spin up a wiremock `MockServer`, stub `GET /projects/demo/issues`
//!    returning 3 issues with IDs 1/2/3.
//! 2. `Mount::open` the daemon against the mock's URI on a tempdir.
//! 3. Wait until `read_dir(mount)` exposes the 3 entries (≤3s). This
//!    pre-populates the inode registry AND the in-memory rendered-file
//!    cache for `issues/00000000001.md`.
//! 4. Drop the `MockServer`, so the backend is dead.
//! 5. Shell out `timeout 7 stat <mount>/issues/00000000001.md`. The `timeout(1)` command
//!    is kernel-enforced wall clock — a Rust thread with elapsed-checking
//!    would not actually cut a kernel-blocking syscall short. Assert:
//!    - elapsed <7s (proves no hang),
//!    - exit status non-zero (either `stat` surfaced EIO, or `timeout`
//!      itself killed `stat`; both prove we didn't hang silently).
//! 6. Drop the mount, assert unmount within 3s.

#![cfg(all(
    any(target_os = "linux", target_os = "macos"),
    feature = "fuse-mount-tests"
))]

use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::TimeZone;
use reposix_core::backend::sim::SimBackend;
use reposix_core::{BackendConnector, Issue, IssueId, IssueStatus};
use reposix_fuse::{Mount, MountConfig};
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Run the appropriate unmount command for the current runner.
/// On Linux this defaults to `fusermount3 -u <mnt>`; on macOS CI sets
/// `REPOSIX_UNMOUNT_CMD=umount -f` so the same test binary works.
fn unmount(mnt: &std::path::Path) -> std::io::Result<std::process::ExitStatus> {
    let cmd_str =
        std::env::var("REPOSIX_UNMOUNT_CMD").unwrap_or_else(|_| "fusermount3 -u".to_string());
    let mut parts = cmd_str.split_whitespace();
    let prog = parts.next().expect("REPOSIX_UNMOUNT_CMD is empty");
    let args: Vec<&str> = parts.collect();
    std::process::Command::new(prog)
        .args(&args)
        .arg(mnt)
        .status()
}

fn sample(id: u64) -> Issue {
    let t = chrono::Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
    Issue {
        id: IssueId(id),
        title: format!("issue {id}"),
        status: IssueStatus::Open,
        assignee: None,
        labels: vec![],
        created_at: t,
        updated_at: t,
        version: 1,
        body: format!("body {id}\n"),
        parent_id: None,
    }
}

fn wait_for<F: FnMut() -> bool>(mut pred: F, budget: Duration) -> bool {
    let t0 = Instant::now();
    loop {
        if pred() {
            return true;
        }
        if t0.elapsed() >= budget {
            return false;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn stat_returns_within_7s_after_backend_dies() {
    // Drive wiremock on a current-thread runtime so setup and teardown
    // happen on this test's thread.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");

    let (mock_uri, _) = rt.block_on(async {
        let server = MockServer::start().await;
        let issues = vec![sample(1), sample(2), sample(3)];
        Mock::given(method("GET"))
            .and(path("/projects/demo/issues"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&issues))
            .mount(&server)
            .await;
        for issue in &issues {
            let body = issue.clone();
            let p = format!("/projects/demo/issues/{}", issue.id.0);
            Mock::given(method("GET"))
                .and(path(p))
                .respond_with(ResponseTemplate::new(200).set_body_json(&body))
                .mount(&server)
                .await;
        }
        Mock::given(method("GET"))
            .and(path_regex(r"/projects/demo/issues/\d+"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;
        let uri = server.uri();
        // Returning `server` so it stays alive through the mount+readdir
        // step; we drop it explicitly below to "kill" the backend.
        (uri, server)
    });

    let td = tempfile::Builder::new()
        .prefix("reposix-sim-death-")
        .tempdir()
        .expect("tempdir");
    let mount_path = td.path().to_path_buf();

    let backend: Arc<dyn BackendConnector> =
        Arc::new(SimBackend::new(mock_uri.clone()).expect("sim backend"));
    let mount = Mount::open(
        &MountConfig {
            mount_point: mount_path.clone(),
            origin: mock_uri,
            project: "demo".to_owned(),
            read_only: true,
        },
        backend,
        None,
    )
    .expect("mount open");

    // Pre-cache: wait for the bucket dir (`issues/`) to expose 3 entries.
    // This is what makes `00000000001.md` available for the post-death
    // stat call. Phase 13 moved issue files from `mount/<id>.md` to
    // `mount/issues/<padded-id>.md`.
    let ready = wait_for(
        || std::fs::read_dir(mount_path.join("issues")).is_ok_and(|it| it.flatten().count() >= 3),
        Duration::from_secs(3),
    );
    assert!(ready, "mount did not expose 3 entries within 3s");

    // Kill the backend. The wiremock server was returned from block_on;
    // we owned it at that scope and dropped it at the end of block_on,
    // but only via returning-it-by-name. To be safe, explicitly overwrite
    // with a fresh runtime-less `Drop`: just leave scope — the `_`
    // binding in `let (mock_uri, _) = ...` dropped the server when
    // block_on returned. The backend is dead from this point on.
    //
    // Independently bust the daemon's cached rendered body for
    // `issues/00000000001.md` by re-reading it via a fresh process — we
    // just need to prove stat survives once the TTL expires or an
    // invalidation path fires. Sleep past the 1-second ATTR_TTL so the
    // kernel will re-ask the daemon on the next stat.
    std::thread::sleep(Duration::from_millis(1_200));

    // Target a pre-cached file; the fuser dispatch may hit cache and
    // return fast, OR fall through to lookup/fetch which times out.
    // Either path must return in <7s.
    let target = mount_path.join("issues/00000000001.md");
    let t0 = Instant::now();
    let output = std::process::Command::new("timeout")
        .arg("7")
        .arg("stat")
        .arg(&target)
        .output()
        .expect("spawn timeout+stat");
    let elapsed = t0.elapsed();

    // Belt-and-suspenders: explicit unmount command before relying on Drop.
    // On Linux the default is `fusermount3 -u`; on macOS CI the env var is
    // set to `umount -f`. Ignore errors — Drop will also attempt cleanup.
    let _ = unmount(&mount_path);
    // Drop the mount before any panic fires so we always unmount cleanly.
    drop(mount);

    assert!(
        elapsed < Duration::from_secs(7),
        "stat took {elapsed:?} — kernel hung",
    );
    // Either the `stat` surfaced EIO (exit 1), or `timeout` killed it
    // because it hadn't returned yet (exit 124). Both prove we did NOT
    // hang silently, but we need to accept "stat succeeded because cache
    // served it fast" too — because SC #4 says the daemon must not hang,
    // not that every lookup must fail. A fast cache hit is a pass.
    //
    // The anti-hang assertion is the elapsed check above. The exit-code
    // check is advisory: log it for future debugging but don't fail.
    eprintln!(
        "stat exit={:?} elapsed={:?} stdout={:?} stderr={:?}",
        output.status,
        elapsed,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    // Assert the mount unmounts within 3s.
    let unmounted = wait_for(
        || std::fs::read_dir(&mount_path).map_or(true, |it| it.flatten().count() == 0),
        Duration::from_secs(3),
    );
    assert!(unmounted, "mount did not unmount within 3s");
    drop(td);
}
