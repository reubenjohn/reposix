# Phase 18: OP-2 Remainder — Tree-Recursive + Mount-Root `_INDEX.md` - Research

**Researched:** 2026-04-14
**Domain:** FUSE inode dispatch, `TreeSnapshot` DFS traversal, synthesized file rendering
**Confidence:** HIGH — all findings verified directly from the committed codebase

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions (inherited from Phase 15)
- LD-15-01: Filename is `_INDEX.md` (leading underscore, visible in `ls`)
- LD-15-02: YAML frontmatter + markdown pipe-table format
- LD-15-10: Deterministic row order (ascending `id`)

### Claude's Discretion
- Whether tree-level `_INDEX.md` lists ALL descendants (full DFS) or only direct children
- Inode allocation strategy for tree-dir `_INDEX.md` entries (see §Architectural Responsibility Map)
- Mount-root `_INDEX.md` exact format and which counts to include
- Cache invalidation strategy for tree-level and mount-root indexes

### Deferred Ideas (OUT OF SCOPE)
- User-configurable column set in the index
- `_INDEX.md`-in-`git diff` round-trip semantics (OP-3, Phase 20)
- `reposix refresh` subcommand (OP-3, Phase 20)
- Per-directory indexes for `labels/` or `spaces/` layouts (OP-1, Phase 19)
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| INDEX-01 | `cat mount/tree/<subdir>/_INDEX.md` returns a recursive markdown sitemap of that subtree, computed via cycle-safe DFS from `TreeSnapshot` | §Architecture Patterns covers DFS traversal via existing `TreeSnapshot` methods; §Standard Stack shows no new deps needed |
| INDEX-02 | `cat mount/_INDEX.md` returns a whole-mount overview listing all backends, buckets, and top-level entry counts | §Architecture Patterns covers mount-root render; §Inode Strategy shows `ROOT_INDEX_INO = 6` is the correct allocation |
</phase_requirements>

---

## Summary

Phase 18 extends the `_INDEX.md` synthesis from Phase 15 (bucket level) to two additional levels: every `tree/<subdir>/` directory and the mount root. Both are purely additive: no new dependencies, no schema changes, no backend API calls beyond what the existing `refresh_issues` path already does.

The key insight is that `TreeSnapshot` already contains the complete tree structure in two maps: `dirs: HashMap<u64, TreeDir>` (interior nodes) and `symlink_targets: HashMap<u64, String>` (leaves). A tree-level `_INDEX.md` is a DFS walk of these maps starting from a given `TreeDir`, collecting `(depth, name, symlink_target)` tuples and rendering them as a markdown outline. Cycle safety is already baked in — `TreeSnapshot::build` breaks all cycles before the snapshot is constructed, so the snapshot itself is a DAG (actually a forest of trees); no second cycle-detection layer is needed during DFS rendering.

The mount-root `_INDEX.md` is even simpler: it lists the three top-level entries (`.gitignore`, `<bucket>/`, `tree/`) with their entry counts (derived from the already-cached issue list), similar to the bucket index frontmatter.

**Primary recommendation:** Allocate `ROOT_INDEX_INO = 6` and `TREE_DIR_INDEX_INO_BASE` from the reserved `6..=0xFFFF` synthetic range; add two new `InodeKind` variants; add `render_tree_index` and `render_mount_root_index` pure functions mirroring `render_bucket_index`; hook all five FUSE callbacks (getattr, lookup, readdir, read, write/setattr/unlink for EROFS rejection) using the exact same pattern as `BucketIndex`.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Tree-level `_INDEX.md` content rendering | `reposix-fuse/fs.rs` | `reposix-fuse/tree.rs` | Pure function over `TreeSnapshot`; same tier as `render_bucket_index` |
| Mount-root `_INDEX.md` content rendering | `reposix-fuse/fs.rs` | — | Purely derived from already-held state (issue count + bucket name) |
| Inode allocation for new synthetics | `reposix-fuse/inode.rs` | — | Inode layout is owned by this module; new constants go here |
| Tree-level `_INDEX.md` inode-per-dir | `reposix-fuse/tree.rs` | `reposix-fuse/fs.rs` | Two options — see §Inode Strategy |
| Cache invalidation | `reposix-fuse/fs.rs` (`refresh_issues`) | — | Already drops `bucket_index_bytes` on refresh; same pattern applies |
| FUSE dispatch (lookup/readdir/read/getattr) | `reposix-fuse/fs.rs` | — | All callbacks already structured around `InodeKind::classify` |

