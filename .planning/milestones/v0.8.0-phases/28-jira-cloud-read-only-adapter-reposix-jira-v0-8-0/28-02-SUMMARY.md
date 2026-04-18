---
plan: 28-02
phase: 28
status: complete
commit: b437415
wave: 2
---

# Plan 28-02 Summary: Contract test + CLI integration

## What Was Built

**crates/reposix-jira/tests/contract.rs** — 3-arm contract test proving JiraBackend correctly implements the BackendConnector seam:
- `contract_sim`: sim arm (always runs)
- `contract_jira_wiremock`: wiremock arm (always runs)
- `contract_jira_live`: live arm (#[ignore]-gated + skip_if_no_env! with JIRA_EMAIL, JIRA_API_TOKEN, REPOSIX_JIRA_INSTANCE)

**crates/reposix-cli/src/list.rs** — CLI list integration:
- `ListBackend::Jira` variant added to enum
- `read_jira_env_from` pure-fn helper (collects ALL missing vars in one error, never echoes values)
- `read_jira_env` thin production adapter
- Jira dispatch arm in `run()` with `list_issues_strict` support for `--no-truncate`
- 3 unit tests: all-empty-fails, partial-missing-lists-all, all-set-succeeds

**crates/reposix-cli/src/mount.rs** — `--backend jira` dispatch with pre-spawn allowlist and env-var fast-fail checks

**crates/reposix-cli/src/refresh.rs** — Jira arm in `backend_label()`, bucket mapping (`"issues"`), and `fetch_issues()`

**crates/reposix-cli/src/spaces.rs** — Jira arm returns clear not-supported error

## Security Invariants Confirmed

| Invariant | Status |
|-----------|--------|
| `read_jira_env_from` never echoes credential values | ✓ Names only in error messages |
| `contract_jira_live` gated by `#[ignore]` + `skip_if_no_env!` | ✓ Never runs in CI without opt-in |
| mount.rs pre-spawn checks name env vars but not values | ✓ Follows T-28-02-01 mitigation |

## Test Results

Contract tests: 2 passed (sim + wiremock), 1 ignored (live)  
CLI unit tests: 3 passed (read_jira_env_from)  
Full workspace: 0 failures

`cargo clippy --workspace --all-targets -- -D warnings`: clean

## Deviations

None — all tasks completed as specified.

## Self-Check: PASSED
