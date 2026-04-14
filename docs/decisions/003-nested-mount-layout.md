---
status: accepted
date: 2026-04-14
supersedes: docs/decisions/002-confluence-page-mapping.md (§"Option A: flat" layout decision; everything else in ADR-002 remains valid)
---

# ADR-003: Nested mount layout — pages/ + tree/ symlinks for Confluence hierarchy

- **Status:** Accepted
- **Date:** 2026-04-14
- **Deciders:** reposix core team (overnight session 4)
- **Supersedes:** [ADR-002](002-confluence-page-mapping.md) §"Options"
  layout decision — the flat-layout choice only. The field-mapping,
  auth-decision, pagination-decision, and rate-limit-decision sections of
  ADR-002 remain authoritative.
- **Superseded by:** none
- **Scope:** the FUSE mount root layout emitted by `reposix-fuse` for any
  `IssueBackend` implementation (sim, GitHub, Confluence, future
  adapters). Code lives in `crates/reposix-fuse/src/{fs.rs,tree.rs,inode.rs}`
  and `crates/reposix-core/src/path.rs`.

## Context

ADR-002 chose a flat `<padded-id>.md` layout at the FUSE mount root for the
initial Confluence adapter (Phase 11). That shipped as v0.3 and proved the
read-path works, but left a gap: Confluence is natively hierarchical — every
page has a `parentId` — and `ls mount/` hides that structure. HANDOFF.md
OP-1 ("folder structure inside the mount") identifies this as the next
user-visible win, and the [hero image](../social/assets/hero.png) has been
advertising the tree-like UX since v0.1.

The original design options from ADR-002 §Options were:

- **(A) Flat** — what v0.3 shipped. One directory, `<padded-id>.md` siblings.
- **(B) `PageBackend` trait** — a new backend-shape carrying recursive
  nodes. Correct long-term but weeks of work across every consumer.
- **(C) `parent_id: Option<IssueId>` on the core `Issue`** — rejected in
  ADR-002 because "a field defined for one backend and ignored by others is
  worse than an explicit ADR about lost metadata."

Phase 13 reopens the decision with new information: the symlink overlay
pattern below makes option (C) actually cheap — FUSE synthesizes the tree
itself, no new trait, no duplicate content. The cost is one capability bit
on the existing `IssueBackend` trait plus an optional field on `Issue`.

Phase 13 resolves the layout question with **Option (D): pages/ bucket +
tree/ symlink overlay**. One writable bucket keyed by the stable numeric
id. One synthesized read-only overlay of FUSE symlinks at human-readable
slug paths. Merge-conflict semantics stay sane because the canonical target
path never changes.

## Decision

At the mount root, `readdir` emits three entries:

1. `<root_collection>/` — writable bucket. `pages/` for Confluence,
   `issues/` for sim and GitHub. Determined by a new trait method
   `IssueBackend::root_collection_name() -> &'static str` (default
   `"issues"`, Confluence overrides to `"pages"`). Every canonical
   `<padded-id>.md` lives here; this is the sole writable, git-tracked
   path.
2. `tree/` — synthesized read-only directory, emitted iff the backend
   reports `supports(BackendFeature::Hierarchy)` or any loaded issue has
   `parent_id.is_some()`. Contains FUSE symlinks at slug paths reflecting
   the parentId graph. Not emitted for sim or GitHub.
3. `.gitignore` — synthesized read-only file, always present, contents
   `/tree/\n` (7 bytes). Keeps the derived overlay out of
   `git add` / `git push`.

Symlinks in `tree/` dissolve the merge-conflict problem: title renames,
reparenting, and sibling reshuffles only move or rename the symlinks. The
writable target is always the stable numeric-id path. Git diffs only
surface real body edits. Two-agent concurrent edits collapse to standard
body-content merges on a single stable file.

### Trade-off accepted

Hard-linking or bind-mounting would give the same UX without kernel
path-resolution overhead, but FUSE symlinks cost exactly one extra
`readlink` round-trip per access and require zero special cases in the
write path. We picked the cheaper-to-implement path.

