---
phase: 81
plan: 01
title: "DVCS-PERF-L1-01..03 — L1 conflict detection (cache cursor + helper precheck rewrite + sync --reconcile + perf regression)"
wave: 1
depends_on: [80]
requirements: [DVCS-PERF-L1-01, DVCS-PERF-L1-02, DVCS-PERF-L1-03]
files_modified:
  - crates/reposix-cache/src/cache.rs
  - crates/reposix-remote/src/main.rs
  - crates/reposix-remote/src/precheck.rs
  - crates/reposix-remote/tests/perf_l1.rs
  - crates/reposix-cli/src/main.rs
  - crates/reposix-cli/src/lib.rs
  - crates/reposix-cli/src/sync.rs
  - crates/reposix-cli/tests/sync.rs
  - quality/catalogs/perf-targets.json
  - quality/catalogs/agent-ux.json
  - quality/catalogs/doc-alignment.json
  - quality/gates/perf/list-call-count.sh
  - quality/gates/agent-ux/sync-reconcile-subcommand.sh
  - CLAUDE.md
autonomous: true
mode: standard
---

# Phase 81 Plan 01 — L1 perf migration (DVCS-PERF-L1-01..03)

<objective>
Land the L1 conflict-detection migration for v0.13.0's DVCS topology
BEFORE the bus remote ships in P82–P83, so the bus inherits the cheap
push path (one `list_changed_since` REST call + actual writes) instead
of the expensive one (paginated `list_records` walk on every push).
This is the v0.13.0 milestone's secondary value-add — single-backend
pushes get the same per-push cost improvement that bus pushes need
to satisfy the DVCS thesis ("DVCS at the same UX as plain git").

This is a **single plan, four sequential tasks** per RESEARCH.md
§ "Plan Splitting":

- **T01** — Catalog-first: 3 rows (perf + agent-ux + doc-alignment) +
  2 TINY verifier shells (status FAIL).
- **T02** — Cache cursor wrappers (`read_last_fetched_at` /
  `write_last_fetched_at` over `meta::get_meta`/`set_meta`) +
  helper precheck rewrite (new `precheck.rs` module + `handle_export`
  rewrite from `list_records` to `list_changed_since`-driven check).
- **T03** — `reposix sync --reconcile` subcommand (clap variant + new
  `sync.rs` module + smoke test).
- **T04** — Perf regression test + positive-control + catalog flip
  FAIL → PASS + CLAUDE.md update + per-phase push.

Sequential (T01 → T02 → T03 → T04). Per CLAUDE.md "Build memory budget"
the executor holds the cargo lock sequentially across T02 → T03 → T04.
T01 is doc-only (catalog rows + verifier shell scaffolding).

**Architecture (read BEFORE diving into tasks):**

The cursor lives at `meta.last_fetched_at` (single row in the existing
`meta` SQLite table; key/value/updated_at columns;
`crates/reposix-cache/src/meta.rs:9-34`). Both `Cache::build_from`
(`builder.rs:119`) and `Cache::sync` (`builder.rs:329`) already write
this row on success — P81's contribution is the helper-side READ on
push entry + WRITE after successful execute. The new
`Cache::read_last_fetched_at` and `Cache::write_last_fetched_at` are
thin wrappers that mirror P80's `read_mirror_synced_at` /
`write_mirror_synced_at` shape. Parsing: existing
`chrono::DateTime::parse_from_rfc3339` per
`crates/reposix-cache/src/builder.rs:233`.

The `BackendConnector::list_changed_since(project, since) -> Vec<RecordId>`
trait method already exists at `crates/reposix-core/src/backend.rs:253`
with a default impl + per-backend overrides on all 4 connectors:
- sim: `?since=<RFC3339>` query param (`crates/reposix-core/src/backend/sim.rs:281-303`)
- confluence: CQL `lastModified > "..."`
- github: `?since=<RFC3339>`
- jira: JQL `updated >= "..."`

The new precheck function lives in `crates/reposix-remote/src/precheck.rs`
(NEW module). Both single-backend `handle_export` (P81) and the future
bus handler (P82+) call it. Function signature:

```rust
pub(crate) fn precheck_export_against_changed_set(
    state: &mut State,
    parsed: &ParsedExport,
) -> Result<PrecheckOutcome>;
```

`PrecheckOutcome` is one of `Conflicts(Vec<(RecordId, u64, u64, String)>)`
| `Ok(prior: Vec<Record>)` (the `Vec<Record>` returned for the success
case is what `plan()` consumes — D-03 ratifies that `plan()`'s signature
is unchanged).

