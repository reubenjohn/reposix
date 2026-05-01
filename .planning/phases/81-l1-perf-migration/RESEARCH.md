# Phase 81: L1 perf migration — `list_changed_since`-based conflict detection — Research

**Researched:** 2026-05-01
**Domain:** git remote helper conflict detection; cache delta sync; CLI surface extension
**Confidence:** HIGH for trait + cache state inventory; MEDIUM for net algorithm shape (one architectural surface needs planner decision — see §3 "delete detection")

## Summary

Phase 81 replaces the unconditional `state.backend.list_records(...)` call in `handle_export` (currently around line 334–348 of `crates/reposix-remote/src/main.rs` — POST P80 the line numbers shifted; the call site is unchanged in shape but accompanied by P80's mirror-ref writes after acceptance). The substrate is already in place:

- **`BackendConnector::list_changed_since(project, since) -> Vec<RecordId>`** exists on the trait with a default impl and is overridden by all four shipped backends (`sim`, `confluence`, `github`, `jira`) using native incremental queries (`?since=`, CQL `lastModified > "..."`, JQL `updated >= "..."`).
- **`Cache::sync()`** in `crates/reposix-cache/src/builder.rs` already implements the L1 algorithm end-to-end against its OWN cursor (`meta.last_fetched_at`): read cursor → `list_changed_since` → eager-materialize changed blobs → rebuild full tree → atomic SQL transaction. Phase 81's helper-side precheck reuses the same surfaces.
- **`meta.last_fetched_at`** is already the canonical cursor row in `cache.db`. P80's `refs/mirrors/<sot>-synced-at` is a SEPARATE timestamp (last successful OUTBOUND mirror sync) and must not be conflated with the INBOUND SoT-sync cursor used here.

**Primary recommendation:** Single plan, four tasks (Task 1: catalog rows; Task 2: helper precheck rewrite + delete-detection seam decision; Task 3: `reposix sync --reconcile` subcommand; Task 4: N=200 perf regression test + CLAUDE.md doc update + L2/L3 inline comment). The one decision the planner cannot punt on is the **delete-detection seam** in §3 — `list_changed_since` returns IDs of changed records but does NOT signal backend-side deletions on Confluence (nothing to find via `lastModified > x`). The recommended path is "L1 trusts the cache for the prior set; deletes are detected the way today's `plan()` does, but against `cache.list_record_ids()` instead of a freshly-fetched `list_records()`." Documented inline as a known L2/L3-hardening surface.

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

## Common Pitfalls

### Pitfall 1: First-ever push (no `last_fetched_at` row yet)

**What goes wrong:** Cursor is `None`. `list_changed_since(None)` is not a valid call.

**Recommended handling:** Treat absent cursor as "fall back to today's behavior on this push only" — call `list_records` once, populate `meta.last_fetched_at` to `Utc::now()` after success, subsequent pushes hit the L1 fast path. This mirrors `Cache::sync`'s seed-vs-delta branch (`builder.rs:239-249`). The first-push cost is unchanged; every subsequent push is fast.

**Alternative considered + rejected:** treat `None` as `epoch (1970-01-01)`. Rejected because for any non-tiny backend, `list_changed_since(epoch)` returns the entire dataset and the call is paginated identically to `list_records` — no win. Confluence CQL `lastModified > "1970-01-01 00:00"` against a TokenWorld-sized space would still take 100 calls.

### Pitfall 2: Clock skew between agent machine and backend

**What goes wrong:** Agent machine clock is 30s behind real time; agent pushes at agent-time `T0`; backend records the write at backend-time `T0 + 30s`. We write `last_fetched_at = Utc::now() = T0` on agent. Next push, `list_changed_since(T0)` returns the just-pushed record because backend-time `T0 + 30s > T0`. False positive: we'll re-check our OWN write.

**Recommended handling:** This is a classical "use the server's clock for the cursor" problem. Two options: (a) use the SoT's `Record.updated_at` from the response to the just-completed write — the cursor becomes `max(updated_at) over just-written records, plus epsilon`; (b) accept the false-positive on the cursor's first-tick after a push as a known L1 quirk — `list_changed_since` returns a small set, and `find_oid_for_record` against the just-updated cache will report version-equal (we just synced our own write into the cache via the post-write step), so the precheck passes.

**Recommendation:** option (b). It's simpler and self-healing. Document inline.

### Pitfall 3: `list_changed_since` does not signal backend-side deletes

**What goes wrong (the load-bearing L1 caveat):** Confluence CQL `lastModified > X` returns CHANGED pages. A page someone DELETED on Confluence does not appear in the result — it's gone. Current `handle_export` calls `list_records` and detects deletes by *absence* (a record in cache but NOT in the fresh list = deleted on backend → would conflict if the agent tries to update it). With L1 we no longer have the fresh list. Two paths:

- **L1-strict (recommended):** Trust the cache for the prior set. `plan()` is called with `prior = cache.list_record_ids()` materialized into `Record` objects via `cache.find_oid_for_record + read_blob + frontmatter::parse`. Backend-side deletes are NOT detected on the precheck — the agent's PATCH against a deleted Confluence page will fail at REST time with a 404, surfaced as a normal write error. **Trade-off documented in CLAUDE.md OP + helper code comment.**
- **L1-paranoid (NOT recommended):** Periodically (every Nth push, every Nth minute) do a full `list_records` walk. Defeats the purpose of L1.

**Recommendation:** L1-strict. The "agent edits a record someone else deleted on backend" race is rare (the lethal-trifecta cases that motivate `Tainted` are the high-stakes risks; this race is "user-visible but recoverable — REST 404 is a clean error"). L2/L3 (v0.14.0) will harden this via either a background reconcile (L2) or transactional cache writes (L3). Document the trade-off via:

  1. Inline comment in `handle_export` near the new precheck (per success criterion 5).
  2. CLAUDE.md addition under § Architecture or § Threat model noting the L1 trade and the `reposix sync --reconcile` escape hatch.
  3. `docs/concepts/dvcs-topology.md` (P85, not P81) gets the user-facing explanation.

### Pitfall 4: Cache write fails between successful REST writes and cursor update

**What goes wrong:** REST writes succeed; `cache.write_last_fetched_at` fails (disk full, mutex poisoned). On next push, cursor is stale → `list_changed_since(stale_cursor)` returns the records we just wrote → false-positive conflict UNLESS the cache also failed to update its `oid_map`/blob state for those records.

**Recommendation:** Same as today — best-effort cache writes don't poison the push acknowledgement. The cache and backend can drift; `reposix sync --reconcile` is the recovery path. P80 already established this pattern for mirror-ref writes (lines 514–520 of `main.rs`); the new `last_fetched_at` write follows it.

### Pitfall 5: `parse_export_stream` blobs vs cache prior version

**What goes wrong:** The `prior` slice is currently `Vec<Record>` (rich shape with version + updated_at). After L1, the prior comes from cache. The cache's blob is the rendered frontmatter at last sync — `frontmatter::parse(blob_bytes)` recovers the `version` and other fields. This works but adds a parse step per checked record. For typical pushes (1–5 records changed), cost is negligible.

**Recommendation:** Lazy: only parse the prior blob if the record is in the changed set AND the agent is pushing it. The hot path (record not in changed set) skips the parse entirely.

### Pitfall 6: `Cache::sync` is async; helper sits in a sync `block_on`

**What goes wrong:** Borrowing `Cache::sync`'s code path directly (e.g., to share the cursor + atomic-transaction shape) would require holding the runtime through async calls. This is fine — `handle_export` already calls `state.rt.block_on(state.backend.list_records(...))` (line 335).

**Recommendation:** Same idiom for `list_changed_since`: `state.rt.block_on(state.backend.list_changed_since(&state.project, since))`.

### Pitfall 7: Tainted bytes still pass through the prior-blob read

**What goes wrong:** Cache's `read_blob` returns `Tainted<Vec<u8>>` per OP-2. The conflict check parses the blob to extract `version`. Parsing tainted bytes is fine (no IO side effects), but care is needed not to leak the tainted body into a log line or hint.

**Recommendation:** Use `Tainted::inner_ref()` just for the version field; never echo body bytes to stderr. Existing audit logging in `log_helper_push_rejected_conflict` only takes record id + versions — same shape.

**Important caveat (added P81 plan-check 2026-05-01):** the cache's `read_blob` is `async` AND fetches from the backend on demand when the blob isn't materialized (`crates/reposix-cache/src/builder.rs:442-470`). Calling it from the precheck would defeat the L1 perf goal by adding a backend GET per cache prior. T02 introduces a new sync gix-only primitive `Cache::read_blob_cached(oid) -> Result<Option<Tainted<Vec<u8>>>>` that returns `Ok(None)` on cache miss instead of fetching. The precheck uses `read_blob_cached`; cache misses fall through to the no-conflict path (the cache's blob will be fetched on demand later via the existing `read_blob` async path during execute).

## Code Examples

### Example 1: New precheck loop (replacement for lines 334–382 of `main.rs`)

```rust
// Source: synthesized from existing handle_export pattern + Cache::sync flow.
// Step 1: read cursor.
let since_opt: Option<chrono::DateTime<chrono::Utc>> = state.cache.as_ref()
    .and_then(|c| c.read_last_fetched_at().ok().flatten());

// Step 2: if no cursor, fall back to today's full walk for THIS push only.
let changed_ids: Vec<reposix_core::RecordId> = match since_opt {
    Some(since) => match state.rt.block_on(
        state.backend.list_changed_since(&state.project, since)
    ) {
        Ok(v) => v,
        Err(e) => return fail_push(proto, state, "backend-unreachable",
            &format!("list_changed_since failed: {e:#}")).map_err(Into::into),
    },
    None => {
        // First-push fallback. Surfaced via tracing::info — single line, not a hot
        // path at scale.
        tracing::info!("no last_fetched_at cursor; running full list_records (first push)");
        let prior = state.rt.block_on(state.backend.list_records(&state.project))
            .map_err(/* same error path */)?;
        prior.iter().map(|r| r.id).collect()
    }
};

// Step 3: build conflict set. Only records in changed_ids AND in our push.
let changed_set: std::collections::HashSet<_> = changed_ids.iter().copied().collect();
let mut conflicts: Vec<(reposix_core::RecordId, u64, u64, String)> = Vec::new();
for (path, mark) in &parsed.tree {
    let Some(id_num) = issue_id_from_path(path) else { continue; };
    let id = reposix_core::RecordId(id_num);
    if !changed_set.contains(&id) { continue; }   // hot-path bail; no parse
    let Some(cache) = state.cache.as_ref() else { continue; };  // no cache → can't compare
    let Some(prior_oid) = cache.find_oid_for_record(id)? else { continue; };  // record new in cache
    let prior_bytes = cache.read_blob(prior_oid)?;  // Tainted<Vec<u8>>
    let prior_text = String::from_utf8_lossy(prior_bytes.inner_ref());
    let Ok(prior_record) = reposix_core::frontmatter::parse(&prior_text) else { continue; };
    // Re-fetch the now-current backend version to surface in the error message.
    // ONE GET per actually-conflicting record — bounded by changed_set ∩ push_set,
    // typically zero or one. NOT a list call.
    let backend_now = state.rt.block_on(state.backend.get_record(&state.project, id))
        .map_err(/* ... */)?;
    let Some(blob_bytes) = parsed.blobs.get(mark) else { continue; };
    let new_text = String::from_utf8_lossy(blob_bytes);
    let Ok(new_record) = reposix_core::frontmatter::parse(&new_text) else { continue; };
    if new_record.version != backend_now.version {
        conflicts.push((id, new_record.version, backend_now.version,
                        backend_now.updated_at.to_rfc3339()));
    }
}
// Step 4: same reject path as today (lines 384–427 unchanged).

// Step 5: plan() now takes prior derived from cache, NOT a fresh REST list.
let prior: Vec<reposix_core::Record> = state.cache.as_ref()
    .map(|c| c.list_record_ids())
    .transpose()?
    .unwrap_or_default()
    .into_iter()
    .filter_map(|id| /* read blob, parse — see helper fn */)
    .collect();
let actions = match plan(&prior, &parsed) { /* same as today */ };

// Step 6: after successful execute, update cursor.
if !any_failure {
    if let Some(cache) = state.cache.as_ref() {
        let _ = cache.write_last_fetched_at(chrono::Utc::now());
        // ... existing P80 mirror-ref writes follow
    }
}
```

### Example 2: New CLI subcommand

```rust
// crates/reposix-cli/src/main.rs — add to enum Cmd:
/// On-demand cache reconciliation against the SoT (escape hatch for L1
/// cache-desync per `architecture-sketch.md` § "Performance subtlety").
///
/// Without --reconcile, this is a no-op stub that prints a hint pointing at
/// `--reconcile` (a v0.13.0 contract the bus remote leans on).
///
/// Examples:
///   reposix sync --reconcile               # full list_records walk + cache rebuild
///   reposix sync --reconcile /tmp/repo
Sync {
    /// Force a full list_records walk + cache rebuild (drops L1 cursor; the
    /// next push behaves like a first-push).
    #[arg(long)]
    reconcile: bool,
    /// Working-tree directory. Defaults to cwd.
    path: Option<PathBuf>,
}
```

The handler in `crates/reposix-cli/src/sync.rs` (NEW, ~30 lines): resolve cache from working tree, call `cache.build_from().await` (already exists; does the full list_records + tree rebuild + last_fetched_at upsert), print a one-line summary.

## Catalog Row Design (catalog-first per QG-06)

Three rows mint BEFORE the helper edit lands:

### Row 1 — `perf-targets/handle-export-list-call-count`
**Dimension:** `perf` (existing `quality/catalogs/perf-targets.json`)
**Cadence:** `pre-pr` (NOT weekly — this is a regression test, not a benchmark)
**Kind:** `mechanical`
**Sources:** `crates/reposix-remote/src/main.rs::handle_export`, `crates/reposix-remote/tests/perf_l1.rs`
**Verifier:** new shell script `quality/gates/perf/list-call-count.sh` that runs `cargo test -p reposix-remote --test perf_l1 -- --include-ignored` and asserts exit 0.
**Asserts:** "with N=200 records seeded in the sim and a one-record edit pushed, the precheck makes ≤1 `list_changed_since` REST call AND zero `list_records` REST calls (modulo the one-time first-push fallback in test setup)."

### Row 2 — `agent-ux/sync-reconcile-subcommand`
**Dimension:** `agent-ux` (existing `quality/catalogs/agent-ux.json`)
**Cadence:** `pre-pr`
**Kind:** `mechanical`
**Sources:** `crates/reposix-cli/src/main.rs`, `crates/reposix-cli/src/sync.rs`, `crates/reposix-cli/tests/sync.rs`
**Verifier:** `cargo run -p reposix-cli -- sync --reconcile --help` exits 0 AND a smoke test in `tests/sync.rs` runs `reposix sync --reconcile` against the sim and asserts the cache was rebuilt (e.g., `last_fetched_at` advanced).

### Row 3 — `docs-alignment/perf-subtlety-prose-bound`
**Dimension:** `docs-alignment` (existing catalog)
**Sources:** `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § "Performance subtlety: today's `list_records` walk on every push" — the prose paragraph asserting "L1 trades one safety property: today's `list_records` would catch a record that exists on backend but is missing from cache" — bound to `crates/reposix-remote/tests/perf_l1.rs::l1_does_not_call_list_records`.
**Use the existing `bind` verb** in `reposix-quality doc-alignment bind`. No new verifier script (the test IS the verifier).

## Test Fixture Strategy

The `crates/reposix-sim` simulator does not currently expose REST-call counters. **Use wiremock instead** — same approach as the P73 connector contract tests. Pattern:

```rust
// crates/reposix-remote/tests/perf_l1.rs (NEW)
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate, Request};

#[tokio::test]
async fn l1_precheck_uses_list_changed_since_not_list_records() {
    let server = MockServer::start().await;
    let project = "demo";

    // Seed 200 records via the sim's existing JSON shape.
    let records: Vec<serde_json::Value> = (1..=200).map(seeded_record).collect();

    // Mock list_records — assert NOT called.
    Mock::given(method("GET"))
        .and(path(format!("/projects/{project}/issues")))
        .and(no_since_query())  // helper: query_param_exists is wiremock 0.6+; we want
                                // "no `since` query param" → list_records, not the delta.
        .respond_with(ResponseTemplate::new(200).set_body_json(&records))
        .expect(0)              // CRITICAL: zero list_records calls on success path.
        .mount(&server).await;

    // Mock list_changed_since — assert called exactly once.
    Mock::given(method("GET"))
        .and(path(format!("/projects/{project}/issues")))
        .and(query_param_exists("since"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&serde_json::json!([])))
        .expect(1)              // Empty result → no per-record GET, no conflict.
        .mount(&server).await;

    // Drive the helper: env var override origin → server.uri(); export verb;
    // single-record edit fast-import stream on stdin.
    drive_export_verb(&server.uri(), project, single_record_edit()).await;

    // wiremock asserts via Drop: panics if expectations unmet.
}
```

The "no_since" matcher is a closure: `Mock::given(...).and(|r: &Request| !r.url.query_pairs().any(|(k, _)| k == "since"))`. wiremock 0.6 supports custom matchers via `Match` trait.

The N=200 figure comes from architecture-sketch (5,000-record / page-50 = 100 calls; a 200-record / page-50 simulation = 4 paginated calls — enough to make the difference observable while keeping the test sub-second). Confirm sim's page size in P81 Task 1; if sim doesn't paginate at 50, scale N up so the assertion `expect(0)` is meaningful.

## Plan Splitting

**Recommended: SINGLE PLAN with 4 tasks.**

| Task | Goal | Cargo-heavy? |
|------|------|--------------|
| **Task 1** | Catalog rows + new `Cache::read_last_fetched_at` / `write_last_fetched_at` public methods + cli `Sync { reconcile }` subcommand stub | Yes — `cargo check -p reposix-cache -p reposix-cli` |
| **Task 2** | Helper precheck rewrite (replace lines 334–382) + `plan()` called against cache-derived prior + cursor write after success + L2/L3 inline comment + first-push fallback | Yes — `cargo check -p reposix-remote`; `cargo test -p reposix-remote` for existing conflict-detection tests |
| **Task 3** | `reposix sync --reconcile` handler in `crates/reposix-cli/src/sync.rs` + smoke test | Yes — `cargo test -p reposix-cli --test sync` |
| **Task 4** | `crates/reposix-remote/tests/perf_l1.rs` (N=200 wiremock-counted regression) + CLAUDE.md update + README.md (if Commands section names `reposix sync --reconcile`) + verdict push | Yes — `cargo test -p reposix-remote --test perf_l1` |

4 cargo-heavy tasks fits inside the CLAUDE.md "≤4 cargo-heavy tasks per plan" guideline. Sequential per CLAUDE.md "Build memory budget"; per-crate `cargo check`/`test` invocations only.

If any task balloons (e.g., Task 2 reveals a hidden coupling between `plan()` and the `&[Record]` shape that requires a wider refactor), surface as a SURPRISES-INTAKE candidate per OP-8 rather than expanding the phase.

## Pitfalls and Risks (consolidated)

(See § Common Pitfalls above for detailed analysis.)

| Risk | Severity | Mitigation |
|------|----------|------------|
| Backend-side deletes silently miss precheck | MEDIUM (user-visible at REST time as 404) | Document the L1 trade-off inline + in CLAUDE.md; user recovery via `reposix sync --reconcile`. |
| Clock skew false-positives | LOW | Self-healing on next push; document as known quirk. |
| First-push has no cursor → falls back to `list_records` | LOW (intentional) | Already in algo; one-time cost. |
| Cache write fails after REST write succeeds | LOW (P80 already establishes "best-effort cache writes don't poison ack" pattern) | Same pattern; warn-log; user recovers via `reposix sync --reconcile`. |
| `wiremock::Match::expect(0)` doesn't actually fail RED if list_records is called | MEDIUM (test could silently pass) | Verify wiremock semantics during Task 4; add a positive control (a SECOND test that DOES expect a list_records call to ensure the matcher works). |
| Tainted prior-blob bytes leak into log lines | LOW-MEDIUM (OP-2 violation) | Existing `log_helper_push_rejected_conflict` only takes `id + versions`; preserve that shape; never log blob body. |

## Documentation Deferrals

- **`docs/concepts/dvcs-topology.md`** — P85, NOT P81. P85 will write the full L2/L3 deferral note + user-facing "what `--reconcile` does" prose.
- **P81 documents inline:**
  - One comment block in `crates/reposix-remote/src/main.rs` near the new precheck citing `architecture-sketch.md` § "Performance subtlety" + the v0.14.0 hardening doc (per success criterion 5).
  - Two CLAUDE.md additions: (1) § Commands gets `reposix sync --reconcile` documented under the "Local dev loop" block; (2) § Architecture (or a new sub-section) names the L1 trade-off in 1–2 sentences and points at `architecture-sketch.md`.

**Confirmed:** No new docs/site pages in P81. Doc-alignment row binds the existing architecture-sketch prose to the regression test; no fresh prose is authored.

## Sources

### Primary (HIGH confidence)
- `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § "Performance subtlety: today's `list_records` walk on every push" + § "3. Bus remote with cheap-precheck + SoT-first-write".
- `.planning/research/v0.13.0-dvcs/decisions.md` § Q3.1 (RATIFIED inline L1).
- `crates/reposix-core/src/backend.rs:215–264` (BackendConnector trait + `list_changed_since` default + signature `Vec<RecordId>`).
- `crates/reposix-confluence/src/lib.rs:141–`, `crates/reposix-github/src/lib.rs:472–`, `crates/reposix-jira/src/lib.rs:108–`, `crates/reposix-core/src/backend/sim.rs:281–303` — all four backends override `list_changed_since` with native incremental queries.
- `crates/reposix-cache/src/builder.rs::sync` (lines 220–end) — full L1 reference impl already in cache crate.
- `crates/reposix-cache/src/meta.rs` — `set_meta`/`get_meta` API + `last_fetched_at` is already a known cursor key (see `delta_sync.rs` tests).
- `crates/reposix-cache/src/cache.rs::list_record_ids` (line 345) + `find_oid_for_record` (line 381) — read-side primitives for cache-derived prior.
- `crates/reposix-remote/src/main.rs::handle_export` (lines 299–550 post-P80) — current call site shape.
- `crates/reposix-remote/src/diff.rs::plan` (line 99) — existing `prior: &[Record]` signature.
- `crates/reposix-cli/src/main.rs:37–326` — Cmd enum; confirms NO existing `Sync` subcommand.
- `quality/catalogs/perf-targets.json` + `quality/catalogs/agent-ux.json` — existing dimension homes for new rows.

### Secondary (verified via cross-reference)
- P80 mirror-ref ship pattern: `crates/reposix-cache/src/mirror_refs.rs` (`write_mirror_synced_at`, `read_mirror_synced_at`) — the public-API shape the new `read_last_fetched_at`/`write_last_fetched_at` should mirror.
- P79 attach precedent: `crates/reposix-cli/src/attach.rs` — clap subcommand structure for the new `Sync` subcommand.
- P73 wiremock idiom: `crates/reposix-confluence/tests/auth_header.rs::auth_header_basic_byte_exact` — pattern for byte-exact REST-call matchers in tests.

## Metadata

**Confidence breakdown:**
- Trait + cache substrate inventory: HIGH — code paths verified via grep + read.
- New algorithm shape: HIGH for the happy path; MEDIUM for the delete-detection seam (see §3 Pitfall 3 — planner must ratify "L1-strict" trade-off explicitly).
- Catalog row design: HIGH — existing dimension homes + verifier shapes are well-precedented.
- Test fixture strategy: MEDIUM — wiremock `expect(0)` semantics need confirmation in Task 4 (positive-control test recommended).
- L2/L3 deferral: HIGH — explicit in `decisions.md` Q3.1 and architecture-sketch.

**Research date:** 2026-05-01
**Valid until:** 2026-05-30 (30 days; substrate is stable Rust code with workspace-pinned deps).

## Open Questions for the Planner

1. **Delete-detection seam (Pitfall 3).** Confirm L1-strict (cache-trusted prior, REST 404 on backend-deleted record) is the chosen path. The architecture-sketch already says yes; this research confirms no surprise blockers in the code; the planner just needs to make the trade explicit in PLAN.md so the executing subagent and the verifier subagent both grade against the same contract.

2. **`Sync` subcommand vs `Refresh --reconcile`.** Architecture-sketch and ROADMAP use `reposix sync --reconcile`. The CLI today has `reposix refresh` (no `sync`). Two paths: (a) NEW `Sync { reconcile }` subcommand — recommended; (b) ADD `--reconcile` flag to existing `Refresh`. Recommend (a) — `refresh` writes `.md` files into a working tree (different from cache rebuild); conflating the two would muddy CLI semantics. (a) is also what the docs tell users to type.

3. **Should `plan()` signature change to `Vec<RecordId>` for the prior set?** Today `plan()` consumes `&[Record]` to compute deletes (records-in-prior-not-in-new). With L1, the prior shape is "IDs the cache knows about + lazily-fetched bodies." Two paths: (a) keep `plan()` signature; helper materializes a `Vec<Record>` from cache before calling — extra parses but minimal change to `diff.rs`; (b) change `plan()` to take `Vec<RecordId>` for the prior-id set + a closure to fetch the prior `Record` on demand — cleaner but wider blast radius. Recommend (a) — change is local to `handle_export`, `diff.rs` test surface untouched.

4. **Catalog row: `perf-targets.json` vs new `dvcs-perf.json`?** Recommend reuse `perf-targets.json` — three existing rows, well-precedented, freshly de-WAIVED in v0.12.1 P63. New file would orphan one row in its own catalog and complicate runner discovery.

5. **CLAUDE.md update scope.** Two paragraphs (one in § Commands, one in § Architecture or a new "L1 conflict detection" sub-section under § Architecture). Confirm the verifier-subagent grade rubric expects CLAUDE.md to land in the same PR per QG-07 (yes — checked).
