//! The [`Cache`] struct. Holds the backend, project, gix bare repo, and
//! `cache.db` connection.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use reposix_core::BackendConnector;

use crate::db::open_cache_db;
use crate::error::{Error, Result};
use crate::meta;
use crate::path::resolve_cache_path;

/// Backing bare-repo cache for one `(backend, project)` tuple.
///
/// Created via [`Cache::open`]. Call [`Cache::build_from`] to populate
/// the tree; call [`Cache::read_blob`] to materialize a blob on demand.
pub struct Cache {
    pub(crate) backend: Arc<dyn BackendConnector>,
    pub(crate) backend_name: String,
    pub(crate) project: String,
    pub(crate) path: PathBuf,
    pub(crate) repo: gix::Repository,
    /// Wrapped in [`Mutex`] because [`rusqlite::Connection`] is not
    /// [`Send`]-safe across `await` points; interior mutability lets
    /// the async methods acquire the lock, do a short SQL call, and
    /// drop it before awaiting.
    pub(crate) db: Mutex<rusqlite::Connection>,
}

impl Cache {
    /// Open (or create) the cache at the deterministic path for
    /// `(backend_name, project)`.
    ///
    /// Side effects: [`std::fs::create_dir_all`] on the parent,
    /// [`gix::init_bare`] on the target, and [`open_cache_db`] on
    /// `<cache-path>/cache.db`. Idempotent — re-opening an existing
    /// cache rebinds the handles without touching content.
    ///
    /// On second and subsequent opens, the `meta` table is consulted
    /// for an `identity` row; if present and mismatched with the
    /// caller's `(backend_name, project)`, returns
    /// [`Error::CacheCollision`]. On first open the identity is
    /// written.
    ///
    /// # Errors
    /// - [`Error::Io`] for directory creation failure or no
    ///   discoverable cache root.
    /// - [`Error::Git`] if `gix::init_bare` fails.
    /// - [`Error::Sqlite`] if the cache DB cannot be opened or its
    ///   schema cannot be loaded.
    /// - [`Error::CacheCollision`] if the cache belongs to a
    ///   different `(backend, project)` tuple.
    pub fn open(
        backend: Arc<dyn BackendConnector>,
        backend_name: impl Into<String>,
        project: impl Into<String>,
    ) -> Result<Self> {
        let backend_name = backend_name.into();
        let project = project.into();
        let path = resolve_cache_path(&backend_name, &project)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        // Idempotent open: if the cache dir already holds a bare git
        // repo, reuse it; only `init_bare` for fresh dirs. `gix::init_bare`
        // refuses non-empty dirs, which would break Q1.3 (re-attach against
        // the same SoT must succeed). The presence of `HEAD` is the
        // canonical "this is a git dir" signal — any partial init that
        // produced HEAD is reusable; anything more partial is rare enough
        // we let gix surface the diagnostic.
        let mut repo = if path.join("HEAD").exists() {
            gix::open(&path).map_err(|e| Error::Git(e.to_string()))?
        } else {
            gix::init_bare(&path).map_err(|e| Error::Git(e.to_string()))?
        };

        // Provide a default committer/author identity so `Cache::build_from`
        // can produce a commit even when the host has no `user.name` /
        // `user.email` configured (typical of CI runners). The
        // `noreply.invalid` domain is RFC-2606 reserved and signals
        // "not a real address" to anyone reading the log. User-level
        // git config (e.g. `~/.gitconfig`) is loaded BEFORE this fallback
        // is applied, but we only set values if the keys are absent —
        // so a configured `user.name` / `user.email` still wins.
        if repo.committer().is_none() {
            let mut snap = repo.config_snapshot_mut();
            if snap.string(gix::config::tree::User::NAME).is_none() {
                snap.set_value(&gix::config::tree::User::NAME, "reposix-cache")
                    .map_err(|e| Error::Git(format!("set user.name: {e}")))?;
            }
            if snap.string(gix::config::tree::User::EMAIL).is_none() {
                snap.set_value(
                    &gix::config::tree::User::EMAIL,
                    "reposix-cache@noreply.invalid",
                )
                .map_err(|e| Error::Git(format!("set user.email: {e}")))?;
            }
            snap.commit().map_err(|e| Error::Git(e.to_string()))?;
        }

        // Hide the private sync-tag namespace from `git upload-pack
        // --advertise-refs`. Without this, every `refs/reposix/sync/<ts>`
        // would leak into the helper's protocol-v2 advertisement and the
        // agent's working tree would see (and try to fetch from) tags it
        // has no business with. `transfer.hideRefs` is the only
        // server-side knob that affects upload-pack's advertised set.
        //
        // Set unconditionally — the underlying `git config` write is
        // idempotent (same value, no diff). We use raw `git config` here
        // because gix's `set_value` writes to the in-memory snapshot only
        // (the snapshot is dropped once we're done with `Cache::open`).
        ensure_hide_sync_refs(&path)?;

        // cache.db lives inside the bare repo dir so a single path
        // scheme covers both git state and cache state.
        let db = open_cache_db(&path)?;

        // Identity check: Plan 02 writes on first open, errors on
        // mismatch. Phase 33 may refine the semantics (e.g. wipe +
        // re-seed).
        let expected = format!("{backend_name}:{project}");
        if let Some(found) = meta::get_meta(&db, "identity")? {
            if found != expected {
                return Err(Error::CacheCollision { expected, found });
            }
        } else {
            meta::set_meta(&db, "identity", &expected)?;
        }

        Ok(Self {
            backend,
            backend_name,
            project,
            path,
            repo,
            db: Mutex::new(db),
        })
    }

