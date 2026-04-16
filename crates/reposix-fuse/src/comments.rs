//! Lazy in-memory overlay for the Phase-23 `pages/<id>.comments/` directories.
//!
//! Mirrors [`crate::labels::LabelSnapshot`] but with two critical differences:
//! - **Lazy per-page fetch.** Labels ride on the `list_issues` payload for free;
//!   comments require an extra HTTP round-trip per page, so the snapshot
//!   populates on-demand when the kernel first `readdir`s or `lookup`s a
//!   `CommentsDir` inode.
//! - **Per-comment inode allocation.** `LabelSnapshot` allocates sequentially
//!   from a base; [`CommentsSnapshot`] uses an atomic counter because
//!   comment-file inodes are allocated lazily across multiple page visits.
//!
//! # Security
//!
//! - T-23-03-01: comment body text is tainted; `render_comment_file` places
//!   the body AFTER a closing `---\n\n` frontmatter fence so body content
//!   cannot inject YAML fields.
//! - T-23-03-02: `render_comment_file` returns `None` when the comment `id` or
//!   `parent_comment_id` is not strictly numeric ASCII — prevents path-traversal
//!   via filename construction (WR-02 pattern from `resolve_space_id`).
//! - T-23-03-03: comment bodies NEVER appear in `tracing::*` spans. Only
//!   `page_id`, `comment_id`, `kind` are logged.

#![allow(clippy::module_name_repetitions)]

use std::fmt::Write as _;
use std::sync::atomic::{AtomicU64, Ordering};

use dashmap::DashMap;

pub use crate::inode::{COMMENTS_DIR_INO_BASE, COMMENTS_FILE_INO_BASE};

/// A single comment file entry inside a `.comments/` directory.
#[derive(Debug, Clone)]
pub struct CommentEntry {
    /// Inode for this comment file.
    pub file_ino: u64,
    /// Filename: `<comment-id>.md` (`comment_id` is numeric-ASCII only).
    pub filename: String,
    /// Rendered file bytes (YAML frontmatter + Markdown body).
    pub rendered: Vec<u8>,
}

/// Lazy per-page comment cache.
///
/// FUSE callbacks read from this via the `ReposixFs` instance. Writes happen
/// in two paths:
/// 1. `lookup(Bucket, "<id>.comments")` → allocates a `CommentsDir` inode
///    (does NOT fetch comments yet — an empty dir inode is cheap).
/// 2. `readdir(CommentsDir)` / `lookup(CommentsDir, name)` → triggers a
///    one-shot `ConfluenceBackend::list_comments(page_id)` call, renders
///    each comment, populates `by_page` + `file_ino_to_page`.
///
/// Uses `DashMap` throughout so FUSE callback threads (fuser workers) can
/// populate without a global write lock.
#[derive(Debug, Default)]
pub struct CommentsSnapshot {
    /// `page_ino` → (fetched: bool, `Vec<CommentEntry>`).
    ///
    /// `fetched = true` means we've successfully called `list_comments` for
    /// this page at least once (entries may still be empty for pages with no
    /// comments). `fetched = false` means the dir inode is allocated but no
    /// `list_comments` call has completed yet.
    by_page: DashMap<u64, (bool, Vec<CommentEntry>)>,
    /// `page_ino` → `CommentsDir` inode (the dir materialized on first `lookup`).
    page_to_dir_ino: DashMap<u64, u64>,
    /// `CommentsDir` inode → `page_ino` (reverse map for getattr/readdir).
    dir_ino_to_page: DashMap<u64, u64>,
    /// `CommentFile` inode → `page_ino` (for read/getattr dispatch).
    file_ino_to_page: DashMap<u64, u64>,
    /// Monotonic allocator for `CommentsDir` inodes.
    next_dir_ino: AtomicU64,
    /// Monotonic allocator for `CommentFile` inodes.
    next_file_ino: AtomicU64,
}

