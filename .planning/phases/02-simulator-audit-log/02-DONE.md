---
phase: 02-simulator-audit-log
status: complete
completed_at: 2026-04-13
commits:
  - 3c004f6  # feat(02-01): sim storage layer — AppState, db.rs, seed.json, ApiError
  - d29e47c  # feat(02-01): issue CRUD routes + transitions wired through AppState
  - 0eb6eb4  # feat(02-02): audit middleware + body capture + DB write
  - 171c775  # feat(02-02): rate-limit layer + run_with_listener + integration tests
plans_complete: 2/2
requirements_closed:
  - FC-01  # Simulator-first architecture
  - FC-06  # Audit log (SQLite, queryable) — writes half
  - SG-06  # Audit log append-only (enforced in sim via Phase-1 triggers)
tests:
  sim_lib_unit: 26
  sim_integration: 3
  total_sim: 29
  workspace_total: 107
roadmap_sc:
  SC1: PASS  # list returns >= 3
  SC2: PASS  # GET /projects/demo/issues/1 returns 200 + id + version
  SC3: PASS  # PATCH bogus If-Match returns 409
  SC4: PASS  # audit count grows; UPDATE trigger fires with 'append-only'
  SC5: PASS  # integration tests green on ephemeral port
---

# Phase 2: Simulator + Audit Log — DONE

Shipped in ~90 min across 4 commits. All 5 ROADMAP success criteria green
against a freshly-built binary (confirmed by
`scripts/phase2_goal_backward.sh`).

## Shipped files

### Core extension
- `crates/reposix-core/src/http.rs` — added
  `HttpClient::request_with_headers_and_body<U, B: Into<reqwest::Body>>`
  so integration tests can exercise the allowlist-gated PATCH+If-Match
  path without bypassing SG-01.

### Simulator

| File | Purpose |
|------|---------|
| `crates/reposix-sim/Cargo.toml` | +`sha2`, `hex`; dev-dep +`tempfile` |
| `crates/reposix-sim/src/lib.rs` | `SimConfig` extended, `build_router(state, rps)`, `prepare_state`, `run_with_listener`, `run` |
| `crates/reposix-sim/src/main.rs` | clap `--bind/--db/--seed-file/--no-seed/--ephemeral/--rate-limit` |
| `crates/reposix-sim/src/state.rs` | `AppState { db: Arc<Mutex<Connection>>, config: Arc<SimConfig> }` |
| `crates/reposix-sim/src/db.rs` | `open_db` (WAL + 5s busy_timeout + synchronous=NORMAL + audit schema load) |
| `crates/reposix-sim/src/seed.rs` | Deterministic `load_seed` / `apply_seed`; fixed 2026-04-13T00:00:00Z timestamps |
| `crates/reposix-sim/src/error.rs` | `ApiError { NotFound, BadRequest, VersionMismatch, Db, Json, Internal }` + `IntoResponse` |
| `crates/reposix-sim/src/routes/mod.rs` | Merges sub-routers |
| `crates/reposix-sim/src/routes/issues.rs` | GET list/get, POST create, PATCH update, DELETE |
| `crates/reposix-sim/src/routes/transitions.rs` | GET transitions |
| `crates/reposix-sim/src/middleware/mod.rs` | Middleware index |
| `crates/reposix-sim/src/middleware/audit.rs` | OUTERMOST: writes one audit row per req |
| `crates/reposix-sim/src/middleware/rate_limit.rs` | Per-agent governor bucket, 429+Retry-After |
| `crates/reposix-sim/fixtures/seed.json` | 3-issue demo project with adversarial bodies |
| `crates/reposix-sim/tests/api.rs` | 3 end-to-end integration tests |

### Scripts (dogfooding per CLAUDE.md §4)

- `scripts/phase2_smoke.sh` — replay SC1/SC2/SC3 against a running binary.
- `scripts/phase2_goal_backward.sh` — full SC1–SC5 harness (build + run +
  SQL assertions + integration tests).

### Planning

- `.planning/phases/02-simulator-audit-log/02-01-SUMMARY.md`
- `.planning/phases/02-simulator-audit-log/02-02-SUMMARY.md`
- `.planning/phases/02-simulator-audit-log/02-DONE.md` (this file)

## Commit SHAs

| Plan | Task | SHA | Message |
|------|------|-----|---------|
| 02-01 | 1 | `3c004f6` | feat(02-01): sim storage layer — AppState, db.rs, seed.json, ApiError |
| 02-01 | 2 | `d29e47c` | feat(02-01): issue CRUD routes + transitions wired through AppState |
| 02-02 | 1 | `0eb6eb4` | feat(02-02): audit middleware + body capture + DB write |
| 02-02 | 2 | `171c775` | feat(02-02): rate-limit layer + run_with_listener + integration tests |

Interleaved Phase 3 commits (from a parallel execution agent) sit between
my commits — they do not touch any Phase 2 file and the Phase 2 test
suite compiles & passes cleanly against the tree with Phase 3 scaffolding
present.

## Test counts

- `cargo test -p reposix-sim --lib`: **26 passed**
- `cargo test -p reposix-sim --test api`: **3 passed**
- `cargo test --workspace`: all green (workspace-wide 100+ tests)
- `cargo clippy --workspace --all-targets -- -D warnings`: clean
- `./scripts/phase2_goal_backward.sh`: **ALL FIVE SUCCESS CRITERIA PASS**

## Requirements closed

- **FC-01** Simulator-first architecture — real axum server at
  `127.0.0.1:7878`, JSON issue tracker, seeded `demo` project.
- **FC-06** Audit log (writes half) — every HTTP request records one
  `audit_events` row via the outermost middleware.
- **SG-06** Audit log append-only (enforcement half) — triggers from
  Phase-1 are loaded by `open_db`; integration test asserts a direct
  UPDATE fails with `"append-only"` in the error string.

## Deferred for Phase 3/S

- FC-03 (FUSE mount) — Phase 3 consumes this sim over HTTP.
- SG-01 enforcement at FUSE boundary — Phase 3 integration.
- `/dashboard` / `/audit` streaming endpoints — Phase 4 demo if time.

## Verification

One command replays the whole phase:

```bash
./scripts/phase2_goal_backward.sh
```

Expected final line: `ALL FIVE SUCCESS CRITERIA PASS`.
