-- Source: pattern from crates/reposix-core/fixtures/audit.sql (append-only triggers
-- + idempotent DROP TRIGGER IF EXISTS pattern lifted verbatim).
-- Phase 31 Plan 02 — audit_events_cache schema. SG-06 append-only invariant
-- enforced via BEFORE UPDATE/DELETE triggers; paired with DEFENSIVE flag
-- in db.rs::open_cache_db to block writable_schema bypass.

BEGIN;

CREATE TABLE IF NOT EXISTS audit_events_cache (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    ts            TEXT    NOT NULL,
    -- NOTE: this CHECK is on `CREATE TABLE IF NOT EXISTS`, so existing
    -- cache.db files keep the older CHECK list. That's acceptable for
    -- v0.9.0 (pre-1.0, cache is local ephemeral state). Phase 33 adds
    -- 'delta_sync' alongside the helper_* ops the audit module already
    -- inserts in best-effort mode (failures are warn-logged). Phase 34
    -- extends the list with 'blob_limit_exceeded' (ARCH-09) and the four
    -- 'helper_push_*' ops (ARCH-08, ARCH-10). On stale cache.db files
    -- the new ops will fail the CHECK and fall through the audit
    -- best-effort path (warn-logged); fresh caches see the full list.
    op            TEXT    NOT NULL CHECK (op IN (
        'materialize',
        'egress_denied',
        'tree_sync',
        'helper_connect',
        'helper_advertise',
        'helper_fetch',
        'helper_fetch_error',
        'delta_sync',
        'blob_limit_exceeded',
        'helper_push_started',
        'helper_push_accepted',
        'helper_push_rejected_conflict',
        'helper_push_sanitized_field'
    )),
    backend       TEXT    NOT NULL,
    project       TEXT    NOT NULL,
    issue_id      TEXT,
    oid           TEXT,
    bytes         INTEGER,
    reason        TEXT
);

CREATE TABLE IF NOT EXISTS meta (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS oid_map (
    oid       TEXT PRIMARY KEY,
    issue_id  TEXT NOT NULL,
    backend   TEXT NOT NULL,
    project   TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_oid_map_issue
    ON oid_map(backend, project, issue_id);

DROP TRIGGER IF EXISTS audit_cache_no_update;
CREATE TRIGGER audit_cache_no_update BEFORE UPDATE ON audit_events_cache
    BEGIN
        SELECT RAISE(ABORT, 'audit_events_cache is append-only');
    END;

DROP TRIGGER IF EXISTS audit_cache_no_delete;
CREATE TRIGGER audit_cache_no_delete BEFORE DELETE ON audit_events_cache
    BEGIN
        SELECT RAISE(ABORT, 'audit_events_cache is append-only');
    END;

COMMIT;
