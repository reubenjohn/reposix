---
phase: 13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren
plan: C
type: execute
wave: 3
depends_on: [A, B1, B2, B3]
files_modified:
  - crates/reposix-fuse/src/fs.rs
  - crates/reposix-fuse/src/inode.rs
  - crates/reposix-fuse/tests/readdir.rs
  - crates/reposix-fuse/tests/nested_layout.rs
autonomous: true
requirements:
  - OP-1
user_setup: []

must_haves:
  truths:
    - "Mount root readdir returns exactly: `.`, `..`, `.gitignore`, `<bucket>/` (where bucket = `backend.root_collection_name()`), AND `tree/` when `backend.supports(BackendFeature::Hierarchy) == true` OR when any loaded issue has `parent_id.is_some()`"
    - "Reading `mount/.gitignore` returns exactly the bytes `/tree/\\n` (7 bytes)"
    - "`<bucket>/` readdir returns exactly `.`, `..`, and one `<padded-id>.md` per issue — the OLD flat `<id>.md` at mount root is NOT accessible"
    - "`tree/` readdir returns the root children of the `TreeSnapshot` (dirs + symlinks as appropriate)"
    - "`getattr(symlink_ino)` returns `FileAttr { kind: FileType::Symlink, size: target_bytes.len() as u64, perm: 0o777, nlink: 1, .. }`"
    - "`readlink(symlink_ino)` returns the target string via `reply.data(target.as_bytes())`"
    - "End-to-end: `cat mount/tree/<slug>.md` returns the same bytes as `cat mount/pages/<padded-id>.md` when `<slug>` symlinks to that padded id — proves the kernel resolves the symlink through our FUSE lookup/read path"
    - "Inode-kind dispatch: `lookup`/`getattr` branch on inode range (ROOT / BUCKET / GITIGNORE / FIRST_ISSUE..TREE_DIR_BASE / TREE_DIR..TREE_SYMLINK_BASE / TREE_SYMLINK..) BEFORE any HashMap lookup"
    - "`fusermount3 -u /tmp/mnt` succeeds cleanly after every test"
    - "All pre-existing `tests/readdir.rs` assertions updated to look for `mount/issues/<padded>.md` (sim backend) instead of `mount/<padded>.md`"
    - "New `tests/nested_layout.rs` runs wiremock-Confluence + FUSE mount, exercises 3-level hierarchy, collision, cycle-break, readlink target, .gitignore content, writes-follow-symlink"
  artifacts:
    - path: "crates/reposix-fuse/src/fs.rs"
      provides: "Extended Filesystem impl: readlink() added; lookup/getattr/readdir dispatch on inode kind; root synthesizes .gitignore + bucket + tree"
      contains: "readlink"
    - path: "crates/reposix-fuse/src/inode.rs"
      provides: "Fixed inode constants for root, bucket, gitignore, tree_root; documentation of the layout"
      contains: "BUCKET_DIR_INO"
    - path: "crates/reposix-fuse/tests/readdir.rs"
      provides: "Updated assertions for new root layout (sim backend)"
    - path: "crates/reposix-fuse/tests/nested_layout.rs"
      provides: "New integration test against wiremock-Confluence with --ignored flag for FUSE-requiring portion"
      min_lines: 300
  key_links:
    - from: "crates/reposix-fuse/src/fs.rs"
      to: "reposix_fuse::tree::TreeSnapshot (Wave B2)"
      via: "stored in ReposixFs struct; rebuilt on each list_issues refresh"
      pattern: "TreeSnapshot"
    - from: "crates/reposix-fuse/src/fs.rs"
      to: "reposix_core::backend::IssueBackend::root_collection_name (Wave A)"
      via: "called once at mount init; cached in ReposixFs::bucket field"
      pattern: "root_collection_name"
    - from: "crates/reposix-fuse/src/fs.rs"
      to: "reposix_core::backend::BackendFeature::Hierarchy (Wave A)"
      via: "`backend.supports(BackendFeature::Hierarchy)` gates tree/ emission"
      pattern: "BackendFeature::Hierarchy"
