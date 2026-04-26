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

/// Emit a fast-import stream for `issues` to `w`. Issues are sorted by id ASC
/// so the resulting commit's tree is deterministic and SHA-stable.
///
/// # Errors
/// Returns any [`io::Error`] from the writer or any rendering failure
/// translated into an `io::Error::Other`.
pub(crate) fn emit_import_stream<W: Write>(w: &mut W, issues: &[Record]) -> io::Result<()> {
    // First line of every import response: `feature done\n` per
    // gitremote-helpers(7); marks the stream as well-formed.
    writeln!(w, "feature done")?;

    let mut sorted: Vec<&Record> = issues.iter().collect();
    sorted.sort_by_key(|i| i.id.0);

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
    writeln!(w, "commit refs/reposix/origin/main")?;
    writeln!(w, "mark :{commit_mark}")?;
    writeln!(w, "committer reposix-helper <bot@reposix> 0 +0000")?;
    let msg = "Sync from REST snapshot\n";
    writeln!(w, "data {}", msg.len())?;
    w.write_all(msg.as_bytes())?;
    for (i, issue) in sorted.iter().enumerate() {
        writeln!(w, "M 100644 :{} {:04}.md", blob_marks[i], issue.id.0)?;
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
            // git may or may not emit a trailing LF after the data; consume
            // one if present so the next line read aligns to the next
            // directive.
            let mut maybe_nl = String::new();
            let _ = r.read_line(&mut maybe_nl);
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
