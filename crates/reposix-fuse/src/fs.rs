//! Read-only [`Filesystem`] implementation.
//!
//! Callbacks implemented: `init`, `getattr`, `lookup`, `readdir`, `read`.
//! Every other callback uses fuser's default (`ENOSYS`), which the kernel
//! treats as "operation not supported" for our read-only use case.
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
//! `fetch::fetch_issues` / `fetch_issue` enforce a 5-second wall clock
//! inside the futures we block on, so no callback ever blocks the kernel
//! longer than ~5s. On timeout we reply `libc::EIO`.

use std::ffi::OsStr;
use std::io;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use dashmap::DashMap;
use fuser::{
    FileAttr, FileHandle, FileType, Filesystem, INodeNo, ReplyAttr, ReplyData, ReplyDirectory,
    ReplyEntry, Request,
};
use reposix_core::http::{client, ClientOpts, HttpClient};
use reposix_core::path::validate_issue_filename;
use reposix_core::{frontmatter, Issue};
use tokio::runtime::Runtime;
use tracing::warn;

use crate::fetch::{fetch_issue, fetch_issues, FetchError};
use crate::inode::InodeRegistry;

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
}

/// Read-only FUSE filesystem backed by a reposix-compatible HTTP API.
pub struct ReposixFs {
    /// Tokio runtime owned by the FS; used for `block_on` on callbacks.
    rt: Arc<Runtime>,
    /// Sealed allowlisted HTTP client (SG-01).
    http: Arc<HttpClient>,
    /// Backend origin (e.g. `http://127.0.0.1:7878`).
    origin: String,
    /// Project slug (e.g. `demo`).
    project: String,
    /// `X-Reposix-Agent` header value, computed once at construction
    /// (SG-05 audit attribution).
    agent: String,
    /// Inode registry.
    registry: InodeRegistry,
    /// Rendered-file cache. Invalidated wholesale on the next `readdir`
    /// refresh (entries overwritten).
    cache: DashMap<u64, Arc<CachedFile>>,
    /// Root directory attributes — cached once at construction so `getattr`
    /// and `readdir` don't recompute every time.
    root_attr: FileAttr,
}

impl std::fmt::Debug for ReposixFs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Skip the Arc/DashMap fields (not meaningfully printable); show
        // the address-ish essentials.
        f.debug_struct("ReposixFs")
            .field("origin", &self.origin)
            .field("project", &self.project)
            .field("agent", &self.agent)
            .finish_non_exhaustive()
    }
}

impl ReposixFs {
    /// Build a new read-only FUSE filesystem.
    ///
    /// # Errors
    /// Returns any error constructing the Tokio runtime or the sealed
    /// [`HttpClient`] (e.g. `REPOSIX_ALLOWED_ORIGINS` un-parseable).
    pub fn new(origin: String, project: String) -> anyhow::Result<Self> {
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
        // libc::getuid/getgid on 0.2.x are safe `extern "C"` fns but vary
        // by toolchain: some flag them `unsafe fn`. Use the `nix`-free
        // approach: `libc::c_uint` cast + dedicated safe helpers if
        // available. libc 0.2.184 exposes them as safe fns on Linux.
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
            perm: 0o555,
            nlink: 2,
            uid,
            gid,
            rdev: 0,
            blksize: 4096,
            flags: 0,
        };
        Ok(Self {
            rt,
            http,
            origin,
            project,
            agent,
            registry: InodeRegistry::new(),
            cache: DashMap::new(),
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
            perm: 0o444,
            nlink: 1,
            uid: uid_safe(),
            gid: gid_safe(),
            rdev: 0,
            blksize: 4096,
            flags: 0,
        }
    }

