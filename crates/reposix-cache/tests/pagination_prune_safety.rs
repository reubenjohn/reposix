//! P94 D1 — pagination-truncation prune-safety (DATA-LOSS guard).
//!
//! The hazard (SURPRISES-INTAKE.md, HIGH; ratified fix
//! `.planning/CONSULT-DECISIONS.md` 2026-07-05 [FABLE] pagination-truncation
//! prune-safety fork): `meta::prune_oid_map` (272882c) DELETEs `oid_map` rows
//! whose `issue_id` is absent from a `keep_ids` set built from
//! `backend.list_records(&project)`. The GitHub / JIRA / Confluence connectors
//! can silently return a TRUNCATED `Ok(partial_list)` at a pagination or size
//! cap — so a truncated `keep_ids` wipes `oid_map` rows for records that are
//! STILL LIVE upstream, merely beyond the cap. A live record then looks
//! ghost-deleted, and the loss recurs on EVERY sync. The simulator never
//! truncates, so every sim-backed gate (including all of P93's GREEN runs) is
//! structurally blind to this class of bug — hence a capped-MOCK connector.
//!
//! This test exercises the DELETION-direction `oid_map` prune through the REAL
//! `Cache::sync` / `Cache::build_from` paths with a mock that can toggle
//! between a complete and a truncated listing.
//!
//! DP-2 prove-before-fix: the first committed form of
//! `truncation_prunes_live_row_beyond_cap` asserted the CURRENT (buggy)
//! deletion — executed proof the hazard is real on today's code. The fix
//! (Fork A: gate both prune sites on a per-listing `is_complete` signal) then
//! evolved it to assert PRESERVATION, and added the complementary regressions
//! (a complete listing STILL prunes a genuinely-absent row; the full-rebuild
//! `build_from` path is gated too).

#![allow(clippy::missing_panics_doc)]

mod common;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::{sample_issues, CacheDirGuard};
use reposix_cache::Cache;
use reposix_core::backend::{BackendConnector, BackendFeature, DeleteReason, Listing};
use reposix_core::{Error as CoreError, Record, RecordId, Result as CoreResult, Untainted};
use tempfile::tempdir;

/// A `BackendConnector` that models a paginating backend hitting a cap.
///
/// `full` is the TRUE backend state — every record that genuinely exists
/// upstream, always fetchable via `get_record`. `visible` is what
/// `list_records` returns, and can be swapped to a truncated prefix to model a
/// pagination cap being hit. The two diverging is exactly the real-connector
/// condition the sim can never reproduce.
struct CappingMock {
    /// The true, complete backend state (every live record). `get_record`
    /// resolves against this — a record beyond the pagination cap is still
    /// LIVE, not deleted.
    full: Vec<Record>,
    /// What the listing returns. Swapping to a strict prefix of `full` models
    /// either a truncation cap or a genuine upstream delete — which one is
    /// signalled by `is_complete`.
    visible: Mutex<Vec<Record>>,
    /// What `list_records_complete` reports. `false` models a truncation (the
    /// visible set is a PREFIX); `true` models a complete listing (the visible
    /// set is the WHOLE current backend state, e.g. after a genuine delete).
    is_complete: AtomicBool,
}

impl CappingMock {
    fn new(full: Vec<Record>) -> Self {
        let visible = Mutex::new(full.clone());
        Self {
            full,
            visible,
            is_complete: AtomicBool::new(true),
        }
    }

    fn visible_subset(&self, ids: &[u64]) -> Vec<Record> {
        self.full
            .iter()
            .filter(|r| ids.contains(&r.id.0))
            .cloned()
            .collect()
    }

    /// Model a PAGINATION CAP: the listing is a prefix (`ids`) and the backend
    /// reports `is_complete = false`. `full` (and `get_record`) is untouched —
    /// the omitted records are still LIVE upstream, merely beyond the cap.
    fn truncate_to(&self, ids: &[u64]) {
        *self.visible.lock().unwrap() = self.visible_subset(ids);
        self.is_complete.store(false, Ordering::SeqCst);
    }

    /// Model a GENUINE upstream delete: the listing is COMPLETE (`is_complete =
    /// true`) and simply no longer contains the deleted ids. The prune SHOULD
    /// fire here — the records really are gone.
    fn complete_delete_to(&self, ids: &[u64]) {
        *self.visible.lock().unwrap() = self.visible_subset(ids);
        self.is_complete.store(true, Ordering::SeqCst);
    }
}

