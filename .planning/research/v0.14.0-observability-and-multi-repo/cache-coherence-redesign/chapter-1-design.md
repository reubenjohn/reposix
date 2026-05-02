← [back to index](./index.md)

# Design — plumbing, layers, cost, and residuals

## What's already plumbed

The trait surface assumes optimistic concurrency from day one. `crates/reposix-core/src/backend.rs:53-75`:

```rust
pub enum BackendFeature {
    StrongVersioning,  // sim, confluence
    // ...
}

pub enum VersioningModel {
    Strong,     // confluence: version.number = current + 1
    Etag,       // HTTP If-Match
    Timestamp,  // best-effort, race window
}
```

`update_record` carries `expected_version: Option<u64>` (`backend.rs:299-304`). Per-backend status today:

| Adapter | Versioning | Plumbed? | Real-backend confirmed? |
|---|---|---|---|
| `reposix-sim` | `Strong` (`If-Match: "<n>"`) | Yes (`crates/reposix-core/src/backend/sim.rs:333`) | n/a (in-process) |
| `reposix-confluence` | `Strong` (`version.number = current + 1`) | Yes (`crates/reposix-confluence/src/lib.rs:343`) | Partial — sim-style assertion only; needs real-Confluence concurrent-edit test |
| `reposix-github` | `Etag` (advertised) | **NO** — see `crates/reposix-github/src/lib.rs:386` `// (no etags plumbed yet)` | n/a |
| `reposix-jira` | none | n/a — `// JIRA has no ETag — expected_version is silently ignored` (`crates/reposix-jira/src/lib.rs:286`) | n/a |

So the trait is correct but implementation is uneven. **Two of four backends honor `expected_version`; one has the wiring missing; one has no etag at all.**

The L1 precheck (`list_changed_since`) was added in v0.13.0 P81 because:
- It was *cheaper* than the legacy `list_records` walk.
- It worked uniformly across all four backends regardless of versioning support.

Reasonable shipping move at the time. But it left the strong-versioning machinery half-used.

---

## The redesign — four layers, very different from L2/L3

### Layer 1 — Conflict detection moves from precheck to write

For backends with `StrongVersioning`:

- Helper sends every PATCH with `If-Match: <cached_version>` (or `version.number = cached + 1`, per backend).
- Backend rejects 412 / 409 on stale cache.
- Helper translates rejection → `error refs/heads/main fetch first` exactly as L1 does today.
- **No precheck REST call.** The conflict check is the write.

For backends without `StrongVersioning` (JIRA today): keep an L1-style precheck OR use timestamp-based `If-Unmodified-Since` semantics. Per-backend strategy lives in Phase A research.

### Layer 2 — Cache becomes a read cache, not a safety oracle

Today's mental model: "the cache must be correct or pushes break." That's what makes desync scary.

New mental model: "the cache is a fast local copy of backend state. Correctness lives in the backend; the cache is a hint."

- Stale reads are tolerated. `cat issues/0042.md` may return slightly stale content. Most agentic workflows are *editing* — the conflict check on write catches stale-version edits. Pure-read drift only matters for human-driven workflows where the agent is summarizing, and even there the staleness is bounded by webhook cadence + `list_changed_since` window.
- `reposix sync --reconcile` stays as the user-explicit "I want a guaranteed-fresh read cache" command. Becomes a read-freshness tool, not a safety mechanism.

### Layer 3 — Webhook-driven cache freshness (already shipped in v0.13.0 P84)

The mirror-sync webhook substrate from v0.13.0 already calls reposix on every backend mutation. Extend the workflow to also advance the cache cursor + materialize the changed records into the cache. Cache stays sub-second-fresh on the happy path.

Already-shipped artifact: `docs/guides/dvcs-mirror-setup-template.yml` runs `cargo binstall reposix-cli` + connector-specific sync command. The cache update is incremental (one or two `get_record` calls per webhook event).

### Layer 4 — `reposix sync --reconcile` as defense-in-depth

Stays. Documented as "force a full read-cache refresh." Not on any hot path. Useful for:
- Recovery after a webhook outage.
- User-paranoid "I want to be sure" before a large bulk operation.
- Debugging when something looks wrong.

