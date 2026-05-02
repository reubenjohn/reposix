← [back to index](./index.md) · phase 81 plan 01

## Task 81-01-T02a — Read-first + cache cursor wrappers (`read_last_fetched_at` / `write_last_fetched_at`)

*This is part 1 of 3 for T02. Continues in [T02b](./T02b-state-widening-precheck-module.md) (State widening + `precheck.rs`) and [T02c](./T02c-handle-export-rewrite-cursor-write.md) (`handle_export` rewrite + cursor-write + build/commit).*

<read_first>
- `crates/reposix-cache/src/meta.rs` (entire file — 67 lines; the
  `set_meta` / `get_meta` API the new wrappers call).
- `crates/reposix-cache/src/cache.rs` lines 1-100 (`Cache::open` +
  field declarations — confirm `db: Mutex<Connection>` field
  availability and the existing log-helper-* family pattern).
- `crates/reposix-cache/src/cache.rs` lines 232-310 (`log_helper_*`
  family — style precedent for new method placement).
- `crates/reposix-cache/src/cache.rs` lines 345-400 (`list_record_ids`
  + `find_oid_for_record` — used by the new precheck path).
- `crates/reposix-cache/src/builder.rs` lines 226-249 (`Cache::sync`
  cursor read+seed-fallback shape — T02's `read_last_fetched_at`
  parses RFC3339 verbatim from this pattern).
