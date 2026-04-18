# Phase 13: Nested mount layout — Research

**Researched:** 2026-04-14
**Domain:** FUSE synthesized symlinks + Confluence REST v2 hierarchy + deterministic slug generation
**Confidence:** HIGH (codebase claims + fuser 0.17.0 source inspected directly + Atlassian public docs)

## Summary

Phase 13 adds a per-backend writable bucket (`pages/` for Confluence, `issues/` for sim/GitHub) plus a synthesized read-only `tree/` overlay of FUSE symlinks that mirror Confluence's native parentId hierarchy. All design decisions are **locked by user sign-off pre-sleep** (see CONTEXT.md). This research investigates only the mechanics the planner needs:

1. **fuser 0.17.0 is the actual dep** (not 0.15 as CLAUDE.md states — the comment is stale; `Cargo.lock` shows `version = "0.17.0"`, and `crates/reposix-fuse/Cargo.toml` pins `version = "0.17"`). A synthesized read-only symlink needs exactly three callbacks: `lookup`, `getattr`, `readlink`. No `mknod`/`symlink`/`mkdir` overrides needed — those are for write-path creation, which we explicitly don't want.
2. **Confluence REST v2** returns `parentId: String` and `parentType: "page" | "folder" | "whiteboard" | "database" | ...`. We filter to `parentType == "page"` (or absent) and treat everything else as orphan for this phase.
3. **Zero new deps needed** for slugification. ASCII-only `[a-z0-9-]+` with 60-byte truncation is sufficient for Confluence title content and keeps the trust surface minimal.

**Primary recommendation:** Implement `readlink` + extend `lookup`/`getattr` to return `FileType::Symlink` for tree nodes. Partition the inode space into three disjoint ranges (root bucket, tree dirs, tree symlinks) so a u64 inode immediately encodes its node kind. Cache the tree snapshot at `readdir("tree/")` time; never mutate it from write-path callbacks.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **Per-backend root collection name.** New trait method `IssueBackend::root_collection_name(&self) -> &'static str`. Default `"issues"`. Confluence overrides to `"pages"`.
- **Two top-level dirs at mount root:** `<root_collection>/` (writable, real files) and `tree/` (read-only symlinks, emitted iff the backend reports `BackendFeature::Hierarchy` OR ≥1 page has a parent_id). Plus a synthesized `.gitignore`.
- **Real files live only in `<root_collection>/<padded-id>.md`** — single source of truth.
- **`tree/` contains FUSE symlinks only.** Targets are depth-correct relative paths so `readlink -f` resolves inside the mount.
- **Symlinks dissolve merge-conflict hell.** Title renames, reparenting, sibling reshuffles only change symlink NAMES and LOCATIONS; the stable numeric-id target never moves.
- **`Issue::parent_id: Option<IssueId>`** — new field on the core type. GitHub/sim always return `None`; Confluence populates from REST v2 `parentId`.
- **Slug algorithm** (locked, implemented as `reposix_core::path::slugify_title(&str) -> String`):
  1. Unicode-lowercase.
  2. Replace every run of non-`[a-z0-9]` ASCII with a single `-`.
  3. Trim leading/trailing `-`.
  4. Truncate to 60 bytes.
  5. If empty/`.`/`..`/all-`-`, fall back to `page-<padded-id>`.
