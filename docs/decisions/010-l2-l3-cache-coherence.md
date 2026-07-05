# ADR-010 — L2/L3 cache coherence and `SotPartialFail` recovery

| | |
|---|---|
| **Status** | **ACCEPTED** |
| **Ratified** | 2026-07-05 by the P93 coordinator — reversibility verdict is NOT an irreversible fork (reversible internal cache-strategy behind `Cache::sync`; no wire/ref/`Tainted`/audit/latency contract change), so no E2 escalation. |
| **Date** | 2026-07-05 |
| **Phase** | v0.13.0 P93 (RBF-LR-01 / RBF-LR-02 / RBF-LR-03; closes D-P92-03) |
| **Supersedes / amends** | none — operates *within* the ratified partial-clone + lazy-materialization architecture (P78–P88) |

## Context

### The confirmed defect (D-P92-03) — a broken tree↔`oid_map` invariant

A second writer's `git pull --rebase` after a two-writer conflict dies with
`fatal: git upload-pack: not our ref <oid>` / `could not fetch <oid> from
promisor remote` **whenever the conflicting write lands in the same wall-clock
second as the puller's cache cursor**. This is reproduced, deterministic, and
NOT environmental: 4/4 same-second runs fail, and the 2-second-gap negative
control is clean (executed transcripts + root cause:
[`.planning/phases/93-cache-coherence/93-DP2-REPRO-NOTES.md`](../../.planning/phases/93-cache-coherence/93-DP2-REPRO-NOTES.md);
repro commit `9c46e49`).

The failure is two coupled defects — a *trigger* and a latent *amplifier*:

- **Trigger — the sim (and every real backend) drops same-second writes from
  the delta query.** `crates/reposix-sim/src/routes/issues.rs:138-139` stamps
  `updated_at` at **seconds** precision (`SecondsFormat::Secs`); the `since`
  filter (`issues.rs:179-184`) truncates the cursor to seconds too and compares
  with a **strict** `updated_at > ?2`. Any write whose truncated second is `<=`
  the cursor's truncated second is invisible to `list_changed_since`, even
  though `list_records` (unfiltered) still returns its new content. **This is
  not a sim artifact** — Confluence, JIRA, and GitHub `updated_at` are all
  second-resolution too, so any real backend re-creates the trigger under clock
  skew or a tight write/fetch race.

- **Amplifier — the actual defect — `Cache::sync` builds a tree entry it cannot
  serve.** `crates/reposix-cache/src/builder.rs::sync()` sources the git **tree**
  from `list_records` — the FULL current backend state (Step 4,
  `builder.rs:293-328`) — so a changed record's tree entry is *always* its
  current-content OID, reflecting the write regardless of the delta query. But
  blob materialization + `oid_map` population (Steps 3 & 5,
  `builder.rs:265-291`, `builder.rs:373-406`) cover only the
  `list_changed_since` **delta**. When the trigger makes those two sources
  disagree, the HEAD tree references an OID for which the object store has no
  blob and `oid_map` has no row — a **dangling tree entry**.

  On the puller's lazy fetch, that OID reaches `Cache::read_blob`
  (`builder.rs:450-458`) → `oid_map` miss → `Error::UnknownOid`. The helper
  (`crates/reposix-remote/src/stateless_connect.rs:369-385`) treats
  `UnknownOid` as non-fatal and leaves the `want` for `git upload-pack`, which
  rejects it: `not our ref <oid>`.

**The violated invariant:** *every blob OID the HEAD tree references must be
resolvable by `read_blob`.* The tree is derived from a broader source
(`list_records`, always current) than the `oid_map`/blob bookkeeping (the
`list_changed_since` delta); any under-report of the delta relative to the tree
produces an unservable OID. The seconds boundary is today's trigger; the
tree↔`oid_map` divergence is the durable fragility. A fix that only tightens
the sim's timestamp precision papers over a cache-layer invariant break that a
real backend will re-open.

Note the blast-radius boundary the repro settles: **the bug is on the FETCH/sync
path only.** The PUSH precheck path (`precheck.rs`) reads the SoT directly and is
not vulnerable. `build_from` (`builder.rs:56-191`), the full-rebuild seed path,
is coherent by construction — it calls `put_oid_mapping` for **every** record it
lists (`builder.rs:112-121`), so its tree never references an unrecorded OID.
Only the **delta** path under-covers `oid_map` relative to the tree it emits.

