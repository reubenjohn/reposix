---
phase: 21
plan: C
subsystem: reposix-confluence + reposix-cli
tags: [hardening, confluence, truncation, tenant-redaction, SG-05, HARD-02, HARD-05]
dependency_graph:
  requires: [21-A]
  provides: [list_issues_strict, --no-truncate CLI flag, tenant URL redaction]
  affects: [reposix-confluence, reposix-cli]
tech_stack:
  added: []
  patterns: [redact_url helper, list_issues_impl(strict: bool) private helper, wiremock integration test via assert_cmd]
key_files:
  created:
    - crates/reposix-cli/tests/no_truncate.rs
  modified:
    - crates/reposix-confluence/src/lib.rs
    - crates/reposix-cli/src/main.rs
    - crates/reposix-cli/src/list.rs
    - crates/reposix-cli/Cargo.toml
decisions:
  - "list_issues_strict is a concrete method on ConfluenceBackend only (not on IssueBackend trait) — avoids forcing all backends to change"
  - "list_issues_impl(strict: bool) is the shared private helper; both public methods delegate to it"
  - "redact_url() applied to all error paths in lib.rs (not just list_issues) — all methods leak tenant via full URL"
  - "--no-truncate is a no-op for sim/github backends, documented in help text"
metrics:
  duration: ~25m
  completed: "2026-04-15"
  tasks_completed: 2
  files_modified: 4
  files_created: 1
---

# Phase 21 Plan C: Truncation + Tenant-URL Redaction Summary

One-liner: Strict 500-page truncation guard (`list_issues_strict`) + full tenant-URL redaction across all `ConfluenceBackend` error paths + `--no-truncate` CLI flag wired end-to-end.

## What Was Built

### Task C1: ConfluenceBackend::list_issues_strict + redact_url (HARD-02, HARD-05)

**`redact_url(raw: &str) -> String`** — private free function that strips the host+scheme from any URL, leaving only path+query. Used in every error format string in `lib.rs` so tenant hostnames never appear in returned errors or tracing spans.

Before (HARD-05 leak):
```
confluence returned 500 for GET https://reuben-john.atlassian.net/wiki/api/v2/spaces/123/pages: ...
```

After (redacted):
```
confluence returned 500 for GET /wiki/api/v2/spaces/123/pages: ...
```

**`list_issues_impl(&self, project: &str, strict: bool)`** — private helper that contains the full pagination loop. The `strict` flag gates both truncation sites:
- `pages > MAX_ISSUES_PER_LIST / PAGE_SIZE`: in strict mode returns `Err(Error::Other("...exceeds 500-page cap; refusing to truncate (strict mode)"))`, in warn mode emits `tracing::warn!` and breaks.
- `out.len() >= MAX_ISSUES_PER_LIST`: in strict mode returns `Err`, in warn mode returns `Ok(capped)`.

**`pub async fn list_issues_strict(&self, project: &str) -> Result<Vec<Issue>>`** — public concrete method on `ConfluenceBackend` (not on `IssueBackend` trait). Delegates to `list_issues_impl(project, true)`.

**`IssueBackend::list_issues`** now delegates to `list_issues_impl(project, false)` — identical behaviour to before (backwards-compatible).

**URL redaction scope** (Rule 2 auto-fix): All error paths in `lib.rs` were audited. `redact_url()` was applied to `resolve_space_id`, `get_issue` (both ADF and storage fallback paths), `create_issue`, `update_issue`, and `delete_or_close`. No error path leaks the tenant hostname.

**New unit tests (3):**

| Test | What it proves |
|------|---------------|
| `truncation_warn_on_default_list` | `list_issues` returns `Ok(5 items)` at cap, does not call page 6 |
| `truncation_errors_in_strict_mode` | `list_issues_strict` returns `Err` containing `"strict mode"` and `"500-page cap"` at cap |
| `list_error_message_omits_tenant` | HTTP 500 error does NOT contain host:port, DOES contain `/wiki/api/v2/` path |

### Task C2: --no-truncate CLI flag (HARD-02 CLI surface)

