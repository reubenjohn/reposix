---
phase: 34
plan: 01
title: "Blob limit guardrail — REPOSIX_BLOB_LIMIT enforcement"
status: complete
requirements: [ARCH-09]
---

# Phase 34 Plan 01 — Summary

## What shipped

- **Audit-vocabulary surface area** extended in `crates/reposix-cache/fixtures/cache_schema.sql`: 5 new ops added to the `audit_events_cache` CHECK list (`blob_limit_exceeded` + 4 `helper_push_*` for Plan 02 pre-emptive coverage). Triggers (append-only) untouched.
- **New audit helpers** in `crates/reposix-cache/src/audit.rs`: `log_blob_limit_exceeded`, `log_helper_push_started`, `log_helper_push_accepted`, `log_helper_push_rejected_conflict`, `log_helper_push_sanitized_field`. All best-effort (warn-log on SQL failure).
- **Cache wrapper methods** in `crates/reposix-cache/src/cache.rs`: matching public methods so call sites don't reach into `audit::*` directly.
- **`REPOSIX_BLOB_LIMIT` env var plumbing** in `crates/reposix-remote/src/stateless_connect.rs`:
  - `DEFAULT_BLOB_LIMIT: u32 = 200`
  - `BLOB_LIMIT_EXCEEDED_FMT` const with literal backticks around `` `git sparse-checkout set <pathspec>` `` (dark-factory teaching mechanism, ARCH-09).
  - `parse_blob_limit(Option<&str>) -> u32` pure helper (test-friendly; no OnceLock state).
  - `configured_blob_limit() -> u32` reads env once via `std::sync::OnceLock`, falls back to default on garbage with a `tracing::warn!`.
  - `format_blob_limit_message(N, M)` substitutes `{N}` and `{M}` in the const.
- **Enforcement** inside `proxy_one_rpc` AFTER request-frames drained, BEFORE `upload-pack` spawn:
  - Predicate: `command == "fetch" && limit != 0 && want_count > limit`.
  - Stderr first (agent-facing), audit row second (`Cache::log_blob_limit_exceeded`), then `anyhow::bail!` so the helper exits non-zero (`ExitCode::from(2)` via `main()`).
  - Fail-closed: no partial pack ever sent.

## Tests added

- `crates/reposix-cache/src/audit.rs::tests` (+5): `log_blob_limit_exceeded_inserts_row`, `log_helper_push_started_inserts_row`, `log_helper_push_accepted_records_summary`, `log_helper_push_rejected_conflict_records_versions`, `log_helper_push_sanitized_field_records_field_name`.
- `crates/reposix-remote/src/stateless_connect.rs::tests` (+9): `blob_limit_message_contains_literal_git_sparse_checkout`, `parse_blob_limit_default_when_absent`, `parse_blob_limit_zero_means_unlimited_value`, `parse_blob_limit_falls_back_on_garbage`, `parse_blob_limit_accepts_5`, `blob_limit_check_logic_refuses_above_limit`, `blob_limit_check_logic_passes_at_exactly_limit`, `blob_limit_check_logic_zero_means_unlimited`, `blob_limit_check_logic_skips_non_fetch_commands`.

Net: +14 unit tests over Plan 33 baseline.

## Acceptance criteria — status

All Plan-01 acceptance criteria from `34-01-PLAN.md` met.

## Notes for downstream phases

- The dark-factory literal-string regression test (`blob_limit_message_contains_literal_git_sparse_checkout`) is the load-bearing test for Phase 35's agent-self-correction benchmarks: any change that drops the literal `git sparse-checkout` from the message will trip it.
- `BLOB_LIMIT` `OnceLock` is process-global; integration tests that touch it should use `parse_blob_limit` (the pure helper) instead. Documented in code.
