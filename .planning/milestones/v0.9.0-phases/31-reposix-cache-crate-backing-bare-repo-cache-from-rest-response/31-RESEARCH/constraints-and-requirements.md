← [back to index](./index.md)

# User Constraints, Phase Requirements, and Architectural Responsibility Map

## User Constraints (from CONTEXT.md)

### Locked Decisions

**Operating-principle hooks (non-negotiable — per project CLAUDE.md):**

- **Audit log non-optional (OP-3).** Every blob materialization writes one row to the `audit` table in `cache.db` (SQLite WAL, BEFORE UPDATE/DELETE RAISE trigger). Columns: `ts`, `op` (`materialize` | `egress_denied` | `tree_sync`), `backend`, `project`, `issue_id`, `oid`, `bytes`. Missing audit rows = feature is not done.
- **Tainted-by-default (OP-2).** Cache returns blob bytes wrapped in `reposix_core::Tainted<Vec<u8>>`. A compile-fail test asserts that calling any egress-side-effect function on `Tainted` without `sanitize()` is a type error. Mirrors the policy used by FUSE writes pre-v0.9.0 — the type system encodes the trust boundary so reviewers cannot accidentally introduce a trifecta.
- **Egress allowlist (security guardrail).** The cache constructs zero new `reqwest::Client` instances. All HTTP goes through `reposix_core::http::client()` which honours `REPOSIX_ALLOWED_ORIGINS`. A denied origin returns `Error::EgressDenied` AND writes an audit row with `op=egress_denied`. `clippy::disallowed_methods` in `clippy.toml` enforces this at lint time.
- **Simulator-first (OP-1).** Every test in this crate runs against `SimBackend`. No real-backend test in this phase (real backends land in Phase 35+).
- **No hidden state (OP-4).** The cache path is deterministic from `(backend, project)`. No session-local directories, no `/tmp` fallbacks.

**Cache construction strategy:**

- **Tree sync = full.** Cache calls `BackendConnector::list_issues()` once per sync, builds a tree object listing every `<bucket>/<id>.md` path, writes as a single commit. Tree metadata is cheap (≤500KB for 10k issues).
- **Blob materialization = lazy.** Blobs are NOT fetched during `list_issues`. They are fetched only on demand via `Cache::read_blob(oid)`, which looks up the OID → issue_id mapping, calls `BackendConnector::get_issue(issue_id)`, writes the blob to `.git/objects`, returns bytes.
- **Commit message format:** `sync(<backend>:<project>): <N> issues at <ISO8601>` — parseable, auditable.
- **Refspec namespace** (referenced here for Phase 32 consumer): cache publishes `refs/heads/main` internally, but the helper will map to `refs/reposix/*` per ARCH-05.

**Atomicity:**

- Preferred ordering for sync: bare-cache write first, then update cache DB's `last_fetched_at`. If REST fetch succeeds but cache write fails, `last_fetched_at` stays old and the next sync retries. If cache write succeeds but timestamp update fails, double-fetch but no data loss.
- For READ (blob materialization), atomicity is simpler: write blob → return bytes → write audit row. Audit row failure logs WARN but still returns bytes (audit failure must not poison the user flow, but should be visible).
- Full atomic reconciler deferred to Phase 33 (delta sync).

### Claude's Discretion

All other implementation details — specifically: exact crate module layout, choice of `gix` vs `git2` for git object writes (gix preferred per Rust-only build policy, but `git2` acceptable if gix coverage is insufficient), SQLite schema for the `meta` table beyond the `last_fetched_at` single-row requirement.

### Specific Ideas

- Cache path default is `$XDG_CACHE_HOME/reposix/<backend>-<project>.git`, overridable via `REPOSIX_CACHE_DIR`.
- The `meta` table schema: `key TEXT PRIMARY KEY, value TEXT, updated_at TEXT NOT NULL`. `last_fetched_at` is one row.
- A single synthesis commit per sync is acceptable for v0.9.0. Multi-commit histories deferred.
- Bare repo is initialised with `extensions.partialClone = origin` and the reposix-cache is the promisor.

### Deferred Ideas (OUT OF SCOPE)

- Multi-commit history (one commit per backend update timestamp) — v0.10.0 + observability.
- Cache eviction policy (LRU, TTL, quota).
- Full atomic rollback for REST-write + cache-update — deferred to Phase 33/34.
- `reposix gc` subcommand for manual eviction — v0.10.0.

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| ARCH-01 | New crate `crates/reposix-cache/` constructs a bare git repo from REST responses via `BackendConnector`. Tree fully populated (filenames, blob OIDs); blobs lazy. | "What you need to know" §1 (gix 0.82 has all required primitives), §3 (tree construction), §4 (lazy materialization OID map) |
| ARCH-02 | One audit row per blob materialization. Cache returns `Tainted<Vec<u8>>`. Append-only schema (BEFORE UPDATE/DELETE RAISE). | "What you need to know" §5 (SQLite schema), §6 (Tainted compile-fail via trybuild) |
| ARCH-03 | Reuses `reposix_core::http::client()`. Zero new `reqwest::Client`. Denied origin → `Error::EgressDenied` + audit row `op=egress_denied`. clippy `disallowed_methods` enforces. | "What you need to know" §7 (existing factory + clippy.toml already configured) |

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| REST → typed `Issue` | `reposix-core::backend::sim` (and other backends) | — | Already shipped; this phase consumes the trait |
| Issue → on-disk markdown bytes | `reposix-core::issue::frontmatter::render` | — | Already shipped; pure function with no I/O |
| `Vec<Issue>` → bare-repo tree+commit | `reposix-cache::builder` (NEW) | `gix` 0.82 (pure Rust git impl) | The new substrate; no existing component does this |
| OID → issue_id → REST → blob bytes | `reposix-cache::builder::read_blob` (NEW) | `gix::Repository::write_blob` | Lazy materialization is the cache's defining behaviour |
| Append-only audit | `reposix-cache::audit` (NEW, lifts `reposix_core::audit` pattern) | `rusqlite` 0.32 bundled, BEFORE UPDATE/DELETE triggers | Mirror existing simulator audit table; cache-specific schema |
| Trust boundary | `reposix_core::Tainted<Vec<u8>>` (existing) + new compile-fail fixture | `trybuild` 1.0.116 | The type already exists; this phase wires it to a new sink |
| Egress allowlist | `reposix_core::http::client()` (existing) | `clippy::disallowed_methods` (existing `clippy.toml`) | Single legal HTTP construction site, lint-enforced |
| OID ↔ issue_id mapping | `reposix-cache::builder` SQLite sidecar | `rusqlite` 0.32 | Required because git's blob OIDs are content-addressed and don't carry issue IDs |
