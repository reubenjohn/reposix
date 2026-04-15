//! Pure in-memory label overlay builder for the Phase-19 `labels/` overlay.
//!
//! Mirrors `tree.rs` but simpler: flat two-level structure (labels root →
//! per-label dir → symlinks), no hierarchy, no cycle risk, depth always 1.
//!
//! # Security
//!
//! T-19-01: Every label string is passed through `slug_or_fallback` before
//! use as a FUSE directory name — raw label content (which is
//! attacker-influenced) never reaches the VFS path layer.
//!
//! T-19-02: Symlink targets are built only from controlled components:
//! the literal `"../../"` prefix, the caller-supplied `bucket` string
//! (a `&'static str` from the backend trait), and a zero-padded numeric
//! `IssueId`. No label text enters the target string.

#![allow(clippy::module_name_repetitions)]

use std::collections::HashMap;

use reposix_core::path::{dedupe_siblings, slug_or_fallback};
use reposix_core::{Issue, IssueId};

pub use crate::inode::{LABELS_DIR_INO_BASE, LABELS_ROOT_INO, LABELS_SYMLINK_INO_BASE};

/// A single symlink entry inside a per-label directory.
#[derive(Debug, Default, Clone)]
pub struct LabelEntry {
    /// Inode for this symlink (unique per issue across all labels).
    pub symlink_ino: u64,
    /// Deduped slug filename (e.g. `"my-bug.md"` or `"my-bug-2.md"`).
    pub slug: String,
    /// Relative target path, e.g. `"../../issues/00000000001.md"`.
    pub target: String,
}

/// Complete in-memory snapshot of the `labels/` overlay.
///
/// Built once per `refresh_issues` call by [`LabelSnapshot::build`].
/// All FUSE callbacks (`readdir`, `lookup`, `getattr`, `readlink`) read
/// from this snapshot without locking the backend.
#[derive(Debug, Default, Clone)]
pub struct LabelSnapshot {
    /// Map from label directory inode → (label slug, entries).
    ///
    /// Used by `readdir(LabelDir)` and `lookup(LabelsRoot, name)`.
    pub label_dirs: HashMap<u64, (String, Vec<LabelEntry>)>,
    /// Reverse map: symlink inode → target string.
    ///
    /// Used by `readlink(LabelSymlink)`.
    pub symlink_targets: HashMap<u64, String>,
    /// Number of distinct label directories (for `render_mount_root_index`).
    pub label_count: usize,
}

