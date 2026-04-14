# Phase 15 CONTEXT — Dynamic `_INDEX.md` synthesized in FUSE bucket directory (OP-2 partial)

> Status: scoped in session 5, 2026-04-14 ~10:20 PDT.
> Author: Claude Opus 4.6 1M.
> Follows Phase 14 (v0.4.1, tagged + pushed).

## Phase identity

**Name:** Dynamic `_INDEX.md` synthesized in FUSE bucket directory (OP-2 partial).

**Scope tag:** v0.5.0 (feature scope — adds a user-visible synthesized file at `mount/pages/_INDEX.md` / `mount/issues/_INDEX.md`).

**Addresses:** OP-2 from v0.3-era HANDOFF.md. Partial scope — ships the bucket-dir level only, not the recursive `tree/` level. `tree/INDEX.md` is deferred to a follow-up phase because it adds cycle-safe recursive synthesis complexity the bucket-level doesn't need.

## Goal (one paragraph)

Add a synthesized `_INDEX.md` read-only file to the mount bucket directory (`mount/pages/` for Confluence, `mount/issues/` for sim + GitHub). Agents running `cat mount/pages/_INDEX.md` get a single-shot markdown summary of every tracked issue/page — id, title, status, updated_at — without needing a separate `ls` + N `stat`s. The file is computed at read time from the same `IssueBackend::list_issues` cache that backs `readdir`. It shows up in `ls` output (by design, so agents can discover it), but its leading underscore keeps it out of naive `*.md` glob patterns. Not included in `tree/` yet; tree-level synthesis is a follow-up.

## Success criteria

- **SC-15-01:** `cat mount/<bucket>/_INDEX.md` returns a valid markdown document when the mount is populated (≥1 issue/page present). Well-formed YAML frontmatter. Table body with one row per issue.
- **SC-15-02:** `ls mount/<bucket>/` includes `_INDEX.md` alongside the real `<padded-id>.md` files. The synthesized entry renders with a distinct inode (not allocated by `InodeRegistry`).
- **SC-15-03:** `_INDEX.md` is READ-ONLY — writes via the FUSE `write` / `setattr` / `unlink` / `create` callbacks on this inode surface `EROFS` (or `EACCES`, whichever matches the Phase-13 read-only discipline for synthesized nodes). A user cannot `echo > mount/<bucket>/_INDEX.md` or `rm mount/<bucket>/_INDEX.md`.
- **SC-15-04:** Generation is stable — successive `cat` calls produce byte-identical output within a single mount session (because the issue list is cached). A mount restart may change the output (new issues seeded, etc.) — that's OK.
- **SC-15-05:** `git ls-files mount/<bucket>/` (from inside the FUSE mount) tracks `_INDEX.md` as a regular file. It does NOT show up in `.gitignore`. This is deliberate — the index is a legitimate part of the checkout so `git diff` can show changes over time when paired with OP-3's cache-refresh future work.
- **SC-15-06:** Naive `*.md` glob patterns (`ls mount/<bucket>/*.md`, `grep -l foo mount/<bucket>/*.md`) do NOT match `_INDEX.md`. Proof: the leading `_` sorts and globs separately in zsh/bash by convention; `.md`-with-underscore only matches `_*.md` or `*_*.md` patterns.
- **SC-15-07:** `cargo test --workspace --locked` green (274+2 expected; 2 new tests for the index file).
- **SC-15-08:** `cargo clippy --workspace --all-targets --locked -- -D warnings` clean.
- **SC-15-09:** `bash scripts/green-gauntlet.sh --full` green. Smoke 4/4 unchanged.
- **SC-15-10:** Docs updated: README.md Quickstart mentions `_INDEX.md` as a discoverable file; `docs/architecture.md` or `docs/reference/` mentions the synthesis pattern; CHANGELOG `[Unreleased]` (or `[v0.5.0]`) has an `### Added` entry.

## Locked decisions

