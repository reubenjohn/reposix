---
phase: 81
title: "L1 perf migration — `list_changed_since`-based conflict detection"
milestone: v0.13.0
requirements: [DVCS-PERF-L1-01, DVCS-PERF-L1-02, DVCS-PERF-L1-03]
depends_on: [80]
plans:
  - 81-01-PLAN.md  # DVCS-PERF-L1-01..03 (catalog → cache cursor + precheck wiring → sync subcommand → perf regression test + close)
waves:
  1: [81-01]
---

# Phase 81 — L1 perf migration (overview)

This is the THIRD DVCS-substantive phase of milestone v0.13.0 — the
"performance" leg that lands BEFORE the bus remote ships in P82–P83 so
the bus inherits the cheap conflict-detection path, NOT the expensive
one. Per `decisions.md` Q3.1 (RATIFIED inline L1) and the
architecture-sketch's strong recommendation: ship L1 as part of v0.13.0
or the bus remote's `git push` round-trip will do 100+ REST calls on
every push and dismiss reposix as a toy. **Single plan, four sequential
tasks** per RESEARCH.md § "Plan Splitting":

- **T01 — Catalog-first.** Three rows mint BEFORE any Rust edits: one in
  `quality/catalogs/perf-targets.json`
  (`perf/handle-export-list-call-count`), one in
  `quality/catalogs/agent-ux.json` (`agent-ux/sync-reconcile-subcommand`),
  one in `quality/catalogs/doc-alignment.json`
  (`docs-alignment/perf-subtlety-prose-bound`, minted via the
  `reposix-quality doc-alignment bind` verb — Principle A applies in
  the docs-alignment dim). Two new TINY shell verifiers under
  `quality/gates/perf/` and `quality/gates/agent-ux/`. Initial status
  `FAIL`. Hand-edit per documented gap (NOT Principle A) for the perf +
  agent-ux rows — same shape as P80's `agent-ux/mirror-refs-*` rows,
  tracked by GOOD-TO-HAVES-01.
- **T02 — Cache cursor wrappers + helper precheck rewrite.** Two new
  thin public methods on `Cache` (`read_last_fetched_at`,
  `write_last_fetched_at`) that wrap the existing
  `meta::get_meta`/`set_meta` calls keyed by `"last_fetched_at"` (NOT a
  new table). PLUS one new sync gix-only primitive
  `Cache::read_blob_cached(oid) -> Result<Option<Tainted<Vec<u8>>>>`
  (H1 fix from PLAN-CHECK 2026-05-01) — local-only inspector that
  returns `Ok(None)` on cache miss instead of fetching from backend.
  The precheck path uses this NOT the async `read_blob` (which would
  add a hidden backend GET per cache prior, defeating L1). Helper `handle_export` (lines 334–348 + 384–427 + 489–528
  in `crates/reposix-remote/src/main.rs` post-P80) gets the L1
  precheck-rewrite: read cursor → `list_changed_since(since)` → for each
  changed record AND in our push, parse cache prior + re-GET backend
  current → compare versions → reject on mismatch; otherwise `plan()`
  against cache-derived prior; on success update cursor. New free
  function `precheck_export_against_changed_set` lives in a new module
  `crates/reposix-remote/src/precheck.rs` so both `handle_export` (P81)
  and the future bus handler (P82–P83) consume the same code path —
  the SoT-precheck and the single-backend precheck are the same
  algorithm. L1-strict delete trade-off ratified in plan body (D-04
  below) and surfaced as an inline comment in `precheck.rs` citing
  `architecture-sketch.md § Performance subtlety` and the v0.14.0
  L2/L3 deferral target. First-push fallback: when `last_fetched_at`
  is `None`, fall through to the existing `list_records` walk for THIS
  push only, then write the cursor; subsequent pushes hit the L1 fast
  path.
- **T03 — `reposix sync --reconcile` CLI.** New `Sync { reconcile }`
  subcommand in `crates/reposix-cli/src/main.rs` + new module
  `crates/reposix-cli/src/sync.rs` (~30 lines: a thin wrapper over
  `Cache::build_from`). Without `--reconcile` the subcommand prints a
  one-line hint pointing at `--reconcile` (NOT a no-op error — the
  command exists in v0.13.0 specifically as the L1 escape hatch).
  Smoke test in `crates/reposix-cli/tests/sync.rs` (one
  `#[tokio::test]`) drives `reposix sync --reconcile` against the sim,
  asserts cache was rebuilt (`last_fetched_at` advanced).
- **T04 — Perf regression test + CLAUDE.md update + verifier flip +
  close.** New integration test
  `crates/reposix-remote/tests/perf_l1.rs` with N=200 records seeded
  in a wiremock mock server, asserts the precheck makes ≥1
  `list_changed_since` REST call AND **zero** `list_records` REST
  calls. Includes a positive-control test that flips `expect(0)` to
  `expect(1)` and confirms the matcher fails RED if reverted —
  closes RESEARCH.md MEDIUM risk "wiremock semantics need confirmation".
  Flip the three catalog rows FAIL → PASS via the runner before the
  per-phase push. CLAUDE.md update lands in the same commit (one
  paragraph in § Commands documenting `reposix sync --reconcile`; one
  paragraph in § Architecture naming the L1 cost-vs-correctness trade
  + pointing at `architecture-sketch.md`). `git push origin main`
  with pre-push GREEN. The orchestrator then dispatches the verifier
  subagent.

