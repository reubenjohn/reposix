//! ARCH-01: after `build_from`, no blob objects exist in `.git/objects/`.
//! The tree references blob OIDs, but the objects themselves are only
//! persisted via `Cache::read_blob` (Plan 02).

mod common;

use gix::prelude::FindExt as _;
use reposix_cache::Cache;
use tempfile::tempdir;
use wiremock::MockServer;

#[tokio::test]
async fn no_blob_objects_after_build_from() {
    let tmp = tempdir().unwrap();
    let prev = common::set_cache_dir(tmp.path());

    let server = MockServer::start().await;
    let issues = common::sample_issues("proj-1", 5);
    common::seed_mock(&server, "proj-1", &issues).await;

    let cache = Cache::open(common::sim_backend(&server), "sim", "proj-1").unwrap();
    cache.build_from().await.unwrap();

    // Walk the object DB and tally by kind.
    let repo = gix::open(cache.repo_path()).unwrap();
    let mut blob_count = 0_usize;
    let mut tree_count = 0_usize;
    let mut commit_count = 0_usize;
    let mut buf = Vec::new();
    for oid_res in repo.objects.iter().unwrap() {
        let oid = oid_res.unwrap();
        let obj = repo
            .objects
            .find(&oid, &mut buf)
            .expect("find known object");
        match obj.kind {
            gix::object::Kind::Blob => blob_count += 1,
            gix::object::Kind::Tree => tree_count += 1,
            gix::object::Kind::Commit => commit_count += 1,
            gix::object::Kind::Tag => {}
        }
    }
    assert_eq!(
        blob_count, 0,
        "expected zero blobs after lazy build_from; found {blob_count}"
    );
    assert!(tree_count >= 1, "expected at least one tree object");
    assert_eq!(commit_count, 1, "expected exactly one commit");

    common::restore_cache_dir(prev);
}
