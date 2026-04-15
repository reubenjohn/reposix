---
phase: "19"
plan_id: "19-A"
type: execute
wave: 1
depends_on: []
goal: "Implement `labels/` read-only symlink overlay in the FUSE mount — new module, inode constants, FUSE dispatch, and >= 5 unit tests."
files_modified:
  - crates/reposix-fuse/src/inode.rs
  - crates/reposix-fuse/src/labels.rs
  - crates/reposix-fuse/src/fs.rs
  - crates/reposix-fuse/src/lib.rs
autonomous: true
requirements:
  - LABEL-01
  - LABEL-02
  - LABEL-03
  - LABEL-04
  - LABEL-05

must_haves:
  truths:
    - "`ls mount/labels/` lists one directory per distinct label present on any issue"
    - "`ls mount/labels/<label>/` lists symlinks whose names are deduped slugs"
    - "Each symlink in `labels/<label>/` resolves to `../../<bucket>/<padded-id>.md`"
    - "Write attempts on any path under `labels/` return EROFS"
    - "`labels/` row appears in mount-root `_INDEX.md` with distinct label count"
    - "`mount/.gitignore` contains `/tree/\\nlabels/\\n`"
    - "Label snapshot is rebuilt on every `refresh_issues` call — no stale data after a relabel"
  artifacts:
    - path: "crates/reposix-fuse/src/labels.rs"
      provides: "LabelSnapshot struct + build fn + unit tests (>= 5)"
      exports:
        - "LabelSnapshot"
        - "LABELS_ROOT_INO"
        - "LABELS_DIR_INO_BASE"
        - "LABELS_SYMLINK_INO_BASE"
    - path: "crates/reposix-fuse/src/inode.rs"
      provides: "LABELS_ROOT_INO, LABELS_DIR_INO_BASE, LABELS_SYMLINK_INO_BASE constants"
    - path: "crates/reposix-fuse/src/fs.rs"
      provides: "InodeKind arms, readdir/lookup/getattr/readlink dispatch, GITIGNORE_BYTES update, labels row in render_mount_root_index"
    - path: "crates/reposix-fuse/src/lib.rs"
      provides: "pub mod labels;"
  key_links:
    - from: "fs.rs:refresh_issues"
      to: "labels.rs:LabelSnapshot::build"
      via: "rebuild after tree snapshot"
      pattern: "LabelSnapshot::build"
    - from: "fs.rs:InodeKind::classify"
      to: "LABELS_DIR_INO_BASE / LABELS_SYMLINK_INO_BASE"
      via: "match arms placed BEFORE TreeSymlink catch-all"
      pattern: "LABELS_SYMLINK_INO_BASE.*LABELS_DIR_INO_BASE.*LABELS_ROOT_INO.*TREE_SYMLINK_INO_BASE"
    - from: "fs.rs:readlink"
      to: "LabelSnapshot::symlink_targets"
      via: "lock label_snapshot, look up ino"
      pattern: "label_snapshot.*symlink_targets"
---

<objective>
Implement the `labels/` read-only symlink overlay directory in the FUSE mount.

Purpose: LABEL-01..05 from the Phase 19 scope. Every issue that carries labels appears as a symlink inside `mount/labels/<label>/`, pointing back to the canonical file in the bucket. No new backend API surface is required — `Issue::labels: Vec<String>` is already populated by the simulator and GitHub adapter.

Output:
- `crates/reposix-fuse/src/labels.rs` — new pure module (mirrors `tree.rs` but simpler: flat list, no hierarchy, no cycle detection)
- `crates/reposix-fuse/src/inode.rs` — three new inode constants with const-assertion block
- `crates/reposix-fuse/src/fs.rs` — new InodeKind variants + full FUSE dispatch (readdir/lookup/getattr/readlink), GITIGNORE_BYTES update, labels row in `render_mount_root_index`, `label_snapshot` field wired into `refresh_issues`
- `crates/reposix-fuse/src/lib.rs` — `pub mod labels;` declaration
- >= 5 unit tests inside `labels.rs`
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/19-op-1-remainder-labels-and-spaces-directory-views-as-read-onl/19-RESEARCH.md
@crates/reposix-fuse/src/inode.rs
@crates/reposix-fuse/src/tree.rs
@crates/reposix-fuse/src/fs.rs
@crates/reposix-fuse/src/lib.rs

