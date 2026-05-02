---
phase: 80
title: "Mirror-lag refs (`refs/mirrors/<sot>-head`, `<sot>-synced-at`)"
milestone: v0.13.0
requirements: [DVCS-MIRROR-REFS-01, DVCS-MIRROR-REFS-02, DVCS-MIRROR-REFS-03]
depends_on: [79]
plans:
  - 80-01-PLAN.md  # DVCS-MIRROR-REFS-01..03 (catalog → cache impl → helper wiring → integration tests + close)
waves:
  1: [80-01]
---

# Phase 80 — Mirror-lag refs (overview)

This is the SECOND DVCS-substantive phase of milestone v0.13.0 — the
"observability" leg of the bus-remote story. It lands BEFORE the bus
remote ships (P82–P83) so the refs are already in place when the bus
inherits the wiring point. **Single plan, four sequential tasks** per
RESEARCH.md § "Plan splitting":

- **T01 — Catalog-first.** Three rows in `quality/catalogs/agent-ux.json`
  (`mirror-refs-write-on-success`, `mirror-refs-readable-by-vanilla-fetch`,
  `mirror-refs-cited-in-reject-hint`) + three TINY shell verifiers under
  `quality/gates/agent-ux/`. Initial status `FAIL`. Hand-edit per
  documented gap (NOT Principle A) — same shape as P79's
  `agent-ux/reposix-attach-against-vanilla-clone` row, mints tracked by
  GOOD-TO-HAVES-01.
- **T02 — Cache crate impl.** New module `crates/reposix-cache/src/mirror_refs.rs`
  (writer + reader, mirroring `sync_tag.rs` shape verbatim); new
  `audit::log_mirror_sync_written`; pub mod + re-exports in `lib.rs`.
  Unit tests for writer/reader round-trip + annotated-tag message-body
  parsing. Per-crate cargo only (`cargo check -p reposix-cache`,
  `cargo nextest run -p reposix-cache`).
- **T03 — Helper crate wiring.** Insert two ref writes (head + synced-at)
  + audit-row write into `handle_export`'s success branch (lines 469–489
  per `crates/reposix-remote/src/main.rs` as of 2026-05-01); add
  `refs/mirrors/*` to the helper's stateless-connect ref advertisement;
  compose reject-hint stderr from `cache.read_mirror_synced_at` (None →
  omit hint cleanly per RESEARCH.md pitfall 7). Per-crate cargo only.
- **T04 — Integration tests + verifier flip + CLAUDE.md update + close.**
  Three integration tests in `crates/reposix-remote/tests/mirror_refs.rs`
  (one per catalog row); flip catalog rows FAIL → PASS via the runner;
  CLAUDE.md update (one paragraph in § Architecture / Threat model);
  per-phase `git push origin main`. Per-crate cargo only.

Sequential — never parallel. Even though T02 (cache) and T03 (helper)
touch different crates, sequencing per CLAUDE.md "Build memory budget"
rule (one cargo invocation at a time, never two in parallel) and per
RESEARCH.md § "Test fixture strategy" makes this strictly sequential.

## Wave plan

Strictly sequential — one plan, four tasks. T01 → T02 → T03 → T04
within the same plan body. The plan is its own wave.

| Wave | Plans  | Cargo? | File overlap        | Notes                                                                                    |
|------|--------|--------|---------------------|------------------------------------------------------------------------------------------|
| 1    | 80-01  | YES    | none with prior phase | catalog + cache crate + helper crate + integration tests + close — all in one plan body |

`files_modified` audit (single-plan phase, no cross-plan overlap to
audit):

| Plan  | Files                                                                                                                                                                                                                                                                          |
|-------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 80-01 | `crates/reposix-cache/src/mirror_refs.rs` (new), `crates/reposix-cache/src/audit.rs`, `crates/reposix-cache/src/lib.rs`, `crates/reposix-remote/src/main.rs`, `crates/reposix-remote/src/stateless_connect.rs`, `crates/reposix-remote/tests/mirror_refs.rs` (new), `quality/catalogs/agent-ux.json`, `quality/gates/agent-ux/mirror-refs-write-on-success.sh` (new), `quality/gates/agent-ux/mirror-refs-readable-by-vanilla-fetch.sh` (new), `quality/gates/agent-ux/mirror-refs-cited-in-reject-hint.sh` (new), `CLAUDE.md` |

Per CLAUDE.md "Build memory budget" the executor holds the cargo lock
sequentially across T02 → T03 → T04. No parallel cargo invocations.
Doc-only tasks (T01: catalog row + verifier shell scaffolding;
T04 epilogue: CLAUDE.md edit) do NOT compile and may interleave freely
with other doc-only work outside this phase if the orchestrator schedules
them — but within this plan they remain sequential for executor
simplicity.

## Plan summary table

| Plan  | Goal                                                                                       | Tasks | Cargo?  | Catalog rows minted        | Tests added                                                                                  | Files modified (count) |
|-------|--------------------------------------------------------------------------------------------|-------|---------|----------------------------|----------------------------------------------------------------------------------------------|------------------------|
| 80-01 | Mirror-lag refs (cache helpers + helper wiring + advertise + reject-hint + tests + close)  | 4     | YES     | 3 (status FAIL → PASS at T04) | 4 unit (writer/reader round-trip + annotated-tag message + read-none-when-absent + ref-name-validation) + 3 integration (one per catalog row) | ~11 (1 new cache module + 1 new test file + 3 new verifier shells + cache audit + helper wiring + advertise + catalog + CLAUDE.md) |

Total: 4 tasks across 1 plan. Wave plan: sequential.

Test count: 4 unit (in `mirror_refs.rs` `#[cfg(test)] mod tests`) + 3
integration (in `crates/reposix-remote/tests/mirror_refs.rs`) = 7 total.

## Chapters

- **[Architecture & constraints](./chapter-1-architecture.md)** — S1 (refs live in cache bare repo), S2 (sot_sha = post-write synthesis-commit OID), hard constraints, threat model crosswalk.
- **[Execution & close](./chapter-2-execution.md)** — Phase-close protocol, risks + mitigations, +2 reservation, subagent delegation + verifier criteria, verification approach.
