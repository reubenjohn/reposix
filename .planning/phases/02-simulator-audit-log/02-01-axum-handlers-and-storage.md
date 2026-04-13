---
phase: 02-simulator-audit-log
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/reposix-sim/Cargo.toml
  - crates/reposix-sim/src/lib.rs
  - crates/reposix-sim/src/main.rs
  - crates/reposix-sim/src/state.rs
  - crates/reposix-sim/src/db.rs
  - crates/reposix-sim/src/seed.rs
  - crates/reposix-sim/src/error.rs
  - crates/reposix-sim/src/routes/mod.rs
  - crates/reposix-sim/src/routes/issues.rs
  - crates/reposix-sim/src/routes/transitions.rs
  - crates/reposix-sim/fixtures/seed.json
autonomous: true
requirements: [FC-01, FC-06]
est_minutes: 90
risks:
  - "rusqlite + parking_lot::Mutex held across .await points (must scope locks tightly; no async in critical section)."
  - "axum 0.7 Path extractor requires tuple for multi-param; wrong arity yields compile errors that can mimic routing bugs."
  - "Seed JSON schema drift vs. reposix_core::Issue Serialize output — seed must round-trip through Issue::deserialize."
  - "rusqlite bundled build inflates compile time; confirm cold-build still finishes in the allotted window."

must_haves:
  truths:
    - "A running sim binds 127.0.0.1:<port> with /healthz reachable."
    - "GET /projects/demo/issues returns a JSON array with length >= 3 after --seed-file."
    - "GET /projects/demo/issues/1 returns 200 with frontmatter.id=1 and a non-null version integer."
    - "POST /projects/demo/issues creates a new issue, returns 201 with Location header."
    - "PATCH /projects/demo/issues/1 with correct If-Match bumps version and returns 200."
    - "PATCH /projects/demo/issues/1 with If-Match: \"bogus\" returns 409 with body {error, current, sent}."
    - "DELETE /projects/demo/issues/1 returns 204; subsequent GET returns 404."
    - "SQLite DB at --db path exists in WAL mode with issues table populated from seed."
    - "audit_events table exists (loaded via reposix_core::audit::load_schema) but is not yet written to (that is plan 02-02)."
  artifacts:
    - path: "crates/reposix-sim/src/state.rs"
      provides: "AppState { db: Arc<parking_lot::Mutex<rusqlite::Connection>>, config: Arc<SimConfig> }"
    - path: "crates/reposix-sim/src/db.rs"
      provides: "issues table DDL, open_db(path, ephemeral) -> Connection with WAL + audit schema loaded"
      contains: "PRAGMA journal_mode=WAL"
    - path: "crates/reposix-sim/src/seed.rs"
      provides: "load_seed(conn, path) populates project + issues deterministically"
    - path: "crates/reposix-sim/src/routes/issues.rs"
      provides: "list/get/create/patch/delete handlers"
    - path: "crates/reposix-sim/src/routes/transitions.rs"
      provides: "GET .../transitions returning legal next states (best-effort v0.1)"
    - path: "crates/reposix-sim/src/error.rs"
      provides: "ApiError enum impl IntoResponse returning {error, message, details}"
    - path: "crates/reposix-sim/fixtures/seed.json"
      provides: "demo project + 3 seed issues (script-tag body, fake version:999 body, plain body)"
      contains: "\"slug\": \"demo\""
  key_links:
    - from: "crates/reposix-sim/src/lib.rs::build_router"
      to: "crates/reposix-sim/src/routes/issues.rs"
      via: "Router::nest(\"/projects/:slug/issues\", issues::router(state))"
      pattern: "routes::issues"
    - from: "crates/reposix-sim/src/db.rs::open_db"
      to: "reposix_core::audit::load_schema"
      via: "direct call on the opened Connection"
      pattern: "audit::load_schema"
    - from: "crates/reposix-sim/src/main.rs"
      to: "crates/reposix-sim/src/lib.rs::run"
      via: "tokio::main parses clap args, calls run(SimConfig)"
      pattern: "run\\(SimConfig"
---

