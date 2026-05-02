# References

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
