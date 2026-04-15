//! FUSE [`Filesystem`] implementation — read path (Phase 3, Phase 10
//! rewire) + write path (Phase S) + Phase 13 nested layout (Wave C).
//!
//! Read-only callbacks: `init`, `getattr`, `lookup`, `readdir`, `read`,
//! `readlink`. Write callbacks (Phase S): `setattr`, `write`, `flush`,
//! `release`, `create`, `unlink`. Every other callback uses fuser's default
//! (`ENOSYS`). When `MountConfig::read_only` is true (set at mount time),
//! the filesystem is mounted with `MountOption::RO` so the kernel refuses
//! writes at the VFS layer before they ever reach our callbacks.
//!
//! # Phase 13 layout (Wave C)
//!
//! The mount root now exposes three synthesized entries (`ls mount/`):
//!
//! - `.gitignore` — read-only file with the 7-byte content `/tree/\n`.
//!   Inode [`GITIGNORE_INO`].
//! - `<bucket>/` — the per-backend root collection directory, keyed from
//!   `IssueBackend::root_collection_name()` (`"issues"` for sim + GitHub,
//!   `"pages"` for Confluence). Inode [`BUCKET_DIR_INO`]. Contains the
//!   real `<padded-id>.md` files (11-digit zero-padded).
//! - `tree/` — synthesized read-only symlink overlay. Inode
//!   [`TREE_ROOT_INO`]. Only emitted when
//!   `backend.supports(BackendFeature::Hierarchy)` is `true` OR when
//!   any loaded issue has `parent_id.is_some()`. Populated from
//!   [`TreeSnapshot`] (Wave B2).
//!
//! The old flat root (`mount/<padded-id>.md`) is removed.
//!
//! # Inode-kind dispatch
//!
//! Each callback starts with an [`InodeKind::classify`] call that partitions
//! the u64 inode space into fixed synthetic slots, dynamic real-file inodes,
//! tree-dir inodes, and tree-symlink inodes — no map lookups needed to know
//! which branch to take. See `crates/reposix-fuse/src/inode.rs` for the
//! numeric layout.
//!
//! # Backend seam (Phase 10 + Phase 14)
//!
//! Both read and write paths speak to a `dyn IssueBackend` trait object
//! rather than any backend's REST shape directly. Reads use
//! `list_issues` / `get_issue`; writes (`release`, `create`) use
//! `update_issue` / `create_issue`. A Phase 14 refactor lifted the write
//! path onto the trait — the simulator's wire shape now lives only in
//! `SimBackend` (in `reposix-core`).
//!
//! # Async bridging
//!
//! FUSE callbacks are synchronous methods. We own a Tokio runtime on the
//! struct and `runtime.block_on(...)` the per-callback HTTP work. Because
//! FUSE callbacks live on fuser's own worker threads (NOT the Tokio runtime
//! threads), `block_on` is deadlock-safe — it blocks the fuser worker, not
//! a Tokio executor.
//!
//! # Timeouts (SG-07)
//!
//! Every backend call is wrapped in a 5-second `tokio::time::timeout`
//! inside `list_issues_with_timeout` / `get_issue_with_timeout` /
//! `update_issue_with_timeout` / `create_issue_with_timeout`. On timeout
//! we reply `libc::EIO` so the kernel never hangs on a dead backend.
//!
//! # Egress discipline (SG-03)
//!
//! Every PATCH / POST body goes through
//! `Tainted::new(parsed_issue).then(sanitize(...))` before it reaches the
//! backend. The `Untainted<Issue>` type, combined with `SimBackend`'s
//! `render_patch_body` / `render_create_body` emitting only the mutable-
//! field subset, guarantees server-controlled fields
//! (`id`/`version`/`created_at`/`updated_at`) physically cannot appear in
//! the wire bytes.

use std::ffi::OsStr;
use std::io;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

use dashmap::DashMap;
use fuser::{
    FileAttr, FileHandle, FileType, Filesystem, INodeNo, ReplyAttr, ReplyCreate, ReplyData,
    ReplyDirectory, ReplyEmpty, ReplyEntry, ReplyWrite, Request,
};
use reposix_core::path::validate_issue_filename;
use reposix_core::{
    backend::BackendFeature, frontmatter, sanitize, Issue, IssueBackend, IssueId, IssueStatus,
    ServerMetadata, Tainted, Untainted,
};
use serde::Deserialize;
use thiserror::Error;
use tokio::runtime::Runtime;
use tracing::warn;

use crate::inode::{
    InodeRegistry, BUCKET_DIR_INO, BUCKET_INDEX_INO, FIRST_ISSUE_INODE, GITIGNORE_INO,
    ROOT_INDEX_INO, ROOT_INO, TREE_INDEX_ALLOC_END, TREE_INDEX_ALLOC_START, TREE_ROOT_INO,
};
use crate::tree::{TreeSnapshot, TREE_DIR_INO_BASE, TREE_SYMLINK_INO_BASE};

/// Exact bytes served for `mount/.gitignore`. Const; no runtime input.
/// The trailing newline is mandatory per POSIX text-file convention and
/// matches what tools like `git check-ignore` expect.
const GITIGNORE_BYTES: &[u8] = b"/tree/\n";

/// Filename of the synthesized bucket index (Phase 15). Leading `_`
/// keeps the file out of naive `*.md` globs while still being visible
/// to `ls`. The `INDEX` spelling matches the `README.md`/`LICENSE`
/// convention for "meta" files at a directory root.
const BUCKET_INDEX_FILENAME: &str = "_INDEX.md";

/// SG-07 ceilings for backend calls. A dead backend MUST surface EIO to
/// the kernel within these budgets so callbacks never wedge the VFS.
const READ_GET_TIMEOUT: Duration = Duration::from_secs(5);
const READ_LIST_TIMEOUT: Duration = Duration::from_secs(15);

/// Errors reachable from the FUSE backend helpers. Formerly lived in
/// `crate::fetch`; moved here in Phase 14 when the write path was lifted
/// onto [`IssueBackend`]. Name retained (not renamed to `FsError`) to
/// minimize diff; a future cleanup phase may rename.
///
/// Intentionally opaque to callers — the FUSE callback path ultimately
/// collapses all of these to `EIO` or `ENOENT`, but keeping the variants
/// distinct aids tests and logs.
#[derive(Debug, Error)]
enum FetchError {
    /// Wall-clock timeout elapsed before the backend responded. The FUSE
    /// callback MUST map this to `libc::EIO`.
    #[error("backend did not respond within timeout")]
    Timeout,
    /// Backend reported the issue does not exist.
    #[error("issue not found")]
    NotFound,
    /// Transport-level failure (TCP refused, TLS handshake failure, etc).
    /// Surfaces from `reposix_core::Error::Http`.
    #[error("transport: {0}")]
    Transport(#[from] reqwest::Error),
    /// The target origin is not allowlisted. Bubbles up from the sealed
    /// `HttpClient` inside the trait impl.
    #[error("origin not allowlisted: {0}")]
    Origin(String),
    /// Backend returned a 200 but the JSON body did not match our schema.
    /// Surfaces from `reposix_core::Error::Json`.
    #[error("parse: {0}")]
    Parse(#[from] serde_json::Error),
    /// Other core errors (e.g. allowlist env var un-parseable, non-success
    /// HTTP status the backend collapsed into `Error::Other`, or any
    /// `reposix_core::Error::Other` that didn't match a more specific
    /// arm).
    #[error("core: {0}")]
    Core(String),
    /// Backend returned 409 on a PATCH — the `If-Match` version did not
    /// match the current server version. The FUSE callback maps this to
    /// `libc::EIO`; the user must `git pull --rebase` to reconcile.
    #[error("version mismatch: current={current}")]
    Conflict {
        /// Current server version, parsed from the 409 response body.
        current: u64,
    },
}

/// Body shape for 409 Conflict responses from the sim's PATCH handler.
/// Matches `{"error":"version_mismatch","current":N,"sent":"..."}` — we
/// only need `current` to surface back into [`FetchError::Conflict`]. The
/// `#[serde(default)]` container implicit in omitting other fields means
/// extra keys are ignored (forward-compatible).
#[derive(Deserialize)]
struct ConflictBody {
    current: u64,
}

/// Map a `reposix_core::Error` from an `IssueBackend` call into a
/// [`FetchError`].
///
/// The `"version mismatch:"` arm (Phase 14 Wave B1) strips the prefix and
/// JSON-parses the tail to recover the `current` integer into
/// [`FetchError::Conflict`], preserving the diagnostic log line the
/// `release` callback emits on 409. A malformed body degrades to
/// `current: 0` so we never fail to surface a conflict.
fn backend_err_to_fetch(e: reposix_core::Error) -> FetchError {
    match e {
        reposix_core::Error::InvalidOrigin(o) => FetchError::Origin(o),
        reposix_core::Error::Http(t) => FetchError::Transport(t),
        reposix_core::Error::Json(j) => FetchError::Parse(j),
        reposix_core::Error::Other(msg) if msg.starts_with("not found") => FetchError::NotFound,
        reposix_core::Error::Other(msg) if msg.starts_with("version mismatch:") => {
            // Strip "version mismatch: " (with or without trailing space)
            // and parse the JSON tail. The sim always emits a JSON body;
            // non-sim backends that surface a version mismatch via this
            // prefix would be expected to do the same — but if the tail
            // is unparseable we still surface the conflict (current=0)
            // rather than demoting it to a generic Core error.
            let tail = msg
                .strip_prefix("version mismatch:")
                .unwrap_or(&msg)
                .trim_start();
            let current = serde_json::from_str::<ConflictBody>(tail)
                .map(|b| b.current)
                .unwrap_or(0);
            FetchError::Conflict { current }
        }
        other => FetchError::Core(other.to_string()),
    }
}

async fn list_issues_with_timeout(
    backend: &Arc<dyn IssueBackend>,
    project: &str,
) -> Result<Vec<Issue>, FetchError> {
    match tokio::time::timeout(READ_LIST_TIMEOUT, backend.list_issues(project)).await {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(e)) => Err(backend_err_to_fetch(e)),
        Err(_) => Err(FetchError::Timeout),
    }
}

async fn get_issue_with_timeout(
    backend: &Arc<dyn IssueBackend>,
    project: &str,
    id: IssueId,
) -> Result<Issue, FetchError> {
    match tokio::time::timeout(READ_GET_TIMEOUT, backend.get_issue(project, id)).await {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(e)) => Err(backend_err_to_fetch(e)),
        Err(_) => Err(FetchError::Timeout),
    }
}

/// PATCH wrapper — routes through [`IssueBackend::update_issue`] with the
/// same 5-second wall-clock ceiling the read path enforces. The outer
/// [`tokio::time::timeout`] is the belt-and-suspenders guard against a
/// backend that returns headers and then stalls on the body (reqwest's
/// inner `total_timeout` covers up through header-read only; see
/// `14-RESEARCH.md#Q3`).
async fn update_issue_with_timeout(
    backend: &Arc<dyn IssueBackend>,
    project: &str,
    id: IssueId,
    patch: Untainted<Issue>,
    expected_version: Option<u64>,
) -> Result<Issue, FetchError> {
    match tokio::time::timeout(
        READ_GET_TIMEOUT,
        backend.update_issue(project, id, patch, expected_version),
    )
    .await
    {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(e)) => Err(backend_err_to_fetch(e)),
        Err(_) => Err(FetchError::Timeout),
    }
}

/// POST wrapper — routes through [`IssueBackend::create_issue`] with the
/// same 5-second wall-clock ceiling. Symmetric to
/// [`update_issue_with_timeout`].
async fn create_issue_with_timeout(
    backend: &Arc<dyn IssueBackend>,
    project: &str,
    issue: Untainted<Issue>,
) -> Result<Issue, FetchError> {
    match tokio::time::timeout(READ_GET_TIMEOUT, backend.create_issue(project, issue)).await {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(e)) => Err(backend_err_to_fetch(e)),
        Err(_) => Err(FetchError::Timeout),
    }
}

/// TTLs applied to every reply.
const ENTRY_TTL: Duration = Duration::from_secs(1);
const ATTR_TTL: Duration = Duration::from_secs(1);

