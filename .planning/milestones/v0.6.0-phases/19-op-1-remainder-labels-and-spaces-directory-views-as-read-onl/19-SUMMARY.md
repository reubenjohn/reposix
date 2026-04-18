---
phase: "19"
plan_id: "19-B"
subsystem: reposix-fuse
tags: [fuse, labels, overlay, read-only, inode, symlink, release]
dependency_graph:
  requires: [phase-18]
  provides: [labels-overlay-fuse, phase-19-complete]
  affects:
    - crates/reposix-fuse/src/labels.rs
    - crates/reposix-fuse/src/inode.rs
    - crates/reposix-fuse/src/fs.rs
    - crates/reposix-fuse/src/lib.rs
    - crates/reposix-fuse/tests/readdir.rs
    - CHANGELOG.md
    - .planning/STATE.md
    - .planning/ROADMAP.md
tech_stack:
  added: []
  patterns:
    - LabelSnapshot mirrors TreeSnapshot (pure module, no fuser dependency)
    - inode-range dispatch before catch-all (critical ordering)
    - sequential inode allocation (deterministic, no hash collision)
requirements_closed: [LABEL-01, LABEL-02, LABEL-03, LABEL-04, LABEL-05]
key_files:
  created:
    - crates/reposix-fuse/src/labels.rs
    - .planning/phases/19-op-1-remainder-labels-and-spaces-directory-views-as-read-onl/19-SUMMARY.md
  modified:
    - crates/reposix-fuse/src/inode.rs
    - crates/reposix-fuse/src/fs.rs
    - crates/reposix-fuse/src/lib.rs
    - crates/reposix-fuse/tests/readdir.rs
    - CHANGELOG.md
    - .planning/STATE.md
    - .planning/ROADMAP.md
decisions:
  - "Sequential LABELS_DIR_INO_BASE + offset allocation (deterministic, no hash collision risk)"
  - "Label snapshot rebuilt unconditionally on refresh_issues (mirrors tree snapshot, prevents stale-data after relabel)"
  - "labels/ row unconditional in render_mount_root_index even when label_count=0"
  - "GITIGNORE_BYTES updated to /tree/\\nlabels/\\n (15 bytes) so git status stays clean"
  - "Entry slugs include .md suffix — consistent with bucket filenames and tree/ targets"
  - "spaces/ deferred — requires new IssueBackend::list_spaces() method (API-breaking, Confluence-only)"
metrics:
  duration: ~35 minutes (19-A ~25m + 19-B ~10m)
  completed: 2026-04-15
  tasks_completed: 4
  files_changed: 8
status: complete
---

# Phase 19 Summary: OP-1 remainder — `labels/` read-only symlink overlay

## One-liner

`labels/` read-only FUSE overlay built via `LabelSnapshot` module with full `InodeKind` dispatch,
`readlink`/`readdir`/`lookup`/`getattr`/write-EROFS arms, `.gitignore` + `_INDEX.md` updates,
and 17 new tests; LABEL-01..05 closed.

## What Was Built

### Plan 19-A: inode constants + `labels.rs` module + FUSE dispatch

**Task A-1: inode constants + pure module**

- Added three inode constants to `inode.rs`:
  - `LABELS_ROOT_INO = 0x7_FFFF_FFFF`
  - `LABELS_DIR_INO_BASE = 0x10_0000_0000`
  - `LABELS_SYMLINK_INO_BASE = 0x14_0000_0000`
- Extended `fixed_inodes_are_disjoint_from_dynamic_ranges` test with 4 label-range assertions
- Created `crates/reposix-fuse/src/labels.rs` (pure module — no fuser, no async):
  - `LabelEntry`: `symlink_ino`, `slug` (deduped `.md` filename), `target` (relative path)
  - `LabelSnapshot`: `label_dirs` map, `symlink_targets` reverse map, `label_count`
  - `LabelSnapshot::build(bucket, issues)`: groups by label, sorts, dedupes dir slugs via
    `dedupe_siblings`, allocates sequential dir inodes, dedupes entry slugs per group
  - 9 unit tests covering empty, single-label, multi-label, target format, slug sanitization,
    constant ordering, attr size, deduplication, and stable sort

**Task A-2: FUSE dispatch wiring**

- `GITIGNORE_BYTES` updated from `b"/tree/\n"` to `b"/tree/\nlabels/\n"` (15 bytes)
- Three new `InodeKind` variants: `LabelsRoot`, `LabelDir`, `LabelSymlink`
- `classify()` updated with label arms **before** `TREE_SYMLINK_INO_BASE` catch-all
  (label ranges 0x10/0x14 are numerically above tree-symlink range 0xC — ordering is critical)
- `ReposixFs` struct: `label_snapshot: Arc<RwLock<LabelSnapshot>>` + `labels_attr: FileAttr`
- `refresh_issues`: rebuilds `LabelSnapshot` unconditionally after tree snapshot
- `render_mount_root_index`: new `label_count` parameter; `labels/` row always emitted
- FUSE callbacks wired: `getattr`, `lookup`, `readdir`, `read` (EISDIR/EINVAL), `readlink`,
  `setattr`/`write` (EROFS)
- 8 new behavior-block tests in `fs.rs`; `readdir.rs` integration test updated

### Plan 19-B: green gauntlet + CHANGELOG + STATE

- Full workspace green-gauntlet: `cargo fmt --check`, `cargo clippy -D warnings`,
  `cargo test --workspace --quiet`, `cargo check --workspace` — all EXIT 0
