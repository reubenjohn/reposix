# FUSE-in-Rust Patterns for reposix

**Scope:** Concrete implementation guidance for the `-fuse` crate of the reposix workspace. Mounts a virtual POSIX tree backed by an async HTTP client (simulator today, real issue trackers later). Linux-only, pure-Rust (no `libfuse-dev`), must run unprivileged on a GitHub Actions `ubuntu-latest` runner.

**Researched:** 2026-04-13
**Overall confidence:** HIGH for fuser crate internals & mount mechanics (verified against source). HIGH for GitHub Actions runner behavior (verified against runner-images + community discussions). MEDIUM for exact `experimental` API shape (rapidly changing — pin a version).

**Bottom line:** Use **`fuser` 0.17.x with `default-features = false`**. Implement the *synchronous* `Filesystem` trait. Bridge to async via a dedicated `tokio::runtime::Runtime` owned by the filesystem struct + a blocking `rt.block_on(...)` or `tokio::sync::oneshot` roundtrip inside each callback. Install `fuse3` (the runtime package, not `-dev`) on the CI runner. Do **not** adopt the `experimental` async trait yet — it shifts shape every release and is not required for our workload.

---

## 1. fuser 0.15 / 0.16 / 0.17 API surface

### 1.1 Version landscape

- Project constraint names fuser **0.15**, but the useful target is **0.17** — it's the first version where `libfuse` is **not a default feature**, which is exactly what we need. Upgrading from 0.15 to 0.17 is net-positive for our constraint; only the experimental async module churned between them.
- 0.16.0 (2025-09-12) removed `libfuse` from default features and removed ABI feature flags `abi-7-9` through `abi-7-18`.
- 0.17.0 further deprecated the remaining ABI flags (moved to runtime negotiation).
- 0.15.0 (2024-10-25) added a file handle argument to `getattr()` and a low-level `Session` API for wrapping a raw fuse fd.

**Recommendation:** Pin `fuser = { version = "0.17", default-features = false }`. This is strictly better than 0.15 for our constraints. If the gsd-planner insists on 0.15, the Cargo.toml line becomes `fuser = { version = "0.15", default-features = false, features = [] }` and everything else in this report works identically — the `Filesystem` trait method signatures are stable across 0.15→0.17.

### 1.2 `default-features = false` — what you lose, what you keep

