//! Integration test for the read-only FUSE mount: `ls` lists issues as
//! `0001.md 0002.md 0003.md`, `cat 0001.md` returns the rendered
//! frontmatter+body.
//!
//! Backend is a wiremock `MockServer` seeded with 3 synthetic issues,
//! wrapped in a `SimBackend` to drive the Phase-10 IssueBackend seam;
//! the mount lives on a `tempfile::tempdir()`. Gated `target_os = "linux"`
//! because fuser/FUSE3 mounts only exist on Linux.

#![cfg(target_os = "linux")]

use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::TimeZone;
use reposix_core::backend::sim::SimBackend;
use reposix_core::{Issue, IssueBackend, IssueId, IssueStatus};
use reposix_fuse::{Mount, MountConfig};
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn sample(id: u64, body: &str) -> Issue {
    let t = chrono::Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
    Issue {
        id: IssueId(id),
        title: format!("issue {id}"),
        status: IssueStatus::Open,
        assignee: None,
        labels: vec!["test".to_owned()],
        created_at: t,
        updated_at: t,
        version: 1,
        body: body.to_owned(),
    }
}

fn wait_for_ready<F: FnMut() -> bool>(mut pred: F, budget: Duration) -> bool {
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn mount_lists_and_reads_issues() {
    let server = MockServer::start().await;
    let issues = vec![
        sample(1, "content of issue 1\n"),
        sample(2, "content of issue 2\n"),
        sample(3, "content of issue 3\n"),
    ];

    // List endpoint
    Mock::given(method("GET"))
        .and(path("/projects/demo/issues"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&issues))
        .mount(&server)
        .await;
    // Per-issue endpoints. Use a regex so the 3 IDs share one mock.
    for issue in &issues {
        let body = issue.clone();
        let p = format!("/projects/demo/issues/{}", issue.id.0);
        Mock::given(method("GET"))
            .and(path(p))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .mount(&server)
            .await;
    }
    // Any other /issues/ lookup returns 404.
    Mock::given(method("GET"))
        .and(path_regex(r"/projects/demo/issues/\d+"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let td = tempfile::Builder::new()
        .prefix("reposix-fuse-test-")
        .tempdir()
        .expect("tempdir");
    let mount_path = td.path().to_path_buf();

    // Spawn mount on a blocking task so fuser's BackgroundSession + its
    // kernel thread are allowed to sit on a native thread.
    let origin = server.uri();
    let backend: Arc<dyn IssueBackend> =
        Arc::new(SimBackend::new(origin.clone()).expect("sim backend"));
    let mount = tokio::task::spawn_blocking({
        let mount_path = mount_path.clone();
        move || {
            Mount::open(
                &MountConfig {
                    mount_point: mount_path,
                    origin,
                    project: "demo".to_owned(),
                    read_only: true,
                },
                backend,
            )
        }
    })
    .await
    .expect("join spawn_blocking")
    .expect("mount opened");

    // Wait for readdir to see 3 entries (kernel may need a beat to route
    // the first stat — up to 3s is generous).
    let ready = {
        let mp = mount_path.clone();
        wait_for_ready(
            move || {
                std::fs::read_dir(&mp)
                    .map(|it| it.flatten().count() >= 3)
                    .unwrap_or(false)
            },
            Duration::from_secs(3),
        )
    };
    assert!(ready, "mount did not expose 3 entries within 3s");

    // Assertion 1: ls shows 0001.md, 0002.md, 0003.md.
    let mut names: Vec<String> = std::fs::read_dir(&mount_path)
        .expect("read_dir")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(
        names,
        vec![
            "0001.md".to_owned(),
            "0002.md".to_owned(),
            "0003.md".to_owned()
        ],
        "mount listing mismatch"
    );

    // Assertion 2: cat 0001.md starts with "---\n" and contains "id: 1".
    let one = std::fs::read_to_string(mount_path.join("0001.md")).expect("read 0001.md");
    assert!(
        one.starts_with("---\n"),
        "missing frontmatter fence: {one:?}"
    );
    assert!(
        one.contains("id: 1"),
        "missing id: 1 in rendered file: {one:?}"
    );
    assert!(
        one.contains("content of issue 1"),
        "body missing from rendered file"
    );

    // Drop the mount — fuser's UmountOnDrop unmounts. Wait up to 3s for
    // the path to become "not-our-FS" again (read_dir returns empty since
    // tempdir is actually empty).
    drop(mount);
    let unmounted = wait_for_ready(
        || {
            std::fs::read_dir(&mount_path)
                .map(|it| it.flatten().count() == 0)
                .unwrap_or(true)
        },
        Duration::from_secs(3),
    );
    assert!(unmounted, "mount did not unmount within 3s");

    // tempdir drop cleans up the mount directory.
    drop(td);
}
