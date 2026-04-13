---
phase: 01-core-contracts-security-guardrails
plan: 03
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/reposix-core/Cargo.toml
  - crates/reposix-core/src/lib.rs
  - crates/reposix-core/src/audit.rs
  - crates/reposix-core/fixtures/audit.sql
  - crates/reposix-core/examples/show_audit_schema.rs
  - crates/reposix-core/tests/audit_schema.rs
autonomous: true
requirements:
  - SG-06
  - FC-06
user_setup: []

must_haves:
  truths:
    - "`reposix_core::audit::SCHEMA_SQL` is a non-empty `&'static str` loaded via `include_str!` from `fixtures/audit.sql`."
    - "`cargo run -q -p reposix-core --example show_audit_schema` prints DDL that contains `CREATE TRIGGER audit_no_update BEFORE UPDATE` and `CREATE TRIGGER audit_no_delete BEFORE DELETE` on `audit_events`."
    - "An in-memory SQLite opened and loaded with `SCHEMA_SQL` has a table `audit_events` with the exact columns listed in 01-CONTEXT.md."
    - "`pragma trigger_list` on the loaded DB returns at least `audit_no_update` and `audit_no_delete`."
    - "An `UPDATE audit_events SET path = 'x' WHERE id = 1` after inserting a row fails with the SQLite error raised by the BEFORE UPDATE trigger."
    - "An `DELETE FROM audit_events WHERE id = 1` after inserting a row fails with the SQLite error raised by the BEFORE DELETE trigger."
  artifacts:
    - path: "crates/reposix-core/fixtures/audit.sql"
      provides: "SQLite DDL for audit_events + BEFORE UPDATE/DELETE triggers"
      contains: "CREATE TRIGGER audit_no_update"
    - path: "crates/reposix-core/src/audit.rs"
      provides: "SCHEMA_SQL constant + load_schema(conn) helper"
      exports: ["SCHEMA_SQL", "load_schema"]
    - path: "crates/reposix-core/examples/show_audit_schema.rs"
      provides: "binary that prints SCHEMA_SQL — consumed by ROADMAP SC #3"
      contains: "SCHEMA_SQL"
    - path: "crates/reposix-core/tests/audit_schema.rs"
      provides: "integration test proving triggers fire on UPDATE/DELETE"
      contains: "audit_update_is_rejected_by_trigger"
  key_links:
    - from: "crates/reposix-core/src/audit.rs"
      to: "crates/reposix-core/fixtures/audit.sql"
      via: "include_str! at compile time"
      pattern: "include_str!"
    - from: "crates/reposix-core/examples/show_audit_schema.rs"
      to: "crates/reposix-core/src/audit.rs::SCHEMA_SQL"
      via: "println!(\"{SCHEMA_SQL}\")"
      pattern: "SCHEMA_SQL"
    - from: "crates/reposix-core/tests/audit_schema.rs"
      to: "crates/reposix-core/src/audit.rs::load_schema"
      via: "load_schema(&conn) then exec UPDATE/DELETE, assert Err"
      pattern: "load_schema"
---

<objective>
Publish the committed SQLite DDL that Phase 2 will load at sim-startup time. The schema must be append-only by construction: every UPDATE or DELETE against `audit_events` must fail with a trigger-raised `SQLITE_CONSTRAINT_TRIGGER`, not a prose comment. This plan closes SG-06 (audit log append-only) and the schema half of FC-06 (queryable SQLite audit log).

Purpose: make it physically impossible for Phase 2 (or anyone else) to mutate audit rows, without needing a runtime check. The DB itself refuses.

