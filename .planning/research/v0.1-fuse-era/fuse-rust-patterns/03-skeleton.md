# Concrete skeleton: virtual markdown FS with read + write

## 3. Concrete skeleton: virtual markdown FS with read + write

This is a **complete, working** reference. Drop it into `crates/reposix-fuse/src/lib.rs` and iterate. Uses only the sync `Filesystem` trait — async comes in §5.

### 3.1 Cargo.toml

```toml
[package]
name = "reposix-fuse"
version = "0.1.0"
edition = "2021"
rust-version = "1.82"

[dependencies]
fuser           = { version = "0.17", default-features = false }
libc            = "0.2"
log             = "0.4"
parking_lot     = "0.12"         # faster + non-poisoning Mutex
dashmap         = "6"             # concurrent inode map
tokio           = { version = "1", features = ["rt-multi-thread", "sync", "macros", "time"] }
anyhow          = "1"
thiserror       = "2"
```

Note: `tokio` is optional if we go full-sync for the first slice. But we'll need it in §5, and it costs nothing to include now.

### 3.2 In-memory state model

```rust
use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use dashmap::DashMap;
use fuser::{
    FileAttr, FileType, Filesystem, Generation, INodeNo, FileHandle,
    KernelConfig, MountOption, Request,
    ReplyAttr, ReplyCreate, ReplyData, ReplyDirectory, ReplyEmpty,
    ReplyEntry, ReplyOpen, ReplyWrite, Errno, OpenFlags, LockOwner,
    TimeOrNow,
};
use parking_lot::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// The concrete kind of object at a given inode.
#[derive(Debug, Clone)]
enum NodeKind {
    Dir { children: BTreeMap<OsString, u64> },
    File { bytes: Vec<u8> },
}

#[derive(Debug, Clone)]
struct Node {
    ino: u64,
    parent: u64,
    kind: NodeKind,
    mode: u16,
    uid: u32,
    gid: u32,
    mtime: SystemTime,
    ctime: SystemTime,
    crtime: SystemTime,
    nlink: u32,
}

impl Node {
    fn file_attr(&self) -> FileAttr {
        let (size, kind) = match &self.kind {
            NodeKind::Dir { children } => (children.len() as u64 * 64, FileType::Directory),
            NodeKind::File { bytes }   => (bytes.len()    as u64,      FileType::RegularFile),
        };
        FileAttr {
            ino: INodeNo(self.ino),
            size,
            blocks: size.div_ceil(512),
            atime: self.mtime,
            mtime: self.mtime,
            ctime: self.ctime,
            crtime: self.crtime,
            kind,
            perm: self.mode,
            nlink: self.nlink,
            uid: self.uid,
            gid: self.gid,
            rdev: 0,
            flags: 0,
            blksize: 4096,
        }
    }
}

/// Our filesystem. All state behind RwLock so &self callbacks can mutate.
pub struct ReposixFs {
    inodes:  Arc<RwLock<DashMap<u64, Node>>>,
    next_ino: AtomicU64,
    uid: u32,
    gid: u32,
    // §5: we'll add an async runtime handle here.
}

impl ReposixFs {
    pub fn new() -> Self {
        let inodes = DashMap::new();
        let now = SystemTime::now();
        let uid = unsafe { libc::getuid() };
        let gid = unsafe { libc::getgid() };

        // Root dir is inode 1.
        let root = Node {
            ino: 1, parent: 1,
            kind: NodeKind::Dir { children: BTreeMap::new() },
            mode: 0o755,
            uid, gid,
            mtime: now, ctime: now, crtime: now,
            nlink: 2,
        };
        inodes.insert(1, root);

        ReposixFs {
            inodes: Arc::new(RwLock::new(inodes)),
            next_ino: AtomicU64::new(2),
            uid, gid,
        }
    }

    fn alloc_ino(&self) -> u64 { self.next_ino.fetch_add(1, Ordering::SeqCst) }

    /// Seed data for testing: creates /DEMO-1.md with markdown content.
    pub fn seed_demo(&self) {
        let ino = self.alloc_ino();
        let now = SystemTime::now();
        let content = b"---\nstatus: open\nassignee: alice\n---\n\n# Demo ticket\n\nHello.\n";
        let node = Node {
            ino, parent: 1,
            kind: NodeKind::File { bytes: content.to_vec() },
            mode: 0o644,
            uid: self.uid, gid: self.gid,
            mtime: now, ctime: now, crtime: now,
            nlink: 1,
        };
        let guard = self.inodes.write();
        guard.insert(ino, node);
        if let Some(mut root) = guard.get_mut(&1) {
            if let NodeKind::Dir { children } = &mut root.kind {
                children.insert(OsString::from("DEMO-1.md"), ino);
            }
        }
    }
}
```

### 3.3 `getattr` — the most-called operation

```rust
impl Filesystem for ReposixFs {
    fn getattr(&self, _req: &Request, ino: INodeNo,
               _fh: Option<FileHandle>, reply: ReplyAttr) {
        let inodes = self.inodes.read();
        match inodes.get(&ino.0) {
            Some(n) => reply.attr(&Duration::from_secs(1), &n.file_attr()),
            None    => reply.error(Errno::ENOENT),
        }
    }
```

