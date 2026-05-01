# Cache coherence redesign — optimistic concurrency on writes (v0.14.0 proposal)

**Status.** Proposal. Not yet ratified by the v0.14.0 planner. Counter-proposal to the L2/L3 framing in `vision-and-mental-model.md § L2/L3 cache-desync hardening` (which remains in place as one option for the planner to choose between).

**Authored.** 2026-05-01, after v0.13.0 milestone-close. Owner conversation `2026-05-01` raised the question: *"Is it truly a trade-off? Can we not have performance and safety?"* This doc is the design that argues we mostly can.

**Reading order for the v0.14.0 planner.**
1. `vision-and-mental-model.md` — the milestone scope (OTel + origin-of-truth + cache hardening + +2 reservation). Treat the L2/L3 section there as Option A.
2. This doc — Option B. Read both, ratify one (or hybridize) in `decisions.md` at planning time.
3. `.planning/research/v0.13.0-dvcs/architecture-sketch.md § "Performance subtlety"` — origin of the L1 ↔ L2/L3 ladder.

---

## TL;DR

The L2/L3 ladder treats conflict detection as something we have to do *before* writing, in a precheck. But every well-designed REST API already provides conflict detection on the *write itself*, via etags / `If-Match` / version preconditions. Lean on that, and the precheck-vs-cache-staleness trade-off mostly evaporates.

- **Performance:** drop the `list_changed_since` precheck for backends with strong versioning. One round-trip saved per push.
- **Safety:** every PATCH carries `If-Match: <cached_version>` (or backend equivalent). Backend rejects 412 / version-mismatch on stale cache. Helper translates → `error refs/heads/main fetch first`. Identical recovery UX to today's L1.
- **Self-healing:** if cache write fails after a successful backend write, next push's `If-Match` uses the stale version, gets 412, fetches fresh, retries. No background reconcile job needed.
- **Read cache:** stale cache reads are tolerated. Most agentic workflows don't notice, and `reposix sync --reconcile` stays as the user-explicit "I want a fresh read cache" escape hatch.

The proposal is not novel — it's standard optimistic concurrency control. The novelty is recognizing that v0.13.0's L1/L2/L3 framing absorbed a precheck-shaped assumption from the original `list_records` walk and never relaxed it.

---

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

---

## Phase shape — borrowing v0.13.0's success pattern

v0.13.0 shipped 11 phases with high success rate (verifier-GREEN every phase, +2 reservation actually drained, RETROSPECTIVE distillation OP-9 ratified). The pattern that worked:

1. Architecture sketch + decisions.md ratified BEFORE any phase scoped.
2. Each phase owned ONE piece of the surface; no phase spanned multiple subsystems.
3. Catalog rows minted on each phase's first commit defined the phase's GREEN contract.
4. Verifier subagent on every close.
5. **Two reservation slots at milestone end** absorbed surprises + good-to-haves polish.

Cache-coherence redesign should mirror this. Proposed phase decomposition:

### Phase A — Validation research (no code)

**Mode:** top-level, gsd-phase-researcher subagent fan-out (mirror v0.13.0 P78 pattern).

Real-backend probes against the three sanctioned targets (TokenWorld, `reubenjohn/reposix`, JIRA TEST):
- **Confluence:** issue concurrent edits via `version.number = current + 1`; confirm 409 fires; measure race window. Confirm what the existing `reposix-confluence` adapter does end-to-end against real Confluence (not just sim contract test).
- **GH Issues:** confirm `ETag` on GET; confirm `If-Match` on PATCH; measure 412 rate under deliberate concurrent edits.
- **JIRA:** confirm `If-Unmodified-Since` support per Atlassian docs; if absent, fall back to read-then-write or keep L1-precheck. Measure timestamp-resolution race window on JIRA TEST.
- **Sim:** baseline already passes; confirm trait contract still holds.

