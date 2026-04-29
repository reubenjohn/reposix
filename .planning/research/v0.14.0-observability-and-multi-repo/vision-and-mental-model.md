# v0.14.0 — Vision and Mental Model

> **Audience.** The next agent picking up v0.14.0 planning, after v0.13.0 (DVCS thesis) closes. Read this BEFORE running `/gsd-new-milestone v0.14.0`. Sibling research bundles to read first:
> - `.planning/research/v0.13.0-dvcs/vision-and-mental-model.md` — the prior milestone's thesis. Explains why three items below were carved out of v0.13.0 instead of being absorbed into it.
> - `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § "Performance subtlety: today's `list_records` walk on every push" — the L1 work that ships in v0.13.0 and sets the stage for L2/L3 here.
>
> **Source attribution.** The "Observability scope" section below is pulled forward from `.planning/research/v0.10.0-post-pivot/milestone-plan.md` § "v0.13.0 — Observability & Multi-Repo" (lines 72–86 of that file). The original numbered it as the v0.13.0 milestone; the DVCS thesis jumped ahead in v0.13.0 planning on 2026-04-29, so the observability scope renumbered to v0.14.0 without scope changes. The phase numbers (54–57) survive verbatim because the v0.13.0 ROADMAP claimed phases 58+. **Do not edit the source** — it's a historical artifact; this doc supersedes it by attribution.
>
> **Drafted:** 2026-04-29 by the v0.12.1 planning session, alongside the v0.13.0-dvcs research bundle. Owner-approved via discussion transcript that day.

## The thing we are building

v0.14.0 is the **operational maturity** milestone. v0.13.0 ships a thesis-level shift (DVCS over REST); v0.14.0 makes the resulting system observable, multi-tenant, and self-correcting at scale. Three independently-motivated scopes converge into one milestone because they all share the same root: "we now have multiple writers, multiple backends, and a cache that's load-bearing in places we couldn't see before."

The three scopes:

1. **Observability** — OTel spans on the helper hot paths, `reposix tail` for live audit-event streaming, and a multi-project helper process that can serve more than one project from one invocation.
2. **Origin-of-truth frontmatter enforcement** — a guardrail (`origin_backend: <name>` frontmatter field) that prevents the bus remote from misrouting a write across multiple ISSUES backends (e.g., GH Issues + JIRA simultaneously fronted by one bus). Carved out of v0.13.0 because the v0.13.0 bus pattern is "one ISSUES backend (SoT) + one plain-git mirror" where this can't go wrong.
3. **L2/L3 cache-desync hardening** — v0.13.0 ships L1 (replace `list_records` with `list_changed_since`, trust cache as prior). L2 (periodic full-resync) and L3 (transactional cache writes) defer here once we have OTel telemetry on actual desync incidence.

The litmus test for "we shipped v0.14.0" is that an operator running reposix at non-trivial scale (multiple projects, multiple backends, real-backend traffic) can answer the following four questions in real time, without `sqlite3` after the fact:

- *"What's the p99 push latency for this project right now?"* (OTel + dashboard)
- *"Show me audit events for backend X in the last 60 seconds."* (`reposix tail`)
- *"Why did this push to JIRA route to confluence instead?"* (origin-of-truth frontmatter check tripped or failed to trip — surfaces in audit + tracing)
- *"How often does the cache desync from the backend, and which writes caused it?"* (telemetry → drives L2-vs-L3 decision)

## Observability scope (pulled forward from v0.10.0-post-pivot)

> **Source.** `.planning/research/v0.10.0-post-pivot/milestone-plan.md` § "v0.13.0 — Observability & Multi-Repo", restated here verbatim except for the milestone label. Renamed from v0.13.0 → v0.14.0 because DVCS jumped ahead in v0.13.0 planning (2026-04-29). Phase numbers (54–57) preserved.

**Thesis.** A user running reposix at any non-trivial scale (multiple projects, multiple agents, real backends) can see what's happening in real time. "Audit log" stops being a ground-truth artifact you `sqlite3` after the fact and becomes a live signal.

**Success gate.**
- Helper emits OpenTelemetry traces (configurable via `OTEL_EXPORTER_OTLP_ENDPOINT`); a sample dashboard JSON ships in `docs/reference/observability/`.
- `reposix tail` streams audit events from the SQLite WAL in real time (think `journalctl -f`).
- A single `git-remote-reposix` process can serve multiple projects from one helper invocation (cache shared between projects on the same backend); CI test asserts cross-project isolation despite shared process.

**Phases (sketch).**

- **Phase 54 — OTel spans on cache + helper hot paths.** `tracing` + `tracing-opentelemetry` integration. Spans on every blob materialization, every `command=fetch`, every push attempt. Sampling configurable.
- **Phase 55 — `reposix tail` subcommand.** Streams audit table inserts (SQLite `update_hook` or polling fallback). Default human-readable, `--json` for piping. Dogfoodable for the Phase 57 dashboard.
- **Phase 56 — Multi-project helper process.** One `git-remote-reposix` invocation can serve `reposix::github/repo-a` and `reposix::github/repo-b` from one cache directory. Cross-project isolation enforced at the cache-key level. Required for v0.15.0+ plugin contributions where one helper hosts many backends.
- **Phase 57 — Project dashboard page.** Static page (or simple WASM) rendering audit-log rollups: pushes/day, blob-fetch rate, p99 latency by op, top contributors. Backed by the `reposix tail --json` stream.

## Origin-of-truth frontmatter enforcement

**Problem.** v0.13.0's bus remote handles "one ISSUES backend (SoT) + one plain-git mirror." That topology has only one place writes can land — confluence (or JIRA, or GH Issues, whichever is configured as SoT) — and the mirror is read-only from the SoT's perspective. There's no way to misroute a write because there's only one writable target.

The next topology — fanning out across **two ISSUES backends** (e.g., GH Issues + JIRA simultaneously, with confluence as a third reference SoT) — opens the misrouting failure mode. A record originally created in JIRA gets edited locally, gets pushed via a bus that fans out to both JIRA and GH Issues, and the helper has no signal that the record's "home" is JIRA. If the GH Issues fan-out succeeds and JIRA fails, the record now exists in two backends with conflicting versions and no clear winner.

**Sketch.** A frontmatter field that records the record's origin backend at create time and is checked at every push:

```yaml
---
id: 42
origin_backend: jira
title: ...
---
```

- The `BackendConnector::create_record` impl writes `origin_backend: <self.name>` into frontmatter at creation.
- The bus remote's push-time conflict detection extends with: *"for every record being written, check that `origin_backend` matches the target backend; if not, reject with `error: record 42 originated in jira, cannot push to gh-issues. To migrate origin, use `reposix migrate-origin <id> <new-backend>` (see docs)."*
- A new `reposix migrate-origin <id> <new-backend>` subcommand handles the legitimate migration case (e.g., decommissioning JIRA in favor of GH Issues): rewrites the field, audits the migration, requires confirmation flag.
- The frontmatter field allowlist (CLAUDE.md "Threat model" → "Frontmatter field allowlist") extends to treat `origin_backend` as server-controlled. Clients cannot rewrite it via `git push`; the migrate command is the only legitimate path.

