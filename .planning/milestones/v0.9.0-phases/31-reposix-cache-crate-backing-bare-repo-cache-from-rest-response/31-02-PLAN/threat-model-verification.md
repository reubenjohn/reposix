← [back to index](./index.md)

# Threat Model, Verification, Success Criteria, Output

## Threat Model

### Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Backend → Cache (blob materialization) | `BackendConnector::get_issue` returns tainted `Issue`; cache renders via `frontmatter::render` and writes bytes as a git blob. The bytes surface to callers wrapped in `Tainted<Vec<u8>>`. |
| Allowlist gate → Cache audit | `reposix_core::Error::InvalidOrigin` is the trust boundary between "egress denied" (must be audited) and "other backend failure" (ordinary error path). Misclassification means a silently-denied egress looks identical to a network timeout. |
| SQLite schema → attacker with handle | `cache.db` holds the append-only audit table. An attacker with the same DB handle could attempt `DROP TRIGGER`, `PRAGMA writable_schema`, or direct `sqlite_master` edits. |
| Filesystem → other local users | `cache.db` contains audit history and oid-to-issue-id mappings. World-readable would leak the agent's browsing pattern. |

### STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-31-02-01 | Information Disclosure | `cache.db` readable by other local users reveals audit history + which issues the agent materialized. | mitigate | File created with `mode(0o600)` in `open_cache_db` (lifted pattern from `reposix-cli/src/cache_db.rs:71`). Integration test `audit_is_append_only.rs` implicitly relies on the 0o600 file open succeeding for the current process. |
| T-31-02-02 | Tampering | In-process attacker attempts `DROP TRIGGER audit_cache_no_update`, `DROP TABLE audit_events_cache`, or `PRAGMA writable_schema=ON` to disable append-only enforcement. | mitigate | `SQLITE_DBCONFIG_DEFENSIVE` enabled via `reposix_core::audit::enable_defensive(&conn)` BEFORE any schema statement. Blocks `writable_schema` edits and `sqlite_master` row-level mutations. Paired with trigger definitions using `DROP TRIGGER IF EXISTS + CREATE TRIGGER` in a single transaction (copied from `reposix_core/fixtures/audit.sql` pattern). |
| T-31-02-03 | Tampering | Row-level attacker runs `UPDATE audit_events_cache SET ts='...'` or `DELETE FROM audit_events_cache` to rewrite history. | mitigate | BEFORE UPDATE and BEFORE DELETE triggers `RAISE(ABORT, 'audit_events_cache is append-only')` — any such statement returns `SQLITE_CONSTRAINT` with that message. Integration test `audit_is_append_only.rs` asserts both paths. |
| T-31-02-04 | Information Disclosure | Allowlist-denied egress fails silently — operators see a generic "backend error" and miss the audit signal. | mitigate | `read_blob` explicitly detects `reposix_core::Error::InvalidOrigin` (and substring-matches `"invalid origin"` / `"allowlist"` as a fallback for wrapping backends), fires `log_egress_denied` BEFORE returning, and returns `Error::Egress(_)` — a distinct variant separate from `Error::Backend`. Integration test `egress_denied_logs.rs` asserts both the typed error and the audit row. |
| T-31-02-05 | Tampering | `Cache::read_blob` writes a blob, backend's second call returns different bytes (eventual consistency), tree now references an OID the cache cannot reproduce. | mitigate | `gix::Repository::write_blob` returns the actual OID it computed; `read_blob` asserts equality with the requested OID and returns `Error::OidDrift { requested, actual, issue_id }` on mismatch. Operational signal: persistent `OidDrift` firing is a backend-race bug — covered in RESEARCH §Pitfall 1. |
| T-31-02-06 | Denial of Service | Audit-row INSERT fails (disk full, SQLite busy). `read_blob` must not poison the user flow. | accept / mitigate | Best-effort audit pattern: `log_materialize` / `log_egress_denied` / `log_tree_sync` all return `()` and emit `tracing::warn!` with target `reposix_cache::audit_failure` on failure. Consistent with CONTEXT.md "audit failure must not poison the user flow but should be visible." Operators treat persistent WARN as P1. |
| T-31-02-07 | Spoofing | An attacker who can place a pre-baked `cache.db` at `$XDG_CACHE_HOME/reposix/sim-proj-1.git/` before `Cache::open` runs could seed arbitrary audit history. | accept | The attacker already has write access to the user's cache dir; they can equally seed `~/.bash_history`. OS-level file permissions (mode 0700 on parent dir is NOT enforced by Plan 02; the default umask is the only defence — hardening deferred to Phase 35 CLI setup which controls the init path). Low-value target: the cache is a mirror of a public-to-the-user REST API. |
| T-31-02-08 | Elevation of Privilege | The `cli_compat` lift keeps the EXCLUSIVE lock contract. If Phase 35 CLI refactor forgets to re-acquire the EXCLUSIVE lock in the refresh subcommand, two concurrent `reposix refresh` invocations could race. | mitigate | Lift is verbatim — the `map_busy` + `PRAGMA locking_mode = EXCLUSIVE` pattern is preserved 1:1. The four pre-existing tests (`open_creates_schema`, `update_metadata_roundtrip`, `lock_conflict_returns_error`, `open_is_idempotent`) MUST still pass after the lift — acceptance criterion enforced. |