impl CommentsSnapshot {
    /// Create an empty snapshot with allocators pinned to the declared bases.
    #[must_use]
    pub fn new() -> Self {
        Self {
            by_page: DashMap::new(),
            page_to_dir_ino: DashMap::new(),
            dir_ino_to_page: DashMap::new(),
            file_ino_to_page: DashMap::new(),
            next_dir_ino: AtomicU64::new(COMMENTS_DIR_INO_BASE),
            next_file_ino: AtomicU64::new(COMMENTS_FILE_INO_BASE),
        }
    }

    /// Allocate (or return existing) `CommentsDir` inode for `page_ino`.
    ///
    /// Idempotent: the second call for the same `page_ino` returns the same inode.
    /// Does NOT fetch comments — populates only the dir ↔ page maps.
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

    /// Look up the `page_ino` for a given `CommentsDir` inode.
    #[must_use]
    pub fn page_of_dir(&self, dir_ino: u64) -> Option<u64> {
        self.dir_ino_to_page.get(&dir_ino).map(|v| *v)
    }

    /// Look up the `page_ino` for a given `CommentFile` inode.
    #[must_use]
    pub fn page_of_file(&self, file_ino: u64) -> Option<u64> {
        self.file_ino_to_page.get(&file_ino).map(|v| *v)
    }

    /// Has this page's comments been fetched yet?
    #[must_use]
    pub fn is_fetched(&self, page_ino: u64) -> bool {
        self.by_page.get(&page_ino).is_some_and(|e| e.0)
    }

    /// Install rendered entries for a page. Overwrites any prior state
    /// (so `refresh_issues` can invalidate). Allocates file inodes.
    pub fn install_entries(
        &self,
        page_ino: u64,
        entries: Vec<(String, Vec<u8>)>, // (filename, rendered)
    ) {
        let mut with_inos: Vec<CommentEntry> = Vec::with_capacity(entries.len());
        for (filename, rendered) in entries {
            let file_ino = self.next_file_ino.fetch_add(1, Ordering::SeqCst);
            self.file_ino_to_page.insert(file_ino, page_ino);
            with_inos.push(CommentEntry {
                file_ino,
                filename,
                rendered,
            });
        }
        self.by_page.insert(page_ino, (true, with_inos));
    }

    /// Borrow the entries for a page (read-only view for `readdir`).
    ///
    /// Returns `None` if the dir inode was never allocated OR if the
    /// page exists but its entries have not been fetched yet.
    #[must_use]
    pub fn entries_if_fetched(&self, page_ino: u64) -> Option<Vec<CommentEntry>> {
        self.by_page.get(&page_ino).and_then(|e| {
            if e.0 { Some(e.1.clone()) } else { None }
        })
    }

    /// Look up a single comment entry by file inode.
    #[must_use]
    pub fn entry_by_file_ino(&self, file_ino: u64) -> Option<CommentEntry> {
        let page_ino = self.page_of_file(file_ino)?;
        let entries = self.by_page.get(&page_ino)?;
        entries.1.iter().find(|e| e.file_ino == file_ino).cloned()
    }
}

/// Validate that `s` contains only ASCII digits (WR-02 pattern).
/// Returns true for "123", false for "", "abc", "1/2", "../etc".
fn is_numeric_ascii(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
}

