---
phase: 03-readonly-fuse-mount-cli
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/reposix-core/src/http.rs
  - crates/reposix-core/tests/http_allowlist.rs
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

# NOTE: this plan extends Phase 1's `reposix-core::http` contract by adding
# `HttpClient::request_with_headers`. The FUSE daemon is the first caller that
# needs to attach `X-Reposix-Agent` on every request so the Phase-2 audit log
# attributes correctly; we grow the API minimally here rather than bouncing
# back to Phase 1. The sealed-newtype invariant (SG-01) is preserved: the new
# method performs the same per-call allowlist recheck as `request`, and
# `inner: reqwest::Client` remains private.

must_haves:
  truths:
    - "Mounting /tmp/reposix-mnt against a reposix-compatible backend presents one regular file per issue at the mount root, named `<zero-padded-id>.md`."
    - "`cat <mount>/0001.md` returns the exact bytes of `reposix_core::issue::frontmatter::render(issue)` for the backend's issue 1."
    - "Every path-bearing FUSE op (lookup, any future path resolution) rejects non-`<digits>.md` names with EINVAL via `validate_issue_filename`."
    - "`readdir` and `read` complete within 5s or return EIO — the kernel NEVER hangs on a dead backend."
    - "`fusermount3 -u <mount>` completes within 3s after the FUSE session exits."
    - "`HttpClient::request_with_headers` enforces the same per-call allowlist recheck as `request`, and `HttpClient::get/post/patch/delete/request` delegate to it with an empty header slice (backward compatible)."
  artifacts:
    - path: "crates/reposix-core/src/http.rs"
      provides: "Added `HttpClient::request_with_headers(method, url, &[(&str,&str)]) -> Result<Response>`; existing wrappers delegate to it with `&[]`."
    - path: "crates/reposix-core/tests/http_allowlist.rs"
      provides: "Two appended tests: `request_with_headers_rechecks_allowlist` and `request_with_headers_attaches_header` (wiremock header matcher)."
    - path: "crates/reposix-fuse/src/fs.rs"
      provides: "ReposixFs struct + Filesystem impl (init, getattr, lookup, readdir, read)"
      contains: "impl Filesystem for ReposixFs"
    - path: "crates/reposix-fuse/src/inode.rs"
      provides: "InodeRegistry (DashMap<IssueId,u64> + reverse map + AtomicU64 counter @ 0x1_0000)"
    - path: "crates/reposix-fuse/src/fetch.rs"
      provides: "fetch_issues + fetch_issue helpers built on `HttpClient::request_with_headers`, each wrapped in 5s timeout and attaching `X-Reposix-Agent: reposix-fuse-{pid}`"
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
      to: "reposix_core::http::{client, HttpClient::request_with_headers}"
      via: "sole allowed HTTP construction + send path; header slice carries X-Reposix-Agent"
      pattern: "request_with_headers"
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

From `reposix_core::http` (POST-H-01 — sealed newtype; `reqwest::Client` is private):
```rust
pub struct ClientOpts { pub total_timeout: Duration, pub user_agent: Option<String> }
impl Default for ClientOpts { /* 5s timeout, reposix/<ver> UA */ }

pub struct HttpClient { /* private inner: reqwest::Client */ }
pub fn client(opts: ClientOpts) -> Result<HttpClient>;   // the ONLY legal ctor

impl HttpClient {
    pub async fn request<U: IntoUrl>(&self, method: Method, url: U)
        -> Result<reqwest::Response>;                    // re-checks allowlist
    pub async fn get<U: IntoUrl>(&self, url: U) -> Result<reqwest::Response>;
    pub async fn post<U: IntoUrl>(&self, url: U) -> Result<reqwest::Response>;
    pub async fn patch<U: IntoUrl>(&self, url: U) -> Result<reqwest::Response>;
    pub async fn delete<U: IntoUrl>(&self, url: U) -> Result<reqwest::Response>;

    // ADDED BY THIS PLAN (Task 1 step 0):
    pub async fn request_with_headers<U: IntoUrl>(
        &self, method: Method, url: U, headers: &[(&str, &str)],
    ) -> Result<reqwest::Response>;                      // same allowlist recheck
}
```
There is NO public accessor for the inner `reqwest::Client`. Callers that need
custom headers MUST use `request_with_headers`; the existing `request`/`get`/...
wrappers keep their current signatures and delegate to it with `&[]`.

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
- FUSE requests MUST send header `X-Reposix-Agent: reposix-fuse-{pid}` so the audit log attributes. This is now done by `request_with_headers`, introduced in Task 1.
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
  ROADMAP Phase 3 SC #1 now uses `0001.md`/`/issues/1` numeric IDs (ROADMAP has
  been updated separately). Phase 1's `validate_issue_filename` only accepts
  `<digits>.md`. The Phase-2 seed uses numeric IDs (`IssueId(1)`…`IssueId(3)`).
  Filenames at the FUSE mount are `0001.md 0002.md 0003.md`.
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
- **HttpClient is shared.** `ReposixFs` holds `Arc<HttpClient>` (not
  `Arc<reqwest::Client>` — the sealed newtype is the only legal handle). Fetch
  helpers in `fetch.rs` take `&HttpClient` or `Arc<HttpClient>`.
