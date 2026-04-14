//! Monotonic inode registry for the read-only FUSE mount.
//!
//! # Layout
//!
//! Full mount-level inode layout (Phase 13 Wave C):
//!
//! | Range | Purpose |
//! |-------|---------|
//! | `1` | Mount root (FUSE convention; fixed in [`ROOT_INO`]). |
//! | `2` | [`BUCKET_DIR_INO`] — the per-backend collection directory (`pages/` or `issues/`). |
//! | `3` | [`TREE_ROOT_INO`] — the synthesized `tree/` overlay root (only populated when the backend supports `BackendFeature::Hierarchy`). Mirrored from [`crate::tree::TREE_ROOT_INO`]. |
//! | `4` | [`GITIGNORE_INO`] — the synthesized `/tree/\n` `.gitignore` file. Always present. |
//! | `5` | [`BUCKET_INDEX_INO`] — the synthesized `_INDEX.md` file inside the bucket directory. Always present. |
//! | `6..=0xFFFF` | Reserved for future synthetic files (`/.reposix/audit`, per-space roots, etc.). The [`InodeRegistry`] never allocates in this range. |
//! | `0x1_0000..` | Real issue/page files under `<bucket>/<padded-id>.md`. Allocated monotonically by [`InodeRegistry`]. |
//! | `0x8_0000_0000..0xC_0000_0000` | `tree/` interior directories (allocated by [`crate::tree::TreeSnapshot`]). |
//! | `0xC_0000_0000..u64::MAX` | `tree/` leaf symlinks AND `_self.md` entries (allocated by [`crate::tree::TreeSnapshot`]). |
//!
//! The ranges are intentionally disjoint so every callback in `fs.rs` can
//! classify an inode by numeric range **before** doing any map lookup; the
//! compile-time assertions in `tree.rs` pin the ordering.
//!
//! # Lifetime
//!
//! The registry is in-process only (no `SQLite` persistence — that is Phase S
//! write-path territory). `intern` is idempotent per-issue: the same
//! [`IssueId`] always maps to the same inode for the lifetime of the
//! [`InodeRegistry`]. Old inodes are never reused or expired; this matches
//! the FUSE expectation that inode numbers remain stable across readdir
//! refreshes, so long-lived open file handles keep working even if the
//! backend issue list drifts.

use std::sync::atomic::{AtomicU64, Ordering};

use dashmap::DashMap;
use reposix_core::IssueId;

/// The mount root. Always inode 1 per FUSE convention.
pub const ROOT_INO: u64 = 1;

/// The root-collection bucket directory (`pages/` for Confluence,
/// `issues/` for sim + GitHub).
pub const BUCKET_DIR_INO: u64 = 2;

/// The synthesized `tree/` overlay root directory. Mirrors
/// [`crate::tree::TREE_ROOT_INO`]. Emitted iff the backend supports
/// `BackendFeature::Hierarchy` or any loaded issue has a non-`None`
/// `parent_id`.
pub const TREE_ROOT_INO: u64 = 3;

/// The synthesized `.gitignore` file at the mount root. Always present
/// and always returns the bytes `b"/tree/\n"` (7 bytes). Read-only
/// (`perm: 0o444`).
pub const GITIGNORE_INO: u64 = 4;

/// The synthesized `_INDEX.md` file inside the bucket directory
/// (`mount/issues/_INDEX.md` or `mount/pages/_INDEX.md`). Always
/// present. Rendered on demand from the cached issue list; read-only
/// (`perm: 0o444`). See Phase 15 for the rendering contract.
pub const BUCKET_INDEX_INO: u64 = 5;

/// First dynamic inode. Values `6..=0xFFFF` are reserved for future synthetic
/// files; issues start here.
pub const FIRST_ISSUE_INODE: u64 = 0x1_0000;

/// Bidirectional inode ↔ issue-id map.
#[derive(Debug)]
pub struct InodeRegistry {
    id_to_ino: DashMap<IssueId, u64>,
    ino_to_id: DashMap<u64, IssueId>,
    next: AtomicU64,
}

impl InodeRegistry {
    /// Create an empty registry with `next = FIRST_ISSUE_INODE`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            id_to_ino: DashMap::new(),
            ino_to_id: DashMap::new(),
            next: AtomicU64::new(FIRST_ISSUE_INODE),
        }
    }

    /// Return the inode for `id`, allocating a fresh one on first call.
    ///
    /// Subsequent calls with the same `id` return the same inode (idempotent).
    #[must_use]
    pub fn intern(&self, id: IssueId) -> u64 {
        if let Some(existing) = self.id_to_ino.get(&id) {
            return *existing;
        }
        // Race is harmless: two threads racing will each allocate a fresh inode
        // then one of the two insertions will lose. We accept the wasted inode
        // (the `next` counter bumps twice) rather than holding a write lock
        // around the allocate-and-insert dance.
        let candidate = self.next.fetch_add(1, Ordering::SeqCst);
        match self.id_to_ino.entry(id) {
            dashmap::Entry::Occupied(e) => *e.get(),
            dashmap::Entry::Vacant(e) => {
                e.insert(candidate);
                self.ino_to_id.insert(candidate, id);
                candidate
            }
        }
    }

    /// Reverse lookup: given an inode (issued by `intern`), return the
    /// [`IssueId`] or `None` if the inode is unknown.
    #[must_use]
    pub fn lookup_ino(&self, ino: u64) -> Option<IssueId> {
        self.ino_to_id.get(&ino).map(|e| *e)
    }

    /// Forward lookup: given an [`IssueId`], return its inode (if interned).
    #[must_use]
    pub fn lookup_id(&self, id: IssueId) -> Option<u64> {
        self.id_to_ino.get(&id).map(|e| *e)
    }

    /// Refresh the registry from a fresh list of issue IDs (typically the
    /// result of a `GET /projects/:slug/issues` call inside `readdir`).
    ///
    /// Interns any missing IDs. Returns `(inode, id)` pairs in input order so
    /// the caller can drive the `ReplyDirectory` protocol without a second
    /// lookup. Does NOT remove entries for IDs that disappeared from the
    /// backend — stale inodes remain valid so long-lived open file handles
    /// keep working; the next `read` against a deleted issue will surface
    /// the backend's 404 as `ENOENT`.
    pub fn refresh<I: IntoIterator<Item = IssueId>>(&self, ids: I) -> Vec<(u64, IssueId)> {
        ids.into_iter().map(|id| (self.intern(id), id)).collect()
    }
}