**Why v0.14.0, not later.** Multi-backend bus is the natural follow-on once DVCS ships and one team starts wanting JIRA + GH Issues simultaneously. Without origin-of-truth enforcement, the first such deployment hits silent data corruption. v0.14.0 lands the guardrail BEFORE the multi-backend bus generalizes from 1+1 to 2+1 in a future milestone.

**Success gate.**
- `BackendConnector::create_record` impls in sim, confluence, GH Issues, JIRA all stamp `origin_backend`.
- Bus remote rejects mismatched pushes with the error message above; integration test covers the JIRA-record-pushed-to-GH-Issues failure path.
- `reposix migrate-origin` ships with audit row + dry-run mode + confirmation flag.
- Doc page `docs/concepts/origin-of-truth.md` explains the model, when migration is appropriate, and the failure modes the field protects against.

**Open questions for the planner.**

- `Q-OOT.1` Backfill policy for records that predate the field. Probably: helper assumes the SoT-of-record is whichever backend the record currently exists in; ambiguous if it exists in two. Migration tool's first run does a one-shot stamp.
- `Q-OOT.2` What happens when a record is intentionally cross-posted (rare but real — a PM wants the same issue tracked in JIRA and GH Issues for different audiences)? Probably: cross-posting is a "two records with the same content, different IDs" pattern, not a "one record with two origins" pattern. Document explicitly.
- `Q-OOT.3` Does the field interact with `Tainted<T>`? The value is server-controlled (set by `create_record`), so it's untainted by construction. But it IS read from frontmatter on push, so the read path needs to validate it didn't get edited offline. Treat the field as part of the existing server-controlled-fields allowlist.

## L2/L3 cache-desync hardening

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

## Mental model — three layers, one milestone

```
                              v0.14.0 milestone
                  ┌──────────────────────────────────────────┐
                  │                                          │
                  │  Observability (Phases 54-57)            │
                  │   ├─ OTel spans                          │
                  │   ├─ reposix tail                        │
                  │   ├─ multi-project helper                │
                  │   └─ dashboard page                      │
                  │                                          │
                  │  Origin-of-truth (Phases ~58-59)         │
                  │   ├─ frontmatter field                   │
                  │   ├─ bus-remote check                    │
                  │   └─ migrate-origin command              │
                  │                                          │
                  │  L2/L3 cache hardening (Phases ~60-62)   │
                  │   ├─ desync telemetry (Phase 5x)         │
                  │   ├─ L2 OR L3 decision                   │
                  │   └─ adapter rework (if L3)              │
                  │                                          │
                  │  +2 reservation slots (Phases ~63-64)    │
                  │   ├─ surprises absorption                │
                  │   └─ good-to-haves polish                │
                  │                                          │
                  └──────────────────────────────────────────┘
```