- `crates/reposix-cache/src/lib.rs` (entire file — to confirm the
  cache crate's pub-mod list; `read_blob` returns `Tainted<Vec<u8>>`).
- `crates/reposix-remote/src/main.rs` (entire `handle_export` function
  — currently lines 280-549 post-P80; the rewrite scope is lines
  334-382 + the cursor-write insertion point near line 491).
  **Re-confirm the line numbers via `grep -n 'fn handle_export\|state.backend.list_records\|log_helper_push_accepted\|refresh_for_mirror_head' crates/reposix-remote/src/main.rs`** before editing — P80's edits shifted the region.
- `crates/reposix-remote/src/main.rs` lines 24-32 (`mod backend_dispatch;`
  + `use crate::backend_dispatch::...;` — the new `mod precheck;`
  declaration sits alphabetically between these).
- `crates/reposix-remote/src/main.rs` lines 40-71 (`State` struct —
  confirm `state.cache: Option<Cache>`, `state.backend_name: String`,
  `state.rt: tokio::runtime::Runtime`, `state.backend: Box<dyn BackendConnector>`).
- `crates/reposix-remote/src/diff.rs` lines 99-202 (`plan` function —
  signature `prior: &[Record]` UNCHANGED in P81 per D-03).
- `crates/reposix-remote/src/fast_import.rs` (find `ParsedExport`
  struct — confirm field shape: `commit_message`, `blobs: HashMap<u32, Vec<u8>>`,
  `tree: BTreeMap<String, u32>`, `deletes: Vec<String>`).
- `crates/reposix-core/src/backend.rs` lines 235-264 (`BackendConnector`
  trait definition — confirm `list_records` and `list_changed_since`
  signatures).
- `crates/reposix-core/src/lib.rs` (find `Tainted<T>` re-export — the
  `Tainted::inner_ref()` accessor T02 uses).
- gix 0.83 docs / `crates/reposix-cache/src/cache.rs::read_blob` —
  confirm `read_blob` returns `Tainted<Vec<u8>>`.
</read_first>

<action>
Three concerns in this task; keep ordering: cache wrappers (cache crate)
→ new `precheck.rs` module (remote crate) → `handle_export` rewrite
(remote crate) → cursor-write insertion (remote crate) → cargo check +
nextest + commit.

### 2a. Cache wrappers — `crates/reposix-cache/src/cache.rs`

Append the two wrapper methods to the existing `impl Cache` block. Place
them AFTER the existing `log_*` family (line ~470 post-P79's
`log_attach_walk`). Two methods:

```rust
    /// Read the cache's `meta.last_fetched_at` cursor — the timestamp
    /// of the most recent successful `Cache::build_from` or
    /// `Cache::sync` call. Used by the helper's L1 precheck on push
    /// entry (`crates/reposix-remote/src/precheck.rs`).
    ///
    /// Returns:
    /// - `Ok(Some(ts))` — the cursor is populated; the helper passes
    ///   `ts` to `BackendConnector::list_changed_since`.
    /// - `Ok(None)` — the cursor is absent (fresh cache, never built;
    ///   OR the stored string failed to parse defensively). The
    ///   helper falls through to a `list_records` walk for THIS push
    ///   only (first-push fallback per
    ///   `architecture-sketch.md § Performance subtlety` and
    ///   RESEARCH.md § Pitfall 1).
    ///
    /// # Errors
    /// - [`Error::Sqlite`] for any rusqlite I/O failure.
    ///
    /// # Panics
    /// Panics if the internal `cache.db` mutex is poisoned.
    pub fn read_last_fetched_at(&self) -> Result<Option<chrono::DateTime<chrono::Utc>>> {
        let conn = self.db.lock().expect("cache.db mutex poisoned");
        let raw: Option<String> = crate::meta::get_meta(&conn, "last_fetched_at")?;
        let Some(s) = raw else {
            return Ok(None);
        };
        match chrono::DateTime::parse_from_rfc3339(&s) {
            Ok(dt) => Ok(Some(dt.with_timezone(&chrono::Utc))),
            Err(e) => {
                // Defensive: malformed RFC3339 in the cursor row should
                // not poison the precheck path. WARN-log and fall back
                // to first-push semantics. This is the same shape as
                // the parse-error guard in builder.rs:233-236, except
                // we degrade to None instead of erroring — the helper's
                // first-push fallback is the intended recovery path.
                tracing::warn!(
                    "cache.last_fetched_at malformed: {s:?}: {e}; falling back to first-push semantics"
                );
                Ok(None)
            }
        }
    }

    /// Write the cache's `meta.last_fetched_at` cursor. Called by the
    /// helper after a successful push so the next push's precheck has
    /// a recent cursor.
    ///
    /// Best-effort caller pattern: callers should `tracing::warn!` on
    /// failure and continue. The push still acks `ok` to git. Cursor
    /// drift is recoverable via `reposix sync --reconcile` (the L1
    /// escape hatch).
    ///
    /// # Errors
    /// - [`Error::Sqlite`] for any rusqlite I/O failure.
    ///
    /// # Panics
    /// Panics if the internal `cache.db` mutex is poisoned.
    pub fn write_last_fetched_at(&self, ts: chrono::DateTime<chrono::Utc>) -> Result<()> {
        let conn = self.db.lock().expect("cache.db mutex poisoned");
        crate::meta::set_meta(&conn, "last_fetched_at", &ts.to_rfc3339())
    }
```

Append the two unit tests inside the existing
`#[cfg(test)] mod tests` block at the bottom of `cache.rs` (or, if
that block is too crowded, add a new `#[cfg(test)] mod last_fetched_at_tests`
section — match the existing test-organization style):

```rust
    #[test]
    fn read_last_fetched_at_round_trips() {
        let tmp = tempfile::tempdir().expect("tempdir");
        // Use a deterministic backend; sim is non-network for the test.
        let cache = open_test_cache(tmp.path(), "sim", "demo");
        // Use second precision so to_rfc3339 + parse_from_rfc3339 round-trip exactly.
        let t1: chrono::DateTime<chrono::Utc> =
            "2026-05-01T12:34:56Z".parse().expect("parse t1");
        cache
            .write_last_fetched_at(t1)
            .expect("write_last_fetched_at");
        let read_back = cache
            .read_last_fetched_at()
            .expect("read_last_fetched_at")
            .expect("cursor present after write");
        assert_eq!(read_back, t1);
    }

    #[test]
    fn read_last_fetched_at_returns_none_when_absent() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cache = open_test_cache(tmp.path(), "sim", "demo");
        let result = cache
            .read_last_fetched_at()
            .expect("read should succeed even when cursor absent");
        assert!(result.is_none(), "expected None for fresh cache; got {result:?}");
    }
```

The `open_test_cache` helper MUST be reused if it already exists in
the existing test module; if not, add a fresh one mirroring the P80
test pattern (`Cache::open(path, "sim", "demo")` — the exact signature
matches `crates/reposix-cache/src/cache.rs:54`). The test pattern
relies on the fact that `Cache::open` does NOT auto-call `build_from`
— the cursor remains absent until something writes it. Confirm this
in T02 read_first.

Build serially:

```bash
cargo check -p reposix-cache
cargo clippy -p reposix-cache -- -D warnings
cargo nextest run -p reposix-cache last_fetched_at
```

If `cargo clippy` fires `clippy::pedantic` lints (e.g., `must_use`
attribute on `read_last_fetched_at` — the return type carries semantic
meaning), fix inline; do NOT add `#[allow(...)]` without rationale.
Each new public fn must have a `# Errors` doc.

*Continue to [T02b](./T02b-state-widening-precheck-module.md) for State widening + `precheck.rs` module.*
</action>