- CHANGELOG.md: Added `### Added — Phase 19` section under `[Unreleased]`
- ROADMAP.md: Phase 19 plans marked `[x]`, plan count updated to `2/2`
- STATE.md: Cursor advanced to Phase 20

## Deferred: `spaces/`

`spaces/` directory view was in the original OP-1 scope but is deferred. Reason: implementing
`mount/spaces/<space>/` requires a new `IssueBackend::list_spaces()` trait method — this is
an API-breaking change affecting all backends (sim, GitHub, Confluence). Additionally,
`spaces/` is meaningful only for Confluence (GitHub has no spaces concept). The design work
belongs in a dedicated phase alongside other `IssueBackend` trait extensions.

See `19-RESEARCH.md §Scope Recommendation` for the full rationale.

## Requirements Closed

| ID | Description | Status |
|----|-------------|--------|
| LABEL-01 | `labels/` directory exists at mount root | Closed |
| LABEL-02 | `labels/<label>/` lists issues carrying that label as symlinks | Closed |
| LABEL-03 | Symlinks point to canonical bucket file (`../../<bucket>/<padded-id>.md`) | Closed |
| LABEL-04 | All write operations on label paths return EROFS | Closed |
| LABEL-05 | `_INDEX.md` and `.gitignore` updated for `labels/` | Closed |

## Decisions Made

1. **Sequential dir inode allocation**: `LABELS_DIR_INO_BASE + offset` (sort-order index) rather
   than a hash. Deterministic across mounts, no collision risk, trivially verified.

2. **Unconditional label snapshot rebuild**: `LabelSnapshot::build` called every `refresh_issues`
   regardless of whether issues have labels. Mirrors tree snapshot discipline; avoids stale-data
   bugs after relabeling (T-19-05).

3. **`labels/` row unconditional in index**: Even when `label_count == 0`, the row appears.
   The directory always exists at mount root; count being 0 is informative.

4. **`labels/` in `.gitignore`**: Added alongside `/tree/\n` so git working-tree operations
   on the mount don't track the synthesized overlay as modified files.

5. **Entry slugs include `.md` suffix**: `format!("{entry_slug}.md")` — consistent with bucket
   filenames and `tree/` symlink targets.

6. **`spaces/` deferred**: Requires `IssueBackend::list_spaces()` (API-breaking) and is
   Confluence-only in practice. Scope to a future phase.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing] Integration test updated for new root listing**
- **Found during:** Task A-2 verification
- **Issue:** `readdir.rs` asserted exact root listing `[.gitignore, _INDEX.md, issues]`; adding
  `labels/` caused failure
- **Fix:** Updated expected vec to include `"labels"`, updated gitignore byte count comment
- **Files modified:** `crates/reposix-fuse/tests/readdir.rs`
- **Commit:** 9e5386c

**2. [Rule 1 - Bug] Byte count comment was wrong (16 vs 15)**
- **Found during:** Task A-2 test run
- **Issue:** Plan comment said "(14 bytes)" for `b"/tree/\nlabels/\n"` — actual count is 15
- **Fix:** Corrected to 15 in test assertion and comments throughout
- **Files modified:** `crates/reposix-fuse/src/fs.rs`
- **Commit:** 9e5386c

**3. [Rule 2 - Missing] `#[allow(clippy::too_many_lines)]` on `lookup`**
- **Found during:** Task A-2 clippy run
- **Issue:** Adding `LabelsRoot` and `LabelDir` arms pushed `lookup` over the 100-line limit
- **Fix:** Added allow matching the pattern already used on `readdir`
- **Files modified:** `crates/reposix-fuse/src/fs.rs`
- **Commit:** 9e5386c

## Known Stubs

None. All `labels/` data flows from `Issue::labels` (populated by sim and GitHub adapter).
Confluence adapter returns `labels: vec![]` by design — overlay is present but empty for
Confluence mounts, which is correct behavior documented in CHANGELOG.

## Threat Flags

No new trust boundaries introduced. All four threats mitigated:
- T-19-01: `slug_or_fallback` applied to every label string before FUSE dir name use
- T-19-02: Symlink targets built from controlled components only (no label content)
- T-19-03: O(n·labels) rebuild accepted (no user-visible latency impact)
- T-19-04: EROFS returned for all write callbacks on label paths

## Commits

| Plan | Hash | Message |
|------|------|---------|
| 19-A | c14dbb0 | feat(19-A-task1): add LABELS_* inode constants + LabelSnapshot module |
| 19-A | 9e5386c | feat(19-A): implement labels/ read-only symlink overlay in FUSE |
| 19-B | 9c5ff09 | docs(19-B-task1): workspace green-gauntlet + CHANGELOG Phase 19 entries |

## Self-Check: PASSED

Files exist:
- `crates/reposix-fuse/src/labels.rs` — FOUND (created in 19-A)
- `crates/reposix-fuse/src/inode.rs` — FOUND (LABELS_* constants added)
- `crates/reposix-fuse/src/fs.rs` — FOUND (label dispatch wired)
- `CHANGELOG.md` — FOUND (Phase 19 section added)
- `.planning/ROADMAP.md` — FOUND (Phase 19 marked [x])
- `.planning/STATE.md` — FOUND (cursor advanced to Phase 20)

Commits verified in git log:
- c14dbb0, 9e5386c, 9c5ff09 — all present on main
