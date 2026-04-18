# Phase 13: Nested mount layout — Context

**Gathered:** 2026-04-14 (session 4, overnight agent)
**Status:** Ready for planning
**Source:** Direct pre-sleep design handshake with user (captured verbatim from conversation on 2026-04-14). User explicitly reviewed and signed off on symlink-based tree/ approach.

<domain>
## Phase Boundary

Implements **OP-1 from HANDOFF.md** — "folder structure inside the mount, Confluence parentId tree as the killer case." Extends the v0.3 FUSE mount from a flat single-directory view into a two-view layout:

**In-scope (this phase):**
- Introduce a per-backend **writable collection bucket** at the mount root — `pages/` for Confluence, `issues/` for sim + GitHub. All canonical real files live here keyed by `<padded-id>.md` (the stable, numeric identifier).
- Introduce a FUSE-synthesized **read-only `tree/` overlay** that appears alongside `pages/`/`issues/` when the backend exposes parent/child metadata. `tree/` contains **symlinks only**, one per page, at human-readable slug paths that reflect Confluence's native parentId hierarchy.
- Extend `reposix_core::Issue` with an optional `parent_id: Option<IssueId>` field. GitHub backend always returns `None`; Confluence populates from REST v2 `parentId`.
- Deterministic slug generation with sibling-collision resolution.
- FUSE emits a `.gitignore` at mount root containing `/tree/` so the derived overlay is never captured by `git add`/`git push`.
- BREAKING change: every doc, demo, test, CHANGELOG example that previously said `cat mount/0001.md` now says `cat mount/pages/0001.md` or `cat mount/issues/0001.md`.
- Live verification against the REPOSIX space on `reuben-john.atlassian.net` (homepage 360556 → 3 children).

**Out of scope (deferred):**
- `labels/`, `recent/`, `spaces/` views — future Phase 14+.
- `INDEX.md` synthesis inside each directory — OP-2, future phase.
- Cache refresh via `git pull` semantics — OP-3, future phase.
- Write-through-tree semantics beyond "writes transparently follow the symlink to the underlying pages/ target" (which is the default Linux behavior for symlinks — no extra code needed).
- Per-directory `readdir` streaming for huge spaces (OP-1 design Q#3). 500-page cap still applies from Phase 11; if we hit it, that's a separate hardening item (OP-7).
- GitHub mount tree — GitHub issues don't expose parent metadata, so `tree/` is not emitted for `--backend github`.

</domain>

<decisions>
## Implementation Decisions (locked with user before sleep)

### Layout shape
- **Per-backend root collection name.** `IssueBackend::root_collection_name(&self) -> &'static str`. Default `"issues"`. Confluence overrides to `"pages"`. Named alongside the trait; sim + GitHub take the default.
- **Two top-level dirs at the mount root:** `<root_collection>/` (writable, real files) and `tree/` (read-only symlinks, only emitted when ≥1 page has a parent_id OR the backend reports `supports(BackendFeature::Hierarchy)`). Plus a synthesized `.gitignore` file at root.
- **Real files live only in `<root_collection>/<padded-id>.md`.** This is the sole writable, git-tracked path. No duplication.
- **`tree/` contains FUSE symlinks.** Each leaf points at `../<root_collection>/<padded-id>.md` (when the parent dir is directly under `tree/`) or `../../<root_collection>/<padded-id>.md` (when one level deep), etc. Depth-correct relative paths so `readlink -f` resolves inside the mount.
- **Symlink semantics dissolve the merge-conflict problem.** Title renames, reparenting, and sibling reshuffles only change symlink NAMES and LOCATIONS. The target — `<padded-id>.md` — is stable forever (numeric id is immutable at the backend). Git therefore sees zero diffs on metadata churn; only real body edits show up in `git diff`. Two-agent concurrent edits collapse to standard body-content merges on a single stable path.

### Slug algorithm (applies to `tree/` path components)
- Implemented as `reposix_core::path::slugify_title(&str) -> String`.
- Algorithm:
  1. Unicode-lowercase.
  2. Replace every run of non-`[a-z0-9]` ASCII with a single `-`.
  3. Trim leading/trailing `-`.
  4. Truncate to 60 bytes (safe under ext4 NAME_MAX 255).
  5. If result is empty, or equals `.`/`..`, or is an all-`-` string, fall back to `page-<padded-id>`.
- **Collision resolution (siblings under same parent):** group siblings by slug; sort by ascending `IssueId`; the first keeps the bare slug, the Nth gets suffix `-{N}` (2, 3, 4, …). Deterministic across mounts without embedding the numeric id in the common case.
- **Interior node titles ARE exposed as dir names.** A page with children becomes a directory named by the parent's slug containing both the parent's own `.md` symlink AND child symlinks/dirs — directly under the parent dir as siblings. Concretely:

```
tree/
├── space-homepage.md              -> ../pages/00000360556.md  (leaf — only if no children)
└── space-homepage/                (parent dir — siblings incl. self-link)
    ├── _self.md                   -> ../../pages/00000360556.md  (the parent page itself)
    ├── architecture-notes.md      -> ../../pages/00000065916.md
    └── welcome-to-reposix.md      -> ../../pages/00000131192.md
```

Wait — a page cannot be both `foo.md` (leaf file) and `foo/` (parent dir) simultaneously in one POSIX dir. **Locked rule:** if a page has ≥1 child, it materializes as a directory `foo/`, and the page's own body is exposed as a symlink `foo/_self.md` (leading underscore reserves it against slug collisions — `slugify_title` never emits `_self`, per step 2's non-alpha-strip rule).