### What we don't need

- **Background reconcile job (L2 from v0.13.0's framing).** No periodic full enumeration. Webhook + `list_changed_since` keep the cache fresh enough; conflict detection on write catches the rest.
- **Transactional cache writes (L3 from v0.13.0's framing).** Cache is allowed to be wrong. Backend is the arbiter. Self-heals on next push via 412.

---

## Cost analysis

| Path | L1 (today) | This proposal |
|---|---|---|
| Push precheck | 1 REST call (`list_changed_since`) | **0 REST calls** for `Strong`/`Etag` backends; 1 call for `Timestamp` |
| Push write | N REST calls (one per record in push set) | N REST calls + one HTTP header per call (`If-Match`) |
| Read (`cat issues/X.md`) | 0 REST calls (cache hit) | 0 REST calls (cache hit) |
| Background load | 0 | 0 |
| Webhook overhead | 1 cache update per event | 1 cache update per event (same) |
| User-explicit `--reconcile` | full `list_records` walk | full `list_records` walk (same) |

**Net:** one round-trip saved per push on the hot path. Safety property strictly stronger because the check is on the actual mutation, not a TOCTOU-vulnerable precheck.

Compare to v0.13.0's L2/L3 options:

| Property | L2 (background reconcile) | L3 (transactional cache) | This proposal |
|---|---|---|---|
| Catches user-caused desync (out-of-band backend edits) | Yes, after up to one resync interval | No (only catches adapter-caused desync) | Yes, on next conflicting push |
| Catches adapter-caused desync (cache write failed after backend write) | Yes, after up to one resync interval | Yes, by construction | Yes, on next push (412 → refetch → retry) |
| Per-push cost | L1 + 0 (background) | L1 + transactional overhead | 0 precheck + 1 header per write |
| Engineering scope | scheduling story + audit trail | every adapter | GH etag wiring + JIRA strategy + cache-as-hint reframe |
| Catches catastrophic-but-rare? | Yes (eventually) | Yes (immediately) | Yes (on next push touching the record) |

The proposal lands somewhere between L2 and L3 in catch-immediacy and below both in engineering cost.

---

## Three real residuals — where it isn't fully elegant

### Residual 1 — JIRA has no etag

JIRA's REST API does not expose etags on issues. `expected_version` is silently ignored today. Honest options to surface in Phase A:

**(a) `If-Unmodified-Since` with the issue's `updated` timestamp.** JIRA's `fields.updated` advances on every change. Submit `If-Unmodified-Since: <last_known_updated>`; backend rejects 412 if changed. Race window: writes within the same second can stomp each other (timestamp resolution).

**(b) Read-then-write with `updated` field comparison.** GET the issue immediately before PATCH; compare `updated` timestamp; reject locally if changed. Race window: between the GET and the PATCH (~10–100ms typical).

**(c) Keep L1-style precheck for JIRA only.** `Capability::OptimisticConcurrency::Off` branch in helper. JIRA pays the precheck cost; sim/confluence/GH don't.

Phase A measures the actual race-window incidence on JIRA TEST under deliberate concurrent edit, then picks. Default recommendation: **(a)** — it's standard HTTP and Atlassian explicitly supports `If-Unmodified-Since` per their docs (validate in Phase A).

### Residual 2 — GH Issues etags not plumbed

`crates/reposix-github/src/lib.rs:386` says `// (no etags plumbed yet)`. GitHub's REST API exposes ETags on issue GETs and accepts `If-Match` on PATCHes. Pure engineering work — the design is right; implementation is missing. Phase C lands it.

### Residual 3 — Hard delete detection on read

If a record is hard-deleted on the backend and the user `cat`s its stale cache file, they see stale content. Webhook handles the common case (delete event → cache tombstone). Missed-webhook case: opportunistic check on `list_changed_since` calls handles it after one push round-trip.

Truly silent stale-read is bounded by: `min(webhook_cadence, time_to_next_push)`. For active workflows this is small; for read-only sessions it can be unbounded — `reposix sync --reconcile` is the answer.

This is the only residual that doesn't fully resolve to "free." But it's a *read* problem, not a *write safety* problem, and the existing escape hatch covers it.
