---
phase: 80
plan: 01
title: "DVCS-MIRROR-REFS-01..03 — mirror-lag refs (cache helpers + helper wiring + reject-hint + tests + close)"
wave: 1
depends_on: [79]
requirements: [DVCS-MIRROR-REFS-01, DVCS-MIRROR-REFS-02, DVCS-MIRROR-REFS-03]
files_modified:
  - crates/reposix-cache/src/mirror_refs.rs
  - crates/reposix-cache/src/audit.rs
  - crates/reposix-cache/src/lib.rs
  - crates/reposix-remote/src/main.rs
  - crates/reposix-remote/src/stateless_connect.rs
  - crates/reposix-remote/tests/mirror_refs.rs
  - quality/catalogs/agent-ux.json
  - quality/gates/agent-ux/mirror-refs-write-on-success.sh
  - quality/gates/agent-ux/mirror-refs-readable-by-vanilla-fetch.sh
  - quality/gates/agent-ux/mirror-refs-cited-in-reject-hint.sh
  - CLAUDE.md
autonomous: true
mode: standard
---

# Phase 80 Plan 01 — Mirror-lag refs (DVCS-MIRROR-REFS-01..03)

## Chapters

- **[Objective & Architecture Overview](./01-objective.md)** — goal, task breakdown, and executor context (architecture + ref shapes + hint template).
- **[Trust Boundaries & Threat Model](./02-threat-model.md)** — threat boundaries and STRIDE register.
- **[Task T01: Catalog-first](./03-T01-catalog-first.md)** — mint 3 agent-ux rows + author 3 verifier shells (FAIL status).
- **[Task T02: Cache impl](./04-T02-cache-impl.md)** — `mirror_refs.rs` + `audit::log_mirror_sync_written` + `lib.rs` re-export + 4 unit tests.
- **[Task T03: Helper wiring](./05-T03-helper-wiring.md)** — ref writes in `handle_export` success branch + reject-hint composition + advertisement widening.
- **[Task T04: Integration & close](./06-T04-integration-tests.md)** — integration tests + verifier flip + CLAUDE.md update + per-phase push (terminal).
- **[Must-haves & Requirements](./07-must-haves.md)** — detailed specification of all deliverables.
- **[Canonical References](./08-canonical-refs.md)** — source documents and code precedents.

> **Revision note (2026-05-01).** This plan was revised in response to
> `PLAN-CHECK.md` (verdict YELLOW). Applied: H1 (sim-port routing fix in
> 3 verifier shells + Rust integration helper), H2 (corrected gix 0.83
> `Repository::tag` invocation), H3 (replaced vacuous
> `reject_hint_first_push_omits_synced_at_line` with sim-seeded
> first-push conflict + documented path B unit-test fallback), M1
> (concrete `cargo add --dev` step for `walkdir` + `regex`), M2
> (cache.db path corrected), M3 (Q2.2 verbatim phrase carrier
> disambiguation note), M4 (verifier shells now pick a free
> high-range port instead of fixed 7900–7902). LOW issues L1–L5
> intentionally not addressed (stylistic per PLAN-CHECK.md
> § Recommendation).
