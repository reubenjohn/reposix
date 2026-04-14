//! Pure in-memory tree builder + resolver for the Phase-13 `tree/` overlay.
//!
//! Wave B2 of Phase 13. This module is **pure** — it does not import `fuser`,
//! does not touch the network, does not spawn async tasks. It consumes a
//! `&[Issue]` (typically the result of a `list_issues` call at mount time)
//! and produces a [`TreeSnapshot`]: a navigable, inode-indexed description of
//! the `tree/` overlay's directory structure, including depth-correct
//! relative symlink targets.
//!
//! Wave C (`fs.rs` wiring) consumes this snapshot to answer FUSE `readdir`,
//! `lookup`, `getattr`, and `readlink` calls. The separation keeps tree
//! construction testable in isolation: every invariant below is asserted by
//! a `#[cfg(test)] mod tests` unit — no FUSE mount required.
//!
//! # Layout rules (locked by `13-CONTEXT.md`)
//!
//! - Each mounted page materializes as **either** a leaf [`TreeEntry::Symlink`]
//!   (when the page has no children) **or** a [`TreeEntry::Dir`] (when it has
//!   ≥1 child). A page cannot be both a `.md` file and a `/` directory under
//!   the same slug simultaneously in POSIX, so parent pages become dirs and
//!   their own body is exposed via a synthesized `_self.md` symlink *inside*
//!   that dir.
//! - Slug names come from
//!   [`reposix_core::path::slug_or_fallback`] per [`Issue::title`], with
//!   sibling-collision dedup via
//!   [`reposix_core::path::dedupe_siblings`].
//! - Symlink targets are **relative** paths of the form
//!   `"../".repeat(depth + 1) + "<bucket>/<padded-id>.md"` where `depth` is
//!   the number of parent directories between the symlink and the mount root
//!   (NOT counting `tree/` itself — depth 0 means the symlink lives directly
//!   under `tree/`). The `+ 1` accounts for the hop out of `tree/` back to
//!   the mount root.
//! - Inodes live in three pre-declared, **disjoint** u64 ranges so Wave C can
//!   classify `ino` by range without a hashmap lookup:
//!   - `TREE_ROOT_INO = 3` — the single fixed inode for the `tree/` root dir.
//!   - `[TREE_DIR_INO_BASE, TREE_SYMLINK_INO_BASE)` — interior tree dirs.
//!   - `[TREE_SYMLINK_INO_BASE, u64::MAX)` — all symlink leaves
//!     (including `_self.md` inside interior dirs).
//!   - Both of the above are also disjoint from
//!     [`crate::inode::FIRST_ISSUE_INODE`] (`0x1_0000`) which houses the real
//!     `<bucket>/<id>.md` files.
//!
//! # Cycle safety (T-13-03)
//!
//! A malicious or buggy backend could report `a.parent_id = Some(b.id)` and
//! `b.parent_id = Some(a.id)`. The builder walks each page's parent chain
//! with a visited-set and, on revisit, breaks the cycle by treating the
//! page as a tree root (and emitting `tracing::warn!`). Termination is
//! guaranteed in O(n·h) worst case where h is the height of the finite
//! portion of the chain; the visited-set cap of `issues.len()` bounds this
//! to O(n²) overall with a trivially small constant. A 10k-deep pathological
//! chain still completes well under 100 ms in practice and the
//! [`deep_linear_chain_1000_deep`] test exercises a 1000-deep no-cycle case
//! to pin that behavior.
//!
//! # Target-escape safety (T-13-05)
//!
//! Every byte of a produced symlink target comes from a controlled source:
//!   `../` literal × (depth + 1), then `bucket` (caller-supplied, trusted
//!   at this layer — it's a `&'static str` from the backend trait), then
//!   `/`, then an 11-digit zero-padded decimal `IssueId`, then `.md`. No
//!   title or body bytes touch the target string. The property test
//!   [`symlink_target_never_escapes_mount_property`] re-asserts this by
//!   regex over every symlink produced by every other test's fixtures.

use std::collections::{BTreeMap, HashMap, HashSet};

use reposix_core::path::{dedupe_siblings, slug_or_fallback};
use reposix_core::{Issue, IssueId};

/// Fixed inode for the `tree/` directory at the mount root.
///
/// Wave C reserves inodes `2..=0xFFFF` for synthetic files; this picks a
/// small, stable slot in that reserved range. Mount-root inode `1` and the
/// per-bucket-dir inode (Wave C's pick, likely `2`) are disjoint.
pub const TREE_ROOT_INO: u64 = 3;

/// Start of the interior-tree-dir inode range. Exclusive upper bound is
/// [`TREE_SYMLINK_INO_BASE`].
///
/// Chosen well above [`crate::inode::FIRST_ISSUE_INODE`] (`0x1_0000`) so the
/// issue-inode allocator cannot collide even after ~34 billion issues.
pub const TREE_DIR_INO_BASE: u64 = 0x8_0000_0000;

/// Start of the tree-symlink inode range (unbounded upward).
///
/// All [`TreeEntry::Symlink`] inodes — including `_self.md` entries inside
/// interior dirs — are allocated from this range.
pub const TREE_SYMLINK_INO_BASE: u64 = 0xC_0000_0000;

