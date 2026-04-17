//! Lazy in-memory overlay for Phase-24 `pages/<id>.attachments/` directories.
//!
//! Mirrors [`crate::comments::CommentsSnapshot`] but with two critical differences:
//! - **Binary content.** Attachment bodies are arbitrary bytes (images, PDFs, etc.),
//!   not UTF-8 Markdown. The `rendered` field is `Vec<u8>` and stays empty until
//!   `fetch_attachments_for_page` eagerly downloads the body (up to the 50 MiB cap).
//! - **Filename sanitization.** Attachment titles from the API are sanitized via
//!   [`sanitize_attachment_filename`] before being used as POSIX filenames.
//!   This prevents path traversal via attacker-controlled filenames (T-24-02-01).
//!
//! # Security
//!
//! - T-24-02-01: `sanitize_attachment_filename()` replaces any character outside
//!   `[a-zA-Z0-9._-]` with `_`, preventing directory traversal via the filename.
//! - T-24-02-02: Files > 50 MiB are skipped during eager fetch and return `EFBIG`
//!   on `read()`, preventing unbounded memory use.
//! - T-24-02-03: `entry.rendered` bytes are never written to `tracing::*` spans.
//!   Only `attachment_id`, `file_size`, and `page_id` appear in logs.

#![allow(clippy::module_name_repetitions)]

use std::sync::atomic::{AtomicU64, Ordering};

use dashmap::DashMap;

pub use crate::inode::{ATTACHMENTS_DIR_INO_BASE, ATTACHMENTS_FILE_INO_BASE};

/// A single attachment file entry inside a `.attachments/` directory.
#[derive(Debug, Clone)]
pub struct AttachmentEntry {
    /// Inode for this attachment file.
    pub file_ino: u64,
    /// Sanitized POSIX filename: `[a-zA-Z0-9._-]` only, max 255 bytes.
    pub filename: String,
    /// Binary body; empty `Vec` until `fetch_attachments_for_page` populates it.
    /// T-24-02-03: never log these bytes.
    pub rendered: Vec<u8>,
    /// `fileSize` from the Confluence attachments list endpoint.
    /// Used for `getattr` before the binary body is fetched.
    pub file_size: u64,
    /// Relative download path (e.g. `/wiki/download/attachments/...`).
    /// Prepend base URL before fetching (T-24-02-04: SG-01 allowlist applies).
    pub download_url: String,
    /// MIME type string from the API.
    pub media_type: String,
}

/// Lazy per-page attachment cache.
///
/// FUSE callbacks read from this via `ReposixFs`. Writes happen lazily:
/// `lookup(Bucket, "<id>.attachments")` allocates the dir inode but does NOT
/// fetch attachments. `readdir(AttachmentsDir)` triggers the one-shot
/// `ConfluenceBackend::list_attachments(page_id)` + `download_attachment` call.
///
/// Uses `DashMap` throughout so FUSE callback threads (fuser workers) can
/// populate without a global write lock.
#[derive(Debug, Default)]
pub struct AttachmentsSnapshot {
    /// `page_ino` → (fetched: bool, `Vec<AttachmentEntry>`).
    ///
    /// `fetched = true` means we've successfully called `list_attachments` for
    /// this page at least once (entries may still be empty for pages with no
    /// attachments). `fetched = false` means the dir inode is allocated but no
    /// `list_attachments` call has completed yet.
    by_page: DashMap<u64, (bool, Vec<AttachmentEntry>)>,
    /// `page_ino` → `AttachmentsDir` inode (the dir materialized on first `lookup`).
    page_to_dir_ino: DashMap<u64, u64>,
    /// `AttachmentsDir` inode → `page_ino` (reverse map for getattr/readdir).
    dir_ino_to_page: DashMap<u64, u64>,
    /// `AttachmentFile` inode → `page_ino` (for read/getattr dispatch).
    file_ino_to_page: DashMap<u64, u64>,
    /// Monotonic allocator for `AttachmentsDir` inodes.
    next_dir_ino: AtomicU64,
    /// Monotonic allocator for `AttachmentFile` inodes.
    next_file_ino: AtomicU64,
}