Sequential — never parallel. Even though T02 (cache crate + remote
crate) and T03 (cli crate) touch different crates, sequencing per
CLAUDE.md "Build memory budget" rule (one cargo invocation at a time,
never two in parallel) makes this strictly sequential.

## Wave plan

Strictly sequential — one plan, four tasks. T01 → T02 → T03 → T04
within the same plan body. The plan is its own wave.

| Wave | Plans  | Cargo? | File overlap        | Notes                                                                                    |
|------|--------|--------|---------------------|------------------------------------------------------------------------------------------|
| 1    | 81-01  | YES    | none with prior phase | catalog + cache wrappers + helper precheck rewrite + new CLI subcommand + perf test + close — all in one plan body |

`files_modified` audit (single-plan phase, no cross-plan overlap to
audit; line numbers cited at planning time and require re-confirmation
during T02 read_first):

| Plan  | Files                                                                                                                                                                                                                                                                          |
|-------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 81-01 | `crates/reposix-cache/src/cache.rs` (new methods on `impl Cache`), `crates/reposix-remote/src/main.rs` (lines 334–348 rewritten + 489–528 cursor write), `crates/reposix-remote/src/precheck.rs` (new), `crates/reposix-remote/src/lib.rs` or `mod precheck;` declaration in `main.rs`, `crates/reposix-remote/tests/perf_l1.rs` (new), `crates/reposix-cli/src/main.rs` (new `Sync` Cmd variant), `crates/reposix-cli/src/lib.rs` (new `pub mod sync;`), `crates/reposix-cli/src/sync.rs` (new), `crates/reposix-cli/tests/sync.rs` (new), `quality/catalogs/perf-targets.json` (1 new row), `quality/catalogs/agent-ux.json` (1 new row), `quality/catalogs/doc-alignment.json` (1 row bound via `reposix-quality doc-alignment bind`), `quality/gates/perf/list-call-count.sh` (new), `quality/gates/agent-ux/sync-reconcile-subcommand.sh` (new), `CLAUDE.md` |

Per CLAUDE.md "Build memory budget" the executor holds the cargo lock
sequentially across T02 → T03 → T04. No parallel cargo invocations.
Doc-only tasks (T01: catalog rows + 2 verifier shells; T04 epilogue:
CLAUDE.md edit) do NOT compile and may interleave freely with other
doc-only work outside this phase if the orchestrator schedules them —
but within this plan they remain sequential for executor simplicity.

## Plan summary table

| Plan  | Goal                                                                                                          | Tasks | Cargo? | Catalog rows minted | Tests added                                                                                                           | Files modified (count) |
|-------|---------------------------------------------------------------------------------------------------------------|-------|--------|----------------------|-----------------------------------------------------------------------------------------------------------------------|------------------------|
| 81-01 | L1 conflict detection (cache cursor + helper precheck rewrite + sync --reconcile CLI + perf regression test) | 4     | YES    | 3 (status FAIL → PASS at T04) | 2 unit (cursor read/write round-trip + cursor None-when-absent) + 1 cli smoke + 2 integration (perf regression positive + positive-control RED) = 5 total | ~14 (1 new precheck module + 1 new sync module + 1 new cli test + 1 new perf test + 2 new verifier shells + 3 catalog edits + cache.rs + main.rs + lib.rs + CLAUDE.md) |

Total: 4 tasks across 1 plan. Wave plan: sequential.

Test count: 2 unit cursor wrappers + 2 unit read_blob_cached (in `cache.rs` `#[cfg(test)] mod tests`) + 1
CLI smoke (`crates/reposix-cli/tests/sync.rs::sync_reconcile_advances_cursor`) + 2
integration (in `crates/reposix-remote/tests/perf_l1.rs::l1_precheck_uses_list_changed_since_not_list_records`
+ `positive_control_list_records_call_fails_red`) = 5 total.

## Chapters

- **[Decisions (D-01..D-05)](./decisions.md)** — Five open questions ratified at plan time: delete trade-off, sync subcommand shape, `plan()` signature, catalog home, CLAUDE.md scope.
- **[Subtle architectural points (S1, S2)](./architecture.md)** — INBOUND vs OUTBOUND cursor distinction; first-push fallback for `None` cursor.
- **[Hard constraints + Threat model](./constraints-and-threats.md)** — 11 hard constraints; STRIDE surface for three new P81 threat entries.
- **[Phase-close, risks, delegation, verification](./phase-close.md)** — Close protocol, risks table, +2 reservation, delegation table, verification approach.
