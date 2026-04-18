---
phase: 20-op-3-reposix-refresh-subcommand-and-git-diff-cache-for-mount
plan_id: 20-B
wave: 2
goal: >
  Integration-test the full `reposix refresh` flow against the in-process
  simulator: single refresh writes committed .md files; second refresh with no
  backend changes produces an empty git diff; fetched_at.txt timestamp is
  current; FUSE-active guard fires. Polish error messages and edge cases.
depends_on:
  - 20-A
type: execute
autonomous: true
requirements:
  - REFRESH-01
  - REFRESH-02
  - REFRESH-03
  - REFRESH-04

must_haves:
  truths:
    - "Integration test: spin up simulator, call run_refresh, assert committed
      .md files match list_issues output"
    - "`git diff HEAD~1` in the mount dir is empty after a second refresh with
      no backend changes (idempotency test)"
    - "`fetched_at.txt` contains a valid ISO-8601 UTC timestamp within 10s of
      test wall-clock time"
    - "FUSE-active guard integration test: create a live fuse.pid, call
      run_refresh, assert error message contains 'FUSE mount is active'"
    - "`cargo test --workspace --quiet` is fully green"
  artifacts:
    - path: "crates/reposix-cli/tests/refresh_integration.rs"
      provides: "Integration tests: refresh_writes_md_files, refresh_is_idempotent,
        refresh_fuse_active_guard, fetched_at_timestamp_is_current"
      exports: []
  key_links:
    - from: "crates/reposix-cli/tests/refresh_integration.rs"
      to: "crates/reposix-sim (spawned via reposix-cli binary or in-process)"
      via: "SimBackend::new(origin).list_issues() in run_refresh"
      pattern: "run_refresh"
---

<objective>
Write integration tests for the `reposix refresh` subcommand using the
in-process simulator. Verify the end-to-end flow: fetch → render → write →
git commit → idempotency (second run, no diff). Polish error messages and
ensure the full workspace test suite is green.

Purpose: Prove the "mount-as-time-machine" property at the integration level —
not just unit tests for individual functions, but the observable git history
that agents will rely on.

Output:
- `crates/reposix-cli/tests/refresh_integration.rs` — 4 integration tests
- Edge case handling polish in refresh.rs (git config in bare env, missing
  gitignore entries, first-run with no .git dir)
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/20-op-3-reposix-refresh-subcommand-and-git-diff-cache-for-mount/20-RESEARCH.md
@.planning/phases/20-op-3-reposix-refresh-subcommand-and-git-diff-cache-for-mount/20-A-SUMMARY.md

<interfaces>
<!-- Contracts the executor needs from Wave A output. -->

From crates/reposix-cli/src/refresh.rs (Wave A):
```rust
pub struct RefreshConfig {
    pub mount_point: std::path::PathBuf,
    pub origin: String,
    pub project: String,
    pub backend: crate::list::ListBackend,
    pub offline: bool,
}
pub async fn run_refresh(cfg: RefreshConfig) -> anyhow::Result<()>;
```

From crates/reposix-cli/src/cache_db.rs (Wave A):
```rust
pub struct CacheDb(rusqlite::Connection);
pub fn open_cache_db(mount: &std::path::Path) -> anyhow::Result<CacheDb>;
pub fn update_metadata(db: &CacheDb, backend_name: &str, project: &str,
    last_fetched_at: &str, commit_sha: Option<&str>) -> anyhow::Result<()>;
```

From reposix-core SimBackend (used to spin up test backend):
```rust
pub struct SimBackend { … }
impl SimBackend {
    pub fn new(origin: impl Into<String>) -> anyhow::Result<Self>;
}
impl IssueBackend for SimBackend {
    async fn list_issues(&self, project: &str) -> Result<Vec<Issue>>;
}
```

