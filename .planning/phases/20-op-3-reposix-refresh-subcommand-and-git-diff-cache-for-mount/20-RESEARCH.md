# Phase 20: `reposix refresh` subcommand and git-diff cache for mount (OP-3) — Research

**Researched:** 2026-04-15
**Domain:** Rust CLI subcommand, SQLite WAL persistence, git process invocation, FUSE in-memory cache extension
**Confidence:** HIGH (codebase study) / MEDIUM (git approach selection)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
1. **Cache location:** `.reposix/cache.db` inside the mount (stays inside the git working tree — `git log` works natively).
2. **Is cache git-tracked?** Only rendered `.md` files are committed; the DB is gitignored. Each `reposix refresh` commit includes only the `.md` files (+ `.reposix/fetched_at.txt`) — not the binary DB blob.
3. **Commit author:** `reposix <backend>@<tenant>`.
4. **Concurrent mount safety:** SQLite WAL advisory lock; error-and-exit on conflict (not block-and-wait).
5. **Offline mode:** `--offline` flag on `reposix mount` and `reposix refresh`; zero egress when set.
6. **Invalidation:** `--force` semantics for v0.6.0 (overwrite cache with current backend state). No merge/rebase mode yet.

### Claude's Discretion
- New crate vs module: whether `reposix-cache` is a new crate or a module inside `reposix-fuse` or `reposix-cli`.
- Git invocation: `std::process::Command("git")` vs `git2` crate.
- Scope of Wave 0 vs later waves: which parts to ship in Phase 20 vs defer.

### Deferred Ideas (OUT OF SCOPE)
- `git pull --rebase` semantics (merge server changes with local edits) — future phase.
- Multi-space mount.
- Time-bucketed views (`mount/recent/<yyyy-mm-dd>/`).
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CACHE-01 | `reposix refresh` subcommand re-fetches all pages from the backend and writes a git commit into the mount's working tree | CLI subcommand pattern from `main.rs` + `list.rs`; `refresh_issues()` in `fs.rs`; git via `std::process::Command` |
| CACHE-02 | `git diff HEAD~1` in the mount shows what changed at the backend since the last refresh | Requires the mount to be a git repo (`git init`), files committed with deterministic content; `frontmatter::render` already deterministic |
| CACHE-03 | `mount/.reposix/fetched_at.txt` records the timestamp of the last backend round-trip | Simple file write in `reposix-cache` or `reposix-cli/src/refresh.rs`; `.reposix/` directory creation |
</phase_requirements>

---

## Summary

Phase 20 adds the "mount-as-time-machine" UX: a `reposix refresh` CLI subcommand that (1) re-fetches all issues/pages from the configured backend, (2) writes each rendered `.md` file into the mount's own git working tree, and (3) commits the result so `git log`/`git diff` inside the mount becomes a history of backend snapshots. The `_INDEX.md` files (shipped in Phases 15 and 18) serve as the natural change sentinel — `git diff HEAD~1 _INDEX.md` shows the summary delta between two syncs.

The current cache mechanism is an in-memory `DashMap<u64, Arc<CachedFile>>` inside `ReposixFs`, rebuilt on each `readdir` call. That cache is process-lifetime only — there is no persistence, no git tracking of files. Phase 20 adds a parallel git-backed layer: `reposix refresh` writes the same `.md` bytes (already deterministically produced by `frontmatter::render`) to real files in `<mount>/<bucket>/`, then commits them. The FUSE read path is NOT rewritten in Phase 20; the in-memory cache remains the live-read mechanism.