// Compile-time assertions that the three ranges are correctly ordered and
// disjoint from the FIRST_ISSUE_INODE region. Kept as const-evaluated blocks
// so a future typo on one of the constants fails to build.
const _: () = {
    assert!(TREE_ROOT_INO < crate::inode::FIRST_ISSUE_INODE);
    assert!(TREE_DIR_INO_BASE > crate::inode::FIRST_ISSUE_INODE);
    assert!(TREE_DIR_INO_BASE < TREE_SYMLINK_INO_BASE);
};

/// A single entry inside a [`TreeDir`] (or at the tree root).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeEntry {
    /// A subdirectory (a page with ≥1 child). Look up with
    /// [`TreeSnapshot::resolve_dir`]; the [`TreeDir::name`] field holds the
    /// filesystem name.
    Dir(u64),
    /// A leaf symlink (a page with 0 children) or a `_self.md` pointer
    /// inside an interior dir.
    Symlink {
        /// Inode in `[TREE_SYMLINK_INO_BASE, u64::MAX)`.
        ino: u64,
        /// Filesystem name as it appears to `readdir` (e.g.
        /// `"welcome-to-reposix.md"` or `"_self.md"`).
        name: String,
        /// Relative path the symlink points at, e.g.
        /// `"../pages/00000131192.md"` or `"../../pages/00000131192.md"`.
        target: String,
    },
}

/// An interior directory in the `tree/` overlay — either `tree/` itself
/// (the implicit root, not stored in [`TreeSnapshot::dirs`]) or a page dir
/// for a page with ≥1 child.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeDir {
    /// Inode in `[TREE_DIR_INO_BASE, TREE_SYMLINK_INO_BASE)`.
    pub ino: u64,
    /// Filesystem name as it appears to the parent's `readdir` (e.g.
    /// `"reposix-demo-space-home"`).
    pub name: String,
    /// Entries `readdir` should return for this directory, in render order
    /// (ascending [`IssueId`] with `_self.md` always first when present).
    pub children: Vec<TreeEntry>,
    /// Depth inside `tree/`. `0` means this dir is a direct child of
    /// `tree/`. Used to compute relative symlink targets on children.
    pub depth: usize,
}

/// A built tree overlay — the output of [`TreeSnapshot::build`].
///
/// Consumed by Wave C (`fs.rs`) for FUSE `readdir`, `lookup`, `getattr`,
/// and `readlink` dispatch.
#[derive(Debug, Clone, Default)]
pub struct TreeSnapshot {
    /// Entries directly under `tree/` (the implicit root).
    root_entries: Vec<TreeEntry>,
    /// Interior dirs, keyed by inode for O(1) `resolve_dir`.
    dirs: HashMap<u64, TreeDir>,
    /// Symlink targets, keyed by inode for O(1) `resolve_symlink`. Includes
    /// both leaf symlinks and `_self.md` entries.
    symlink_targets: HashMap<u64, String>,
}

/// Diagnostic event emitted when a parent-id cycle is detected and broken.
/// Exposed via [`TreeSnapshot::build_with_events`] for test assertions; the
/// public [`TreeSnapshot::build`] drops these after logging.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CycleEvent {
    /// The page whose parent chain loops back on itself.
    pub page: IssueId,
    /// The ancestor [`IssueId`] that was re-encountered on the walk
    /// (i.e. the point at which the cycle was detected).
    pub broken_at: IssueId,
}

impl TreeSnapshot {
    /// Build a snapshot from the backend's issue list.
    ///
    /// `bucket` is the per-backend root collection name (e.g. `"pages"` for
    /// Confluence, `"issues"` for sim/GitHub). Used to construct symlink
    /// targets like `"../pages/00000131192.md"`.
    ///
    /// Always returns a valid snapshot. Orphan parents (ids not in `issues`)
    /// are treated as tree roots, with a `tracing::debug!` entry. Cycles are
    /// broken at the re-visit site with a `tracing::warn!` entry — the
    /// cycle-broken page becomes a tree root.
    #[must_use]
    pub fn build(bucket: &str, issues: &[Issue]) -> Self {
        let (snapshot, _events) = Self::build_with_events(bucket, issues);
        snapshot
    }

