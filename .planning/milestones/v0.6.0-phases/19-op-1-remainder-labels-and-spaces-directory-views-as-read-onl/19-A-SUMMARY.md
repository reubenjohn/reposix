---
phase: "19"
plan_id: "19-A"
subsystem: reposix-fuse
tags: [fuse, labels, overlay, read-only, inode, symlink]
dependency_graph:
  requires: []
  provides: [labels-overlay-fuse]
  affects: [reposix-fuse/src/fs.rs, reposix-fuse/src/labels.rs, reposix-fuse/src/inode.rs]
tech_stack:
  added: []
  patterns: [LabelSnapshot mirrors TreeSnapshot, inode-range dispatch before catch-all]
key_files:
  created:
    - crates/reposix-fuse/src/labels.rs
  modified:
    - crates/reposix-fuse/src/inode.rs
    - crates/reposix-fuse/src/fs.rs
    - crates/reposix-fuse/src/lib.rs
    - crates/reposix-fuse/tests/readdir.rs
decisions:
  - "Used sequential LABELS_DIR_INO_BASE + offset for dir inodes (deterministic, no hash collision risk)"
  - "Label snapshot built unconditionally on refresh_issues regardless of hierarchy flag"
  - "labels/ row emitted unconditionally in render_mount_root_index even when label_count=0"
  - "GITIGNORE_BYTES updated to b\"/tree/\\nlabels/\\n\" (15 bytes)"
metrics:
  duration: ~25 minutes
  completed: 2026-04-15T08:05:37Z
  tasks_completed: 2
  files_changed: 5
---

# Phase 19 Plan A: labels/ Read-Only Symlink Overlay Summary

## One-liner

`labels/` read-only FUSE overlay built from `Issue::labels` via `LabelSnapshot` module with full `InodeKind` dispatch, `readdir`/`lookup`/`getattr`/`readlink`/write-EROFS arms, and 17 new tests.

## What Was Built

### Task A-1: inode constants + `labels.rs` pure module

- Added `LABELS_ROOT_INO` (`0x7_FFFF_FFFF`), `LABELS_DIR_INO_BASE` (`0x10_0000_0000`), `LABELS_SYMLINK_INO_BASE` (`0x14_0000_0000`) to `inode.rs`
- Extended `fixed_inodes_are_disjoint_from_dynamic_ranges` test with 4 label-range assertions
- Created `crates/reposix-fuse/src/labels.rs` — pure module (no fuser, no async):
  - `LabelEntry` struct: `symlink_ino`, `slug` (deduped `.md` filename), `target` (relative path)
  - `LabelSnapshot` struct: `label_dirs` map, `symlink_targets` reverse map, `label_count`
  - `LabelSnapshot::build(bucket, issues)`: groups by label → sorts → dedupes dir slugs via `dedupe_siblings` → allocates sequential dir inodes → dedupes entry slugs per group
  - 9 unit tests covering empty, single-label, multi-label, target format, slug sanitization, constant ordering, attr size, deduplication, and stable sort
- Added `pub mod labels` + re-exports to `lib.rs`

### Task A-2: FUSE dispatch wiring

- `GITIGNORE_BYTES` updated from `b"/tree/\n"` to `b"/tree/\nlabels/\n"` (15 bytes)
- Three new `InodeKind` variants: `LabelsRoot`, `LabelDir`, `LabelSymlink`
- `classify()` updated with label arms **before** `TREE_SYMLINK_INO_BASE` catch-all (critical ordering — label ranges 0x10/0x14 are above tree-symlink range 0xC)
- `ReposixFs` struct: `label_snapshot: Arc<RwLock<LabelSnapshot>>` + `labels_attr: FileAttr`
- `refresh_issues`: rebuilds `LabelSnapshot` unconditionally after tree snapshot (T-19-05)
- `render_mount_root_index`: new `label_count: usize` parameter; `labels/` row always emitted
- FUSE callbacks wired:
  - `getattr`: `LabelsRoot` → `labels_attr`; `LabelDir` → dir attr with dynamic ino; `LabelSymlink` → symlink attr with target len
  - `lookup`: Root arm handles `"labels"` name; `LabelsRoot` arm finds dir by slug; `LabelDir` arm finds symlink by name
  - `readdir`: Root emits `labels` entry; `LabelsRoot` lists per-label dirs; `LabelDir` lists symlinks
  - `read`: `LabelsRoot | LabelDir` → EISDIR; `LabelSymlink` → EINVAL (use readlink)
  - `readlink`: `LabelSymlink` → looks up `symlink_targets` map
  - `setattr` / `write`: `LabelsRoot | LabelDir | LabelSymlink` → EROFS (T-19-04)
