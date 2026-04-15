---
phase: 18-op-2-remainder-tree-recursive-and-mount-root-index-md-synthe
plan: "02"
subsystem: reposix-fuse
tags: [fuse, index, synthesis, op-2, inode, tree, mount-root, dfs]

requires:
  - phase: 15-dynamic-index-md-synthesized-in-fuse-bucket-directory-op-2-p
    provides: BUCKET_INDEX_INO=5, render_bucket_index, BucketIndex inode kind — baseline for Phase 18 constants and dispatch pattern
provides:
  - ROOT_INDEX_INO=6 (fixed inode for mount/_INDEX.md)
  - TREE_INDEX_ALLOC_START=7 / TREE_INDEX_ALLOC_END=0xFFFF (dynamic AtomicU64 range)
  - render_tree_index — DFS subtree sitemap (YAML frontmatter + pipe-table depth|name|target)
  - render_mount_root_index — whole-mount overview (YAML frontmatter + pipe-table entry|kind|count)
  - synthetic_file_attr — generalised FileAttr builder for any read-only synthetic file
  - Full FUSE dispatch for RootIndex + TreeDirIndex (getattr/lookup/readdir/read/setattr/write)
  - scripts/dev/test-tree-index.sh — mount-root _INDEX.md live-mount smoke script
affects: [phase-19-op-1-remainder, phase-20-op-3-cache-refresh]

tech-stack:
  added: []
  patterns:
    - AtomicU64-allocator (fetch_add with SeqCst, saturate at upper bound)
    - DashMap-lazy-cache (per-tree-dir render cache keyed by dir_ino)
    - DFS-stack-traversal (reverse-push children; cycle-free TreeSnapshot needs no visited set)
    - YAML-frontmatter-pipe-table (same schema as bucket index; kind key distinguishes levels)
    - synthetic_file_attr generalises bucket_index_attr by taking ino as parameter

key-files:
  created:
    - scripts/dev/test-tree-index.sh
    - .planning/phases/18-op-2-remainder-tree-recursive-and-mount-root-index-md-synthe/18-SUMMARY.md
  modified:
    - crates/reposix-fuse/src/inode.rs
    - crates/reposix-fuse/src/fs.rs
    - crates/reposix-fuse/tests/readdir.rs
    - CHANGELOG.md

key-decisions:
  - "Stack-based DFS for render_tree_index (no visited set needed; TreeSnapshot is cycle-free)"
  - "Option B inode strategy: AtomicU64 + DashMap, separate from InodeRegistry — avoids contention with issue inode allocation"
  - "synthetic_file_attr generalises bucket_index_attr with ino parameter for RootIndex and TreeDirIndex"
  - "tree/ shows — for count in mount-root index (tree has no flat issue count)"
  - "tree-dir _INDEX.md live-mount test skipped in smoke script: reposix-sim always returns parent_id=null; tree/ overlay requires Confluence backend"
  - "readdir.rs integration test updated: mount root listing now includes _INDEX.md (Phase 18 correctly adds ROOT_INDEX_INO)"

patterns-established:
  - "Synthetic file pattern: fixed or dynamically-allocated ino + lazy RwLock/DashMap cache + synthetic_file_attr + EROFS on setattr/write"
  - "InodeKind classify() extension: add range arm before fallthrough; use (START..=END).contains(&n)"

requirements-completed:
  - INDEX-01
  - INDEX-02

duration: 15min
completed: "2026-04-15"
---

# Phase 18: OP-2 Remainder — Tree-Recursive and Mount-Root `_INDEX.md` Summary

**DFS tree-index and mount-root overview synthesis via AtomicU64 inode allocator, DashMap render caches, and full FUSE dispatch — closes OP-2 across all three mount levels (bucket Phase 15, tree-subdir + mount-root Phase 18).**

## Performance

- **Duration:** ~15 min (Wave 1: 5 min, Wave 2: 10 min)
- **Started:** 2026-04-15T07:10:00Z
- **Completed:** 2026-04-15T07:30:00Z
- **Tasks:** 4 (2 from Wave 1, 2 from Wave 2)
- **Files modified:** 5 (inode.rs, fs.rs, readdir.rs, CHANGELOG.md, test-tree-index.sh)

## Accomplishments

