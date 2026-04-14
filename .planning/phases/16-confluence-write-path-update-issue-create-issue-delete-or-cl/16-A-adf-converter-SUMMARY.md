---
phase: 16
wave: A
slug: adf-converter
status: SHIPPED
completed: 2026-04-14
duration_approx: 35m
tasks_completed: 3
tasks_total: 3
test_count_before: 278
test_count_after: 296
tests_added: 18
commits:
  - hash: 48aec91
    message: "feat(16-A): add pulldown-cmark workspace dep"
  - hash: 5c3c273
    message: "feat(16-A): implement adf.rs markdown_to_storage + adf_to_markdown + 18 unit tests"
key_files:
  created:
    - crates/reposix-confluence/src/adf.rs
  modified:
    - crates/reposix-confluence/src/lib.rs
    - Cargo.toml
    - Cargo.lock
    - crates/reposix-confluence/Cargo.toml
subsystem: reposix-confluence
tags: [adf, markdown, confluence, converter, wave-a]
---

# Phase 16 Wave A: ADF Converter SUMMARY

## One-liner

Pure-function `adf.rs` module in `reposix-confluence` with `markdown_to_storage` (pulldown-cmark HTML) and `adf_to_markdown` (recursive `serde_json::Value` visitor), 18 unit tests covering the full WRITE-04 construct matrix.

## Goal

Ship a network-free ADF ↔ Markdown converter as the substrate for Wave B's write methods. No HTTP, no wiremock, no backend — pure functions only.

## Status: SHIPPED

All three tasks completed; all verification gates passed.

## Tasks Completed

| Task | Description | Commit |
|------|-------------|--------|
| A1 | Add `pulldown-cmark = "0.13"` to workspace + confluence Cargo.toml | 48aec91 |
| A2 | Create `adf.rs` with `markdown_to_storage` + `adf_to_markdown` | 5c3c273 |
| A3 | 18 inline unit tests (exceeds required ≥17) | 5c3c273 |

## Verification Gates

| Gate | Result |
|------|--------|
| `cargo test -p reposix-confluence adf` | 18 passed, 0 failed |
| `cargo test -p reposix-confluence` | full crate, no regressions |
| `cargo test --workspace` | 296 total (baseline 278 + 18 new) |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo fmt --all --check` | passes |
| No network calls in adf.rs | confirmed (grep clean) |

## Test Count

- Before: **278**
- After: **296** (+18)
- Required minimum: 293 — **exceeded**

## Success Criteria Verification

1. `crates/reposix-confluence/src/adf.rs` exists, referenced via `pub mod adf;` in `lib.rs` — PASS
2. `pulldown-cmark = "0.13"` in both workspace and confluence Cargo.toml; `cargo tree` shows single v0.13.3 line — PASS
3. `cargo test -p reposix-confluence adf` runs 18 tests, all green — PASS (exceeds ≥15 requirement)
4. `cargo clippy --workspace --all-targets -- -D warnings` clean — PASS
5. Workspace test count ≥ 293 — PASS (296)
6. No network call from any test — PASS

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Missing `Value` import in test module**
- **Found during:** Task A3 compilation
- **Issue:** `#[cfg(test)] mod tests` used `Value` type for the deep-nesting test but only imported `serde_json::json`
- **Fix:** Added `Value` to the `use serde_json::{json, Value}` import
- **Files modified:** `crates/reposix-confluence/src/adf.rs`
- **Commit:** 5c3c273

**2. [Rule 1 - Bug] Clippy pedantic violations**
- **Found during:** Verification gate
- **Issue:** `.map(Vec::as_slice).unwrap_or(&[])` should be `.map_or(&[], Vec::as_slice)`; `level as usize` cast could truncate on 32-bit targets
- **Fix:** Applied clippy suggestions: `map_or`, `usize::try_from(level).unwrap_or(1)`
- **Files modified:** `crates/reposix-confluence/src/adf.rs`
- **Commit:** 5c3c273

**3. [Rule 1 - Style] `cargo fmt` reformatting**
- **Found during:** Verification gate
- **Issue:** Several long function signatures and assert lines exceeded the line length threshold
- **Fix:** `cargo fmt --all` applied
- **Files modified:** `crates/reposix-confluence/src/adf.rs`
- **Commit:** 5c3c273

## Known Stubs

None — this wave is pure-function with no UI or data-source dependencies.

## Threat Flags

None — no new network endpoints, auth paths, or trust-boundary schema changes introduced. The `adf.rs` module is a pure in-process converter. T-16-A-04 (embedded HTML passthrough) is documented in the module doc as an accepted risk for v0.6.0.

## Self-Check: PASSED

- `crates/reposix-confluence/src/adf.rs` exists and is 749 lines
- Commits 48aec91 and 5c3c273 exist in git log
- 296 total workspace tests (verified by `cargo test --workspace`)
- All clippy and fmt gates clean

## Unblocks

Wave B (write methods on `ConfluenceBackend`) and Wave D (struct rename + feature flags) can now proceed. Both depend on `adf::markdown_to_storage` being available.
