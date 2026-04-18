---
phase: 19-op-1-remainder-labels-and-spaces-directory-views-as-read-onl
fixed_at: 2026-04-15T09:30:00Z
review_path: .planning/phases/19-op-1-remainder-labels-and-spaces-directory-views-as-read-onl/REVIEW.md
iteration: 1
findings_in_scope: 2
fixed: 2
skipped: 0
status: all_fixed
---

# Phase 19: Code Review Fix Report

**Fixed at:** 2026-04-15T09:30:00Z
**Source review:** `.planning/phases/19-op-1-remainder-labels-and-spaces-directory-views-as-read-onl/REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 2 (WR-01, IN-01; IN-03 was included within WR-01 scope per instructions)
- Fixed: 2
- Skipped: 0

## Fixed Issues

### WR-01: inode.rs layout table describes label ranges as tree-symlink space

**Files modified:** `crates/reposix-fuse/src/inode.rs`, `crates/reposix-fuse/src/tree.rs`
**Commit:** f92b7a1
**Applied fix:**
- Split the final layout table row `0xC_0000_0000..u64::MAX` into three rows:
  - `0xC_0000_0000..0x10_0000_0000` — tree leaf symlinks and `_self.md` entries
  - `0x7_FFFF_FFFF` (fixed) — `labels/` overlay root (`LABELS_ROOT_INO`)
  - `0x10_0000_0000..0x14_0000_0000` — `labels/` per-label interior directories
  - `0x14_0000_0000..u64::MAX` — `labels/` leaf symlinks
- Added compile-time assertion in `tree.rs` const block: `assert!(TREE_SYMLINK_INO_BASE < crate::inode::LABELS_DIR_INO_BASE)`

### IN-01: GITIGNORE_INO doc comment still says "7 bytes" / `b"/tree/\n"`

**Files modified:** `crates/reposix-fuse/src/inode.rs`
**Commit:** f92b7a1 (same atomic commit as WR-01)
**Applied fix:** Updated `GITIGNORE_INO` doc comment from `b"/tree/\n"` (7 bytes) to `b"/tree/\nlabels/\n"` (15 bytes).

---

_Fixed: 2026-04-15T09:30:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
