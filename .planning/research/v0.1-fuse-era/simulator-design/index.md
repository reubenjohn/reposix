# Simulator Design — `reposix-sim`

**Audience:** the agent(s) who will implement `crates/reposix-sim` tonight.
**Mode:** ecosystem + feasibility, code-heavy. Confidence: HIGH for axum / governor / rusqlite shapes, MEDIUM for Jira workflow semantics (modeled conservatively from public docs), HIGH for GitHub Issues semantics (validated against `docs.github.com/en/rest/issues/issues`, API version `2026-03-10`).
**North star:** the StrongDM dark-factory pattern from `docs/research/agentic-engineering-reference.md` §1. A swarm of agent-shaped clients hammers `/projects/{slug}/issues/...` overnight; the simulator must be **fast, free, deterministic, and faithful enough that bugs caught here would also occur in production.**

The defining tension is *fidelity vs. velocity*. Every behavior in §2 is non-negotiable because each one corresponds to a class of bug that would otherwise only surface against a real backend (where we have no credentials, no quota, no time). Everything else — pagination edge cases, custom field types, rich text rendering — is explicitly out of v0.1.

---

## 0. TL;DR for the impatient implementer

```
crates/reposix-sim/
├── Cargo.toml             # axum 0.7, tower-governor, rusqlite (bundled), serde, etc.
├── src/
│   ├── main.rs            # binary entrypoint: `reposix-sim --db sim.db --port 7878`
│   ├── lib.rs             # `pub fn build_router(state: AppState) -> Router`
│   ├── state.rs           # AppState { db: Arc<Mutex<Connection>>, limiters, config }
│   ├── routes/
│   │   ├── projects.rs    # GET /projects, GET /projects/{slug}
│   │   ├── issues.rs      # GET/POST/PATCH/DELETE /projects/{slug}/issues[/id]
│   │   ├── transitions.rs # GET /projects/{slug}/issues/{id}/transitions, POST .../transition
│   │   ├── perms.rs       # GET /projects/{slug}/permissions
│   │   └── dashboard.rs   # GET / -> embedded HTML, GET /_audit -> JSON for the UI
│   ├── middleware/
│   │   ├── audit.rs       # tower::Layer that writes one row per request
│   │   ├── rate_limit.rs  # GovernorLayer wrapper keyed on `X-Agent-Token`
│   │   ├── etag.rs        # If-Match / If-None-Match handling
│   │   ├── chaos.rs       # latency injection, fault injection, controlled by config
│   │   └── auth.rs        # bearer-token -> agent_id + role
│   ├── db/
│   │   ├── schema.sql     # embedded via include_str!
│   │   ├── seed.rs        # deterministic seeding from a u64 RNG seed
│   │   └── audit.rs       # append-only audit-log writer
│   ├── domain/
│   │   ├── issue.rs       # Issue, IssueState, IssuePatch
│   │   ├── workflow.rs    # transition table, validate(from, to) -> Result
│   │   └── rbac.rs        # Role, Permission, can(role, action)
│   └── ui/
│       └── index.html     # vibe-coded dashboard, ~200 lines, no build step
└── tests/
    ├── contract.rs        # property-style tests vs. real GitHub public API
    └── workflow.rs        # state-machine tests for transitions
```

Build target: a single `reposix-sim` binary with `--db`, `--port`, `--seed`, `--chaos`, `--rate-limit` flags. Boots in under 100 ms. SQLite file is the only on-disk state.

---

## Chapters

- [ch01 — Endpoint surface for v0.1](./ch01-endpoints.md)
- [ch02 — Behavior fidelity (dark-factory non-negotiables)](./ch02-behavior-fidelity.md)
- [ch03 — State persistence — SQLite + WAL + audit log](./ch03-persistence.md)
- [ch04 — Observability dashboard](./ch04-dashboard.md)
- [ch05 — Seed data](./ch05-seed-data.md)
- [ch06 — Multi-project / multi-tenant](./ch06-multi-project.md)
- [ch07 — Concrete axum skeleton](./ch07-axum-skeleton.md)
- [ch08 — Validating fidelity — contract harness vs. real GitHub](./ch08-contract-harness.md)
- [ch09 — Sources and roadmap implications](./ch09-sources-roadmap.md)