---

## Standard Stack

### Core (no new dependencies)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `fuser` | 0.17 (workspace) | FUSE callbacks | Already in workspace; `default-features = false` |
| `chrono` | workspace | `generated_at` timestamp in frontmatter | Same as `render_bucket_index` |
| `std::fmt::Write` | stdlib | `write!` into `String` (infallible) | Same pattern as `render_bucket_index` |

**No new `Cargo.toml` additions required.** [VERIFIED: grep Cargo.toml + codebase]

---

## Architecture Patterns

### FUSE Synthetic File Read Path (verified from fs.rs)

Every synthesized file in this codebase follows the same five-step pattern:

1. **Constant inode** declared in `inode.rs`
2. **`InodeKind` variant** added to the enum
3. **`InodeKind::classify`** match arm added (numeric range or exact match)
4. **Pure render function** `fn render_X(…) -> Vec<u8>` — no IO, deterministic
5. **Five callback dispatch arms**: `getattr`, `lookup` (in parent), `readdir` (in parent), `read` (on the file inode), `write`/`setattr`/`unlink` → `EROFS`

The `bucket_index_bytes_or_render` pattern (lazy cache on `RwLock<Option<Arc<Vec<u8>>>>`) should be replicated:

```rust
// Source: crates/reposix-fuse/src/fs.rs:758-783
fn bucket_index_bytes_or_render(&self) -> Arc<Vec<u8>> {
    if let Ok(guard) = self.bucket_index_bytes.read() {
        if let Some(bytes) = guard.as_ref() {
            return bytes.clone();
        }
    }
    // render + store
    let rendered = Arc::new(render_bucket_index(…));
    if let Ok(mut guard) = self.bucket_index_bytes.write() {
        *guard = Some(rendered.clone());
    }
    rendered
}
```

For tree-dir indexes the cache must be keyed by dir inode because each subdirectory has its own content. Use `DashMap<u64, Arc<Vec<u8>>>` for O(1) concurrent access (same crate already in workspace). [VERIFIED: DashMap already imported in fs.rs]

### Inode Strategy for Tree-Dir `_INDEX.md`

**Option A — Single fixed inode from reserved range.**
Allocate `TREE_DIR_INDEX_INO_BASE = 6` (or any value in `6..=0xFFFF`). Problem: every `tree/<subdir>/_INDEX.md` would share the same inode, which confuses `stat`, hardlink counts, and tools that use inode to distinguish files. **Not recommended.**

**Option B — Dynamic allocation from `6..=0xFFFF` reserved range (recommended).**
Allocate a fresh inode per tree-dir `_INDEX.md` lazily, stored in a `DashMap<u64 /* dir_ino */, u64 /* index_ino */>` on `ReposixFs`. The allocator uses an `AtomicU64` starting at `6`, capped at `0xFFFF`. Reserve `6` for mount-root `_INDEX.md` (`ROOT_INDEX_INO = 6`); tree-dir index inodes start at `7`. With at most `0xFFFF - 7 = 65528` tree dirs this is more than enough for any real Confluence space. [ASSUMED — no hard spec, but 65528 dirs is defensible]

**Recommended allocation layout:**

| Constant | Value | Purpose |
|----------|-------|---------|
| `ROOT_INDEX_INO` | `6` | Mount-root `_INDEX.md` |
| `TREE_INDEX_ALLOC_START` | `7` | First dynamically-allocated tree-dir index inode |
| `TREE_INDEX_ALLOC_END` | `0xFFFF` | Last valid (inclusive) |