- **Collision resolution:** group siblings by slug → sort by ascending `IssueId` → first keeps bare slug, Nth gets suffix `-{N}` (2, 3, 4, …).
- **Interior node titles ARE dir names.** Page with ≥1 child → directory `foo/` containing `_self.md` symlink (parent's own body) + child symlinks/dirs as siblings. `_self` reserved by construction (leading `_` is stripped by step 2 → slugify_title never emits it).
- **Hierarchy feature via `BackendFeature::Hierarchy`** — defaults `false`, Confluence overrides `true`. No new trait method for "give me the tree" — FUSE groups in-memory from `list_issues`.
- **Orphan parents** (parent_id points outside the mounted list) → treated as tree roots.
- **`.gitignore` at mount root** always synthesized with content `/tree/\n`. Read-only virtual file; no collision detection with user-authored `.gitignore` (virgin-mount assumption).
- **BREAKING change documented in CHANGELOG** under v0.4.0. Every demo/README path goes from `mount/<id>.md` to `mount/<bucket>/<id>.md`.
- **Ship as v0.4.0.** Release script `scripts/tag-v0.4.0.sh`. CI release.yml auto-uploads prebuilt binaries on tag push.

### Claude's Discretion

- Inode allocation strategy for `tree/` nodes (separate ranges vs unified allocator with kind tags).
- Whether to cache the slug map in `Mount` at open vs recompute on each `readdir`.
- Whether to split a new module `crates/reposix-fuse/src/tree.rs` vs extending `inode.rs` in place (fs.rs is 848 lines — splitting is probably correct).
- Wiremock test matrix (which hierarchy shapes: deep, wide, orphan-parent, cycle).
- Whether to proptest the slug/collision resolver.

### Deferred Ideas (OUT OF SCOPE)

- `labels/`, `recent/`, `spaces/` views (Phase 14+).
- `INDEX.md` synthesis per dir (OP-2).
- `git pull` cache refresh (OP-3).
- Comments at `pages/<id>.comments/` (OP-9).
- Multi-space mount with `spaces/<key>/`.
- GitHub labels/milestones overlays.
- Write-through-tree semantics beyond default POSIX symlink resolution.
- Per-dir `readdir` streaming for huge spaces.
- GitHub tree/ (no parent metadata exposed).

</user_constraints>

## Project Constraints (from CLAUDE.md)

- `#![forbid(unsafe_code)]` in every crate — applies to new `tree.rs` module.
- `#![warn(clippy::pedantic)]` — allow-list any new lint exception with rationale.
- All public items documented; `# Errors` section on every `Result`-returning fn.
- Times use `chrono::DateTime<Utc>`; frontmatter uses `serde_yaml` 0.9; never JSON on disk.
- `fuser` stays `default-features = false` (no `libfuse-dev` on dev host, no passwordless sudo).
- **SG-03 egress discipline** — tree/ symlinks are inbound-only in this phase; writes go to pages/ via existing PATCH path, which is already SG-03-clean.
- **Tainted by default** — Confluence titles feed slug generation. Slug output must not be routed into any network-side-effect path. It only lands in readdir/readlink replies to the kernel, which is safe.
- **No hidden state** — tree snapshot must be reconstructible from a fresh `list_issues` call; no on-disk cache.
- **Mount point = git repo** — the `.gitignore` emission is the mechanism that honors this rule when `tree/` is present.

## Phase Requirements

CONTEXT.md does not enumerate REQ-IDs explicitly; the locked decisions serve as the requirement list. The planner should treat each "Locked Decision" bullet as a requirement to be validated.

## Standard Stack

### Core (already in-tree — no new deps)

| Library | Version | Purpose | Why Standard |
|---|---|---|---|
| `fuser` | 0.17.0 | FUSE trait (`Filesystem`, `FileType::Symlink`, `readlink`, `ReplyData`) | [VERIFIED: Cargo.lock line `name = "fuser"` version `0.17.0`]. Already used; `FileType::Symlink` variant confirmed in `/home/reuben/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/fuser-0.17.0/src/lib.rs:143`. |
| `dashmap` | existing | Concurrent inode/slug maps | Already used in `InodeRegistry`. |
| `tokio` | 1 | Runtime for async backend calls from FUSE callbacks | Already wired. |
| `chrono` | existing | Timestamps on synthesized `.gitignore` | Already used. |
| `serde_json` | existing | Extract `parentId`/`parentType` from Confluence response | Already used in `parse_next_cursor`. |
| `tracing::warn` | existing | Cycle detection, orphan logs | Already used. |

### Supporting — proposed additions (optional)

| Library | Version | Purpose | When to Use |
|---|---|---|---|
| `proptest` | ^1 (dev-dep) | Property tests for slug collisions | If agent picks "proptest the resolver" from discretion list. |

### Alternatives Considered (rejected)

| Instead of | Could Use | Tradeoff |
|---|---|---|
| Hand-rolled ASCII slug | `slug` crate (~21M downloads, [CITED: docs.rs/slug]) | `slug` pulls `deunicode` which is a 500KB unicode transliteration table. Overkill for Confluence titles (which are typically ASCII-ish already). Zero-dep is consistent with the "minimum trust surface" project philosophy. |
| Hand-rolled ASCII slug | `deunicode` directly | Same reason. Transliteration of `"Æneid" → "AEneid"` is nice-to-have, not required. Confluence pages with non-ASCII titles still get a readable slug via the ASCII-fallback rule in step 5. |
| One unified inode range | Separate ranges per node kind | Unified is simpler but forces every `getattr`/`lookup` to consult a DashMap. Separate ranges (e.g. tree-dirs at `0x8_0000_0000..`, tree-symlinks at `0xC_0000_0000..`) let `getattr` branch on inode value immediately without a lookup. Agent's discretion per CONTEXT.md. |

**Installation:** no new runtime deps. Optional: `proptest` as dev-dep in `reposix-core/Cargo.toml`.

**Version verification:** fuser 0.17.0 confirmed against Cargo.lock. `FileType::Symlink` confirmed via direct source inspection; readlink signature confirmed as `fn readlink(&self, _req: &Request, ino: INodeNo, reply: ReplyData)` (returns target bytes via `reply.data(&bytes)`).

## Architecture Patterns

### Recommended Project Structure

```
crates/reposix-core/src/
├── backend.rs              # extend: add BackendFeature::Hierarchy, root_collection_name()
├── issue.rs                # extend: add parent_id: Option<IssueId>
└── path.rs                 # extend: add slugify_title(), sibling_dedupe()

crates/reposix-confluence/src/
└── lib.rs                  # extend: deserialize parentId + parentType; filter to page-parents

crates/reposix-fuse/src/
├── fs.rs                   # extend: dispatch on inode range; add readlink(); root lookup
├── inode.rs                # extend: add tree-dir + tree-symlink namespaces
└── tree.rs                 # NEW: TreeSnapshot type; build_tree(&[Issue]) -> TreeSnapshot;
                            # resolve_tree_inode(ino) -> TreeNode; path-to-target for symlinks
```

### Pattern 1: Inode-range dispatch

**What:** Partition u64 inode space by node kind. `getattr`/`lookup`/`readlink` branch on the range before any map lookup.

**When to use:** FUSE filesystems with multiple synthesized namespaces (tree view vs flat view).

**Example:**
```rust
// Suggested layout (agent's discretion per CONTEXT.md):
const ROOT_INO: u64 = 1;
const BUCKET_DIR_INO: u64 = 2;           // pages/ or issues/ dir inode
const TREE_DIR_INO: u64 = 3;             // tree/ root dir inode
const GITIGNORE_INO: u64 = 4;            // synthesized .gitignore
// 0x1_0000..0x8_0000_0000 -> real files in bucket (existing FIRST_ISSUE_INODE scheme)
// 0x8_0000_0000..0xC_0000_0000 -> tree/ interior dirs
// 0xC_0000_0000..            -> tree/ leaf symlinks
```

### Pattern 2: Synthesized read-only symlink (the core mechanic)

**What:** A symlink that has no on-disk existence. `lookup` returns `FileAttr { kind: FileType::Symlink, size: target_bytes.len(), ... }`; `readlink(ino)` returns the target bytes.

**When to use:** Every tree/ leaf.

**Example** (adapted from the fuser 0.17.0 contract, trait source at `fuser-0.17.0/src/lib.rs:469`):
```rust
fn readlink(&self, _req: &Request, ino: INodeNo, reply: ReplyData) {
    match self.tree.resolve_symlink(ino.0) {
        Some(target_rel_path) => reply.data(target_rel_path.as_bytes()),
        None => reply.error(fuser::Errno::from_i32(libc::ENOENT)),
    }
}
```

The kernel handles path resolution after `readlink` returns. A relative target like `../pages/00000131192.md` is resolved by the VFS in the same FUSE mount — no special cross-mount logic needed, because the target stays inside our FS (`readlink(2)` just returns bytes; the kernel does `path_lookup` from there). [CITED: man7.org/linux/man-pages/man2/readlink.2.html]

### Pattern 3: Symlink size = target length (POSIX)

**What:** `st_size` for a symlink equals the byte length of the target string, not 0. Getting this wrong causes `ls -l` to show size 0 (like the restic bug at [CITED: github.com/restic/restic/issues/3667]).

**When to use:** Every `FileAttr` returned for a symlink inode.

**Example:**
```rust
let target = "../pages/00000131192.md";
FileAttr {
    ino: INodeNo(symlink_ino),
    size: target.len() as u64,   // <-- MUST be target bytes length
    blocks: 0,
    kind: FileType::Symlink,
    perm: 0o777,                  // symlink perm is conventionally 0777 (ignored by kernel)
    nlink: 1,
    ..
}
```

### Anti-Patterns to Avoid

- **Caching the tree snapshot across `readdir` calls.** The tree must reflect the backend's current parentId graph. Rebuild on every `readdir("tree/")` — Confluence is slow (~500ms for 500 pages) but correctness wins. (Or: cache with TTL of 5s, same as `ATTR_TTL`. Agent picks.)
- **Overwriting the user's `.gitignore`.** CONTEXT.md locks the virgin-mount assumption; honor it by always synthesizing — but log a WARN if we ever detect a prior `.gitignore` at mount time. Don't silently shadow.
- **Embedding numeric IDs in slugs by default.** Step 5's fallback (`page-<id>`) is the only time numeric ID leaks into the human-visible path. Siblings use `-N` suffixes instead. This is the "clean URL" property.
- **Letting cycles wedge the resolver.** Confluence theoretically can't return cycles (a page can't be its own ancestor), but defensive code: during build_tree, use a visited-set; if a node appears twice on its ancestor chain, break the cycle and emit `WARN` log.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---|---|---|---|
| Symlink path resolution | A manual "follow the symlink" helper in `read()` | The kernel VFS does it for free | `readlink` returns bytes; VFS does the path-walk and re-enters our FS at `lookup("pages/00000131192.md")`. Implementing it ourselves would be wrong and slower. |
| Parent-chain path building | Recursion by handwritten deref | A `HashMap<IssueId, Vec<IssueId>>` children index + iterative BFS | Stack overflow risk on deep trees; iterative is trivially O(n). |
| Title deduplication | `String::chars().nth()` scanning | Group-by-slug followed by sort+enumerate | The locked algorithm already specifies this; implement it literally. |
| Unicode case-folding | Full ICU `toLowerCase` | `str::to_lowercase()` (std) | The slug algorithm strips non-ASCII anyway in step 2. ASCII-lowercasing post-strip is equivalent for slug purposes. |
| `.gitignore` file emission | A tokio file-backed cache | A const `&[u8]` plus a dedicated fixed inode | It's 7 bytes. `reply.data(b"/tree/\n")` on every `read()`. |

