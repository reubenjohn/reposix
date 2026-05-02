# FUSE-in-Rust Patterns for reposix

**Scope:** Concrete implementation guidance for the `-fuse` crate of the reposix workspace. Mounts a virtual POSIX tree backed by an async HTTP client (simulator today, real issue trackers later). Linux-only, pure-Rust (no `libfuse-dev`), must run unprivileged on a GitHub Actions `ubuntu-latest` runner.

**Researched:** 2026-04-13
**Overall confidence:** HIGH for fuser crate internals & mount mechanics (verified against source). HIGH for GitHub Actions runner behavior (verified against runner-images + community discussions). MEDIUM for exact `experimental` API shape (rapidly changing — pin a version).

**Bottom line:** Use **`fuser` 0.17.x with `default-features = false`**. Implement the *synchronous* `Filesystem` trait. Bridge to async via a dedicated `tokio::runtime::Runtime` owned by the filesystem struct + a blocking `rt.block_on(...)` or `tokio::sync::oneshot` roundtrip inside each callback. Install `fuse3` (the runtime package, not `-dev`) on the CI runner. Do **not** adopt the `experimental` async trait yet — it shifts shape every release and is not required for our workload.

---

1. [fuser 0.15 / 0.16 / 0.17 API surface](./01-fuser-api-surface.md)
2. [Mounting inside a GitHub Actions runner](./02-github-actions-mount.md)
3. [Concrete skeleton: virtual markdown FS with read + write](./03-skeleton.md)
4. [Inode allocation strategy for a dynamic remote-backed FS](./04-inode-allocation.md)
5. [Async bridge: calling reqwest from sync FUSE callbacks](./05-async-bridge.md)
6. [Known gotchas](./06-gotchas.md)
7. [Library choice: fuser vs fuse3 vs fuse_mt vs polyfuse](./07-library-choice.md)
8. [Phase-planning guidance for gsd-planner](./08-phase-planning.md)
9. [References](./09-references.md)
