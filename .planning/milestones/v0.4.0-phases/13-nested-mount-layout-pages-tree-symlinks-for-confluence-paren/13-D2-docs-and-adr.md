---
phase: 13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren
plan: D2
type: execute
wave: 4
depends_on: [C]
files_modified:
  - docs/decisions/003-nested-mount-layout.md
  - CHANGELOG.md
  - CLAUDE.md
  - README.md
  - docs/decisions/002-confluence-page-mapping.md
autonomous: true
requirements:
  - OP-1
user_setup: []

must_haves:
  truths:
    - "`docs/decisions/003-nested-mount-layout.md` exists with YAML frontmatter (`status: accepted`, `date: 2026-04-14`, `supersedes: 002-confluence-page-mapping (Option-A-section)`) and documents the design"
    - "ADR-003 covers: layout shape (pages/ + tree/ + .gitignore), slug algorithm, sibling dedup, cycle-break behavior, `_self.md` convention, known limitation: no Unicode NFC normalization (T-13-06)"
    - "`docs/decisions/002-confluence-page-mapping.md` has a note at the top pointing to ADR-003 as the superseding decision for the layout question"
    - "`CHANGELOG.md` has a `## [v0.4.0] — 2026-04-14` block containing `### BREAKING` + `### Added` sections matching the template in CONTEXT.md §specifics"
    - "`CLAUDE.md` tech-stack section says `fuser 0.17` (not 0.15); the reason-comment is unchanged"
    - "`README.md` has a new top-level section (before `## Quickstart` or similar early section) titled `## Folder-structure mount (v0.4+)` describing the `pages/` + `tree/` layout with a short usage snippet"
    - "`mkdocs build --strict` exits 0 with the new ADR included in the nav"
  artifacts:
    - path: "docs/decisions/003-nested-mount-layout.md"
      provides: "ADR-003 full design doc"
      min_lines: 120
      contains: "## Decision"
    - path: "CHANGELOG.md"
      provides: "[v0.4.0] block with BREAKING + Added sections"
    - path: "CLAUDE.md"
      provides: "fuser version corrected to 0.17"
    - path: "README.md"
      provides: "Folder-structure mount section"
  key_links:
    - from: "docs/decisions/003-nested-mount-layout.md"
      to: "docs/decisions/002-confluence-page-mapping.md"
      via: "supersedes frontmatter field + inline link"
      pattern: "002-confluence-page-mapping"
    - from: "README.md"
      to: "docs/decisions/003-nested-mount-layout.md"
      via: "link from the new Folder-structure mount section"
      pattern: "003-nested-mount-layout"
---

