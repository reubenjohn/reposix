---
phase: 79
plan: 02
title: "DVCS-ATTACH-01..02 (scaffold) — `reposix attach` clap surface + cache reconciliation module"
wave: 2
depends_on: [79-01]
requirements: [DVCS-ATTACH-01, DVCS-ATTACH-02, DVCS-ATTACH-04]
files_modified:
  - crates/reposix-cli/src/main.rs
  - crates/reposix-cli/src/lib.rs
  - crates/reposix-cli/src/attach.rs
  - crates/reposix-cache/src/lib.rs
  - crates/reposix-cache/src/reconciliation.rs
  - crates/reposix-cache/src/db.rs
  - crates/reposix-cache/src/cache.rs
  - quality/catalogs/agent-ux.json
  - quality/gates/agent-ux/reposix-attach.sh
autonomous: true
mode: standard
---

# Phase 79 Plan 02 — `reposix attach` scaffold (DVCS-ATTACH-01..02 scaffold + 04 type-assertion)

<objective>
Scaffold `reposix attach <backend>::<project>` as a real clap subcommand
in `crates/reposix-cli/`, mint the agent-ux catalog row + verifier (status
FAIL), land the cache reconciliation table + module + new public Cache
APIs + audit hook in `crates/reposix-cache/`, and provide the
type-system assertion that `Cache::read_blob` returns `Tainted<Vec<u8>>`
(reframed DVCS-ATTACH-04 part 1).

This plan is **scaffold-only** per checker B1 — integration tests +
behavior coverage land in 79-03. The split keeps each plan ≤ 4
cargo-heavy tasks under the context budget.

What this plan delivers (sufficient for 79-03 to write tests against):

- A clap `Cmd::Attach(AttachArgs)` that compiles and dispatches to a
  `reposix_cli::attach::run(args)` async function — even if the function
  body is partially scaffolded; integration tests in 79-03 are what
  drives full behavior.
- New SQLite table `cache_reconciliation` (record_id INTEGER PRIMARY
  KEY, oid TEXT, local_path TEXT, attached_at TEXT) created idempotently
  by `open_cache_db`.
- New module `crates/reposix-cache/src/reconciliation.rs` with
  `walk_and_reconcile`, `ReconciliationReport`, and `OrphanPolicy`.
- Three new public Cache APIs (`Cache::list_record_ids`,
  `Cache::find_oid_for_record`, `Cache::connection_mut`) — confirmed
  not-yet-extant via grep at planning time. Each carries `# Errors`
  doc + clippy::pedantic compliance per CLAUDE.md "Code style".
- `Cache::log_attach_walk` audit hook on the existing
  `audit_events_cache` table (OP-3 — UNCONDITIONAL; the audit write
  lands in 79-02 T03, NOT deferred to 79-03).
- Type-system assertion test in `crates/reposix-cache/src/reconciliation.rs`
  `#[cfg(test)] mod tests` that proves `Cache::read_blob` returns
  `Tainted<Vec<u8>>` (DVCS-ATTACH-04 reframed part 1).