**Primary recommendation:** Implement `reposix refresh` as a standalone async function in `crates/reposix-cli/src/refresh.rs` — no new crate needed. Use `std::process::Command("git")` for all git operations. A new `crates/reposix-cache` crate is DEFERRED to a later phase; the SQLite cache.db for Phase 20 is a minimal metadata store (last-fetch timestamp and backend config), not a full issue cache replacement.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Backend re-fetch (list + get all) | `reposix-cli` (refresh command) | `reposix-core` (`IssueBackend`) | Refresh is a CLI-orchestration concern, not a FUSE concern; it reuses the existing backend trait |
| Write `.md` files to mount | `reposix-cli` (refresh command) | `reposix-core` (`frontmatter::render`) | Refresh writes to the real filesystem (not via FUSE); same render function used by FUSE ensures identical bytes |
| Git operations (init/add/commit) | `reposix-cli` (refresh command) | None (shell out to git) | No `git2` in workspace; `std::process::Command` is the established pattern for external processes in this codebase |
| `.reposix/cache.db` metadata | `reposix-cli` or new `reposix-cache` module | `reposix-core` (rusqlite already present) | SQLite for metadata (last_fetched_at, backend_name, project); rusqlite 0.32 bundled already in workspace |
| SQLite WAL lock / concurrency guard | `reposix-cache` (wherever cache.db lives) | None | File-based advisory lock; error-and-exit on conflict |
| `fetched_at.txt` sentinel | `reposix-cli` (refresh command) | None | Plain file write; no DB needed for this |
| `--offline` FUSE read path | `reposix-fuse` (`ReposixFs`) | `reposix-cache` | When offline, reads serve from cache instead of calling the backend; FUSE owns the read dispatch |

---

## Standard Stack

### Core (already in workspace — no new deps for MVP)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `rusqlite` | 0.32.1 (`bundled`) | SQLite cache.db for metadata | [VERIFIED: Cargo.lock] Already present in workspace with `bundled` feature; no system `libsqlite3` required |
| `chrono` | 0.4 | `fetched_at` timestamps | [VERIFIED: workspace Cargo.toml] Already a workspace dep |
| `tokio` | 1 | Async re-fetch | [VERIFIED: workspace Cargo.toml] Already a workspace dep |
| `std::process::Command` | stdlib | Git invocation | [VERIFIED: codebase] Established pattern — see `mount.rs` `watchdog_unmount` (fusermount3), `sim.rs` child spawn |
| `anyhow` | 1 | Error propagation at binary boundary | [VERIFIED: workspace Cargo.toml] Standard for binary crates in this project |
| `reposix-core` | workspace | `IssueBackend` trait + `frontmatter::render` | [VERIFIED: codebase] All existing CLI commands use this path |

### Supporting (new additions)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `git2` crate | — | Programmatic git operations | REJECTED for Phase 20 — see below. Adds ~1MB to binary, requires C lib, not in workspace. |
| `tempfile` | 3 | Atomic write of rendered `.md` files | [VERIFIED: crates/reposix-cli/Cargo.toml] Already a dev-dep; may be needed for test setup |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `std::process::Command("git")` | `git2` crate | `git2` is programmatic but adds 50+ transitive deps, requires C compilation, no bundled build. `Command` is simpler, already established in codebase for external processes, and produces correct output. Git 2.x is already present on the host. |
| New `reposix-cache` crate | Module in `reposix-cli` | New crate adds workspace overhead. For Phase 20's narrow scope (metadata DB + file writes), a module `crates/reposix-cli/src/refresh.rs` + `crates/reposix-cli/src/cache_db.rs` is sufficient. Promote to crate in Phase 21+ if reused by FUSE. |
| Full SQLite issue cache | Plain `fetched_at.txt` + filesystem `.md` files | Full cache.db replacing in-memory cache is a large change to `ReposixFs`. Phase 20 scopes to: (a) metadata-only DB (last fetch time, backend config), (b) real `.md` files in mount as the "cache". The FUSE live-read path is untouched. |

**Version verification:** `rusqlite` 0.32.1 confirmed in Cargo.lock. [VERIFIED: Cargo.lock]

---

## Architecture Patterns

### System Architecture Diagram

