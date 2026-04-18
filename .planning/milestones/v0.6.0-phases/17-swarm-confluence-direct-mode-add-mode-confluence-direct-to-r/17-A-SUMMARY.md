---
phase: 17
plan: A
subsystem: reposix-swarm
tags: [swarm, confluence, workload, cli]
dependency_graph:
  requires: []
  provides: [Mode::ConfluenceDirect, ConfluenceDirectWorkload]
  affects: [reposix-swarm/Cargo.toml, reposix-swarm/src/lib.rs, reposix-swarm/src/confluence_direct.rs, reposix-swarm/src/main.rs]
tech_stack:
  added: [reposix-confluence (runtime dep)]
  patterns: [IssueBackend trait generalization, per-client ConfluenceBackend construction, parking_lot::Mutex<StdRng> for deterministic per-client RNG]
key_files:
  created:
    - crates/reposix-swarm/src/confluence_direct.rs
  modified:
    - crates/reposix-swarm/Cargo.toml
    - crates/reposix-swarm/src/lib.rs
    - crates/reposix-swarm/src/main.rs
decisions:
  - "Used Mode::ConfluenceDirect variant with kebab-case rendering confluence-direct via #[clap(rename_all = kebab_case)]"
  - "Workload is strictly read-only (list + 3xget, no patch) per locked decision in CONTEXT.md; writes deferred to Phase 21 / OP-7"
  - "Per-client ConfluenceBackend construction in run_swarm factory closure ensures independent rate_limit_gate per client"
  - "Credentials not logged — passed directly into ConfluenceCreds which has manual Debug redaction (T-17-01 mitigated)"
metrics:
  duration_seconds: 433
  completed_date: "2026-04-14"
  tasks_completed: 2
  tasks_total: 2
  files_changed: 4
---

# Phase 17 Plan A: Workload and CLI Summary

**One-liner:** `reposix-swarm --mode confluence-direct` wiring `ConfluenceDirectWorkload` (list + 3xget, no patch) against `ConfluenceBackend` with per-client credential injection and `ATLASSIAN_API_KEY` env fallback.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 17-A-01 | Add reposix-confluence dep + ConfluenceDirectWorkload stub | 5ecec37 | Cargo.toml, lib.rs, confluence_direct.rs (new) |
| 17-A-02 | Implement Workload::step + Mode::ConfluenceDirect CLI dispatch | 0ebc58d | confluence_direct.rs, main.rs |

## Artifacts Created

### `crates/reposix-swarm/src/confluence_direct.rs` (new)

Contains `ConfluenceDirectWorkload` struct with:
- `backend: ConfluenceBackend` — per-client instance (own rate-limit gate)
- `space: String` — Confluence space key (maps to `project` param in `IssueBackend`)
- `rng: Mutex<StdRng>` — seeded per-client for determinism
- `ids: Mutex<Vec<IssueId>>` — id cache refreshed on each `list_issues` call
- `new(base_url, creds, space, seed)` — builds via `ConfluenceBackend::new_with_base_url`
- `random_id()` — picks a random cached id
- `Workload::step` — 1x list + up to 3x get; no patch (read-only locked decision)
- `elapsed_us` helper function

### `crates/reposix-swarm/src/main.rs` (modified)

- Added `Mode::ConfluenceDirect` variant (renders as `confluence-direct`)
- Added `Mode::as_str` arm returning `"confluence-direct"`
- Added `--email` and `--api-token` (env: `ATLASSIAN_API_KEY`) CLI args
- Added `Mode::ConfluenceDirect` dispatch arm calling `ConfluenceDirectWorkload::new` via `run_swarm`
- Credentials moved directly into `ConfluenceCreds`; never logged (T-17-01 mitigated)

### `crates/reposix-swarm/src/lib.rs` (modified)

Added `pub mod confluence_direct;` export.

### `crates/reposix-swarm/Cargo.toml` (modified)

Added `reposix-confluence = { path = "../reposix-confluence" }` under `[dependencies]`.

## Verification Results

- `cargo check --workspace` — PASS
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS (0 warnings)
- `cargo test --workspace` — PASS (317 tests, matches Phase 16 baseline; Wave 1 adds no tests)
- `--mode confluence-direct --help` — shows `--email` and `--api-token` flags
- `--mode confluence-direct` without `--email` — fails cleanly with `--email required for confluence-direct`

## No Tests Added

Wave 1 (this plan) adds no tests. Wave 2 (plan 17-B) owns:
- `confluence_direct_3_clients_5s` test in `tests/mini_e2e.rs` (wiremock, 3 clients, 5s)
- `tests/confluence_real_tenant.rs` (`#[ignore]` real-tenant smoke test)

## Deviations from Plan

**1. [Rule 1 - Bug] Fixed doc-markdown clippy lint in module docstring**
- **Found during:** Task 17-A-01 clippy run
- **Issue:** `ConfluenceBackend` in `//!` doc comments not wrapped in backticks — triggered `clippy::doc_markdown` with `-D warnings`
- **Fix:** Changed bare `ConfluenceBackend` to `[`ConfluenceBackend`]` (intra-doc link style) in lines 1 and 5 of the module docstring
- **Files modified:** `crates/reposix-swarm/src/confluence_direct.rs`
- **Commit:** 5ecec37 (included in task commit after fix)

**2. Combined Workload impl into task 17-A-01 commit scope**
- The plan split the struct stub (17-A-01) from the `Workload` impl (17-A-02) into separate task actions. The `Workload::step` implementation was written in the initial file creation (Write tool) since both tasks operate on the same file and the impl was fully specified in the plan. The task 17-A-02 commit captures the `main.rs` changes (Mode variant + CLI args + dispatch arm) as the distinguishing work of that task.

## Threat Model Compliance

| Threat ID | Status | Evidence |
|-----------|--------|----------|
| T-17-01 (creds leak) | Mitigated | `email`/`api_token` moved directly into `ConfluenceCreds`; no `tracing::info!` or `println!` on credentials in dispatch arm |
| T-17-02 (SSRF via --target) | Mitigated | `base_url` passed unchanged to `ConfluenceBackend::new_with_base_url`; SG-01 allowlist check inside `HttpClient` — no string concatenation in main.rs |
| T-17-03 (DoS) | Accepted | Workload hammers backend by design; rate limiting via transparent `rate_limit_gate` |
| T-17-04 (Tampering) | Mitigated by scope | Read-only workload; no write ops in Phase 17 |

## Known Stubs

None — the workload is functional end-to-end; the only deferral is test coverage (Wave 2).

## Self-Check: PASSED

- `crates/reposix-swarm/src/confluence_direct.rs` — FOUND
- `crates/reposix-swarm/src/lib.rs` contains `pub mod confluence_direct` — FOUND
- `crates/reposix-swarm/src/main.rs` contains `ConfluenceDirect` — FOUND
- `crates/reposix-swarm/Cargo.toml` contains `reposix-confluence` — FOUND
- Commit 5ecec37 — FOUND
- Commit 0ebc58d — FOUND