From crates/reposix-sim (in-process or child process):
// Tests can spawn `reposix sim --ephemeral --bind 127.0.0.1:<port>` as a child
// process using assert_cmd or std::process::Command, then call run_refresh
// pointing at that port. Use a random port (try 0 binding or pick from a test
// range) to avoid collisions with parallel test runs.
// Alternatively: use the reposix_sim crate directly in-process if it exposes
// a `start_ephemeral(bind_addr)` or similar function.
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Integration tests — full refresh flow with simulator</name>
  <files>crates/reposix-cli/tests/refresh_integration.rs</files>
  <behavior>
    - Test `refresh_writes_md_files`: spin up sim (ephemeral), call run_refresh
      on a tempdir, assert that `<tempdir>/issues/` contains at least 1 .md
      file; assert `git log --oneline` in tempdir shows exactly 1 commit; assert
      commit author contains "reposix".
    - Test `refresh_idempotent_no_diff`: after `refresh_writes_md_files` baseline,
      call run_refresh again with no backend changes; `git diff HEAD~1 --stat`
      output is empty (no changed files other than fetched_at.txt); assert
      `git log --oneline` shows exactly 2 commits (one per run).
    - Test `refresh_fuse_active_guard`: write a fuse.pid file containing the
      current process PID (which is definitely alive); call run_refresh; assert
      error message contains "FUSE mount is active".
    - Test `fetched_at_is_current_timestamp`: after a refresh, read
      `<mount>/.reposix/fetched_at.txt`; parse as `chrono::DateTime<Utc>`;
      assert it is within 30 seconds of `chrono::Utc::now()`.
  </behavior>
  <action>
Create `crates/reposix-cli/tests/refresh_integration.rs`.

**Simulator setup strategy:**

Option A (preferred if reposix-sim exposes an in-process start function):
Check if `reposix_sim` exposes a `spawn_ephemeral` or `start_server` async fn
that returns a bound address. If yes, use it directly in `#[tokio::test]`.

Option B (fallback): Spawn `reposix sim --ephemeral --bind 127.0.0.1:0` as a
child process via `assert_cmd::Command` or `std::process::Command`. Parse the
bound port from stdout. This requires the sim to print its bind address on
startup — check if it already does so by reading `crates/reposix-sim/src/main.rs`
or the sim startup code. If it does not print the port, use a fixed port
(e.g., 17878 + test index) with a small retry loop.

Option C (simplest, no network needed for basic file-write tests): Inline a
minimal fake backend. Create a `struct FakeBackend { issues: Vec<Issue> }`
that implements `IssueBackend`. Call `run_refresh` with a helper that
overrides the backend dispatch (refactor `refresh.rs` to accept a
`Box<dyn IssueBackend>` parameter, or extract the inner logic into a testable
`run_refresh_with_backend(cfg, backend)` function). This is the cleanest
approach because it avoids port management entirely.

**Recommended approach:** Refactor `refresh.rs` in this wave to extract
`pub(crate) async fn run_refresh_inner(cfg: &RefreshConfig,
issues: Vec<reposix_core::Issue>) -> anyhow::Result<String>` (returns commit
SHA or summary string). The `run_refresh` public fn builds the backend and
calls this inner fn. Tests call `run_refresh_inner` with a fixed Vec<Issue>
directly — no simulator process needed for the file-write and git tests.

The `refresh_fuse_active_guard` test calls `run_refresh` (not the inner fn)
because it tests the guard that runs before backend construction; no network
needed.

**Test structure:**

```rust
#[tokio::test]
async fn refresh_writes_md_files() {
    let dir = tempfile::tempdir().unwrap();
    let issues = vec![make_test_issue(1, "Test issue")];
    // call run_refresh_inner(cfg, issues)
    // assert dir/issues/00000000001.md exists
    // assert `git log` in dir has 1 commit
    // assert commit author contains "reposix"
}

#[tokio::test]
async fn refresh_idempotent_no_diff() {
    let dir = tempfile::tempdir().unwrap();
    let issues = vec![make_test_issue(1, "Stable issue")];
    // first refresh
    run_refresh_inner(&cfg, issues.clone()).await.unwrap();
    // second refresh — same issues
    run_refresh_inner(&cfg, issues).await.unwrap();
    // git diff HEAD~1 --stat must show only fetched_at.txt changed (or be empty
    // if fetched_at is excluded from diff by test granularity)
    // assert git log shows 2 commits
}

#[tokio::test]
async fn refresh_fuse_active_guard() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".reposix")).unwrap();
    let pid = std::process::id(); // current process is alive
    std::fs::write(dir.path().join(".reposix/fuse.pid"),
                   pid.to_string()).unwrap();
    let err = run_refresh(RefreshConfig { … offline: false, … }).await
        .expect_err("must fail when FUSE is active");
    assert!(err.to_string().contains("FUSE mount is active"),
            "unexpected error: {err}");
}

#[tokio::test]
async fn fetched_at_is_current_timestamp() {
    let dir = tempfile::tempdir().unwrap();
    let issues = vec![make_test_issue(2, "Clock test")];
    let before = chrono::Utc::now();
    run_refresh_inner(&cfg, issues).await.unwrap();
    let after = chrono::Utc::now();
    let txt = std::fs::read_to_string(
        dir.path().join(".reposix/fetched_at.txt")).unwrap();
    let ts = txt.trim().parse::<chrono::DateTime<chrono::Utc>>().unwrap();
    assert!(ts >= before && ts <= after,
            "fetched_at {ts} not in [{before}, {after}]");
}

fn make_test_issue(id: u64, title: &str) -> reposix_core::Issue {
    reposix_core::Issue {
        id: reposix_core::IssueId(id),
        title: title.to_owned(),
        status: reposix_core::IssueStatus::Open,
        body: String::new(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        // fill remaining fields with defaults
        …
    }
}
```