```
reposix refresh --mount <path> --backend sim --project demo
        │
        ▼
[CLI: crates/reposix-cli/src/refresh.rs]
        │  1. open/create .reposix/cache.db (WAL, advisory lock)
        │  2. check --offline flag → skip if set
        │  3. backend.list_issues(project) + backend.get_issue() per id
        │     └── IssueBackend trait (existing)
        │  4. frontmatter::render(issue) → bytes
        │     └── same bytes FUSE serves on read
        │  5. write <mount>/<bucket>/<padded-id>.md to real filesystem
        │     └── these are OUTSIDE the FUSE mount (written before or after unmount)
        │     OR: the mount must NOT be active during refresh (see Pitfall 1)
        │  6. write <mount>/_INDEX.md, <mount>/<bucket>/_INDEX.md (render_*_index)
        │  7. write <mount>/.reposix/fetched_at.txt
        │  8. update cache.db metadata (last_fetched_at, backend, project)
        │
        ▼
[Git subprocess: std::process::Command("git")]
        │  1. git -C <mount> init (idempotent)
        │  2. git -C <mount> add <bucket>/*.md _INDEX.md <bucket>/_INDEX.md
        │                      .reposix/fetched_at.txt
        │  3. git -C <mount> commit -m "reposix refresh: <backend>/<project> at <ts>"
        │              --author "reposix <backend>@<project>"
        │              --allow-empty-message (if no changes)
        │
        ▼
[Mount working tree after refresh]
  <mount>/
    .git/                  ← git history of syncs
    .gitignore             ← served by FUSE: "/tree/\nlabels/\n"
                              + ".reposix/cache.db" (added by refresh)
    .reposix/
      cache.db             ← metadata only; gitignored
      fetched_at.txt       ← committed; human-readable timestamp
    issues/                ← (or pages/ for Confluence)
      00000000001.md
      00000000002.md
      _INDEX.md
    _INDEX.md
    tree/ → (FUSE symlink overlay; not committed)
    labels/ → (FUSE symlink overlay; not committed)
```

**Key architectural constraint:** The mount path is simultaneously a FUSE mountpoint and a git working tree. `reposix refresh` writes to the *underlying real directory* that FUSE is mounted over. When FUSE is active, writes to `<mount>/<bucket>/*.md` go through the FUSE write callbacks — which is NOT what we want (those callbacks PATCH the backend). This is Pitfall 1; see below.

### Recommended Project Structure

```
crates/reposix-cli/src/
├── main.rs          # add Refresh variant to Cmd enum
├── refresh.rs       # new — RefreshConfig + run_refresh()
├── cache_db.rs      # new — open_cache_db(), update_metadata(), lock guard
├── mount.rs         # existing — unchanged
├── list.rs          # existing — unchanged
└── demo.rs          # existing — unchanged
```

The FUSE mount (`crates/reposix-fuse/`) is NOT modified in Phase 20. The `--offline` flag can be added to `reposix mount` as a forward-compat flag but the offline *read path* (serving from cache.db) is a separate, larger change deferred to Phase 21.

### Pattern 1: Git Subprocess Invocation

**What:** Shell out to `git` via `std::process::Command` with `-C <dir>` to target the mount directory.
**When to use:** All git operations in Phase 20 — init, add, commit.
**Example:**
```rust
// Source: [ASSUMED — follows established pattern in crates/reposix-cli/src/mount.rs]
fn git_commit_refresh(mount: &Path, author: &str, message: &str) -> anyhow::Result<()> {
    // git init is idempotent
    let status = std::process::Command::new("git")
        .args(["-C", &mount.display().to_string(), "init"])
        .status()?;
    anyhow::ensure!(status.success(), "git init failed");

    // Stage only the files refresh wrote — never `git add .`
    std::process::Command::new("git")
        .args(["-C", &mount.display().to_string(), "add",
               "_INDEX.md", "issues/", ".reposix/fetched_at.txt"])
        .status()?;

    // Commit with --allow-empty to handle "no changes since last refresh"
    std::process::Command::new("git")
        .args(["-C", &mount.display().to_string(),
               "commit", "--allow-empty",
               "--author", author,
               "-m", message])
        .status()?;
    Ok(())
}
```

### Pattern 2: SQLite Metadata DB (cache.db)

