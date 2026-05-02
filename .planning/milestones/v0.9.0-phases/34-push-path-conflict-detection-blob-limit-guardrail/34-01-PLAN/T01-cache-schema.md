← [back to index](./index.md)

# Task 01-T01 — Extend `cache_schema.sql` CHECK constraint with `blob_limit_exceeded`

<read_first>
- `crates/reposix-cache/fixtures/cache_schema.sql` (entire file)
- `crates/reposix-cache/src/db.rs` (entire file — confirm schema is loaded via `execute_batch` and that DEFENSIVE comes before schema)
</read_first>

<action>
Edit `crates/reposix-cache/fixtures/cache_schema.sql`. In the `op TEXT NOT NULL CHECK (op IN (...))` list, after `'delta_sync'` add `'blob_limit_exceeded'` AND `'helper_push_started'` AND `'helper_push_accepted'` AND `'helper_push_rejected_conflict'` AND `'helper_push_sanitized_field'` (the last four are added pre-emptively so Plan 02 does not need a second migration; Plan 01 only writes `blob_limit_exceeded`).

Add an inline comment above the CHECK list noting that v0.9.0 cache.db files keep the older CHECK list because `CREATE TABLE IF NOT EXISTS` skips on existing tables. For fresh caches the new list applies; for existing caches the audit insert will fail with a CHECK violation and fall through the `warn!` best-effort path — acceptable for Plan 01/02 because audit rows are best-effort. (The existing comment on lines 9-13 already documents this for Phase 33's `delta_sync`; extend that note.)

Do NOT change the trigger definitions. Do NOT add new columns.

Verify shape after edit:

```bash
grep -A 12 "op            TEXT" crates/reposix-cache/fixtures/cache_schema.sql
```

Must output a CHECK list ending with the five new entries. Trigger block unchanged.
</action>

<acceptance_criteria>
- `grep "'blob_limit_exceeded'" crates/reposix-cache/fixtures/cache_schema.sql` matches once.
- `grep "'helper_push_started'" crates/reposix-cache/fixtures/cache_schema.sql` matches once.
- `grep "'helper_push_accepted'" crates/reposix-cache/fixtures/cache_schema.sql` matches once.
- `grep "'helper_push_rejected_conflict'" crates/reposix-cache/fixtures/cache_schema.sql` matches once.
- `grep "'helper_push_sanitized_field'" crates/reposix-cache/fixtures/cache_schema.sql` matches once.
- `grep "audit_cache_no_update" crates/reposix-cache/fixtures/cache_schema.sql` still matches (trigger preserved).
- `cargo build -p reposix-cache` exits 0.
- The existing test `crates/reposix-cache/src/db.rs::tests::cache_db_has_append_only_triggers` still passes.
</acceptance_criteria>

<threat_model>
Adding values to the CHECK list expands what audit rows can be written but does NOT bypass the append-only triggers (which apply to UPDATE/DELETE, not INSERT). The DEFENSIVE PRAGMA + 0o600 file mode are unaffected. No exfil surface.
</threat_model>