The L1-strict delete trade-off (D-01) is RATIFIED:
`list_changed_since` does NOT report backend-side deletes (Confluence
CQL `lastModified > X` returns nothing for deleted pages). The cache is
trusted as the prior set; backend-deleted records surface as REST 404
on PATCH at write time. User recovery: `reposix sync --reconcile` (T03).

Annotated-tag-style citation: the inline comment in `precheck.rs`
references BOTH `.planning/research/v0.13.0-dvcs/architecture-sketch.md
§ Performance subtlety` AND
`.planning/research/v0.14.0-observability-and-multi-repo/vision-and-mental-model.md
§ L2/L3 cache-desync hardening` — verbatim citation per D-01.

**First-push fallback (S2 in overview):** when `read_last_fetched_at()`
returns `Ok(None)`, fall through to the existing `list_records` walk
for THIS push only, then write the cursor. Subsequent pushes hit the L1
fast path. Surfaced via `tracing::info!` (single line, NOT a hot path
at scale).

**Best-effort vs hard-error semantics:**

- **Cursor write:** best-effort. `tracing::warn!` on failure; the push
  still acks `ok` to git. Matches the existing `Cache::log_*` family
  precedent (let-else + WARN). NO new error variant.
- **`list_changed_since` REST failure:** same shape as today's
  `list_records` failure path — `fail_push(...,
  "backend-unreachable", ...)`. Existing reject path; no new code.

This plan **must run cargo serially** per CLAUDE.md "Build memory
budget". Per-crate fallback (`cargo check -p reposix-cache`,
`cargo check -p reposix-remote`, `cargo check -p reposix-cli`) used
instead of workspace-wide.

This plan terminates with `git push origin main` (per CLAUDE.md push
cadence) with pre-push GREEN. The catalog rows' initial FAIL status
is acceptable through T01–T03 because the rows are `pre-pr` cadence
(NOT `pre-push`); the runner re-grades to PASS during T04 BEFORE the
push commits.
</objective>

<must_haves>
**Cache crate (T02)** — `crates/reposix-cache/src/cache.rs`:
- `pub fn read_last_fetched_at(&self) -> Result<Option<chrono::DateTime<chrono::Utc>>>`
  — wraps `meta::get_meta(conn, "last_fetched_at")`; parses RFC3339; returns
  `Ok(None)` when the row is absent OR the stored string fails to parse
  (defensive WARN-log, fall through to first-push semantics).
- `pub fn write_last_fetched_at(&self, ts: DateTime<Utc>) -> Result<()>`
  — wraps `meta::set_meta(...)`. Best-effort caller pattern with
  `tracing::warn!` guard.
- **`pub fn read_blob_cached(&self, oid: gix::ObjectId) -> Result<Option<Tainted<Vec<u8>>>>`**
  — NEW sync gix-only primitive. Reads the blob directly from the cache's
  bare repo via `gix::Repository::find_object`; returns `Ok(None)` when the
  object is not present (instead of fetching from backend). This is the
  local-only inspector counterpart to the async materializer `read_blob`.
  The precheck path (T02 § 2b) MUST use this — calling the async
  `read_blob` would add a hidden backend GET per cache prior and defeat the
  L1 perf goal. See H1 fix in PLAN-CHECK.md.
- 2 unit tests: `read_last_fetched_at_round_trips`,
  `read_last_fetched_at_returns_none_when_absent`.
- 1 unit test: `read_blob_cached_returns_some_when_blob_in_repo`,
  `read_blob_cached_returns_none_when_blob_absent` (2 unit tests for the
  new primitive).
- `# Errors` doc on each new pub fn; `cargo clippy -p reposix-cache --
  -D warnings` clean.

**Remote crate (T02)** — new file `crates/reposix-remote/src/precheck.rs`
(≤ 200 lines, `pub(crate)` exports):
- Module doc-comment cites the L1-strict delete trade-off (D-01) verbatim
  with both architecture-sketch + v0.14.0 doc references (the only place
  outside CLAUDE.md where future agents reading the helper code find the
  cost-vs-correctness rationale).
- `pub(crate) enum PrecheckOutcome { Conflicts(Vec<(RecordId, u64, u64, String)>), Proceed { prior: Vec<Record> } }`.
- **`pub(crate) fn precheck_export_against_changed_set(cache: Option<&Cache>, backend: &dyn BackendConnector, project: &str, rt: &Runtime, parsed: &ParsedExport) -> anyhow::Result<PrecheckOutcome>`** (M1 fix — narrowed dependencies for P82 bus-handler reuse). The function takes its dependencies explicitly rather than `&mut State`, so the future bus handler (`BusState { sot, mirror }` in P82) can construct the same call without conforming to the single-backend `State` shape. The single-backend `handle_export` call site does ~10 lines of plumbing: `precheck_export_against_changed_set(state.cache.as_ref(), state.backend.as_ref(), &state.project, &state.rt, &parsed)`.
- Full algorithm implemented per T02 § 2b. Hot-path optimization:
  per-record parse + GET only fires for ids in `changed_set ∩ push_set`
  (D-03 + RESEARCH.md § Pitfall 5).