### Alternatives rejected

- **Duplicate-content directories** (each page rendered once under
  `pages/<id>.md` and again under `tree/<slug>.md`). Rejected: doubles the
  writable surface; every edit would need dual-write with conflict
  resolution, and the whole point of this design is to keep the writable
  substrate single-source-of-truth.
- **ID-only tree paths** (`tree/00000360556/00000131192.md`). Rejected:
  indistinguishable from the flat view, loses the ergonomic win that makes
  `cd` into a wiki worthwhile.
- **Git-tracked `tree/`** (no `.gitignore` emission). Rejected: every title
  rename or reparent would show up as a massive symlink churn in
  `git status` and would dominate `git diff`. The whole point of the
  stable-id canonical path is that git only sees real body edits.

## Slug algorithm

Implemented as `reposix_core::path::slugify_title(&str) -> String`:

1. Unicode-lowercase.
2. Replace every run of non-`[a-z0-9]` bytes (including multi-byte UTF-8)
   with a single `-`.
3. Trim leading/trailing `-`.
4. Byte-truncate to 60 on a UTF-8 char boundary.
5. If the result is empty / `.` / `..` / all-`-`, `slug_or_fallback` falls
   back to `page-<11-digit-padded-id>`.

The 60-byte limit fits ext4 NAME_MAX (255) with ample room for collision
suffixes (`-NN`). 11-digit padding matches the existing `<padded-id>.md`
convention under `<root_collection>/`.

## Collision resolution

When two siblings under the same parent produce the same slug, group by
slug → sort by ascending `IssueId` → the first keeps the bare slug, the
Nth gets suffix `-N` (2, 3, 4, ...). Deterministic across mounts without
embedding the numeric id in the common case. Tested by
`nested_layout_collision_gets_suffixed` in
`crates/reposix-fuse/tests/nested_layout.rs`.

## Cycle handling

Confluence's data model forbids parent cycles, but adversarial seed data
or a backend bug could produce one. `TreeSnapshot::build` runs an
iterative DFS with a visited-set; if it finds a cycle, it breaks at the
deepest repeated ancestor, emits a `tracing::warn!`, and treats the
cycle-break node as an orphan root. The mount stays usable under a 5s
open budget and a 3s readdir budget — proven by
`nested_layout_cycle_does_not_hang` in the FUSE integration tests.

## `_self.md` convention

A page with ≥1 child materializes as a directory named by the parent's
slug. POSIX forbids a dir and a file with the same name in one parent,
so the parent's own body is exposed as a symlink `_self.md` inside its
own directory. `_self` is reserved by construction — step 2 of the slug
algorithm strips the leading `_` to `-`, so `slugify_title` can never
emit `_self`.

Concretely, the REPOSIX demo space renders as:

```
tree/
└── reposix-demo-space-home/
    ├── _self.md                 -> ../../pages/00000360556.md
    ├── architecture-notes.md    -> ../../pages/00000065916.md
    ├── demo-plan.md             -> ../../pages/00000425985.md
    └── welcome-to-reposix.md    -> ../../pages/00000131192.md
```

## .gitignore emission

Content is hard-coded `/tree/\n` (7 bytes). Inode `4`
(`GITIGNORE_INO`), `perm: 0o444`, never writable — the kernel rejects
`write()` at the VFS layer, and the FUSE `write()` callback also returns
`EROFS` for `InodeKind::Gitignore` as belt-and-braces. The bytes come from
a compile-time `const GITIGNORE_BYTES: &[u8] = b"/tree/\n";` — no runtime
input, no `format!` call, no content-injection path.

We assume the mount root is a virgin working tree created by
`reposix mount` — the v0.4 design explicitly does NOT handle collision
with a pre-existing user-authored `.gitignore`. If that bites in practice,
Phase 14+ can address it.

## Known limitations

