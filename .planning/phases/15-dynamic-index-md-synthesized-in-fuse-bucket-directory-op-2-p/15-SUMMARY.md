---
phase: 15
slug: dynamic-index-md-synthesized-in-fuse-bucket-directory-op-2-p
name: "Dynamic _INDEX.md synthesized in FUSE bucket directory (OP-2 partial)"
shipped: 2026-04-14
scope_tag: v0.5.0
status: SHIPPED
waves:
  - A: _INDEX.md synthesis + inode dispatch + tests (6a2e256, a94e970, 3309d4c)
  - B: CHANGELOG [v0.5.0] + workspace version bump + SUMMARY + STATE cursor + tag script (this wave)
closes_gaps:
  - HANDOFF.md OP-2 (partial) — synthesized `_INDEX.md` at bucket-dir level
  - Recursive `tree/_INDEX.md`, mount-root `_INDEX.md`, and OP-3 cache-refresh
    integration remain open as follow-ups.
tests:
  workspace_passing: 278
  workspace_failing: 0
  delta_vs_phase_14: +4
  ignored: 11
notable_files_added:
  - crates/reposix-fuse/src/fs.rs (render_bucket_index + BucketIndex inode dispatch + 4 new unit tests)
  - crates/reposix-fuse/src/inode.rs (BUCKET_INDEX_INO = 5 const + doc update)
  - scripts/dev/test-bucket-index.sh (live FUSE proof script)
  - CHANGELOG.md [v0.5.0] entry
  - README.md Folder-structure section mentions `_INDEX.md`
  - scripts/tag-v0.5.0.sh
---

# Phase 15 SUMMARY

> Shipped: 2026-04-14. Scope tag: v0.5.0.

## tl;dr

Added a synthesized read-only `_INDEX.md` file to the FUSE mount's bucket
directory (`mount/issues/_INDEX.md` for sim + GitHub,
`mount/pages/_INDEX.md` for Confluence). Agents `cat`-ing the file get a
single-shot markdown sitemap of every tracked issue/page without a
separate `ls` + N `stat`s: YAML frontmatter (`backend`, `project`,
`issue_count`, `generated_at`) plus a pipe-table with columns
`id | status | title | updated`, sorted ascending by numeric `id`.
Content is generated lazily on first read from the same in-memory
issue-list cache that backs `readdir`, and invalidated whenever that
cache refreshes. The file is strictly read-only: `touch`, `echo >`,
`rm`, `setattr`, and `create` with target name `_INDEX.md` all surface
`EROFS`/`EACCES`. The leading underscore keeps it out of naive `*.md`
globs (`ls mount/<bucket>/*.md` skips it) while remaining visible in
`ls`. Closes OP-2 from the v0.3-era HANDOFF.md at the bucket-dir level;
recursive `tree/_INDEX.md`, mount-root `_INDEX.md`, and OP-3
cache-refresh integration remain open.

## What shipped

**Wave A (Phase 15-A — 3 commits on main):**

- `6a2e256` — **`feat(15-A): reserve BUCKET_INDEX_INO=5 for synthesized
  bucket index`**. `crates/reposix-fuse/src/inode.rs` gains a
  `BUCKET_INDEX_INO: u64 = 5` const (with doc-comment citing LD-15-07).
  The inode-layout module doc at the top of `inode.rs` gains a row for
  `5 → bucket _INDEX.md`. The `reserved_range_is_unmapped` test is
  narrowed from `5..=0xFFFF` to `6..=0xFFFF` so inode 5 is now
  specifically accounted for.
- `a94e970` — **`feat(15-A): synthesize _INDEX.md in FUSE bucket dir
  (OP-2 partial)`**. `crates/reposix-fuse/src/fs.rs` gains a new
  `InodeKind::BucketIndex` variant, `render_bucket_index` pure
  function, `bucket_index_bytes: RwLock<Option<Arc<Vec<u8>>>>` cache
  field on `ReposixFs`, and dispatch for `lookup`, `readdir`,
  `getattr`, `read`, `create`, `unlink`, `write`, `setattr`, `release`
  on the new synthetic inode. +4 unit tests in the `fs::tests` module.
