//! FIX-01 / FIX-02 — reproduction-backed proof of the Confluence
//! list-vs-get oid-drift class AND of what `reposix sync --reconcile`
//! can (and cannot) recover.
//!
//! The hazard this file locks in: a backend whose `list_records` render
//! of an id disagrees with its `get_record` render of the SAME id (the
//! pre-fix Confluence LIST path omitted `body-format`, so listed pages
//! carried an EMPTY body while `get_record` returned the real ADF body).
//! `build_from` computes each `oid_map` entry from the LIST body
//! (`builder.rs`: render → `compute_hash`), so the stored oid is derived
//! from the empty body; `read_blob` then fetches via `get_record` (real
//! body), re-renders, and the `written_oid != oid` drift guard fires.
//!
//! `DriftingMock` is backend-agnostic — the divergence lives in
//! `Record.body`, not in membership (contrast `pagination_prune_safety.rs`'s
//! `CappingMock`, which diverges by membership). Three tests:
//!
//! - `pre_fix_divergent_bodies_trigger_oid_drift` (FIX-01 mechanism):
//!   divergent list/get bodies → `read_blob` aborts with `Error::OidDrift`.
//! - `reconcile_does_not_clear_stale_list_oid_while_bodies_diverge`
//!   (FIX-02 non-recovery): a SECOND `build_from()` — exactly what
//!   `reposix sync --reconcile` runs (`sync.rs::run` calls `Cache::build_from`
//!   directly) — leaves the stale list-derived oid UNCHANGED and `read_blob`
//!   still errors. Empirical proof (not assumption) that `--reconcile`
//!   CANNOT recover the systematic list-vs-get class — the doc scope in
//!   `error.rs`/`sync.rs` is backed by a reproduction (SC4).
//! - `aligned_bodies_resolve_without_drift` (FIX-01 resolution): once the
//!   list body matches the get body (the post-FIX-01 render-parity state),
//!   `read_blob` resolves cleanly and yields the real record's bytes.
//!
//! This file also standing-asserts the drift guard is NOT weakened
//! (T-114-04): silence the `written_oid != oid` check and tests (a)/(b)
//! turn RED.

#![allow(clippy::missing_panics_doc)]

mod common;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::{sample_issues, CacheDirGuard};
use reposix_cache::{Cache, Error};
use reposix_core::backend::{BackendConnector, BackendFeature, DeleteReason, Listing};
use reposix_core::frontmatter;
use reposix_core::{Error as CoreError, Record, RecordId, Result as CoreResult, Untainted};
use tempfile::tempdir;

/// A `BackendConnector` whose LIST render of an id can disagree with its
/// GET render of the same id — the exact real-connector condition the sim
/// (which serves identical bytes from both paths) can never reproduce.
///
/// `full` is the TRUE backend state (real bodies); `get_record` ALWAYS
/// resolves against it. When `aligned` is false (the default, modelling the
/// pre-fix Confluence LIST path), `list_records` returns records with an
/// EMPTY body; when true (the post-FIX-01 render-parity state), the list
/// body matches `full`.
struct DriftingMock {
    /// The true backend state — real bodies. `get_record` resolves here
    /// unconditionally (the real Confluence `get_record` ADF render).
    full: Vec<Record>,
    /// Whether the LIST render matches the GET render. `false` (default)
    /// clears the list body to `String::new()` (pre-fix); `true` returns
    /// the real body (post-FIX-01 parity).
    aligned: AtomicBool,
}

impl DriftingMock {
    fn new(full: Vec<Record>) -> Self {
        Self {
            full,
            aligned: AtomicBool::new(false),
        }
    }

    /// Flip to the post-FIX-01 state: the LIST body matches the GET body.
    fn align(&self) {
        self.aligned.store(true, Ordering::SeqCst);
    }

