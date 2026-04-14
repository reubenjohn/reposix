# Phase 18 CONTEXT — OP-2 Remainder: Tree-Recursive + Mount-Root _INDEX.md

> Status: queued (session 6, 2026-04-14). Not yet planned or executed.
> Follows Phase 17. Milestone: v0.6.0.
> Phase 15 shipped the bucket-level `_INDEX.md`. This phase ships the remaining two levels.

## Phase identity

**Name:** OP-2 remainder — tree-recursive and mount-root `_INDEX.md` synthesis extending `TreeSnapshot` dfs.

**Scope tag:** v0.6.0 (feature scope — completes OP-2 started in Phase 15).

**Addresses:** HANDOFF.md OP-2 remainder + session-5 §"Post-review cleanup candidates (15-SUMMARY follow-ups)".

## Goal (one paragraph)

Complete OP-2 by synthesizing `_INDEX.md` at two additional levels: (1) `mount/tree/<subdir>/_INDEX.md` — a recursive sitemap of that subtree, computed via cycle-safe DFS from `TreeSnapshot`; and (2) `mount/_INDEX.md` — a whole-mount overview listing all backends, buckets, and top-level entry counts. Combined with Phase 15's bucket-level `_INDEX.md`, agents can now get any level of the hierarchy in one `cat` operation. The tree-level synthesis reuses the existing `TreeSnapshot::dfs` infrastructure from Phase 13.

## Source design context (migrated from HANDOFF.md)

### OP-2 original design questions

From HANDOFF.md §OP-2:

> Every directory should contain a synthesized `INDEX.md` (or `_INDEX.md` — leading underscore to keep it out of naive `*.md` globs) that FUSE generates on read.
>
> **Design questions:**
> - Markdown table, YAML frontmatter block, or both? *(Resolved by Phase 15: YAML frontmatter + markdown table — use the same format.)*
> - Included in `ls` output (could confuse naive users) or hidden from `readdir` and only accessible by explicit path? *(Resolved by Phase 15: visible in `ls`, leading `_` keeps out of `*.md` globs.)*
> - Cached or regenerated on every read? *(Resolved by Phase 15: in-memory cache, same cadence as the bucket readdir cache.)*
> - Does it include nested subdirectories? For `tree/parent-a/INDEX.md`, is the index for just that dir, or recursive? *(Open — this phase resolves it: recursive, using TreeSnapshot::dfs.)*

### From session-5 §"Post-review cleanup candidates (15-SUMMARY follow-ups)"

> - Tree-level recursive `_INDEX.md` (biggest remaining OP-2 piece).
> - Mount-root `_INDEX.md` (smallest remaining OP-2 piece).
> - User-configurable column set in the index.
> - `_INDEX.md`-in-`git diff` round-trip semantics (ties into OP-3).

### From session-5 §OP-2 rollup

> **OP-2 — dynamic `_INDEX.md`.** Bucket level shipped (v0.5.0). Remaining: `tree/<subdir>/_INDEX.md` (recursive synthesis, cycle-safe — straightforward extension of `TreeSnapshot::dfs`) and `mount/_INDEX.md` (whole-mount overview).

## Design questions to resolve

1. **Recursion depth limit:** For `tree/<subdir>/_INDEX.md`, does the index list ALL descendants (DFS) or only direct children? Recursive is more powerful but may be very long for deep Confluence spaces.
2. **Cycle safety:** `TreeSnapshot::dfs` already has cycle detection from Phase 13. Verify it's reused here, not re-implemented.
3. **Mount-root `_INDEX.md` format:** Lists all buckets (`issues/`, `pages/`, `tree/`) with counts. Should it include `tree/` entry counts or just the bucket entry counts?
4. **Fixed inode for tree-level `_INDEX.md`:** Phase 15 used `BUCKET_INDEX_INO = 5`. Each tree subdirectory would need its own inode for `_INDEX.md`. Dynamic allocation? Or a naming scheme?
5. **Tie-in with OP-3:** When `reposix refresh` runs (Phase 20), should it invalidate and regenerate all `_INDEX.md` caches? Or is that implicit (same cache invalidation as `list_issues`)?

## Canonical refs

- `crates/reposix-fuse/src/fs.rs` — bucket-level `_INDEX.md` implementation from Phase 15; model tree-level on this.
- `crates/reposix-fuse/src/inode.rs` — `BUCKET_INDEX_INO = 5`; reserved synthetic range `5..=0xFFFF`.
- `crates/reposix-fuse/src/tree.rs` — `TreeSnapshot`, `dfs`, sibling-slug deduplication (Phase 13).
- `.planning/phases/15-dynamic-index-md-synthesized-in-fuse-bucket-directory-op-2-p/15-CONTEXT.md` — Phase 15 locked decisions.
