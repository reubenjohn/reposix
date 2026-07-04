//! Tree-diff planner + SG-02 bulk-delete cap.
//!
//! Given the prior issue list (from `GET /projects/.../issues`) and a parsed
//! fast-export stream, compute the per-issue actions to apply. The cap is
//! enforced BEFORE any action is returned for execution, so a refused push
//! never makes a single HTTP call.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, HashMap, HashSet};

use reposix_core::frontmatter;
use reposix_core::path::record_id_from_path;
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
    /// Two tree paths resolve to the same record id — ambiguous intent,
    /// push is rejected (e.g. `issues/42.md` + `pages/42.md`, or
    /// `issues/42.md` + `issues/0042.md` after padding collapse).
    #[error(
        "two paths in the pushed tree resolve to the same record id {id} \
         ({first} and {second}); remove one and retry"
    )]
    DuplicateRecordId {
        /// The colliding record id.
        id: u64,
        /// First path seen for this id.
        first: String,
        /// Second path seen for this id.
        second: String,
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
/// - [`PlanError::DuplicateRecordId`] if two tree paths resolve to one id.
pub(crate) fn plan(
    prior: &[Record],
    parsed: &ParsedExport,
) -> Result<Vec<PlannedAction>, PlanError> {
    // SAFETY GUARD (litmus REOPEN second-push mass-delete BLOCKER).
    //
    // git re-invokes the `export` helper on EVERY push (our `list
    // for-push` answers `?`, so git can never decide the ref is
    // up-to-date). On a no-new-commit push git emits a stream with NO
    // `commit` directive — `feature done` / `reset refs/heads/main` /
    // `from 000…000` / `done`. That parses to an empty `ParsedExport`.
    //
    // Without this guard the empty tree walks `prior` and plans a DELETE
    // for EVERY record — the exact mass-delete that trashed a live
    // Confluence space on a second push (3 real DELETEs, audit
    // 2026-07-04T21:44).
    //
    // The distinction is semantic, not size-based:
    //   - `saw_commit == false` → git exported NOTHING to apply. No-op.
    //   - `saw_commit == true` + empty tree → the user genuinely emptied
    //     the tree, committed, and pushed. That IS a real bulk delete and
    //     is (correctly) governed by the SG-02 cap below.
    //
    // Keying on the presence of a `commit` directive — not on tree
    // emptiness — is what separates "nothing to push" from "delete
    // everything." Never conflate the two.
    if !parsed.saw_commit {
        return Ok(Vec::new());
    }

    let prior_by_id: HashMap<RecordId, &Record> = prior.iter().map(|i| (i.id, i)).collect();

    // Bucket-aware matching (Wave-5.5, confluence mass-delete BLOCKER):
    // prior records and tree entries are matched by RECORD ID, never by path
    // string. The id is extracted via the shared bucket-aware parser
    // (`record_id_from_path`, sanctioned buckets: issues/ + pages/), so the
    // tree's bucket spelling can never cause a record to be misclassified as
    // Create+Delete — the failure mode that mass-deleted a live Confluence
    // space when a `pages/`-shaped tree hit an `issues/`-keyed planner.

    // BUG-2 (QL-001): the export stream can legitimately carry a path both
    // as an `M` add and a later `D` delete (git fast-export emits this for
    // some rewrite shapes). Deletes win, keyed by id: an id named by a `D`
    // line must NOT be treated as live, or a delete-then-re-add would
    // spuriously survive as a Create/Update.
    let deleted_ids: HashSet<RecordId> = parsed
        .deletes
        .iter()
        .filter_map(|p| record_id_from_path(p))
        .map(RecordId)
        .collect();

    let mut creates: Vec<PlannedAction> = Vec::new();
    let mut updates: Vec<PlannedAction> = Vec::new();
    let mut deletes: Vec<PlannedAction> = Vec::new();

    // Ids present as live records in the new tree (for the delete walk),
    // plus the first path seen per id (for duplicate detection).
    let mut live_tree_ids: BTreeMap<RecordId, &str> = BTreeMap::new();

    // Walk the new tree.
    for (path, mark) in &parsed.tree {
        // BUG-2 (QL-001): only `<sanctioned-bucket>/<id>.md` paths are
        // records. `reposix refresh` writes `.reposix/fetched_at.txt` +
        // `.reposix/.gitignore` into the same tree; without this filter
        // `frontmatter::parse` rejects them as invalid blobs and the whole
        // push fails. Non-record paths are silently ignored.
        let Some(id_num) = record_id_from_path(path) else {
            continue;
        };
        let id = RecordId(id_num);
        // Two paths resolving to one id (cross-bucket or padding collision)
        // is ambiguous intent — refuse loudly rather than guess.
        if let Some(first) = live_tree_ids.get(&id) {
            return Err(PlanError::DuplicateRecordId {
                id: id_num,
                first: (*first).to_owned(),
                second: path.clone(),
            });
        }
        live_tree_ids.insert(id, path.as_str());
        // Deletes win over adds for an id appearing in both (see above).
        if deleted_ids.contains(&id) {
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
        match prior_by_id.get(&id).copied() {
            Some(prior_issue) => {
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
                let prior_rendered =
                    frontmatter::render(prior_issue).map_err(|e| PlanError::InvalidBlob {
                        path: path.clone(),
                        source: e,
                    })?;
                let new_rendered =
                    frontmatter::render(&new_issue).map_err(|e| PlanError::InvalidBlob {
                        path: path.clone(),
                        source: e,
                    })?;
                let equivalent =
                    normalize_for_compare(&prior_rendered) == normalize_for_compare(&new_rendered);
                if !equivalent {
                    updates.push(PlannedAction::Update {
                        id,
                        prior_version: prior_issue.version,
                        new: new_issue,
                    });
                }
            }
            None => {
                creates.push(PlannedAction::Create(new_issue));
            }
        }
    }

    // Walk prior to find deletes, keyed by id: a prior record is deleted
    // when its id is absent from the new tree's record ids, OR an explicit
    // `D <path>` line names it (deletes win over adds — see above). Sorted
    // by id for deterministic action order.
    let mut prior_sorted: Vec<&Record> = prior.iter().collect();
    prior_sorted.sort_by_key(|p| p.id);
    for p in prior_sorted {
        if !live_tree_ids.contains_key(&p.id) || deleted_ids.contains(&p.id) {
            deletes.push(PlannedAction::Delete {
                id: p.id,
                prior_version: p.version,
            });
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
        // Genuine emptied-tree commit (saw_commit=true): user removed all
        // files, committed, pushed. This IS a real bulk delete governed by
        // the SG-02 cap — distinct from a no-commit stream (saw_commit=false)
        // which is a no-op (see plan()'s safety guard).
        let parsed = ParsedExport {
            saw_commit: true,
            ..Default::default()
        };
        let actions = plan(&prior, &parsed).expect("5 deletes is at the cap");
        assert_eq!(actions.len(), 5);
    }

    #[test]
    fn six_deletes_fires_cap() {
        let prior: Vec<Record> = (1..=6).map(sample).collect();
        let parsed = ParsedExport {
            saw_commit: true,
            ..Default::default()
        };
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
            saw_commit: true,
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
            saw_commit: true,
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
            saw_commit: true,
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
            saw_commit: true,
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

    // --- Wave-5.5 regressions: `pages/<id>.md` (confluence bucket) ---------
    //
    // The P91 vision-litmus real-run proved a pages/-shaped tree pushed
    // against an issues/-keyed planner mass-deletes the backend (every prior
    // record → Delete, every tree blob → Create). These pin the QL-001
    // criteria for the confluence bucket. LITERAL `pages/<id>.md` strings on
    // purpose (magic-fixture hazard, raise-list §3).

    /// Build a pages/-bucket [`ParsedExport`]: each prior record re-emitted
    /// unchanged under its `pages/<id>.md` path. LITERAL paths on purpose.
    fn pages_noop_export(prior: &[Record]) -> ParsedExport {
        let mut blobs: HashMap<u64, Vec<u8>> = HashMap::new();
        let mut tree: BTreeMap<String, u64> = BTreeMap::new();
        for (i, issue) in prior.iter().enumerate() {
            let mark = u64::try_from(i).unwrap() + 1;
            let rendered = frontmatter::render(issue).expect("render sample");
            blobs.insert(mark, rendered.into_bytes());
            tree.insert(format!("pages/{}.md", issue.id.0), mark);
        }
        ParsedExport {
            commit_message: "no-op pages push\n".to_owned(),
            blobs,
            tree,
            deletes: vec![],
            saw_commit: true,
        }
    }

    /// Wave-5.5 / QL-001 criterion 3 for the pages bucket: a full-tree push
    /// of `pages/<id>.md` records against a matching prior produces ZERO
    /// Deletes and ZERO Creates. Against the pre-fix planner this was the
    /// mass-delete: every prior → Delete (the litmus-observed data loss).
    #[test]
    fn pages_full_tree_push_emits_zero_deletes() {
        let prior: Vec<Record> = (1..=6).map(sample).collect();
        let parsed = pages_noop_export(&prior);
        let actions = plan(&prior, &parsed).expect("pages full-tree push must plan clean");
        assert_eq!(
            actions.len(),
            0,
            "unchanged pages/ full-tree push must be a pure no-op; got: {actions:?}"
        );
    }

    /// Wave-5.5 / QL-001 criteria 1+2 for the pages bucket: one edited
    /// `pages/<id>.md` record round-trips as exactly one Update — no
    /// Creates, no Deletes, no invalid-blob reject.
    #[test]
    fn pages_single_edit_is_one_update() {
        let prior: Vec<Record> = (1..=3).map(sample).collect();
        let mut parsed = pages_noop_export(&prior);
        let mut edited = sample(2);
        edited.body = "a real confluence edit\n".to_owned();
        let rendered = frontmatter::render(&edited).expect("render edited");
        parsed.blobs.insert(2, rendered.into_bytes());
        let actions = plan(&prior, &parsed).expect("pages single edit plans clean");
        assert_eq!(
            actions.len(),
            1,
            "exactly one action expected; got: {actions:?}"
        );
        assert!(
            matches!(&actions[0], PlannedAction::Update { id, .. } if id.0 == 2),
            "expected a single Update for id 2; got: {actions:?}"
        );
    }

    /// Wave-5.5: bucket spelling is irrelevant to matching (id-keyed
    /// planner) — an issues/-shaped tree against the same prior also
    /// round-trips as a no-op. The two buckets are interchangeable for the
    /// CONSUMER side; producers pick per-backend via `bucket_for_backend`.
    #[test]
    fn cross_bucket_tree_still_matches_by_id() {
        let prior: Vec<Record> = (1..=3).map(sample).collect();
        // pages/-keyed prior semantics, issues/-shaped tree: still a no-op.
        let parsed = canonical_noop_export(&prior);
        let actions = plan(&prior, &parsed).expect("cross-bucket push must plan clean");
        assert_eq!(
            actions.len(),
            0,
            "id-keyed matching must ignore bucket spelling; got: {actions:?}"
        );
    }

    /// Wave-5.5: two tree paths resolving to one record id (cross-bucket or
    /// padding collision) is ambiguous — the planner must refuse loudly,
    /// never guess.
    #[test]
    fn duplicate_record_id_across_buckets_is_refused() {
        let prior: Vec<Record> = vec![sample(1)];
        let mut parsed = canonical_noop_export(&prior); // issues/1.md
        let rendered = frontmatter::render(&prior[0]).expect("render");
        parsed.blobs.insert(50, rendered.into_bytes());
        parsed.tree.insert("pages/1.md".to_owned(), 50); // same id, other bucket
        let err = plan(&prior, &parsed).expect_err("duplicate id must be refused");
        assert!(
            matches!(err, PlanError::DuplicateRecordId { id: 1, .. }),
            "expected DuplicateRecordId for id 1; got: {err:?}"
        );
    }

    /// Wave-5.5 composition check: the SG-02 bulk-delete cap still fires on
    /// a genuinely delete-shaped pages/ push (defense-in-depth preserved —
    /// the bucket fix must NOT weaken the cap).
    #[test]
    fn pages_bulk_delete_still_capped() {
        let prior: Vec<Record> = (1..=6).map(sample).collect();
        let parsed = ParsedExport {
            commit_message: "cleanup\n".to_owned(),
            saw_commit: true,
            ..Default::default() // real commit, empty tree → 6 deletes
        };
        let err = plan(&prior, &parsed).expect_err("6 deletes must still be refused");
        assert!(
            matches!(
                err,
                PlanError::BulkDeleteRefused {
                    count: 6,
                    limit: 5,
                    ..
                }
            ),
            "SG-02 cap must still fire; got: {err:?}"
        );
    }

    // --- Litmus REOPEN: second-push mass-delete BLOCKER ---------------------
    //
    // The P91 real-TokenWorld vision litmus mass-deleted a live Confluence
    // space on a SECOND push that carried no new commit. git re-invoked the
    // export helper (our `list for-push` answers `?`, so git can never decide
    // the ref is current), and fast-export emitted a stream with NO `commit`
    // directive. The old planner read that empty parse as "delete every
    // record". These pin the fix at the planner layer AND at the parse+plan
    // boundary using the EXACT bytes git sends (captured from a local sim
    // reproduction: `feature done` / `reset` / `from 000…` / `done`).

    /// Planner layer: a stream that carried NO commit (`saw_commit == false`)
    /// must plan ZERO actions against a non-empty prior — never a delete-all.
    /// RED against pre-fix code (which returned one Delete per prior record).
    #[test]
    fn no_commit_stream_plans_no_actions() {
        let prior: Vec<Record> = (1..=3).map(sample).collect();
        // Empty parse with saw_commit=false — exactly what a no-new-commit
        // push produces. Three prior records, cap is 5, so the SG-02 cap
        // would NOT have caught this (3 <= 5): the guard is the only defense.
        let parsed = ParsedExport::default();
        assert!(!parsed.saw_commit, "sanity: default has no commit");
        let actions = plan(&prior, &parsed).expect("no-commit stream must plan clean");
        assert_eq!(
            actions.len(),
            0,
            "no-commit stream must be a pure no-op, NEVER a mass-delete; got: {actions:?}"
        );
    }

    /// Parse+plan boundary: feed the LITERAL bytes git's remote-helper export
    /// pipes on a no-new-commit push, parse them, then plan against a
    /// 3-record prior. The whole point is that `reset`/`from` WITHOUT a
    /// `commit` must not be diffed as an empty tree. RED against pre-fix code
    /// (3 spurious Deletes — the exact 21:44 incident shape).
    #[test]
    fn no_commit_export_stream_is_a_noop_end_to_end() {
        use crate::fast_import::parse_export_stream;
        use std::io::Cursor;

        // Verbatim capture from the local sim reproduction (repro2.sh).
        let stream = b"feature done\nreset refs/heads/main\nfrom 0000000000000000000000000000000000000000\ndone\n";
        let mut cur = Cursor::new(&stream[..]);
        let parsed = parse_export_stream(&mut cur).expect("parse no-commit stream");
        assert!(
            !parsed.saw_commit,
            "a reset/from stream with no `commit` line must have saw_commit=false"
        );
        assert!(parsed.tree.is_empty(), "no M lines → empty tree");

        let prior: Vec<Record> = (1..=3).map(sample).collect();
        let actions = plan(&prior, &parsed).expect("plan must not error");
        let deletes = actions
            .iter()
            .filter(|a| matches!(a, PlannedAction::Delete { .. }))
            .count();
        assert_eq!(
            deletes, 0,
            "no-new-commit push must issue ZERO deletes (second-push mass-delete BLOCKER); got: {actions:?}"
        );
        assert_eq!(
            actions.len(),
            0,
            "and zero actions overall; got: {actions:?}"
        );
    }

    /// The fix must NOT weaken the legitimate case: a REAL commit that empties
    /// the tree (user `git rm`'d everything, committed, pushed) is still a
    /// bulk delete and still hits the SG-02 cap. `saw_commit=true` + empty
    /// tree + 6 prior → refused. Guards against an over-broad short-circuit.
    #[test]
    fn real_commit_emptying_tree_still_hits_cap() {
        let prior: Vec<Record> = (1..=6).map(sample).collect();
        let parsed = ParsedExport {
            commit_message: "delete all the things\n".to_owned(),
            saw_commit: true, // a REAL commit that happens to have an empty tree
            ..Default::default()
        };
        let err = plan(&prior, &parsed)
            .expect_err("a real emptied-tree commit is a bulk delete → capped");
        assert!(
            matches!(err, PlanError::BulkDeleteRefused { count: 6, .. }),
            "emptied-tree commit must still trip SG-02; got: {err:?}"
        );
    }
}