<objective>
Turn the `reposix-sim` skeleton into a real REST issue tracker: CRUD handlers over
SQLite (WAL mode, `parking_lot::Mutex<Connection>`), deterministic seed loader, and a
CLI (`--bind`, `--db`, `--seed-file`, `--no-seed`, `--ephemeral`, `--rate-limit`).
This plan lands everything the integration test will exercise *except* the audit
middleware and rate-limit layer — those are plan 02-02's job, wired on top of the
router this plan returns.

Purpose: make the data plane real so ROADMAP Phase-2 success criteria 1-3 (list
returns >=3, GET returns id+version, PATCH with bogus If-Match returns 409) pass
end-to-end.

Output: a running sim that persists to SQLite, honors optimistic concurrency, and
boots in <2s.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/phases/02-simulator-audit-log/02-CONTEXT.md
@.planning/research/simulator-design.md
@crates/reposix-core/src/issue.rs
@crates/reposix-core/src/audit.rs
@crates/reposix-sim/src/lib.rs
@crates/reposix-sim/Cargo.toml
@clippy.toml

<interfaces>
<!-- Extracted from crates/reposix-core. Executor uses these verbatim; no exploration. -->

From `reposix-core/src/issue.rs`:
```rust
pub struct IssueId(pub u64);
pub enum IssueStatus { Open, InProgress, InReview, Done, WontFix }
pub struct Issue {
    pub id: IssueId,
    pub title: String,
    pub status: IssueStatus,
    pub assignee: Option<String>,
    pub labels: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: u64,
    pub body: String,
}
```
Serialize output is the canonical JSON shape returned by GET.

From `reposix-core/src/audit.rs`:
```rust
pub fn load_schema(conn: &rusqlite::Connection) -> Result<()>;
pub const SCHEMA_SQL: &str;
```
Call `load_schema(&conn)` once at `open_db` time. Triggers are installed by the
fixture; plan 02-02's middleware uses the table via INSERT only.

From `reposix-core::http`:
- `reqwest::Client` construction is **forbidden by clippy lint** outside
  `reposix-core/src/http.rs`. This plan has no clients to construct; tests in
  plan 02-02 will use `reposix_core::http::client(ClientOpts::default())`.
</interfaces>

<constraints>
- `#![forbid(unsafe_code)]` and `#![warn(clippy::pedantic)]` stay on. Every new
  `Result` fn has a `# Errors` section. Public items documented.
- SQLite connection lives in `Arc<parking_lot::Mutex<Connection>>`. **Never hold the
  lock across `.await`.** Pattern: `let row = { let conn = state.db.lock(); conn.query_row(...) };`
- `If-Match` parsing honors RFC 7232 quoted-etag form (`"<version>"`). Absent header
  → treat as wildcard (allow). Present and mismatch → 409 with body
  `{"error":"version_mismatch","current":<n>,"sent":"<raw>"}`.
- `id` is zero-padded to 4 digits *only* at the filename boundary (Phase 3); JSON
  responses use the raw `u64` via `IssueId(pub u64)` Serialize.
- Seed loader must be deterministic: fixed `created_at`/`updated_at` (e.g.
  `2026-04-13T00:00:00Z`) and `version = 1`.
- One of the seed issues MUST contain a `<script>alert(1)</script>` body; another
  MUST contain a line `version: 999` inside its body text. These are the
  adversarial fixtures Phase 3 will use.
- Do NOT add `Tainted<T>` wrapping here — Phase 3 introduces sanitize on the write
  path. Leave a comment in create/patch handlers:
  `// TODO(phase-3): wrap inbound body in Tainted<T> before frontmatter stripping.`
