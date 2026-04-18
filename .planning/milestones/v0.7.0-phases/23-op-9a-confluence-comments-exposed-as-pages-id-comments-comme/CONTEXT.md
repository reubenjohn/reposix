# Phase 23 CONTEXT — Confluence comments exposed as `pages/<id>.comments/<cid>.md` (OP-9a)

> Status: scoped in session 5, 2026-04-14.
> Author: planning agent, session 6 prep.
> Ties into Phase 19 (labels/spaces directory views) — folder-structure foundation.

## Phase identity

**Name:** Confluence comments exposed as `pages/<id>.comments/<cid>.md` (OP-9a).

**Scope tag:** v0.7.0 (read-only; no comment write path in this phase).

**Addresses:** OP-9 (comments sub-item) from HANDOFF.md. Read-only in this phase. Ties into Phase 19's labels/spaces directory views (same folder-structure pattern).

**Depends on:** Phase 19 (OP-1 remainder — labels/spaces directory views) ships the multi-level inode dispatch pattern this phase reuses.

## Goal (one paragraph)

Confluence pages have inline comments and footer comments that today are invisible in the FUSE mount. This phase exposes them as a synthesized subdirectory: `pages/<padded-id>.comments/<comment-id>.md`. Each comment file has YAML frontmatter (id, author, created_at, resolved, parent_comment_id for threaded replies) and a markdown body. The canonical agent workflow — `cat pages/0001.comments/*.md | grep "blocker"` — is the primary motivator: it converts a JSON-walking task into a plain-text grep, which is the reposix design thesis applied to comments. This phase also adds a `reposix spaces --backend confluence` subcommand that lists all readable spaces (eliminating the requirement for users to know the space key up front).

## Source design context

From HANDOFF.md §OP-9 (comments bullets, verbatim):

- **Comments.** `GET /wiki/api/v2/pages/{id}/inline-comments` + `footer-comments`. Expose as `pages/<id>.comments/<comment-id>.md` — ties into OP-1 folder-structure. Agent workflow: `cat pages/0001.comments/*.md | grep "blocker"` is infinitely cleaner than walking the JSON.
- **Spaces index.** `GET /wiki/api/v2/spaces` to enumerate. Today `--project` requires the user to know the space key up front. A `reposix spaces --backend confluence` subcommand would list them; a `--project all` or multi-space mount (`reposix mount --backend confluence --project '*'`) could mount every readable space under `spaces/<key>/...`.

## Design questions

1. **Inode allocation for `.comments/` dirs.** Each `.comments/` subdirectory needs an inode. Use the existing `InodeRegistry` or reserve a synthetic range? The `.comments/` dir per page scales as O(n_pages) — for a 500-page space, that is 500 new dir inodes. Evaluate whether `InodeRegistry` is cheap enough at that scale or whether a deterministic hash-based inode scheme is better.
2. **Threaded replies.** Inline comments can have replies (parent_comment_id). For v0.7.0: flatten all comments into the same `.comments/` dir (no subdirectory per thread). Expose `parent_comment_id` in frontmatter for agent traversal. Nested dirs are a future extension.
3. **Resolved comments.** Include resolved comments by default (they contain historical context agents often need) or exclude them? Add a `--include-resolved` / `--exclude-resolved` flag or just always include with `resolved: true` in frontmatter.
4. **`reposix spaces` subcommand.** Read-only listing (like `reposix list`). Output: table of space key + name + URL. Needs its own dispatch arm in `reposix-cli`. Shares the `ConfluenceBackend` HTTP client.
5. **Pagination.** `inline-comments` and `footer-comments` are paginated. Use the same cursor-based pattern as `list_issues`. Apply the 500-page cap + WARN from OP-7 (Phase 21).

## Canonical refs

- `crates/reposix-confluence/src/lib.rs` — Confluence adapter; add `list_comments(page_id)` method.
- `crates/reposix-fuse/src/fs.rs` — add `.comments/` dir dispatch in `lookup`/`readdir`/`getattr`/`read`.
- `.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/` — precedent for multi-level dir synthesis in FUSE.
- `.planning/phases/19-op-1-remainder-labels-and-spaces-directory-views-as-read-onl/CONTEXT.md` — sibling phase; same directory-view pattern.
- Confluence v2 API: `GET /wiki/api/v2/pages/{id}/inline-comments`, `GET /wiki/api/v2/pages/{id}/footer-comments`, `GET /wiki/api/v2/spaces`.
- `HANDOFF.md §OP-9` — original design capture.
