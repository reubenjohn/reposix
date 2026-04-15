---
phase: 18-op-2-remainder-tree-recursive-and-mount-root-index-md-synthe
fixed_at: 2026-04-15T00:00:00Z
review_path: .planning/phases/18-op-2-remainder-tree-recursive-and-mount-root-index-md-synthe/REVIEW.md
iteration: 1
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 18: Code Review Fix Report

**Fixed at:** 2026-04-15
**Source review:** `.planning/phases/18-op-2-remainder-tree-recursive-and-mount-root-index-md-synthe/REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 3
- Fixed: 3
- Skipped: 0

## Fixed Issues

### WR-01: DFS traversal produces inverted sibling order

**Files modified:** `crates/reposix-fuse/src/fs.rs`
**Commit:** 7be2396
**Applied fix:** Rewrote `render_tree_index` DFS to use a per-entry stack (`Vec<(&TreeEntry, depth)>`) instead of a per-slice stack. Initial children and each Dir's children are pushed in reversed order so the first entry pops (and is appended to `rows`) first, producing correct pre-order left-to-right traversal. Removed the `iter().rev()` on the slice which was causing siblings to be processed in reverse order.

### IN-01: Inode exhaustion at `TREE_INDEX_ALLOC_END` is silent

**Files modified:** `crates/reposix-fuse/src/fs.rs`
**Commit:** 7be2396
**Applied fix:** Added `tracing::warn!` in `alloc_tree_index_ino` when `ino > TREE_INDEX_ALLOC_END`, logging the raw inode value and the cap value so exhaustion is observable in traces.

### IN-02: `tree_index_full_dfs` test assertions too weak

**Files modified:** `crates/reposix-fuse/src/fs.rs`
**Commit:** 7be2396
**Applied fix:** Extended `tree_index_full_dfs` test with four explicit ordering assertions: `_self.md` at depth 0 is present, `child/` at depth 0 is present, `_self.md` appears before `child/` (pre-order DFS), and at least one row at depth 1 (grandchild). All 46 unit tests pass and `cargo clippy` is clean.

---

_Fixed: 2026-04-15_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
