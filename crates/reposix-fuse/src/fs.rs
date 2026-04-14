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
//! # Backend seam (Phase 10)
//!
//! The read path speaks to a `dyn IssueBackend` trait object rather than
//! the simulator's REST shape directly, so the same daemon can mount the
//! simulator (`SimBackend`) or real GitHub (`GithubReadOnlyBackend`). The
//! write path still speaks the simulator REST shape via [`fetch`]; a v0.3
//! cleanup will lift it onto the trait too.
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
//! Read-path backend calls are wrapped in a 5-second `tokio::time::timeout`
//! inside `list_issues_with_timeout` / `get_issue_with_timeout`. Write-path
//! `fetch::*` helpers carry their own 5s ceiling. On timeout we reply
//! `libc::EIO` so the kernel never hangs on a dead backend.
//!
//! # Egress discipline (SG-03)
//!
//! Every PATCH / POST body goes through
//! `Tainted::new(parsed_issue).then(sanitize(...))` before serialization;
//! the sanitized `Untainted<Issue>` is then serialized via the
//! `EgressPayload` shape inside `fetch.rs`, so server-controlled fields
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
use reposix_core::http::{client, ClientOpts, HttpClient};
use reposix_core::path::validate_issue_filename;
use reposix_core::{
    backend::BackendFeature, frontmatter, sanitize, Issue, IssueBackend, IssueId, IssueStatus,
    ServerMetadata, Tainted,
};
use tokio::runtime::Runtime;
use tracing::warn;

use crate::fetch::{patch_issue, post_issue, FetchError};
use crate::inode::{
    InodeRegistry, BUCKET_DIR_INO, FIRST_ISSUE_INODE, GITIGNORE_INO, ROOT_INO, TREE_ROOT_INO,
};
use crate::tree::{TreeSnapshot, TREE_DIR_INO_BASE, TREE_SYMLINK_INO_BASE};

/// Exact bytes served for `mount/.gitignore`. Const; no runtime input.
/// The trailing newline is mandatory per POSIX text-file convention and
/// matches what tools like `git check-ignore` expect.
const GITIGNORE_BYTES: &[u8] = b"/tree/\n";

/// SG-07 ceilings for read-path backend calls. A dead backend MUST
/// surface EIO to the kernel within these budgets so callbacks never
/// wedge the VFS.
const READ_GET_TIMEOUT: Duration = Duration::from_secs(5);
const READ_LIST_TIMEOUT: Duration = Duration::from_secs(15);

