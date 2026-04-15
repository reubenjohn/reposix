---
phase: 21
plan: B
subsystem: reposix-swarm
tags: [hardening, swarm, contention, if-match, HARD-01]
dependency_graph:
  requires: [21-A]
  provides: [Mode::Contention CLI mode, ContentionWorkload, contention_e2e integration test]
  affects: [crates/reposix-swarm]
tech_stack:
  added: []
  patterns: [If-Match optimistic concurrency, swarm workload trait, TDD red-green]
key_files:
  created:
    - crates/reposix-swarm/src/contention.rs
    - crates/reposix-swarm/tests/contention_e2e.rs
  modified:
    - crates/reposix-swarm/src/lib.rs
    - crates/reposix-swarm/src/main.rs
decisions:
  - "ContentionWorkload uses GET-then-PATCH-with-Some(version) pattern; each client races independently (no cross-client synchronisation), guaranteeing intentional races"
  - "IssueId(1) used directly (public tuple struct field) — no ::new() constructor exists in reposix-core"
  - "map_or(false, ...) replaced with is_some_and(...) per clippy::unnecessary_map_or (Rust 1.82+)"
metrics:
  duration: ~8 minutes
  completed: 2026-04-15T17:49:24Z
  tasks_completed: 2
  files_modified: 4
---

# Phase 21 Plan B: Contention Swarm Mode Summary

**One-liner:** `Mode::Contention` workload with N-client If-Match storm proving sim's 409 path is deterministic and torn-write-free (HARD-01).

## What Was Built

### New Files

**`crates/reposix-swarm/src/contention.rs`** (125 lines)
- `ContentionWorkload` struct with `SimBackend`, fixed `target_id: IssueId`, per-client `StdRng`
- Constructor: `ContentionWorkload::new(origin, project, target_id, seed) -> anyhow::Result<Self>`
- `Workload::step`: GET current version → PATCH with `Some(version)` → win recorded as `OpKind::Patch`; 409 Conflict recorded as `ErrorKind::Conflict`; GET failures are non-fatal (logged in metrics, step returns `Ok(())`)
- Follows exact analog pattern from `sim_direct.rs`: `SimBackend::with_agent_suffix`, `elapsed_us` helper, `sanitize(Tainted::new(...), ServerMetadata {...})`

**`crates/reposix-swarm/tests/contention_e2e.rs`** (153 lines)
- 50 clients, 5-second duration, rate limit 200 rps per agent bucket
- Asserts: `| patch |` row present, Conflict errors present in "Errors by class" section, no `| Other` errors, audit log has ≥ 1 row
- Uses shared `spawn_sim` + `audit_row_count` helpers copied verbatim from `mini_e2e.rs`

### Modified Files

**`crates/reposix-swarm/src/lib.rs`**: added `pub mod contention;` alphabetically between `confluence_direct` and `driver`.

**`crates/reposix-swarm/src/main.rs`**:
- `Mode::Contention` variant added to enum with `as_str() -> "contention"`
- `--target-issue <u64>` arg (`Option<u64>`) added to `Args`
- Dispatch arm: validates `--target-issue` is present, constructs `IssueId(target_id_raw)`, calls `run_swarm` factory with `ContentionWorkload::new`
- `use reposix_swarm::contention::ContentionWorkload;` import added

## CLI Invocation Example

```bash
# Start sim first:
cargo run -p reposix-sim -- --db /tmp/demo.db --seed

# Run contention mode (50 clients, 30s, issue id 1):
cargo run -p reposix-swarm -- \
  --mode contention \
  --target http://127.0.0.1:7878 \
  --project demo \
  --target-issue 1 \
  --clients 50 \
  --duration 30 \
  --audit-db /tmp/demo.db
```

## Test Count Delta

| Before | After | Delta |
|--------|-------|-------|
| (swarm unit) 4 | 4 | 0 |
| (swarm integration) 2 | 3 | +1 |
| (workspace total) ~317 | ~318 | +1 |

## Verification

- `cargo test -p reposix-swarm --locked` — all 7 tests pass (4 unit + 3 integration)
- `cargo clippy -p reposix-swarm --all-targets -- -D warnings` — clean
- `cargo test --workspace` — all pass, zero failures
- `cargo run -p reposix-swarm -- --help | grep contention` — mode is discoverable

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `IssueId::new(1)` constructor does not exist**
- **Found during:** Task B1 (compile check)
- **Issue:** Plan template used `IssueId::new(1)` but `IssueId` is `pub struct IssueId(pub u64)` — a public newtype with no named constructor
- **Fix:** Used `IssueId(1)` direct tuple struct construction
- **Files modified:** `crates/reposix-swarm/tests/contention_e2e.rs`

**2. [Rule 1 - Lint] `doc_markdown` clippy pedantic lint**
- **Found during:** Task B2 clippy run
- **Issue:** `step()` in doc comment needed backticks
- **Fix:** Changed to `\`step()\``
- **Files modified:** `crates/reposix-swarm/src/contention.rs`

**3. [Rule 1 - Lint] `unnecessary_map_or` clippy lint**
- **Found during:** Task B2 clippy run
- **Issue:** `.map_or(false, |s| ...)` replaced with `.is_some_and(|s| ...)`
- **Fix:** Applied clippy suggestion
- **Files modified:** `crates/reposix-swarm/tests/contention_e2e.rs`

### TDD Note

The test passed immediately on first run (GREEN without a RED phase). The `ContentionWorkload` implementation and test were written together as a complete unit; the sim's 409 path was already correct so no RED iteration was needed. Both the workload and test are committed together in the B1 commit.

## Commits

| Hash | Message |
|------|---------|
| `bfe107d` | `feat(21-B): ContentionWorkload module + contention_e2e integration test` |
| `f6d1e63` | `feat(21-B): Mode::Contention + ContentionWorkload wired end-to-end (HARD-01)` |

## Threat Surface Scan

No new trust boundaries introduced. `ContentionWorkload` uses the same `SimBackend` → `http://127.0.0.1:*` allowlist as `SimDirectWorkload`. The `sanitize(Tainted::new(...), ServerMetadata {...})` path ensures server-controlled fields (`id`, `created_at`, `version`) cannot be overwritten by client-side mutations (T-21-B-02 mitigated as planned).

## Self-Check: PASSED

- `crates/reposix-swarm/src/contention.rs` — exists (125 lines, > 80 line minimum)
- `crates/reposix-swarm/tests/contention_e2e.rs` — exists (153 lines)
- `pub mod contention` in `lib.rs` — confirmed
- `Mode::Contention` in `main.rs` — confirmed
- `use reposix_swarm::contention::ContentionWorkload` in `main.rs` — confirmed
- Commits `bfe107d` and `f6d1e63` — both present on `main`