impl Default for InodeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_at_first_issue_inode() {
        let r = InodeRegistry::new();
        assert_eq!(r.intern(IssueId(7)), FIRST_ISSUE_INODE);
    }

    #[test]
    fn intern_is_idempotent() {
        let r = InodeRegistry::new();
        let first = r.intern(IssueId(7));
        let second = r.intern(IssueId(7));
        assert_eq!(first, second);
    }

    #[test]
    fn intern_allocates_sequentially() {
        let r = InodeRegistry::new();
        let a = r.intern(IssueId(7));
        let b = r.intern(IssueId(42));
        assert_eq!(a, FIRST_ISSUE_INODE);
        assert_eq!(b, FIRST_ISSUE_INODE + 1);
    }

    #[test]
    fn lookup_ino_returns_id() {
        let r = InodeRegistry::new();
        let ino = r.intern(IssueId(7));
        assert_eq!(r.lookup_ino(ino), Some(IssueId(7)));
    }

    #[test]
    fn lookup_ino_unknown_is_none() {
        let r = InodeRegistry::new();
        assert_eq!(r.lookup_ino(5), None);
        assert_eq!(r.lookup_ino(0xFFFF), None);
    }

    #[test]
    fn refresh_returns_pairs_in_order() {
        let r = InodeRegistry::new();
        let pairs = r.refresh([IssueId(1), IssueId(2), IssueId(3)]);
        assert_eq!(pairs.len(), 3);
        assert_eq!(pairs[0].1, IssueId(1));
        assert_eq!(pairs[1].1, IssueId(2));
        assert_eq!(pairs[2].1, IssueId(3));
    }

    #[test]
    fn refresh_stable_inodes_across_calls() {
        let r = InodeRegistry::new();
        let first = r.refresh([IssueId(1), IssueId(2), IssueId(3)]);
        let second = r.refresh([IssueId(1), IssueId(2), IssueId(3)]);
        assert_eq!(first, second);
    }

    #[test]
    fn reserved_range_is_unmapped() {
        let r = InodeRegistry::new();
        // Inodes 6..=0xFFFF are reserved-for-future-synthetics; the
        // registry never allocates here. (1..=5 are fixed synthetic slots
        // — root/bucket/tree-root/gitignore/bucket-index — also not
        // allocated by the dynamic registry.)
        for ino in 6..=0xFFFF {
            assert!(
                r.lookup_ino(ino).is_none(),
                "ino {ino:#x} should be unmapped"
            );
        }
    }

    #[test]
    #[allow(
        clippy::assertions_on_constants,
        reason = "every assertion below is a CONSTANT relationship between named \
                  u64 consts defined across two sibling modules (inode::* and \
                  tree::*). The whole point of the test is to fail the build if \
                  a future refactor ever drifts one of them — a const_eval_select \
                  inside a test body is the simplest, most readable form of that \
                  check, and the lint's suggested `const { assert!(..) }` block \
                  would require hoisting all six comparisons out of the test \
                  discovery path."
    )]
    fn fixed_inodes_are_disjoint_from_dynamic_ranges() {
        // Fixed synthetic inodes at the mount root live below the
        // dynamic issue range, which in turn lives below the tree-dir and
        // tree-symlink ranges. This matches the layout doc at the top of
        // the module and the compile-time assertion in `tree.rs`.
        assert!(ROOT_INO < FIRST_ISSUE_INODE);
        assert!(BUCKET_DIR_INO < FIRST_ISSUE_INODE);
        assert!(TREE_ROOT_INO < FIRST_ISSUE_INODE);
        assert!(GITIGNORE_INO < FIRST_ISSUE_INODE);
        assert!(BUCKET_INDEX_INO < FIRST_ISSUE_INODE);
        // The tree dir / symlink bases live strictly above the dynamic
        // issue range (see tree.rs compile-time assertion for the
        // full cross-module invariant).
        assert!(FIRST_ISSUE_INODE < crate::tree::TREE_DIR_INO_BASE);
        assert!(crate::tree::TREE_DIR_INO_BASE < crate::tree::TREE_SYMLINK_INO_BASE);
        // The five fixed slots must be distinct from each other.
        let fixed = [
            ROOT_INO,
            BUCKET_DIR_INO,
            TREE_ROOT_INO,
            GITIGNORE_INO,
            BUCKET_INDEX_INO,
        ];
        for (i, a) in fixed.iter().enumerate() {
            for b in fixed.iter().skip(i + 1) {
                assert_ne!(a, b, "fixed inodes must be pairwise distinct");
            }
        }
        // TREE_ROOT_INO here must equal the tree module's declared const
        // (both represent the same directory; Wave C dispatch uses the
        // inode:: re-export, but tree.rs still owns the canonical
        // definition).
        assert_eq!(TREE_ROOT_INO, crate::tree::TREE_ROOT_INO);
    }
}