/// Map a `reposix_core::Error` from an `IssueBackend` call into a
/// [`FetchError`].
fn backend_err_to_fetch(e: reposix_core::Error) -> FetchError {
    match e {
        reposix_core::Error::InvalidOrigin(o) => FetchError::Origin(o),
        reposix_core::Error::Http(t) => FetchError::Transport(t),
        reposix_core::Error::Json(j) => FetchError::Parse(j),
        reposix_core::Error::Other(msg) if msg.starts_with("not found") => FetchError::NotFound,
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

/// TTLs applied to every reply.
const ENTRY_TTL: Duration = Duration::from_secs(1);
const ATTR_TTL: Duration = Duration::from_secs(1);

/// Render the canonical on-disk filename for an issue: 11-digit
/// zero-padded decimal + `.md`. Matches the padding codified by Wave B2
/// in `tree::symlink_target` so every symlink target finds the real file.
fn issue_filename(id: IssueId) -> String {
    format!("{:011}.md", id.0)
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
    /// A real issue file in the bucket — `FIRST_ISSUE_INODE..TREE_DIR_INO_BASE`.
    RealFile,
    /// An interior tree directory — `TREE_DIR_INO_BASE..TREE_SYMLINK_INO_BASE`.
    TreeDir,
    /// A tree leaf symlink or `_self.md` — `TREE_SYMLINK_INO_BASE..`.
    TreeSymlink,
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

/// FUSE filesystem backed by an [`IssueBackend`] trait object for the read
/// path and by the simulator's REST shape (via [`fetch`]) for the write
/// path.
pub struct ReposixFs {
    /// Tokio runtime owned by the FS; used for `block_on` on callbacks.
    rt: Arc<Runtime>,
    /// Read-path backend (Phase 10).
    backend: Arc<dyn IssueBackend>,
    /// Sealed allowlisted HTTP client (SG-01) used by the write path.
    http: Arc<HttpClient>,
    /// Simulator origin used by the write path.
    origin: String,
    /// Project slug (sim) or `owner/repo` (github).
    project: String,
    /// `X-Reposix-Agent` header value (SG-05 audit attribution).
    agent: String,
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
}

impl std::fmt::Debug for ReposixFs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReposixFs")
            .field("backend", &self.backend.name())
            .field("origin", &self.origin)
            .field("project", &self.project)
            .field("agent", &self.agent)
            .field("bucket", &self.bucket)
            .field("hierarchy_feature", &self.hierarchy_feature)
            .finish_non_exhaustive()
    }
}

impl ReposixFs {
    /// Build a new FUSE filesystem whose read path is served by `backend`.
    ///
    /// # Errors
    /// Returns any error constructing the Tokio runtime or the sealed
    /// [`HttpClient`] (e.g. `REPOSIX_ALLOWED_ORIGINS` un-parseable).
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
        let http = Arc::new(client(ClientOpts::default())?);
        let agent = format!("reposix-fuse-{}", std::process::id());
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
            http,
            origin,
            project,
            agent,
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
    #[allow(clippy::unused_self)]
    fn symlink_attr(&self, ino: u64, target: &str) -> FileAttr {
        let now = SystemTime::now();
        FileAttr {
            ino: INodeNo(ino),
            size: target.len() as u64,
            blocks: 0,
            atime: now,
            mtime: now,
            ctime: now,
            crtime: now,
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

        Ok(issues)
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
        | FetchError::Status(_)
        | FetchError::Parse(_)
        | FetchError::Core(_)
        | FetchError::Conflict { .. }
        | FetchError::BadRequest(_) => libc::EIO,
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
                reply.error(fuser::Errno::from_i32(libc::ENOENT));
            }
            InodeKind::Bucket => {
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
                self.reply_tree_entry(&snap, &dir.children, name_str, reply);
            }
            InodeKind::Gitignore | InodeKind::RealFile | InodeKind::TreeSymlink => {
                reply.error(fuser::Errno::from_i32(libc::ENOTDIR));
            }
            InodeKind::Unknown => reply.error(fuser::Errno::from_i32(libc::ENOENT)),
        }
    }

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
                let mut out: Vec<(u64, FileType, String)> = Vec::with_capacity(sorted.len() + 2);
                out.push((BUCKET_DIR_INO, FileType::Directory, ".".to_owned()));
                out.push((ROOT_INO, FileType::Directory, "..".to_owned()));
                for issue in &sorted {
                    let ino = self.registry.intern(issue.id);
                    out.push((ino, FileType::RegularFile, issue_filename(issue.id)));
                }
                out
            }
            InodeKind::TreeRoot => {
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
                let mut out: Vec<(u64, FileType, String)> = Vec::with_capacity(dir.children.len() + 2);
                out.push((dir.ino, FileType::Directory, ".".to_owned()));
                // Parent inode — we don't track the reverse relation in the
                // snapshot, so use `TREE_ROOT_INO` as a conservative default.
                // The kernel follows explicit paths, not `..` entries, so
                // this is cosmetic only (matters for `ls -la` display).
                out.push((TREE_ROOT_INO, FileType::Directory, "..".to_owned()));
                collect_tree_entries(&snap, &dir.children, &mut out);
                out
            }
            InodeKind::Gitignore | InodeKind::RealFile | InodeKind::TreeSymlink => {
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
            InodeKind::Gitignore | InodeKind::TreeSymlink => {
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
        let result = self.rt.block_on(patch_issue(
            &self.http,
            &self.origin,
            &self.project,
            id,
            version,
            untainted,
            &self.agent,
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
        let result = self.rt.block_on(post_issue(
            &self.http,
            &self.origin,
            &self.project,
            untainted,
            &self.agent,
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
