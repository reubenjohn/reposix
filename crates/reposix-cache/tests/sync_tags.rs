//! Phase v0.11.0 §3b — time-travel via git tags.
//!
//! Verifies that `Cache::sync` writes a deterministic ref under
//! `refs/reposix/sync/<ISO8601-no-colons>` per sync, that the audit row
//! `op='sync_tag_written'` is appended, that the list/at APIs return the
//! tags in the right order, and that the helper's protocol-v2
//! advertisement does NOT propagate these private refs.

#![allow(clippy::missing_panics_doc)]

use std::sync::Arc;

use reposix_cache::{list_sync_tags_at, Cache};
use reposix_core::BackendConnector;
use wiremock::MockServer;

mod common;
use common::{sample_issues, seed_mock, sim_backend, CacheDirGuard};

#[tokio::test(flavor = "multi_thread")]
async fn tag_sync_creates_ref() {
    let server = MockServer::start().await;
    let issues = sample_issues("demo", 3);
    seed_mock(&server, "demo", &issues).await;

    let cache_root = tempfile::tempdir().unwrap();
    let _env = CacheDirGuard::new(cache_root.path());
    let backend: Arc<dyn BackendConnector> = sim_backend(&server);
    let cache = Cache::open(backend, "sim", "demo").expect("Cache::open");

    let report = cache.sync().await.expect("seed sync");
    let commit = report.new_commit.expect("seed produces commit");

    // The seed sync must have written one tag pointing at the synthesis commit.
    let tags = cache.list_sync_tags().expect("list_sync_tags");
    assert_eq!(tags.len(), 1, "seed sync writes exactly one tag, got {tags:?}");
    assert!(
        tags[0].name.starts_with("refs/reposix/sync/"),
        "unexpected tag name: {}",
        tags[0].name
    );
    assert_eq!(tags[0].commit, commit, "tag must point at synthesis commit");
}

