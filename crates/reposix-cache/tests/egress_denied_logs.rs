//! ARCH-03: non-allowlisted backend origin -> `Error::Egress` +
//! `op='egress_denied'` audit row.
//!
//! We use a stub [`BackendConnector`] whose `get_issue` always returns
//! `reposix_core::Error::InvalidOrigin`, simulating the allowlist gate
//! firing. `list_issues` delegates to a real `SimBackend` so
//! `build_from` can seed the `oid_map`.

mod common;

use std::sync::Arc;

use async_trait::async_trait;
use common::CacheDirGuard;
use reposix_cache::Cache;
use reposix_core::backend::{BackendConnector, BackendFeature, DeleteReason};
use reposix_core::{Error as CoreError, Record, RecordId, Result as CoreResult, Untainted};
use tempfile::tempdir;
use wiremock::MockServer;

/// Stub backend whose `get_issue` always returns
/// `Error::InvalidOrigin`. `list_issues` delegates to the inner
/// simulator so [`Cache::build_from`] can still seed `oid_map`.
struct EgressRejectingBackend {
    inner: Arc<dyn BackendConnector>,
}

#[async_trait]
impl BackendConnector for EgressRejectingBackend {
    fn name(&self) -> &'static str {
        "egress-rejecting-stub"
    }
    fn supports(&self, _feature: BackendFeature) -> bool {
        false
    }
    async fn list_issues(&self, project: &str) -> CoreResult<Vec<Record>> {
        self.inner.list_issues(project).await
    }
    async fn get_issue(&self, _project: &str, _id: RecordId) -> CoreResult<Record> {
        Err(CoreError::InvalidOrigin("https://evil.example:443/".into()))
    }
    async fn create_issue(&self, _: &str, _: Untainted<Record>) -> CoreResult<Record> {
        Err(CoreError::Other("unsupported in stub".into()))
    }
    async fn update_issue(
        &self,
        _project: &str,
        _id: RecordId,
        _issue: Untainted<Record>,
        _expected_version: Option<u64>,
    ) -> CoreResult<Record> {
        Err(CoreError::Other("unsupported in stub".into()))
    }
    async fn delete_or_close(&self, _: &str, _: RecordId, _: DeleteReason) -> CoreResult<()> {
        Err(CoreError::Other("unsupported in stub".into()))
    }
}

#[tokio::test]
async fn egress_denied_writes_audit_row_and_returns_egress_error() {
    let tmp = tempdir().unwrap();
    let _g = CacheDirGuard::new(tmp.path());

    let server = MockServer::start().await;
    let issues = common::sample_issues("proj-egress", 3);
    common::seed_mock(&server, "proj-egress", &issues).await;

    let inner = common::sim_backend(&server);
    let backend: Arc<dyn BackendConnector> = Arc::new(EgressRejectingBackend { inner });
    let cache = Cache::open(backend, "sim", "proj-egress").unwrap();
    cache.build_from().await.unwrap();

    // Pick any oid from the map.
    let db = rusqlite::Connection::open(cache.repo_path().join("cache.db")).unwrap();
    let oid_hex: String = db
        .query_row("SELECT oid FROM oid_map LIMIT 1", [], |r| r.get(0))
        .unwrap();
    let oid = gix::ObjectId::from_hex(oid_hex.as_bytes()).unwrap();

    // read_blob MUST return Error::Egress.
    let res = cache.read_blob(oid).await;
    match res {
        Err(reposix_cache::Error::Egress(_)) => {}
        other => panic!("expected Error::Egress, got {other:?}"),
    }

    // Audit row must exist with op='egress_denied'.
    let denied_count: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM audit_events_cache WHERE op = 'egress_denied'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(denied_count, 1);

    // No `materialize` row was written.
    let mat_count: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM audit_events_cache WHERE op = 'materialize'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(mat_count, 0);
}