- Catalog row `agent-ux/reposix-attach-against-vanilla-clone` minted in
  `quality/catalogs/agent-ux.json` with `status: FAIL` + the TINY
  verifier shell at `quality/gates/agent-ux/reposix-attach.sh`. Hand-edit
  per documented gap (NOT Principle A — see OVERVIEW § "New
  GOOD-TO-HAVES entry"). The runner re-grades to PASS at end-of-79-03.

What this plan defers to 79-03:

- The 6 reconciliation-case integration tests + the
  `attach_against_vanilla_clone_sets_partial_clone` post-condition test.
- The 2 re-attach tests (idempotent + reject).
- The Tainted-materialization integration test (DVCS-ATTACH-04 reframed
  part 2).
- The audit-row presence integration test.
- The CLAUDE.md update + per-phase catalog-flip push.

Architecture (for executor context):

Post-attach, the working tree has TWO remotes:

- `origin` — plain-git (the GH mirror that originally seeded the clone).
  KEEPS plain-git semantics; NOT mutated by attach.
- `<reposix-remote-name>` (default `reposix`) — reposix-equipped; the new
  push target that bus-remote machinery (P82-P83) will eventually drive.
  `extensions.partialClone` is set to THIS remote (NOT origin).

The cache:
- Lives at `resolve_cache_path(<sot-backend>, <sot-project>)` per Q1.1
  (DECIDED: derives from SoT URL, NOT from `remote.origin.url`).
- Stores OIDs (filenames + tree structure; blobs lazy on first
  materialize) — same contract as `Cache::build_from`.
- `cache_reconciliation` table is reconciliation state, NOT an audit
  table — append-only audit semantics do not apply (INSERT OR REPLACE
  on re-attach).
- `Cache::read_blob` (existing API at `crates/reposix-cache/src/builder.rs:436`)
  is the materialization seam; it already returns `Tainted<Vec<u8>>`
  (OP-2 contract preserved — no new sanitize call sites).

Reconciliation walk (DVCS-ATTACH-02 — implemented in 79-02 T03; tested in 79-03 T01):

| Local file                         | Backend record               | Resolution                                                     |
|------------------------------------|------------------------------|----------------------------------------------------------------|
| `*.md` with `id: N` in frontmatter | record `id: N` exists        | match; row in `cache_reconciliation`                           |
| `*.md` with `id: N`                | no record `id: N`            | warn; skip; offer `--orphan-policy={delete-local,fork-as-new,abort}` |
| `*.md` with no `id` field          | n/a                          | warn; skip; not a reposix-managed file                         |
| Two local files claim `id: N`      | n/a                          | hard error (exit non-zero with both file paths in stderr)      |
| Backend record id=N                | no local file                | normal; cache marks for next fetch                             |

Re-attach semantics (DVCS-ATTACH-03 — implemented in 79-02 T02; tested in 79-03 T02):

- Same SoT spec → IDEMPOTENT (Q1.3): refreshes cache state against the
  current backend; updates `cache_reconciliation` rows; no special-casing
  of init-vs-attach origins.
- Different SoT spec → REJECTED (Q1.2): clear error citing the existing
  attached SoT.

This plan **must run cargo serially** per CLAUDE.md "Build memory budget".
It runs in Wave 2 (after 79-01 POC completes); the orchestrator MUST have
read POC-FINDINGS.md and resolved any REVISE/SPLIT routing BEFORE this
plan begins (per 79-PLAN-OVERVIEW.md § "POC findings → planner re-engagement").
</objective>

<must_haves>
- `reposix attach <spec>` exists in `clap` Cli enum (`crates/reposix-cli/src/main.rs`).
- `reposix attach --help` documents:
  - `<spec>` — `<backend>::<project>` form (sim, github, confluence, jira).
  - `--no-bus` — skip the `?mirror=` query param (single-SoT remote).
  - `--orphan-policy` — `delete-local | fork-as-new | abort` (default `abort` per architecture-sketch row 2).
  - `--mirror-name` — name of the existing plain-git remote that maps to
    the GH mirror (default `origin`); used to compose the `?mirror=` URL.
  - `--remote-name` — name to use for the new reposix remote (default
    `reposix`).
- New file `crates/reposix-cli/src/attach.rs` implements the subcommand
  body. The Q1.2 reject path AND the Q1.3 idempotent path are wired in
  this plan (so 79-03 T02 has something to test).
- New file `crates/reposix-cache/src/reconciliation.rs` implements the
  reconciliation walk + the `cache_reconciliation` table CRUD +
  `walk_and_reconcile` + `ReconciliationReport` + `OrphanPolicy`.
- `crates/reposix-cache/src/db.rs` `open_cache_db` creates the
  `cache_reconciliation` table via `CREATE TABLE IF NOT EXISTS` (idempotent
  on re-attach).
- 4 unit tests in `crates/reposix-cache/src/{db.rs, reconciliation.rs}` (`#[cfg(test)] mod tests`):
  1. `cache_reconciliation_table_create_is_idempotent` — call create twice;
     second call is no-op (in `db.rs`).
  2. `walk_collects_id_to_paths_map` — given a tempdir with mixed `.md`
     files, the walk yields the expected `HashMap<RecordId, Vec<PathBuf>>`
     (in `reconciliation.rs`).
  3. `duplicate_id_aborts_without_writes` — two files with id=42 produce
     a non-empty `report.duplicate_id_files`; no rows in
     `cache_reconciliation`.
  4. `cache_read_blob_returns_tainted_type` — type-system assertion
     test (DVCS-ATTACH-04 reframed part 1). The test imports
     `reposix_core::Tainted`, compiles a function `fn _is_tainted(_: Tainted<Vec<u8>>) {}`,
     and feeds the result of `cache.read_blob(oid).await?` into it.
     Compilation failure if `read_blob` ever stops returning `Tainted`.
     Test body uses a tempdir cache + 1 fake blob to satisfy the
     async runtime requirement.
- 3 new public APIs on `Cache` (in `crates/reposix-cache/src/cache.rs`),
  with `# Errors` doc sections + clippy::pedantic clean:
  - `pub fn list_record_ids(&self) -> Result<Vec<RecordId>>` — returns
    the set of backend record IDs known to the cache (from the most
    recent `build_from`'s tree). Reconciliation walker uses this for the
    backend-set comparison.
  - `pub fn find_oid_for_record(&self, id: RecordId) -> Result<Option<gix::ObjectId>>`
    — returns the blob OID for a given record id from the most recent
    tree, if any. Reconciliation walker uses this for the `oid` column
    in `cache_reconciliation` rows.
  - `pub fn connection_mut(&mut self) -> Result<&mut rusqlite::Connection>`
    — exposes the SQLite connection for the reconciliation transaction.
    (May be `pub(crate)` if a stricter seam suffices and the tests in
    `reconciliation.rs` can reach it; prefer `pub(crate)` per
    least-privilege.)
- `Cache::log_attach_walk(report: &ReconciliationReport) -> Result<()>` —
  audit hook on the existing `audit_events_cache` table per OP-3.
  Signature mirrors the existing `Cache::log_helper_*` family. Writes a
  single row with `event_type = "attach_walk"` and a JSON payload
  summarizing the report. **OP-3 is unconditional — this audit write
  lands in this plan, NOT deferred to 79-03.**
- `attach::run` calls `Cache::log_attach_walk` before returning — so any
  attach (test or real) leaves a discoverable audit row.
- The type-system assertion test (`cache_read_blob_returns_tainted_type`)
  proves `Cache::read_blob` returns `Tainted<Vec<u8>>`. This is the
  REFRAMED DVCS-ATTACH-04 acceptance part 1; part 2 (the integration
  test that forces one materialization after attach) lands in 79-03 T02.
- `attach` does NOT touch `origin` — the existing GH mirror remote keeps
  plain-git semantics. (Tested in 79-03 T01.)
- `extensions.partialClone` is set to the new `<remote-name>` (default
  `reposix`), NOT to `origin`. (Tested in 79-03 T01.)
- New catalog row in `quality/catalogs/agent-ux.json`:
  `agent-ux/reposix-attach-against-vanilla-clone` — kind: mechanical,
  cadence: pre-pr, verifier: `quality/gates/agent-ux/reposix-attach.sh`
  (TINY shell verifier 15-30 lines). **Hand-edit per documented gap
  (NOT Principle A) — see OVERVIEW § "New GOOD-TO-HAVES entry".**
  Initial status FAIL; re-grades to PASS at 79-03 T03 push.
- Catalog-first commit: T01 commits the catalog row + the verifier shell
  script BEFORE T02-T03 implementation lands.
- All cargo invocations in this plan are SERIAL (one at a time per CLAUDE.md
  Build memory budget); per-crate fallback (`cargo check -p <crate>`,
  `cargo nextest run -p <crate>`) used instead of workspace-wide.
- Plan terminates with `git push origin main` (per CLAUDE.md push cadence)
  with pre-push GREEN. The catalog row's initial FAIL status is acceptable
  at this push because the row is `pre-pr` cadence (NOT pre-push), so the
  pre-push gate does not include it. If for any reason pre-push runs the
  pre-pr scope, the row's FAIL is documented as expected for this push
  (verifier flips PASS in 79-03 T03).
</must_haves>

<canonical_refs>
- `.planning/REQUIREMENTS.md` DVCS-ATTACH-01..04 — verbatim acceptance.
  DVCS-ATTACH-04 row will be reframed by orchestrator BEFORE verifier
  dispatch (see OVERVIEW § "Reframe of DVCS-ATTACH-04").
- `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § "1. `reposix attach <backend>::<project>`" — sketch + open questions Q1.1, Q1.2, Q1.3.
- `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § "Reconciliation cases" — the 5-row resolution table verbatim.
- `.planning/research/v0.13.0-dvcs/decisions.md` § "Phase-N (`reposix attach`) decisions" — Q1.1/1.2/1.3 ratifications.
- `.planning/research/v0.13.0-dvcs/vision-and-mental-model.md` § "Risks and how we'll know early" — early-signal triggers.
- `research/v0.13.0-dvcs/poc/POC-FINDINGS.md` — POC findings; orchestrator MUST have surfaced any REVISE/SPLIT items via planner re-engagement BEFORE this plan executes.
- `.planning/phases/79-poc-reposix-attach-core/79-PLAN-OVERVIEW.md` § "Reframe of DVCS-ATTACH-04" — the load-bearing reframe drives T03's type-system assertion test.
- `.planning/phases/79-poc-reposix-attach-core/79-PLAN-OVERVIEW.md` § "New GOOD-TO-HAVES entry" — drives T01's hand-edit-not-Principle-A annotation.
- `crates/reposix-cli/src/init.rs:45-96` — `translate_spec_to_url` the attach also calls.
- `crates/reposix-cli/src/init.rs:99-129` — `run_git`, `run_git_in` helpers (attach reuses).
- `crates/reposix-cli/src/main.rs:60-86` — clap subcommand pattern (attach mirrors `Init`).
- `crates/reposix-cache/src/path.rs:22-38` — `resolve_cache_path` (Q1.1 contract).
- `crates/reposix-cache/src/cache.rs:31-200` — `Cache::open` + the existing pub fn surface (lines 54, 132, 138, 144, plus the `log_helper_*` family at 152-309 — the new `log_attach_walk` joins this family). **Confirmed at planning time via grep:** `list_record_ids`, `find_oid_for_record`, `connection_mut` do NOT yet exist; T03 adds them.
- `crates/reposix-cache/src/builder.rs:25-56` — `impl Cache` block; `Cache::build_from(&self) -> Result<gix::ObjectId>` is at line 56 (takes no args; uses backend internally).
- `crates/reposix-cache/src/builder.rs:436` — `pub async fn read_blob(&self, oid: gix::ObjectId) -> Result<Tainted<Vec<u8>>>` — the existing OP-2 contract; the type-system assertion test in T03 imports this signature.
- `crates/reposix-cache/src/db.rs:35-118` — `open_cache_db` (where the new `cache_reconciliation` table CREATE goes).
- `crates/reposix-cache/src/audit.rs` — existing `audit_events_cache` schema; new `Cache::log_attach_walk` writes to this table using the existing pattern from `log_helper_*`.
- `crates/reposix-core/src/record.rs:99-200` — `frontmatter::parse` API.
- `crates/reposix-core/src/record.rs:60` — `Record.id: RecordId` field shape (`RecordId(u64)` wrapper).
- `crates/reposix-core/src/taint.rs` — `Tainted<T>` discipline.
- `crates/reposix-cli/Cargo.toml` — deps (clap, anyhow, etc.).
- `quality/catalogs/agent-ux.json` — existing catalog file; new row joins.
- `quality/catalogs/README.md` § "Unified schema" — required fields per row.
- `quality/PROTOCOL.md` § "Principle A — Subagents propose; tools validate and mint" — the rule that the hand-edit annotates a documented gap from (NOT applies).
- `quality/gates/agent-ux/dark-factory.sh` — TINY-shape verifier precedent (the new attach verifier mirrors this shape).
- `CLAUDE.md` § "Build memory budget" — strict serial cargo.
- `CLAUDE.md` § "Push cadence — per-phase".
- `CLAUDE.md` § Operating Principles OP-1 (simulator-first), OP-2 (Tainted), OP-3 (audit log), OP-7 (verifier subagent), OP-8 (+2 reservation).

This plan introduces no new threat-model surface beyond what already exists
in `Cache::read_blob` (already returns `Tainted<Vec<u8>>`) and the simulator
(`127.0.0.1:7878` default; allowlist-enforced for real backends via
`REPOSIX_ALLOWED_ORIGINS`). The reconciliation walk reads bytes from the
working tree (which is local FS — already trusted) and queries the
backend through the existing `BackendConnector` trait (which already
honors the allowlist). No new HTTP construction site, no new shell-out,
no new sanitization branch. No `<threat_model>` delta required.
</canonical_refs>

---

## Chapters

- [T01 — Catalog-first: mint the agent-ux row + author the verifier shell](./t01.md)
  Mint `agent-ux/reposix-attach-against-vanilla-clone` + TINY verifier shell; status FAIL; hand-edit per documented gap (NOT Principle A).

- [T02 — Subcommand body: `reposix attach <spec>` clap surface + dispatch + Q1.2/1.3 wiring](./t02.md)
  Author `crates/reposix-cli/src/attach.rs` (≤ 250 lines); wire `Cmd::Attach` into `main.rs`; re-export. Q1.2 reject + Q1.3 idempotent paths wired.

- [T03 — Cache reconciliation table + module + new public APIs + audit hook + Tainted type assertion](./t03.md)
  `cache_reconciliation` schema; 3 new `Cache` APIs + `log_attach_walk`; `reconciliation.rs` walker (≤ 250 lines); 4 unit tests incl. DVCS-ATTACH-04 type assertion; serial build + per-phase push.