impl AttachmentsSnapshot {
    /// Create an empty snapshot with allocators pinned to the declared bases.
    #[must_use]
    pub fn new() -> Self {
        Self {
            by_page: DashMap::new(),
            page_to_dir_ino: DashMap::new(),
            dir_ino_to_page: DashMap::new(),
            file_ino_to_page: DashMap::new(),
            next_dir_ino: AtomicU64::new(ATTACHMENTS_DIR_INO_BASE),
            next_file_ino: AtomicU64::new(ATTACHMENTS_FILE_INO_BASE),
        }
    }

    /// Allocate (or return existing) `AttachmentsDir` inode for `page_ino`.
    ///
    /// Idempotent: the second call for the same `page_ino` returns the same inode.
    /// Does NOT fetch attachments — populates only the dir ↔ page maps.
    pub fn ensure_dir(&self, page_ino: u64) -> u64 {
        if let Some(existing) = self.page_to_dir_ino.get(&page_ino) {
            return *existing;
        }
        let dir_ino = self.next_dir_ino.fetch_add(1, Ordering::SeqCst);
        // Race: two threads could both bump; the DashMap insert below picks one winner.
        match self.page_to_dir_ino.entry(page_ino) {
            dashmap::Entry::Occupied(e) => *e.get(),
            dashmap::Entry::Vacant(e) => {
                e.insert(dir_ino);
                self.dir_ino_to_page.insert(dir_ino, page_ino);
                self.by_page.insert(page_ino, (false, Vec::new()));
                dir_ino
            }
        }
    }

    /// Mark a page's attachments as fetched and store entries.
    /// Allocates `file_ino` for each entry that does not already have one.
    /// The entries passed in are expected to have `file_ino` pre-allocated
    /// via [`Self::alloc_file_ino`].
    pub fn mark_fetched(&self, page_ino: u64, entries: Vec<AttachmentEntry>) {
        for entry in &entries {
            self.file_ino_to_page.insert(entry.file_ino, page_ino);
        }
        self.by_page.insert(page_ino, (true, entries));
    }

    /// Has this page's attachments been fetched yet?
    #[must_use]
    pub fn is_fetched(&self, page_ino: u64) -> bool {
        self.by_page.get(&page_ino).is_some_and(|e| e.0)
    }

    /// Return the entries for a page if fetched, else `None`.
    #[must_use]
    pub fn entries_for_page(&self, page_ino: u64) -> Option<Vec<AttachmentEntry>> {
        self.by_page
            .get(&page_ino)
            .and_then(|e| if e.0 { Some(e.1.clone()) } else { None })
    }

    /// Look up the `page_ino` for a given `AttachmentsDir` inode.
    #[must_use]
    pub fn page_of_dir(&self, dir_ino: u64) -> Option<u64> {
        self.dir_ino_to_page.get(&dir_ino).map(|v| *v)
    }

    /// Look up a single attachment entry by file inode.
    #[must_use]
    pub fn entry_by_file_ino(&self, file_ino: u64) -> Option<AttachmentEntry> {
        let page_ino = *self.file_ino_to_page.get(&file_ino)?;
        let guard = self.by_page.get(&page_ino)?;
        guard.1.iter().find(|e| e.file_ino == file_ino).cloned()
    }

    /// Allocate a fresh file inode for a new attachment entry.
    pub fn alloc_file_ino(&self) -> u64 {
        self.next_file_ino.fetch_add(1, Ordering::SeqCst)
    }