**What:** Lightweight SQLite DB under `.reposix/cache.db` for refresh metadata. NOT the full issue cache.
**When to use:** Track `last_fetched_at`, `backend_name`, `project`; enforce the advisory lock.
**Schema:**
```sql
-- Source: [ASSUMED — follows reposix-core/src/audit.rs pattern]
CREATE TABLE IF NOT EXISTS refresh_meta (
    id              INTEGER PRIMARY KEY CHECK (id = 1),  -- single-row sentinel
    backend_name    TEXT NOT NULL,
    project         TEXT NOT NULL,
    last_fetched_at TEXT NOT NULL,   -- ISO-8601 UTC
    commit_sha      TEXT             -- git commit SHA after last successful refresh
);
```

The WAL advisory lock is the exclusive-write semantics: SQLite WAL mode + `PRAGMA locking_mode=EXCLUSIVE` while refresh is running. Concurrent `reposix refresh` gets `SQLITE_BUSY` → convert to `anyhow` error with "another refresh is in progress".

### Pattern 3: `reposix refresh` CLI Subcommand

Follows the existing pattern exactly: add a `Refresh` variant to `Cmd` in `main.rs`, implement `refresh::run(...)` in `refresh.rs`.

```rust
// Source: [VERIFIED: crates/reposix-cli/src/main.rs]
Refresh {
    /// Mount point (must be an initialized reposix mount).
    mount_point: PathBuf,
    /// Backend origin.
    #[arg(long, default_value = "http://127.0.0.1:7878")]
    origin: String,
    /// Project slug.
    #[arg(long, default_value = "demo")]
    project: String,
    /// Which backend to speak.
    #[arg(long, value_enum, default_value_t = list::ListBackend::Sim)]
    backend: list::ListBackend,
    /// Serve from cache; no egress.
    #[arg(long)]
    offline: bool,
},
```

### Anti-Patterns to Avoid

- **Never `git add .` in the mount.** The mount contains `.git/`, `tree/` (FUSE-managed), `labels/` (FUSE-managed), and `.reposix/cache.db` (should be gitignored). Stage only the explicit list of files refresh wrote.
- **Never write `.md` files through the FUSE mount path while it is active.** FUSE write callbacks route writes to `backend.update_issue()` / `backend.create_issue()`. Instead: detect if mount is active (presence of `.reposix/fuse.pid` or FUSE-specific stat), refuse to run refresh while mounted, or run refresh only on an unmounted directory.
- **Never commit cache.db as a git blob.** It is a binary SQLite file that changes on every write. It would bloat history. Gitignore it from the first `git init`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| SQLite bundled build | Custom build scripts or system-dep sqlite3 | `rusqlite` with `bundled` feature | [VERIFIED: Cargo.lock] Already in workspace; compiles sqlite3 in-process; no system dep |
| Concurrent-writer detection | Custom file lock daemon | SQLite WAL `PRAGMA locking_mode=EXCLUSIVE` | SQLite provides advisory locking; `SQLITE_BUSY` is the standard conflict signal |
| Deterministic issue rendering | Custom serializer | `reposix_core::frontmatter::render` | [VERIFIED: codebase] The FUSE read path uses this; using the same function guarantees `git diff` shows real backend changes, not serialization artifacts |
| Git history management | Custom history file | `git init` + `git commit` via subprocess | FUSE mount is already designed as a git working tree (`.gitignore` is synthesized); leveraging real git means agents get full `git log`, `git diff`, `git show` for free |

**Key insight:** The most important invariant is that `frontmatter::render(issue)` is called identically by both the FUSE read path and the `reposix refresh` write path. If these diverge, `git diff` shows phantom changes. Use `reposix_core::frontmatter::render` in both places — it is already the canonical renderer.

---

## Scope Recommendation

### What to build in Phase 20 (MVP — 2-3 waves)

**Wave A — Foundations:**
- `crates/reposix-cli/src/cache_db.rs`: `open_cache_db(mount: &Path)`, WAL mode, schema (`refresh_meta` table), exclusive lock / busy-error, `update_metadata(...)`.
- `.reposix/` directory creation + `.gitignore` entry for `cache.db`.

