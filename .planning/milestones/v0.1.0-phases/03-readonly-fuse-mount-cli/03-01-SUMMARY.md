---
phase: 03-readonly-fuse-mount-cli
plan: 01
status: complete
commits:
  - hash: 032e979  # feat(03-01): inode registry + 5s-timeout HTTP fetch helpers
  - hash: 2acd9e4  # feat(03-01): read-only FUSE Filesystem impl + mount binary + readdir test
  - hash: 4fbccc9  # test(03-01): prove 5s timeout — stat returns <7s after backend dies
tests_added:
  reposix_core_integration: 2  # request_with_headers_rechecks_allowlist, request_with_headers_attaches_header
  reposix_fuse_lib_inode: 7
  reposix_fuse_lib_fetch: 7
  reposix_fuse_integration_readdir: 1
  reposix_fuse_integration_sim_death: 1  # #[ignore]-gated
requirements_closed:
  - FC-03-read
  - FC-05  # partial; completed by 03-02
  - SG-04  # enforcement at FUSE boundary
  - SG-07
---

# Phase 3-01 Summary: Read-only FUSE Impl

Three commits, ~645 insertions net (fuse crate + core tests), 15 new
passing unit tests + 1 default-on integration test + 1 `#[ignore]`-gated
integration test.

## What shipped

### `reposix_core::http::request_with_headers` (01-ext)

Additive method on the sealed `HttpClient` newtype:

```rust
pub async fn request_with_headers<U: IntoUrl>(
    &self,
    method: Method,
    url: U,
    headers: &[(&str, &str)],
) -> Result<reqwest::Response>;
```

Same per-call allowlist recheck as `request`; existing
`request`/`get`/`post`/`patch`/`delete` delegate to it with an empty
header slice so SG-01 stays locked in (compile-fail fixture at
`crates/reposix-core/tests/compile-fail/http_client_inner_not_pub.rs`
still passes). Two tests appended:

- `request_with_headers_rechecks_allowlist`: fast-reject (<500ms) for a
  non-allowlisted origin.
- `request_with_headers_attaches_header`: wiremock `header()` matcher
  proves the header actually hits the wire.

### `reposix-fuse`

| File | Role |
|------|------|
| `src/inode.rs` | `InodeRegistry` — `DashMap<IssueId, u64>` + reverse map + `AtomicU64` at `0x1_0000` |
| `src/fetch.rs` | `fetch_issues` / `fetch_issue` on `HttpClient::request_with_headers` with 5s `tokio::time::timeout` + `X-Reposix-Agent` header |
| `src/fs.rs` | `ReposixFs` + `Filesystem` impl (`init`/`getattr`/`lookup`/`readdir`/`read`) |
| `src/lib.rs` | `MountConfig { mount_point, origin, project, read_only }` + `Mount::open(&cfg)` → `spawn_mount2` with `FSName`/`Subtype`/`DefaultPermissions`/`RO` |
| `src/main.rs` | clap-derive CLI: `reposix-fuse <mount> --backend <origin> [--project demo] [--read-only]` |
| `tests/readdir.rs` | wiremock-backed mount test; ls/cat assertions on `0001.md 0002.md 0003.md` |
| `tests/sim_death_no_hang.rs` | `#[ignore]` — `timeout 7 stat <mount>/0001.md` after backend dies; 1.23s measured |
| `Cargo.toml` | +`rustix = "0.38" { features = ["process"] }` for safe `getuid`/`getgid` |

### Commit hashes

| Task | SHA | Subject |
|------|-----|---------|
| Task 1 (part a) | merged into `3c004f6` | `HttpClient::request_with_headers` (see deviation note below) |
| Task 1 (part b) | `032e979` | `feat(03-01): inode registry + 5s-timeout HTTP fetch helpers` |
| Task 2 | `2acd9e4` | `feat(03-01): read-only FUSE Filesystem impl + mount binary + readdir test` |
| Task 3 | `4fbccc9` | `test(03-01): prove 5s timeout — stat returns <7s after backend dies` |

## Inode strategy (as shipped)

- `1` = root directory, hard-coded.
- `2..=0xFFFF` = reserved for future synthetic files (`/.reposix/audit`
  etc.). Registry never allocates here.
- `0x1_0000+` = issues, allocated monotonically via `AtomicU64`.
- `refresh(&[IssueId])` returns `(ino, id)` pairs in input order for the
  `ReplyDirectory` protocol. Stale IDs are *not* evicted — open file
  handles stay valid even if the backend drops an issue, and the next
  `read` against a deleted issue surfaces as backend 404 → `ENOENT`.

