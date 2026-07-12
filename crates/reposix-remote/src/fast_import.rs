//! `git fast-import` / `git fast-export` stream emit + parse for the narrow
//! one-file-per-issue tree shape this helper uses.
//!
//! Shared invariant: blobs are produced via `reposix_core::issue::frontmatter::render`
//! — the SAME function the cache materializer uses — so SHAs are stable across
//! `git push` and `git cat-file blob <oid>` for unchanged content.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, HashMap};
use std::io::{self, BufRead, Write};

use reposix_core::frontmatter;
use reposix_core::Record;

/// Render an [`Record`] to the byte sequence we publish as a fast-import blob.
/// Single-line wrapper around [`frontmatter::render`] so any future change to
/// the renderer surfaces here as a compile-or-test signal.
///
/// # Errors
/// Propagates any error from [`frontmatter::render`] (YAML serialization).
pub(crate) fn render_blob(issue: &Record) -> Result<String, reposix_core::Error> {
    frontmatter::render(issue)
}

/// The client's current tracking tip, resolved by the caller from
/// `refs/reposix/origin/main` before an import batch (RBF-LR-03).
///
/// Both fields are hex object ids the caller obtains via `git rev-parse`
/// (`refs/reposix/origin/main` and `…^{{tree}}`). Carrying the tree oid
/// alongside the commit oid lets [`emit_import_stream`] run the no-op
/// tree-equality guard WITHOUT re-shelling git — keeping the emit function
/// pure and unit-testable.
///
/// The commit oid is a static-ref lookup (`git rev-parse --verify
/// refs/reposix/origin/main`), never a remote-influenced byte, so no
/// `Tainted<T>` routing concern applies (OP-2; PLAN §2 taint note).
#[derive(Debug, Clone)]
pub(crate) struct ImportParent {
    /// Hex oid of the current `refs/reposix/origin/main` commit.
    pub(crate) commit: String,
    /// Hex oid of that commit's tree (`<commit>^{tree}`).
    pub(crate) tree: String,
}

/// Compute the git tree oid of the snapshot `sorted` would produce, WITHOUT
/// writing anything to an object store. Mirrors the exact two-level tree shape
/// the cache builder emits (`reposix-cache::builder::build_from`): an inner
/// `<bucket>/` subtree of `<id>.md` blobs, nested under a single-entry outer
/// tree — so the oid is byte-for-byte comparable with `git rev-parse
/// <parent>^{{tree}}` on a ref that a prior [`emit_import_stream`] created.
///
/// `sorted` must already be id-sorted by the caller; tree entries are
/// re-sorted by filename bytes here (git's canonical tree ordering, which
/// differs from numeric id order once ids reach two digits).
fn snapshot_tree_oid(sorted: &[&Record], bucket: &str) -> io::Result<gix::ObjectId> {
    use gix::objs::WriteTo;
    let hash_kind = gix::hash::Kind::Sha1;
    let mut entries: Vec<gix::objs::tree::Entry> = Vec::with_capacity(sorted.len());
    for issue in sorted {
        let bytes = render_blob(issue)
            .map_err(|e| io::Error::other(format!("render blob {}: {e}", issue.id.0)))?
            .into_bytes();
        let oid = gix::objs::compute_hash(hash_kind, gix::object::Kind::Blob, &bytes)
            .map_err(|e| io::Error::other(format!("hash blob {}: {e}", issue.id.0)))?;
        entries.push(gix::objs::tree::Entry {
            mode: gix::object::tree::EntryKind::Blob.into(),
            filename: reposix_core::path::record_filename(issue.id.0).into(),
            oid,
        });
    }
    // git orders tree entries by raw filename bytes, NOT by numeric id.
    entries.sort_by(|a, b| a.filename.cmp(&b.filename));

    let inner = gix::objs::Tree { entries };
    let mut inner_buf = Vec::new();
    inner
        .write_to(&mut inner_buf)
        .map_err(|e| io::Error::other(format!("serialize inner tree: {e}")))?;
    let inner_oid = gix::objs::compute_hash(hash_kind, gix::object::Kind::Tree, &inner_buf)
        .map_err(|e| io::Error::other(format!("hash inner tree: {e}")))?;

    let outer = gix::objs::Tree {
        entries: vec![gix::objs::tree::Entry {
            mode: gix::object::tree::EntryKind::Tree.into(),
            filename: bucket.as_bytes().into(),
            oid: inner_oid,
        }],
    };
    let mut outer_buf = Vec::new();
    outer
        .write_to(&mut outer_buf)
        .map_err(|e| io::Error::other(format!("serialize outer tree: {e}")))?;
    gix::objs::compute_hash(hash_kind, gix::object::Kind::Tree, &outer_buf)
        .map_err(|e| io::Error::other(format!("hash outer tree: {e}")))
}

