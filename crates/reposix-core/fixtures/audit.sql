-- Append-only audit log for every network-touching action in reposix.
-- Loaded at simulator startup via reposix_core::audit::load_schema.
-- SG-06 (audit log append-only) is enforced by the BEFORE UPDATE / DELETE
-- triggers below; integration tests assert both fire on real SQLite.

CREATE TABLE IF NOT EXISTS audit_events (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    ts               TEXT    NOT NULL,
    agent_id         TEXT,
    method           TEXT    NOT NULL,
    path             TEXT    NOT NULL,
    status           INTEGER,
    request_body     TEXT,
    response_summary TEXT
);

CREATE TRIGGER IF NOT EXISTS audit_no_update
    BEFORE UPDATE ON audit_events
    BEGIN
        SELECT RAISE(ABORT, 'audit_events is append-only');
    END;

CREATE TRIGGER IF NOT EXISTS audit_no_delete
    BEFORE DELETE ON audit_events
    BEGIN
        SELECT RAISE(ABORT, 'audit_events is append-only');
    END;