**Wave B — `reposix refresh` subcommand:**
- `crates/reposix-cli/src/refresh.rs`: `run(RefreshConfig)` — fetch all issues, render `.md` files, write `fetched_at.txt`, update cache.db.
- `crates/reposix-cli/src/main.rs`: `Refresh` variant in `Cmd`.
- Git subprocess integration: `git init` (idempotent), selective `git add`, `git commit --allow-empty`.
- Unit tests for `cache_db` (open/lock/update/lock-conflict).

**Wave C — Integration + docs:**
- Integration test: sim → `reposix refresh` → inspect committed files → second refresh → `git log` shows 2 commits.
- `fetched_at.txt` content verified in test.
- CHANGELOG `[v0.6.0]` entry.

### What to defer (NOT Phase 20)

| Deferred Item | Reason | Target |
|---------------|--------|--------|
| Full SQLite issue cache replacing `DashMap` in FUSE | Large FUSE rewrite; orthogonal to the refresh UX | Phase 21+ |
| `--offline` read path in FUSE (serve `.md` from cache.db) | Requires FUSE dispatch change | Phase 21+ |
| `reposix-cache` as its own crate | Only needed if `reposix-fuse` also consumes it | Phase 21+ |
| `--offline` flag on `reposix mount` | Forward-compat only in Phase 20; behavior is `--offline` = error/noop | Phase 20 flag declaration, runtime behavior Phase 21 |
| `git pull` hook integration | Complex; requires the mount to understand git hooks | Future |

**Scope rationale:** The user-visible win (mount-as-time-machine via `git log`/`git diff`) is delivered entirely by the `reposix refresh` subcommand writing real files and committing them. The SQLite cache.db in Phase 20 is metadata-only (~20 lines of schema). The full cache replacement is a separate, larger phase that should not block the UX delivery.

---

## Common Pitfalls

### Pitfall 1: Writing through an active FUSE mount
**What goes wrong:** `reposix refresh` opens `<mount>/issues/00000000001.md` for writing while FUSE is active. The kernel routes the write through the FUSE `write` callback, which calls `backend.update_issue()` — patching the tracker with the same data you just fetched. Audit log gets spurious writes.
**Why it happens:** The mount point is both a FUSE mountpoint and a real directory. Open file handles see the FUSE layer.
**How to avoid:** `reposix refresh` MUST detect if the mount has an active FUSE session. Strategy: check for `.reposix/fuse.pid` (a PID file the daemon writes on mount) or stat the mountpoint with `statvfs` and check for FUSE fstype. Simplest for Phase 20: refuse refresh if the FUSE process is running; require user to unmount first, refresh, then re-mount.
**Warning signs:** Unexpected PATCH requests in the sim audit log during refresh.

### Pitfall 2: `git add .` including tree/ and labels/ symlinks
**What goes wrong:** `git add .` in the mount also stages the `tree/` and `labels/` FUSE-managed symlinks (if mount is not active) or fails with ENOENT (if they are dangling). Either way, the commit history gets polluted or `git add` errors.
**Why it happens:** FUSE synthesizes `tree/` and `labels/` as virtual directories; they have no real backing on disk.
**How to avoid:** Always use an explicit file list: `git add <bucket>/ _INDEX.md <bucket>/_INDEX.md .reposix/fetched_at.txt`. Never `git add .`.

### Pitfall 3: Non-deterministic render causing spurious git diffs
**What goes wrong:** Two consecutive `reposix refresh` runs with no backend changes produce a non-empty `git diff` because `frontmatter::render` produces different bytes (e.g. timestamp in the rendered body, map iteration order in YAML).
**Why it happens:** Serialization order is not guaranteed unless pinned.
**How to avoid:** `frontmatter::render` already uses `serde_yaml` 0.9 with deterministic field ordering (it serializes a struct, not a HashMap). The existing FUSE unit tests verify the render output is stable. Verify in Phase 20 integration test: refresh twice with no backend changes → `git diff HEAD~1` is empty.