/// Emit a fast-import stream for `issues` to `w`. Issues are sorted by id ASC
/// so the resulting commit's tree is deterministic and SHA-stable.
///
/// `bucket` is the backend's canonical record bucket per
/// [`reposix_core::path::bucket_for_backend`] (`"issues"` or `"pages"`).
///
/// `parent` is the client's current `refs/reposix/origin/main` tip
/// (RBF-LR-03). When `Some`, the synthesized "Sync from REST snapshot" commit
/// chains onto it via a `from <commit>` directive, so every fetch is a
/// fast-forward and `git pull --rebase && git push` reconciles cleanly. When
/// `None` (first fetch, ref absent) the commit is a parentless root — the
/// original seed behavior.
///
/// **No-op guard.** With a parent, an *unchanged* `SoT` would otherwise mint a
/// fresh empty commit every fetch, growing the ref unboundedly. When the
/// freshly-built snapshot tree equals `parent.tree`, we emit a reset-only
/// stream (`reset` + `from <parent>`, NO commit block) so the fetch leaves the
/// ref exactly where it was — mirroring the export-side no-op detection
/// (`saw_commit`).
///
/// **Namespace (RBF-LR-03 layer-2).** All ref writes here target the helper's
/// PRIVATE import namespace `refs/reposix-import/*`, disjoint from the user
/// tracking namespace `refs/reposix/origin/*`. git fetch maps the private ns
/// into the tracking ns via the caller's `remote.origin.fetch`, remaining the
/// sole writer of `refs/reposix/origin/main` — this restores the two-namespace
/// remote-helper contract and eliminates the fetch-time `cannot lock ref`
/// double-writer. The `parent` oid is still read from the *tracking* ref (the
/// last-fetched tip, the correct chain source).
///
/// # Errors
/// Returns any [`io::Error`] from the writer or any rendering/hashing failure
/// translated into an `io::Error::Other`.
pub(crate) fn emit_import_stream<W: Write>(
    w: &mut W,
    issues: &[Record],
    bucket: &str,
    parent: Option<&ImportParent>,
) -> io::Result<()> {
    // First line of every import response: `feature done\n` per
    // gitremote-helpers(7); marks the stream as well-formed.
    writeln!(w, "feature done")?;

    let mut sorted: Vec<&Record> = issues.iter().collect();
    sorted.sort_by_key(|i| i.id.0);

    // No-op guard (RBF-LR-03): if the snapshot tree is identical to the
    // parent commit's tree, emit a reset-only stream that keeps the ref at
    // `<parent>` — no blobs, no commit, so the fetch is a true no-op and the
    // ref cannot grow. Only reachable with a parent (first-fetch seeds always
    // emit a real commit).
    if let Some(p) = parent {
        let new_tree = snapshot_tree_oid(&sorted, bucket)?;
        if new_tree.to_hex().to_string() == p.tree {
            writeln!(w, "reset refs/reposix-import/main")?;
            writeln!(w, "from {}", p.commit)?;
            writeln!(w, "done")?;
            return Ok(());
        }
    }

    let mut mark: u64 = 0;
    let mut blob_marks: Vec<u64> = Vec::with_capacity(sorted.len());
    for issue in &sorted {
        mark += 1;
        let bytes = render_blob(issue)
            .map_err(|e| io::Error::other(format!("render blob {}: {e}", issue.id.0)))?;
        writeln!(w, "blob")?;
        writeln!(w, "mark :{mark}")?;
        writeln!(w, "data {}", bytes.len())?;
        w.write_all(bytes.as_bytes())?;
        writeln!(w)?;
        blob_marks.push(mark);
    }

    mark += 1;
    let commit_mark = mark;
    // RBF-LR-03 layer-2: write the helper's PRIVATE import namespace
    // (`refs/reposix-import/*`), NOT the user tracking ref
    // (`refs/reposix/origin/*`). git fetch maps the private ns into the
    // tracking ns via the caller's `remote.origin.fetch`, so git fetch is the
    // SOLE writer of `refs/reposix/origin/main`. Writing the tracking ref
    // directly here made the helper AND git fetch both update it in one fetch:
    // the stream fast-forwarded it T0→T1 underneath git, then git's post-import
    // ref transaction (expected-old = pre-fetch T0) failed against the
    // already-moved T1 → `cannot lock ref … is at T1 but expected T0`, aborting
    // `git pull --rebase`. NEVER name `refs/heads/*` here — that is the caller's
    // real working branch (`git fast-import --refspec` does not exist on git
    // 2.25, so no remap protects it → catastrophic clobber). Two-namespace
    // remote-helper contract, same shape as git-remote-hg/cinnabar.
    writeln!(w, "commit refs/reposix-import/main")?;
    writeln!(w, "mark :{commit_mark}")?;
    writeln!(w, "committer reposix-helper <bot@reposix> 0 +0000")?;
    let msg = "Sync from REST snapshot\n";
    writeln!(w, "data {}", msg.len())?;
    w.write_all(msg.as_bytes())?;
    // Chain onto the current tracking tip so the fetch is a fast-forward
    // (RBF-LR-03). `from` MUST sit after the commit-message `data` block and
    // before the `M`/`D` tree directives per the git-fast-import(1) commit
    // grammar. Absent on the first-fetch seed (parentless root).
    if let Some(p) = parent {
        writeln!(w, "from {}", p.commit)?;
    }
    for (i, issue) in sorted.iter().enumerate() {
        // Canonical `<bucket>/<id>.md` path (QL-001 / D91-01, bucket-aware
        // per Wave-5.5) — matches the cache/stateless-connect production
        // read path so a fetch→edit→push loop round-trips without churn.
        writeln!(
            w,
            "M 100644 :{} {}",
            blob_marks[i],
            reposix_core::path::record_path(bucket, issue.id.0)
        )?;
    }

    writeln!(w, "done")?;
    Ok(())
}