    /// The listing view: real bodies when aligned, empty bodies otherwise.
    fn listed(&self) -> Vec<Record> {
        if self.aligned.load(Ordering::SeqCst) {
            self.full.clone()
        } else {
            self.full
                .iter()
                .map(|r| {
                    let mut r = r.clone();
                    r.body = String::new(); // pre-fix: listed pages carry no body
                    r
                })
                .collect()
        }
    }
}

#[async_trait]
impl BackendConnector for DriftingMock {
    fn name(&self) -> &'static str {
        "drifting-mock"
    }
    fn supports(&self, _feature: BackendFeature) -> bool {
        false
    }
    async fn list_records(&self, _project: &str) -> CoreResult<Vec<Record>> {
        Ok(self.listed())
    }
    /// Report the listing COMPLETE — this test isolates the render-drift
    /// class from the pagination-prune class (`pagination_prune_safety.rs`);
    /// completeness must not be the variable under test here.
    async fn list_records_complete(&self, _project: &str) -> CoreResult<Listing> {
        Ok(Listing {
            records: self.listed(),
            is_complete: true,
        })
    }
    async fn list_changed_since(
        &self,
        _project: &str,
        _since: DateTime<Utc>,
    ) -> CoreResult<Vec<RecordId>> {
        Ok(vec![])
    }
    async fn get_record(&self, _project: &str, id: RecordId) -> CoreResult<Record> {
        // Always the REAL body — a listed id whose body was cleared is
        // still fully fetchable with its true content.
        self.full
            .iter()
            .find(|r| r.id == id)
            .cloned()
            .ok_or_else(|| CoreError::NotFound {
                project: "demo".into(),
                id: id.0.to_string(),
            })
    }
    async fn create_record(&self, _: &str, _: Untainted<Record>) -> CoreResult<Record> {
        Err(CoreError::Other("unsupported in drifting-mock".into()))
    }
    async fn update_record(
        &self,
        _: &str,
        _: RecordId,
        _: Untainted<Record>,
        _: Option<u64>,
    ) -> CoreResult<Record> {
        Err(CoreError::Other("unsupported in drifting-mock".into()))
    }
    async fn delete_or_close(&self, _: &str, _: RecordId, _: DeleteReason) -> CoreResult<()> {
        Err(CoreError::Other("unsupported in drifting-mock".into()))
    }
}

/// Read issue `issue_id`'s recorded `oid_map` oid (hex) for this mock's
/// `(backend="mock", project="demo")` identity — the `build_from`-time oid,
/// derived from the LIST body.
fn oid_hex_for(cache: &Cache, issue_id: &str) -> String {
    let db = rusqlite::Connection::open(cache.repo_path().join("cache.db")).unwrap();
    let oid_hex: String = db
        .query_row(
            "SELECT oid FROM oid_map WHERE issue_id = ?1 AND backend = 'mock' AND project = 'demo'",
            rusqlite::params![issue_id],
            |r| r.get(0),
        )
        .unwrap();
    oid_hex
}

/// FIX-01 mechanism / reproduction: divergent LIST vs GET bodies for the
/// SAME id make `read_blob` abort with `Error::OidDrift`.
///
/// `build_from` stores the empty-body-derived oid in `oid_map`; `read_blob`
/// fetches the real body via `get_record`, re-renders, and the
/// `written_oid != oid` guard (`builder.rs`) fires. Proves the guard is live
/// (T-114-04) and reproduces the FIX-01 defect on a backend-agnostic mock.
#[tokio::test]
async fn pre_fix_divergent_bodies_trigger_oid_drift() {
    let tmp = tempdir().unwrap();
    let _g = CacheDirGuard::new(tmp.path());

    let issues = sample_issues("demo", 3);
    let backend = Arc::new(DriftingMock::new(issues)); // NOT aligned
    let cache = Cache::open(backend.clone(), "mock", "demo").unwrap();

    // build_from renders the EMPTY-body list records → oid_map holds the
    // empty-body-derived oid for each id (the pre-fix Confluence LIST state).
    cache.build_from().await.expect("build_from");

    let oid = gix::ObjectId::from_hex(oid_hex_for(&cache, "1").as_bytes()).unwrap();

    // read_blob calls get_record (REAL body) → re-renders → written_oid ≠ the
    // empty-body oid in oid_map → the drift guard fires for issue 1.
    match cache.read_blob(oid).await {
        Err(Error::OidDrift { issue_id, .. }) => {
            assert_eq!(
                issue_id, "1",
                "OidDrift must name the issue whose LIST/GET bodies diverged"
            );
        }
        other => panic!("expected OidDrift, got {other:?}"),
    }
}

