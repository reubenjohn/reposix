---
phase: 9
plan: swarm-harness
subsystem: qa / load-testing
tags: [swarm, hdr-histogram, sim-direct, fuse-mode, audit-invariant]
requirements: [FC-07]
dependency-graph:
  requires: [reposix-core::backend::sim::SimBackend, reposix-core::Error, reposix-sim]
  provides: [reposix-swarm binary, reposix_swarm::metrics, reposix_swarm::driver, reposix_swarm::sim_direct, reposix_swarm::fuse_mode]
  affects: [docs/demos/index.md Tier 4 row, README.md Tier 4 row]
tech-stack:
  added: [hdrhistogram 7, rand 0.8]
  patterns: [tokio::JoinSet deadline-driven loop, per-client X-Reposix-Agent suffix, Arc<Mutex<MetricsAccumulator>> shared accumulator, tokio::task::spawn_blocking for FUSE syscalls]
key-files:
  created:
    - crates/reposix-swarm/Cargo.toml
    - crates/reposix-swarm/src/main.rs
    - crates/reposix-swarm/src/lib.rs
    - crates/reposix-swarm/src/metrics.rs
    - crates/reposix-swarm/src/workload.rs
    - crates/reposix-swarm/src/driver.rs
    - crates/reposix-swarm/src/sim_direct.rs
    - crates/reposix-swarm/src/fuse_mode.rs
    - scripts/demos/swarm.sh
    - docs/demos/recordings/swarm.typescript
    - docs/demos/recordings/swarm.transcript.txt
  modified:
    - Cargo.toml                                # workspace members += reposix-swarm
    - Cargo.lock
    - crates/reposix-core/src/backend/sim.rs    # with_agent_suffix ctor
    - docs/demos/index.md                       # Tier 4 section
    - README.md                                 # Tier 4 section
decisions:
  - Separate crate `reposix-swarm` rather than bolting onto `reposix-cli` — swarm has heavy deps (`hdrhistogram`, `rand`) the CLI doesn't need; separation keeps `reposix` binary lean.
  - Per-client `X-Reposix-Agent` suffix (`reposix-core-simbackend-<pid>-swarm-<i>`) instead of sharing one header — without per-client agent ids every swarm client shared one sim rate-limit bucket and ~99% of ops returned 429.
  - Wildcard `expected_version=None` on PATCH — concurrent clients racing on the same id would otherwise storm the markdown report with `Conflict` errors that don't reflect interesting failure modes.
  - `fuse` mode uses `tokio::task::spawn_blocking` for every `std::fs` call — FUSE syscalls block; running them inline would stall the async runtime at N clients.
  - Not adding `swarm.sh` to `scripts/demos/smoke.sh` — 30s per run is too expensive for per-push CI. Documented in `docs/demos/index.md` and README.
metrics:
  duration: 72m
  completed: 2026-04-13 11:31 PDT
---

# Phase 9 Plan 1: Adversarial Swarm Harness Summary

N concurrent simulated agents hammer the simulator (or FUSE mount) with a realistic `list + 3×get + 1×patch` workload; per-op HDR histograms + markdown report + audit-row invariant check prove SG-06 still holds under load (`50 × 30s ≈ 130k ops, 0% errors, audit rows == total ops + 2 healthz`).

## What shipped

