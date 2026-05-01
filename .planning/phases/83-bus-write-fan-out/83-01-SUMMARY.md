# 83-01 Plan Summary — Bus remote write fan-out core

Plan 1 of 2 in phase 83 (THE RISKIEST PHASE per ROADMAP). Six sequential tasks, six atomic commits. Phase verifier graded GREEN at end of 83-02.

## Tasks shipped (6/6)

**T01 — Catalog-first.** Four `agent-ux` rows minted with `status: FAIL` BEFORE Rust:
- `agent-ux/bus-write-sot-first-success`
- `agent-ux/bus-write-mirror-fail-returns-ok` (held FAIL through 83-01 close per M1; flipped PASS at 83-02 T04)
- `agent-ux/bus-write-no-helper-retry`
- `agent-ux/bus-write-no-mirror-remote-still-fails`

Each with TINY shell verifier under `quality/gates/agent-ux/`.

**T02 — `apply_writes` refactor lift.** Lifted the `handle_export` write loop (lines 343-606 of `main.rs`) into shared `crates/reposix-remote/src/write_loop.rs::apply_writes` with narrow-deps signature `(cache, backend, backend_name, project, rt, proto, parsed: &ParsedExport)` per B1 fix. Both `handle_export` (single-backend) and `bus_handler::handle_bus_export` (bus path) call into it. NO `parsed.clone()` anywhere.

**T03 — Cache audit op + schema delta.** Added `helper_push_partial_fail_mirror_lag` to the `audit_events_cache` op CHECK list (P79/P80 precedent — no migration). Removed legacy stub `mirror_lag_partial_failure` per M2. New helper `audit::log_helper_push_partial_fail_mirror_lag` mirrors `log_helper_push_accepted` shape.

**T04 — bus_handler write fan-out.** Replaced P82's "P83 not yet shipped" emit with the SoT-first → mirror-best-effort algorithm:
- Call `apply_writes` for SoT writes.
- On SoT fail: bail; mirror unchanged; helper exits non-zero.
- On SoT success: run plain `git push <mirror_remote> main` subprocess (NO `--force-with-lease` per D-08).
- On mirror success: write `refs/mirrors/<sot>-synced-at`; emit `ok refs/heads/main`; cache audit row `helper_push_accepted`.
- On mirror failure: write `refs/mirrors/<sot>-head` only (NOT `synced-at`); cache audit row `helper_push_partial_fail_mirror_lag`; stderr warn; emit `ok refs/heads/main` anyway (SoT contract satisfied).

NO helper-side retry (Q3.6).

**T05 — Integration tests.** 2 happy-path/no-mirror integration tests:
- `tests/bus_write_happy.rs` — full SoT-first happy path; both refs updated; PATCH fired; audit rows in both tables.
- `tests/bus_write_no_mirror_remote.rs` — bus URL with no `git remote add` configured; STEP 0 in bus_handler bails BEFORE `ensure_cache`; no cache opened (Rule 1 fix narrowed assertion).

Two new helpers in `tests/common.rs`: `make_failing_mirror_fixture` + `count_audit_cache_rows`.

**T06 — Catalog flip + CLAUDE.md + close.** Runner-driven flip of 3 P83-01 rows FAIL→PASS (row 2 stays FAIL by design). CLAUDE.md updated: § Architecture bus write fan-out paragraph. Push to origin/main.

## Commits

| SHA | Subject |
|---|---|
| `3857f9a` | quality(agent-ux): mint 4 bus-write-core catalog rows + 4 TINY verifiers (catalog-first) |
| `76cf527` | refactor(reposix-remote): lift handle_export write loop into write_loop::apply_writes (P83 prelude) |
| `836dc6f` | feat(reposix-cache): add helper_push_partial_fail_mirror_lag audit op (DVCS-BUS-WRITE-02 OP-3) |
| `6978369` | feat(reposix-remote): bus_handler write fan-out replacing deferred-shipped stub (DVCS-BUS-WRITE-01..05) |
| `b2d18cc` | test(reposix-remote): bus write happy-path + no-mirror-remote regression integration tests |
| `4b7be9d` | quality(agent-ux): flip 3 P83-01 rows FAIL→PASS + CLAUDE.md update + dark-factory verifier broaden (close) |

## In-phase deviations (eager-resolution per OP-8)

1. **bus_precheck_b.rs test stale post-T04.** P82-era assertion expected the deferred-shipped stub `error refs/heads/main bus-write-not-yet-shipped`; T04 deleted that stub. Updated assertions to the new contract.
2. **Plan T05 §5c assertion 6 contradicted code.** Plan claimed `helper_backend_instantiated: 1` on no-remote-configured path; tracing showed STEP 0 bails BEFORE `ensure_cache`. Updated assertion to match architectural invariant: NO cache opened on bail-out.
3. **`dark-factory.sh` verifier regressed by T02 lift.** Plan moved `git pull --rebase` from main.rs to write_loop.rs; broadened the verifier shell's grep scope (commit 4b7be9d).
4. **Promoted ad-hoc bash JSON-validation to `scripts/p83-validate-catalog-rows.py`** per CLAUDE.md §4 (modes: `present|fail-init|p83-01-close|p83-02-close`).
