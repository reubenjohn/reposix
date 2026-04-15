---
phase: 18-op-2-remainder-tree-recursive-and-mount-root-index-md-synthe
reviewed: 2026-04-15T00:00:00Z
depth: standard
files_reviewed: 2
files_reviewed_list:
  - crates/reposix-fuse/src/inode.rs
  - crates/reposix-fuse/src/fs.rs
findings:
  critical: 0
  warning: 1
  info: 2
  total: 3
status: issues_found
---

# Phase 18: Code Review Report

**Reviewed:** 2026-04-15
**Depth:** standard
**Files Reviewed:** 2
**Status:** issues_found

## Summary

Phase 18 adds tree-recursive `_INDEX.md` synthesis per tree-dir and a mount-root `_INDEX.md` overview file. The implementation is well-structured: pure render functions, lazy-invalidated caches following the Phase 15 pattern, EROFS enforcement on all write callbacks for the new synthetic inodes, and `escape_index_cell` correctly applied to all tainted strings (issue titles, tree dir names, symlink targets) in both the table cells and YAML frontmatter.

All 46 unit tests pass and `cargo clippy` is clean.

Three issues were found: one warning-level correctness issue with the DFS traversal ordering in `render_tree_index`, and two info-level issues.

## Warnings

### WR-01: DFS traversal produces inverted sibling order — `_self.md` appears after child dir at same depth

**File:** `crates/reposix-fuse/src/fs.rs:385-403`

**Issue:** The stack-based DFS iterates `entries.iter().rev()` and immediately appends symlinks to `rows` as they are encountered, while Dir entries get a row appended and then push their children to the stack. Because `iter().rev()` processes children right-to-left, a Dir entry that precedes a symlink sibling in storage order gets appended to `rows` BEFORE that symlink, even though the symlink comes first in the original children list.

Concrete example with `parent_dir.children = [Symlink(_self.md), Dir(child_ino)]` (which is the actual ordering — tree.rs guarantees `_self.md` is always first):

```
iter().rev() processes: Dir(child) first, then Symlink(_self.md)
→ rows: [(depth=0, "child/"), (depth=0, "_self.md"), ...child's contents...]
```

Expected pre-order DFS:
```
→ rows: [(depth=0, "_self.md"), (depth=0, "child/"), ...child's contents...]
```

The doc comment at line 369–374 claims "Performs a DFS from `root_dir`... rows in DFS traversal order" and the inline comment at line 385 says "Push children in reverse order so left-to-right children pop correctly." The second comment is partially true for Dir-to-stack pushes, but it does not account for the immediate-push of sibling symlinks happening in reversed order.

The unit test `tree_index_full_dfs` only asserts `data_rows.len() >= 3` and that the text contains `"child/"` or `"_self.md"` — it does not verify ordering, so this bug is not caught.

**Impact:** The generated `_INDEX.md` shows the parent directory's `_self.md` entry *after* child directory entries at the same depth level, which inverts the expected reading order (a user would expect `_self.md` — the page itself — before the subdirectory listing). This is a documentation-quality bug; the file is still parseable and complete.

**Fix:** Either (a) iterate `entries` forward (not reversed) and push Dirs to the stack in reversed order before processing the slice, or (b) collect all entries for a level into `rows` in forward order before pushing any child stacks. The simplest correct approach:

```rust
while let Some((entries, depth)) = stack.pop() {
    // Process in forward order; push Dir children to stack in reverse so they
    // pop in forward order.
    for entry in entries {
        match entry {
            crate::tree::TreeEntry::Symlink { name, target, .. } => {
                rows.push((depth, name.clone(), target.clone()));
            }
            crate::tree::TreeEntry::Dir(ino) => {
                if let Some(dir) = snapshot.resolve_dir(*ino) {
                    rows.push((depth, format!("{}/", dir.name), String::new()));
                    // Reverse so first child pops first.
                    let mut children = dir.children.as_slice();
                    // Push a reversed-iterator wrapper or collect+reverse:
                    stack.push((dir.children.as_slice(), depth + 1));
                    // But only after reversing the child slice representation — or
                    // switch to pushing individual entries in reverse at pop time.
                }
            }
        }
    }
}
```