Add to `inode.rs` alongside existing constants. Update the inode layout doc table. Update `reserved_range_is_unmapped` test to only assert `8..=0xFFFF` are unmapped (or exclude the dynamically allocated range — see note in §Common Pitfalls). [VERIFIED: test at inode.rs:201-213 currently asserts `6..=0xFFFF`; must be narrowed]

**Option C — Allocate tree-dir index inodes from `TreeSnapshot`.**
Add an `index_ino` field to `TreeDir` and allocate from a new `TREE_INDEX_INO_BASE` in the `TREE_DIR` range. Clean but requires touching `tree.rs` (pure module) and wiring the new inodes through the snapshot's resolution maps. More surgical but higher blast radius. Not recommended for this phase.

**Decision for planner:** Use Option B.

### Tree-Level `_INDEX.md` DFS Render Algorithm

`TreeSnapshot` after `build` is guaranteed cycle-free (cycles broken in `effective_parent_of`). Walk the subtree rooted at a `TreeDir` using a depth-annotated stack:

```rust
// Source: derived from tree.rs TreeDir/TreeEntry structure [VERIFIED]
fn render_tree_index(
    root_dir: &TreeDir,
    snapshot: &TreeSnapshot,
    project: &str,
    generated_at: chrono::DateTime<chrono::Utc>,
) -> Vec<u8> {
    use std::fmt::Write as _;
    // DFS stack: (entries_slice, depth_relative_to_root)
    let mut stack: Vec<(&[TreeEntry], usize)> = vec![(root_dir.children.as_slice(), 0)];
    let mut rows: Vec<(usize, String, String)> = Vec::new(); // (depth, name, path)
    while let Some((entries, depth)) = stack.pop() {
        for entry in entries.iter().rev() { // rev to maintain left-to-right order on pop
            match entry {
                TreeEntry::Symlink { name, target, .. } => {
                    rows.push((depth, name.clone(), target.clone()));
                }
                TreeEntry::Dir(ino) => {
                    if let Some(dir) = snapshot.resolve_dir(*ino) {
                        rows.push((depth, dir.name.clone() + "/", String::new()));
                        stack.push((&dir.children, depth + 1));
                    }
                }
            }
        }
    }
    // render YAML frontmatter + outline table
    // …
}
```

**Recursion depth:** Full DFS (all descendants). Justification: the primary use case is "show me the entire subtree in one `cat`" — shallow indexes can be obtained from `ls`. For deep Confluence spaces (hundreds of pages), output may be long but never causes memory exhaustion since we're just iterating the already-built in-memory snapshot. [ASSUMED — design decision; user constraint open]

**Format for tree-level rows:** Indented markdown heading or pipe-table with `depth | name | path` columns. Pipe-table is consistent with bucket-level index (LD-15-02). Recommended:

```markdown
---
kind: tree-index
project: demo
subtree: reposix-demo-space-home
entry_count: 7
generated_at: 2026-04-14T17:15:00Z
---

# Subtree index: tree/reposix-demo-space-home/

| depth | name | target |
| --- | --- | --- |
| 0 | _self.md | ../../pages/00000131192.md |
| 1 | child-page.md | ../../../pages/00000131193.md |
…
```

### Mount-Root `_INDEX.md` Format

Simple overview; no DFS needed. Derived entirely from already-held state:

```markdown
---
kind: mount-index
backend: confluence
project: demo
bucket: pages
issue_count: 42
generated_at: 2026-04-14T17:15:00Z
---

# Mount index — demo

| entry | kind | count |
| --- | --- | --- |
| .gitignore | file | — |
| pages/ | directory | 42 |
| tree/ | directory | 42 |
```

`issue_count` from `self.cache.len()` (same source as bucket index). `tree/` is only listed when `should_emit_tree()` is true. [VERIFIED: should_emit_tree() logic at fs.rs:577-590]

### lookup dispatch for `_INDEX.md` inside a `TreeDir`

`lookup(parent=TreeDir_ino, name="_INDEX.md")` must be handled in the `InodeKind::TreeDir` arm:

