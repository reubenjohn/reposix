//! Monotonic inode registry for the read-only FUSE mount.
//!
//! # Layout
//!
//! - Inode `1` is the root directory (handled by the Filesystem impl, not the
//!   registry — reserved).
//! - Inodes `2..=0xFFFF` are reserved for future synthetic files
//!   (`/.reposix/audit`, etc.). The registry never allocates in this range.
//! - Issues are allocated inodes starting at `0x1_0000` (`65_536`), increasing
//!   monotonically as they are first seen.
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

/// First dynamic inode. Values `2..=0xFFFF` are reserved for future synthetic
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
        for ino in 2..=0xFFFF {
            assert!(
                r.lookup_ino(ino).is_none(),
                "ino {ino:#x} should be unmapped"
            );
        }
    }
}