#[async_trait]
impl BackendConnector for CappingMock {
    fn name(&self) -> &'static str {
        "capping-mock"
    }
    fn supports(&self, _feature: BackendFeature) -> bool {
        false
    }
    async fn list_records(&self, _project: &str) -> CoreResult<Vec<Record>> {
        Ok(self.visible.lock().unwrap().clone())
    }
    /// The completeness-aware listing the cache actually gates its prune on.
    /// `is_complete = false` (a truncation cap) must make the cache SKIP the
    /// prune; `true` must let it fire.
    async fn list_records_complete(&self, _project: &str) -> CoreResult<Listing> {
        Ok(Listing {
            records: self.visible.lock().unwrap().clone(),
            is_complete: self.is_complete.load(Ordering::SeqCst),
        })
    }
    /// Report NOTHING as changed, always. This isolates the prune under test
    /// from the delta path's step-3 blob materialization: the HEAD tree and the
    /// `oid_map` prune are driven purely by `list_records` (step 4), so the
    /// only variable is the completeness of that listing.
    async fn list_changed_since(
        &self,
        _project: &str,
        _since: DateTime<Utc>,
    ) -> CoreResult<Vec<RecordId>> {
        Ok(vec![])
    }
    async fn get_record(&self, _project: &str, id: RecordId) -> CoreResult<Record> {
        // Resolves against `full`: a record truncated out of `list_records` is
        // still LIVE and fetchable.
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
        Err(CoreError::Other("unsupported in capping-mock".into()))
    }
    async fn update_record(
        &self,
        _: &str,
        _: RecordId,
        _: Untainted<Record>,
        _: Option<u64>,
    ) -> CoreResult<Record> {
        Err(CoreError::Other("unsupported in capping-mock".into()))
    }
    async fn delete_or_close(&self, _: &str, _: RecordId, _: DeleteReason) -> CoreResult<()> {
        Err(CoreError::Other("unsupported in capping-mock".into()))
    }
}

/// STANDING REGRESSION (evolved from the DP-2 repro). The delta-sync prune site
/// (`builder.rs`, Step-5 transaction) must NOT delete a live record's `oid_map`
/// row when the listing was TRUNCATED.
///
/// Backend truly holds issues 1, 2, 3. A complete seed populates `oid_map` for
/// all three. The backend then PAGINATES: the listing returns only {1, 2} with
/// `is_complete = false`, while issue 3 stays live upstream. The Fork-A gate
/// makes the delta sync SKIP the prune — issue 3's row SURVIVES (no data loss).
#[tokio::test]
async fn truncated_delta_sync_preserves_live_row_beyond_cap() {
    let tmp = tempdir().unwrap();
    let _g = CacheDirGuard::new(tmp.path());

    let full = sample_issues("demo", 3);
    let mock = Arc::new(CappingMock::new(full));
    let backend: Arc<dyn BackendConnector> = mock.clone();
    let cache = Cache::open(backend, "sim", "demo").unwrap();

    // Seed sync (build_from): complete listing {1,2,3} → oid_map rows for all.
    cache.sync().await.expect("seed sync");
    assert!(
        cache.find_oid_for_record(RecordId(3)).unwrap().is_some(),
        "issue 3 must have an oid_map row after the complete seed (precondition)"
    );

    // The backend now TRUNCATES its listing to {1,2} (is_complete=false). Issue
    // 3 is STILL LIVE (get_record(3) resolves) — merely beyond the cap.
    mock.truncate_to(&[1, 2]);

    let r = cache
        .sync()
        .await
        .expect("delta sync with a truncated listing");
    assert!(r.since.is_some(), "second sync must take the delta path");

    let after = cache.find_oid_for_record(RecordId(3)).unwrap();
    eprintln!("PRESERVATION: find_oid_for_record(3) after truncated delta sync = {after:?}");
    assert!(
        after.is_some(),
        "DATA LOSS: a TRUNCATED (is_complete=false) delta sync pruned issue 3's \
         oid_map row even though issue 3 is still LIVE upstream. The Fork-A gate \
         at builder.rs's delta prune site must SKIP the prune on an incomplete \
         listing."
    );
    // The surviving row keeps the record resolvable via list_record_ids too.
    let ids: Vec<u64> = cache
        .list_record_ids()
        .unwrap()
        .iter()
        .map(|r| r.0)
        .collect();
    assert!(
        ids.contains(&3),
        "issue 3 must remain in list_record_ids after a truncated sync, got {ids:?}"
    );
}

