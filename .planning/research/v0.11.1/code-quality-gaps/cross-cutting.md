← [back to index](./index.md)

# Cross-cutting refactor opportunities

### CC-1. Promote `cache_path_from_worktree` (the canonical one) to `reposix-cache`

`reposix-cache::path::resolve_cache_path` already takes `(backend, project)`. The CLI wrapper `worktree_helpers::cache_path_from_worktree` reads `remote.origin.url` + `parse_remote_url` + `backend_slug_from_origin`. The *whole pipeline* is generic and belongs in `reposix-cache` as `Cache::path_for_worktree(work: &Path) -> Result<PathBuf>`. The `reposix-cli` layer disappears. Same logic also wanted by the `reposix-remote` helper for the dark-factory rebase teaching path.

### CC-2. Two cache.db schemas → one

See P0-4. `reposix-cli/src/cache_db.rs` (`refresh_meta` schema) and `reposix-cache/src/db.rs` (`audit_events_cache + meta + oid_map`) coexist on disk at different paths. The `refresh_meta` shape is one row of `(backend, project, last_fetched_at, commit_sha)` — that is a `meta` table by another name. Migrate `refresh.rs` to use the cache crate's `meta::set_meta('refresh.last_fetched_at', …)`, delete `cache_db.rs`. Cuts ~250 LOC + one SQLite file per worktree.

### CC-3. Backend-write-side audit hook → trait method, not per-backend SQLite connection

Each backend that mutates external state (`confluence::create_record`, `jira::create_record`, etc.) should take an `&dyn AuditSink` instead of an `Option<Arc<Mutex<Connection>>>`. The simulator's `audit_events` table writer becomes one impl; the cache's `audit_events_cache` writer becomes another. Backends drop `rusqlite/sha2/hex` deps entirely.

### CC-4. Single `# Panics` doc cluster

`crates/reposix-cache/src/cache.rs` has 13 audit-forwarder methods with identical 5-line `# Panics` docs. After P1-8 (`with_db` helper), one module-level `## Panics` note replaces all of them.