### 3.4 `lookup` — path resolution

```rust
    fn lookup(&self, _req: &Request, parent: INodeNo, name: &OsStr,
              reply: ReplyEntry) {
        let inodes = self.inodes.read();
        let parent_node = match inodes.get(&parent.0) {
            Some(n) => n,
            None => return reply.error(Errno::ENOENT),
        };
        let children = match &parent_node.kind {
            NodeKind::Dir { children } => children,
            _ => return reply.error(Errno::ENOTDIR),
        };
        match children.get(name) {
            Some(&child_ino) => {
                drop(parent_node);
                match inodes.get(&child_ino) {
                    Some(n) => reply.entry(
                        &Duration::from_secs(1),
                        &n.file_attr(),
                        Generation(0),
                    ),
                    None => reply.error(Errno::ENOENT),
                }
            }
            None => reply.error(Errno::ENOENT),
        }
    }
```

### 3.5 `readdir` — directory listing

```rust
    fn readdir(&self, _req: &Request, ino: INodeNo, _fh: FileHandle,
               offset: u64, mut reply: ReplyDirectory) {
        let inodes = self.inodes.read();
        let node = match inodes.get(&ino.0) {
            Some(n) => n,
            None => return reply.error(Errno::ENOENT),
        };
        let children = match &node.kind {
            NodeKind::Dir { children } => children,
            _ => return reply.error(Errno::ENOTDIR),
        };

        // First two entries are always . and ..
        let mut entries: Vec<(u64, FileType, OsString)> = vec![
            (node.ino,    FileType::Directory, OsString::from(".")),
            (node.parent, FileType::Directory, OsString::from("..")),
        ];
        for (name, &child_ino) in children {
            if let Some(child) = inodes.get(&child_ino) {
                let kind = match child.kind {
                    NodeKind::Dir  { .. } => FileType::Directory,
                    NodeKind::File { .. } => FileType::RegularFile,
                };
                entries.push((child_ino, kind, name.clone()));
            }
        }

        for (i, (child_ino, kind, name)) in entries
            .into_iter().enumerate().skip(offset as usize)
        {
            // next-offset = i+1; if add returns true, buffer is full — stop.
            if reply.add(INodeNo(child_ino), (i + 1) as u64, kind, &name) {
                break;
            }
        }
        reply.ok();
    }
```

### 3.6 `read` — file content

```rust
    fn read(&self, _req: &Request, ino: INodeNo, _fh: FileHandle,
            offset: u64, size: u32, _flags: OpenFlags,
            _lock: Option<LockOwner>, reply: ReplyData) {
        let inodes = self.inodes.read();
        let node = match inodes.get(&ino.0) {
            Some(n) => n,
            None => return reply.error(Errno::ENOENT),
        };
        let bytes = match &node.kind {
            NodeKind::File { bytes } => bytes,
            _ => return reply.error(Errno::EISDIR),
        };
        let start = (offset as usize).min(bytes.len());
        let end   = (start + size as usize).min(bytes.len());
        reply.data(&bytes[start..end]);
    }
```

### 3.7 `write` — mutate file content

```rust
    fn write(&self, _req: &Request, ino: INodeNo, _fh: FileHandle,
             offset: u64, data: &[u8], _write_flags: u32,
             _flags: OpenFlags, _lock: Option<LockOwner>,
             reply: ReplyWrite) {
        let inodes = self.inodes.read(); // read guard OK — DashMap is concurrent
        let mut node = match inodes.get_mut(&ino.0) {
            Some(n) => n,
            None => return reply.error(Errno::ENOENT),
        };
        let bytes = match &mut node.kind {
            NodeKind::File { bytes } => bytes,
            _ => return reply.error(Errno::EISDIR),
        };
        let end = offset as usize + data.len();
        if bytes.len() < end { bytes.resize(end, 0); }
        bytes[offset as usize..end].copy_from_slice(data);
        node.mtime = SystemTime::now();
        reply.written(data.len() as u32);
    }
```

### 3.8 `create` — open-or-create with `O_CREAT`

```rust
    fn create(&self, _req: &Request, parent: INodeNo, name: &OsStr,
              mode: u32, _umask: u32, _flags: OpenFlags,
              reply: ReplyCreate) {
        let inodes = self.inodes.read();

        // Deny if parent missing or non-dir.
        let mut parent_node = match inodes.get_mut(&parent.0) {
            Some(p) => p,
            None => return reply.error(Errno::ENOENT),
        };
        let children = match &mut parent_node.kind {
            NodeKind::Dir { children } => children,
            _ => return reply.error(Errno::ENOTDIR),
        };
        if children.contains_key(name) {
            return reply.error(Errno::EEXIST);
        }

        let ino = self.alloc_ino();
        let now = SystemTime::now();
        let file = Node {
            ino, parent: parent.0,
            kind: NodeKind::File { bytes: Vec::new() },
            mode: (mode & 0o7777) as u16,
            uid: self.uid, gid: self.gid,
            mtime: now, ctime: now, crtime: now,
            nlink: 1,
        };
        children.insert(name.to_os_string(), ino);
        drop(parent_node);
        inodes.insert(ino, file.clone());

        let fh = FileHandle(0);  // we ignore fh — state is in the inode
        reply.created(
            &Duration::from_secs(1),
            &file.file_attr(),
            Generation(0),
            fh,
            0,
        );
    }
```

