//! Tree-diff planner + SG-02 bulk-delete cap.
//!
//! Given the prior issue list (from `GET /projects/.../issues`) and a parsed
//! fast-export stream, compute the per-issue actions to apply. The cap is
//! enforced BEFORE any action is returned for execution, so a refused push
//! never makes a single HTTP call.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, HashMap};

use reposix_core::frontmatter;
use reposix_core::{Issue, RecordId};
use thiserror::Error;

use crate::fast_import::ParsedExport;

/// SG-02 cap.
pub const BULK_DELETE_LIMIT: usize = 5;

/// SG-02 override tag accepted in the fast-import commit message.
pub const ALLOW_BULK_DELETE_TAG: &str = "[allow-bulk-delete]";

/// Per-issue intent computed from the diff.
#[derive(Debug)]
#[allow(dead_code)] // `prior_version` is read by the executor only on Update path
pub enum PlannedAction {
    /// Path appeared in new tree but not the prior list — POST.
    Create(Issue),
    /// Both prior and new bytes exist; bytes differ — PATCH with prior version.
    Update {
        /// Issue id (server-known).
        id: RecordId,
        /// Prior version (for `If-Match`).
        prior_version: u64,
        /// New issue parsed from the new blob bytes.
        new: Issue,
    },
    /// Path was in prior list but not in the new tree — DELETE candidate.
    Delete {
        /// Issue id (server-known).
        id: RecordId,
        /// Prior version (informational; DELETE is unconditional).
        prior_version: u64,
    },
}

/// Bulk-delete cap rejection.
#[derive(Debug, Error)]
pub enum PlanError {
    /// More than [`BULK_DELETE_LIMIT`] deletes without the override tag.
    #[error(
        "refusing to push (would delete {count} issues; cap is {limit}; commit message tag '{tag}' overrides)"
    )]
    BulkDeleteRefused {
        /// Number of deletes that would have been issued.
        count: usize,
        /// The configured cap.
        limit: usize,
        /// The override tag the commit message must contain.
        tag: &'static str,
    },
    /// A new-tree blob failed frontmatter parse — push is rejected.
    #[error("invalid issue blob at path {path}: {source}")]
    InvalidBlob {
        /// Tree path that failed parsing.
        path: String,
        /// Underlying parser error.
        #[source]
        source: reposix_core::Error,
    },
}

fn issue_id_from_path(path: &str) -> Option<u64> {
    let stem = path.strip_suffix(".md")?;
    stem.parse::<u64>().ok()
}

/// Canonicalize a rendered blob for semantic-equivalence comparison
/// (M-03). Normalizes `\r\n` → `\n` (CRLF from git's normalization) and
/// trims trailing whitespace on the final byte sequence (absorbs
/// blob-ends-with-or-without-LF variation). Intra-body whitespace is
/// preserved — we only want to canonicalize envelope noise, not content.
fn normalize_for_compare(s: &str) -> String {
    s.replace("\r\n", "\n").trim_end().to_owned()
}

