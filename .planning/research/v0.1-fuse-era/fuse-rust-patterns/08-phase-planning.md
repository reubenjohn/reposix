# Phase-planning guidance for gsd-planner

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
