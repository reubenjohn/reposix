//! Unit tests for [`Cache::read_blob_cached`] (DVCS-PERF-L1-01 / H1 fix).
//!
//! `read_blob_cached` is the sync gix-only inspector — the L1 precheck
//! path uses it instead of the async `read_blob` to avoid a backend
//! GET per cache prior. Returns `Ok(None)` on cache miss instead of
//! fetching.

#![allow(clippy::missing_panics_doc)]

use std::sync::Arc;

use reposix_cache::Cache;
use reposix_core::backend::sim::SimBackend;
use reposix_core::BackendConnector;
use wiremock::MockServer;

mod common;
use common::{sample_issues, seed_mock, CacheDirGuard};

fn sim_backend(server: &MockServer) -> Arc<dyn BackendConnector> {
    Arc::new(SimBackend::new(server.uri()).expect("SimBackend::new"))
}

#[tokio::test]
async fn read_blob_cached_returns_some_when_blob_in_repo() {
    // After a Cache::sync, blobs for materialized issues live in the
    // cache's bare repo. read_blob_cached resolves their OIDs locally
    // (no backend egress) and returns the bytes.
    let server = MockServer::start().await;
    let issues = sample_issues("demo", 3);
    seed_mock(&server, "demo", &issues).await;

    let cache_root = tempfile::tempdir().expect("tempdir");
    let _env = CacheDirGuard::new(cache_root.path());
    let backend = sim_backend(&server);
    let cache = Cache::open(backend.clone(), "sim", "demo").expect("Cache::open");
    cache.sync().await.expect("seed sync");

    // Materialize a blob via the existing async path so its OID
    // resolves locally. We then use read_blob_cached against the same
    // OID and assert the bytes match.
    let id = issues[0].id;
    let oid = cache
        .find_oid_for_record(id)
        .expect("find_oid_for_record")
        .expect("oid present after sync");
    let materialized = cache.read_blob(oid).await.expect("read_blob materialize");

    let cached = cache.read_blob_cached(oid).expect("read_blob_cached");
    let cached = cached.expect("blob present after materialize");
    assert_eq!(
        cached.inner_ref(),
        materialized.inner_ref(),
        "read_blob_cached bytes must match the async read_blob bytes"
    );
}

#[tokio::test]
async fn read_blob_cached_returns_none_when_blob_absent() {
    // A made-up OID that doesn't resolve to any object in the cache's
    // bare repo — read_blob_cached returns Ok(None), NOT an error and
    // NOT a backend GET.
    let server = MockServer::start().await;
    let cache_root = tempfile::tempdir().expect("tempdir");
    let _env = CacheDirGuard::new(cache_root.path());
    let backend = sim_backend(&server);
    let cache = Cache::open(backend, "sim", "demo").expect("Cache::open");

    // 40-char hex string of zeros — a deterministic invalid OID. gix
    // resolves it as "not found" (no exotic discriminant).
    let bogus: gix::ObjectId =
        gix::ObjectId::from_hex(b"0000000000000000000000000000000000000001").expect("parse oid");
    let result = cache
        .read_blob_cached(bogus)
        .expect("read_blob_cached should not error for missing oid");
    assert!(
        result.is_none(),
        "expected None for missing oid; got Some(_)"
    );
}
