---
phase: 20-op-3-reposix-refresh-subcommand-and-git-diff-cache-for-mount
plan_id: 20-A
wave: 1
goal: >
  Implement the `reposix refresh` CLI subcommand: FUSE-active detection, SQLite
  metadata DB (cache.db), issue fetching + deterministic .md rendering, git
  init/add/commit, and fetched_at.txt sentinel.
depends_on: []
type: execute
autonomous: true
requirements:
  - REFRESH-01
  - REFRESH-03
  - REFRESH-04
  - REFRESH-05

must_haves:
  truths:
    - "`reposix refresh --mount <path> --project demo` fetches issues from the
      simulator and writes <N>.md files under <mount>/issues/"
    - "`git log` inside the mount dir shows one commit per refresh run"
    - "`.reposix/fetched_at.txt` contains an ISO-8601 UTC timestamp after each run"
    - "Running `reposix refresh` while `.reposix/fuse.pid` contains a live PID
      exits with a non-zero status and a human-readable error message"
    - "`--offline` flag is accepted by the parser but returns an error explaining
      offline mode is not yet implemented"
  artifacts:
    - path: "crates/reposix-cli/src/cache_db.rs"
      provides: "open_cache_db(), update_metadata(), lock guard via SQLite WAL EXCLUSIVE"
      exports: ["open_cache_db", "update_metadata", "CacheDb"]
    - path: "crates/reposix-cli/src/refresh.rs"
      provides: "run_refresh() — orchestrates fetch, render, write, git commit"
      exports: ["run_refresh", "RefreshConfig"]
    - path: "crates/reposix-cli/src/main.rs"
      provides: "Refresh variant in Cmd enum + dispatch"
      contains: "Cmd::Refresh"
  key_links:
    - from: "crates/reposix-cli/src/main.rs"
      to: "crates/reposix-cli/src/refresh.rs"
      via: "run_refresh(config).await"
      pattern: "run_refresh"
    - from: "crates/reposix-cli/src/refresh.rs"
      to: "reposix_core::frontmatter::render"
      via: "frontmatter::render(&issue)"
      pattern: "frontmatter::render"
    - from: "crates/reposix-cli/src/refresh.rs"
      to: "std::process::Command(\"git\")"
      via: "git_refresh_commit()"
      pattern: "Command::new\\(\"git\"\\)"
---

<objective>
Implement the `reposix refresh` subcommand end-to-end: add the clap variant,
write `refresh.rs` (fetch + render + file write + git commit) and `cache_db.rs`
(SQLite WAL metadata store with exclusive lock / busy detection).

Purpose: Delivers the "mount-as-time-machine" UX — after this plan, running
`reposix refresh` produces a committed git snapshot of the backend's current
state, so `git diff HEAD~1` shows real backend changes.

Output:
- `crates/reposix-cli/src/cache_db.rs` — CacheDb wrapper, open_cache_db,
  update_metadata, advisory lock via WAL+EXCLUSIVE
- `crates/reposix-cli/src/refresh.rs` — RefreshConfig struct, run_refresh()
  async fn, git subprocess helper, FUSE-active guard, --offline stub
- Updated `crates/reposix-cli/src/main.rs` — Refresh variant + dispatch arm
- Unit tests embedded in cache_db.rs covering open, update, lock-conflict
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

<interfaces>
<!-- Key contracts the executor needs. Extracted from codebase. -->

From crates/reposix-cli/src/main.rs (Cmd enum, add Refresh after List):
```rust
#[derive(Debug, Subcommand)]
enum Cmd {
    Sim { … },
    Mount { mount_point: PathBuf, origin: String, project: String, backend: list::ListBackend, read_only: bool },
    Demo { keep_running: bool },
    List { project: String, origin: String, backend: list::ListBackend, format: list::ListFormat },
    Version,
    // ADD: Refresh { … }
}
```