- **Error flow (H4 fix):** uses `anyhow::Result<PrecheckOutcome>` and
  `anyhow::Error` throughout. NO typed `Error::BackendUnreachable` /
  `Error::Cache` variants — the remote crate uses `anyhow` (see
  `crates/reposix-remote/src/main.rs:18`: `use anyhow::{Context, Result}`;
  there is NO `crates/reposix-remote/src/error.rs`). Reject-path stderr
  string `"backend-unreachable"` is preserved at the `fail_push(diag, ...)`
  call site in `handle_export` via `.context("backend-unreachable: ...")`.

**Remote crate (T02)** — `crates/reposix-remote/src/main.rs`:
- `mod precheck;` declaration alongside existing module declarations.
- **`State` visibility widened (H3 fix):** `struct State` (currently private
  at line 42) → `pub(crate) struct State`; the four fields `rt`, `backend`,
  `project`, `cache` widened to `pub(crate)`. Also widen the private free
  function `fn issue_id_from_path(path: &str) -> Option<u64>` (line 554)
  to `pub(crate) fn issue_id_from_path(...)`. These widenings let the
  sibling `precheck.rs` module import them via `use crate::{State,
  issue_id_from_path};` (NOT `crate::main::...` — `main.rs` is the binary
  root, not a `main` sub-module). Verify the new visibility holds via
  `cargo check -p reposix-remote` after the precheck.rs file lands.
- `handle_export` rewrite: lines 334–382 (post-P80; re-confirm via grep —
  `grep -n 'fn handle_export\|state.backend.list_records\|log_helper_push_accepted\|refresh_for_mirror_head' crates/reposix-remote/src/main.rs` shows current `list_records` call at line 336)
  replaced with a single `precheck::precheck_export_against_changed_set(...)`
  call matched on `PrecheckOutcome`. Existing reject branch (lines
  384–427) consumes `Conflicts(c)` UNCHANGED; existing
  `plan(&prior, &parsed)` call (line 429) consumes
  `Proceed { prior }` UNCHANGED.
- Cursor write inserted in success branch BETWEEN
  `cache.log_helper_push_accepted` and the P80 mirror-refs block:
  ```rust
  if let Err(e) = cache.write_last_fetched_at(chrono::Utc::now()) {
      tracing::warn!("write_last_fetched_at failed: {e:#}");
  }
  ```
- D-03: `plan()` signature in `diff.rs` UNCHANGED. Helper materializes
  `Vec<Record>` from cache before calling `plan()`. No widening of
  `diff.rs` test surface.

**CLI crate (T03)** — new file `crates/reposix-cli/src/sync.rs` (~50
lines):
- `pub async fn run(reconcile: bool, path: Option<PathBuf>) -> anyhow::Result<()>`.
- `reconcile=false`: prints a hint pointing at `--reconcile`; exits 0
  (NOT an error — bare `reposix sync` reserved for future flag combos
  per D-02).
- `reconcile=true`: opens cache via existing helper (whichever shape
  refresh.rs / attach.rs uses; do NOT introduce a new accessor); calls
  `cache.build_from().await?`; prints synthesis-commit OID.
- `# Errors` doc.

**CLI crate (T03)** — `crates/reposix-cli/src/{lib.rs,main.rs}`:
- `pub mod sync;` in lib.rs (alphabetical between `spaces` and `tokens`).
- `Sync { reconcile: bool, path: Option<PathBuf> }` clap variant in
  main.rs `enum Cmd`.
- `Cmd::Sync { reconcile, path } => sync::run(reconcile, path).await`
  match arm in main.
- `use reposix_cli::{..., sync, ...}` updated.
- `cargo run -p reposix-cli -- sync --reconcile --help` exits 0;
  `cargo run -p reposix-cli -- sync` (no flags) exits 0 + prints the
  hint.

**Tests (T03 + T04)**:
- `crates/reposix-cli/tests/sync.rs::sync_reconcile_advances_cursor`
  (1 test) — wiremock-backed sim; seed sync; sleep 1100 ms;
  `reposix_cli::sync::run(true, ...)`; assert `last_fetched_at`
  advanced.