    /// On-disk path to the bare repo (the `<backend>-<project>.git` dir).
    #[must_use]
    pub fn repo_path(&self) -> &std::path::Path {
        &self.path
    }

    /// Backend name (written into audit rows).
    #[must_use]
    pub fn backend_name(&self) -> &str {
        &self.backend_name
    }

    /// Project slug (written into audit rows).
    #[must_use]
    pub fn project(&self) -> &str {
        &self.project
    }

    /// Write an `op='helper_connect'` audit row. Best-effort.
    ///
    /// # Panics
    /// Panics if the internal `cache.db` mutex is poisoned.
    pub fn log_helper_connect(&self, service: &str) {
        let db = self.db.lock().expect("cache.db mutex poisoned");
        crate::audit::log_helper_connect(&db, &self.backend_name, &self.project, service);
    }

    /// Write an `op='helper_advertise'` audit row. Best-effort.
    ///
    /// # Panics
    /// Panics if the internal `cache.db` mutex is poisoned.
    pub fn log_helper_advertise(&self, bytes: u32) {
        let db = self.db.lock().expect("cache.db mutex poisoned");
        crate::audit::log_helper_advertise(&db, &self.backend_name, &self.project, bytes);
    }

    /// Write an `op='helper_fetch'` audit row. Best-effort. The `stats`
    /// payload is produced by the `reposix-remote::stateless_connect`
    /// handler; we accept a by-value wrapper (via a structural trait)
    /// so the two crates don't need to share a type. See the
    /// `HelperFetchRecord` trait below.
    ///
    /// # Panics
    /// Panics if the internal `cache.db` mutex is poisoned.
    pub fn log_helper_fetch<R: HelperFetchRecord>(&self, stats: &R) {
        let db = self.db.lock().expect("cache.db mutex poisoned");
        crate::audit::log_helper_fetch(
            &db,
            &self.backend_name,
            &self.project,
            stats.command(),
            stats.want_count(),
            stats.request_bytes(),
            stats.response_bytes(),
        );
    }

    /// Write an `op='helper_fetch_error'` audit row. Best-effort.
    ///
    /// # Panics
    /// Panics if the internal `cache.db` mutex is poisoned.
    pub fn log_helper_fetch_error(&self, exit_code: i32, stderr_tail: &str) {
        let db = self.db.lock().expect("cache.db mutex poisoned");
        crate::audit::log_helper_fetch_error(
            &db,
            &self.backend_name,
            &self.project,
            exit_code,
            stderr_tail,
        );
    }