/// Render the canonical on-disk filename for an issue: 11-digit
/// zero-padded decimal + `.md`. Matches the padding codified by Wave B2
/// in `tree::symlink_target` so every symlink target finds the real file.
fn issue_filename(id: IssueId) -> String {
    format!("{:011}.md", id.0)
}

/// Escape a single pipe table cell so a `|` inside the value cannot
/// close the cell prematurely. Also folds any embedded newlines to
/// spaces — pipe-tables are single-line per row. Other characters
/// pass through untouched; no HTML entity escaping (agents read this
/// via `cat`, not a browser).
fn escape_index_cell(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '|' => out.push_str(r"\|"),
            '\r' | '\n' => out.push(' '),
            other => out.push(other),
        }
    }
    out
}

/// Pure render of the bucket `_INDEX.md` body (Phase 15, OP-2 partial).
///
/// Produces a YAML-frontmatter + markdown-pipe-table document listing
/// every issue/page in `issues`, sorted ascending by `id`. The shape is
/// LD-15-02 (frontmatter keys) + LD-15-10 (deterministic order):
///
/// ```markdown
/// ---
/// backend: sim
/// project: demo
/// issue_count: 2
/// generated_at: 2026-04-14T17:15:00Z
/// ---
///
/// # Index of issues/ — demo (2 issues)
///
/// | id | status | title | updated |
/// | --- | --- | --- | --- |
/// | 1 | open | Hello | 2026-04-14 |
/// | 2 | open | World | 2026-04-14 |
/// ```
///
/// `bucket` is the directory name ("issues" or "pages") and drives the
/// pluralised header noun (`"pages"` vs `"issues"`). `generated_at` is
/// injected rather than read from the wall clock so callers (and tests)
/// control determinism. Empty `issues` renders a valid document with
/// `issue_count: 0` and a body-less (header-only) table.
///
/// Pipe characters in titles are escaped to `\|`; embedded newlines fold
/// to spaces. No truncation.
fn render_bucket_index(
    issues: &[Issue],
    backend_name: &str,
    project: &str,
    bucket: &str,
    generated_at: chrono::DateTime<chrono::Utc>,
) -> Vec<u8> {
    use std::fmt::Write as _;

    let mut sorted: Vec<&Issue> = issues.iter().collect();
    sorted.sort_by_key(|i| i.id.0);

    let ts = generated_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let count = sorted.len();
    // Frontmatter key ordering is stable so agents parsing this can
    // rely on field layout without a YAML library if they really want
    // to. serde_yaml would reorder fields alphabetically.
    let mut out = String::with_capacity(256 + count * 96);
    out.push_str("---\n");
    // `write!` into a `String` is infallible; the `.ok()` suppresses the
    // never-fires `fmt::Error` without unwrap noise.
    let _ = writeln!(out, "backend: {backend_name}");
    let _ = writeln!(out, "project: {project}");
    let _ = writeln!(out, "issue_count: {count}");
    let _ = writeln!(out, "generated_at: {ts}");
    out.push_str("---\n\n");
    let _ = writeln!(out, "# Index of {bucket}/ — {project} ({count} {bucket})");
    out.push('\n');
    out.push_str("| id | status | title | updated |\n");
    out.push_str("| --- | --- | --- | --- |\n");
    for issue in &sorted {
        let day = issue.updated_at.format("%Y-%m-%d");
        let title = escape_index_cell(&issue.title);
        let _ = writeln!(
            out,
            "| {id} | {status} | {title} | {day} |",
            id = issue.id.0,
            status = issue.status.as_str(),
        );
    }
    out.into_bytes()
}

/// Pure render of a tree-directory `_INDEX.md` body (Phase 18, OP-2 INDEX-01).
///
/// Performs a DFS from `root_dir` using the `snapshot` for resolution.
/// The `snapshot` is cycle-free by construction (see [`crate::tree::TreeSnapshot::build`]),
/// so no visited-set is needed. Produces YAML frontmatter + a pipe-table
/// with columns `depth | name | target`; rows in DFS traversal order.
///
/// `escape_index_cell` is applied to all `name` and `target` values
/// (T-18-01 mitigation).
fn render_tree_index(
    root_dir: &crate::tree::TreeDir,
    snapshot: &crate::tree::TreeSnapshot,
    project: &str,
    generated_at: chrono::DateTime<chrono::Utc>,
) -> Vec<u8> {
    use std::fmt::Write as _;

    // DFS: each stack entry is (&TreeEntry, depth_relative_to_root_dir).
    // Push siblings in reverse order so the first sibling pops (and is
    // appended to `rows`) first, producing correct pre-order left-to-right
    // traversal.
    let mut rows: Vec<(usize, String, String)> = Vec::new();
    let mut stack: Vec<(&crate::tree::TreeEntry, usize)> = root_dir
        .children
        .iter()
        .rev()
        .map(|e| (e, 0_usize))
        .collect();
    while let Some((entry, depth)) = stack.pop() {
        match entry {
            crate::tree::TreeEntry::Symlink { name, target, .. } => {
                rows.push((depth, name.clone(), target.clone()));
            }
            crate::tree::TreeEntry::Dir(ino) => {
                if let Some(dir) = snapshot.resolve_dir(*ino) {
                    rows.push((depth, format!("{}/", dir.name), String::new()));
                    // Push children in reverse so first child pops first.
                    for child in dir.children.iter().rev() {
                        stack.push((child, depth + 1));
                    }
                }
            }
        }
    }
    let ts = generated_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let count = rows.len();
    let mut out = String::with_capacity(256 + count * 80);
    out.push_str("---\n");
    let _ = writeln!(out, "kind: tree-index");
    let _ = writeln!(out, "project: {project}");
    let _ = writeln!(out, "subtree: {}", escape_index_cell(&root_dir.name));
    let _ = writeln!(out, "entry_count: {count}");
    let _ = writeln!(out, "generated_at: {ts}");
    out.push_str("---\n\n");
    let _ = writeln!(
        out,
        "# Subtree index: tree/{}/",
        escape_index_cell(&root_dir.name)
    );
    out.push('\n');
    out.push_str("| depth | name | target |\n");
    out.push_str("| --- | --- | --- |\n");
    for (depth, name, target) in &rows {
        let _ = writeln!(
            out,
            "| {depth} | {name} | {target} |",
            name = escape_index_cell(name),
            target = escape_index_cell(target),
        );
    }
    out.into_bytes()
}

/// Pure render of the mount-root `_INDEX.md` body (Phase 18, OP-2 INDEX-02).
///
/// Produces YAML frontmatter + a pipe-table with columns `entry | kind | count`.
/// The `tree/` row is only emitted when `tree_present` is `true`.
fn render_mount_root_index(
    backend_name: &str,
    project: &str,
    bucket: &str,
    issue_count: usize,
    tree_present: bool,
    generated_at: chrono::DateTime<chrono::Utc>,
) -> Vec<u8> {
    use std::fmt::Write as _;

    let ts = generated_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let mut out = String::with_capacity(512);
    out.push_str("---\n");
    let _ = writeln!(out, "kind: mount-index");
    let _ = writeln!(out, "backend: {backend_name}");
    let _ = writeln!(out, "project: {project}");
    let _ = writeln!(out, "bucket: {bucket}");
    let _ = writeln!(out, "issue_count: {issue_count}");
    let _ = writeln!(out, "generated_at: {ts}");
    out.push_str("---\n\n");
    let _ = writeln!(out, "# Mount index — {project}");
    out.push('\n');
    out.push_str("| entry | kind | count |\n");
    out.push_str("| --- | --- | --- |\n");
    let _ = writeln!(out, "| .gitignore | file | — |");
    let _ = writeln!(out, "| {bucket}/ | directory | {issue_count} |");
    if tree_present {
        let _ = writeln!(out, "| tree/ | directory | — |");
    }
    out.into_bytes()
}

/// Partition of the u64 inode space. Every callback entry point runs
/// [`InodeKind::classify`] first and branches on the result before any
/// map lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InodeKind {
    /// Mount root (inode 1).
    Root,
    /// The `<bucket>/` directory (inode 2).
    Bucket,
    /// The `tree/` overlay root (inode 3).
    TreeRoot,
    /// The synthesized `.gitignore` file (inode 4).
    Gitignore,
    /// The synthesized `_INDEX.md` file inside the bucket (inode 5).
    BucketIndex,
    /// A real issue file in the bucket — `FIRST_ISSUE_INODE..TREE_DIR_INO_BASE`.
    RealFile,
    /// An interior tree directory — `TREE_DIR_INO_BASE..TREE_SYMLINK_INO_BASE`.
    TreeDir,
    /// A tree leaf symlink or `_self.md` — `TREE_SYMLINK_INO_BASE..`.
    TreeSymlink,
    /// The synthesized `_INDEX.md` at the mount root (inode 6).
    RootIndex,
    /// A per-tree-dir synthesized `_INDEX.md` (inodes 7..=0xFFFF).
    TreeDirIndex,
    /// Unassigned — surface as ENOENT.
    Unknown,
}

impl InodeKind {
    fn classify(ino: u64) -> Self {
        match ino {
            ROOT_INO => Self::Root,
            BUCKET_DIR_INO => Self::Bucket,
            TREE_ROOT_INO => Self::TreeRoot,
            GITIGNORE_INO => Self::Gitignore,
            BUCKET_INDEX_INO => Self::BucketIndex,
            ROOT_INDEX_INO => Self::RootIndex,
            n if (TREE_INDEX_ALLOC_START..=TREE_INDEX_ALLOC_END).contains(&n) => Self::TreeDirIndex,
            n if n >= TREE_SYMLINK_INO_BASE => Self::TreeSymlink,
            n if n >= TREE_DIR_INO_BASE => Self::TreeDir,
            n if n >= FIRST_ISSUE_INODE => Self::RealFile,
            _ => Self::Unknown,
        }
    }
}

/// Rendered-and-cached file bytes keyed by inode.
#[derive(Debug)]
struct CachedFile {
    issue: Issue,
    rendered: String,
    /// `true` if this entry was populated by a backend `get_issue` call
    /// (full body guaranteed present). `false` if populated by `list_issues`
    /// — some backends (Confluence v2) return body-less stubs on the list
    /// endpoint.
    body_fetched: bool,
}

