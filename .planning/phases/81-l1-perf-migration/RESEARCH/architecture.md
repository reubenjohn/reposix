# Architecture

← [back to index](./index.md)

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Push-time conflict detection | helper (`reposix-remote`) | cache (read prior tree state) | The helper owns the protocol turn; the cache owns the prior. |
| Cursor read/write (`last_fetched_at`) | cache (`reposix-cache::meta`) | helper (callsite reads cursor before precheck) | Cursor is durable cache state — it survives helper restarts. |
| `list_changed_since` REST call | core trait (`BackendConnector`) | per-connector overrides | Trait already in place; overrides already shipped. |
| `reposix sync --reconcile` | cli (`reposix-cli`) | cache (`build_from`) | `Cache::build_from` already does the full walk; the CLI is a thin wrapper. |
| L2/L3 hardening (background reconcile / transactional cache writes) | DEFERRED to v0.14.0 | — | Per `decisions.md` Q3.1; out of scope. |

## Standard Stack

### Core (already in place — no new deps)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `chrono` | 0.4.x (workspace pin) | `DateTime<Utc>` for `last_fetched_at` cursor | Project-wide convention per CLAUDE.md "Tech stack". |
| `rusqlite` | 0.32 (bundled) | `meta` table reads/writes | Already wired; transaction boundary in `Cache::sync` is the model. |
| `gix` | 0.83 | Tree walks for "find prior version of record N" | Used by P79 attach reconciliation; same lookup pattern. |
| `wiremock` | latest workspace | REST-call counting in N=200 perf test | Already used in P73 connector contract tests + P79 attach tests. |
| `tokio` | 1.x | runtime hop in `state.rt.block_on(...)` | Existing pattern in `handle_export`. |

### Don't add

| Tempting addition | Why NOT |
|-------------------|---------|
| New `cache_state` table | `meta` already does this — reuse `meta::set_meta`/`get_meta` with key `"last_fetched_at"`. |
| `criterion` benchmark crate | Phase budget is doc-light + 4 tasks; criterion-shaped microbench is overkill. A cargo-test-driven N=200 wiremock count assertion is the catalog row's preferred shape (deterministic, fast, CI-cheap). |

**Installation:** none — fully in-tree.

**Version verification (skipped):** all dependencies already pinned in workspace `Cargo.toml`; phase adds zero new deps.

## Architecture Patterns

### System Architecture Diagram (push-time precheck flow, post-L1)

```
git push
  │
  ▼
git-remote-reposix (export verb)
  │
  ▼
[NEW] read cursor: meta.last_fetched_at
  │
  ▼ since
backend.list_changed_since(project, since)  ◄── ONE REST call (paginated only if backend
  │                                              actually has many recent changes; the
  │                                              hot path is "empty result, push-OK").
  │
  ▼ Vec<RecordId>
For each pushed record (read fast-import stream):
  ├── id ∈ changed_ids? ──── yes ──► open prior version via cache.find_oid_for_record + read_blob
  │                                  parse prior frontmatter → backend version
  │                                  parse new frontmatter → local version
  │                                  if local != prior → CONFLICT (reject with id + versions)
  │
  └── id ∉ changed_ids? ──── push proceeds for this record (cache prior is trusted)
  │
  ▼ no conflicts → plan() against cache.list_record_ids() (NOT fresh list_records)
  │
  ▼ execute REST writes
  │
  ▼ on success: meta.set_meta("last_fetched_at", now()) AND P80 mirror-ref writes
  │
  ▼ ok refs/heads/main
```

### Recommended Project Structure

No new files; edits land in:

```
crates/
├── reposix-remote/src/main.rs          # handle_export precheck rewrite
├── reposix-remote/src/diff.rs          # plan() takes Vec<RecordId> for "prior IDs" (NOT &[Record])
│                                       # OR: plan() unchanged; caller fabricates Vec<Record>
│                                       # from cache (see §3 delete-detection seam)
├── reposix-cli/src/main.rs             # new Sync subcommand with --reconcile flag
├── reposix-cli/src/sync.rs             # NEW — thin wrapper over Cache::build_from
└── reposix-remote/tests/perf_l1.rs     # NEW — N=200 wiremock-counted regression test
```

### Pattern 1: Cursor read at start of `handle_export`

```rust
// Source: mirrors the existing pattern in crates/reposix-cache/src/builder.rs::sync (lines 226–238)
let since: Option<chrono::DateTime<chrono::Utc>> = state.cache.as_ref()
    .and_then(|c| c.read_last_fetched_at().ok().flatten());
```

A new public method `Cache::read_last_fetched_at() -> Result<Option<DateTime<Utc>>>` is
the cleanest seam — the helper currently has no `meta` access — and it parallels the
existing public `read_mirror_synced_at` from P80. Implementation: thin wrapper over
the already-private `meta::get_meta(conn, "last_fetched_at")`.

### Pattern 2: Cursor write after successful push

```rust
// Where today's mirror-ref writes already live (around line 506–520 of main.rs post-P80).
if let Some(cache) = state.cache.as_ref() {
    if let Err(e) = cache.write_last_fetched_at(chrono::Utc::now()) {
        tracing::warn!("write_last_fetched_at failed: {e:#}");
    }
    // ... existing P80 ref writes follow
}
```

A new public `Cache::write_last_fetched_at(ts) -> Result<()>`. Best-effort
(WARN, no poison) — same shape as the P80 `write_mirror_synced_at` that
already lives 14 lines below. Atomicity: the cursor write is a single-row
SQL upsert into `meta`; SQLite's autocommit semantics make it atomic.
Wrapping the cursor + ref writes in a `Connection::transaction` is NOT
required because mirror-ref writes are gix object-DB writes (separate
storage) — the existing P80 code path also does not bracket them.

### Anti-Patterns to Avoid

- **Don't fetch `list_records` "just to be safe."** The point of L1 is to
  drop the unconditional walk. A defensive fallback that calls
  `list_records` when `list_changed_since` returns empty re-introduces
  the cost the phase exists to eliminate.
- **Don't conflate `last_fetched_at` with `refs/mirrors/<sot>-synced-at`.**
  The first measures INBOUND staleness (when did the cache last fetch
  from SoT); the second measures OUTBOUND staleness (when did the GH
  mirror last receive a push). They MOVE TOGETHER on a successful push
  but are conceptually distinct and live in different storage layers
  (`meta` table vs git refs).
- **Don't overload `Cache::sync` for the helper precheck.** `Cache::sync`
  already calls `list_changed_since` AND eagerly materializes blobs AND
  rebuilds the tree. The helper precheck only needs the ID list — it
  does NOT want to materialize blobs (that would re-introduce REST cost
  on the hot path). Use `state.backend.list_changed_since(...)` directly.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Per-record version comparison | New struct that walks tree + parses frontmatter | Existing pattern in `handle_export` lines 355–382 (the loop already there) | The shape is identical; just swap the source of `prior_by_id`. |
| Cursor durability | New table `cache_state` | `meta` table + `meta::set_meta` / `get_meta` | Already exists, already used by `Cache::sync`, already covered by `delta_sync.rs` atomicity tests. |
| REST call counting in tests | Custom HTTP middleware | `wiremock::MockServer::received_requests()` + assertion on path matchers | Pattern shipped in `crates/reposix-confluence/tests/auth_header.rs` (P73). |
| Time computation for cursor | `SystemTime::now()` + manual RFC3339 formatting | `chrono::Utc::now().to_rfc3339()` | Already canonical per `crates/reposix-cache/src/builder.rs:119`. |