**Git environment in tests:** The git subprocess inside `run_refresh_inner`
needs `GIT_AUTHOR_NAME`, `GIT_AUTHOR_EMAIL`, `GIT_COMMITTER_NAME`,
`GIT_COMMITTER_EMAIL` to be set in the Command environment (already done in
Wave A) so the commit does not fail with "Please tell me who you are" in CI.
Verify the Wave A implementation does this. If it was omitted, add it during
this task's polish pass.

**Refactor in refresh.rs (if not already done):**
If `run_refresh` in Wave A was not split into inner/outer, split it now:
```rust
// Public — builds backend from cfg and calls inner
pub async fn run_refresh(cfg: RefreshConfig) -> anyhow::Result<()>

// Testable — receives already-fetched issues
pub(crate) async fn run_refresh_inner(
    cfg: &RefreshConfig,
    issues: Vec<reposix_core::Issue>,
) -> anyhow::Result<()>
```

Make sure `run_refresh` still checks `cfg.offline` and `is_fuse_active` before
calling `run_refresh_inner`.

**Edge cases to handle (polish in refresh.rs):**
1. `<mount>/issues/` directory does not exist yet — `create_dir_all` handles this.
2. `.git` does not exist yet in mount — `git init` is idempotent and handles this.
3. No issues returned from backend — write zero .md files; still commit
   (with `--allow-empty`); `fetched_at.txt` still written.
4. Issue file already exists from previous run — overwrite via `std::fs::write`
   (no `.tmp` needed for this phase; atomic write deferred).
  </action>
  <verify>
    <automated>cargo test -p reposix-cli -- refresh --nocapture 2>&1 | tail -40</automated>
  </verify>
  <done>
    All 4 integration tests pass:
    - `refresh_writes_md_files` green
    - `refresh_idempotent_no_diff` green (second diff contains only fetched_at.txt)
    - `refresh_fuse_active_guard` green
    - `fetched_at_is_current_timestamp` green
    `cargo test --workspace --quiet` shows 0 failures.
  </done>
</task>

<task type="auto">
  <name>Task 2: Workspace gate — full suite green + clippy clean</name>
  <files>
    crates/reposix-fuse/src/fs.rs,
    crates/reposix-cli/src/refresh.rs
  </files>
  <action>
Run `cargo test --workspace --quiet` and `cargo clippy --workspace --all-targets -- -D warnings`.
Fix any failures found. Expected issues to watch for:

1. **GITIGNORE_BYTES test in fs.rs:** The RESEARCH.md notes that expanding
   `GITIGNORE_BYTES` to include `.reposix/` could break the existing assertion
   at line 2742. Check whether Wave A's `git_refresh_commit` writes
   `.reposix/.gitignore` (sub-directory scope) instead of modifying the root
   `.gitignore`. If the sub-directory approach was used (recommended in
   RESEARCH.md Pitfall 4 option B), no change to `fs.rs` is needed. Verify
   this invariant:
   ```
   grep -n "GITIGNORE_BYTES" crates/reposix-fuse/src/fs.rs
   ```
   If `GITIGNORE_BYTES` was NOT modified → no action needed.
   If it WAS modified → check that the fs.rs test at line ~2742 was also updated.

