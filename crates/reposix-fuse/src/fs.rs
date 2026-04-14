//! FUSE [`Filesystem`] implementation — read path (Phase 3, Phase 10
//! rewire) + write path (Phase S).
//!
//! Read-only callbacks: `init`, `getattr`, `lookup`, `readdir`, `read`.
//! Write callbacks (Phase S): `setattr`, `write`, `flush`, `release`,
//! `create`, `unlink`. Every other callback uses fuser's default
//! (`ENOSYS`). When `MountConfig::read_only` is true (set at mount time),
//! the filesystem is mounted with `MountOption::RO` so the kernel refuses
//! writes at the VFS layer before they ever reach our callbacks.
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
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use dashmap::DashMap;
use fuser::{
    FileAttr, FileHandle, FileType, Filesystem, INodeNo, ReplyAttr, ReplyCreate, ReplyData,
    ReplyDirectory, ReplyEmpty, ReplyEntry, ReplyWrite, Request,
};
use reposix_core::http::{client, ClientOpts, HttpClient};
use reposix_core::path::validate_issue_filename;
use reposix_core::{
    frontmatter, sanitize, Issue, IssueBackend, IssueId, IssueStatus, ServerMetadata, Tainted,
};
use tokio::runtime::Runtime;
use tracing::warn;

use crate::fetch::{patch_issue, post_issue, FetchError};
use crate::inode::InodeRegistry;

/// SG-07 ceilings for read-path backend calls. A dead backend MUST
/// surface EIO to the kernel within these budgets so callbacks never
/// wedge the VFS.
///
/// `READ_GET_TIMEOUT` (5s) covers the per-issue `get_issue` fetch — one
/// HTTP round-trip on every backend.
///
/// `READ_LIST_TIMEOUT` (15s) covers the directory `list_issues` fetch.
/// The simulator returns the whole project in one round-trip so 5s is
/// plenty there, but `GithubReadOnlyBackend::list_issues` paginates up
/// to 5 pages × 100 issues/page sequentially; on a cold connection that
/// can comfortably eat the old 5s budget. 15s keeps the kernel
/// non-blocked while letting the cold path finish.
const READ_GET_TIMEOUT: Duration = Duration::from_secs(5);
const READ_LIST_TIMEOUT: Duration = Duration::from_secs(15);

/// Map a `reposix_core::Error` from an `IssueBackend` call into a
/// [`FetchError`]. The fuse layer already has a `fetch_errno` mapper for
/// [`FetchError`], so routing trait errors through that enum keeps the
/// errno mapping in one place. `Error::Other("not found: ...")` — the
/// trait's documented contract for 404 — becomes [`FetchError::NotFound`].
/// `Error::InvalidOrigin` becomes [`FetchError::Origin`]. Everything else
/// collapses to [`FetchError::Core`] (mapped to `EIO`).
fn backend_err_to_fetch(e: reposix_core::Error) -> FetchError {
    match e {
        reposix_core::Error::InvalidOrigin(o) => FetchError::Origin(o),
        reposix_core::Error::Http(t) => FetchError::Transport(t),
        reposix_core::Error::Json(j) => FetchError::Parse(j),
        reposix_core::Error::Other(msg) if msg.starts_with("not found") => FetchError::NotFound,
        other => FetchError::Core(other.to_string()),
    }
}

/// Wrap a `backend.list_issues` call in the SG-07 ceiling.
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

/// Wrap a `backend.get_issue` call in the SG-07 ceiling.
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

/// Root inode ID.
const ROOT_INO: u64 = 1;

/// Rendered-and-cached file bytes keyed by inode.
#[derive(Debug)]
struct CachedFile {
    issue: Issue,
    rendered: String,
    /// `true` if this entry was populated by a backend `get_issue` call
    /// (full body guaranteed present). `false` if populated by `list_issues`
    /// — some backends (Confluence v2) return body-less stubs on the list
    /// endpoint, so a cached-from-list entry must trigger a per-issue
    /// `get_issue` on first read to fill in the body. GitHub's list endpoint
    /// DOES return bodies, so `body_fetched: false` entries from its list
    /// path still render correctly — the post-lookup re-fetch is redundant
    /// there but cheap (one HTTP round-trip on first `cat`).
    body_fetched: bool,
}