---

<objective>
Wave-C integrator. Wire `TreeSnapshot` (B2) into the existing `ReposixFs` FUSE implementation: add the `<bucket>/` root directory, the synthesized `.gitignore` file, the conditional `tree/` directory, and the new `readlink` callback. Extend `lookup`/`getattr`/`readdir` with inode-range dispatch. Update existing `tests/readdir.rs`; add new `tests/nested_layout.rs` with a wiremock-Confluence-driven 3-level hierarchy FUSE integration test.

Purpose: This is the plan that actually makes `ls mount/` show the new layout and `readlink mount/tree/foo.md` return the right string. It's the only Wave-C plan because fs.rs is a single hot file — splitting it would thrash file ownership and force sequential waves.

Output: Edits to `fs.rs` (the core wiring — probably +300 lines), `inode.rs` (add fixed-inode constants), updated `tests/readdir.rs`, new `tests/nested_layout.rs` (integration test with `--ignored` for FUSE mount portion).
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-CONTEXT.md
@.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-RESEARCH.md
@.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-A-core-foundations.md
@.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-B1-confluence-parent-id.md
@.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-B2-fuse-tree-module.md
@.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-B3-frontmatter-parent-id.md
@CLAUDE.md
@crates/reposix-fuse/src/fs.rs
@crates/reposix-fuse/src/inode.rs
@crates/reposix-fuse/src/tree.rs
@crates/reposix-fuse/tests/readdir.rs

<interfaces>
<!-- After Waves A + B2 ship: -->

```rust
// reposix_fuse::tree — the module Wave C wires in
pub const TREE_ROOT_INO: u64 = 3;
pub const TREE_DIR_INO_BASE: u64 = 0x8_0000_0000;
pub const TREE_SYMLINK_INO_BASE: u64 = 0xC_0000_0000;
pub struct TreeSnapshot { /* ... */ }
impl TreeSnapshot {
    pub fn build(bucket: &str, issues: &[Issue]) -> Self;
    pub fn resolve_symlink(&self, ino: u64) -> Option<&str>;
    pub fn resolve_dir(&self, ino: u64) -> Option<&TreeDir>;
    pub fn root_children(&self) -> &[TreeEntry];
    pub fn is_empty(&self) -> bool;
}

// reposix_core::backend
pub trait IssueBackend: Send + Sync {
    fn supports(&self, feature: BackendFeature) -> bool;
    fn root_collection_name(&self) -> &'static str { "issues" }
    // ... other methods unchanged ...
}
```

<!-- Proposed new fixed inodes in inode.rs (add alongside FIRST_ISSUE_INODE): -->

```rust
pub const ROOT_INO: u64 = 1;           // the mount root (fuser default)
pub const BUCKET_DIR_INO: u64 = 2;     // pages/ or issues/ directory
pub const TREE_ROOT_INO: u64 = 3;      // mirror tree::TREE_ROOT_INO
pub const GITIGNORE_INO: u64 = 4;      // synthesized .gitignore
// FIRST_ISSUE_INODE = 0x1_0000 (unchanged; real files under <bucket>/)
// TREE_DIR_INO_BASE = 0x8_0000_0000 (from tree module)
// TREE_SYMLINK_INO_BASE = 0xC_0000_0000 (from tree module)
```

<!-- fuser 0.17 callback signatures (from 13-RESEARCH.md §"readlink in fs.rs"): -->

```rust
fn readlink(&self, _req: &Request, ino: INodeNo, reply: ReplyData) {
    let Some(target) = self.tree.resolve_symlink(ino.0) else {
        reply.error(fuser::Errno::from_i32(libc::ENOENT));
        return;
    };
    reply.data(target.as_bytes());
}
```

The `FileAttr` for a symlink — note `size = target_bytes.len()`:

