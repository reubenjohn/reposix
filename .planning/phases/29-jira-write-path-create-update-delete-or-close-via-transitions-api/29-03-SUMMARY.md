---
id: 29-03
status: complete
commit: 8eca6a0
---

# Plan 29-03 Summary — delete_or_close via transitions + supports() + contract + docs + ship

## What shipped

### Task T1: `delete_or_close` via transitions
Two-step flow: GET `/rest/api/3/issue/{id}/transitions` → filter `statusCategory.key == "done"` → POST selected transition. Preference logic: `NotPlanned`/`Duplicate` reasons try to match "won't"/"wont"/"reject"/"not planned"/"invalid"/"duplicate" transition names, fall back to first done transition. `DeleteReason::WontFix` doesn't exist as a variant — mapped to `NotPlanned`/`Duplicate`. On HTTP 400, retries with `{"fields":{"resolution":{"name":"Done"}}}`. DELETE fallback with `tracing::warn!` when no done transitions found. Audit rows on all paths.

### Task T2: `supports()` + delete wiremock tests
- `supports()` now returns true for `BackendFeature::Hierarchy | Delete | Transitions`
- Renamed test 10 to `supports_reports_delete_and_transitions`
- Added 3 wiremock tests: `delete_or_close_via_transitions`, `delete_or_close_wontfix_picks_reject` (uses `DeleteReason::NotPlanned`), `delete_or_close_fallback_delete` (verifies DELETE is called via `expect(1)`)

### Task T3: Contract test extension
- Added `make_untainted_for_contract` helper
- Added `assert_write_contract<B>` (create → update → delete → assert-gone)
- Added `build_jira_wiremock_write_server()` using FIFO mock ordering (corrected from plan's LIFO assumption)
- Added `contract_jira_wiremock_write` (always runs)
- Added `contract_jira_live_write` (`#[ignore]`-gated)

### Task T4: Docs + workspace
- Updated `lib.rs` module doc comment (removed "read-only", removed stale Phase 29 stub note)
- Updated `CHANGELOG.md` [v0.8.0] section with Phase 29 additions
- `tag-v0.8.0.sh` already existed and is correct — not regenerated
- `cargo test --workspace`: all suites pass
- `cargo clippy --workspace --all-targets -- -D warnings`: clean
- `cargo fmt --all --check`: clean
- `grep -c "not supported" crates/reposix-jira/src/lib.rs`: 0

## Test results
- 31 unit tests + 5 contract tests (3 run, 2 ignored) pass
- Full workspace test count: consistent with Phase 28 baseline + 19 new tests

## Files modified
- `crates/reposix-jira/src/lib.rs` (+410/-21 lines)
- `crates/reposix-jira/tests/contract.rs` (+185 lines)
- `CHANGELOG.md` (+30 lines)