/// FUSE filesystem backed by an [`IssueBackend`] trait object for both
/// read and write paths (Phase 10 + Phase 14).
pub struct ReposixFs {
    /// Tokio runtime owned by the FS; used for `block_on` on callbacks.
    rt: Arc<Runtime>,
    /// Backend for all I/O — reads (`list_issues`/`get_issue`) and writes
    /// (`create_issue`/`update_issue`). SG-05 audit attribution lives on
    /// the backend's internal `X-Reposix-Agent` header (set at
    /// construction via `SimBackend::with_agent_suffix`).
    backend: Arc<dyn IssueBackend>,
    /// Backend origin retained for diagnostic rendering (Debug, tracing).
    /// Not plumbed through any I/O path — the backend owns its own origin.
    origin: String,
    /// Project slug (sim) or `owner/repo` (github).
    project: String,
    /// Inode registry (real issue files under the bucket).
    registry: InodeRegistry,
    /// Per-backend bucket name (`"issues"` or `"pages"`). Set once at
    /// construction from `backend.root_collection_name()`.
    bucket: &'static str,
    /// Whether the backend natively advertises hierarchy support
    /// (`BackendFeature::Hierarchy`). Set once at construction. Even if
    /// this is `false`, `tree/` may still emit when at least one issue
    /// reports a `parent_id` — see `should_emit_tree`.
    hierarchy_feature: bool,
    /// Tree overlay snapshot. Rebuilt on each `readdir` refresh whenever
    /// the issue list is re-fetched. Empty at mount open; filled in on
    /// first `readdir` that hits either the root or the bucket.
    tree: Arc<RwLock<TreeSnapshot>>,
    /// Rendered-file cache. Invalidated wholesale on the next `readdir`
    /// refresh (entries overwritten).
    cache: DashMap<u64, Arc<CachedFile>>,
    /// Per-inode write buffers. See `release` for the drain/PATCH path.
    write_buffers: DashMap<u64, Vec<u8>>,
    /// Monotonic file-handle allocator for `create`.
    next_fh: AtomicU64,
    /// Root directory attributes — cached once at construction.
    root_attr: FileAttr,
    /// Bucket directory attributes — same shape as `root_attr`, different inode.
    bucket_attr: FileAttr,
    /// `tree/` directory attributes.
    tree_attr: FileAttr,
    /// `.gitignore` file attributes (size is compile-time constant).
    gitignore_attr: FileAttr,
    /// Stable timestamp captured once at mount construction. Used for
    /// symlink and tree-dir `FileAttr` so repeated `stat` calls don't
    /// report drifting `st_mtim` (IN-03 from 13-REVIEW.md).
    mount_time: SystemTime,
    /// Cached rendered bytes for the synthesized bucket `_INDEX.md`
    /// (Phase 15). Computed lazily on first `read` / `getattr` /
    /// `lookup` that references the bucket index, then reused for
    /// subsequent calls until the next [`refresh_issues`] drops it.
    /// `None` means "render on next access"; `Some(arc)` means "serve
    /// these bytes". Wrapped in `Arc` so callbacks can clone cheaply
    /// without holding the lock across the FUSE reply.
    bucket_index_bytes: RwLock<Option<Arc<Vec<u8>>>>,
    /// Cached rendered bytes for the mount-root `_INDEX.md` (Phase 18).
    /// Same lazily-invalidated pattern as `bucket_index_bytes`.
    mount_root_index_bytes: RwLock<Option<Arc<Vec<u8>>>>,
    /// Per-tree-dir `_INDEX.md` render cache, keyed by tree-dir inode.
    /// Cleared on each `refresh_issues`.
    tree_dir_index_cache: DashMap<u64, Arc<Vec<u8>>>,
    /// Forward map: tree-dir inode → its `_INDEX.md` inode (lazily allocated).
    tree_index_inodes: DashMap<u64, u64>,
    /// Reverse map: `_INDEX.md` inode → tree-dir inode. Needed for `getattr`
    /// when the kernel calls with the index inode before any `lookup`.
    tree_index_ino_reverse: DashMap<u64, u64>,
    /// Monotonic inode allocator for tree-dir `_INDEX.md` files.
    /// Starts at `TREE_INDEX_ALLOC_START` (7); capped at `TREE_INDEX_ALLOC_END`.
    tree_index_alloc: AtomicU64,
}

impl std::fmt::Debug for ReposixFs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReposixFs")
            .field("backend", &self.backend.name())
            .field("origin", &self.origin)
            .field("project", &self.project)
            .field("bucket", &self.bucket)
            .field("hierarchy_feature", &self.hierarchy_feature)
            .finish_non_exhaustive()
    }
}

impl ReposixFs {
    /// Build a new FUSE filesystem whose read and write paths are served
    /// by `backend` via the [`IssueBackend`] trait.
    ///
    /// # Errors
    /// Returns any error constructing the Tokio runtime. Allowlist / HTTP
    /// client construction happens inside `backend` itself (e.g. at
    /// `SimBackend::new`); any resulting [`anyhow::Error`] from runtime
    /// build or inode registry setup propagates here.
    pub fn new(
        backend: Arc<dyn IssueBackend>,
        origin: String,
        project: String,
    ) -> anyhow::Result<Self> {
        let rt = Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .thread_name("reposix-fuse-rt")
                .build()?,
        );
        let bucket = backend.root_collection_name();
        let hierarchy_feature = backend.supports(BackendFeature::Hierarchy);
        let now = SystemTime::now();
        let uid = uid_safe();
        let gid = gid_safe();
        let dir_attr = |ino: u64, perm: u16| FileAttr {
            ino: INodeNo(ino),
            size: 0,
            blocks: 0,
            atime: now,
            mtime: now,
            ctime: now,
            crtime: now,
            kind: FileType::Directory,
            perm,
            nlink: 2,
            uid,
            gid,
            rdev: 0,
            blksize: 4096,
            flags: 0,
        };
        let root_attr = dir_attr(ROOT_INO, 0o755);
        let bucket_attr = dir_attr(BUCKET_DIR_INO, 0o755);
        // tree/ is read-only (0o555): symlinks inside are immutable derived
        // views, and the kernel should refuse `mkdir`/`touch` at the VFS
        // layer without reaching our (un-implemented) write callbacks.
        let tree_attr = dir_attr(TREE_ROOT_INO, 0o555);
        let gitignore_attr = FileAttr {
            ino: INodeNo(GITIGNORE_INO),
            size: GITIGNORE_BYTES.len() as u64,
            blocks: 1,
            atime: now,
            mtime: now,
            ctime: now,
            crtime: now,
            kind: FileType::RegularFile,
            perm: 0o444,
            nlink: 1,
            uid,
            gid,
            rdev: 0,
            blksize: 4096,
            flags: 0,
        };
        Ok(Self {
            rt,
            backend,
            origin,
            project,
            registry: InodeRegistry::new(),
            bucket,
            hierarchy_feature,
            tree: Arc::new(RwLock::new(TreeSnapshot::default())),
            cache: DashMap::new(),
            write_buffers: DashMap::new(),
            next_fh: AtomicU64::new(1),
            root_attr,
            bucket_attr,
            tree_attr,
            gitignore_attr,
            mount_time: now,
            bucket_index_bytes: RwLock::new(None),
            mount_root_index_bytes: RwLock::new(None),
            tree_dir_index_cache: DashMap::new(),
            tree_index_inodes: DashMap::new(),
            tree_index_ino_reverse: DashMap::new(),
            tree_index_alloc: AtomicU64::new(TREE_INDEX_ALLOC_START),
        })
    }

    /// `true` if `tree/` should be surfaced at the mount root. Gated by
    /// either the backend-feature bit or a non-empty tree snapshot.
    fn should_emit_tree(&self) -> bool {
        if self.hierarchy_feature {
            return true;
        }
        // Non-hierarchy backends (sim/github) still get a `tree/` overlay
        // if at least one loaded issue carries a `parent_id` — useful for
        // future shims but harmless when absent.
        self.tree
            .read()
            .map(|snap| !snap.is_empty())
            .unwrap_or(false)
    }

    /// Return a `FileAttr` for a real issue file.
    #[allow(clippy::unused_self)]
    fn file_attr(&self, ino: u64, issue: &Issue, size: u64) -> FileAttr {
        let atime: SystemTime = issue.updated_at.into();
        let mtime = atime;
        let ctime: SystemTime = issue.created_at.into();
        FileAttr {
            ino: INodeNo(ino),
            size,
            blocks: size.div_ceil(512),
            atime,
            mtime,
            ctime,
            crtime: ctime,
            kind: FileType::RegularFile,
            perm: 0o644,
            nlink: 1,
            uid: uid_safe(),
            gid: gid_safe(),
            rdev: 0,
            blksize: 4096,
            flags: 0,
        }
    }

    /// Synthesize a `FileAttr` for a tree symlink. Size MUST equal the
    /// target byte length — a 0-size symlink surfaces as a 0-byte file
    /// in `ls -l` (restic bug, see `13-RESEARCH.md` §Pitfall 1).
    ///
    /// Timestamps use the stable `mount_time` so repeated `stat` calls
    /// return consistent `st_mtim` (IN-03 from 13-REVIEW.md). Tree
    /// symlinks are immutable per snapshot; drifting mtime would confuse
    /// rsync, make, backup tools.
    fn symlink_attr(&self, ino: u64, target: &str) -> FileAttr {
        FileAttr {
            ino: INodeNo(ino),
            size: target.len() as u64,
            blocks: 0,
            atime: self.mount_time,
            mtime: self.mount_time,
            ctime: self.mount_time,
            crtime: self.mount_time,
            kind: FileType::Symlink,
            // Symlink perm bits are ignored by the kernel but 0o777 is the
            // POSIX convention.
            perm: 0o777,
            nlink: 1,
            uid: uid_safe(),
            gid: gid_safe(),
            rdev: 0,
            blksize: 512,
            flags: 0,
        }
    }

    /// `FileAttr` for an interior tree dir. Depth doesn't matter for
    /// attr purposes.
    fn tree_dir_attr(&self, ino: u64) -> FileAttr {
        let mut a = self.tree_attr;
        a.ino = INodeNo(ino);
        a
    }

    /// Resolve a name in the `<bucket>/` directory to an (inode, cache entry)
    /// pair. Does the backend fetch + render on miss; populates cache.
    fn resolve_name(&self, name: &str) -> Result<(u64, Arc<CachedFile>), FetchError> {
        let id = validate_issue_filename(name).map_err(|e| FetchError::Core(e.to_string()))?;
        if let Some(ino) = self.registry.lookup_id(id) {
            if let Some(c) = self.cache.get(&ino) {
                if c.body_fetched {
                    return Ok((ino, c.clone()));
                }
            }
        }
        let issue = self
            .rt
            .block_on(get_issue_with_timeout(&self.backend, &self.project, id))?;
        let rendered = frontmatter::render(&issue).map_err(|e| FetchError::Core(e.to_string()))?;
        let ino = self.registry.intern(id);
        let entry = Arc::new(CachedFile {
            issue: issue.clone(),
            rendered,
            body_fetched: true,
        });
        self.cache.insert(ino, entry.clone());
        Ok((ino, entry))
    }

    fn resolve_ino(&self, ino: u64) -> Result<Arc<CachedFile>, FetchError> {
        if let Some(c) = self.cache.get(&ino) {
            if c.body_fetched {
                return Ok(c.clone());
            }
        }
        let Some(id) = self.registry.lookup_ino(ino) else {
            return Err(FetchError::NotFound);
        };
        let issue = self
            .rt
            .block_on(get_issue_with_timeout(&self.backend, &self.project, id))?;
        let rendered = frontmatter::render(&issue).map_err(|e| FetchError::Core(e.to_string()))?;
        let entry = Arc::new(CachedFile {
            issue,
            rendered,
            body_fetched: true,
        });
        self.cache.insert(ino, entry.clone());
        Ok(entry)
    }

    /// Refresh the issue list from the backend, re-populate the inode
    /// registry + rendered-body cache, and rebuild the tree snapshot.
    fn refresh_issues(&self) -> Result<Vec<Issue>, FetchError> {
        let issues = self
            .rt
            .block_on(list_issues_with_timeout(&self.backend, &self.project))?;

        for issue in &issues {
            let ino = self.registry.intern(issue.id);
            let rendered = match frontmatter::render(issue) {
                Ok(s) => s,
                Err(e) => {
                    warn!(error = %e, id = %issue.id, "frontmatter render failed");
                    continue;
                }
            };
            self.cache.insert(
                ino,
                Arc::new(CachedFile {
                    issue: issue.clone(),
                    rendered,
                    body_fetched: false,
                }),
            );
        }

        // Rebuild tree snapshot. Always recompute; it's a pure transform
        // over the list and the cost is O(n) where n = issue count.
        let has_any_parent = issues.iter().any(|i| i.parent_id.is_some());
        if self.hierarchy_feature || has_any_parent {
            let snap = TreeSnapshot::build(self.bucket, &issues);
            if let Ok(mut guard) = self.tree.write() {
                *guard = snap;
            }
        } else if let Ok(mut guard) = self.tree.write() {
            *guard = TreeSnapshot::default();
        }

        // Drop the `_INDEX.md` render cache — the underlying issue list
        // just changed, so any cached bytes are stale. The next
        // `read(BUCKET_INDEX_INO)` re-renders against the fresh cache.
        if let Ok(mut guard) = self.bucket_index_bytes.write() {
            guard.take();
        }
        // Also invalidate the mount-root index and all tree-dir index caches
        // (Phase 18 — issue count and tree shape may have changed).
        if let Ok(mut guard) = self.mount_root_index_bytes.write() {
            guard.take();
        }
        self.tree_dir_index_cache.clear();

        Ok(issues)
    }

    /// Return the cached rendered bytes for the bucket `_INDEX.md`,
    /// rendering on demand if the cache is empty. Uses the already-
    /// populated rendered-file cache as the snapshot — we iterate the
    /// `CachedFile` entries rather than calling the backend again, so
    /// a caller that invoked `refresh_issues` immediately beforehand
    /// gets a coherent view. If the cache is empty (cold mount, no
    /// prior `readdir`), this still renders correctly against the
    /// empty slice.
    fn bucket_index_bytes_or_render(&self) -> Arc<Vec<u8>> {
        // Fast path: hit.
        if let Ok(guard) = self.bucket_index_bytes.read() {
            if let Some(bytes) = guard.as_ref() {
                return bytes.clone();
            }
        }
        // Miss: materialise the issue snapshot from the cache.
        // Collecting + sorting happens inside `render_bucket_index`.
        let issues: Vec<Issue> = self
            .cache
            .iter()
            .map(|entry| entry.value().issue.clone())
            .collect();
        let rendered = Arc::new(render_bucket_index(
            &issues,
            self.backend.name(),
            &self.project,
            self.bucket,
            chrono::Utc::now(),
        ));
        if let Ok(mut guard) = self.bucket_index_bytes.write() {
            *guard = Some(rendered.clone());
        }
        rendered
    }

    /// Synthesize a `FileAttr` for the bucket `_INDEX.md`. Size reflects
    /// the current rendered byte length (LD-15-08 — size is truthful,
    /// not a placeholder); `perm` is `0o444` to match `.gitignore`.
    fn bucket_index_attr(&self, size: u64) -> FileAttr {
        FileAttr {
            ino: INodeNo(BUCKET_INDEX_INO),
            size,
            blocks: size.div_ceil(512).max(1),
            atime: self.mount_time,
            mtime: self.mount_time,
            ctime: self.mount_time,
            crtime: self.mount_time,
            kind: FileType::RegularFile,
            perm: 0o444,
            nlink: 1,
            uid: uid_safe(),
            gid: gid_safe(),
            rdev: 0,
            blksize: 4096,
            flags: 0,
        }
    }

    /// Generalised `FileAttr` for any read-only synthetic file.
    /// Parameterised on inode so both `RootIndex` (inode 6) and
    /// `TreeDirIndex` (inodes 7..=0xFFFF) can share the same shape.
    fn synthetic_file_attr(&self, ino: u64, size: u64) -> FileAttr {
        FileAttr {
            ino: INodeNo(ino),
            size,
            blocks: size.div_ceil(512).max(1),
            atime: self.mount_time,
            mtime: self.mount_time,
            ctime: self.mount_time,
            crtime: self.mount_time,
            kind: FileType::RegularFile,
            perm: 0o444,
            nlink: 1,
            uid: uid_safe(),
            gid: gid_safe(),
            rdev: 0,
            blksize: 4096,
            flags: 0,
        }
    }

    /// Allocate a fresh inode for a tree-dir `_INDEX.md`, saturating at
    /// `TREE_INDEX_ALLOC_END` if the space is exhausted.
    fn alloc_tree_index_ino(&self) -> u64 {
        let ino = self.tree_index_alloc.fetch_add(1, Ordering::SeqCst);
        if ino > TREE_INDEX_ALLOC_END {
            tracing::warn!(
                ino,
                max = TREE_INDEX_ALLOC_END,
                "tree-dir _INDEX.md inode space exhausted; new dirs will share inode {}",
                TREE_INDEX_ALLOC_END
            );
        }
        ino.min(TREE_INDEX_ALLOC_END)
    }

    /// Return the stable `_INDEX.md` inode for `dir_ino`, allocating one on
    /// first call. Subsequent calls for the same `dir_ino` are idempotent.
    fn tree_dir_index_ino(&self, dir_ino: u64) -> u64 {
        use dashmap::Entry;
        match self.tree_index_inodes.entry(dir_ino) {
            Entry::Occupied(e) => *e.get(),
            Entry::Vacant(e) => {
                let ino = self.alloc_tree_index_ino();
                e.insert(ino);
                self.tree_index_ino_reverse.insert(ino, dir_ino);
                ino
            }
        }
    }

    /// Return cached (or freshly rendered) bytes for the tree-dir `_INDEX.md`
    /// corresponding to `dir_ino`. Returns an empty vec if `dir_ino` is not
    /// in the snapshot (should not happen in normal operation).
    fn tree_dir_index_bytes_or_render(
        &self,
        dir_ino: u64,
        snap: &crate::tree::TreeSnapshot,
    ) -> Arc<Vec<u8>> {
        if let Some(cached) = self.tree_dir_index_cache.get(&dir_ino) {
            return cached.clone();
        }
        let Some(dir) = snap.resolve_dir(dir_ino) else {
            return Arc::new(Vec::new());
        };
        let rendered = Arc::new(render_tree_index(
            dir,
            snap,
            &self.project,
            chrono::Utc::now(),
        ));
        self.tree_dir_index_cache.insert(dir_ino, rendered.clone());
        rendered
    }

    /// Return cached (or freshly rendered) bytes for the mount-root `_INDEX.md`.
    fn mount_root_index_bytes_or_render(&self) -> Arc<Vec<u8>> {
        if let Ok(guard) = self.mount_root_index_bytes.read() {
            if let Some(bytes) = guard.as_ref() {
                return bytes.clone();
            }
        }
        let issue_count = self.cache.len();
        let tree_present = self.should_emit_tree();
        let rendered = Arc::new(render_mount_root_index(
            self.backend.name(),
            &self.project,
            self.bucket,
            issue_count,
            tree_present,
            chrono::Utc::now(),
        ));
        if let Ok(mut guard) = self.mount_root_index_bytes.write() {
            *guard = Some(rendered.clone());
        }
        rendered
    }
}

