# Observability scope (pulled forward from v0.10.0-post-pivot)

[← index](./index.md)

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
