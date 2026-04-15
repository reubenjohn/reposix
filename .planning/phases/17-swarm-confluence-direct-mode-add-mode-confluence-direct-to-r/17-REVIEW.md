---
phase: 17-re-review
reviewed: 2026-04-14T20:30:00Z
depth: standard
files_reviewed: 7
files_reviewed_list:
  - crates/reposix-swarm/src/confluence_direct.rs
  - crates/reposix-swarm/src/main.rs
  - crates/reposix-swarm/src/lib.rs
  - crates/reposix-swarm/Cargo.toml
  - crates/reposix-swarm/tests/mini_e2e.rs
  - crates/reposix-swarm/tests/confluence_real_tenant.rs
  - CHANGELOG.md
findings:
  critical: 0
  warning: 0
  info: 3
  total: 3
status: clean
---

# Phase 17: Code Review Report (Re-review after fixes)

**Reviewed:** 2026-04-14T20:30:00Z
**Depth:** standard
**Files Reviewed:** 7
**Status:** clean

## Summary

Re-review of Phase 17 (`confluence-direct` swarm mode) after fixes were applied per `17-REVIEW-FIX.md` iteration 1. All three LOW findings from the prior review have been correctly resolved. No new issues were introduced by the fixes. The three INFO items below were out-of-scope for the fix pass and remain; none constitute bugs or security concerns.

### Previous LOW findings — verification

**LOW-01 (`--email` env fallback):** Fixed. `main.rs:67` now carries `#[arg(long, env = "ATLASSIAN_EMAIL")]` with an updated doc comment naming the env-var fallback. `--email` is now on equal footing with `--api-token` / `ATLASSIAN_API_KEY` for process-listing privacy.

**LOW-02 (missing `| get |` assertion in wiremock test):** Fixed. `mini_e2e.rs:291-294` adds `assert!(markdown.contains("| get "), "summary missing get row...")` immediately after the existing `| list |` check, ensuring `get_issue` metric recording is tested.

**LOW-03 (page-get stub ignores id — undocumented):** Fixed. `mini_e2e.rs:244-248` adds a `// NOTE:` block comment explicitly acknowledging the intentional stub behaviour and the blind spot it creates for id-routing bugs. The note accurately describes the trade-off and defers a fix to a future concern.

No regressions were introduced by any of the three fixes.

## Info

### IN-01: `elapsed_us` helper duplicated across three modules

**File:** `crates/reposix-swarm/src/confluence_direct.rs:113`, `crates/reposix-swarm/src/sim_direct.rs:157`, `crates/reposix-swarm/src/fuse_mode.rs:146`
**Issue:** The `elapsed_us(start: Instant) -> u64` helper function is copy-pasted verbatim across all three workload modules. Any change (unit, overflow behaviour) requires three edits.
**Fix:** Promote to `pub(crate)` in `crates/reposix-swarm/src/metrics.rs` or a new `util.rs`, then import at each callsite.

### IN-02: `ids` field uses a plain comment instead of a doc comment

**File:** `crates/reposix-swarm/src/confluence_direct.rs:29`
**Issue:** The `ids` field is annotated with `// Cached ids from the most recent…` instead of `/// Cached ids from the most recent…`. The struct is `pub`; the inconsistency with the rest of the codebase is minor but visible in `rustdoc`.
**Fix:** Change `//` to `///` on that line.

### IN-03: Wiremock test runs for 5 seconds against zero-latency stubs

**File:** `crates/reposix-swarm/tests/mini_e2e.rs:261`
**Issue:** `confluence_direct_3_clients_5s` uses `Duration::from_secs(5)`. The sim-direct counterpart uses 1.5 s. With wiremock responding immediately, 750 ms would satisfy the `total_ops >= 3` floor while halving CI time.
**Fix (low priority):** Reduce to `Duration::from_millis(750)` and rename to `confluence_direct_3_clients_750ms`.

---

_Reviewed: 2026-04-14T20:30:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