- `mount/_INDEX.md` (inode 6 = `ROOT_INDEX_INO`): whole-mount overview with YAML frontmatter (`kind: mount-index`) + pipe-table listing `.gitignore`, `<bucket>/` (with issue count), and `tree/` when hierarchy is active
- `mount/tree/<subdir>/_INDEX.md` (inodes 7..=0xFFFF, dynamically allocated): recursive subtree sitemap via DFS from `TreeSnapshot`; YAML frontmatter (`kind: tree-index`) + pipe-table with `depth | name | target` columns; all descendants listed
- 6 new unit tests in `fs.rs`: frontmatter keys, DFS ordering, empty dir edge case, tree/ row gating, inode idempotency and reverse-map population
- `cargo fmt --all` applied to workspace (formatting cleanup across fs.rs, main.rs, test files)
- CHANGELOG.md `[Unreleased]` section updated with both Phase 18 feature entries

## Task Commits

1. **Task 1: Constants + classify arms** — `dfb9018` (feat(18-A): reserve ROOT_INDEX_INO + TREE_INDEX_ALLOC range + classify arms + clippy)
2. **Task 2: Full FUSE dispatch + tests** — `4002a4f` (feat(18-A-task2): render_tree_index + render_mount_root_index + full FUSE dispatch + 6 unit tests)
3. **Task 3 (Wave B): Green-gauntlet + CHANGELOG** — `cd5493b` (docs(18-B-task1): workspace green-gauntlet pass + CHANGELOG [Unreleased] Phase 18 entries)
4. **Task 4 (Wave B): Smoke script + SUMMARY** — (this commit — docs(18-B))

## Files Created/Modified

- `crates/reposix-fuse/src/inode.rs` — `ROOT_INDEX_INO=6`, `TREE_INDEX_ALLOC_START=7`, `TREE_INDEX_ALLOC_END=0xFFFF`; layout doc updated; two tests tightened
- `crates/reposix-fuse/src/fs.rs` — `InodeKind::RootIndex` + `InodeKind::TreeDirIndex`; `render_tree_index`, `render_mount_root_index`, `synthetic_file_attr`, `alloc_tree_index_ino`, `tree_dir_index_ino`, `tree_dir_index_bytes_or_render`, `mount_root_index_bytes_or_render`; full FUSE dispatch updated; 6 new unit tests
- `crates/reposix-fuse/tests/readdir.rs` — expected root listing updated to include `_INDEX.md`
- `CHANGELOG.md` — Phase 18 entries under `[Unreleased]`; Phase 17 entry preserved
- `scripts/dev/test-tree-index.sh` — mount-root `_INDEX.md` live-mount smoke script (sim backend); tree-dir section documents sim limitation

## Decisions Made

