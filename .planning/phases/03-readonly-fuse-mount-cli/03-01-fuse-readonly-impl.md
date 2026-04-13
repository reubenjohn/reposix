---
phase: 03-readonly-fuse-mount-cli
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/reposix-fuse/Cargo.toml
  - crates/reposix-fuse/src/lib.rs
  - crates/reposix-fuse/src/fs.rs
  - crates/reposix-fuse/src/inode.rs
  - crates/reposix-fuse/src/fetch.rs
  - crates/reposix-fuse/src/main.rs
  - crates/reposix-fuse/tests/readdir.rs
  - crates/reposix-fuse/tests/sim_death_no_hang.rs
autonomous: true
requirements:
  - FC-03-read
  - FC-05
  - SG-04
  - SG-07
user_setup: []

must_haves:
  truths:
    - "Mounting /tmp/reposix-mnt against a reposix-compatible backend presents one regular file per issue at the mount root, named `<zero-padded-id>.md`."
    - "`cat <mount>/0001.md` returns the exact bytes of `reposix_core::issue::frontmatter::render(issue)` for the backend's issue 1."
    - "Every path-bearing FUSE op (lookup, any future path resolution) rejects non-`<digits>.md` names with EINVAL via `validate_issue_filename`."
    - "`readdir` and `read` complete within 5s or return EIO — the kernel NEVER hangs on a dead backend."
    - "`fusermount3 -u <mount>` completes within 3s after the FUSE session exits."
  artifacts:
    - path: "crates/reposix-fuse/src/fs.rs"
      provides: "ReposixFs struct + Filesystem impl (init, getattr, lookup, readdir, read)"
      contains: "impl Filesystem for ReposixFs"
    - path: "crates/reposix-fuse/src/inode.rs"
      provides: "InodeRegistry (DashMap<IssueId,u64> + reverse map + AtomicU64 counter @ 0x1_0000)"
    - path: "crates/reposix-fuse/src/fetch.rs"
      provides: "fetch_issues + fetch_issue helpers built on reposix_core::http::request, each wrapped in 5s timeout"
    - path: "crates/reposix-fuse/src/lib.rs"
      provides: "Public Mount + MountConfig; re-exports ReposixFs and mount helpers"
    - path: "crates/reposix-fuse/src/main.rs"
      provides: "`reposix-fuse` binary: `reposix-fuse <mount_point> --backend <origin> --project <slug>`"
    - path: "crates/reposix-fuse/tests/readdir.rs"
      provides: "wiremock-backed integration test: mount a tempdir, assert ls shows `0001.md 0002.md 0003.md`"
    - path: "crates/reposix-fuse/tests/sim_death_no_hang.rs"
      provides: "#[ignore]-gated test proving `stat` on a mounted file returns within 7s after the backend dies, with non-zero exit"
  key_links:
    - from: "crates/reposix-fuse/src/fs.rs :: readdir"
      to: "crates/reposix-fuse/src/fetch.rs :: fetch_issues"
      via: "self.rt.block_on(fetch_issues(...))"
      pattern: "rt\\.block_on"
    - from: "crates/reposix-fuse/src/fs.rs :: lookup"
      to: "reposix_core::path::validate_issue_filename"
      via: "direct call on every name argument BEFORE touching the inode registry"
      pattern: "validate_issue_filename"
    - from: "crates/reposix-fuse/src/fetch.rs"
      to: "reposix_core::http::client + reposix_core::http::request"
      via: "sole allowed HTTP construction + send path"
      pattern: "reposix_core::http::(client|request)"
---

<objective>
Turn `reposix-fuse` from a skeleton into a working read-only FUSE daemon.

Purpose: satisfies ROADMAP Phase 3 success criteria #1–#4 (ls lists issues,
cat returns frontmatter, grep -r works, kernel never hangs >5s when backend
dies) and SG-07 (FUSE never blocks the kernel forever).

