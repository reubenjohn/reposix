---
phase: 13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren
reviewed: 2026-04-14T00:00:00Z
depth: deep
files_reviewed: 7
files_reviewed_list:
  - crates/reposix-core/src/path.rs
  - crates/reposix-core/src/backend.rs
  - crates/reposix-core/src/issue.rs
  - crates/reposix-confluence/src/lib.rs
  - crates/reposix-fuse/src/tree.rs
  - crates/reposix-fuse/src/fs.rs
  - crates/reposix-fuse/src/inode.rs
  - crates/reposix-fuse/tests/nested_layout.rs
findings:
  critical: 0
  warning: 2
  info: 4
  total: 6
status: issues_found
---

# Phase 13: Code Review Report

**Reviewed:** 2026-04-14
**Depth:** deep (cross-file; tree/fs/inode invariants; security focus list)
**Files Reviewed:** 7 production modules + 1 security-critical integration test
**Status:** **PASS** — no HIGH findings. Phase is ship-ready for `v0.4.0`.

## Summary

The Phase 13 change-set is disciplined and well-tested. The security-critical invariants — slug output shape, symlink-target construction, cycle-safe parent walks, inode-range disjointness, frontmatter backward-compat — are each backed by direct unit tests and, in several cases, compile-time assertions. The three threat-model IDs explicitly called out in CONTEXT (T-13-01 slug escape, T-13-05 target escape, T-13-03 cycle-bypass) all have tight, bounded mitigations.

No bytes of attacker-controlled input reach a symlink target. `slugify_title` is ASCII-only by construction and proven so by an adversarial corpus (`path.rs:360`). Cycles detect in O(n·h) with a per-page visited-set seeded with the page's own id so self-cycles break on the first step (`tree.rs:416`). The `InodeKind::classify` partition is backed by a cross-module compile-time assertion (`tree.rs:94`) and an in-module runtime test (`inode.rs:220`). `validate_issue_filename` still rejects `..`, `/`, and NUL at the bucket boundary, so the BREAKING path change does not weaken the write-path validators.

Findings below are all WARNING/INFO — graceful-degradation polish, not correctness gaps.

## Critical Issues

None.

## Warnings

### WR-01: First-touch `ls tree/` before `ls mount/` returns empty listing

**File:** `crates/reposix-fuse/src/fs.rs:706-717` (TreeRoot readdir) and `fs.rs:627-633` (TreeRoot lookup)
**Issue:** `TreeRoot` readdir reads `self.tree` unconditionally and returns whatever is there — which is `TreeSnapshot::default()` (empty) until a prior `Root`- or `Bucket`-inode readdir has triggered `refresh_issues()`. For a Confluence mount (`hierarchy_feature = true`), `should_emit_tree()` returns `true` at the start, so `lookup(ROOT, "tree")` succeeds before any refresh. A user whose first action is `ls mount/tree/` (or a scripted invocation like `cat mount/tree/foo/_self.md`) sees an empty directory and no symlinks, silently. No error, no diagnostic — just wrong data.

The integration tests in `nested_layout.rs` mask this because `boot_mount` waits for `.gitignore` + `pages` + `tree` to surface, which forces a Root readdir first (line 143-160). Real users won't.

**Fix:** In the `InodeKind::TreeRoot` arm of `readdir` (fs.rs:706), call `self.refresh_issues()` before reading the snapshot, mirroring what the `Root` arm does at line 669. The extra GET is amortized by the body cache. Same pattern for `TreeDir` is less important — if `TreeRoot` is fresh, `TreeDir` inodes in the snapshot are valid.

```rust
InodeKind::TreeRoot => {
    if let Err(e) = self.refresh_issues() {
        warn!(error = %e, "tree readdir refresh failed (non-fatal)");
    }
    let Ok(snap) = self.tree.read() else {
        reply.error(fuser::Errno::from_i32(libc::EIO));
        return;
    };
    // ... existing code
}
```

### WR-02: Symlink target byte length is not capped against `PATH_MAX`