<interfaces>
<!-- Key types from crates/reposix-fuse/src/inode.rs -->
```rust
pub const ROOT_INO: u64 = 1;
pub const BUCKET_DIR_INO: u64 = 2;
pub const TREE_ROOT_INO: u64 = 3;
pub const GITIGNORE_INO: u64 = 4;
pub const BUCKET_INDEX_INO: u64 = 5;
pub const ROOT_INDEX_INO: u64 = 6;
pub const TREE_INDEX_ALLOC_START: u64 = 7;
pub const TREE_INDEX_ALLOC_END: u64 = 0xFFFF;
pub const FIRST_ISSUE_INODE: u64 = 0x1_0000;
```

<!-- Key constants from crates/reposix-fuse/src/tree.rs -->
```rust
pub const TREE_ROOT_INO: u64 = 3;
pub const TREE_DIR_INO_BASE: u64 = 0x8_0000_0000;
pub const TREE_SYMLINK_INO_BASE: u64 = 0xC_0000_0000;
```

<!-- New inode constants to add — from 19-RESEARCH.md §Inode Strategy -->
/// Fixed inode for the `labels/` root directory.
/// Just below TREE_DIR_INO_BASE; disjoint from issue inodes (0x1_0000..)
/// and the tree-index allocator range (7..=0xFFFF).
pub const LABELS_ROOT_INO: u64 = 0x7_FFFF_FFFF;
/// Start of per-label directory inode range (one inode per distinct label).
pub const LABELS_DIR_INO_BASE: u64 = 0x10_0000_0000;
/// Start of label symlink inode range (one inode per issue-in-label).
pub const LABELS_SYMLINK_INO_BASE: u64 = 0x14_0000_0000;

<!-- Key types from reposix-core -->
pub struct Issue {
    pub id: IssueId,
    pub title: String,
    pub labels: Vec<String>,  // already populated by sim + GitHub
    pub parent_id: Option<IssueId>,
    // ...
}
pub struct IssueId(pub u64);

// reposix_core::path helpers
pub fn slug_or_fallback(title: &str, id: IssueId) -> String;
pub fn dedupe_siblings(entries: Vec<(IssueId, String)>) -> Vec<(IssueId, String)>;

<!-- Existing InodeKind in fs.rs — current classify() arms (abridged) -->
enum InodeKind {
    Root, Bucket, TreeRoot, Gitignore, BucketIndex,
    RealFile, TreeDir, TreeSymlink, RootIndex, TreeDirIndex, Unknown,
}

fn classify(ino: u64) -> Self {
    match ino {
        ROOT_INO => Self::Root,
        BUCKET_DIR_INO => Self::Bucket,
        TREE_ROOT_INO => Self::TreeRoot,
        GITIGNORE_INO => Self::Gitignore,
        BUCKET_INDEX_INO => Self::BucketIndex,
        ROOT_INDEX_INO => Self::RootIndex,
        n if (TREE_INDEX_ALLOC_START..=TREE_INDEX_ALLOC_END).contains(&n) => Self::TreeDirIndex,
        n if n >= TREE_SYMLINK_INO_BASE => Self::TreeSymlink,  // catch-all for 0xC_0000_0000..
        n if n >= TREE_DIR_INO_BASE => Self::TreeDir,
        n if n >= FIRST_ISSUE_INODE => Self::RealFile,
        _ => Self::Unknown,
    }
}

<!-- GITIGNORE_BYTES in fs.rs — must change -->
const GITIGNORE_BYTES: &[u8] = b"/tree/\n";   // BEFORE
// Must become:
const GITIGNORE_BYTES: &[u8] = b"/tree/\nlabels/\n";   // AFTER

<!-- render_mount_root_index signature in fs.rs -->
fn render_mount_root_index(
    backend_name: &str,
    project: &str,
    bucket: &str,
    issue_count: usize,
    tree_present: bool,
    generated_at: chrono::DateTime<chrono::Utc>,
) -> Vec<u8>
// Currently emits table rows for .gitignore, <bucket>/, and (if tree_present) tree/
// Must add a labels/ row: | labels/ | directory | {distinct_label_count} |

