---
phase: 82
plan: 01
title: "DVCS-BUS-URL-01..02-PRECHECK + DVCS-BUS-FETCH-01 — Bus remote URL parser, prechecks, fetch dispatch"
wave: 1
depends_on: [81]
requirements: [DVCS-BUS-URL-01, DVCS-BUS-PRECHECK-01, DVCS-BUS-PRECHECK-02, DVCS-BUS-FETCH-01]
files_modified:
  - crates/reposix-remote/src/bus_url.rs
  - crates/reposix-remote/src/bus_handler.rs
  - crates/reposix-remote/src/precheck.rs
  - crates/reposix-remote/src/main.rs
  - crates/reposix-remote/tests/bus_url.rs
  - crates/reposix-remote/tests/bus_capabilities.rs
  - crates/reposix-remote/tests/bus_precheck_a.rs
  - crates/reposix-remote/tests/bus_precheck_b.rs
  - crates/reposix-remote/tests/common.rs
  - quality/catalogs/agent-ux.json
  - quality/gates/agent-ux/bus-url-parses-query-param-form.sh
  - quality/gates/agent-ux/bus-url-rejects-plus-delimited.sh
  - quality/gates/agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first.sh
  - quality/gates/agent-ux/bus-precheck-b-sot-drift-emits-fetch-first.sh
  - quality/gates/agent-ux/bus-fetch-not-advertised.sh
  - quality/gates/agent-ux/bus-no-remote-configured-error.sh
  - CLAUDE.md
autonomous: true
mode: standard
---

# Phase 82 Plan 01

Six sequential tasks (T01–T06) to implement the bus remote's read/dispatch surface for v0.13.0.

## Chapters

- **[Objective & Threat Model](./01-objective.md)** — Plan goal, architectural decisions, threat register.
- **[Task T01 — Catalog-first](./T01-catalog-first.md)** — 6 catalog rows + 6 verifier shells.
- **[Task T02 — bus_url.rs parser](./T02-bus-url-parser.md)** — Parser module + 4 unit tests.
- **[Task T03 — SoT-drift wrapper](./T03-sot-drift-wrapper.md)** — `precheck_sot_drift_any` + test.
- **[Task T04 — bus_handler module](./T04-bus-handler.md)** — Handler + main.rs wiring (split into subchapters).
- **[Task T05 — Integration tests](./T05-integration-tests.md)** — 4 test files (split into subchapters).
- **[Task T06 — Catalog flip & push](./T06-catalog-flip-push.md)** — FAIL → PASS flip + CLAUDE.md + push.
- **[References](./08-references.md)** — Canonical references.
- **[Plan-internal close](./09-plan-internal-close.md)** — Terminal actions.

## Quick Links

- **Specification:** `.planning/REQUIREMENTS.md` (DVCS-BUS-URL-01..DVCS-BUS-FETCH-01)
- **Architecture:** `.planning/research/v0.13.0-dvcs/architecture-sketch.md`
- **Research:** `.planning/phases/82-bus-remote-url-parser/82-RESEARCH.md`
