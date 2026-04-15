---
phase: 18
plan: "01"
subsystem: reposix-fuse
tags: [fuse, index, synthesis, op-2, inode]
dependency_graph:
  requires: [phase-15-bucket-index]
  provides: [ROOT_INDEX_INO, TREE_INDEX_ALLOC_START, render_tree_index, render_mount_root_index]
  affects: [crates/reposix-fuse/src/inode.rs, crates/reposix-fuse/src/fs.rs, crates/reposix-fuse/tests/readdir.rs]
tech_stack:
  added: []
  patterns: [DashMap-lazy-cache, AtomicU64-allocator, DFS-stack-traversal, YAML-frontmatter-pipe-table]
key_files:
  created: []
  modified:
    - crates/reposix-fuse/src/inode.rs
    - crates/reposix-fuse/src/fs.rs
    - crates/reposix-fuse/tests/readdir.rs
decisions:
  - "Used stack-based DFS (reverse-push children) rather than recursive DFS to avoid stack overflow on deep trees; TreeSnapshot is cycle-free so no visited set needed"
  - "synthetic_file_attr generalises bucket_index_attr by taking ino as parameter, sharing the shape between ROOT_INDEX_INO and TreeDirIndex inodes"
  - "tree_dir_index_ino_is_stable test builds a real ReposixFs via MockServer rather than mocking DashMap internals — validates the full allocator path"
  - "readdir.rs integration test updated to include _INDEX.md in expected root listing (Rule 1 fix: test was asserting old behavior)"
  - "escape_index_cell applied to both name and target in render_tree_index rows (T-18-01 mitigation)"
metrics:
  duration_minutes: 5
  completed_date: "2026-04-15"
  tasks_completed: 2
  files_modified: 3
---

# Phase 18 Plan 01: OP-2 Remainder — Tree-Recursive and Mount-Root `_INDEX.md` Synthesis

**One-liner:** DFS tree-index and mount-root overview synthesis using AtomicU64 lazy inode allocator, DashMap caches, and full FUSE dispatch for `RootIndex`/`TreeDirIndex` kinds.

## What Was Built

### New Constants (`inode.rs`)
- `ROOT_INDEX_INO = 6` — fixed inode for `mount/_INDEX.md`
- `TREE_INDEX_ALLOC_START = 7` — start of dynamic per-tree-dir index inodes
- `TREE_INDEX_ALLOC_END = 0xFFFF` — inclusive upper bound (65,528 possible tree-dir indexes)
- Layout doc table updated: row `6..=0xFFFF` split into `6` (ROOT_INDEX_INO) and `7..=0xFFFF` (dynamic AtomicU64)
- `reserved_range_is_unmapped` test updated: loop now `TREE_INDEX_ALLOC_START..=TREE_INDEX_ALLOC_END`
- `fixed_inodes_are_disjoint_from_dynamic_ranges` test updated: `ROOT_INDEX_INO` added to `fixed` array

### New Enum Variants (`fs.rs`)
- `InodeKind::RootIndex` — classifies inode 6
- `InodeKind::TreeDirIndex` — classifies inodes 7..=0xFFFF

### New Render Functions (`fs.rs`)
- `render_tree_index(root_dir, snapshot, project, generated_at) -> Vec<u8>` — stack-based DFS, YAML frontmatter (`kind: tree-index`, `project`, `subtree`, `entry_count`, `generated_at`) + pipe-table (`depth | name | target`)
- `render_mount_root_index(backend_name, project, bucket, issue_count, tree_present, generated_at) -> Vec<u8>` — YAML frontmatter (`kind: mount-index`, `backend`, `project`, `bucket`, `issue_count`, `generated_at`) + pipe-table (`entry | kind | count`), `tree/` row gated on `tree_present`

### New Struct Fields (`ReposixFs`)
- `mount_root_index_bytes: RwLock<Option<Arc<Vec<u8>>>>` — lazy cache for `mount/_INDEX.md`
- `tree_dir_index_cache: DashMap<u64, Arc<Vec<u8>>>` — per-tree-dir render cache keyed by dir_ino
- `tree_index_inodes: DashMap<u64, u64>` — forward map: dir_ino → index_ino
- `tree_index_ino_reverse: DashMap<u64, u64>` — reverse map: index_ino → dir_ino (needed for Pitfall 2 getattr)
- `tree_index_alloc: AtomicU64` — starts at `TREE_INDEX_ALLOC_START`, capped at `TREE_INDEX_ALLOC_END`

### New Helper Methods (`ReposixFs`)
- `alloc_tree_index_ino()` — fetch_add with SeqCst, saturating at `TREE_INDEX_ALLOC_END`
- `tree_dir_index_ino(dir_ino)` — idempotent allocator using `dashmap::Entry`, populates both maps
- `tree_dir_index_bytes_or_render(dir_ino, snap)` — DashMap cache with lazy render on miss
- `mount_root_index_bytes_or_render()` — RwLock cache with lazy render on miss
- `synthetic_file_attr(ino, size)` — generalised `FileAttr` for any read-only synthetic file (perm: 0o444, RegularFile)