Output: `crates/reposix-fuse` builds an FS that, when pointed at a Phase-2
simulator on `http://127.0.0.1:7878` with project `demo`, presents the three
seeded issues as `0001.md`, `0002.md`, `0003.md` at the mount root.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/03-readonly-fuse-mount-cli/03-CONTEXT.md
@.planning/research/fuse-rust-patterns.md
@CLAUDE.md
@crates/reposix-core/src/path.rs
@crates/reposix-core/src/http.rs
@crates/reposix-core/src/issue.rs
@crates/reposix-fuse/src/lib.rs
@crates/reposix-fuse/Cargo.toml

<interfaces>
<!-- Contracts the executor consumes directly — no codebase spelunking. -->

From `reposix_core::path`:
```rust
pub fn validate_issue_filename(name: &str) -> Result<IssueId>;
// IssueId(u64). Returns Error::InvalidPath on junk.
// MUST be called on every path component that crosses the FUSE boundary.
```

From `reposix_core::http`:
```rust
pub struct ClientOpts { pub total_timeout: Duration, pub user_agent: Option<String> }
impl Default for ClientOpts { /* 5s timeout, reposix/<ver> UA */ }
pub fn client(opts: ClientOpts) -> Result<reqwest::Client>;     // ONLY legal ctor
pub async fn request(client: &reqwest::Client, method: reqwest::Method, url: &str)
    -> Result<reqwest::Response>;                                // re-checks allowlist
```

From `reposix_core::issue`:
```rust
pub struct IssueId(pub u64);                  // Display: "{}", e.g. "42"
pub struct Issue { pub id: IssueId, pub title: String, pub status: IssueStatus,
                   pub assignee: Option<String>, pub labels: Vec<String>,
                   pub created_at: DateTime<Utc>, pub updated_at: DateTime<Utc>,
                   pub version: u64, pub body: String }
pub mod frontmatter { pub fn render(issue: &Issue) -> Result<String>; }
```

Phase 2 simulator wire shape (already decided in `02-CONTEXT.md`):
- `GET  http://<origin>/projects/<slug>/issues`        → `[Issue, ...]`  (JSON array)
- `GET  http://<origin>/projects/<slug>/issues/<id>`   → `Issue`         (single JSON)
- `GET  http://<origin>/healthz`                       → 200 OK (used by 03-02; informational here)
- Issues carry `id: IssueId(u64)`. Filename at FUSE is `format!("{:04}.md", id.0)`.
- FUSE requests MUST send header `X-Reposix-Agent: reposix-fuse-{pid}` so the audit log attributes.
</interfaces>

<fuser_api_hints>
<!-- From .planning/research/fuse-rust-patterns.md §3, §5, §6. Executor should
     verify actual fuser 0.15 type names against installed crate (the workspace
     dep may be 0.15 or 0.17; research doc uses 0.17). Stick to the sync
     Filesystem trait regardless of version. -->

- `fuser::MountOption::{FSName,Subtype,AutoUnmount,DefaultPermissions}`. DO NOT
  use `AllowOther` (SG / CONTEXT).
- `fuser::spawn_mount2(filesystem, path, &opts) -> io::Result<BackgroundSession>`.
  Drop the session to unmount. We use this for both the binary (foreground main
  keeps it alive until SIGINT) and the `readdir` integration test.
- `Filesystem::init(&mut self, _req, _config) -> Result<(), libc::c_int>` — no-op
  return Ok(()).
- `getattr(&self, _req, ino, _fh, reply: ReplyAttr)` — for ino==1 return root
  dir attrs (cache in `root_attr` field, computed once at construction). For
  ino in registry, use cached `Issue` → `FileAttr` with
  `size = rendered.len() as u64`, `kind = RegularFile`, `perm = 0o444`,
  `uid = libc::getuid()`, `gid = libc::getgid()`,
  `atime/mtime = updated_at`, `ctime/crtime = created_at`. Unknown → ENOENT.
