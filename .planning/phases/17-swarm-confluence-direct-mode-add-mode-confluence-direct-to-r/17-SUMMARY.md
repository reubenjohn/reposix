---
phase: 17
plan: B
subsystem: reposix-swarm
tags: [swarm, confluence, wiremock, tests, docs]
dependency_graph:
  requires: [Phase 17 Plan A — ConfluenceDirectWorkload + Mode::ConfluenceDirect]
  provides: [confluence_direct_3_clients_5s wiremock test, live_confluence_direct_smoke #[ignore] test, CHANGELOG entry]
  affects:
    - crates/reposix-swarm/Cargo.toml
    - crates/reposix-swarm/tests/mini_e2e.rs
    - crates/reposix-swarm/tests/confluence_real_tenant.rs
    - CHANGELOG.md
    - .planning/STATE.md
tech_stack:
  added: [wiremock 0.6 (dev-dep), reposix-confluence (dev-dep)]
  patterns: [wiremock MockServer stubs for Confluence v2 API, #[ignore] env-var-gated real-tenant smoke test]
key_files:
  created:
    - crates/reposix-swarm/tests/confluence_real_tenant.rs
    - .planning/phases/17-swarm-confluence-direct-mode-add-mode-confluence-direct-to-r/17-SUMMARY.md
  modified:
    - crates/reposix-swarm/Cargo.toml
    - crates/reposix-swarm/tests/mini_e2e.rs
    - CHANGELOG.md
    - .planning/STATE.md
decisions:
  - "Wiremock stubs registered without .expect(N) — call counts are non-deterministic across a 5s duration run (RESEARCH.md Risk 2)"
  - "Page list stub returns empty _links to ensure single-page pagination exit (RESEARCH.md Risk 3)"
  - "Page get stub uses path_regex + query_param(body-format, atlas_doc_format) matcher (RESEARCH.md Pitfall 2)"
  - "No audit-row assertion in CI test — read-only workload writes 0 audit rows (RESEARCH.md audit caveat)"
  - "Real-tenant test silently skips (returns early with eprintln) when any of the three env vars is absent — opt-in smoke, not a required gate"
metrics:
  duration_seconds: 480
  completed_date: "2026-04-14"
  tasks_completed: 2
  tasks_total: 2
  files_changed: 6
---

# Phase 17: Swarm Confluence-Direct Mode — Full Phase Summary

**One-liner:** Wiremock CI test `confluence_direct_3_clients_5s` + `#[ignore]` real-tenant smoke + CHANGELOG entry proving `--mode confluence-direct` runs end-to-end against a mocked Confluence API.

## Phase 17 Goal

Add a `--mode confluence-direct` workload to `reposix-swarm` that exercises `ConfluenceBackend` directly (no FUSE overhead), mirroring the existing `SimDirectWorkload` pattern. This closes SWARM-01 + SWARM-02 from the session-4 open gaps list and proves Phase 14's `IssueBackend` trait generalizes across backends under concurrent load.

## Artifacts Created / Modified

### Wave A (Plan 17-A) — shipped separately

| File | Action | Description |
|------|--------|-------------|
| `crates/reposix-swarm/src/confluence_direct.rs` | created | `ConfluenceDirectWorkload` struct + `Workload::step` (list + 3×get, no patch) |
| `crates/reposix-swarm/src/lib.rs` | modified | Added `pub mod confluence_direct;` |
| `crates/reposix-swarm/src/main.rs` | modified | `Mode::ConfluenceDirect` variant + `--email`/`--api-token` args + dispatch arm |
| `crates/reposix-swarm/Cargo.toml` | modified | Added `reposix-confluence = { path = "../reposix-confluence" }` under `[dependencies]` |

Wave A commits: `5ecec37` (workload stub + module), `0ebc58d` (Workload::step + CLI dispatch)

### Wave B (Plan 17-B) — this plan

| File | Action | Description |
|------|--------|-------------|
| `crates/reposix-swarm/Cargo.toml` | modified | Added `reposix-confluence` + `wiremock = "0.6"` under `[dev-dependencies]` |
| `crates/reposix-swarm/tests/mini_e2e.rs` | modified | Added `confluence_direct_3_clients_5s` wiremock test (3 clients × 5s) |
| `crates/reposix-swarm/tests/confluence_real_tenant.rs` | created | `#[ignore]` `live_confluence_direct_smoke` test with silent env-var skip |
| `CHANGELOG.md` | modified | Phase 17 entry under `[Unreleased]` mentioning `confluence-direct` |
| `.planning/STATE.md` | modified | Phase 17 closure recorded; cursor advanced to Phase 18 |

Wave B commits: `52fb4e9` (wiremock CI test + dev-deps), `[this commit]` (real-tenant test + CHANGELOG + SUMMARY + STATE)

## Requirements Closed

| Req ID | Description | Evidence |
|--------|-------------|---------|
| SWARM-01 | `reposix-swarm --mode confluence-direct` exercises `ConfluenceBackend` directly | `Mode::ConfluenceDirect` dispatch in `main.rs`; `confluence_direct_3_clients_5s` test passes |
| SWARM-02 | Swarm produces summary matching sim-direct format; no Other-class errors | `markdown.contains("Clients: 3")`, `markdown.contains("| list ")`, `total_ops >= 3`, no `| Other` in error section |

## Test Count Delta

| Baseline (Phase 16) | New tests added | Total |
|---------------------|-----------------|-------|
| 317 | +1 (`confluence_direct_3_clients_5s` in mini_e2e.rs) | 318 |

The `live_confluence_direct_smoke` test in `confluence_real_tenant.rs` is ignored in normal runs (registered as 1 ignored test). It does not contribute to the pass count.

## Verification Results

- `cargo test --workspace` — PASS (318 active tests, 0 failures)
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS (0 warnings)
- `cargo test -p reposix-swarm --test mini_e2e` — PASS (both sim test + confluence wiremock test)
- `cargo test -p reposix-swarm --test confluence_real_tenant -- --ignored live_confluence_direct_smoke` — PASS (returns immediately with "skip: ATLASSIAN_EMAIL not set" when env absent)

## Deferred Work

| Item | Deferred To | Rationale |
|------|-------------|-----------|
| Write operations (`create_issue`, `update_issue`, `delete_or_close`) in swarm | Phase 21 / OP-7 | Read-only locked decision for Phase 17 (CONTEXT.md) |
| 50-client × 30s real-tenant load runs | Phase 21 HARD-01 | Too expensive for CI; 3×10s smoke is sufficient |
| Write-contention testing | Phase 21 HARD-01 | Requires write ops to be in the workload first |
| Space-ID caching in `ConfluenceBackend` | Phase 21 optimization | Currently re-resolves on every `list_issues` call; acceptable for Phase 17 |

## Threat Model Compliance

| Threat ID | Status | Evidence |
|-----------|--------|---------|
| T-17-05 (info disclosure in real-tenant test logging) | Mitigated | `eprintln!` prints var names only ("skip: ATLASSIAN_EMAIL not set"), never values; `ConfluenceCreds` Debug redacts `api_token` |
| T-17-06 (wiremock fixture tampering) | Accepted | Fixtures are test-author-controlled; stay inside test process |
| T-17-07 (real-tenant silent-skip repudiation) | Accepted | Intentional opt-in smoke; developer verifies locally with `--ignored` |
| T-17-08 (DoS via 3×10s real-tenant) | Mitigated | Locked decision (not 50×30s); `rate_limit_gate` absorbs 429s |
| T-17-09 (REPOSIX_ALLOWED_ORIGINS in real-tenant test) | Mitigated | Test deliberately does NOT set allowlist; caller must set explicitly |

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None — the workload and tests are fully functional; no placeholder data flows to any UI or output.

## Self-Check: PASSED

- `crates/reposix-swarm/tests/mini_e2e.rs` contains `confluence_direct_3_clients_5s` — FOUND
- `crates/reposix-swarm/tests/confluence_real_tenant.rs` contains `#[ignore` — FOUND
- `crates/reposix-swarm/Cargo.toml` contains `wiremock` — FOUND
- `CHANGELOG.md` contains `confluence-direct` — FOUND
- Commit `52fb4e9` — FOUND
- `cargo test --workspace` green, 318 active tests — VERIFIED
- `cargo clippy --workspace --all-targets -- -D warnings` clean — VERIFIED
