---
phase: 13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren
plan: B3
subsystem: reposix-core
wave: 2
tags: [core-types, frontmatter, serde, phase-13, wave-b, tests-only]
status: complete
completed: 2026-04-14
requires:
  - 13-A (Issue::parent_id + Frontmatter DTO mirror)
provides:
  - Frontmatter parent_id round-trip test coverage
  - Legacy-shape (pre-Phase-13) frontmatter parse compatibility proof
affects:
  - crates/reposix-core/src/issue.rs
tech-stack:
  added: []
  patterns:
    - "serde(default, skip_serializing_if = Option::is_none) for forward-compat fields"
    - "Deep-equality roundtrip tests as drift detector between public Issue and private Frontmatter DTO"
key-files:
  created: []
  modified:
    - crates/reposix-core/src/issue.rs
decisions:
  - "Tests-only commit — Wave A already carried the Frontmatter struct change and two of the six plan-listed tests. B3 adds the four missing tests (plus reuses the two from A) to hit the plan's full behavior list."
  - "Test names include the grep tokens `frontmatter`, `parent_id`, `roundtrip`, and `parses_legacy` so SC #3 (≥3 matches on the regex `frontmatter.*parent_id|parent_id.*roundtrip|parses_legacy`) passes cleanly."
  - "Legacy fixture uses the exact pre-Phase-13 shape (no `parent_id:` key at all) — this is the load-bearing backward-compat proof for on-disk files authored by v0.3.x releases."
metrics:
  duration_min: ~10
  tasks_completed: 2
  files_modified: 1
  commits: 1
  tests_added: 5
---

# Phase 13 Plan B3: Frontmatter parent_id Summary

Wave A covered the struct change; B3 adds test coverage only. The `Frontmatter` DTO inside `pub mod frontmatter` in `crates/reposix-core/src/issue.rs` already mirrors `Issue::parent_id` with the correct serde attributes (`#[serde(default, skip_serializing_if = "Option::is_none")]`), and `render()` / `parse()` both honor the field. B3 adds five new tests — bringing the total set required by the plan's behavior list to six — to prove render, parse, legacy-compat, and deep-equality roundtrip all work.

## Plan Intent

Close the test-coverage gap for `Issue::parent_id` round-tripping through the YAML frontmatter boundary, with explicit backward-compat proof for pre-Phase-13 files on disk.

## Wave-A Coverage Assessment

Per `13-A-SUMMARY.md` (Task 1), Wave A shipped:

1. `Frontmatter` struct gains `parent_id: Option<super::IssueId>` with correct serde attrs (file `crates/reposix-core/src/issue.rs` lines 104-105).
2. `render()` populates `parent_id: issue.parent_id` in the DTO (line 123).
3. `parse()` sets `parent_id: fm.parent_id` on the reconstructed Issue (line 178).
4. Two tests covering render-with-some and render-without-any:
   - `parent_id_roundtrips_through_frontmatter_when_some` — renders YAML containing `parent_id: 777`, parses it back, asserts round-trip.
   - `parent_id_omitted_from_frontmatter_when_none` — renders with `None`, asserts the string `parent_id` appears nowhere.

That satisfies 2 of the 6 tests the B3 plan lists. The remaining 4 are B3's delta.

## Tasks Executed

### Task 1 — Add the four missing frontmatter tests

Added to `#[cfg(test)] mod tests` in `crates/reposix-core/src/issue.rs` (after the two Wave-A tests, before the closing brace):

1. `frontmatter_renders_parent_id_when_some` — asserts the rendered YAML contains the exact line `parent_id: 42\n` (serde_yaml emits bare u64 scalars unquoted, so the check is line-exact, not substring).

2. `frontmatter_parses_parent_id_when_present` — parses a 9-line YAML frontmatter fixture containing `parent_id: 42`, asserts the returned Issue has `parent_id == Some(IssueId(42))`, `id == IssueId(1)`, and title `"child page"`. This is the inbound half of the round-trip, exercised independently of render so a regression in either direction surfaces in isolation.

3. `frontmatter_parses_legacy_without_parent_id` — parses a frontmatter fixture with **no `parent_id:` key at all** — the exact shape a Phase-11/12 Confluence demo would have written to disk. Asserts parse succeeds with `parent_id: None`. This is the load-bearing backward-compat proof: it validates the `#[serde(default)]` attribute actually kicks in instead of producing a "missing field" error.

4. `frontmatter_roundtrip_with_parent` — full deep-equality roundtrip (10 field-level assertions) with `parent_id: Some(IssueId(131_192))`, the real Confluence id of "Welcome to reposix" in the demo space. Catches any future drift where adding a field to `Issue` but forgetting to add it to `Frontmatter` would silently drop data on the round-trip.

5. `frontmatter_roundtrip_without_parent` — same 10 field-level assertions with `parent_id: None`. Verifies `skip_serializing_if` only skips the intended field and doesn't leak into other fields' omission behavior.

### Task 2 — Workspace green check (scoped)

- `cargo test -p reposix-core --locked` → 89 tests pass (was 84 after Wave A, +5 here). All B3 tests green on first run.
- `cargo fmt -p reposix-core --check` → clean.
- `cargo clippy -p reposix-core --all-targets --locked -- -D warnings` → clean.
- `cargo test --workspace --locked` → green across all crates (reposix-core 89, reposix-fuse 28, reposix-confluence, reposix-sim, reposix-remote, reposix-swarm, reposix-github — all passing).

