# fuser 0.15 / 0.16 / 0.17 API surface

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