/// FIX-02 non-recovery: a SECOND `build_from()` — exactly what `reposix sync
/// --reconcile` runs (`sync.rs::run` → `Cache::build_from`) — leaves the
/// stale list-derived oid UNCHANGED while list/get bodies still diverge, and
/// `read_blob` STILL aborts with `OidDrift`.
///
/// This is the empirical backing for the corrected doc scope: `--reconcile`
/// re-lists the SAME empty body and recomputes the SAME oid, so it CANNOT
/// heal the systematic list-vs-get class (SC4) — proven against a
/// reproduction, not asserted.
#[tokio::test]
async fn reconcile_does_not_clear_stale_list_oid_while_bodies_diverge() {
    let tmp = tempdir().unwrap();
    let _g = CacheDirGuard::new(tmp.path());

    let issues = sample_issues("demo", 3);
    let backend = Arc::new(DriftingMock::new(issues)); // NOT aligned
    let cache = Cache::open(backend.clone(), "mock", "demo").unwrap();

    // First build_from → stale empty-body oid in oid_map.
    cache.build_from().await.expect("first build_from");
    let oid_a = oid_hex_for(&cache, "1");

    // A second build_from IS `reposix sync --reconcile` (a forced full
    // list_records walk + rebuild). The mock is still NOT aligned — the list
    // body is still empty — so the recomputed oid is byte-identical.
    cache.build_from().await.expect("reconcile build_from");
    let oid_b = oid_hex_for(&cache, "1");

    assert_eq!(
        oid_a, oid_b,
        "reconcile (a second build_from) must leave the stale list-derived oid \
         UNCHANGED while list/get bodies still diverge: it re-lists the SAME \
         empty body and recomputes the SAME oid, so it CANNOT heal this class"
    );

    // And the blob STILL cannot be materialized — reconcile did not recover it.
    let oid = gix::ObjectId::from_hex(oid_b.as_bytes()).unwrap();
    match cache.read_blob(oid).await {
        Err(Error::OidDrift { issue_id, .. }) => {
            assert_eq!(
                issue_id, "1",
                "OidDrift must persist for issue 1 after a reconcile that could not heal it"
            );
        }
        other => panic!("expected OidDrift after reconcile, got {other:?}"),
    }
}

/// FIX-01 resolution: once the LIST body matches the GET body (the
/// post-FIX-01 render-parity state), `read_blob` resolves cleanly and the
/// materialized bytes equal the canonical render of the real record.
#[tokio::test]
async fn aligned_bodies_resolve_without_drift() {
    let tmp = tempdir().unwrap();
    let _g = CacheDirGuard::new(tmp.path());

    let issues = sample_issues("demo", 3);
    let backend = Arc::new(DriftingMock::new(issues));
    backend.align(); // list body == get body (post-FIX-01 parity)
    let cache = Cache::open(backend.clone(), "mock", "demo").unwrap();

    cache.build_from().await.expect("build_from");

    let oid = gix::ObjectId::from_hex(oid_hex_for(&cache, "1").as_bytes()).unwrap();

    let bytes = cache
        .read_blob(oid)
        .await
        .expect("aligned bodies must resolve without drift")
        .into_inner();

    let expected = frontmatter::render(&backend.full[0]).unwrap().into_bytes();
    assert_eq!(
        bytes, expected,
        "resolved bytes must equal the real record's canonical render"
    );
}
