# Phase 19: OP-1 Remainder — Labels + Spaces Directory Views — Research

**Researched:** 2026-04-15
**Domain:** FUSE overlay extension (reposix-fuse), inode layout, Confluence labels API
**Confidence:** HIGH (all claims verified against source code in this session)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions (from Phase 13)
- **Symlinks, not duplicate files** (ADR-003): canonical file in bucket; overlays are symlinks.
- **Read-only overlays:** Writes go through the symlink target (the canonical bucket file).
- **Slug-deduplication:** `dedupe_siblings` from `crates/reposix-fuse/src/tree.rs` must be reused for label-scoped issue lists.

### Claude's Discretion
- Label fetch strategy: lazy (on first `ls labels/`) vs eager (fetched on mount).
- `recent/` scope: CONTEXT.md says defer unless trivial after labels/ lands.
- Symlink target format: same relative `../../issues/<padded-id>.md` as `tree/`, or alternate.
- `.gitignore` update for `/labels/\n` and `/spaces/\n`.

### Deferred Ideas (OUT OF SCOPE)
- `recent/<yyyy-mm-dd>/` directory view.
- Confluence-specific attachment label views (OP-9b scope).
- Write semantics for `mv labels/bug/0001.md labels/p1/0001.md`.
- Multi-space mount via `--project '*'` (OP-9 scope).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| LAB-01 | `mount/labels/` directory present at mount root for all backends | `InodeKind` dispatch needs two new fixed inodes; pattern follows existing `Tree` arm |
| LAB-02 | `mount/labels/<label>/` subdirectory per distinct label, populated from `Issue::labels` | `Issue.labels: Vec<String>` already populated by sim and GitHub adapter; Confluence deferred (labels field is `vec![]`) |
| LAB-03 | Each entry inside `mount/labels/<label>/` is a symlink pointing to `../../<bucket>/<padded-id>.md` | Depth is 2: `labels/` (depth 0 from root) then `<label>/` (depth 1); target formula: `../../<bucket>/<padded-id>.md` |
| LAB-04 | `labels/` overlay is strictly read-only | Consistent with `tree/`; same `EROFS` + `EPERM` guards in write callbacks |
| LAB-05 | Slug collision inside a label group resolved via `dedupe_siblings` | Already in `reposix_core::path`; no new code needed |
| SPC-01 | `mount/spaces/` directory present at mount root when backend supports multiple spaces | New `BackendFeature::MultiSpace` variant OR gated on `ConfluenceBackend` only via a new optional trait method |
| SPC-02 | `mount/spaces/<key>/` subdirectory per space lists all pages in that space as symlinks | Requires `GET /wiki/api/v2/spaces` + per-space page list; significant new Confluence adapter code |
| SPC-03 | `spaces/` overlay is strictly read-only | Same discipline as `labels/` and `tree/` |
</phase_requirements>

---

## Summary

Phase 13 shipped `tree/` as a parentId-hierarchy overlay. Phase 19 adds two more
read-only overlay directories: `labels/` (grouping issues/pages by their label
strings) and, conditionally, `spaces/` (a Confluence-only multi-space view).