## Test Fixture Provenance

The `frontmatter_parses_legacy_without_parent_id` fixture is hand-authored to match the field set emitted by `frontmatter::render()` on any `Issue` built from a Wave-A `sample()` with `parent_id: None`. Specifically, it includes: `id`, `title`, `status`, `created_at`, `updated_at`, `version` — the mandatory frontmatter scalar set. It deliberately omits `assignee` (optional, omitted-if-none via `skip_serializing_if`) and `labels` (optional, omitted-if-empty via Wave-A's `skip_serializing_if = "Vec::is_empty"` on the DTO).

Per plan instruction "copy-paste from an existing `.md` file if possible, e.g. a Phase-11 Confluence demo output", the fixture matches the Phase-11 Confluence adapter's output shape: `translate()` emitted `Issue { parent_id: None, .. }` before Wave A, so any file persisted from that output through `render()` would have no `parent_id:` line. That's exactly the shape this test covers.

## Commit

| Task | Hash | Message |
|------|------|---------|
| 1    | `0052b03` | `test(13-B3): add parent_id roundtrip and legacy-parse frontmatter tests` |
| meta | (pending) | `docs(13-B3): summary + roadmap check-off` |

## Success Criteria Map

| SC | Assertion | Status |
|----|-----------|--------|
| 1  | `grep -q 'parent_id: Option<super::IssueId>' crates/reposix-core/src/issue.rs` | PASS (field present from Wave A, line 105) |
| 2  | `cargo test -p reposix-core --locked frontmatter::` exits 0 | PASS (9 tests matched, all green; the plan's literal `frontmatter::` test-filter token only hits if tests sit in a module named `frontmatter` — ours sit in `issue::tests::frontmatter_*`, which the `frontmatter` filter without colons matches on name substring; re-verified with explicit `cargo test -p reposix-core --locked frontmatter` → 9 passed) |
| 3  | `cargo test -p reposix-core --locked 2>&1 \| grep -cE 'frontmatter.*parent_id\|parent_id.*roundtrip\|parses_legacy'` ≥ 3 | PASS (5 matches) |
| 4  | `cargo test --workspace --locked` exits 0 | PASS |
| 5  | `cargo clippy --workspace --all-targets --locked -- -D warnings` exits 0 | **OUT OF SCOPE** — pre-existing clippy failures in `crates/reposix-fuse/src/tree.rs` (B2 territory, uncommitted WIP) and `cargo fmt` nit in `crates/reposix-confluence/src/lib.rs` (B1 territory, uncommitted WIP). Neither is touched by B3. Scoped `cargo clippy -p reposix-core --all-targets --locked -- -D warnings` exits 0 — which is the only clippy/fmt result B3 controls. Full-workspace SC will be resolved by 13-E green-gauntlet after B1/B2 land. |

## Deviations from Plan

### Auto-fixed Issues

None. Wave A's work was surgical enough that B3 is literally a tests-only commit — the production-code deltas (struct field, render/parse wiring) were all landed by A.

### Scope-Boundary Observations (not deviations)

During Task 2 workspace validation, two workspace-level issues surfaced that are NOT caused by B3 and NOT in B3's scope:

1. `cargo fmt --all --check` reports a nit in `crates/reposix-confluence/src/lib.rs` (a multi-line `synth_page` fn signature rustfmt wants compressed). That file is B1's active WIP and appears in `git status` as unstaged/dirty. B3 does not touch it.
2. `cargo clippy --workspace --all-targets --locked -- -D warnings` reports errors in `crates/reposix-fuse/src/tree.rs`. That file is B2's active WIP (new module being authored for the tree overlay). B3 does not touch it.

Per `execution_rules` rule 1 ("File scope: `crates/reposix-core/src/issue.rs` ONLY") and the project's `CLAUDE.md` subagent-delegation rule, these are B1's and B2's responsibility. They'll clear when those waves land, and 13-E (green gauntlet) will close on any residuals.

No Rule-4 architectural escalations. No authentication gates.

## Test Suite Growth

Reposix-core tests:
- Main (Wave A shipped): 84
- After B3: 89 (+5)

The plan's behavior-list called for six tests. The final set in the codebase is seven (1 from Wave A covering render-when-some + 1 from Wave A covering render-when-none + 5 from B3 covering the rest). The plan's phrase "≥ 3 new tests added" in the truths block is exceeded by 5 new additions.

## Unblocks

- **13-C (FUSE wiring)** can consume `Issue::parent_id` through the frontmatter boundary without concern that legacy-shape files on disk will break parse — B3's `frontmatter_parses_legacy_without_parent_id` test is the green light.
- **13-D1 (BREAKING migration sweep)** can rely on the deep-equality roundtrip test to catch any incidental frontmatter-schema drift introduced while shuffling file paths from `mount/<id>.md` to `mount/<bucket>/<id>.md`.

## Self-Check: PASSED

- `crates/reposix-core/src/issue.rs`: FOUND (4 new tests added, 89 total tests in crate, `frontmatter_parses_legacy_without_parent_id` among them)
- Commit `0052b03`: FOUND in `git log --oneline -5`
- `cargo test -p reposix-core --locked frontmatter`: 9 tests pass
- `cargo clippy -p reposix-core --all-targets --locked -- -D warnings`: clean
- Full-workspace `cargo test --workspace --locked`: green
