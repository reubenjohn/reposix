//! Tree construction and lazy blob materialization.

use chrono::Utc;
use reposix_core::{frontmatter, RecordId, Tainted};

use crate::cache::Cache;
use crate::error::{Error, Result};
use crate::{audit, meta};

/// Result of a single [`Cache::sync`] invocation.
#[derive(Debug, Clone)]
pub struct SyncReport {
    /// Issue IDs the backend reported as changed since `since`.
    /// Empty on the seed path (no prior cursor).
    pub changed_ids: Vec<RecordId>,
    /// The timestamp passed to the backend, or `None` on the seed
    /// path (no prior `last_fetched_at`).
    pub since: Option<chrono::DateTime<chrono::Utc>>,
    /// New HEAD commit, or `None` on the seed path if `build_from`
    /// returned no commit (it always does in practice; this stays
    /// `Option` for forward-compat).
    pub new_commit: Option<gix::ObjectId>,
}

impl Cache {
    /// Sync the tree from the backend and commit to `refs/heads/main`.
    ///
    /// Does NOT materialize blobs — the returned commit references blob
    /// OIDs that are only persisted on demand (see [`Cache::read_blob`]).
    ///
    /// Side effects on `cache.db`:
    /// - one `INSERT OR REPLACE` per issue into `oid_map` linking the
    ///   computed blob OID to its `issue_id` and `(backend, project)`;
    /// - one `op='tree_sync'` audit row (best-effort);
    /// - `meta.last_fetched_at` upserted to the current UTC RFC-3339
    ///   timestamp.
    ///
    /// Commit message format:
    /// `sync(<backend>:<project>): <N> issues at <ISO8601>`.
    ///
    /// # Errors
    /// - [`Error::Backend`] if `list_records` fails.
    /// - [`Error::Egress`] if `list_records` fails with the allowlist
    ///   variant (`reposix_core::Error::InvalidOrigin`); the
    ///   `egress_denied` audit row is written first.
    /// - [`Error::Render`] if frontmatter rendering fails for any issue.
    /// - [`Error::Git`] if any gix operation fails.
    /// - [`Error::Io`] if the `HEAD` file cannot be written.
    /// - [`Error::Sqlite`] if `oid_map` or `meta` updates fail.
    ///
    /// # Panics
    /// Panics if the internal `cache.db` mutex is poisoned (another
    /// thread panicked while holding it). A panic in this path means
    /// the process state is corrupt; this method does not attempt to
    /// recover.
    pub async fn build_from(&self) -> Result<gix::ObjectId> {
        // List issues. If this fails with an egress-denial variant, fire
        // the audit row BEFORE returning the typed error — same shape
        // as `read_blob`'s egress path.
        let issues = match self.backend.list_records(&self.project).await {
            Ok(v) => v,
            Err(e) => return Err(self.classify_backend_error(&e, None)),
        };

        // Render each issue, compute the blob OID WITHOUT writing the
        // blob object. The tree references each blob_oid; the blob
        // itself is persisted only when `read_blob(oid)` is called.
        // This is the lazy-blob invariant the `git-remote-reposix`
        // stateless-connect handler relies on.
        //
        // NOTE: we deliberately bypass `Repository::edit_tree` because
        // its `write()` validates that every referenced object already
        // exists in the object DB (gix 0.82
        // `write_cursor -> MissingObject`). That check is fatal to the
        // lazy-blob invariant: we WANT to write a tree that references
        // blobs we have not persisted. We assemble `gix_object::Tree`
        // manually and call `Repository::write_object`, which does no
        // such validation.
        let hash_kind = self.repo.object_hash();

        // Build entries for the `issues/` subtree: `<id>.md -> blob_oid`,
        // keeping the `issue_id` alongside so we can populate oid_map
        // with the same computed OIDs the tree references.
        let mut records: Vec<(gix::objs::tree::Entry, String)> = Vec::with_capacity(issues.len());
        for issue in &issues {
            let rendered = frontmatter::render(issue)?;
            let bytes = rendered.into_bytes();
            let oid = gix::objs::compute_hash(hash_kind, gix::object::Kind::Blob, &bytes)
                .map_err(|e| Error::Git(e.to_string()))?;
            let filename = format!("{}.md", issue.id.0);
            records.push((
                gix::objs::tree::Entry {
                    mode: gix::object::tree::EntryKind::Blob.into(),
                    filename: filename.into(),
                    oid,
                },
                issue.id.0.to_string(),
            ));
        }
        // Sort by filename — git's tree-entry ordering for plain files
        // is lexicographic by raw bytes.
        records.sort_by(|a, b| a.0.filename.cmp(&b.0.filename));

        // Populate oid_map + fire tree_sync audit row + upsert
        // last_fetched_at. We hold the lock only for these fast SQL
        // calls; it's released before the git object writes.
        {
            let conn = self.db.lock().expect("cache db mutex poisoned");
            for (entry, issue_id) in &records {
                meta::put_oid_mapping(
                    &conn,
                    &self.backend_name,
                    &self.project,
                    &entry.oid.to_hex().to_string(),
                    issue_id,
                )?;
            }
            audit::log_tree_sync(&conn, &self.backend_name, &self.project, records.len());
            meta::set_meta(&conn, "last_fetched_at", &Utc::now().to_rfc3339())?;
        }

        let inner_tree = gix::objs::Tree {
            entries: records.into_iter().map(|(e, _)| e).collect(),
        };
        let inner_tree_oid = self
            .repo
            .write_object(&inner_tree)
            .map_err(|e| Error::Git(e.to_string()))?
            .detach();

        // Outer tree with one entry: `issues/` -> inner_tree_oid.
        let outer_tree = gix::objs::Tree {
            entries: vec![gix::objs::tree::Entry {
                mode: gix::object::tree::EntryKind::Tree.into(),
                filename: b"issues".as_slice().into(),
                oid: inner_tree_oid,
            }],
        };
        let tree_oid = self
            .repo
            .write_object(&outer_tree)
            .map_err(|e| Error::Git(e.to_string()))?
            .detach();

        // Commit. If `refs/heads/main` already points at a commit (re-seed
        // path — e.g. Q1.3 idempotent re-attach), chain the new commit
        // off the existing one. `repo.commit` writes the ref via
        // `PreviousValue::MustExistAndMatch(...)` semantics: passing a
        // mismatched parent set against an existing ref errors out.
        // Reusing the existing tip as parent makes seed-then-seed
        // succeed without losing history.
        let msg = format!(
            "sync({}:{}): {} issues at {}",
            self.backend_name,
            self.project,
            inner_tree.entries.len(),
            Utc::now().to_rfc3339()
        );
        let parent_commit: Option<gix::ObjectId> = self
            .repo
            .find_reference("refs/heads/main")
            .ok()
            .and_then(|mut r| r.peel_to_id().ok().map(gix::Id::detach));
        let commit_oid = self
            .repo
            .commit("refs/heads/main", msg, tree_oid, parent_commit)
            .map_err(|e| Error::Git(e.to_string()))?;

        // Explicitly point HEAD at refs/heads/main to defend against
        // `init.defaultBranch = master` leaking in from the user's
        // ~/.gitconfig (RESEARCH §Pitfall 5).
        let head_path = self.path.join("HEAD");
        std::fs::write(&head_path, "ref: refs/heads/main\n")?;

        // Time-travel: tag this seed-path commit as a sync point. Best-effort
        // — a failed tag write should not poison the seed sync (which already
        // committed `meta.last_fetched_at`).
        let new_oid = commit_oid.detach();
        if let Err(e) = self.tag_sync(new_oid, Utc::now()) {
            tracing::warn!(target: "reposix_cache::sync_tag",
                           backend = self.backend_name.as_str(),
                           project = self.project.as_str(),
                           "tag_sync (seed) failed: {e}");
        }

        Ok(new_oid)
    }