**Labels overlay is straightforward.** The `Issue` struct already carries
`pub labels: Vec<String>` [VERIFIED: crates/reposix-core/src/issue.rs:64].
The GitHub adapter already populates it from `GhLabel.name`
[VERIFIED: crates/reposix-github/src/lib.rs:131–142]. The Confluence adapter
explicitly defers it (`labels: vec![]`, with doc comment "deferred — labels live
on a separate endpoint") [VERIFIED: crates/reposix-confluence/src/lib.rs:26].
The simulator stores labels as a JSON TEXT column and round-trips them correctly
[VERIFIED: crates/reposix-sim/src/routes/issues.rs:45–81]. So for sim + GitHub,
all data needed for `labels/` is present in the existing `list_issues()` response.
No new backend method or trait extension is required for the labels view.

**Spaces overlay is significantly more complex.** The current `IssueBackend` trait
has no concept of "list available spaces/projects". Each backend is constructed with
a single project slug and `list_issues` fetches within that project.
`GET /wiki/api/v2/spaces` would need a new optional method or a separate
`SpaceAwareBackend` trait, new inode ranges, and a new FUSE dispatch arm. This is
substantial new surface area that will likely surface unexpected edge cases.

**Scope recommendation: ship `labels/` in Phase 19; defer `spaces/` to Phase 20.**
The labels view has all data in hand (no new backend API surface), fits cleanly
into the existing `InodeKind` dispatch pattern, and satisfies the primary OP-1
deliverable. `spaces/` is a Confluence-only feature requiring new trait surface,
new HTTP calls, and has a meaningful design question (does it show ALL spaces or
is it a supplementary index over the current space?). Deferring avoids scope creep.

**Primary recommendation:** Implement `labels/` only in Phase 19. `spaces/` is
a separate Phase 20 scope unit.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Label-to-issue index | FUSE daemon (in-memory) | — | Labels already in `list_issues()` payload; no extra HTTP needed |
| Symlink target generation | FUSE daemon | reposix-core/path | Follows `tree/` pattern; depth-aware relative paths |
| Inode allocation for `labels/` dirs | FUSE daemon (new constant + AtomicU64 allocator) | inode.rs layout doc | Two new fixed inodes + dynamic range for label-dir inodes |
| Label slug safety | reposix-core/path (dedupe_siblings) | — | Already used by tree/; reuse without changes |
| Confluence labels data | Confluence adapter (deferred) | — | `GET /wiki/api/v2/pages?label=X` — not implemented in v0.6 |
| spaces/ feature | Confluence adapter (new method, Phase 20) | — | Requires new optional trait method and HTTP endpoint |

---

## Standard Stack

No new crates are required. All needed building blocks already exist in the
workspace.

### Core (existing — no version changes)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `fuser` | 0.17 | FUSE callbacks | Already used; `default-features=false` per CLAUDE.md |
| `dashmap` | (workspace) | Concurrent map for label-dir inode cache | Already used for `cache`, `write_buffers`, `tree_index_inodes` |
| `reposix-core` | workspace | `Issue`, `IssueId`, `dedupe_siblings`, `slug_or_fallback` | All needed helpers already present |

**No `cargo add` commands needed.**

---

## Architecture Patterns

### System Architecture Diagram

```
list_issues()
    │
    ▼
[Issue list cache] ──► refresh_issues()
    │                       │
    │              rebuild tree snapshot  ◄── (existing)
    │              rebuild label index    ◄── (NEW: HashMap<String, Vec<IssueId>>)
    │
    ▼
FUSE dispatch (InodeKind::classify)
    │
    ├── ROOT readdir ──► emit "labels/" entry (LABELS_ROOT_INO)
    │
    ├── LabelsRoot readdir ──► iterate label_index.keys(), emit per-label dir inodes
    │
    ├── LabelDir readdir ──► iterate label_index[label], emit symlink entries
    │
    ├── LabelSymlink readlink ──► return "../../<bucket>/<padded-id>.md"
    │
    └── (existing tree/ / bucket/ arms unchanged)
```

### Recommended Project Structure

No new files needed. All changes land in existing files:

```
crates/
├── reposix-fuse/src/
│   ├── inode.rs     ← add LABELS_ROOT_INO, LABELS_DIR_INO_BASE, LABELS_SYMLINK_INO_BASE constants
│   ├── fs.rs        ← add InodeKind arms, LabelIndex struct, label snapshot field on ReposixFs
│   └── labels.rs    ← NEW: LabelSnapshot (mirrors tree.rs structure for labels)
└── reposix-core/
    └── (no changes needed)
```

### Pattern 1: Label Snapshot (mirrors TreeSnapshot)

The `labels/` overlay is simpler than `tree/` because:
- There is no hierarchy — every label is a flat list of symlinks.
- No `_self.md` entries (no page-that-is-also-a-dir duality).
- No cycle risk.
- Depth is always exactly 1 (`labels/<label>/` is depth 0 inside the labels root).

**LabelSnapshot structure:**

```rust
// Source: mirrors crates/reposix-fuse/src/tree.rs pattern
pub struct LabelSnapshot {
    /// Map from label string → vec of (symlink_ino, padded_id_filename, slug).
    /// slug is the deduped name that readdir returns; padded_id_filename is
    /// the target file component.
    label_entries: HashMap<String, Vec<LabelEntry>>,
    /// Reverse: label dir inode → label string (for readdir dispatch).
    label_dirs: HashMap<u64, String>,
    /// Reverse: symlink inode → target string (for readlink dispatch).
    symlink_targets: HashMap<u64, String>,
    /// Next dir inode allocator (starts at LABELS_DIR_INO_BASE).
    // NOTE: these are allocated at snapshot build time, not per-call.
}

pub struct LabelEntry {
    pub symlink_ino: u64,
    pub slug: String,       // deduped via dedupe_siblings
    pub target: String,     // e.g. "../../pages/00000131192.md"
}
```

**Build algorithm:**

```rust
// Source: mirrors TreeSnapshot::build in crates/reposix-fuse/src/tree.rs
pub fn build(bucket: &str, issues: &[Issue]) -> LabelSnapshot {
    // Step 1: collect (label → vec<(IssueId, slug)>) from all issues
    let mut by_label: HashMap<String, Vec<(IssueId, String)>> = HashMap::new();
    for issue in issues {
        for label in &issue.labels {
            by_label
                .entry(label.clone())
                .or_default()
                .push((issue.id, slug_or_fallback(&issue.title, issue.id)));
        }
    }
    // Step 2: dedupe slugs within each label group
    // Step 3: allocate dir inodes for each label, symlink inodes for each entry
    // Step 4: build reverse maps
    // Symlink target: "../../{bucket}/{padded-id}.md"
    //   depth=1 → 2 "../" hops (one for label-dir, one for labels-root)
}
```

**Symlink target depth:**

`labels/<label>/0001-title.md` → `../../issues/00000000001.md`

Depth inside labels root = 1 (one directory deep). The formula from `tree.rs`:
`"../".repeat(depth + 1) + bucket + "/" + padded_id + ".md"`
where `depth + 1 = 2`. [VERIFIED: crates/reposix-fuse/src/tree.rs:31-32]

### Pattern 2: InodeKind Extension

Two new fixed inodes (added to `inode.rs`):

| Constant | Value | Purpose |
|----------|-------|---------|
| `LABELS_ROOT_INO` | `8` | The `labels/` overlay root directory |
| `GITIGNORE_INO` | 4 | (existing — no change) |

Two new dynamic inode ranges (added after existing `TREE_SYMLINK_INO_BASE`):

| Range | Purpose |
|-------|---------|
| `0x10_0000_0000..0x14_0000_0000` | `labels/` interior dirs (one per distinct label) |
| `0x14_0000_0000..0x18_0000_0000` | `labels/` leaf symlinks |

These ranges are above `TREE_SYMLINK_INO_BASE` (`0xC_0000_0000`) and fully disjoint
from all existing ranges. `InodeKind::classify` gains two new match arms.
[VERIFIED: crates/reposix-fuse/src/inode.rs:1-18 — current layout ends at `0xC_0000_0000..u64::MAX`]

**CRITICAL:** The existing `TREE_SYMLINK_INO_BASE` range is declared as
`0xC_0000_0000..u64::MAX` (unbounded). The new label ranges must be carved out
ABOVE the existing tree-symlink allocator's realistic ceiling. Since
`TREE_SYMLINK_INO_BASE` starts at `0xC_0000_0000` and each symlink gets +1,
a space with 10,000 pages would use inodes up to `0xC_0000_2710`. The new
label range at `0x10_0000_0000` provides >16GB of headroom. This is safe.
[VERIFIED: crates/reposix-fuse/src/tree.rs:86-89]

### Pattern 3: ReposixFs Field Addition

```rust
// New field on ReposixFs (in fs.rs):
/// Label overlay snapshot. Rebuilt on each refresh_issues call.
/// Built in parallel with the tree snapshot.
label_snapshot: Arc<RwLock<LabelSnapshot>>,

/// `labels/` root directory attribute (perm 0o555, like tree/).
labels_attr: FileAttr,
```

The `refresh_issues` method gains one line after the tree rebuild:
```rust
let label_snap = LabelSnapshot::build(self.bucket, &issues);
if let Ok(mut guard) = self.label_snapshot.write() {
    *guard = label_snap;
}
```

### Pattern 4: FUSE Dispatch (readdir / lookup / getattr / readlink)

**Root readdir:** Emit `("labels", LABELS_ROOT_INO, Directory)` unconditionally
(same as `bucket` is always emitted). The overlay is only useful when issues have
labels, but an empty `ls labels/` is harmless and avoids the complexity of a
conditional gate.

**Root lookup:** `name == "labels"` → return `labels_attr`.

**LabelsRoot readdir:** Lock `label_snapshot`, iterate `label_dirs`, emit one
`Directory` entry per label (using the label string as the dir name, sanitized
through `slugify_title` for filesystem safety — see Pitfall 2 below).

**LabelDir readdir:** Lock `label_snapshot`, look up by dir inode, iterate
`LabelEntry` list, emit `Symlink` entries.

**readlink:** Lock `label_snapshot`, look up `symlink_targets[ino]`.

**getattr:** Label dirs get a clone of `labels_attr` with the label dir inode.
Label symlinks use `symlink_attr(ino, target)` (existing helper, unchanged).

### Pattern 5: .gitignore Update

The synthesized `.gitignore` currently serves `b"/tree/\n"` as a compile-time
constant (`GITIGNORE_BYTES`). With `labels/` added this must become
`b"/tree/\nlabels/\n"` (14 bytes). This is a breaking change to the constant
only — no structural change to the file inode or dispatch.

### Anti-Patterns to Avoid

- **Allocating new inode ranges inside the existing `TREE_SYMLINK_INO_BASE..u64::MAX` gap.** That range's upper bound is only `u64::MAX` because TreeSnapshot's allocator starts at `0xC_0000_0000` and counts up — it will never realistically reach `0x10_0000_0000`. But the classify() match arm `n >= TREE_SYMLINK_INO_BASE => TreeSymlink` would misclassify any labels-range inodes allocated between `0xC_0000_0000` and the label range start. **Always declare the label ranges above `TREE_SYMLINK_INO_BASE` AND update `classify()` to check label ranges BEFORE the `TreeSymlink` catch-all.**
- **Using label strings directly as FUSE dir names without sanitization.** GitHub/Confluence labels can contain spaces, slashes, colons. Run through `slugify_title` + `dedupe_siblings` when two labels collide after slugification.
- **Forgetting to invalidate the label snapshot on `refresh_issues`.** The tree snapshot is already invalidated and rebuilt; labels must follow the same pattern or stale data will linger.
- **Making `labels/` conditional on non-empty labels.** An empty `ls labels/` is harmless and avoids the complexity of a runtime gate. Make it unconditional.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Slug collision within a label group | Custom dedup | `reposix_core::path::dedupe_siblings` | Already tested, deterministic, handles 3+ colliders |
| Label slug sanitization | Custom validator | `reposix_core::path::slugify_title` | Security-tested against adversarial inputs (T-13-01/02) |
| Symlink target construction | Ad-hoc string format | Follow `tree.rs` depth formula: `"../".repeat(depth+1) + bucket + "/" + padded_id + ".md"` | Security-tested (T-13-05 in tree.rs) |
| Inode classification | New per-kind maps | Range-based `classify()` match arm | Existing pattern; O(1), no map lookup |

---

## Inode Strategy

### Current Layout (VERIFIED: crates/reposix-fuse/src/inode.rs)

| Range | Purpose |
|-------|---------|
| `1` | `ROOT_INO` |
| `2` | `BUCKET_DIR_INO` |
| `3` | `TREE_ROOT_INO` |
| `4` | `GITIGNORE_INO` |
| `5` | `BUCKET_INDEX_INO` |
| `6` | `ROOT_INDEX_INO` |
| `7..=0xFFFF` | Per-tree-dir `_INDEX.md` inodes |
| `0x1_0000..` | Real issue/page files |
| `0x8_0000_0000..0xC_0000_0000` | `tree/` interior directories |
| `0xC_0000_0000..u64::MAX` | `tree/` leaf symlinks (allocated from `TREE_SYMLINK_INO_BASE`) |

### Proposed Extension for Phase 19

| Constant | Value | Purpose |
|----------|-------|---------|
| `LABELS_ROOT_INO` | `8` | Fixed inode for the `labels/` root directory |
| `LABELS_DIR_INO_BASE` | `0x10_0000_0000` | Start of per-label dir inode range |
| `LABELS_SYMLINK_INO_BASE` | `0x14_0000_0000` | Start of label symlink inode range |

**Why `8` for LABELS_ROOT_INO:** The fixed range `1..=6` is fully occupied.
Slot `7` is `TREE_INDEX_ALLOC_START`. The range `7..=0xFFFF` is reserved for
per-tree-dir `_INDEX.md` inodes. `LABELS_ROOT_INO = 0x1_0000_0000` would work
but is far from the others. Using `8` is cleaner: it's the next available fixed
slot BEFORE the dynamic tree-index range starts at `7`. Wait — `7` is
`TREE_INDEX_ALLOC_START` and the range `7..=0xFFFF` is dynamically allocated.
So `8` through `0xFFFF` are part of the tree-index allocator's range. **Do NOT
use 8 as a fixed inode.** Instead, use `0x1_0000_0001` (one above
`FIRST_ISSUE_INODE`) — but that collides with the issue inode range.

**Correct approach:** Add `LABELS_ROOT_INO` as a new large fixed constant
above the issue inode range but below the tree-dir range. A clean choice:

| Constant | Value | Notes |
|----------|-------|-------|
| `LABELS_ROOT_INO` | `0x7_FFFF_FFFF` | Just below `TREE_DIR_INO_BASE` (`0x8_0000_0000`); guaranteed disjoint from issue inodes (`0x1_0000..`) and tree dirs (`0x8_0000_0000..`) |
| `LABELS_DIR_INO_BASE` | `0x10_0000_0000` | One full 4GB block above `TREE_SYMLINK_INO_BASE` |
| `LABELS_SYMLINK_INO_BASE` | `0x14_0000_0000` | One full 4GB block above `LABELS_DIR_INO_BASE` |

The existing compile-time assertions in `tree.rs` only check:
- `TREE_ROOT_INO < FIRST_ISSUE_INODE`
- `TREE_DIR_INO_BASE > FIRST_ISSUE_INODE`
- `TREE_DIR_INO_BASE < TREE_SYMLINK_INO_BASE`

Phase 19 must add analogous const assertions:
```rust
const _: () = {
    assert!(LABELS_ROOT_INO < TREE_DIR_INO_BASE);
    assert!(LABELS_ROOT_INO > crate::inode::FIRST_ISSUE_INODE + 0xFFFF);
    assert!(LABELS_DIR_INO_BASE > TREE_SYMLINK_INO_BASE);
    assert!(LABELS_DIR_INO_BASE < LABELS_SYMLINK_INO_BASE);
};
```

**`classify()` update — order matters:**
The current `TreeSymlink` arm is `n >= TREE_SYMLINK_INO_BASE`. It must remain
above the new label arms, OR the label ranges (starting at `0x10_0000_0000`)
must be checked BEFORE the `>= TREE_SYMLINK_INO_BASE` arm. Since
`0x10_0000_0000 > 0xC_0000_0000`, a new label arm must come BEFORE the
`TreeSymlink` arm in the `match`.

---

## Runtime State Inventory

This is a greenfield extension (new overlay directories), not a rename/refactor.
The `labels/` overlay is computed purely from the in-memory `list_issues()` result
with no persistent storage. No runtime state inventory is required.

---

## Common Pitfalls

### Pitfall 1: `classify()` arm ordering — label inodes misclassified as TreeSymlink

**What goes wrong:** The existing `TreeSymlink` match arm is `n >= TREE_SYMLINK_INO_BASE`
(a catch-all for the upper half of the u64 range). If label inode ranges
(`0x10_0000_0000` and `0x14_0000_0000`) are added WITHOUT a prior match arm that
checks for them, every label-dir and label-symlink inode will be routed to
`InodeKind::TreeSymlink` and return `ENOENT` from the tree snapshot lookup.

**Prevention:** Add the new `LabelsRoot`, `LabelDir`, `LabelSymlink` arms BEFORE
the `TreeSymlink` arm in `InodeKind::classify()`. Add a const assertion that
`LABELS_DIR_INO_BASE > TREE_SYMLINK_INO_BASE` to pin the ordering.

### Pitfall 2: Unsafe label strings as FUSE directory names

**What goes wrong:** GitHub and Confluence labels can contain spaces, colons,
forward slashes (e.g. `status/in-progress`, `Type: Bug`). Using the raw label
string as a FUSE directory name will either corrupt the path or crash the kernel
VFS layer.

**Prevention:** Apply `slug_or_fallback(label, synthetic_id)` to every label
string before using it as a directory name. When two different labels produce the
same slug (e.g. `Status: Bug` and `status bug` both → `status-bug`), use
`dedupe_siblings` within the labels root level. The `synthetic_id` for the fallback
can be a hash of the label string reduced to a u64 — or a sequential allocator;
determinism matters more than the specific value.

**Warning signs:** `readdir` returning entries with `/` in the name; FUSE mount
failing with `EINVAL` on `lookup`.

### Pitfall 3: Stale label snapshot after `refresh_issues`

**What goes wrong:** The issue cache is refreshed by `refresh_issues()` (called
at root/bucket/tree readdir time). If `label_snapshot` is not also rebuilt at
that point, a user who `ls labels/` after issues have been added/relabeled will
see stale entries.

**Prevention:** Rebuild `LabelSnapshot` inside `refresh_issues()`, immediately
after the `TreeSnapshot::build(...)` call. Follow the same `Arc<RwLock<>>` swap
pattern already in use for `tree`.

### Pitfall 4: `GITIGNORE_BYTES` is a compile-time `&[u8]` constant

**What goes wrong:** Adding `labels/` to the gitignore requires changing the
content from `"/tree/\n"` to `"/tree/\nlabels/\n"`. The current constant is
`const GITIGNORE_BYTES: &[u8] = b"/tree/\n"`. Changing it also changes `size`
in `gitignore_attr` (currently hardcoded to `GITIGNORE_BYTES.len()`). If the size
is not updated, `read()` will truncate the new content.

**Prevention:** `gitignore_attr.size` is derived from `GITIGNORE_BYTES.len()` at
construction time — since the constant drives both values this is safe as long as
the constant itself is updated. Confirm with a test that the rendered bytes equal
`b"/tree/\nlabels/\n"` exactly.

### Pitfall 5: Symlink size = 0 regression

**What goes wrong:** Phase 13 discovered that a 0-size symlink surfaces as a
0-byte file in `ls -l` (see 13-RESEARCH.md §Pitfall 1 / `13-REVIEW.md` IN-03).
Label symlinks must use `symlink_attr(ino, target)` which sets `size = target.len()`.
Do NOT construct `FileAttr` inline with `size: 0`.

**Prevention:** Reuse the existing `self.symlink_attr(ino, target)` helper for
every label symlink getattr. Add a unit test asserting size == target.len() for
a sample label entry.

---

## Code Examples

### LabelSnapshot::build (sketch)

```rust
// Mirrors crates/reposix-fuse/src/tree.rs — TreeSnapshot::build_with_events
use reposix_core::path::{dedupe_siblings, slug_or_fallback};
use reposix_core::{Issue, IssueId};

pub fn build(bucket: &str, issues: &[Issue]) -> LabelSnapshot {
    let mut by_label: HashMap<String, Vec<(IssueId, String)>> = HashMap::new();
    for issue in issues {
        for label in &issue.labels {
            by_label
                .entry(label.clone())
                .or_default()
                .push((issue.id, slug_or_fallback(&issue.title, issue.id)));
        }
    }

    let mut label_dirs: HashMap<u64, String> = HashMap::new();
    let mut symlink_targets: HashMap<u64, String> = HashMap::new();
    // slug → label (for deduping the label-dir names themselves)
    let mut label_slugs: Vec<(/* synthetic_id */ u64, String)> = by_label
        .keys()
        .enumerate()
        .map(|(i, k)| (i as u64, slug_or_fallback(k, IssueId(i as u64))))
        .collect();
    let deduped_label_names = dedupe_siblings(
        label_slugs.iter().map(|(i, s)| (IssueId(*i), s.clone())).collect()
    );

    let mut next_dir = LABELS_DIR_INO_BASE;
    let mut next_sym = LABELS_SYMLINK_INO_BASE;

    for ((label, entries), (_, dir_slug)) in by_label.into_iter().zip(deduped_label_names) {
        let dir_ino = next_dir; next_dir += 1;
        label_dirs.insert(dir_ino, dir_slug);

        let deduped = dedupe_siblings(entries);
        // depth = 1 → 2 "../" hops
        for (id, slug) in deduped {
            let target = format!("../../{bucket}/{:011}.md", id.0);
            let sym_ino = next_sym; next_sym += 1;
            symlink_targets.insert(sym_ino, target);
        }
    }
    LabelSnapshot { label_dirs, symlink_targets, /* ... */ }
}
```

### InodeKind::classify extension

```rust
// After the TREE_SYMLINK_INO_BASE arm, BEFORE the TreeSymlink catch-all:
n if n >= LABELS_SYMLINK_INO_BASE => Self::LabelSymlink,
n if n >= LABELS_DIR_INO_BASE    => Self::LabelDir,
n if n == LABELS_ROOT_INO        => Self::LabelsRoot,
// existing:
n if n >= TREE_SYMLINK_INO_BASE  => Self::TreeSymlink,
```

---

## Scope Recommendation

**Implement `labels/` only in Phase 19. Defer `spaces/` to Phase 20.**

### Why labels/ fits Phase 19

1. All data is in hand: `Issue::labels: Vec<String>` is populated by sim and GitHub.
2. No new backend trait methods required.
3. Pattern is a direct simplification of `tree/` (no hierarchy, no cycles).
4. New code is localized: a new `labels.rs` module + additions to `fs.rs` and `inode.rs`.
5. The `.gitignore` content update is a single constant change.

### Why spaces/ should be deferred

1. **New trait surface.** `IssueBackend` has no `list_projects()` or `list_spaces()` method.
   Adding one is an API-breaking change that needs a new `BackendFeature` variant and
   default implementation returning `Err("not supported")`.
2. **Confluence-only.** The sim and GitHub backends have no meaningful spaces concept.
   A `spaces/` overlay would only be populated for Confluence — unusual for a cross-backend
   feature.
3. **Design ambiguity.** The CONTEXT.md notes that `mount/spaces/<key>/` should list "all
   pages in that Confluence space" — but the current mount already has a project-scoped
   `pages/` bucket. Does `spaces/` show the current space plus others? Is it an alternative
   mount point? These design questions should be resolved in a CONTEXT.md discussion before
   implementation.
4. **New HTTP endpoints.** `GET /wiki/api/v2/spaces` requires pagination handling,
   credential scoping, and new wiremock fixtures for tests.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Flat mount root (`mount/<id>.md`) | Nested: `mount/issues/` or `mount/pages/` | Phase 13 | `labels/` and `spaces/` are Phase 19+ additions in this same nested layout |
| Hard-coded `/tree/\n` gitignore | Will need `/tree/\nlabels/\n` | Phase 19 | Single constant update |

**Deprecated/outdated:**
- None for this phase — it's purely additive.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `LABELS_ROOT_INO = 0x7_FFFF_FFFF` is free (below `TREE_DIR_INO_BASE = 0x8_0000_0000`, above issue inode range) | Inode Strategy | Inode collision; classify() misroutes lookup — would be caught by const assertion at compile time |
| A2 | Confluence labels endpoint (`GET /wiki/api/v2/pages?label=X`) is out of scope for Phase 19 (Confluence `Issue.labels` is `vec![]`) | Scope Recommendation | If Confluence labels are needed urgently, more backend work is required before `labels/` is useful for Confluence users |
| A3 | `ls labels/` with an empty snapshot (no issues have labels) should return an empty directory, not ENOENT | Architecture Patterns | If ENOENT is preferred for empty overlays, root `readdir` needs a conditional gate — adds complexity |
| A4 | The label string itself (before slugification) is what appears in the `_INDEX.md` sort key | Don't Hand-Roll | Minor: ordering in index would differ from filesystem slug ordering |

**If this table is empty:** N/A — four assumptions identified above.

---

## Open Questions

1. **Label dir naming when two labels produce the same slug**
   - What we know: `dedupe_siblings` handles slug collisions within a single parent.
   - What's unclear: The "synthetic id" used as the `IssueId` fallback in `slug_or_fallback`
     must be deterministic across mounts. Using the label's position in a sorted list of
     all label strings gives determinism.
   - Recommendation: Sort `by_label.keys()` alphabetically, assign synthetic IDs 0..N,
     then call `dedupe_siblings` on the (id, slug) pairs.

2. **Should `labels/` be listed in the root `_INDEX.md`?**
   - What we know: `render_mount_root_index` currently has rows for bucket, tree (conditional).
   - What's unclear: Should it add a `labels/ | directory | <count>` row?
   - Recommendation: Yes — emit a `labels/` row with the count of distinct labels.
     This is a one-line change to `render_mount_root_index`.

3. **Should empty label directories appear in `ls labels/`?**
   - What we know: An issue can have labels removed; the snapshot is rebuilt on `readdir`.
   - What's unclear: Edge cases around concurrent label mutation mid-readdir.
   - Recommendation: Since the snapshot is atomic (rebuilt whole and swapped), there are
     no concurrent mutation edge cases. Empty label directories simply won't appear because
     `by_label` only creates entries for labels that appear on at least one issue.

---

## Environment Availability

Step 2.6: SKIPPED — Phase 19 is a code/config-only extension to the FUSE daemon. No new
external tools, services, or CLIs are required. Existing prerequisites (fuser, fusermount3)
are already gated behind `--ignored` tests and documented in CLAUDE.md.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `cargo test` |
| Config file | `rust-toolchain.toml` (stable channel) |
| Quick run command | `cargo test -p reposix-fuse --lib` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| LAB-01 | `labels/` dir appears in root `readdir` | unit | `cargo test -p reposix-fuse --lib -- fs::tests::labels_root_in_readdir` | ❌ Wave 0 |
| LAB-02 | `ls labels/<label>/` returns symlink entries for all issues with that label | unit | `cargo test -p reposix-fuse --lib -- labels::tests::build_label_snapshot` | ❌ Wave 0 |
| LAB-03 | Symlink targets are `../../<bucket>/<padded-id>.md` | unit | `cargo test -p reposix-fuse --lib -- labels::tests::symlink_target_depth` | ❌ Wave 0 |
| LAB-04 | Write to label symlink returns EROFS | unit | `cargo test -p reposix-fuse --lib -- fs::tests::label_symlink_write_erofs` | ❌ Wave 0 |
| LAB-05 | Slug collisions within a label group get `-2`, `-3` suffixes | unit | `cargo test -p reposix-fuse --lib -- labels::tests::label_slug_dedup` | ❌ Wave 0 |
| LAB-03 | E2E: `readlink mount/labels/bug/00000000001.md` resolves to canonical file | integration | `cargo test -p reposix-fuse --release -- --ignored labels_e2e` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p reposix-fuse --lib`
- **Per wave merge:** `cargo test --workspace`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `crates/reposix-fuse/src/labels.rs` — `LabelSnapshot`, `LabelEntry`, `build()`; covers LAB-02, LAB-03, LAB-05
- [ ] Unit tests in `labels.rs` — `build_label_snapshot`, `symlink_target_depth`, `label_slug_dedup`
- [ ] Unit tests in `fs.rs` — `labels_root_in_readdir`, `label_symlink_write_erofs`
- [ ] Integration test stub — `crates/reposix-fuse/tests/labels_e2e.rs` (FUSE mount + wiremock sim with labeled issues)

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | `labels/` is read-only; `EROFS`/`EPERM` on write callbacks |
| V5 Input Validation | yes | Label strings from remote → run through `slugify_title` before use as FUSE dir name |
| V6 Cryptography | no | — |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Label string injection (label = `"../../etc"`) | Tampering | `slugify_title` strips all non-`[a-z0-9-]`; cannot produce path separators [VERIFIED: path.rs:96-133] |
| Symlink target escape (label symlink pointing outside mount) | Tampering | Target constructed from controlled sources only: literal `"../"` × 2, `bucket` (`&'static str` from backend), padded numeric id, `.md` — no label string bytes in target [VERIFIED: tree.rs:57-64, same pattern applies] |
| Inode squatting (malicious backend returns id that collides with `LABELS_ROOT_INO`) | Elevation of Privilege | `LABELS_ROOT_INO` is in a range unreachable by `InodeRegistry` (which starts at `FIRST_ISSUE_INODE = 0x1_0000` and counts up into issue-inode range only) |

---

## Sources

### Primary (HIGH confidence)
- `crates/reposix-fuse/src/inode.rs` — inode layout, constants, `InodeRegistry`
- `crates/reposix-fuse/src/tree.rs` — `TreeSnapshot`, `Builder`, symlink target formula, depth rules
- `crates/reposix-fuse/src/fs.rs` — `InodeKind::classify`, `readdir`, `lookup`, `getattr`, `readlink`, `refresh_issues`, `ReposixFs` struct fields
- `crates/reposix-core/src/issue.rs` — `Issue` struct, `labels: Vec<String>` field confirmed
- `crates/reposix-core/src/backend.rs` — `IssueBackend` trait, `BackendFeature` enum — no `Labels` or `Spaces` variants exist
- `crates/reposix-core/src/path.rs` — `dedupe_siblings`, `slug_or_fallback`, `slugify_title`
- `crates/reposix-github/src/lib.rs` — `GhLabel.name` mapped to `Issue.labels`
- `crates/reposix-confluence/src/lib.rs` — `labels: vec![]` with explicit "deferred" comment; `supports()` returns Hierarchy+Delete+StrongVersioning only
- `crates/reposix-sim/src/routes/issues.rs` — labels stored as JSON TEXT column, fully round-tripped

### Secondary (MEDIUM confidence)
- `crates/reposix-fuse/tests/nested_layout.rs` — integration test pattern for FUSE + wiremock; Phase 19 integration tests should follow this template

### Tertiary (LOW confidence)
- None.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — verified against source; no new crates needed
- Architecture: HIGH — derived from direct reading of fs.rs, inode.rs, tree.rs dispatch pattern
- Inode strategy: HIGH — computed from verified constants; const assertions provide compile-time safety net
- Pitfalls: HIGH — directly observed from existing code structure
- Confluence labels status: HIGH — explicit `labels: vec![]` + doc comment in confluence/src/lib.rs

**Research date:** 2026-04-15
**Valid until:** 90 days (stable internal codebase; no external dependencies)
