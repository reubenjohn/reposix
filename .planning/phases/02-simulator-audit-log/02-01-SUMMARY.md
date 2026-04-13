---
phase: 02-simulator-audit-log
plan: 01
subsystem: simulator
status: complete
tasks: 2/2
commits:
  - 3c004f6  # feat(02-01): sim storage layer — AppState, db.rs, seed.json, ApiError
  - d29e47c  # feat(02-01): issue CRUD routes + transitions wired through AppState
requirements:
  - FC-01
  - FC-06
roadmap_success_criteria:
  SC1: PASS  # GET /projects/demo/issues length >= 3
  SC2: PASS  # GET /projects/demo/issues/1 returns 200 + id + version
  SC3: PASS  # PATCH If-Match bogus returns 409
  SC4: DEFERRED  # audit counts/trigger — plan 02-02
  SC5: DEFERRED  # integration test — plan 02-02
tests:
  unit: 20
  integration: 0
---

# Phase 2 Plan 01: axum handlers + storage Summary

One-liner: `reposix-sim` now serves the full issue-tracker CRUD shape
(list/get/create/patch/delete + Jira-style transitions) against a WAL-mode
SQLite store, with optimistic concurrency via `If-Match` and a deterministic
seed loader.

## Shipped

### File list

- `crates/reposix-sim/src/state.rs` — `AppState { db: Arc<Mutex<Connection>>, config: Arc<SimConfig> }`.
  Clone via Arc clones; lock is always sync (never held across `.await`).
- `crates/reposix-sim/src/db.rs` — `open_db(path, ephemeral)`; enables WAL +
  `synchronous=NORMAL` + `busy_timeout=5000` on file-backed DBs; creates
  `issues` table; calls `reposix_core::audit::load_schema` to install append-only
  triggers.
- `crates/reposix-sim/src/error.rs` — `ApiError` enum + `IntoResponse`. Variants:
  `NotFound` (404), `BadRequest(String)` (400), `VersionMismatch{current,sent}`
  (409), `Db(rusqlite::Error)` / `Json(serde_json::Error)` / `Internal(String)`
  (all 500 opaque, full detail logged via `tracing::error!`).
- `crates/reposix-sim/src/seed.rs` — `load_seed(&conn, path)` /
  `apply_seed(&conn, &SeedFile)`; `INSERT OR IGNORE` so it's idempotent;
  fixed `2026-04-13T00:00:00Z` timestamps; version=1.
- `crates/reposix-sim/fixtures/seed.json` — 3-issue `demo` project; adversarial
  bodies intact (`<script>alert(1)</script>` in issue 1; literal `version: 999`
  line in issue 3).
- `crates/reposix-sim/src/routes/mod.rs` — merges issues + transitions sub-routers.
- `crates/reposix-sim/src/routes/issues.rs` — GET list/get, POST create,
  PATCH update, DELETE. PATCH uses `TransactionBehavior::Immediate` so the
  version-check + UPDATE are atomic. `FieldUpdate<T>` three-state enum
  replaces `Option<Option<T>>` for nullable-assignee semantics
  (Unchanged/Clear/Set).
- `crates/reposix-sim/src/routes/transitions.rs` — `GET .../:id/transitions`.
- `crates/reposix-sim/src/lib.rs` — `SimConfig` extended with `seed_file`,
  `ephemeral`, `rate_limit_rps`; `build_router(state)` now merges the full
  routes; `run(cfg)` opens DB + seeds + serves.
- `crates/reposix-sim/src/main.rs` — clap args `--bind`, `--db`, `--seed-file`,
  `--no-seed`, `--ephemeral`, `--rate-limit` (rate-limit parsed but not yet
  wired — plan 02-02 consumes it).
- `scripts/phase2_smoke.sh` — replay-able Bash harness for ROADMAP SC1/SC2/SC3.

### AppState shape (for plan 02-02 to consume)

```rust
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<parking_lot::Mutex<rusqlite::Connection>>,
    pub config: Arc<SimConfig>,
}

impl AppState {
    pub fn new(conn: Connection, config: SimConfig) -> Self { /* Arc wraps */ }
}
```