/// Parsed shape of the fast-export stream git pipes us during `export`.
#[derive(Debug, Default)]
pub(crate) struct ParsedExport {
    /// The commit message body (between `data N` and the next directive).
    pub(crate) commit_message: String,
    /// Mark → blob bytes.
    pub(crate) blobs: HashMap<u64, Vec<u8>>,
    /// Path → mark for `M 100644 :MARK <path>` entries in the new tree.
    pub(crate) tree: BTreeMap<String, u64>,
    /// Paths explicitly removed via `D <path>` lines.
    pub(crate) deletes: Vec<String>,
    /// `true` iff the stream carried at least one `commit ...` directive.
    ///
    /// CRITICAL SAFETY SIGNAL (litmus REOPEN mass-delete BLOCKER). git's
    /// remote-helper `export` re-invokes the helper on EVERY `git push`,
    /// even when the local ref already matches what git tracks in
    /// `refs/reposix/*` — because our `list for-push` answers `?` (remote
    /// value unknown), git can never decide the ref is up-to-date and
    /// re-runs export unconditionally. On a no-new-commit push git emits a
    /// stream with NO `commit` directive at all — just:
    /// `feature done` / `reset refs/heads/main` / `from 000…000` / `done`.
    /// That is NOT an empty *tree* (which would arrive as a `commit` with
    /// zero `M`/explicit `D` lines and be governed by the SG-02 cap); it is
    /// the ABSENCE of any commit. The diff planner MUST refuse to diff such
    /// a stream — treating "no commit parsed" as "user deleted every
    /// record" mass-deleted a live Confluence space on a second push
    /// (audit: 3 real DELETEs at 21:44 in transcript 2026-07-04T21-36-37Z).
    pub(crate) saw_commit: bool,
}