- **DFS strategy:** Stack-based DFS (reverse-push children) rather than recursive DFS — avoids stack overflow on deep trees; `TreeSnapshot` is cycle-free so no visited set is needed.
- **Inode allocation strategy (Option B):** `AtomicU64` counter + `DashMap` forward/reverse maps, separate from `InodeRegistry` — avoids lock contention with the hot issue-allocation path. Range `7..=0xFFFF` gives 65,528 tree-dir indexes (more than any realistic mount).
- **`synthetic_file_attr` generalisation:** Factored out from `bucket_index_attr` to accept `ino` as a parameter, shared by `ROOT_INDEX_INO` and `TreeDirIndex` paths. Reduces code duplication and ensures consistent perm/nlink/size behavior across all synthetic files.
- **`tree/` count in mount-root index:** Shows `—` (em dash) rather than a numeric count — the tree overlay has no flat issue count (it's a hierarchy, not a flat list).
- **Smoke script tree-dir section:** `reposix-sim` always returns `parent_id=null`; `tree/` never populates with a sim backend. Live-mount tree-dir `_INDEX.md` test is deferred to a future Confluence-backend smoke script. The render path is covered by 6 unit tests in `fs.rs`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Integration test `readdir.rs` asserting stale mount root listing**
- **Found during:** Wave 1, Task 2 (`cargo test`)
- **Issue:** `mount_lists_and_reads_issues` integration test asserted mount root listing was `[".gitignore", "issues"]` but Phase 18 correctly adds `_INDEX.md` to the root `readdir` output.
- **Fix:** Updated expected list to `[".gitignore", "_INDEX.md", "issues"]` with updated comment.
- **Files modified:** `crates/reposix-fuse/tests/readdir.rs`
- **Committed in:** `4002a4f`

**2. [Rule 1 - Clippy] `manual_range_contains` warning in `classify`**
- **Found during:** Wave 1, Task 1 clippy run
- **Issue:** `n >= TREE_INDEX_ALLOC_START && n <= TREE_INDEX_ALLOC_END` triggered `clippy::manual_range_contains`.
- **Fix:** Rewrote as `(TREE_INDEX_ALLOC_START..=TREE_INDEX_ALLOC_END).contains(&n)`.
- **Committed in:** `dfb9018`

**3. [Rule 2 - Missing allow] `clippy::too_many_lines` on `readdir`**
- **Found during:** Wave 1, Task 1 clippy run
- **Issue:** Adding new arms pushed `readdir` past 100 lines.
- **Fix:** Added `#[allow(clippy::too_many_lines, reason = "...")]` with rationale.
- **Committed in:** `dfb9018`

**4. [Rule 1 - Test] Unused helper + unused imports in test module**
- **Found during:** Wave 1, Task 2 compilation
- **Fix:** Removed unused `mk_tree_dir_and_snap` helper; removed three unused imports; replaced wildcard match pattern with explicit variant for `clippy::match_wildcard_for_single_variants`.
- **Committed in:** `4002a4f`

**5. [Rule 1 - Fmt] Workspace formatting drift**
- **Found during:** Wave 2, Task 1 `cargo fmt --all --check`
- **Issue:** Several files in `reposix-fuse/src/fs.rs`, `reposix-swarm/src/main.rs`, and swarm tests had formatting that diverged from `rustfmt` style after Phase 17 and Wave 1 commits.
- **Fix:** `cargo fmt --all` applied.
- **Files modified:** `crates/reposix-fuse/src/fs.rs`, `crates/reposix-swarm/src/main.rs`, `crates/reposix-swarm/tests/confluence_real_tenant.rs`, `crates/reposix-swarm/tests/mini_e2e.rs`
- **Committed in:** `cd5493b`

**6. [Rule 1 - Deviation] Smoke script tree-dir section: sim does not support `parent_id`**
- **Found during:** Wave 2, Task 2 script authoring
- **Issue:** Plan called for POSTing two issues with `parent_id` to the simulator to populate `tree/`. The `reposix-sim` backend always returns `parent_id: null`; neither the seed loader nor the REST `CreateIssueBody` carries `parent_id`. The `tree/` overlay is never populated with a sim backend, making the tree-dir `_INDEX.md` assertion impossible in a live sim-backed mount.
- **Fix:** Script tests mount-root `_INDEX.md` fully (assertions on `kind: mount-index`, `backend`, `project`, table header, EROFS on write). Tree-dir section documents the limitation and points to the 6 unit tests that cover `kind: tree-index` rendering end-to-end.
- **Impact:** No loss of correctness coverage — unit tests verify the render path; the live-mount smoke fills the integration gap at the mount-root level.

---

**Total deviations:** 6 auto-fixed (4 Rule 1 bugs/clippy/test, 1 Rule 2 missing-allow, 1 Rule 1 script deviation)
**Impact on plan:** All auto-fixes necessary for correctness or build cleanliness. Script deviation avoids a false test (sim cannot exercise tree/). No scope creep.

## Known Stubs

None — all dispatch arms are fully implemented. Tree-dir live-mount test is documented as requiring Confluence backend, not a stub.

## Threat Flags

None — no new network endpoints or auth paths. Attacker-influenced fields (`name`, `target`) pass through `escape_index_cell` (T-18-01 mitigation, Wave 1).

## Next Phase Readiness

OP-2 is now **fully closed**:
- Phase 15: `mount/<bucket>/_INDEX.md` (bucket level, inode 5)
- Phase 18: `mount/tree/<subdir>/_INDEX.md` (tree-dir level, inodes 7..=0xFFFF) + `mount/_INDEX.md` (mount-root level, inode 6)

Agents can now `cat` any level of the mount hierarchy to get a machine-readable sitemap.

Next recommended target: **Phase 19** (OP-1 remainder — complete the `git push` remote helper round-trip).

---
*Phase: 18-op-2-remainder-tree-recursive-and-mount-root-index-md-synthe*
*Completed: 2026-04-15*