    /// Write an `op='blob_limit_exceeded'` audit row. Best-effort.
    ///
    /// # Panics
    /// Panics if the internal `cache.db` mutex is poisoned.
    pub fn log_blob_limit_exceeded(&self, want_count: u32, limit: u32) {
        let db = self.db.lock().expect("cache.db mutex poisoned");
        crate::audit::log_blob_limit_exceeded(
            &db,
            &self.backend_name,
            &self.project,
            want_count,
            limit,
        );
    }

    /// Write an `op='helper_push_started'` audit row. Best-effort.
    ///
    /// # Panics
    /// Panics if the internal `cache.db` mutex is poisoned.
    pub fn log_helper_push_started(&self, ref_name: &str) {
        let db = self.db.lock().expect("cache.db mutex poisoned");
        crate::audit::log_helper_push_started(&db, &self.backend_name, &self.project, ref_name);
    }

    /// Write an `op='helper_push_accepted'` audit row. Best-effort.
    ///
    /// # Panics
    /// Panics if the internal `cache.db` mutex is poisoned.
    pub fn log_helper_push_accepted(&self, files_touched: u32, summary: &str) {
        let db = self.db.lock().expect("cache.db mutex poisoned");
        crate::audit::log_helper_push_accepted(
            &db,
            &self.backend_name,
            &self.project,
            files_touched,
            summary,
        );
    }

    /// Write an `op='helper_push_rejected_conflict'` audit row. Best-effort.
    ///
    /// # Panics
    /// Panics if the internal `cache.db` mutex is poisoned.
    pub fn log_helper_push_rejected_conflict(
        &self,
        issue_id: &str,
        local_version: u64,
        backend_version: u64,
    ) {
        let db = self.db.lock().expect("cache.db mutex poisoned");
        crate::audit::log_helper_push_rejected_conflict(
            &db,
            &self.backend_name,
            &self.project,
            issue_id,
            local_version,
            backend_version,
        );
    }

    /// Write an `op='helper_push_sanitized_field'` audit row. Best-effort.
    ///
    /// # Panics
    /// Panics if the internal `cache.db` mutex is poisoned.
    pub fn log_helper_push_sanitized_field(&self, issue_id: &str, field: &str) {
        let db = self.db.lock().expect("cache.db mutex poisoned");
        crate::audit::log_helper_push_sanitized_field(
            &db,
            &self.backend_name,
            &self.project,
            issue_id,
            field,
        );
    }

    /// Write an `op='token_cost'` audit row — one per helper RPC turn.
    /// `chars_in` is the request-bytes received from the agent; `chars_out`
    /// is the response-bytes sent back. `kind` is `"fetch"` or `"push"`.
    /// Token estimate is `chars / 4` (conservative English-text heuristic).
    /// Best-effort: SQL errors WARN-log.
    ///
    /// See `.planning/research/v0.11.0/vision-and-innovations.md` §3c.
    ///
    /// # Panics
    /// Panics if the internal `cache.db` mutex is poisoned.
    pub fn log_token_cost(&self, chars_in: u64, chars_out: u64, kind: &str) {
        let db = self.db.lock().expect("cache.db mutex poisoned");
        crate::audit::log_token_cost(
            &db,
            &self.backend_name,
            &self.project,
            chars_in,
            chars_out,
            kind,
        );
    }

    /// Write an `op='helper_backend_instantiated'` audit row.
    /// Best-effort. Emitted by the git remote helper after the
    /// URL-scheme dispatcher resolves a `(backend_kind, project)`
    /// pair and the cache directory is opened. `project_for_backend`
    /// is the live project string passed to `BackendConnector`
    /// methods (may differ from `self.project()` for GitHub:
    /// `owner/repo` vs the cache-safe `owner-repo`).
    ///
    /// # Panics
    /// Panics if the internal `cache.db` mutex is poisoned.
    pub fn log_helper_backend_instantiated(&self, project_for_backend: &str) {
        let db = self.db.lock().expect("cache.db mutex poisoned");
        crate::audit::log_helper_backend_instantiated(
            &db,
            &self.backend_name,
            &self.project,
            project_for_backend,
        );
    }