```rust
FileAttr {
    ino: INodeNo(symlink_ino),
    size: target.as_bytes().len() as u64,   // CRITICAL — not 0, or ls -l shows 0
    blocks: 0,
    atime: ..., mtime: ..., ctime: ..., crtime: ...,
    kind: FileType::Symlink,
    perm: 0o777,  // symlink perm is ignored by kernel; convention is 0o777
    nlink: 1,
    uid: 0, gid: 0, rdev: 0, blksize: 512, flags: 0,
}
```
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Extend `inode.rs` with fixed inodes + re-export tree constants</name>
  <files>
    crates/reposix-fuse/src/inode.rs
  </files>
  <behavior>
    - `inode::ROOT_INO == 1`, `inode::BUCKET_DIR_INO == 2`, `inode::TREE_ROOT_INO == 3`, `inode::GITIGNORE_INO == 4`.
    - `FIRST_ISSUE_INODE` remains `0x1_0000` (unchanged — the reserved range 5..=0xFFFF stays reserved for future synthetics).
    - Module doc comment updated to reflect the new inode layout including tree ranges (the ranges themselves live in `tree::`).
    - New test `fixed_inodes_are_disjoint_from_dynamic_ranges`: asserts that ROOT/BUCKET/TREE_ROOT/GITIGNORE are all < FIRST_ISSUE_INODE, and that FIRST_ISSUE_INODE < TREE_DIR_INO_BASE < TREE_SYMLINK_INO_BASE.
  </behavior>
  <action>
    Add to `crates/reposix-fuse/src/inode.rs`:
    ```rust
    /// The mount root. Always inode 1 per FUSE convention.
    pub const ROOT_INO: u64 = 1;
    /// The root-collection bucket directory (`pages/` for Confluence, `issues/` for sim+GitHub).
    pub const BUCKET_DIR_INO: u64 = 2;
    /// The synthesized `tree/` overlay root directory. Emitted iff the backend
    /// supports [`BackendFeature::Hierarchy`](reposix_core::backend::BackendFeature::Hierarchy)
    /// or any loaded issue has `parent_id.is_some()`.
    pub const TREE_ROOT_INO: u64 = 3;
    /// The synthesized `.gitignore` file at the mount root. Always present.
    pub const GITIGNORE_INO: u64 = 4;
    ```
    Update the existing module doc comment (lines 3-20 of `inode.rs`) to document the full layout:
    ```
    // Inode 1        = mount root
    // Inode 2        = <bucket>/  (pages/ or issues/)
    // Inode 3        = tree/      (conditional)
    // Inode 4        = .gitignore (synthesized, read-only)
    // Inode 5..=0xFFFF = reserved for future synthetics
    // Inode 0x1_0000..0x8_0000_0000 = real files under <bucket>/<padded-id>.md
    // Inode 0x8_0000_0000..0xC_0000_0000 = tree/ interior directories
    // Inode 0xC_0000_0000..u64::MAX = tree/ leaf symlinks
    ```
    Update the `reserved_range_is_unmapped` test to check inode 5..=0xFFFF (not 2..=0xFFFF).

    Add a new test `fixed_inodes_are_disjoint_from_dynamic_ranges` that asserts the invariants.
  </action>
  <verify>
    <automated>cargo test -p reposix-fuse --locked inode:: &amp;&amp; grep -q 'pub const GITIGNORE_INO: u64 = 4' crates/reposix-fuse/src/inode.rs</automated>
  </verify>
  <done>
    Fixed inodes declared; doc comment updated; existing inode tests still green. Commit: `feat(13-C-1): declare fixed inodes for bucket, tree root, gitignore`.
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Wire `TreeSnapshot` + root layout + readlink into `fs.rs`</name>
  <files>
    crates/reposix-fuse/src/fs.rs,
    crates/reposix-fuse/tests/readdir.rs
  </files>
  <behavior>
    After mounting with a backend whose `root_collection_name() == "issues"` and `supports(Hierarchy) == false` (sim):
    - `read_dir(mount)` yields `[".gitignore", "issues"]` (plus `.` and `..` for libc's readdir).
    - `read_to_string(mount/.gitignore) == "/tree/\n"`.
    - `read_dir(mount/issues)` yields N entries each of form `00000000NNN.md`.
    - `read_to_string(mount/issues/00000000001.md).starts_with("---")` — frontmatter renders unchanged from prior behavior.
    - `read_dir(mount)` does NOT include `"tree"` (because supports(Hierarchy) == false AND no issue has parent_id).

    After mounting with a backend whose `root_collection_name() == "pages"` and `supports(Hierarchy) == true` (confluence wiremock):
    - `read_dir(mount)` yields `[".gitignore", "pages", "tree"]`.
    - `read_dir(mount/tree)` yields the slugs produced by TreeSnapshot root_children.
    - `read_link(mount/tree/<slug>.md)` returns a `../pages/<padded>.md`-shaped string.
    - `read_to_string(mount/tree/<slug>.md)` returns the body bytes of the underlying page (kernel symlink resolution round-trips through lookup+read).

    The integration tests exercise all of the above. The existing `tests/readdir.rs` assertions (which look for `<padded>.md` at `mount/` root) are updated to look at `mount/issues/<padded>.md`.
  </behavior>
  <action>
    **Extend `ReposixFs` struct** (or whatever the concrete name is — check fs.rs) with two new fields:
    ```rust
    struct ReposixFs {
        // ... existing fields (backend, client, registry, runtime handle, etc.) ...
        bucket: &'static str,              // set at construction from backend.root_collection_name()
        emit_tree: bool,                   // set from backend.supports(BackendFeature::Hierarchy) at construction
        tree: std::sync::RwLock<reposix_fuse::tree::TreeSnapshot>,  // rebuilt on readdir refresh
    }
    ```
    At constructor time:
    ```rust
    let bucket = backend.root_collection_name();
    let emit_tree = backend.supports(BackendFeature::Hierarchy);
    let tree = RwLock::new(TreeSnapshot::build(bucket, &[]));  // empty until first list
    ```

    **Refresh the tree snapshot whenever issues are listed**. The existing fs.rs almost certainly has a "refresh issues" code path inside `readdir(ROOT)` or `readdir(BUCKET_DIR)`. Extend it to:
    ```rust
    let issues = list_issues_via_runtime(...)?;
    self.registry.refresh(issues.iter().map(|i| i.id));
    if self.emit_tree || issues.iter().any(|i| i.parent_id.is_some()) {
        *self.tree.write().unwrap() = TreeSnapshot::build(self.bucket, &issues);
    }
    ```

    **Inode-kind dispatch helper** — add a helper function at the top of `fs.rs`:
    ```rust
    enum InodeKind { Root, Bucket, TreeRoot, Gitignore, RealFile, TreeDir, TreeSymlink, Unknown }
    fn classify(ino: u64) -> InodeKind {
        use crate::inode::*;
        use reposix_fuse::tree::{TREE_DIR_INO_BASE, TREE_SYMLINK_INO_BASE};
        match ino {
            ROOT_INO => InodeKind::Root,
            BUCKET_DIR_INO => InodeKind::Bucket,
            TREE_ROOT_INO => InodeKind::TreeRoot,
            GITIGNORE_INO => InodeKind::Gitignore,
            n if n >= TREE_SYMLINK_INO_BASE => InodeKind::TreeSymlink,
            n if n >= TREE_DIR_INO_BASE => InodeKind::TreeDir,
            n if n >= FIRST_ISSUE_INODE => InodeKind::RealFile,
            _ => InodeKind::Unknown,
        }
    }
    ```
    Use `classify(ino)` in the first line of `lookup`, `getattr`, `read`, `readdir`, `readlink` to branch on kind before any other work.

    **`lookup(parent, name, reply)`** — dispatch on `classify(parent)`:
    - `Root`: name == `.gitignore` → reply with `FileAttr { ino: GITIGNORE_INO, size: 7, kind: RegularFile, perm: 0o444, .. }`. name == bucket → reply with Dir attrs (BUCKET_DIR_INO). name == "tree" AND emit_tree is true → reply with Dir attrs (TREE_ROOT_INO). Else ENOENT.
    - `Bucket`: existing logic (match `<padded-id>.md` → look up in registry → build `FileAttr`).
    - `TreeRoot`: match `name` against `self.tree.read().unwrap().root_children()` — for Dir entries reply Dir attrs, for Symlink entries reply Symlink attrs (size = target.len()).
    - `TreeDir(ino)`: match `name` against `self.tree.read().unwrap().resolve_dir(parent.0).unwrap().children` — same dispatch as TreeRoot.
    - Everything else: ENOENT.

    **`getattr(ino, reply)`** — dispatch on `classify(ino)`:
    - `Root`, `Bucket`, `TreeRoot`, `TreeDir`: Directory attrs with appropriate size (0 for synthesized dirs is fine).
    - `Gitignore`: RegularFile, size 7, perm 0o444 (read-only).
    - `RealFile`: existing logic.
    - `TreeSymlink`: resolve via `tree.read().resolve_symlink(ino.0)`, build `FileAttr { kind: FileType::Symlink, size: target.as_bytes().len() as u64, perm: 0o777, .. }`.
    - `Unknown`: ENOENT.

    **`read(ino, offset, size, reply)`** — extend with Gitignore case:
    ```rust
    InodeKind::Gitignore => {
        const CONTENT: &[u8] = b"/tree/\n";
        let start = offset.min(CONTENT.len() as i64) as usize;
        let end = (start + size as usize).min(CONTENT.len());
        reply.data(&CONTENT[start..end]);
    }
    ```
    Real-file case unchanged. TreeSymlink and TreeDir cases return `EINVAL` (you don't `read()` a symlink — you `readlink()` it).

    **`readdir(ino, ..., reply)`** — dispatch on `classify(ino)`:
    - `Root`: emit `.`, `..`, `.gitignore` (GITIGNORE_INO, RegularFile), bucket name (BUCKET_DIR_INO, Directory), and conditionally `tree` (TREE_ROOT_INO, Directory) if `self.emit_tree || !self.tree.read().unwrap().is_empty()`.
    - `Bucket`: existing logic — list all `<padded-id>.md` via registry.
    - `TreeRoot`: emit `.`, `..`, then iterate `self.tree.read().unwrap().root_children()` — for each entry, emit with the right FileType (Directory or Symlink).
    - `TreeDir(ino)`: emit `.`, `..`, then iterate `self.tree.read().unwrap().resolve_dir(ino.0).unwrap().children`.
    - Everything else: EINVAL.

    **NEW: `readlink(ino, reply)`** — add the method (currently absent from the trait impl):
    ```rust
    fn readlink(&self, _req: &Request, ino: INodeNo, reply: ReplyData) {
        let snap = self.tree.read().unwrap();
        match snap.resolve_symlink(ino.0) {
            Some(target) => reply.data(target.as_bytes()),
            None => reply.error(libc::ENOENT),
        }
    }
    ```

    **Update `crates/reposix-fuse/tests/readdir.rs`**:
    - Change the `read_dir(mount)` assertion from `assert_contains_file("00000000001.md", ...)` to `assert_contains_file("issues/00000000001.md", ...)` (adjust to match the actual existing test idiom; the point is the entries now live under `mount/issues/` not `mount/`).
    - Assert `mount/.gitignore` exists and contains `/tree/\n`.
    - Assert `mount/tree/` does NOT exist (sim backend → supports(Hierarchy) == false, no parent_ids).
    - Do NOT change the test file name or the test name — keep the git history clean.

    Run all existing + updated tests. `cargo test -p reposix-fuse --release -- --ignored --test-threads=1` for the FUSE-mount half.
  </action>
  <verify>
    <automated>cargo test -p reposix-fuse --locked &amp;&amp; cargo test -p reposix-fuse --release --locked -- --ignored --test-threads=1 readdir &amp;&amp; cargo clippy --workspace --all-targets --locked -- -D warnings</automated>
  </verify>
  <done>
    `fs.rs` has `classify` helper; `lookup`/`getattr`/`read`/`readdir`/`readlink` all dispatch on inode kind. `.gitignore` and `<bucket>/` and (conditional) `tree/` synthesized at root. Existing `tests/readdir.rs` updated and still green under sim backend. Commit: `feat(13-C-2): synthesize bucket/.gitignore/tree root + readlink dispatch`.
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 3: Add `tests/nested_layout.rs` wiremock-Confluence FUSE integration test</name>
  <files>
    crates/reposix-fuse/tests/nested_layout.rs
  </files>
  <behavior>
    New integration test file with the following `#[test]` fns (all `#[ignore]`-gated with `--test-threads=1` convention matching `readdir.rs`):

    - `nested_layout_three_level_hierarchy`: wiremock returns 4 pages per the demo shape (1 root homepage, 3 children under it). Mount with `ConfluenceReadOnlyBackend::new_with_base_url(server.uri(), ...)`. Assert:
      - `read_dir(mnt)` contains `pages`, `tree`, `.gitignore` (exactly; no `0*.md` at root).
      - `read_dir(mnt/pages)` contains 4 `<padded>.md` entries.
      - `read_dir(mnt/tree)` contains 1 entry named `reposix-demo-space-home` (a directory, since it has children).
      - `read_dir(mnt/tree/reposix-demo-space-home)` contains exactly `_self.md`, `welcome-to-reposix.md`, `architecture-notes.md`, `demo-plan.md` (modulo IssueId tiebreak; the wiremock fixture should set the IDs so the slugs are deterministic).
      - `std::fs::read_link(mnt/tree/reposix-demo-space-home/_self.md)` → `"../../pages/00000360556.md"` (exact match).
      - `std::fs::read_link(mnt/tree/reposix-demo-space-home/welcome-to-reposix.md)` → `"../../pages/00000131192.md"`.
      - `std::fs::read_to_string(mnt/tree/reposix-demo-space-home/welcome-to-reposix.md)` returns the body of page 131192 (proves the symlink resolves through FUSE's own lookup+read pipeline). First line must be `---` (frontmatter fence).
      - `std::fs::read_to_string(mnt/.gitignore)` == `"/tree/\n"`.
    - `nested_layout_collision_gets_suffixed`: wiremock returns 3 pages with identical title "Same Title" under the same parent. Mount. Assert `read_dir(mnt/tree/<parent>)` contains `_self.md`, `same-title.md`, `same-title-2.md`, `same-title-3.md` (ascending-IssueId tiebreak matches Wave-A dedupe contract).
    - `nested_layout_cycle_does_not_hang`: wiremock returns pages A (parent=B.id), B (parent=A.id). Mount. Assert readdir completes within 3s and doesn't panic; the FUSE mount stays alive; both pages appear as tree roots (cycle broken per B2 contract). Capture `tracing` output via `tracing_subscriber::fmt::with_test_writer` or simply assert the mount is usable — the WARN log is an audit nicety, not a mandatory test assertion.
    - `nested_layout_gitignore_content_exact`: `read_to_string(mnt/.gitignore)` == `/tree/\n` (byte-for-byte; 7 bytes; trailing newline required).
    - `nested_layout_readlink_target_depth_is_correct`: build a 3-level tree, `readlink` at each depth, assert target has exactly `depth + 1` `../` prefixes.

    All tests use `#[ignore]` + `--test-threads=1` (file-level header comment documents this).
  </behavior>
  <action>
    Create `crates/reposix-fuse/tests/nested_layout.rs` modeled after the existing `tests/readdir.rs` (which does the wiremock + FUSE mount dance). Key differences:

    1. Use `reposix_confluence::ConfluenceReadOnlyBackend` as the backend (not sim). Phase 11-C shipped a contract test with this exact setup — see `crates/reposix-confluence/tests/` for the wiremock fixture pattern.

    2. File-level header documenting `--test-threads=1`:
    ```rust
    //! FUSE integration tests for the Phase-13 nested mount layout.
    //!
    //! Gated behind `#[ignore]`. Run with:
    //!   cargo test -p reposix-fuse --release -- --ignored --test-threads=1 nested_layout
    //!
    //! Why `--test-threads=1`: each test mounts FUSE in a tempdir; concurrent
    //! mounts race on fusermount3 -u and leave the system in an inconsistent
    //! state. Matches the existing `readdir.rs` convention.
    ```

    3. **Test fixture helper** (private fn at top of file):
    ```rust
    fn demo_space_fixture() -> Vec<serde_json::Value> {
        // 4 pages, 3-level deep:
        //   360556 "reposix demo space Home"  (root, no parent)
        //     131192 "Welcome to reposix"     (parent=360556)
        //     65916  "Architecture notes"     (parent=360556)
        //     425985 "Demo plan"              (parent=360556)
        // Matches the CONTEXT.md §specifics demo target.
        vec![ /* ... serde_json!({...}) objects ... */ ]
    }
    ```

    4. **Mount + assert helper** — reuse whatever helper `readdir.rs` uses to spin up wiremock + FUSE + tempdir. If none exists as a reusable helper, factor out the common dance into a private fn in `nested_layout.rs`.

    5. Write the five tests listed in the behavior section.

    Run with:
    ```bash
    cargo test -p reposix-fuse --release --locked -- --ignored --test-threads=1 nested_layout
    ```

    If `fusermount3` is not installed on the dev host (unlikely per CLAUDE.md but possible), document in SUMMARY and fall back to running in CI only.
  </action>
  <verify>
    <automated>cargo test -p reposix-fuse --release --locked -- --ignored --test-threads=1 nested_layout 2>&amp;1 | tee /tmp/13C-nested-layout.log &amp;&amp; grep -q "test result: ok" /tmp/13C-nested-layout.log &amp;&amp; grep -oE 'test result: ok\. [0-9]+ passed' /tmp/13C-nested-layout.log | head -1 | awk '{ exit ($4 &gt;= 5 ? 0 : 1) }'</automated>
  </verify>
  <done>
    All 5 nested_layout tests pass on a fusermount3-capable host. Symlink targets verified byte-for-byte. `.gitignore` content verified. Cycle test proves the mount doesn't hang. Commit: `feat(13-C-3): FUSE integration tests for nested mount layout`.
  </done>
</task>

<task type="auto">
  <name>Task 4: Workspace-wide green check (incl. --ignored)</name>
  <files>
    (no file edits; validation only)
  </files>
  <action>
    ```bash
    cargo fmt --all --check
    cargo clippy --workspace --all-targets --locked -- -D warnings
    cargo test --workspace --locked
    cargo test --workspace --release --locked -- --ignored --test-threads=1
    ```
  </action>
  <verify>
    <automated>cargo fmt --all --check &amp;&amp; cargo clippy --workspace --all-targets --locked -- -D warnings &amp;&amp; cargo test --workspace --locked &amp;&amp; cargo test --workspace --release --locked -- --ignored --test-threads=1</automated>
  </verify>
  <done>
    Full workspace green including the `--ignored` FUSE integration half. No unmount leaks (check `mount | grep reposix` returns empty after tests).
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Kernel VFS → FUSE readlink callback | The kernel blindly trusts whatever bytes `reply.data(...)` returns. A malformed target could theoretically escape the mount — but targets are constructed only from our own trusted state (bucket, depth, padded_id). |
| Tainted issue list (from Confluence) → tree rebuild | Same trust posture as Wave B2: cycle/orphan handled by `TreeSnapshot::build`. Wave C just consumes. |
| Write path via symlink | `cat > tree/foo.md` opens the symlink target (`pages/<id>.md`) via kernel path resolution. We do NOT need custom write logic; the default POSIX semantics route the write to our existing `pages/` write handler (which remains read-only in v0.4 for Confluence; SimBackend has write support elsewhere). |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|---|---|---|---|---|
| T-13-05 | Tampering | Symlink escaping mount | mitigate | `readlink` returns bytes from `TreeSnapshot::resolve_symlink`, which comes from `tree.rs` and was validated by B2's `symlink_target_never_escapes_mount_property` test. C's integration test `nested_layout_readlink_target_depth_is_correct` is the end-to-end audit. |
| T-13-C1 | DoS | `readdir(tree/)` on a pathological space (10k pages, deep) blocks the kernel | accept + note | Tree rebuild is O(n) per `list_issues` refresh. Phase 11 caps `list_issues` at 500 pages. If that cap is lifted (OP-7), revisit. Not a new vector vs. the existing Phase-3 `readdir` path. |
| T-13-C2 | Tampering | `.gitignore` content injection | mitigate | Content is a compile-time `&'static [u8] = b"/tree/\n"`. No user input. Inode 4 is read-only (`perm: 0o444`). |
| T-13-C3 | Information disclosure | Mount reveals tree structure to any local user | accept | FUSE mounts inherit POSIX permissions. The mount is owned by the user who ran `reposix mount`; default perms apply. Same risk profile as a read-accessible git repo. |

**Block-on-high:** T-13-05 integration test `nested_layout_readlink_target_depth_is_correct` is mandatory green before commit.
</threat_model>

<verification>
Nyquist coverage:
- **Unit (inode module):** `fixed_inodes_are_disjoint_from_dynamic_ranges` + existing tests updated for the new reserved-range boundary (5..=0xFFFF instead of 2..=0xFFFF).
- **Integration (sim backend, existing readdir test):** Updated to look at `mount/issues/<padded>.md`; `.gitignore` content verified; `tree/` absence under sim verified.
- **Integration (confluence wiremock, new nested_layout test):** 5 tests — 3-level hierarchy, collision, cycle, `.gitignore` exact content, readlink depth.
- **Workspace-wide:** `--ignored` pass included — real FUSE mounts exercised.
</verification>

<success_criteria>
Each a Bash assertion runnable from repo root:

1. `grep -qE 'pub const BUCKET_DIR_INO: u64 = 2' crates/reposix-fuse/src/inode.rs` exits 0.
2. `grep -qE 'pub const GITIGNORE_INO: u64 = 4' crates/reposix-fuse/src/inode.rs` exits 0.
3. `grep -qE 'fn readlink' crates/reposix-fuse/src/fs.rs` exits 0.
4. `grep -qE 'TreeSnapshot' crates/reposix-fuse/src/fs.rs` exits 0.
5. `grep -qE 'root_collection_name' crates/reposix-fuse/src/fs.rs` exits 0.
6. `grep -qE 'BackendFeature::Hierarchy' crates/reposix-fuse/src/fs.rs` exits 0.
7. `grep -qE '/tree/\\\\n' crates/reposix-fuse/src/fs.rs` exits 0 (gitignore content const).
8. `test -f crates/reposix-fuse/tests/nested_layout.rs` exits 0.
9. `cargo test -p reposix-fuse --locked` exits 0.
10. `cargo test -p reposix-fuse --release --locked -- --ignored --test-threads=1 nested_layout` exits 0 with ≥5 tests passing.
11. `cargo test -p reposix-fuse --release --locked -- --ignored --test-threads=1 readdir` exits 0 (pre-existing readdir test still green under updated layout).
12. `cargo clippy --workspace --all-targets --locked -- -D warnings` exits 0.
13. `mount | grep -c reposix` returns `0` after the test suite (no leaked FUSE mounts).
</success_criteria>

<output>
After completion, create `.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-C-SUMMARY.md` documenting:
- Final inode layout chosen (paste the doc comment)
- Final test counts (readdir + nested_layout)
- Any deviations from the suggested `classify()` dispatch pattern
- T-13-05 integration test output (paste the `readlink` assertion line for one deep symlink)
- Confirmation that `mount | grep reposix` returns empty after the suite runs
</output>
