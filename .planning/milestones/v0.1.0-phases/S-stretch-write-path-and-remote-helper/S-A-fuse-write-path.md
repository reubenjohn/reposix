---
phase: S-stretch-write-path-and-remote-helper
plan: A
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/reposix-fuse/Cargo.toml
  - crates/reposix-fuse/src/lib.rs
  - crates/reposix-fuse/src/fs.rs
  - crates/reposix-fuse/src/fetch.rs
  - crates/reposix-fuse/tests/write.rs
autonomous: true
requirements:
  - FC-03-write
  - FC-04
  - SG-03
  - SG-04
  - SG-07
user_setup: []

# BUDGET: 60 min wall clock. HARD CUT 06:00 PDT.
# Tasks 1 + 2 = MINIMUM VIABLE (write round-trip via PATCH on existing inode).
# Task 3 (create/unlink) is NICE-TO-HAVE; skip if Task 2 finishes after T+45min.
# If S-A overruns by >15 min, S-B is SKIPPED entirely per orchestrator policy.

must_haves:
  truths:
    - "Opening a mounted issue file for write, replacing its bytes, and closing the fd causes a single PATCH /projects/{slug}/issues/{id} with `If-Match: <version>` to fire against the backend within 5s."
    - "On HTTP 409 from the PATCH, the close() syscall returns EIO (the agent learns it must `git pull --rebase`); on 5s timeout, EIO."
    - "PATCH bodies pass through `Tainted::new(...).then(sanitize)` so server-controlled fields (id, version, created_at, updated_at) are stripped before egress."
    - "`create(parent, name, ...)` validates the new name via `validate_issue_filename`, POSTs an empty issue, allocates an inode, and returns it as a fresh fh."
    - "`unlink(parent, name)` succeeds locally without firing DELETE — actual removal is the helper's job on `git push` (per CONTEXT.md write-path decision)."
    - "Mount config respects `read_only`: when `read_only=true`, MountOption::RO is set and write callbacks return EROFS; when false, write path is live."
  artifacts:
    - path: "crates/reposix-fuse/src/fetch.rs"
      provides: "New `patch_issue(http, origin, project, id, version, sanitized_body, agent)` and `post_issue(http, origin, project, body_json, agent)` async helpers; both wrap I/O in `tokio::time::timeout(FETCH_TIMEOUT)` and return a typed `FetchError`. PATCH body is the JSON-serialized inner of `Untainted<Issue>`. PATCH attaches `If-Match: <version>` header."
    - path: "crates/reposix-fuse/src/fs.rs"
      provides: "Adds `setattr` (size-only no-op for now — the real truncate happens in `write` since we replace bytes wholesale on flush), `write` (per-fh DashMap<u64, Vec<u8>> buffer keyed by fh; appends/overwrites at offset), `flush` (no-op, returns ok), `release` (drains buffer, parses YAML, sanitizes, PATCHes; on success updates cache; on failure returns EIO via reply.error). `create` (validates name, POSTs, interns inode, allocates fh, seeds buffer). `unlink` (validates name, evicts cache + registry but does NOT call DELETE)."
      contains: "fn write\\b|fn release\\b|fn create\\b|fn unlink\\b|fn setattr\\b"
    - path: "crates/reposix-fuse/src/lib.rs"
      provides: "MountOption::RO is now conditional: only included when `cfg.read_only` is true. When false, the daemon mounts read-write."
    - path: "crates/reposix-fuse/Cargo.toml"
      provides: "(minor) Adds `serde_yaml` to dev-deps if needed for write tests; runtime already has it via workspace inheritance from Phase 3."
    - path: "crates/reposix-fuse/tests/write.rs"
      provides: "wiremock-backed unit-style tests, `#[ignore]`-gated where they need a live mount. MINIMUM: `release_patches_with_if_match`, `release_409_returns_eio`, `release_timeout_returns_eio`. STRETCH: `create_posts_and_returns_inode`, `unlink_does_not_call_delete`, `sanitize_strips_server_fields_on_egress`."
  key_links:
    - from: "crates/reposix-fuse/src/fs.rs :: release"
      to: "crates/reposix-fuse/src/fetch.rs :: patch_issue"
      via: "self.rt.block_on(patch_issue(..., If-Match: cached.issue.version, sanitize(Tainted::new(parsed_issue), ServerMetadata{...}).into_inner()))"
      pattern: "patch_issue|If-Match"
    - from: "crates/reposix-fuse/src/fs.rs :: write"
      to: "crates/reposix-fuse/src/fs.rs :: write_buffers (DashMap<u64, Vec<u8>>)"
      via: "DashMap entry-or-insert keyed by fh; offset+data slice copied into buffer (resize if needed)"
      pattern: "write_buffers|DashMap<u64, Vec<u8>>"
    - from: "crates/reposix-fuse/src/fs.rs :: create + write + release"
      to: "reposix_core::path::validate_issue_filename + reposix_core::sanitize"
      via: "validate_issue_filename on every name; sanitize before every PATCH/POST egress"
      pattern: "validate_issue_filename|sanitize\\("
    - from: "crates/reposix-fuse/src/lib.rs :: Mount::open"
      to: "fuser::MountOption::RO"
      via: "conditional inclusion based on cfg.read_only"
      pattern: "if cfg.read_only|MountOption::RO"
