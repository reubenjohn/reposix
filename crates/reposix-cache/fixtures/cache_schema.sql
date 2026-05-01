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
    -- v0.11.0 §3j adds 'cache_gc' (LRU/TTL/all eviction). v0.11.0 §3c
    -- adds 'token_cost' (per-RPC token-economy ledger). v0.13.0 P79-02
    -- adds 'attach_walk' (DVCS-ATTACH-02; one row per `reposix attach`
    -- walk completion); v0.13.0 P80-01 adds 'mirror_sync_written'
    -- (DVCS-MIRROR-REFS-02 OP-3; one row per `handle_export` success
    -- branch — written even when ref writes fail). v0.13.0 P83-01 adds
    -- 'helper_push_partial_fail_mirror_lag' (DVCS-BUS-WRITE-02 OP-3;
    -- one row per bus push where SoT writes succeeded but the mirror
    -- push subprocess failed — recovery on next push or via webhook
    -- sync per Q3.6).
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
        'helper_push_sanitized_field',
        'helper_backend_instantiated',
        'sync_tag_written',
        'cache_gc',
        'token_cost',
        'attach_walk',
        'mirror_sync_written',
        'helper_push_partial_fail_mirror_lag'
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

-- v0.13.0 P79-02: cache_reconciliation tracks the working-tree → backend
-- record mapping captured at `reposix attach` time. INSERT OR REPLACE on
-- re-attach (idempotent per Q1.3). Reconciliation state, NOT an audit
-- table — append-only triggers do not apply.
CREATE TABLE IF NOT EXISTS cache_reconciliation (
    record_id    INTEGER PRIMARY KEY,
    oid          TEXT    NOT NULL,
    local_path   TEXT    NOT NULL,
    attached_at  TEXT    NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_cache_reconciliation_local_path
    ON cache_reconciliation(local_path);

-- v0.13.0 P79-02: extend the audit_events_cache CHECK list with
-- `attach_walk` (DVCS-ATTACH-02 / OP-3) so `Cache::log_attach_walk`
-- can write its row. Sibling P83-01 event
-- 'helper_push_partial_fail_mirror_lag' joined this list per
-- DVCS-BUS-WRITE-02; bus push paths write this row when SoT writes
-- succeed but the mirror push subprocess fails (Q3.6 RATIFIED
-- no-retry). NOTE: the CREATE TABLE IF NOT EXISTS above keeps the
-- legacy CHECK on existing cache.db files. Fresh caches created from
-- this schema get the extended list. See audit.sql comment "On stale
-- cache.db files the new ops will fail the CHECK and fall through
-- the audit best-effort path (warn-logged)".

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
