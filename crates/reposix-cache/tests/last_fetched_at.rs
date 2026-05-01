//! Unit tests for [`Cache::read_last_fetched_at`] / [`Cache::write_last_fetched_at`]
//! (DVCS-PERF-L1-01).
//!
//! These wrappers around `meta::set_meta` / `meta::get_meta` are the
//! sync gix-local primitives the helper's L1 precheck consumes on push
//! entry (`crates/reposix-remote/src/precheck.rs`). The contract:
//!
//! - Round-trip: write `T1`, read back `T1` (RFC3339 second-precision).
//! - Absent cursor → `Ok(None)` (fresh cache, no `build_from` yet).
//! - Malformed cursor → `Ok(None)` (defensive WARN-log fallback).

#![allow(clippy::missing_panics_doc)]

use std::sync::Arc;

use reposix_cache::Cache;
use reposix_core::backend::sim::SimBackend;
use reposix_core::BackendConnector;
use wiremock::MockServer;

mod common;
use common::CacheDirGuard;

fn sim_backend(server: &MockServer) -> Arc<dyn BackendConnector> {
    Arc::new(SimBackend::new(server.uri()).expect("SimBackend::new"))
}

#[tokio::test]
async fn read_last_fetched_at_round_trips() {
    let server = MockServer::start().await;
    let cache_root = tempfile::tempdir().expect("tempdir");
    let _env = CacheDirGuard::new(cache_root.path());
    let backend = sim_backend(&server);
    let cache = Cache::open(backend, "sim", "demo").expect("Cache::open");

    // Use second precision so to_rfc3339 + parse_from_rfc3339 round-trip exactly.
    let t1: chrono::DateTime<chrono::Utc> = "2026-05-01T12:34:56Z".parse().expect("parse t1");
    cache
        .write_last_fetched_at(t1)
        .expect("write_last_fetched_at");
    let read_back = cache
        .read_last_fetched_at()
        .expect("read_last_fetched_at")
        .expect("cursor present after write");
    assert_eq!(read_back, t1);
}

#[tokio::test]
async fn read_last_fetched_at_returns_none_when_absent() {
    let server = MockServer::start().await;
    let cache_root = tempfile::tempdir().expect("tempdir");
    let _env = CacheDirGuard::new(cache_root.path());
    let backend = sim_backend(&server);
    let cache = Cache::open(backend, "sim", "demo").expect("Cache::open");
    let result = cache
        .read_last_fetched_at()
        .expect("read should succeed even when cursor absent");
    assert!(
        result.is_none(),
        "expected None for fresh cache; got {result:?}"
    );
}