    /// Return the set of backend record IDs known to the cache from the
    /// most recent `build_from` tree.
    ///
    /// Implementation: queries the `oid_map` rows belonging to this
    /// `(backend, project)` pair. Each row encodes `(oid, issue_id)`
    /// for one record currently in the tree; we return the
    /// distinct, parsed `RecordId` set.
    ///
    /// # Errors
    /// Returns [`Error::Sqlite`] if the underlying query fails or
    /// [`Error::Backend`] if a stored `issue_id` cannot be parsed back
    /// into a `u64` (data corruption — should never happen in practice).
    ///
    /// # Panics
    /// Panics if the internal `cache.db` mutex is poisoned.
    pub fn list_record_ids(&self) -> Result<Vec<reposix_core::RecordId>> {
        let db = self.db.lock().expect("cache.db mutex poisoned");
        let mut stmt = db
            .prepare(
                "SELECT DISTINCT issue_id FROM oid_map \
                 WHERE backend = ?1 AND project = ?2",
            )
            .map_err(|e| Error::Sqlite(format!("prepare list_record_ids: {e}")))?;
        let rows = stmt
            .query_map(
                rusqlite::params![&self.backend_name, &self.project],
                |row| row.get::<_, String>(0),
            )
            .map_err(|e| Error::Sqlite(format!("query list_record_ids: {e}")))?;
        let mut out = Vec::new();
        for r in rows {
            let id_str = r.map_err(|e| Error::Sqlite(format!("row list_record_ids: {e}")))?;
            let id_num: u64 = id_str.parse().map_err(|_| {
                Error::Backend(format!("oid_map issue_id `{id_str}` is not numeric"))
            })?;
            out.push(reposix_core::RecordId(id_num));
        }
        Ok(out)
    }

    /// Return the blob OID for a given backend record id from the
    /// most-recent tree, if any. Reads the `oid_map` row that joins
    /// this `(backend, project, issue_id)` triple.
    ///
    /// # Errors
    /// Returns [`Error::Sqlite`] if the underlying query fails or
    /// [`Error::Git`] if the stored OID hex cannot parse back to a
    /// `gix::ObjectId` (data corruption).
    ///
    /// # Panics
    /// Panics if the internal `cache.db` mutex is poisoned.
    pub fn find_oid_for_record(&self, id: reposix_core::RecordId) -> Result<Option<gix::ObjectId>> {
        use rusqlite::OptionalExtension as _;
        let db = self.db.lock().expect("cache.db mutex poisoned");
        let id_str = id.0.to_string();
        let oid_hex: Option<String> = db
            .query_row(
                "SELECT oid FROM oid_map \
                 WHERE backend = ?1 AND project = ?2 AND issue_id = ?3",
                rusqlite::params![&self.backend_name, &self.project, &id_str],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|e| Error::Sqlite(format!("find_oid_for_record: {e}")))?;
        let Some(hex) = oid_hex else {
            return Ok(None);
        };
        let oid = gix::ObjectId::from_hex(hex.as_bytes())
            .map_err(|e| Error::Git(format!("oid_map oid `{hex}` is not a valid hex OID: {e}")))?;
        Ok(Some(oid))
    }