From crates/reposix-cli/src/list.rs (reuse ListBackend — do not redefine):
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ListBackend { Sim, Github, Confluence }
```

From reposix-core IssueBackend trait (async trait, dyn dispatch via Box):
```rust
pub trait IssueBackend {
    async fn list_issues(&self, project: &str) -> Result<Vec<Issue>>;
}
// Implementations: SimBackend::new(origin) -> Result<SimBackend>
```

From reposix_core::frontmatter::render:
```rust
pub fn render(issue: &Issue) -> Result<String, Error>
// Returns the deterministic YAML frontmatter + body string that FUSE serves.
// Same bytes on repeated calls with same input — safe to use as git content.
```

From reposix-core Issue struct (relevant fields):
```rust
pub struct Issue {
    pub id: IssueId,   // IssueId(u64)
    pub title: String,
    pub status: IssueStatus,
    pub updated_at: DateTime<Utc>,
    // … other fields
}
```

From crates/reposix-core/src/audit.rs (SQLite WAL pattern to mirror for cache_db):
```rust
pub fn open_audit_db(path: &Path) -> crate::Result<rusqlite::Connection> {
    let conn = rusqlite::Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    enable_defensive(&conn)?;
    load_schema(&conn)?;
    Ok(conn)
}
```

From crates/reposix-cli/src/mount.rs (std::process::Command pattern):
```rust
// Existing pattern — reuse for git subprocess calls in refresh.rs
let status = std::process::Command::new("fusermount3")
    .args(["-u", &mount_point.display().to_string()])
    .status()?;