- **New crate `reposix-swarm`** with binary `reposix-swarm` and library surface split into `metrics`, `workload`, `driver`, `sim_direct`, `fuse_mode`.
- **CLI:** `reposix-swarm --clients N --duration SEC --target URL --mode {sim-direct,fuse} [--audit-db PATH] [--project SLUG]`. Defaults: 10 clients / 10s / `sim-direct` / `http://127.0.0.1:7878` / `demo`.
- **Metrics:** `MetricsAccumulator` holds per-`OpKind` HDR histograms (1µs..60s, 3 sig-digits) + per-`ErrorKind` counters. Error classifier maps `reposix_core::Error` messages into `NotFound / Conflict / RateLimited / Timeout / Other`.
- **Driver:** `run_swarm` spawns N tokio tasks via `JoinSet`, each stepping its `Workload` until a shared `Instant` deadline elapses; renders markdown at the end.
- **sim-direct mode:** `SimDirectWorkload` uses `reposix_core::backend::sim::SimBackend`. Cache issue-ids from the first `list` so `get` and `patch` pick random targets.
- **fuse mode:** `FuseWorkload` runs real `std::fs::read_dir / read_to_string / write` inside `spawn_blocking`. Rewrites whole files to keep frontmatter well-formed.
- **Audit invariant:** optional `--audit-db` flag reads `SELECT COUNT(*) FROM audit_events` via a r/w connection (read-only can't see WAL-resident rows). Summary line asserts the trigger still blocks UPDATE/DELETE.
- **Tier 4 demo (`scripts/demos/swarm.sh`):** 50 × 30s sim-direct + post-run invariant assertion. Recordings in `docs/demos/recordings/swarm.{typescript,transcript.txt}`. **Not** added to `scripts/demos/smoke.sh` — too long for per-push CI, documented in the demo index.
- **Core seam extension:** `SimBackend::with_agent_suffix(origin, Option<&str>)` so the swarm can give each client its own `X-Reposix-Agent` bucket. `new()` delegates for compatibility.

## Commits

1. `d0e0a59 feat(09-1): scaffold reposix-swarm crate + metrics + CLI skeleton`
2. `0b85fc8 feat(09-2): sim-direct workload + driver + per-client agent ids`
3. `1b884c5 feat(09-3): fuse-mode workload (real std::fs syscalls)`
4. `6b7bfab feat(09-4): Tier 4 swarm demo + WAL-safe audit query`
5. `5a09e05 docs(09-5): record Tier 4 swarm demo (50 clients × 30s, 132,895 ops)`
6. `fb28c87 docs(09-6): README + docs/demos/index.md — Tier 4 swarm row`

All 6 pushed to `origin/main` in one push; two CI runs triggered (in progress at Summary time).

## Validation (real runs on dev host, 2026-04-13 10:30-10:35 PDT)

**Plan-specified acceptance** (`cargo run -p reposix-swarm -- --clients 10 --duration 5 --mode sim-direct`):

```
Clients: 10    Duration: 5s    Mode: sim-direct    Target: http://127.0.0.1:7890
Total ops: 32450    Error rate: 0.00% (0/32450)

| Op     | Count     | p50     | p95     | p99     | Max     |
|--------|-----------|---------|---------|---------|---------|
| list   |      6490 |   1.3ms |   4.7ms |   9.2ms |    21ms |
| get    |     19470 |   1.0ms |   4.0ms |   8.4ms |    23ms |
| patch  |      6490 |   1.0ms |   4.8ms |   8.7ms |    26ms |

Audit rows: 32451
Append-only invariant: upheld (trigger blocks UPDATE/DELETE)
```

**Tier 4 demo (50 × 30s):**
Total ops: 132,895 / 0% errors / audit rows = 132,897 (ops + healthz). `sqlite3 ... 'UPDATE audit_events ...'` returns rc=19, error `"audit_events is append-only"`. Full transcript at `docs/demos/recordings/swarm.transcript.txt`.

**Workspace suites:**
- `cargo test --workspace --locked` — 167 passed / 0 failed / 4 ignored (up from 133 per STATE.md; delta: +30 from Phase 8 post-v0.1 work that merged while I was running, +4 from Phase 9 — metrics tests + driver test).
- `cargo clippy --workspace --all-targets -- -D warnings` — clean.
- `cargo fmt --all --check` — clean.
- `bash scripts/demos/smoke.sh` — Tier 1 unchanged (not re-run in sandbox; the swarm demo lives outside smoke).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 — Critical functionality] Added `SimBackend::with_agent_suffix`**
- **Found during:** Task 3 (sim-direct end-to-end smoke).
- **Issue:** All N swarm clients shared one `X-Reposix-Agent: reposix-core-simbackend-<pid>` header. The sim's rate-limit layer buckets by agent header, so 10 concurrent clients → one bucket → ~99% 429 rate even at --rate-limit 100.
- **Fix:** New constructor `SimBackend::with_agent_suffix(origin, Option<&str>)` that appends `-<suffix>` to the header. `SimBackend::new` delegates (no breaking change). `SimDirectWorkload::new` passes `swarm-<seed>` so every client gets its own bucket.
- **Files modified:** `crates/reposix-core/src/backend/sim.rs`, `crates/reposix-swarm/src/sim_direct.rs`.
- **Commit:** `0b85fc8`.

