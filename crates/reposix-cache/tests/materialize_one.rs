//! ARCH-02: one `read_blob` = one blob in `.git/objects` + one
//! materialize audit row; return type is `Tainted<Vec<u8>>`.

mod common;

use common::CacheDirGuard;
use gix::prelude::FindExt as _;
use reposix_cache::Cache;
use reposix_core::frontmatter;
use reposix_core::RecordId;
use tempfile::tempdir;
use wiremock::MockServer;

#[tokio::test]
async fn read_blob_materializes_exactly_one_and_audits() {
    let tmp = tempdir().unwrap();
    let _g = CacheDirGuard::new(tmp.path());

    let server = MockServer::start().await;
    let issues = common::sample_issues("proj-1", 5);
    common::seed_mock(&server, "proj-1", &issues).await;

    let cache = Cache::open(common::sim_backend(&server), "sim", "proj-1").unwrap();
    cache.build_from().await.unwrap();

    // Pick issue 1's OID from the oid_map — tests read the SQL
    // directly because the plan does not yet expose a public helper.
    let db = rusqlite::Connection::open(cache.repo_path().join("cache.db")).unwrap();
    let oid_hex: String = db
        .query_row(
            "SELECT oid FROM oid_map WHERE issue_id = '1' AND backend = 'sim' AND project = 'proj-1'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let oid = gix::ObjectId::from_hex(oid_hex.as_bytes()).unwrap();
    drop(db);

    // Count objects before.
    let repo = gix::open(cache.repo_path()).unwrap();
    let blob_before: usize = repo
        .objects
        .iter()
        .unwrap()
        .filter_map(std::result::Result::ok)
        .filter(|o| {
            let mut buf = Vec::new();
            let d = repo.objects.find(o, &mut buf).expect("find object");
            d.kind == gix::object::Kind::Blob
        })
        .count();
    assert_eq!(
        blob_before, 0,
        "Plan 01 invariant: no blobs before read_blob"
    );
    drop(repo);

    // Materialize.
    let tainted = cache.read_blob(oid).await.expect("read_blob succeeds");

    // Type is `Tainted<Vec<u8>>` — proven at compile time. At runtime
    // the bytes match `frontmatter::render` of issue 1.
    let inner = tainted.into_inner();
    let issue_1 = common::sim_backend(&server)
        .get_issue("proj-1", RecordId(1))
        .await
        .unwrap();
    let expected = frontmatter::render(&issue_1).unwrap();
    assert_eq!(inner, expected.into_bytes());

    // One blob object now exists.
    let repo = gix::open(cache.repo_path()).unwrap();
    let blob_after: usize = repo
        .objects
        .iter()
        .unwrap()
        .filter_map(std::result::Result::ok)
        .filter(|o| {
            let mut buf = Vec::new();
            let d = repo.objects.find(o, &mut buf).expect("find object");
            d.kind == gix::object::Kind::Blob
        })
        .count();
    assert_eq!(blob_after, 1, "exactly one blob after read_blob");

    // Exactly one materialize audit row.
    let db = rusqlite::Connection::open(cache.repo_path().join("cache.db")).unwrap();
    let mat_count: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM audit_events_cache WHERE op = 'materialize'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(mat_count, 1);

    // Second read_blob on same OID: blob count stays at 1 (content-
    // addressed); audit count goes to 2.
    let _ = cache.read_blob(oid).await.unwrap();
    let mat_count2: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM audit_events_cache WHERE op = 'materialize'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        mat_count2, 2,
        "second read_blob fires a second materialize audit row"
    );

    // A tree_sync audit row was written by build_from.
    let sync_count: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM audit_events_cache WHERE op = 'tree_sync'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(sync_count, 1);
}

#[tokio::test]
async fn unknown_oid_returns_error() {
    let tmp = tempdir().unwrap();
    let _g = CacheDirGuard::new(tmp.path());

    let server = MockServer::start().await;
    let issues = common::sample_issues("proj-unk", 2);
    common::seed_mock(&server, "proj-unk", &issues).await;

    let cache = Cache::open(common::sim_backend(&server), "sim", "proj-unk").unwrap();
    cache.build_from().await.unwrap();

    // Random oid not in oid_map.
    let bogus = gix::ObjectId::from_hex(b"0123456789abcdef0123456789abcdef01234567").unwrap();
    let err = cache
        .read_blob(bogus)
        .await
        .expect_err("unknown oid must err");
    assert!(
        matches!(err, reposix_cache::Error::UnknownOid(_)),
        "expected UnknownOid, got {err:?}"
    );
}