Output:
  - `crates/reposix-core/fixtures/audit.sql` with the full DDL.
  - `crates/reposix-core/src/audit.rs` exporting `SCHEMA_SQL: &'static str` (via `include_str!`) and `load_schema(conn: &Connection) -> Result<()>`.
  - `crates/reposix-core/examples/show_audit_schema.rs` — stdout-prints `SCHEMA_SQL`. This is the binary ROADMAP SC #3 exercises.
  - `crates/reposix-core/tests/audit_schema.rs` — integration test that opens an in-memory SQLite DB, loads the schema, inserts a row, then asserts both UPDATE and DELETE fail; also asserts `pragma trigger_list` lists both triggers.
  - `rusqlite = { workspace = true }` added to `reposix-core`'s `[dependencies]` (with `bundled`; already configured at workspace level — CLAUDE.md §Tech stack).
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/phases/01-core-contracts-security-guardrails/01-CONTEXT.md
@.planning/research/threat-model-and-critique.md
@CLAUDE.md
@Cargo.toml
@crates/reposix-core/Cargo.toml
@crates/reposix-core/src/lib.rs

<interfaces>
Public surface this plan MUST expose from `reposix_core::audit`:

    /// Canonical DDL for the append-only audit_events table.
    /// Phase 2's sim crate calls `load_schema(&conn)` at startup before any writes.
    pub const SCHEMA_SQL: &'static str = include_str!("../fixtures/audit.sql");

    /// Load the schema into an open SQLite connection. Idempotent — all
    /// statements use `IF NOT EXISTS`.
    ///
    /// # Errors
    /// Returns `Error::Other` wrapping the underlying `rusqlite::Error` if
    /// any statement in the DDL fails (typically a bad connection or a
    /// concurrent-schema-change race).
    pub fn load_schema(conn: &rusqlite::Connection) -> Result<()>;

