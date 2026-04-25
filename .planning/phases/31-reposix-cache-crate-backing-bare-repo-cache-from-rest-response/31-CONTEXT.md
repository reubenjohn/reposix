# Phase 31: `reposix-cache` crate — backing bare-repo cache from REST responses - Context

**Gathered:** 2026-04-24
**Status:** Ready for planning
**Mode:** Auto-generated (discuss skipped via workflow.skip_discuss=true)

<domain>
## Phase Boundary

Foundation crate for the v0.9.0 architecture pivot. Creates `crates/reposix-cache/` which materializes REST API responses (via existing `BackendConnector` trait from `reposix-core`) into a real on-disk bare git repository. The cache is the substrate that `git-remote-reposix` will proxy protocol-v2 traffic to in Phase 32. No protocol work in this phase — just REST-in, bare-repo-out.

The crate owns: tree construction from issue listings, blob creation from issue content (markdown + YAML frontmatter), lazy blob materialization on demand, SQLite audit + meta DB colocated with the bare repo, and egress allowlist enforcement.

The cache is a normal bare git repo at a path like `$XDG_CACHE_HOME/reposix/<backend>-<project>.git`. Git doesn't know it was synthesized from REST — `git fsck` passes, `git log` shows a synthesis commit per sync.

</domain>

<decisions>
## Implementation Decisions

### Operating-principle hooks (non-negotiable — per project CLAUDE.md)

- **Audit log non-optional (OP-3).** Every blob materialization writes one row to the `audit` table in `cache.db` (SQLite WAL, BEFORE UPDATE/DELETE RAISE trigger). Columns: `ts`, `op` (`materialize` | `egress_denied` | `tree_sync`), `backend`, `project`, `issue_id`, `oid`, `bytes`. Missing audit rows = feature is not done.
- **Tainted-by-default (OP-2).** Cache returns blob bytes wrapped in `reposix_core::Tainted<Vec<u8>>`. A compile-fail test asserts that calling any egress-side-effect function on `Tainted` without `sanitize()` is a type error. Mirrors the policy used by FUSE writes pre-v0.9.0 — the type system encodes the trust boundary so reviewers cannot accidentally introduce a trifecta.
- **Egress allowlist (security guardrail).** The cache constructs zero new `reqwest::Client` instances. All HTTP goes through `reposix_core::http::client()` which honours `REPOSIX_ALLOWED_ORIGINS`. A denied origin returns `Error::EgressDenied` AND writes an audit row with `op=egress_denied`. `clippy::disallowed_methods` in `clippy.toml` enforces this at lint time.
- **Simulator-first (OP-1).** Every test in this crate runs against `SimBackend`. No real-backend test in this phase (real backends land in Phase 35+).
- **No hidden state (OP-4).** The cache path is deterministic from `(backend, project)`. No session-local directories, no `/tmp` fallbacks.

### Cache construction strategy

- **Tree sync = full.** Cache calls `BackendConnector::list_issues()` once per sync, builds a tree object listing every `<bucket>/<id>.md` path, writes as a single commit. Tree metadata is cheap (≤500KB for 10k issues).
- **Blob materialization = lazy.** Blobs are NOT fetched during `list_issues`. They are fetched only on demand via `Cache::read_blob(oid)`, which looks up the OID → issue_id mapping, calls `BackendConnector::get_issue(issue_id)`, writes the blob to `.git/objects`, returns bytes.
- **Commit message format:** `sync(<backend>:<project>): <N> issues at <ISO8601>` — parseable, auditable.
- **Refspec namespace** (referenced here for Phase 32 consumer): cache publishes `refs/heads/main` internally, but the helper will map to `refs/reposix/*` per ARCH-05.

### Atomicity (open question — see architecture-pivot-summary §7 Q2)

- Preferred ordering for sync operations: **bare-cache first, then update cache DB's `last_fetched_at`.** Rationale: if the REST fetch succeeds but the cache write fails, `last_fetched_at` stays old and the next sync retries. If the cache write succeeds but the timestamp update fails, we double-fetch but don't lose data.
- For READ (blob materialization), atomicity is simpler: write blob → return bytes → write audit row. If audit row fails, the operation logs a WARN but still returns bytes (pragmatic — audit failure must not poison the user flow, but should be visible).
- Full atomic reconciler deferred to Phase 33 (delta sync).

### Test surface