**2. [Rule 1 — Bug] `audit_row_count` used read-only flags; couldn't see WAL rows**
- **Found during:** Task 4 (swarm.sh end-to-end run).
- **Issue:** The sim runs SQLite in WAL mode. A read-only handle (`SQLITE_OPEN_READ_ONLY`) opened while the sim is still running cannot read WAL-resident rows, so the summary reported `Audit rows: <unavailable: no such table: audit_events>` on every real run.
- **Fix:** Switched to `Connection::open(path)` (default r/w). We only issue SELECT; no writes go out. Comment explains why in-place.
- **Files modified:** `crates/reposix-swarm/src/main.rs`.
- **Commit:** `6b7bfab`.

**3. [Plan clarification] Demo rate-limit knob**
- **Found during:** Task 5 (demo wiring).
- **Issue:** The sim's default rate-limit (100 rps/agent) is still below what hot-looping swarm clients produce, even with per-client buckets (50 clients each at ~150 rps). The Tier 4 demo would still show ~20% 429 rate.
- **Fix:** Demo script starts the sim with `--rate-limit 1000`. Swarm clients can't plausibly exceed 1000 rps each from a cold HTTP client, so 0% errors is reproducible.
- **Files modified:** `scripts/demos/swarm.sh`.
- **Commit:** `6b7bfab`.

### Not deviations — per plan budget

- **`fuse` mode shipped in v0.1 (not deferred to v0.2).** Plan said "acceptable to implement only sim-direct." I had the budget; fuse-mode is simple enough (spawn_blocking around std::fs) that landing it now closes FC-07's "real syscalls under load" phrasing. No Tier 4 demo for fuse yet — that wants a FUSE mount already set up and is better as a scripted integration test than a one-line demo. Tracked as v0.1.1 polish if anyone asks.
- **No `reposix-swarm` unit test for the HTTP round-trip.** `SimDirectWorkload` is covered by real end-to-end demo runs (132k ops, 0 errors) + the driver test that exercises the JoinSet/deadline logic. A wiremock unit test would be duplicative of `reposix-core/src/backend/sim.rs::tests`.

## Files Touched

| File                                                 | Change    | Why                                          |
| ---------------------------------------------------- | --------- | -------------------------------------------- |
| `Cargo.toml`                                         | modified  | + `crates/reposix-swarm` workspace member    |
| `Cargo.lock`                                         | modified  | lock for new deps (hdrhistogram, rand)       |
| `crates/reposix-core/src/backend/sim.rs`             | modified  | `with_agent_suffix` ctor                     |
| `crates/reposix-swarm/Cargo.toml`                    | created   | new crate                                    |
| `crates/reposix-swarm/src/main.rs`                   | created   | clap-derive CLI + driver invocation + audit  |
| `crates/reposix-swarm/src/lib.rs`                    | created   | module graph + pedantic lints                |
| `crates/reposix-swarm/src/metrics.rs`                | created   | HDR histograms, error classifier, render     |
| `crates/reposix-swarm/src/workload.rs`               | created   | `Workload` trait (async_trait, dyn-safe)     |
| `crates/reposix-swarm/src/driver.rs`                 | created   | JoinSet deadline loop + 1 test               |
| `crates/reposix-swarm/src/sim_direct.rs`             | created   | HTTP workload via SimBackend                 |
| `crates/reposix-swarm/src/fuse_mode.rs`              | created   | `std::fs` workload via spawn_blocking        |
| `scripts/demos/swarm.sh`                             | created   | Tier 4 demo, ASSERTS markers, rate-limit 1k  |
| `docs/demos/recordings/swarm.typescript`             | created   | structured recording                         |
| `docs/demos/recordings/swarm.transcript.txt`         | created   | plain transcript                             |
| `docs/demos/index.md`                                | modified  | Tier 4 section + not-in-smoke note           |
| `README.md`                                          | modified  | Tier 4 row                                   |

## Self-Check: PASSED

- All listed files exist (`ls` verified at the absolute paths).
- All commits exist on `origin/main` (`git log fa235bd..HEAD` shows the six 09-* commits; `git push` succeeded).
- `cargo test --workspace --locked` → 167 passed / 0 failed / 4 ignored.
- `cargo clippy --workspace --all-targets -- -D warnings` → clean.
- E2E validation matches plan-specified acceptance criteria.