<!-- refresh_issues in fs.rs (abridged) -->
fn refresh_issues(&self) -> Result<Vec<Issue>, FetchError> {
    // ... fetches issues, rebuilds tree ...
    let has_any_parent = issues.iter().any(|i| i.parent_id.is_some());
    if self.hierarchy_feature || has_any_parent {
        let snap = TreeSnapshot::build(self.bucket, &issues);
        if let Ok(mut guard) = self.tree.write() { *guard = snap; }
    } else if let Ok(mut guard) = self.tree.write() { *guard = TreeSnapshot::default(); }
    // ... invalidates caches ...
    Ok(issues)
}
// Must add AFTER tree rebuild:
//   let label_snap = LabelSnapshot::build(self.bucket, &issues);
//   if let Ok(mut guard) = self.label_snapshot.write() { *guard = label_snap; }
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task A-1: inode constants + `labels.rs` pure module</name>
  <files>
    crates/reposix-fuse/src/inode.rs
    crates/reposix-fuse/src/labels.rs
    crates/reposix-fuse/src/lib.rs
  </files>
  <behavior>
    - Test 1: `LabelSnapshot::build` with two issues each having label "bug" → label_dirs has exactly one entry; both issues appear in that entry's symlink list
    - Test 2: Issue with two labels "bug" and "p1" → appears in both `labels/bug/` and `labels/p1/` independently
    - Test 3: Symlink target format — `labels/bug/0001-title.md` → `"../../issues/00000000001.md"` (exactly 2 `../` hops, zero-padded 11-digit id)
    - Test 4: Label with slash or space (e.g. `"Type: Bug"`) is slugified via `slug_or_fallback`; the dir name returned from readdir does NOT contain raw spaces or colons
    - Test 5: Two different labels that produce the same slug (e.g. `"Status: Bug"` and `"status bug"`) are deduped via `dedupe_siblings` at the label-dir level — both appear as distinct entries
    - Test 6: `LABELS_ROOT_INO < TREE_DIR_INO_BASE` const assertion holds (compile-time)
    - Test 7: `LABELS_DIR_INO_BASE > TREE_SYMLINK_INO_BASE` and `LABELS_DIR_INO_BASE < LABELS_SYMLINK_INO_BASE` const assertions hold
    - Test 8: symlink `FileAttr.size == target.len()` for a sample entry (guards Pitfall 5 from RESEARCH)
    - Test 9: `GITIGNORE_BYTES` equals `b"/tree/\nlabels/\n"` exactly (guards Pitfall 4)
  </behavior>
  <action>
**Step 1 — inode.rs: add three new public constants and const-assertion block.**

Add immediately after `ROOT_INDEX_INO`:

```rust
/// Fixed inode for the `labels/` overlay root directory.
/// Lives just below `TREE_DIR_INO_BASE` (0x8_0000_0000); above the
/// issue inode range (0x1_0000..) and the tree-index allocator range
/// (TREE_INDEX_ALLOC_START..=TREE_INDEX_ALLOC_END = 7..=0xFFFF).
pub const LABELS_ROOT_INO: u64 = 0x7_FFFF_FFFF;

/// Start of the per-label interior directory inode range.
/// One inode allocated per distinct label slug. Range:
/// `LABELS_DIR_INO_BASE .. LABELS_SYMLINK_INO_BASE`.
pub const LABELS_DIR_INO_BASE: u64 = 0x10_0000_0000;

/// Start of the label-symlink inode range.
/// One inode allocated per (label, issue) pair. Range:
/// `LABELS_SYMLINK_INO_BASE .. u64::MAX` (in practice much less).
pub const LABELS_SYMLINK_INO_BASE: u64 = 0x14_0000_0000;
```

Also extend the `fixed_inodes_are_disjoint_from_dynamic_ranges` test in inode.rs to assert:
- `LABELS_ROOT_INO < crate::tree::TREE_DIR_INO_BASE`
- `LABELS_ROOT_INO > FIRST_ISSUE_INODE`
- `LABELS_DIR_INO_BASE > crate::tree::TREE_SYMLINK_INO_BASE`
- `LABELS_DIR_INO_BASE < LABELS_SYMLINK_INO_BASE`