### ApiError JSON shape

- `ApiError::NotFound` → 404 `{"error":"not_found","message":"not found"}`
- `ApiError::BadRequest(msg)` → 400 `{"error":"bad_request","message":<msg>}`
- `ApiError::VersionMismatch{current,sent}` → 409
  `{"error":"version_mismatch","current":<u64>,"sent":<string>}`
- `ApiError::{Db,Json,Internal}` → 500 opaque
  `{"error":"internal","message":"internal error"}` (detail logged server-side)

### Issues table DDL

```sql
CREATE TABLE IF NOT EXISTS issues (
    project    TEXT    NOT NULL,
    id         INTEGER NOT NULL,
    title      TEXT    NOT NULL,
    status     TEXT    NOT NULL,
    assignee   TEXT,
    labels     TEXT    NOT NULL DEFAULT '[]',
    created_at TEXT    NOT NULL,
    updated_at TEXT    NOT NULL,
    version    INTEGER NOT NULL DEFAULT 1,
    body       TEXT    NOT NULL DEFAULT '',
    PRIMARY KEY(project, id)
);
```

### If-Match parsing rule

1. Read header `If-Match`.
2. Absent → treat as wildcard (allow; increment version regardless).
3. Present → strip surrounding `"`, call `u64::from_str(trimmed)`.
4. Parse failure → mismatch, return 409 with `sent = <raw_unquoted>`.
5. Parse success but not equal to current DB version → mismatch, 409.
6. Parse success and equal → succeed; bump version, update `updated_at`.

### Mutable field allow-list on PATCH

`PatchIssueBody { title?, body?, status?, assignee?, labels? }` with
`#[serde(deny_unknown_fields)]`. Any attempt to send `id`, `version`,
`created_at`, or `updated_at` fails deserialization. `assignee` uses the
three-valued `FieldUpdate<String>` enum for null-means-clear semantics.

## Tests

- `cargo test -p reposix-sim --lib`: 20 green (4 error, 3 db, 5 seed, 8
  routes).
- `cargo clippy -p reposix-sim --all-targets -- -D warnings`: clean.
- `scripts/phase2_smoke.sh`: SC1/SC2/SC3 all PASS on fresh-seeded live
  binary.

## Deviations from plan

- **Rule 2 addition — adversarial unit test.** Plan did not require a
  "patch rejects server-managed fields" route test; I added
  `patch_ignores_server_managed_fields_via_deny_unknown` because the
  `deny_unknown_fields` deserializer attribute is a correctness
  requirement (T-02-01 / SG-03 boundary) and an untested boundary is not
  really a boundary.
- **Rule 3 addition — `scripts/phase2_smoke.sh`.** The plan's
  `<verify>` blocks contain large inline shell snippets that would trip the
  ad-hoc-bash hook when executed by agents. Promoted to a committed
  script per CLAUDE.md §4 so the next agent runs one named command.
- **FieldUpdate<T> instead of Option<Option<T>>.** `clippy::option_option`
  is denied workspace-wide. Small three-state enum preserves the
  "absent vs null vs value" semantics without the lint hit.

## Known stubs

None. All shipped paths are wired end-to-end. The only deliberate TODO is
the Phase-3 `Tainted<T>` wrapping comment on the create/patch handlers,
which is per-plan design (Phase 2 does not introduce the tainted type,
Phase 3 does).

## Threat flags

None — the surface matches what the plan's `<threat_model>` already
covered (T-02-01 through T-02-06). Audit middleware (plan 02-02) will
introduce the remaining trust boundaries.

## Self-Check: PASSED

- Files exist: state.rs, db.rs, error.rs, seed.rs, seed.json, routes/*.rs,
  scripts/phase2_smoke.sh — all present.
- Commits exist: `git log --oneline --all | grep -E '3c004f6|d29e47c'` → 2
  matches.
- Tests: 20 unit tests pass (was 12 after task 1, +8 route tests).
- Live smoke: SC1=3 issues, SC2=200+id+version, SC3=409+version_mismatch.