### Pitfall 4: `.gitignore` conflict between FUSE-synthesized and refresh-written
**What goes wrong:** The FUSE daemon synthesizes `.gitignore` with `/tree/\nlabels/\n`. Refresh wants to also add `.reposix/cache.db` to `.gitignore`. Writing a real `.gitignore` to disk conflicts with the FUSE-synthesized one when FUSE is active.
**Why it happens:** FUSE intercepts reads/writes to `.gitignore`; the synthesized bytes are baked in as `GITIGNORE_BYTES: &[u8] = b"/tree/\nlabels/\n"`.
**How to avoid (two options):**
- A) Expand the FUSE-synthesized `.gitignore` to include `.reposix/` (change `GITIGNORE_BYTES` in `fs.rs`). [RECOMMENDED — single source of truth]
- B) Write `.reposix/.gitignore` as a subdirectory-scoped ignore file (git allows this) rather than modifying the root `.gitignore`.

### Pitfall 5: SQLite WAL files polluting git status
**What goes wrong:** SQLite WAL mode creates `cache.db-wal` and `cache.db-shm` alongside `cache.db`. If only `cache.db` is in `.gitignore`, `cache.db-wal` shows in `git status` as untracked.
**Why it happens:** SQLite WAL mode is the standard pattern in this codebase (see `demo.rs` WAL sibling cleanup) and `*.db` is already in the root `.gitignore`.
**How to avoid:** The root `.gitignore` already has `*.db`, `*.db-wal`, `*.db-shm` — check `crates/reposix-core/fixtures/audit.sql` and confirm glob coverage. [VERIFIED: root `.gitignore` has `*.db`, `*.db-shm`, `*.db-wal`]

---

## Code Examples

### Opening cache.db with WAL mode
```rust
// Source: [ASSUMED — follows reposix-core/src/audit.rs pattern, adapted]
// Note: audit.rs uses DbConfig::SQLITE_DBCONFIG_DEFENSIVE; cache.db does not need
// append-only semantics, so DEFENSIVE is optional here.
pub fn open_cache_db(mount: &Path) -> anyhow::Result<rusqlite::Connection> {
    let dir = mount.join(".reposix");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("cache.db");
    let conn = rusqlite::Connection::open(&path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "locking_mode", "EXCLUSIVE")?;
    conn.execute_batch(CACHE_SCHEMA_SQL)?;
    Ok(conn)
}
```

### Rendering and writing issue files
```rust
// Source: [VERIFIED: reposix_core::frontmatter::render usage in fs.rs lines 874-888]
for issue in &issues {
    let rendered = reposix_core::frontmatter::render(issue)?;
    let padded = format!("{:011}.md", issue.id.0);
    let dest = mount.join(bucket).join(&padded);
    std::fs::write(&dest, rendered.as_bytes())?;
}
```