- `crates/reposix-remote/tests/perf_l1.rs::l1_precheck_uses_list_changed_since_not_list_records`
  (1 test) — N=200 wiremock harness; mock `list_records` with
  `expect(0)`; mock `list_changed_since` with `expect(1..)` returning
  empty array; drive export verb; wiremock Drop asserts at test-end.
- `crates/reposix-remote/tests/perf_l1.rs::positive_control_list_records_call_fails_red`
  (1 test, `#[should_panic(expected = "Verifications failed")]`) — same
  setup but `list_records` mock has `expect(1)`; confirms wiremock
  actually fails RED on Drop (closes RESEARCH.md MEDIUM risk).

**Catalog rows + verifiers (T01)** — 3 rows + 2 TINY shells:
- `quality/catalogs/perf-targets.json` ←
  `perf/handle-export-list-call-count` (status FAIL initial; hand-edit
  per documented gap with `_provenance_note` citing GOOD-TO-HAVES-01).
- `quality/catalogs/agent-ux.json` ←
  `agent-ux/sync-reconcile-subcommand` (status FAIL initial; hand-edit).
- `quality/catalogs/doc-alignment.json` ←
  `docs-alignment/perf-subtlety-prose-bound` minted via
  `reposix-quality doc-alignment bind` (Principle A applies); binds
  the architecture-sketch prose paragraph "L1 trades one safety
  property: today's `list_records` would catch a record that exists
  on backend but missing from cache" to `tests/perf_l1.rs::l1_precheck_uses_list_changed_since_not_list_records`.
- `quality/gates/perf/list-call-count.sh` (~30 lines; delegates to
  `cargo test -p reposix-remote --test perf_l1`).
- `quality/gates/agent-ux/sync-reconcile-subcommand.sh` (~30 lines;
  delegates to `cargo run -- sync --reconcile --help` + `cargo test
  -p reposix-cli --test sync`).
- All three rows flip FAIL → PASS via the runner during T04 BEFORE
  the per-phase push commits.

**CLAUDE.md (T04 — D-05; two paragraphs):**
- § Commands → "Local dev loop" block: bullet for `reposix sync
  --reconcile` (1 line).
- § Architecture: "L1 conflict detection (P81+)" paragraph (3-5
  sentences) naming the L1-strict trade-off and citing the
  architecture-sketch.

**Phase-close contract:**
- Plan terminates with `git push origin main` in T04 (per CLAUDE.md
  push cadence) with pre-push GREEN. Verifier subagent dispatch is
  an orchestrator-level action AFTER push lands — NOT a plan task.
- All cargo invocations SERIAL (one at a time per CLAUDE.md "Build
  memory budget"). Per-crate (`-p reposix-cache`, `-p reposix-remote`,
  `-p reposix-cli`) only. NO `cargo --workspace`.
- NO new error variants on `RemoteError` or `cache::Error`.
  Cursor-read failures fall back to first-push semantics (Ok(None));
  cursor-write failures WARN-log. Best-effort callers throughout.
- L1-strict delete trade-off (D-01) RATIFIED in three places: plan
  body, inline comment in `precheck.rs` (T02), CLAUDE.md (T04).
</must_haves>

## Chapters

This plan is split into the following chapters for readability:

- **[Threat Analysis](./01-threat-analysis.md)** — Trust boundaries and STRIDE threat register.
- **[Task 01: Catalog-first](./T01-catalog-first.md)** — Mint 3 catalog rows and author 2 verifier shells.
- **[Task 02a: Cache Cursor Wrappers](./T02a-cache-cursor-wrappers.md)** — Read-first checklist + `read_last_fetched_at` / `write_last_fetched_at` wrappers + unit tests + build steps.
- **[Task 02b: State Widening & precheck.rs](./T02b-state-widening-precheck-module.md)** — `pub(crate) struct State` widening, `issue_id_from_path` visibility, and new `precheck.rs` module with full algorithm.
- **[Task 02c: handle_export Rewrite & Cursor Write](./T02c-handle-export-rewrite-cursor-write.md)** — `handle_export` precheck call, cursor-write insertion, serial build/test, and commit.
- **[Task 03: Sync Reconcile CLI](./T03-sync-reconcile.md)** — `reposix sync --reconcile` CLI subcommand and smoke test.
- **[Task 04: Perf Test & Catalog Flip](./T04-perf-test-close.md)** — Perf regression test, positive-control, catalog flip, CLAUDE.md update, and per-phase push.
- **[Close Protocol](./05-close-protocol.md)** — Plan-internal close protocol and orchestrator actions.
