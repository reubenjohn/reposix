---
phase: 34
plan: 02
title: "Push-time conflict detection + frontmatter allowlist + push audit ops"
status: complete
requirements: [ARCH-08, ARCH-10]
---

# Phase 34 Plan 02 — Summary

## What shipped

- **Push-time conflict detection** in `crates/reposix-remote/src/main.rs::handle_export`:
  - Lazy-opens the cache (best-effort) and writes `helper_push_started` audit row.
  - After `parse_export_stream` and `list_issues`, builds `prior_by_id` index, then walks `parsed.tree`. For each existing issue (path matches a prior id), parses the new blob's frontmatter and compares its `version` to the prior's. Mismatch → conflict.
  - On conflict: writes `helper_push_rejected_conflict` audit row with `(issue_id, local_version, backend_version)`, emits a free-form stderr diagnostic line containing the issue id, ISO-8601 timestamp, and `Run: git pull --rebase`, and writes the canned `error refs/heads/main fetch first` status. NO `plan()` call, NO REST writes.
  - On accept: writes `helper_push_accepted` audit row with `(files_touched, deterministic comma-separated id summary)` then emits `ok refs/heads/main`.
- **`helper_push_sanitized_field` audit row** wired in `execute_action::PlannedAction::Update`: one row per Update with `field="version"` (the dominant server-controlled field; row is per-issue, not per-field). Audit fires BEFORE the `sanitize()` call so an audit reader can see the boundary.
- **`issue_id_from_path`** helper added to `main.rs` (4-line pure fn) so paths like `0042.md` cleanly parse to `42` and non-issue paths return `None`.

## Tests added

- **`crates/reposix-remote/tests/push_conflict.rs`** (3 integration tests, +326 lines):
  - `stale_base_push_emits_fetch_first_and_writes_no_rest` — ARCH-08 regression. Strict expectation 0 on PATCH/POST/DELETE wiremocks; helper exits non-zero with `error refs/heads/main fetch first` on stdout and `issue 2`+`git pull --rebase` on stderr.
  - `clean_push_emits_ok_and_mutates_backend` — happy path. Strict expectation 1 on PATCH; helper exits zero with `ok refs/heads/main`.
  - `frontmatter_strips_server_controlled_fields` — ARCH-10 regression. Inbound blob has `id: 999999` and `version: 999` (with `version: 3` matching backend so conflict-check passes). Captured PATCH body asserted to contain neither `999999` nor `version` field nor `id` field — the sim's `deny_unknown_fields` and the helper's stripping work together.

Net: +3 integration tests over Plan 33 baseline. Pre-existing tests (`bulk_delete_cap`, 3 tests) still pass — regression preserved.

## Acceptance criteria — status

All Plan-02 acceptance criteria from `34-02-PLAN.md` met. `grep` invariants:

- `'helper_push_*' + 'blob_limit_exceeded'` schema entries: 5 (verified).
- `log_helper_push_*` audit fns in `audit.rs`: 4 (verified by Plan 01).
- `log_helper_push_*` Cache wrappers in `cache.rs`: 4 (verified by Plan 01).
- `log_helper_push_sanitized_field` invocations in `main.rs`: 1 (in `execute_action::Update`).

## Notes for downstream phases

- **Phase 35 latency benchmarks** can `grep` the audit table for the 5 new ops to derive push/fetch operation counts without instrumenting the helper.
- **Phase 35 dark-factory test** for push conflict cycle: assert that an agent running into `error refs/heads/main fetch first` then doing `git pull --rebase && git push` succeeds — relies on the canned status string being byte-identical (`error refs/heads/main fetch first` — committed as a literal in `handle_export`).
- **Phase 35 dark-factory test** for blob-limit recovery: agent receiving `BLOB_LIMIT_EXCEEDED_FMT` and running `git sparse-checkout set <pathspec>` succeeds — relies on the literal string committed in `stateless_connect.rs`.
- The reject-path atomicity guarantee (no REST calls fire when conflict detected) is enforced by control flow: `handle_export` returns BEFORE `plan()` runs. `git fsck` invariant is implicit (cache untouched on the helper-export path; the bare-cache update is a Phase 32 concern via `stateless-connect`).
