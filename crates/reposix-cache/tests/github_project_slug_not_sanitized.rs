//! P104 / S-260707-gh404 — GitHub helper-path 404 regression guard.
//!
//! ROOT CAUSE (confirmed): `Cache` carried a SINGLE `project` field used for
//! BOTH the on-disk cache directory AND the backend REST calls. Every caller
//! (the git-remote helper, `reposix attach`, `reposix sync`) fed the
//! filesystem-SANITIZED form (`owner/repo` → `owner-repo`) to `Cache::open`,
//! so `build_from` called `list_records_complete("owner-repo")` and the GitHub
//! connector formatted `GET /repos/owner-repo/issues` → 404. Only GitHub has a
//! slash in its project slug, so only GitHub 404'd.
//!
//! FIX: the backend keeps the RAW slug; sanitization lives ONLY at on-disk path
//! derivation (`reposix_core::path::sanitize_project_for_cache`, called inside
//! `reposix_cache::path::resolve_cache_path`). `Cache::open` stores the RAW
//! slug in `self.project` (REST calls get `owner/repo`) while the cache dir is
//! still `github-owner-repo.git`.
//!
//! This test pins BOTH halves of that split with a RECORDING mock backend that
//! captures the exact project string reaching `list_records_complete`. It is
//! the cheap guard (no GitHub token, runs in `-p reposix-cache`); the real
//! front-door 200 is owned by `agent-ux/github-front-door-real-backend`.
//!
//! FAILS-THEN-PASSES: on the PRE-FIX `resolve_cache_path` (which did not
//! sanitize), opening with the raw `owner/repo` slug produced a NESTED
//! `github-owner/repo.git` directory — the `repo_path()` assertion below fails
//! (file name is `repo.git`, not `github-owner-repo.git`). The fix makes path
//! derivation sanitize, so the dir is the flat `github-owner-repo.git` and the
//! test passes.

#![allow(clippy::missing_panics_doc)]

mod common;

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::CacheDirGuard;
use reposix_cache::Cache;
use reposix_core::backend::{BackendConnector, BackendFeature, DeleteReason, Listing};
use reposix_core::{Error as CoreError, Record, RecordId, Result as CoreResult, Untainted};
use tempfile::tempdir;

/// A `BackendConnector` that RECORDS every `project` string it is asked to
/// list. The whole point of the fix is that the backend-facing project must be
/// the RAW `owner/repo` slug, never the filesystem-sanitized `owner-repo`.
struct RecordingMock {
    /// Every project string passed to `list_records` / `list_records_complete`,
    /// in call order.
    listed_projects: Mutex<Vec<String>>,
}

impl RecordingMock {
    fn new() -> Self {
        Self {
            listed_projects: Mutex::new(Vec::new()),
        }
    }

    fn record(&self, project: &str) {
        self.listed_projects
            .lock()
            .unwrap()
            .push(project.to_owned());
    }

    fn projects_seen(&self) -> Vec<String> {
        self.listed_projects.lock().unwrap().clone()
    }
}

#[async_trait]
impl BackendConnector for RecordingMock {
    fn name(&self) -> &'static str {
        "recording-mock"
    }
    fn supports(&self, _feature: BackendFeature) -> bool {
        false
    }
    async fn list_records(&self, project: &str) -> CoreResult<Vec<Record>> {
        self.record(project);
        Ok(vec![])
    }
    async fn list_records_complete(&self, project: &str) -> CoreResult<Listing> {
        self.record(project);
        Ok(Listing {
            records: vec![],
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
    async fn get_record(&self, project: &str, id: RecordId) -> CoreResult<Record> {
        Err(CoreError::NotFound {
            project: project.to_owned(),
            id: id.0.to_string(),
        })
    }
    async fn create_record(&self, _: &str, _: Untainted<Record>) -> CoreResult<Record> {
        Err(CoreError::Other("unsupported in recording-mock".into()))
    }
    async fn update_record(
        &self,
        _: &str,
        _: RecordId,
        _: Untainted<Record>,
        _: Option<u64>,
    ) -> CoreResult<Record> {
        Err(CoreError::Other("unsupported in recording-mock".into()))
    }
    async fn delete_or_close(&self, _: &str, _: RecordId, _: DeleteReason) -> CoreResult<()> {
        Err(CoreError::Other("unsupported in recording-mock".into()))
    }
}

/// The GitHub 404 regression guard. Opening a cache for `github::owner/repo`
/// and driving `build_from` must:
///   1. call the backend with the RAW `owner/repo` slug (NOT `owner-repo`), so
///      the GitHub connector formats `GET /repos/owner/repo/issues` (200), and
///   2. still lay the cache down at the sanitized flat dir
///      `github-owner-repo.git` (no embedded slash / nested subdirectory).
#[tokio::test]
async fn github_slug_reaches_backend_raw_but_cache_dir_is_sanitized() {
    let tmp = tempdir().unwrap();
    let _g = CacheDirGuard::new(tmp.path());

    let mock = Arc::new(RecordingMock::new());
    let backend: Arc<dyn BackendConnector> = mock.clone();

    // Callers now pass the RAW slug; sanitization happens inside the cache at
    // path-derivation time only.
    let cache = Cache::open(backend, "github", "owner/repo").unwrap();
    cache.build_from().await.unwrap();

    // (1) The backend must have seen the RAW slug — this is the anti-404
    // contract. A pre-sanitized `owner-repo` here is exactly the bug.
    let seen = mock.projects_seen();
    assert!(
        seen.iter().any(|p| p == "owner/repo"),
        "backend must receive the RAW slug `owner/repo`, got {seen:?} \
         (a sanitized `owner-repo` is the GitHub 404 bug)"
    );
    assert!(
        !seen.iter().any(|p| p == "owner-repo"),
        "backend must NEVER receive the sanitized `owner-repo`, got {seen:?}"
    );

    // (2) The on-disk cache path is STILL sanitized to a flat directory — no
    // embedded slash, no nested `owner/` subdir. This is the half that fails on
    // the pre-fix `resolve_cache_path` (which produced `github-owner/repo.git`).
    let file_name = cache
        .repo_path()
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap();
    assert_eq!(
        file_name,
        "github-owner-repo.git",
        "cache dir must be the sanitized flat `github-owner-repo.git`, got {:?}",
        cache.repo_path()
    );
}