```
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: cache_db.rs — SQLite metadata store with WAL exclusive lock</name>
  <files>crates/reposix-cli/src/cache_db.rs</files>
  <behavior>
    - Test: open_cache_db on a fresh tempdir creates .reposix/cache.db and
      refresh_meta table (single-row sentinel schema).
    - Test: update_metadata writes backend_name, project, last_fetched_at;
      a second call overwrites (INSERT OR REPLACE).
    - Test: opening a second connection while the first holds EXCLUSIVE mode
      returns an error whose message contains "another refresh is in progress"
      (SQLite SQLITE_BUSY mapped to anyhow error).
    - Test: open_cache_db is idempotent — calling twice on the same path is OK.
  </behavior>
  <action>
Create `crates/reposix-cli/src/cache_db.rs` with:

1. `CACHE_SCHEMA_SQL` const:
   ```sql
   CREATE TABLE IF NOT EXISTS refresh_meta (
       id              INTEGER PRIMARY KEY CHECK (id = 1),
       backend_name    TEXT NOT NULL,
       project         TEXT NOT NULL,
       last_fetched_at TEXT NOT NULL,
       commit_sha      TEXT
   );
   ```

2. `pub struct CacheDb(rusqlite::Connection)` — newtype wrapping the
   connection so Drop releases the exclusive lock.

3. `pub fn open_cache_db(mount: &std::path::Path) -> anyhow::Result<CacheDb>`:
   - `std::fs::create_dir_all(mount.join(".reposix"))`
   - Open the file with `OpenOptions::new().write(true).create(true).mode(0o600)`
     first (to set permissions), then `rusqlite::Connection::open(path)`.
   - `conn.pragma_update(None, "journal_mode", "WAL")?`
   - `conn.pragma_update(None, "locking_mode", "EXCLUSIVE")?`
   - Execute CACHE_SCHEMA_SQL via `conn.execute_batch`.
   - Map `rusqlite::Error::SqliteFailure` with `SQLITE_BUSY` (extended code
     `rusqlite::ffi::SQLITE_BUSY`) → `anyhow::anyhow!("another refresh is in
     progress; unmount or wait for the previous refresh to finish")`.
   - Return `Ok(CacheDb(conn))`.

4. `pub fn update_metadata(db: &CacheDb, backend_name: &str, project: &str,
   last_fetched_at: &str, commit_sha: Option<&str>) -> anyhow::Result<()>`:
   - `INSERT OR REPLACE INTO refresh_meta VALUES (1, ?, ?, ?, ?)`.

5. `impl CacheDb { pub fn conn(&self) -> &rusqlite::Connection { &self.0 } }`

Lint/code notes:
- `#![forbid(unsafe_code)]` is at crate root — no unsafe here.
- `use std::os::unix::fs::OpenOptionsExt;` for `.mode(0o600)`.
- All public items need doc comments with `# Errors` sections.
- `clippy::pedantic`: use `u32` not `usize` for `mode()`, `i64` not `u64` for
  the sqlite id check (SQLite INTEGER is signed 64-bit).

Embedded unit tests at bottom of file (use tempfile::tempdir()):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn open_creates_schema() { … }

    #[test]
    fn update_metadata_roundtrip() { … }

    #[test]
    fn lock_conflict_returns_error() { … }

    #[test]
    fn open_is_idempotent() { … }
}
```
  </action>
  <verify>
    <automated>cargo test -p reposix-cli -- cache_db --nocapture 2>&1 | tail -20</automated>
  </verify>
  <done>
    All four cache_db unit tests pass. `cargo clippy -p reposix-cli -- -D warnings`
    is clean. `open_cache_db` creates `.reposix/cache.db` with permissions 0o600.
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: refresh.rs + main.rs — fetch, render, write, git commit, clap wiring</name>
  <files>
    crates/reposix-cli/src/refresh.rs,
    crates/reposix-cli/src/main.rs
  </files>
  <behavior>
    - Test (unit): `is_fuse_active` returns true when a tmp file named fuse.pid
      contains the current process's PID (self-process is definitely alive).
    - Test (unit): `is_fuse_active` returns false when .reposix/fuse.pid does
      not exist.
    - Test (unit): `is_fuse_active` returns false when .reposix/fuse.pid contains
      a PID that does not exist (e.g. 99999999).
    - Test (unit): `git_refresh_commit` with a pre-initialized git repo + a
      committed file: calling the helper adds the staged file and creates a
      commit whose author matches the argument exactly.
    - NOTE: Full integration test (sim + full refresh flow) is Wave B.
  </behavior>
  <action>
**Create `crates/reposix-cli/src/refresh.rs`:**

```rust
//! `reposix refresh` — re-fetch backend issues, write .md files, git commit.
//!
//! # Errors
//! Every public function documents its error conditions.
```

1. `pub struct RefreshConfig`:
   ```rust
   pub struct RefreshConfig {
       pub mount_point: std::path::PathBuf,
       pub origin: String,
       pub project: String,
       pub backend: crate::list::ListBackend,
       pub offline: bool,
   }
   ```

2. `pub async fn run_refresh(cfg: RefreshConfig) -> anyhow::Result<()>`:
   - If `cfg.offline` → `anyhow::bail!("--offline mode is not yet implemented for refresh; serve existing .md files from the mount directly")`.
   - Call `is_fuse_active(&cfg.mount_point)?`; if true → `anyhow::bail!("FUSE mount is active at {}; run `reposix unmount` first, then refresh", cfg.mount_point.display())`.
   - Open `cache_db::open_cache_db(&cfg.mount_point)?`.
   - Build backend and call `list_issues` — same dispatch pattern as `list.rs`:
     - `ListBackend::Sim` → `SimBackend::new(cfg.origin)?.list_issues(&cfg.project).await?`
     - `ListBackend::Github` → `GithubReadOnlyBackend::new(token)?.list_issues(&cfg.project).await?`
     - `ListBackend::Confluence` → read env vars, `ConfluenceBackend::new(creds, &tenant)?.list_issues(&cfg.project).await?`
   - Determine bucket name: `"issues"` for Sim/Github, `"pages"` for Confluence.
   - `std::fs::create_dir_all(cfg.mount_point.join(bucket))?`
   - For each issue: render via `reposix_core::frontmatter::render(&issue)?`,
     write to `<mount>/<bucket>/<padded-id>.md` where padded-id is
     `format!("{:011}", issue.id.0)`.
   - Write `<mount>/.reposix/fetched_at.txt` with
     `chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)`.
   - Construct author string: `format!("reposix <{}@{}>", cfg.backend_label(), cfg.project)`.
   - Construct commit message: `format!("reposix refresh: {backend}/{project} — {n} issues at {ts}", …)`.
   - Call `git_refresh_commit(&cfg.mount_point, bucket, &author, &message)?`.
   - `cache_db::update_metadata(&db, &cfg.backend_label(), &cfg.project, &ts, None)?`
   - Print summary to stdout: `println!("refreshed {n} issues into {}", cfg.mount_point.display())`.

3. `fn backend_label(cfg: &RefreshConfig) -> &'static str`:
   Returns `"simulator"` for Sim, `"github"` for Github, `"confluence"` for
   Confluence. Add as a method on `RefreshConfig`:
   ```rust
   pub fn backend_label(&self) -> &'static str { … }
   ```