fn uid_safe() -> u32 {
    rustix::process::getuid().as_raw()
}

fn gid_safe() -> u32 {
    rustix::process::getgid().as_raw()
}

/// Map a [`FetchError`] to a kernel errno.
fn fetch_errno(e: &FetchError) -> i32 {
    match e {
        FetchError::NotFound => libc::ENOENT,
        FetchError::Origin(_) => libc::EACCES,
        FetchError::Timeout
        | FetchError::Transport(_)
        | FetchError::Parse(_)
        | FetchError::Core(_)
        | FetchError::Conflict { .. } => libc::EIO,
    }
}

impl Filesystem for ReposixFs {
    fn init(&mut self, _req: &Request, _config: &mut fuser::KernelConfig) -> Result<(), io::Error> {
        Ok(())
    }

    fn getattr(&self, _req: &Request, ino: INodeNo, _fh: Option<FileHandle>, reply: ReplyAttr) {
        let ino_u = ino.0;
        match InodeKind::classify(ino_u) {
            InodeKind::Root => reply.attr(&ATTR_TTL, &self.root_attr),
            InodeKind::Bucket => reply.attr(&ATTR_TTL, &self.bucket_attr),
            InodeKind::TreeRoot => reply.attr(&ATTR_TTL, &self.tree_attr),
            InodeKind::Gitignore => reply.attr(&ATTR_TTL, &self.gitignore_attr),
            InodeKind::BucketIndex => {
                let bytes = self.bucket_index_bytes_or_render();
                let attr = self.bucket_index_attr(bytes.len() as u64);
                reply.attr(&ATTR_TTL, &attr);
            }
            InodeKind::RealFile => {
                if let Some(c) = self.cache.get(&ino_u) {
                    let size = if let Some(buf) = self.write_buffers.get(&ino_u) {
                        buf.len() as u64
                    } else {
                        c.rendered.len() as u64
                    };
                    let attr = self.file_attr(ino_u, &c.issue, size);
                    reply.attr(&ATTR_TTL, &attr);
                } else {
                    reply.error(fuser::Errno::from_i32(libc::ENOENT));
                }
            }
            InodeKind::TreeDir => {
                let Ok(snap) = self.tree.read() else {
                    reply.error(fuser::Errno::from_i32(libc::EIO));
                    return;
                };
                if snap.resolve_dir(ino_u).is_some() {
                    let attr = self.tree_dir_attr(ino_u);
                    reply.attr(&ATTR_TTL, &attr);
                } else {
                    reply.error(fuser::Errno::from_i32(libc::ENOENT));
                }
            }
            InodeKind::TreeSymlink => {
                let Ok(snap) = self.tree.read() else {
                    reply.error(fuser::Errno::from_i32(libc::EIO));
                    return;
                };
                if let Some(target) = snap.resolve_symlink(ino_u) {
                    let attr = self.symlink_attr(ino_u, target);
                    reply.attr(&ATTR_TTL, &attr);
                } else {
                    reply.error(fuser::Errno::from_i32(libc::ENOENT));
                }
            }
            InodeKind::RootIndex => {
                let bytes = self.mount_root_index_bytes_or_render();
                let attr = self.synthetic_file_attr(ROOT_INDEX_INO, bytes.len() as u64);
                reply.attr(&ATTR_TTL, &attr);
            }
            InodeKind::TreeDirIndex => {
                // Pitfall 2: kernel may call getattr before lookup (e.g. after
                // remount). Reverse-look up the dir_ino from tree_index_ino_reverse.
                let Some(dir_ino) = self.tree_index_ino_reverse.get(&ino_u).map(|v| *v) else {
                    reply.error(fuser::Errno::from_i32(libc::ENOENT));
                    return;
                };
                let Ok(snap) = self.tree.read() else {
                    reply.error(fuser::Errno::from_i32(libc::EIO));
                    return;
                };
                let bytes = self.tree_dir_index_bytes_or_render(dir_ino, &snap);
                let attr = self.synthetic_file_attr(ino_u, bytes.len() as u64);
                reply.attr(&ATTR_TTL, &attr);
            }
            InodeKind::Unknown => reply.error(fuser::Errno::from_i32(libc::ENOENT)),
        }
    }

    fn lookup(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEntry) {
        let parent_u = parent.0;
        let Some(name_str) = name.to_str() else {
            reply.error(fuser::Errno::from_i32(libc::EINVAL));
            return;
        };
        match InodeKind::classify(parent_u) {
            InodeKind::Root => {
                if name_str == ".gitignore" {
                    reply.entry(&ENTRY_TTL, &self.gitignore_attr, fuser::Generation(0));
                    return;
                }
                if name_str == self.bucket {
                    reply.entry(&ENTRY_TTL, &self.bucket_attr, fuser::Generation(0));
                    return;
                }
                if name_str == "tree" && self.should_emit_tree() {
                    reply.entry(&ENTRY_TTL, &self.tree_attr, fuser::Generation(0));
                    return;
                }
                if name_str == "_INDEX.md" {
                    let bytes = self.mount_root_index_bytes_or_render();
                    let attr = self.synthetic_file_attr(ROOT_INDEX_INO, bytes.len() as u64);
                    reply.entry(&ENTRY_TTL, &attr, fuser::Generation(0));
                    return;
                }
                reply.error(fuser::Errno::from_i32(libc::ENOENT));
            }
            InodeKind::Bucket => {
                if name_str == BUCKET_INDEX_FILENAME {
                    let bytes = self.bucket_index_bytes_or_render();
                    let attr = self.bucket_index_attr(bytes.len() as u64);
                    reply.entry(&ENTRY_TTL, &attr, fuser::Generation(0));
                    return;
                }
                if validate_issue_filename(name_str).is_err() {
                    reply.error(fuser::Errno::from_i32(libc::ENOENT));
                    return;
                }
                match self.resolve_name(name_str) {
                    Ok((ino, cached)) => {
                        let size = cached.rendered.len() as u64;
                        let attr = self.file_attr(ino, &cached.issue, size);
                        reply.entry(&ENTRY_TTL, &attr, fuser::Generation(0));
                    }
                    Err(e) => {
                        warn!(error = %e, name = %name_str, "lookup failed");
                        reply.error(fuser::Errno::from_i32(fetch_errno(&e)));
                    }
                }
            }
            InodeKind::TreeRoot => {
                let Ok(snap) = self.tree.read() else {
                    reply.error(fuser::Errno::from_i32(libc::EIO));
                    return;
                };
                self.reply_tree_entry(&snap, snap.root_entries(), name_str, reply);
            }
            InodeKind::TreeDir => {
                let Ok(snap) = self.tree.read() else {
                    reply.error(fuser::Errno::from_i32(libc::EIO));
                    return;
                };
                let Some(dir) = snap.resolve_dir(parent_u) else {
                    reply.error(fuser::Errno::from_i32(libc::ENOENT));
                    return;
                };
                if name_str == "_INDEX.md" {
                    let index_ino = self.tree_dir_index_ino(parent_u);
                    let bytes = self.tree_dir_index_bytes_or_render(parent_u, &snap);
                    let attr = self.synthetic_file_attr(index_ino, bytes.len() as u64);
                    reply.entry(&ENTRY_TTL, &attr, fuser::Generation(0));
                    return;
                }
                self.reply_tree_entry(&snap, &dir.children, name_str, reply);
            }
            InodeKind::Gitignore
            | InodeKind::BucketIndex
            | InodeKind::RealFile
            | InodeKind::TreeSymlink
            | InodeKind::RootIndex
            | InodeKind::TreeDirIndex => {
                reply.error(fuser::Errno::from_i32(libc::ENOTDIR));
            }
            InodeKind::Unknown => reply.error(fuser::Errno::from_i32(libc::ENOENT)),
        }
    }

