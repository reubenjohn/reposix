# Library choice: fuser vs fuse3 vs fuse_mt vs polyfuse

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
