# v0.11.0 cache location study

**Status:** Research draft. Owner picks A/B/C, then routes to `/gsd-plan-phase`.
**Origin question:** should the cache live INSIDE `/tmp/repo/.git/reposix/cache.db` instead of `~/.cache/`? `rm -rf /tmp/repo` orphans the cache; XDG accumulates dead repos.

---

## Current state

### Where is `cache.db` opened?

Single resolver, single opener.

- **Resolver:** `crates/reposix-cache/src/path.rs::resolve_cache_path(backend, project)` (lines 22-38). Precedence: `REPOSIX_CACHE_DIR` env var → `dirs::cache_dir()` → error. Returns `<root>/reposix/<backend>-<project>.git`.
- **Opener:** `crates/reposix-cache/src/db.rs::open_cache_db(cache_dir)` (lines 32-59). `cache_dir.join("cache.db")`, pre-creates `0o600`, opens via rusqlite, applies DEFENSIVE flag, sets WAL, runs `fixtures/cache_schema.sql`.

The high-level constructor is `crates/reposix-cache/src/cache.rs::Cache::open(backend, backend_name, project)` (lines 54-128):

```rust
let path = resolve_cache_path(&backend_name, &project)?;        // line 61
let mut repo = gix::init_bare(&path)...                          // line 65
ensure_hide_sync_refs(&path)?;                                   // line 102
let db = open_cache_db(&path)?;                                  // line 106 — same dir as the bare repo
```

Comment at line 104: *"cache.db lives inside the bare repo dir so a single path scheme covers both git state and cache state."*

Production callers:
- `crates/reposix-remote/src/main.rs:217` (`ensure_cache`, the only helper-side call site).
- `crates/reposix-cli/src/{gc.rs:66, doctor.rs:270, history.rs:93, tokens.rs:74, refresh.rs:83}` — each opens `<cache_path>/cache.db` directly (gc, doctor go through `open_cache_db`; history, tokens use raw `rusqlite::Connection::open`).

All four worktree-aware CLI subcommands (`gc`, `doctor`, `history`, `tokens`) resolve the cache from a tree by reading `remote.origin.url`, parsing it back into `(backend_slug, project)`, and feeding that to `resolve_cache_path`. See `gc.rs:146-172::cache_path_from_worktree`, mirrored verbatim in `history.rs` and `tokens.rs`.

### What lives in the cache dir besides `cache.db`?

A real bare git repo (`gix::init_bare`). Standard layout: `HEAD`, `config`, `description`, `refs/heads/main` (synthesised), `refs/reposix/sync/<ISO8601>` (private time-travel tags from `crates/reposix-cache/src/sync_tag.rs`), `objects/` (trees, commits, materialized blobs), `packed-refs`. Plus `cache.db` + `cache.db-wal` + `cache.db-shm` (audit_events_cache, meta, oid_map). The `config` is mutated to set `transfer.hideRefs = refs/reposix/sync/` (`cache.rs:320+::ensure_hide_sync_refs`).

The cache dir is **both** a bare git repo **and** a SQLite-flanked audit store. The `.git` suffix is intentional.

### How is the cache dir keyed?

By `(backend_slug, cache_project)`. Backend slug is the URL scheme (`sim`, `github`, `confluence`, `jira`). `cache_project` is `parsed.project` after `sanitize_project_for_cache` (GitHub `owner/repo` → `owner-repo`; others pass through). Identity check at `cache.rs:111-118`: on second open, `meta.identity` must equal `"{backend_name}:{project}"` else `Error::CacheCollision`.

**Implication: two working trees that both `init github::reubenjohn/reposix` SHARE one cache directory.** This is the central design fact for option B.

### How is the cache discovered when the helper runs?

`git-remote-reposix` is execed by git as `argv = [bin, alias, url]` (`main.rs:97-102`). The helper does NOT receive the working tree path or `GIT_DIR` via argv. It instantiates the cache from URL alone: `parse_dispatch_url(url)` → `(backend_slug, project)` → `Cache::open(...)` (line 217-222). Cache dir is a deterministic function of the URL only.

Per `.planning/research/v0.1-fuse-era/git-remote-helper.md:44`: *"`GIT_DIR` — path to the calling repo's `.git` directory. Always set."* Documented git protocol behaviour: git exports `GIT_DIR` when invoking remote helpers. **The helper does not currently read it**, but it is available for option B.

### What do the docs say?