### The second defect — `refresh_for_mirror_head` and the L1 "no-op" (RBF-LR-02)

`crates/reposix-cache/src/mirror_refs.rs:297-299` — `refresh_for_mirror_head`
forwards to `build_from()`. It is **not** itself a no-op: when called it does the
full, coherent rebuild. The "no-op post-write" behavior is **caller-side**: the
write loop gates the call behind `files_touched > 0`
(`crates/reposix-remote/src/write_loop.rs:295`, the "L1 no-op-push perf-skip"),
with the comment *"Self-healing on next non-trivial push"*
(`write_loop.rs:293-294`) admitting a transient staleness window on genuine
no-op pushes. That skip is asserted as a *feature* in
`crates/reposix-remote/tests/perf_l1.rs:386-390` (a no-op push must issue zero
`list_records` calls).

So the honest statement of RBF-LR-02 is narrower than "refresh no-ops post-write":
`refresh_for_mirror_head → build_from` is coherent on the write path; the
"no-op" is a `files_touched`-gated caller skip whose only cost is a mirror-head
ref that lags until the next non-trivial push. The chosen model must (a) leave
the write-path refresh coherent, and (b) make the **fetch/delta** path — the
thing D-P92-03 actually breaks — coherent too, so that *any* subsequent
`reposix sync` cannot re-introduce a dangling entry.

## Options

### Option A — re-fetch on cache miss (the "L2" strategy)

*Canonical id: `re-fetch-on-cache-miss` (L2).*

Make `read_blob` resolve an unknown OID by re-fetching: on an `oid_map` miss,
scan the backend (`list_records`, render each record, hash, match the requested
OID) and materialize the winner on demand.

- **For:** resilient to *any* future coherence gap, not just this one — a
  belt-and-suspenders safety net at the read boundary.
- **Against — network on the read/fetch path.** The miss is resolved *during*
  `git upload-pack` want-resolution (`stateless_connect.rs:369-385`), so a read
  now triggers an **O(N) outbound scan** (there is no OID→record index for an
  *unknown* OID — that index *is* `oid_map`, which is what missed). This inflates
  fetch latency and can breach the documented envelope
  ([`docs/benchmarks/latency.md`](../benchmarks/latency.md)).
- **Against — expanded taint/egress surface on a hot path.** A read path that
  today never dials out would now make outbound HTTP. It stays inside the
  `reposix_core::http::client()` allowlist (`read_blob` already calls
  `backend.get_record` through it, and `classify_backend_error` fires
  `egress_denied` on denial — `builder.rs:465-476`, `builder.rs:524-538`), so it
  does not *widen* the allowlist, but it does move remote-byte ingestion onto the
  fetch-serving path where OP-1/OP-2 review is most sensitive. Each re-fetched
  blob is still `Tainted<Vec<u8>>` and content-verified against the requested OID
  (`OidDrift` guard, `builder.rs:489-495`), so correctness holds — but the
  surface grows.
- **Against — it papers over the invariant** rather than restoring it: the tree
  can still be emitted with unrecorded OIDs; A just makes that survivable at
  read time.

### Option B — transactional cache writes (the "L3" strategy) — **recommended**

*Canonical id: `transactional-cache-writes` (L3).*

Restore the tree↔`oid_map` invariant at its source: `Cache::sync`'s delta path
must never emit a tree entry for an OID that is not simultaneously recorded in
`oid_map`. Concretely, in Step 4 (`builder.rs:293-328`), when the tree is built
from `list_records`, upsert an `oid_map` row for **every** record's computed OID
(not just the `list_changed_since` delta) inside the existing atomic transaction
(`builder.rs:378-406`) — exactly the coverage `build_from` already provides
(`builder.rs:112-121`). The materialize-difference is a **metadata** upsert;
**blob bytes stay lazy** (materialized on `read_blob`), so the ratified
lazy-materialization property is preserved.