    #[allow(
        clippy::too_many_lines,
        reason = "readdir dispatches over all InodeKind variants; splitting would obscure the \
                  exhaustive-match structure. Task 2 will add more arms; a refactor phase can \
                  extract helpers after the full dispatch is in place."
    )]
    fn readdir(
        &self,
        _req: &Request,
        ino: INodeNo,
        _fh: FileHandle,
        offset: u64,
        mut reply: ReplyDirectory,
    ) {
        let ino_u = ino.0;
        let entries: Vec<(u64, FileType, String)> = match InodeKind::classify(ino_u) {
            InodeKind::Root => {
                // Ensure at least one refresh has populated the tree before
                // deciding whether to emit `tree/`. When the backend
                // advertises Hierarchy the answer is already "yes" — but we
                // still prime the cache so `lookup("tree")` resolves.
                // Non-hierarchy backends without any parent_id return a
                // no-op empty snapshot here.
                if let Err(e) = self.refresh_issues() {
                    warn!(error = %e, "root readdir refresh failed (non-fatal)");
                }
                let mut out: Vec<(u64, FileType, String)> = Vec::with_capacity(5);
                out.push((ROOT_INO, FileType::Directory, ".".to_owned()));
                out.push((ROOT_INO, FileType::Directory, "..".to_owned()));
                out.push((
                    GITIGNORE_INO,
                    FileType::RegularFile,
                    ".gitignore".to_owned(),
                ));
                out.push((BUCKET_DIR_INO, FileType::Directory, self.bucket.to_owned()));
                if self.should_emit_tree() {
                    out.push((TREE_ROOT_INO, FileType::Directory, "tree".to_owned()));
                }
                out.push((
                    ROOT_INDEX_INO,
                    FileType::RegularFile,
                    "_INDEX.md".to_owned(),
                ));
                out
            }
            InodeKind::Bucket => {
                let issues = match self.refresh_issues() {
                    Ok(v) => v,
                    Err(e) => {
                        warn!(error = %e, "bucket readdir fetch failed");
                        reply.error(fuser::Errno::from_i32(fetch_errno(&e)));
                        return;
                    }
                };
                let mut sorted = issues;
                sorted.sort_by_key(|i| i.id.0);
                let mut out: Vec<(u64, FileType, String)> = Vec::with_capacity(sorted.len() + 3);
                out.push((BUCKET_DIR_INO, FileType::Directory, ".".to_owned()));
                out.push((ROOT_INO, FileType::Directory, "..".to_owned()));
                // `_INDEX.md` is emitted before the real `<padded-id>.md`
                // entries so agents doing `head -1 <(ls <bucket>/)` see
                // the index first. Plan 15-A, LD-15-01.
                out.push((
                    BUCKET_INDEX_INO,
                    FileType::RegularFile,
                    BUCKET_INDEX_FILENAME.to_owned(),
                ));
                for issue in &sorted {
                    let ino = self.registry.intern(issue.id);
                    out.push((ino, FileType::RegularFile, issue_filename(issue.id)));
                }
                out
            }
            InodeKind::TreeRoot => {
                // WR-01: ensure the tree snapshot is populated on first touch.
                // Without this, a user whose first command is `ls mount/tree/`
                // (before any `ls mount/` or `ls mount/pages/`) sees an empty
                // directory silently — a wrong-data regression. Matches the
                // Root / Bucket readdir refresh pattern. Error is non-fatal:
                // fall through to the (possibly empty) cached snapshot so the
                // user sees stale data rather than EIO.
                if let Err(e) = self.refresh_issues() {
                    tracing::warn!(
                        error = %e,
                        "tree readdir refresh failed (non-fatal); serving cached snapshot"
                    );
                }
                let Ok(snap) = self.tree.read() else {
                    reply.error(fuser::Errno::from_i32(libc::EIO));
                    return;
                };
                let mut out: Vec<(u64, FileType, String)> =
                    Vec::with_capacity(snap.root_entries().len() + 2);
                out.push((TREE_ROOT_INO, FileType::Directory, ".".to_owned()));
                out.push((ROOT_INO, FileType::Directory, "..".to_owned()));
                collect_tree_entries(&snap, snap.root_entries(), &mut out);
                out
            }
            InodeKind::TreeDir => {
                let Ok(snap) = self.tree.read() else {
                    reply.error(fuser::Errno::from_i32(libc::EIO));
                    return;
                };
                let Some(dir) = snap.resolve_dir(ino_u) else {
                    reply.error(fuser::Errno::from_i32(libc::ENOENT));
                    return;
                };
                let mut out: Vec<(u64, FileType, String)> =
                    Vec::with_capacity(dir.children.len() + 2);
                out.push((dir.ino, FileType::Directory, ".".to_owned()));
                // Parent inode — we don't track the reverse relation in the
                // snapshot, so use `TREE_ROOT_INO` as a conservative default.
                // The kernel follows explicit paths, not `..` entries, so
                // this is cosmetic only (matters for `ls -la` display).
                out.push((TREE_ROOT_INO, FileType::Directory, "..".to_owned()));
                let index_ino = self.tree_dir_index_ino(ino_u);
                out.push((index_ino, FileType::RegularFile, "_INDEX.md".to_owned()));
                collect_tree_entries(&snap, &dir.children, &mut out);
                out
            }
            InodeKind::Gitignore
            | InodeKind::BucketIndex
            | InodeKind::RealFile
            | InodeKind::TreeSymlink
            | InodeKind::RootIndex
            | InodeKind::TreeDirIndex => {
                reply.error(fuser::Errno::from_i32(libc::ENOTDIR));
                return;
            }
            InodeKind::Unknown => {
                reply.error(fuser::Errno::from_i32(libc::ENOENT));
                return;
            }
        };
        let start = usize::try_from(offset).unwrap_or(usize::MAX);
        for (i, (ino, kind, name)) in entries.into_iter().enumerate().skip(start) {
            let next = (i + 1) as u64;
            if reply.add(INodeNo(ino), next, kind, name) {
                break;
            }
        }
        reply.ok();
    }

    fn read(
        &self,
        _req: &Request,
        ino: INodeNo,
        _fh: FileHandle,
        offset: u64,
        size: u32,
        _flags: fuser::OpenFlags,
        _lock_owner: Option<fuser::LockOwner>,
        reply: ReplyData,
    ) {
        let ino_u = ino.0;
        match InodeKind::classify(ino_u) {
            InodeKind::Root | InodeKind::Bucket | InodeKind::TreeRoot | InodeKind::TreeDir => {
                reply.error(fuser::Errno::from_i32(libc::EISDIR));
            }
            InodeKind::Gitignore => {
                let start = usize::try_from(offset)
                    .unwrap_or(usize::MAX)
                    .min(GITIGNORE_BYTES.len());
                let end = start
                    .saturating_add(size as usize)
                    .min(GITIGNORE_BYTES.len());
                reply.data(&GITIGNORE_BYTES[start..end]);
            }
            InodeKind::BucketIndex => {
                let bytes = self.bucket_index_bytes_or_render();
                let start = usize::try_from(offset)
                    .unwrap_or(usize::MAX)
                    .min(bytes.len());
                let end = start.saturating_add(size as usize).min(bytes.len());
                reply.data(&bytes[start..end]);
            }
            InodeKind::TreeSymlink => {
                // Symlinks are read via `readlink(2)`, not `read(2)`. If the
                // kernel ever hands us a read() on one, it's almost
                // certainly a misrouted lookup — surface EINVAL.
                reply.error(fuser::Errno::from_i32(libc::EINVAL));
            }
            InodeKind::RealFile => {
                if let Some(buf) = self.write_buffers.get(&ino_u) {
                    let bytes = buf.as_slice();
                    let start = usize::try_from(offset)
                        .unwrap_or(usize::MAX)
                        .min(bytes.len());
                    let end = start.saturating_add(size as usize).min(bytes.len());
                    reply.data(&bytes[start..end]);
                    return;
                }
                let cached = match self.resolve_ino(ino_u) {
                    Ok(c) => c,
                    Err(e) => {
                        warn!(error = %e, ino = ino_u, "read failed");
                        reply.error(fuser::Errno::from_i32(fetch_errno(&e)));
                        return;
                    }
                };
                let bytes = cached.rendered.as_bytes();
                let start = usize::try_from(offset)
                    .unwrap_or(usize::MAX)
                    .min(bytes.len());
                let end = start.saturating_add(size as usize).min(bytes.len());
                reply.data(&bytes[start..end]);
            }
            InodeKind::RootIndex => {
                let bytes = self.mount_root_index_bytes_or_render();
                let start = usize::try_from(offset)
                    .unwrap_or(usize::MAX)
                    .min(bytes.len());
                let end = start.saturating_add(size as usize).min(bytes.len());
                reply.data(&bytes[start..end]);
            }
            InodeKind::TreeDirIndex => {
                let Some(dir_ino) = self.tree_index_ino_reverse.get(&ino_u).map(|v| *v) else {
                    reply.error(fuser::Errno::from_i32(libc::ENOENT));
                    return;
                };
                let Ok(snap) = self.tree.read() else {
                    reply.error(fuser::Errno::from_i32(libc::EIO));
                    return;
                };
                let bytes = self.tree_dir_index_bytes_or_render(dir_ino, &snap);
                let start = usize::try_from(offset)
                    .unwrap_or(usize::MAX)
                    .min(bytes.len());
                let end = start.saturating_add(size as usize).min(bytes.len());
                reply.data(&bytes[start..end]);
            }
            InodeKind::Unknown => reply.error(fuser::Errno::from_i32(libc::ENOENT)),
        }
    }

    fn readlink(&self, _req: &Request, ino: INodeNo, reply: ReplyData) {
        let ino_u = ino.0;
        match InodeKind::classify(ino_u) {
            InodeKind::TreeSymlink => {
                let Ok(snap) = self.tree.read() else {
                    reply.error(fuser::Errno::from_i32(libc::EIO));
                    return;
                };
                match snap.resolve_symlink(ino_u) {
                    Some(target) => reply.data(target.as_bytes()),
                    None => reply.error(fuser::Errno::from_i32(libc::ENOENT)),
                }
            }
            _ => reply.error(fuser::Errno::from_i32(libc::EINVAL)),
        }
    }

    // ------------------------------------------------------------------ //
    // Write path (Phase S). Bucket-scoped — only real-file inodes accept  //
    // writes; everything else rejects with EROFS or EISDIR/EINVAL.         //
    // ------------------------------------------------------------------ //

    #[allow(clippy::too_many_arguments)]
    fn setattr(
        &self,
        _req: &Request,
        ino: INodeNo,
        _mode: Option<u32>,
        _uid: Option<u32>,
        _gid: Option<u32>,
        size: Option<u64>,
        _atime: Option<fuser::TimeOrNow>,
        _mtime: Option<fuser::TimeOrNow>,
        _ctime: Option<SystemTime>,
        _fh: Option<FileHandle>,
        _crtime: Option<SystemTime>,
        _chgtime: Option<SystemTime>,
        _bkuptime: Option<SystemTime>,
        _flags: Option<fuser::BsdFileFlags>,
        reply: ReplyAttr,
    ) {
        let ino_u = ino.0;
        match InodeKind::classify(ino_u) {
            InodeKind::Root => reply.attr(&ATTR_TTL, &self.root_attr),
            InodeKind::Bucket => reply.attr(&ATTR_TTL, &self.bucket_attr),
            InodeKind::TreeRoot => reply.attr(&ATTR_TTL, &self.tree_attr),
            InodeKind::Gitignore => reply.attr(&ATTR_TTL, &self.gitignore_attr),
            InodeKind::BucketIndex => {
                // `_INDEX.md` is synthesized read-only (Phase 15, LD-15-05).
                // Deny truncate / chmod / chown / utimensat by returning
                // EROFS, matching the gitignore / tree-symlink discipline.
                reply.error(fuser::Errno::from_i32(libc::EROFS));
            }
            InodeKind::RootIndex | InodeKind::TreeDirIndex => {
                // Synthesized read-only files — deny all metadata mutations
                // (T-18-02, T-18-03).
                reply.error(fuser::Errno::from_i32(libc::EROFS));
            }
            InodeKind::TreeDir => {
                let attr = self.tree_dir_attr(ino_u);
                reply.attr(&ATTR_TTL, &attr);
            }
            InodeKind::TreeSymlink => {
                // Symlinks don't support setattr. Per POSIX kernel also
                // rarely routes here (lchmod etc.), but reject defensively.
                reply.error(fuser::Errno::from_i32(libc::EPERM));
            }
            InodeKind::RealFile => {
                if let Some(0) = size {
                    self.write_buffers.insert(ino_u, Vec::new());
                } else if let Some(new_size) = size {
                    let mut entry = self.write_buffers.entry(ino_u).or_insert_with(|| {
                        self.cache
                            .get(&ino_u)
                            .map(|c| c.rendered.as_bytes().to_vec())
                            .unwrap_or_default()
                    });
                    let target = usize::try_from(new_size).unwrap_or(usize::MAX);
                    entry.resize(target, 0);
                }
                if let Some(c) = self.cache.get(&ino_u) {
                    let cur_size = self
                        .write_buffers
                        .get(&ino_u)
                        .map_or(c.rendered.len() as u64, |b| b.len() as u64);
                    let attr = self.file_attr(ino_u, &c.issue, cur_size);
                    reply.attr(&ATTR_TTL, &attr);
                } else {
                    reply.error(fuser::Errno::from_i32(libc::ENOENT));
                }
            }
            InodeKind::Unknown => reply.error(fuser::Errno::from_i32(libc::ENOENT)),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn write(
        &self,
        _req: &Request,
        ino: INodeNo,
        _fh: FileHandle,
        offset: u64,
        data: &[u8],
        _write_flags: fuser::WriteFlags,
        _flags: fuser::OpenFlags,
        _lock_owner: Option<fuser::LockOwner>,
        reply: ReplyWrite,
    ) {
        let ino_u = ino.0;
        match InodeKind::classify(ino_u) {
            InodeKind::Root | InodeKind::Bucket | InodeKind::TreeRoot | InodeKind::TreeDir => {
                reply.error(fuser::Errno::from_i32(libc::EISDIR));
            }
            InodeKind::Gitignore
            | InodeKind::BucketIndex
            | InodeKind::TreeSymlink
            | InodeKind::RootIndex
            | InodeKind::TreeDirIndex => {
                reply.error(fuser::Errno::from_i32(libc::EROFS));
            }
            InodeKind::RealFile => {
                let offset_usize = usize::try_from(offset).unwrap_or(usize::MAX);
                let end = offset_usize.saturating_add(data.len());
                let mut entry = self.write_buffers.entry(ino_u).or_insert_with(|| {
                    self.cache
                        .get(&ino_u)
                        .map(|c| c.rendered.as_bytes().to_vec())
                        .unwrap_or_default()
                });
                if entry.len() < end {
                    entry.resize(end, 0);
                }
                entry[offset_usize..end].copy_from_slice(data);
                let written = u32::try_from(data.len()).unwrap_or(u32::MAX);
                reply.written(written);
            }
            InodeKind::Unknown => reply.error(fuser::Errno::from_i32(libc::ENOENT)),
        }
    }

    fn flush(
        &self,
        _req: &Request,
        _ino: INodeNo,
        _fh: FileHandle,
        _lock_owner: fuser::LockOwner,
        reply: ReplyEmpty,
    ) {
        reply.ok();
    }

    fn release(
        &self,
        _req: &Request,
        ino: INodeNo,
        _fh: FileHandle,
        _flags: fuser::OpenFlags,
        _lock_owner: Option<fuser::LockOwner>,
        _flush: bool,
        reply: ReplyEmpty,
    ) {
        let ino_u = ino.0;
        // Only real files have a write buffer worth draining; everything
        // else is either a dir, a synthetic file, or a symlink.
        if !matches!(InodeKind::classify(ino_u), InodeKind::RealFile) {
            reply.ok();
            return;
        }
        let Some((_, bytes)) = self.write_buffers.remove(&ino_u) else {
            reply.ok();
            return;
        };
        if bytes.is_empty() {
            reply.ok();
            return;
        }
        let text = match std::str::from_utf8(&bytes) {
            Ok(s) => s.to_owned(),
            Err(e) => {
                warn!(error = %e, ino = ino_u, "release: non-utf8 write buffer");
                reply.error(fuser::Errno::from_i32(libc::EIO));
                return;
            }
        };
        let Some(cached) = self.cache.get(&ino_u).map(|c| c.clone()) else {
            warn!(ino = ino_u, "release: no cached issue; cannot PATCH");
            reply.error(fuser::Errno::from_i32(libc::EIO));
            return;
        };
        let parsed = match frontmatter::parse(&text) {
            Ok(i) => i,
            Err(e) => {
                warn!(error = %e, ino = ino_u, "release: parse failed");
                reply.error(fuser::Errno::from_i32(libc::EIO));
                return;
            }
        };
        let meta = ServerMetadata {
            id: cached.issue.id,
            created_at: cached.issue.created_at,
            updated_at: cached.issue.updated_at,
            version: cached.issue.version,
        };
        let untainted = sanitize(Tainted::new(parsed), meta);
        let version = cached.issue.version;
        let id = cached.issue.id;
        let result = self.rt.block_on(update_issue_with_timeout(
            &self.backend,
            &self.project,
            id,
            untainted,
            Some(version),
        ));
        match result {
            Ok(updated) => {
                if let Ok(rendered) = frontmatter::render(&updated) {
                    self.cache.insert(
                        ino_u,
                        Arc::new(CachedFile {
                            issue: updated,
                            rendered,
                            body_fetched: true,
                        }),
                    );
                }
                reply.ok();
            }
            Err(FetchError::Conflict { current }) => {
                warn!(
                    ino = ino_u,
                    current, "release: 409 conflict — user must git pull --rebase"
                );
                reply.error(fuser::Errno::from_i32(libc::EIO));
            }
            Err(e) => {
                warn!(error = %e, ino = ino_u, "release: PATCH failed");
                reply.error(fuser::Errno::from_i32(fetch_errno(&e)));
            }
        }
    }

    fn create(
        &self,
        _req: &Request,
        parent: INodeNo,
        name: &OsStr,
        _mode: u32,
        _umask: u32,
        _flags: i32,
        reply: ReplyCreate,
    ) {
        // Only the bucket directory accepts new files. Root, tree/, tree
        // dirs are synthesized and read-only.
        if !matches!(InodeKind::classify(parent.0), InodeKind::Bucket) {
            reply.error(fuser::Errno::from_i32(libc::EROFS));
            return;
        }
        let Some(name_str) = name.to_str() else {
            reply.error(fuser::Errno::from_i32(libc::EINVAL));
            return;
        };
        // Refuse to let the user shadow the synthesized `_INDEX.md`
        // with a real issue file — LD-15-05. EACCES distinguishes this
        // from the generic "not an issue filename" EINVAL below.
        if name_str == BUCKET_INDEX_FILENAME {
            reply.error(fuser::Errno::from_i32(libc::EACCES));
            return;
        }
        let Ok(id) = validate_issue_filename(name_str) else {
            reply.error(fuser::Errno::from_i32(libc::EINVAL));
            return;
        };
        let now = chrono::Utc::now();
        let placeholder = Issue {
            id,
            title: format!("issue {}", id.0),
            status: IssueStatus::Open,
            assignee: None,
            labels: vec![],
            created_at: now,
            updated_at: now,
            version: 0,
            body: String::new(),
            parent_id: None,
        };
        let meta = ServerMetadata {
            id,
            created_at: now,
            updated_at: now,
            version: 0,
        };
        let untainted = sanitize(Tainted::new(placeholder), meta);
        let result = self.rt.block_on(create_issue_with_timeout(
            &self.backend,
            &self.project,
            untainted,
        ));
        match result {
            Ok(new_issue) => {
                let ino = self.registry.intern(new_issue.id);
                let rendered = match frontmatter::render(&new_issue) {
                    Ok(s) => s,
                    Err(e) => {
                        warn!(error = %e, "create: render failed");
                        reply.error(fuser::Errno::from_i32(libc::EIO));
                        return;
                    }
                };
                let size = rendered.len() as u64;
                let attr = self.file_attr(ino, &new_issue, size);
                self.cache.insert(
                    ino,
                    Arc::new(CachedFile {
                        issue: new_issue,
                        rendered,
                        body_fetched: true,
                    }),
                );
                let fh = self.next_fh.fetch_add(1, Ordering::Relaxed);
                reply.created(
                    &ENTRY_TTL,
                    &attr,
                    fuser::Generation(0),
                    FileHandle(fh),
                    fuser::FopenFlags::empty(),
                );
            }
            Err(e) => {
                warn!(error = %e, name = %name_str, "create: POST failed");
                reply.error(fuser::Errno::from_i32(fetch_errno(&e)));
            }
        }
    }

    fn unlink(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEmpty) {
        if !matches!(InodeKind::classify(parent.0), InodeKind::Bucket) {
            reply.error(fuser::Errno::from_i32(libc::EROFS));
            return;
        }
        let Some(name_str) = name.to_str() else {
            reply.error(fuser::Errno::from_i32(libc::EINVAL));
            return;
        };
        // Refuse `rm <bucket>/_INDEX.md` — the synthesized index is
        // read-only (LD-15-05). Returning EACCES (rather than EROFS)
        // matches the `create` error for symmetry.
        if name_str == BUCKET_INDEX_FILENAME {
            reply.error(fuser::Errno::from_i32(libc::EACCES));
            return;
        }
        let Ok(id) = validate_issue_filename(name_str) else {
            reply.error(fuser::Errno::from_i32(libc::EINVAL));
            return;
        };
        if let Some(ino) = self.registry.lookup_id(id) {
            self.cache.remove(&ino);
            self.write_buffers.remove(&ino);
        }
        reply.ok();
    }
}

/// Shared helper used by `lookup` for `TreeRoot` and `TreeDir` parents:
/// search `entries` for `name`, build the appropriate `FileAttr`, reply.
impl ReposixFs {
    fn reply_tree_entry(
        &self,
        snap: &TreeSnapshot,
        entries: &[crate::tree::TreeEntry],
        name: &str,
        reply: ReplyEntry,
    ) {
        for entry in entries {
            match entry {
                crate::tree::TreeEntry::Dir(dir_ino) => {
                    if let Some(dir) = snap.resolve_dir(*dir_ino) {
                        if dir.name == name {
                            let attr = self.tree_dir_attr(*dir_ino);
                            reply.entry(&ENTRY_TTL, &attr, fuser::Generation(0));
                            return;
                        }
                    }
                }
                crate::tree::TreeEntry::Symlink {
                    ino,
                    name: n,
                    target,
                } => {
                    if n == name {
                        let attr = self.symlink_attr(*ino, target);
                        reply.entry(&ENTRY_TTL, &attr, fuser::Generation(0));
                        return;
                    }
                }
            }
        }
        reply.error(fuser::Errno::from_i32(libc::ENOENT));
    }
}

/// Flatten a slice of `TreeEntry` into the `(ino, FileType, name)` triples
/// that `ReplyDirectory::add` consumes. Used by `readdir`.
fn collect_tree_entries(
    snap: &TreeSnapshot,
    entries: &[crate::tree::TreeEntry],
    out: &mut Vec<(u64, FileType, String)>,
) {
    for entry in entries {
        match entry {
            crate::tree::TreeEntry::Dir(dir_ino) => {
                if let Some(dir) = snap.resolve_dir(*dir_ino) {
                    out.push((dir.ino, FileType::Directory, dir.name.clone()));
                }
            }
            crate::tree::TreeEntry::Symlink { ino, name, .. } => {
                out.push((*ino, FileType::Symlink, name.clone()));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    //! Tests for the Phase 14 Wave B1 helpers: `backend_err_to_fetch`'s
    //! version-mismatch arm (Q1/Task B1.2) and the new
    //! `update_issue_with_timeout` / `create_issue_with_timeout` wrappers
    //! (Q3/Task B1.3).

    use super::*;
    use chrono::TimeZone;
    use reposix_core::backend::sim::SimBackend;
    use std::time::Instant;
    use wiremock::matchers::{any, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn sample_untainted() -> Untainted<Issue> {
        let t = chrono::Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
        sanitize(
            Tainted::new(Issue {
                id: IssueId(0),
                title: "x".into(),
                status: IssueStatus::Open,
                assignee: None,
                labels: vec![],
                created_at: t,
                updated_at: t,
                version: 0,
                body: String::new(),
                parent_id: None,
            }),
            ServerMetadata {
                id: IssueId(1),
                created_at: t,
                updated_at: t,
                version: 1,
            },
        )
    }

    #[test]
    fn backend_err_to_fetch_maps_version_mismatch_with_current() {
        // Task B1.2 proof: a `reposix_core::Error::Other("version mismatch:
        // <json>")` surfaces as `FetchError::Conflict { current: N }`
        // with N extracted from the JSON tail. Mirrors the sim's
        // `{"error":"version_mismatch","current":7,"sent":"1"}` body shape.
        let input = reposix_core::Error::Other(
            "version mismatch: {\"error\":\"version_mismatch\",\"current\":7,\"sent\":\"1\"}"
                .into(),
        );
        let out = backend_err_to_fetch(input);
        match out {
            FetchError::Conflict { current } => assert_eq!(current, 7),
            other => panic!("expected Conflict {{ current: 7 }}, got {other:?}"),
        }
    }

    #[test]
    fn backend_err_to_fetch_maps_malformed_version_mismatch_to_current_zero() {
        // Graceful-degradation fallback: if the body JSON is unparseable we
        // still surface a Conflict (current=0) rather than demoting to
        // FetchError::Core, so the release diagnostic path always fires on
        // the 409 signal. Mirrors the old fetch.rs:246 unwrap_or.
        let input = reposix_core::Error::Other("version mismatch: not json at all".into());
        let out = backend_err_to_fetch(input);
        match out {
            FetchError::Conflict { current } => assert_eq!(current, 0),
            other => panic!("expected Conflict {{ current: 0 }}, got {other:?}"),
        }
    }

    #[test]
    fn backend_err_to_fetch_maps_not_found() {
        // Regression guard: the "not found" arm was there before Phase 14
        // and must keep mapping cleanly.
        let input = reposix_core::Error::Other("not found: http://127.0.0.1/x".into());
        assert!(matches!(backend_err_to_fetch(input), FetchError::NotFound));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn update_issue_with_timeout_times_out_within_budget() {
        // Re-home of fetch.rs::patch_issue_times_out_within_budget onto the
        // new wrapper. A 10-second backend delay + 5s outer timeout must
        // surface Err(FetchError::Timeout) within ~5.5s wall clock. This
        // proves the outer `tokio::time::timeout` wrapper restores the
        // defence-in-depth the old `fetch::patch_issue` provided on top of
        // reqwest's 5s total_timeout (see 14-RESEARCH.md#Q3).
        let server = MockServer::start().await;
        Mock::given(any())
            .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(10)))
            .mount(&server)
            .await;
        let backend: Arc<dyn IssueBackend> = Arc::new(SimBackend::new(server.uri()).unwrap());
        let t0 = Instant::now();
        let err =
            update_issue_with_timeout(&backend, "demo", IssueId(1), sample_untainted(), Some(1))
                .await
                .unwrap_err();
        let elapsed = t0.elapsed();
        // Either our outer timeout fired (FetchError::Timeout) or reqwest's
        // inner 5s total_timeout fired first (FetchError::Transport with
        // is_timeout()); both prove the 5s ceiling holds.
        let ok = matches!(err, FetchError::Timeout)
            || matches!(&err, FetchError::Transport(e) if e.is_timeout());
        assert!(ok, "expected timeout-flavored error, got {err:?}");
        assert!(
            elapsed < Duration::from_millis(5_800),
            "should return within 5.5s; took {elapsed:?}"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn create_issue_with_timeout_times_out_within_budget() {
        // Symmetric POST timeout guard; ensures the new create wrapper has
        // the same 5s ceiling as update.
        let server = MockServer::start().await;
        Mock::given(any())
            .respond_with(ResponseTemplate::new(201).set_delay(Duration::from_secs(10)))
            .mount(&server)
            .await;
        let backend: Arc<dyn IssueBackend> = Arc::new(SimBackend::new(server.uri()).unwrap());
        let t0 = Instant::now();
        let err = create_issue_with_timeout(&backend, "demo", sample_untainted())
            .await
            .unwrap_err();
        let elapsed = t0.elapsed();
        let ok = matches!(err, FetchError::Timeout)
            || matches!(&err, FetchError::Transport(e) if e.is_timeout());
        assert!(ok, "expected timeout-flavored error, got {err:?}");
        assert!(
            elapsed < Duration::from_millis(5_800),
            "should return within 5.5s; took {elapsed:?}"
        );
    }

    // ------------------------------------------------------------------ //
    // Phase 15-A: bucket `_INDEX.md` render pure-function tests.          //
    // ------------------------------------------------------------------ //

    fn mk_issue(id: u64, title: &str, day: (i32, u32, u32)) -> Issue {
        let (y, m, d) = day;
        let t = chrono::Utc.with_ymd_and_hms(y, m, d, 12, 0, 0).unwrap();
        Issue {
            id: IssueId(id),
            title: title.into(),
            status: IssueStatus::Open,
            assignee: None,
            labels: vec![],
            created_at: t,
            updated_at: t,
            version: 1,
            body: String::new(),
            parent_id: None,
        }
    }

    #[test]
    fn bucket_index_renders_frontmatter_and_table() {
        // Two sample issues; verify frontmatter keys, header line, table
        // header row, separator row, and both data rows.
        let issues = vec![
            mk_issue(65_916, "Architecture notes", (2026, 4, 14)),
            mk_issue(131_192, "Welcome to reposix", (2026, 4, 14)),
        ];
        let generated = chrono::Utc
            .with_ymd_and_hms(2026, 4, 14, 17, 15, 0)
            .unwrap();
        let bytes = render_bucket_index(&issues, "simulator", "demo", "issues", generated);
        let text = std::str::from_utf8(&bytes).expect("utf-8");

        // Frontmatter — fence + four named keys + count.
        assert!(text.starts_with("---\n"), "missing open fence: {text}");
        assert!(
            text.contains("backend: simulator\n"),
            "missing backend key: {text}"
        );
        assert!(
            text.contains("project: demo\n"),
            "missing project key: {text}"
        );
        assert!(
            text.contains("issue_count: 2\n"),
            "missing issue_count key: {text}"
        );
        assert!(
            text.contains("generated_at: 2026-04-14T17:15:00Z\n"),
            "missing generated_at key: {text}"
        );
        assert!(text.contains("\n---\n\n"), "missing close fence: {text}");

        // Header line.
        assert!(
            text.contains("# Index of issues/ — demo (2 issues)\n"),
            "missing markdown header: {text}"
        );

        // Table header + separator + both data rows.
        assert!(
            text.contains("| id | status | title | updated |\n"),
            "missing table header: {text}"
        );
        assert!(
            text.contains("| --- | --- | --- | --- |\n"),
            "missing table separator: {text}"
        );
        assert!(
            text.contains("| 65916 | open | Architecture notes | 2026-04-14 |\n"),
            "missing row for 65916: {text}"
        );
        assert!(
            text.contains("| 131192 | open | Welcome to reposix | 2026-04-14 |\n"),
            "missing row for 131192: {text}"
        );
    }

    #[test]
    fn bucket_index_row_order_is_ascending_by_id() {
        // Feed three issues in reverse id order; the rendered table must
        // still list them ascending. Matches LD-15-10 (deterministic order).
        let issues = vec![
            mk_issue(425_985, "Demo plan", (2026, 4, 14)),
            mk_issue(65_916, "Architecture notes", (2026, 4, 14)),
            mk_issue(131_192, "Welcome to reposix", (2026, 4, 14)),
        ];
        let generated = chrono::Utc
            .with_ymd_and_hms(2026, 4, 14, 17, 15, 0)
            .unwrap();
        let bytes = render_bucket_index(&issues, "confluence", "REPOSIX", "pages", generated);
        let text = std::str::from_utf8(&bytes).expect("utf-8");

        let pos_65916 = text.find("| 65916 ").expect("65916 row missing");
        let pos_131192 = text.find("| 131192 ").expect("131192 row missing");
        let pos_425985 = text.find("| 425985 ").expect("425985 row missing");
        assert!(
            pos_65916 < pos_131192 && pos_131192 < pos_425985,
            "rows out of order: 65916@{pos_65916} < 131192@{pos_131192} < 425985@{pos_425985}\n{text}"
        );
        // Also pin the pluralised bucket noun to "pages" for Confluence.
        assert!(
            text.contains("# Index of pages/ — REPOSIX (3 pages)\n"),
            "missing pages-flavoured header: {text}"
        );
    }

    #[test]
    fn bucket_index_empty_list_is_valid_markdown() {
        // Zero issues must still render a valid document — frontmatter
        // with issue_count: 0, markdown header, table header, table
        // separator, and NO data rows. Agents reading an empty bucket
        // should not see a parse error.
        let generated = chrono::Utc.with_ymd_and_hms(2026, 4, 14, 0, 0, 0).unwrap();
        let bytes = render_bucket_index(&[], "simulator", "demo", "issues", generated);
        let text = std::str::from_utf8(&bytes).expect("utf-8");

        assert!(
            text.contains("issue_count: 0\n"),
            "missing zero count: {text}"
        );
        assert!(
            text.contains("# Index of issues/ — demo (0 issues)\n"),
            "missing zero-count header: {text}"
        );
        assert!(
            text.contains("| id | status | title | updated |\n"),
            "missing table header: {text}"
        );
        assert!(
            text.contains("| --- | --- | --- | --- |\n"),
            "missing table separator: {text}"
        );
        // No data rows — the only table-pipe lines are the header and
        // separator, so the trailing-newline byte count should be
        // bounded: header+sep occupies 2 lines, plus the usual pre-table
        // content. Assert "no row-shaped line with a decimal id".
        for line in text.lines() {
            // Body rows start with `| <digit> ...`. The header row
            // starts with `| id ...`, separator with `| --- ...`.
            assert!(
                !(line.starts_with("| ")
                    && !line.starts_with("| id ")
                    && !line.starts_with("| --- ")),
                "unexpected table data row in empty-list index: {line}\n{text}"
            );
        }
    }

    #[test]
    fn bucket_index_escapes_pipe_in_title() {
        // A `|` inside a title must be escaped to `\|` so the row's
        // column count survives. Also verifies newlines fold to spaces.
        let issues = vec![mk_issue(7, "foo | bar\nbaz", (2026, 4, 14))];
        let generated = chrono::Utc.with_ymd_and_hms(2026, 4, 14, 0, 0, 0).unwrap();
        let bytes = render_bucket_index(&issues, "simulator", "demo", "issues", generated);
        let text = std::str::from_utf8(&bytes).expect("utf-8");
        assert!(
            text.contains(r"foo \| bar baz"),
            "pipe not escaped / newline not folded: {text}"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn update_issue_with_timeout_happy_path_returns_issue() {
        // Positive-path coverage: a fast 200 from the backend lands as
        // `Ok(Issue)` through the wrapper. Cheap companion to the timeout
        // tests — together they pin both limbs of the `tokio::time::timeout`
        // match.
        let server = MockServer::start().await;
        let body = serde_json::json!({
            "id": 1,
            "title": "hello",
            "status": "open",
            "labels": [],
            "created_at": "2026-04-13T00:00:00Z",
            "updated_at": "2026-04-13T00:00:00Z",
            "version": 2,
            "body": ""
        });
        Mock::given(method("PATCH"))
            .and(path("/projects/demo/issues/1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(&server)
            .await;
        let backend: Arc<dyn IssueBackend> = Arc::new(SimBackend::new(server.uri()).unwrap());
        let got =
            update_issue_with_timeout(&backend, "demo", IssueId(1), sample_untainted(), Some(1))
                .await
                .expect("update");
        assert_eq!(got.id, IssueId(1));
        assert_eq!(got.version, 2);
    }

    // ------------------------------------------------------------------ //
    // Phase 18-A: tree-index + mount-root index render pure-function tests. //
    // ------------------------------------------------------------------ //

    #[test]
    fn render_tree_index_frontmatter_and_table() {
        // One dir with 2 symlink children → 2 data rows.
        use crate::tree::{TreeDir, TreeEntry, TREE_DIR_INO_BASE, TREE_SYMLINK_INO_BASE};
        let children = vec![
            TreeEntry::Symlink {
                ino: TREE_SYMLINK_INO_BASE + 1,
                name: "welcome.md".to_owned(),
                target: "../pages/00000000001.md".to_owned(),
            },
            TreeEntry::Symlink {
                ino: TREE_SYMLINK_INO_BASE + 2,
                name: "arch-notes.md".to_owned(),
                target: "../pages/00000000002.md".to_owned(),
            },
        ];
        let root_dir = TreeDir {
            ino: TREE_DIR_INO_BASE + 1,
            name: "demo-space".to_owned(),
            children,
            depth: 0,
        };
        let snap = crate::tree::TreeSnapshot::default();
        let generated = chrono::Utc.with_ymd_and_hms(2026, 4, 15, 10, 0, 0).unwrap();
        let bytes = render_tree_index(&root_dir, &snap, "demo", generated);
        let text = std::str::from_utf8(&bytes).expect("utf-8");

        // Frontmatter
        assert!(text.starts_with("---\n"), "missing open fence: {text}");
        assert!(text.contains("kind: tree-index\n"), "missing kind: {text}");
        assert!(text.contains("project: demo\n"), "missing project: {text}");
        assert!(
            text.contains("subtree: demo-space\n"),
            "missing subtree: {text}"
        );
        assert!(
            text.contains("entry_count: 2\n"),
            "missing entry_count: {text}"
        );
        assert!(
            text.contains("generated_at: 2026-04-15T10:00:00Z\n"),
            "missing generated_at: {text}"
        );
        assert!(text.contains("\n---\n\n"), "missing close fence: {text}");

        // Table header + separator + both data rows
        assert!(
            text.contains("| depth | name | target |\n"),
            "missing table header: {text}"
        );
        assert!(
            text.contains("| --- | --- | --- |\n"),
            "missing table separator: {text}"
        );
        assert!(
            text.contains("| 0 | welcome.md | ../pages/00000000001.md |"),
            "missing welcome row: {text}"
        );
        assert!(
            text.contains("| 0 | arch-notes.md | ../pages/00000000002.md |"),
            "missing arch-notes row: {text}"
        );
    }

    #[test]
    fn tree_index_full_dfs() {
        // 3-level nested snapshot: root_dir → child_dir → grandchild symlink
        // + root_dir direct symlink.  DFS must visit all 3 entries.
        use crate::tree::{TreeEntry, TreeSnapshot};

        // Build from real issues so resolve_dir works.
        // parent(id=1) → child(id=2); child(id=2) → grandchild(id=3).
        let t = chrono::Utc.with_ymd_and_hms(2026, 4, 15, 0, 0, 0).unwrap();
        let issues = vec![
            Issue {
                id: IssueId(1),
                title: "parent".to_owned(),
                status: IssueStatus::Open,
                assignee: None,
                labels: vec![],
                created_at: t,
                updated_at: t,
                version: 1,
                body: String::new(),
                parent_id: None,
            },
            Issue {
                id: IssueId(2),
                title: "child".to_owned(),
                status: IssueStatus::Open,
                assignee: None,
                labels: vec![],
                created_at: t,
                updated_at: t,
                version: 1,
                body: String::new(),
                parent_id: Some(IssueId(1)),
            },
            Issue {
                id: IssueId(3),
                title: "grandchild".to_owned(),
                status: IssueStatus::Open,
                assignee: None,
                labels: vec![],
                created_at: t,
                updated_at: t,
                version: 1,
                body: String::new(),
                parent_id: Some(IssueId(2)),
            },
        ];
        let snap = TreeSnapshot::build("pages", &issues);
        // The root_entries should contain one Dir entry (the "parent" dir).
        let root_entries = snap.root_entries();
        assert_eq!(root_entries.len(), 1, "expected 1 root entry");
        let parent_ino = match &root_entries[0] {
            TreeEntry::Dir(ino) => *ino,
            other @ TreeEntry::Symlink { .. } => panic!("expected Dir, got {other:?}"),
        };
        let parent_dir = snap.resolve_dir(parent_ino).expect("parent dir");

        let generated = chrono::Utc.with_ymd_and_hms(2026, 4, 15, 0, 0, 0).unwrap();
        let bytes = render_tree_index(parent_dir, &snap, "demo", generated);
        let text = std::str::from_utf8(&bytes).expect("utf-8");

        // DFS should find: _self.md (parent itself), child/ dir, _self.md
        // (child itself), grandchild symlink — at minimum 3+ entries.
        // The exact count depends on TreeSnapshot internals but must be >= 3.
        let data_rows: Vec<&str> = text
            .lines()
            .filter(|l| {
                l.starts_with("| ") && !l.starts_with("| depth ") && !l.starts_with("| --- ")
            })
            .collect();
        assert!(
            data_rows.len() >= 3,
            "expected >= 3 DFS rows, got {}: {text}",
            data_rows.len()
        );
        // _self.md for parent must be at depth 0 and appear before child/.
        let self_md_pos = data_rows
            .iter()
            .position(|l| l.contains("_self.md") && l.contains("| 0 |"));
        let child_dir_pos = data_rows
            .iter()
            .position(|l| l.contains("child/") && l.contains("| 0 |"));
        assert!(
            self_md_pos.is_some(),
            "_self.md row at depth 0 must be present: {text}"
        );
        assert!(
            child_dir_pos.is_some(),
            "child/ row at depth 0 must be present: {text}"
        );
        assert!(
            self_md_pos.unwrap() < child_dir_pos.unwrap(),
            "_self.md must appear before child/ in pre-order DFS: {text}"
        );
        // Grandchild entries must have depth 1.
        assert!(
            data_rows.iter().any(|l| l.contains("| 1 |")),
            "grandchild must have depth 1: {text}"
        );
    }

    #[test]
    fn tree_index_empty() {
        // Empty children → entry_count: 0 and no data rows.
        use crate::tree::{TreeDir, TREE_DIR_INO_BASE};
        let root_dir = TreeDir {
            ino: TREE_DIR_INO_BASE + 1,
            name: "empty-space".to_owned(),
            children: vec![],
            depth: 0,
        };
        let snap = crate::tree::TreeSnapshot::default();
        let generated = chrono::Utc.with_ymd_and_hms(2026, 4, 15, 0, 0, 0).unwrap();
        let bytes = render_tree_index(&root_dir, &snap, "demo", generated);
        let text = std::str::from_utf8(&bytes).expect("utf-8");

        assert!(
            text.contains("entry_count: 0\n"),
            "missing zero count: {text}"
        );
        // Table header and separator present, no data rows.
        assert!(
            text.contains("| depth | name | target |\n"),
            "missing table header: {text}"
        );
        assert!(
            text.contains("| --- | --- | --- |\n"),
            "missing separator: {text}"
        );
        // No data rows: no line starting with `| ` that isn't the header or separator.
        for line in text.lines() {
            assert!(
                !(line.starts_with("| ")
                    && !line.starts_with("| depth ")
                    && !line.starts_with("| --- ")),
                "unexpected data row in empty index: {line}\n{text}"
            );
        }
    }

    #[test]
    fn render_mount_root_index_frontmatter_and_table() {
        // tree_present=true, 3 issues → all frontmatter keys + 3 table rows.
        let generated = chrono::Utc.with_ymd_and_hms(2026, 4, 15, 12, 0, 0).unwrap();
        let bytes = render_mount_root_index("simulator", "demo", "issues", 3, true, generated);
        let text = std::str::from_utf8(&bytes).expect("utf-8");

        assert!(text.starts_with("---\n"), "missing open fence: {text}");
        assert!(text.contains("kind: mount-index\n"), "missing kind: {text}");
        assert!(
            text.contains("backend: simulator\n"),
            "missing backend: {text}"
        );
        assert!(text.contains("project: demo\n"), "missing project: {text}");
        assert!(text.contains("bucket: issues\n"), "missing bucket: {text}");
        assert!(
            text.contains("issue_count: 3\n"),
            "missing issue_count: {text}"
        );
        assert!(
            text.contains("generated_at: 2026-04-15T12:00:00Z\n"),
            "missing generated_at: {text}"
        );
        assert!(text.contains("\n---\n\n"), "missing close fence: {text}");

        // Table rows
        assert!(
            text.contains("| entry | kind | count |\n"),
            "missing table header: {text}"
        );
        assert!(
            text.contains("| .gitignore | file | — |"),
            "missing gitignore row: {text}"
        );
        assert!(
            text.contains("| issues/ | directory | 3 |"),
            "missing issues row: {text}"
        );
        assert!(
            text.contains("| tree/ | directory | — |"),
            "missing tree row: {text}"
        );
    }

    #[test]
    fn mount_root_index_no_tree_row() {
        // tree_present=false → no `tree/` row.
        let generated = chrono::Utc.with_ymd_and_hms(2026, 4, 15, 0, 0, 0).unwrap();
        let bytes = render_mount_root_index("simulator", "demo", "issues", 0, false, generated);
        let text = std::str::from_utf8(&bytes).expect("utf-8");

        assert!(
            !text.contains("| tree/ |"),
            "`tree/` row must be absent when tree_present=false: {text}"
        );
        // Other rows still present
        assert!(
            text.contains("| .gitignore | file | — |"),
            "missing gitignore row: {text}"
        );
        assert!(
            text.contains("| issues/ | directory | 0 |"),
            "missing issues row: {text}"
        );
    }

    #[test]
    fn tree_dir_index_ino_is_stable() {
        // Call tree_dir_index_ino(42) twice on the same ReposixFs; both
        // calls must return the same inode (idempotent per-dir allocation).
        use reposix_core::backend::sim::SimBackend;
        let server = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async { wiremock::MockServer::start().await });
        let backend: Arc<dyn IssueBackend> = Arc::new(SimBackend::new(server.uri()).unwrap());
        let fs = ReposixFs::new(backend, server.uri(), "demo".to_owned()).expect("ReposixFs::new");

        let first = fs.tree_dir_index_ino(42);
        let second = fs.tree_dir_index_ino(42);
        assert_eq!(
            first, second,
            "tree_dir_index_ino must be idempotent for same dir_ino"
        );
        // Different dir_inos must get different index inodes.
        let other = fs.tree_dir_index_ino(99);
        assert_ne!(
            first, other,
            "distinct dir_inos must get distinct index inodes"
        );
        // Reverse map must be populated.
        assert_eq!(
            fs.tree_index_ino_reverse.get(&first).map(|v| *v),
            Some(42u64),
            "reverse map must resolve first inode back to dir_ino 42"
        );
    }
}
