# Inode allocation strategy for a dynamic remote-backed FS

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