### Selective git add + commit
```rust
// Source: [ASSUMED — follows std::process::Command pattern in mount.rs]
fn git_refresh_commit(mount: &Path, bucket: &str, author: &str, msg: &str) -> anyhow::Result<()> {
    let g = |args: &[&str]| -> anyhow::Result<()> {
        let status = std::process::Command::new("git")
            .arg("-C").arg(mount)
            .args(args)
            .status()?;
        anyhow::ensure!(status.success(), "git {:?} failed", args);
        Ok(())
    };
    g(&["init"])?;
    // Stage only refresh-written files; never glob the whole tree
    g(&["add", bucket, "_INDEX.md",
        &format!("{bucket}/_INDEX.md"),
        ".reposix/fetched_at.txt"])?;
    // --allow-empty: idempotent if backend has not changed
    g(&["commit", "--allow-empty", "--author", author, "-m", msg])?;
    Ok(())
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| FUSE mount is live-on-every-read | Phase 20 adds explicit refresh + git commit | Phase 20 | Mount becomes a time machine; FUSE live-read path unchanged |
| No git history of backend state | `git log` in mount shows refresh history | Phase 20 | Agents can `git diff` to see backend changes |
| `cache: DashMap` in FUSE (in-memory, ephemeral) | Unchanged in Phase 20; `.md` files on disk serve as persistent cache | Phase 20 | Persistent readable state survives unmount; FUSE cache upgrade deferred |

**Deprecated/outdated:**
- The CONTEXT.md design suggestion that `cache.db` stores the full issue data is descoped for Phase 20 — it stores metadata only. Full cache.db use-as-FUSE-backing is Phase 21+.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `reposix refresh` must run on an UNMOUNTED directory (not while FUSE is active), because writing through an active FUSE mount would trigger `update_issue` backend calls | Architecture Patterns / Pitfall 1 | If there is a way to bypass the FUSE layer (e.g., write to the underlying device directly) the design could allow concurrent mount+refresh, but this is kernel-level and unsafe |
| A2 | `std::process::Command("git")` works correctly within the `#![forbid(unsafe_code)]` constraint (it is safe Rust) | Standard Stack | N/A — this is definitively safe; no risk |
| A3 | The FUSE-synthesized `.gitignore` at `mount/.gitignore` can be extended (by changing `GITIGNORE_BYTES`) to include `.reposix/` without breaking existing tests | Pitfall 4 | If tests assert the exact content `b"/tree/\nlabels/\n"` they will need updating; search shows `GITIGNORE_BYTES` is a const — tests checking exact bytes must be updated |
| A4 | `git commit --allow-empty` is available on git 2.x (present on host: git 2.25.1) | Standard Stack | Confirmed — `--allow-empty` is available since git 1.5.4.6 [VERIFIED: git version 2.25.1] |
| A5 | The `refresh` command can reconstruct backend+project config from `.reposix/cache.db` metadata without requiring the user to re-specify all flags on every call | Standard Stack | If cache.db stores backend+project, subsequent `reposix refresh <mount>` needs no flags — but v0.6.0 can require flags explicitly and Phase 21 can add config persistence |

---

## Open Questions

1. **Mount state detection (Pitfall 1 mitigation)**
   - What we know: FUSE mounts are visible via `cat /proc/mounts` (Linux) or `statvfs`; the FUSE FS type is `fuse.reposix`.
   - What's unclear: Should `reposix refresh` auto-detect and refuse, or should it be a documented prerequisite? Should `reposix mount` write a PID file to `.reposix/fuse.pid`?
   - Recommendation: Phase 20 uses the simple approach — check `mount_point.join(".reposix/fuse.pid")` existence. If found and PID is alive, error "unmount before refreshing". `reposix-fuse` writes this file on startup, deletes on clean shutdown.

2. **`--offline` flag behavior in Phase 20**
   - What we know: CONTEXT.md says add `--offline` to both `reposix mount` and `reposix refresh`.
   - What's unclear: For `reposix refresh --offline`, the only sensible behavior is "error: cannot refresh without network". For `reposix mount --offline`, the FUSE read path needs to serve from `.md` files already on disk.
   - Recommendation: Phase 20 adds `--offline` as a flag to both commands. `reposix refresh --offline` returns an error with a clear message. `reposix mount --offline` adds the flag to `MountConfig` but the offline read path (serving from disk files, not backend) is Phase 21 scope.

3. **`_INDEX.md` writing in refresh**
   - What we know: Phase 18 ships `render_bucket_index` and `render_mount_root_index` functions in `fs.rs`.
   - What's unclear: Are those rendering functions exposed as public functions `reposix-cli` can call, or are they private to the FUSE module?
   - Recommendation: Extract `render_bucket_index` and `render_mount_root_index` to `reposix-core` or expose them from `reposix-fuse` as `pub` — so `reposix refresh` can write `_INDEX.md` files with the same content the FUSE daemon would serve. This avoids divergence.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `git` binary | `reposix refresh` git operations | ✓ | 2.25.1 | None — git is a hard requirement |
| `fusermount3` | Mount/unmount lifecycle | ✓ | Present (Ubuntu default) | `fusermount` (v2 alias) |
| `rusqlite` bundled | cache.db | ✓ | 0.32.1 in Cargo.lock | N/A — bundled SQLite |

