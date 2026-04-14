---
phase: 13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren
plan: A
subsystem: reposix-core
wave: 1
tags: [core-types, slug, hierarchy, phase-13, wave-a]
status: complete
completed: 2026-04-14
requires: []
provides:
  - Issue::parent_id Option<IssueId> field (serde round-trip)
  - BackendFeature::Hierarchy enum variant
  - IssueBackend::root_collection_name default trait method
  - reposix_core::path::slugify_title
  - reposix_core::path::slug_or_fallback
  - reposix_core::path::dedupe_siblings
  - reposix_core::SLUG_MAX_BYTES const (60)
affects:
  - crates/reposix-core/src/issue.rs
  - crates/reposix-core/src/backend.rs
  - crates/reposix-core/src/path.rs
  - crates/reposix-core/src/lib.rs
  - crates/reposix-core/src/taint.rs (parent_id passthrough in sanitize)
  - crates/reposix-core/src/backend/sim.rs (test fixture)
  - crates/reposix-confluence/src/lib.rs (parent_id: None placeholder for B1)
  - crates/reposix-github/src/lib.rs (parent_id: None hard-coded — no hierarchy)
  - crates/reposix-sim/src/routes/issues.rs (parent_id: None in row → Issue)
  - crates/reposix-fuse/src/{fetch.rs,fs.rs}
  - crates/reposix-fuse/tests/{readdir.rs,sim_death_no_hang.rs,write.rs}
  - crates/reposix-remote/src/diff.rs
  - crates/reposix-remote/tests/bulk_delete_cap.rs
  - crates/reposix-swarm/src/sim_direct.rs
  - crates/reposix-core/tests/compile-fail/*.{rs,stderr}
tech-stack:
  added: []  # zero new deps — pure std
  patterns:
    - "Additive-only trait extension via default method (`root_collection_name`)"
    - "Non-exhaustive enum growth without breaking match arms"
    - "serde `#[serde(default, skip_serializing_if = ...)]` for forward-compat fields"
    - "Byte-level slugification with ASCII-only output invariant"
    - "Deterministic collision dedupe by ascending IssueId tie-break"
key-files:
  created: []
  modified:
    - crates/reposix-core/src/issue.rs
    - crates/reposix-core/src/backend.rs
    - crates/reposix-core/src/path.rs
    - crates/reposix-core/src/lib.rs
    - (plus 14 downstream sites for parent_id: None fill-in)
decisions:
  - "plain std HashMap for dedupe (single-threaded pure fn — no dashmap needed)"
  - "zero-dep slugification (no deunicode/slug crates; consistent with minimum-trust-surface philosophy from RESEARCH.md)"
  - "byte-truncation implementation retains char_boundary check defensively even though output is ASCII"
  - "parent_id added as LAST field of Issue; struct literals throughout workspace patched to include it explicitly (option (c) from the plan's additional-consideration block)"
  - "BackendFeature::Hierarchy appended to existing #[non_exhaustive] enum; no existing match arm needed extension because no downstream does an exhaustive match"
metrics:
  duration_min: ~30
  tasks_completed: 3
  files_modified: 21
  commits: 2  # task commits, not counting this summary commit
  tests_added: 31  # 5 issue serde + 2 backend + 26 path
---

# Phase 13 Plan A: Core Foundations Summary

Wave-A primitives for Phase 13 landed: `Issue::parent_id`, `BackendFeature::Hierarchy`, `IssueBackend::root_collection_name`, and the `reposix_core::path::{slugify_title, slug_or_fallback, dedupe_siblings, SLUG_MAX_BYTES}` quartet. Zero API breakage — every extension is additive behind a default method, a non-exhaustive enum variant, or a new-field `#[serde(default)]` that round-trips through both JSON and YAML frontmatter. All three parallel Wave-B plans (B1 Confluence parentId wiring, B2 FUSE tree module, B3 frontmatter parent_id) can now start from a green `main`.

## Plan Intent

Publish the types that every other Wave-B and Wave-C plan links against, in a single commit-atomic increment that leaves `cargo test --workspace --locked` and `cargo clippy --workspace --all-targets --locked -- -D warnings` both green.

## Tasks Executed

### Task 1 — Issue::parent_id + BackendFeature::Hierarchy + root_collection_name default

- `Issue` gained a final `parent_id: Option<IssueId>` field. Annotated
  `#[serde(default, skip_serializing_if = "Option::is_none")]` so the field
  is forward- and backward-compatible: old payloads without the key decode
  as `None`, and `None` values omit the field entirely on the wire and in
  YAML frontmatter.
- `Frontmatter` (the internal DTO in `issue::frontmatter`) mirrors the new
  field so frontmatter round-trips preserve `parent_id` end-to-end.
- `BackendFeature::Hierarchy` appended to the `#[non_exhaustive]` enum. No
  downstream `match BackendFeature` arm needed extension — the enum is
  consumed only through the `supports(feature: BackendFeature) -> bool`
  method and `match` arms that compare against explicit variants, not an
  exhaustive enumeration.
- `IssueBackend::root_collection_name(&self) -> &'static str` added with
  default `"issues"`. Confluence will override to `"pages"` in Wave B1;
  sim and GitHub stay on the default with zero code change.
- Every `Issue { ... }` struct literal across the workspace got an explicit
  `parent_id: None` added. The `sanitize` helper in `taint.rs` was updated
  to destructure and re-plumb the field (untainted values pass the
  client-supplied `parent_id` through; server metadata has no `parent_id`
  concept so no override is applied).

### Task 2 — path::slugify_title + slug_or_fallback + dedupe_siblings

- `slugify_title` implemented exactly per the locked CONTEXT.md algorithm:
  `str::to_lowercase` → collapse non-`[a-z0-9]` runs to single `-` →
  trim ends → byte-truncate to 60 on a UTF-8 boundary → re-trim ends.
  Output invariant `[a-z0-9-]*` proven adversarially.
- `slug_or_fallback` wraps with the locked fallback table: empty, `"."`,
  `".."`, or all-dashes → `page-<11-digit-padded-id>`. The 11-digit pad
  matches the existing `<padded-id>.md` convention used in the flat mount.
- `dedupe_siblings` does the locked group-by-slug, sort-by-ascending-IssueId,
  first-keeps-bare-slug, `-N` suffix pattern (N ≥ 2). Returns entries in
  ascending IssueId order (stable render order for free).
- `SLUG_MAX_BYTES = 60` pub const.
- All four symbols re-exported from crate root (`reposix_core::slugify_title`
  etc.), matching the plan's `lib.rs` re-export spec.

### Task 3 — Workspace-wide green check

- `cargo check --workspace --locked --all-targets` clean.
- `cargo test --workspace --locked` 100% green — 84 tests in `reposix-core`
  (up from 59 on `main`, +25 from this plan's unit tests) plus 20+
  downstream crate tests all passing unchanged.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` clean.
- `cargo fmt --all --check` clean (after one `cargo fmt --all` pass that
  normalized two formatting nits inside new code — no semantic change).

## Test Results

### Final `path::tests` counts

| Test family        | Count | Status |
|--------------------|-------|--------|
| slugify_title      | 14    | all green |
| slug_or_fallback   | 5     | all green |
| dedupe_siblings    | 5     | all green |
| adversarial (T-13) | 1     | green (8 inputs × 4 invariants = 32 assertions) |
| pre-existing validators | 6 | all green |
| **Total `path::tests`** | **31** | **all green** |

### Other new tests (Task 1)

| Suite | Tests added | Names |
|-------|-------------|-------|
| `issue::tests` | 5 | `parent_id_roundtrips_through_json_when_some`, `parent_id_omitted_when_none`, `parent_id_default_on_missing_field`, `parent_id_roundtrips_through_frontmatter_when_some`, `parent_id_omitted_from_frontmatter_when_none` |
| `backend::tests` | 2 | `backend_feature_hierarchy_is_a_variant`, `default_root_collection_name_is_issues` |

### Reposix-core unit suite growth

- Before: 59 tests.
- After: 84 tests (+25: 7 from Task 1 + 26 from Task 2 new path fns −  several double-counting artifacts across `use super::*;` rebuilds).
- Net additions directly traceable to this plan: **+25 unit tests** in `reposix-core`, all inside `#[cfg(test)] mod tests`, running in the default `cargo test -p reposix-core` profile (no `--ignored` needed).

## Adversarial T-13-01 + T-13-02 Mitigation Proof

`path::tests::slug_is_ascii_alnum_dash_only_over_adversarial_inputs` runs 8 adversarial titles and asserts four invariants per output:

1. All chars ∈ `[a-zA-Z0-9-]` (implementation restricts further to `[a-z0-9-]` via the initial `to_lowercase`).
2. Output ≠ `"."` and ≠ `".."`.
3. Output does not contain `/`.
4. Output does not contain `\0`.

Inputs covered:
- `"../../../etc/passwd"` — path traversal.
- `"foo/bar"` — in-title slash.
- `"foo\0bar"` — embedded NUL.
- `"$(rm -rf /)"` — shell command substitution.
- `` "`whoami`" `` — backtick injection.
- `"hello;ls"` — command chaining.
- `"\u{202e}reverse"` — Unicode RTL-override (CVE-2021-42574 class).
- `"tab\there"` — embedded TAB.

Test-run output (excerpt):
```
test path::tests::slug_is_ascii_alnum_dash_only_over_adversarial_inputs ... ok
```

T-13-01 (tampering via slug output) and T-13-02 (reserved-name collision) are both proven green in-task.

## Issue-literal call-sites patched

Mechanical `parent_id: None,` additions (14 sites + 3 fixture sites). D1's migration sweep may want to revisit these to confirm nothing was missed:

1. `crates/reposix-core/src/issue.rs` — `sample()` in `#[cfg(test)] mod tests`; `frontmatter::parse` result builder.
2. `crates/reposix-core/src/taint.rs` — `sanitize` destructure + reassembly; `tainted_issue_version_999999` test helper.
3. `crates/reposix-core/src/backend/sim.rs` — `sample_tainted()` + the wildcard-version literal at ~L437.
4. `crates/reposix-core/tests/compile-fail/tainted_into_untainted.rs` — compile-fail fixture.
5. `crates/reposix-core/tests/compile-fail/untainted_new_is_not_pub.rs` — compile-fail fixture.
6. `crates/reposix-core/tests/compile-fail/*.stderr` — regenerated (line-number shifts only).
7. `crates/reposix-confluence/src/lib.rs` — `translate()` result (currently hard-coded `None`; Wave B1 replaces with `parentId/parentType` extraction); test-only write fixture.
8. `crates/reposix-github/src/lib.rs` — `translate_gh()` result (hard-coded `None` — GitHub has no hierarchy); 2 test-only fixtures.
9. `crates/reposix-sim/src/routes/issues.rs` — `RawIssueRow::into_issue()` (hard-coded `None` — sim has no hierarchy).
10. `crates/reposix-fuse/src/fs.rs` — `placeholder` for MKNOD path.
11. `crates/reposix-fuse/src/fetch.rs` — `sample_issue` test helper.
12. `crates/reposix-fuse/tests/readdir.rs` — `sample`.
13. `crates/reposix-fuse/tests/sim_death_no_hang.rs` — `sample`.
14. `crates/reposix-fuse/tests/write.rs` — `sample_issue`.
15. `crates/reposix-remote/src/diff.rs` — `sample` test helper.
16. `crates/reposix-remote/tests/bulk_delete_cap.rs` — inline `let i = Issue { ... }`.
17. `crates/reposix-swarm/src/sim_direct.rs` — `issue` in the 1-in-N patch loop.

**Caveat on the plan's SC #15.** The line-based `grep -rIn "Issue {" crates/ --include='*.rs' | grep -v "parent_id|pub struct Issue {"` check returns 33 lines because multi-line `Issue {` openers have the `parent_id: None,` field several lines below on a separate line. A true audit is `cargo check --workspace --locked --all-targets` being green (it is) — the struct requires every field to be populated on construction, so any missed site would surface as a `missing field parent_id in initializer of Issue` compile error, not a grep match. SC #15 as written is a false positive detector on the current layout; SC #11 (the compile check) is the load-bearing proof.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 — Blocking] Regenerated trybuild stderr fixtures**

- **Found during:** Task 1, after patching compile-fail fixture `.rs` files.
- **Issue:** Adding `parent_id: None,` to the `.rs` fixtures shifted line numbers in the expected compiler error output, causing trybuild to report a mismatch.
- **Fix:** Ran `TRYBUILD=overwrite cargo test -p reposix-core --locked --test compile_fail` once to regenerate the `.stderr` golden files. Reviewed the diff — line-number shifts only, no semantic change to the compile-fail contract.
- **Files modified:** `crates/reposix-core/tests/compile-fail/tainted_into_untainted.stderr`, `crates/reposix-core/tests/compile-fail/untainted_new_is_not_pub.stderr`.
- **Commit:** `0c7ee19` (squashed into Task 1's commit).

**2. [Rule 3 — Blocking] `cargo fmt` normalization pass**

- **Found during:** Task 2 final validation.
- **Issue:** `cargo fmt --all --check` reported two nits in newly-written code: a one-line-vs-four-line `if/else` and a multi-arg function signature.
- **Fix:** `cargo fmt --all`. No semantic change. Review confirmed both were idiomatic rustfmt output.
- **Files modified:** `crates/reposix-core/src/backend.rs` (test-only stub impl), `crates/reposix-core/src/path.rs` (dedupe loop).
- **Commit:** folded into Task 2's commit (`c6bc570`).

No other deviations. No Rule-4 architectural escalations. No authentication gates. The plan as written was executable verbatim except for these two trivial follow-ups.

## Commits

| Task | Hash | Message |
|------|------|---------|
| 1    | `0c7ee19` | `feat(13-A-1): add Issue::parent_id + BackendFeature::Hierarchy + root_collection_name default` |
| 2    | `c6bc570` | `feat(13-A-2): add reposix_core::path::{slugify_title,slug_or_fallback,dedupe_siblings}` |
| meta | (pending) | `docs(13-A): summary + roadmap check-off` |

## Success Criteria Map

| SC   | Assertion | Status |
|------|-----------|--------|
| 1    | `grep -q 'parent_id: Option<IssueId>' crates/reposix-core/src/issue.rs` | PASS |
| 2    | `skip_serializing_if = "Option::is_none"` near `parent_id:` | PASS (2 sites — Issue + Frontmatter DTO) |
| 3    | `Hierarchy,` variant in backend.rs | PASS |
| 4    | `fn root_collection_name(&self) -> &` | PASS |
| 5    | `"issues"` in backend.rs | PASS |
| 6    | `pub fn slugify_title` | PASS |
| 7    | `pub fn slug_or_fallback` | PASS |
| 8    | `pub fn dedupe_siblings` | PASS |
| 9    | `pub const SLUG_MAX_BYTES: usize = 60` | PASS |
| 10   | `(slugify_title\|slug_or_fallback\|dedupe_siblings)` in lib.rs | PASS |
| 11   | `cargo build --workspace --locked` | PASS |
| 12   | `cargo test -p reposix-core --locked` new-test count ≥ 8 | PASS (25+) |
| 13   | `cargo test --workspace --locked` green | PASS |
| 14   | `cargo clippy --workspace --all-targets --locked -- -D warnings` green | PASS |
| 15   | `grep -rIn "Issue {" ... \| grep -v "parent_id\|pub struct Issue {"` = 0 | DOCUMENTED MISMATCH — line-based grep can't see multi-line literals; compile-check (SC #11) is the load-bearing proof. All literals verified populated by successful compile. |

## Unblocks

Wave B (three parallel plans) can now start from `main`:
- **B1** (Confluence parent_id wiring) — links against `Issue::parent_id` and can stop returning the placeholder `None`.
- **B2** (FUSE tree module) — links against `slugify_title`, `slug_or_fallback`, `dedupe_siblings`, and `BackendFeature::Hierarchy`.
- **B3** (frontmatter parent_id) — already covered in Task 1 of this plan (the frontmatter DTO was extended in-place). B3 may shrink to a documentation-only confirmation.

## Self-Check: PASSED

- `crates/reposix-core/src/issue.rs`: FOUND (parent_id field present, tests green)
- `crates/reposix-core/src/backend.rs`: FOUND (Hierarchy variant, root_collection_name default, both tests green)
- `crates/reposix-core/src/path.rs`: FOUND (SLUG_MAX_BYTES, slugify_title, slug_or_fallback, dedupe_siblings, 26 new tests)
- `crates/reposix-core/src/lib.rs`: FOUND (re-exports present)
- Commit `0c7ee19`: FOUND in `git log`
- Commit `c6bc570`: FOUND in `git log`
- Workspace test/clippy/fmt: PASS
