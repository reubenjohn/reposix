# Phase 3: Read-only FUSE mount + CLI orchestrator — Context

**Gathered:** 2026-04-13
**Status:** Ready for planning
**Source:** Auto-generated from PROJECT.md + ROADMAP.md + research/ (discuss step skipped per user instruction)

<domain>
## Phase Boundary

**In scope:**
- `reposix-fuse` becomes a working FUSE daemon. Mount semantics:
  - `/mnt/reposix/` (mount root) — readdir lists nothing here (phase 4 can add a `README` file).
  - Actually, since Phase 1's filename rule is `<id>.md`, the FUSE mount presents issues at the mount root directly: `/mnt/reposix/0001.md`, `/mnt/reposix/0002.md`, etc.
  - Operations supported: `init`, `getattr`, `lookup`, `readdir`, `read`. All write ops return `EROFS` (read-only mount for MVD).
  - Inode allocation: monotonic counter starting at `0x1_0000`, persistent issue_id→inode map in `DashMap<IssueId, u64>` rebuilt on every `readdir` (refreshes from sim).
  - Lazy fetch: `readdir` calls `GET /projects/:slug/issues`; `read` calls `GET /projects/:slug/issues/:id`. Both have a 5-second timeout; on timeout return `EIO` to the kernel (SG-07).
  - Filename validator from Phase 1 is called on every path-bearing op; rejection returns `EINVAL`.
  - Tokio runtime owned by the FS struct (per FUSE research §5.2). Sync FUSE callbacks `block_on` an HTTP call.
- `reposix-cli` (the top-level `reposix` binary) becomes the orchestrator:
  - `reposix sim [flags]` — delegates to `reposix-sim` (spawns it as a child process or links the lib and `tokio::spawn`s it; whichever is simpler).
  - `reposix mount <mount_point> --backend http://127.0.0.1:7878 --project demo` — spawns the FUSE session in the foreground (Ctrl-C to unmount).
  - `reposix demo` — orchestrates: spawn sim → mount → run scripted `ls/cat/grep` → tail audit log → cleanup.
  - All HTTP via `reposix_core::http::client()` (allowlist enforced).
- Tests:
  - `crates/reposix-fuse/tests/readdir.rs`: integration test against a `wiremock` backend, mounts on a tempdir, asserts `ls` returns the expected files.
  - `crates/reposix-fuse/tests/sim_death_no_hang.rs` (`#[ignore]`-gated): boots sim, mounts, kills sim, asserts `stat <file>` returns within 7s with non-zero exit.
  - `crates/reposix-cli/tests/cli.rs`: `cargo run -- --help` shows all subcommands; `reposix demo` exits 0 in <30s.

**Out of scope:**
- Write path (`write`, `create`, `unlink`, `setattr`) — Phase S.
- Subdirectory hierarchy (`projects/<slug>/<id>.md`) — for v0.1 we mount one project per mount, project-as-filesystem-root.
- TTL tuning under load — research §"Open Questions" notes this; v0.1 uses default 1s TTL.
- macOS support — out of scope per PROJECT.md.

</domain>

<decisions>
## Implementation Decisions

### FUSE crate choice
- `fuser` 0.17 with `default-features = false` (already in `Cargo.toml`).
- Use the **sync** `Filesystem` trait, NOT `experimental::AsyncFilesystem` (per FUSE research §3 — experimental API churn risk).
- Tokio runtime owned by the FS struct: `tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build()`. Stored as `Arc<tokio::runtime::Runtime>` so callbacks can `runtime.block_on(...)`.
- Mount options: `MountOption::FSName("reposix")`, `MountOption::AutoUnmount`, `MountOption::DefaultPermissions`, `MountOption::AllowOther` is OFF (security).

### Inode strategy
- Reserved: `1` = root, `2..0xFFFF` reserved for synthetic future files (e.g. `/.reposix/audit.log`).
- Dynamic: `0x1_0000+` for issues. `DashMap<IssueId, u64>` for forward lookup, `DashMap<u64, IssueId>` for reverse. AtomicU64 next-inode counter.
- On `readdir`, refresh both maps from a fresh `GET /projects/:slug/issues`. Don't expire entries — old inodes remain valid until the FS dies.

