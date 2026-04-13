//! Tree-diff planner + SG-02 bulk-delete cap.
//!
//! Given the prior issue list (from `GET /projects/.../issues`) and a parsed
//! fast-export stream, compute the per-issue actions to apply. The cap is
//! enforced BEFORE any action is returned for execution, so a refused push
//! never makes a single HTTP call.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, HashMap};

use reposix_core::frontmatter;
use reposix_core::{Issue, IssueId};
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
        id: IssueId,
        /// Prior version (for `If-Match`).
        prior_version: u64,
        /// New issue parsed from the new blob bytes.
        new: Issue,
    },
    /// Path was in prior list but not in the new tree — DELETE candidate.
    Delete {
        /// Issue id (server-known).
        id: IssueId,
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
    let prior_by_id: HashMap<IssueId, &Issue> = prior.iter().map(|i| (i.id, i)).collect();
    let prior_by_path: BTreeMap<String, IssueId> = prior
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
                let bytes_match = prior_issue
                    .map(|p| {
                        frontmatter::render(p).map(|s: String| s.as_bytes() == bytes.as_slice())
                    })
                    .transpose()
                    .map_err(|e| PlanError::InvalidBlob {
                        path: path.clone(),
                        source: e,
                    })?
                    .unwrap_or(false);
                if !bytes_match {
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
        if let Some(id) = issue_id_from_path(path).map(IssueId) {
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
            id: IssueId(id),
            title: format!("issue {id}"),
            status: IssueStatus::Open,
            assignee: None,
            labels: vec![],
            created_at: t,
            updated_at: t,
            version: 1,
            body: "body".to_owned(),
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
}