    /// Lock the cache's `cache.db` connection for transactional use.
    ///
    /// Returns a [`std::sync::MutexGuard`] over the underlying
    /// [`rusqlite::Connection`]. Callers can begin a transaction via
    /// `Connection::transaction` on the dereferenced guard, but should
    /// keep the lock held only as long as needed (no `await` while
    /// holding it — `Connection` is not `Send`).
    ///
    /// `pub(crate)` because the only consumer is the reconciliation
    /// walker in this crate; broadening visibility would invite
    /// helper-crate code to bypass the typed APIs above.
    ///
    /// # Errors
    /// Returns [`Error::Sqlite`] if the mutex is poisoned (a previous
    /// holder panicked while writing).
    pub(crate) fn connection_mut(&self) -> Result<std::sync::MutexGuard<'_, rusqlite::Connection>> {
        self.db
            .lock()
            .map_err(|_| Error::Sqlite("cache.db mutex poisoned".into()))
    }

    /// Write a single `audit_events_cache` row recording that an
    /// `attach` walk completed (DVCS-ATTACH-02 / OP-3). Best-effort
    /// in line with the rest of the `log_*` family — SQL errors
    /// WARN-log.
    ///
    /// `event_type` is the `op` column value (currently always
    /// `"attach_walk"`; the parameter shape matches POC-FINDINGS F04
    /// so siblings like P83's `mirror_lag_partial_failure` reuse
    /// this surface). `payload_json` is encoded into the existing
    /// `reason` column as a JSON string.
    ///
    /// # Errors
    /// Returns [`Error::Sqlite`] if the SQL insert fails. Unlike the
    /// other `log_*` helpers (which are best-effort and return `()`),
    /// this one returns `Result` so the caller can surface OP-3
    /// breakage to the user — `reposix attach` MUST report an audit
    /// failure rather than silently dropping the row.
    ///
    /// # Panics
    /// Panics if the internal `cache.db` mutex is poisoned.
    pub fn log_attach_walk(
        &self,
        event_type: &str,
        payload_json: &serde_json::Value,
    ) -> Result<()> {
        let db = self.db.lock().expect("cache.db mutex poisoned");
        let payload = payload_json.to_string();
        db.execute(
            "INSERT INTO audit_events_cache (ts, op, backend, project, reason) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                chrono::Utc::now().to_rfc3339(),
                event_type,
                &self.backend_name,
                &self.project,
                payload,
            ],
        )
        .map_err(|e| Error::Sqlite(format!("log_attach_walk: {e}")))?;
        Ok(())
    }
}

/// Ensure `transfer.hideRefs` includes our private sync-tag namespace.
/// Idempotent — `git config --add` skipping duplicate values would simplify
/// this, but git treats `transfer.hideRefs` as a multi-valued key without
/// dedup, so we read first and only add if absent.
fn ensure_hide_sync_refs(repo_path: &std::path::Path) -> Result<()> {
    let want = "refs/reposix/sync/";
    // Read all current values; skip if already present.
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(["config", "--get-all", "transfer.hideRefs"])
        .output()
        .map_err(|e| {
            Error::Git(format!(
                "spawn `git config --get-all transfer.hideRefs`: {e}"
            ))
        })?;
    // Exit code 1 with empty stdout means "key not set" — that's fine.
    let stdout = String::from_utf8_lossy(&out.stdout);
    if stdout.lines().any(|l| l.trim() == want) {
        return Ok(());
    }
    // Add the value.
    let add = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(["config", "--add", "transfer.hideRefs", want])
        .output()
        .map_err(|e| Error::Git(format!("spawn `git config --add transfer.hideRefs`: {e}")))?;
    if !add.status.success() {
        return Err(Error::Git(format!(
            "git config --add transfer.hideRefs failed: {}",
            String::from_utf8_lossy(&add.stderr).trim()
        )));
    }
    Ok(())
}

/// Structural accessor for a helper-fetch RPC-turn record. Implemented
/// by `reposix-remote::stateless_connect::RpcStats` — we use a trait
/// so that crate does not depend on a `reposix-cache`-defined struct
/// and vice-versa, keeping the cache crate free of
/// transport-layer concepts.
pub trait HelperFetchRecord {
    /// The `command=<word>` keyword if extracted from the first data
    /// frame (`fetch`, `ls-refs`, etc.). `None` if not a recognizable
    /// protocol-v2 command.
    fn command(&self) -> Option<&str>;
    /// Count of `want ` lines observed in the request.
    fn want_count(&self) -> u32;
    /// Total request bytes written to `upload-pack` stdin.
    fn request_bytes(&self) -> u32;
    /// Total response bytes read from `upload-pack` stdout.
    fn response_bytes(&self) -> u32;
}