- **For — restores the invariant by construction.** After the fix, no
  `Cache::sync` output can reference an OID `read_blob` cannot resolve; the
  same-second trigger becomes harmless (the tree reflects the write *and* the
  OID is resolvable → the puller lazily fetches it, no `not our ref`).
- **For — no read-path network, no new egress surface, cheap.** The extra work
  is per-sync `oid_map` upserts under a lock already held; the object store and
  the outbound path are untouched. Latency envelope unaffected.
- **For — it is the direction the repro notes already recommend** (candidate
  fix (a)) and the RUNBOOK §C default (narrow cache-layer fix + ADR).
- **Against — eager reconciliation per sync.** Every delta sync now touches
  `oid_map` for the full record set, not just the changed IDs — O(N) SQLite
  upserts where the changed set might be O(1). This is bounded (metadata only,
  no network, no blob writes) and matches what `build_from` already does; the
  cost is acceptable for the coherence guarantee.

### Option C — hybrid (B primary + a bounded reconcile escape hatch) — **recommended shape**

Adopt **B** as the invariant fix, and rely on `reposix sync --reconcile` to
self-heal any cache that has *already* drifted (a cache written by a pre-fix
binary, or one that drifted for a reason we did not foresee). Keep `read_blob`'s
`UnknownOid` **fatal-to-that-want** as today (the helper leaves it for
upload-pack), and **defer Option A's read-path re-fetch to v0.14.0** as a
documented, bounded exposure. This is precisely the RUNBOOK §C Decision-2
option (b): *narrow fix + ADR documenting the bounded exposure + a `reposix sync
--reconcile` teaching-string; NO re-deferral to v0.14.0 without owner sign-off.*

**Caveat the fix wave MUST close — the reconcile hatch is currently broken.**
`reposix sync --reconcile` already exists
(`crates/reposix-cli/src/sync.rs:45-104`) and its doc-comment (`sync.rs:48-49`)
promises *"a full list_records walk + cache rebuild (the L1 escape hatch)"* — but
line 98 calls `cache.sync()`, which takes the **delta** path whenever
`last_fetched_at` is present (only an *absent* cursor forwards to `build_from`).
So today the "escape hatch" re-runs the very delta path D-P92-03 breaks and
**cannot recover a drifted cache** — a lying doc *and* a broken recovery move.
Option B's fix makes even the delta path coherent (so `--reconcile` stops
*producing* drift), but to honour the "full rebuild" promise and heal a
pre-fix-drifted cache, the fix wave must make `--reconcile` force a full
`build_from` (e.g. clear `last_fetched_at` first, or call `build_from` directly).

### Sub-decision — sim/backend second-resolution precision

Is a precision fix (sim `updated_at` sub-second; cursor `>=` with de-dup)
defense-in-depth, or does the invariant fix make it moot?

**Decision: the invariant fix (B) makes the precision fix moot *for coherence*;
a precision fix alone would NOT fix the amplifier and MUST NOT ship as the sole
remedy.** With B, Step 4 still rebuilds the tree from `list_records` (so a
same-second write's content is reflected) and now records its OID (so
`read_blob` resolves it) — coherence holds whether or not `list_changed_since`
under-reports. A precision-only fix (sub-second sim timestamps) would hide the
trigger on the sim while leaving the tree↔`oid_map` amplifier live for the
second-resolution real backends (Confluence/JIRA/GitHub), i.e. it would fake-fix
the reproduced bug. Recommendation: ship B; treat sim sub-second precision +
cursor `>=`-with-de-dup as **optional defense-in-depth** (it improves
delta *efficiency* — fewer missed changes → more eager pre-materialization — and
de-risks other `list_changed_since` consumers such as the push precheck's
changed-set), explicitly **not** load-bearing for the coherence guarantee.

## Decision

**Ratified (ACCEPTED 2026-07-05 by the P93 coordinator):**

1. **Coherence model: Option C = Option B (transactional cache writes) + a
   `reposix sync --reconcile` escape hatch.** The fix wave restores the
   tree↔`oid_map` invariant inside `Cache::sync`'s atomic transaction by
   upserting `oid_map` for the full `list_records` set (metadata only; blobs stay
   lazy), and flips the RED regression
   `delta_sync_tree_references_only_resolvable_oids` GREEN by dropping its
   `#[ignore]` in the same commit.