The three scopes are independently shippable but share a common substrate (the OTel work in Phase 54 powers the desync telemetry that gates L2-vs-L3, and the audit-table extensions for origin-of-truth use the same infrastructure as `reposix tail`). Sequencing matters: OTel first, then origin-of-truth (which can run in parallel with desync data collection), then the cache-hardening decision.

## Out of scope (explicitly deferred)

- **Plugin ecosystem / `BackendConnector` API stabilization.** The original v0.10.0-post-pivot/milestone-plan.md called this v0.14.0; with observability + origin-of-truth + cache hardening absorbing the milestone, plugin work moves to v0.15.0 or later. The phase-58 stub from `.planning/research/v0.10.0-post-pivot/milestone-plan.md` is a starting point, not a v0.14.0 commitment.
- **Bus remote with N > 2 endpoints.** Same deferral as v0.13.0 — the algorithm generalizes when a real third-endpoint use case appears. v0.14.0's origin-of-truth field unblocks this generalization but doesn't ship it.
- **Daemon-mode sync.** Webhook-driven CI is the v0.13.0 default and remains so in v0.14.0. A long-running daemon for backends without webhooks is deferred to whichever milestone first lands such a backend.
- **Unifying `audit_events` and `audit_events_cache` behind a `dyn AuditSink` trait (CC-3).** Tracked as a separate refactor; not blocked on v0.14.0 work and may slot into a polish phase elsewhere. The OTel work and `reposix tail` operate on the existing two-table schema.

## Risks and how we'll know early

- **OTel adds non-trivial overhead on the helper hot path.** Sampling-based mitigation is standard but `tracing-opentelemetry` is not free. **Early signal:** measure helper RPC turn-around with OTel enabled at 100% sample vs disabled, in the dark-factory regression. If overhead is > 10% at full sampling, document the recommended sample rate and ship with a sane default.
- **`origin_backend` enforcement breaks existing checkouts that predate the field.** Backfill is the answer (per Q-OOT.1) but the rollout matters. **Early signal:** prototype the migration in a sandbox space with deliberately-unstamped records; if backfill needs > 1 manual step from the user, redesign before shipping.
- **Cache desync incidence is too low to measure in a one-week window.** If Phase 5x's data is too sparse to drive the L2-vs-L3 decision, we're stuck. **Mitigation:** plan a fall-back of "ship L2 by default" if telemetry is inconclusive; L2 is cheap enough to ship without the data and L3 can layer on later if hot.
- **Multi-project helper (Phase 56) leaks state across projects.** Cross-project isolation is the load-bearing claim. **Mitigation:** the CI test in the success gate is non-negotiable; treat any cross-project leak as a security regression, not a feature bug.

## Tie-back to project-level invariants

- **OP-1 (simulator-first):** all v0.14.0 phases run end-to-end against the simulator. Two simulators in one process serve as "two ISSUES backends" for the origin-of-truth tests. Real-backend tests gate the milestone close, not individual phase closes.
- **OP-2 (tainted by default):** OTel span attributes and audit-event records carry tainted bytes from the backend. The dashboard page (Phase 57) MUST treat audit-event payload as tainted — escape on render, never inject into HTML directly. The `reposix tail --json` stream emits tainted bytes; the consumer is responsible for downstream sanitization.
- **OP-3 (audit log):** every cache repair (L2) or transactional rollback (L3) writes to `audit_events_cache`. Every `reposix migrate-origin` writes to both tables (cache audit for the helper turn, backend audit for the rewrite REST call).
- **OP-7 (verifier subagent grades GREEN):** every v0.14.0 phase close dispatches the verifier per `quality/PROTOCOL.md`. The L2-vs-L3 decision (Phase 5x telemetry → Phase 5x+1 implementation) is itself a verifier-graded artifact: the verdict file cites the desync-incidence data and explains the choice.
- **OP-8 (+2 phase practice):** v0.14.0 reserves its last two phases for surprises absorption + good-to-haves polish. Multi-scope milestones surface MORE surprises than single-thesis milestones (three intersecting designs); do not omit the +2 reservation.

## Where to start when you pick this up

1. Read this doc.
2. Read `.planning/research/v0.13.0-dvcs/vision-and-mental-model.md` and `.planning/research/v0.13.0-dvcs/architecture-sketch.md` (especially § "Performance subtlety: today's `list_records` walk on every push") for the L1 work that ships in v0.13.0.
3. Read `.planning/research/v0.10.0-post-pivot/milestone-plan.md` § "v0.13.0 — Observability & Multi-Repo" for the original observability scope (this doc's Phase 54-57 sketch is restated from there).
4. Skim `crates/reposix-cache/src/audit.rs` and `crates/reposix-core/src/audit.rs` — the dual-table audit schema is what `reposix tail` reads from.
5. Skim `crates/reposix-remote/src/main.rs::handle_export` for the per-record version-check that origin-of-truth extends.
6. Run `/gsd-new-milestone v0.14.0`. Hand the planner this doc as the primary input; the v0.13.0-dvcs sibling docs are secondary context.