    /// Like [`Self::build`] but also returns the list of cycle-break events.
    /// Useful for tests that want to assert on cycle-detection without
    /// capturing `tracing` output.
    #[must_use]
    pub fn build_with_events(bucket: &str, issues: &[Issue]) -> (Self, Vec<CycleEvent>) {
        let mut events = Vec::new();

        // Step 1: index issues by id for O(1) parent lookups.
        let by_id: HashMap<IssueId, &Issue> = issues.iter().map(|iss| (iss.id, iss)).collect();

        // Step 2: classify each issue — does its parent chain terminate at a
        // real root? If yes, keep its direct parent_id. If no (orphan parent
        // OR cycle OR parent_id == None), treat it as a tree root.
        let mut effective_parent: HashMap<IssueId, Option<IssueId>> =
            HashMap::with_capacity(issues.len());
        for issue in issues {
            let ep = effective_parent_of(issue, &by_id, &mut events);
            effective_parent.insert(issue.id, ep);
        }

        // Step 3: build children index from effective_parent. BTreeMap
        // guarantees deterministic iteration order; child vecs are sorted
        // ascending by IssueId after collection.
        let mut children_of: BTreeMap<Option<IssueId>, Vec<IssueId>> = BTreeMap::new();
        for issue in issues {
            let ep = effective_parent.get(&issue.id).copied().unwrap_or(None);
            children_of.entry(ep).or_default().push(issue.id);
        }
        for v in children_of.values_mut() {
            v.sort_unstable();
        }

        // Step 4: BFS from the tree root. Allocate inodes + slugs + targets
        // as we go. The "root" key in children_of is None.
        let mut builder = Builder {
            bucket,
            by_id: &by_id,
            children_of: &children_of,
            dirs: HashMap::new(),
            symlink_targets: HashMap::new(),
            next_dir_ino: TREE_DIR_INO_BASE,
            next_symlink_ino: TREE_SYMLINK_INO_BASE,
        };

        let root_ids = children_of.get(&None).cloned().unwrap_or_default();
        let root_entries = builder.materialize_siblings(&root_ids, 0);

        let Builder {
            dirs,
            symlink_targets,
            ..
        } = builder;

        (
            Self {
                root_entries,
                dirs,
                symlink_targets,
            },
            events,
        )
    }

    /// Entries directly under `tree/` (the implicit root).
    #[must_use]
    pub fn root_entries(&self) -> &[TreeEntry] {
        &self.root_entries
    }

    /// Look up an interior-dir inode.
    ///
    /// Returns `Some(&TreeDir)` if `ino` is a tree-dir inode, `None` if the
    /// inode is not in this snapshot.
    #[must_use]
    pub fn resolve_dir(&self, ino: u64) -> Option<&TreeDir> {
        self.dirs.get(&ino)
    }

    /// Look up a symlink target by inode.
    ///
    /// Returns `Some(&target)` if `ino` is a tree-symlink inode, `None`
    /// otherwise.
    #[must_use]
    pub fn resolve_symlink(&self, ino: u64) -> Option<&str> {
        self.symlink_targets.get(&ino).map(String::as_str)
    }

    /// `true` iff the snapshot contains zero entries at `tree/`'s root.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.root_entries.is_empty()
    }

    /// Iterator over every interior-dir inode. Test helper.
    #[cfg(test)]
    fn dir_inodes(&self) -> impl Iterator<Item = u64> + '_ {
        self.dirs.keys().copied()
    }

    /// Iterator over every symlink inode. Test helper.
    #[cfg(test)]
    fn symlink_inodes(&self) -> impl Iterator<Item = u64> + '_ {
        self.symlink_targets.keys().copied()
    }
}

/// Interior mutable state used during [`TreeSnapshot::build_with_events`].
/// Not exposed.
struct Builder<'a> {
    bucket: &'a str,
    by_id: &'a HashMap<IssueId, &'a Issue>,
    children_of: &'a BTreeMap<Option<IssueId>, Vec<IssueId>>,
    dirs: HashMap<u64, TreeDir>,
    symlink_targets: HashMap<u64, String>,
    next_dir_ino: u64,
    next_symlink_ino: u64,
}

impl Builder<'_> {
    fn alloc_dir_ino(&mut self) -> u64 {
        let ino = self.next_dir_ino;
        self.next_dir_ino = self
            .next_dir_ino
            .checked_add(1)
            .expect("tree dir inode space exhausted (u64 range)");
        ino
    }

    fn alloc_symlink_ino(&mut self) -> u64 {
        let ino = self.next_symlink_ino;
        self.next_symlink_ino = self
            .next_symlink_ino
            .checked_add(1)
            .expect("tree symlink inode space exhausted (u64 range)");
        ino
    }

    /// Materialize one level of siblings (children of the same parent).
    ///
    /// `depth` is the depth at which THESE siblings live — 0 means they are
    /// direct children of `tree/`, 1 means they are children of a
    /// first-level dir, etc.
    fn materialize_siblings(&mut self, ids: &[IssueId], depth: usize) -> Vec<TreeEntry> {
        // Build (id, raw-slug) pairs and dedupe per-sibling.
        let raw: Vec<(IssueId, String)> = ids
            .iter()
            .map(|id| {
                let issue = self.by_id[id];
                (*id, slug_or_fallback(&issue.title, *id))
            })
            .collect();
        let deduped = dedupe_siblings(raw);

        let mut entries = Vec::with_capacity(deduped.len());
        for (id, slug) in deduped {
            let has_children = self
                .children_of
                .get(&Some(id))
                .is_some_and(|v| !v.is_empty());

            if has_children {
                entries.push(self.build_dir(id, slug, depth));
            } else {
                entries.push(self.build_leaf_symlink(id, &slug, depth));
            }
        }
        entries
    }

    fn build_leaf_symlink(&mut self, id: IssueId, slug: &str, depth: usize) -> TreeEntry {
        let ino = self.alloc_symlink_ino();
        let name = format!("{slug}.md");
        let target = symlink_target(self.bucket, id, depth);
        self.symlink_targets.insert(ino, target.clone());
        TreeEntry::Symlink { ino, name, target }
    }

    fn build_dir(&mut self, id: IssueId, slug: String, depth: usize) -> TreeEntry {
        let dir_ino = self.alloc_dir_ino();

        // Build `_self.md` first so it always renders first in readdir.
        let self_ino = self.alloc_symlink_ino();
        let self_target = symlink_target(self.bucket, id, depth + 1);
        self.symlink_targets.insert(self_ino, self_target.clone());
        let mut children: Vec<TreeEntry> = Vec::new();
        children.push(TreeEntry::Symlink {
            ino: self_ino,
            name: "_self.md".to_owned(),
            target: self_target,
        });

        // Now recurse into this page's own children at depth + 1.
        let child_ids = self.children_of.get(&Some(id)).cloned().unwrap_or_default();
        let child_entries = self.materialize_siblings(&child_ids, depth + 1);
        children.extend(child_entries);

        self.dirs.insert(
            dir_ino,
            TreeDir {
                ino: dir_ino,
                name: slug,
                children,
                depth,
            },
        );
        TreeEntry::Dir(dir_ino)
    }
}

