//! QL-001 Assertion-2 root-cause regression: `find_oid_for_record` must
//! return the CURRENT blob oid for a record, not a stale historical one.
//!
//! `oid_map`'s PRIMARY KEY is `oid` (the blob hash), so `issue_id` is
//! non-unique. `put_oid_mapping`'s `INSERT OR REPLACE (oid, issue_id, …)` keys
//! on the oid, so each time a record's content changes, `build_from` inserts a
//! NEW `(new_oid, issue_id)` row while the prior `(old_oid, issue_id)` row
//! survives (intentional — the reverse `get_issue_for_oid` lookup needs the
//! history so a lazy fetch of any historical blob resolves). But the forward
//! lookup must return the record's CURRENT oid. Without an `ORDER BY`, SQLite
//! returned the first-inserted (stale) row, so after a push bumped a record the
//! L1 precheck's Step-5 prior read the OLD version's blob and diffed the
//! freshly-merged working tree against it — emitting a spurious PATCH the
//! backend rejected with a 409 version mismatch (the extra mutation that
//! reddened `agent-ux/real-git-push-e2e` Assertion 2).
//!
//! RED against the pre-fix query (returns the v1 oid); GREEN once the query
//! orders `rowid DESC LIMIT 1`.

#![forbid(unsafe_code)]

mod common;

use common::{sample_issues, seed_mock, sim_backend, CacheDirGuard};
use reposix_cache::Cache;
use tempfile::tempdir;
use wiremock::MockServer;

#[tokio::test]
async fn find_oid_for_record_returns_current_not_stale_oid() {
    let tmp = tempdir().unwrap();
    let _g = CacheDirGuard::new(tmp.path());
    let project = "proj-1";

    // 1. Backend at v1: build the cache → oid_map gets issue 1's v1 oid.
    let server = MockServer::start().await;
    let mut issues = sample_issues(project, 1); // issue 1, version 1, "body of issue 1"
    seed_mock(&server, project, &issues).await;
    let cache = Cache::open(sim_backend(&server), "sim", project).unwrap();
    cache.build_from().await.expect("seed build (v1)");
    let v1_oid = cache
        .find_oid_for_record(reposix_core::RecordId(1))
        .expect("query")
        .expect("issue 1 mapped after v1 build");

    // 2. Backend advances to v2 (new body + bumped version), as a push would
    //    leave it. Re-seed the wiremock and rebuild the cache: build_from now
    //    inserts issue 1's v2 oid ALONGSIDE the surviving v1 row.
    issues[0].version = 2;
    issues[0].body = "EDITED body of issue 1 — e2e edit".to_owned();
    issues[0].updated_at = chrono::Utc::now();
    server.reset().await;
    seed_mock(&server, project, &issues).await;
    cache.build_from().await.expect("rebuild (v2)");

    // 3. The forward lookup MUST now resolve to the v2 (current) oid — a lazy
    //    read of it drives the fresh prior. Pre-fix this returned v1_oid.
    let current = cache
        .find_oid_for_record(reposix_core::RecordId(1))
        .expect("query")
        .expect("issue 1 still mapped after v2 build");
    assert_ne!(
        current, v1_oid,
        "find_oid_for_record must NOT return the stale v1 oid after the record \
         changed to v2 (this is the QL-001 Assertion-2 cache-desync: a stale \
         prior makes the push planner emit a spurious, 409-rejected PATCH)"
    );

    // Cross-check: the current oid matches a fresh render of the v2 record, and
    // the v1 oid is still resolvable in reverse (history retained for lazy fetch).
    let v2_rendered = reposix_core::frontmatter::render(&issues[0]).expect("render v2");
    let expected_v2_oid = gix::objs::compute_hash(
        gix::hash::Kind::Sha1,
        gix::object::Kind::Blob,
        v2_rendered.as_bytes(),
    )
    .expect("hash v2");
    assert_eq!(
        current, expected_v2_oid,
        "the current oid must be the v2 blob's hash"
    );
    assert_ne!(v1_oid, expected_v2_oid, "sanity: v1 and v2 oids differ");
}
