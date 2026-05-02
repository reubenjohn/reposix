← [back to index](./index.md)

# Phase shape and v0.14.0 sequencing

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
