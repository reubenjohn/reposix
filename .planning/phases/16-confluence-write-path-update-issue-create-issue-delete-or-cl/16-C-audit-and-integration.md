# Plan: Wave C — Audit log on `ConfluenceBackend` + read-path ADF switch + round-trip integration test

## Goal

Wire the Phase 1 audit log schema (SG-06) into `ConfluenceBackend` so that every `create_issue` / `update_issue` / `delete_or_close` call inserts a single row into a client-side SQLite connection, and flip the read path to request `?body-format=atlas_doc_format` with an ADF→Markdown conversion (+ storage fallback on empty). Prove WRITE-04 end-to-end with one wiremock round-trip integration test (create → read → body matches). Still no docs / no version bump — those live in Wave D.

## Wave

C (depends on: Wave B merged — write methods live; unblocks: Wave D).

## Addresses

- **Locked decision LD-16-03** — every write call gets an audit log row. This wave is the ONLY place this decision is satisfied; the write methods from Wave B currently have no audit wiring.
- **Requirement WRITE-04** — ADF ↔ Markdown round-trip. Wave A built the converter; Wave C flips the read path to use it and proves round-trip via integration test.

## Tasks

### C1. Add `rusqlite` + `sha2` dependencies to `reposix-confluence`

- **File:** `crates/reposix-confluence/Cargo.toml`
- **Action:** Under `[dependencies]` add:
  - `rusqlite = { workspace = true }` (already workspace-pinned at `0.32` with `bundled`).
  - `sha2 = "0.10"` (for the audit row's `response_summary` sha256-prefix). If sha2 isn't a workspace dep yet, add it to `[workspace.dependencies]` in root Cargo.toml first and reference as `.workspace = true`.
  - Keep `parking_lot = { workspace = true }` (already present) — will hold the audit `Mutex<Connection>`.
- **Expected line impact:** 2–3 lines.
- **Verification:** `cargo check -p reposix-confluence`.

### C2. Add optional `audit: Option<Arc<Mutex<Connection>>>` field + `with_audit` builder on `ConfluenceBackend`

- **File:** `crates/reposix-confluence/src/lib.rs`
- **Action:**
  - Add field `audit: Option<Arc<parking_lot::Mutex<rusqlite::Connection>>>` on the struct (place after `rate_limit_gate`).
  - Initialize to `None` in `new_with_base_url`.
  - Add a public builder method:
    ```rust
    /// Attach an audit log connection. Every write call (POST/PUT/DELETE)
    /// inserts one row into `audit_events` when an audit connection is
    /// present; writes succeed even if the audit insert fails (best-effort,
    /// log-and-swallow — the Confluence round-trip has already committed).
    ///
    /// The caller is responsible for opening the connection via
    /// [`reposix_core::audit::open_audit_db`] so the schema and triggers
    /// are loaded before the first insert.
    #[must_use]
    pub fn with_audit(mut self, conn: Arc<parking_lot::Mutex<rusqlite::Connection>>) -> Self {
        self.audit = Some(conn);
        self
    }
    ```
  - Extend the manual `Debug` impl to print `audit: "<present>" | "<none>"` (NOT the connection pointer).
- **Expected line impact:** ~25 lines.
- **Verification:** `cargo check -p reposix-confluence`; `cargo clippy -p reposix-confluence -- -D warnings`.

### C3. Add a private `audit_write` helper + wire it into all three write methods

- **File:** `crates/reposix-confluence/src/lib.rs`
- **Action:**
  - Implement a private helper on `ConfluenceBackend`:
    ```rust
    /// Insert one audit row for a completed write call. Best-effort —
    /// any failure is logged via `tracing::error!` and swallowed. Sync-
    /// scoped (no `.await` held across the lock).
    fn audit_write(
        &self,
        method: &'static str,       // "POST" | "PUT" | "DELETE"
        path: &str,                 // e.g. "/wiki/api/v2/pages/12345"
        status: u16,
        request_summary: &str,      // truncated title (first 256 chars)
        response_bytes: &[u8],
    ) {
        let Some(ref audit) = self.audit else { return };
        let ts = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let sha_hex = {
            use sha2::{Digest, Sha256};
            let digest = Sha256::digest(response_bytes);
            format!("{digest:x}")
        };
        let response_summary = format!("{status}:{}", &sha_hex[..16]);
        let conn = audit.lock();
        if let Err(e) = conn.execute(
            "INSERT INTO audit_events \
             (ts, agent_id, method, path, status, request_body, response_summary) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                ts,
                format!("reposix-confluence-{}", std::process::id()),
                method,
                path,
                i64::from(status),
                request_summary,
                response_summary,
            ],
        ) {
            tracing::error!(error = %e, "confluence audit insert failed");
        }
    }
    ```
  - In `create_issue`, `update_issue`, `delete_or_close` (touched in Wave B): after the response is received and `status` is captured (both success AND failure paths), call `self.audit_write(...)`. Log on failure as well — the audit log records the attempt, not just the success. A failed write still gets a row with the real status code.
  - Pick a stable `path` format: always `/wiki/api/v2/pages/{id}` for update/delete, `/wiki/api/v2/pages` for create. Match the HTTP URL path-component (no query string).
  - `request_summary` is `issue.title` truncated to 256 chars (for POST/PUT) or empty string for DELETE. **Never the body** — RESEARCH §Audit log pattern for Confluence explicitly forbids full page content.
- **Expected line impact:** ~40 lines for the helper + ~10 lines of wiring across 3 methods = ~50 lines.
- **Verification:** `cargo check -p reposix-confluence`; `cargo clippy -p reposix-confluence --all-targets -- -D warnings`.

### C4. Switch read path to `?body-format=atlas_doc_format` with storage fallback

- **File:** `crates/reposix-confluence/src/lib.rs` (the `get_issue` method around L669; also `list_issues` body parsing if it requests body)
- **Action:**
  - Change the GET query from `?body-format=storage` to `?body-format=atlas_doc_format` in `get_issue`.
  - Update `ConfPage` / `ConfBody` deserialization: the ADF body is a JSON object, not a string. Add a new `atlas_doc_format: Option<AdfBody>` field (or parse into `serde_json::Value` and pass to `adf::adf_to_markdown`).
  - In `translate`: if ADF body is present → call `adf::adf_to_markdown(&adf_value)` and store the result in `issue.body`. If ADF body is empty or parsing fails → fall back to re-fetching with `?body-format=storage` (second round-trip; acceptable for pre-ADF pages per RESEARCH Risk 6). Implementation: it's cleaner to keep `get_issue` in charge of the fallback — call `get_issue_with_format(id, "atlas_doc_format")` first, if the parsed body is empty/null, call `get_issue_with_format(id, "storage")`.
  - Update `list_issues`: if it currently includes body fetching, apply the same shape. (Grep to confirm — if `list_issues` doesn't request body, skip.)
  - Update the mapping table in the crate-level `//!` doc: `body` source is now `adf_to_markdown(page.body.atlas_doc_format)` with storage fallback.
- **Expected line impact:** ~50 lines (new struct variants, fallback logic, doc update).
- **Verification:** Existing `get_issue_*` wiremock tests in `crates/reposix-confluence/src/lib.rs` + `crates/reposix-confluence/tests/contract.rs` will break — they mock `?body-format=storage` responses. Update mocks to serve ADF JSON instead; add one new test `get_issue_falls_back_to_storage_when_adf_empty` that mocks the empty-ADF → storage retry chain. All existing read-path tests must stay green.

### C5. Audit log unit tests with in-memory SQLite

- **File:** `crates/reposix-confluence/src/lib.rs` (tests mod)
- **Action:** Five tests from RESEARCH.md §Audit Log Tests, same fn names for traceability:

  | Test fn name | Assertion |
  |---|---|
  | `update_issue_writes_audit_row` | After a wiremock PUT succeeds, `SELECT COUNT(*) FROM audit_events WHERE method = 'PUT'` = 1 |
  | `create_issue_writes_audit_row` | After POST succeeds, one row with `method = 'POST'`, `path = '/wiki/api/v2/pages'` |
  | `delete_or_close_writes_audit_row` | After DELETE → 204, one row with `method = 'DELETE'`, status = 204 |
  | `audit_row_has_correct_method_and_path` | For the PUT case, `path = '/wiki/api/v2/pages/99'`, `status = 200`, `agent_id` starts with `"reposix-confluence-"`, `response_summary` matches `^\d+:[0-9a-f]{16}$` |
  | `audit_insert_failure_does_not_mask_write_result` | Construct the backend with an audit connection that has the `audit_events` table DROPPED — the PUT call still returns `Ok`, and `tracing::error!` is emitted (verify with a `tracing` subscriber capture if already used in this crate; otherwise assert via observable effect — the write result). Simpler alternative: pass a closed connection and assert write still succeeds. |
  | `audit_records_failed_writes` | (new, not in research) — PUT → 409; assert one audit row exists with `status = 409` |

  **Test helpers:** open an in-memory SQLite with `rusqlite::Connection::open_in_memory()`, then call `reposix_core::audit::load_schema(&conn)` to load the table + triggers. Wrap in `Arc::new(parking_lot::Mutex::new(conn))` and pass via `.with_audit(...)`.

- **Expected line impact:** ~200 lines (6 tests including helper fixture fn).
- **Verification:** `cargo test -p reposix-confluence audit` — 6 tests green.

### C6. Wiremock round-trip integration test (WRITE-04 end-to-end)

- **File:** `crates/reposix-confluence/tests/roundtrip.rs` (new integration test)
- **Action:** One `#[tokio::test]` that:
  1. Starts a `wiremock::MockServer`.
  2. Stubs `POST /wiki/api/v2/pages` to accept any body, capture it, and respond with a page-JSON containing `id: "777"` and echoing back the submitted `body.storage.value` as the returned ADF (hand-crafted mock: parse storage, rebuild as simple ADF headings/paragraphs, OR just return a pre-computed ADF value that corresponds to a known input).
  3. Stubs `GET /wiki/api/v2/pages/777?body-format=atlas_doc_format` to return the ADF built above.
  4. Builds a `ConfluenceBackend` pointing at the mock server, with an in-memory audit connection attached.
  5. Calls `create_issue("REPOSIX", Untainted::new(issue_with_markdown_body))`.
  6. Calls `get_issue("REPOSIX", IssueId(777))`.
  7. Asserts the returned body, converted ADF→Markdown, contains the headings/paragraphs from the original Markdown input (`# Title`, `hello world`, fenced code block).
  8. Asserts `audit_events` has exactly 1 row with `method = 'POST'`.
- **Why integration not inline:** Exercises the full converter + write + read + audit stack in a single test, which is the closest we get to WRITE-04's acceptance criterion without a real tenant. Keeps the crate-level `lib.rs` tests focused on per-method assertions.
- **Expected line impact:** ~120 lines.
- **Verification:** `cargo test -p reposix-confluence --test roundtrip` green.

## Verification

Before merging Wave C:

```bash
cargo test -p reposix-confluence                                     # in-tree tests (A + B + C)
cargo test -p reposix-confluence --test roundtrip                    # integration
cargo test -p reposix-confluence --test contract                     # read-path regression
cargo test --workspace                                               # no regressions
cargo clippy --workspace --all-targets -- -D warnings                # clean
cargo fmt --all --check                                              # formatted
```

Test count expected after Wave C ≈ 307 (post-B) + 6 (C5) + 1 (C6) + 1 (C4 fallback) = **315+**. Must NOT decrease from baseline 278.

## Threat model

| Threat ID | STRIDE | Component | Disposition | Mitigation |
|---|---|---|---|---|
| T-16-C-01 | Repudiation | Agent writes a page; no record of the action | Mitigate | Every write call gets an audit row. Enforced by `{create,update,delete_or_close}_writes_audit_row` tests. `audit_records_failed_writes` guarantees failed writes are also recorded (important for post-hoc attack analysis). |
| T-16-C-02 | Tampering | Audit row gets edited after the fact to erase evidence | Mitigate | `load_schema` loads the append-only triggers (`audit_no_update`, `audit_no_delete`) from the Phase 1 fixture. SG-06 enforcement comes from the triggers, not the backend code — we depend on `reposix_core::audit::load_schema` being called. Documented in the `with_audit` rustdoc: "caller MUST use `open_audit_db` which loads the triggers." |
| T-16-C-03 | DoS | Audit connection lock contention blocks the write path | Mitigate | `parking_lot::Mutex` — fast, no poisoning. Lock held only for the sync `INSERT` — no `.await` across the critical section (RESEARCH Risk 4). Write call doesn't block waiting for audit; even a `std::sync::Mutex` would be fine at the expected write rate. |
| T-16-C-04 | Information Disclosure | Audit row leaks page body | Mitigate | `request_body` column stores title only (truncated to 256 chars). `response_summary` stores status + sha256 hex prefix — no body bytes. Locked by `audit_row_has_correct_method_and_path` asserting the `response_summary` shape. |
| T-16-C-05 | Tampering | ADF body from remote contains a malicious node type that triggers panic in `adf_to_markdown` | Mitigate | Already covered in Wave A (T-16-A-01 + T-16-A-03 — fallback marker, depth cap). Wave C just flips the switch; the defenses are in place. |
| T-16-C-06 | Denial of Service | Audit DB disk fills | Accept | Best-effort insert, log-and-swallow on failure. Operational concern, not a security bug — write path keeps working even when audit DB is full. Monitor via `tracing::error!` output in production. |

## Success criteria

1. `ConfluenceBackend::with_audit(…)` exists and attaches a `rusqlite` connection; `grep -n 'with_audit' crates/reposix-confluence/src/lib.rs` returns ≥2 hits (definition + doc example).
2. All three write methods call `self.audit_write(…)` on every code path (success AND error). `rg 'audit_write\(' crates/reposix-confluence/src/lib.rs` returns ≥3 hits.
3. `get_issue` requests `?body-format=atlas_doc_format` by default and falls back to `storage` on empty ADF. `rg 'body-format=atlas_doc_format' crates/reposix-confluence/src/lib.rs` returns ≥1 hit.
4. `cargo test -p reposix-confluence --test roundtrip` passes — full create → read → compare roundtrip.
5. Workspace test count ≥ 315.
6. `cargo clippy --workspace --all-targets -- -D warnings` clean.
7. No existing read-path test regressed (contract.rs + inline get_issue tests all green after mock updates).