```rust
// In lookup(), InodeKind::TreeDir branch:
if name_str == TREE_DIR_INDEX_FILENAME {
    let index_ino = self.tree_dir_index_ino(parent_u); // allocate-or-lookup
    let bytes = self.tree_dir_index_bytes_or_render(parent_u, &snap);
    let attr = self.synthetic_file_attr(index_ino, bytes.len() as u64);
    reply.entry(&ENTRY_TTL, &attr, fuser::Generation(0));
    return;
}
```

`tree_dir_index_ino` is a helper that does `tree_index_inodes.entry(dir_ino).or_insert(alloc_next())`. [ASSUMED — exact method name; pattern is verified from codebase]

### readdir dispatch for `_INDEX.md` inside a `TreeDir`

In the `InodeKind::TreeDir` arm of `readdir`, emit `_INDEX.md` immediately after `.` and `..`, before the real entries — same ordering discipline as bucket index. [VERIFIED: bucket readdir at fs.rs:1006-1020]

### Recommended Project Structure (changes only)

```
crates/reposix-fuse/src/
├── inode.rs        # Add ROOT_INDEX_INO=6, TREE_INDEX_ALLOC_START=7
├── fs.rs           # Add InodeKind variants, render fns, cache fields, dispatch arms
└── (tree.rs)       # No changes required — snapshot is already cycle-free
```

### Anti-Patterns to Avoid

- **Visiting TreeSnapshot dirs recursively by inode**: the snapshot's `dirs` map is keyed by inode; use `resolve_dir` rather than iterating the map directly (ordering is undefined). Use the DFS stack pattern above.
- **Re-implementing cycle detection in the render path**: the snapshot is already a forest. Cycle detection is done once at `build` time. No need for a `visited` set during render DFS.
- **Using `self.cache.iter()` for tree-index content**: the cache contains bucket issues, not tree structure. Use `TreeSnapshot` for tree-index DFS; use `self.cache.len()` only for counts.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Pipe-table cell escaping | custom escaper | `escape_index_cell` (already in fs.rs:282) | Already handles `\|` and newline-fold |
| Inode allocation | custom allocator | `AtomicU64` + `DashMap` (same pattern as `InodeRegistry`) | Thread-safe, already in workspace |
| Cycle detection in DFS | visited-set during render | Already done in `TreeSnapshot::build` | Build-time guarantees forest structure |

---

## Common Pitfalls

### Pitfall 1: `reserved_range_is_unmapped` test breaks
**What goes wrong:** The test at `inode.rs:201-213` currently asserts ALL inodes `6..=0xFFFF` are unmapped by `InodeRegistry`. Adding dynamic tree-dir index allocation in that range means the test must be updated.
**Why it happens:** Test pins invariant that was correct before this phase.
**How to avoid:** Narrow the assertion to `(TREE_INDEX_ALLOC_START + N)..=0xFFFF` after seeding N dirs, or restructure the test to only assert `InodeRegistry` (dynamic registry) never allocates there — which remains true since tree-dir index inodes are allocated by a separate `AtomicU64`.
**Warning signs:** Clippy/test failure on `reserved_range_is_unmapped`.

### Pitfall 2: `getattr` on tree-dir index inode hit with unknown inode
**What goes wrong:** If `getattr(ino=X)` is called with a tree-dir index inode before `lookup` has been called (e.g. after a mount restart with kernel dentry cache), the `DashMap` won't have the entry yet and the callback returns `ENOENT`.
**Why it happens:** The kernel may call `getattr` without a prior `lookup` (e.g. open-by-path). The lazy allocation only fires on `lookup`.
**How to avoid:** In `getattr`, for inodes in the `7..=0xFFFF` range, reverse-look up the dir from the `tree_index_inodes` map (reverse mapping: index_ino → dir_ino), then render. Or pre-allocate index inodes for all tree dirs during `refresh_issues`.
**Warning signs:** `stat mount/tree/<subdir>/_INDEX.md` after remount returns ENOENT.