<objective>
Wave-D2. Land the documentation surface of Phase 13: ADR-003 (the locked design spec, superseding ADR-002's flat-layout recommendation), CHANGELOG v0.4.0 entry, CLAUDE.md fuser-version correction (0.15 → 0.17), and a new README section for the folder-structure mount. No code, no scripts — pure docs.

Purpose: CONTEXT.md explicitly locks ADR-003 as mandatory. Live-Confluence users need the README section to know the new path. The CHANGELOG is the v0.4.0 release-note artifact. The CLAUDE.md correction fixes a known-stale fact that mis-guides future agents.

Output: One new ADR file, three edits to existing docs. `mkdocs build --strict` remains green.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-CONTEXT.md
@.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-RESEARCH.md
@docs/decisions/001-github-state-mapping.md
@docs/decisions/002-confluence-page-mapping.md
@CHANGELOG.md
@CLAUDE.md
@README.md

<interfaces>
<!-- ADR template — mirror 002-confluence-page-mapping.md structure: -->

```markdown
---
status: accepted
date: 2026-04-14
supersedes: docs/decisions/002-confluence-page-mapping.md (layout section only)
---

# ADR-003: Nested mount layout — pages/ + tree/ symlinks

## Context
## Decision
## Slug algorithm
## Collision resolution
## Cycle handling
## .gitignore emission
## Known limitations
## Consequences
## References
```

<!-- CHANGELOG template (from CONTEXT.md §specifics): -->

```markdown
## [v0.4.0] — 2026-04-14

### BREAKING
- **FUSE mount layout reshuffled.** ...

### Added
- **`tree/` overlay exposes Confluence's native parentId hierarchy.** ...
- **`Issue::parent_id: Option<IssueId>`** ...
- **`IssueBackend::root_collection_name`** ...
- **Mount-level `.gitignore`** ...
- **ADR-003** documents the pages/ + tree/ symlink design.
- Release script `scripts/tag-v0.4.0.sh`.
```
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Write ADR-003 + supersede ADR-002</name>
  <files>
    docs/decisions/003-nested-mount-layout.md,
    docs/decisions/002-confluence-page-mapping.md
  </files>
  <action>
    Create `docs/decisions/003-nested-mount-layout.md` with the full design. Structure (copy frontmatter/shape from ADR-002; don't invent a new format):

    ```markdown
    ---
    status: accepted
    date: 2026-04-14
    supersedes: docs/decisions/002-confluence-page-mapping.md (§"Option A: flat" layout decision; everything else in ADR-002 remains valid)
    ---

    # ADR-003: Nested mount layout — pages/ + tree/ symlinks for Confluence hierarchy

    ## Context

    ADR-002 chose a flat `<padded-id>.md` layout at the FUSE mount root for the
    initial Confluence adapter (Phase 11). That shipped as v0.3 and proved the
    read-path works, but left a gap: Confluence is natively hierarchical — every
    page has a `parentId` — and `ls mount/` hides that structure. HANDOFF.md
    OP-1 ("folder structure inside the mount") identifies this as the next
    user-visible win.

    The original design options from ADR-002 §Options were:
    - (A) Flat — what v0.3 shipped.
    - (B) Directories mirroring parentId — rejected in ADR-002 because it
      duplicates content and confuses the write path.
    - (C) Something in between.

    Phase 13 resolves this with **Option (D): pages/ bucket + tree/ symlink
    overlay**. One writable bucket keyed by stable numeric id. One synthesized
    read-only overlay of FUSE symlinks at human-readable slug paths.

    ## Decision

    At the mount root, emit three entries:

    1. `<root_collection>/` — writable bucket. `pages/` for Confluence, `issues/`
       for sim and GitHub. Determined by a new trait method
       `IssueBackend::root_collection_name() -> &'static str` (default `"issues"`,
       Confluence overrides to `"pages"`). Every canonical `<padded-id>.md`
       lives here; this is the sole writable path.
    2. `tree/` — synthesized read-only directory, emitted iff the backend
       reports `supports(BackendFeature::Hierarchy)` or any loaded issue has
       `parent_id.is_some()`. Contains FUSE symlinks at slug paths reflecting
       the parentId graph.
    3. `.gitignore` — synthesized read-only file, always present, contents
       `/tree/\n`. Keeps the derived overlay out of `git add` / `git push`.

    Symlinks in `tree/` dissolve the merge-conflict problem: title renames,
    reparenting, sibling reshuffles only move or rename the symlinks. The
    writable target is always the stable numeric-id path. Git diffs only
    surface real body edits. Two-agent concurrent edits collapse to standard
    body-content merges on a single stable file.

    ### Trade-off accepted

    Hard-linking or bind-mounting would give the same UX without kernel
    path-resolution overhead, but FUSE symlinks cost exactly one extra
    `readlink` round-trip per access and require zero special cases in the
    write path. We picked the cheaper-to-implement path.

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
    embedding the numeric id in the common case.

    ## Cycle handling

    Confluence's data model forbids parent cycles, but adversarial seed data
    or a backend bug could produce one. `TreeSnapshot::build` runs an
    iterative DFS with a visited-set; if it finds a cycle, it breaks at the
    deepest repeated ancestor, emits a `tracing::warn!`, and treats the
    cycle-break node as an orphan root. The mount stays usable.

    ## `_self.md` convention

    A page with ≥1 child materializes as a directory named by the parent's
    slug. POSIX forbids a dir and a file with the same name in one parent,
    so the parent's own body is exposed as a symlink `_self.md` inside its
    own directory. `_self` is reserved by construction — step 2 of the slug
    algorithm strips leading `_` to `-`, so `slugify_title` can never emit
    `_self`.

    ## .gitignore emission

    Content is hard-coded `/tree/\n` (7 bytes). Inode 4, `perm: 0o444`,
    never writable. We assume the mount root is a virgin working tree
    created by `reposix mount` — the v0.4 design explicitly does NOT handle
    collision with a pre-existing user-authored `.gitignore`. If that bites
    in practice, Phase 14+ can address it.

    ## Known limitations

    1. **No Unicode NFC normalization.** Two visually-identical titles that
       differ in NFC form (composed `é` vs decomposed `é`) will produce
       different slugs. Not worth the dependency budget (would pull `deunicode`
       or equivalent ~500KB). Acceptable for v0.4; reconsider if a real user
       hits it.
    2. **Tree does not live-refresh on backend change.** The snapshot is
       rebuilt at each `readdir("tree/")` by re-listing from the backend
       (via the existing lazy-fetch path). If the backend changes between
       fetches, the mount shows the stale tree until the next readdir. This
       is consistent with v0.3's caching semantics.
    3. **GitHub and sim backends do not emit `tree/`.** GitHub issues don't
       expose parent metadata; sim issues have no hierarchy field. Only
       Confluence overrides `supports(BackendFeature::Hierarchy)` to `true`.

    ## Consequences

    - **Breaking.** Every caller that read `mount/<padded-id>.md` now reads
      `mount/<root_collection>/<padded-id>.md`. Demos, docs, README, and
      tests are updated in the same release (v0.4.0). `CHANGELOG.md` lists
      the migration.
    - **Additive.** The core `Issue` struct gains `parent_id: Option<IssueId>`
      with `#[serde(default)]` — legacy frontmatter files on disk still parse
      unchanged.
    - **Writeable path unchanged.** `pages/` / `issues/` support the same
      write semantics the flat layout had (which is to say, read-only for
      Confluence in v0.4). `tree/foo.md` writes via symlink — kernel VFS
      resolves to `pages/<id>.md` transparently, no custom FUSE logic.

    ## References

    - `.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-CONTEXT.md` — locked design handshake with user
    - `.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-RESEARCH.md` — implementation research
    - HANDOFF.md §OP-1 — original scope statement
    - ADR-002 — superseded layout decision
    ```

    **Append a note to the top of `docs/decisions/002-confluence-page-mapping.md`** (right after the frontmatter, before the main body):
    ```markdown
    > **Superseded in part (2026-04-14).** The flat-layout decision in this
    > ADR's §Option A was replaced by ADR-003 (`pages/` bucket + `tree/`
    > symlink overlay). The field-mapping sections below (Confluence page →
    > Issue field mapping) remain authoritative.
    ```

    Run `mkdocs build --strict` locally to confirm ADR-003 renders without errors (nav config auto-discovers new ADRs if the site uses `nav:` glob, else update `mkdocs.yml`).
  </action>
  <verify>
    <automated>test -f docs/decisions/003-nested-mount-layout.md &amp;&amp; grep -q "## Decision" docs/decisions/003-nested-mount-layout.md &amp;&amp; grep -q "## Slug algorithm" docs/decisions/003-nested-mount-layout.md &amp;&amp; grep -q "ADR-003" docs/decisions/002-confluence-page-mapping.md &amp;&amp; mkdocs build --strict</automated>
  </verify>
  <done>
    ADR-003 exists, covers all 9 required sections, supersedes ADR-002's layout decision. ADR-002 has the supersede note. mkdocs strict build green. Commit: `docs(13-D2-1): ADR-003 nested mount layout + supersede ADR-002 layout section`.
  </done>
</task>

<task type="auto">
  <name>Task 2: CHANGELOG v0.4.0 block + CLAUDE.md fuser fix + README section</name>
  <files>
    CHANGELOG.md,
    CLAUDE.md,
    README.md
  </files>
  <action>
    **CHANGELOG.md** — prepend a `## [v0.4.0] — 2026-04-14` block after any `[Unreleased]` header (or at top if no Unreleased). Use the exact template from `13-CONTEXT.md §specifics`:

    ```markdown
    ## [v0.4.0] — 2026-04-14

    ### BREAKING

    - **FUSE mount layout reshuffled.** Previously, issues/pages rendered as
      `<padded-id>.md` at the mount root. They now render under a per-backend
      collection bucket: `issues/<padded-id>.md` for sim + GitHub,
      `pages/<padded-id>.md` for Confluence. Callers that `cat mount/0001.md`
      must switch to `cat mount/issues/0001.md` (or `mount/pages/...`). Run
      `ls mount/` to discover the bucket.

    ### Added

    - **`tree/` overlay exposes Confluence's native parentId hierarchy.** When
      mounting a Confluence space, a synthesized, read-only `tree/`
      subdirectory appears alongside `pages/`, containing symlinks at
      human-readable slug paths. Navigate the wiki with `cd`. (OP-1 from v0.3
      HANDOFF.md — the "hero.png" promise.)
    - **`Issue::parent_id: Option<IssueId>`** field on the core type;
      Confluence backend populates from REST v2 `parentId`.
    - **`IssueBackend::root_collection_name`** trait method — default
      `"issues"`, Confluence returns `"pages"`.
    - **Mount-level `.gitignore`** auto-emitted containing `/tree/` so the
      derived overlay never enters `git add` / `git push`.
    - **`BackendFeature::Hierarchy`** capability variant.
    - **ADR-003** documents the `pages/` + `tree/` symlink design.
    - Release script `scripts/tag-v0.4.0.sh` + Confluence-tree demo
      `scripts/demos/07-mount-real-confluence-tree.sh`.
    ```

    If `[Unreleased]` had any bullets pending, move them under the `v0.4.0` Added section as appropriate (agent's judgement).

    **CLAUDE.md** — fix the fuser version claim on line 40. Change:
    ```
    - FUSE: `fuser` 0.15 with `default-features = false`. **Reason:** ...
    ```
    to:
    ```
    - FUSE: `fuser` 0.17 with `default-features = false`. **Reason:** the dev host lacks `pkg-config` and `libfuse-dev`, and we have no passwordless sudo to install them. Runtime mounting uses `fusermount`/`fusermount3` binaries (already present on Ubuntu and on `ubuntu-latest` GitHub runners after `apt install fuse3`).
    ```
    The reason-comment is unchanged — only the version number. Confirm the actual Cargo.lock version matches (already verified in 13-RESEARCH.md §Core stack — it's 0.17.0).

    **README.md** — insert a new top-level section `## Folder-structure mount (v0.4+)` right before the existing Quickstart (or equivalent early section). If unclear where to put it, place it immediately after the project pitch paragraph. Content:

    ```markdown
    ## Folder-structure mount (v0.4+)

    Mounting a Confluence space exposes the page hierarchy as a navigable
    directory tree:

    ```bash
    reposix mount /tmp/mnt --backend confluence --project REPOSIX &
    ls /tmp/mnt
    # .gitignore  pages/  tree/

    # Flat view — every page keyed by its stable numeric id
    ls /tmp/mnt/pages
    # 00000065916.md  00000131192.md  00000360556.md  00000425985.md

    # Hierarchy view — symlinks at human-readable slug paths
    cd /tmp/mnt/tree/reposix-demo-space-home
    ls
    # _self.md  architecture-notes.md  demo-plan.md  welcome-to-reposix.md

    cat welcome-to-reposix.md     # follows symlink into pages/00000131192.md
    readlink welcome-to-reposix.md
    # ../../pages/00000131192.md
    ```

    `tree/` is synthesized at mount time. It is read-only, `git`-ignored (via
    the auto-emitted `.gitignore`), and backed entirely by FUSE symlinks —
    there is no duplicate content and no dual-write path. Writes to
    `tree/foo.md` transparently follow the symlink to the canonical
    `pages/<id>.md` file.

    For sim and GitHub backends, the mount shows `issues/` instead of
    `pages/` and does not emit `tree/` (those backends don't expose a
    parent-child hierarchy). See [ADR-003](docs/decisions/003-nested-mount-layout.md)
    for the full design.
    ```

    Run `mkdocs build --strict` again after these edits.
  </action>
  <verify>
    <automated>grep -q '## \[v0.4.0\]' CHANGELOG.md &amp;&amp; grep -qE 'fuser.*0\.17' CLAUDE.md &amp;&amp; grep -q 'Folder-structure mount' README.md &amp;&amp; grep -q '003-nested-mount-layout' README.md &amp;&amp; mkdocs build --strict</automated>
  </verify>
  <done>
    CHANGELOG has v0.4.0 block. CLAUDE.md fuser line corrected. README has the new section linking to ADR-003. mkdocs strict green. Commit: `docs(13-D2-2): CHANGELOG v0.4.0 + CLAUDE.md fuser-0.17 + README folder-structure section`.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

No new boundaries — docs-only plan.

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|---|---|---|---|---|
| T-13-06 | Tampering | Unicode NFC normalization gap | accept + document | ADR-003 §"Known limitations" §1 explicitly lists this with rationale. Future users who hit it have the ADR as reference for why it's not mitigated in v0.4. |
| T-13-D2-1 | Repudiation | Stale CLAUDE.md misguides future agents | mitigate | Fix the fuser version claim (0.15 → 0.17); the rest of the file's claims were verified during research and remain accurate. |
</threat_model>

<verification>
Nyquist coverage:
- **`mkdocs build --strict`** — proves the ADR renders cleanly and is discoverable through the site nav.
- **Grep checks** — verify each required doc change landed.
- **No code tests needed** — docs-only plan.
</verification>

<success_criteria>
Each a Bash assertion runnable from repo root:

1. `test -f docs/decisions/003-nested-mount-layout.md` exits 0.
2. `wc -l docs/decisions/003-nested-mount-layout.md | awk '{ exit ($1 >= 120 ? 0 : 1) }'` exits 0 (min 120 lines).
3. `grep -q "## Decision" docs/decisions/003-nested-mount-layout.md` exits 0.
4. `grep -q "Slug algorithm" docs/decisions/003-nested-mount-layout.md` exits 0.
5. `grep -q "Known limitations" docs/decisions/003-nested-mount-layout.md` exits 0.
6. `grep -q "NFC" docs/decisions/003-nested-mount-layout.md` exits 0 (T-13-06 documented).
7. `grep -q "Superseded in part" docs/decisions/002-confluence-page-mapping.md` exits 0.
8. `grep -q "## \[v0.4.0\]" CHANGELOG.md` exits 0.
9. `grep -q "### BREAKING" CHANGELOG.md` exits 0.
10. `grep -qE "fuser.*0\.17" CLAUDE.md && ! grep -qE "fuser.*0\.15" CLAUDE.md` exits 0.
11. `grep -q "Folder-structure mount" README.md` exits 0.
12. `grep -q "003-nested-mount-layout" README.md` exits 0.
13. `mkdocs build --strict` exits 0.
</success_criteria>

<output>
After completion, create `.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-D2-SUMMARY.md` documenting:
- ADR-003 final section count and line count
- CHANGELOG block inserted (paste the first 10 lines)
- CLAUDE.md diff for the fuser version line
- README section title + first 3 lines
- mkdocs strict build output's success line
</output>
