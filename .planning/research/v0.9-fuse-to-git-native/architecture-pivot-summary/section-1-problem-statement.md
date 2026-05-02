[index](./index.md)

# 1. Problem Statement

## FUSE is slow by design

The current architecture mounts a FUSE filesystem where every `cat` or `ls` triggers a live REST API call to the backing service (GitHub, Confluence, Jira). There is no caching layer between the agent and the network. This means:

- **Latency on every read.** `cat issues/2444.md` blocks on an HTTP round-trip. Agents that read dozens of files serially accumulate seconds of wall-clock delay.
- **Fan-out on directory listing.** `ls issues/` on a project with 10,000 Confluence pages means 10,000 API calls (or a single paginated call that returns 10,000 items and blocks until complete). Neither is acceptable for an autonomous agent loop.
- **No offline story.** If the network drops, the mount is dead. FUSE returns EIO on every read.

## FUSE has operational pain

Beyond performance, FUSE imposes environmental requirements that hurt portability:

- **`fusermount3` / `/dev/fuse` permissions.** The dev host lacks `pkg-config` and `libfuse-dev`, and we have no passwordless sudo to install them. WSL2 environments require additional kernel module configuration.
- **Build dependency on `fuser`.** The `fuser` crate with default features requires `libfuse-dev` headers at compile time. We already use `default-features = false` to avoid this, but it constrains what FUSE features we can use.
- **Mount lifecycle complexity.** Mount/unmount, stale mounts after crashes, `/etc/mtab` cleanup, and the `fuse-mount-tests` feature gate (which must be excluded from `cargo test --workspace` because FUSE tests are unsafe in WSL2) all add operational surface area.
- **Integration test fragility.** FUSE integration tests require `/dev/fuse`, `--release` builds, and `--test-threads=1`. They cannot run in standard CI without `fuse3` packages installed.

## The fundamental mismatch

The project's core thesis is: "the mount point IS a git working tree; `git diff` is the change set." But with FUSE, the mount is a virtual filesystem that *pretends* to be a git repo. Writes go through FUSE callbacks, not `git commit`. The change-tracking story is incomplete because the working tree is synthetic.