    /// Update the binary body for an attachment once downloaded.
    /// No-op if `file_ino` is not found.
    pub fn update_entry_rendered(&self, file_ino: u64, bytes: Vec<u8>) {
        let Some(page_ino) = self.file_ino_to_page.get(&file_ino).map(|v| *v) else {
            return;
        };
        if let Some(mut guard) = self.by_page.get_mut(&page_ino) {
            if let Some(entry) = guard.1.iter_mut().find(|e| e.file_ino == file_ino) {
                entry.rendered = bytes;
            }
        }
    }
}

/// Sanitize a Confluence attachment title for use as a POSIX filename.
///
/// Allows `[a-zA-Z0-9._-]`; replaces everything else with `_`.
/// Returns `None` if the result is empty or exceeds 255 bytes.
///
/// # Security (T-24-02-01)
/// Path separators (`/`, `\`) and null bytes are replaced with `_`, preventing
/// directory traversal via attacker-controlled attachment filenames.
#[must_use]
pub fn sanitize_attachment_filename(title: &str) -> Option<String> {
    let sanitized: String = title
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                c
            } else {
                '_'
            }
        })
        .collect();
    if sanitized.is_empty() || sanitized.len() > 255 {
        return None;
    }
    Some(sanitized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attachments_snapshot_ensure_dir_idempotent() {
        let snap = AttachmentsSnapshot::new();
        let d1 = snap.ensure_dir(100);
        let d2 = snap.ensure_dir(100); // second call must return same inode
        assert_eq!(d1, d2, "ensure_dir is idempotent per page_ino");
        assert!(d1 >= ATTACHMENTS_DIR_INO_BASE);
    }

    #[test]
    fn attachments_snapshot_page_of_dir() {
        let snap = AttachmentsSnapshot::new();
        let dir_ino = snap.ensure_dir(200);
        assert_eq!(snap.page_of_dir(dir_ino), Some(200));
    }

    #[test]
    fn attachments_snapshot_entry_by_file_ino() {
        let snap = AttachmentsSnapshot::new();
        let _ = snap.ensure_dir(300);
        let file_ino = snap.alloc_file_ino();
        let entry = AttachmentEntry {
            file_ino,
            filename: "test.png".to_owned(),
            rendered: b"PNG_DATA".to_vec(),
            file_size: 8,
            download_url: "/wiki/download/test.png".to_owned(),
            media_type: "image/png".to_owned(),
        };
        snap.mark_fetched(300, vec![entry.clone()]);
        let found = snap.entry_by_file_ino(file_ino).expect("entry must exist");
        assert_eq!(found.filename, "test.png");
        assert_eq!(found.file_ino, file_ino);
    }

    #[test]
    fn sanitize_attachment_filename_allows_alnum_dot_dash() {
        assert_eq!(
            sanitize_attachment_filename("my-file_v1.0.pdf"),
            Some("my-file_v1.0.pdf".to_owned())
        );
    }

    #[test]
    fn sanitize_attachment_filename_replaces_slash() {
        // Path traversal attempt: slashes must become underscores
        assert_eq!(
            sanitize_attachment_filename("../../etc/passwd.txt"),
            Some(".._.._etc_passwd.txt".to_owned())
        );
    }

    #[test]
    fn sanitize_attachment_filename_empty_returns_none() {
        assert_eq!(sanitize_attachment_filename(""), None);
    }

    #[test]
    fn sanitize_attachment_filename_all_special_returns_some() {
        // "!!!" → "___" — non-empty after replacement, so Some
        assert_eq!(
            sanitize_attachment_filename("!!!"),
            Some("___".to_owned())
        );
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn attachment_ino_constants_ordering() {
        assert!(
            ATTACHMENTS_DIR_INO_BASE > crate::inode::COMMENTS_FILE_INO_BASE,
            "ATTACHMENTS_DIR_INO_BASE must be above COMMENTS_FILE_INO_BASE"
        );
        assert!(
            ATTACHMENTS_FILE_INO_BASE > ATTACHMENTS_DIR_INO_BASE,
            "ATTACHMENTS_FILE_INO_BASE must be above ATTACHMENTS_DIR_INO_BASE"
        );
    }
}
