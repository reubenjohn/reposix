//! Phase v0.11.0 §3j — `Cache::gc` integration tests.
//!
//! Exercises the LRU / TTL / All strategies, dry-run mode, and the
//! "blob re-materialises on next read after eviction" round-trip.

#![allow(clippy::missing_panics_doc)]

use std::sync::Arc;

use reposix_cache::{Cache, GcStrategy};
use reposix_core::BackendConnector;
use wiremock::MockServer;

mod common;
use common::{sample_issues, seed_mock, sim_backend, CacheDirGuard};

/// Materialise every blob via `read_blob` so they exist as loose
/// objects on disk (otherwise the cache only has the lazy tree).
async fn materialize_all(cache: &Cache) {
    let oids: Vec<String> = {
        let conn = rusqlite::Connection::open(cache.repo_path().join("cache.db")).unwrap();
        let mut stmt = conn.prepare("SELECT oid FROM oid_map").unwrap();
        let v: Vec<String> = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .map(std::result::Result::unwrap)
            .collect();
        v
    };
    for oid_hex in oids {
        let oid = gix::ObjectId::from_hex(oid_hex.as_bytes()).unwrap();
        let _ = cache.read_blob(oid).await.unwrap();
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn gc_all_evicts_every_blob() {
    let server = MockServer::start().await;
    let issues = sample_issues("demo", 5);
    seed_mock(&server, "demo", &issues).await;

    let cache_root = tempfile::tempdir().unwrap();
    let _env = CacheDirGuard::new(cache_root.path());
    let backend: Arc<dyn BackendConnector> = sim_backend(&server);
    let cache = Cache::open(backend, "sim", "demo").expect("Cache::open");
    cache.sync().await.expect("seed sync");
    materialize_all(&cache).await;

    let report = cache.gc(GcStrategy::All, false).expect("gc");
    assert_eq!(report.count(), 5, "all 5 materialised blobs evicted");
    assert!(
        report.bytes_reclaimed() > 0,
        "must reclaim non-zero bytes from evicting real blobs"
    );

    // Trees + commits must still resolve — sanity check that gc
    // didn't touch non-blob objects.
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(cache.repo_path())
        .args(["rev-parse", "--verify", "refs/heads/main"])
        .output()
        .unwrap();
    assert!(out.status.success(), "main ref still resolves after gc");
}

#[tokio::test(flavor = "multi_thread")]
async fn gc_dry_run_doesnt_touch_disk() {
    let server = MockServer::start().await;
    let issues = sample_issues("demo", 3);
    seed_mock(&server, "demo", &issues).await;

    let cache_root = tempfile::tempdir().unwrap();
    let _env = CacheDirGuard::new(cache_root.path());
    let backend: Arc<dyn BackendConnector> = sim_backend(&server);
    let cache = Cache::open(backend, "sim", "demo").expect("Cache::open");
    cache.sync().await.expect("seed sync");
    materialize_all(&cache).await;

    // Capture file count before.
    let before = count_loose_objects(cache.repo_path());

    let report = cache.gc(GcStrategy::All, true).expect("gc dry-run");
    assert_eq!(
        report.count(),
        3,
        "dry-run reports planned evictions ({}); got {}",
        3,
        report.count()
    );
    assert!(report.dry_run, "dry_run flag preserved on report");

    let after = count_loose_objects(cache.repo_path());
    assert_eq!(before, after, "dry-run must NOT remove anything from disk");
}

#[tokio::test(flavor = "multi_thread")]
async fn gc_lru_respects_size_cap() {
    let server = MockServer::start().await;
    let issues = sample_issues("demo", 5);
    seed_mock(&server, "demo", &issues).await;

    let cache_root = tempfile::tempdir().unwrap();
    let _env = CacheDirGuard::new(cache_root.path());
    let backend: Arc<dyn BackendConnector> = sim_backend(&server);
    let cache = Cache::open(backend, "sim", "demo").expect("Cache::open");
    cache.sync().await.expect("seed sync");
    materialize_all(&cache).await;

    // Force eviction by setting a tiny size cap.
    let report = cache
        .gc(
            GcStrategy::Lru {
                max_size_bytes: 1, // 1 byte: forces eviction of all but 0 / a few
            },
            false,
        )
        .expect("gc lru");
    // We don't know exactly how many will be evicted (depends on the
    // sum-vs-cap arithmetic), but at least one must be evicted because
    // the cap is below the per-blob size.
    assert!(
        !report.evicted.is_empty(),
        "lru with cap=1 must evict at least one blob"
    );
    assert!(
        report.bytes_after <= report.bytes_before,
        "post-eviction size must not exceed pre-eviction"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn gc_lru_with_huge_cap_evicts_nothing() {
    let server = MockServer::start().await;
    let issues = sample_issues("demo", 3);
    seed_mock(&server, "demo", &issues).await;

    let cache_root = tempfile::tempdir().unwrap();
    let _env = CacheDirGuard::new(cache_root.path());
    let backend: Arc<dyn BackendConnector> = sim_backend(&server);
    let cache = Cache::open(backend, "sim", "demo").expect("Cache::open");
    cache.sync().await.expect("seed sync");
    materialize_all(&cache).await;

    let report = cache
        .gc(
            GcStrategy::Lru {
                max_size_bytes: u64::MAX,
            },
            false,
        )
        .expect("gc lru huge");
    assert_eq!(
        report.count(),
        0,
        "lru with cap=u64::MAX should evict nothing, got {:?}",
        report.evicted
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn gc_ttl_zero_days_evicts_everything() {
    let server = MockServer::start().await;
    let issues = sample_issues("demo", 3);
    seed_mock(&server, "demo", &issues).await;

    let cache_root = tempfile::tempdir().unwrap();
    let _env = CacheDirGuard::new(cache_root.path());
    let backend: Arc<dyn BackendConnector> = sim_backend(&server);
    let cache = Cache::open(backend, "sim", "demo").expect("Cache::open");
    cache.sync().await.expect("seed sync");
    materialize_all(&cache).await;

    // max_age_days=0 means "evict anything older than now" — but mtime
    // is "now" exactly, and the cutoff comparison is strict <. Sleep
    // briefly to ensure mtime is strictly older than the cutoff.
    std::thread::sleep(std::time::Duration::from_millis(50));

    let report = cache
        .gc(GcStrategy::Ttl { max_age_days: 0 }, false)
        .expect("gc ttl");
    assert_eq!(
        report.count(),
        3,
        "ttl=0 (post-sleep) should evict everything"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn gc_ttl_long_window_keeps_everything() {
    let server = MockServer::start().await;
    let issues = sample_issues("demo", 2);
    seed_mock(&server, "demo", &issues).await;

    let cache_root = tempfile::tempdir().unwrap();
    let _env = CacheDirGuard::new(cache_root.path());
    let backend: Arc<dyn BackendConnector> = sim_backend(&server);
    let cache = Cache::open(backend, "sim", "demo").expect("Cache::open");
    cache.sync().await.expect("seed sync");
    materialize_all(&cache).await;

    let report = cache
        .gc(
            GcStrategy::Ttl {
                max_age_days: 365 * 100,
            },
            false,
        )
        .expect("gc ttl long");
    assert_eq!(
        report.count(),
        0,
        "100-year TTL should evict nothing on a freshly-materialised cache"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn gc_audit_row_per_eviction() {
    let server = MockServer::start().await;
    let issues = sample_issues("demo", 4);
    seed_mock(&server, "demo", &issues).await;

    let cache_root = tempfile::tempdir().unwrap();
    let _env = CacheDirGuard::new(cache_root.path());
    let backend: Arc<dyn BackendConnector> = sim_backend(&server);
    let cache = Cache::open(backend, "sim", "demo").expect("Cache::open");
    cache.sync().await.expect("seed sync");
    materialize_all(&cache).await;

    cache.gc(GcStrategy::All, false).expect("gc");

    let conn = rusqlite::Connection::open(cache.repo_path().join("cache.db")).unwrap();
    let n: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM audit_events_cache WHERE op = 'cache_gc'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(n, 4, "one cache_gc audit row per evicted blob");

    // Spot-check the reason payload encodes the strategy.
    let reason: String = conn
        .query_row(
            "SELECT reason FROM audit_events_cache WHERE op = 'cache_gc' LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(
        reason.contains("strategy=all"),
        "reason should encode strategy: {reason}"
    );
    assert!(
        reason.starts_with("evicted:"),
        "non-dry-run rows tagged `evicted:`: {reason}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn gc_evicted_blob_remateralizes_on_read() {
    let server = MockServer::start().await;
    let issues = sample_issues("demo", 2);
    seed_mock(&server, "demo", &issues).await;

    let cache_root = tempfile::tempdir().unwrap();
    let _env = CacheDirGuard::new(cache_root.path());
    let backend: Arc<dyn BackendConnector> = sim_backend(&server);
    let cache = Cache::open(backend, "sim", "demo").expect("Cache::open");
    cache.sync().await.expect("seed sync");
    materialize_all(&cache).await;

    // Read one blob + remember its OID.
    let oids: Vec<String> = {
        let conn = rusqlite::Connection::open(cache.repo_path().join("cache.db")).unwrap();
        let mut stmt = conn
            .prepare("SELECT oid FROM oid_map ORDER BY oid LIMIT 1")
            .unwrap();
        let v: Vec<String> = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .map(std::result::Result::unwrap)
            .collect();
        v
    };
    let target_oid = gix::ObjectId::from_hex(oids[0].as_bytes()).unwrap();

    cache.gc(GcStrategy::All, false).expect("gc");

    // After eviction, read_blob should re-fetch transparently.
    let bytes = cache
        .read_blob(target_oid)
        .await
        .expect("re-fetch after gc");
    assert!(!bytes.inner_ref().is_empty(), "blob re-fetched non-empty");
}

#[tokio::test(flavor = "multi_thread")]
async fn gc_never_evicts_tree_or_commit_objects() {
    let server = MockServer::start().await;
    let issues = sample_issues("demo", 3);
    seed_mock(&server, "demo", &issues).await;

    let cache_root = tempfile::tempdir().unwrap();
    let _env = CacheDirGuard::new(cache_root.path());
    let backend: Arc<dyn BackendConnector> = sim_backend(&server);
    let cache = Cache::open(backend, "sim", "demo").expect("Cache::open");
    cache.sync().await.expect("seed sync");
    materialize_all(&cache).await;

    let trees_before = count_objects_of_type(cache.repo_path(), "tree");
    let commits_before = count_objects_of_type(cache.repo_path(), "commit");
    assert!(trees_before > 0, "must have trees before gc");
    assert!(commits_before > 0, "must have commits before gc");

    cache.gc(GcStrategy::All, false).expect("gc");

    let trees_after = count_objects_of_type(cache.repo_path(), "tree");
    let commits_after = count_objects_of_type(cache.repo_path(), "commit");
    assert_eq!(trees_after, trees_before, "gc must not touch trees");
    assert_eq!(commits_after, commits_before, "gc must not touch commits");
}

// ---------- helpers -------------------------------------------------------

fn count_loose_objects(cache_path: &std::path::Path) -> usize {
    let mut count = 0;
    let objects = cache_path.join("objects");
    let Ok(prefix_iter) = std::fs::read_dir(&objects) else {
        return 0;
    };
    for prefix_entry in prefix_iter.flatten() {
        let p = prefix_entry.path();
        if !p.is_dir() {
            continue;
        }
        let name = match prefix_entry.file_name().to_str() {
            Some(s) => s.to_owned(),
            None => continue,
        };
        if name.len() != 2 {
            continue;
        }
        if let Ok(it) = std::fs::read_dir(&p) {
            count += it.flatten().filter(|e| e.path().is_file()).count();
        }
    }
    count
}

fn count_objects_of_type(cache_path: &std::path::Path, kind: &str) -> usize {
    // Walk loose objects and ask `git cat-file -t` for each.
    let mut count = 0;
    let objects = cache_path.join("objects");
    let Ok(prefix_iter) = std::fs::read_dir(&objects) else {
        return 0;
    };
    for prefix_entry in prefix_iter.flatten() {
        let pname = match prefix_entry.file_name().to_str() {
            Some(s) => s.to_owned(),
            None => continue,
        };
        if pname.len() != 2 {
            continue;
        }
        let p = prefix_entry.path();
        if let Ok(it) = std::fs::read_dir(&p) {
            for entry in it.flatten() {
                let oname = match entry.file_name().to_str() {
                    Some(s) => s.to_owned(),
                    None => continue,
                };
                if oname.len() != 38 {
                    continue;
                }
                let oid = format!("{pname}{oname}");
                let out = std::process::Command::new("git")
                    .arg("-C")
                    .arg(cache_path)
                    .args(["cat-file", "-t", &oid])
                    .output()
                    .unwrap();
                if !out.status.success() {
                    continue;
                }
                let t = String::from_utf8_lossy(&out.stdout).trim().to_owned();
                if t == kind {
                    count += 1;
                }
            }
        }
    }
    count
}