/// FUSE filesystem backed by an [`IssueBackend`] trait object for the read
/// path and by the simulator's REST shape (via [`fetch`]) for the write
/// path. Read-only or read-write depending on the mount option; all write
/// callbacks are live when mounted RW (the simulator's 409 handling is
/// what we rely on for optimistic concurrency — see `release`).
pub struct ReposixFs {
    /// Tokio runtime owned by the FS; used for `block_on` on callbacks.
    rt: Arc<Runtime>,
    /// Read-path backend (Phase 10). All `readdir`/`lookup`/`read` calls
    /// route through this trait object. Clonable via `Arc` so `fetch::*`
    /// write-path calls remain independent.
    backend: Arc<dyn IssueBackend>,
    /// Sealed allowlisted HTTP client (SG-01) used by the write path
    /// (`release` PATCH + `create` POST). The read path uses `backend`.
    http: Arc<HttpClient>,
    /// Simulator origin used by the write path (e.g. `http://127.0.0.1:7878`).
    /// Ignored by the read path.
    origin: String,
    /// Project slug (sim) or `owner/repo` (github).
    project: String,
    /// `X-Reposix-Agent` header value, computed once at construction
    /// (SG-05 audit attribution). Attached to write-path PATCH/POST only.
    agent: String,
    /// Inode registry.
    registry: InodeRegistry,
    /// Rendered-file cache. Invalidated wholesale on the next `readdir`
    /// refresh (entries overwritten).
    cache: DashMap<u64, Arc<CachedFile>>,
    /// Per-inode write buffers. `write` appends/overwrites bytes here and
    /// `release` drains them and `PATCH`es upstream. For v0.1 we key by
    /// inode rather than file handle — the kernel always supplies both but
    /// keying by ino simplifies reopens in the typical agent flow (one
    /// writer per file at a time).
    write_buffers: DashMap<u64, Vec<u8>>,
    /// Monotonic file-handle allocator for `create`. Starts at 1; 0 is a
    /// sentinel meaning "no fh assigned".
    next_fh: AtomicU64,
    /// Root directory attributes — cached once at construction so `getattr`
    /// and `readdir` don't recompute every time.
    root_attr: FileAttr,
}

impl std::fmt::Debug for ReposixFs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Skip the Arc/DashMap fields (not meaningfully printable); show
        // the address-ish essentials.
        f.debug_struct("ReposixFs")
            .field("backend", &self.backend.name())
            .field("origin", &self.origin)
            .field("project", &self.project)
            .field("agent", &self.agent)
            .finish_non_exhaustive()
    }
}

impl ReposixFs {
    /// Build a new FUSE filesystem whose read path is served by `backend`.
    /// `origin` is the simulator REST origin used by the write path
    /// (PATCH/POST from `release`/`create`); pass any non-empty value when
    /// mounting a read-only backend like `GithubReadOnlyBackend`.
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
        let now = SystemTime::now();
        let uid = uid_safe();
        let gid = gid_safe();
        let root_attr = FileAttr {
            ino: INodeNo(ROOT_INO),
            size: 0,
            blocks: 0,
            atime: now,
            mtime: now,
            ctime: now,
            crtime: now,
            kind: FileType::Directory,
            perm: 0o755,
            nlink: 2,
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
            cache: DashMap::new(),
            write_buffers: DashMap::new(),
            next_fh: AtomicU64::new(1),
            root_attr,
        })
    }

    #[allow(clippy::unused_self)] // kept as a method for symmetry with root_attr + future per-fs overrides
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

    /// Resolve a name in the root directory to an (inode, cache entry) pair.
    /// Does the backend fetch + render on miss; populates cache.
    fn resolve_name(&self, name: &str) -> Result<(u64, Arc<CachedFile>), FetchError> {
        let id = validate_issue_filename(name).map_err(|e| FetchError::Core(e.to_string()))?;
        // Cache hit AND body already fetched → return directly.
        if let Some(ino) = self.registry.lookup_id(id) {
            if let Some(c) = self.cache.get(&ino) {
                if c.body_fetched {
                    return Ok((ino, c.clone()));
                }
            }
        }
        // Miss, or list-populated stub needing body → fetch + render + cache.
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
}

