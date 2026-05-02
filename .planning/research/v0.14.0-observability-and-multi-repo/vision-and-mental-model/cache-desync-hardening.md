# L2/L3 cache-desync hardening

[← index](./index.md)

> **See also (2026-05-01):** `cache-coherence-redesign.md` (sibling doc) proposes an **Option B** that avoids the L2/L3 fork by leaning on optimistic concurrency at write time (`If-Match` / version preconditions) rather than pre-check-then-write. The planner should ratify Option A (this section) vs Option B (sibling doc) vs a hybrid in `decisions.md` before scoping the cache-hardening phases. The sibling doc also expands the cache-hardening scope from 3 phases to 6 (research → POC → 3 per-backend → reframe), borrowing v0.13.0's success pattern of validation-before-implementation; the milestone-shape implications are discussed there.

**Problem.** v0.13.0 ships **L1** for the conflict-detection path: replace the per-push `list_records` walk with `list_changed_since(last_fetched_at)`. The trade-off is documented in `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § "Performance subtlety": L1 is much faster but trusts the cache as the prior tree state. If the cache desyncs from the backend (a write succeeded backend-side but failed cache-side, or vice versa), L1's conflict detection silently misses it.

L1 is the right v0.13.0 trade because (a) the bus remote's promise of "DVCS at the same UX as plain git" requires sub-100ms-ish push latency, which the old `list_records` walk made impossible at scale; (b) we don't have desync incidence numbers yet — we'd be guessing whether L2 or L3 is the right hardening; (c) `reposix sync --reconcile` ships in v0.13.0 as a manual escape hatch.

L2 and L3 are the v0.14.0 question: which one ships, and when?

**The two options.**

| Layer | Mechanism | What it costs | Right when |
|---|---|---|---|
| **L2** | L1 + a periodic full-resync. Background job (or every-Nth-push hook) that runs `list_records` and reconciles against cache. Repairs desync after the fact. | One full enumeration per resync interval. Need a scheduling story (cron daemon? on-push-N? user-explicit?). Repair operations need careful audit trails. | Desync is "user-visible" — happens often enough that users notice but rarely enough that an hourly reconcile is acceptable. |
| **L3** | L1 + transactional cache writes. Every backend write path (sim, confluence, GH Issues, JIRA adapters) wraps the cache update in a transaction with the backend write; partial failure rolls back consistently. Desync becomes impossible by construction. | Real engineering across every adapter. Backend APIs aren't transactional — we're approximating with a write-ahead log + reconciliation-on-recovery. | Desync is "rare-but-catastrophic" — happens almost never but causes silent data corruption when it does. |

**The decision rule.** Ship the OTel work in Phase 54 first. Add a `cache.desync.detected` span attribute and audit-event subtype that fires whenever `reposix sync --reconcile` finds a discrepancy. Run for at least one weekly cycle (preferably across the three real backends — TokenWorld confluence, reubenjohn/reposix GH Issues, JIRA TEST project). Decide:

- If desync incidence is **> 1 per 1000 pushes** → ship **L2**. Background resync is a good ROI; users will hit desync often enough to need automatic repair.
- If desync incidence is **< 1 per 100,000 pushes** → ship **L3**. Hardening every write path is real work, but at this rate L2's full-resync overhead is wasted; L3 prevents the rare-but-catastrophic cases.
- If somewhere in between → start with L2 (cheap to ship, broad coverage) and queue L3 for v0.15.0.

**Phase sketch (post-OTel telemetry collection).**

- **Phase 5x — Cache desync telemetry.** Wire `reposix sync --reconcile` to emit OTel + audit events on every discrepancy. Run for one weekly cycle minimum across all three real backends.
- **Phase 5x+1 — L2 OR L3 (decision driven by Phase 5x data).**
  - If L2: design the resync scheduler (probably a flag on the helper config + an `on-push-N` interceptor; daemon-mode deferred). Wire reconciliation into the existing `reposix sync` machinery. Audit events for every repair.
  - If L3: design the transactional write protocol. Each adapter's `create_record` / `update_record` / `delete_or_close` wraps the cache update + backend write in a write-ahead log. Recovery on next start replays incomplete entries. Real-backend tests across all three sanctioned targets.
- **Phase 5x+2 — Migration / removal of `--reconcile` escape hatch.** If L3 ships, the manual `--reconcile` stays as a defense-in-depth no-op. If L2 ships, document `--reconcile` as "force an immediate resync rather than waiting for the schedule."

**Why v0.14.0, not v0.13.0.** L2 needs a scheduling story; L3 needs a real-engineering audit of every adapter. Both are bigger than v0.13.0 can absorb without doubling its scope, and neither is on the DVCS critical path. v0.13.0 ships L1 + the manual escape hatch (`reposix sync --reconcile`); v0.14.0 lets the OTel data drive the choice.

**Open questions for the planner.**

- `Q-CD.1` Is the desync-incidence measurement window in Phase 5x just the weekly cycle, or do we need a longer baseline? Probably one week per backend is enough for a coarse decision, but document the methodology.
- `Q-CD.2` Does `cache.desync.detected` count "discrepancies the user caused by editing the backend out-of-band" the same as "discrepancies caused by failed adapter writes"? They have the same data signature but very different remediation. Telemetry needs to distinguish.
- `Q-CD.3` If we ship L3, does the write-ahead log live in the existing `audit_events_cache` table or a separate `cache_wal` table? Probably separate — the audit table is append-only-by-design and shouldn't carry transactional rollback semantics.
