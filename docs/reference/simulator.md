---
title: Simulator reference — reposix-sim
---

# Simulator reference

`reposix-sim` is the in-process axum HTTP server that backs every demo, unit test, and autonomous agent loop in the project. It speaks the same wire shape that the real backends (GitHub, Confluence, JIRA) speak — far enough that `BackendConnector` implementations can swap between sim and real without code changes.

The simulator is the **default** backend per project [Operating Principle](../research/agentic-engineering-reference.md) "simulator-first". Real backends are gated behind `REPOSIX_ALLOWED_ORIGINS` and explicit credential env vars (see [Testing targets](testing-targets.md)).

## What it is

- A standalone binary at `crates/reposix-sim/` (`cargo run -p reposix-sim`).
- An axum 0.7 router exposing the issue-tracker REST shape (`/projects/<slug>/issues`, `/issues/<id>`, etc.) plus a `/healthz` probe.
- A SQLite store (bundled `rusqlite`, WAL mode) holding the issues table and the append-only `audit_events` table.
- A deterministic seed loader (`crates/reposix-sim/fixtures/seed.json`) that lets every test run from a known starting state.

Latency envelope on the dev host: `9 ms` to list issues, `8 ms` to get one issue, `8 ms` to PATCH one — see [latency](../benchmarks/latency.md). The sim is the lower bound for transport overhead; treat real-backend numbers as the upper bound.

## Run it

```bash
cargo run -p reposix-sim
# 2026-04-24T12:00:00.000Z  INFO reposix_sim: reposix-sim listening addr=127.0.0.1:7878
```

The default bind is `127.0.0.1:7878`. The `reposix init sim::<slug>` command wires that address into the helper URL — keep the port if you want `reposix init` to "just work" with no extra flags.

To seed the canonical demo project at startup:

```bash
cargo run -p reposix-sim -- \
    --seed-file crates/reposix-sim/fixtures/seed.json
```

The fixture defines a single project (`demo`) with five issues; that's the project the [first-run tutorial](../tutorials/first-run.md) targets.

## Configuration

| Flag | Default | Purpose |
|------|---------|---------|
| `--bind` | `127.0.0.1:7878` | Listen address. Use `127.0.0.1:0` for an ephemeral port (tests). |
| `--db` | `runtime/sim.db` | SQLite file path. Created if absent. Ignored when `--ephemeral` is set. |
| `--seed-file` | — | Path to a seed JSON. Without one, the DB starts empty. |
| `--no-seed` | off | Skip seeding even if `--seed-file` is provided. |
| `--ephemeral` | off | Use `:memory:` SQLite — every restart is a clean slate. |
| `--rate-limit` | `100` | Per-agent request quota (req/sec). Tune down to exercise rate-limit code paths. |

There is no auth on the sim. Anyone who can reach the bind address can read and write issues — that's deliberate (it removes a category of test-only ceremony) and it's why the sim should never bind to a non-loopback address.

## Where the data lives

- **Sim DB** — `runtime/sim.db` by default, `:memory:` when `--ephemeral`. This holds the issues table and the sim's own `audit_events` table (one row per HTTP request through the audit middleware).
- **Cache DB (helper-side)** — `<XDG_CACHE_HOME>/reposix/sim-<project>.git/cache.db` (or `<root>` from `REPOSIX_CACHE_DIR`). This is the helper's audit log — what reposix did against the sim — separate from the sim's own audit of incoming HTTP. Both tables are append-only via `BEFORE UPDATE/DELETE RAISE` triggers.

The two audit logs answer two different questions: the sim's `audit_events` shows what hit the sim; the cache's `audit_events_cache` shows what the helper attempted. They're disjoint by design — see the [trust model](../how-it-works/trust-model.md) for the full ops vocabulary.

## REST surface

Routes the sim implements (full reference in `crates/reposix-sim/src/routes/`):

- `GET /healthz` — liveness probe.
- `GET /projects` / `GET /projects/<slug>` — project listing + lookup.
- `GET /projects/<slug>/issues[?since=<rfc3339>]` — list issues, with optional `since` for delta sync (matches the `BackendConnector::list_changed_since` shape).
- `GET /projects/<slug>/issues/<id>` — single issue.
- `POST /projects/<slug>/issues` — create.
- `PATCH /projects/<slug>/issues/<id>` — update; supports `If-Match: "<version>"` for optimistic concurrency.
- `DELETE /projects/<slug>/issues/<id>` — real delete.

The shape is documented in the [HTTP API reference](http-api.md); the sim is the canonical implementation of that document.

## Limitations (deliberate)

- **No real auth.** No tokens, no OAuth dance. Use `REPOSIX_ALLOWED_ORIGINS` to keep traffic on loopback.
- **No rate limiting beyond a per-agent token bucket.** No back-pressure that mirrors GitHub's `x-ratelimit-remaining` or Atlassian's `Retry-After`.
- **No webhooks, no streaming.** The sim is a request/response server. Subscribe-style sync against real backends is not modelled.
- **No multi-tenant isolation.** One sim process serves one DB. Spawn separate processes if you need separate state for separate test runs.
- **Tainted by default.** Even though the sim is local, every byte returned counts as untrusted input — seeds are authored by agents and the simulator is part of the trifecta surface. The [trust model](../how-it-works/trust-model.md) covers why.

## Where it's used

- The `cargo test --workspace` suite spins ephemeral sims via `run_with_listener` on `127.0.0.1:0`.
- `scripts/dark-factory-test.sh sim` runs the [`reposix-agent-flow`](../guides/integrate-with-your-agent.md) regression against a fresh sim.
- The [first-run tutorial](../tutorials/first-run.md) is the user-facing entrypoint.
- Latency capture (`scripts/latency-bench.sh`) measures sim numbers as the lower bound published in [latency](../benchmarks/latency.md).

## See also

- [HTTP API reference](http-api.md) — full route + payload spec.
- [First-run tutorial](../tutorials/first-run.md) — five-minute walkthrough against the sim.
- [Testing targets](testing-targets.md) — how to graduate from sim to a real backend.
- [Trust model](../how-it-works/trust-model.md) — the simulator is treated as tainted input despite being local.