2. **clippy::pedantic warnings in refresh.rs / cache_db.rs:** Common offenders:
   - `clippy::must_use_candidate` on public fns that return `Result` — add
     `#[must_use]` or allow the lint with rationale.
   - `clippy::missing_errors_doc` — add `# Errors` sections to all public fns.
   - `clippy::doc_markdown` — backtick-wrap identifiers in doc comments.
   - `clippy::module_name_repetitions` — if struct is named `RefreshConfig` in
     module `refresh`, rename to `Config` within the module if clippy fires.

3. **rustix Signal::None / kill_process 0:** If `is_fuse_active` uses a
   signal-0 kill, verify the rustix 0.38 API compiles correctly. If rustix
   does not expose signal 0 cleanly, use the `/proc/<pid>/status` file existence
   check as the fallback (read `/proc/{pid}/status`, return Ok(true) if exists,
   Ok(false) if NotFound, propagate other errors).

4. **Git commit in CI:** The `git_refresh_commit` helper must set
   `GIT_AUTHOR_NAME`, `GIT_COMMITTER_NAME`, `GIT_AUTHOR_EMAIL`,
   `GIT_COMMITTER_EMAIL` as `.env()` on the `Command` for the commit step.
   Without these, `git commit` fails in bare CI environments (Ubuntu runners
   have no global git config). Verify this is present; add if missing.

5. **tempfile dev-dep:** `tempfile = "3"` is already in
   `crates/reposix-cli/Cargo.toml` as a regular dep. Verify it is either in
   `[dependencies]` (acceptable for test helpers) or moved to
   `[dev-dependencies]`. The existing `Cargo.toml` shows `tempfile = "3"` as a
   regular dep — that's fine; leave it.

After all fixes, run the full suite one final time:
```
cargo test --workspace --quiet
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```

If `cargo fmt --all --check` fails, run `cargo fmt --all` and re-check.

The goal of this task is a clean "workspace gate" — zero test failures, zero
clippy warnings, zero fmt diffs.
  </action>
  <verify>
    <automated>cargo test --workspace --quiet 2>&1 | tail -20 && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | grep -E "^error|warning\[" | head -20</automated>
  </verify>
  <done>
    `cargo test --workspace --quiet` — 0 failures.
    `cargo clippy --workspace --all-targets -- -D warnings` — 0 warnings/errors.
    `cargo fmt --all --check` — 0 diffs.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Test harness → git subprocess | Tests call git via Command; must set author env vars to avoid CI failures |
| Issue body content → .md files on disk | Tainted backend content written to real filesystem; no additional surface vs Wave A |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-20B-01 | Tampering | Test PID in fuse.pid guard test | accept | Test writes current process PID; self-signal is safe and intentional. No production risk. |
| T-20B-02 | Information Disclosure | Test tempdir contents | accept | tempfile::tempdir() creates a 0700 directory; issue .md contents are synthetic test data only. |
| T-20B-03 | Repudiation | git commit author in CI | mitigate | Setting GIT_AUTHOR_* env vars on the commit Command ensures the author is always "reposix <label@project>" regardless of the CI runner's git config. Audit log row shows the correct author. |
</threat_model>

<verification>
1. `cargo test -p reposix-cli -- refresh` — all 4 integration tests + existing unit tests pass
2. `cargo test --workspace --quiet` — zero failures across all crates
3. `cargo clippy --workspace --all-targets -- -D warnings` — zero warnings
4. `cargo fmt --all --check` — zero diffs
5. Manual smoke: `git diff HEAD~1` after two refreshes with same data = only `fetched_at.txt` changed
</verification>

<success_criteria>
- `refresh_writes_md_files`: .md files committed, git log shows author contains "reposix"
- `refresh_idempotent_no_diff`: second refresh with no backend changes → git diff shows only fetched_at.txt
- `refresh_fuse_active_guard`: live pid in fuse.pid → run_refresh returns error with "FUSE mount is active"
- `fetched_at_is_current_timestamp`: timestamp within 30s of test wall clock
- Workspace CI gate: cargo test --workspace --quiet = 0 failures; clippy = 0 warnings
</success_criteria>

<output>
After completion, create `.planning/phases/20-op-3-reposix-refresh-subcommand-and-git-diff-cache-for-mount/20-B-SUMMARY.md`
with the standard summary template fields filled in.
</output>
