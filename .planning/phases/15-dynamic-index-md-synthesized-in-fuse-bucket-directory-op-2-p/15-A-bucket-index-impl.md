# Phase 15 — Wave A — `_INDEX.md` synthesis in the FUSE bucket dir

> Wave A of Phase 15. Serial, single agent. ~30 min wall-clock.
> Depends on: none (Phase 14 refactor is complete; this phase is additive).

## Files to touch

- **`crates/reposix-fuse/src/inode.rs`** — add `BUCKET_INDEX_INO: u64 = 5` const with a doc-comment. Update the inode-layout module doc at the top of the file to add a row for `5` → bucket `_INDEX.md`. Update the `reserved_range_is_unmapped` test to iterate from `6..=0xFFFF` instead of `5..=0xFFFF` (inode 5 is no longer unmapped — it's the bucket index).
- **`crates/reposix-fuse/src/fs.rs`** — this is the main work:
  - Add a new `InodeKind::BucketIndex` variant in the classify enum (near `Gitignore`).
  - Extend `InodeKind::classify` to return `BucketIndex` for `ino == BUCKET_INDEX_INO`.
  - Add a `render_bucket_index` pure function on `ReposixFs` (or a free function) that takes `&[Issue]`, `backend_name`, `project`, and returns `Vec<u8>` — the rendered markdown doc.
  - Add an in-memory cache field `bucket_index_bytes: RwLock<Option<Arc<Vec<u8>>>>` on the `ReposixFs` struct. Reset to `None` whenever the issue cache is refreshed (whenever `refresh_issues` or equivalent runs).
  - Hook `lookup(BUCKET_DIR_INO, "_INDEX.md")` to return the synthetic `FileAttr` with inode `BUCKET_INDEX_INO`, size = rendered bytes length (computed lazily; cache ingest triggers a render).
  - Hook `readdir(BUCKET_DIR_INO, ...)` to emit `_INDEX.md` as the first entry (before the real `<padded-id>.md` files), as a regular file with inode `BUCKET_INDEX_INO`.
  - Hook `getattr(BUCKET_INDEX_INO)` to return the synthetic `FileAttr`.
  - Hook `read(BUCKET_INDEX_INO, ...)` to serve bytes from the cache (rendering on first read).
  - Hook `create(BUCKET_DIR_INO, name = "_INDEX.md", ...)` to return `EACCES` (or `EROFS`) — prevent the user shadowing the synthetic.
  - Hook `unlink(BUCKET_DIR_INO, name = "_INDEX.md")` → same errno.
  - Hook `write` / `setattr` / `release` with `BUCKET_INDEX_INO` as the inode → return `EROFS` or default ENOSYS.

## Files to test

- `crates/reposix-fuse/src/fs.rs` — add tests inside the existing `#[cfg(test)] mod tests` block OR in a dedicated `#[cfg(test)] mod bucket_index_tests`:
  - `bucket_index_renders_frontmatter_and_table` — unit test for the `render_bucket_index` pure function. Feed in two sample issues, assert the output contains the frontmatter keys (`backend`, `project`, `issue_count`), the markdown table header, and both issue rows.
  - `bucket_index_row_order_is_ascending_by_id` — feed issues in reverse order, assert the rendered table sorts ascending.
  - `bucket_index_empty_list_is_valid_markdown` — feed an empty slice, assert the output has a well-formed frontmatter with `issue_count: 0` and a valid (but row-less) table OR a "no issues" placeholder.
- Optionally (time permitting) a FUSE integration test in `crates/reposix-fuse/tests/` that mounts over wiremock, reads `<bucket>/_INDEX.md`, parses the frontmatter, and asserts against known issue count. If adding, gate with `#[ignore]` if fusermount3 isn't always available in CI.

## Non-scope for Wave A

- `tree/_INDEX.md` — not in this wave.
- Mount-root `_INDEX.md` — not in this wave.
- OP-3 cache-refresh integration — not in this wave.
- Docs / README / CHANGELOG — that's Wave B.

## Render shape (locked)

```markdown
---
backend: confluence
project: REPOSIX
issue_count: 4
generated_at: 2026-04-14T17:15:00Z
---

# Index of pages/ — REPOSIX (4 pages)

| id | status | title | updated |
| --- | --- | --- | --- |
| 65916 | open | Architecture notes | 2026-04-14 |
| 131192 | open | Welcome to reposix | 2026-04-14 |
| 360556 | open | reposix demo space Home | 2026-04-14 |
| 425985 | open | Demo plan | 2026-04-14 |
```

Header formulation: `# Index of <bucket>/ — <project> (<N> <rows>)`. `<rows>` is `"pages"` if the backend renders pages, `"issues"` otherwise — reuse `backend.root_collection_name()`.

Frontmatter fields (keys pinned — agents may parse):

- `backend` — `IssueBackend::name()`.
- `project` — the mount's project slug string.
- `issue_count` — `u64` decimal.
- `generated_at` — ISO 8601 UTC.

Table rows: sort ascending by `id`; columns in order `id`, `status`, `title`, `updated`. `updated` is the `YYYY-MM-DD` prefix of `updated_at` (drop the time — the time doesn't fit the table and one-day resolution is plenty for a sitemap).

Escape any `|` in titles as `\|` so pipe-tables don't break.

## Acceptance criteria

- `cargo test -p reposix-fuse --locked` green (+3 or more new tests for the render function).
- `cargo clippy -p reposix-fuse --all-targets --locked -- -D warnings` clean.
- `cargo fmt --all --check` clean.
- Manual FUSE mount test (if fusermount3 available):
  - Start `reposix-sim` with seed data.
  - Mount `target/debug/reposix-fuse /tmp/reposix-15-mnt`.
  - `ls /tmp/reposix-15-mnt/issues/` — `_INDEX.md` appears.
  - `cat /tmp/reposix-15-mnt/issues/_INDEX.md` — valid markdown output.
  - `touch /tmp/reposix-15-mnt/issues/_INDEX.md` — permission denied.
  - `rm /tmp/reposix-15-mnt/issues/_INDEX.md` — permission denied.
  - Unmount cleanly.

## Commit message template

`feat(15-A): synthesize _INDEX.md in FUSE bucket dir (OP-2 partial)`

Include Co-Authored-By trailer. Single atomic commit is preferred; at most split into "add BUCKET_INDEX_INO + inode.rs layout update" + "fs.rs wiring + tests" two commits if the agent prefers.

## Bar

- Working tree clean after commit.
- All `fs.rs` write-callback branches (`create`, `unlink`, `write`, `setattr`, `release`) for `BUCKET_INDEX_INO` surface `EROFS` or `EACCES`.
- `bucket_index_bytes` cache safely dropped on issue-list refresh (no stale index after new issues arrive).
- No `#[allow(dead_code)]`.

## Do NOT

- Do NOT extend to tree/ or mount root.
- Do NOT add a `_INDEX.md` cache that outlives the mount.
- Do NOT change any other backend's behavior — the new file appears only via FUSE synthesis.
- Do NOT block Wave B (docs) — if time is tight, split docs into its own PR/commit.