**Key insight:** FUSE callbacks are request/reply over a kernel socket. Keep handlers stateless where possible — derive everything from the current `list_issues` snapshot + a precomputed `TreeSnapshot` struct.

## Runtime State Inventory

**Not applicable.** This phase is net-new mount-layout semantics with no rename/refactor/migration involved. Confirmed by each category:

| Category | Items Found | Action Required |
|---|---|---|
| Stored data | None — the SQLite audit log schema is unchanged; no on-disk state for tree/ (all synthesized). | None |
| Live service config | None — neither the simulator nor any real backend stores tree metadata; it's derived at mount time. | None |
| OS-registered state | None — no systemd units, launchd plists, cron jobs. | None |
| Secrets/env vars | `REPOSIX_ALLOWED_ORIGINS` unchanged; Confluence creds via existing `ConfluenceCreds`. | None |
| Build artifacts | None — `cargo build` is the only build path; no egg-info equivalents. | None |

## Common Pitfalls

### Pitfall 1: Returning `size: 0` for symlinks
**What goes wrong:** `ls -l tree/foo.md` shows size 0; `stat` reports the symlink as empty; some tools silently skip zero-size symlinks.
**Why it happens:** Copy-pasting from the directory `FileAttr` code path, which uses `size: 0`.
**How to avoid:** Always set `size = target_bytes.len() as u64`. Add a unit test.
**Warning signs:** `ls -l mount/tree/` outputs column 5 = 0 for every symlink.

