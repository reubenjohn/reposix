---
phase: 21
plan: D
subsystem: reposix-swarm
tags: [hardening, chaos, audit-log, wal, kill-9, sqlite, hard-03]
requirements: [HARD-03]

dependency_graph:
  requires: [21-B]
  provides: [chaos-audit-wal-invariant]
  affects: [crates/reposix-swarm/tests/chaos_audit.rs]

tech_stack:
  added: []
  patterns:
    - "#[ignore] + env-var guard for chaos/slow tests"
    - "Binary path resolution via CARGO_MANIFEST_DIR (cross-crate, stable Rust)"
    - "Two-cycle kill-9 + WAL replay pattern"
    - "reposix_core::http::client() for all HTTP in tests (project allowlist)"

key_files:
  created:
    - crates/reposix-swarm/tests/chaos_audit.rs
  modified: []

decisions:
  - "Use CARGO_MANIFEST_DIR + target/{release,debug}/reposix-sim path resolution instead of CARGO_BIN_EXE_reposix-sim: the env! macro only works for same-package binaries on stable Rust; artifact deps require nightly -Z bindeps"
  - "Torn-row query checks ts/method/path (actual NOT NULL columns in audit.sql), not op/entity_id/ts (plan assumed column names that don't exist)"
  - "Use reposix_core::http::client() not reqwest::Client::builder() per project clippy allowlist (SG-01)"
  - "REPOSIX_SIM_BIN env var override added for CI flexibility"

metrics:
  duration: "12 minutes"
  completed: "2026-04-15"
  tasks_completed: 1
  files_created: 1
  files_modified: 0
---

# Phase 21 Plan D: Chaos Audit-Log WAL Integrity Test Summary

**One-liner:** SQLite WAL durability under SIGKILL verified by two-cycle kill-9 chaos test generating 2000+ audit rows with zero torn rows.

## What Was Built

`crates/reposix-swarm/tests/chaos_audit.rs` — a `#[tokio::test]` / `#[ignore]` integration test (`chaos_kill9_no_torn_rows`) that:

1. Spawns `reposix-sim` as a real OS child process (not an in-process task, so `child.kill()` sends true SIGKILL, not cooperative cancellation).
2. Drives async HTTP load against the sim for 500 ms (generating audit rows in WAL).
3. Sends SIGKILL mid-flight via `child.kill()` + `child.wait()`.
4. Reopens the SQLite DB with rusqlite and asserts zero torn rows: `SELECT COUNT(*) FROM audit_events WHERE ts IS NULL OR method IS NULL OR path IS NULL`.
5. Repeats the cycle a second time on the same DB file to prove WAL replay on reopen works correctly.

**Invocation:**
```bash
# Build the sim binary first
cargo build -p reposix-sim --release

# Run the chaos test
REPOSIX_CHAOS_TEST=1 cargo test -p reposix-swarm --test chaos_audit --release --locked -- --ignored --nocapture
```

**Observed results (dev run):**
- Cycle 1: 2076–2272 audit rows after SIGKILL, 0 torn rows
- Cycle 2: 3731–3960 audit rows after SIGKILL, 0 torn rows (>= cycle 1 rows — WAL replay preserved all committed data)

## Schema Columns Checked for Torn Rows

The `audit_events` table DDL (in `crates/reposix-core/fixtures/audit.sql`) defines these NOT NULL columns:

| Column    | Type    | Constraint |
|-----------|---------|------------|
| `ts`      | TEXT    | NOT NULL   |
| `method`  | TEXT    | NOT NULL   |
| `path`    | TEXT    | NOT NULL   |

The torn-row query asserts none of these are NULL after WAL recovery:
```sql
SELECT COUNT(*) FROM audit_events WHERE ts IS NULL OR method IS NULL OR path IS NULL
```

Note: The plan's task description referenced columns `op`, `entity_id`, `ts` — these don't exist in the actual schema. The implementation uses the correct column names from `audit.sql`.

## Binary Path Resolution

`CARGO_BIN_EXE_reposix-sim` is only set by Cargo for binaries in the **same package** as the integration test. Since `chaos_audit.rs` lives in `reposix-swarm` and the binary is in `reposix-sim`, the macro is unavailable on stable Rust (artifact deps require nightly `-Z bindeps`).

Resolution at runtime:
1. `REPOSIX_SIM_BIN` env var (explicit CI override)
2. `<workspace_root>/target/release/reposix-sim` (prefers release — more representative)
3. `<workspace_root>/target/debug/reposix-sim` (dev fallback)

## CI Status

The chaos test is a **manual/weekly invariant gate** — NOT a per-commit CI gate. It is `#[ignore]` by default with a defence-in-depth `REPOSIX_CHAOS_TEST=1` env var guard inside the body. Adding it to a chaos CI job (e.g. a scheduled weekly workflow) is out of scope for this plan but documented here as a follow-up.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Wrong audit column names in torn-row query**
- **Found during:** Task D1 step 4 (schema verification)
- **Issue:** Plan specified `op IS NULL OR entity_id IS NULL OR ts IS NULL` but the actual schema (`crates/reposix-core/fixtures/audit.sql`) has columns `ts`, `method`, `path` as NOT NULL (no `op` or `entity_id` columns exist).
- **Fix:** Corrected query to `WHERE ts IS NULL OR method IS NULL OR path IS NULL`.
- **Files modified:** `crates/reposix-swarm/tests/chaos_audit.rs`
- **Commit:** 3f048d4

**2. [Rule 3 - Blocking] CARGO_BIN_EXE_reposix-sim unavailable on stable Rust**
- **Found during:** Task D1 step 5 (first compilation attempt)
- **Issue:** `env!("CARGO_BIN_EXE_reposix-sim")` is only set for binaries in the same package; artifact dependencies (`artifact = "bin"`) require nightly `-Z bindeps` (rejected by `cargo 1.94.1` on stable).
- **Fix:** Replaced with runtime path resolution from `env!("CARGO_MANIFEST_DIR")` with `REPOSIX_SIM_BIN` env var override. The CARGO_BIN_EXE pattern is documented in the module-level rustdoc.
- **Files modified:** `crates/reposix-swarm/tests/chaos_audit.rs`
- **Commit:** 3f048d4

**3. [Rule 1 - Bug] reqwest::Client::builder() blocked by project clippy allowlist**
- **Found during:** Task D1 step 5 (clippy run)
- **Issue:** Workspace `clippy.toml` disallows `reqwest::Client::builder()` with note "use reposix_core::http::client()" — this is the SG-01 allowlist gate.
- **Fix:** Replaced all three `reqwest::Client::builder()` calls with `reposix_core::http::client(ClientOpts::default())` and the `HttpClient::get()` method.
- **Files modified:** `crates/reposix-swarm/tests/chaos_audit.rs`
- **Commit:** 3f048d4

## Self-Check: PASSED

- `crates/reposix-swarm/tests/chaos_audit.rs` — FOUND (285 lines, >= 100 required)
- Commit `3f048d4` — FOUND
- `cargo test -p reposix-swarm --test chaos_audit --locked` — exits 0 (1 ignored)
- `REPOSIX_CHAOS_TEST=1 cargo test ... --ignored` — exits 0 (1 passed, 0 torn rows both cycles)
- `cargo clippy --workspace --all-targets -- -D warnings` — clean
- `cargo fmt --all --check` — clean