- **Header attachment via `request_with_headers`.** Both `fetch_issues` and
  `fetch_issue` call `http.request_with_headers(Method::GET, url,
  &[("X-Reposix-Agent", &agent)])`, where `agent = format!("reposix-fuse-{}",
  std::process::id())` is computed once at `ReposixFs::new` and carried as a
  string field on the fs (so we don't `format!` per call).
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
| T-03-03 | Information Disclosure | egress to non-allowlisted origin | mitigate | All HTTP goes through `HttpClient::request_with_headers` (sealed newtype) which re-checks `REPOSIX_ALLOWED_ORIGINS` per call. The raw `reqwest::Client` is physically unreachable: `inner` is private, no `Deref`/`AsRef`/accessor exists (compile-fail fixture locks this). |
| T-03-04 | Elevation of Privilege | other-user access to mount | mitigate | `MountOption::AllowOther` OFF; rely on `DefaultPermissions` + `0o444` mode bits. |
| T-03-05 | Repudiation | backend receives anonymous FUSE traffic | mitigate | Every outbound request carries `X-Reposix-Agent: reposix-fuse-{pid}` via `request_with_headers`; the Phase-2 audit middleware captures the header. |
| T-03-06 | Spoofing | attacker-controlled issue body rendered into FS content | accept | Bodies are bytes-in-bytes-out; no template expansion, no shell interpolation. `frontmatter::render` already escapes YAML (tested in Phase 1). |
| T-03-07 | Tampering | integer overflow in `offset+size` for `read` | mitigate | Use `usize` saturating arithmetic (`offset.min(len)`, `(offset+size).min(len)`) rather than wrapping adds. |
</threat_model>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Extend HttpClient with request_with_headers + inode registry + fetch helpers (the foundation)</name>
  <files>
    crates/reposix-core/src/http.rs,
    crates/reposix-core/tests/http_allowlist.rs,
    crates/reposix-fuse/Cargo.toml,
    crates/reposix-fuse/src/inode.rs,
    crates/reposix-fuse/src/fetch.rs,
    crates/reposix-fuse/src/lib.rs
  </files>
  <behavior>
    <!-- http.rs additive extension -->
    - `HttpClient::request_with_headers(method, url, headers)` parses `url`,
      runs the same allowlist recheck as `request`, then builds the reqwest
      `RequestBuilder`, iterates `headers` invoking `.header(k, v)` on each
      tuple, and sends. Signature:
      `pub async fn request_with_headers<U: IntoUrl>(&self, method: Method,
      url: U, headers: &[(&str, &str)]) -> Result<reqwest::Response>`.
    - The existing `HttpClient::request` now delegates:
      `self.request_with_headers(method, url, &[]).await`. The existing
      `get/post/patch/delete` wrappers continue to call `self.request(...)`
      unchanged (transitively going through `request_with_headers`).
    - A request whose URL's origin is NOT allowlisted is rejected with
      `Error::InvalidOrigin(_)` BEFORE any headers are attached and before
      any I/O.
    - A request whose URL IS allowlisted and whose headers include
      `("X-Reposix-Agent", "reposix-fuse-123")` causes the outgoing request to
      carry that exact header pair, verifiable by wiremock's
      `header("X-Reposix-Agent", "reposix-fuse-123")` matcher.

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
    - `fetch_issues(&http, origin, project, agent)` hits
      `{origin}/projects/{project}/issues`, 5s timeout, returns `Vec<Issue>`.
      Wiremock mock returning `[{...},{...},{...}]` → 3 issues parsed.
    - `fetch_issue(&http, origin, project, IssueId(1), agent)` hits
      `{origin}/projects/{project}/issues/1`, 5s timeout, returns `Issue`.
    - `fetch_issue` against a wiremock that sleeps 10s returns
      `Err(FetchError::Timeout)` within 5.5s wall-clock (measured in test).
    - `fetch_issue` against a 404 returns `Err(FetchError::NotFound)`; a 500
      returns `Err(FetchError::Http(_))`; a non-allowlisted origin returns
      `Err(FetchError::Origin(_))` (bubbled from `HttpClient::request_with_headers`).
    - Both helpers attach header `X-Reposix-Agent: <agent>` via the
      `request_with_headers` header slice. Verified via wiremock matcher.
  </behavior>
  <action>
    0. **Extend `reposix-core::http` first** (this MUST land before `fetch.rs`
       is written — it is the new contract the FUSE daemon consumes):
       - In `crates/reposix-core/src/http.rs`, add
         `HttpClient::request_with_headers` with the signature above. Its body
         mirrors the existing `request` up through the `allowlist.iter().any(...)`
         gate, then does:
         ```rust
         let mut builder = self.inner.request(method, parsed);
         for (k, v) in headers {
             builder = builder.header(*k, *v);
         }
         let resp = builder.send().await?;
         Ok(resp)
         ```
       - Change the existing `request` to delegate:
         `self.request_with_headers(method, url, &[]).await`. The `get/post/patch/delete`
         wrappers already go through `request`, so no edits there.
       - Preserve the doc comments on `request` (redirect-recheck hook,
         `InvalidOrigin` / `Other` / `Http` error contract) and copy an
         abbreviated version onto `request_with_headers` with one extra line:
         "Headers are attached in order; duplicates are allowed and preserved
         (reqwest does not dedupe)."
       - In `crates/reposix-core/tests/http_allowlist.rs`, APPEND two
         `#[tokio::test]` functions:
         - `request_with_headers_rechecks_allowlist`: with env unset, call
           `c.request_with_headers(Method::GET, "https://evil.example/",
           &[("X-Reposix-Agent", "reposix-fuse-1")]).await` and assert
           `Err(Error::InvalidOrigin(_))` AND elapsed < 500ms (same pattern as
           existing `egress_to_non_allowlisted_host_is_rejected`).
         - `request_with_headers_attaches_header`: spin up a `MockServer`,
           `Mock::given(any())
              .and(wiremock::matchers::header("X-Reposix-Agent", "reposix-fuse-123"))
              .respond_with(ResponseTemplate::new(200))
              .mount(&server).await;`
           then call `c.request_with_headers(Method::GET, &server.uri(),
           &[("X-Reposix-Agent", "reposix-fuse-123")]).await.expect(...)`; assert
           status 200. (If the header is missing, wiremock returns its default
           404 and the assertion fails.)
       - Run `cargo test -p reposix-core --test http_allowlist` — all existing
         tests MUST still pass, plus the two new ones.

    1. `Cargo.toml` updates (`crates/reposix-fuse/Cargo.toml`):
       - Ensure `dashmap`, `parking_lot`, `tokio`, `reqwest`, `libc`, `fuser`,
         `serde_json`, `chrono` are in `[dependencies]` (all are workspace deps
         already — just list them if not present).
       - Add `[dev-dependencies]`: `wiremock = "0.6"`, `tempfile = "3"`,
         `tokio = { workspace = true, features = ["macros","rt-multi-thread","time"] }`.

    2. `src/inode.rs` — `InodeRegistry` struct per `<behavior>`, using
       `DashMap<IssueId, u64>` + `DashMap<u64, IssueId>` + `AtomicU64`.
       `intern`, `lookup_ino`, `refresh` public-in-crate. Unit tests inline.

    3. `src/fetch.rs` — `FetchError` enum (`Timeout`, `NotFound`,
       `Http(reqwest::Error)`, `Origin(reposix_core::error::Error)`,
       `Parse(serde_json::Error)`); `fetch_issues` and `fetch_issue` take
       `&HttpClient` (or `Arc<HttpClient>`), `origin: &str`, `project: &str`,
       `agent: &str`. Both construct the URL, then call:
       ```rust
       let fut = http.request_with_headers(
           Method::GET, &url,
           &[("X-Reposix-Agent", agent)],
       );
       let resp = tokio::time::timeout(Duration::from_secs(5), fut)
           .await
           .map_err(|_| FetchError::Timeout)?
           .map_err(|e| match e {
               reposix_core::Error::InvalidOrigin(_) => FetchError::Origin(e),
               _ => FetchError::from_core(e),
           })?;
       ```
       Map status → `FetchError::NotFound` on 404, `FetchError::Http(_)` on
       other 4xx/5xx. Parse JSON into `Vec<Issue>` / `Issue`. NO direct
       `reqwest::Client::new()` anywhere — the sealed `HttpClient` newtype
       from step 0 is the only path.

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

    6. `cargo fmt`, `cargo clippy -p reposix-core -p reposix-fuse --all-targets -- -D warnings`.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && cargo test -p reposix-core --test http_allowlist request_with_headers -- --nocapture && cargo test -p reposix-fuse --lib inode::tests fetch::tests -- --nocapture</automated>
    <manual>
      `cargo clippy -p reposix-core -p reposix-fuse --all-targets -- -D warnings` clean;
      `grep -RIn 'reqwest::Client::new\|reqwest::ClientBuilder' crates/reposix-fuse/ --include='*.rs' | wc -l` equals 0.
    </manual>
  </verify>
  <done>
    `HttpClient::request_with_headers` shipped with allowlist recheck + two
    passing tests in `http_allowlist.rs`. Inode registry + HTTP fetch helpers
    compile, unit-tested, and lint-clean. All outbound HTTP flows through the
    sealed `HttpClient` newtype. No unsafe; no direct reqwest ctor. Commit
    subjects (two commits):
    `feat(03-01): HttpClient::request_with_headers (allowlisted + header slice)`,
    then
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
      builds its own Tokio multi-thread runtime (2 workers), builds the HTTP
      client via `reposix_core::http::client(ClientOpts::default())` (returns
      `HttpClient`), wraps it in `Arc<HttpClient>`, computes
      `agent = format!("reposix-fuse-{}", std::process::id())` and stores it,
      seeds root inode (1) with cached attrs computed from `libc::getuid/getgid`
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
       - Store `Arc<tokio::runtime::Runtime>`, `Arc<reposix_core::http::HttpClient>`
         (NOT `Arc<reqwest::Client>` — the inner client is sealed),
         `origin: String`, `project: String`, `agent: String`
         (`format!("reposix-fuse-{}", std::process::id())`),
         `registry: InodeRegistry`,
         `cache: DashMap<u64, Arc<CachedFile>>`, `root_attr: FileAttr`.
       - `lookup`: `name.to_str().ok_or(EINVAL)` →
         `validate_issue_filename(s).map_err(|_| EINVAL)` → registry lookup
         or fetch+intern → reply entry with attrs built from `CachedFile`.
         Fetch helpers receive `&self.http` and `&self.agent`.
       - `readdir`: `self.rt.block_on(fetch_issues(&self.http, &self.origin,
         &self.project, &self.agent))`; on any
         `FetchError::Timeout|Http|Origin` → `reply.error(libc::EIO)`. Refresh
         registry. Emit entries per research §3.5.
       - `read`: resolve ino → `IssueId` via `registry.lookup_ino`. If cache
         miss, block_on `fetch_issue(&self.http, ..., &self.agent)`, render,
         insert. Slice and reply.
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
      `[ "$(grep -RIn 'AllowOther' crates/reposix-fuse/ --include='*.rs' | wc -l)" = "0" ]`.
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

    use std::time::{Duration, Instant};

    #[test]
    #[ignore] // run only under `cargo test --release -- --ignored`
    fn stat_returns_within_7s_after_backend_dies() {
        // 1. Start a wiremock server on a random port, stub GET /projects/demo/issues
        //    to return [IssueId(1), IssueId(2), IssueId(3)]. Start the mount via
        //    Mount::open. Wait for `read_dir(mount)` to return 3 entries (≤3s).
        //    This step pre-caches `0001.md` in the inode registry AND the
        //    in-memory CachedFile map.
        // 2. Drop/shutdown the wiremock server so the backend is now dead.
        // 3. Target `<mount>/0001.md` — the file is already in the registry, so
        //    the kernel will route `stat(2)` to `getattr`, which hits the cache
        //    and returns fast... UNLESS the TTL has expired, in which case
        //    `lookup` fires → fetch_issue → timeout → EIO. Either path must
        //    return in <7s; the timeout path is what we're proving doesn't hang.
        //    To force the timeout path deterministically, drop cache by calling
        //    `read_dir` on the mount first (which re-fetches and times out),
        //    then `stat` the pre-known path.
        //    Simpler: just target `0001.md` directly with a FRESH process (the
        //    shell-invoked `stat` binary); the mount's getattr may hit cache
        //    and return fast, OR lookup/read may fall through to the timeout.
        //    Both are pass conditions per SC #4 ("<7s, non-zero exit if EIO").
        //    The assertion is: elapsed < 7s.
        // 4. Shell out the syscall so the timeout is KERNEL-ENFORCED at wall
        //    clock (not Rust-owned):
        //        std::process::Command::new("timeout")
        //            .arg("7")
        //            .arg("stat")
        //            .arg(cached_file_path)
        //    Measure with std::time::Instant immediately before `.output()`
        //    and immediately after. Assert:
        //        !output.status.success()   // EIO surfaces as non-zero OR
        //                                   // `timeout` itself killed `stat`
        //                                   // (also non-zero). Either proves
        //                                   // we did NOT hang silently.
        //        elapsed < Duration::from_secs(7)
        // 5. Drop the Mount; assert the mount point unmounts within 3s
        //    (spin-poll `std::fs::read_dir` — once the FS is unmounted, the
        //    path becomes an ordinary empty tempdir).
    }
    ```

    Implementation notes:

    - Use `tokio::runtime::Builder::new_current_thread().enable_all().build()`
      to drive wiremock setup/teardown.
    - **Use `std::process::Command::new("timeout").arg("7").arg("stat").arg(...)`
      — NOT `std::fs::metadata` in a thread**. The stdlib fallback cannot
      enforce a wall-clock timeout (a blocking syscall inside a thread can sit
      forever regardless of what the main thread does); `timeout(1)` is
      kernel-enforced and matches how a real shell user would test this. The
      `coreutils` `timeout` binary is present on every Linux CI image we use;
      if missing, the test should fail with a clear skip message rather than
      fall back to an unreliable Rust timer.
    - **Target `0001.md` (pre-cached via the initial `read_dir`), NOT `0099.md`.**
      The SC-#4 scenario is: the daemon was serving a live backend, the
      backend died, and the kernel now asks about a file it has already seen.
      A never-seen file hitting `lookup` cold is a weaker scenario (it only
      proves lookup times out, not that cached entries eventually flush through
      to EIO). `0001.md` exercises the realistic path.
    - Assertions: `assert!(!output.status.success(), "expected non-zero exit,
      got {:?}", output.status);` AND
      `assert!(elapsed < Duration::from_secs(7), "stat took {elapsed:?}");`.

    No changes needed to production code; this test exercises the existing
    `tokio::time::timeout` from Task 1. If it hangs, Task 1 is buggy.

    `cargo fmt`, then verify: `cargo test -p reposix-fuse --release --test sim_death_no_hang -- --ignored`.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && cargo test -p reposix-fuse --release --test sim_death_no_hang -- --ignored --test-threads=1</automated>
  </verify>
  <done>
    Test passes: `timeout 7 stat <mount>/0001.md` returns in <7s with non-zero
    exit after the wiremock backend is shut down. Mount cleanup within 3s.
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
  cargo clippy -p reposix-core -p reposix-fuse --all-targets -- -D warnings && \
  cargo test -p reposix-core -p reposix-fuse && \
  cargo test -p reposix-fuse --release -- --ignored --test-threads=1 && \
  [ "$(grep -RIn 'reqwest::Client::new\|reqwest::ClientBuilder' crates/reposix-fuse/ --include='*.rs' | wc -l)" = "0" ] && \
  grep -q 'validate_issue_filename' crates/reposix-fuse/src/fs.rs && \
  [ "$(grep -RIn 'AllowOther' crates/reposix-fuse/ --include='*.rs' | wc -l)" = "0" ]
```
</verification>

<success_criteria>
- ROADMAP Phase 3 SC #1 (ls shows `0001.md 0002.md 0003.md`) satisfied by
  `tests/readdir.rs`.
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
capturing: inode strategy as shipped, exact timeout layering, the
`request_with_headers` API shape as it actually went in (signature, doc text),
any fuser API quirks discovered, and the commit hashes (two or three).
</output>
</content>
</invoke>