### Pitfall 2: Incorrect relative path depth
**What goes wrong:** `cat tree/foo/bar.md` → ENOENT because `../../pages/00000131192.md` from `tree/foo/bar.md` resolves to `pages/00000131192.md` which is correct, but `../pages/...` from the same file resolves to `tree/pages/00000131192.md` which doesn't exist.
**Why it happens:** Hand-computing depth from a node's position instead of walking the tree-dir chain.
**How to avoid:** In `TreeSnapshot`, record each node's depth (0 for direct children of `tree/`, 1 for grandchildren, etc.). Symlink target = `"../".repeat(depth + 1) + &format!("{bucket}/{padded_id}.md")`.
**Warning signs:** `readlink -f` returns a path outside the mount, or `cat` through the symlink returns ENOENT despite `ls` showing the file.

### Pitfall 3: Non-ASCII slug fallback miss
**What goes wrong:** A Confluence page titled `"日本語"` slugs to empty string → triggers fallback `page-<id>`, which is fine. But a title of `"🚀 Rocket"` slugs to `rocket` (space + emoji stripped). Collision-deduplication is correct.
**Why it happens:** Step 2 replaces every non-`[a-z0-9]` ASCII byte with `-`. Multi-byte UTF-8 sequences are each multiple non-ASCII bytes → each becomes `-` → the `-` run collapses.
**How to avoid:** Run the slugify step byte-by-byte on UTF-8 bytes, treating any byte > 0x7f as "non-[a-z0-9]". Document with a test: `slugify_title("日本語") == ""` then falls back.
**Warning signs:** Panics in slice indexing → process UTF-8 by `.chars()` not `.bytes()`.

### Pitfall 4: Silently dropping pages with non-page parentType
**What goes wrong:** Confluence v2 has `parentType ∈ {page, folder, whiteboard, database, ...}`. If we treat `parentType != "page"` as "no parent," those pages appear as tree roots rather than their actual logical children.
**Why it happens:** The API surface is broader than page-hierarchy.
**How to avoid:** For v0.4.0 phase 13, document that `parentType != "page"` → orphan-root in tree/. Log a single DEBUG line per such page so future Phase 14+ can track which tenants need folder support. [CITED: developer.atlassian.com/cloud/confluence/rest/v2/api-group-page/]
**Warning signs:** Mysterious extra roots showing up in `tree/` for spaces known to have deep hierarchies.

### Pitfall 5: FUSE test flakiness under `--test-threads=N>1`
**What goes wrong:** Two FUSE mounts race on `fusermount3 -u` paths; test artifacts accumulate.
**Why it happens:** Each `Mount::open` spawns a background kernel-worker thread; tempdirs get cleaned out of order.
**How to avoid:** Keep the existing `--test-threads=1` convention from `readdir.rs`. Document in the new test file header.
**Warning signs:** Intermittent `EBUSY` on unmount; test passes in isolation but fails in CI.