- `3309d4c` — **`chore(15-A): scripts/dev/test-bucket-index.sh live
  proof`**. An end-to-end bash script that starts `reposix-sim`, mounts
  FUSE, lists the bucket, cats `_INDEX.md`, attempts
  `touch`/`rm`/`echo >`, asserts every write path errors, and unmounts
  cleanly. Produces the transcript pasted in the "Live verification
  transcript" section below.

**Wave B (Phase 15-B — this wave):**

- `docs(15-B): CHANGELOG [v0.5.0] + workspace version bump` — promotes
  the empty `[Unreleased]` block to `[v0.5.0] — 2026-04-14` with an
  `### Added` entry for `_INDEX.md`. Also fills in the compare links
  for `v0.5.0` and `v0.4.1` at the bottom of CHANGELOG.md (the latter
  had been missed in the Phase-14 close-out). Bumps workspace version
  `0.4.1 → 0.5.0` and regenerates Cargo.lock via `cargo check
  --workspace --offline`. README's Folder-structure section now
  mentions `_INDEX.md` with a short illustrative example.
- `docs(15-B): Phase 15 SUMMARY.md + STATE.md cursor` — this file plus
  STATE.md cursor advance.
- `chore(15-B): scripts/tag-v0.5.0.sh` — clone of `tag-v0.4.1.sh`
  adapted for v0.5.0. Not invoked by the executor; the orchestrator
  runs it after `green-gauntlet --full` green.

### Render shape (locked by LD-15-02, verified in tests)

```markdown
---
backend: simulator
project: demo
issue_count: 6
generated_at: 2026-04-14T18:21:44Z
---

# Index of issues/ — demo (6 issues)

| id | status | title | updated |
| --- | --- | --- | --- |
| 1 | open | database connection drops under load | 2026-04-13 |
| 2 | in_progress | add `--no-color` flag to CLI | 2026-04-13 |
| 3 | in_review | document the new auth flow | 2026-04-13 |
| 4 | open | flaky integration test on CI | 2026-04-13 |
| 5 | in_progress | document the audit-log schema | 2026-04-13 |
| 6 | open | investigate p99 latency spike on PATCH | 2026-04-13 |
```

Header formulation: `# Index of <bucket>/ — <project> (<N> <rows>)`.
`<rows>` is `"pages"` if the backend renders pages (Confluence),
`"issues"` otherwise (sim, GitHub). `updated` is the `YYYY-MM-DD`
prefix of `Issue::updated_at` (time dropped — doesn't fit the table
and one-day resolution is plenty for a sitemap). `|` inside titles is
escaped to `\|`; embedded newlines fold to spaces so the pipe-table
column count is preserved.

### Test count delta

v0.4.1 baseline: 274 workspace tests. v0.5.0: **278 workspace tests
(+4)**. Ignored: 11 (unchanged).

**+4 new unit tests in `crates/reposix-fuse/src/fs.rs` (inside the
`#[cfg(test)] mod tests` block):**

- `bucket_index_renders_frontmatter_and_table` — feeds two sample
  issues to `render_bucket_index`, asserts the output contains all
  four frontmatter keys (`backend`, `project`, `issue_count`,
  `generated_at`), the markdown header, the table header + separator,
  and both issue rows.
- `bucket_index_row_order_is_ascending_by_id` — feeds three issues in
  reverse `id` order, asserts the rendered table sorts ascending (pins
  LD-15-10). Also pins the Confluence-flavoured header
  `# Index of pages/ — REPOSIX (3 pages)`.
- `bucket_index_empty_list_is_valid_markdown` — feeds an empty slice,
  asserts the output still has `issue_count: 0`, a zero-count header,
  and a valid table header+separator with zero data rows.
- `bucket_index_escapes_pipe_in_title` — feeds a title containing `|`
  and embedded newline, asserts the `|` renders as `\|` and the
  newline folds to a space.

**Assertions updated (0 net count change) in 2 existing tests:** the
`inode.rs` `reserved_range_is_unmapped` test iterates `6..=0xFFFF`
instead of `5..=0xFFFF` (inode 5 is no longer unmapped — it's the
bucket index). The existing readdir count assertions in
`crates/reposix-fuse/tests/` that expected N entries for N issues were
updated to expect N+1 (accounting for the new `_INDEX.md` entry).