**`crates/reposix-cli/src/main.rs`**: Added `#[arg(long)] no_truncate: bool` to the `List` subcommand with doc comment: "Error instead of silently capping at 500 pages (Confluence only). No-op for --backend sim and --backend github."

**`crates/reposix-cli/src/list.rs`**: Updated `run(... no_truncate: bool)` signature. In the `ListBackend::Confluence` branch: `if no_truncate { b.list_issues_strict(&project).await? } else { b.list_issues(&project).await? }`.

**`crates/reposix-cli/tests/no_truncate.rs`**: Integration tests using `assert_cmd` + `wiremock`:
- `no_truncate_flag_appears_in_list_help`: `reposix list --help` stdout contains `"no-truncate"`
- `no_truncate_errors_when_space_exceeds_cap`: `reposix list --backend confluence --no-truncate` exits non-zero

## Acceptance Criteria Verification

```
grep -qE "pub async fn list_issues_strict" crates/reposix-confluence/src/lib.rs  ✓
grep -qE "fn list_issues_impl" crates/reposix-confluence/src/lib.rs               ✓
grep -qE "fn redact_url" crates/reposix-confluence/src/lib.rs                     ✓
grep -q "strict mode" crates/reposix-confluence/src/lib.rs                        ✓
grep -q "no_truncate" crates/reposix-cli/src/main.rs                              ✓
grep -q "no_truncate" crates/reposix-cli/src/list.rs                              ✓
grep -q "list_issues_strict" crates/reposix-cli/src/list.rs                       ✓
grep -q "fn list_error_message_omits_tenant" crates/reposix-confluence/src/lib.rs ✓
0 matches for "for GET {url}" in lib.rs (all redacted)                             ✓
cargo test --workspace --locked                                                    ✓ (all pass)
cargo clippy --workspace --all-targets -- -D warnings                             ✓ (clean)
cargo fmt --all --check                                                            ✓ (clean)
cargo run -p reposix-cli -- list --help | grep no-truncate                        ✓
```

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Security] Redact tenant URLs in all error paths, not just list_issues**
- **Found during:** Task C1 (scanning for `{url}` in format strings)
- **Issue:** `resolve_space_id`, `get_issue` (×2), `create_issue`, `update_issue` (×2), and `delete_or_close` (×2) all included full URLs with tenant hostnames in error messages. HARD-05 requires no tenant leak in any error path, not just the pagination loop.
- **Fix:** Applied `redact_url(&url)` to all 9 remaining sites in `lib.rs`.
- **Files modified:** `crates/reposix-confluence/src/lib.rs`
- **Commit:** `4b5d9bf`

## Commits

| Hash | Message |
|------|---------|
| `d05557d` | `feat(21-C): list_issues_strict + tenant-URL redaction (HARD-02 HARD-05)` |
| `f172fbc` | `feat(21-C): --no-truncate flag on reposix list (HARD-02 CLI surface)` |
| `4b5d9bf` | `fix(21-C): redact tenant URLs in all error paths (HARD-05 complete)` |

## Known Stubs

None. All functionality is fully wired.

## Threat Flags

No new network endpoints, auth paths, or trust-boundary-crossing surfaces introduced. The `--no-truncate` flag is additive-only and strictly reduces data flow (errors before returning partial data).

## Self-Check: PASSED

- `crates/reposix-confluence/src/lib.rs` — exists, contains `list_issues_strict`, `list_issues_impl`, `redact_url`
- `crates/reposix-cli/src/list.rs` — exists, contains `no_truncate`, `list_issues_strict`
- `crates/reposix-cli/src/main.rs` — exists, contains `no_truncate`
- `crates/reposix-cli/tests/no_truncate.rs` — exists
- Commits `d05557d`, `f172fbc`, `4b5d9bf` — all present in `git log`
- `cargo test --workspace --locked` — all test results ok, 0 failed
- `cargo clippy --workspace --all-targets -- -D warnings` — Finished, 0 errors
- `cargo fmt --all --check` — clean
