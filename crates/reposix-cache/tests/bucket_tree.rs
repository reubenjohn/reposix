//! Wave-5.5 bucket-aware cache tree: the outer tree entry is the backend's
//! canonical record bucket (`issues/` for sim/github/jira, `pages/` for
//! confluence). Regression for the confluence mass-delete BLOCKER
//! (SURPRISES-INTAKE 2026-07-04 21:00): the cache used to hardcode
//! `issues/` for ALL backends, so the confluence cache tree disagreed with
//! `reposix refresh`'s documented `pages/<id>.md` UX and every real
//! confluence push misclassified all records.

mod common;

use common::CacheDirGuard;
use reposix_cache::Cache;
use tempfile::tempdir;
use wiremock::MockServer;

/// Return the sole top-level tree entry name of HEAD's tree.
fn head_tree_entry_name(repo_path: &std::path::Path) -> String {
    let repo = gix::open(repo_path).unwrap();
    let head_commit = repo
        .find_reference("refs/heads/main")
        .unwrap()
        .peel_to_commit()
        .unwrap();
    let tree = head_commit.tree().unwrap();
    let entries: Vec<String> = tree
        .iter()
        .map(|e| e.unwrap().filename().to_string())
        .collect();
    assert_eq!(entries.len(), 1, "outer tree must have one bucket entry");
    entries.into_iter().next().unwrap()
}

#[tokio::test]
async fn confluence_cache_tree_uses_pages_bucket() {
    let tmp = tempdir().unwrap();
    let _g = CacheDirGuard::new(tmp.path());
    let server = MockServer::start().await;
    let issues = common::sample_issues("SPACE", 3);
    common::seed_mock(&server, "SPACE", &issues).await;

    let cache = Cache::open(common::sim_backend(&server), "confluence", "SPACE").unwrap();
    cache.build_from().await.unwrap();

    assert_eq!(
        head_tree_entry_name(cache.repo_path()),
        "pages",
        "confluence cache tree must nest records under pages/ (Wave-5.5)"
    );
}

#[tokio::test]
async fn sim_cache_tree_uses_issues_bucket() {
    let tmp = tempdir().unwrap();
    let _g = CacheDirGuard::new(tmp.path());
    let server = MockServer::start().await;
    let issues = common::sample_issues("proj-1", 3);
    common::seed_mock(&server, "proj-1", &issues).await;

    let cache = Cache::open(common::sim_backend(&server), "sim", "proj-1").unwrap();
    cache.build_from().await.unwrap();

    assert_eq!(
        head_tree_entry_name(cache.repo_path()),
        "issues",
        "sim cache tree must nest records under issues/"
    );
}