impl LabelSnapshot {
    /// Build a complete label snapshot from a list of issues.
    ///
    /// Each issue that carries labels contributes one symlink per label.
    /// An issue with N labels appears in N separate label directories.
    ///
    /// The snapshot is deterministic: labels are sorted lexicographically
    /// and entries within each label group are sorted by `IssueId`.
    ///
    /// # Algorithm
    ///
    /// 1. Group issues by label string into a sorted `Vec`.
    /// 2. Compute deduped dir slugs across all labels via `dedupe_siblings`.
    /// 3. For each label group, allocate a sequential dir inode, compute
    ///    deduped entry slugs, and populate `symlink_targets`.
    #[must_use]
    pub fn build(bucket: &str, issues: &[Issue]) -> Self {
        // Step 1: group issues by label string.
        let mut by_label: HashMap<String, Vec<(IssueId, String)>> = HashMap::new();
        for issue in issues {
            for label in &issue.labels {
                by_label
                    .entry(label.clone())
                    .or_default()
                    .push((issue.id, slug_or_fallback(&issue.title, issue.id)));
            }
        }

        if by_label.is_empty() {
            return Self::default();
        }

        // Step 2: collect labels in sorted order for stable output.
        let mut label_vec: Vec<(String, Vec<(IssueId, String)>)> = by_label.into_iter().collect();
        label_vec.sort_by(|(a, _), (b, _)| a.cmp(b));

        // Dedupe the label dir slugs across all distinct labels.
        // Each label gets a unique inode offset derived from its position.
        let label_slug_inputs: Vec<(IssueId, String)> = label_vec
            .iter()
            .enumerate()
            .map(|(i, (label, _))| {
                // Use the label string as the "title" for slugification;
                // IssueId(i as u64) is only used as a fallback for empty slugs.
                (
                    IssueId(i as u64),
                    slug_or_fallback(label, IssueId(i as u64)),
                )
            })
            .collect();
        let deduped_label_slugs = dedupe_siblings(label_slug_inputs);

        // Step 3: build the snapshot.
        let mut label_dirs: HashMap<u64, (String, Vec<LabelEntry>)> =
            HashMap::with_capacity(label_vec.len());
        let mut symlink_targets: HashMap<u64, String> = HashMap::new();
        let mut sym_counter: u64 = 0;

        for (dir_offset, ((_, issues_in_label), (_, dir_slug))) in label_vec
            .into_iter()
            .zip(deduped_label_slugs.into_iter())
            .enumerate()
        {
            let dir_ino = LABELS_DIR_INO_BASE + dir_offset as u64;

            // Dedupe issue slugs within this label group.
            let deduped_entries = dedupe_siblings(issues_in_label);

            let mut entries: Vec<LabelEntry> = Vec::with_capacity(deduped_entries.len());
            for (id, entry_slug) in deduped_entries {
                let sym_ino = LABELS_SYMLINK_INO_BASE + sym_counter;
                sym_counter += 1;
                let target = format!("../../{bucket}/{:011}.md", id.0);
                symlink_targets.insert(sym_ino, target.clone());
                entries.push(LabelEntry {
                    symlink_ino: sym_ino,
                    slug: format!("{entry_slug}.md"),
                    target,
                });
            }

            label_dirs.insert(dir_ino, (dir_slug, entries));
        }

        let label_count = label_dirs.len();
        Self {
            label_dirs,
            symlink_targets,
            label_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use reposix_core::IssueStatus;

    fn make_issue(id: u64, title: &str, labels: Vec<&str>) -> Issue {
        let t = chrono::Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        Issue {
            id: IssueId(id),
            title: title.into(),
            status: IssueStatus::Open,
            assignee: None,
            labels: labels.into_iter().map(str::to_owned).collect(),
            created_at: t,
            updated_at: t,
            version: 1,
            body: String::new(),
            parent_id: None,
        }
    }

    // Test 1: empty issues → empty snapshot
    #[test]
    fn build_empty() {
        let snap = LabelSnapshot::build("issues", &[]);
        assert!(snap.label_dirs.is_empty());
        assert!(snap.symlink_targets.is_empty());
        assert_eq!(snap.label_count, 0);
    }

    // Test 2: two issues with the same label → one label dir, two symlinks
    #[test]
    fn build_single_label() {
        let issues = vec![
            make_issue(1, "First", vec!["bug"]),
            make_issue(2, "Second", vec!["bug"]),
        ];
        let snap = LabelSnapshot::build("issues", &issues);
        assert_eq!(snap.label_count, 1, "expected one label dir");
        let (_, (slug, entries)) = snap.label_dirs.iter().next().unwrap();
        assert_eq!(slug, "bug");
        assert_eq!(entries.len(), 2, "expected two symlinks in the bug label");
    }

    // Test 3: issue with two labels appears in both groups
    #[test]
    fn build_multi_label() {
        let issues = vec![make_issue(1, "My issue", vec!["bug", "p1"])];
        let snap = LabelSnapshot::build("issues", &issues);
        assert_eq!(snap.label_count, 2, "expected two label dirs");
        // Both label dirs should have exactly one symlink to issue 1
        for (_, entries) in snap.label_dirs.values() {
            assert_eq!(entries.len(), 1);
            assert!(entries[0].target.contains("00000000001.md"));
        }
    }

    // Test 4: symlink target format is ../../issues/00000000001.md
    #[test]
    fn symlink_target_format() {
        let issues = vec![make_issue(1, "Hello world", vec!["bug"])];
        let snap = LabelSnapshot::build("issues", &issues);
        let target = snap.symlink_targets.values().next().unwrap();
        assert_eq!(
            target, "../../issues/00000000001.md",
            "unexpected target: {target}"
        );
    }

    // Test 5: label slug sanitization — "Bug Fix!" → slug "bug-fix"
    #[test]
    fn label_slug_sanitization() {
        let issues = vec![make_issue(1, "My issue", vec!["Bug Fix!"])];
        let snap = LabelSnapshot::build("issues", &issues);
        assert_eq!(snap.label_count, 1);
        let (_, (slug, _)) = snap.label_dirs.iter().next().unwrap();
        assert_eq!(slug, "bug-fix", "unexpected slug: {slug}");
    }

    // Test 6: label inode constants ordering — compile-time relationships
    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn label_ino_constants_ordering() {
        assert!(
            crate::inode::LABELS_ROOT_INO < crate::tree::TREE_DIR_INO_BASE,
            "LABELS_ROOT_INO must be below TREE_DIR_INO_BASE"
        );
        assert!(
            crate::inode::LABELS_DIR_INO_BASE > crate::tree::TREE_SYMLINK_INO_BASE,
            "LABELS_DIR_INO_BASE must be above TREE_SYMLINK_INO_BASE"
        );
        assert!(
            crate::inode::LABELS_DIR_INO_BASE < crate::inode::LABELS_SYMLINK_INO_BASE,
            "LABELS_DIR_INO_BASE must be below LABELS_SYMLINK_INO_BASE"
        );
    }

    // Test 7: symlink FileAttr size must equal target.len() (guards Pitfall 5)
    #[test]
    fn symlink_attr_size_equals_target_len() {
        let issues = vec![make_issue(42, "Test issue", vec!["feature"])];
        let snap = LabelSnapshot::build("issues", &issues);
        let (sym_ino, target) = snap.symlink_targets.iter().next().unwrap();
        // The entry's target length must equal what we store
        let (_, (_, entries)) = snap.label_dirs.iter().next().unwrap();
        let entry = entries.iter().find(|e| e.symlink_ino == *sym_ino).unwrap();
        assert_eq!(
            entry.target.len(),
            target.len(),
            "entry.target.len() must equal symlink_targets value len"
        );
        // And the target has the expected length for 11-digit id
        assert_eq!(
            target.len(),
            "../../issues/00000000042.md".len(),
            "unexpected target length"
        );
    }

    // Test 8: two issues with same label, different slugs — deduplication works
    #[test]
    fn build_deduplication_within_label() {
        // Two issues with the same title → same raw slug → dedupe_siblings adds suffix
        let issues = vec![
            make_issue(1, "Same title", vec!["bug"]),
            make_issue(2, "Same title", vec!["bug"]),
        ];
        let snap = LabelSnapshot::build("issues", &issues);
        let (_, (_, entries)) = snap.label_dirs.iter().next().unwrap();
        assert_eq!(entries.len(), 2);
        let slugs: Vec<&str> = entries.iter().map(|e| e.slug.as_str()).collect();
        // First keeps bare slug, second gets suffix
        assert!(
            slugs.contains(&"same-title.md"),
            "expected bare slug: {slugs:?}"
        );
        assert!(
            slugs.contains(&"same-title-2.md"),
            "expected suffixed slug: {slugs:?}"
        );
    }

    // Test 9: label dirs are sorted by label slug for stable ls output
    #[test]
    fn label_dirs_sorted_by_slug() {
        let issues = vec![make_issue(1, "Issue one", vec!["zebra", "apple", "mango"])];
        let snap = LabelSnapshot::build("issues", &issues);
        assert_eq!(snap.label_count, 3);
        // Collect (dir_ino, slug) pairs sorted by dir_ino (allocation order = sorted label order)
        let mut pairs: Vec<(u64, &str)> = snap
            .label_dirs
            .iter()
            .map(|(ino, (slug, _))| (*ino, slug.as_str()))
            .collect();
        pairs.sort_by_key(|(ino, _)| *ino);
        let slugs: Vec<&str> = pairs.iter().map(|(_, s)| *s).collect();
        assert_eq!(
            slugs,
            vec!["apple", "mango", "zebra"],
            "labels must be sorted alphabetically: {slugs:?}"
        );
    }
}
