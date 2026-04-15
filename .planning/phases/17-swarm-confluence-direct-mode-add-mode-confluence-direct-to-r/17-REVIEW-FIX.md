---
phase: 17
fixed_at: 2026-04-14T20:16:40Z
review_path: .planning/phases/17-swarm-confluence-direct-mode-add-mode-confluence-direct-to-r/17-REVIEW.md
iteration: 1
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 17: Code Review Fix Report

**Fixed at:** 2026-04-14T20:16:40Z
**Source review:** .planning/phases/17-swarm-confluence-direct-mode-add-mode-confluence-direct-to-r/17-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 3 (all LOW severity; INFO findings excluded per fix_scope=critical_warning)
- Fixed: 3
- Skipped: 0

## Fixed Issues

### LOW-01: `--email` missing `env = "ATLASSIAN_EMAIL"` fallback

**Files modified:** `crates/reposix-swarm/src/main.rs`
**Commit:** `1adc078`
**Applied fix:** Updated the `--email` arg attribute from `#[arg(long)]` to `#[arg(long, env = "ATLASSIAN_EMAIL")]` and expanded the doc comment to mention the env-var fallback. This brings `--email` to parity with `--api-token` / `ATLASSIAN_API_KEY` and prevents the email address from being visible in process listings when supplied via environment variable.

### LOW-02: Wiremock test never asserts `| get |` row in the summary

**Files modified:** `crates/reposix-swarm/tests/mini_e2e.rs`
**Commit:** `2519f44`
**Applied fix:** Added `assert!(markdown.contains("| get "), ...)` immediately after the existing `| list |` assertion in `confluence_direct_3_clients_5s`. This ensures that if `get_issue` calls stop recording metrics the test will fail visibly rather than silently passing on list-only ops.

### LOW-03: Page-get stub ignores requested id — document the limitation

**Files modified:** `crates/reposix-swarm/tests/mini_e2e.rs`
**Commit:** `2519f44` (bundled with LOW-02 — both edits touched the same file before staging)
**Applied fix:** Added a `// NOTE:` block comment at the `path_regex` page-get stub explaining that it always returns `sample_page("10001", "Page 1")` regardless of requested id, that this is intentional for load-testing purposes, and that a more precise per-id stub set would be needed to catch id-routing bugs in `ConfluenceBackend::get_issue`.

---

**Verification:** `cargo clippy -p reposix-swarm --all-targets -- -D warnings` — clean (0 warnings). `cargo test -p reposix-swarm --test mini_e2e` — 2 passed, 0 failed.

_Fixed: 2026-04-14T20:16:40Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