**Step 2 — labels.rs: new pure module.**

Create `crates/reposix-fuse/src/labels.rs`. It is pure — no `fuser` import, no async.

Module-level `#![allow(clippy::module_name_repetitions)]` if needed for `LabelSnapshot`.

```rust
//! Pure in-memory label overlay builder for the Phase-19 `labels/` overlay.
//!
//! Mirrors `tree.rs` but simpler: flat two-level structure (labels root →
//! per-label dir → symlinks), no hierarchy, no cycle risk, depth always 1.
```

Define (following RESEARCH.md §Pattern 1 exactly):

```rust
use std::collections::HashMap;
use reposix_core::path::{dedupe_siblings, slug_or_fallback};
use reposix_core::{Issue, IssueId};
use crate::inode::{LABELS_DIR_INO_BASE, LABELS_SYMLINK_INO_BASE};

pub use crate::inode::{LABELS_ROOT_INO, LABELS_DIR_INO_BASE, LABELS_SYMLINK_INO_BASE};

#[derive(Debug, Default, Clone)]
pub struct LabelEntry {
    pub symlink_ino: u64,
    pub slug: String,
    pub target: String,  // e.g. "../../issues/00000000001.md"
}

#[derive(Debug, Default, Clone)]
pub struct LabelSnapshot {
    /// dir_ino → (label_dir_slug, entries)
    pub label_dirs: HashMap<u64, (String, Vec<LabelEntry>)>,
    /// symlink_ino → target string (for readlink dispatch)
    pub symlink_targets: HashMap<u64, String>,
    /// distinct label count (for render_mount_root_index)
    pub label_count: usize,
}
```

Implement `LabelSnapshot::build(bucket: &str, issues: &[Issue]) -> Self`:

1. Group issues by label: `HashMap<String, Vec<(IssueId, String /* title-slug */)>>`.
2. Collect all label strings, compute their dir slugs via `slug_or_fallback(label, IssueId(i as u64))` for each (i, label) enumerate pair, then dedupe dir slugs across the whole label set using `dedupe_siblings`.
3. For each label group (in the deduped order):
   - Allocate `dir_ino = LABELS_DIR_INO_BASE + offset` (sequential, starting 0).
   - For issues in the group, call `dedupe_siblings(entries)` to get deduped entry slugs.
   - For each `(id, entry_slug)`: allocate `sym_ino = LABELS_SYMLINK_INO_BASE + sym_counter`, build `target = format!("../../{bucket}/{:011}.md", id.0)`, insert into `symlink_targets`.
4. Populate `label_dirs` and set `label_count`.

IMPORTANT on step 3: the `zip` of `by_label.into_iter()` with `deduped_label_names` must be order-stable. Collect `by_label` into a `Vec` sorted by label string before zipping, and produce `label_slugs` from that same sorted vec so the zip alignment is guaranteed.

Depth formula (from RESEARCH §Pattern 1): `"../../{bucket}/{padded_id:011}.md"` — depth is always 1 under `labels/`, so the prefix is always `../../` (2 hops).

**Step 3 — lib.rs: add `pub mod labels;`.**

Add after `pub mod tree;`:
```rust
pub mod labels;
```

Also add to the `pub use` block:
```rust
pub use labels::{LabelSnapshot, LABELS_DIR_INO_BASE, LABELS_ROOT_INO, LABELS_SYMLINK_INO_BASE};
```

**Step 4 — tests inside labels.rs** (>= 5, per `<behavior>` block).

Write all tests listed in the `<behavior>` block. Tests 6, 7, 8, 9 are compile-time assertions or simple unit assertions; tests 1–5 exercise `LabelSnapshot::build` directly with controlled inputs.

