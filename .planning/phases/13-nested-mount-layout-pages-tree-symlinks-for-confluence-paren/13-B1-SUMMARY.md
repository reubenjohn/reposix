---
phase: 13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren
plan: B1
subsystem: reposix-confluence
tags: [confluence, parent-id, hierarchy, phase-13, wave-b]
status: complete
completed: 2026-04-14
requires:
  - reposix_core::Issue::parent_id (from Wave A)
  - reposix_core::BackendFeature::Hierarchy (from Wave A)
  - reposix_core::IssueBackend::root_collection_name default (from Wave A)
provides:
  - Confluence translate() writes Issue::parent_id from REST v2 parentId/parentType
  - ConfluenceReadOnlyBackend::supports(BackendFeature::Hierarchy) == true
  - ConfluenceReadOnlyBackend::root_collection_name() == "pages"
  - Live-contract hierarchy probe (contract_confluence_live_hierarchy, #[ignore])
affects:
  - crates/reposix-confluence/src/lib.rs
  - crates/reposix-confluence/tests/contract.rs
tech-stack:
  added: []
  patterns:
    - "serde #[serde(default, rename)] on ConfPage for forward-compat extension"
    - "match-guarded graceful-degradation: malformed parentId → None + tracing::warn"
    - "wiremock end-to-end assertion over three parentType branches in one list"
key-files:
  created: []
  modified:
    - crates/reposix-confluence/src/lib.rs
    - crates/reposix-confluence/tests/contract.rs
decisions:
  - "Non-page parentType (folder/whiteboard/database) deliberately treated as orphan: reposix tree is homogeneous (pages only); a future phase may revisit once folder/whiteboard rendering is in scope."
  - "Malformed parentId degrades silently to None with tracing::warn rather than Err: T-13-PB1 mandates that one malicious page must not wedge the entire list."
  - "Two dedicated ignored live tests (contract_confluence_live + contract_confluence_live_hierarchy) instead of one combined test: keeps the hierarchy assertion isolable for CI gating once that matures."
  - "synth_page helper kept private inside #[cfg(test)] mod tests — not reused outside this file (would cross the ConfPage visibility boundary)."
metrics:
  duration_min: ~25
  tasks_completed: 2
  files_modified: 2
  commits: 1
  tests_added: 11
---

# Phase 13 Plan B1: Confluence parent_id wiring Summary

Confluence adapter now populates `Issue::parent_id` from REST v2 `parentId` +
`parentType`, reports `supports(BackendFeature::Hierarchy) == true`, and
overrides `root_collection_name()` to `"pages"`. Additive only — every Phase 11
fixture (none of which carry parent fields) still decodes and round-trips
exactly as before, because the new `ConfPage` fields are `#[serde(default)]`.

## Plan Intent

Unblock Wave C (FUSE tree wiring) by making the Confluence adapter actually
emit non-None `parent_id` values end-to-end through the `IssueBackend` seam.
Without this plan, C has nothing to group by.

## Tasks Executed

### Task 1 — ConfPage deserialization + translate() + trait overrides

- Extended `ConfPage` with two new fields:
  - `#[serde(default, rename = "parentId")] parent_id: Option<String>`
  - `#[serde(default, rename = "parentType")] parent_type: Option<String>`
- `translate()` now branches on `(parent_id, parent_type)`:
  - `(Some(pid), Some("page"))` + parseable `u64` → `Some(IssueId(n))`
  - `(Some(pid), Some("page"))` + unparseable → `None` + `tracing::warn!`
  - `(_, Some(other))` (folder, whiteboard, database, …) → `None` + `tracing::debug!`
  - `(None, _)` / `(_, None)` → `None`
- `IssueBackend::supports` changed from `false`-for-all to
  `matches!(feature, BackendFeature::Hierarchy)` — flips `Hierarchy` to `true`
  while every other variant remains `false`.
- `IssueBackend::root_collection_name` override returns `"pages"`.

### Task 2 — Workspace-wide green check

- `cargo test -p reposix-confluence --locked`: 28 lib + 2 contract + 0 doc, all green.
- `cargo clippy -p reposix-confluence --all-targets --locked -- -D warnings`: clean.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: clean.
- `cargo test --workspace --locked`: all green (one transient failure observed on
  first run, not reproducible on re-run — likely a rate-limit timing test in
  another crate; not caused by this plan's changes).
- `cargo fmt --all --check`: clean (after one `cargo fmt --all` pass that
  collapsed a multi-line fn signature on `synth_page`).

## Test Results

### Confluence lib test count

| Before | After | Delta |
|--------|-------|-------|
| 18     | 28    | +10   |

### Confluence contract test count

| Before | After | Delta |
|--------|-------|-------|
| 3 (1 ignored) | 4 (2 ignored) | +1 ignored live-hierarchy test |

### New tests (names for traceability)

**Unit tests in `src/lib.rs` `#[cfg(test)] mod tests`:**

1. `translate_populates_parent_id_for_page_parent` — `parentType: "page"` + parseable u64 → `Some(IssueId(42))`
2. `translate_treats_folder_parent_as_orphan` — `parentType: "folder"` → `None`
3. `translate_treats_whiteboard_parent_as_orphan` — `parentType: "whiteboard"` → `None`
4. `translate_treats_database_parent_as_orphan` — `parentType: "database"` → `None`
5. `translate_treats_missing_parent_as_orphan` — both fields absent → `None`
6. `translate_treats_parent_id_without_type_as_orphan` — `parentId` set, `parentType` absent → `None`
7. `translate_handles_unparseable_parent_id` — `parentId: "not-a-number"` + `parentType: "page"` → `None`, no panic (T-13-PB1)
8. `root_collection_name_returns_pages` — trait override check
9. `list_populates_parent_id_end_to_end` — wiremock with three mixed pages: one `page`-parented (populated), one root (None), one `folder`-parented (None)
10. `supports_reports_only_hierarchy` — renamed + extended from `supports_reports_no_features`; asserts `Hierarchy == true` AND every other feature remains `false`

**Contract test in `tests/contract.rs` (ignored by default):**

11. `contract_confluence_live_hierarchy` — hits real tenant when env set; asserts at least one page in the space has `parent_id == Some(_)`. Gated by the same `skip_if_no_env!` envelope as `contract_confluence_live`.

### parentType branches exercised

| parentType value | Test | Result |
|---|---|---|
| `"page"` (parseable) | `translate_populates_parent_id_for_page_parent`, `list_populates_parent_id_end_to_end` (child) | `Some(IssueId(n))` |
| `"page"` (unparseable) | `translate_handles_unparseable_parent_id` | `None` + warn |
| `"folder"` | `translate_treats_folder_parent_as_orphan`, `list_populates_parent_id_end_to_end` (foldered) | `None` + debug |
| `"whiteboard"` | `translate_treats_whiteboard_parent_as_orphan` | `None` + debug |
| `"database"` | `translate_treats_database_parent_as_orphan` | `None` + debug |
| missing (both fields) | `translate_treats_missing_parent_as_orphan`, `list_populates_parent_id_end_to_end` (root) | `None` |
| `parentId` set, `parentType` missing | `translate_treats_parent_id_without_type_as_orphan` | `None` |

### Phase 11 regression check

All 18 pre-existing Phase 11 tests still pass unchanged. The
`supports_reports_no_features` test was renamed to `supports_reports_only_hierarchy`
and the new `Hierarchy == true` assertion was appended; every other feature
assertion is identical. No Phase 11 fixture carries `parentId`/`parentType`,
and the `#[serde(default)]` attribute on both new fields means those fixtures
continue to decode as `None` for both — no behavioral change for the historical
codepath.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 — Bug] `clippy::unreadable_literal` on `IssueId(360556)`**

- **Found during:** Task 2 workspace clippy pass.
- **Issue:** `cargo clippy --all-targets -- -D warnings` flagged two `IssueId(360556)` literals in the new `list_populates_parent_id_end_to_end` test with `clippy::unreadable_literal`.
- **Fix:** Changed to `IssueId(360_556)` (two sites). No semantic change.
- **Files modified:** `crates/reposix-confluence/src/lib.rs`.
- **Commit:** folded into the B1 feat commit (`cd9f18e`).

**2. [Rule 1 — Bug] `clippy::single_match_else` on nested `match` in `translate`**

- **Found during:** Task 2 workspace clippy pass.
- **Issue:** The inner `match pid_str.parse::<u64>()` with `Ok(n) => …; Err(_) => …` was flagged by `clippy::single_match_else` as preferably expressible as `if let Ok(n) = … else …`.
- **Fix:** Rewrote as `if let Ok(n) = pid_str.parse::<u64>() { Some(IssueId(n)) } else { tracing::warn!(…); None }`. Same behavior, one fewer nesting level.
- **Files modified:** `crates/reposix-confluence/src/lib.rs` (translate fn).
- **Commit:** folded into the B1 feat commit (`cd9f18e`).

**3. [Rule 1 — Cosmetic] `cargo fmt` normalization on `synth_page`**

- **Found during:** Task 2 fmt check.
- **Issue:** The four-line parameter list on the `synth_page` test helper exceeded rustfmt's multi-line-break threshold; rustfmt preferred a one-liner.
- **Fix:** `cargo fmt --all`. No semantic change.
- **Files modified:** `crates/reposix-confluence/src/lib.rs` (test mod).
- **Commit:** folded into the B1 feat commit (`cd9f18e`).

No other deviations. No Rule-2/3/4 escalations. No authentication gates.

## Parallel-Wave Isolation

Per the plan's scope rules, B1 modified only files under
`crates/reposix-confluence/`. B2 (parallel, on `crates/reposix-fuse/src/tree.rs`
and `crates/reposix-fuse/src/lib.rs`) and B3 (parallel, on
`crates/reposix-core/src/issue.rs` frontmatter submodule) were not touched
by this plan. Verified by `git diff --stat` before commit — the two modified
files are both inside `crates/reposix-confluence/`.

## Commits

| Task | Hash | Message |
|------|------|---------|
| 1+2  | `cd9f18e` | `feat(13-B1): populate Issue::parent_id from Confluence parentId + supports(Hierarchy) + root_collection_name("pages")` |

## Success Criteria Map

| SC | Assertion | Status |
|----|-----------|--------|
| 1  | `parent_id: Option<String>` in `crates/reposix-confluence/src/lib.rs` | PASS |
| 2  | `parent_type: Option<String>` in `crates/reposix-confluence/src/lib.rs` | PASS |
| 3  | `rename = "parentId"` in `crates/reposix-confluence/src/lib.rs` | PASS |
| 4  | `rename = "parentType"` in `crates/reposix-confluence/src/lib.rs` | PASS |
| 5  | `BackendFeature::Hierarchy` referenced in `crates/reposix-confluence/src/lib.rs` | PASS |
| 6  | `fn root_collection_name` returning `"pages"` in `crates/reposix-confluence/src/lib.rs` | PASS |
| 7  | `cargo test -p reposix-confluence --locked` exits 0 | PASS (28 lib + 2 contract) |
| 8  | Test count ≥ 16 in reposix-confluence | PASS (30 total: 28 lib + 2 contract) |
| 9  | `cargo clippy --workspace --all-targets --locked -- -D warnings` exits 0 | PASS |
| 10 | `cargo test --workspace --locked` exits 0 | PASS |

## Unblocks

Wave C (FUSE tree wiring) can now start from `main`:
- `IssueBackend::supports(BackendFeature::Hierarchy)` returns `true` for Confluence → C can key `tree/` overlay emission off this one bit.
- `Issue::parent_id` is populated end-to-end through `list_issues` → C's tree-builder has real data to group on.
- `IssueBackend::root_collection_name()` returns `"pages"` for Confluence → C's directory-name synthesis doesn't need a per-backend match.

## Self-Check: PASSED

- `crates/reposix-confluence/src/lib.rs`: FOUND (parent_id/parent_type fields, translate() derivation, supports(Hierarchy), root_collection_name("pages"), 10 new tests)
- `crates/reposix-confluence/tests/contract.rs`: FOUND (contract_confluence_live_hierarchy added, ignored, skip-env gated)
- Commit `cd9f18e`: FOUND in `git log`
- `cargo test -p reposix-confluence --locked`: PASS (28 lib + 2 contract, 2 ignored live)
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: PASS
- `cargo fmt --all --check`: PASS