/// Compute the per-issue actions for the new tree. Enforces the SG-02
/// bulk-delete cap unless the commit message contains
/// [`ALLOW_BULK_DELETE_TAG`].
///
/// Returns the actions in deterministic order: Creates, then Updates, then
/// Deletes. (A rename-shaped delete-then-create still works because each
/// Issue id is independent.)
///
/// # Errors
/// - [`PlanError::BulkDeleteRefused`] if the cap fires.
/// - [`PlanError::InvalidBlob`] if a new-tree blob can't parse as an issue.
pub fn plan(prior: &[Issue], parsed: &ParsedExport) -> Result<Vec<PlannedAction>, PlanError> {
    let prior_by_id: HashMap<RecordId, &Issue> = prior.iter().map(|i| (i.id, i)).collect();
    let prior_by_path: BTreeMap<String, RecordId> = prior
        .iter()
        .map(|i| (format!("{:04}.md", i.id.0), i.id))
        .collect();

    let mut creates: Vec<PlannedAction> = Vec::new();
    let mut updates: Vec<PlannedAction> = Vec::new();
    let mut deletes: Vec<PlannedAction> = Vec::new();

    // Walk the new tree.
    for (path, mark) in &parsed.tree {
        let Some(bytes) = parsed.blobs.get(mark) else {
            // mark unresolved — skip
            continue;
        };
        let text = String::from_utf8_lossy(bytes);
        let new_issue = frontmatter::parse(&text).map_err(|e| PlanError::InvalidBlob {
            path: path.clone(),
            source: e,
        })?;
        match prior_by_path.get(path).copied() {
            Some(id) => {
                let prior_issue = prior_by_id.get(&id).copied();
                // Normalized-compare (M-03): render BOTH sides through
                // `frontmatter::render`, normalize line endings to LF and
                // trim trailing whitespace, then compare. Byte-for-byte
                // compare against the raw new blob emitted a spurious
                // PATCH on every push for unchanged content, because git's
                // trailing-newline + CRLF handling (and any future YAML-
                // serializer quirks) can diverge from `render`'s output
                // even when the Issue is semantically identical. Rendering
                // both sides + stripping line-ending noise funnels them
                // through the same canonicalization, so "no edits" pushes
                // emit zero Update actions — matching the user's mental
                // model.
                let equivalent = if let Some(p) = prior_issue {
                    let prior_rendered =
                        frontmatter::render(p).map_err(|e| PlanError::InvalidBlob {
                            path: path.clone(),
                            source: e,
                        })?;
                    let new_rendered =
                        frontmatter::render(&new_issue).map_err(|e| PlanError::InvalidBlob {
                            path: path.clone(),
                            source: e,
                        })?;
                    normalize_for_compare(&prior_rendered) == normalize_for_compare(&new_rendered)
                } else {
                    false
                };
                if !equivalent {
                    let prior_version = prior_issue.map_or(0, |p| p.version);
                    updates.push(PlannedAction::Update {
                        id,
                        prior_version,
                        new: new_issue,
                    });
                }
            }
            None => {
                creates.push(PlannedAction::Create(new_issue));
            }
        }
    }

    // Walk prior to find deletes.
    for (path, &id) in &prior_by_path {
        if !parsed.tree.contains_key(path) {
            // also tolerate explicit `D <path>` lines but the result is the same.
            let prior_version = prior_by_id.get(&id).map_or(0, |p| p.version);
            deletes.push(PlannedAction::Delete { id, prior_version });
        }
    }
    // Also honor explicit `D <path>` lines (in case the new tree omitted
    // doesn't cover them — git fast-export typically uses one or the other).
    for path in &parsed.deletes {
        if let Some(id) = issue_id_from_path(path).map(RecordId) {
            // Only add if not already counted.
            if !deletes
                .iter()
                .any(|a| matches!(a, PlannedAction::Delete { id: x, .. } if *x == id))
                && prior_by_id.contains_key(&id)
            {
                let prior_version = prior_by_id.get(&id).map_or(0, |p| p.version);
                deletes.push(PlannedAction::Delete { id, prior_version });
            }
        }
    }

    let delete_count = deletes.len();
    if delete_count > BULK_DELETE_LIMIT && !parsed.commit_message.contains(ALLOW_BULK_DELETE_TAG) {
        return Err(PlanError::BulkDeleteRefused {
            count: delete_count,
            limit: BULK_DELETE_LIMIT,
            tag: ALLOW_BULK_DELETE_TAG,
        });
    }

    let mut actions = Vec::with_capacity(creates.len() + updates.len() + deletes.len());
    actions.extend(creates);
    actions.extend(updates);
    actions.extend(deletes);
    Ok(actions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use reposix_core::IssueStatus;

    fn sample(id: u64) -> Issue {
        let t = chrono::Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
        Issue {
            id: RecordId(id),
            title: format!("issue {id}"),
            status: IssueStatus::Open,
            assignee: None,
            labels: vec![],
            created_at: t,
            updated_at: t,
            version: 1,
            body: "body".to_owned(),
            parent_id: None,
            extensions: std::collections::BTreeMap::new(),
        }
    }

    #[test]
    fn five_deletes_passes_cap() {
        let prior: Vec<Issue> = (1..=5).map(sample).collect();
        let parsed = ParsedExport::default(); // empty new tree
        let actions = plan(&prior, &parsed).expect("5 deletes is at the cap");
        assert_eq!(actions.len(), 5);
    }

    #[test]
    fn six_deletes_fires_cap() {
        let prior: Vec<Issue> = (1..=6).map(sample).collect();
        let parsed = ParsedExport::default();
        let err = plan(&prior, &parsed).expect_err("6 deletes must be refused");
        assert!(matches!(
            err,
            PlanError::BulkDeleteRefused {
                count: 6,
                limit: 5,
                ..
            }
        ));
    }

    #[test]
    fn six_deletes_with_allow_tag_passes() {
        let prior: Vec<Issue> = (1..=6).map(sample).collect();
        let parsed = ParsedExport {
            commit_message: "[allow-bulk-delete] cleanup\n".to_owned(),
            ..Default::default()
        };
        let actions = plan(&prior, &parsed).expect("override tag must bypass cap");
        assert_eq!(actions.len(), 6);
    }

    /// M-03: An unchanged push (user pulled, made no edits, pushed back)
    /// must emit zero Update actions. This exercises the normalized-compare
    /// path in `plan()` — rendering both sides via `frontmatter::render`
    /// canonicalizes away trailing-newline and YAML-serializer quirks that
    /// a raw byte-compare would flag as spurious Updates.
    #[test]
    fn unchanged_push_emits_no_patches() {
        let prior: Vec<Issue> = (1..=3).map(sample).collect();
        // Simulate `git pull` → no edits → `git push`:
        // each new-tree blob is the exact render output of the prior issue.
        // The planner must recognize these as equivalent and skip PATCH.
        let mut blobs: HashMap<u64, Vec<u8>> = HashMap::new();
        let mut tree: BTreeMap<String, u64> = BTreeMap::new();
        for (i, issue) in prior.iter().enumerate() {
            let mark = u64::try_from(i).unwrap() + 1;
            let rendered = frontmatter::render(issue).expect("render sample");
            blobs.insert(mark, rendered.into_bytes());
            tree.insert(format!("{:04}.md", issue.id.0), mark);
        }
        let parsed = ParsedExport {
            commit_message: "no-op push\n".to_owned(),
            blobs,
            tree,
            deletes: vec![],
        };
        let actions = plan(&prior, &parsed).expect("unchanged push must plan clean");
        assert_eq!(
            actions.len(),
            0,
            "unchanged push should emit ZERO actions; got: {actions:?}"
        );
    }

    /// M-03: Even when the new blob has an EXTRA trailing `\n` (as git's
    /// normalization sometimes produces after a round-trip through the
    /// working tree), the normalized-compare must treat it as a no-op.
    #[test]
    fn extra_trailing_newline_is_a_noop() {
        let prior: Vec<Issue> = vec![sample(1)];
        let mut rendered = frontmatter::render(&prior[0]).expect("render");
        rendered.push('\n'); // the exact edge case M-03 calls out
        let mut blobs = HashMap::new();
        blobs.insert(1, rendered.into_bytes());
        let mut tree = BTreeMap::new();
        tree.insert("0001.md".to_owned(), 1);
        let parsed = ParsedExport {
            commit_message: "noop\n".to_owned(),
            blobs,
            tree,
            deletes: vec![],
        };
        let actions = plan(&prior, &parsed).expect("plan");
        assert_eq!(
            actions.len(),
            0,
            "trailing-newline variation must be a no-op; got: {actions:?}"
        );
    }
}