- `docs/guides/troubleshooting.md:96` — *"DB lives at `<XDG_CACHE_HOME>/reposix/<backend>-<project>.git/cache.db`."*
- `docs/reference/simulator.md:54` — same path.
- `docs/tutorials/first-run.md:99` — copy-pastable `sqlite3 ~/.cache/reposix/sim-demo.git/cache.db ...`.
- `docs/concepts/mental-model-in-60-seconds.md:51` — `sqlite3 ~/.reposix/cache.db` (pre-existing typo, wrong even today).
- `docs/how-it-works/time-travel.md:19` — *"tag lives inside the cache's bare repo at `~/.cache/reposix/<backend>-<project>.git`."*

---

## Design alternatives

### A. Status quo: `<XDG_CACHE_HOME>/reposix/<scheme>-<project>.git/`

**Pro:**
- Already shipped in v0.10.0; zero migration cost.
- One cache shared across many working trees of the same project — single audit log, bandwidth amortised across clones (matters for GitHub rate limits, Confluence/JIRA throttles).
- `reposix gc --strategy ttl` operates on the canonical cache regardless of tree state. `reposix history` works after `rm -rf <worktree>` because the cache outlives the tree.
- Stable resolver path independent of any single tree.

**Con (the owner's complaint, restated precisely):**
- `rm -rf /tmp/repo` orphans the cache. There is no GC-of-caches today; `reposix gc` evicts blobs *within* one cache, not whole caches.
- XDG cache accumulates `<scheme>-<project>.git` dirs over many init/destroy cycles.
- Cache lifetime decoupled from working tree lifetime — surprising for a tool whose pitch is "it's just git."

### B. Co-located: `<worktree>/.git/reposix/cache.git/`

**Pro:**
- `rm -rf <worktree>` is sufficient cleanup. No XDG accumulation.
- Lifetime model matches `git clone` intuition exactly.
- `git push` does not push `.git/`, so cache contents (issue bodies) cannot egress via accidental `git push origin somewhere-else`. Confirmed: git only ships objects reachable from refs being pushed; `.git/reposix/` is private metadata never traversed.
- Per-tree caches mean one tree's compromised `cache.db` doesn't disclose cross-tree audit history.

**Con — discovery:**
- Helper must read `GIT_DIR` (small change in `main.rs::real_main`), but `Cache::open` no longer takes only `(backend, project)` — needs a path or env-aware factory. Touches every test using `Cache::open` (~6 files) plus four CLI subcommands.
- Worktree-resolved CLI subcommands (`history`, `tokens`, `gc`, `doctor`) get *simpler* — no XDG fallback needed.

**Con — multi-worktree (the load-bearing concern):**
- Two clones of the same project = two independent caches, two fetches, two audit logs. For sim/demo fine. For a real GitHub repo: doubles rate-limit footprint, contradicts the audit-as-system-of-record principle (one project = one truth).
- **For a dark-factory spawning N parallel workspaces of one project, B is actively bad** — N caches, N× egress. v0.11.0 §3d (multi-project helper daemon) explicitly assumes shared cache.

**Con — bare repo lifetime:**
- A bare git repo nested inside another git repo's `.git/` is unusual but workable; `.git/reposix/cache.git/` is the natural layout. `transfer.hideRefs` writes target the inner repo's own `config`, no conflict with the outer.

**Con — sharing forecloses:**
- Future `git clone --reference`-style "two trees share one objects pool" requires the cache outlive any single tree. B forecloses without a `--reference` flag.

### C. Hybrid: pointer file in `.git/`, blob storage in XDG

`.git/reposix/cache-pointer` contains a path like `~/.cache/reposix/github-reubenjohn-reposix.git/`. Created by `reposix init`, consulted by helper via `$GIT_DIR/reposix/cache-pointer`.

**Pro:**
- Multi-clone sharing preserved (multiple pointers → same XDG cache).
- `rm -rf <worktree>` removes the pointer; `reposix gc --orphans` walks XDG and deletes caches with no live pointers (refcount-via-filesystem).
- Lifetime owned by explicit GC, not side effects.

**Con:**
- More moving parts: pointer format, validation, orphan scan, refcount semantics.
- Pointer-divergence bugs: stale pointer → confusion.
- "Just git" mental model erodes — there's now a reposix-private file in `.git/` that has to stay in sync.

---

## Recommendation

**Stay on A and add `reposix gc --orphans`.**

Justification:

1. **Multi-clone amortisation is load-bearing for the dark-factory pattern.** v0.11.0 §3d (multi-project helper daemon), §3c (token-cost ledger), §3b (time-travel via tags) all assume a stable per-`(backend, project)` cache, not per-tree. B works against this; C complicates it.
2. **The owner's pain is narrow and locally fixable.** "XDG accumulates dead repos" is hygiene, solvable by ONE new subcommand: `reposix gc --orphans` walks `<root>/reposix/*.git/`, opens each `cache.db`, reads a `meta.last_used_at` row, evicts entries older than N days (or unreachable from any live tree the user lists). One day of work.
3. **`rm -rf <worktree>` is not the canonical "I'm done" gesture.** Demo trees in `/tmp/` get wiped on reboot anyway. Long-term projects often want the cache to survive `rm -rf .git && git init` — option B does not.
4. **Privacy under A is acceptable.** XDG cache is `0o600`; same-user processes already read it. B doesn't materially improve threat model.
5. **Migration cost.** A is zero; B is a non-trivial helper signature change + doc rewrite + tutorial-runner update; C is B's cost plus pointer-file design.

The owner's instinct ("`rm -rf` should suffice") is right *as a UX pole star*, but the right tool is `gc --orphans`, not relocation. Keeps amortisation, kills the accumulation grievance.

---

## Migration plan (only if recommendation flips to B)

### Code changes

- **`crates/reposix-cache/src/path.rs`** — add `resolve_cache_path_from_git_dir(git_dir) -> PathBuf` returning `git_dir.join("reposix/cache.git")`. Keep `resolve_cache_path` for orphan-GC.
- **`crates/reposix-cache/src/cache.rs`** — `Cache::open` takes `cache_dir: PathBuf` directly; resolution moves to callers. Or add `Cache::open_at(cache_dir, ...)`; deprecate the old constructor.
- **`crates/reposix-remote/src/main.rs`** — `real_main` reads `GIT_DIR`, computes `cache_dir = git_dir/reposix/cache.git`, passes to `Cache::open`. Bail with clear error if `GIT_DIR` unset.
- **`crates/reposix-cli/src/{gc,doctor,history,tokens}.rs`** — replace `resolve_cache_path(backend, project)` with `resolve_cache_path_from_git_dir(work.join(".git"))`. Worktree-derived `(backend, project)` becomes informational only (audit identity check).
- **Test updates** — every `Cache::open(backend, "sim", "proj-1")` in `crates/reposix-cache/tests/*` becomes `Cache::open_at(tmpdir, ...)`. The `REPOSIX_CACHE_DIR` env-var dance in `tests/common/mod.rs` and `tests/history.rs` largely goes away.

### Backward compat

Pre-1.0; **do not** support both locations during transition. Release note: *"v0.11.0 moves caches into `<worktree>/.git/reposix/`. Run `reposix init` to recreate; old caches under `~/.cache/reposix/` can be deleted."* Add a `reposix doctor` finding that detects an old XDG cache for the current project and offers `--migrate`.

### Tutorial / doc updates

- `docs/tutorials/first-run.md:99` — `sqlite3 /tmp/reposix-demo/.git/reposix/cache.git/cache.db ...`
- `docs/guides/troubleshooting.md:96`, `docs/reference/simulator.md:54`, `docs/how-it-works/time-travel.md:19` — rewrite paths.
- `docs/concepts/mental-model-in-60-seconds.md:51` — fix pre-existing `~/.reposix/cache.db` typo at the same time.
- `docs/how-it-works/filesystem-layer.md` — diagram (cache box now inside `.git/`).
- `scripts/tutorial-runner.sh` — verify new path.
- `reposix-banned-words` skill — add `~/.cache/reposix` so docs don't regress.

### Tests

- New: `crates/reposix-cli/tests/co_located_cache.rs` — assert `Cache::open` writes inside `.git/reposix/cache.git/`.
- New: `crates/reposix-remote/tests/git_dir_required.rs` — helper bails clean when `GIT_DIR` unset.
- New: `tests/cleanup_via_rm_rf.rs` — init + populate + `rm -rf` worktree, assert no orphan files survive.
- Modified: drop env-var dance from ~12 test files.

### Estimated effort

**3-4 working days** for B end-to-end. **~1 day** for `gc --orphans` under A.

**Author recommendation reaffirmed: ship `gc --orphans` (1 day) under A. If a reasoned case for per-tree caches emerges later (e.g. multi-tenant CI), revisit B.**
