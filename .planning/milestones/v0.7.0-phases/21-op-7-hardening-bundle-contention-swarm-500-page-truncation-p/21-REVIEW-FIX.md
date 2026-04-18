---
phase: 21
fixed_at: 2026-04-15T00:00:00Z
review_path: .planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-REVIEW.md
iteration: 1
findings_in_scope: 2
fixed: 2
skipped: 0
status: all_fixed
---

# Phase 21: Code Review Fix Report

**Fixed at:** 2026-04-15T00:00:00Z
**Source review:** `.planning/phases/21-op-7-hardening-bundle-contention-swarm-500-page-truncation-p/21-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope (WR-*): 2
- Fixed: 2
- Skipped: 0

## Fixed Issues

### WR-01: 409 response body echoed unredacted and unbounded into error string

**Files modified:** `crates/reposix-confluence/src/lib.rs`
**Commit:** `403b953`
**Applied fix:** In the `CONFLICT` arm of `update_issue`, replaced the raw
`String::from_utf8_lossy(&bytes)` interpolation with a 256-char preview via `.chars().take(256)`
and added `redact_url(&url)` to strip the tenant hostname. The error message now reads
`"confluence version conflict for PUT {redacted_url}: {body_preview}"`, matching the pattern used
by every other error path in the function.

### WR-02: `no_truncate` integration test never reaches the mock; asserts the wrong failure mode

**Files modified:** `crates/reposix-cli/tests/no_truncate.rs`
**Commit:** `e643b68`
**Applied fix:** Renamed `no_truncate_errors_when_space_exceeds_cap` to
`no_truncate_flag_exits_nonzero_when_backend_unreachable` and replaced the doc comment with a
clear smoke-test description explaining that wiremock redirect is not wired and that cap-logic
coverage lives in `reposix-confluence::tests` (`list_strict_errors_at_cap`). The assertion
message was updated to honestly state the failure mode: connection refused to
`test-tenant.atlassian.net`. A follow-up clippy `doc_markdown` lint (missing backticks around
the test name in the doc comment) was also corrected in the same commit scope.

## Deferred (Info-level, out of scope for fix_scope=critical_warning)

- **IN-01:** Chaos test uses fixed port 7979 — bind to ephemeral port to avoid concurrent-run collisions.
- **IN-02:** `ConfLinks` struct is dead code — remove or use directly to eliminate `#[allow(dead_code)]`.
- **IN-03:** `contention_e2e` invariant assertions use fragile markdown string matching — consider structured summary type.

---

_Fixed: 2026-04-15T00:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