    /// Delta-sync the cache against the backend.
    ///
    /// Flow:
    /// 1. Read `meta.last_fetched_at`. If absent → fall through to
    ///    [`Cache::build_from`] (seed path) and return.
    /// 2. Call [`reposix_core::BackendConnector::list_changed_since`] with
    ///    the cursor. Returns the IDs the backend reports changed.
    /// 3. For each changed ID, GET the full issue, render to canonical
    ///    bytes, write the blob into the bare repo (eager materialization
    ///    on the delta path — changed items are almost certainly what
    ///    the agent is about to read).
    /// 4. Re-list the full issue set via `list_records` (cheap metadata)
    ///    and rebuild the tree with current blob OIDs. Tree sync is
    ///    unconditional full per CONTEXT.md §"Tree sync vs. blob
    ///    materialization (locked)".
    /// 5. In ONE [`rusqlite::Transaction`]: upsert `oid_map` rows for
    ///    changed items, update `meta.last_fetched_at`, insert the
    ///    `op='delta_sync'` audit row.
    ///
    /// An empty-delta sync still bumps `last_fetched_at` and writes an
    /// audit row with `bytes=0` — intentional so audit history has one
    /// row per fetch invocation.
    ///
    /// # Errors
    /// Mirrors [`Cache::build_from`] plus the seed-path errors. The
    /// transaction boundary guarantees no torn state: if any post-fetch
    /// step fails, `last_fetched_at` is unchanged and the next sync
    /// retries the same window.
    ///
    /// # Panics
    /// Panics if the internal `cache.db` mutex is poisoned.
    // 5-step orchestration (read cursor → list_changed_since → materialize
    // changed blobs → rebuild full tree → atomic SQL transaction) is
    // intrinsic to the spec; splitting would obscure the audit ordering
    // documented above.
    #[allow(clippy::too_many_lines)]
    pub async fn sync(&self) -> Result<SyncReport> {
        // Step 1: read last_fetched_at. Absent → seed path.
        let since_raw: Option<String> = {
            let conn = self.db.lock().expect("cache db mutex poisoned");
            meta::get_meta(&conn, "last_fetched_at")?
        };
        let since = match since_raw {
            Some(ref s) => Some(
                chrono::DateTime::parse_from_rfc3339(s)
                    .map_err(|e| Error::Sqlite(format!("bad last_fetched_at {s}: {e}")))?
                    .with_timezone(&Utc),
            ),
            None => None,
        };
        let Some(since_dt) = since else {
            // Seed path: forward to build_from. build_from already writes
            // last_fetched_at and the tree_sync audit row. We do NOT also
            // write a delta_sync audit row — the seed is a distinct event.
            let commit = self.build_from().await?;
            return Ok(SyncReport {
                changed_ids: Vec::new(),
                since: None,
                new_commit: Some(commit),
            });
        };

        // Step 2: incremental query.
        let changed_ids = match self
            .backend
            .list_changed_since(&self.project, since_dt)
            .await
        {
            Ok(v) => v,
            Err(e) => return Err(self.classify_backend_error(&e, None)),
        };

        // Step 3: materialize each changed issue's blob eagerly.
        let hash_kind = self.repo.object_hash();
        let mut changed_blob_oids: Vec<(RecordId, gix::ObjectId)> =
            Vec::with_capacity(changed_ids.len());
        for id in &changed_ids {
            let issue = match self.backend.get_record(&self.project, *id).await {
                Ok(i) => i,
                Err(e) => return Err(self.classify_backend_error(&e, Some(&id.0.to_string()))),
            };
            let rendered = frontmatter::render(&issue)?;
            let bytes = rendered.into_bytes();
            let blob_oid = self
                .repo
                .write_blob(&bytes)
                .map_err(|e| Error::Git(e.to_string()))?
                .detach();
            // Sanity: recompute expected hash and compare.
            let expected = gix::objs::compute_hash(hash_kind, gix::object::Kind::Blob, &bytes)
                .map_err(|e| Error::Git(e.to_string()))?;
            if expected != blob_oid {
                return Err(Error::Git(format!(
                    "hash mismatch for issue {}: expected {expected} got {blob_oid}",
                    id.0
                )));
            }
            changed_blob_oids.push((*id, blob_oid));
        }

        // Step 4: re-list the full current set for unconditional full
        // tree sync (per CONTEXT.md §Tree sync vs. blob materialization).
        let all_issues = match self.backend.list_records(&self.project).await {
            Ok(v) => v,
            Err(e) => return Err(self.classify_backend_error(&e, None)),
        };
        let mut records: Vec<(gix::objs::tree::Entry, String)> =
            Vec::with_capacity(all_issues.len());
        for issue in &all_issues {
            // Prefer the freshly-written blob oid for changed items.
            // For unchanged items, recompute the OID without writing the
            // blob (lazy-blob invariant — only the changed delta is
            // materialized).
            let oid = if let Some((_, freshly_written)) =
                changed_blob_oids.iter().find(|(id, _)| *id == issue.id)
            {
                *freshly_written
            } else {
                let rendered = frontmatter::render(issue)?;
                let bytes = rendered.into_bytes();
                gix::objs::compute_hash(hash_kind, gix::object::Kind::Blob, &bytes)
                    .map_err(|e| Error::Git(e.to_string()))?
            };
            let filename = format!("{}.md", issue.id.0);
            records.push((
                gix::objs::tree::Entry {
                    mode: gix::object::tree::EntryKind::Blob.into(),
                    filename: filename.into(),
                    oid,
                },
                issue.id.0.to_string(),
            ));
        }
        records.sort_by(|a, b| a.0.filename.cmp(&b.0.filename));

        let inner_tree = gix::objs::Tree {
            entries: records.iter().map(|(e, _)| e.clone()).collect(),
        };
        let inner_tree_oid = self
            .repo
            .write_object(&inner_tree)
            .map_err(|e| Error::Git(e.to_string()))?
            .detach();
        let outer_tree = gix::objs::Tree {
            entries: vec![gix::objs::tree::Entry {
                mode: gix::object::tree::EntryKind::Tree.into(),
                filename: b"issues".as_slice().into(),
                oid: inner_tree_oid,
            }],
        };
        let tree_oid = self
            .repo
            .write_object(&outer_tree)
            .map_err(|e| Error::Git(e.to_string()))?
            .detach();

        // Parent = current HEAD commit (for chained history).
        let parent_commit: Option<gix::ObjectId> = self
            .repo
            .find_reference("refs/heads/main")
            .ok()
            .and_then(|mut r| r.peel_to_id().ok().map(gix::Id::detach));
        let msg = format!(
            "delta-sync({}:{}): {} changed (of {}) at {}",
            self.backend_name,
            self.project,
            changed_ids.len(),
            all_issues.len(),
            Utc::now().to_rfc3339()
        );
        let new_commit = self
            .repo
            .commit("refs/heads/main", msg, tree_oid, parent_commit)
            .map_err(|e| Error::Git(e.to_string()))?
            .detach();

        // Step 5: ATOMIC transaction — oid_map upserts + last_fetched_at
        // + delta_sync audit row.
        let since_iso = since_dt.to_rfc3339();
        let now_iso = Utc::now().to_rfc3339();
        let items_returned = changed_ids.len();
        {
            let mut conn = self.db.lock().expect("cache db mutex poisoned");
            let tx = conn.transaction()?;
            for (id, oid) in &changed_blob_oids {
                tx.execute(
                    "INSERT OR REPLACE INTO oid_map (oid, issue_id, backend, project) \
                     VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![
                        oid.to_hex().to_string(),
                        id.0.to_string(),
                        &self.backend_name,
                        &self.project,
                    ],
                )?;
            }
            tx.execute(
                "INSERT INTO meta (key, value, updated_at) VALUES ('last_fetched_at', ?1, ?2) \
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
                rusqlite::params![&now_iso, &now_iso],
            )?;
            audit::log_delta_sync_tx(
                &tx,
                &self.backend_name,
                &self.project,
                Some(&since_iso),
                items_returned,
            )?;
            tx.commit()?;
        }