1. **No Unicode NFC normalization (T-13-06).** Two visually-identical
   titles that differ in NFC form (composed `é` vs decomposed `é`) will
   produce different slugs. Not worth the dependency budget (would pull
   `deunicode` or equivalent ~500KB) for the v0.4 read-only path.
   Acceptable for v0.4; reconsider if a real user hits it. Tracked as
   `T-13-06` in the Phase-13 threat register.
2. **Tree does not live-refresh on backend change.** The snapshot is
   rebuilt at each `readdir("tree/")` by re-listing from the backend
   (via the existing lazy-fetch path). If the backend changes between
   fetches, the mount shows the stale tree until the next readdir. This
   is consistent with v0.3's caching semantics.
3. **Confluence-only for v0.4.** GitHub issues don't expose parent
   metadata; sim issues have no hierarchy field. Only Confluence overrides
   `supports(BackendFeature::Hierarchy)` to `true`, so `tree/` is emitted
   only for `--backend confluence`. Sim and GitHub mounts show `issues/`
   + `.gitignore` (with the same `/tree/\n` content, harmlessly inert).
4. **500-page cap still applies.** Phase 11 capped `list_issues` at 500
   pages per invocation. `TreeSnapshot::build` consumes whatever
   `list_issues` returns, so spaces with >500 pages will render a
   truncated tree. OP-7 tracks paginated readdir as a future hardening.
5. **No write-through-tree beyond POSIX symlink semantics.** Writes to
   `tree/foo.md` work because the kernel VFS resolves the symlink to
   `pages/<id>.md` transparently — no custom FUSE logic. But this also
   means `mv tree/foo.md tree/bar.md` (renaming a symlink) silently
   fails; renames in `tree/` are not supported.

## Consequences

- **Breaking.** Every caller that read `mount/<padded-id>.md` now reads
  `mount/<root_collection>/<padded-id>.md`. Demos, docs, README, and
  tests are updated in the same release (v0.4.0). `CHANGELOG.md` lists
  the migration under `### BREAKING`.
- **11-digit id padding.** Previously `{:04}.md`, now `{:011}.md` to
  accommodate astronomical Confluence page IDs without overflow and match
  `TreeSnapshot::symlink_target()` byte-for-byte.
- **Additive core change.** The core `Issue` struct gains
  `parent_id: Option<IssueId>` with `#[serde(default)]` — legacy
  frontmatter files on disk still parse unchanged.
- **Writeable path unchanged.** `pages/` / `issues/` support the same
  write semantics the flat layout had (which is to say, read-only for
  Confluence in v0.4). `tree/foo.md` writes via symlink — kernel VFS
  resolves to `pages/<id>.md` transparently, no custom FUSE logic.
- **Git-merge hell dissolves.** Title renames and reparenting change
  only FUSE-synthesized symlinks, which are `.gitignore`'d. Two agents
  editing the same page body produce a standard single-file merge on
  the stable `pages/<id>.md` path.

## References

- `.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-CONTEXT.md`
  — locked design handshake with user (2026-04-14 pre-sleep session)
- `.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-RESEARCH.md`
  — implementation research
- `.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-C-SUMMARY.md`
  — Wave C integrator summary (FUSE wiring)
- [HANDOFF.md](https://github.com/reubenjohn/reposix/blob/main/HANDOFF.md)
  §OP-1 — original scope statement
- [ADR-002](002-confluence-page-mapping.md) — superseded layout decision
  (flat layout); field-mapping sections remain authoritative
- [ADR-001](001-github-state-mapping.md) — sibling ADR, same
  read-path-only philosophy
- [`crates/reposix-fuse/src/fs.rs`](https://github.com/reubenjohn/reposix/blob/main/crates/reposix-fuse/src/fs.rs)
  — canonical implementation; if this ADR and the code disagree, the code
  wins and this ADR is stale
- [`crates/reposix-core/src/path.rs`](https://github.com/reubenjohn/reposix/blob/main/crates/reposix-core/src/path.rs)
  — `slugify_title` + `slug_or_fallback`