**Output.** `cache-coherence-validation.md` next to this doc, with empirical findings + recommended per-backend strategy. **Decision-blocking:** if validation shows GH Issues doesn't actually accept `If-Match` (or accepts it but the semantics differ from the docs), Phase B onward replans.

### Phase B — Sim POC

**Mode:** standard `/gsd-execute-phase` (single-crate scope).

- Implement "drop precheck, lean on `If-Match`" against the simulator only.
- Demonstrate self-healing under simulated cache-write-failure (backend 200, cache write injected to fail; next push 412s on stale version, refetches, retries).
- Demonstrate stale-read tolerance in agent UX (cat returns stale; edit + push 412s + recovers cleanly).

**Catalog rows.** `cache-coherence/optimistic-concurrency-poc-sim` (mechanical) + `cache-coherence/self-healing-on-cache-write-fail` (mechanical, fault-injected).

### Phase C — GH Issues etag wiring

**Mode:** standard `/gsd-execute-phase`.

Plumb `ETag` round-trip in `reposix-github`. Real-backend test against `reubenjohn/reposix` issues (sanctioned per `docs/reference/testing-targets.md`).

**Catalog row.** `cache-coherence/gh-issues-if-match` (mechanical, real-backend).

### Phase D — JIRA strategy

**Mode:** standard `/gsd-execute-phase`. Strategy (a/b/c) determined by Phase A.

Real-backend test against JIRA TEST project.

**Catalog row.** `cache-coherence/jira-soft-version` (mechanical, real-backend).

### Phase E — Confluence real-backend confirmation

**Mode:** standard `/gsd-execute-phase`.

The wiring is already there but the real-backend concurrent-edit test isn't. Land it — TokenWorld is sanctioned for free mutation.

**Catalog row.** `cache-coherence/confluence-strong-versioning-real` (mechanical, real-backend).

### Phase F — Cache reframing + precheck removal + docs

**Mode:** standard `/gsd-execute-phase`.