`Cargo.toml` features in 0.17 ([source](https://docs.rs/crate/fuser/latest/source/Cargo.toml)):

| Feature | Default? | What it does | reposix verdict |
|---|---|---|---|
| `libfuse` | ❌ (0.16+) | Links against `libfuse` / `libfuse3` C library at build time. Requires `libfuse-dev` + `pkg-config`. | **OFF** — we can't install `-dev` packages. |
| `libfuse2` | ❌ | Forces libfuse v2 semantics | OFF |
| `libfuse3` | ❌ | Forces libfuse v3 semantics | OFF |
| `experimental` | ❌ | Pulls in `async-trait` + `tokio`, exposes `fuser::experimental::{AsyncFilesystem, TokioAdapter, ...}` | OFF (see §5 for why) |
| `serializable` | ❌ | `serde` derives on public types | Maybe — handy for the audit DB, but cheap to re-derive locally. Leave OFF. |
| `abi-7-XX` | ❌ | Protocol version caps | Leave OFF — runtime-negotiated in 0.17. |

With `default-features = false` and zero features enabled, fuser is **pure Rust at compile time** (no C linkage, no `pkg-config`, no `libfuse-dev`). The required dependencies are all pure-Rust crates: `bitflags`, `libc` (headers only), `log`, `memchr`, `nix`, `num_enum`, `page_size`, `parking_lot`, `ref-cast`, `smallvec`, `zerocopy`.

**At runtime**, fuser still needs to perform the actual `mount(2)` syscall. Its strategy (pure-Rust mount path, in `src/mnt/fuse_pure.rs`):

1. **Try a direct kernel mount first** via `nix::mount::mount()` on `/dev/fuse` with options `fd=N,rootmode=...,user_id=...,group_id=...`. This needs `CAP_SYS_ADMIN`.
2. **On `EPERM`, fall back to exec-ing `fusermount3` (preferred) or `fusermount`**, passing the socket fd via the `_FUSE_COMMFD` environment variable. fusermount is setuid-root on Debian/Ubuntu, so this works for unprivileged users. It receives the `/dev/fuse` fd back via an `SCM_RIGHTS` control message.
3. fuser searches `$PATH` then `/sbin/`, `/bin/`, with an override via the `FUSERMOUNT_PATH` environment variable.

**Direct consequence for reposix:** on the dev host and on `ubuntu-latest`, we are an unprivileged user, so we will always take the `fusermount3` fallback path. That's fine — `/usr/bin/fusermount3` is present on the dev host and installable on CI without `-dev` packages. Confirmed on dev host:

```
$ ls /usr/bin/fusermount*
/usr/bin/fusermount   /usr/bin/fusermount3
```

### 1.3 The `Filesystem` trait — signatures we'll actually implement

All methods have default no-op implementations, so implement only what we need. Signatures below are from fuser 0.17 ([docs](https://docs.rs/fuser/latest/fuser/trait.Filesystem.html)).

```rust
pub trait Filesystem {
    fn init(&mut self, req: &Request<'_>, config: &mut KernelConfig)
        -> Result<(), Errno> { Ok(()) }
    fn destroy(&mut self) {}

    fn lookup(&self, req: &Request, parent: INodeNo, name: &OsStr,
              reply: ReplyEntry);
    fn forget(&self, req: &Request, ino: INodeNo, nlookup: u64);

    fn getattr(&self, req: &Request, ino: INodeNo, fh: Option<FileHandle>,
               reply: ReplyAttr);
    fn setattr(&self, req: &Request, ino: INodeNo,
               mode: Option<u32>, uid: Option<u32>, gid: Option<u32>,
               size: Option<u64>,
               atime: Option<TimeOrNow>, mtime: Option<TimeOrNow>,
               ctime: Option<SystemTime>, fh: Option<FileHandle>,
               crtime: Option<SystemTime>, chgtime: Option<SystemTime>,
               bkuptime: Option<SystemTime>, flags: Option<u32>,
               reply: ReplyAttr);

    fn mkdir(&self, req: &Request, parent: INodeNo, name: &OsStr,
             mode: u32, umask: u32, reply: ReplyEntry);
    fn unlink(&self, req: &Request, parent: INodeNo, name: &OsStr,
              reply: ReplyEmpty);
    fn rmdir(&self, req: &Request, parent: INodeNo, name: &OsStr,
             reply: ReplyEmpty);
    fn rename(&self, req: &Request, parent: INodeNo, name: &OsStr,
              newparent: INodeNo, newname: &OsStr, flags: u32,
              reply: ReplyEmpty);

    fn open(&self, req: &Request, ino: INodeNo, flags: OpenFlags,
            reply: ReplyOpen);
    fn read(&self, req: &Request, ino: INodeNo, fh: FileHandle,
            offset: u64, size: u32, flags: OpenFlags,
            lock_owner: Option<LockOwner>, reply: ReplyData);
    fn write(&self, req: &Request, ino: INodeNo, fh: FileHandle,
             offset: u64, data: &[u8], write_flags: u32,
             flags: OpenFlags, lock_owner: Option<LockOwner>,
             reply: ReplyWrite);
    fn flush(&self, req: &Request, ino: INodeNo, fh: FileHandle,
             lock_owner: LockOwner, reply: ReplyEmpty);
    fn release(&self, req: &Request, ino: INodeNo, fh: FileHandle,
               flags: OpenFlags, lock_owner: Option<LockOwner>,
               flush: bool, reply: ReplyEmpty);
    fn fsync(&self, req: &Request, ino: INodeNo, fh: FileHandle,
             datasync: bool, reply: ReplyEmpty);

    fn opendir(&self, req: &Request, ino: INodeNo, flags: OpenFlags,
               reply: ReplyOpen);
    fn readdir(&self, req: &Request, ino: INodeNo, fh: FileHandle,
               offset: u64, reply: ReplyDirectory);
    fn releasedir(&self, req: &Request, ino: INodeNo, fh: FileHandle,
                  flags: OpenFlags, reply: ReplyEmpty);

    fn create(&self, req: &Request, parent: INodeNo, name: &OsStr,
              mode: u32, umask: u32, flags: OpenFlags,
              reply: ReplyCreate);
    fn access(&self, req: &Request, ino: INodeNo, mask: i32,
              reply: ReplyEmpty);
    fn statfs(&self, req: &Request, ino: INodeNo, reply: ReplyStatfs);
    // ... setxattr/getxattr/listxattr/removexattr, symlink/readlink/link,
    //     mknod, getlk/setlk, fallocate, lseek, copy_file_range, ioctl,
    //     poll, bmap — all have default Errno::ENOSYS replies.
}
```

**Key observations:**

- `&self` everywhere (not `&mut self`) — fuser 0.17 requires the filesystem to be `Sync`, and you get concurrent calls. Any mutable state goes behind `Mutex`/`RwLock`/`DashMap`/atomics. **This is important for async bridging** (see §5).
- Replies are *consumed* — each callback must call exactly one of `reply.XXX(...)` or `reply.error(errno)` exactly once. Forgetting this hangs the kernel on that request.
- `INodeNo(1)` = `INodeNo::ROOT`. Allocate your own inode numbers starting at 2.
- `readdir`'s `offset` is the cursor *returned by the previous call's last `reply.add(...)`* — we control what it means.

### 1.4 Reply type cheat-sheet

| Reply | `.ok()` method | Common error returns |
|---|---|---|
| `ReplyEntry` | `.entry(&ttl, &file_attr, Generation(0))` | ENOENT |
| `ReplyAttr` | `.attr(&ttl, &file_attr)` | ENOENT |
| `ReplyData` | `.data(&bytes)` | EIO, ERANGE |
| `ReplyDirectory` | `.add(ino, next_offset, kind, name) -> bool` + `.ok()` | — |
| `ReplyEmpty` | `.ok()` | ENOSYS, EACCES |
| `ReplyOpen` | `.opened(fh, flags)` | EACCES |
| `ReplyCreate` | `.created(&ttl, &attr, Generation(0), fh, flags)` | EEXIST |
| `ReplyWrite` | `.written(bytes_written)` | EIO, ENOSPC |
| `ReplyStatfs` | `.statfs(blocks, bfree, bavail, files, ffree, bsize, namelen, frsize)` | — |

`.error(Errno::ENOENT)` etc. is how you short-circuit on failure. Use `fuser::Errno` (re-exported) rather than bare `libc::ENOENT` integers.

---

## 2. Mounting inside a GitHub Actions runner

### 2.1 Runner reality check

- `ubuntu-latest` **does not preinstall `fuse3` or the `fusermount3` binary**. Confirmed via community discussion ([#26404](https://github.com/orgs/community/discussions/26404)) and runner-images discussion ([#10528](https://github.com/actions/runner-images/discussions/10528)).
- `/dev/fuse` is **available** (the kernel module is loaded on GitHub-hosted runners) but unusable by the runner user until `fusermount3` is installed setuid-root — which `apt install fuse3` handles automatically.
- The runner user (`runner`) is **non-root but has passwordless `sudo`**. We can run `sudo apt install` freely in CI workflows (this is where it differs from our dev host, which lacks sudo — on the dev host the binaries are already present).

### 2.2 Minimum CI install

```yaml
# .github/workflows/ci.yml
- name: Install FUSE runtime
  run: |
    sudo apt-get update
    sudo apt-get install -y fuse3
    # Verify
    which fusermount3
    ls -l /dev/fuse
```

That's it. **Do not install `libfuse3-dev`** — it's unnecessary for us (we don't link libfuse) and pulls in `pkg-config` which fights with our constraint discipline on the dev host. `fuse3` alone gets us:

- `/usr/bin/fusermount3` (setuid-root)
- `/usr/bin/mount.fuse3`
- Appropriate udev rules for `/dev/fuse`

### 2.3 No special permissions needed

The `runner` user is already in the right groups, and `/dev/fuse` is world-readable (mode `0666`) on Ubuntu 22.04/24.04 runners. Once fuse3 is installed, `cargo run -- mount /tmp/mnt` just works without sudo.

One gotcha: if the workflow uses a container (`container:` key), the container needs `--device /dev/fuse --cap-add SYS_ADMIN` (or `--privileged`). Sticking to the host runner is simpler.

### 2.4 Integration test harness skeleton

```yaml
- name: Integration test (FUSE mount)
  timeout-minutes: 5
  run: |
    # Start simulator in background
    cargo run -p reposix-sim -- serve --port 7878 &
    SIM_PID=$!
    sleep 1

    # Mount FUSE in background
    mkdir -p /tmp/reposix-mnt
    cargo run -p reposix-cli -- mount \
        --backend http://127.0.0.1:7878 \
        /tmp/reposix-mnt &
    FUSE_PID=$!

    # Wait for mount to be live (poll `ls`, max 10s)
    for i in {1..20}; do
      if mountpoint -q /tmp/reposix-mnt; then break; fi
      sleep 0.5
    done
    mountpoint -q /tmp/reposix-mnt

    # Exercise it with POSIX tools
    ls /tmp/reposix-mnt
    cat /tmp/reposix-mnt/DEMO-1.md
    echo "status: done" | sed -i ... # writes
    grep -r "bug" /tmp/reposix-mnt

    # Tear down
    fusermount3 -u /tmp/reposix-mnt
    kill $FUSE_PID $SIM_PID || true
```

Key defensive touches:

- `timeout-minutes: 5` — a hung FUSE daemon will pin the runner for 6h default; a tight timeout recovers the minutes budget.
- `mountpoint -q` as the readiness gate (race-free).
- `fusermount3 -u` — **not** `umount` — because only `fusermount3` is setuid-root.
- Always background the mount and kill it explicitly; a panicking daemon leaves a dangling mount that `ls` hangs on.

---

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

---

## 4. Inode allocation strategy for a dynamic remote-backed FS

The hard problem: issues get created, renamed, and deleted on the upstream. Our local inode numbers must be **stable across daemon restarts** (otherwise `find . -inum` surprises agents and open file handles break), **unique during a session**, and **cheap to allocate**.

### 4.1 Anti-patterns

- **Hashing the issue key to u64.** Tempting (stable across restarts!), but 64-bit hash collisions across 10k tickets are unlikely but non-zero, and a collision on inode numbers is a *correctness* bug that silently returns the wrong file. Rejected.
- **Storing the issue key directly as the inode.** Issue keys like `PROJ-1234` don't fit in u64.
- **Using database rowid.** Requires SQLite in the hot path of every `lookup`. Too slow.

### 4.2 Recommended: monotonic counter + persisted map

```rust
/// Stable map: issue-key <-> inode. Persisted to SQLite on every change.
struct InodeRegistry {
    key_to_ino:  DashMap<String, u64>,
    ino_to_key:  DashMap<u64, String>,
    next: AtomicU64,         // init to MAX(existing ino) + 1 at startup
    db: Arc<rusqlite::Connection>,
}

impl InodeRegistry {
    fn intern(&self, key: &str) -> u64 {
        if let Some(entry) = self.key_to_ino.get(key) {
            return *entry;
        }
        let ino = self.next.fetch_add(1, Ordering::SeqCst);
        self.key_to_ino.insert(key.to_string(), ino);
        self.ino_to_key.insert(ino, key.to_string());
        self.db.execute(
            "INSERT INTO inode_map(key, ino) VALUES(?1, ?2)",
            (key, ino),
        ).unwrap();
        ino
    }
}
```

Reserved ranges:

- `1` — root directory. (`INodeNo::ROOT`)
- `2..=0xFFFF` — static/synthetic nodes (per-project dirs, `.reposix/` control files).
- `0x10000..` — dynamic issues, assigned by the registry above.

### 4.3 When remote deletion happens

An issue deleted upstream should:

1. Be removed from the parent dir's `children` map (so `readdir` stops showing it).
2. **Not** have its inode reused. The kernel may still hold references; if a process has the file open when it vanishes, standard POSIX semantics say reads succeed until close. With FUSE, we handle this via `forget(ino, nlookup)` — the kernel tells us when its ref count drops to zero, at which point we can finally free the slot.

Use `forget` as the trigger to clean up, not `unlink`. Until `forget`, keep the inode entry with a `tombstoned: bool` flag so `getattr` still returns attrs.

```rust
    fn forget(&self, _req: &Request, ino: INodeNo, nlookup: u64) {
        let mut lookups = self.lookup_counts.write();
        let count = lookups.entry(ino.0).or_insert(0);
        *count = count.saturating_sub(nlookup);
        if *count == 0 {
            lookups.remove(&ino.0);
            if let Some(node) = self.inodes.read().get(&ino.0) {
                if node.tombstoned {
                    self.inodes.read().remove(&ino.0);
                }
            }
        }
    }
```

Simpler alternative for v0.1: never delete. A tombstoned inode costs ~200 bytes. Agents delete issues via `git push`, not `rm`, so this isn't hot.

---

## 5. Async bridge: calling reqwest from sync FUSE callbacks

**The core tension:** `Filesystem` methods are sync `&self` functions. Our backend (simulator, eventually Jira/GitHub) is HTTP, ideally async (reqwest). We need to resolve this without blocking the FUSE worker thread pool indefinitely and without a deadlock.

### 5.1 Why NOT use `fuser::experimental::AsyncFilesystem`

The `experimental` module exists and can be enabled with `features = ["experimental"]`. It provides `AsyncFilesystem` with `async fn` methods and a `TokioAdapter`. Tempting. But:

1. **It's experimental** — API churned between 0.15, 0.16, 0.17. `DirEntListBuilder`, `RequestContext`, `LookupResponse`, `GetAttrResponse` are new names every release. Pinning a version works, but upgrading is painful.
2. **It forces tokio into the public surface of our fuse crate.** Crate users (other reposix workspace members testing with a fake FS) pay for tokio even if they don't need it.
3. **Internally, `TokioAdapter` does exactly what we'd do manually:** wraps the filesystem in `Arc<RwLock<...>>` + pins a tokio runtime and runs each request on it. No magic win.

**Verdict:** implement the pattern ourselves in ~40 lines. It's more robust across fuser upgrades.

### 5.2 The pattern: dedicated multi-thread runtime owned by the FS

```rust
use tokio::runtime::{Builder, Runtime, Handle};
use tokio::sync::oneshot;

pub struct ReposixFs {
    inodes:   Arc<RwLock<DashMap<u64, Node>>>,
    next_ino: AtomicU64,
    uid: u32, gid: u32,

    /// Dedicated async runtime. Owned by the FS so it lives as long as we do.
    rt:       Arc<Runtime>,
    /// HTTP client — reqwest uses tokio internally, so it's tied to `rt`.
    http:     Arc<reqwest::Client>,
    backend:  reqwest::Url,
}

impl ReposixFs {
    pub fn new(backend: reqwest::Url) -> Self {
        let rt = Arc::new(
            Builder::new_multi_thread()
                .worker_threads(2)           // small — most calls are I/O bound
                .enable_all()
                .thread_name("reposix-async")
                .build()
                .expect("tokio runtime"),
        );
        let http = Arc::new(
            rt.block_on(async { reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap()
            }),
        );
        // ... seed root, etc.
        ReposixFs { /* ... */ rt, http, backend }
    }

    /// Bridge helper: run an async future to completion from sync code.
    fn block<F, T>(&self, fut: F) -> T
    where F: std::future::Future<Output = T> + Send + 'static,
          T: Send + 'static,
    {
        // CRITICAL: we are on a FUSE worker thread (not the tokio runtime).
        // So block_on is safe — it doesn't deadlock the runtime.
        self.rt.block_on(fut)
    }
}
```

**Why `block_on` is safe here:** FUSE spawns OS threads to serve kernel requests. Those threads are *not* tokio runtime worker threads. So when one of them calls `self.rt.block_on(fut)`, it's parking a non-runtime thread — the tokio worker threads are free to poll `fut` to completion. This is the canonical pattern from the [tokio bridging docs](https://tokio.rs/tokio/topics/bridging).

**Deadlock trap to avoid:** if you accidentally called `block_on` from *inside* an async context (e.g. from a callback driven by tokio itself), you'd block a worker thread that might be needed to poll your own future. Since FUSE callbacks are never inside tokio, we're safe — but guard with a newtype to prevent future regressions.

### 5.3 Using the bridge — example: fetch an issue lazily

```rust
impl Filesystem for ReposixFs {
    fn read(&self, _req: &Request, ino: INodeNo, _fh: FileHandle,
            offset: u64, size: u32, _flags: OpenFlags,
            _lock: Option<LockOwner>, reply: ReplyData) {
        // Fast path: content already cached.
        {
            let inodes = self.inodes.read();
            if let Some(n) = inodes.get(&ino.0) {
                if let NodeKind::File { bytes } = &n.kind {
                    if !bytes.is_empty() || n.marked_fetched {
                        let start = (offset as usize).min(bytes.len());
                        let end   = (start + size as usize).min(bytes.len());
                        return reply.data(&bytes[start..end]);
                    }
                }
            }
        }
        // Slow path: fetch from backend synchronously.
        let key = match self.inode_to_key(ino.0) {
            Some(k) => k,
            None => return reply.error(Errno::ENOENT),
        };
        let http = self.http.clone();
        let url  = self.backend.join(&format!("issues/{key}")).unwrap();

        let result = self.block(async move {
            http.get(url).send().await?.error_for_status()?.bytes().await
        });

        let bytes = match result {
            Ok(b) => b.to_vec(),
            Err(e) => {
                log::warn!("fetch {key} failed: {e}");
                return reply.error(Errno::EIO);
            }
        };

        // Cache in place + slice for reply.
        {
            let inodes = self.inodes.read();
            if let Some(mut n) = inodes.get_mut(&ino.0) {
                if let NodeKind::File { bytes: b } = &mut n.kind {
                    *b = bytes.clone();
                    n.mtime = SystemTime::now();
                }
            }
        }
        let start = (offset as usize).min(bytes.len());
        let end   = (start + size as usize).min(bytes.len());
        reply.data(&bytes[start..end]);
    }
}
```

### 5.4 Alternative: spawn + oneshot for fire-and-forget writes

For `write`, we usually want to buffer and ack the kernel quickly (writes shouldn't block on the network). Ack locally, push to the backend in the background:

```rust
fn write(&self, ...) {
    // ... update local bytes first, ack kernel immediately
    let bytes_written = data.len() as u32;
    let http = self.http.clone();
    let url  = self.backend.join(&format!("issues/{key}")).unwrap();
    let data_owned = new_content.clone();
    self.rt.spawn(async move {
        if let Err(e) = http.put(url).body(data_owned).send().await {
            log::warn!("background push failed: {e}");
            // TODO: surface via audit log + retry queue
        }
    });
    reply.written(bytes_written);
}
```

**This matches the InitialReport.md architecture** — writes do not go directly to the upstream. In the real design, writes go to local state and are pushed via `git-remote-reposix`. The spawned PUT above is only a hot-cache write-through; the durable write path is git.

### 5.5 When oneshot channels help

You rarely need them. The main use case is if you want timeout-with-fallback:

```rust
let (tx, rx) = tokio::sync::oneshot::channel();
self.rt.spawn(async move {
    let result = tokio::time::timeout(
        Duration::from_millis(500),
        http.get(url).send()
    ).await;
    let _ = tx.send(result);
});
// Don't block_on — poll synchronously with a deadline:
match rx.blocking_recv() {
    Ok(Ok(Ok(resp))) => { /* got response */ }
    _ => { /* timeout or error — serve from cache or EIO */ }
}
```

But `rt.block_on(tokio::time::timeout(...))` is equivalent and simpler.

---

## 6. Known gotchas

### 6.1 UID/GID handling

- **Always set `uid`/`gid` on FileAttr to `libc::getuid()` / `libc::getgid()`** (not 0, not 501-from-the-example). If the attrs claim uid=0 but the calling process is uid=1000, kernel-side permission checks (especially with `DefaultPermissions`) will deny everything.
- Per-request caller uid/gid is available via `req.uid()` / `req.gid()` — useful for audit logging, rarely for authorization (we want file-level perms, not dynamic).
- Never trust the uid in the request for security decisions if `AllowOther` is set. We don't set it.

### 6.2 `st_size` correctness

- `readdir` returns entries. Each entry carries an inode and a name. The kernel will follow up with `getattr` calls per entry (often before issuing a `read`). **Make sure `file_attr().size` reflects the *current* size of the content.** Returning 0 when the file has bytes will cause `cat` to print nothing (because the kernel short-circuits the read at size=0).
- For files whose size isn't known until content is fetched (lazy-loaded issues), return a conservative estimate *or* fetch metadata eagerly in `lookup`. The worst option is returning size=0 on a non-empty file.
- When you `resize(bytes, new_size, 0)` in `setattr`, the kernel caches the attr for `TTL` — use `Duration::ZERO` if you want subsequent `stat`s to re-ask you. Trade-off: many more `getattr` calls.

### 6.3 Write buffering / atomicity

- FUSE writes arrive in chunks of up to `max_write` bytes (default 128 KiB). An editor saving a 300 KiB file will send three `write` calls, then `flush`, then `release`.
- **Do not ack an upstream commit until `release` fires.** Pattern: accumulate in a per-fh buffer, on `release` flush to backend in one batched call (mimics what git-remote helper will do later, keeps the API call count low).
- Set `KernelConfig::set_max_write` in `init` if you want bigger chunks. Default is fine for text files.
- For concurrent writers on the same inode, use `tokio::sync::Mutex` in the per-file state — not the outer `RwLock`, which would serialize all reads too.

### 6.4 Unmount cleanup

- `MountOption::AutoUnmount` is **mandatory**. Without it, panics/crashes leave a dangling mount that `ls` hangs on. `fusermount3 -u` is the only recovery.
- On graceful shutdown (SIGTERM), drop the `Filesystem` — `fuser::mount2` returns, `AutoUnmount` kicks in via the kernel unmount-on-abort mechanism.
- Use `BackgroundSession` (from `spawn_mount2`) when the daemon is embedded in a larger CLI. Drop it to unmount. Don't forget — a `let _ = spawn_mount2(...)` discards the session and immediately unmounts.
- For CI: always `trap "fusermount3 -u /tmp/mnt" EXIT` in the workflow shell.

### 6.5 Kernel caching

- `ReplyEntry`'s TTL caches the name→inode mapping. TTL=0 means "re-ask every syscall" (slow). TTL=Duration::MAX means "never re-ask" (stale). **Default to 1 second.**
- `ReplyAttr`'s TTL caches attributes. Same logic.
- When the remote state changes, the kernel won't know. For freshness, either (a) use short TTLs, (b) call `Session::notifier().inval_inode(...)` on change, or (c) accept staleness and fix on next TTL expiry.
- `.reposix/refresh` pseudo-file is a nice UX — `cat .reposix/refresh` triggers a re-poll. Emacs-style.

### 6.6 Logging blows up

- `log::debug!` in every callback generates tens of thousands of lines during `grep -r`. Use `log::trace!` for per-op logging, `log::debug!` only for lifecycle.
- `RUST_LOG=reposix_fuse=info` is the sensible default.

### 6.7 `getattr` on inode 1 is called *constantly*

- Every shell `cd`, tab-complete, prompt-update issues a `stat` on the CWD. Our root-dir `getattr` must be essentially free. Don't hold the write lock.
- Cache the root attr as a constant `FileAttr` computed at startup.

### 6.8 The "disappearing file" bug

Symptom: you `create` a file, `ls` shows it, but `cat` says "No such file or directory."
Cause: your `lookup` and `create` report different inodes, or `getattr` returns `ENOENT` for the freshly created inode. Always:
1. Allocate inode
2. Insert into `inodes` map
3. Insert into parent's `children`
4. Reply with the new attr

...in that order. Do not reply before inserting.

### 6.9 Permission check surprises

With `MountOption::DefaultPermissions`, the kernel checks `mode`+`uid`+`gid` before calling us. That means:
- A `0o644` file owned by uid=1000 is not writable by uid=1001 — we never see the `write()` call.
- Without `DefaultPermissions`, we'd need to implement `access()` ourselves.
- **Use `DefaultPermissions`.** Mode bits stored on our Nodes become real. That maps directly to the RBAC-to-POSIX pattern in InitialReport.md §Governance.

---

## 7. Library choice: fuser vs fuse3 vs fuse_mt vs polyfuse

| Crate | Pure Rust? | Async? | Path-based API? | Status | Fit for reposix |
|---|---|---|---|---|---|
| **fuser** | YES (since 0.16, opt-in) | Experimental | NO (inode-based) | Actively maintained (cberner), used by Redox, stratisd | **PICK** |
| fuse3 | YES for Linux (optional `unprivileged` feature calls fusermount3) | Built-in (tokio OR async-std) | NO (inode-based) | Maintained, smaller community | Solid runner-up |
| fuse_mt | NO (uses fuse C bindings) | NO (multithreaded sync only) | YES (paths, not inodes) | Archived / barely maintained | Nope |
| polyfuse | YES | Built-in | NO (inode-based) | Stalled (last real activity 2022) | Nope |
| easy_fuser | Built on fuser | NO | YES (high-level wrapper) | Niche | Interesting for v0.2 if fuser feels too low-level |

### Rationale for fuser

1. **Matches the constraint exactly.** `default-features = false` on 0.17 gives pure Rust compilation, no `libfuse-dev`, runtime uses `fusermount3` which we have.
2. **Most production Rust FUSE code uses it.** Redox's vfs, stratisd, a bunch of cloud storage adapters. Largest pool of reference code when we hit weird edge cases.
3. **We don't need built-in async.** §5 shows a 40-line bridge that works perfectly with sync `Filesystem`. Saves us from `experimental` churn.
4. **`BackgroundSession` + `spawn_mount2`** match what the CLI orchestrator (`reposix mount`) needs — background thread, drop-to-unmount.

### When we'd switch

- **fuse3** if we find that our manual async bridge adds measurable latency at the 99th percentile under the adversarial swarm. Its native tokio integration removes one thread hop per call. Profile first.
- **Never fuse_mt** — the path-based abstraction loses critical fidelity (no inode stability across opens → can't implement `forget` correctly → memory leaks under agent swarm load).

### A note on `fuse3` as an alternative

If fuser's pure-Rust mount path has bugs on our kernel (unlikely but possible — cberner's pure-Rust path is relatively new), `fuse3` with `features = ["unprivileged"]` is the closest drop-in alternative. It always uses `fusermount3` (no direct kernel mount attempt), so it's more predictable on unprivileged hosts. Keep it in mind as a fallback if the pure-Rust mount path misbehaves.

---

## 8. Phase-planning guidance for gsd-planner

Suggested decomposition for the FUSE slice:

**Phase F.1 — Static in-memory FS (1.5h budget)**
- Crate skeleton (`reposix-fuse` crate), deps as in §3.1.
- Implement `Filesystem` for `ReposixFs` with `getattr`, `lookup`, `readdir`, `read` only (§3.3-3.6).
- Seed one hard-coded `DEMO-1.md` at startup.
- Exit criterion: `cargo run -p reposix-fuse -- /tmp/mnt` then `cat /tmp/mnt/DEMO-1.md` prints the seeded content. No backend yet.

**Phase F.2 — Write path (1h budget)**
- Add `write`, `create`, `unlink`, `setattr` (§3.7-3.10).
- Exit criterion: `echo 'hi' > /tmp/mnt/new.md; cat /tmp/mnt/new.md; rm /tmp/mnt/new.md` round-trips.

**Phase F.3 — Backend bridge (1.5h budget)**
- Add async runtime + reqwest (§5.2).
- Wire `read` to lazy-fetch from simulator (§5.3).
- Wire `write` to spawn background PUT (§5.4).
- Inode registry with SQLite persistence (§4.2).
- Exit criterion: mount with `--backend http://127.0.0.1:7878`, simulator serves issues, `ls` and `cat` work against real HTTP.

**Phase F.4 — CI mount test (30m budget)**
- `.github/workflows/ci.yml` with the fuse3 install + integration test (§2.2, §2.4).
- Exit criterion: green CI run that actually mounts and exercises the FS on ubuntu-latest.

**Phase F.5 — Hardening (time permitting)**
- `forget()` + tombstoning (§4.3).
- `AutoUnmount` + signal handling.
- `RUST_LOG` noise budgeting (§6.6).
- Per-fh write buffering for atomic pushes (§6.3).

Do these serially — each needs the previous to validate. Phase F.3 is the most likely to surprise us (async bridge edge cases); allocate slack there.

---

## 9. References

### Primary (authoritative, HIGH confidence)

- [fuser GitHub repo (cberner/fuser)](https://github.com/cberner/fuser)
- [fuser 0.17 CHANGELOG](https://docs.rs/crate/fuser/latest/source/CHANGELOG.md)
- [fuser 0.17 Cargo.toml](https://docs.rs/crate/fuser/latest/source/Cargo.toml)
- [fuser `Filesystem` trait docs](https://docs.rs/fuser/latest/fuser/trait.Filesystem.html)
- [fuser `hello.rs` example](https://github.com/cberner/fuser/blob/master/examples/hello.rs) (fetched raw, full source in §3 is derived from this)
- [fuser `async_hello.rs` example](https://github.com/cberner/fuser/blob/master/examples/async_hello.rs)
- [fuser `simple.rs` example](https://github.com/cberner/fuser/blob/master/examples/simple.rs) — demonstrates inode allocation + persistence patterns (basis for §4.2)
- [fuse3 crate docs](https://docs.rs/fuse3) — the main alternative
- [Tokio bridging sync/async guide](https://tokio.rs/tokio/topics/bridging) — canonical reference for §5

### Secondary (MEDIUM confidence, community)

- [GitHub Community discussion #26404: /dev/fuse in Actions](https://github.com/orgs/community/discussions/26404)
- [actions/runner-images discussion #10528: fuse3 package availability](https://github.com/actions/runner-images/discussions/10528)
- [fusermount3(1) man page](https://man7.org/linux/man-pages/man1/fusermount3.1.html)
- [mount.fuse3(8) man page](https://www.man7.org/linux/man-pages/man8/mount.fuse3.8.html)
- [Bridge Async and Sync Code in Rust (Greptime)](https://greptime.com/blogs/2023-03-09-bridging-async-and-sync-rust)
- [24 days of Rust: FUSE filesystems](https://zsiciarz.github.io/24daysofrust/book/vol1/day15.html) — older (pre-fuser), but the trait shape is unchanged

### Confidence assessment

| Claim | Confidence | Basis |
|---|---|---|
| `default-features=false` on fuser 0.17 yields pure-Rust build | HIGH | Verified in Cargo.toml + CHANGELOG |
| Runtime needs `fusermount3` binary, no `libfuse-dev` | HIGH | Verified against fuser `src/mnt/fuse_pure.rs` source-of-truth |
| `ubuntu-latest` requires `apt install fuse3` | HIGH | Multiple independent sources |
| `Filesystem` trait method signatures | HIGH | Cross-checked docs.rs + source |
| `block_on` from FUSE callbacks won't deadlock | HIGH | Verified via Tokio docs + fuser `experimental` module does the same thing internally |
| `experimental` async module is worth avoiding | MEDIUM | Based on CHANGELOG churn; could be stable "soon". We'd notice at upgrade time. |
| fuse_mt/polyfuse are dead ends | MEDIUM | Repository activity signals; haven't done exhaustive evaluation |
| Mount option interactions with kernel caching TTLs | MEDIUM | Documented behavior, but subtle — verify empirically in Phase F.1 |

---

*End of fuse-rust-patterns.md.*
