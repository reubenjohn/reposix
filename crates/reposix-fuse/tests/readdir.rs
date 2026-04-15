//! Integration test for the read-only FUSE mount under the Phase-13
//! nested layout: `ls mount/` shows `.gitignore` + `issues/` (no `tree/`
//! because `SimBackend` doesn't advertise `BackendFeature::Hierarchy`
//! and the fixture issues have no `parent_id`), `ls mount/issues/`
//! lists the 3 seeded issues as `00000000001.md` / 002 / 003 (11-digit
//! padding), and `cat mount/issues/00000000001.md` returns the rendered
//! frontmatter + body. `mount/.gitignore` returns exactly `/tree/\n`.
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
        parent_id: None,
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

    // Wait for readdir to see the new root layout (.gitignore + issues/).
    // The mount is usable as soon as `.gitignore` is visible; that's a
    // synthesized entry served without any backend round-trip.
    let ready = {
        let mp = mount_path.clone();
        wait_for_ready(
            move || {
                std::fs::read_dir(&mp)
                    .map(|it| {
                        let names: Vec<_> = it
                            .flatten()
                            .map(|e| e.file_name().to_string_lossy().into_owned())
                            .collect();
                        names.iter().any(|n| n == "issues")
                            && names.iter().any(|n| n == ".gitignore")
                    })
                    .unwrap_or(false)
            },
            Duration::from_secs(3),
        )
    };
    assert!(ready, "mount did not expose new root layout within 3s");

    // Assertion 1: `ls mount/` shows `.gitignore`, `_INDEX.md`, and `issues/`
    // — and NOT `tree/` (SimBackend doesn't advertise Hierarchy, fixture
    // issues have no parent_id). Phase 18 adds `_INDEX.md` at the root.
    let mut root_names: Vec<String> = std::fs::read_dir(&mount_path)
        .expect("read_dir mount")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    root_names.sort();
    assert_eq!(
        root_names,
        vec![
            ".gitignore".to_owned(),
            "_INDEX.md".to_owned(),
            "issues".to_owned(),
        ],
        "mount root listing mismatch; expected .gitignore + _INDEX.md + issues (no tree/)"
    );

    // Assertion 2: `cat mount/.gitignore` returns exactly `/tree/\n` (7 bytes).
    let gi = std::fs::read(mount_path.join(".gitignore")).expect("read .gitignore");
    assert_eq!(
        gi, b"/tree/\n",
        "gitignore should be exactly /tree/\\n (7 bytes); got {gi:?}"
    );

    // Assertion 3: `ls mount/issues/` shows the 3 seeded issues as
    // 11-digit-padded `<padded-id>.md` entries, plus the synthesized
    // `_INDEX.md` summary (Phase 15 Wave A, LD-15-01).
    let mut issue_names: Vec<String> = std::fs::read_dir(mount_path.join("issues"))
        .expect("read_dir issues/")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    issue_names.sort();
    assert_eq!(
        issue_names,
        vec![
            "00000000001.md".to_owned(),
            "00000000002.md".to_owned(),
            "00000000003.md".to_owned(),
            "_INDEX.md".to_owned(),
        ],
        "issues/ listing mismatch"
    );

    // Assertion 4: `cat mount/issues/00000000001.md` starts with "---\n"
    // and carries frontmatter + body.
    let one = std::fs::read_to_string(mount_path.join("issues/00000000001.md"))
        .expect("read issues/00000000001.md");
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