### Hierarchy method on the backend trait
- Add `fn supports(&self, BackendFeature::Hierarchy) -> bool` that defaults to `false`. Confluence overrides to `true`.
- No new trait method for "give me the tree" — FUSE builds it in-memory from the `Vec<Issue>` returned by `list_issues`, using `issue.parent_id` to group.
- Pages whose `parent_id` points at an id NOT in the mounted list (e.g., parent is outside the space) are treated as tree roots.

### Writable substrate stays single-source-of-truth
- A write via `tree/foo.md` opens the symlink target (`pages/<id>.md`) via normal POSIX symlink resolution — no special-case code in the FUSE layer. This is the single most important property: **symlinks eliminate dual-write paths.**
- The FUSE `readlink` handler returns the correct relative target; that's the only new FUSE op.

### .gitignore emission
- FUSE synthesizes a read-only `.gitignore` file at the mount root containing `/tree/\n` (and the file is itself gitignored-by-its-own-content? No — the `.gitignore` is a real file at the mount root so git-remote-reposix treats it as part of the tree; but its content causes git to skip the `tree/` dir so no churn).
- If an existing `.gitignore` is already present at the mount root (carried from the user's repo config), FUSE does NOT overwrite. Instead, `readdir` shows both the user's file AND the synthesized one at `_reposix_gitignore` — skip that edge case for now, document in SUMMARY that we assume virgin mount roots.
- Simpler decision: **FUSE ALWAYS synthesizes `.gitignore`**, and since the mount is expected to be a clean working tree created by `reposix mount`, collision with a user-authored file is not expected. If it ever bites, Phase 14+.

### BREAKING change handling
- CHANGELOG `### BREAKING` callout under `[Unreleased]` / `[v0.4.0]`:
  - "FUSE layout: issues and Confluence pages are now under a per-backend collection dir (`issues/`, `pages/`). Callers that read `mount/<id>.md` at the root must read `mount/<bucket>/<id>.md`. Use `ls mount/` to discover the bucket name."
- Migration is mechanical: every demo script, README quickstart, doc example, docstring, blog post gets a single find-replace from `mount/<id>.md` to `mount/<bucket>/<id>.md`. Ship all touched files in the phase.
- The sim's REST API is UNCHANGED. The `IssueBackend` trait shape is UNCHANGED except for the new `supports(BackendFeature::Hierarchy)` variant and `root_collection_name`. Wire-format and CLI surfaces are unaffected.

### Release tag
- Ship as **v0.4.0** (feature scope + breaking path change). Script at `scripts/tag-v0.4.0.sh` modeled after `scripts/tag-v0.3.0.sh`.
- CI release.yml (from OP-4) auto-triggers prebuilt binary upload on tag push.

### Claude's Discretion (agent will decide)
- Exact FUSE inode allocation strategy for `tree/` nodes — probably extend the existing registry with a separate namespace (tree-inode range = ROOT + TREE_OFFSET + ...) vs a unified allocator with node-kind tags. Agent picks during implementation; test must cover deep path resolution.
- Whether to cache the computed slug map in the Mount struct (memoize at `Mount::open` from `list_issues` output) or recompute on each `readdir`. Probably cache at open; test that subsequent edits don't leave the tree stale until next mount.
- Whether to separate "TreeNode" types from the existing inode registry in a new module `crates/reposix-fuse/src/tree.rs`, or extend `inode.rs` in place. Agent picks based on file-size budget (fs.rs is already 848 lines — splitting is probably correct).
- Exact test matrix for the wiremock-driven tree tests — agent decides which hierarchies to exercise (deep, wide, orphan-parent, cycle? — cycles must be detected and broken, emit WARN log).
- Whether to run the tree resolver under a proptest for property-based collision testing.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Prior art inside this repo
- `HANDOFF.md` §"OP-1 — Folder structure inside the mount" (lines 233-259) — the user-authored scope statement this phase implements.
- `HANDOFF.md` §"The mission for the next session" (lines 474-486) — the rules for how this phase ships: GSD cycle, live-verify, atomic commits, HANDOFF augmentation.
- `docs/decisions/002-confluence-page-mapping.md` — ADR-002 picked "Option A (flat)"; this phase is the Option-B successor. ADR-003 must explicitly supersede the relevant sections of ADR-002.
- `CLAUDE.md` §"Operating Principles (project-specific)" — rule #5 ("Mount point = git repo") is the constraint that makes `.gitignore` emission non-optional.
- `.planning/phases/11-confluence-adapter/11-F-SUMMARY.md` — the Confluence adapter this phase builds on.

### Code to read before planning
- `crates/reposix-core/src/backend.rs` — the `IssueBackend` trait, lines 94-192. Extension points for `root_collection_name` and `BackendFeature::Hierarchy`.
- `crates/reposix-core/src/issue.rs` — `Issue` struct at lines 52-76. Site for the new `parent_id` field.
- `crates/reposix-core/src/path.rs` — existing path validators. Site for new `slugify_title` helper (must NOT relax `validate_issue_filename`'s title-rejection invariant).
- `crates/reposix-confluence/src/lib.rs` — `ConfPage` struct (lines 203-215) and `translate()` (lines 291-312). Where `parentId` deserialization lands.
- `crates/reposix-fuse/src/fs.rs` — the filesystem impl. `readdir` at line 419, `readlink` (currently unimplemented), inode allocation scheme.
- `crates/reposix-fuse/src/inode.rs` — inode registry (FIRST_ISSUE_INODE = 0x1_0000).
- `crates/reposix-fuse/tests/readdir.rs` — the existing readdir test that will need updating.

### External references
- POSIX symlink semantics (relative-target resolution) — standard man 7 symlink.
- Confluence REST v2 `parentId` shape — already used by Phase 11; response schema in `reposix-confluence/src/lib.rs` struct `ConfPage`.

</canonical_refs>

<specifics>
## Specific Ideas

### Demo target (primary success criterion)
The REPOSIX Confluence space (tenant `reuben-john.atlassian.net`, space key `REPOSIX`) contains:
- Homepage `360556` — "reposix demo space Home" (no parent, tree root)
- `131192` — "Welcome to reposix" (child of homepage)
- `65916` — "Architecture notes" (child of homepage)
- `425985` — "Demo plan" (child of homepage)

After this phase ships, the following must work against real Confluence:
```bash
mkdir -p /tmp/reposix-tree-mnt
reposix mount /tmp/reposix-tree-mnt --backend confluence --project REPOSIX &
sleep 3

# Both views visible at root
ls /tmp/reposix-tree-mnt
# -> .gitignore  pages/  tree/

# Flat view still works (new path)
cat /tmp/reposix-tree-mnt/pages/00000131192.md | head -5

# Hero UX: cd into the hierarchy
cd /tmp/reposix-tree-mnt/tree/reposix-demo-space-home
ls
# -> _self.md  architecture-notes.md  demo-plan.md  welcome-to-reposix.md

cat welcome-to-reposix.md | head -5    # follows symlink into pages/00000131192.md
readlink welcome-to-reposix.md
# -> ../../pages/00000131192.md

# .gitignore keeps tree/ out of git
cd /tmp/reposix-tree-mnt
cat .gitignore
# -> /tree/

fusermount3 -u /tmp/reposix-tree-mnt
```

### Test matrix
- `sim` backend: keep flat (no parent_id populated). `ls mount/` shows `issues/`, NO `tree/`, NO `.gitignore` change (actually still emit `.gitignore` with just nothing inside, OR skip emission when tree/ is absent — agent decides).
- `Confluence` wiremock: 3-level hierarchy fixture (root → 2 children → 1 grandchild under each); test that `readdir` of `tree/` returns two dirs, each containing `_self.md` + children.
- Collision test: 2 siblings with identical title → second gets `-2` suffix deterministically.
- Empty/unicode title test: title = `"   "` → fallback to `page-<id>`; title with emoji → emoji stripped, alphanumeric preserved.
- Symlink resolution test: `openat(tree/foo.md)` opens the correct underlying `<id>.md` bytes.
- `.gitignore` content test: `cat mount/.gitignore` returns `/tree/\n` exactly.

### CHANGELOG block template
```markdown
## [v0.4.0] — 2026-04-14

### BREAKING
- **FUSE mount layout reshuffled.** Previously, issues/pages rendered as `<padded-id>.md` at the mount root. They now render under a per-backend collection bucket: `issues/<padded-id>.md` for sim + GitHub, `pages/<padded-id>.md` for Confluence. Callers that `cat mount/0001.md` must switch to `cat mount/issues/0001.md` (or `pages/`). Run `ls mount/` to discover the bucket.

### Added
- **`tree/` overlay exposes Confluence's native parentId hierarchy.** When mounting a Confluence space, a synthesized, read-only `tree/` subdirectory appears alongside `pages/`, containing symlinks at human-readable slug paths. Navigate the wiki with `cd`. (OP-1 from v0.3 HANDOFF.md — the "hero.png" promise.)
- **`Issue::parent_id: Option<IssueId>`** field on the core type; Confluence backend populates from REST v2 `parentId`.
- **`IssueBackend::root_collection_name`** trait method — default `"issues"`, Confluence returns `"pages"`.
- **Mount-level `.gitignore`** auto-emitted containing `/tree/` so the derived overlay never enters `git add`/`git push`.
- **ADR-003** documents the pages/ + tree/ symlink design.
- Release script `scripts/tag-v0.4.0.sh`.
```

</specifics>

<deferred>
## Deferred Ideas

- OP-2 `INDEX.md` synthesis (planned as future phase)
- OP-3 `git pull` cache refresh (future phase)
- OP-9 Confluence comments at `pages/<id>.comments/` (future phase; may blend with OP-1 once tree/ exists)
- Multi-space mount with `spaces/<key>/` (future phase)
- GitHub labels/ and milestones/ views (future phase; needs label metadata on Issue)
- Whiteboards, attachments, live-docs (OP-9)
- Any write-through-tree semantics beyond passive symlink resolution

</deferred>

---

*Phase: 13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren*
*Context gathered: 2026-04-14 via direct user handshake (skip_discuss: true per .planning/config.json, but captured here because design requires explicit user sign-off on BREAKING change + symlink vs duplicate-content call)*