- `lookup(&self, _req, parent, name, reply: ReplyEntry)` — require parent==1
  (else ENOTDIR). Convert `name: &OsStr` to `&str` via `to_str()` — non-UTF-8
  → EINVAL. Call `validate_issue_filename(s)` — Err → EINVAL. Then consult
  the cache; on miss, `rt.block_on(fetch_issue(...))`; on 5s-timeout or
  transport error → EIO. On HTTP 404 → ENOENT.
- `readdir(&self, _req, ino, _fh, offset, mut reply: ReplyDirectory)` — ino==1
  only. Fetch the full issue list via `rt.block_on(fetch_issues(...))` (5s
  timeout; on timeout → EIO). Refresh the registry, then emit `.`, `..`, then
  each issue as `(ino, RegularFile, "{:04}.md")`. Use the `offset`/`reply.add`
  protocol exactly as research §3.5.
- `read(&self, _req, ino, _fh, offset, size, _flags, _lock, reply: ReplyData)`
  — resolve `ino → IssueId` via reverse map. Use cached rendered string if
  present (reset cache when readdir refreshes); else `rt.block_on(fetch_issue(...))`
  then `frontmatter::render`. Slice `bytes[offset..offset+size]` bounded by
  length. Unknown ino → ENOENT. Backend failure / timeout → EIO.
- Every other `Filesystem` method is a write op: default impl returns ENOSYS,
  which the kernel treats as EROFS for our purposes. Do NOT override them.
</fuser_api_hints>

<decisions>
- **Filename format: `{:04}.md` (4-digit zero-padded, per CONTEXT.md §File rendering).**
  ROADMAP Phase 3 SC #1 literally says `DEMO-1.md DEMO-2.md DEMO-3.md`; Phase 1's
  `validate_issue_filename` only accepts `<digits>.md`. The Phase-2 seed already
  uses numeric IDs (`IssueId(1)`…`IssueId(3)`). We deviate from ROADMAP literal
  text and use `0001.md 0002.md 0003.md`. The ROADMAP success criteria are
  rewritten in each task's `<verify>` block with the actual expected strings.
  This is documented in `03-WAVES.md` so downstream phases inherit the decision.
- **No-unsafe remains: `#![forbid(unsafe_code)]` at crate root.** `libc::getuid`
  / `libc::getgid` are safe FFI (no unsafe required in 0.2+). If an older libc
  flags them as unsafe, wrap in a tiny cached OnceLock rather than opening an
  unsafe block.
- **Runtime: owned by `ReposixFs`.** `tokio::runtime::Builder::new_multi_thread()
  .worker_threads(2).enable_all().build()` stored as `Arc<Runtime>`. FUSE
  callbacks live on non-runtime OS threads, so `rt.block_on` is deadlock-safe
  (research §5.2).
- **Timeout: 5s on every HTTP op, enforced at two layers.** `ClientOpts::default()`
  already sets 5s total timeout; we additionally wrap each awaited call in
  `tokio::time::timeout(Duration::from_secs(5), ...)` inside `fetch.rs` so a
  library-level misconfiguration cannot degrade us. On elapsed-timeout return
  a typed `FetchError::Timeout` which the callback maps to `libc::EIO`.