- 8 new behavior-block tests in `fs.rs` (classify, GITIGNORE_BYTES, render_mount_root_index)
- Integration test `readdir.rs` updated: `labels` added to expected root listing, gitignore bytes updated

## Decisions Made

1. **Sequential dir inode allocation**: `LABELS_DIR_INO_BASE + offset` (sort-order index) rather than a hash. Deterministic across mounts, no collision risk, trivially verified.

2. **Unconditional label snapshot rebuild**: `LabelSnapshot::build` is called every `refresh_issues` regardless of whether any issues have labels. Mirrors tree snapshot discipline; avoids stale-data bugs after a relabel (T-19-05).

3. **`labels/` row unconditional in index**: Even when `label_count == 0`, the row appears. The directory always exists at the mount root; its count being 0 is informative, not a reason to hide it.

4. **`labels/` in `.gitignore`**: Added alongside `/tree/\n` so git working-tree operations on the mount don't track the synthesized overlay as modified files.

5. **Entry slugs include `.md` suffix**: `format!("{entry_slug}.md")` — consistent with bucket filenames and `tree/` symlink targets; agents using `cat labels/bug/0001.md` get the expected extension.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing] Integration test updated for new root listing**
- **Found during:** Task A-2 verification
- **Issue:** `readdir.rs` integration test asserted exact root listing `[.gitignore, _INDEX.md, issues]`; adding `labels/` made it fail
- **Fix:** Updated expected vec to include `"labels"` and updated gitignore byte count comment from 16 to 15
- **Files modified:** `crates/reposix-fuse/tests/readdir.rs`
- **Commit:** 9e5386c

**2. [Rule 1 - Bug] Byte count comment was wrong (16 vs 15)**
- **Found during:** Task A-2 test run
- **Issue:** Plan comment said "(14 bytes)" for `b"/tree/\nlabels/\n"` — actual count is 15 (7 + 8)
- **Fix:** Corrected to 15 in test assertion and comments throughout
- **Files modified:** `crates/reposix-fuse/src/fs.rs`
- **Commit:** 9e5386c

**3. [Rule 2 - Missing] `#[allow(clippy::too_many_lines)]` on `lookup`**
- **Found during:** Task A-2 clippy run
- **Issue:** Adding `LabelsRoot` and `LabelDir` arms pushed `lookup` over the 100-line clippy limit
- **Fix:** Added `#[allow(clippy::too_many_lines)]` matching the pattern already used on `readdir`
- **Files modified:** `crates/reposix-fuse/src/fs.rs`
- **Commit:** 9e5386c

## Known Stubs

None. All `labels/` data flows from `Issue::labels` (already populated by sim and GitHub adapter). Confluence adapter returns `labels: vec![]` by design (deferred to OP-9b scope) — the overlay is present but empty for Confluence mounts, which is correct behavior.

## Threat Flags

No new trust boundaries introduced beyond those modeled in the plan's `<threat_model>`. All four threats (T-19-01 through T-19-04) are mitigated:
- T-19-01: `slug_or_fallback` applied to every label string before use as FUSE dir name
- T-19-02: Symlink targets built from controlled components only (no label content)
- T-19-03: O(n·labels) rebuild accepted
- T-19-04: EROFS returned for all write callbacks on label paths

## Self-Check: PASSED

Files exist:
- `crates/reposix-fuse/src/labels.rs` — FOUND
- `crates/reposix-fuse/src/inode.rs` — FOUND (with LABELS_* constants)
- `crates/reposix-fuse/src/fs.rs` — FOUND (with label dispatch)
- `crates/reposix-fuse/src/lib.rs` — FOUND (with pub mod labels)

Commits:
- `c14dbb0` — feat(19-A-task1): add LABELS_* inode constants + LabelSnapshot module
- `9e5386c` — feat(19-A): implement labels/ read-only symlink overlay in FUSE

Tests: 63 passed, 0 failed
Clippy: clean
Workspace: clean