2. **Refresh-path redesign (RBF-LR-02).** `refresh_for_mirror_head` stays as the
   full-rebuild `build_from` (already coherent). The *honest* reconciliation of
   the L1 promise is the **RBF-LR-04 "keep + qualify"** branch: retain the
   `files_touched > 0` skip (`write_loop.rs:295`) — refreshing the mirror-head on
   a genuine no-op push is wasted work with **zero** coherence benefit, since a
   no-op push changes nothing in the SoT — but re-document it in
   `docs/concepts/dvcs-topology.md` as a *semantic* no-op (nothing changed →
   nothing to refresh), not a coherence shortcut, and point at `reposix sync
   --reconcile` as the manual catch-up. The unqualified asterisk is replaced by
   this bounded-exposure explanation rather than silently dropped. (The fix wave
   MAY instead choose "always refresh, remove the asterisk" — an unconditional
   refresh — if it judges the perf cost negligible; the ADR leaves that lever to
   the fix-wave, since both are honest. Either way, no lying doc.)

3. **`SotPartialFail` recovery semantics (RBF-LR-03).** `SotPartialFail`
   (`write_loop.rs:105`, emitted at `write_loop.rs:277`) means at least one
   `execute_action` failed after others succeeded — the SoT is **partially
   written**. The helper emits `error refs/heads/main some-actions-failed`; no
   `oid_map`/mirror-head/cursor advance happens for the failed push (those writes
   are on the `SotOk` branch, `write_loop.rs:282-313`). **Convergence:** the
   partially-applied writes are now part of the SoT. On the agent's next push,
   PRECHECK B (`precheck_export_against_changed_set`, `write_loop.rs:159-160`)
   re-reads the current SoT, `diff::plan` (`write_loop.rs:210`) recomputes
   against that new base, and only the still-needed actions are replanned and
   applied — the already-landed writes are diffed away. **What the agent sees:**
   the `some-actions-failed` protocol error plus per-action `error: <e>` stderr
   (`write_loop.rs:268`), and a normal `git push` retry converges. The cache
   converges via the same PRECHECK-B re-read on the next push (or a `reposix
   sync` / `--reconcile`); there is no torn cache state because the failed push
   never advanced the cursor or `oid_map`.

4. **Test co-location.** SC2 pins `cargo test -p reposix-cache --test
   cache_coherence`, but `SotPartialFail` + PRECHECK B live in **reposix-remote**
   (`write_loop.rs` / `precheck.rs`), not reposix-cache. **Decision:**
   - the tree↔`oid_map` **coherence invariant** test lives in
     `crates/reposix-cache/tests/cache_coherence.rs` (satisfies SC2's exact
     command), alongside the existing delta-sync RED regression that flips GREEN;
   - the **`SotPartialFail` recovery** test lives in a reposix-remote binary,
     `crates/reposix-remote/tests/partial_failure_recovery.rs` (simulate
     SoT-success-on-some + fail-on-one → assert the next push reads the new SoT
     via PRECHECK B and replans only the remainder);
   - the real-backend arm (`partial_failure_recovery_real_confluence`,
     RBF-LR-03/`p93-partial-failure-recovery-real-confluence`) lives in
     `crates/reposix-cli/tests/agent_flow_real.rs` as an `#[ignore]` smoke.
   **SC2's exact command needs a companion assert:** the catalog row
   `agent-ux/p93-cache-coherence-refresh-honest` should add
   `cargo test -p reposix-remote --test partial_failure_recovery` as a companion
   command (its asserts already say "wherever it lands"); the reposix-cache
   `cache_coherence` command alone cannot prove the reposix-remote recovery path.

## Reversibility / blast-radius assessment

This section is load-bearing for the coordinator's escalation routing.

**Does the recommended model (B/C) change any PUBLIC or irreversible contract?**
Assessed against each locked surface:

| Surface | Option B/C (recommended) | Option A (rejected/deferred) |
|---|---|---|
| protocol-v2 fetch **wire** format | **No change** — same want/have exchange; the OID simply resolves now | No change |
| `refs/reposix/*` / `refs/mirrors/*` **ref** format | **No change** | No change |
| `Tainted<T>` sanitize boundary | **No change** — blobs stay lazy, still `Tainted` on read | No change (re-fetch still `Tainted` + `OidDrift`-checked) |
| audit-table shape (OP-3) | **No change** — reuses `oid_map` + existing `delta_sync` row; no new table/column | No change (adds `materialize` rows on a new path) |
| documented latency envelope | **No change** — extra work is per-sync metadata upserts, no network | **Changes** — adds O(N) outbound scan on the read/fetch path |
| agent-facing behavior | **Fixes** it (`not our ref` → clean lazy fetch); no *new* behavior | Also fixes, but adds read-path latency variance |
| on-disk cache schema | Internal to `Cache::sync`; cache is a **rebuildable** artifact, explicitly NOT a stability contract (ADR-009 §"On-disk cache schema") | Same |

**Reversibility rating.** Option **B/C is fully reversible**: it lives entirely
behind `Cache::sync` (a rebuildable-cache internal), touches no wire/ref/taint/
audit/latency contract, and could be reverted by rebuilding any cache with the
prior binary. Option **A is reversible but higher-blast-radius**: it moves remote
I/O onto the read path (a perf-envelope and OP-1/OP-2-surface change), so it is
the more consequential lever — which is *why* the recommendation defers A.

**Conclusion — one unambiguous sentence:**

> **This decision IS NOT an irreversible fork — it is a reversible internal
> cache-strategy choice within the ratified partial-clone direction.**

*Reasoning:* the recommended fix restores an invariant that `build_from` already
upholds, using the same `oid_map` bookkeeping, entirely inside `Cache::sync`; it
alters no public wire/ref/taint/audit/latency contract, and the cache it writes
is a rebuildable artifact by ADR-009. Nothing here re-opens the P78–P88
partial-clone/lazy-materialization axis; it makes the *existing* axis honest. The
coordinator can ratify and proceed without an L1 (E2 fable-consult) escalation.
(Had the recommendation been Option A — outbound HTTP on the read path — the
latency-envelope + hot-path-taint change would warrant a closer look, though it
too stays reversible.)

## Consequences

### What the fix wave must implement

1. **Cache-layer coherence fix (RBF-LR-01 / D-P92-03).** In
   `Cache::sync` Step 4, upsert `oid_map` for the full `list_records` set inside
   the atomic transaction (blobs stay lazy). Drop the `#[ignore]` from
   `crates/reposix-cache/tests/delta_sync.rs::delta_sync_tree_references_only_resolvable_oids`
   in the **same** commit; add the coherence-invariant assertions to a new
   `crates/reposix-cache/tests/cache_coherence.rs`.
2. **Refresh-path honesty (RBF-LR-02 / RBF-LR-04).** Keep-and-qualify the
   `files_touched > 0` skip in `docs/concepts/dvcs-topology.md` + root
   `CLAUDE.md` (or remove the asterisk via unconditional refresh), in the **same
   PR** as the fix (ROADMAP SC6 / QG-07). No lying doc.
3. **`SotPartialFail` recovery test (RBF-LR-03).** Add
   `crates/reposix-remote/tests/partial_failure_recovery.rs` (sim) + the
   `partial_failure_recovery_real_confluence` `#[ignore]` smoke in
   `crates/reposix-cli/tests/agent_flow_real.rs`.
4. **Fix `reposix sync --reconcile`** (`crates/reposix-cli/src/sync.rs:98`) to
   force a full `build_from` (clear `last_fetched_at` first, or call `build_from`
   directly) so it honours its "full list_records walk + cache rebuild" promise
   and can heal a pre-fix-drifted cache — today it calls `cache.sync()` (delta
   path) and cannot. Also refresh the stale/dangling doc cross-reference at
   `docs/guides/troubleshooting.md:352` (points at a dvcs-topology "Out of scope"
   anchor that has no L1/L2/L3 content, and says L3 "defers to v0.14.0" — which
   this ADR overrides).
5. **(Optional defense-in-depth)** sim sub-second `updated_at` + cursor
   `>=`-with-de-dup — improves delta efficiency; NOT load-bearing for coherence.

### Which tests gate it