#[tokio::test(flavor = "multi_thread")]
async fn multiple_syncs_create_multiple_tags() {
    let server = MockServer::start().await;
    let issues = sample_issues("demo", 2);
    seed_mock(&server, "demo", &issues).await;

    let cache_root = tempfile::tempdir().unwrap();
    let _env = CacheDirGuard::new(cache_root.path());
    let backend: Arc<dyn BackendConnector> = sim_backend(&server);
    let cache = Cache::open(backend, "sim", "demo").expect("Cache::open");

    // Seed sync #1.
    cache.sync().await.expect("sync 1 (seed)");
    // Sleep so the second sync's slug differs at second granularity.
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
    cache.sync().await.expect("sync 2 (delta, empty)");
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
    cache.sync().await.expect("sync 3 (delta, empty)");

    let tags = cache.list_sync_tags().expect("list_sync_tags");
    assert_eq!(tags.len(), 3, "expected 3 tags, got {tags:?}");
    // Chronological order — list returns ascending by timestamp.
    assert!(
        tags[0].timestamp <= tags[1].timestamp && tags[1].timestamp <= tags[2].timestamp,
        "tags must be sorted by timestamp ascending: {tags:?}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn list_sync_tags_returns_sorted() {
    // Indirect proof — write three tags out of order via the public API
    // and assert the listing is sorted ascending.
    let server = MockServer::start().await;
    let issues = sample_issues("demo", 1);
    seed_mock(&server, "demo", &issues).await;

    let cache_root = tempfile::tempdir().unwrap();
    let _env = CacheDirGuard::new(cache_root.path());
    let backend: Arc<dyn BackendConnector> = sim_backend(&server);
    let cache = Cache::open(backend, "sim", "demo").expect("Cache::open");

    let r = cache.sync().await.expect("seed sync");
    let commit = r.new_commit.unwrap();

    // Add two more tags at fabricated timestamps via tag_sync directly.
    let t_old: chrono::DateTime<chrono::Utc> = "2025-01-01T00:00:00Z".parse().unwrap();
    let t_mid: chrono::DateTime<chrono::Utc> = "2025-06-15T12:30:00Z".parse().unwrap();
    cache.tag_sync(commit, t_mid).expect("tag t_mid");
    cache.tag_sync(commit, t_old).expect("tag t_old");

    let tags = cache.list_sync_tags().expect("list_sync_tags");
    assert!(tags.len() >= 3, "expected ≥3 tags, got {tags:?}");
    // Sorted ascending.
    for w in tags.windows(2) {
        assert!(
            w[0].timestamp <= w[1].timestamp,
            "tags not sorted: {} > {}",
            w[0].name,
            w[1].name
        );
    }
    // First two are the explicitly-added ancient tags.
    assert_eq!(tags[0].timestamp, t_old);
    assert_eq!(tags[1].timestamp, t_mid);
}

#[tokio::test(flavor = "multi_thread")]
async fn sync_tag_at_finds_closest_not_after() {
    let server = MockServer::start().await;
    let issues = sample_issues("demo", 1);
    seed_mock(&server, "demo", &issues).await;

    let cache_root = tempfile::tempdir().unwrap();
    let _env = CacheDirGuard::new(cache_root.path());
    let backend: Arc<dyn BackendConnector> = sim_backend(&server);
    let cache = Cache::open(backend, "sim", "demo").expect("Cache::open");
    let r = cache.sync().await.expect("seed sync");
    let commit = r.new_commit.unwrap();

    let t1: chrono::DateTime<chrono::Utc> = "2026-01-01T00:00:00Z".parse().unwrap();
    let t2: chrono::DateTime<chrono::Utc> = "2026-02-01T00:00:00Z".parse().unwrap();
    let t3: chrono::DateTime<chrono::Utc> = "2026-03-01T00:00:00Z".parse().unwrap();
    cache.tag_sync(commit, t1).unwrap();
    cache.tag_sync(commit, t2).unwrap();
    cache.tag_sync(commit, t3).unwrap();

    // Target between t2 and t3 → must select t2.
    let mid: chrono::DateTime<chrono::Utc> = "2026-02-15T00:00:00Z".parse().unwrap();
    let chosen = cache.sync_tag_at(mid).expect("sync_tag_at").expect("Some");
    assert_eq!(chosen.timestamp, t2);

    // Target before t1 — but the seed sync also wrote a tag at "now",
    // which is also after t1. So a target before t1 must yield None.
    let way_back: chrono::DateTime<chrono::Utc> = "2024-01-01T00:00:00Z".parse().unwrap();
    let none = cache.sync_tag_at(way_back).expect("sync_tag_at way back");
    assert!(none.is_none(), "target predates all tags, expected None, got {none:?}");

    // Target exactly at t3 — must return t3 (≤ rule).
    let exact = cache
        .sync_tag_at(t3)
        .expect("sync_tag_at exact")
        .expect("Some");
    assert_eq!(exact.timestamp, t3);
}

#[tokio::test(flavor = "multi_thread")]
async fn sync_tags_audit_row_written() {
    let server = MockServer::start().await;
    let issues = sample_issues("demo", 1);
    seed_mock(&server, "demo", &issues).await;

    let cache_root = tempfile::tempdir().unwrap();
    let _env = CacheDirGuard::new(cache_root.path());
    let backend: Arc<dyn BackendConnector> = sim_backend(&server);
    let cache = Cache::open(backend, "sim", "demo").expect("Cache::open");
    cache.sync().await.expect("seed sync");

    // Inspect the audit DB for the new op.
    let conn = rusqlite::Connection::open(cache.repo_path().join("cache.db")).unwrap();
    let n: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM audit_events_cache WHERE op = 'sync_tag_written'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(n, 1, "exactly one sync_tag_written row from seed sync");

    // Reason column carries the full ref name.
    let reason: String = conn
        .query_row(
            "SELECT reason FROM audit_events_cache WHERE op = 'sync_tag_written' LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(
        reason.starts_with("refs/reposix/sync/"),
        "reason should be the full ref name, got {reason}"
    );

    // OID column carries the synthesis commit.
    let oid: String = conn
        .query_row(
            "SELECT oid FROM audit_events_cache WHERE op = 'sync_tag_written' LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(oid.len(), 40, "oid column should hold a 40-char SHA1, got {oid}");
}

#[tokio::test(flavor = "multi_thread")]
async fn helper_does_not_export_sync_tags() {
    // Ground-truth: the helper's protocol-v2 advertisement is generated by
    // `git upload-pack --advertise-refs --stateless-rpc <bare-repo>`. By
    // git's default, upload-pack advertises only refs/heads/, refs/tags/,
    // refs/notes/. Our private namespace (refs/reposix/sync/) is therefore
    // hidden.
    //
    // We test this directly: drive `git upload-pack --advertise-refs` over
    // the bare repo and grep for our sync-tag namespace. If git ever
    // changes default visibility, this test fails loudly.
    let server = MockServer::start().await;
    let issues = sample_issues("demo", 2);
    seed_mock(&server, "demo", &issues).await;

    let cache_root = tempfile::tempdir().unwrap();
    let _env = CacheDirGuard::new(cache_root.path());
    let backend: Arc<dyn BackendConnector> = sim_backend(&server);
    let cache = Cache::open(backend, "sim", "demo").expect("Cache::open");
    cache.sync().await.expect("seed sync");

    // Sanity: the tag exists in the bare repo.
    let tags = list_sync_tags_at(cache.repo_path()).expect("list_sync_tags_at");
    assert_eq!(tags.len(), 1);

    // Run upload-pack --advertise-refs and inspect what git would advertise.
    let out = std::process::Command::new("git")
        .args([
            "upload-pack",
            "--strict",
            "--advertise-refs",
            "--stateless-rpc",
        ])
        .arg(cache.repo_path())
        .output()
        .expect("spawn git upload-pack --advertise-refs");
    assert!(
        out.status.success(),
        "upload-pack --advertise-refs failed: stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let adv = String::from_utf8_lossy(&out.stdout);
    assert!(
        !adv.contains("refs/reposix/sync/"),
        "BUG: upload-pack advertised a sync tag — namespace is leaking to the agent.\nAdvertisement: {adv}"
    );
    // Sanity-check: the heads ref IS advertised (otherwise the test isn't
    // actually exercising upload-pack visibility at all).
    assert!(
        adv.contains("refs/heads/main"),
        "smoke check: refs/heads/main missing from advertisement: {adv}"
    );
}
