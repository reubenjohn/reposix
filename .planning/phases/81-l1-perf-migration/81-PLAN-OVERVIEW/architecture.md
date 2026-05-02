← [back to index](./index.md)

# Subtle architectural points (read before T02)

The two below are flagged because they are the most likely sources of
T02 review friction. Executor must internalize them before writing
the wiring code.

## S1 — `last_fetched_at` is the INBOUND cursor; `refs/mirrors/<sot>-synced-at` is the OUTBOUND cursor

These two timestamps move together on a successful push but are
conceptually distinct. P80 introduced `refs/mirrors/<sot>-synced-at`
as the OUTBOUND cursor (when did the GH mirror last receive a push from
us). P81 wires the existing INBOUND cursor (`meta.last_fetched_at` —
when did the cache last fetch from SoT) into the helper's precheck
path.

**Why this matters for T02.** A reviewer skimming the wiring may expect
the helper to read `cache.read_mirror_synced_at(&backend_name)` for the
"since" parameter. That would be wrong. `mirror_synced_at` measures
OUTBOUND staleness (last successful mirror push); `list_changed_since`
needs INBOUND staleness (last successful SoT fetch). Read
`cache.read_last_fetched_at()` (NEW in T02) — the wrapper around
`meta::get_meta(conn, "last_fetched_at")` already populated by
`Cache::build_from` and `Cache::sync` (`crates/reposix-cache/src/builder.rs:119`,
`:329` — the existing canonical writers).

P80's `mirror_refs.rs` doc-comment explicitly distinguishes the two; P81
adds a parallel comment in `precheck.rs` so future agents don't conflate
them.

## S2 — First-push fallback (no cursor yet)

The cache's `meta.last_fetched_at` row is populated by the FIRST call to
`Cache::build_from` (during `reposix init` or `reposix attach`). On a
fresh install where the agent runs `reposix init` and then immediately
`git push`, the cursor IS populated — `init` calls `build_from` which
writes `last_fetched_at = Utc::now()` per
`crates/reposix-cache/src/builder.rs:119`. Push 0 already has a cursor.

**However**, there is a real scenario where the cursor is `None`: when
`state.cache` is `Some` but the cache was opened lazily by the helper
(i.e., the user `git clone`'d the cache's bare repo manually OR a
malformed install) and no `build_from` has run. In that case
`read_last_fetched_at()` returns `Ok(None)`.

**Decision (per RESEARCH.md § Pitfall 1):** treat `None` as "fall through
to the existing `list_records` walk for THIS push only; subsequent pushes
hit the L1 fast path." The cost is unchanged for the rare first-push
case; every subsequent push is fast.

**Alternative considered + rejected:** `None` → `epoch (1970-01-01)`.
Rejected because for any non-tiny backend, `list_changed_since(epoch)`
returns the entire dataset and the call is paginated identically to
`list_records` — no win. Confluence CQL `lastModified > "1970-01-01
00:00:00"` against a TokenWorld-sized space would still take 100 calls.

T02's wiring uses an explicit `match` on the cursor and routes to the
existing `list_records` code path (verbatim from current `handle_export`
lines 334–348) when the cursor is absent, then writes the cursor on
success.