## Exact timeout layering

Two layers of 5-second ceiling, each sufficient on its own:

1. **Library level** — `ClientOpts::default().total_timeout =
   Duration::from_secs(5)` is baked into every `HttpClient` built via
   `reposix_core::http::client(...)`. If we later tune this for a
   use-case with longer-running requests, the next layer still holds.
2. **Per-call** — `tokio::time::timeout(Duration::from_secs(5),
   http.request_with_headers(...))` inside `fetch::fetch_issue` +
   `fetch::fetch_issues`. Returns `FetchError::Timeout` on elapsed, which
   the FUSE callback maps to `libc::EIO`.

Measured wall clock on the dev host (`sim_death_no_hang.rs`,
`timeout 7 stat <mount>/0001.md`): **1.23s** to surface EIO after the
backend dies.

## Fuser 0.17 quirks discovered

- `MountOption::AllowOther` does not exist in 0.17 — ACL is set via
  `fuser::Config::acl: SessionACL { All | RootAndOwner | Owner }`.
  The grep invariant in the WAVES exit check (`grep -c AllowOther ==
  0`) still holds trivially.
- `MountOption::AutoUnmount` is **incompatible** with
  `SessionACL::Owner` (fuser refuses the session at startup). Dropping
  `AutoUnmount` was the only option consistent with SG
  "no-allow-other". The CLI's `MountProcess::Drop` watchdog
  (`fusermount3 -u` with 3s SIGKILL fallback) substitutes.
- `fuser::Config` is `#[non_exhaustive]` — cannot use struct-literal
  update syntax; must `let mut c = Config::default(); c.mount_options
  = ...;`.
- `Filesystem::init` returns `Result<(), std::io::Error>`, not
  `Result<(), i32>` as research §3 claimed.
- `readdir`/`read` offsets are `u64`, not `i64` (diverges from research
  doc — the fuser API tightened).
- `libc::getuid` / `libc::getgid` are `unsafe fn` in libc 0.2.x — kept
  `#![forbid(unsafe_code)]` at the crate root by adding `rustix` as a
  dep and calling `rustix::process::getuid().as_raw()`.

## Deviations from plan

1. **[Rule 3 — Fix attribution] Phase 2 commit scooped our 01-ext
   extension.** The plan called for a standalone
   `feat(01-ext): HttpClient::request_with_headers for audit
   attribution` commit. Phase 2 and Phase 3 were running in parallel;
   Phase 2's commit `3c004f6 feat(02-01)` happened to stage our
   modified `crates/reposix-core/src/http.rs` alongside its own sim
   files (`git add -A`-style), so `request_with_headers` is physically
   on `main` under that commit message. The code is correct and tests
   pass; only the attribution is muddled. No history rewrite attempted
   (would be a force-push). Documented here as the authoritative
   mapping: Task 1 step 0 → `3c004f6`.

2. **[Rule 3 — API adaptation] Drop `AutoUnmount`.** See fuser 0.17
   quirks above. The CLI's `MountProcess` watchdog (Plan 03-02 Task 1)
   is the compensating control.

3. **[Rule 2 — Correctness] `libc` → `rustix` for uid/gid.** Plan
   assumed `libc::getuid` / `getgid` were safe; they're `unsafe fn` in
   libc 0.2.x. Added `rustix` to fuse + cli deps to keep
   `#![forbid(unsafe_code)]` intact. Minimal dep surface.

## Real-world verification

Smoke-tested manually against a live Phase-2 sim BEFORE committing:

```console
$ ./target/debug/reposix-sim --bind 127.0.0.1:17878 --ephemeral \
      --seed-file crates/reposix-sim/fixtures/seed.json &
$ ./target/debug/reposix-fuse /tmp/reposix-mnt \
      --backend http://127.0.0.1:17878 --project demo &
$ ls /tmp/reposix-mnt
0001.md  0002.md  0003.md
$ head -1 /tmp/reposix-mnt/0001.md
---
$ grep -l database /tmp/reposix-mnt/*.md
/tmp/reposix-mnt/0001.md
```

All 3 positive ROADMAP SC (#1, #2, #3) observed under real kernel FUSE,
not just wiremock. SC #4 proven by `sim_death_no_hang.rs` (1.23s).