/// REGRESSION (no functional loss of the legitimate prune): a COMPLETE listing
/// that genuinely no longer contains a record MUST still prune its `oid_map`
/// row. The Fork-A gate skips the prune ONLY on `is_complete = false`; a
/// complete listing (the sim's always-true posture, and a real backend's normal
/// case) keeps the DELETION-direction coherence prune firing.
#[tokio::test]
async fn complete_delta_sync_still_prunes_genuinely_absent_row() {
    let tmp = tempdir().unwrap();
    let _g = CacheDirGuard::new(tmp.path());

    let full = sample_issues("demo", 3);
    let mock = Arc::new(CappingMock::new(full));
    let backend: Arc<dyn BackendConnector> = mock.clone();
    let cache = Cache::open(backend, "sim", "demo").unwrap();

    cache.sync().await.expect("seed sync");
    assert!(
        cache.find_oid_for_record(RecordId(3)).unwrap().is_some(),
        "issue 3 must have an oid_map row after the complete seed (precondition)"
    );

    // Issue 3 is GENUINELY deleted upstream: the listing is COMPLETE
    // (is_complete=true) and simply no longer contains it.
    mock.complete_delete_to(&[1, 2]);

    let r = cache.sync().await.expect("delta sync, complete listing");
    assert!(r.since.is_some(), "second sync must take the delta path");

    let after = cache.find_oid_for_record(RecordId(3)).unwrap();
    assert!(
        after.is_none(),
        "REGRESSION: a COMPLETE (is_complete=true) listing that dropped issue 3 \
         must STILL prune its now-ghost oid_map row — the Fork-A gate must not \
         over-broadly disable the legitimate DELETION-direction prune. Got \
         {after:?}"
    );
    let ids: Vec<u64> = cache
        .list_record_ids()
        .unwrap()
        .iter()
        .map(|r| r.0)
        .collect();
    assert!(
        !ids.contains(&3) && ids.contains(&1) && ids.contains(&2),
        "after a complete delete, list_record_ids must drop 3 and keep 1,2; got {ids:?}"
    );
}

/// STANDING REGRESSION for the OTHER prune call site: the full-rebuild
/// `build_from` path (`builder.rs`, also the `reposix sync --reconcile` path)
/// must likewise SKIP the prune on a truncated listing. `--reconcile` re-lists
/// in full and hits the SAME pagination cap, so an ungated prune here would turn
/// the documented recovery command into the data-loss vector.
#[tokio::test]
async fn truncated_build_from_preserves_live_row_beyond_cap() {
    let tmp = tempdir().unwrap();
    let _g = CacheDirGuard::new(tmp.path());

    let full = sample_issues("demo", 3);
    let mock = Arc::new(CappingMock::new(full));
    let backend: Arc<dyn BackendConnector> = mock.clone();
    let cache = Cache::open(backend, "sim", "demo").unwrap();

    // Complete build_from → oid_map rows for {1,2,3}.
    cache.build_from().await.expect("initial build_from");
    assert!(
        cache.find_oid_for_record(RecordId(3)).unwrap().is_some(),
        "issue 3 must have an oid_map row after the complete build_from (precondition)"
    );

    // Truncate, then re-run build_from (the --reconcile full-rebuild path).
    mock.truncate_to(&[1, 2]);
    cache
        .build_from()
        .await
        .expect("reconcile build_from with a truncated listing");

    let after = cache.find_oid_for_record(RecordId(3)).unwrap();
    eprintln!("PRESERVATION: find_oid_for_record(3) after truncated build_from = {after:?}");
    assert!(
        after.is_some(),
        "DATA LOSS: a TRUNCATED (is_complete=false) build_from/reconcile pruned \
         issue 3's oid_map row even though issue 3 is still LIVE upstream. The \
         Fork-A gate at builder.rs's build_from prune site must SKIP the prune \
         on an incomplete listing."
    );
}