### File rendering
- Filename: `format!("{:04}.md", issue.id.0)` (zero-padded to 4 digits for visual order; the validator accepts any number of digits so 5+ digit issues still parse).
- File contents: `reposix_core::issue::frontmatter::render(&issue)` — already implemented in Phase 1.
- `st_size`: byte length of the rendered string. Critical: `cat` truncates to `st_size`, so this MUST match what `read` returns. Cache the rendered string per inode for the duration of the open.
- `st_mode`: `0o100444` (regular file, read-only-by-all). UID/GID from `libc::getuid()` / `libc::getgid()`.
- Timestamps: from `Issue.created_at` and `Issue.updated_at`.

### HTTP client
- Single `reqwest::Client` built via `reposix_core::http::client(ClientOpts::default())`, shared `Arc<Client>`.
- Helper: `async fn fetch_issue(client, origin, project, id) -> Result<Issue, Error>` — wraps the URL build, allowlist recheck, and 5s timeout.
- All requests include `X-Reposix-Agent: reposix-fuse-{pid}` header so the simulator audit log can attribute.

### CLI orchestrator
- `clap` derive style.
- `reposix sim` — for v0.1, spawn `cargo run -p reposix-sim` as a child process (simpler than linking; the demo scripts will work the same way). v0.2 can flatten.
- `reposix mount` — calls into `reposix-fuse` library. Foreground; SIGINT unmounts cleanly.
- `reposix demo` — bash-style orchestration in Rust:
  1. Start sim subprocess (own a `Child` handle, kill on drop).
  2. Wait for `GET /healthz` to return 200 (5s timeout).
  3. Spawn FUSE mount in a child process (so the demo can control it independently).
  4. Wait for `mountpoint -q <mount>` to succeed (3s timeout).
  5. Run scripted `ls`, `cat <one file>`, `grep -ril database` against the mount, capturing output.
  6. Tail the audit DB and print the last 5 rows.
  7. Cleanup: `fusermount3 -u <mount>` then kill the sim.
- All output is structured + colored where helpful (`tracing` formatter).

### Tests
- `wiremock` for the backend in unit tests (already a `reposix-core` dev dep; add to `reposix-fuse` if not).
- The mount tests must run on Linux only (`#[cfg(target_os = "linux")]`). They require `fusermount3`.
- The "sim_death" test is `#[ignore]`-gated so it doesn't run in `cargo test` by default; runs in CI's `integration` job.

### Claude's discretion
- Exact wire shape of the HTTP fetch responses (must match what Phase 2 emits — there's coordination needed; the FUSE planner should design `Issue` JSON shape to match `reposix_core::Issue` Serialize output, which is what Phase 2 will also emit).
- Demo script verbosity / colors.
- Whether the FUSE mount is daemonized (`fuser::spawn_mount2`) or run foreground in `reposix mount`. Foreground is simpler for the demo; daemonized is nicer for `reposix demo`.

</decisions>

<canonical_refs>
## Canonical References

- `.planning/research/fuse-rust-patterns.md` — primary blueprint. §3 (skeleton), §5 (async bridge), §6 (gotchas), §10 (sub-phase breakdown).
- `.planning/research/threat-model-and-critique.md` — SG-07 (no kernel hang), SG-04 (filename rules at FUSE boundary).
- `crates/reposix-core/src/path.rs` — `validate_issue_filename` (must be called at every path-bearing op).
- `crates/reposix-core/src/http.rs` — `client()` factory (the only legal HTTP client constructor).
- `crates/reposix-core/src/issue.rs` — `Issue`, `frontmatter::render`.
- [fuser 0.17 docs](https://docs.rs/fuser/0.17/) — Filesystem trait.
- [libc::getuid](https://docs.rs/libc/latest/libc/fn.getuid.html)

</canonical_refs>

<specifics>
## Specific Ideas

- The `fuser::Session::run()` call blocks forever until unmounted. Wrap it in `tokio::task::spawn_blocking` from the CLI so the orchestrator can shut down cleanly.
- The "demo" subcommand should print a banner before each step ("[1/6] Starting simulator..."). Makes the asciinema recording legible.
- `reposix mount --read-only` flag is a redundant alias for the v0.1 default behavior; keep it for forward-compat with Phase S.

</specifics>

<deferred>
## Deferred Ideas

- Notification-based cache invalidation (research §"Open Questions") — needs the simulator to expose an SSE endpoint. v0.2.
- Mount inside a Linux user namespace for stronger isolation. v0.2.
- `/.reposix/audit` synthetic file inside the mount that exposes recent audit rows.

</deferred>

---

*Phase: 03-readonly-fuse-mount-cli*
*Context: 2026-04-13 via auto-mode*