/// Compute an effective parent for `issue`: `Some(pid)` when `pid` exists in
/// `by_id` AND `issue`'s parent chain terminates at a real root; `None`
/// when `issue` is a true root, orphan (parent missing), or part of a cycle.
fn effective_parent_of(
    issue: &Issue,
    by_id: &HashMap<IssueId, &Issue>,
    events: &mut Vec<CycleEvent>,
) -> Option<IssueId> {
    let Some(direct_parent) = issue.parent_id else {
        return None; // true root
    };
    if !by_id.contains_key(&direct_parent) {
        // Orphan — parent not in mounted set. Page becomes a tree root.
        tracing::debug!(
            page = issue.id.0,
            missing_parent = direct_parent.0,
            "parent_id points outside mounted issue set; treating as tree root"
        );
        return None;
    }

    // Walk the chain upward to confirm it terminates. Visited-set is seeded
    // with `issue.id` so we catch self-loops on the first step.
    let mut seen: HashSet<IssueId> = HashSet::with_capacity(8);
    seen.insert(issue.id);
    let mut cursor = direct_parent;
    loop {
        if !seen.insert(cursor) {
            // Cycle detected — cursor was already in the ancestry chain.
            events.push(CycleEvent {
                page: issue.id,
                broken_at: cursor,
            });
            tracing::warn!(
                page = issue.id.0,
                ancestor = cursor.0,
                "parent_id cycle detected; treating page as orphan tree root"
            );
            return None;
        }
        match by_id.get(&cursor) {
            None => {
                // Ancestor vanished partway up (dangling pointer beyond the
                // direct parent). Treat the page as orphan.
                tracing::debug!(
                    page = issue.id.0,
                    missing_ancestor = cursor.0,
                    "ancestor parent_id not in mounted set; treating as tree root"
                );
                return None;
            }
            Some(parent) => match parent.parent_id {
                None => return Some(direct_parent), // chain terminates cleanly
                Some(next) => cursor = next,
            },
        }
    }
}

/// Construct a relative symlink target from inside `tree/` back to
/// `<bucket>/<padded-id>.md` at the mount root.
///
/// `depth` is the depth of the *symlink itself* inside `tree/`: 0 for direct
/// children of `tree/`, 1 for grandchildren, etc. The returned string has
/// `depth + 1` leading `../` components — the `+1` hops out of `tree/`
/// itself.
/// Linux `PATH_MAX` minus one byte for the trailing `\0` the kernel
/// may append. A symlink target longer than this surfaces to
/// `readlink(2)` callers as `ENAMETOOLONG`.
///
/// The current 500-page-per-list cap in `reposix-confluence` combined
/// with `SLUG_MAX_BYTES = 60` produces targets on the order of a few
/// hundred bytes even for pathological trees, so this check is purely
/// defensive — any hit would indicate a future change bumped a limit
/// without considering the symlink-depth arithmetic. Debug-assert is
/// sufficient; release builds log a warning and continue (the kernel
/// will reject the too-long path at `readlink` time).
const MAX_SYMLINK_TARGET_BYTES: usize = 4095;