fn uid_safe() -> u32 {
    // `libc::getuid` is `unsafe fn` in libc 0.2.x, so we route through
    // rustix's safe wrapper instead. Keeps `#![forbid(unsafe_code)]`
    // intact at the crate root.
    rustix::process::getuid().as_raw()
}

fn gid_safe() -> u32 {
    rustix::process::getgid().as_raw()
}

/// Map a [`FetchError`] to a kernel errno. Timeouts and transport errors
/// collapse to `EIO` so the kernel unblocks; `NotFound` is `ENOENT`;
/// `Origin` is `EACCES` (allowlist rejection is permission-denied
/// semantics for FS consumers). `Parse`/`Core`/`Conflict`/`BadRequest`
/// are all `EIO`.
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
        if ino_u == ROOT_INO {
            reply.attr(&ATTR_TTL, &self.root_attr);
            return;
        }
        // Cache-only path for getattr — never fetch here (research §6.7).
        // If not cached, surface ENOENT; the kernel will re-lookup first.
        if let Some(c) = self.cache.get(&ino_u) {
            // Size reflects the write buffer if one is pending; otherwise
            // the rendered cached bytes.
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

    fn lookup(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEntry) {
        if parent.0 != ROOT_INO {
            reply.error(fuser::Errno::from_i32(libc::ENOTDIR));
            return;
        }
        let Some(name_str) = name.to_str() else {
            reply.error(fuser::Errno::from_i32(libc::EINVAL));
            return;
        };
        // `validate_issue_filename` runs inside `resolve_name` but do an
        // early reject so we don't waste an HTTP call on junk names.
        if validate_issue_filename(name_str).is_err() {
            reply.error(fuser::Errno::from_i32(libc::EINVAL));
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

    fn readdir(
        &self,
        _req: &Request,
        ino: INodeNo,
        _fh: FileHandle,
        offset: u64,
        mut reply: ReplyDirectory,
    ) {
        if ino.0 != ROOT_INO {
            reply.error(fuser::Errno::from_i32(libc::ENOTDIR));
            return;
        }
        // Refresh issue list. On any failure, reply EIO — no kernel hang.
        let issues = match self
            .rt
            .block_on(list_issues_with_timeout(&self.backend, &self.project))
        {
            Ok(v) => v,
            Err(e) => {
                warn!(error = %e, "readdir fetch failed");
                reply.error(fuser::Errno::from_i32(fetch_errno(&e)));
                return;
            }
        };

        // Populate cache with rendered frontmatter for fast `stat` +
        // `sim_death_no_hang`. `body_fetched: false` flags these entries as
        // needing a per-issue `get_issue` on first `read`, because some
        // backends (Confluence v2) return body-less stubs on the list
        // endpoint — reading directly from this cache would yield empty
        // bodies. The GitHub list endpoint DOES carry bodies so the re-fetch
        // there is redundant but harmless (one round-trip on first `cat`).
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

        // Build the directory listing: `.`, `..`, then one entry per issue.
        let mut entries: Vec<(u64, FileType, String)> = Vec::with_capacity(issues.len() + 2);
        entries.push((ROOT_INO, FileType::Directory, ".".to_owned()));
        entries.push((ROOT_INO, FileType::Directory, "..".to_owned()));
        let mut sorted = issues;
        sorted.sort_by_key(|i| i.id.0);
        for issue in &sorted {
            let ino = self.registry.intern(issue.id);
            let name = format!("{:04}.md", issue.id.0);
            entries.push((ino, FileType::RegularFile, name));
        }

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
        if ino.0 == ROOT_INO {
            reply.error(fuser::Errno::from_i32(libc::EISDIR));
            return;
        }
        // Serve unflushed writes from the buffer if present (so the agent
        // can `cat` its own in-progress edits).
        if let Some(buf) = self.write_buffers.get(&ino.0) {
            let bytes = buf.as_slice();
            let start = usize::try_from(offset)
                .unwrap_or(usize::MAX)
                .min(bytes.len());
            let end = start.saturating_add(size as usize).min(bytes.len());
            reply.data(&bytes[start..end]);
            return;
        }
        let cached = match self.resolve_ino(ino.0) {
            Ok(c) => c,
            Err(e) => {
                warn!(error = %e, ino = ino.0, "read failed");
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

    // ------------------------------------------------------------------ //
    // Write path (Phase S).                                              //
    // ------------------------------------------------------------------ //

    #[allow(clippy::too_many_arguments)] // fuser trait signature
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
        if ino_u == ROOT_INO {
            reply.attr(&ATTR_TTL, &self.root_attr);
            return;
        }
        // `echo >` / `open(O_TRUNC)` calls setattr(size=0) before write.
        // We honor that by clearing the buffer so subsequent writes start
        // clean.
        if let Some(0) = size {
            self.write_buffers.insert(ino_u, Vec::new());
        } else if let Some(new_size) = size {
            // Non-zero truncate: resize the buffer (seed from cache if needed).
            let mut entry = self.write_buffers.entry(ino_u).or_insert_with(|| {
                self.cache
                    .get(&ino_u)
                    .map(|c| c.rendered.as_bytes().to_vec())
                    .unwrap_or_default()
            });
            let target = usize::try_from(new_size).unwrap_or(usize::MAX);
            entry.resize(target, 0);
        }
        // Reply with the current attr. Size comes from the buffer when it
        // exists (post-truncate) so the kernel's expectation matches.
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

    #[allow(clippy::too_many_arguments)] // fuser trait signature
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
        if ino_u == ROOT_INO {
            reply.error(fuser::Errno::from_i32(libc::EISDIR));
            return;
        }
        let offset_usize = usize::try_from(offset).unwrap_or(usize::MAX);
        let end = offset_usize.saturating_add(data.len());
        let mut entry = self.write_buffers.entry(ino_u).or_insert_with(|| {
            // Seed from cached rendered bytes so `echo >>` (append) or
            // sed-style partial edits see the right prefix.
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

    fn flush(
        &self,
        _req: &Request,
        _ino: INodeNo,
        _fh: FileHandle,
        _lock_owner: fuser::LockOwner,
        reply: ReplyEmpty,
    ) {
        // We push on release, not flush. flush can fire multiple times
        // (e.g. on dup/dup2) and we'd PATCH repeatedly if we flushed here.
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
        // Atomically take the buffer if one exists.
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
        // Look up cached Issue (current version).
        let Some(cached) = self.cache.get(&ino_u).map(|c| c.clone()) else {
            // New file (create() path) not yet in cache — Task 3 handles
            // this via POST; for the MIN-VIABLE we EIO out.
            warn!(ino = ino_u, "release: no cached issue; cannot PATCH");
            reply.error(fuser::Errno::from_i32(libc::EIO));
            return;
        };
        // Parse the new bytes as frontmatter+body.
        let parsed = match frontmatter::parse(&text) {
            Ok(i) => i,
            Err(e) => {
                warn!(error = %e, ino = ino_u, "release: parse failed");
                reply.error(fuser::Errno::from_i32(libc::EIO));
                return;
            }
        };
        // Sanitize with server metadata from the cached issue.
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
                // Update cache with new rendered bytes. Write-path responses
                // carry full body, so `body_fetched: true`.
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
        if parent.0 != ROOT_INO {
            reply.error(fuser::Errno::from_i32(libc::ENOTDIR));
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
        // Minimal issue for POST — title derived from the requested id,
        // empty body. The user will `write` real content next (which
        // releases → PATCH).
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
        if parent.0 != ROOT_INO {
            reply.error(fuser::Errno::from_i32(libc::ENOTDIR));
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
        // Per CONTEXT.md: unlink does NOT call DELETE. The git-remote
        // helper is responsible for materializing deletes on `git push`,
        // so the bulk-delete cap (SG-02) can fire there. Locally we only
        // evict from the rendered cache (keeping the registry mapping
        // stable so a subsequent `create` with the same id stays
        // consistent).
        if let Some(ino) = self.registry.lookup_id(id) {
            self.cache.remove(&ino);
            self.write_buffers.remove(&ino);
        }
        reply.ok();
    }
}