---

<objective>
Extend the read-only FUSE daemon (Phase 3) into a writable mount: agents can
`echo > /mnt/0001.md`, `sed -i`, `touch new.md`, `rm` — and the changes
round-trip through the simulator via PATCH/POST as appropriate.

Purpose: closes ROADMAP Phase S success criteria #1 (write round-trip
through sim) and unlocks Plan S-B (the helper depends on the FUSE daemon
mutating issues so the demo's `sed && git commit && git push` flow has
something to push).

Output: `crates/reposix-fuse` builds with five new Filesystem callbacks
implemented; `cargo test -p reposix-fuse` is green; the existing read-only
tests still pass; the `release` callback is the single point that makes the
PATCH HTTP call on close().

BUDGET: 60 minutes wall clock. Minimum viable = Tasks 1 + 2 (PATCH path
only). Task 3 (create/unlink) is the stretch. If task 2 finishes after
T+45min, COMMIT what you have and STOP — do not start task 3.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/S-stretch-write-path-and-remote-helper/S-CONTEXT.md
@.planning/research/fuse-rust-patterns.md
@CLAUDE.md
@crates/reposix-core/src/http.rs
@crates/reposix-core/src/taint.rs
@crates/reposix-core/src/path.rs
@crates/reposix-core/src/issue.rs
@crates/reposix-fuse/src/fs.rs
@crates/reposix-fuse/src/fetch.rs
@crates/reposix-fuse/src/lib.rs

<interfaces>
<!-- Contracts the executor consumes directly — no codebase spelunking needed. -->

From `reposix_core::http` (sealed newtype, SG-01):
```rust
pub struct HttpClient { /* private inner */ }
impl HttpClient {
    pub async fn request_with_headers_and_body<U, B>(
        &self, method: Method, url: U,
        headers: &[(&str, &str)],
        body: Option<B>,
    ) -> Result<reqwest::Response>
    where U: IntoUrl, B: Into<reqwest::Body>;
}
// Allowlist re-checked on every call. Use Method::PATCH and Method::POST.
```

From `reposix_core::taint` (SG-03):
```rust
pub struct Tainted<T>(/* private */); impl<T> Tainted<T> {
    pub fn new(value: T) -> Self;
    pub fn into_inner(self) -> T;
}
pub struct Untainted<T>(/* private */); impl<T> Untainted<T> {
    pub fn into_inner(self) -> T;
}
pub struct ServerMetadata { pub id: IssueId, pub created_at: DateTime<Utc>,
                            pub updated_at: DateTime<Utc>, pub version: u64 }
pub fn sanitize(tainted: Tainted<Issue>, server: ServerMetadata) -> Untainted<Issue>;
// MUST call this before PATCH/POST egress. id/version/created_at/updated_at
// from `tainted` are discarded; agent-controlled fields preserved.
```

From `reposix_core::path` (SG-04):
```rust
pub fn validate_issue_filename(name: &str) -> Result<IssueId>;
// MUST be called on every name argument (create, unlink) before any I/O.
// Rejects "../etc/passwd.md", "my bug.md", "thing.txt", etc.
```