### Pitfall 6: INodeNo wrapper confusion
**What goes wrong:** Passing a raw `u64` where `INodeNo` is expected (or vice versa).
**Why it happens:** fuser 0.17 wraps all inode args as `INodeNo(u64)`; the existing codebase sometimes destructures with `.0` and sometimes not.
**How to avoid:** Audit every new `lookup`/`readlink`/`getattr` handler branch for consistent `INodeNo` → `u64` extraction. Grep for `ino.0` usage.
**Warning signs:** Compile errors about `INodeNo` not implementing `PartialEq<u64>`.

## Code Examples

### Extending `Issue` with `parent_id`

```rust
// crates/reposix-core/src/issue.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: IssueId,
    pub title: String,
    pub status: IssueStatus,
    // ... existing fields ...
    /// Parent in a hierarchy-supporting backend (currently Confluence only).
    /// Always `None` for sim/GitHub. When `Some`, is the parent page/issue id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<IssueId>,
}
```

### Extending Confluence deserialization

```rust
// crates/reposix-confluence/src/lib.rs
#[derive(Debug, Deserialize)]
struct ConfPage {
    id: String,
    status: String,
    title: String,
    #[serde(rename = "createdAt")]
    created_at: chrono::DateTime<chrono::Utc>,
    version: ConfVersion,
    #[serde(default, rename = "ownerId")]
    owner_id: Option<String>,
    #[serde(default)]
    body: Option<ConfPageBody>,
    /// NEW: Confluence v2 parent — only meaningful when `parent_type == "page"`.
    #[serde(default, rename = "parentId")]
    parent_id: Option<String>,
    #[serde(default, rename = "parentType")]
    parent_type: Option<String>,
}

fn translate(page: ConfPage) -> Result<Issue> {
    // ... existing translation ...
    let parent_id = match (page.parent_id.as_deref(), page.parent_type.as_deref()) {
        (Some(pid), Some("page")) => pid.parse::<u64>().ok().map(IssueId),
        (Some(_), Some(other)) => {
            tracing::debug!(parent_type = %other, page_id = %page.id, "non-page parent, treating as orphan");
            None
        }
        _ => None,
    };
    Ok(Issue { /* ... */ parent_id })
}
```

### Slug helper (zero-dep)

```rust
// crates/reposix-core/src/path.rs
/// Truncation limit in bytes. 60 is safely under ext4 NAME_MAX (255) with
/// room for a `-NN` collision suffix.
pub const SLUG_MAX_BYTES: usize = 60;

/// Convert a free-form title to a URL-/filesystem-safe slug.
/// See CONTEXT.md §slug-algorithm for the full spec.
#[must_use]
pub fn slugify_title(title: &str) -> String {
    let lower = title.to_lowercase();
    let mut out = String::with_capacity(lower.len());
    let mut last_was_dash = true; // suppress leading dashes
    for ch in lower.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            last_was_dash = false;
        } else if !last_was_dash {
            out.push('-');
            last_was_dash = true;
        }
    }
    while out.ends_with('-') { out.pop(); }
    // Byte-safe truncation (char boundaries).
    if out.len() > SLUG_MAX_BYTES {
        let mut end = SLUG_MAX_BYTES;
        while !out.is_char_boundary(end) { end -= 1; }
        out.truncate(end);
        while out.ends_with('-') { out.pop(); }
    }
    out
}

pub fn slug_or_fallback(title: &str, id: IssueId) -> String {
    let s = slugify_title(title);
    if s.is_empty() || s == "." || s == ".." || s.chars().all(|c| c == '-') {
        format!("page-{:011}", id.0)
    } else {
        s
    }
}
```

### Sibling collision dedupe

```rust
/// Given a vec of (IssueId, slug) pairs that share one parent, return
/// (IssueId, final_slug) with `-N` suffixes on collisions.
pub fn dedupe_siblings(mut siblings: Vec<(IssueId, String)>) -> Vec<(IssueId, String)> {
    siblings.sort_by_key(|(id, _)| *id);
    let mut seen: std::collections::HashMap<String, u32> = Default::default();
    let mut out = Vec::with_capacity(siblings.len());
    for (id, slug) in siblings {
        let n = seen.entry(slug.clone()).or_insert(0);
        *n += 1;
        let final_slug = if *n == 1 { slug } else { format!("{slug}-{n}") };
        out.push((id, final_slug));
    }
    out
}
```

### readlink in fs.rs

