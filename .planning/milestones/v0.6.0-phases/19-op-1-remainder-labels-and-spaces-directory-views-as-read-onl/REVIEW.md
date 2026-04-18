---
phase: 19-op-1-remainder-labels-and-spaces-directory-views-as-read-onl
reviewed: 2026-04-15T09:00:00Z
depth: standard
files_reviewed: 4
files_reviewed_list:
  - crates/reposix-fuse/src/inode.rs
  - crates/reposix-fuse/src/labels.rs
  - crates/reposix-fuse/src/fs.rs
  - crates/reposix-fuse/src/lib.rs
findings:
  critical: 0
  warning: 1
  info: 3
  total: 4
status: issues_found
---

# Phase 19: Code Review Report

**Reviewed:** 2026-04-15T09:00:00Z
**Depth:** standard
**Files Reviewed:** 4
**Status:** issues_found

## Summary

Phase 19 adds a `labels/` read-only symlink overlay to the FUSE filesystem. The implementation is
structurally sound: `classify()` arm ordering is correct (label ranges appear before the
`>= TREE_SYMLINK_INO_BASE` catch-all), EROFS is enforced in `setattr` and EROFS/EISDIR are
correct in `write` for all label paths, symlink targets use `../../<bucket>/<padded-id>.md`
(no attacker-controlled text in the target), `LabelSnapshot` is rebuilt unconditionally in
`refresh_issues`, and `RwLock` usage avoids deadlock (write guard dropped before subsequent read).

The three Info issues are doc staleness and a redundant variable shadow. One Warning calls out
the layout table in `inode.rs` that is now actively misleading — it describes label ranges as
part of the tree-symlink range, which conflicts with the documented design invariant that all
ranges are disjoint and self-documented.

---

## Warnings

### WR-01: inode.rs layout table describes label ranges as tree-symlink space

**File:** `crates/reposix-fuse/src/inode.rs:18`

**Issue:** The layout table at the top of `inode.rs` says:

```
| `0xC_0000_0000..u64::MAX` | `tree/` leaf symlinks AND `_self.md` entries ...
```

Phase 19 allocates `LABELS_DIR_INO_BASE = 0x10_0000_0000` and
`LABELS_SYMLINK_INO_BASE = 0x14_0000_0000` from this "infinite upward" range that the
table assigns entirely to tree symlinks. The `classify()` function handles this correctly via
arm ordering (label arms appear first), but the table now actively contradicts the actual
layout. A future contributor reading the table would conclude that tree symlinks own the full
`[0xC_0000_0000, u64::MAX)` region — the same region where label inodes now live. The
design principle stated in the same doc is "ranges are intentionally disjoint so every callback
can classify an inode by numeric range before doing any map lookup"; the table should reflect
the true post-Phase-19 partition.

**Fix:**
```rust
//! | `0x8_0000_0000..0xC_0000_0000` | `tree/` interior directories ...
//! | `0xC_0000_0000..0x10_0000_0000` | `tree/` leaf symlinks and `_self.md` entries ...
//! | `0x7_FFFF_FFFF` (fixed) | `labels/` overlay root directory ([`LABELS_ROOT_INO`]).
//! | `0x10_0000_0000..0x14_0000_0000` | `labels/` per-label interior directories.
//! | `0x14_0000_0000..u64::MAX` | `labels/` leaf symlinks (one per label×issue pair).
```

Also add a compile-time assertion in `tree.rs` alongside the existing ones:
```rust
assert!(TREE_SYMLINK_INO_BASE < crate::inode::LABELS_DIR_INO_BASE);
```
This pins the disjoint-range invariant the way existing assertions pin `TREE_DIR_INO_BASE`.

---

## Info

### IN-01: GITIGNORE_INO doc comment still says "7 bytes" / `b"/tree/\n"`

**File:** `crates/reposix-fuse/src/inode.rs:52-54`

**Issue:** The doc comment reads:
```
/// and always returns the bytes `b"/tree/\n"` (7 bytes).
```
Phase 19 changed `GITIGNORE_BYTES` to `b"/tree/\nlabels/\n"` (15 bytes). The constant's own
doc is now incorrect, and the mismatch will mislead anyone who uses it to compute the
pre-baked `gitignore_attr.size`.

**Fix:**
```rust
/// and always returns the bytes `b"/tree/\nlabels/\n"` (15 bytes). Read-only
/// (`perm: 0o444`).
```

---

### IN-02: Variable shadowing in `LabelsRoot` and `LabelDir` lookup arms looks like a safety bypass

**File:** `crates/reposix-fuse/src/fs.rs:1303` and `crates/reposix-fuse/src/fs.rs:1322`

**Issue:** The `lookup` function guards against non-UTF-8 names at line 1224:
```rust
let Some(name_str) = name.to_str() else {
    reply.error(fuser::Errno::from_i32(libc::EINVAL));
    return;
};
```
The `LabelsRoot` arm (line 1303) and `LabelDir` arm (line 1322) then immediately re-bind
`name_str` via `name.to_string_lossy()`, shadowing the already-validated `&str`. Functionally
safe — the outer guard ensures the name is valid UTF-8 before we reach these arms — but the
redundant shadowing makes the code look as though it is intentionally bypassing the UTF-8
check. A reviewer or future contributor who adds a branch inside these arms may not realize the
outer guard already holds.

**Fix:** Delete the inner re-binding in both arms and use the outer `name_str` directly:
```rust
InodeKind::LabelsRoot => {
    // name_str is already validated UTF-8 by the guard above.
    if let Ok(snap) = self.label_snapshot.read() {
        let hit = snap
            .label_dirs
            .iter()
            .find(|(_, (slug, _))| slug.as_str() == name_str);
        ...
    }
}
```

---

### IN-03: No compile-time assertion guards the tree-symlink/label-dir boundary

**File:** `crates/reposix-fuse/src/tree.rs:94-98`

**Issue:** The existing const block asserts `TREE_DIR_INO_BASE < TREE_SYMLINK_INO_BASE` but
does not assert `TREE_SYMLINK_INO_BASE < LABELS_DIR_INO_BASE`. The design doc says "compile-time
assertions pin the ordering" — this assertion is missing and a future refactor that raises
`LABELS_DIR_INO_BASE` could silently break `classify()` without a build failure. The same
pattern exists in `labels.rs` test `label_ino_constants_ordering` as a runtime test, but the
prior tree constants are pinned at compile time.

**Fix:** Add to the existing `const _: ()` block in `tree.rs`:
```rust
const _: () = {
    assert!(TREE_ROOT_INO < crate::inode::FIRST_ISSUE_INODE);
    assert!(TREE_DIR_INO_BASE > crate::inode::FIRST_ISSUE_INODE);
    assert!(TREE_DIR_INO_BASE < TREE_SYMLINK_INO_BASE);
    // Phase 19: label ranges live above tree-symlink space.
    assert!(TREE_SYMLINK_INO_BASE < crate::inode::LABELS_DIR_INO_BASE);
};
```

---

_Reviewed: 2026-04-15T09:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