Full workspace `cargo test --workspace --offline` run-time: <15s.

## Live verification transcript

From `scripts/dev/test-bucket-index.sh` (simulator backend, 6 seeded
issues):

```
[1/4] start sim + mount FUSE
ready: sim=http://127.0.0.1:7866 mount=/tmp/reposix-15-bucket-index-mnt

[2/4] ls /tmp/reposix-15-bucket-index-mnt/issues/
00000000001.md  00000000002.md  00000000003.md  00000000004.md
00000000005.md  00000000006.md  _INDEX.md

[3/4] cat /tmp/reposix-15-bucket-index-mnt/issues/_INDEX.md
---
backend: simulator
project: demo
issue_count: 6
generated_at: 2026-04-14T18:21:44Z
---

# Index of issues/ — demo (6 issues)

| id | status | title | updated |
| --- | --- | --- | --- |
| 1 | open | database connection drops under load | 2026-04-13 |
| 2 | in_progress | add `--no-color` flag to CLI | 2026-04-13 |
| 3 | in_review | document the new auth flow | 2026-04-13 |
| 4 | open | flaky integration test on CI | 2026-04-13 |
| 5 | in_progress | document the audit-log schema | 2026-04-13 |
| 6 | open | investigate p99 latency spike on PATCH | 2026-04-13 |

[4/4] verify _INDEX.md is read-only
== BUCKET INDEX PROOF OK ==
```

`[4/4]` confirms that `touch _INDEX.md`, `rm _INDEX.md`, and
`echo >_INDEX.md` all surface `EROFS`/`EACCES` and that the mount
unmounts cleanly. The exact `generated_at` timestamp will vary per run
(LD-15-04 — the snapshot is cached, so successive `cat`s within one
mount session produce byte-identical output; a restart may change it).

## Decisions locked

Re-summarised from `15-CONTEXT.md`; these were pinned before Wave A and
all held through execution.

- **LD-15-01** — Filename is `_INDEX.md` (leading underscore). Keeps it
  out of `*.md` globs (SC-15-06). Visible in `ls`. Capital `INDEX`
  consistent with `README.md` / `LICENSE` naming.
- **LD-15-02** — YAML frontmatter + markdown table. Frontmatter
  carries machine-readable metadata (`backend`, `project`,
  `issue_count`, `generated_at`). Body is a pipe-table with columns
  `id | status | title | updated`.
- **LD-15-03** — Non-recursive. Lists direct children of the bucket
  only; does NOT descend into `tree/` or anywhere else.
- **LD-15-04** — Regenerated on every read. Cache is tied to the
  existing issue-list snapshot; if that snapshot is stale, so is the
  index. No separate refresh cadence.
- **LD-15-05** — Read-only. `write`, `unlink`, `setattr`, `release`,
  `create(BUCKET_DIR_INO, "_INDEX.md")` all return `EROFS`/`EACCES`.
  The `create` rejection fires before inode allocation so users cannot
  shadow the synthetic entry with a real file.
- **LD-15-06** — Only in bucket dir. `mount/pages/_INDEX.md` or
  `mount/issues/_INDEX.md`. NOT the mount root, NOT inside `tree/`.
- **LD-15-07** — Fixed inode. `BUCKET_INDEX_INO = 5`, in the reserved
  synthetic-file range `5..=0xFFFF` per `crates/reposix-fuse/src/inode.rs`.
  No `InodeRegistry` entry; no `DashMap` allocation.
- **LD-15-08** — Size is truthful. `FileAttr.size` from `getattr`
  reflects the actual rendered byte length (computed lazily on first
  touch via `bucket_index_bytes_or_render`), not a placeholder.
- **LD-15-09** — No backend-capability gating. Every backend that
  presents a bucket (sim, github, confluence today) gets `_INDEX.md`.
  No new `BackendFeature` flag.
- **LD-15-10** — Deterministic row order. Sort ascending by numeric
  `id`. Matches the sim's `ORDER BY id ASC` and Confluence's per-space
  page-id ordering, so the table reads top-to-bottom in the same order
  as `ls <bucket>/`.