From `reposix_core::issue::frontmatter`:
```rust
pub fn render(issue: &Issue) -> Result<String>;
pub fn parse(text: &str) -> Result<Issue>;
// `parse` reverses `render`; round-trips byte-for-byte via the deterministic
// renderer. Plan S-B reuses these same two functions — DO NOT introduce a
// second renderer/parser anywhere.
```

From `crates/reposix-fuse/src/fetch.rs` (existing):
```rust
pub const FETCH_TIMEOUT: Duration = Duration::from_secs(5);
pub enum FetchError { Timeout, NotFound, Status(StatusCode), Transport(Error),
                      Origin(String), Parse(serde_json::Error), Core(String),
                      // ADD: Conflict { current: u64 }, // for 409 from PATCH
                      // ADD: BadRequest(String),        // for 4xx from POST
                    }
pub async fn fetch_issues(...) -> Result<Vec<Issue>, FetchError>;
pub async fn fetch_issue(...) -> Result<Issue, FetchError>;
```

From `crates/reposix-fuse/src/fs.rs` (existing read-only):
```rust
pub struct ReposixFs { rt, http, origin, project, agent, registry, cache, root_attr }
// Methods to extend Filesystem impl: setattr, write, flush, release, create, unlink.
// `&self` only — all mutable state behind DashMap / parking_lot::RwLock / atomics.
// Add: write_buffers: DashMap<u64, Vec<u8>>  (fh -> pending bytes)
// Add: next_fh: AtomicU64                    (start at 1; 0 is sentinel)
```

From the simulator (target API contract — already shipped in Phase 2):
```
PATCH  /projects/{slug}/issues/{id}
  Headers: If-Match: <version>   (bare digit OK; quoted "v" also OK)
           Content-Type: application/json
  Body: {"title": "...", "status": "...", "body": "...", ...}  (deny_unknown_fields)
  -> 200 + Issue JSON   (success; version bumped)
  -> 409 + {"error":"version_mismatch", "current": N}   (etag mismatch)
  -> 404 if missing
POST   /projects/{slug}/issues
  Body: {"title": "...", "body": "", "status": "open", ...}
  -> 201 + Issue JSON + Location header
DELETE /projects/{slug}/issues/{id}  -- NOT called from FUSE; helper's job.
```
</interfaces>