### FUSE Dispatch (`fs.rs` `Filesystem` impl)
All five callbacks updated:
- `getattr`: `RootIndex` → render + synthetic_file_attr; `TreeDirIndex` → reverse-lookup + render (handles Pitfall 2: kernel calls getattr before lookup)
- `lookup(Root)`: `"_INDEX.md"` → ROOT_INDEX_INO attr
- `lookup(TreeDir)`: `"_INDEX.md"` → allocate index_ino + render attr (before `reply_tree_entry`)
- `readdir(Root)`: adds `(ROOT_INDEX_INO, RegularFile, "_INDEX.md")` entry
- `readdir(TreeDir)`: adds `(index_ino, RegularFile, "_INDEX.md")` entry after `.` and `..`
- `read`: `RootIndex` + `TreeDirIndex` → serve rendered bytes with offset/size slicing
- `setattr`/`write`: `RootIndex` + `TreeDirIndex` → EROFS (T-18-02, T-18-03)
- `refresh_issues`: invalidates `mount_root_index_bytes` and clears `tree_dir_index_cache`

### Unit Tests (6 new, in `fs.rs`)
1. `render_tree_index_frontmatter_and_table` — 2-symlink dir; asserts all frontmatter keys + 2 data rows
2. `tree_index_full_dfs` — parent→child→grandchild issue hierarchy; asserts DFS visits ≥3 entries
3. `tree_index_empty` — empty dir; asserts `entry_count: 0`, no data rows
4. `render_mount_root_index_frontmatter_and_table` — `tree_present=true`; asserts all keys + 3 table rows
5. `mount_root_index_no_tree_row` — `tree_present=false`; asserts `tree/` row absent
6. `tree_dir_index_ino_is_stable` — calls `tree_dir_index_ino(42)` twice; asserts idempotency + distinct inodes for distinct dir_inos + reverse map populated

**Test count delta:** 40 → 46 in `reposix-fuse` unit tests (+6).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Integration test `readdir.rs` asserting stale mount root listing**
- **Found during:** Task 2 (`cargo test`)
- **Issue:** `mount_lists_and_reads_issues` integration test asserted mount root listing was `[".gitignore", "issues"]` but Phase 18 correctly adds `_INDEX.md` to the root `readdir` output.
- **Fix:** Updated expected list to `[".gitignore", "_INDEX.md", "issues"]` with updated comment.
- **Files modified:** `crates/reposix-fuse/tests/readdir.rs`
- **Commit:** 4002a4f

**2. [Rule 1 - Clippy] `manual_range_contains` warning in `classify`**
- **Found during:** Task 1 clippy run
- **Issue:** `n >= TREE_INDEX_ALLOC_START && n <= TREE_INDEX_ALLOC_END` triggered `clippy::manual_range_contains`.
- **Fix:** Rewrote as `(TREE_INDEX_ALLOC_START..=TREE_INDEX_ALLOC_END).contains(&n)`.
- **Commit:** dfb9018

**3. [Rule 2 - Missing allow] `clippy::too_many_lines` on `readdir`**
- **Found during:** Task 1 clippy run
- **Issue:** Adding stubs pushed `readdir` from 100 to 101 lines. Task 2 will add more arms.
- **Fix:** Added `#[allow(clippy::too_many_lines, reason = "...")]` with rationale. No functional change.
- **Commit:** dfb9018

**4. [Rule 1 - Test] Unused `mk_tree_dir_and_snap` helper and unused imports in test**
- **Found during:** Task 2 compilation
- **Fix:** Removed unused helper function; removed `TreeDir`, `TREE_DIR_INO_BASE`, `TREE_SYMLINK_INO_BASE` from unused import list in `tree_index_full_dfs`. Replaced wildcard pattern in test match with explicit variant to satisfy `clippy::match_wildcard_for_single_variants`.
- **Commit:** 4002a4f

## Known Stubs

None — all dispatch arms are fully implemented. The plan's objectives are completely satisfied.

## Threat Flags

None — no new network endpoints or auth paths introduced. All attacker-influenced fields (`name`, `target`) pass through `escape_index_cell` as required by T-18-01.

## Self-Check: PASSED

- `ROOT_INDEX_INO`, `TREE_INDEX_ALLOC_START`, `TREE_INDEX_ALLOC_END` exported from `inode.rs`: confirmed
- `render_tree_index`, `render_mount_root_index` defined in `fs.rs`: confirmed
- All 6 new tests pass: confirmed (46 total in reposix-fuse)
- Workspace gate: all 32 test suites pass, clippy clean
- Commits: dfb9018 (Task 1), 4002a4f (Task 2)