    /// Resolve a name in the root directory to an (inode, cache entry) pair.
    /// Does the HTTP fetch + render on miss; populates cache.
    fn resolve_name(&self, name: &str) -> Result<(u64, Arc<CachedFile>), FetchError> {
        let id = validate_issue_filename(name).map_err(|e| FetchError::Core(e.to_string()))?;
        // If we have it, return.
        if let Some(ino) = self.registry.lookup_id(id) {
            if let Some(c) = self.cache.get(&ino) {
                return Ok((ino, c.clone()));
            }
        }
        // Fetch + render + intern + cache.
        let issue = self.rt.block_on(fetch_issue(
            &self.http,
            &self.origin,
            &self.project,
            id,
            &self.agent,
        ))?;
        let rendered = frontmatter::render(&issue).map_err(|e| FetchError::Core(e.to_string()))?;
        let ino = self.registry.intern(id);
        let entry = Arc::new(CachedFile {
            issue: issue.clone(),
            rendered,
        });
        self.cache.insert(ino, entry.clone());
        Ok((ino, entry))
    }

    fn resolve_ino(&self, ino: u64) -> Result<Arc<CachedFile>, FetchError> {
        if let Some(c) = self.cache.get(&ino) {
            return Ok(c.clone());
        }
        let Some(id) = self.registry.lookup_ino(ino) else {
            return Err(FetchError::NotFound);
        };
        let issue = self.rt.block_on(fetch_issue(
            &self.http,
            &self.origin,
            &self.project,
            id,
            &self.agent,
        ))?;
        let rendered = frontmatter::render(&issue).map_err(|e| FetchError::Core(e.to_string()))?;
        let entry = Arc::new(CachedFile { issue, rendered });
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
/// semantics for FS consumers). `Parse`/`Core` are `EIO`.
fn fetch_errno(e: &FetchError) -> i32 {
    match e {
        FetchError::NotFound => libc::ENOENT,
        FetchError::Origin(_) => libc::EACCES,
        FetchError::Timeout
        | FetchError::Transport(_)
        | FetchError::Status(_)
        | FetchError::Parse(_)
        | FetchError::Core(_) => libc::EIO,
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
            let attr = self.file_attr(ino_u, &c.issue, c.rendered.len() as u64);
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
        let issues = match self.rt.block_on(fetch_issues(
            &self.http,
            &self.origin,
            &self.project,
            &self.agent,
        )) {
            Ok(v) => v,
            Err(e) => {
                warn!(error = %e, "readdir fetch failed");
                reply.error(fuser::Errno::from_i32(fetch_errno(&e)));
                return;
            }
        };

        // Populate cache with rendered bodies so subsequent `read` is fast
        // and `sim_death_no_hang` has a pre-warmed entry to stat.
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
                }),
            );
        }

        // Build the directory listing: `.`, `..`, then one entry per issue.
        // `offset` is the sequence number of the entry to START after; we
        // emit subsequent entries and let fuser truncate when the buffer
        // fills. When reply.add returns true the buffer is full.
        let mut entries: Vec<(u64, FileType, String)> = Vec::with_capacity(issues.len() + 2);
        entries.push((ROOT_INO, FileType::Directory, ".".to_owned()));
        entries.push((ROOT_INO, FileType::Directory, "..".to_owned()));
        // Sort by ID for deterministic listing (matches readdir test
        // expectations).
        let mut sorted = issues;
        sorted.sort_by_key(|i| i.id.0);
        for issue in &sorted {
            let ino = self.registry.intern(issue.id);
            let name = format!("{:04}.md", issue.id.0);
            entries.push((ino, FileType::RegularFile, name));
        }

        let start = usize::try_from(offset).unwrap_or(usize::MAX);
        for (i, (ino, kind, name)) in entries.into_iter().enumerate().skip(start) {
            // `i + 1` is the offset to hand to the kernel so the next
            // readdir resumes after this entry.
            let next = (i + 1) as u64;
            if reply.add(INodeNo(ino), next, kind, name) {
                // Buffer full — the kernel will call us back with the
                // updated offset.
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
}
