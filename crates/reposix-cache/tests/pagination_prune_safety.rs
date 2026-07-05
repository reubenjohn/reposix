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

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::{sample_issues, CacheDirGuard};
use reposix_cache::Cache;
use reposix_core::backend::{BackendConnector, BackendFeature, DeleteReason};
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
    /// What `list_records` currently returns. Swapping to a strict prefix of
    /// `full` models a truncation at a pagination/size cap.
    visible: Mutex<Vec<Record>>,
}

impl CappingMock {
    fn new(full: Vec<Record>) -> Self {
        let visible = Mutex::new(full.clone());
        Self { full, visible }
    }

    /// Restrict `list_records` to the records whose id is in `ids` — models the
    /// connector truncating its listing at a cap. `full` (and therefore
    /// `get_record`) is untouched: the omitted records are still LIVE upstream.
    fn set_visible(&self, ids: &[u64]) {
        let subset: Vec<Record> = self
            .full
            .iter()
            .filter(|r| ids.contains(&r.id.0))
            .cloned()
            .collect();
        *self.visible.lock().unwrap() = subset;
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

/// DP-2 confirmed repro (asserts the CURRENT, buggy behavior).
///
/// Backend truly holds issues 1, 2, 3. A complete seed populates `oid_map` for
/// all three. The backend then PAGINATES: `list_records` returns only {1, 2}
/// while issue 3 stays live upstream. Today's unconditional prune builds
/// `keep_ids` from the truncated listing and DELETES issue 3's `oid_map`
/// row — silent data loss the sim can never surface.
#[tokio::test]
async fn truncation_prunes_live_row_beyond_cap() {
    let tmp = tempdir().unwrap();
    let _g = CacheDirGuard::new(tmp.path());

    let full = sample_issues("demo", 3);
    let mock = Arc::new(CappingMock::new(full));
    let backend: Arc<dyn BackendConnector> = mock.clone();
    let cache = Cache::open(backend, "sim", "demo").unwrap();

    // Seed sync (build_from): complete listing {1,2,3} → oid_map rows for all.
    cache.sync().await.expect("seed sync");
    let before = cache.find_oid_for_record(RecordId(3)).unwrap();
    assert!(
        before.is_some(),
        "issue 3 must have an oid_map row after the complete seed (setup precondition)"
    );

    // The backend now truncates its listing to {1,2}. Issue 3 is STILL LIVE
    // (get_record(3) resolves) — it is merely beyond the pagination cap.
    mock.set_visible(&[1, 2]);

    // Delta sync. On TODAY's code the prune runs UNCONDITIONALLY against the
    // truncated keep_ids {1,2}, wiping issue 3's oid_map row.
    let r = cache
        .sync()
        .await
        .expect("delta sync with a truncated listing");
    assert!(r.since.is_some(), "second sync must take the delta path");

    let after = cache.find_oid_for_record(RecordId(3)).unwrap();
    eprintln!("REPRO: find_oid_for_record(3) before={before:?} after={after:?}");
    assert!(
        after.is_none(),
        "REPRO EXPECTATION (current buggy behavior): a truncated list_records \
         PRUNED issue 3's oid_map row even though issue 3 is still LIVE upstream \
         (get_record(3) resolves). This is the DATA-LOSS hazard. Once the \
         completeness gate lands this assertion is flipped to PRESERVATION."
    );
}
