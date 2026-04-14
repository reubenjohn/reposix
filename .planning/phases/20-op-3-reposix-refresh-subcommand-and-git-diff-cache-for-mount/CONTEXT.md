# Phase 20 CONTEXT — `reposix refresh` subcommand and git-diff cache for mount (OP-3)

> Status: scoped in session 5, 2026-04-14.
> Author: planning agent, session 6 prep.
> Follows Phase 18 (OP-2 remainder — tree-recursive + mount-root _INDEX.md).

## Phase identity

**Name:** `reposix refresh` subcommand and git-diff cache for mount (OP-3).

**Scope tag:** v0.6.0 (new crate + subcommand — `reposix-cache` + `reposix refresh`).

**Addresses:** OP-3 from HANDOFF.md. Full scope — persistent SQLite cache, `reposix refresh` subcommand, git-commit-into-mount helper, mount-as-time-machine UX.

**Depends on:** Phase 18 (OP-2 remainder) — tree-recursive `_INDEX.md` is the obvious sync anchor; `git diff _INDEX.md` across pulls is the primary user-visible signal that motivates OP-3.

## Goal (one paragraph)

Today the FUSE mount is live-on-every-read with an in-memory cache that evaporates on unmount. OP-3 makes the mount persistent and diff-able: a new `crates/reposix-cache` crate stores a SQLite WAL database at `mount/.reposix/cache.db`; a new `reposix refresh` CLI subcommand re-fetches all pages from the backend and writes a git commit into the mount's own working tree; `git log` inside the mount shows the history of backend sync points; `git diff HEAD~1` shows what changed at the backend since the last pull. The `_INDEX.md` file (shipped in Phase 15, extended in Phase 18) is the natural sync anchor — it changes on every refresh that adds, removes, or updates any issue/page — making `git diff _INDEX.md` the one-line agent command for "what changed since last sync."

## Source design context

From HANDOFF.md §OP-3 (verbatim):

> Today's mount is **live-on-every-read** — each `cat` may fire an HTTP call (first read populates the cache; re-reads hit the cache until the mount exits). That's fine for accuracy but wrong for the user's mental model.
>
> The user's insight: **the mount point is already a git repo.** The natural refresh semantic is `git pull`. Proposal:
> - `mount/.reposix/cache.db` (sqlite) — persistent content cache.
> - `mount/.reposix/fetched_at.txt` — timestamp of the last backend round-trip.
> - `git pull` in the mount triggers a hook that calls a new `reposix refresh` subcommand → it re-fetches all pages + writes a git commit into the mount's own working tree.
> - `git log` in the mount shows the history of backend sync points. The mount becomes a **time machine** over the backend.
> - `git diff HEAD~1` shows "what changed at the backend since the last pull." That is an insanely good agent UX.
>
> **Primary tech spike:** SQLite with WAL mode + a tiny commit-into-mount helper. A working prototype would be ~300 LoC in a new `crates/reposix-cache` crate.

From session-5 rollup:

> **OP-3 — cache refresh via `git pull` semantics.** Not started. Now that `_INDEX.md` is the obvious sync anchor (`git diff _INDEX.md` across pulls shows what changed), the ROI is higher — `mount-as-time-machine` gets concrete.

## Design questions

These must be resolved (via `/gsd-discuss-phase 20`) before planning:

1. **Where does the cache live?** `.reposix/` hidden inside the mount (stays inside the git working tree — `git log` works natively) vs a sibling `runtime/<tenant>-<space>.db` out-of-tree (no git pollution, but needs a custom `reposix log` viewer). Recommendation: `.reposix/cache.db` inside the mount, with `.reposix/` added to `.gitignore` at the cache-DB level but `cache.db` committed as a blob artifact on each refresh commit.
2. **Is the cache a git-tracked artifact?** If the DB itself is committed as a binary blob, `git log` works without a helper. If only the rendered `.md` files are committed (and the DB is gitignored), the mount still looks like a time machine but the DB is ephemeral. Pick one.
3. **Commit author.** `reposix <backend>@<tenant>` so human vs agent commits are distinguishable in `git log`.
4. **Concurrent mount safety.** Two `reposix mount` processes on the same path, or two `reposix refresh` calls racing, need a file lock on `.reposix/cache.db` (SQLite WAL advisory lock or an explicit `.reposix/cache.lock` flock). Define the failure mode: error-and-exit or block-and-wait.
5. **Offline mode.** If the backend is down, the cache is authoritative. Add a `--offline` CLI flag to `reposix mount` (and `reposix refresh`) that guarantees zero egress. FUSE reads serve from cache.db only.
6. **Invalidation vs extend.** `git pull --force` vs `git pull --rebase` have different reposix equivalents. For v0.6.0, one refresh mode is enough (`--force` semantics — overwrite the cache with the current backend state). `--rebase` (merge server changes with local edits) is a future phase.

## Canonical refs

- `.planning/phases/15-dynamic-index-md-synthesized-in-fuse-bucket-directory-op-2-p/15-CONTEXT.md` — `_INDEX.md` is the sync anchor this phase builds on.
- `.planning/phases/18-op-2-remainder-tree-recursive-and-mount-root-index-md-synthe/CONTEXT.md` — Phase 18 extends `_INDEX.md` to tree-level; Phase 20 depends on that being done first.
- `HANDOFF.md §OP-3` — original design capture.
- `crates/reposix-core/src/issue.rs` — `IssueBackend` trait; refresh calls `list_issues` + `fetch_issue` per id.
- `crates/reposix-fuse/src/fs.rs` — current in-memory cache (`Arc<Vec<Issue>>`); Phase 20 replaces with cache.db reads.
- Tech stack: `rusqlite` 0.32 with `bundled` feature (already in workspace) — no new system dep for SQLite.