The cleanest fix is to iterate forward and push each Dir's children to the stack reversed:

```rust
while let Some((entries, depth)) = stack.pop() {
    for entry in entries {   // forward — preserves storage order
        match entry {
            crate::tree::TreeEntry::Symlink { name, target, .. } => {
                rows.push((depth, name.clone(), target.clone()));
            }
            crate::tree::TreeEntry::Dir(ino) => {
                if let Some(dir) = snapshot.resolve_dir(*ino) {
                    rows.push((depth, format!("{}/", dir.name), String::new()));
                    // Push reversed so first child pops (and is appended) first.
                    let rev_children: Vec<_> = dir.children.iter().rev().cloned().collect();
                    // ... push rev_children to stack
                }
            }
        }
    }
}
```

Note: this also requires updating the `tree_index_full_dfs` test to assert ordering explicitly once the fix is in.

## Info

### IN-01: Inode exhaustion at `TREE_INDEX_ALLOC_END` is silent — no `warn!()` at saturation

**File:** `crates/reposix-fuse/src/fs.rs:966-968`

**Issue:** `alloc_tree_index_ino` saturates at `TREE_INDEX_ALLOC_END` (0xFFFF = 65535) silently. When the allocator overflows (the 65529th unique tree-dir requests an `_INDEX.md` inode), `fetch_add` returns a value above 0xFFFF, the `.min(TREE_INDEX_ALLOC_END)` caps it at 0xFFFF, and `tree_dir_index_ino` inserts the capped inode into both the forward and reverse maps. This silently overwrites the reverse map entry for inode 0xFFFF with the newest dir_ino, so subsequent `getattr` or `read` on that inode serve the wrong directory's index content for all previously registered dirs that also saturated to 0xFFFF.

The docstring says "more than enough for any real Confluence space," which is correct in practice (65528 distinct tree dirs is implausible). However, the silent corruption is worth a tracing log at the saturation boundary.

**Fix:** Add a one-time `tracing::warn!` when saturation is detected:

```rust
fn alloc_tree_index_ino(&self) -> u64 {
    let ino = self.tree_index_alloc.fetch_add(1, Ordering::SeqCst);
    if ino > TREE_INDEX_ALLOC_END {
        tracing::warn!(
            ino,
            max = TREE_INDEX_ALLOC_END,
            "tree-dir _INDEX.md inode space exhausted; new dirs will share inode {}",
            TREE_INDEX_ALLOC_END
        );
    }
    ino.min(TREE_INDEX_ALLOC_END)
}
```

### IN-02: `tree_index_full_dfs` test assertions are too weak to catch traversal order regressions

**File:** `crates/reposix-fuse/src/fs.rs:2300-2372` (approximate — the `tree_index_full_dfs` test)

**Issue:** The test only asserts `data_rows.len() >= 3` and `text.contains("child/") || text.contains("_self.md")`. These assertions do not verify:
- That `_self.md` appears before child directory entries at the same depth.
- That depth values are assigned correctly (e.g., `_self.md` at depth 0, grandchild at depth 2).
- That DFS pre-order is actually produced.

This means the order regression described in WR-01 passes the test suite undetected.

**Fix:** Add explicit ordering assertions after the existing `data_rows.len() >= 3` check:

```rust
// _self.md for parent must be depth 0 and appear before child/.
let self_md_pos = data_rows.iter().position(|l| l.contains("_self.md") && l.contains("| 0 |"));
let child_dir_pos = data_rows.iter().position(|l| l.contains("child/") && l.contains("| 0 |"));
assert!(self_md_pos.is_some(), "_self.md row at depth 0 must be present");
assert!(child_dir_pos.is_some(), "child/ row at depth 0 must be present");
assert!(
    self_md_pos.unwrap() < child_dir_pos.unwrap(),
    "_self.md must appear before child/ in pre-order DFS"
);
// Grandchild must be at depth 1.
assert!(
    data_rows.iter().any(|l| l.contains("| 1 |")),
    "grandchild must have depth 1"
);
```

---

_Reviewed: 2026-04-15_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