**Missing dependencies with no fallback:** None.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in (`#[test]` + `#[tokio::test]`) |
| Config file | `Cargo.toml` per crate |
| Quick run command | `cargo test -p reposix-cli` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CACHE-01 | `reposix refresh` subcommand fetches issues and commits to git | integration | `cargo test -p reposix-cli -- refresh` | ❌ Wave A |
| CACHE-02 | Second refresh with no backend changes → empty `git diff HEAD~1` | integration | `cargo test -p reposix-cli -- refresh_idempotent` | ❌ Wave B |
| CACHE-03 | `fetched_at.txt` written with current UTC timestamp | unit | `cargo test -p reposix-cli -- cache_db` | ❌ Wave A |
| (lock) | Concurrent `open_cache_db` → second caller gets `SQLITE_BUSY` error | unit | `cargo test -p reposix-cli -- cache_db_lock_conflict` | ❌ Wave A |

### Sampling Rate
- **Per task commit:** `cargo test -p reposix-cli`
- **Per wave merge:** `cargo test --workspace`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `crates/reposix-cli/src/refresh.rs` — new file
- [ ] `crates/reposix-cli/src/cache_db.rs` — new file
- [ ] No new test framework — existing `#[test]` infrastructure sufficient

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | N/A — refresh uses same backend credentials as `reposix list` |
| V3 Session Management | no | N/A |
| V4 Access Control | yes | `.reposix/cache.db` created with `0o600` permissions; only mount owner can read |
| V5 Input Validation | yes | `fetched_at.txt` content is a formatted `chrono::DateTime<Utc>` — no user input written to it |
| V6 Cryptography | no | No new crypto surface |

### Known Threat Patterns for `reposix refresh`

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Tainted issue body rendered to `.md` file on disk | Information Disclosure | `frontmatter::render` is already the canonical renderer; no new attack surface. The `Tainted<T>` / `Untainted<T>` pattern applies if refresh also writes back to backend — but Phase 20 refresh is read-only (fetch → render → write to disk). |
| Git commit message injection via issue title | Tampering | Commit message is the fixed string `"reposix refresh: <backend>/<project> at <ts>"` — not issue titles. No injection surface. |
| `.reposix/cache.db` readable by other users | Information Disclosure | Create with `0o600` or via `rusqlite::Connection::open` on a file pre-created with `OpenOptions::new().mode(0o600)`. |

---

## Sources

### Primary (HIGH confidence)
- `crates/reposix-fuse/src/fs.rs` — `refresh_issues()`, `ReposixFs` struct, `DashMap<u64, Arc<CachedFile>>`, `frontmatter::render` usage
- `crates/reposix-core/src/audit.rs` — SQLite WAL pattern (`open_audit_db`, `enable_defensive`, schema fixture)
- `crates/reposix-cli/src/main.rs` — existing `Cmd` enum, subcommand dispatch pattern
- `crates/reposix-fuse/src/inode.rs` — inode layout, comment "registry is in-process only (no SQLite persistence — that is Phase S write-path territory)"
- `Cargo.toml` (workspace root) — `rusqlite = { version = "0.32", features = ["bundled"] }` already in workspace deps
- `Cargo.lock` — rusqlite 0.32.1, git2 absent
- `.planning/REQUIREMENTS.md` — CACHE-01, CACHE-02, CACHE-03 requirement text

### Secondary (MEDIUM confidence)
- `.planning/phases/20-op-3-reposix-refresh-subcommand-and-git-diff-cache-for-mount/CONTEXT.md` — design decisions from session 5 discussion
- `HANDOFF.md §OP-3` — original design capture: "~300 LoC in a new `reposix-cache` crate"

### Tertiary (LOW confidence)
- N/A — all major claims verified against codebase

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries verified in workspace Cargo.toml / Cargo.lock
- Architecture: HIGH (existing patterns) / MEDIUM (git invocation — no prior git subprocess in codebase; follows general pattern)
- Pitfalls: HIGH — Pitfall 1 (write-through-FUSE) is a structural constraint verified by reading FUSE write callback routing; Pitfalls 2-5 verified against existing code

**Research date:** 2026-04-15
**Valid until:** 2026-05-15 (stable deps; rusqlite/chrono/tokio versions very stable)