## Verification

Phase 31 Wave 2 verification (depends on Wave 1 being green):

1. `cargo check --workspace` — exit 0.
2. `cargo clippy --workspace --all-targets -- -D warnings` — exit 0.
3. `cargo test -p reposix-cache --test audit_is_append_only` — exit 0.
4. `cargo test -p reposix-cache --test materialize_one` — exit 0.
5. `cargo test -p reposix-cache --test egress_denied_logs` — exit 0.
6. `cargo test --workspace` — exit 0 (no regression in CLI or other crates).
7. `grep -rnE "reqwest::(Client::new|Client::builder|ClientBuilder::new)" crates/reposix-cache/src/` returns empty.
8. `! test -f crates/reposix-cli/src/cache_db.rs && test -f crates/reposix-cache/src/cli_compat.rs` (lift completed).

No manual verification — every ARCH-02 / ARCH-03 behavior is covered by automated tests.

## Success Criteria

- [ ] `audit_events_cache` exists and accepts INSERT but rejects UPDATE and DELETE with `SQLITE_CONSTRAINT` + message `"audit_events_cache is append-only"`.
- [ ] `meta` and `oid_map` tables exist and are writable (normal CRUD).
- [ ] `cache.db` file is created with mode 0o600 and has DEFENSIVE flag enabled.
- [ ] `Cache::build_from` records one `tree_sync` audit row + N `oid_map` rows + upserts `last_fetched_at` in `meta`.
- [ ] `Cache::read_blob(oid)` materializes exactly one blob per call, writes one `materialize` audit row per call, and returns `Tainted<Vec<u8>>`.
- [ ] Pointing the cache at a backend whose `get_issue` returns `reposix_core::Error::InvalidOrigin` causes `read_blob` to return `Error::Egress(_)` AND write exactly one `op='egress_denied'` audit row BEFORE returning.
- [ ] `Error::OidDrift` variant exists and is wired to the `write_blob`-returned-OID-mismatch path.
- [ ] Zero `reqwest::Client` constructors in `crates/reposix-cache/src/`.
- [ ] `cache_db.rs` lifted from `reposix-cli` into `reposix-cache::cli_compat`; four pre-existing CLI tests pass from the new home; `reposix-cli` continues to build and test.
- [ ] `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings` both green.

## Output

After completion, create `.planning/phases/31-reposix-cache-crate-backing-bare-repo-cache-from-rest-response/31-02-SUMMARY.md`. Include:
- Commit SHAs for each of the three task commits.
- The exact shape of the `InvalidOrigin` detection in `read_blob` (whether it uses `matches!` on the enum variant, `.to_string().contains(...)`, or both) — Plan 03 consumers need to know.
- Confirmation that the three new integration tests pass, with sample audit-row counts captured in the SUMMARY (e.g. "after materialize_one, audit_events_cache had 1 tree_sync + 1 materialize row = 2 rows total; egress_denied_logs had 1 tree_sync + 1 egress_denied = 2 rows").
- Note whether `reposix-cli`'s `refresh.rs` needed import-path updates or whether the `pub use reposix_cache::cli_compat as cache_db;` shim was sufficient.
- Any WARN-emitting code paths added to `tracing` (target, fields) so operators know what to grep for.