- **Inode strategy: AtomicU64 starting at `0x1_0000`.** Registry persists only
  in-process (we do NOT open SQLite here — that's phase S). Root = 1.
- **Cache: `DashMap<u64, Arc<CachedFile>>` where `CachedFile { issue: Issue,
  rendered: String, size: u64 }`.** Populated on fetch, used by getattr/read.
  Invalidated by the next `readdir` refresh (overwritten wholesale).
- **TTLs: 1s for both ReplyEntry and ReplyAttr** (CONTEXT.md: default).
- **Tests gated**: `#[cfg(target_os = "linux")]` on the test modules. The
  `sim_death_no_hang` test is `#[ignore]`-gated; the `readdir` test runs in
  default `cargo test`.
</decisions>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| kernel → FUSE daemon | Untrusted filename bytes arrive as `&OsStr` in lookup. |
| FUSE daemon → backend (sim/remote) | Outbound HTTP; must go via the allowlisted factory. |
| backend → FUSE daemon | Issue JSON is tainted per PROJECT.md (agent-seeded simulator counts). |

## STRIDE Threat Register

| ID | Category | Component | Disposition | Mitigation |
|----|----------|-----------|-------------|------------|
| T-03-01 | Tampering | `lookup` path arg | mitigate | Call `reposix_core::path::validate_issue_filename` on every name before touching the registry; non-UTF-8 → EINVAL; `../etc/passwd.md` rejected by the validator. |
| T-03-02 | Denial of Service | sync FUSE callback blocking on dead backend | mitigate | `tokio::time::timeout(5s, ...)` around every `.await`; on elapsed → EIO. Proven by `sim_death_no_hang.rs`. |
| T-03-03 | Information Disclosure | egress to non-allowlisted origin | mitigate | All HTTP goes through `reposix_core::http::request` which re-checks `REPOSIX_ALLOWED_ORIGINS` per call. No direct `reqwest::Client::new()` anywhere (clippy lint already set up in Phase 1). |
| T-03-04 | Elevation of Privilege | other-user access to mount | mitigate | `MountOption::AllowOther` OFF; rely on `DefaultPermissions` + `0o444` mode bits. |
| T-03-05 | Repudiation | backend receives anonymous FUSE traffic | mitigate | Every outbound request carries `X-Reposix-Agent: reposix-fuse-{pid}` so the Phase-2 audit log attributes (audit middleware captures the header). |
| T-03-06 | Spoofing | attacker-controlled issue body rendered into FS content | accept | Bodies are bytes-in-bytes-out; no template expansion, no shell interpolation. `frontmatter::render` already escapes YAML (tested in Phase 1). |
| T-03-07 | Tampering | integer overflow in `offset+size` for `read` | mitigate | Use `usize` saturating arithmetic (`offset.min(len)`, `(offset+size).min(len)`) rather than wrapping adds. |
</threat_model>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Inode registry + fetch helpers (the foundation)</name>
  <files>
    crates/reposix-fuse/Cargo.toml,
    crates/reposix-fuse/src/inode.rs,
    crates/reposix-fuse/src/fetch.rs,
    crates/reposix-fuse/src/lib.rs
  </files>
  <behavior>
    <!-- inode.rs -->
    - `InodeRegistry::new()` starts `next` at `0x1_0000` (65_536).
    - `intern(IssueId(7))` called the first time returns `0x1_0000`; second time
      returns the same value (idempotent).
    - `intern(IssueId(42))` after `intern(IssueId(7))` returns `0x1_0001`.
    - `lookup_ino(0x1_0000)` returns `Some(IssueId(7))`.
    - `lookup_ino(5)` returns `None` (2..=0xFFFF reserved range is unmapped).
    - `refresh(&[IssueId(1), IssueId(2), IssueId(3)])` returns a `Vec<(u64, IssueId)>`
      of length 3 in input order; repeat calls return stable inodes for repeat IDs.
    <!-- fetch.rs -->
    - `fetch_issues(&client, origin, project)` hits `{origin}/projects/{project}/issues`,
      5s timeout, returns `Vec<Issue>`. `wiremock` mock returning `[{...},{...},{...}]`
      → 3 issues parsed.
    - `fetch_issue(&client, origin, project, IssueId(1))` hits
      `{origin}/projects/{project}/issues/1`, 5s timeout, returns `Issue`.
    - `fetch_issue` against a wiremock that sleeps 10s returns
      `Err(FetchError::Timeout)` within 5.5s wall-clock (measured in test).
    - `fetch_issue` against a 404 returns `Err(FetchError::NotFound)`; a 500
      returns `Err(FetchError::Http(_))`; a non-allowlisted origin returns
      `Err(FetchError::Origin(_))` (bubble from `reposix_core::http::request`).
    - Both helpers attach header `X-Reposix-Agent: reposix-fuse-{pid}`. Verified
      via wiremock matcher.
  </behavior>
  <action>
    1. `Cargo.toml` updates (Cargo.toml lives at `crates/reposix-fuse/Cargo.toml`):
       - Ensure `dashmap`, `parking_lot`, `tokio`, `reqwest`, `libc`, `fuser`,
         `serde_json`, `chrono` are in `[dependencies]` (all are workspace deps
         already — just list them if not present).
       - Add `[dev-dependencies]`: `wiremock = "0.6"`, `tempfile = "3"`,
         `tokio = { workspace = true, features = ["macros","rt-multi-thread","time"] }`.
    2. `src/inode.rs` — `InodeRegistry` struct per `<behavior>`, using
       `DashMap<IssueId, u64>` + `DashMap<u64, IssueId>` + `AtomicU64`.
       `intern`, `lookup_ino`, `refresh` public-in-crate. Unit tests inline.
    3. `src/fetch.rs` — `FetchError` enum (`Timeout`, `NotFound`, `Http(reqwest::Error)`,
       `Origin(reposix_core::error::Error)`, `Parse(serde_json::Error)`);
       `fetch_issues` and `fetch_issue` as specified. Both call
       `reposix_core::http::request(client, Method::GET, &url)` — the ONLY
       legal path per SG-01 — and wrap the whole `.await` chain in
       `tokio::time::timeout(Duration::from_secs(5), ...)`. Attach the
       `X-Reposix-Agent: reposix-fuse-{pid}` header by cloning the underlying
       `RequestBuilder` after `request` returns its `Response`... actually
       `reposix_core::http::request` does not expose a builder, so instead:
       use `client.request(Method::GET, parsed_url).header(...)` directly AFTER
       calling a tiny `check_origin(url)?` helper that replicates the allowlist
       check. Wait — re-read `crates/reposix-core/src/http.rs`: the allowlist
       IS inside `request`. Executor: refactor by *copying* `check_origin` as a
       small pub helper inside `reposix-core::http` in this commit (simple
       additive change to `http.rs` — extract the `load_allowlist_from_env()
       + matches()` block into `pub(crate) fn check_allowed(url: &Url) -> Result<()>`
       and re-export it as `pub fn check_allowed(url: &str) -> Result<()>`).
       Then `fetch.rs` calls `reposix_core::http::check_allowed(&url)?` THEN
       uses `client.request(...).header(...).send().await`. This preserves the
       SG-01 invariant (no unauthorised ctor; allowlist is enforced) and lets
       us attach headers cleanly.

       **Simpler alternative** if executor judges the above refactor too large:
       add header support to `reposix_core::http::request` itself by accepting
       an `impl IntoIterator<Item=(&str,&str)>` header list. This is a one-line
       change to the signature plus a loop. Either approach is fine — pick the
       smaller diff.

    4. `src/lib.rs` — replace the current placeholder `Mount`:
       - Keep `MountConfig { mount_point, origin, read_only }`.
       - Add `pub struct Mount { session: fuser::BackgroundSession }` (fuser
         version-gated; if fuser 0.15 lacks `BackgroundSession` import path
         use `fuser::spawn_mount2`'s actual return type). Drop = unmount.
       - `Mount::open(cfg) -> anyhow::Result<Self>`: create mount dir if
         missing, construct `ReposixFs` (from Task 2), spawn, store session.
       - Expose `pub use fs::ReposixFs;` and `pub use inode::InodeRegistry;` —
         mod declarations added (`mod fs;` lands in Task 2; for this task
         add only `pub mod inode;` and `pub mod fetch;`).
    5. No `unsafe` blocks. `#![forbid(unsafe_code)]` must remain at crate root.
    6. `cargo fmt -p reposix-fuse`, `cargo clippy -p reposix-fuse --all-targets -- -D warnings`.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && cargo test -p reposix-fuse --lib inode::tests fetch::tests -- --nocapture</automated>
    <manual>
      `cargo clippy -p reposix-fuse --all-targets -- -D warnings` clean;
      `grep -RIn 'reqwest::Client::new\|reqwest::ClientBuilder' crates/reposix-fuse/ --include='*.rs' | wc -l` equals 0.
    </manual>
  </verify>
  <done>
    Inode registry + HTTP fetch helpers compile, unit-tested, and lint-clean.
    All outbound HTTP flows through `reposix_core::http`. No unsafe; no direct
    reqwest ctor. Commit subject:
    `feat(03-01): inode registry + 5s-timeout HTTP fetch helpers`.
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Filesystem impl + mount binary + readdir integration test</name>
  <files>
    crates/reposix-fuse/src/fs.rs,
    crates/reposix-fuse/src/lib.rs,
    crates/reposix-fuse/src/main.rs,
    crates/reposix-fuse/tests/readdir.rs
  </files>
  <behavior>
    <!-- fs.rs -->
    - `ReposixFs::new(origin: String, project: String) -> anyhow::Result<Self>`
      builds its own Tokio multi-thread runtime (2 workers), builds the reqwest
      client via `reposix_core::http::client(ClientOpts::default())`, seeds
      root inode (1) with cached attrs computed from `libc::getuid/getgid`
      and `SystemTime::now()`.
    - `impl Filesystem for ReposixFs` implements init (Ok), getattr, lookup,
      readdir, read per `<fuser_api_hints>`. No other methods overridden.
    - `lookup` on `"../etc/passwd"` → EINVAL. `lookup` on `"0001.md"` with a
      live backend → populated entry with the cached attrs. `lookup` on
      `"9999.md"` with 404 backend → ENOENT.
    - `readdir` on inode 1 → entries `.`, `..`, and one entry per issue in the
      backend's `GET /projects/{slug}/issues` response, named
      `format!("{:04}.md", issue.id.0)`, in ID-ascending order.
    - `read(ino, 0, size, ...)` returns bytes of `frontmatter::render(&issue)`
      truncated to `size`. `read` past EOF returns an empty slice (not error).
    - All HTTP paths timed out (backend unresponsive 10s) → callback replies
      EIO within 6s wall-clock.
    <!-- main.rs -->
    - `reposix-fuse <mount_point> --backend <origin> --project <slug>` mounts
      in the foreground; Ctrl-C unmounts cleanly via `AutoUnmount`.
    <!-- tests/readdir.rs -->
    - `#[cfg(target_os = "linux")]` + `#[tokio::test(flavor="multi_thread")]`
      (note: the test spawns FUSE on a tempdir via `Mount::open`, then shells
      out `ls`-equivalent via `std::fs::read_dir`). wiremock serves
      `GET /projects/demo/issues` returning 3 issues with IDs 1, 2, 3. Assert
      `std::fs::read_dir(mount).sorted_names()` equals
      `["0001.md","0002.md","0003.md"]`. Then read `0001.md` via
      `std::fs::read_to_string`, assert it starts with `"---\n"` and contains
      `"id: 1"`. Finally drop the `Mount` and assert the mount point is empty
      / unmounted within 3s.
  </behavior>
  <action>
    1. Create `src/fs.rs` implementing `ReposixFs` exactly as in research §3
       skeleton but read-only. Key adaptations:
       - Store `Arc<tokio::runtime::Runtime>`, `Arc<reqwest::Client>`,
         `origin: String`, `project: String`, `registry: InodeRegistry`,
         `cache: DashMap<u64, Arc<CachedFile>>`, `root_attr: FileAttr`.
       - `lookup`: `name.to_str().ok_or(EINVAL)` →
         `validate_issue_filename(s).map_err(|_| EINVAL)` → registry lookup
         or fetch+intern → reply entry with attrs built from `CachedFile`.
       - `readdir`: `self.rt.block_on(fetch_issues(...))`; on any
         `FetchError::Timeout|Http|Origin` → `reply.error(libc::EIO)`. Refresh
         registry. Emit entries per research §3.5.
       - `read`: resolve ino → `IssueId` via `registry.lookup_ino`. If cache
         miss, block_on `fetch_issue`, render, insert. Slice and reply.
       - `getattr`: ino==1 → `root_attr` clone; else cache lookup; else
         ENOENT. Do NOT fetch from `getattr` — keep it fast (research §6.7).
       - TTL: `Duration::from_secs(1)` for both replies.
    2. Update `src/lib.rs`: `mod fs; pub use fs::ReposixFs;` + wire `Mount::open`
       to `spawn_mount2` with options `[FSName("reposix"), Subtype("reposix"),
       AutoUnmount, DefaultPermissions]` — explicitly NO `AllowOther`.
    3. Rewrite `src/main.rs` with clap-derive:
       ```
       #[derive(Parser)] struct Args {
         mount_point: PathBuf,
         #[arg(long)] backend: String,
         #[arg(long, default_value = "demo")] project: String,
       }
       ```
       `main` installs `tracing_subscriber`, constructs `ReposixFs`, calls
       `fuser::mount2(fs, mount_point, &opts)` (blocking — no `spawn`), and
       relies on `MountOption::AutoUnmount` to clean up on SIGINT.
    4. Create `tests/readdir.rs` per `<behavior>`. Must use wiremock
       (already added in task 1 dev-deps) and `tempfile::tempdir()`. Gate the
       module with `#![cfg(target_os = "linux")]` at file top. Use
       `Mount::open`'s `BackgroundSession` (drop-to-unmount). Wait ≤3s for
       mount readiness by polling `std::fs::read_dir` until it returns 3
       entries (sleep 50ms between polls, timeout 3s → fail test).
    5. Pedantic allowances (add `#[allow(clippy::foo)]` with rationale) only
       if a specific lint fires — do not blanket-allow.
    6. `cargo fmt`, `cargo clippy -p reposix-fuse --all-targets -- -D warnings`,
       `cargo test -p reposix-fuse`.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && cargo test -p reposix-fuse --test readdir -- --nocapture</automated>
    <manual>
      `cargo clippy -p reposix-fuse --all-targets -- -D warnings` clean;
      `grep -c 'validate_issue_filename' crates/reposix-fuse/src/fs.rs` ≥ 1;
      `grep -c 'AllowOther' crates/reposix-fuse/src/` equals 0.
    </manual>
  </verify>
  <done>
    FUSE daemon mounts, lists issues as `0001.md..000N.md`, serves frontmatter
    on read, rejects non-`<digits>.md` names with EINVAL. Integration test
    (wiremock-backed) passes on Linux. Commit subject:
    `feat(03-01): read-only FUSE Filesystem impl + mount binary + readdir test`.
  </done>
</task>

<task type="auto">
  <name>Task 3: Backend-death-does-not-hang integration test (SG-07)</name>
  <files>
    crates/reposix-fuse/tests/sim_death_no_hang.rs
  </files>
  <action>
    Create `crates/reposix-fuse/tests/sim_death_no_hang.rs` that proves ROADMAP
    Phase 3 SC #4: killing the backend makes `stat` return in <7s, not hang
    forever. Structure:

    ```
    #![cfg(target_os = "linux")]

    #[test]
    #[ignore] // run only under `cargo test --release -- --ignored`
    fn stat_returns_within_7s_after_backend_dies() {
        // 1. Start a wiremock server on a random port, stub GET /projects/demo/issues
        //    to return [issue1..3]. Start the mount via Mount::open. Wait for
        //    `read_dir(mount)` to return 3 entries (≤3s).
        // 2. Drop/shutdown the wiremock server so the backend is now dead.
        // 3. Clear the in-memory cache by issuing a readdir refresh. One way:
        //    just call `std::fs::read_dir(&mount).collect::<Vec<_>>()` — the
        //    read will try to refresh and timeout after 5s → EIO. That is the
        //    scenario SC #4 tests.
        //    Simpler: stat a *new* filename that isn't cached yet, forcing
        //    lookup → fetch_issue → timeout → EIO. E.g. stat `0099.md`.
        // 4. Spawn `std::process::Command::new("stat").arg(path).arg("-c").arg("%s")`
        //    with a std::time::Instant measurement around `.output()`.
        //    Assert `elapsed < Duration::from_secs(7)` AND
        //    `!output.status.success()` (non-zero exit from EIO → "Input/output error").
        // 5. Drop the Mount; assert the mount point unmounts within 3s
        //    (spin-poll `mountpoint -q` or check `read_dir` returns err).
    }
    ```

    Use `tokio::runtime::Builder::new_current_thread().enable_all().build()` to
    drive wiremock. Use `std::process::Command` not tokio — the `stat` call
    must be measured on a blocking OS thread to mirror how a real shell would
    call it. If `stat` binary is unavailable in CI, fall back to
    `std::fs::metadata(path)` inside a `std::thread::spawn` with an
    `Instant::now()` deadline check.

    No changes needed to production code; this test exercises the existing
    `tokio::time::timeout` from Task 1. If it hangs, Task 1 is buggy.

    `cargo fmt`, then verify: `cargo test -p reposix-fuse --release --test sim_death_no_hang -- --ignored`.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && cargo test -p reposix-fuse --release --test sim_death_no_hang -- --ignored --test-threads=1</automated>
  </verify>
  <done>
    Test passes: `stat` on a mounted file returns in <7s with non-zero exit
    after the wiremock backend is shut down. Mount cleanup within 3s.
    SG-07 is regression-proofed. Commit subject:
    `test(03-01): prove 5s timeout — stat returns <7s after backend dies`.
  </done>
</task>

</tasks>

<verification>
Phase-local exit check (manually runnable):

```
cd /home/reuben/workspace/reposix && \
  cargo fmt --all --check && \
  cargo clippy -p reposix-fuse --all-targets -- -D warnings && \
  cargo test -p reposix-fuse && \
  cargo test -p reposix-fuse --release -- --ignored --test-threads=1 && \
  [ "$(grep -RIn 'reqwest::Client::new\|reqwest::ClientBuilder' crates/reposix-fuse/ --include='*.rs' | wc -l)" = "0" ] && \
  grep -q 'validate_issue_filename' crates/reposix-fuse/src/fs.rs && \
  ! grep -q 'AllowOther' crates/reposix-fuse/src/fs.rs
```
</verification>

<success_criteria>
- ROADMAP Phase 3 SC #1 (ls shows issues — now asserted as `0001.md 0002.md
  0003.md` per decision above) satisfied by `tests/readdir.rs`.
- SC #2 (cat prints `---` + `id: N`) satisfied by `tests/readdir.rs` body
  assertion.
- SC #3 (`grep -r` works) satisfied transitively: once `read` serves bytes
  correctly, `grep -r` on the mount works by stdlib composition — no extra
  code required. Will be exercised end-to-end by plan 03-02's `demo`.
- SC #4 (no kernel hang) satisfied by `tests/sim_death_no_hang.rs`.
- SC #5 partial: `cargo test -p reposix-fuse` green — `-p reposix-cli` part
  lands in plan 03-02.
</success_criteria>

<output>
After completion, create `.planning/phases/03-readonly-fuse-mount-cli/03-01-SUMMARY.md`
capturing: inode strategy as shipped, exact timeout layering, any fuser API
quirks discovered, and the three commit hashes.
</output>