## Follow-ups

None are blockers for v0.5.0; all are candidate work for a future phase.

- **Tree-level `_INDEX.md` (recursive).** `mount/tree/_INDEX.md` and
  `mount/tree/<subdir>/_INDEX.md` — per-directory recursive index
  inside the `tree/` symlink overlay. Adds cycle-safe recursive
  synthesis complexity the bucket-level path deliberately doesn't
  need. Deferred per LD-15-03 / LD-15-06.
- **Mount-root `_INDEX.md`.** A single top-level sitemap at
  `mount/_INDEX.md` listing every bucket (and eventually every tree
  subdir). Useful for multi-backend mounts if that ever lands; today
  every mount has exactly one bucket.
- **OP-3 cache-refresh integration.** A `reposix refresh` subcommand
  that forces a re-fetch of the issue-list cache so the mount's
  `_INDEX.md` (and the rest of the mount) can pick up server-side
  changes without an unmount + remount. Phase 15 deliberately does NOT
  wire `_INDEX.md` to any refresh trigger other than the existing
  one; the file is stale iff the cache is stale.
- **User-configurable column set.** Today the table pins `id | status |
  title | updated`. Callers who want labels, assignee, or updated-time
  precision would need a future `--index-style=…` knob. Not in scope
  per the "do NOT add a `--index-style` flag" constraint in the Phase
  15 CONTEXT.
- **Bucket-index content inside `git diff`.** `_INDEX.md` IS a regular
  file from the git-remote helper's perspective (SC-15-05 — tracked,
  not gitignored), so a future `git diff` after an OP-3 refresh would
  show it as a changed file. The round-trip semantics are unspecified
  today; a follow-up should either gitignore `_INDEX.md` or document
  "changes to `_INDEX.md` are derived, not part of any PATCH" before
  a user commits the file into a real tracker.

## Success criteria — outcome

| ID       | Criterion                                                              | Status |
|----------|------------------------------------------------------------------------|--------|
| SC-15-01 | `cat mount/<bucket>/_INDEX.md` returns valid markdown                  | PASS   |
| SC-15-02 | `ls mount/<bucket>/` includes `_INDEX.md` with distinct synthetic inode | PASS  |
| SC-15-03 | All write paths on `BUCKET_INDEX_INO` surface `EROFS`/`EACCES`         | PASS   |
| SC-15-04 | Successive `cat` produces byte-identical output within one session     | PASS   |
| SC-15-05 | `_INDEX.md` tracked by git (not in `.gitignore`)                       | PASS   |
| SC-15-06 | Naive `*.md` globs skip `_INDEX.md` due to leading underscore          | PASS   |
| SC-15-07 | `cargo test --workspace --locked` green (278 passing, +4)              | PASS   |
| SC-15-08 | `cargo clippy --workspace --all-targets --locked -- -D warnings` clean | PASS   |
| SC-15-09 | `scripts/green-gauntlet.sh --full` — gated on orchestrator             | PEND\* |
| SC-15-10 | CHANGELOG `[v0.5.0]` + README mention of `_INDEX.md`                   | PASS   |

\* SC-15-09 is run by the orchestrator before tag push; not blocking on
Wave B (docs are additive and non-code).

## Self-Check: PASSED

- `crates/reposix-fuse/src/inode.rs` — FOUND; `BUCKET_INDEX_INO` const
  present.
- `crates/reposix-fuse/src/fs.rs` — FOUND; `render_bucket_index`
  function + 4 `bucket_index_*` unit tests present.
- `scripts/dev/test-bucket-index.sh` — FOUND.
- CHANGELOG.md `[v0.5.0]` section — FOUND (added this wave).
- Cargo.toml `version = "0.5.0"` — FOUND (added this wave).
- README.md `_INDEX.md` mention in Folder-structure section — FOUND
  (added this wave).
- Commits `6a2e256`, `a94e970`, `3309d4c` (Wave A) — FOUND in `git log
  --oneline`. Wave B commit hashes recorded in the Session Continuity
  section of STATE.md after this wave finishes.