</constraints>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: AppState, db.rs, seed.json, ApiError — storage layer compiles with unit tests</name>
  <files>
    crates/reposix-sim/src/state.rs,
    crates/reposix-sim/src/db.rs,
    crates/reposix-sim/src/seed.rs,
    crates/reposix-sim/src/error.rs,
    crates/reposix-sim/fixtures/seed.json,
    crates/reposix-sim/src/lib.rs
  </files>
  <behavior>
    - `open_db(path: &Path, ephemeral: bool) -> Result<Connection>` opens SQLite, sets
      `PRAGMA journal_mode=WAL` (skipped for `:memory:`), creates `issues` table with
      columns `(id INTEGER, project TEXT, title TEXT, status TEXT, assignee TEXT NULL,
      labels TEXT NOT NULL /* JSON array */, created_at TEXT, updated_at TEXT,
      version INTEGER NOT NULL DEFAULT 1, body TEXT NOT NULL, PRIMARY KEY(project,id))`,
      calls `reposix_core::audit::load_schema(&conn)`.
    - Unit test: `open_db(":memory:", true)` succeeds; calling twice is idempotent;
      `SELECT name FROM sqlite_master WHERE type='trigger'` returns `audit_no_update`
      and `audit_no_delete`.
    - `load_seed(&conn, path)` reads JSON with `{project, issues[]}` and INSERTs rows
      (idempotent via `INSERT OR IGNORE`). Returns count inserted.
    - Unit test: loading `fixtures/seed.json` into in-memory db yields `COUNT(*) = 3`
      and at least one body contains `version: 999`.
    - `ApiError` enum variants: `NotFound`, `BadRequest(String)`,
      `VersionMismatch{current:u64, sent:String}`, `Db(rusqlite::Error)`,
      `Json(serde_json::Error)`. `impl IntoResponse` returns JSON
      `{error, message, details?}` with correct status.
    - Unit test:
      `ApiError::VersionMismatch{current:5, sent:"bogus".into()}.into_response().status() == 409`.
  </behavior>
  <action>
    1. Create `state.rs` with `pub struct AppState { pub db: Arc<parking_lot::Mutex<rusqlite::Connection>>, pub config: Arc<SimConfig> }`. Implement `Clone` manually (Arc clones).
    2. Create `error.rs` with the enum above. Use `thiserror::Error`. `IntoResponse`
       maps `VersionMismatch` → 409, `NotFound` → 404, `BadRequest` → 400,
       `Db`/`Json` → 500 with opaque message (log via `tracing::error!`; do not leak
       rusqlite internals to clients).
    3. Create `db.rs` with `open_db` as specified. DDL as `const ISSUES_SQL: &str`.
       Use `conn.execute_batch`. Set WAL via `conn.pragma_update`.
    4. Create `fixtures/seed.json` with the exact shape from 02-CONTEXT.md §Seed data:
       issue 1 body contains `<script>alert(1)</script>` and reproduction steps;
       issue 2 body is a plain enhancement description; issue 3 body contains a
       literal line `version: 999` among markdown.
    5. Create `seed.rs` with `load_seed` using `serde_json::from_str` into an internal
       `SeedFile { project: SeedProject, issues: Vec<SeedIssue> }` struct; INSERT OR
       IGNORE each row. Deterministic timestamps:
       `Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap()`.
    6. Update `lib.rs` to `mod state; mod db; mod seed; mod error;` and re-export
       `pub use state::AppState;`. Leave `build_router` returning just `/healthz` for
       now.
    7. Unit tests in each module (behavior block above). Place in `#[cfg(test)] mod tests`.
  </action>
  <verify>
    <automated>cargo test -p reposix-sim --lib -- --nocapture 2>&amp;1 | tail -40 &amp;&amp; cargo clippy -p reposix-sim --all-targets -- -D warnings</automated>
  </verify>
  <done>
    All unit tests green. `cargo clippy -p reposix-sim -- -D warnings` clean. Seed JSON
    round-trips. Audit triggers visible via `sqlite_master` after `open_db`.
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Routes (list/get/create/patch/delete + transitions) wired through AppState</name>
  <files>
    crates/reposix-sim/src/routes/mod.rs,
    crates/reposix-sim/src/routes/issues.rs,
    crates/reposix-sim/src/routes/transitions.rs,
    crates/reposix-sim/src/lib.rs,
    crates/reposix-sim/src/main.rs
  </files>
  <behavior>
    - `GET /projects/:slug/issues` → 200 with `Vec<Issue>` JSON, ordered by id ASC.
    - `GET /projects/:slug/issues/:id` → 200 with single Issue, 404 if absent.
    - `POST /projects/:slug/issues` body `{title, body?, status?, assignee?, labels?}`
      → 201 with full Issue (server assigns id = max(existing)+1, version=1,
      timestamps=now). `Location: /projects/{slug}/issues/{id}` header.
    - `PATCH /projects/:slug/issues/:id` with `If-Match: "<n>"` header. Mutable:
      `{title?, body?, status?, assignee?, labels?}`. Server fields (`id`,
      `created_at`, `version`, `updated_at`) are stripped server-side (ignored if
      sent). On success: bump version, set updated_at, return 200 with new Issue.
      Absent If-Match → allow (wildcard). Stale → 409
      `{error:"version_mismatch", current:<n>, sent:"<raw>"}`.
    - `DELETE /projects/:slug/issues/:id` → 204 on success, 404 if absent.
    - `GET /projects/:slug/issues/:id/transitions` → 200 with
      `{current_state, available:[...]}` where available lists all 5 `IssueStatus`
      values minus current (v0.1 best-effort; true workflow rules deferred).
    - Unit tests use `axum::body::Body::from` + `tower::ServiceExt::oneshot` against
      `build_router(AppState)` with an in-memory seeded db. Cover: list, get 200/404,
      create 201, patch success and 409, delete 204, transitions.
  </behavior>
  <action>
    1. Create `routes/mod.rs` with `pub fn router(state: AppState) -> Router` that
       merges `issues::router(state.clone())` and `transitions::router(state)`.
       Return a single `Router` with `.with_state(state)`.
    2. Create `routes/issues.rs` with handler functions per behavior. Use
       `axum::extract::{Path, State, Json}` and `axum::http::{HeaderMap, StatusCode}`.
       Parse `If-Match` by stripping surrounding quotes, then `u64::from_str`. If
       parsing fails → treat as `sent="<raw>"` and return 409 (anything non-numeric
       is a mismatch).
    3. DB access pattern: every handler does
       `let result = { let conn = state.db.lock(); /* sync rusqlite */ };` then any
       `.await` *outside* the lock. No `.await` inside the critical section.
    4. For PATCH: use `conn.transaction_with_behavior(TransactionBehavior::Immediate)`,
       read current version, compare, UPDATE with version bump, commit. On mismatch
       return 409 via `ApiError::VersionMismatch`.
    5. Create `routes/transitions.rs` with the single GET handler above.
    6. Update `lib.rs::build_router` to accept `AppState` and return
       `Router::new().route("/healthz", get(healthz)).merge(routes::router(state))`.
       Update `run()`: builds state (`open_db`, `load_seed` if `cfg.seed &&
       cfg.seed_file.is_some()`), calls `axum::serve(listener, build_router(state))`.
    7. Rewrite `main.rs` with clap: `--bind`, `--db`, `--seed-file`, `--no-seed`,
       `--ephemeral`, `--rate-limit` (parsed but not yet wired — plan 02-02 consumes
       it). Default bind `127.0.0.1:7878`, default db `runtime/sim.db`.
    8. Extend `SimConfig` in `lib.rs` to include `pub seed_file: Option<PathBuf>`,
       `pub rate_limit_rps: u32` (default 100). `ephemeral()` stays valid.
    9. Unit tests in `routes/issues.rs` under `#[cfg(test)] mod tests` using
       `tower::ServiceExt`. Cover each endpoint; assert JSON bodies via
       `serde_json::from_slice`.
  </action>
  <verify>
    <automated>cargo test -p reposix-sim --lib -- --nocapture 2>&amp;1 | tail -60 &amp;&amp; cargo clippy -p reposix-sim --all-targets -- -D warnings &amp;&amp; cargo run -p reposix-sim -- --bind 127.0.0.1:17878 --ephemeral --seed-file crates/reposix-sim/fixtures/seed.json &amp; SIM_PID=$!; sleep 1; test "$(curl -sf http://127.0.0.1:17878/projects/demo/issues | python3 -c 'import sys,json;print(len(json.load(sys.stdin)))')" -ge 3; CODE=$(curl -s -o /dev/null -w '%{http_code}' -X PATCH -H 'If-Match: "bogus"' -H 'Content-Type: application/json' -d '{"status":"done"}' http://127.0.0.1:17878/projects/demo/issues/1); kill $SIM_PID; test "$CODE" = "409"</automated>
  </verify>
  <done>
    All route unit tests green. Clippy clean. Live smoke: list returns >=3, PATCH with
    bogus If-Match returns 409. `cargo run -p reposix-sim --` boots in <2s.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| external client → axum router | Untrusted JSON arrives at POST/PATCH handlers. |
| axum handler → rusqlite | Parameter binding must prevent SQL injection. |
| seed.json file → db | Seed is trusted (developer-authored) but contains adversarial fixtures (`<script>`, fake `version: 999`) so downstream code is exercised against them. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-02-01 | Tampering | PATCH handler (server-managed fields) | mitigate | Ignore client-supplied `id`, `created_at`, `version`, `updated_at` in the PATCH body; only apply the mutable-field allow-list. Phase 3 will add `Tainted<T>` enforcement on the inbound write path. |
| T-02-02 | Tampering | PATCH handler (optimistic concurrency) | mitigate | `If-Match` version check inside an `IMMEDIATE` transaction; 409 on mismatch. This is the guarantee Phase S's `git push` relies on to turn stale writes into merge conflicts. |
| T-02-03 | Injection | all rusqlite calls | mitigate | Use `rusqlite::params!` / bound parameters exclusively; no `format!` into SQL. Clippy `sql-injection`-adjacent lints stay on. |
| T-02-04 | Info disclosure | ApiError::Db / Json responses | mitigate | 500 response body is opaque (`{"error":"internal"}`); full rusqlite error is `tracing::error!`'d to logs, not returned. |
| T-02-05 | DoS | unbounded POST body | accept (v0.1) | Phase 1's `tower-http` limit layer is not yet attached here; plan 02-02 adds the 1 MiB `to_bytes` cap in the audit middleware, which sits outermost. Document here so plan 02-02 owns the fix. |
| T-02-06 | Repudiation | handler actions without audit | accept (this plan only) | No audit rows are written in this plan; every handler response is non-repudiable only after plan 02-02 attaches the audit middleware. Tracked as a phase-level gap until 02-02 ships. |
</threat_model>

<verification>
Phase-level checks this plan contributes to (plan 02-02 completes audit + rate-limit):
- `cargo test -p reposix-sim --lib` green.
- `cargo clippy -p reposix-sim --all-targets -- -D warnings` clean.
- Live: ROADMAP success criteria 1, 2, 3 pass with bare sim (no audit/rate-limit
  layers attached yet; criterion 4 requires plan 02-02).

Goal-backward Bash (after 02-01 alone):
```bash
cargo run -p reposix-sim -- --bind 127.0.0.1:17878 --ephemeral \
  --seed-file crates/reposix-sim/fixtures/seed.json &
PID=$!; sleep 1
curl -sf http://127.0.0.1:17878/projects/demo/issues | jq 'length' | grep -q '^[3-9]'
curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:17878/projects/demo/issues/1 | grep -q '^200$'
curl -s -o /dev/null -w '%{http_code}' -X PATCH -H 'If-Match: "bogus"' \
  -H 'Content-Type: application/json' -d '{"status":"done"}' \
  http://127.0.0.1:17878/projects/demo/issues/1 | grep -q '^409$'
kill $PID
# SC1, SC2, SC3 of ROADMAP Phase-2 pass.
```
</verification>

<success_criteria>
- All `must_haves.truths` verifiable via the commands in each task's `<verify>`.
- No `reqwest::Client::{new,builder}` usage anywhere in `reposix-sim` (clippy enforces).
- No lock held across `.await` (reviewable by grepping `state.db.lock` near `.await`).
- ROADMAP Phase-2 success criteria 1, 2, 3 pass against the live binary.
</success_criteria>

<output>
After completion, create `.planning/phases/02-simulator-audit-log/02-01-SUMMARY.md`
documenting: actual file list, seed schema, ApiError JSON shape, If-Match parsing
rule, and any deviations (e.g. if the issues DDL changed). Plan 02-02 reads this
SUMMARY to learn the AppState and ApiError shapes it will extend.
</output>
</content>