### Pitfall 3: `readdir` parent inode for `_INDEX.md` inside tree dirs
**What goes wrong:** The tree-dir `readdir` callback currently hardcodes `TREE_ROOT_INO` as the `..` entry inode (fs.rs:1064). If the DFS walk visits subdirs, the `..` inode is wrong for deeper nodes — cosmetic issue only.
**Why it happens:** Phase 13 accepted this as cosmetic (comment at fs.rs:1062-1064). No change required.
**Warning signs:** None for this phase; `ls -la mount/tree/<deep>/..` may show wrong parent inode but `cd ..` works fine.

### Pitfall 4: Empty tree snapshot renders empty tree-level `_INDEX.md`
**What goes wrong:** If `tree/` is not populated (backend doesn't support hierarchy), but somehow `_INDEX.md` is requested inside a tree dir, the render will see an empty entry list.
**Why it happens:** Not a real issue — if there are no tree dirs, there's no `lookup` target and no `readdir` entry emitting `_INDEX.md`. The render path is only reachable when a valid `TreeDir` exists.
**How to avoid:** No action needed; document for clarity.

---

## Code Examples

### render_bucket_index signature (model for new render fns)
```rust
// Source: crates/reposix-fuse/src/fs.rs:324-365
fn render_bucket_index(
    issues: &[Issue],
    backend_name: &str,
    project: &str,
    bucket: &str,
    generated_at: chrono::DateTime<chrono::Utc>,
) -> Vec<u8>
```

### TreeDir children iteration (verified)
```rust
// Source: crates/reposix-fuse/src/tree.rs:100-137
// TreeDir.children is Vec<TreeEntry>
// TreeEntry::Dir(u64) — interior dir inode
// TreeEntry::Symlink { ino, name, target } — leaf
// TreeSnapshot::resolve_dir(ino) -> Option<&TreeDir>
// TreeSnapshot::root_entries() -> &[TreeEntry]
```

### DashMap lazy index-inode allocator pattern
```rust
// Mirrors InodeRegistry::intern() at inode.rs:86-106
// Use AtomicU64 for allocator + DashMap<u64,u64> for dir_ino -> index_ino mapping
// Add to ReposixFs struct alongside bucket_index_bytes
```

### Cache invalidation on refresh
```rust
// Source: crates/reposix-fuse/src/fs.rs:740-745
// Drop bucket_index_bytes on refresh — add parallel drops for:
//   mount_root_index_bytes: RwLock<Option<Arc<Vec<u8>>>>
//   tree_dir_index_cache: DashMap<u64, Arc<Vec<u8>>>  (just .clear())
```

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Full DFS (all descendants) for tree-level `_INDEX.md` | Architecture Patterns | If user wants shallow-only, render function needs depth limit param |
| A2 | `ROOT_INDEX_INO = 6`, tree-dir inodes start at `7` from reserved range | Architecture Patterns | If range is reserved for something else, pick different values |
| A3 | `DashMap<u64,u64>` for dir-ino→index-ino mapping (Option B inode strategy) | Architecture Patterns | If Option C preferred, tree.rs must be modified |
| A4 | Tree-dir `_INDEX.md` cache via `DashMap<u64, Arc<Vec<u8>>>` | Architecture Patterns | Memory overhead proportional to number of tree dirs; acceptable for <500 pages |

---

## Open Questions

1. **Recursion depth: full DFS vs. direct children only?**
   - What we know: CONTEXT.md says "recursive is more powerful but may be very long for deep Confluence spaces"
   - What's unclear: user preference
   - Recommendation: ship full DFS with no truncation (matching bucket index's "no truncation" policy); add a note to CONTEXT.md that depth-limiting is a future flag

2. **Mount-root `_INDEX.md`: include `tree/` entry count?**
   - What we know: `tree/` dir contains the same N pages as the bucket, just structured differently
   - What's unclear: whether showing `42` for both `pages/` and `tree/` is confusing
   - Recommendation: include `tree/` row but show count as `—` (same as `.gitignore`) since `tree/` contains dirs and symlinks, not a flat list of N items

3. **`reserved_range_is_unmapped` test update strategy?**
   - Recommendation: restructure to verify `InodeRegistry` never allocates in `6..=0xFFFF` (the registry is separate from the new `AtomicU64` allocator for synthetic inodes). The test currently uses `r.lookup_ino(ino)` which tests `InodeRegistry` — so the invariant remains correct as long as we don't add the new allocator to `InodeRegistry`.

---

## Environment Availability

Step 2.6: SKIPPED (phase is pure Rust code changes, no new external dependencies).

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `cargo test` |
| Config file | none — workspace Cargo.toml |
| Quick run command | `cargo test -p reposix-fuse --quiet` |
| Full suite command | `cargo test --workspace --quiet` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| INDEX-01 | `render_tree_index` pure fn produces valid YAML frontmatter + table for a known `TreeSnapshot` | unit | `cargo test -p reposix-fuse render_tree_index` | ❌ Wave 0 |
| INDEX-01 | DFS visits all descendants (not just direct children) | unit | `cargo test -p reposix-fuse tree_index_full_dfs` | ❌ Wave 0 |
| INDEX-01 | Empty subtree renders valid doc with `entry_count: 0` | unit | `cargo test -p reposix-fuse tree_index_empty` | ❌ Wave 0 |
| INDEX-01 | Lookup `_INDEX.md` inside TreeDir returns non-ENOENT | integration (in-process) | `cargo test -p reposix-fuse tree_dir_index_lookup` | ❌ Wave 0 |
| INDEX-02 | `render_mount_root_index` pure fn produces valid doc | unit | `cargo test -p reposix-fuse render_mount_root_index` | ❌ Wave 0 |
| INDEX-02 | Mount-root lookup of `_INDEX.md` returns correct attr | unit | `cargo test -p reposix-fuse mount_root_index_lookup` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p reposix-fuse --quiet`
- **Per wave merge:** `cargo test --workspace --quiet && cargo clippy --workspace --all-targets -- -D warnings`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] Unit tests for `render_tree_index` — cover `fs.rs` test module
- [ ] Unit tests for `render_mount_root_index` — cover `fs.rs` test module
- [ ] Existing `reserved_range_is_unmapped` test in `inode.rs` must be updated to remain valid after new constants

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | yes | `escape_index_cell` (existing) applied to all title/name fields in render fns |
| V4 Access Control | yes | New synthetic files must return `EROFS` on all write callbacks (same as `BucketIndex`) |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Tainted page title in `_INDEX.md` body | Tampering / Info Disclosure | `escape_index_cell` — already in codebase, must be applied to tree-index rows |
| Symlink target in `_INDEX.md` table cell | Tampering | Target bytes come from `TreeEntry::Symlink.target` which is constructed from controlled inputs only (no title bytes) — safe per T-13-05 |
| `_INDEX.md` write attempt on tree-dir index | Tampering | `EROFS` in `write`/`setattr`/`create` callbacks — same pattern as `BucketIndex` |

---

## Sources

### Primary (HIGH confidence)
- `crates/reposix-fuse/src/fs.rs` — full bucket-level `_INDEX.md` implementation verified; read path, render function, cache pattern, dispatch arms all read directly
- `crates/reposix-fuse/src/inode.rs` — inode layout, reserved range `6..=0xFFFF`, existing constants verified
- `crates/reposix-fuse/src/tree.rs` — `TreeSnapshot` struct, `TreeDir`, `TreeEntry`, `build` (cycle-safe), `resolve_dir`, `resolve_symlink` all verified
- `.planning/phases/15-dynamic-index-md-synthesized-in-fuse-bucket-directory-op-2-p/15-CONTEXT.md` — Phase 15 locked decisions verified

### Secondary (MEDIUM confidence)
- CONTEXT.md Phase 18 — design questions and open questions read directly

---

## Metadata

**Confidence breakdown:**
- Standard Stack: HIGH — no new deps; existing workspace imports confirmed
- Architecture: HIGH — patterns cloned from verified Phase 15 implementation
- Inode Strategy (Option B): MEDIUM — design choice not locked; alternatives exist
- Pitfalls: HIGH — derived from actual code inspection

**Research date:** 2026-04-14
**Valid until:** 2026-05-14 (stable codebase, no external deps)