</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1 [MIN-VIABLE]: Add patch_issue/post_issue helpers + plumb FetchError variants</name>
  <files>crates/reposix-fuse/src/fetch.rs</files>
  <behavior>
    - `patch_issue(http, origin, project, id, version, sanitized: Untainted<Issue>, agent)` async fn:
      - URL: `{origin}/projects/{project}/issues/{id}`
      - Headers: `If-Match: {version}`, `Content-Type: application/json`, `X-Reposix-Agent: {agent}`
      - Body: `serde_json::to_vec(sanitized.into_inner())?`. Strip server-managed fields
        from the JSON serializer side too — the simulator's `PatchIssueBody` has
        `#[serde(deny_unknown_fields)]`, so include only `title`, `status`, `assignee`,
        `labels`, `body` keys (write a small `PatchPayload` struct or `serde_json::Value`
        massage). Reuse `sanitize()` to enforce — this is defense in depth.
      - 5s `tokio::time::timeout` wrapper (same pattern as `fetch_issues`).
      - Return `FetchError::Conflict { current }` on 409; parse `current` from JSON body.
      - Return `FetchError::Timeout` / `FetchError::Transport` / `FetchError::Status` otherwise.
    - `post_issue(http, origin, project, body: Untainted<Issue>, agent)` async fn:
      - URL: `{origin}/projects/{project}/issues`
      - Headers: `Content-Type: application/json`, `X-Reposix-Agent: {agent}`
      - Body: same trimmed shape as PATCH (`title`, `body`, `status`, `assignee`, `labels`).
      - 5s timeout. 201 → parse Issue from body; 4xx → `FetchError::BadRequest(msg)`.
    - Add `Conflict { current: u64 }` and `BadRequest(String)` variants to `FetchError`.
    - Tests (3): `patch_issue_sends_if_match_header` (wiremock matcher on `If-Match: 1`,
      assert call), `patch_issue_409_returns_conflict` (wiremock 409 + body, assert
      `FetchError::Conflict { current: 7 }`), `patch_issue_times_out_within_budget`
      (wiremock with 10s delay, assert returns within 5.5s with timeout-flavored error).
  </behavior>
  <action>
    1. Read `crates/reposix-fuse/src/fetch.rs` to understand current shape and test patterns.
    2. Extend `FetchError` enum with `Conflict { current: u64 }` and `BadRequest(String)`.
    3. Add `patch_issue` and `post_issue` async fns. Reuse the `tokio::time::timeout`
       wrapper pattern from `fetch_issue`. Build the payload via a private
       `#[derive(Serialize)] struct EgressPayload { title, status, assignee, labels, body }`
       constructed from the sanitized `Issue` — this enforces the deny_unknown_fields
       contract on our side.
    4. Use `http.request_with_headers_and_body(Method::PATCH, &url, &headers, Some(body_bytes))`
       — header slice must contain `If-Match`, `Content-Type`, `X-Reposix-Agent`.
    5. Add unit tests as described in `<behavior>`. Use the existing `sample_issue()` helper
       and wiremock patterns from this file's `mod tests`.
    6. `# Errors` doc section on every Result-returning fn.
    7. COMMIT: `feat(reposix-fuse): patch/post issue helpers with If-Match + 5s cap`.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && cargo test -p reposix-fuse --lib fetch 2>&1 | tail -30</automated>
  </verify>
  <done>
    `cargo test -p reposix-fuse --lib fetch::tests::patch_issue_sends_if_match_header
    fetch::tests::patch_issue_409_returns_conflict
    fetch::tests::patch_issue_times_out_within_budget` all pass; clippy clean.
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 2 [MIN-VIABLE]: write/flush/release + per-fh buffer + conditional MountOption::RO</name>
  <files>crates/reposix-fuse/src/fs.rs, crates/reposix-fuse/src/lib.rs</files>
  <behavior>
    - On `open` (default fuser impl is fine — we don't need to override unless we want
      a real fh; for v0.1, override `open` to allocate a fh from `next_fh: AtomicU64`).
      Actually simpler: have `write` use `ino` as the fh key (kernel always supplies
      both). Skip overriding `open` — the FUSE kernel passes a 0 fh by default which
      we treat as "use ino as buffer key". Document the choice.
    - `setattr` — accept any (mode, uid, gid, size, *time) and just `reply.attr` with
      the existing cached attr (no-op other than replying success). The truncate on `>`
      redirect is handled implicitly: the kernel's open(O_TRUNC) → setattr(size=0) →
      write(offset=0, data) sequence drops old bytes in our buffer at write time. For
      the MIN-VIABLE: when `size: Some(0)` and a buffer exists, clear it.
    - `write(ino, fh, offset, data)` — get-or-insert the buffer keyed by `ino` (NOT
      `fh` for the v0.1 simplification), `resize(end, 0)` if too small, `copy_from_slice`
      from `offset`. Return `reply.written(data.len())`.
    - `flush(ino, fh)` — `reply.ok()`. (We push on `release`, not `flush`.)
    - `release(ino, fh, ...)` — drain the buffer for `ino`. If empty → `reply.ok()`.
      Otherwise:
      1. Take the bytes, drop them from the DashMap (atomic remove).
      2. Look up cached `Issue` (current version) in `self.cache.get(&ino)`. If absent,
         this is a new file just `create`d; treat as POST (Task 3 scope) — for MIN-VIABLE,
         this case can `reply.error(EIO)`.
      3. Parse the new bytes as a frontmatter+body with `frontmatter::parse(&str)`. On
         parse error → `reply.error(EIO)`.
      4. Wrap parsed `Issue` in `Tainted::new(_)`. Build `ServerMetadata` from the
         cached issue (id, created_at, version, updated_at). Call `sanitize(tainted,
         server) -> Untainted<Issue>`.
      5. `self.rt.block_on(patch_issue(..., version=cached.issue.version, sanitized, agent))`.
      6. On `Ok(updated_issue)`: render and update the cache entry with new bytes +
         new version. `reply.ok()`.
      7. On `FetchError::Conflict { .. }`: `warn!`, `reply.error(EIO)` per CONTEXT.md
         decision (the helper handles 409 → conflict marker; the FUSE daemon doesn't).
      8. On any other FetchError: `reply.error(EIO)`.
    - In `lib.rs::Mount::open`: change `MountOption::RO` to be conditionally pushed
      only when `cfg.read_only` is true. Add a comment explaining the toggle.
    - Tests in `crates/reposix-fuse/tests/write.rs` (`#[ignore]`-gated where they need
      a real mount):
      - `release_patches_on_buffered_write` — wiremock asserts a single PATCH with
        `If-Match: 1`, body containing `"status":"done"`, returns 200.
      - `release_on_409_returns_eio` — wiremock 409, assert close() syscall returns
        EIO via `nix::errno`. (Or, if real-mount is too heavy in the budget, write a
        unit-style test that calls the `Filesystem::release` method directly with a
        mock fuser reply — see fuser docs for the test harness.)
      - `release_timeout_returns_eio` — wiremock 10s delay, assert EIO within ~5.5s.
  </behavior>
  <action>
    1. Read `crates/reposix-fuse/src/fs.rs` for current struct + Filesystem impl.
    2. Add `write_buffers: DashMap<u64, Vec<u8>>` field to `ReposixFs`. Initialise
       in `new()`. (`fh` is used as the conceptual key but for v0.1 we key by `ino`.)
    3. Implement `setattr` (no-op replying with existing attr; clear buffer on size=0).
    4. Implement `write` (buffer append/overwrite, return written length).
    5. Implement `flush` (just `reply.ok()`).
    6. Implement `release` (the meat — see `<behavior>`).
    7. Map `FetchError::Conflict { .. }` in `fetch_errno()` to `libc::EIO` (not EAGAIN
       — the user must `git pull` to reconcile; EIO is the documented signal in
       CONTEXT.md).
    8. In `lib.rs`, change the `let options = vec![..., MountOption::RO]` block to
       a `let mut options = vec![...]; if cfg.read_only { options.push(MountOption::RO); }`.
    9. Tests in `tests/write.rs`. Use wiremock for the HTTP side. For the FUSE-side
       call, prefer calling `ReposixFs::release` directly with a mock `ReplyEmpty`
       constructed via `fuser::Reply::__test_only(...)` if available; otherwise gate
       behind `#[ignore]` and use `tempfile::TempDir` + real mount.
    10. COMMIT: `feat(reposix-fuse): write/flush/release path with PATCH on close`.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && cargo test -p reposix-fuse 2>&1 | tail -30 && cargo clippy -p reposix-fuse --all-targets -- -D warnings 2>&1 | tail -10</automated>
  </verify>
  <done>
    Non-`#[ignore]` tests pass; clippy clean; `MountOption::RO` is now conditional;
    existing read-only tests still pass; `cargo test -p reposix-fuse --release --
    --ignored release_patches_on_buffered_write` (best-effort) demonstrates a real
    mount + write + close → PATCH round-trip if a wiremock backend is reachable.
    **Decision point:** if elapsed time at end of this task is >T+45min, STOP and
    skip Task 3.
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 3 [STRETCH — skip if budget tight]: create/unlink + sanitize-on-egress proof</name>
  <files>crates/reposix-fuse/src/fs.rs, crates/reposix-fuse/tests/write.rs</files>
  <behavior>
    - `create(parent, name, mode, ...)`:
      1. Reject if `parent != ROOT_INO` → `reply.error(ENOTDIR)`.
      2. `validate_issue_filename(name)` — reject EINVAL on failure.
      3. Build a minimal `Issue` (default status = open, empty title placeholder
         derived from id, empty body — the user will `write` the real content next).
         Wrap in `Tainted::new`, `sanitize` with placeholder ServerMetadata
         (the POST will return the authoritative one anyway).
      4. `self.rt.block_on(post_issue(..., sanitized_payload, agent))`.
      5. On success: intern the returned id into the registry → fresh inode.
         Render and cache. `reply.created(&ENTRY_TTL, &attr, Generation(0), fh, flags)`.
      6. On any error: map via `fetch_errno`.
    - `unlink(parent, name)`:
      1. `parent != ROOT_INO` → `reply.error(ENOTDIR)`.
      2. `validate_issue_filename(name)` → EINVAL on failure.
      3. Look up inode via `registry.lookup_id(id)`. If absent → ENOENT.
      4. `self.cache.remove(&ino)` — evict from rendered cache. DO NOT call DELETE.
         (Per CONTEXT.md: "actual DELETE happens on `git push` so the bulk-delete cap
         can fire".) DO NOT remove from the registry — keeping the inode mapping
         alive avoids re-lookup confusion if the user immediately recreates.
      5. `reply.ok()`.
    - Tests:
      - `create_posts_and_returns_inode` — wiremock asserts POST, returns 201 + Issue.
        Verify `Filesystem::create` reply contains a fresh inode > FIRST_ISSUE_INODE.
      - `unlink_does_not_call_delete` — wiremock with NO matcher for DELETE; call
        unlink; assert wiremock saw zero requests AFTER the initial readdir.
      - `sanitize_strips_server_fields_on_egress` — write a buffer containing
        `version: 999999` in YAML; release; assert the wiremock-captured PATCH body
        does NOT contain `version` field at all (because our `EgressPayload` struct
        doesn't have one) — this is the SG-03 proof at the FUSE boundary.
  </behavior>
  <action>
    1. Implement `create` and `unlink` per `<behavior>`.
    2. Add the three tests to `tests/write.rs`.
    3. COMMIT: `feat(reposix-fuse): create+unlink with sanitize-on-egress proof`.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && cargo test -p reposix-fuse 2>&1 | tail -20</automated>
  </verify>
  <done>
    All three new tests pass; clippy clean; no DELETE call ever fires from the FUSE
    daemon (verified by wiremock mismatch counter).
  </done>
</task>

</tasks>

<verification>
- `cargo fmt --all --check` exits 0.
- `cargo clippy -p reposix-fuse --all-targets -- -D warnings` exits 0.
- `cargo test -p reposix-fuse` exits 0 (non-ignored tests).
- `crates/reposix-fuse/src/fs.rs` contains `fn write\b`, `fn release\b`, and at least
  one `patch_issue` call inside `release`.
- `crates/reposix-fuse/src/lib.rs` shows `MountOption::RO` only inside a
  `cfg.read_only` branch (not unconditional).
- `grep -RIn 'reqwest::Client::new\|reqwest::ClientBuilder' crates/reposix-fuse/`
  returns nothing (SG-01 invariant preserved — all egress through `HttpClient`).
- No new `unsafe` blocks; `#![forbid(unsafe_code)]` still at crate root.
</verification>

<success_criteria>
**Minimum viable (Tasks 1+2 only):** PATCH round-trip from `release` works against
wiremock. `release` returns EIO on 409 and on 5s timeout. `MountOption::RO` is
conditional. `cargo test -p reposix-fuse` is green.

**Full plan (all three tasks):** Above plus `create` POSTs new issues and `unlink`
locally evicts without calling DELETE. Sanitize-on-egress is proven by a test that
writes `version: 999999` in YAML and confirms the egress payload omits `version`.

**Plan exit gate:** the composite below exits 0 (or, if Task 3 was skipped, the
subset excluding the Task 3 lines):
```
cargo fmt --all --check && \
cargo clippy -p reposix-fuse --all-targets -- -D warnings && \
cargo test -p reposix-fuse && \
grep -q 'fn write' crates/reposix-fuse/src/fs.rs && \
grep -q 'fn release' crates/reposix-fuse/src/fs.rs && \
grep -q 'patch_issue' crates/reposix-fuse/src/fs.rs && \
grep -q 'if cfg.read_only' crates/reposix-fuse/src/lib.rs
```
</success_criteria>

<output>
After completion, create
`.planning/phases/S-stretch-write-path-and-remote-helper/S-A-SUMMARY.md`
recording: tasks completed (1, 2, optionally 3), elapsed wall time, any
deviations from `<behavior>`, and the next-action signal for orchestrator to
decide whether to proceed to Plan S-B.
</output>