- Strip `list_changed_since` precheck for backends with strong versioning. Branch on `Capability::StrongVersioning` so JIRA (Phase D's chosen strategy) keeps whichever path it needs.
- Reframe `docs/concepts/dvcs-topology.md` cache section: cache is a hint, not an oracle.
- Update `docs/guides/troubleshooting.md` § DVCS issues with the new 412-driven rejection messages.
- `reposix sync --reconcile` doc text shifts from "fix desync" to "force a fresh read cache."

**Catalog rows.** `cache-coherence/precheck-removed`, `docs/cache-coherence-mental-model`.

### Phase G — Surprises absorption (+1 reservation)

OP-8 standing rule. Drains `SURPRISES-INTAKE.md` entries logged during Phases A-F.

### Phase H — Good-to-haves polish (+1 reservation)

OP-8 standing rule. Drains `GOOD-TO-HAVES.md` XS items.

**Total:** 6 implementation phases + 2 reservation slots = 8 phases for the cache-coherence scope.

---

## Sequencing within v0.14.0

The current vision doc shapes v0.14.0 as: OTel (Phases 54-57) → origin-of-truth (Phases ~58-59) → cache hardening (Phases ~60-62) → +2 reservation (Phases ~63-64). **11 phases.**

This proposal expands the cache-hardening scope from 3 phases to 6 (research + POC + 3 per-backend + reframe). New milestone shape:

- OTel: Phases 54-57 (4)
- Origin-of-truth: Phases 58-59 (2)
- Cache coherence (this doc): Phases 60-65 (6)
- +2 reservation: Phases 66-67

**14 phases total.** That's larger than v0.13.0's 11. The planner should evaluate whether v0.14.0 absorbs all three scopes or whether one defers.

**Suggested cut, if the milestone needs to shrink.** Defer origin-of-truth to v0.15.0. Cache coherence is foundational (the OTel telemetry from Phase 54 measures *something* — better that the something be a redesigned cache than a half-redesigned one). Origin-of-truth unlocks multi-backend bus (still future); cache coherence unlocks today's correctness story.

Alternative: ship Phase A (validation research) in v0.14.0 alongside OTel + origin-of-truth, defer Phases B-F to v0.15.0. Mirrors v0.13.0's "research before commitment" gate. Risk: leaves the L1 trade-off in place for another milestone.

The planner should choose with the OTel telemetry the key input: if Phase 54-55 shows desync incidence is genuinely bothering users, prioritize cache coherence next. If it's negligible, origin-of-truth is the next-most-leveraged scope.

---

## Open questions for the planner (decisions.md style)

- `Q-CC.1` Does GitHub's Issues REST API actually accept `If-Match` on PATCH `/repos/{owner}/{repo}/issues/{issue_number}`? Docs imply yes; Phase A validates. **Decision-blocking** — if no, Phase C replans (likely fall back to JIRA-style read-then-write).
- `Q-CC.2` What's JIRA's actual support for `If-Unmodified-Since` on issue PATCH? Atlassian docs say yes for some endpoints, ambiguous for issues. Phase A validates. **Decision-blocking.**
- `Q-CC.3` Confluence `version.number = current + 1` — does the server accept the body unconditionally and reject only on 409, or does it require a separate `If-Match` header? Sim contract test asserts the body shape; real Confluence may have more.
- `Q-CC.4` Does the cache's audit row for `update_record` carry the `expected_version` value? If so, recovery audit is rich; if not, P83's `helper_push_partial_fail_*` family doesn't extend cleanly to the new 412 rejection shape. Likely needs a new audit op `helper_push_412_stale_version`.
- `Q-CC.5` Bus push (v0.13.0 P83) interaction: bus push fans SoT-first, mirror-best-effort. If SoT 412s on `If-Match`, what happens? Probably: helper acks `error refs/heads/main fetch first` exactly as today's bus precheck B does. Mirror push never fires (bus rule). But the `helper_push_partial_fail_mirror_lag` op shouldn't fire either — this is a pre-mirror rejection. New audit op needed: `helper_push_412_pre_mirror`.
- `Q-CC.6` Read-cache freshness for non-write workflows: how stale is acceptable? If an agent runs `grep -r TODO issues/` on a year-old cache, that's not a safety problem but it's a UX problem. Document a "freshness budget" in `docs/concepts/dvcs-topology.md`?
- `Q-CC.7` Does `reposix sync --reconcile` need to learn the new audit op vocabulary? If a reconcile finds a record where `cache.version != backend.version`, that's a "cache-update-failed-after-backend-200" signal — distinct from "user edited backend out-of-band." Telemetry should distinguish (mirrors Q-CD.2 from the L2/L3 doc).
- `Q-CC.8` Migration: existing checkouts have caches built before this change. Do they need a one-shot `--reconcile` to populate version metadata, or is the cache schema already version-complete? Likely already complete (`Record.version` is in the cache schema today) but Phase A confirms.

---

## Risks — known unknowns and unknown unknowns

### Known unknowns (Phase A drains)

- **Real-backend etag/version semantics may differ from docs.** Most likely culprit: GH Issues. Mitigation: Phase A is decision-blocking before Phase B-F scope.
- **JIRA timestamp resolution may be coarse enough to make `If-Unmodified-Since` useless.** If `updated` is second-resolution and concurrent edits within 1s are common, race window is real. Mitigation: Phase A measures incidence on TEST project; Phase D picks strategy from data.
- **Confluence's 409 semantics may have edge cases under attachment-bearing pages.** P82's audit work uncovered some non-trivial Confluence rate-limit behavior; concurrent-edit response may have similar surprises. Mitigation: Phase A includes attachment-bearing concurrent-edit test.
- **Cache-write-failure recovery via 412 assumes the next push touches the same record.** What if the user's next 1000 pushes all touch *other* records? The stale record sits silently desyncd. Mitigation: webhook + opportunistic verification on `list_changed_since`. If still concerning, add a "verify N records per push" piggyback (cheap; bounded). Likely a Phase F sub-task.

### Unknown unknowns (the +2 reservation absorbs)

- **Bus-remote interaction.** P83 ratified SoT-first + mirror-best-effort. Adding 412 rejection at SoT introduces a new failure mode mid-push. Detailed audit-trail design owned by Q-CC.5.
- **Multi-actor races.** v0.13.0 P82-83 didn't surface these because the bus is push-only and pushes serialize on the helper. Cache-write-side races (helper push + webhook listener both updating the cache simultaneously) may surface in Phase B/C/D under fault injection.
- **Connector-API churn.** If we discover GH Issues needs a slightly different `update_record` signature (e.g., the etag is opaque and shouldn't be `Option<u64>`), the trait surface changes. Backwards-compat for in-flight v0.14.0 phases needs management. Mitigate by Phase A producing a "trait surface confirmation" doc before Phase B touches code.
- **Rate-limit interaction.** The proposal *reduces* REST call count on the success path (no precheck) but `If-Match` headers don't change rate-limit cost. On the failure path (412 + refetch + retry), the worst case is 2x the success path. For backends with tight rate limits (Confluence Cloud: 5000 req/hr), persistent contention on hot records could make rate limits hurt sooner. Phase A measures.
- **Webhook-driven cache update (Layer 3) introduces a writer to the cache from outside the helper.** Today the helper is the sole writer; a CI workflow updating the cache concurrently is a new shape. Audit-log invariants (append-only) hold trivially, but cache-tree atomicity may not. Likely needs Phase F to add a cache-side write lock or branch the writes.

The **+2 reservation slots are non-optional**, mirroring v0.13.0's pattern. The "absorption-not-skipping" honesty check (OP-8 verifier) applies to every Phase A-F close.

---

## Tie-back to project invariants

- **OP-1 (simulator-first):** Phase B is sim-only by design. Phases C/D/E require real backends but are gated on `--ignored` test runs.
- **OP-2 (tainted by default):** `If-Match` headers carry version numbers (server-controlled, untainted). Cache-side version data is server-controlled at write time. Read-side version read is untainted.
- **OP-3 (audit log dual-table):** every 412 rejection writes a row to `audit_events_cache`. Every successful 412-driven retry writes both (`audit_events_cache` for the helper turn, `audit_events` for the backend write). New audit ops: `helper_push_412_stale_version`, `helper_push_412_retry_success`.
- **OP-6 (real backends are first-class):** Phases C/D/E land real-backend tests; "simulator-only coverage" doesn't satisfy.
- **OP-7 (verifier subagent on close):** every Phase A-F dispatches a verifier per `quality/PROTOCOL.md`. Phase A's verifier grades the validation findings against the per-backend predictions in this doc — if findings contradict the doc, that's a RED that triggers replanning.
- **OP-8 (+2 reservation):** Phases G/H reserved. Eager-resolution preference applies for trivial in-phase findings.
- **OP-9 (milestone-close distillation):** v0.14.0's RETROSPECTIVE.md section captures the cache-coherence redesign learnings cross-milestone.

---

## Where to start when you pick this up

1. Read this doc.
2. Read `vision-and-mental-model.md § L2/L3 cache-desync hardening` for the option-A framing this proposal counters.
3. Skim `crates/reposix-core/src/backend.rs:53-167` (capability + versioning enums) — they're the trait substrate this proposal completes.
4. Skim `crates/reposix-core/src/backend/sim.rs:333-345` for the canonical `If-Match` shape.
5. Run `/gsd-discuss-phase` with this doc + the vision doc as inputs. Ratify Option A vs Option B vs hybrid in `decisions.md`.
6. If Option B (this doc) is ratified: `/gsd-plan-phase` for Phase A first; do NOT scope Phases B-F before Phase A's findings land.