/// Parse a git fast-export stream from `r`, narrowed to the one-file-per-issue
/// tree shape this helper supports.
///
/// # Errors
/// Returns any [`io::Error`] from the reader or any malformed stream
/// translated into `io::Error::Other`.
#[allow(clippy::too_many_lines)] // narrow parser; readability beats split fns here
pub(crate) fn parse_export_stream<R: BufRead>(r: &mut R) -> io::Result<ParsedExport> {
    let mut out = ParsedExport::default();
    let mut current_blob_mark: Option<u64> = None;
    // True after a `commit ...` directive — the next `data N` is the commit
    // message (not a blob payload). Reset to false at the start of each
    // `blob` block.
    let mut in_commit = false;
    loop {
        let mut line = String::new();
        let n = r.read_line(&mut line)?;
        if n == 0 {
            break;
        }
        // strip newline
        if line.ends_with('\n') {
            line.pop();
            if line.ends_with('\r') {
                line.pop();
            }
        }
        if line.is_empty() {
            continue;
        }
        if line == "done" {
            break;
        }
        if line.starts_with("feature ")
            || line.starts_with("option ")
            || line.starts_with("progress ")
            || line.starts_with("checkpoint")
            || line.starts_with("reset ")
            || line.starts_with("from ")
            || line.starts_with("author ")
            || line.starts_with("committer ")
            || line.starts_with("tagger ")
            || line.starts_with("encoding ")
            || line.starts_with("original-oid ")
        {
            continue;
        }
        if line == "blob" {
            current_blob_mark = None;
            in_commit = false;
            continue;
        }
        if let Some(rest) = line.strip_prefix("mark :") {
            let mark: u64 = rest
                .parse()
                .map_err(|e| io::Error::other(format!("bad mark `{rest}`: {e}")))?;
            // A `mark :N` after `commit ...` is the commit's mark, not a
            // blob mark — only record it as a blob mark when we're outside
            // a commit block.
            if !in_commit {
                current_blob_mark = Some(mark);
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("data ") {
            let len: usize = rest
                .parse()
                .map_err(|e| io::Error::other(format!("bad data len `{rest}`: {e}")))?;
            let mut buf = vec![0u8; len];
            r.read_exact(&mut buf)?;
            // git may or may not emit a trailing LF after the `data` payload.
            // Peek exactly ONE byte and consume it only when it is an LF.
            //
            // BUG-3 (QL-001): the previous `read_line` consumed the entire
            // following line, not a single byte. For a blob payload that was
            // harmless (git always follows blob data with a bare LF). But git
            // fast-export emits the commit-MESSAGE payload immediately followed
            // by the first `M 100644 :N issues/<id>.md` directive with NO
            // separating LF — so `read_line` swallowed that first M-line,
            // dropping the lowest-id record from the parsed tree and
            // classifying it as a spurious Delete. Peeking one byte via
            // `fill_buf`/`consume` never crosses a directive boundary.
            let has_trailing_lf = matches!(r.fill_buf()?.first(), Some(&b'\n'));
            if has_trailing_lf {
                r.consume(1);
            }
            if in_commit {
                // Commit message body.
                out.commit_message = String::from_utf8_lossy(&buf).into_owned();
            } else if let Some(mark) = current_blob_mark.take() {
                out.blobs.insert(mark, buf);
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("commit ") {
            // The commit's ref name; we don't act on it.
            let _ = rest;
            current_blob_mark = None;
            in_commit = true;
            // Record that a real commit was present in the stream. The
            // planner keys the no-op-vs-delete-all decision on this: a
            // stream with zero `commit` directives (no-new-commit push)
            // must NEVER be diffed as an empty tree (mass-delete BLOCKER).
            out.saw_commit = true;
            continue;
        }
        if let Some(rest) = line.strip_prefix("M 100644 :") {
            // `:<MARK> <path>`
            let (mark_str, path) = rest
                .split_once(' ')
                .ok_or_else(|| io::Error::other(format!("bad M line `{line}`")))?;
            let mark: u64 = mark_str
                .parse()
                .map_err(|e| io::Error::other(format!("bad M mark `{mark_str}`: {e}")))?;
            out.tree.insert(path.to_owned(), mark);
            continue;
        }
        if let Some(rest) = line.strip_prefix("D ") {
            out.deletes.push(rest.to_owned());
        }
        // Unknown directives we tolerate silently (e.g. `tag`, `note`). Real
        // git fast-export with a sensible refspec should never emit these for
        // the tree shape we maintain.
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use reposix_core::{RecordId, RecordStatus};
    use std::io::Cursor;

    /// Minimal deterministic record fixture. Fixed timestamps keep the
    /// rendered blob — and therefore the tree/commit oids — stable across
    /// runs so the round-trip test's fast-forward assertion is reproducible.
    fn rec(id: u64, body: &str) -> Record {
        use chrono::TimeZone;
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
            body: body.to_owned(),
            parent_id: None,
            extensions: std::collections::BTreeMap::new(),
        }
    }

    /// Render an `emit_import_stream` to a UTF-8 String for line assertions.
    fn emit_to_string(issues: &[Record], parent: Option<&ImportParent>) -> String {
        let mut buf: Vec<u8> = Vec::new();
        emit_import_stream(&mut buf, issues, "issues", parent).expect("emit");
        String::from_utf8(buf).expect("utf8 stream")
    }

    /// RBF-LR-03: when the tracking ref exists, the synthesized commit MUST
    /// carry a `from <parent-commit>` directive so the fetch is a
    /// fast-forward. The `from` line must sit inside the commit block (after
    /// the commit-message `data`, before the `M` tree directives).
    #[test]
    // test-name-honesty: ok — asserts the emitted stream contains `from <oid>` in the commit block
    fn emit_import_stream_with_parent_emits_from_line() {
        let parent = ImportParent {
            commit: "1234567890123456789012345678901234567890".to_owned(),
            // A tree oid that will NOT match the snapshot, so the no-op guard
            // does not fire and a real commit is emitted.
            tree: "0000000000000000000000000000000000000000".to_owned(),
        };
        let out = emit_to_string(&[rec(1, "body v1")], Some(&parent));
        assert!(
            out.contains(&format!("from {}", parent.commit)),
            "with a parent the commit must chain via `from <oid>`; stream=\n{out}"
        );
        // Ordering: `from` after `data <n>` (commit message), before the `M` line.
        let from_pos = out.find("from ").expect("from line present");
        let m_pos = out.find("\nM 100644 ").expect("M line present");
        assert!(
            from_pos < m_pos,
            "`from` must precede the tree `M` directives; stream=\n{out}"
        );
        assert!(out.contains("commit refs/reposix-import/main"));
        assert_no_collapsed_namespace(&out);
    }

    /// RBF-LR-03 layer-2 regression guard (clobber / namespace-collapse). The
    /// emitted import stream MUST NEVER name the user tracking ref
    /// `refs/reposix/origin/main` (that collapses the two-namespace contract →
    /// fetch-time `cannot lock ref` double-writer) nor the caller's real
    /// working branch `refs/heads/main` (git 2.25 has no
    /// `fast-import --refspec` remap → catastrophic clobber) as a
    /// `commit`/`reset` target. The helper writes ONLY the private
    /// `refs/reposix-import/*` namespace.
    fn assert_no_collapsed_namespace(out: &str) {
        for line in out.lines() {
            if let Some(target) = line
                .strip_prefix("commit ")
                .or_else(|| line.strip_prefix("reset "))
            {
                assert_ne!(
                    target, "refs/reposix/origin/main",
                    "import stream must NOT write the user tracking ref (namespace collapse → `cannot lock ref`); stream=\n{out}"
                );
                assert_ne!(
                    target, "refs/heads/main",
                    "import stream must NOT name the caller's working branch (clobber risk); stream=\n{out}"
                );
            }
        }
    }

    /// The first-fetch seed path (ref absent → `None`) must remain a
    /// PARENTLESS root commit — no `from` directive — exactly as before
    /// RBF-LR-03, so a fresh clone still bootstraps.
    #[test]
    // test-name-honesty: ok — asserts the None-parent stream carries no `from` directive
    fn emit_import_stream_no_parent_is_parentless() {
        let out = emit_to_string(&[rec(1, "body v1")], None);
        assert!(
            !out.contains("\nfrom "),
            "the parentless seed must not emit a `from` line; stream=\n{out}"
        );
        assert!(out.contains("commit refs/reposix-import/main"));
        assert_no_collapsed_namespace(&out);
    }

    /// No-op guard: when the snapshot tree equals the parent commit's tree,
    /// emit a reset-only stream (`reset` + `from <parent>`, NO `commit`) so
    /// the ref stays put and cannot grow unboundedly on repeated no-op fetches.
    #[test]
    // test-name-honesty: ok — asserts an unchanged snapshot emits reset-only, no commit block
    fn unchanged_tree_emits_no_commit() {
        let records = [rec(1, "stable body"), rec(2, "another")];
        let sorted: Vec<&Record> = records.iter().collect();
        // The parent tree oid IS the snapshot's own tree oid → guard fires.
        let tree = snapshot_tree_oid(&sorted, "issues")
            .expect("tree oid")
            .to_hex()
            .to_string();
        let parent = ImportParent {
            commit: "abcabcabcabcabcabcabcabcabcabcabcabcabca".to_owned(),
            tree,
        };
        let out = emit_to_string(&records, Some(&parent));
        assert!(
            !out.contains("commit refs/reposix-import/main"),
            "an unchanged tree must emit NO commit block; stream=\n{out}"
        );
        assert!(
            out.contains("reset refs/reposix-import/main"),
            "no-op guard must emit a `reset` directive; stream=\n{out}"
        );
        assert_no_collapsed_namespace(&out);
        assert!(
            out.contains(&format!("from {}", parent.commit)),
            "reset must pin the ref at the parent via `from`; stream=\n{out}"
        );
        assert!(
            !out.contains("\nblob\n"),
            "no-op stream must not carry blobs; stream=\n{out}"
        );
    }

    /// Port of `repro/verify-from-parent-fix.sh` against a real `git
    /// fast-import`: a changed snapshot emitted WITH `from <tip>` fast-forwards
    /// the seeded tracking ref (exit 0, linear 2-commit history); the same
    /// changed snapshot emitted PARENTLESS is refused (`does not contain`, ref
    /// unchanged) — proving the parent chaining is what makes recovery work.
    #[test]
    // test-name-honesty: ok — drives real `git fast-import`, asserts fast-forward vs reject on the ref
    fn git_fast_import_roundtrip_with_parent_fast_forwards() {
        use std::process::{Command, Stdio};

        // git 2.25 selects the `import` path unaided; this test drives
        // `git fast-import` directly so it is git-version-agnostic.
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path();
        let git = |args: &[&str]| -> std::process::Output {
            Command::new("git")
                .args(args)
                .current_dir(path)
                .output()
                .expect("spawn git")
        };
        assert!(git(&["init", "-q"]).status.success(), "git init");

        let feed = |stream: &[u8]| -> std::process::Output {
            let mut child = Command::new("git")
                .args(["fast-import", "--quiet"])
                .current_dir(path)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("spawn fast-import");
            child
                .stdin
                .take()
                .unwrap()
                .write_all(stream)
                .expect("write stream");
            child.wait_with_output().expect("wait fast-import")
        };
        let rev_parse = |arg: &str| -> String {
            let out = git(&["rev-parse", arg]);
            assert!(out.status.success(), "rev-parse {arg}");
            String::from_utf8_lossy(&out.stdout).trim().to_owned()
        };

        // 1. Seed a parentless "Sync from REST snapshot" tracking commit.
        let mut seed = Vec::new();
        emit_import_stream(&mut seed, &[rec(1, "v1")], "issues", None).expect("seed emit");
        assert!(feed(&seed).status.success(), "seed fast-import");
        let tip0 = rev_parse("refs/reposix-import/main");
        let tree0 = rev_parse("refs/reposix-import/main^{tree}");

        // 2. Changed snapshot, PARENTLESS → git refuses the non-descendant tip.
        let mut bad = Vec::new();
        emit_import_stream(&mut bad, &[rec(1, "v2-CHANGED")], "issues", None)
            .expect("parentless emit");
        let bad_out = feed(&bad);
        let bad_stderr = String::from_utf8_lossy(&bad_out.stderr);
        assert!(
            bad_stderr.contains("does not contain"),
            "parentless changed snapshot must be refused with `does not contain`; stderr=\n{bad_stderr}"
        );
        assert_eq!(
            rev_parse("refs/reposix-import/main"),
            tip0,
            "refused import must leave the ref unchanged"
        );

        // 3. Changed snapshot WITH from <tip0> → fast-forward, linear history.
        let parent = ImportParent {
            commit: tip0.clone(),
            tree: tree0,
        };
        let mut fixed = Vec::new();
        emit_import_stream(&mut fixed, &[rec(1, "v2-CHANGED")], "issues", Some(&parent))
            .expect("with-parent emit");
        let fixed_out = feed(&fixed);
        assert!(
            fixed_out.status.success(),
            "with-parent changed snapshot must fast-forward; stderr=\n{}",
            String::from_utf8_lossy(&fixed_out.stderr)
        );
        let tip1 = rev_parse("refs/reposix-import/main");
        assert_ne!(tip1, tip0, "ref must advance to the new descendant commit");
        // Parent of tip1 is tip0 → linear 2-commit chain.
        assert_eq!(
            rev_parse("refs/reposix-import/main~1"),
            tip0,
            "the new commit must chain directly onto the seed tip"
        );
    }

    /// QL-001 criterion 2 / BUG-3: git fast-export emits the commit-MESSAGE
    /// `data N` payload immediately followed by the first `M` directive with
    /// NO separating LF. The old `read_line`-based trailing-newline consume
    /// swallowed that entire first M-line, dropping the lowest-id record from
    /// `parsed.tree` and classifying it as a spurious Delete. The peek-one-
    /// byte fix must retain `issues/1.md` in the tree.
    #[test]
    fn commit_message_without_trailing_lf_does_not_swallow_first_m_line() {
        let blob = b"---\nid: 1\n---\nbody\n";
        let msg = b"edit issue 1 (no trailing LF before M)";
        let mut stream: Vec<u8> = Vec::new();
        writeln!(&mut stream, "feature done").unwrap();
        writeln!(&mut stream, "blob").unwrap();
        writeln!(&mut stream, "mark :1").unwrap();
        writeln!(&mut stream, "data {}", blob.len()).unwrap();
        stream.extend_from_slice(blob);
        stream.push(b'\n'); // blob payloads ARE followed by a bare LF
        writeln!(&mut stream, "commit refs/heads/main").unwrap();
        writeln!(&mut stream, "mark :2").unwrap();
        writeln!(&mut stream, "committer test <t@t> 0 +0000").unwrap();
        writeln!(&mut stream, "data {}", msg.len()).unwrap();
        stream.extend_from_slice(msg); // NO trailing LF — the fast-export shape
        writeln!(&mut stream, "M 100644 :1 issues/1.md").unwrap();
        writeln!(&mut stream, "done").unwrap();

        let mut cur = Cursor::new(stream);
        let parsed = parse_export_stream(&mut cur).expect("parse");

        assert!(
            parsed.tree.contains_key("issues/1.md"),
            "BUG-3: first M-line after commit message must survive; tree={:?}",
            parsed.tree
        );
        assert_eq!(parsed.tree.get("issues/1.md"), Some(&1));
        assert!(
            parsed.blobs.contains_key(&1),
            "blob mark 1 must be captured"
        );
        assert_eq!(parsed.commit_message.as_bytes(), msg);
    }

    /// The blob-data path (which IS followed by a bare LF) must still consume
    /// exactly that one LF and align to the next directive — the peek-one-byte
    /// fix must not regress the common case.
    #[test]
    fn blob_data_trailing_lf_is_consumed() {
        let blob = b"hello";
        let mut stream: Vec<u8> = Vec::new();
        writeln!(&mut stream, "feature done").unwrap();
        writeln!(&mut stream, "blob").unwrap();
        writeln!(&mut stream, "mark :7").unwrap();
        writeln!(&mut stream, "data {}", blob.len()).unwrap();
        stream.extend_from_slice(blob);
        stream.push(b'\n');
        writeln!(&mut stream, "done").unwrap();

        let mut cur = Cursor::new(stream);
        let parsed = parse_export_stream(&mut cur).expect("parse");
        assert_eq!(parsed.blobs.get(&7).map(Vec::as_slice), Some(&blob[..]));
    }

    /// Litmus REOPEN: a no-new-commit push emits `reset`/`from` with NO
    /// `commit` directive. `saw_commit` must be `false` so the planner
    /// treats it as a no-op rather than a delete-all. Bytes are the literal
    /// capture from a local sim reproduction of the second-push mass-delete.
    #[test]
    fn reset_from_without_commit_sets_saw_commit_false() {
        let stream = b"feature done\nreset refs/heads/main\nfrom 0000000000000000000000000000000000000000\ndone\n";
        let mut cur = Cursor::new(&stream[..]);
        let parsed = parse_export_stream(&mut cur).expect("parse");
        assert!(
            !parsed.saw_commit,
            "a reset/from stream with no `commit` line must have saw_commit=false"
        );
        assert!(parsed.tree.is_empty(), "no M lines → empty tree");
        assert!(parsed.deletes.is_empty(), "no D lines → no deletes");
    }

    /// The positive half of the contract: a stream that DOES carry a
    /// `commit` directive sets `saw_commit = true`, so a genuine
    /// emptied-tree bulk delete still reaches the SG-02 cap.
    #[test]
    fn commit_directive_sets_saw_commit_true() {
        let msg = b"delete all";
        let mut stream: Vec<u8> = Vec::new();
        writeln!(&mut stream, "feature done").unwrap();
        writeln!(&mut stream, "commit refs/heads/main").unwrap();
        writeln!(&mut stream, "mark :1").unwrap();
        writeln!(&mut stream, "committer test <t@t> 0 +0000").unwrap();
        writeln!(&mut stream, "data {}", msg.len()).unwrap();
        stream.extend_from_slice(msg);
        stream.push(b'\n');
        writeln!(&mut stream, "done").unwrap();
        let mut cur = Cursor::new(stream);
        let parsed = parse_export_stream(&mut cur).expect("parse");
        assert!(
            parsed.saw_commit,
            "a stream with a `commit` directive must set saw_commit=true"
        );
    }
}