Schema required (from 01-CONTEXT.md "Audit-log schema fixture"):

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
</interfaces>
</context>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| sim process to SQLite audit DB | Phase 2 writes rows here; must not be able to tamper with or delete past rows. |
| operator shell to audit DB file | An operator running `sqlite3 runtime/sim-audit.db` can issue arbitrary SQL; triggers still fire unless they drop the trigger first (escalation-requires-admin is acceptable v0.1 scope). |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-01-11 | Tampering | Compromised sim code (or a bug) issues `UPDATE audit_events SET path=...` to erase evidence | mitigate | `BEFORE UPDATE` trigger raises `SQLITE_CONSTRAINT_TRIGGER`; integration test `audit_update_is_rejected_by_trigger` proves it. |
| T-01-12 | Repudiation | Compromised sim code issues `DELETE FROM audit_events` to erase evidence | mitigate | `BEFORE DELETE` trigger raises; integration test `audit_delete_is_rejected_by_trigger` proves it. |
| T-01-13 | Information Disclosure | Audit log used as an exfiltration channel via raw body content (research A5) | accept | The schema defines `request_body TEXT` but Phase 2 is responsible for hashing/redacting before insert. This plan ships only the schema; enforcement at insert-time lives in Phase 2. Documented as a forward reference in the `audit.rs` doc. |
| T-01-14 | Elevation of Privilege | Operator with DB file access drops the triggers | accept | Filesystem-level access is outside v0.1 threat model per research doc; mitigation is DB file mode 0600 (Phase 2 scope). |
</threat_model>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: audit.sql fixture + SCHEMA_SQL constant + load_schema helper</name>
  <files>
    crates/reposix-core/Cargo.toml
    crates/reposix-core/src/lib.rs
    crates/reposix-core/src/audit.rs
    crates/reposix-core/fixtures/audit.sql
  </files>
  <behavior>
    - `SCHEMA_SQL` is a non-empty `&'static str` loaded via `include_str!("../fixtures/audit.sql")`.
    - `load_schema(&conn)` on a fresh `Connection::open_in_memory()` returns `Ok(())`.
    - A second call to `load_schema` on the same connection is a no-op (`IF NOT EXISTS` on every stmt).
    - After load, `conn.pragma_query_value(None, "table_info", ...)` (via a direct query on `sqlite_master`) shows all eight columns of `audit_events`.
    - `audit_events` is the only table created; `audit_no_update` and `audit_no_delete` are the only triggers.
  </behavior>
  <action>
    1. Edit `crates/reposix-core/Cargo.toml`:
       - Under `[dependencies]`, add `rusqlite = { workspace = true }`.
    2. Create `crates/reposix-core/fixtures/audit.sql` with the DDL from the `<interfaces>` block above. Keep it plain ASCII; one blank line between statements; no trailing semicolon on the last `END` (rusqlite's `execute_batch` tolerates either, but match SQLite conventions). The file MUST be valid on SQLite 3.31+ (Ubuntu 20.04 baseline per 01-CONTEXT.md specifics).
    3. Create `crates/reposix-core/src/audit.rs`:

           //! Append-only audit-log schema fixture.
           //!
           //! This module publishes the DDL Phase 2 loads at sim startup. The
           //! schema is the committed artifact — the SQLite triggers, not a
           //! runtime check, enforce SG-06 (audit log append-only).

           use crate::{Error, Result};

           /// Canonical DDL for the audit_events table.
           pub const SCHEMA_SQL: &str = include_str!("../fixtures/audit.sql");

           /// Load the schema into an open connection. Idempotent.
           ///
           /// # Errors
           /// Returns `Error::Other` wrapping the `rusqlite::Error` if the
           /// batch execute fails.
           pub fn load_schema(conn: &rusqlite::Connection) -> Result<()> {
               conn.execute_batch(SCHEMA_SQL)
                   .map_err(|e| Error::Other(format!("load_schema: {e}")))
           }

           #[cfg(test)]
           mod tests {
               use super::*;

               #[test]
               fn schema_sql_is_non_empty_and_contains_triggers() {
                   assert!(!SCHEMA_SQL.is_empty());
                   assert!(SCHEMA_SQL.contains("CREATE TRIGGER"));
                   assert!(SCHEMA_SQL.contains("audit_no_update"));
                   assert!(SCHEMA_SQL.contains("audit_no_delete"));
                   assert!(SCHEMA_SQL.contains("BEFORE UPDATE"));
                   assert!(SCHEMA_SQL.contains("BEFORE DELETE"));
               }

               #[test]
               fn load_schema_on_in_memory_db_succeeds() {
                   let conn = rusqlite::Connection::open_in_memory().unwrap();
                   load_schema(&conn).unwrap();
               }

               #[test]
               fn load_schema_is_idempotent() {
                   let conn = rusqlite::Connection::open_in_memory().unwrap();
                   load_schema(&conn).unwrap();
                   load_schema(&conn).unwrap(); // second call must not error
               }
           }

    4. Edit `crates/reposix-core/src/lib.rs`: add `pub mod audit;`.

    AVOID: parameterizing the fixture path (keep it a literal `include_str!` relative to `src/audit.rs`). AVOID adding any insert/query helpers here — this plan only owns the schema surface; Phase 2 owns the insert path. AVOID runtime-constructing the SQL with `format!` — the fixture IS the source of truth.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix &amp;&amp; cargo test -p reposix-core --lib audit::tests &amp;&amp; test -f crates/reposix-core/fixtures/audit.sql &amp;&amp; grep -q 'CREATE TRIGGER audit_no_update BEFORE UPDATE' crates/reposix-core/fixtures/audit.sql &amp;&amp; grep -q 'CREATE TRIGGER audit_no_delete BEFORE DELETE' crates/reposix-core/fixtures/audit.sql &amp;&amp; cargo clippy -p reposix-core --lib -- -D warnings</automated>
  </verify>
  <done>
    `audit.sql` is committed and contains both triggers; `SCHEMA_SQL` loads into an in-memory DB; unit tests and clippy pedantic are green.
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: examples/show_audit_schema.rs + integration test proving triggers fire</name>
  <files>
    crates/reposix-core/examples/show_audit_schema.rs
    crates/reposix-core/tests/audit_schema.rs
  </files>
  <behavior>
    - `cargo run -q -p reposix-core --example show_audit_schema` exits 0 and prints `SCHEMA_SQL` to stdout. Output contains `CREATE TRIGGER audit_no_update BEFORE UPDATE` and `CREATE TRIGGER audit_no_delete BEFORE DELETE` (ROADMAP SC #3).
    - Integration test `audit_schema_has_expected_columns` loads the schema into `Connection::open_in_memory()` and asserts the result of `SELECT name, type, "notnull" FROM pragma_table_info('audit_events')` matches the spec (8 rows, correct types, `ts`/`method`/`path` NOT NULL).
    - Integration test `audit_schema_lists_both_triggers` queries `SELECT name FROM pragma_trigger_list('audit_events')` (or `SELECT name FROM sqlite_master WHERE type='trigger' AND tbl_name='audit_events'`) and asserts both trigger names appear.
    - Integration test `audit_update_is_rejected_by_trigger` inserts one row, then runs an UPDATE; asserts the error surfaces and is a `SqliteFailure` whose message contains the trigger-raised string `audit_events is append-only`.
    - Integration test `audit_delete_is_rejected_by_trigger` same structure but for DELETE.
    - After a failed UPDATE/DELETE, the inserted row is still present (`SELECT COUNT(*) FROM audit_events` returns 1).
  </behavior>
  <action>
    1. Create `crates/reposix-core/examples/show_audit_schema.rs`:

           //! Prints the audit-log schema DDL to stdout.
           //!
           //! Used by ROADMAP phase-1 SC #3:
           //!   `cargo run -q -p reposix-core --example show_audit_schema`
           //! must emit DDL containing the `audit_no_update` / `audit_no_delete`
           //! triggers.

           fn main() {
               print!("{}", reposix_core::audit::SCHEMA_SQL);
           }

       Keep it trivial — no argv parsing, no flags. The point is a stable stdout.
    2. Create `crates/reposix-core/tests/audit_schema.rs`:

           //! Integration tests for the audit_events schema fixture.
           //! Covers ROADMAP phase-1 SC #3 assertions against a live SQLite DB.

           use reposix_core::audit::{load_schema, SCHEMA_SQL};
           use rusqlite::Connection;

           fn setup() -> Connection {
               let conn = Connection::open_in_memory().unwrap();
               load_schema(&conn).unwrap();
               conn
           }

           fn insert_sample_row(conn: &Connection) {
               conn.execute(
                   "INSERT INTO audit_events (ts, method, path) VALUES (?1, ?2, ?3)",
                   rusqlite::params!["2026-04-13T00:00:00Z", "GET", "/"],
               )
               .expect("insert must succeed on fresh schema");
           }

           #[test]
           fn audit_schema_has_expected_columns() { /* ... */ }

           #[test]
           fn audit_schema_lists_both_triggers() {
               let conn = setup();
               let mut stmt = conn
                   .prepare("SELECT name FROM sqlite_master \
                             WHERE type = 'trigger' AND tbl_name = 'audit_events' \
                             ORDER BY name")
                   .unwrap();
               let triggers: Vec<String> = stmt
                   .query_map([], |r| r.get::<_, String>(0))
                   .unwrap()
                   .map(std::result::Result::unwrap)
                   .collect();
               assert_eq!(
                   triggers,
                   vec!["audit_no_delete".to_string(), "audit_no_update".to_string()]
               );
           }

           #[test]
           fn audit_update_is_rejected_by_trigger() {
               let conn = setup();
               insert_sample_row(&conn);
               let err = conn
                   .execute("UPDATE audit_events SET path = 'x' WHERE id = 1", [])
                   .expect_err("UPDATE must be rejected");
               assert!(err.to_string().contains("append-only"),
                       "error must surface trigger message, got: {err}");
               let count: i64 = conn.query_row("SELECT COUNT(*) FROM audit_events", [], |r| r.get(0)).unwrap();
               assert_eq!(count, 1, "row survived the rejected UPDATE");
           }

           #[test]
           fn audit_delete_is_rejected_by_trigger() {
               let conn = setup();
               insert_sample_row(&conn);
               let err = conn
                   .execute("DELETE FROM audit_events WHERE id = 1", [])
                   .expect_err("DELETE must be rejected");
               assert!(err.to_string().contains("append-only"));
               let count: i64 = conn.query_row("SELECT COUNT(*) FROM audit_events", [], |r| r.get(0)).unwrap();
               assert_eq!(count, 1);
           }

           #[test]
           fn schema_sql_is_stable_bytes() {
               // Canary: if someone edits the fixture, this forces a diff review.
               // We don't pin the exact hash — keeping the fixture human-readable
               // is the point — but we assert bounds so an accidental truncation
               // fails CI.
               assert!(SCHEMA_SQL.len() > 200);
               assert!(SCHEMA_SQL.len() < 2000);
           }

       Fill in `audit_schema_has_expected_columns` by iterating `pragma_table_info('audit_events')` and matching against the expected column list `[("id","INTEGER"), ("ts","TEXT"), ("agent_id","TEXT"), ("method","TEXT"), ("path","TEXT"), ("status","INTEGER"), ("request_body","TEXT"), ("response_summary","TEXT")]`, plus NOT NULL constraints on `ts`, `method`, `path`.
    3. Add a doc-level assertion in the example comment pointing at ROADMAP SC #3 so `grep` finds it.

    AVOID: hitting the real filesystem (use `:memory:`). AVOID relying on rusqlite's `last_insert_rowid()` for the rejected-count assertion — re-query `COUNT(*)` as shown. AVOID asserting the trigger error's exact rusqlite `ErrorCode` — match on the message string, since the code varies across rusqlite minor versions.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix &amp;&amp; cargo run -q -p reposix-core --example show_audit_schema | tee /tmp/reposix-audit-schema.out &amp;&amp; grep -q 'CREATE TRIGGER audit_no_update BEFORE UPDATE' /tmp/reposix-audit-schema.out &amp;&amp; grep -q 'CREATE TRIGGER audit_no_delete BEFORE DELETE' /tmp/reposix-audit-schema.out &amp;&amp; cargo test -p reposix-core --test audit_schema &amp;&amp; cargo clippy -p reposix-core --all-targets -- -D warnings</automated>
  </verify>
  <done>
    `show_audit_schema` emits DDL containing both triggers; all five integration tests pass; both triggers fire on real SQLite; clippy pedantic is clean across the crate's targets.
  </done>
</task>

</tasks>

<verification>
Phase-level checks this plan contributes to:

1. ROADMAP SC #3 (full): `cargo run -q -p reposix-core --example show_audit_schema` emits DDL containing `CREATE TRIGGER audit_no_update BEFORE UPDATE` and `CREATE TRIGGER audit_no_delete BEFORE DELETE`.
2. ROADMAP SC #5 (partial, full for this crate's targets once all plans merge): `cargo clippy -p reposix-core --all-targets -- -D warnings` is clean for the new modules, the example, and the integration test.
3. PROJECT.md SG-06 (audit log append-only): mechanically enforced by the triggers; proven by two integration tests that assert both UPDATE and DELETE fail on real SQLite, not just in prose.
4. PROJECT.md FC-06 (audit log SQLite, queryable): schema published; Phase 2 consumes `SCHEMA_SQL` at sim startup.
</verification>

<success_criteria>
**Goal-backward verification** — if the orchestrator runs:

    cd /home/reuben/workspace/reposix && \
      cargo run -q -p reposix-core --example show_audit_schema > /tmp/schema.out && \
      grep -q 'CREATE TRIGGER audit_no_update BEFORE UPDATE' /tmp/schema.out && \
      grep -q 'CREATE TRIGGER audit_no_delete BEFORE DELETE' /tmp/schema.out && \
      cargo test -p reposix-core --test audit_schema && \
      cargo clippy -p reposix-core --all-targets -- -D warnings

…then phase-1 success-criterion **#3 (full)** passes and **#5 (partial)** passes for this plan's contributions. Combined with plans 01-01 and 01-02, all five phase-1 success-criteria are satisfied.
</success_criteria>

<output>
After completion, create `.planning/phases/01-core-contracts-security-guardrails/01-03-SUMMARY.md` per the summary template. Must include: the exact byte-size of `audit.sql`, the list of triggers (`pragma_trigger_list` output), the error-message fragment the tests match on (`append-only`), and a forward reference noting Phase 2 is responsible for the insert path + body-redaction (T-01-13 disposition: `accept` at schema layer, `mitigate` at insert layer).
</output>
