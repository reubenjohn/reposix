---
phase: 21
status: clean
severity: none
finding_count: 0
reviewed_files: 12
files_reviewed_list:
  - .github/workflows/ci.yml
  - crates/reposix-cli/src/list.rs
  - crates/reposix-cli/src/main.rs
  - crates/reposix-cli/tests/no_truncate.rs
  - crates/reposix-confluence/src/lib.rs
  - crates/reposix-fuse/tests/nested_layout.rs
  - crates/reposix-fuse/tests/sim_death_no_hang.rs
  - crates/reposix-swarm/src/contention.rs
  - crates/reposix-swarm/src/lib.rs
  - crates/reposix-swarm/src/main.rs
  - crates/reposix-swarm/tests/chaos_audit.rs
  - crates/reposix-swarm/tests/contention_e2e.rs
---

# Phase 21: Code Review Report (Iteration 3 — final)

**Reviewed:** 2026-04-15T00:00:00Z
**Depth:** standard
**Files Reviewed:** 12
**Status:** clean

## Summary

Iteration 3 confirms the regression fix is correct. All blocking and warning findings from
iterations 1 and 2 are resolved.

- **WR-01 (iteration 2)**: Unit test `update_issue_409_maps_to_conflict_error` at line 1984-1986
  now correctly asserts `m.starts_with("confluence version conflict")` — which matches the
  production CONFLICT arm at line 1063-1068 that emits
  `"confluence version conflict for PUT {redacted_url}: {body_preview}"`. Test and production code
  are in sync. No CI breakage.

Three info-level findings remain open (non-blocking, no action required to ship):

- **IN-01**: `chaos_audit.rs` hardcodes port 7979 — collision risk under parallel `--ignored` runs.
- **IN-02**: `ConfLinks` struct is dead code masked by `#[allow(dead_code)]`.
- **IN-03**: `contention_e2e` invariants use fragile string match on rendered Markdown.

---

## Info

### IN-01: Chaos test uses fixed port 7979 — concurrent `--ignored` runs will collide

**File:** `crates/reposix-swarm/tests/chaos_audit.rs:41`

**Issue:** `SIM_BIND` is hardcoded to `127.0.0.1:7979`. The `pkill -f reposix-sim` teardown at
line 164 can kill a legitimate simulator process from a parallel test run or developer session.

**Fix:** Bind to port 0 (ephemeral), capture the assigned port, and use it for both health-check
and load URLs.

---

### IN-02: `ConfLinks` struct is dead code — `#[allow(dead_code)]` masks a structural oddity

**File:** `crates/reposix-confluence/src/lib.rs:214-223`

**Issue:** `ConfLinks.next` is annotated `#[allow(dead_code)]` because cursor extraction goes
through `parse_next_cursor` rather than the typed struct. The struct is deserialized and
immediately discarded (`let _ = list.links`). Not a bug but a maintenance hazard.

**Fix:** Remove `ConfLinks` and the typed `links` field from `ConfPageList` and use
`parse_next_cursor` exclusively, or remove `parse_next_cursor` and read `list.links.next` directly.

---

### IN-03: `contention_e2e` invariant uses fragile string match on rendered Markdown

**File:** `crates/reposix-swarm/tests/contention_e2e.rs:119-122`

**Issue:** `markdown.contains("| patch ")` and `"| Conflict"` are format-coupled to the Markdown
renderer. A column rename or padding change silently breaks invariant detection.

**Fix:** Return a structured summary type from `run_swarm` for tests to assert on counts directly,
or add a comment noting these assertions must be updated if `metrics.rs` render format changes.

---

_Reviewed: 2026-04-15T00:00:00Z (iteration 3 — final)_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