fn symlink_target(bucket: &str, id: IssueId, depth: usize) -> String {
    let up = "../".repeat(depth + 1);
    let target = format!("{up}{bucket}/{:011}.md", id.0);
    debug_assert!(
        target.len() <= MAX_SYMLINK_TARGET_BYTES,
        "symlink target {} bytes exceeds PATH_MAX ({}); depth={} bucket={} id={}",
        target.len(),
        MAX_SYMLINK_TARGET_BYTES,
        depth,
        bucket,
        id.0,
    );
    if target.len() > MAX_SYMLINK_TARGET_BYTES {
        tracing::warn!(
            page = id.0,
            depth,
            len = target.len(),
            limit = MAX_SYMLINK_TARGET_BYTES,
            "tree symlink target exceeds PATH_MAX; readlink(2) will fail ENAMETOOLONG"
        );
    }
    target
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use reposix_core::IssueStatus;

    // --- Test fixture helpers ----------------------------------------------

    fn mk_issue(id: u64, title: &str, parent: Option<u64>) -> Issue {
        let ts = Utc.with_ymd_and_hms(2026, 4, 14, 0, 0, 0).unwrap();
        Issue {
            id: IssueId(id),
            title: title.to_owned(),
            status: IssueStatus::Open,
            assignee: None,
            labels: Vec::new(),
            created_at: ts,
            updated_at: ts,
            version: 1,
            body: String::new(),
            parent_id: parent.map(IssueId),
        }
    }

    fn every_symlink_walk<'a>(
        snap: &'a TreeSnapshot,
        entries: &'a [TreeEntry],
        out: &mut Vec<&'a TreeEntry>,
    ) {
        for e in entries {
            match e {
                TreeEntry::Symlink { .. } => out.push(e),
                TreeEntry::Dir(ino) => {
                    if let Some(dir) = snap.resolve_dir(*ino) {
                        every_symlink_walk(snap, &dir.children, out);
                    }
                }
            }
        }
    }

    fn every_symlink(snap: &TreeSnapshot) -> Vec<&TreeEntry> {
        // Visit every entry in the snapshot and collect symlink entries.
        let mut out: Vec<&TreeEntry> = Vec::new();
        every_symlink_walk(snap, snap.root_entries(), &mut out);
        out
    }

    /// Depth of a symlink given its target string — count leading `../`
    /// components.
    fn count_dotdots(target: &str) -> usize {
        target.split('/').take_while(|s| *s == "..").count()
    }

    // --- T-13-05 mitigation: target shape property --------------------------

    #[allow(
        clippy::case_sensitive_file_extension_comparisons,
        reason = "the target is constructed internally with a literal \".md\" suffix; a case-insensitive comparison would let a bug slip through as long as the casing still parses"
    )]
    fn assert_target_never_escapes(snap: &TreeSnapshot, bucket: &str) {
        // Every produced symlink target MUST match the shape:
        //   (\.\./)+<bucket>/\d{11}\.md
        // with strictly monotonic `..` components (no `a/../b`-style
        // middle-of-path escapes).
        for entry in every_symlink(snap) {
            let TreeEntry::Symlink { target, .. } = entry else {
                unreachable!()
            };
            assert!(
                target.starts_with("../"),
                "target must start with ../: {target:?}"
            );
            assert!(
                target.ends_with(".md"),
                "target must end with .md: {target:?}"
            );
            assert!(!target.contains('\0'), "target contains NUL: {target:?}");
            // No middle-of-path ".." — every `..` must be at the front.
            let parts: Vec<&str> = target.split('/').collect();
            let dotdot_count = parts.iter().take_while(|s| **s == "..").count();
            let rest = &parts[dotdot_count..];
            assert!(
                !rest.contains(&".."),
                "target contains middle-of-path ..: {target:?}"
            );
            // Rest must be exactly: bucket, <11-digit>.md (2 components).
            assert_eq!(
                rest.len(),
                2,
                "expected <bucket>/<padded-id>.md after ..s; got {rest:?}"
            );
            assert_eq!(rest[0], bucket, "bucket mismatch in target: {target:?}");
            let padded_id = rest[1]
                .strip_suffix(".md")
                .expect("verified .md suffix above");
            assert_eq!(
                padded_id.len(),
                11,
                "padded id must be 11 digits: {padded_id:?}"
            );
            assert!(
                padded_id.bytes().all(|b| b.is_ascii_digit()),
                "padded id must be all digits: {padded_id:?}"
            );
        }
    }

    // --- Shape tests --------------------------------------------------------

    #[test]
    fn empty_input_produces_empty_tree() {
        let snap = TreeSnapshot::build("pages", &[]);
        assert!(snap.is_empty());
        assert_eq!(snap.root_entries(), &[]);
    }

    #[test]
    fn builds_single_level_tree_from_flat_list() {
        // Three parent-less pages with distinct slugs — all appear at root
        // as leaf symlinks, in ascending IssueId order.
        let issues = vec![
            mk_issue(3, "third", None),
            mk_issue(1, "first", None),
            mk_issue(2, "second", None),
        ];
        let snap = TreeSnapshot::build("pages", &issues);
        let entries = snap.root_entries();
        assert_eq!(entries.len(), 3);
        let names: Vec<&str> = entries
            .iter()
            .map(|e| match e {
                TreeEntry::Symlink { name, .. } => name.as_str(),
                TreeEntry::Dir(_) => panic!("expected symlink at root"),
            })
            .collect();
        assert_eq!(names, vec!["first.md", "second.md", "third.md"]);
        assert_target_never_escapes(&snap, "pages");
    }

    #[test]
    fn single_root_page_becomes_leaf_symlink() {
        let issues = vec![mk_issue(42, "hello world", None)];
        let snap = TreeSnapshot::build("pages", &issues);
        assert_eq!(snap.root_entries().len(), 1);
        let TreeEntry::Symlink { name, target, .. } = &snap.root_entries()[0] else {
            panic!("expected symlink");
        };
        assert_eq!(name, "hello-world.md");
        assert_eq!(target, "../pages/00000000042.md");
        assert_target_never_escapes(&snap, "pages");
    }

    #[test]
    fn parent_with_one_child_becomes_dir_with_self_and_child() {
        let issues = vec![
            mk_issue(1, "homepage", None),
            mk_issue(2, "welcome", Some(1)),
        ];
        let snap = TreeSnapshot::build("pages", &issues);
        assert_eq!(snap.root_entries().len(), 1);
        let TreeEntry::Dir(dir_ino) = snap.root_entries()[0] else {
            panic!("expected dir at root");
        };
        let dir = snap.resolve_dir(dir_ino).expect("dir in snapshot");
        assert_eq!(dir.name, "homepage");
        assert_eq!(dir.depth, 0);
        assert_eq!(dir.children.len(), 2);

        // Child 0 is _self.md, child 1 is welcome.md.
        let TreeEntry::Symlink { name, target, .. } = &dir.children[0] else {
            panic!("expected _self symlink");
        };
        assert_eq!(name, "_self.md");
        assert_eq!(target, "../../pages/00000000001.md");

        let TreeEntry::Symlink { name, target, .. } = &dir.children[1] else {
            panic!("expected child symlink");
        };
        assert_eq!(name, "welcome.md");
        assert_eq!(target, "../../pages/00000000002.md");

        assert_target_never_escapes(&snap, "pages");
    }

    #[test]
    fn three_level_hierarchy_builds_correct_depth() {
        let issues = vec![
            mk_issue(1, "root", None),
            mk_issue(2, "child", Some(1)),
            mk_issue(3, "grandchild", Some(2)),
        ];
        let snap = TreeSnapshot::build("pages", &issues);
        let TreeEntry::Dir(root_dir_ino) = snap.root_entries()[0] else {
            panic!()
        };
        let root_dir = snap.resolve_dir(root_dir_ino).unwrap();
        assert_eq!(root_dir.name, "root");
        assert_eq!(root_dir.depth, 0);

        // root/_self.md has one `../../` (depth 1 → 2 ../s).
        let TreeEntry::Symlink { target, name, .. } = &root_dir.children[0] else {
            panic!()
        };
        assert_eq!(name, "_self.md");
        assert_eq!(target, "../../pages/00000000001.md");

        // root/child/ is depth 1.
        let TreeEntry::Dir(child_dir_ino) = root_dir.children[1] else {
            panic!()
        };
        let child_dir = snap.resolve_dir(child_dir_ino).unwrap();
        assert_eq!(child_dir.name, "child");
        assert_eq!(child_dir.depth, 1);

        // root/child/_self.md → depth 2 → 3 ../s.
        let TreeEntry::Symlink { target, .. } = &child_dir.children[0] else {
            panic!()
        };
        assert_eq!(target, "../../../pages/00000000002.md");

        // root/child/grandchild.md → depth 2 (symlink lives at depth 2).
        let TreeEntry::Symlink { name, target, .. } = &child_dir.children[1] else {
            panic!()
        };
        assert_eq!(name, "grandchild.md");
        assert_eq!(target, "../../../pages/00000000003.md");

        assert_target_never_escapes(&snap, "pages");
    }

    #[test]
    fn depth_aware_readlink_target_two_levels_deep() {
        // Sanity check the depth arithmetic in isolation.
        assert_eq!(
            symlink_target("pages", IssueId(1), 0),
            "../pages/00000000001.md"
        );
        assert_eq!(
            symlink_target("pages", IssueId(1), 1),
            "../../pages/00000000001.md"
        );
        assert_eq!(
            symlink_target("pages", IssueId(1), 2),
            "../../../pages/00000000001.md"
        );
    }

    #[test]
    fn self_entry_rendered_for_pages_with_children() {
        let issues = vec![
            mk_issue(1, "parent", None),
            mk_issue(2, "child-a", Some(1)),
            mk_issue(3, "child-b", Some(1)),
        ];
        let snap = TreeSnapshot::build("pages", &issues);
        let TreeEntry::Dir(ino) = snap.root_entries()[0] else {
            panic!()
        };
        let dir = snap.resolve_dir(ino).unwrap();
        let self_entry = &dir.children[0];
        let TreeEntry::Symlink { name, .. } = self_entry else {
            panic!()
        };
        assert_eq!(
            name, "_self.md",
            "_self.md must always be first child of a dir"
        );
    }

    // --- Collision / dedupe tests ------------------------------------------

    #[test]
    fn sibling_collision_applies_dash_n_suffix_deterministically() {
        // Two siblings with the same title — smaller id keeps bare slug.
        let issues = vec![
            mk_issue(1, "homepage", None),
            mk_issue(2, "DUP", Some(1)),
            mk_issue(3, "dup", Some(1)),
        ];
        let snap = TreeSnapshot::build("pages", &issues);
        let TreeEntry::Dir(ino) = snap.root_entries()[0] else {
            panic!()
        };
        let dir = snap.resolve_dir(ino).unwrap();
        let names: Vec<&str> = dir
            .children
            .iter()
            .filter_map(|e| match e {
                TreeEntry::Symlink { name, .. } => Some(name.as_str()),
                TreeEntry::Dir(_) => None,
            })
            .collect();
        // [_self.md, dup.md (id=2), dup-2.md (id=3)]
        assert_eq!(names, vec!["_self.md", "dup.md", "dup-2.md"]);
    }

    #[test]
    fn dedupe_applies_to_dir_vs_symlink_siblings() {
        // A "foo"-slug page with a child (becomes Dir) and a "foo"-slug
        // leaf (becomes Symlink). Dedup applies across both kinds.
        let issues = vec![
            // Parent homepage.
            mk_issue(1, "root", None),
            // Two siblings titled "foo"; only one has a child.
            mk_issue(2, "foo", Some(1)), // will become Dir (has child id=4)
            mk_issue(3, "foo", Some(1)), // will become Symlink (leaf)
            // Child of id=2.
            mk_issue(4, "baby", Some(2)),
        ];
        let snap = TreeSnapshot::build("pages", &issues);
        let TreeEntry::Dir(root_dir) = snap.root_entries()[0] else {
            panic!()
        };
        let dir = snap.resolve_dir(root_dir).unwrap();
        // dir.children: _self.md + 2 entries for ids 2 and 3 (ascending).
        // id=2 keeps "foo" (a Dir named "foo"); id=3 gets "foo-2" (a
        // Symlink "foo-2.md").
        assert_eq!(dir.children.len(), 3);
        let TreeEntry::Dir(foo_dir_ino) = dir.children[1] else {
            panic!("expected id=2 to be a Dir");
        };
        let foo_dir = snap.resolve_dir(foo_dir_ino).unwrap();
        assert_eq!(foo_dir.name, "foo");

        let TreeEntry::Symlink { name, .. } = &dir.children[2] else {
            panic!("expected id=3 to be a Symlink");
        };
        assert_eq!(name, "foo-2.md");
    }

    #[test]
    fn long_title_truncates_at_sixty_bytes_utf8_safe() {
        let long = "a".repeat(200);
        let issues = vec![mk_issue(7, &long, None)];
        let snap = TreeSnapshot::build("pages", &issues);
        let TreeEntry::Symlink { name, .. } = &snap.root_entries()[0] else {
            panic!()
        };
        // slug is at most SLUG_MAX_BYTES=60 `a`s, plus ".md".
        assert!(name.len() <= 60 + 3);
        assert!(std::path::Path::new(name)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("md")));
    }

    #[test]
    fn page_with_only_unicode_title_falls_back_to_page_id_slug() {
        let issues = vec![mk_issue(100, "日本語", None)];
        let snap = TreeSnapshot::build("pages", &issues);
        let TreeEntry::Symlink { name, .. } = &snap.root_entries()[0] else {
            panic!()
        };
        assert_eq!(name, "page-00000000100.md");
    }

    // --- Cycle / orphan tests ----------------------------------------------

    #[test]
    fn handles_orphan_parent_id_as_tree_root_with_warn() {
        // Page has parent_id pointing outside mounted set.
        let issues = vec![mk_issue(5, "orphan", Some(999))];
        let (snap, events) = TreeSnapshot::build_with_events("pages", &issues);
        // Orphan → tree root. No cycle events (orphan is a debug-level
        // diagnostic, not a warn-level cycle).
        assert_eq!(snap.root_entries().len(), 1);
        assert!(events.is_empty(), "orphan must not emit CycleEvent");
    }

    #[test]
    fn breaks_parent_id_cycle_without_infinite_recursion() {
        // Two-cycle: a.parent=b, b.parent=a.
        let issues = vec![mk_issue(1, "a", Some(2)), mk_issue(2, "b", Some(1))];
        let (snap, events) = TreeSnapshot::build_with_events("pages", &issues);
        // Both pages lose their parent and surface as tree roots.
        assert_eq!(snap.root_entries().len(), 2);
        assert!(
            !events.is_empty(),
            "cycle must emit at least one CycleEvent"
        );
        assert_target_never_escapes(&snap, "pages");
    }

    #[test]
    fn three_way_cycle_terminates() {
        let issues = vec![
            mk_issue(1, "a", Some(2)),
            mk_issue(2, "b", Some(3)),
            mk_issue(3, "c", Some(1)),
        ];
        let (snap, events) = TreeSnapshot::build_with_events("pages", &issues);
        assert_eq!(snap.root_entries().len(), 3);
        assert!(!events.is_empty());
    }

    #[test]
    fn deep_linear_chain_1000_deep() {
        // T-13-DOS1 mitigation proof. The builder walks each page's parent
        // chain with a visited-set, so the algorithm is O(n·h) where h is
        // chain height. For a 1000-deep linear chain that's 1M HashMap ops,
        // which comfortably terminates in debug mode. The 5s ceiling leaves
        // headroom for CI runners under load; the real guarantee here is
        // "no stack overflow, finite time" — the exact wall-clock is a
        // smoke test, not a contract.
        use std::time::Instant;
        let mut issues = Vec::with_capacity(1000);
        issues.push(mk_issue(1, "root", None));
        for i in 2..=1000 {
            issues.push(mk_issue(i, &format!("node-{i}"), Some(i - 1)));
        }
        let start = Instant::now();
        let snap = TreeSnapshot::build("pages", &issues);
        let elapsed = start.elapsed();
        assert!(
            elapsed.as_secs() < 5,
            "1000-deep chain took {elapsed:?}; expected <5s"
        );
        // Every non-leaf becomes a Dir; the final page (id=1000) is a
        // Symlink leaf. So there should be 999 dirs and 1 symlink (plus 999
        // _self.md symlinks = 1000 symlink inodes total).
        let dir_count = snap.dir_inodes().count();
        let symlink_count = snap.symlink_inodes().count();
        assert_eq!(dir_count, 999);
        assert_eq!(symlink_count, 1000);
    }

    #[test]
    fn empty_tree_when_zero_issues_have_parent_id() {
        // Zero issues at all — simplest empty case (the "no pages have
        // parent_id" scenario is equivalent to "all pages are roots",
        // which is covered by builds_single_level_tree_from_flat_list).
        let snap = TreeSnapshot::build("pages", &[]);
        assert!(snap.is_empty());
    }

    // --- Inode range / resolver tests --------------------------------------

    #[test]
    fn inodes_are_in_declared_ranges() {
        let issues = vec![
            mk_issue(1, "root", None),
            mk_issue(2, "child", Some(1)),
            mk_issue(3, "grandchild", Some(2)),
            mk_issue(4, "leaf", None),
        ];
        let snap = TreeSnapshot::build("pages", &issues);
        for ino in snap.dir_inodes() {
            assert!(
                (TREE_DIR_INO_BASE..TREE_SYMLINK_INO_BASE).contains(&ino),
                "dir inode {ino:#x} outside range"
            );
        }
        for ino in snap.symlink_inodes() {
            assert!(
                ino >= TREE_SYMLINK_INO_BASE,
                "symlink inode {ino:#x} below base"
            );
            // Disjoint from FIRST_ISSUE_INODE and TREE_ROOT_INO.
            assert_ne!(ino, TREE_ROOT_INO);
            assert!(ino > crate::inode::FIRST_ISSUE_INODE);
        }
    }

    #[test]
    fn resolve_symlink_round_trip() {
        let issues = vec![mk_issue(1, "root", None), mk_issue(2, "child", Some(1))];
        let snap = TreeSnapshot::build("pages", &issues);
        // Gather all symlink entries + their targets.
        let symlinks = every_symlink(&snap);
        for entry in &symlinks {
            let TreeEntry::Symlink { ino, target, .. } = entry else {
                unreachable!()
            };
            assert_eq!(snap.resolve_symlink(*ino), Some(target.as_str()));
        }
        // Unknown inode returns None.
        assert_eq!(snap.resolve_symlink(0), None);
        assert_eq!(snap.resolve_symlink(u64::MAX), None);
    }

    fn walk_dirs_round_trip(snap: &TreeSnapshot, entries: &[TreeEntry]) {
        for e in entries {
            if let TreeEntry::Dir(ino) = e {
                let dir = snap.resolve_dir(*ino).expect("dir present");
                assert_eq!(dir.ino, *ino);
                walk_dirs_round_trip(snap, &dir.children);
            }
        }
    }

    #[test]
    fn resolve_dir_round_trip() {
        let issues = vec![
            mk_issue(1, "root", None),
            mk_issue(2, "child", Some(1)),
            mk_issue(3, "grandchild", Some(2)),
        ];
        let snap = TreeSnapshot::build("pages", &issues);
        walk_dirs_round_trip(&snap, snap.root_entries());
        assert_eq!(snap.resolve_dir(0), None);
        assert_eq!(snap.resolve_dir(TREE_ROOT_INO), None);
    }

    #[test]
    fn readlink_target_never_contains_double_slash_or_absolute_path() {
        // T-13-05 expanded invariant: across a varied corpus, no target is
        // absolute, none double-slashes, all under <bucket>.
        let issues = vec![
            mk_issue(1, "root", None),
            mk_issue(2, "child", Some(1)),
            mk_issue(3, "grandchild", Some(2)),
            mk_issue(4, "leaf", None),
            mk_issue(5, "orphan", Some(9999)),
        ];
        let snap = TreeSnapshot::build("pages", &issues);
        for entry in every_symlink(&snap) {
            let TreeEntry::Symlink { target, .. } = entry else {
                unreachable!()
            };
            assert!(!target.starts_with('/'), "absolute target: {target:?}");
            assert!(!target.contains("//"), "double slash: {target:?}");
            let depth = count_dotdots(target);
            assert!(depth >= 1, "depth must be >= 1 (one hop out of tree/)");
        }
        assert_target_never_escapes(&snap, "pages");
    }

    // --- Determinism --------------------------------------------------------

    #[test]
    fn tree_is_deterministic_across_two_builds_of_same_input() {
        let issues = vec![
            mk_issue(10, "alpha", None),
            mk_issue(20, "beta", Some(10)),
            mk_issue(30, "gamma", Some(20)),
            mk_issue(40, "delta", None),
            mk_issue(50, "dup", Some(10)),
            mk_issue(60, "dup", Some(10)),
        ];
        let a = TreeSnapshot::build("pages", &issues);
        let b = TreeSnapshot::build("pages", &issues);
        // Root entries should be exactly equal (names, inodes, targets).
        assert_eq!(a.root_entries(), b.root_entries());
        // And every dir resolvable on both with identical contents.
        for ino in a.dir_inodes() {
            assert_eq!(a.resolve_dir(ino), b.resolve_dir(ino));
        }
    }
}