- `SimBackend`-seeded fixtures with 1, 10, and 1000 issues.
- `tree_contains_all_issues`: after `Cache::build_from(backend)`, `git ls-tree -r main` contains exactly N entries.
- `blobs_are_lazy`: no `.git/objects/*` entries for issue blobs after tree build (only tree + commit objects).
- `materialize_one`: `Cache::read_blob(oid_for(issue_0))` creates exactly one blob object and writes exactly one `materialize` audit row.
- `egress_denied_logs`: pointing the cache at an origin outside `REPOSIX_ALLOWED_ORIGINS` returns `Error::EgressDenied` and writes an `egress_denied` audit row.
- `audit_is_append_only`: `UPDATE audit SET ts=0 WHERE 1=1` and `DELETE FROM audit` both fail with `SQLITE_CONSTRAINT` (trigger assertion).
- Compile-fail test (`compile_fail.rs` via `trybuild`): `egress::send(cache.read_blob(oid).unwrap())` fails to compile because `Tainted<Vec<u8>>` does not implement `Into<Untainted<Vec<u8>>>`.

### Claude's Discretion

All other implementation details at Claude's discretion — specifically: the exact crate module layout, choice of `gix` vs `git2` for git object writes (gix preferred per Rust-only build policy, but `git2` acceptable if gix coverage is insufficient), SQLite schema for the `meta` table beyond the `last_fetched_at` single-row requirement.

</decisions>

<code_context>
## Existing Code Insights

### Reusable assets

- `reposix_core::BackendConnector` trait (`crates/reposix-core/src/backend.rs`) — consumed by the cache. `list_issues()` and `get_issue(id)` are the two methods this phase depends on.
- `reposix_core::http::client()` factory — the only legal HTTP client constructor in the workspace. Honours `REPOSIX_ALLOWED_ORIGINS`.
- `reposix_core::Tainted<T>` and `Untainted<T>` newtypes — trust boundary encoding (already shipped for FUSE write path).
- Existing `cache_db.rs` in `crates/reposix-cli/` has a `refresh_meta` single-row SQLite table with `last_fetched_at` — this phase formalises a similar table (possibly lifts the code into `reposix-cache`).
- `SimBackend` in `crates/reposix-sim/` — the default test backend.
- Audit-table pattern: see existing simulator's `audit` table + append-only trigger (pre-v0.9.0).

### Established patterns

- Error types: `thiserror` for typed errors in the crate; `anyhow` only at binary boundaries.
- Tests co-located: `#[cfg(test)] mod tests` at the bottom of each src file; integration tests in `tests/`.
- `#![forbid(unsafe_code)]` + `#![warn(clippy::pedantic)]` at every crate root.
- No JSON-on-disk for issues — YAML frontmatter + Markdown body via `serde_yaml`.

### Integration points

- `Cargo.toml` workspace `members` array — add `crates/reposix-cache`.
- Consumers in later phases: `git-remote-reposix` (Phase 32) will construct a `Cache` instance and tunnel git protocol traffic to it.

</code_context>

<specifics>
## Specific Ideas

- The cache path default is `$XDG_CACHE_HOME/reposix/<backend>-<project>.git`, overridable via `REPOSIX_CACHE_DIR`. This is the "no hidden state" principle (OP-4) — deterministic path, no `/tmp` fallback.
- The `meta` table schema: `key TEXT PRIMARY KEY, value TEXT, updated_at TEXT NOT NULL`. `last_fetched_at` is one row. Keep it simple and extensible.
- A single synthesis commit per sync is acceptable for v0.9.0. Multi-commit histories (one commit per backend update) are v0.10.0+ work — deferred.
- Bare repo is initialised with `extensions.partialClone = origin` and the reposix-cache is the promisor (even though agents don't interact with the cache directly — it's for consistency with the partial-clone model).

</specifics>

<deferred>
## Deferred Ideas

- Multi-commit history (one commit per backend update timestamp) — v0.10.0 + observability.
- Cache eviction policy (LRU, TTL, quota) — architecture-pivot-summary §7 Q1. Deferred to when disk quota becomes a real problem.
- Full atomic rollback for REST-write + cache-update (architecture-pivot-summary §7 Q2) — deferred to Phase 33/34 (push + conflict path).
- `reposix gc` subcommand for manual eviction — deferred to v0.10.0 observability/maintenance milestone.

</deferred>
