← [back to index](./index.md)

# Task 02-T03 — `Cache::sync` — atomic delta materialization

<read_first>
- `crates/reposix-cache/src/cache.rs`
- `crates/reposix-cache/src/builder.rs` (pattern — esp. `build_from` tree-assembly)
- `crates/reposix-cache/src/meta.rs` — `get_meta`, `set_meta`, `put_oid_mapping`
- `crates/reposix-cache/src/lib.rs` — for module re-exports
</read_first>

<action>
Edit `crates/reposix-cache/src/builder.rs` — append a new `impl Cache` block (or add to existing) with:

```rust
/// Result of a single `Cache::sync` invocation.
#[derive(Debug, Clone)]
pub struct SyncReport {
    /// Issue IDs the backend reported as changed since `since`.
    pub changed_ids: Vec<reposix_core::IssueId>,
    /// The timestamp passed to the backend (`None` if this was a seed
    /// sync — no prior `last_fetched_at`).
    pub since: Option<chrono::DateTime<chrono::Utc>>,
    /// New HEAD commit, if the sync produced a non-empty delta AND
    /// actually wrote a new tree. `None` if the delta was empty
    /// (meta update + audit row still happen — an empty sync is
    /// still a real sync with meaningful audit semantics).
    pub new_commit: Option<gix::ObjectId>,
}

impl Cache {
    /// Delta-sync the cache against the backend.
    ///
    /// Flow:
    /// 1. Read `meta.last_fetched_at` (if absent, fall through to
    ///    [`Cache::build_from`] for a full seed and return).
    /// 2. Call `backend.list_changed_since(project, last_fetched_at)`.
    /// 3. For each changed ID, `get_issue`, render, compute blob OID,
    ///    write blob (eager materialization on the delta path —
    ///    changed items are almost certainly what the agent is about
    ///    to read).
    /// 4. Re-list the full issue set via `list_issues` (cheap metadata),
    ///    rebuild the tree with current blob OIDs, commit.
    /// 5. In ONE `rusqlite::Transaction`: upsert `oid_map` for changed
    ///    items, update `meta.last_fetched_at`, insert
    ///    `op='delta_sync'` audit row.
    ///
    /// Idempotent: an empty-delta sync still bumps `last_fetched_at`
    /// and writes an audit row (count=0) — this is intentional so
    /// audit history has one row per fetch.
    ///
    /// # Errors
    /// Mirror [`Cache::build_from`]: `Error::Backend`, `Error::Egress`
    /// (audit row fires BEFORE the error), `Error::Render`, `Error::Git`,
    /// `Error::Sqlite`. The transaction boundary guarantees no partial
    /// meta/audit state: if any post-fetch step fails, `last_fetched_at`
    /// is unchanged and the next sync retries the same window.
    pub async fn sync(&self) -> Result<SyncReport> {
        // Step 1: read last_fetched_at. If absent, fall through to seed.
        let since_raw: Option<String> = {
            let conn = self.db.lock().expect("cache db mutex poisoned");
            meta::get_meta(&conn, "last_fetched_at")?
        };
        let since = match since_raw {
            Some(ref s) => {
                let dt = chrono::DateTime::parse_from_rfc3339(s)
                    .map_err(|e| Error::Sqlite(format!("bad last_fetched_at {s}: {e}")))?
                    .with_timezone(&chrono::Utc);
                Some(dt)
            }
            None => None,
        };
        let Some(since_dt) = since else {
            // Seed path: forward to build_from; wrap its commit oid.
            let commit = self.build_from().await?;
            // build_from already wrote last_fetched_at and tree_sync audit.
            // We do NOT also write a delta_sync audit row — the seed is
            // a distinct event (tree_sync), not a delta.
            return Ok(SyncReport {
                changed_ids: Vec::new(),
                since: None,
                new_commit: Some(commit),
            });
        };

        // Step 2: native-or-default incremental query.
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
        let mut changed_blob_oids: Vec<(reposix_core::IssueId, gix::ObjectId)> =
            Vec::with_capacity(changed_ids.len());
        for id in &changed_ids {
            let issue = match self.backend.get_issue(&self.project, *id).await {
                Ok(i) => i,
                Err(e) => return Err(self.classify_backend_error(&e, Some(&id.0.to_string()))),
            };
            let rendered = reposix_core::frontmatter::render(&issue)?;
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

        // Step 4: re-list the full current set for tree assembly. Tree
        // sync is unconditional full — see CONTEXT.md "Tree sync vs.
        // blob materialization".
        let all_issues = match self.backend.list_issues(&self.project).await {
            Ok(v) => v,
            Err(e) => return Err(self.classify_backend_error(&e, None)),
        };
        let mut records: Vec<(gix::objs::tree::Entry, String)> =
            Vec::with_capacity(all_issues.len());
        for issue in &all_issues {
            // Prefer the freshly-written blob oid for the changed set;
            // for unchanged items, recompute the OID (we have the full
            // Issue from list_issues — render and hash, don't write).
            let is_changed = changed_ids.contains(&issue.id);
            let oid = if is_changed {
                changed_blob_oids
                    .iter()
                    .find(|(id, _)| *id == issue.id)
                    .map(|(_, o)| *o)
                    .expect("changed_blob_oids contains all changed_ids")
            } else {
                let rendered = reposix_core::frontmatter::render(issue)?;
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

        // Parent = current HEAD commit.
        let parent_commit = self
            .repo
            .find_reference("refs/heads/main")
            .ok()
            .and_then(|r| r.try_id().map(|id| id.detach()));
        let msg = format!(
            "delta-sync({}:{}): {} changed (of {}) at {}",
            self.backend_name,
            self.project,
            changed_ids.len(),
            all_issues.len(),
            chrono::Utc::now().to_rfc3339()
        );
        let new_commit = self
            .repo
            .commit(
                "refs/heads/main",
                msg,
                tree_oid,
                parent_commit.into_iter(),
            )
            .map_err(|e| Error::Git(e.to_string()))?
            .detach();

        // Step 5: ATOMIC transaction — oid_map upserts + last_fetched_at + audit.
        let since_iso = since_dt.to_rfc3339();
        let now_iso = chrono::Utc::now().to_rfc3339();
        let items_returned = changed_ids.len();
        {
            let mut conn = self.db.lock().expect("cache db mutex poisoned");
            let tx = conn.transaction().map_err(Error::from)?;
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
            tx.commit().map_err(Error::from)?;
        }

        Ok(SyncReport {
            changed_ids,
            since: Some(since_dt),
            new_commit: Some(new_commit),
        })
    }
}
```

Add `use rusqlite;` or equivalent at the top of `builder.rs` if not already imported. Re-export `SyncReport` from `crates/reposix-cache/src/lib.rs`:

```rust
pub use crate::builder::SyncReport;
```
</action>

<acceptance_criteria>
- `cargo build -p reposix-cache` exits 0.
- `cargo clippy -p reposix-cache --all-targets -- -D warnings` exits 0.
- `grep -n 'pub struct SyncReport' crates/reposix-cache/src/builder.rs` matches.
- `grep -n 'pub async fn sync' crates/reposix-cache/src/builder.rs` matches.
- `grep -n 'pub use.*SyncReport' crates/reposix-cache/src/lib.rs` matches.
</acceptance_criteria>

<threat_model>
Atomic tx is the defense against the torn-state attack (attacker crashes the process mid-sync to confuse downstream consumers about the cache's cursor). The egress allowlist is re-enforced on every `get_issue` / `list_issues` / `list_changed_since` call via `http::client()`. Tainted bytes from the backend flow into blob content — this is acceptable because git's content-addressed storage + the `Tainted<Vec<u8>>` wrapper on `read_blob` preserve the discipline.
</threat_model>