4. `fn is_fuse_active(mount: &std::path::Path) -> anyhow::Result<bool>`:
   - Read `mount/.reposix/fuse.pid`; if `NotFound` → return `Ok(false)`.
   - Parse PID from file content (trim whitespace, parse as `u32`).
   - Send signal 0 to the PID via `rustix::process::kill_process(pid, Signal::None)`:
     - `Ok(())` → process alive → return `Ok(true)`.
     - `Err(rustix::io::Errno::SRCH)` (ESRCH) → process dead → return `Ok(false)`.
     - Other error → propagate.
   - Note: `rustix` is already in `[dependencies]` with `features = ["process"]`.

5. `fn git_refresh_commit(mount: &std::path::Path, bucket: &str, author: &str,
   message: &str) -> anyhow::Result<()>`:
   - Helper closure: `let g = |args: &[&str]| -> anyhow::Result<()> { … }`.
   - `g(&["init"])?` — idempotent.
   - Add gitignore entry for cache.db WAL files: write
     `.reposix/.gitignore` with content `"cache.db\ncache.db-wal\ncache.db-shm\n"`
     before staging (so it's committed alongside fetched_at.txt).
   - `g(&["add", "--", bucket, ".reposix/fetched_at.txt", ".reposix/.gitignore"])?`.
   - `g(&["commit", "--allow-empty", &format!("--author={author}"), "-m", message])?`.
   - Set `GIT_AUTHOR_NAME` and `GIT_COMMITTER_NAME` to `"reposix"`,
     `GIT_AUTHOR_EMAIL` / `GIT_COMMITTER_EMAIL` via env on the Command rather
     than relying solely on `--author` (git requires a valid config or env to
     commit; avoids "user.email not set" error in bare CI environments).

**Update `crates/reposix-cli/src/main.rs`:**

1. Add `mod refresh;` alongside existing `mod list;` etc.

2. Add `Refresh` variant to `Cmd` enum (insert between `List` and `Version`):
   ```rust
   /// Re-fetch all issues/pages from the backend, write .md files into the
   /// mount directory, and create a git commit so `git diff HEAD~1` shows
   /// backend changes.
   ///
   /// The mount must NOT be actively FUSE-mounted (unmount first).
   Refresh {
       /// Mount point (a plain directory that is also a git working tree).
       mount_point: PathBuf,
       /// Backend origin (simulator URL).
       #[arg(long, default_value = "http://127.0.0.1:7878")]
       origin: String,
       /// Project slug (sim) or `owner/repo` (github) or space KEY (confluence).
       #[arg(long, default_value = "demo")]
       project: String,
       /// Which backend to speak.
       #[arg(long, value_enum, default_value_t = list::ListBackend::Sim)]
       backend: list::ListBackend,
       /// Serve from cached .md files; no network egress.
       /// NOTE: offline read path is Phase 21; this flag is accepted but
       /// currently returns an error.
       #[arg(long)]
       offline: bool,
   },
   ```

3. Add dispatch arm in `match cli.cmd`:
   ```rust
   Cmd::Refresh { mount_point, origin, project, backend, offline } => {
       refresh::run_refresh(refresh::RefreshConfig {
           mount_point, origin, project, backend, offline,
       }).await
   }
   ```

Lint/code notes:
- `clippy::pedantic`: avoid `unwrap()` in non-test code; use `?` + anyhow context.
- `anyhow::Context` must be in scope (already in `list.rs` — add `use anyhow::Context;`
  in refresh.rs).
- `use reposix_confluence::{ConfluenceBackend, ConfluenceCreds};` and related
  imports mirror `list.rs` exactly.
- The `is_fuse_active` function using `rustix::process::Signal` — check the
  rustix 0.38 API: the variant is `Signal::Quit` for SIGQUIT; for signal 0 use
  `rustix::process::test_kill_process(pid)` if available, or construct via
  `kill_process` with a zero signal. Inspect the rustix 0.38 docs for the
  correct call. Fallback: read `/proc/<pid>/status` existence (file present =
  process alive) to avoid any unsafe concern.
  
Unit tests at bottom of refresh.rs (use tempfile::tempdir() + std::process::Command
to set up a minimal git repo):
```rust
#[cfg(test)]
mod tests {
    // is_fuse_active tests: no fuse.pid, live pid, dead pid
    // git_refresh_commit test: pre-init repo, write a file, call helper, assert commit exists
}
```
  </action>
  <verify>
    <automated>cargo test -p reposix-cli -- refresh --nocapture 2>&1 | tail -30 && cargo clippy -p reposix-cli -- -D warnings 2>&1 | tail -20</automated>
  </verify>
  <done>
    Unit tests for `is_fuse_active` (3 cases) and `git_refresh_commit` (1 case)
    pass. `cargo clippy -p reposix-cli -- -D warnings` clean. `cargo check
    --workspace` clean. `reposix refresh --help` shows the subcommand.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| backend HTTP → CLI process | Issue bodies + titles are attacker-influenced (tainted); written verbatim to .md files on disk |
| .reposix/fuse.pid → refresh guard | PID file content is file-system controlled; must not trust it blindly (parse as u32, validate with signal-0 check) |
| git subprocess → mount dir | `--author` value derived from user-supplied `--project` and backend label; must not allow shell injection |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-20A-01 | Tampering | git commit --author string | mitigate | Author is `format!("reposix <{label}@{project}>")` where `label` is a `&'static str` (safe) and `project` comes from CLI arg. Pass author as a single `--author=VALUE` argument to Command::args — not via shell string interpolation. No shell injection surface. |
| T-20A-02 | Information Disclosure | .reposix/cache.db | mitigate | Create file with mode 0o600 before opening via rusqlite; only the mount owner can read backend metadata. |
| T-20A-03 | Tampering | fuse.pid TOCTOU | accept | The PID check is advisory (error-and-exit on conflict, not a security boundary). A concurrent attacker controlling fuse.pid can at worst cause a spurious write-through-FUSE; this is mitigated at the FUSE write-callback layer (backend update calls are audited). Low severity. |
| T-20A-04 | Tampering | Issue body rendered to .md | accept | `frontmatter::render` is the canonical renderer used by the FUSE read path. No new rendering surface introduced. Tainted content is written to disk (not re-executed). Existing T-tainted-content controls apply. |
| T-20A-05 | Denial of Service | SQLite EXCLUSIVE lock starvation | mitigate | Map SQLITE_BUSY → clear error "another refresh is in progress". Caller exits non-zero. No blocking wait. |
</threat_model>

<verification>
1. `cargo test -p reposix-cli -- cache_db` — 4 unit tests green
2. `cargo test -p reposix-cli -- refresh` — 4 unit tests green (is_fuse_active x3, git_refresh_commit x1)
3. `cargo clippy --workspace --all-targets -- -D warnings` — zero warnings
4. `cargo check --workspace` — clean
5. `reposix refresh --help` — shows subcommand with all flags documented
</verification>

<success_criteria>
- `cache_db::open_cache_db` creates `.reposix/cache.db` (mode 0600) + schema
- `cache_db::update_metadata` writes backend/project/timestamp metadata
- Concurrent `open_cache_db` → second caller errors with "another refresh is in progress"
- `refresh::run_refresh` with `--offline` → clear error returned
- `refresh::run_refresh` with live `.reposix/fuse.pid` → clear error returned
- `reposix refresh` variant present in clap Cmd enum; `--help` output confirms
- All unit tests pass; workspace clippy clean
</success_criteria>

<output>
After completion, create `.planning/phases/20-op-3-reposix-refresh-subcommand-and-git-diff-cache-for-mount/20-A-SUMMARY.md`
with the standard summary template fields filled in.
</output>
