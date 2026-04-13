# Phase 2 — Wave plan

**Phase:** 02-simulator-audit-log
**Total plans:** 2
**Parallelism:** none — plans are strictly sequential.

## Waves

| Wave | Plan | Autonomous | Files modified |
|------|------|------------|----------------|
| 1 | `02-01-axum-handlers-and-storage.md` | yes | `crates/reposix-sim/{Cargo.toml,src/lib.rs,src/main.rs,src/state.rs,src/db.rs,src/seed.rs,src/error.rs,src/routes/{mod,issues,transitions}.rs,fixtures/seed.json}` |
| 2 | `02-02-audit-middleware-and-rate-limit.md` | yes | `crates/reposix-sim/{Cargo.toml,src/lib.rs,src/main.rs,src/middleware/{mod,audit,rate_limit}.rs,tests/api.rs}` |

## Why sequential (not parallel)

Plan 02-02 modifies files that plan 02-01 creates. Specifically, the shared
files are:

- `crates/reposix-sim/Cargo.toml` — plan 02-01 may touch it (workspace deps
  already satisfy 02-01 needs, but the plan lists it defensively); plan 02-02
  adds `sha2`, `hex`, and `tempfile`.
- `crates/reposix-sim/src/lib.rs` — plan 02-01 rewrites `build_router` and
  `SimConfig`; plan 02-02 attaches layers and adds `run_with_listener`.
- `crates/reposix-sim/src/main.rs` — plan 02-01 introduces the clap surface
  including `--rate-limit` (parsed but unused); plan 02-02 wires that flag into
  the layer.

File ownership overlap on `lib.rs`, `main.rs`, and `Cargo.toml` means 02-02's
wave must strictly follow 02-01's. Additionally, plan 02-02's middleware
imports `AppState` and the `db::open_db` function created by 02-01, and its
integration test exercises the routes registered by 02-01.

## Execution order

1. **Wave 1:** execute `02-01-axum-handlers-and-storage.md` end-to-end. Produce
   `02-01-SUMMARY.md` with the final AppState, ApiError, and seed schemas.
2. **Wave 2:** execute `02-02-audit-middleware-and-rate-limit.md`. Read
   `02-01-SUMMARY.md` first to resolve exact column names and types, then
   implement. Produce `02-02-SUMMARY.md`.
3. **Phase close:** run the goal-backward Bash harness in
   `02-02-audit-middleware-and-rate-limit.md`'s `<verification>` block. All
   five ROADMAP Phase-2 success criteria must print `ALL FIVE SUCCESS CRITERIA
   PASS`.

## Checkpoints

None. Both plans are fully autonomous (no `checkpoint:*` tasks). The final
goal-backward harness is an automated assertion, not a human-verify gate.
</content>
