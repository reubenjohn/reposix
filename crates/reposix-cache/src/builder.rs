//! Tree construction and (Plan 02) lazy blob materialization.

use chrono::Utc;
use reposix_core::frontmatter;

use crate::cache::Cache;
use crate::error::{Error, Result};

impl Cache {
    /// Sync the tree from the backend and commit to `refs/heads/main`.
    ///
    /// Does NOT materialize blobs — the returned commit references blob
    /// OIDs that are only persisted on demand (Plan 02 `read_blob`).
    ///
    /// Commit message format:
    /// `sync(<backend>:<project>): <N> issues at <ISO8601>`.
    ///
    /// # Errors
    /// - [`Error::Backend`] if `list_issues` fails.
    /// - [`Error::Render`] if frontmatter rendering fails for any issue.
    /// - [`Error::Git`] if any gix operation fails.
    /// - [`Error::Io`] if the `HEAD` file cannot be written.
    pub async fn build_from(&self) -> Result<gix::ObjectId> {
        let issues = self
            .backend
            .list_issues(&self.project)
            .await
            .map_err(|e| Error::Backend(e.to_string()))?;

        // Render each issue, compute the blob OID WITHOUT writing the
        // blob object. The tree references each blob_oid; the blob
        // itself is persisted only when `read_blob(oid)` is called
        // (Plan 02). This is the lazy-blob invariant Phase 32's
        // stateless-connect handler relies on.
        //
        // NOTE: we deliberately bypass `Repository::edit_tree` because
        // its `write()` validates that every referenced object already
        // exists in the object DB (gix 0.82
        // `write_cursor -> MissingObject`). That check is fatal to the
        // lazy-blob invariant: we WANT to write a tree that references
        // blobs we have not persisted. We assemble `gix_object::Tree`
        // manually and call `Repository::write_object`, which does no
        // such validation.
        let hash_kind = self.repo.object_hash();

        // Build entries for the `issues/` subtree: `<id>.md -> blob_oid`.
        // Entry order in `gix_object::Tree.entries` must be the sorted
        // order git expects; gix validates this lazily when the tree is
        // written. We pre-sort by filename to be safe.
        let mut inner_entries: Vec<gix::objs::tree::Entry> = Vec::with_capacity(issues.len());
        for issue in &issues {
            let rendered = frontmatter::render(issue)?;
            let bytes = rendered.into_bytes();
            let oid =
                gix::objs::compute_hash(hash_kind, gix::object::Kind::Blob, &bytes)
                    .map_err(|e| Error::Git(e.to_string()))?;
            let filename = format!("{}.md", issue.id.0);
            inner_entries.push(gix::objs::tree::Entry {
                mode: gix::object::tree::EntryKind::Blob.into(),
                filename: filename.into(),
                oid,
            });
        }
        // Sort by filename — git's tree-entry ordering for plain files
        // is lexicographic by raw bytes.
        inner_entries.sort_by(|a, b| a.filename.cmp(&b.filename));

        let inner_tree = gix::objs::Tree {
            entries: inner_entries,
        };
        let inner_tree_oid = self
            .repo
            .write_object(&inner_tree)
            .map_err(|e| Error::Git(e.to_string()))?
            .detach();

        // Outer tree with one entry: `issues/` -> inner_tree_oid.
        let outer_tree = gix::objs::Tree {
            entries: vec![gix::objs::tree::Entry {
                mode: gix::object::tree::EntryKind::Tree.into(),
                filename: b"issues".as_slice().into(),
                oid: inner_tree_oid,
            }],
        };
        let tree_oid = self
            .repo
            .write_object(&outer_tree)
            .map_err(|e| Error::Git(e.to_string()))?
            .detach();

        // Commit.
        let msg = format!(
            "sync({}:{}): {} issues at {}",
            self.backend_name,
            self.project,
            issues.len(),
            Utc::now().to_rfc3339()
        );
        let commit_oid = self
            .repo
            .commit(
                "refs/heads/main",
                msg,
                tree_oid,
                std::iter::empty::<gix::ObjectId>(),
            )
            .map_err(|e| Error::Git(e.to_string()))?;

        // Explicitly point HEAD at refs/heads/main to defend against
        // `init.defaultBranch = master` leaking in from the user's
        // ~/.gitconfig (RESEARCH §Pitfall 5).
        let head_path = self.path.join("HEAD");
        std::fs::write(&head_path, "ref: refs/heads/main\n")?;

        Ok(commit_oid.detach())
    }
}
