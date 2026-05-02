---
phase: 83
plan: 01
title: "DVCS-BUS-WRITE-01..05 — Bus remote write fan-out core (apply_writes refactor + cache audit op + bus_handler write fan-out + happy-path tests)"
wave: 1
depends_on: [80, 82]
requirements: [DVCS-BUS-WRITE-01, DVCS-BUS-WRITE-02, DVCS-BUS-WRITE-03, DVCS-BUS-WRITE-04, DVCS-BUS-WRITE-05]
files_modified:
  - crates/reposix-remote/src/write_loop.rs
  - crates/reposix-remote/src/main.rs
  - crates/reposix-remote/src/bus_handler.rs
  - crates/reposix-cache/src/audit.rs
  - crates/reposix-cache/src/cache.rs
  - crates/reposix-cache/fixtures/cache_schema.sql
  - crates/reposix-remote/tests/common.rs
  - crates/reposix-remote/tests/bus_write_happy.rs
  - crates/reposix-remote/tests/bus_write_no_mirror_remote.rs
  - quality/catalogs/agent-ux.json
  - quality/gates/agent-ux/bus-write-sot-first-success.sh
  - quality/gates/agent-ux/bus-write-mirror-fail-returns-ok.sh
  - quality/gates/agent-ux/bus-write-no-helper-retry.sh
  - quality/gates/agent-ux/bus-write-no-mirror-remote-still-fails.sh
  - CLAUDE.md
autonomous: true
mode: standard
---

# Phase 83 Plan 01 — Bus remote: write fan-out core (DVCS-BUS-WRITE-01..05)

## Objective

Land the SoT-first-write + mirror-best-effort fan-out core for the v0.13.0 bus remote. P82 shipped the read/dispatch surface (URL parser, prechecks A + B, capability branching) and ends in `emit_deferred_shipped_error` after both prechecks pass. P83-01 replaces that stub with the full algorithm:

1. **Read fast-import stream from stdin** (verbatim `parse_export_stream` from `handle_export`).
2. **Apply REST writes to SoT** via shared `write_loop::apply_writes` (lifted from `handle_export` lines 360-606). On success, write `helper_push_accepted` to `audit_events_cache`, advance `last_fetched_at`, derive `sot_sha` via `cache.refresh_for_mirror_head()`, write `refs/mirrors/<sot>-head`. On any SoT-fail, return `WriteOutcome::<variant>`.
3. **Push to mirror** via `Command::new("git").args(["push", mirror_remote_name, "main"])` shell-out (no `--force-with-lease` per D-08; no retry per Q3.6).
4. **Branch on (`WriteOutcome`, `MirrorResult`):**
   - SoT-success + mirror-success → write `synced-at` ref + write `mirror_sync_written` cache audit row + emit `ok refs/heads/main`.
   - SoT-success + mirror-fail → DO NOT write `synced-at` (frozen at last successful mirror sync) + write `helper_push_partial_fail_mirror_lag` cache audit row + stderr WARN + emit `ok refs/heads/main`.
   - SoT-fail → mirror push NEVER attempted; reject lines + audit rows already emitted inside `apply_writes`; return cleanly.

This is a **single plan, six sequential tasks**. Per CLAUDE.md "Build memory budget" the executor holds the cargo lock sequentially. T01 + T06 are doc-or-shell-only; no cargo.

## Chapters

- **[Threat Model](./01-threat-model.md)** — Trust boundaries and STRIDE register for the new mirror shell-out.

- **[Task T01: Catalog-first](./02-T01-catalog.md)** — Mint 4 catalog rows + 4 verifier shells.

- **[Task T02: write_loop refactor](./03-T02-write-loop.md)** — Lift `handle_export` into shared `apply_writes`.

- **[Task T03: Cache audit op](./04-T03-cache-audit.md)** — Schema delta + helper + wrapper + unit test.

- **[Task T04: Bus handler write fan-out](./05-T04-bus-handler.md)** — Replace stub with full algorithm.

- **[Task T05: Integration tests](./06-T05-tests.md)** — Happy-path + regression tests.

- **[Task T06: Phase close](./07-T06-close.md)** — Catalog flip + CLAUDE.md update + push.

