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

## Chapters

- **[Observability scope](./observability-scope.md)** — Phases 54–57; OTel, `reposix tail`, multi-project helper, dashboard.
- **[Origin-of-truth frontmatter enforcement](./origin-of-truth.md)** — `origin_backend` field, bus-remote check, migrate-origin, Q-OOT.1–Q-OOT.3.
- **[L2/L3 cache-desync hardening](./cache-desync-hardening.md)** — L2 vs L3 decision rule, phase sketch, Q-CD.1–Q-CD.3.
- **[Risks and invariants](./risks-and-invariants.md)** — Risks with early signals; OP-1–OP-8 tie-back.

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

## Where to start when you pick this up

1. Read this doc.
2. Read `.planning/research/v0.13.0-dvcs/vision-and-mental-model.md` and `.planning/research/v0.13.0-dvcs/architecture-sketch.md` (especially § "Performance subtlety: today's `list_records` walk on every push") for the L1 work that ships in v0.13.0.
3. Read `.planning/research/v0.10.0-post-pivot/milestone-plan.md` § "v0.13.0 — Observability & Multi-Repo" for the original observability scope (this doc's Phase 54-57 sketch is restated from there).
4. Skim `crates/reposix-cache/src/audit.rs` and `crates/reposix-core/src/audit.rs` — the dual-table audit schema is what `reposix tail` reads from.
5. Skim `crates/reposix-remote/src/main.rs::handle_export` for the per-record version-check that origin-of-truth extends.
6. Run `/gsd-new-milestone v0.14.0`. Hand the planner this doc as the primary input; the v0.13.0-dvcs sibling docs are secondary context.