```rust
// In impl Filesystem for ReposixFs (add new method):
fn readlink(&self, _req: &Request, ino: INodeNo, reply: ReplyData) {
    let Some(target) = self.tree.resolve_symlink(ino.0) else {
        reply.error(fuser::Errno::from_i32(libc::ENOENT));
        return;
    };
    reply.data(target.as_bytes());
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|---|---|---|---|
| fuser `fn readlink(&mut self, ...)` (pre-0.16) | `fn readlink(&self, ...)` + `INodeNo` wrapper | fuser 0.16+ | Matches existing reposix-fuse pattern — no interior-mutability workarounds needed. |
| Confluence REST v1 `ancestors[]` field | v2 `parentId: String, parentType: String` | [CITED: developer.atlassian.com/cloud/confluence/rest/v2/api-group-page/] | v2 is flat (single parent per page); we already use v2, so no migration. |
| `slug` crate with `deunicode` | Hand-rolled ASCII-only slug | Project philosophy (trust surface) | Trades 500KB dep for 30 LOC. |

**Deprecated/outdated:**
- REST v1 `/wiki/rest/api/content` endpoint: we don't use it.
- fuser 0.7-series `&mut self` callback signature: not our version.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|---|---|---|
| A1 | `parentType` can take values `{page, folder, whiteboard, database}` beyond just `page`. Atlassian docs only enumerate `page` explicitly. | Pitfall 4 | Low — we handle the unknown case by treating as orphan, which is correct regardless. |
| A2 | Relative symlinks in FUSE resolve cleanly within the mount. Confirmed by POSIX semantics ([CITED: man7.org/linux/man-pages/man2/readlink.2.html]) but not empirically tested in this repo yet. | Pattern 2 | Medium — Validation Architecture test #3 below exercises this directly. |
| A3 | 60-byte slug truncation + `-NN` suffix fits ext4 NAME_MAX 255. True by inspection (60 + 3 < 255). | Slug algorithm | None. |
| A4 | fuser 0.17.0 accepts `reply.data(bytes)` as the correct readlink reply. Confirmed via source at `fuser-0.17.0/src/lib.rs:469` which types `reply: ReplyData`. | readlink code example | Low. |
| A5 | The existing `readdir.rs` test pattern (wiremock + `spawn_blocking` for Mount) extends cleanly to a Confluence-backend variant. Not yet verified — requires the test harness to accept a `ConfluenceReadOnlyBackend` in place of `SimBackend`. | Validation Architecture | Medium — may require a small refactor of `MountConfig` if Confluence needs different constructor args; Phase 11 already has Confluence wiremock tests so the pattern is proven. |

## Open Questions

1. **Does `tree/` get emitted for a Confluence space where no page has a parent?**
   - What we know: CONTEXT.md says emit iff `supports(Hierarchy) || any(parent_id.is_some())`.
   - What's unclear: if Confluence claims `Hierarchy = true` but a particular space has a flat page list, do we still emit an empty `tree/` dir?
   - Recommendation: emit an empty `tree/` when `supports(Hierarchy) = true` even if 0 parents — keeps the directory contract predictable. Document in SUMMARY.

2. **Should `_self.md` be a symlink or synthesized content?**
   - What we know: CONTEXT.md says "symlink pointing at `../../pages/<id>.md`".
   - What's unclear: nothing — it's locked. But the implementer should verify the depth computation for `_self.md` at root of `tree/` (depth 0) vs deep (depth N).
   - Recommendation: treat `_self.md` as just another tree-symlink-inode with the same machinery; special-case only its *name*.

3. **What does `stat tree/` return for mtime?**
   - What we know: No guidance.
   - What's unclear: kernel tools may behave oddly with `mtime = UNIX_EPOCH`.
   - Recommendation: use the max `updated_at` across all member pages (deterministic, changes when content changes). Falls back to mount time if empty.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|---|---|---|---|---|
| Rust stable | Build | ✓ | 1.82+ via `rust-toolchain.toml` | — |
| `fusermount3` | Runtime mount (Linux only) | ✓ (assumed per CLAUDE.md) | system | `fusermount` (v2) |
| `cargo` | Test | ✓ | — | — |
| FUSE kernel support | Integration tests | ✓ on Linux CI runners (`ubuntu-latest` after `apt install fuse3`) | — | Skip `#[cfg(target_os = "linux")]` tests on other platforms |

No missing blocking deps. This is a pure-Rust additive phase.

## Validation Architecture

### Test Framework