- **LD-15-01 — Filename is `_INDEX.md` (leading underscore).** Keeps it out of `*.md` globs (SC-15-06). Visible in `ls`. Capital `INDEX` consistent with conventional `README.md` / `LICENSE` naming.
- **LD-15-02 — YAML frontmatter + markdown table.** Frontmatter carries metadata agents may want programmatically (`backend`, `project`, `issue_count`, `generated_at`). Body is a pipe-table with columns: `id | status | title | updated` (so lexical sort by row keeps numeric IDs together).
- **LD-15-03 — Non-recursive.** Only lists direct children of the bucket directory (the real `<padded-id>.md` files). Does NOT descend into `tree/` or anywhere else. Recursive indexing is deferred.
- **LD-15-04 — Regenerated on every read.** No cache separate from the existing issue-list cache. `read` callback calls `render_index` against the current snapshot. If the snapshot is stale, the index is stale (same invariant as the rest of the mount today).
- **LD-15-05 — Read-only.** No `write`, no `unlink`, no `create` with target name `_INDEX.md`. The FUSE `create` callback must explicitly reject that filename before the inode allocation step, else the user could shadow the synthetic entry with a real file.
- **LD-15-06 — Only in bucket dir.** `mount/pages/_INDEX.md` or `mount/issues/_INDEX.md`. NOT `mount/_INDEX.md` (mount root), NOT `mount/tree/_INDEX.md` (tree overlay), NOT `mount/tree/<subdir>/_INDEX.md`.
- **LD-15-07 — Fixed inode.** Use `BUCKET_INDEX_INO = 5`, which is in the reserved synthetic-file range `5..=0xFFFF` per `crates/reposix-fuse/src/inode.rs`. No DashMap, no `InodeRegistry` entry.
- **LD-15-08 — Size is truthful.** The `FileAttr.size` returned by `getattr` reflects the actual rendered byte length — not a placeholder. `read` uses a cached `Arc<Vec<u8>>` rendered lazily on first read and cleared on the next `backend.list_issues` refresh (same refresh cadence as the bucket readdir uses).
- **LD-15-09 — No backend-capability gating.** Every backend that presents a bucket (today: sim, github, confluence) gets `_INDEX.md`. No new `BackendFeature` flag.
- **LD-15-10 — Deterministic row order.** Sort by numeric `id` ascending. Matches the sim's `ORDER BY id ASC` and Confluence's per-space page-id ordering, so the table reads top-to-bottom in the same order as `ls <bucket>/`.

## Non-goals / scope boundaries

- Do NOT synthesize `_INDEX.md` inside `tree/` or its subdirectories.
- Do NOT synthesize at the mount root.
- Do NOT implement a `reposix refresh` subcommand (that's OP-3, separate phase).
- Do NOT add per-directory indexes for a recursive spaces/ or labels/ layout (those are OP-1 follow-ups, not OP-2 partial).
- Do NOT add a `--index-style=yaml|table|both` flag. Ship the YAML-frontmatter + table combo as the default; users can file issues if they want other shapes.
- Do NOT cache `_INDEX.md` content to disk. In-memory only.

## Canonical refs

- `crates/reposix-fuse/src/inode.rs:40-58` — inode layout; `BUCKET_DIR_INO = 2`, reserved range `5..=0xFFFF`.
- `crates/reposix-fuse/src/fs.rs` — bucket dir callbacks (lookup, readdir, getattr, read). Model the new `_INDEX.md` entry after how `.gitignore` is rendered at the mount root (inode 4) — that's the cleanest precedent in the code for a synthesized read-only file.
- `crates/reposix-core/src/issue.rs` — `Issue` struct; pulls `id`, `title`, `status`, `updated_at`.
- `.planning/SESSION-5-RATIONALE.md` — session-5 cluster pick; includes OP-2 as a stretch candidate.
- HANDOFF.md OP-2 entry — original design questions (symlinks vs hardlinks — not relevant here; table vs frontmatter — this phase resolves to both; recursive — deferred; visible in ls — YES per LD-15-01).

## Waves

- **Wave A (serial, ~30 min):** Implement `_INDEX.md` synthesis + inode dispatch + tests. Single wave, single agent. Touches `crates/reposix-fuse/src/` only.
- **Wave B (serial, ~15 min):** Docs + CHANGELOG.

Total est. wall-clock: ~45 min.
