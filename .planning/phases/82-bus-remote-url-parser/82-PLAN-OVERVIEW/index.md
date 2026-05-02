---
phase: 82
title: "Bus remote — URL parser, prechecks, fetch dispatch"
milestone: v0.13.0
requirements: [DVCS-BUS-URL-01, DVCS-BUS-PRECHECK-01, DVCS-BUS-PRECHECK-02, DVCS-BUS-FETCH-01]
depends_on: [81]
plans:
  - 82-01-PLAN.md  # DVCS-BUS-URL-01..02-PRECHECK + FETCH-01 (catalog → URL parser → coarser SoT precheck wrapper → bus_handler with PRECHECK A+B + capability branching → P83-not-yet-shipped exit + tests + close)
waves:
  1: [82-01]
---

# Phase 82 — Bus remote: URL parser, prechecks, fetch dispatch (overview)

This is the FOURTH DVCS-substantive phase of milestone v0.13.0 — the
read/dispatch surface of the bus remote. Per `decisions.md` Q3.3
(RATIFIED query-param URL form), Q3.4 (RATIFIED bus PUSH-only;
fetch via single-backend), Q3.5 (RATIFIED no-auto-mutate of git
config), and the architecture-sketch's bus algorithm steps 1–3:
recognize `reposix::<sot-spec>?mirror=<mirror-url>`, run two cheap
prechecks (mirror drift via `git ls-remote`; SoT drift via the P81
substrate's `list_changed_since`-driven check), and refuse the
`stateless-connect` capability so fetch falls through to the
single-backend code path. The WRITE fan-out (steps 4–9 of the bus
algorithm) is explicitly DEFERRED to P83 — P82 ends in a clean
"P83 not yet shipped" error after prechecks pass (Q-B in this plan).

**Single plan, six sequential tasks** per RESEARCH.md § "Plan
Splitting" — only TWO are cargo-heavy (T03 + T05); the other four
are doc/JSON/shell only.

- **T01 — Catalog-first.** Six rows mint BEFORE any Rust edits in
  `quality/catalogs/agent-ux.json` (the existing dimension home
  alongside `agent-ux/dark-factory-sim`,
  `agent-ux/reposix-attach-against-vanilla-clone`, `agent-ux/mirror-refs-*`,
  `agent-ux/sync-reconcile-subcommand` from P80/P81). Five rows track
  the four phase requirements + the no-mirror-configured success
  criterion (SC5). The sixth row covers the new shape `?mirror=` URL
  parses correctly. Six TINY shell verifiers under
  `quality/gates/agent-ux/`. Initial status `FAIL`. Hand-edited per
  documented gap (NOT Principle A) — same shape as P81's
  `agent-ux/sync-reconcile-subcommand` row, per GOOD-TO-HAVES-01.

- **T02 — `bus_url.rs` parser module + unit tests.** New file
  `crates/reposix-remote/src/bus_url.rs` with `pub(crate) enum Route
  { Single(ParsedRemote), Bus { sot: ParsedRemote, mirror_url: String }
  }` and `pub(crate) fn parse(url: &str) -> Result<Route>`. Strips
  the optional `?<query>` segment BEFORE delegating to
  `backend_dispatch::parse_remote_url(base)` (RESEARCH.md Pitfall 2);
  rejects `+`-delimited form with the verbatim hint citing
  `?mirror=`; rejects unknown query keys per Q-C; allows mirror URLs
  with embedded `?` only when percent-encoded (RESEARCH.md Pitfall 7).
  4 unit tests inline + 1 RFC-fuzz-style negative test for the `+`
  rejection.

- **T03 — Coarser SoT-drift precheck wrapper + unit test.** Append a
  10-line `pub(crate) fn precheck_sot_drift_any(...)` to
  `crates/reposix-remote/src/precheck.rs` returning
  `SotDriftOutcome { Drifted { changed_count: usize } | Stable }`.
  Reuses `cache.read_last_fetched_at()` (P81), calls
  `backend.list_changed_since(project, since)`, returns `Stable` on
  empty or no-cursor (first-push policy mirrors
  `precheck_export_against_changed_set`'s no-cursor path). Adds 1 unit
  test inside `precheck.rs`'s existing `#[cfg(test)] mod tests` block.

- **T04 — `bus_handler.rs` + `main.rs` dispatch wiring +
  capabilities branching.** New file `crates/reposix-remote/src/bus_handler.rs`
  with: `STEP 0` (resolve local mirror remote name by URL match per
  Q-A — `git config --get-regexp '^remote\..+\.url$'`; multi-match
  alphabetical-first + WARN per RESEARCH.md Pitfall 4); PRECHECK A
  (mirror drift via `git ls-remote -- <mirror_url> refs/heads/main`
  shell-out, with `--` defang per RESEARCH.md § Security; compare
  against `git rev-parse refs/remotes/<name>/main`); PRECHECK B
  (calls `precheck::precheck_sot_drift_any` from T03); on success
  emit the verbatim "P83 not yet shipped" error per Q-B and exit
  cleanly. `main.rs::real_main` widens to dispatch on `bus_url::parse`'s
  `Route` enum: `Single` continues to `parse_dispatch_url` +
  `instantiate` + existing `handle_export`; `Bus` builds the SoT
  backend via the same `instantiate` then routes to
  `bus_handler::handle_bus_export`. Capabilities branching (5-line
  edit at lines 150-172): `if matches!(route, Route::Single(_))
  { proto.send_line("stateless-connect")?; }`. Bus URL never
  advertises `stateless-connect` (DVCS-BUS-FETCH-01 closure).

- **T05 — Integration tests.** Four new test files under
  `crates/reposix-remote/tests/` — `bus_url.rs` (parser positive +
  rejection golden URL fixtures), `bus_capabilities.rs` (asserts
  capability list omits `stateless-connect` for bus URL), `bus_precheck_a.rs`
  (file:// bare-repo fixture; drifted local mirror ref triggers
  `error refs/heads/main fetch first`), `bus_precheck_b.rs`
  (wiremock-backed sim with seeded `last_fetched_at` cursor;
  `list_changed_since` returns non-empty → `error refs/heads/main
  fetch first`). Each test file closes ONE catalog row.

- **T06 — Catalog flip + CLAUDE.md update + per-phase push.** Run
  `python3 quality/runners/run.py --cadence pre-pr` to flip the 6 rows
  FAIL → PASS. CLAUDE.md update lands in the same commit (one
  paragraph in § Architecture documenting the bus URL form
  `reposix::<sot>?mirror=<mirror>` + Q3.4 PUSH-only contract; one
  bullet in § Commands showing the new push form `git push reposix
  main`). `git push origin main` with pre-push GREEN. The
  orchestrator then dispatches the verifier subagent.

Sequential — never parallel. T01 → T02 → T03 → T04 → T05 → T06.
Even though T02 (bus_url) and T03 (precheck wrapper) touch different
files in the same crate, sequencing per CLAUDE.md "Build memory
budget" rule (one cargo invocation at a time) makes this strictly
sequential.

## Wave plan

Strictly sequential — one plan, six tasks. T01 → T02 → T03 → T04 →
T05 → T06 within the same plan body. The plan is its own wave.

| Wave | Plans  | Cargo? | File overlap        | Notes                                                                                    |
|------|--------|--------|---------------------|------------------------------------------------------------------------------------------|
| 1    | 82-01  | YES (T03+T05)    | none with prior phase | catalog + URL parser + SoT-drift wrapper + bus_handler + capabilities branching + 4 integration tests + close — all in one plan body |

`files_modified` audit (single-plan phase, no cross-plan overlap to
audit; line numbers cited at planning time and require re-confirmation
during T04 read_first):

| Plan  | Files                                                                                                                                                                                                                                                                          |
|-------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 82-01 | `crates/reposix-remote/src/bus_url.rs` (new), `crates/reposix-remote/src/bus_handler.rs` (new), `crates/reposix-remote/src/precheck.rs` (append `precheck_sot_drift_any` + `SotDriftOutcome`), `crates/reposix-remote/src/main.rs` (mod declarations + URL-route dispatch + capabilities branching), `crates/reposix-remote/tests/bus_url.rs` (new), `crates/reposix-remote/tests/bus_capabilities.rs` (new), `crates/reposix-remote/tests/bus_precheck_a.rs` (new), `crates/reposix-remote/tests/bus_precheck_b.rs` (new), `crates/reposix-remote/tests/common.rs` (new — P81 M3 gap copy from `crates/reposix-cache/tests/common/mod.rs`), `quality/catalogs/agent-ux.json` (6 new rows), `quality/gates/agent-ux/bus-url-parses-query-param-form.sh` (new), `quality/gates/agent-ux/bus-url-rejects-plus-delimited.sh` (new), `quality/gates/agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first.sh` (new), `quality/gates/agent-ux/bus-precheck-b-sot-drift-emits-fetch-first.sh` (new), `quality/gates/agent-ux/bus-fetch-not-advertised.sh` (new), `quality/gates/agent-ux/bus-no-remote-configured-error.sh` (new), `CLAUDE.md` |

Per CLAUDE.md "Build memory budget" the executor holds the cargo lock
sequentially across T03 → T05. T01 + T02 + T04 (source-file edits)
need only one `cargo check -p reposix-remote` between them. T06
runs the catalog runner (no cargo) + a final test sweep (`cargo
nextest run -p reposix-remote`). No parallel cargo invocations.

## Plan summary table

| Plan  | Goal                                                                                                          | Tasks | Cargo? | Catalog rows minted | Tests added                                                                                                           | Files modified (count) |
|-------|---------------------------------------------------------------------------------------------------------------|-------|--------|----------------------|-----------------------------------------------------------------------------------------------------------------------|------------------------|
| 82-01 | Bus URL parser + STEP 0 + PRECHECK A + PRECHECK B + capability branching + bus-write deferred-error stub      | 6     | YES (T03+T05) | 6 (status FAIL → PASS at T06) | 4 unit (parse positive, reject `+`, reject unknown key, no-cursor stable) + 1 unit precheck wrapper + 4 integration (bus_url, bus_capabilities, bus_precheck_a, bus_precheck_b) = 9 total | ~16 (2 new modules + 4 new test files + 6 new verifier shells + 1 catalog edit + main.rs + precheck.rs + CLAUDE.md) |

Total: 6 tasks across 1 plan. Wave plan: sequential.

Test count: 4 unit `bus_url::parse` + 1 unit `precheck_sot_drift_any`
(in `precheck.rs` `#[cfg(test)] mod tests`) + 4 integration tests
(`tests/bus_url.rs::*`, `tests/bus_capabilities.rs::*`,
`tests/bus_precheck_a.rs::*`, `tests/bus_precheck_b.rs::*`) = 9 total.

## Chapters

- **[Decisions D-01..D-06](./decisions.md)** — Six ratified decisions: remote-name lookup (D-01/Q-A), clean deferred-error for P83 (D-02/Q-B), unknown query-param rejection (D-03/Q-C), catalog home (D-04), shared BackendConnector pipeline (D-05), `git ls-remote` shell-out (D-06).
- **[Subtle architectural points S1–S2](./architecture.md)** — Capability branching as a 5-line edit (S1); why stdin must not be read before prechecks fire (S2).
- **[Hard constraints and threat model](./constraints-and-threats.md)** — 14 hard constraints + STRIDE crosswalk for the three new shell-out surfaces.
- **[Phase management](./phase-management.md)** — Phase-close protocol, risks + mitigations, +2 reservation candidates, subagent delegation, verification approach.
