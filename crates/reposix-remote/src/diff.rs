//! Tree-diff planner + SG-02 bulk-delete cap.
//!
//! Given the prior issue list (from `GET /projects/.../issues`) and a parsed
//! fast-export stream, compute the per-issue actions to apply. The cap is
//! enforced BEFORE any action is returned for execution, so a refused push
//! never makes a single HTTP call.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, HashMap};

use reposix_core::frontmatter;
use reposix_core::path::{issue_id_from_path, record_path};
use reposix_core::{Record, RecordId};
use thiserror::Error;

use crate::fast_import::ParsedExport;

/// SG-02 cap.
pub(crate) const BULK_DELETE_LIMIT: usize = 5;

/// SG-02 override tag accepted in the fast-import commit message.
pub(crate) const ALLOW_BULK_DELETE_TAG: &str = "[allow-bulk-delete]";

/// Per-issue intent computed from the diff.
#[derive(Debug)]
#[allow(dead_code)] // `prior_version` is read by the executor only on Update path
pub(crate) enum PlannedAction {
    /// Path appeared in new tree but not the prior list — POST.
    Create(Record),
    /// Both prior and new bytes exist; bytes differ — PATCH with prior version.
    Update {
        /// Issue id (server-known).
        id: RecordId,
        /// Prior version (for `If-Match`).
        prior_version: u64,
        /// New issue parsed from the new blob bytes.
        new: Record,
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
pub(crate) enum PlanError {
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
pub(crate) fn plan(
    prior: &[Record],
    parsed: &ParsedExport,
) -> Result<Vec<PlannedAction>, PlanError> {
    let prior_by_id: HashMap<RecordId, &Record> = prior.iter().map(|i| (i.id, i)).collect();
    let prior_by_path: BTreeMap<String, RecordId> =
        prior.iter().map(|i| (record_path(i.id.0), i.id)).collect();

    // BUG-2 (QL-001): the export stream can legitimately carry a path both
    // as an `M` add and a later `D` delete (git fast-export emits this for
    // some rewrite shapes). Deletes win: a path present in `parsed.deletes`
    // must NOT be treated as a live tree entry, or a delete-then-re-add would
    // spuriously survive as a Create/Update.
    let deleted_paths: std::collections::HashSet<&str> =
        parsed.deletes.iter().map(String::as_str).collect();

    let mut creates: Vec<PlannedAction> = Vec::new();
    let mut updates: Vec<PlannedAction> = Vec::new();
    let mut deletes: Vec<PlannedAction> = Vec::new();

    // Walk the new tree.
    for (path, mark) in &parsed.tree {
        // BUG-2 (QL-001): only `issues/<id>.md` paths are records. `reposix
        // refresh` writes `.reposix/fetched_at.txt` + `.reposix/.gitignore`
        // into the same tree; without this filter `frontmatter::parse` rejects
        // them as invalid blobs and the whole push fails. Non-record paths are
        // silently ignored (not parsed, not diffed).
        if issue_id_from_path(path).is_none() {
            continue;
        }
        // Deletes win over adds for a path appearing in both (see above).
        if deleted_paths.contains(path.as_str()) {
            continue;
        }
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
    use reposix_core::RecordStatus;

    fn sample(id: u64) -> Record {
        let t = chrono::Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
        Record {
            id: RecordId(id),
            title: format!("issue {id}"),
            status: RecordStatus::Open,
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
        let prior: Vec<Record> = (1..=5).map(sample).collect();
        let parsed = ParsedExport::default(); // empty new tree
        let actions = plan(&prior, &parsed).expect("5 deletes is at the cap");
        assert_eq!(actions.len(), 5);
    }

    #[test]
    fn six_deletes_fires_cap() {
        let prior: Vec<Record> = (1..=6).map(sample).collect();
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
        let prior: Vec<Record> = (1..=6).map(sample).collect();
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
    // test-name-honesty: ok — unit test of the diff planner's push-plan computation over an in-memory tree, no live git push
    fn unchanged_push_emits_no_patches() {
        let prior: Vec<Record> = (1..=3).map(sample).collect();
        // Simulate `git pull` → no edits → `git push`:
        // each new-tree blob is the exact render output of the prior issue.
        // The planner must recognize these as equivalent and skip PATCH.
        let mut blobs: HashMap<u64, Vec<u8>> = HashMap::new();
        let mut tree: BTreeMap<String, u64> = BTreeMap::new();
        for (i, issue) in prior.iter().enumerate() {
            let mark = u64::try_from(i).unwrap() + 1;
            let rendered = frontmatter::render(issue).expect("render sample");
            blobs.insert(mark, rendered.into_bytes());
            // Canonical `issues/<id>.md` shape — LITERAL, not record_path():
            // a fixture that called the helper would silently follow a
            // regressed helper and mask the bug (raise-list §3 magic-fixture
            // hazard). Hardcoding keeps this RED-if-bug-returns.
            tree.insert(format!("issues/{}.md", issue.id.0), mark);
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
        let prior: Vec<Record> = vec![sample(1)];
        let mut rendered = frontmatter::render(&prior[0]).expect("render");
        rendered.push('\n'); // the exact edge case M-03 calls out
        let mut blobs = HashMap::new();
        blobs.insert(1, rendered.into_bytes());
        let mut tree = BTreeMap::new();
        tree.insert("issues/1.md".to_owned(), 1);
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

    // --- QL-001 regressions: canonical `issues/<id>.md` round-trip ---------
    //
    // These pin the six acceptance criteria at the `plan()` layer (box-
    // independent, no git ≥2.34 required). All fixtures use LITERAL
    // `issues/<id>.md` strings so a regressed `record_path` cannot mask them.

    /// Build a canonical-shape [`ParsedExport`]: each prior record re-emitted
    /// unchanged under its `issues/<id>.md` path. LITERAL paths on purpose.
    fn canonical_noop_export(prior: &[Record]) -> ParsedExport {
        let mut blobs: HashMap<u64, Vec<u8>> = HashMap::new();
        let mut tree: BTreeMap<String, u64> = BTreeMap::new();
        for (i, issue) in prior.iter().enumerate() {
            let mark = u64::try_from(i).unwrap() + 1;
            let rendered = frontmatter::render(issue).expect("render sample");
            blobs.insert(mark, rendered.into_bytes());
            tree.insert(format!("issues/{}.md", issue.id.0), mark);
        }
        ParsedExport {
            commit_message: "no-op push\n".to_owned(),
            blobs,
            tree,
            deletes: vec![],
        }
    }

    /// QL-001 criterion 3: a push of the full seeded tree in the canonical
    /// `issues/<id>.md` shape produces ZERO Deletes (falsifies BUG-1's
    /// create/delete storm). Six records > the bulk-delete cap, so if the
    /// prior-key mismatch returned, `plan()` would either error on the cap
    /// or emit 6 spurious deletes.
    #[test]
    fn full_seeded_tree_push_emits_zero_deletes() {
        let prior: Vec<Record> = (1..=6).map(sample).collect();
        let parsed = canonical_noop_export(&prior);
        let actions = plan(&prior, &parsed).expect("canonical full-tree push must plan clean");
        let deletes = actions
            .iter()
            .filter(|a| matches!(a, PlannedAction::Delete { .. }))
            .count();
        assert_eq!(
            deletes, 0,
            "canonical full-tree push must emit ZERO deletes; got: {actions:?}"
        );
        assert_eq!(
            actions.len(),
            0,
            "unchanged full-tree push must be a pure no-op; got: {actions:?}"
        );
    }

    /// QL-001 criterion 1: a canonical tree that edits exactly one record
    /// round-trips as a single Update (PATCH) — no Creates, no Deletes.
    #[test]
    fn canonical_single_edit_is_one_update() {
        let prior: Vec<Record> = (1..=3).map(sample).collect();
        let mut parsed = canonical_noop_export(&prior);
        // Edit record 2's body: re-render an edited copy under issues/2.md.
        let mut edited = sample(2);
        edited.body = "an actual edit\n".to_owned();
        let rendered = frontmatter::render(&edited).expect("render edited");
        // mark 2 corresponds to prior[1] (id 2) in canonical_noop_export.
        parsed.blobs.insert(2, rendered.into_bytes());
        let actions = plan(&prior, &parsed).expect("single edit plans clean");
        assert_eq!(
            actions.len(),
            1,
            "exactly one action expected; got: {actions:?}"
        );
        match &actions[0] {
            PlannedAction::Update { id, .. } => assert_eq!(id.0, 2, "the edited record is id 2"),
            other => panic!("expected a single Update for id 2, got {other:?}"),
        }
    }

    /// QL-001 criterion 4: a tree carrying `.reposix/` metadata (as
    /// `reposix refresh` writes) pushes without an invalid-blob rejection —
    /// the non-issue paths are filtered out before `frontmatter::parse` (BUG-2).
    #[test]
    fn reposix_metadata_paths_are_ignored_not_rejected() {
        let prior: Vec<Record> = (1..=2).map(sample).collect();
        let mut parsed = canonical_noop_export(&prior);
        // Inject a `.reposix/fetched_at.txt` blob with NON-issue content that
        // would fail frontmatter::parse if the planner tried to parse it.
        parsed.blobs.insert(99, b"2026-07-04T00:00:00Z".to_vec());
        parsed.tree.insert(".reposix/fetched_at.txt".to_owned(), 99);
        let actions = plan(&prior, &parsed)
            .expect("non-issue metadata path must not reject the push (BUG-2)");
        assert_eq!(
            actions.len(),
            0,
            "metadata-only addition is a no-op; got: {actions:?}"
        );
    }

    /// QL-001 BUG-2 (deletes-win): a path present as BOTH an `M` add and a
    /// `D` delete in the same stream yields a net Delete, never a surviving
    /// Create/Update.
    #[test]
    fn delete_wins_over_add_for_same_path() {
        let prior: Vec<Record> = (1..=2).map(sample).collect();
        let mut parsed = canonical_noop_export(&prior);
        // Mark issues/1.md as also deleted in the same stream.
        parsed.deletes.push("issues/1.md".to_owned());
        let actions = plan(&prior, &parsed).expect("delete-wins plans clean");
        // issue 1: delete wins → one Delete. issue 2: unchanged → no action.
        assert_eq!(
            actions.len(),
            1,
            "exactly the net delete survives; got: {actions:?}"
        );
        assert!(
            matches!(&actions[0], PlannedAction::Delete { id, .. } if id.0 == 1),
            "expected a Delete for id 1 (deletes win); got: {actions:?}"
        );
    }
}
