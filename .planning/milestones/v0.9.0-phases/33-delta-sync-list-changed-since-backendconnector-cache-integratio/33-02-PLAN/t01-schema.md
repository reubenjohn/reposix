← [back to index](./index.md)

# Task 02-T01 — Extend schema CHECK constraint for `op='delta_sync'`

<read_first>
- `crates/reposix-cache/fixtures/cache_schema.sql`
- `crates/reposix-cache/src/db.rs::tests::cache_db_has_append_only_triggers`
</read_first>

<action>
Edit `crates/reposix-cache/fixtures/cache_schema.sql` — change the `op IN (...)` list on `audit_events_cache` to include `'delta_sync'` alongside the existing ops AND the ops added in Phase 32 (grep the current file to confirm the exact set; Phase 31's schema had `'materialize','egress_denied','tree_sync'`; Phase 32 may have widened this to include `'helper_connect'`, `'helper_advertise'`, `'helper_fetch'`, `'helper_fetch_error'` — verify before editing).

Replace the existing `CHECK (op IN (...))` line with:

```sql
op TEXT NOT NULL CHECK (op IN (
    'materialize',
    'egress_denied',
    'tree_sync',
    'helper_connect',
    'helper_advertise',
    'helper_fetch',
    'helper_fetch_error',
    'delta_sync'
)),
```

Keep every op from the existing CHECK constraint — only add `'delta_sync'`. If any op listed above is NOT in the current file, remove it from the new list (we are adding one op, not adding four).

**Important:** this is a schema change on `CREATE TABLE IF NOT EXISTS`. Fresh databases will pick up the new constraint; existing cache DBs on disk retain the old CHECK. This is acceptable for v0.9.0 (pre-1.0, cache is local ephemeral state). Document it inline as a `-- NOTE:` comment.
</action>

<acceptance_criteria>
- `grep -c "'delta_sync'" crates/reposix-cache/fixtures/cache_schema.sql` returns 1.
- `cargo test -p reposix-cache cache_db_has_expected_tables` passes.
- `cargo test -p reposix-cache cache_db_has_append_only_triggers` passes.
- A new test asserts that inserting `op='delta_sync'` row succeeds (added in Task 02-T02).
</acceptance_criteria>

<threat_model>
The CHECK constraint is the mechanical enforcement of ARCH-02's "audit table is append-only AND closed-set of ops". Widening the set is an explicit code change, reviewable in diff. Combined with the existing `audit_cache_no_update`/`audit_cache_no_delete` triggers, an attacker who compromises the writing process still cannot forge arbitrary op values.
</threat_model>
