# Known gotchas

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
- **Use `DefaultPermissions`.** Mode bits stored on our Nodes become real. That maps directly to the RBAC-to-POSIX pattern in `docs/research/initial-report.md` §Governance.