        // Time-travel: tag the delta commit. Best-effort — the SQL
        // transaction has already committed cursor + audit row, so a tag
        // write failure here doesn't risk torn state, only loses the tag.
        if let Err(e) = self.tag_sync(new_commit, Utc::now()) {
            tracing::warn!(target: "reposix_cache::sync_tag",
                           backend = self.backend_name.as_str(),
                           project = self.project.as_str(),
                           "tag_sync (delta) failed: {e}");
        }

        Ok(SyncReport {
            changed_ids,
            since: Some(since_dt),
            new_commit: Some(new_commit),
        })
    }

    /// Materialize a blob by OID. Writes the blob object to
    /// `.git/objects/` and returns its bytes wrapped in [`Tainted`].
    ///
    /// Side effects on `cache.db`:
    /// - on success: one `op='materialize'` audit row;
    /// - on egress denial: one `op='egress_denied'` audit row fired
    ///   BEFORE returning [`Error::Egress`].
    ///
    /// Second calls with the same OID re-fire the audit row but do NOT
    /// duplicate the blob (gix content-addresses objects; re-writing
    /// the same bytes yields the same OID and is a no-op on disk).
    ///
    /// # Errors
    /// - [`Error::UnknownOid`] — the OID has no entry in `oid_map`.
    /// - [`Error::Egress`] — the backend's origin is not in the
    ///   `REPOSIX_ALLOWED_ORIGINS` allowlist (audit row fired first).
    /// - [`Error::Backend`] — any other backend failure.
    /// - [`Error::OidDrift`] — backend returned bytes that hash to a
    ///   different OID than requested (eventual-consistency race).
    /// - [`Error::Render`] — frontmatter rendering failed.
    /// - [`Error::Git`] — `gix::Repository::write_blob` failed.
    ///
    /// # Panics
    /// Panics if the internal `cache.db` mutex is poisoned (another
    /// thread panicked while holding it).
    pub async fn read_blob(&self, oid: gix::ObjectId) -> Result<Tainted<Vec<u8>>> {
        let oid_hex = oid.to_hex().to_string();

        // Look up issue_id without holding the lock across the await.
        let issue_id_str = {
            let conn = self.db.lock().expect("cache db mutex poisoned");
            meta::get_issue_for_oid(&conn, &oid_hex)?
                .ok_or_else(|| Error::UnknownOid(oid_hex.clone()))?
        };

        // Parse back to RecordId.
        let issue_num: u64 = issue_id_str.parse().map_err(|_| {
            Error::Backend(format!("oid_map issue_id {issue_id_str} is not numeric"))
        })?;

        // Call backend. On InvalidOrigin, fire egress_denied audit row
        // THEN return Egress.
        let issue = match self
            .backend
            .get_record(&self.project, RecordId(issue_num))
            .await
        {
            Ok(i) => i,
            Err(e) => {
                return Err(self.classify_backend_error(&e, Some(&issue_id_str)));
            }
        };

        // Render and write the blob.
        let rendered = reposix_core::frontmatter::render(&issue)?;
        let bytes = rendered.into_bytes();
        let written_oid = self
            .repo
            .write_blob(&bytes)
            .map_err(|e| Error::Git(e.to_string()))?
            .detach();

        // Consistency check: backend content might have drifted since
        // build_from.
        if written_oid != oid {
            return Err(Error::OidDrift {
                requested: oid_hex,
                actual: written_oid.to_hex().to_string(),
                issue_id: issue_id_str,
            });
        }

        // Audit. Best-effort.
        {
            let conn = self.db.lock().expect("cache db mutex poisoned");
            audit::log_materialize(
                &conn,
                &self.backend_name,
                &self.project,
                &issue_id_str,
                &oid_hex,
                bytes.len(),
            );
        }

        Ok(Tainted::new(bytes))
    }

    /// Map a `reposix_core::Error` from a backend call into the cache's
    /// typed error space. If it looks like an egress-allowlist denial,
    /// fire the `egress_denied` audit row BEFORE returning
    /// [`Error::Egress`]. Otherwise surface as [`Error::Backend`].
    ///
    /// Detection is both typed (`matches!(e,
    /// reposix_core::Error::InvalidOrigin(_))`) and stringly (substring
    /// match on the error message) to handle backend adapters that
    /// wrap the core error in `Error::Other(String)` — the `Confluence`,
    /// `Jira`, and `Github` adapters all do this for non-2xx responses.
    /// A future cleanup may tighten this to a proper typed error refactor.
    fn classify_backend_error(&self, e: &reposix_core::Error, issue_id: Option<&str>) -> Error {
        let emsg = e.to_string();
        let is_egress = matches!(e, reposix_core::Error::InvalidOrigin(_))
            || emsg.contains("blocked origin")
            || emsg.contains("invalid origin")
            || emsg.contains("allowlist");
        if is_egress {
            if let Ok(conn) = self.db.lock() {
                audit::log_egress_denied(&conn, &self.backend_name, &self.project, issue_id, &emsg);
            }
            Error::Egress(emsg)
        } else {
            Error::Backend(emsg)
        }
    }
}