- `agent-ux/p93-l2-l3-coherence-adr` — this ADR's existence + content
  (options + chosen path + trade-off + deferral list). Satisfied by this file.
- `agent-ux/p93-cache-coherence-refresh-honest` — `cargo test -p reposix-cache
  --test cache_coherence` GREEN + the `SotPartialFail` recovery test exists
  (companion: `cargo test -p reposix-remote --test partial_failure_recovery`) +
  refresh-path honesty.
- `agent-ux/p93-delta-sync-coherence-invariant` — the RED regression
  `delta_sync_tree_references_only_resolvable_oids` flips RED→GREEN with its
  `#[ignore]` removed (4 passed, 0 ignored).
- `agent-ux/p93-partial-failure-recovery-real-confluence` (real-backend,
  env-gated), `agent-ux/p93-l1-promise-reconciled` (doc honesty),
  `agent-ux/p93-mid-stream-litmus-t1-t4` (phase-gate re-run of dark-factory
  T1 + T4).

### Positive

- The reproduced `not our ref` two-writer failure is closed at the invariant
  level, robust to the second-resolution real backends — not papered over at the
  sim timestamp.
- No new outbound surface, no latency-envelope change, no wire/ref/audit contract
  change; the cache stays a rebuildable artifact.

### Negative

- Delta sync now does O(N) `oid_map` upserts per sync (metadata only). Bounded and
  already the shape `build_from` uses; acceptable for the coherence guarantee.
- The read-path resilience of Option A is deferred to v0.14.0 as a documented,
  bounded exposure — a cache that drifts for an unforeseen reason still needs a
  manual `reposix sync --reconcile` rather than self-healing on read.

### Neutral

- SC2's `reposix-cache --test cache_coherence` command is retained but needs a
  reposix-remote companion command for the `SotPartialFail` recovery assertion —
  a catalog wording tightening, not a scope change.

## Alternatives

- **Sim-precision-only fix** (sub-second `updated_at`) — **rejected as sole
  remedy**: hides the trigger on the sim, leaves the tree↔`oid_map` amplifier
  live for the second-resolution real backends. Kept only as optional
  defense-in-depth atop B.
- **Option A (re-fetch on cache miss) as the primary fix** — **deferred to
  v0.14.0**: adds outbound HTTP + O(N) scan on the read/fetch path and papers over
  the invariant instead of restoring it. Retained as a possible future
  belt-and-suspenders layer, gated on owner sign-off per RUNBOOK §C.
- **Re-defer the whole item to v0.14.0** — **rejected**: Decision-2 promoted
  RBF-LR-01/02 out of the §6 deferral table precisely because v0.13.0's
  round-trip vision depends on them; re-deferral without owner sign-off re-instates
  the C8 anti-pattern.

## References

- [`.planning/phases/93-cache-coherence/93-DP2-REPRO-NOTES.md`](../../.planning/phases/93-cache-coherence/93-DP2-REPRO-NOTES.md) — executed repro + root cause (repro commit `9c46e49`).
- `crates/reposix-cache/tests/delta_sync.rs::delta_sync_tree_references_only_resolvable_oids` — the RED regression this ADR's fix flips GREEN.
- `crates/reposix-cache/src/builder.rs` — `sync()` Steps 3-5 (`:265-406`), `build_from` (`:56-191`), `read_blob` (`:450-511`).
- `crates/reposix-remote/src/write_loop.rs` — `SotPartialFail` (`:105`, `:277`), PRECHECK B (`:159`), `files_touched > 0` skip (`:295`).
- `crates/reposix-remote/src/stateless_connect.rs:369-385` — the helper's `UnknownOid` handling.
- `crates/reposix-sim/src/routes/issues.rs:138-184` — the seconds-resolution trigger.
- [ADR-007 — Time-travel via git tags](007-time-travel-via-git-tags.md), [ADR-009 — Stability commitment](009-stability-commitment.md) — cache-is-rebuildable + partial-clone precedent.
- [DVCS topology](../concepts/dvcs-topology.md), [Troubleshooting — DVCS push/pull](../guides/troubleshooting.md#dvcs-pushpull-issues) — the L1/L2/L3 mental model this ADR reconciles.