For test 9, import `crate::fs::GITIGNORE_BYTES` in the test — or move it to a shared location accessible from both modules. If `GITIGNORE_BYTES` is private to `fs.rs`, test 9 belongs in `fs.rs`'s existing test module instead; place it there.
  </action>
  <verify>
    <automated>cargo test -p reposix-fuse --quiet 2>&1 | tail -20</automated>
  </verify>
  <done>
    - `crates/reposix-fuse/src/labels.rs` exists with `LabelSnapshot`, `LabelEntry`, `build` function, and >= 5 passing unit tests
    - `inode.rs` exports `LABELS_ROOT_INO`, `LABELS_DIR_INO_BASE`, `LABELS_SYMLINK_INO_BASE`
    - `lib.rs` has `pub mod labels;` and re-exports
    - `cargo test -p reposix-fuse --quiet` is green
    - `cargo clippy -p reposix-fuse --all-targets -- -D warnings` is green
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task A-2: FUSE dispatch wiring + GITIGNORE_BYTES + render_mount_root_index labels row</name>
  <files>
    crates/reposix-fuse/src/fs.rs
  </files>
  <behavior>
    - Test 1: `render_mount_root_index` with `label_count = 3` emits a `| labels/ | directory | 3 |` row in the pipe-table
    - Test 2: `render_mount_root_index` with `label_count = 0` still emits the `labels/` row (unconditional, per RESEARCH §Pattern 4)
    - Test 3: `GITIGNORE_BYTES` equals `b"/tree/\nlabels/\n"` (14 bytes)
    - Test 4: `InodeKind::classify(LABELS_ROOT_INO)` returns `InodeKind::LabelsRoot`
    - Test 5: `InodeKind::classify(LABELS_DIR_INO_BASE)` returns `InodeKind::LabelDir`
    - Test 6: `InodeKind::classify(LABELS_SYMLINK_INO_BASE)` returns `InodeKind::LabelSymlink`
    - Test 7: `InodeKind::classify(LABELS_SYMLINK_INO_BASE - 1)` is NOT `LabelSymlink` (boundary check between LabelDir and LabelSymlink ranges)
    - Test 8: `InodeKind::classify(0xC_0000_0001)` still returns `TreeSymlink` (existing behavior unchanged — the new label arms don't steal tree-symlink inodes)
  </behavior>
  <action>
**Step 1 — GITIGNORE_BYTES constant update.**

Change:
```rust
const GITIGNORE_BYTES: &[u8] = b"/tree/\n";
```
to:
```rust
const GITIGNORE_BYTES: &[u8] = b"/tree/\nlabels/\n";
```

The `gitignore_attr` already derives `size` from `GITIGNORE_BYTES.len()` at construction; no other change needed for correct `getattr` size.

**Step 2 — InodeKind: add three new variants and update classify().**

Add to the `InodeKind` enum:
```rust
/// The `labels/` overlay root directory (LABELS_ROOT_INO = 0x7_FFFF_FFFF).
LabelsRoot,
/// An interior per-label directory (LABELS_DIR_INO_BASE..LABELS_SYMLINK_INO_BASE).
LabelDir,
/// A label leaf symlink (LABELS_SYMLINK_INO_BASE..).
LabelSymlink,
```

Update `classify()`. CRITICAL: the new label arms MUST appear BEFORE the `n >= TREE_SYMLINK_INO_BASE` catch-all, because label ranges (starting 0x10_0000_0000) are numerically above TREE_SYMLINK_INO_BASE (0xC_0000_0000). The order must be:

```rust
fn classify(ino: u64) -> Self {
    match ino {
        ROOT_INO => Self::Root,
        BUCKET_DIR_INO => Self::Bucket,
        TREE_ROOT_INO => Self::TreeRoot,
        GITIGNORE_INO => Self::Gitignore,
        BUCKET_INDEX_INO => Self::BucketIndex,
        ROOT_INDEX_INO => Self::RootIndex,
        LABELS_ROOT_INO => Self::LabelsRoot,
        n if (TREE_INDEX_ALLOC_START..=TREE_INDEX_ALLOC_END).contains(&n) => Self::TreeDirIndex,
        n if n >= LABELS_SYMLINK_INO_BASE => Self::LabelSymlink,  // BEFORE TreeSymlink
        n if n >= LABELS_DIR_INO_BASE => Self::LabelDir,          // BEFORE TreeSymlink
        n if n >= TREE_SYMLINK_INO_BASE => Self::TreeSymlink,
        n if n >= TREE_DIR_INO_BASE => Self::TreeDir,
        n if n >= FIRST_ISSUE_INODE => Self::RealFile,
        _ => Self::Unknown,
    }
}
```

Add to the import block at the top of fs.rs:
```rust
use crate::inode::{
    ..., // existing imports
    LABELS_ROOT_INO,
};
use crate::labels::{LabelSnapshot, LABELS_DIR_INO_BASE, LABELS_SYMLINK_INO_BASE};
```

**Step 3 — ReposixFs struct: add `label_snapshot` field.**

Add after the `tree: Arc<RwLock<TreeSnapshot>>` field:
```rust
/// Label overlay snapshot. Rebuilt on each `refresh_issues` call.
/// Always present (even when no issues have labels — snapshot is empty).
/// Built immediately after the tree snapshot in `refresh_issues`.
label_snapshot: Arc<RwLock<LabelSnapshot>>,
```

Add to `ReposixFs::new` (after `tree: Arc::new(RwLock::new(TreeSnapshot::default()))`):
```rust
label_snapshot: Arc::new(RwLock::new(LabelSnapshot::default())),
```

**Step 4 — refresh_issues: rebuild label snapshot.**

Inside `refresh_issues`, immediately after the `if self.hierarchy_feature || has_any_parent { ... }` block (the tree rebuild), add:
```rust
// Rebuild label snapshot unconditionally — labels are independent of hierarchy.
let label_snap = LabelSnapshot::build(self.bucket, &issues);
if let Ok(mut guard) = self.label_snapshot.write() {
    *guard = label_snap;
}
```

**Step 5 — render_mount_root_index: add labels parameter and row.**

Change signature to:
```rust
fn render_mount_root_index(
    backend_name: &str,
    project: &str,
    bucket: &str,
    issue_count: usize,
    tree_present: bool,
    label_count: usize,          // NEW
    generated_at: chrono::DateTime<chrono::Utc>,
) -> Vec<u8>
```

In the body, add after the `tree/` conditional row:
```rust
let _ = writeln!(out, "| labels/ | directory | {label_count} |");
```

Update all call sites of `render_mount_root_index` in `fs.rs` to pass `label_count`. Derive it from the current `label_snapshot`:
```rust
let label_count = self.label_snapshot
    .read()
    .map(|g| g.label_count)
    .unwrap_or(0);
```

**Step 6 — FUSE callbacks: labels dispatch arms.**

For each callback below, add the new arms. Follow the existing tree-dispatch arms as the exact template.

`getattr` — add after `TreeDir` arm:
```rust
InodeKind::LabelsRoot => reply.attr(&ATTR_TTL, &self.labels_attr),
InodeKind::LabelDir => {
    if let Ok(snap) = self.label_snapshot.read() {
        if snap.label_dirs.contains_key(&ino_u) {
            let mut attr = self.labels_attr;
            attr.ino = INodeNo(ino_u);
            reply.attr(&ATTR_TTL, &attr);
        } else {
            reply.error(fuser::Errno::from_i32(libc::ENOENT));
        }
    } else {
        reply.error(fuser::Errno::from_i32(libc::EIO));
    }
}
InodeKind::LabelSymlink => {
    if let Ok(snap) = self.label_snapshot.read() {
        if let Some(target) = snap.symlink_targets.get(&ino_u) {
            reply.attr(&ATTR_TTL, &self.symlink_attr(ino_u, target));
        } else {
            reply.error(fuser::Errno::from_i32(libc::ENOENT));
        }
    } else {
        reply.error(fuser::Errno::from_i32(libc::EIO));
    }
}
```

`lookup` — inside the `InodeKind::Root` arm (root readdir/lookup for ".gitignore", "bucket", "tree"), add:
```rust
"labels" => reply.entry(&ATTR_TTL, &self.labels_attr, 0),
```

Also add a `InodeKind::LabelsRoot` arm in `lookup` (parent is `labels/`, looking up a label dir by name):
```rust
InodeKind::LabelsRoot => {
    let name_str = name.to_string_lossy();
    if let Ok(snap) = self.label_snapshot.read() {
        // find the dir inode whose slug matches name_str
        let hit = snap.label_dirs.iter()
            .find(|(_, (slug, _))| slug == name_str.as_ref());
        if let Some((&dir_ino, _)) = hit {
            let mut attr = self.labels_attr;
            attr.ino = INodeNo(dir_ino);
            reply.entry(&ATTR_TTL, &attr, 0);
        } else {
            reply.error(fuser::Errno::from_i32(libc::ENOENT));
        }
    } else {
        reply.error(fuser::Errno::from_i32(libc::EIO));
    }
}
InodeKind::LabelDir => {
    let name_str = name.to_string_lossy();
    if let Ok(snap) = self.label_snapshot.read() {
        let parent_entries = snap.label_dirs.get(&parent_u);
        if let Some((_, entries)) = parent_entries {
            let hit = entries.iter().find(|e| e.slug == name_str.as_ref());
            if let Some(entry) = hit {
                let attr = self.symlink_attr(entry.symlink_ino, &entry.target);
                reply.entry(&ATTR_TTL, &attr, 0);
            } else {
                reply.error(fuser::Errno::from_i32(libc::ENOENT));
            }
        } else {
            reply.error(fuser::Errno::from_i32(libc::ENOENT));
        }
    } else {
        reply.error(fuser::Errno::from_i32(libc::EIO));
    }
}
```

`readdir` — add new arms:

For `InodeKind::Root` readdir (the existing arm that builds the `entries` vec), add `("labels".to_owned(), LABELS_ROOT_INO, FileType::Directory)` unconditionally alongside the existing `.gitignore`, bucket, `_INDEX.md`, and `tree` entries.

Add new top-level `readdir` arm for `InodeKind::LabelsRoot`:
```rust
InodeKind::LabelsRoot => {
    if let Err(e) = self.refresh_issues() {
        warn!(error = %e, "refresh_issues failed on labels readdir");
    }
    let mut entries = vec![
        (".".to_owned(), LABELS_ROOT_INO, FileType::Directory),
        ("..".to_owned(), ROOT_INO, FileType::Directory),
    ];
    if let Ok(snap) = self.label_snapshot.read() {
        for (&dir_ino, (slug, _)) in &snap.label_dirs {
            entries.push((slug.clone(), dir_ino, FileType::Directory));
        }
    }
    entries
}
```

Add new arm for `InodeKind::LabelDir`:
```rust
InodeKind::LabelDir => {
    let mut entries = vec![
        (".".to_owned(), ino_u, FileType::Directory),
        ("..".to_owned(), LABELS_ROOT_INO, FileType::Directory),
    ];
    if let Ok(snap) = self.label_snapshot.read() {
        if let Some((_, label_entries)) = snap.label_dirs.get(&ino_u) {
            for entry in label_entries {
                entries.push((entry.slug.clone(), entry.symlink_ino, FileType::Symlink));
            }
        }
    }
    entries
}
```

`readlink` — add after the `TreeSymlink` arm:
```rust
InodeKind::LabelSymlink => {
    if let Ok(snap) = self.label_snapshot.read() {
        if let Some(target) = snap.symlink_targets.get(&ino_u) {
            reply.data(std::path::Path::new(target));
        } else {
            reply.error(fuser::Errno::from_i32(libc::ENOENT));
        }
    } else {
        reply.error(fuser::Errno::from_i32(libc::EIO));
    }
}
```

Write callbacks (`setattr`, `write`, `flush`, `release`, `create`, `unlink`) — add `LabelsRoot | LabelDir | LabelSymlink` to the EROFS/EPERM arm that already handles `TreeRoot | TreeDir | TreeSymlink`. These overlays are strictly read-only.

**Step 7 — `labels_attr` field on ReposixFs.**

Add to the struct and initialise in `new()` using the same `dir_attr(LABELS_ROOT_INO, 0o555)` helper already used for `tree_attr`.

**Step 8 — existing match exhaustiveness.**

Every existing `match InodeKind::classify(...)` in `fs.rs` must now handle the three new variants. Add them to existing catch-all arms or add explicit arms. The `read` callback arm for tree/bucket dirs should also include `LabelsRoot | LabelDir` (directories have no `read` data — reply `EISDIR` or follow existing tree convention).

Use `cargo clippy` feedback to find any non-exhaustive match; fix until clean.
  </action>
  <verify>
    <automated>cargo test -p reposix-fuse --quiet 2>&1 | tail -30</automated>
  </verify>
  <done>
    - `GITIGNORE_BYTES` is `b"/tree/\nlabels/\n"` (14 bytes)
    - `InodeKind` has `LabelsRoot`, `LabelDir`, `LabelSymlink` variants
    - `classify()` routes label inode ranges correctly and BEFORE the TreeSymlink catch-all
    - `render_mount_root_index` emits a `labels/` row with distinct label count
    - `refresh_issues` rebuilds `LabelSnapshot` after tree snapshot
    - All FUSE callbacks handle the three new variants (no non-exhaustive match warnings)
    - Write callbacks return EROFS for all label paths
    - `cargo test -p reposix-fuse --quiet` is green (all tests including the 8 from behavior block)
    - `cargo clippy -p reposix-fuse --all-targets -- -D warnings` is clean
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| issue body → label string | `Issue::labels` comes from the backend (sim, GitHub, Confluence). Raw label values are attacker-influenced. |
| label string → FUSE dir name | Converting a label to a directory name used in FUSE readdir/lookup. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-19-01 | Tampering | `labels.rs:build` — raw label string used as dir name | mitigate | Apply `slug_or_fallback` to every label string; never use raw string as FUSE dir name. Prevents `/`-containing labels from corrupting VFS paths. |
| T-19-02 | Tampering | symlink target string — `../../<bucket>/<padded-id>.md` | mitigate | Target built from controlled components only: literal `../../`, trusted `&'static str` bucket, zero-padded numeric id. No label string content enters the target. Pattern follows T-13-05 (tree.rs). |
| T-19-03 | Denial of Service | `refresh_issues` rebuild cost with O(n·labels) issues | accept | All existing issues contribute at most one entry per label; the HashMap rebuild is O(n). Same order as TreeSnapshot::build which already ships. |
| T-19-04 | Elevation of Privilege | write attempts on `labels/` paths | mitigate | `LabelsRoot | LabelDir | LabelSymlink` added to the EROFS return arm in all write callbacks (`setattr`, `write`, `flush`, `release`, `create`, `unlink`). Consistent with existing tree/ treatment. |
| T-19-05 | Information Disclosure | stale label snapshot served after relabel | mitigate | `LabelSnapshot` rebuilt inside `refresh_issues` on every readdir. Same invalidation pattern as tree snapshot (Pitfall 3 from RESEARCH). |
</threat_model>

<verification>
```bash
# Full test suite for the fuse crate
cargo test -p reposix-fuse --quiet

# Lint — zero warnings required
cargo clippy -p reposix-fuse --all-targets -- -D warnings

# Format
cargo fmt --all -- --check

# Workspace-level sanity (do not break other crates)
cargo check --workspace --quiet
```

Expected: all green. Any clippy warning about non-exhaustive match on InodeKind means a callback arm was missed — fix before declaring done.
</verification>

<success_criteria>
- `crates/reposix-fuse/src/labels.rs` exists with `LabelSnapshot`, `LabelEntry`, `LabelSnapshot::build`, >= 5 unit tests, all green
- Three new inode constants in `inode.rs` with const-assertion block
- `lib.rs` declares `pub mod labels` and re-exports the constants
- `GITIGNORE_BYTES = b"/tree/\nlabels/\n"` (14 bytes)
- `InodeKind` has `LabelsRoot`, `LabelDir`, `LabelSymlink` with correct classify() ordering (label ranges before TreeSymlink catch-all)
- `render_mount_root_index` emits `labels/` row unconditionally with label count
- `refresh_issues` rebuilds label snapshot alongside tree snapshot
- All write callbacks return EROFS for label paths
- `cargo test -p reposix-fuse --quiet` green
- `cargo clippy -p reposix-fuse --all-targets -- -D warnings` clean
</success_criteria>

<output>
After completing both tasks, create `.planning/phases/19-op-1-remainder-labels-and-spaces-directory-views-as-read-onl/19-A-SUMMARY.md` following the template at `@$HOME/.claude/get-shit-done/templates/summary.md`.
</output>