| Property | Value |
|---|---|
| Framework | Rust stdlib `#[test]` + `#[tokio::test]` + `wiremock` 0.6 + `tempfile` |
| Config file | None (cargo test) |
| Quick run command | `cargo test --workspace` |
| Full suite command | `cargo test --workspace --release -- --include-ignored --test-threads=1` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|---|---|---|---|---|
| LOCKED-1 | `Issue::parent_id` round-trips through frontmatter | unit | `cargo test -p reposix-core issue::tests` | ❌ Wave 0 (extend existing tests) |
| LOCKED-2 | `slugify_title` handles ASCII, unicode, edge cases | unit (+ optional proptest) | `cargo test -p reposix-core path::tests::slug` | ❌ Wave 0 |
| LOCKED-3 | `dedupe_siblings` produces deterministic `-N` suffixes | unit | `cargo test -p reposix-core path::tests::dedupe` | ❌ Wave 0 |
| LOCKED-4 | `ConfluenceReadOnlyBackend` populates `parent_id` from v2 `parentId` when `parentType == "page"` | unit + wiremock | `cargo test -p reposix-confluence parent_id` | ❌ Wave 0 |
| LOCKED-5 | `ConfluenceReadOnlyBackend` treats non-page `parentType` as orphan | unit + wiremock | `cargo test -p reposix-confluence non_page_parent` | ❌ Wave 0 |
| LOCKED-6 | FUSE mount root emits `{bucket}/`, `tree/`, `.gitignore` | integration (FUSE) | `cargo test --release -p reposix-fuse --test nested_layout -- --ignored --test-threads=1` | ❌ Wave 0 |
| LOCKED-7 | `readlink(tree/foo.md)` returns correct relative target | integration (FUSE) | same | ❌ Wave 0 |
| LOCKED-8 | `cat tree/foo/bar.md` resolves symlink cleanly and returns underlying page body | integration (FUSE) | same | ❌ Wave 0 |
| LOCKED-9 | `cat mount/.gitignore` returns exactly `/tree/\n` | integration (FUSE) | same | ❌ Wave 0 |
| LOCKED-10 | Sim backend mount shows `issues/` but no `tree/` (feature gated) | integration (FUSE) | extend `tests/readdir.rs` to assert new path | ✅ existing file to update |
| LOCKED-11 | v0.4.0 release script exists and matches v0.3.0 pattern | shell lint | `bash -n scripts/tag-v0.4.0.sh` | ❌ Wave 0 |
| LIVE | Against live Confluence (`reuben-john.atlassian.net`): homepage 360556 → 3 children visible in `tree/` | manual/live | documented runbook in SUMMARY | N/A (manual step) |

### Candidate Validation Patterns (2-3 the planner must pick from)

**Pattern A: Wiremock-driven FUSE integration (primary).**
Extend `tests/readdir.rs` into a new `tests/nested_layout.rs`. Stand up a wiremock `MockServer` returning a canned Confluence page graph (3-level deep, 2 siblings with colliding titles, 1 orphan-parent, 1 non-page-parentType). Mount with `ConfluenceReadOnlyBackend::new_with_base_url(server.uri())`. Assert via `std::fs`:
- `read_dir(mount)` → `{.gitignore, pages, tree}`
- `read_dir(mount/pages)` → 4+ `<padded-id>.md` files
- `read_dir(mount/tree)` → expected slug names
- `std::fs::read_link(mount/tree/<slug>)` → expected relative string
- `std::fs::read_to_string(mount/tree/<slug>)` → body bytes (proves symlink resolves)
- `std::fs::read_to_string(mount/.gitignore)` → `"/tree/\n"`

**Pattern B: Unit-level TreeSnapshot property test (supplementary).**
Property test in `crates/reposix-fuse/src/tree.rs`:
- Generate random DAG of `(IssueId, Option<IssueId>, title: String)`.
- Call `build_tree(&issues)`.
- Assert: every issue appears exactly once in the tree OR is reachable via `_self.md`. No slug collisions within a single parent. All symlink targets are syntactically valid relative paths (regex `^(\.\./)+<bucket>/\d{11}\.md$`). No cycles in parent-chain.
- Shrinker finds minimal counterexample if an invariant breaks.

**Pattern C: Live verification against real Confluence (final gate).**
Mount `reuben-john.atlassian.net` with `REPOSIX` space. Assert:
- `cd /tmp/mnt/tree && ls` shows `reposix-demo-space-home/` (the homepage slug).
- `cd reposix-demo-space-home && ls` shows `_self.md`, `welcome-to-reposix.md`, `architecture-notes.md`, `demo-plan.md`.
- `cat welcome-to-reposix.md | head -5` shows the frontmatter of page 131192.
- `readlink welcome-to-reposix.md` → `../../pages/00000131192.md`.

Captured as a shell script `scripts/demo-nested-mount.sh` that the HANDOFF.md runbook points at. Exit nonzero on failure so next-session agent can rerun as a smoke test.

### Sampling Rate

- **Per task commit:** `cargo test --workspace` (unit + non-FUSE wiremock tests)
- **Per wave merge:** `cargo test --workspace --release -- --include-ignored --test-threads=1` (full suite incl. FUSE integration)
- **Phase gate:** Full suite green + manual live-Confluence run per Pattern C + `scripts/tag-v0.4.0.sh` dry-run

### Wave 0 Gaps

- [ ] `crates/reposix-core/src/path.rs` — extend with `slugify_title`, `slug_or_fallback`, `dedupe_siblings` + their tests
- [ ] `crates/reposix-core/src/issue.rs` — add `parent_id` field + roundtrip test
- [ ] `crates/reposix-core/src/backend.rs` — add `BackendFeature::Hierarchy`, `root_collection_name()` default
- [ ] `crates/reposix-confluence/src/lib.rs` — deserialize parentId/parentType; new wiremock tests for page-parent vs non-page-parent
- [ ] `crates/reposix-fuse/src/tree.rs` — NEW module (`TreeSnapshot`, `build_tree`, `resolve_symlink`)
- [ ] `crates/reposix-fuse/src/inode.rs` — extend namespaces for tree dirs + symlinks
- [ ] `crates/reposix-fuse/src/fs.rs` — add `readlink`; dispatch in `lookup`/`getattr`/`readdir` on new inode ranges; synthesize `.gitignore`
- [ ] `crates/reposix-fuse/tests/readdir.rs` — update to assert new `issues/` root + still passes
- [ ] `crates/reposix-fuse/tests/nested_layout.rs` — NEW integration test (wiremock Confluence + FUSE)
- [ ] `scripts/tag-v0.4.0.sh` — release script modeled after `scripts/tag-v0.3.0.sh`
- [ ] `scripts/demo-nested-mount.sh` — live Confluence demo runbook
- [ ] `docs/decisions/003-nested-mount-layout.md` — ADR-003 superseding ADR-002 (the CONTEXT.md locks this)
- [ ] `CHANGELOG.md` — `[Unreleased]` / `[v0.4.0]` BREAKING block