### 3.9 `unlink` — remove a file

```rust
    fn unlink(&self, _req: &Request, parent: INodeNo, name: &OsStr,
              reply: ReplyEmpty) {
        let inodes = self.inodes.read();
        let mut parent_node = match inodes.get_mut(&parent.0) {
            Some(p) => p,
            None => return reply.error(Errno::ENOENT),
        };
        let children = match &mut parent_node.kind {
            NodeKind::Dir { children } => children,
            _ => return reply.error(Errno::ENOTDIR),
        };
        let Some(child_ino) = children.remove(name) else {
            return reply.error(Errno::ENOENT);
        };
        drop(parent_node);
        // Proper FUSE semantics: decrement nlink, let forget() actually drop.
        // For v0.1 simplicity we remove immediately.
        inodes.remove(&child_ino);
        reply.ok();
    }
```

### 3.10 `setattr` — needed for `truncate` (what `>` redirect does)

This is the one everyone forgets. When a shell runs `echo foo > file.md`, the kernel issues a `setattr` with `size = Some(0)` followed by a `write`. If `setattr` no-ops, the first write works but stale bytes remain past the new data.

```rust
    fn setattr(&self, _req: &Request, ino: INodeNo,
               mode: Option<u32>, uid: Option<u32>, gid: Option<u32>,
               size: Option<u64>,
               _atime: Option<TimeOrNow>, mtime: Option<TimeOrNow>,
               _ctime: Option<SystemTime>, _fh: Option<FileHandle>,
               _crtime: Option<SystemTime>, _chgtime: Option<SystemTime>,
               _bkuptime: Option<SystemTime>, _flags: Option<u32>,
               reply: ReplyAttr) {
        let inodes = self.inodes.read();
        let mut node = match inodes.get_mut(&ino.0) {
            Some(n) => n,
            None => return reply.error(Errno::ENOENT),
        };
        if let Some(m) = mode { node.mode = (m & 0o7777) as u16; }
        if let Some(u) = uid  { node.uid  = u; }
        if let Some(g) = gid  { node.gid  = g; }
        if let Some(new_size) = size {
            if let NodeKind::File { bytes } = &mut node.kind {
                bytes.resize(new_size as usize, 0);
            }
        }
        if let Some(mt) = mtime {
            node.mtime = match mt {
                TimeOrNow::SpecificTime(t) => t,
                TimeOrNow::Now              => SystemTime::now(),
            };
        }
        reply.attr(&Duration::from_secs(1), &node.file_attr());
    }
```

### 3.11 Mount entry point

```rust
    } // end of impl Filesystem

impl ReposixFs {
    pub fn mount_blocking<P: AsRef<std::path::Path>>(
        self, mountpoint: P,
    ) -> std::io::Result<()> {
        let options = vec![
            MountOption::FSName("reposix".into()),
            MountOption::Subtype("reposix".into()),
            MountOption::AutoUnmount,            // fusermount -u on daemon exit
            MountOption::DefaultPermissions,     // let kernel enforce mode bits
        ];
        fuser::mount2(self, mountpoint, &options)
    }

    /// Non-blocking variant — returns a BackgroundSession that unmounts on drop.
    pub fn spawn_mount<P: AsRef<std::path::Path>>(
        self, mountpoint: P,
    ) -> std::io::Result<fuser::BackgroundSession> {
        let options = vec![
            MountOption::FSName("reposix".into()),
            MountOption::AutoUnmount,
            MountOption::DefaultPermissions,
        ];
        fuser::spawn_mount2(self, mountpoint, &options)
    }
}
```

**Mount options worth knowing** (from `fuser::MountOption`):

| Option | What it does | Use for reposix? |
|---|---|---|
| `FSName(String)` | `df -T` shows this as the source | YES — `"reposix"` |
| `Subtype(String)` | Shows as `fuse.reposix` in mount list | YES |
| `AutoUnmount` | `fusermount3` unmounts on daemon exit (even panic) | YES — critical |
| `DefaultPermissions` | Kernel checks mode bits before forwarding call | YES — free security |
| `AllowOther` | Other users can access | NO for v0.1 (security surface) |
| `AllowRoot` | Only root + owner can access | NO |
| `RO` | Read-only mount | NO — we need write |
| `NoExec`, `NoAtime`, `NoSuid`, `NoDev` | Standard POSIX-ish flags | NoSuid, NoDev recommended |
| `CUSTOM(String)` | Pass arbitrary option string to fusermount | escape hatch |

Source: fuser hello.rs example and [fuser MountOption docs](https://docs.rs/fuser/latest/fuser/enum.MountOption.html).