**File:** `crates/reposix-fuse/src/tree.rs:458-461` (`symlink_target`) and `crates/reposix-fuse/src/fs.rs:388-413` (`symlink_attr`)
**Issue:** `symlink_target` returns `"../".repeat(depth + 1) + bucket + "/" + padded_id + ".md"`. On a 1000-deep parent chain (achievable at Confluence's `MAX_ISSUES_PER_LIST` cap, see `confluence/src/lib.rs:609`), a leaf symlink target is ~3 KB. `PATH_MAX` on Linux is typically 4096 bytes, so we're *currently* under the limit — but there is no explicit check, and the per-mount pagination cap (`MAX_ISSUES_PER_LIST / PAGE_SIZE`) is a volume cap, not a depth cap. A future bump of either constant, or a backend that returns deeper trees, silently produces unreadable symlinks: `readlink(2)` returns `ENAMETOOLONG` and tools like `ls -l` / `git` show spurious I/O errors.

Two sub-issues flow from the same root:
1. `tree::symlink_target` happily builds targets > `PATH_MAX` with no log line.
2. `fs::symlink_attr` sets `st_size = target.len() as u64` for an over-length target — inconsistent with what `readlink(2)` will actually return.

**Fix:** Guard at build time in `tree::build_leaf_symlink` / `build_dir`: if `symlink_target(...)` exceeds `libc::PATH_MAX as usize - 1`, emit `tracing::warn!` and fall back to rooting the entry at the tree root (depth=0) — the same degradation pathway cycles use. Alternatively, fail open with the leaf present but log the over-length to make the failure observable.

```rust
let target = symlink_target(self.bucket, id, depth);
if target.len() > 4095 {
    tracing::warn!(page = id.0, len = target.len(),
        "tree symlink target exceeds PATH_MAX; pinning to tree root");
    return self.build_leaf_symlink(id, slug, 0);
}
```

## Info

### IN-01: Tracing `warn!`/`debug!` on attacker-controlled Confluence strings is unbounded

**File:** `crates/reposix-confluence/src/lib.rs:331` (`bad_parent = %pid_str`) and `lib.rs:340` (`parent_type = %other`)
**Issue:** `pid_str` and `other` are fields lifted directly from a Confluence JSON response body. An adversarial tenant (or a compromised page that an admin-user edited) can set `parentId` or `parentType` to a 1 MB string; we then span-format that into a `tracing::warn!` event. This is not a correctness or traffic-amplification bug (the span is emitted once per page per list refresh), but in an operator's log collector it is both a storage-hit and a potential log-injection vector (newline + fake log line). Defense in depth.

**Fix:** Truncate before logging, e.g.:
```rust
let pid_preview = &pid_str[..pid_str.len().min(64)];
tracing::warn!(page_id = %page.id, bad_parent = %pid_preview,
    "confluence parentId not parseable as u64, treating as orphan");
```

### IN-02: `slugify_title` has no input-size guard

**File:** `crates/reposix-core/src/path.rs:97-128`
**Issue:** `slugify_title(title)` calls `title.to_lowercase()` which allocates a fresh String the size of `title` (or larger — some codepoints expand on lowercase). If a Confluence page ever returns a 10 MB title (not today's reality, but not forbidden by the wire shape), we allocate 10 MB per page per mount-refresh. Since the output is capped at `SLUG_MAX_BYTES = 60`, the intermediate allocation is pure waste.

**Fix:** Bound the intermediate: `title.chars().take(SLUG_MAX_BYTES * 4).collect::<String>().to_lowercase()` (4× headroom for the lowercase expansion). Correctness is preserved because the slug is truncated to `SLUG_MAX_BYTES` anyway; any input beyond the first ~240 chars is guaranteed to be sliced off before it reaches the output.

### IN-03: Symlink mtime/atime drift on every `getattr`

**File:** `crates/reposix-fuse/src/fs.rs:392-413` (`symlink_attr`)
**Issue:** `symlink_attr` rebuilds `FileAttr` from `SystemTime::now()` on every call. Each `stat` against a tree symlink reports a fresh timestamp, which will confuse anything that caches or diffs `st_mtim` (rsync `--times`, make, backup tools). The directory attrs (`root_attr`, `tree_attr`, `bucket_attr`) correctly cache `now` once at construction (fs.rs:284). The symlink path should match.

**Fix:** Cache a single `mount_time: SystemTime` on `ReposixFs` at construction, reuse for `symlink_attr` and `tree_dir_attr`. One-line change; no behavioural downside since symlink targets themselves are immutable per snapshot.

### IN-04: `TreeDir` `..` entry parent inode is always `TREE_ROOT_INO`

**File:** `crates/reposix-fuse/src/fs.rs:730-735`
**Issue:** `readdir` on a nested `TreeDir` emits `..` with `ino = TREE_ROOT_INO` regardless of the dir's actual depth. The code-comment at line 730 already notes this is cosmetic ("the kernel follows explicit paths, not `..` entries"), which is true for lookup, but `ls -lai` users will see the wrong inode number in column 1 and tools that verify `.. == parent.ino` may flag it. Storing the parent inode in `TreeDir` adds one `u64` per interior dir (trivial) and removes the cosmetic wart.

**Fix:** Add `parent_ino: u64` to `TreeDir`; plumb through in `Builder::build_dir` (the `depth` branch already knows its caller's inode — threading it is straightforward).

---

## Spot-checks that passed

- **Slug output shape over adversarial inputs** — `path.rs:360` adversarial corpus (RTL override, shell metas, path separators, NUL) all produce `[a-z0-9-]+` with no `.`/`..`/`/`/`\0`. Passes.
- **Symlink target shape** — `tree.rs:523` (`assert_target_never_escapes`) is invoked at the end of *every* tree test via the harness pattern. No bytes of page title or body reach a target string; target construction is `"../".repeat(depth+1) + &'static bucket + "/" + format!("{:011}", id) + ".md"`. Passes.
- **Inode-range disjointness** — `tree.rs:94-98` compile-time asserts + `inode.rs:220` runtime asserts. `InodeKind::classify` match arms are ordered correctly (root/bucket/tree-root/gitignore fixed → symlink range → dir range → issue range → unknown). No overlap possible. Passes.
- **Cycle detection** — self-cycles (seeded visited = {issue.id}), 2-cycles, 3-cycles, diamonds, deep linear chains all terminate. Chain walks are per-page, so A→B→A and B→A→B each independently detect; both emit orphan tree roots with a `warn!`. Passes.
- **Orphan-parent handling** — `effective_parent_of` at `tree.rs:403` checks `by_id.contains_key(direct_parent)` before walking, and again inside the loop at `tree.rs:432` for vanished mid-chain ancestors. Both degrade to tree-root. Passes.
- **`.gitignore` read offset handling** — fs.rs:773-780 clamps `start` and `end` against `GITIGNORE_BYTES.len()` defensively; offset > 7, offset = 0 with size = 100, and normal reads all produce sane slices. Passes.
- **Frontmatter backward-compat** — `issue.rs:267` (`parent_id_default_on_missing_field`) and `issue.rs:331` (`frontmatter_parses_legacy_without_parent_id`) both confirm pre-Phase-13 payloads still deserialize with `parent_id: None`. `#[serde(default, skip_serializing_if = "Option::is_none")]` means None-valued fields never hit the wire. Passes.
- **Malformed `parentId`** — `lib.rs:1154` (`translate_handles_unparseable_parent_id`) confirms `.parse::<u64>()` failure degrades to `None` with a `warn!`, does not panic, does not propagate as `Err`. Exactly what T-13-PB1 mandates. Passes.
- **Confluence `parentType` allowlist** — the match at `lib.rs:326-348` only propagates `Some("page")`. `folder`, `whiteboard`, `database`, and arbitrary attacker-supplied strings all degrade to orphan. Passes.
- **Concurrency** — `TreeSnapshot` is `Arc<RwLock<_>>`. Reads take `.read()`, errors on poisoning surface as `EIO` (fs.rs:562/574/628/635/707/720/821). No interior mutability inside `TreeSnapshot` itself. Passes.
- **Inode kind write rejection** — `write` (fs.rs:915-940), `setattr` (fs.rs:864-872), `create` (fs.rs:1057), and `unlink` (fs.rs:1133) all reject non-`Bucket`/non-`RealFile` parents with `EROFS` / `EPERM` / `EISDIR` as appropriate. No write path reaches a tree symlink, a tree dir, or `.gitignore`. Passes.

## Deferred (not Phase 13 scope, logged for record)

- Release PATCH (`fs.rs:1008`) POSTs to the sim-shape REST endpoint even for Confluence-backed mounts. Pre-existing limitation documented at fs.rs:41-43; not regressed by Phase 13.
- `InodeRegistry::intern` has a documented benign race (inode.rs:86-89) where two concurrent interns of the same id waste one inode from the allocator. Pre-existing.

---

_Reviewed: 2026-04-14_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: deep_
