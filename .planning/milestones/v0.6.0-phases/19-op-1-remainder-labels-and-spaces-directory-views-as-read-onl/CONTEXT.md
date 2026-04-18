# Phase 19 CONTEXT — OP-1 Remainder: Labels + Spaces Directory Views

> Status: queued (session 6, 2026-04-14). Not yet planned or executed.
> Phase 13 shipped the core OP-1 (pages/ + tree/ symlink overlay). This phase ships the remaining views.
> Milestone: v0.6.0.

## Phase identity

**Name:** OP-1 remainder — `labels/` and `spaces/` directory views as read-only symlink overlays for GitHub and Confluence.

**Scope tag:** v0.6.0. Extends Phase 13's nested layout.

**Addresses:** OP-1 remaining items from HANDOFF.md — specifically `labels/`, `spaces/`, multi-space mounts. NOT the design questions already answered by Phase 13 (symlinks vs hardlinks → symlinks; read-only tree overlay; slug collision resolution). Those are in `.planning/phases/13-*/deferred-items.md` and `docs/decisions/003-*` (ADR-003).

## Goal (one paragraph)

Add two new read-only symlink overlay directories to the FUSE mount: (1) `mount/labels/<label>/` — lists all issues/pages carrying that label as symlinks pointing to the canonical file in the bucket (`../../issues/<padded-id>.md` or `../../pages/<padded-id>.md`); and (2) `mount/spaces/<key>/` — lists all pages in that Confluence space as symlinks, enabling a multi-space mount view. Both overlays are read-only (same as `tree/`); writes go through the canonical bucket path. The `labels/` view works for both GitHub (via issue labels) and Confluence (via Confluence labels endpoint).

## Source design context (migrated from HANDOFF.md)

### OP-1 remaining items (from HANDOFF.md, session-4 §Open problems rollup)

> **OP-1 (this session — SHIPPED).** Confluence parentId tree is live. The remaining pieces of OP-1's original scope (`labels/`, `recent/`, `spaces/`, multi-space mount, symlink into `labels/bug/0001.md` from `issues/0001.md`) are explicitly out of scope for v0.4 and deferred to v0.5+.

### OP-1 design questions — remaining open ones (from HANDOFF.md §OP-1)

The following design questions from HANDOFF.md §OP-1 are specifically for the labels/spaces work (Phase 13 answered #1, #2, #4, and #5):

> **3. How does `readdir` perf survive?** A Confluence space with 5000 pages would generate 5000 paths under `tree/…` — that's fine for `ls` but `find .` becomes glacial without proper dir caching.
> **4. Namespace collisions.** Two pages titled "Architecture notes" under different parents — slugs must disambiguate without leaking the numeric id into the human-visible path. *(Resolved in Phase 13 via `dedupe_siblings`.)*

For `labels/`:
- GitHub labels come from `GET /repos/{owner}/{repo}/issues?labels={label}` — already paginated in the GitHub adapter.
- Confluence labels come from `GET /wiki/api/v2/labels` and `GET /wiki/api/v2/pages?label={label}` — separate endpoint, not yet implemented.

For `spaces/`:
- `GET /wiki/api/v2/spaces` lists all readable spaces. Today `--project` requires the user to know the space key. A multi-space mount under `mount/spaces/<key>/...` would surface all accessible spaces.

### From session-5 §OP-1 rollup

> **OP-1 — nested mount layout.** Confluence `tree/` (parentId hierarchy) is live (v0.4.0). Remaining: `labels/`, `recent/`, `spaces/`, multi-space mounts.

## Design questions to resolve before planning

1. **Label fetch strategy:** Should `labels/` be populated lazily (only fetched on first `ls labels/`) or eagerly (fetched on mount)?
2. **`recent/` in scope?** HANDOFF.md mentions `recent/<yyyy-mm-dd>/` as an OP-1 item. Include in this phase or defer?
3. **Multi-space mount:** `--project '*'` to mount all spaces, or a separate `reposix spaces` subcommand? Both are in OP-9's scope too.
4. **Symlink target format:** Same `../../pages/<padded-id>.md` relative format as tree/, or absolute mount-relative paths?
5. **`.gitignore` update:** Phase 13 added `/tree/\n` to the synthesized `.gitignore`. Should `/labels/\n` and `/spaces/\n` also be gitignored?

## Locked decisions (from Phase 13)

- **Symlinks, not duplicate files** (ADR-003): canonical file in bucket; overlays are symlinks.
- **Read-only overlays:** Writes go through the symlink target (the canonical bucket file).
- **Slug-deduplication:** `dedupe_siblings` from `crates/reposix-fuse/src/tree.rs` must be reused for label-scoped issue lists.

## Non-goals / scope boundaries

- Do NOT implement `recent/<yyyy-mm-dd>/` in this phase (defer unless trivial after labels/ lands).
- Do NOT implement Confluence-specific attachment label views (that's OP-9b scope).
- Do NOT implement write semantics for `mv labels/bug/0001.md labels/p1/0001.md` (too exotic).

## Canonical refs

- `crates/reposix-fuse/src/tree.rs` — `TreeSnapshot`, `dedupe_siblings`; model `labels/` tree on this.
- `crates/reposix-fuse/src/fs.rs` — FUSE dispatch; `tree/` symlink handling is the template.
- `docs/decisions/003-*.md` — ADR-003 (nested layout decisions).
- `docs/social/assets/hero.png` — the "hero image" advertising `labels/`, `milestones/` sidebar tree.
