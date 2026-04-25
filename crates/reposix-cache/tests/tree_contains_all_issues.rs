//! ARCH-01: `Cache::build_from` produces a tree with one entry per seeded issue.

mod common;

use reposix_cache::Cache;
use tempfile::tempdir;
use wiremock::MockServer;

#[tokio::test]
async fn tree_contains_all_seeded_issues() {
    let tmp = tempdir().unwrap();
    let prev = common::set_cache_dir(tmp.path());

    let server = MockServer::start().await;
    let issues = common::sample_issues("proj-1", 10);
    common::seed_mock(&server, "proj-1", &issues).await;

    let cache = Cache::open(common::sim_backend(&server), "sim", "proj-1").unwrap();
    cache.build_from().await.unwrap();

    // Walk the tree at refs/heads/main and count blob entries under `issues/`.
    let repo = gix::open(cache.repo_path()).expect("open bare");
    let reference = repo
        .find_reference("refs/heads/main")
        .expect("refs/heads/main exists");
    let mut reference = reference;
    let commit = reference.peel_to_id().expect("peel to id");
    let commit = commit.object().expect("commit object").into_commit();
    let tree = commit.tree().expect("commit tree");
    let mut count = 0_usize;
    let mut saw_expected_path = false;
    for entry_res in tree.iter() {
        let entry = entry_res.expect("tree entry");
        if entry.filename() == "issues" {
            let sub = entry
                .object()
                .expect("subtree object")
                .try_into_tree()
                .expect("subtree is a tree");
            for child_res in sub.iter() {
                let child = child_res.expect("subtree entry");
                count += 1;
                let name = child.filename().to_string();
                if name == "1.md" {
                    saw_expected_path = true;
                }
            }
        }
    }
    assert_eq!(count, 10, "expected 10 blob entries in the tree");
    assert!(saw_expected_path, "expected issues/1.md to exist in tree");

    common::restore_cache_dir(prev);
}

#[tokio::test]
async fn tree_contains_single_issue() {
    let tmp = tempdir().unwrap();
    let prev = common::set_cache_dir(tmp.path());

    let server = MockServer::start().await;
    let issues = common::sample_issues("proj-solo", 1);
    common::seed_mock(&server, "proj-solo", &issues).await;

    let cache = Cache::open(common::sim_backend(&server), "sim", "proj-solo").unwrap();
    cache.build_from().await.unwrap();

    let repo = gix::open(cache.repo_path()).unwrap();
    let mut reference = repo.find_reference("refs/heads/main").unwrap();
    let commit = reference
        .peel_to_id()
        .unwrap()
        .object()
        .unwrap()
        .into_commit();
    let tree = commit.tree().unwrap();
    let mut count = 0_usize;
    for entry_res in tree.iter() {
        let entry = entry_res.unwrap();
        if entry.filename() == "issues" {
            let sub = entry.object().unwrap().try_into_tree().unwrap();
            for c in sub.iter() {
                c.unwrap();
                count += 1;
            }
        }
    }
    assert_eq!(count, 1);

    common::restore_cache_dir(prev);
}