## Security Domain

Per CLAUDE.md and project threat model, ASVS categories applicable:

| ASVS Category | Applies | Standard Control |
|---|---|---|
| V5 Input Validation | yes | Confluence titles feed `slugify_title`. Algorithm is bytewise; no injection surface (slugs land in FUSE replies, not in shell or HTTP). Existing `validate_path_component` still rejects junk. |
| V6 Cryptography | no | No new crypto. |
| V8 Data Protection | partial | New `parent_id` field inherits Confluence page's taint (Phase 11's `Tainted::new`). No disclosure beyond what `list_issues` already exposes. |
| V12 API | yes | `parentId` / `parentType` are Atlassian v2 fields; we parse permissively (`#[serde(default)]`) for forward-compat. |

### Known Threat Patterns for {FUSE + Confluence stack}

| Pattern | STRIDE | Standard Mitigation |
|---|---|---|
| Slug injection via title → path traversal | Tampering | `slugify_title` produces `[a-z0-9-]*`; cannot produce `/`, `..`, `\0`. Validated by unit test. |
| Symlink target injection → escape mount | Tampering | Symlink targets are built from trusted `(padded_id, depth)` data, never from Confluence content. Targets are construction-safe. |
| Tainted title ending up in shell command | Injection | Titles only land in kernel FUSE replies (bytes), never into `std::process::Command` or network egress. Existing SG-03 discipline holds. |
| Huge title (DoS) | DoS | Slug algorithm truncates to 60 bytes. Memory is O(pages). |
| Cycle in parent graph wedges resolver | DoS | Visited-set in build_tree breaks cycles and emits WARN. |

## Sources

### Primary (HIGH confidence)

- fuser 0.17.0 source — `/home/reuben/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/fuser-0.17.0/src/lib.rs` lines 131-143 (FileType enum incl. Symlink), 469-472 (readlink default impl), 414-417 (lookup), 437-439 (getattr)
- fuser 0.15.1 source (for cross-reference that the API is stable) — same path, `fuser-0.15.1`
- Cargo.lock — confirms `fuser = "0.17.0"` is the actual dependency
- `reposix-fuse/src/fs.rs`, `inode.rs`, `tests/readdir.rs` — existing patterns to extend
- `reposix-confluence/src/lib.rs` — existing `ConfPage` struct, line 203-215

### Secondary (MEDIUM confidence)

- Atlassian Confluence REST v2 docs for pages: [Get pages in space](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-page/) — confirms `parentId: String`, `parentType: "page"` shape
- POSIX `readlink(2)` spec: [man7.org/linux/man-pages/man2/readlink.2.html](https://man7.org/linux/man-pages/man2/readlink.2.html) — relative target resolution semantics
- ext4 NAME_MAX = 255 bytes: [ext4 Wikipedia](https://en.wikipedia.org/wiki/Ext4)
- restic mount symlink size-0 bug: [github.com/restic/restic/issues/3667](https://github.com/restic/restic/issues/3667) — motivates Pitfall 1

### Tertiary (LOW confidence — not blocking)

- Rust slug crates comparison: [crates.io/crates/slug](https://crates.io/crates/slug), [crates.io/crates/deunicode](https://crates.io/crates/deunicode) — informed the "don't add a dep" recommendation but the hand-rolled approach is locked by CONTEXT.md philosophy regardless.

## Metadata

**Confidence breakdown:**
- fuser API mechanics: HIGH — inspected source directly
- Confluence v2 parentId/parentType: HIGH — confirmed in Atlassian's public schema
- Slug algorithm viability: HIGH — ASCII-only + 60-byte trunc is trivially correct
- Inode dispatch strategy: MEDIUM — agent's discretion per CONTEXT.md; the two candidate approaches (range-based vs kind-tagged) are both known-good
- Wiremock FUSE test extensibility: MEDIUM — Phase 11 already has Confluence+wiremock; Phase 3 already has FUSE+wiremock; combining them is additive but unverified end-to-end
- Live Confluence target: HIGH — CONTEXT.md names specific page IDs

**Research date:** 2026-04-14
**Valid until:** 2026-05-14 (30 days — stack is stable)
