# Common Pitfalls

← [back to index](./index.md)

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