/// Render a `ConfComment` as a YAML-frontmatter Markdown file.
///
/// Returns `None` if `comment.id` or `comment.parent_comment_id` (when
/// present) is not numeric-ASCII — this defends against path-traversal
/// via filename construction (T-23-03-02).
///
/// Frontmatter fields: `id`, `page_id`, `author`, `created_at`,
/// `updated_at`, `resolved`, `parent_comment_id`, `kind`.
///
/// Body appears AFTER a closing `---\n\n` fence so attacker-influenced
/// body text cannot inject YAML fields (T-23-03-01).
#[must_use]
pub fn render_comment_file(comment: &reposix_confluence::ConfComment) -> Option<Vec<u8>> {
    if !is_numeric_ascii(&comment.id) {
        tracing::warn!(
            comment_id_preview = %comment.id.chars().take(16).collect::<String>(),
            "skipping comment with non-numeric id"
        );
        return None;
    }
    if let Some(pid) = comment.parent_comment_id.as_deref() {
        if !is_numeric_ascii(pid) {
            tracing::warn!(
                comment_id = %comment.id,
                parent_id_preview = %pid.chars().take(16).collect::<String>(),
                "skipping comment with non-numeric parent_comment_id"
            );
            return None;
        }
    }
    // For footer comments there is no resolution concept → always resolved: false.
    // For inline comments: status "open" → false; anything else → true.
    let resolved = match comment.kind {
        reposix_confluence::CommentKind::Footer => false,
        reposix_confluence::CommentKind::Inline => {
            matches!(comment.resolution_status.as_deref(), Some(s) if s != "open")
        }
    };
    let mut out = String::new();
    // Frontmatter — every value is either a controlled literal or a
    // validated numeric string; the author_id (opaque Atlassian accountId)
    // and dates go through YAML-safe quoting.
    out.push_str("---\n");
    let _ = writeln!(out, "id: \"{}\"", comment.id);
    let _ = writeln!(out, "page_id: \"{}\"", comment.page_id);
    // author_id is an opaque accountId string. Escape double-quotes + backslashes
    // to prevent YAML string injection.
    let author_escaped = comment.version.author_id.replace('\\', "\\\\").replace('"', "\\\"");
    let _ = writeln!(out, "author: \"{author_escaped}\"");
    let _ = writeln!(out, "created_at: {}", comment.version.created_at.to_rfc3339());
    let _ = writeln!(out, "updated_at: {}", comment.version.created_at.to_rfc3339());
    let _ = writeln!(out, "resolved: {resolved}");
    match &comment.parent_comment_id {
        Some(pid) => {
            let _ = writeln!(out, "parent_comment_id: \"{pid}\"");
        }
        None => out.push_str("parent_comment_id: null\n"),
    }
    let _ = writeln!(out, "kind: {}", comment.kind.as_str());
    out.push_str("---\n\n");
    // Body — tainted content. body_markdown already handles ADF failure
    // with a safe empty-string fallback.
    out.push_str(&comment.body_markdown());
    Some(out.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use reposix_confluence::{CommentKind, ConfComment, ConfCommentVersion};

    fn make_comment(id: &str, kind: CommentKind, resolved: Option<&str>, parent: Option<&str>) -> ConfComment {
        ConfComment {
            id: id.to_owned(),
            page_id: "98765".to_owned(),
            version: ConfCommentVersion {
                created_at: chrono::Utc.with_ymd_and_hms(2026, 1, 15, 10, 30, 0).unwrap(),
                author_id: "user-a".to_owned(),
                number: 1,
            },
            parent_comment_id: parent.map(str::to_owned),
            resolution_status: resolved.map(str::to_owned),
            body: None,
            kind,
        }
    }

    #[test]
    fn render_comment_file_emits_yaml_frontmatter_for_inline() {
        let c = make_comment("123", CommentKind::Inline, Some("open"), None);
        let bytes = render_comment_file(&c).expect("valid id → Some");
        let s = String::from_utf8(bytes).unwrap();
        assert!(s.starts_with("---\n"), "must begin with frontmatter fence: {s}");
        assert!(s.contains("kind: inline"), "missing kind: {s}");
        assert!(s.contains("resolved: false"), "open → false: {s}");
        assert!(s.contains("parent_comment_id: null"), "null for top-level: {s}");
        assert!(s.contains("id: \"123\""));
        assert!(s.contains("page_id: \"98765\""));
    }

    #[test]
    fn render_comment_file_emits_resolved_true_when_closed() {
        let c = make_comment("1", CommentKind::Inline, Some("resolved"), None);
        let bytes = render_comment_file(&c).expect("ok");
        let s = String::from_utf8(bytes).unwrap();
        assert!(s.contains("resolved: true"), "resolved → true: {s}");
    }

    #[test]
    fn render_comment_file_footer_always_resolved_false() {
        let c = make_comment("2", CommentKind::Footer, None, None);
        let bytes = render_comment_file(&c).expect("ok");
        let s = String::from_utf8(bytes).unwrap();
        assert!(s.contains("kind: footer"));
        assert!(s.contains("resolved: false"), "footer never resolved");
    }

    #[test]
    fn render_comment_file_body_after_frontmatter_fence() {
        // Even if body contains ---, it is after the closing fence
        // and cannot re-open the YAML block.
        let c = make_comment("3", CommentKind::Inline, Some("open"), None);
        // Force-set a body that contains ---. Can't construct ConfPageBody
        // in a test without exposing internals; instead rely on body_markdown
        // returning "" for None and check fence structure.
        let bytes = render_comment_file(&c).expect("ok");
        let s = String::from_utf8(bytes).unwrap();
        // Must have exactly: leading `---\n`, several frontmatter lines,
        // then `---\n\n`, then (empty) body.
        let closing_idx = s.find("---\n\n").expect("must have closing fence");
        assert!(closing_idx > 5, "closing fence must be after first fence");
        // Everything after the closing fence is body; for None body, it's empty.
        assert_eq!(&s[closing_idx + "---\n\n".len()..], "");
    }

    #[test]
    fn render_comment_file_rejects_non_numeric_id() {
        let c = make_comment("../../etc/passwd", CommentKind::Inline, Some("open"), None);
        assert!(render_comment_file(&c).is_none(), "non-numeric id must be rejected");
    }

    #[test]
    fn render_comment_file_rejects_non_numeric_parent_id() {
        let c = make_comment("1", CommentKind::Inline, Some("open"), Some("../sibling"));
        assert!(render_comment_file(&c).is_none(), "non-numeric parent_id must be rejected");
    }

    #[test]
    fn comment_ino_allocation_is_monotonic_and_disjoint() {
        let snap = CommentsSnapshot::new();
        let d1 = snap.ensure_dir(100);
        let d2 = snap.ensure_dir(200);
        let d_dup = snap.ensure_dir(100); // idempotent
        assert_eq!(d1, d_dup, "ensure_dir is idempotent per page_ino");
        assert_ne!(d1, d2);
        assert!((COMMENTS_DIR_INO_BASE..COMMENTS_FILE_INO_BASE).contains(&d1));
        assert!((COMMENTS_DIR_INO_BASE..COMMENTS_FILE_INO_BASE).contains(&d2));

        snap.install_entries(100, vec![
            ("111.md".to_owned(), b"body1".to_vec()),
            ("112.md".to_owned(), b"body2".to_vec()),
        ]);
        let entries = snap.entries_if_fetched(100).expect("fetched");
        assert_eq!(entries.len(), 2);
        for e in &entries {
            assert!(e.file_ino >= COMMENTS_FILE_INO_BASE);
        }
        // Disjoint: all dirs < all files
        for e in &entries {
            assert!(e.file_ino > d1, "file inode must be > dir inodes");
            assert!(e.file_ino > d2);
        }
    }

    #[test]
    fn ensure_dir_does_not_mark_fetched() {
        let snap = CommentsSnapshot::new();
        let _ = snap.ensure_dir(42);
        assert!(!snap.is_fetched(42), "ensure_dir alone must not mark fetched");
        assert!(snap.entries_if_fetched(42).is_none());
    }

    #[test]
    fn install_entries_marks_fetched() {
        let snap = CommentsSnapshot::new();
        let _ = snap.ensure_dir(42);
        snap.install_entries(42, vec![]);
        assert!(snap.is_fetched(42), "install_entries marks fetched even when empty");
        let entries = snap.entries_if_fetched(42).expect("fetched");
        assert!(entries.is_empty());
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn comment_ino_constants_ordering() {
        assert!(
            crate::inode::COMMENTS_DIR_INO_BASE > crate::inode::LABELS_SYMLINK_INO_BASE,
            "COMMENTS_DIR_INO_BASE must be > LABELS_SYMLINK_INO_BASE"
        );
        assert!(
            crate::inode::COMMENTS_DIR_INO_BASE < crate::inode::COMMENTS_FILE_INO_BASE,
            "COMMENTS_DIR_INO_BASE must be < COMMENTS_FILE_INO_BASE"
        );
    }
}
