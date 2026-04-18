---
phase: 13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren
plan: B2
type: execute
wave: 2
depends_on: [A]
files_modified:
  - crates/reposix-fuse/src/tree.rs
  - crates/reposix-fuse/src/lib.rs
  - crates/reposix-fuse/Cargo.toml
autonomous: true
requirements:
  - OP-1
user_setup: []

must_haves:
  truths:
    - "`crates/reposix-fuse/src/tree.rs` is a new module with `pub struct TreeSnapshot`, `pub fn build_tree(bucket: &str, issues: &[Issue]) -> TreeSnapshot`, `TreeSnapshot::resolve_symlink(ino: u64) -> Option<String>`, `TreeSnapshot::resolve_dir(ino: u64) -> Option<&TreeDir>`, `TreeSnapshot::root_children() -> ...`"
    - "`build_tree` assigns inodes from two disjoint, pre-declared const ranges (TREE_DIR_INO_BASE and TREE_SYMLINK_INO_BASE) such that any u64 inode is trivially classified as kind ∈ {root_bucket, tree_dir, tree_symlink} by the consuming layer (C) without a hashmap lookup"
    - "When a page has ≥1 child: it materializes as a DIRECTORY with slug = `slug_or_fallback(title, id)`; the directory contains a `_self.md` symlink pointing at the page's own body plus one entry per child"
    - "When a page has 0 children: it materializes as a SYMLINK file (leaf) with slug = `slug_or_fallback(title, id)`"
    - "Symlink target = `\"../\".repeat(depth + 1) + &format!(\"{bucket}/{padded_id}.md\")` where depth is the depth in `tree/` (0 for direct children of `tree/`, 1 for grandchildren, ...)"
    - "Sibling collision dedup is applied per parent via `reposix_core::path::dedupe_siblings` (from Wave A)"
    - "Cycle detection: `build_tree` with an adversarial `issues` list where `a.parent_id = Some(b.id)` AND `b.parent_id = Some(a.id)` completes in O(n) without panic/stack-overflow; emits `tracing::warn!` for each cycle-broken edge; resulting tree treats the cycle-break node as a tree root"
    - "Orphan parents (parent_id points at an id not present in `issues`) are treated as tree roots"
    - "`_self` never collides with any slug because `slug_or_fallback` never emits `_self` (step 2 of the slug algorithm strips leading `_`)"
    - "The module is PURE (zero tokio, zero HTTP, zero FUSE trait implementations — just in-memory data transforms). It must compile standalone with only `reposix-core` + std."
    - "`cargo test -p reposix-fuse tree::` green with ≥ 12 unit tests"
  artifacts:
    - path: "crates/reposix-fuse/src/tree.rs"
      provides: "TreeSnapshot + TreeDir + TreeNode + build_tree + resolve_symlink + resolve_dir + const inode range bases"
      min_lines: 500
      contains: "pub fn build_tree"
    - path: "crates/reposix-fuse/src/lib.rs"
      provides: "`pub mod tree;` + re-exports of TreeSnapshot, TREE_DIR_INO_BASE, TREE_SYMLINK_INO_BASE"
      contains: "pub mod tree"
  key_links:
    - from: "crates/reposix-fuse/src/tree.rs"
      to: "reposix_core::path::{slug_or_fallback, dedupe_siblings}"
      via: "direct call in build_tree per-parent grouping"
      pattern: "slug_or_fallback|dedupe_siblings"
    - from: "crates/reposix-fuse/src/tree.rs"
      to: "reposix_core::Issue::parent_id"
      via: "grouping key in build_tree"
      pattern: "parent_id"
---

<objective>
Wave-B2. Build the pure in-memory tree module that converts `Vec<Issue>` into a navigable `TreeSnapshot` with inode-ranges, slug-resolved names, collision dedup, cycle detection, and depth-correct relative symlink targets. Zero FUSE coupling — this module does not import `fuser`. Wave C wires it into `fs.rs`.

Purpose: Splitting tree-building from FUSE integration means this plan can run fully parallel with B1 (Confluence wiring) and B3 (frontmatter), each on a disjoint file set. It also means `tree.rs` is unit-testable without FUSE mounts — huge for fast iteration.

Output: One new file `crates/reposix-fuse/src/tree.rs` (~500+ lines), module declaration in `lib.rs`, optional dev-dep on `proptest` in `Cargo.toml` (discretionary; skip if the deterministic tests below suffice).
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-CONTEXT.md
@.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-RESEARCH.md
@.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-A-core-foundations.md
@CLAUDE.md
@crates/reposix-fuse/src/lib.rs
@crates/reposix-fuse/src/inode.rs
@crates/reposix-fuse/Cargo.toml

<interfaces>
<!-- After Wave A ships, these types exist in reposix-core: -->

```rust
use reposix_core::{Issue, IssueId};
use reposix_core::path::{slug_or_fallback, dedupe_siblings, SLUG_MAX_BYTES};

// Issue has new parent_id field:
// pub struct Issue { /* ... */, pub parent_id: Option<IssueId> }
```

<!-- Existing inode registry (extend in C, NOT here): -->

```rust
// crates/reposix-fuse/src/inode.rs
pub const FIRST_ISSUE_INODE: u64 = 0x1_0000;  // real files in <bucket>/
// This plan introduces two NEW const bases. Wave C uses them to dispatch.
```

<!-- Proposed new public API in tree.rs: -->

```rust
/// Tree-interior-directory inodes start here. 0x8_0000_0000.
pub const TREE_DIR_INO_BASE: u64 = 0x8_0000_0000;
/// Tree-leaf-symlink inodes start here. 0xC_0000_0000.
pub const TREE_SYMLINK_INO_BASE: u64 = 0xC_0000_0000;
/// The fixed inode for the `tree/` root directory itself.
pub const TREE_ROOT_INO: u64 = 3;

pub struct TreeSnapshot { /* opaque */ }
pub struct TreeDir {
    pub ino: u64,
    pub name: String,           // slug (e.g. "architecture-notes")
    pub children: Vec<TreeEntry>,
    pub depth: usize,           // 0 for direct children of tree/, +1 per level
}
pub enum TreeEntry {
    Dir(u64),                   // inode; look up with resolve_dir
    Symlink { ino: u64, name: String, target: String }, // target is relative
}

impl TreeSnapshot {
    pub fn build(bucket: &str, issues: &[Issue]) -> Self;
    pub fn resolve_symlink(&self, ino: u64) -> Option<&str>;  // returns target
    pub fn resolve_dir(&self, ino: u64) -> Option<&TreeDir>;
    pub fn root_children(&self) -> &[TreeEntry];
    pub fn is_empty(&self) -> bool;
}
```
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Implement `tree.rs` module (types + `build_tree` + resolvers)</name>
  <files>
    crates/reposix-fuse/src/tree.rs,
    crates/reposix-fuse/src/lib.rs
  </files>
  <behavior>
    Every one of the following as a `#[test]` in `tree.rs::tests`:

    - `empty_input_produces_empty_tree`: `TreeSnapshot::build("pages", &[])` returns `is_empty() == true`.
    - `single_root_page_becomes_leaf_symlink`: one page, `parent_id = None`, no children. Tree has exactly one root entry, a Symlink with target `"../pages/00000000042.md"` (one `../` because depth 0 → `repeat(0+1) == 1`).
    - `parent_with_one_child_becomes_dir_with_self_and_child`:
      - page A (id=1, title="homepage", parent=None)
      - page B (id=2, title="welcome", parent=Some(1))
      Tree root has one Dir entry named `"homepage"` with 2 children: `_self.md` → `"../../pages/00000000001.md"` and `"welcome.md"` → `"../../pages/00000000002.md"`.
    - `three_level_hierarchy_builds_correct_depth`:
      - A (id=1, parent=None, title="root")
      - B (id=2, parent=1, title="child")
      - C (id=3, parent=2, title="grandchild")
      Resulting tree: `root/` (depth 0) contains `_self.md` + `child/` (depth 1). `child/` contains `_self.md` + `grandchild.md` (symlink). `_self.md` at depth 1 has target `"../../../pages/00000000002.md"` (three `../`). `grandchild.md` at depth 1 has target `"../../../pages/00000000003.md"`.
    - `sibling_collision_is_deduped`: two siblings with identical title under the same parent. Asserts the one with smaller IssueId keeps the bare slug, the other gets `-2`. Applies dedupe to the full slug-set per parent including `_self` pseudo-entries (but `_self` is reserved; no real page should slug to `_self`).
    - `orphan_parent_becomes_tree_root`: page B has `parent_id = Some(999)` but id 999 is not in the issues list. B appears as a tree root.
    - `cycle_is_broken_and_warned`: page A parent=Some(B.id), page B parent=Some(A.id). `build_tree` completes (no stack overflow); at least one of A, B appears as a root; WARN log emitted. Test asserts via `tracing_subscriber::fmt::test_writer` capture or a simpler approach: expose an `#[cfg(test)] fn build_tree_with_events(...) -> (TreeSnapshot, Vec<CycleEvent>)` helper that returns both the tree and a list of cycle-break events; assert the list is non-empty. The public `build` just calls this and drops the events.
    - `three_way_cycle_terminates`: A→B→C→A. Must terminate.
    - `deep_linear_chain_1000_deep`: 1000 pages chained A→B→C→... (no cycle). `build_tree` completes in < 100ms and produces a 1000-deep directory.
    - `symlink_target_never_escapes_mount_property`: for every Symlink in every test tree, assert `target.starts_with("../") && !target.contains("/../")` AND `target.ends_with(".md")` AND `target.split('/').filter(|s| *s == "..").count() == depth + 1` AND the non-`..` suffix matches regex `^<bucket>/\d{11}\.md$`. This is T-13-05 mitigation.
    - `inodes_are_in_declared_ranges`: every Dir inode is in `[TREE_DIR_INO_BASE, TREE_SYMLINK_INO_BASE)`, every Symlink inode is in `[TREE_SYMLINK_INO_BASE, u64::MAX)`. No collision with `FIRST_ISSUE_INODE (0x1_0000)` or `TREE_ROOT_INO (3)`.
    - `dedupe_applies_to_dir_vs_symlink_siblings`: a page with children (becomes Dir) and a sibling page (becomes Symlink) under the same parent, both titled "foo" → one keeps "foo", the other gets "foo-2". Dedup operates on slugs regardless of entry-kind.
    - `resolve_symlink_round_trip`: build a tree, iterate all symlink inodes, call `resolve_symlink(ino)`, assert each returns `Some(target)` matching the original `TreeEntry::Symlink { target, .. }`. Unknown inodes return `None`.
    - `resolve_dir_round_trip`: same idea for dirs.
    - `stable_output_across_calls`: calling `build_tree(bucket, &issues)` twice on the same input produces identical inode assignments (deterministic — iteration order comes from sorted IssueIds, not HashMap iteration).
  </behavior>
  <action>
    Create `crates/reposix-fuse/src/tree.rs` with the interface block above. Implementation notes:

    1. **Const declarations** at the top:
    ```rust
    pub const TREE_ROOT_INO: u64 = 3;
    pub const TREE_DIR_INO_BASE: u64 = 0x8_0000_0000;
    pub const TREE_SYMLINK_INO_BASE: u64 = 0xC_0000_0000;
    ```
    Add a debug_assert or const_assert that these ranges don't overlap with `inode::FIRST_ISSUE_INODE = 0x1_0000`. Document the layout in a module-level doc comment.

    2. **`build_tree` algorithm** (iterative, no recursion):
    ```rust
    pub fn build(bucket: &str, issues: &[Issue]) -> Self {
        // Step 1: index by id for O(1) lookup.
        let by_id: HashMap<IssueId, &Issue> = issues.iter().map(|i| (i.id, i)).collect();

        // Step 2: classify each issue as "root" or "child". Cycle-aware.
        //  - "root" if parent_id is None OR points outside by_id OR is part of a cycle.
        //  - DFS on the parent chain with a visited-set to detect cycles.
        //  - If a cycle is detected, break at the deepest ancestor seen twice:
        //    the cycle-break node becomes a root, and a WARN log is emitted.
        let mut cycle_events = vec![];
        let mut effective_parent: HashMap<IssueId, Option<IssueId>> = HashMap::new();
        for issue in issues {
            let mut seen = HashSet::new();
            seen.insert(issue.id);
            let mut cursor = issue.parent_id;
            let effective = loop {
                match cursor {
                    None => break None,                               // true root
                    Some(pid) if !by_id.contains_key(&pid) => break None,  // orphan parent -> root
                    Some(pid) if !seen.insert(pid) => {
                        // cycle: pid is an ancestor of itself
                        cycle_events.push(CycleEvent { page: issue.id, broken_at: pid });
                        tracing::warn!(page = %issue.id.0, ancestor = %pid.0, "parent-id cycle broken; treating as orphan root");
                        break None;
                    }
                    Some(pid) => cursor = by_id[&pid].parent_id,
                }
            };
            effective_parent.insert(issue.id, effective.or(issue.parent_id));
            // NOTE: above is approximate — we actually want to set the EFFECTIVE parent:
            //   if cycle detected, use None; else use the original parent_id (still valid)
            // Re-doing cleanly:
            let final_parent = match effective {
                None if issue.parent_id.is_some() && !by_id.contains_key(&issue.parent_id.unwrap()) => None,  // orphan
                None => None,         // true root OR cycle-broken
                Some(p) => Some(p),   // normal
            };
            effective_parent.insert(issue.id, final_parent);
        }
        // The loop above is too clever — see note. REAL impl:
        // - run DFS ONCE per issue, recording cycle breaks
        // - effective_parent[id] = Some(p) iff no cycle AND p is in by_id; else None
        // (Cleaner impl: detect cycles via a single pass, building a forest.)

        // Step 3: build children index from effective_parent.
        let mut children_of: HashMap<Option<IssueId>, Vec<IssueId>> = HashMap::new();
        for issue in issues {
            let ep = effective_parent[&issue.id];
            children_of.entry(ep).or_default().push(issue.id);
        }
        // Sort children lists by IssueId ascending for determinism (required by dedupe).

        // Step 4: allocate inodes + slugs + targets in a BFS from roots.
        //   A node has children iff children_of.get(&Some(id)).is_some_and(|v| !v.is_empty()).
        //   If has_children -> it's a Dir; else -> it's a Symlink leaf.
        //   Dir also gets a `_self.md` entry: a Symlink inside the dir, target depth+1.
        //   Apply dedupe_siblings per parent before materializing.
        //   Inode allocator: two monotonic counters seeded at the two const bases.

        // Step 5: store into TreeSnapshot:
        //   - dirs: HashMap<u64, TreeDir>
        //   - symlink_targets: HashMap<u64, String>
        //   - root_entries: Vec<TreeEntry>
    }
    ```

    The essential algorithm is: one DFS per node to compute the effective parent (handling cycles), then a single BFS from the resulting forest roots to assign inodes + slugs + targets.

    3. **Symlink target construction**:
    ```rust
    fn symlink_target(bucket: &str, padded_id: &str, depth: usize) -> String {
        let up = "../".repeat(depth + 1);  // +1 because tree/ itself is one level
        format!("{up}{bucket}/{padded_id}")
    }
    // Where padded_id = format!("{:011}.md", id.0), matching existing convention in fs.rs.
    ```
    Confirm the `011` padding matches the existing `<padded-id>.md` scheme by grepping `crates/reposix-fuse/src/fs.rs` for the format string currently used. Use whatever it uses (it's `:011` historically — verify before committing).

    4. **Slug application** (per parent group):
    - Compute `raw_slug = slug_or_fallback(&issue.title, issue.id)` for each sibling.
    - Pass `Vec<(IssueId, String)>` to `dedupe_siblings` — get back deduped `(IssueId, final_slug)`.
    - Append `.md` to symlink names; leave dir names bare.
    - Self-links inside a dir use the literal name `"_self.md"` with depth = parent_depth + 1.

    5. **Cycle detection — clean rewrite**:
    ```rust
    fn effective_parent_of(
        issue: &Issue,
        by_id: &HashMap<IssueId, &Issue>,
        cycle_events: &mut Vec<CycleEvent>,
    ) -> Option<IssueId> {
        let mut seen = HashSet::new();
        seen.insert(issue.id);
        let mut cursor = issue.parent_id?;
        loop {
            if !seen.insert(cursor) {
                // cursor already in chain → cycle
                cycle_events.push(CycleEvent { page: issue.id, broken_at: cursor });
                tracing::warn!(page = %issue.id.0, ancestor = %cursor.0, "parent-id cycle detected; treating page as orphan root");
                return None;
            }
            let Some(parent) = by_id.get(&cursor) else {
                // orphan parent: keep as-is (it's a valid root-producing parent pointer; we treat orphan parents as "this page is a root"). Return None so the page becomes a root.
                return None;
            };
            // The page's DIRECT parent is `cursor`. If cursor is not itself orphaned-or-cyclic, return it.
            // To detect cycle we need to walk upward; but for classifying THIS page's effective parent, we only need: does cursor exist? Then YES, return Some(cursor). The cycle case above already returned None.
            // ... simpler: return Some(issue.parent_id.unwrap()) iff the chain is finite.
            match parent.parent_id {
                Some(next) => cursor = next,
                None => return issue.parent_id,  // chain terminates at a root → this page's direct parent is the original parent_id, which is fine.
            }
        }
    }
    ```
    Pseudocode simplification: the function's real job is to answer "does the chain from this page terminate at a root?" — if yes, return the direct parent_id; if no (orphan OR cycle), return None.

    6. **Module module declaration in `crates/reposix-fuse/src/lib.rs`**:
    ```rust
    pub mod tree;
    pub use tree::{TreeSnapshot, TREE_DIR_INO_BASE, TREE_SYMLINK_INO_BASE, TREE_ROOT_INO};
    ```

    7. **Crate attributes** at the top of `tree.rs`:
    ```rust
    //! Pure in-memory tree builder + resolver for the Phase-13 `tree/` overlay.
    //! See 13-RESEARCH.md §"Recommended Project Structure" and §"Pattern 2"
    //! for context. This module does NOT import `fuser` — Wave C integrates it
    //! with the FUSE trait impl in `fs.rs`.
    // forbid/warn inherits from the crate root.
    ```

    8. **Dev-deps** (optional): if the agent picks proptest for collision resolution, add to `crates/reposix-fuse/Cargo.toml`:
    ```toml
    [dev-dependencies]
    proptest = "1"
    ```
    Otherwise, leave Cargo.toml unchanged. The deterministic tests listed above are sufficient coverage; proptest is the gravy.

    Run `cargo test -p reposix-fuse tree::` and `cargo clippy -p reposix-fuse --all-targets --locked -- -D warnings` until green.
  </action>
  <verify>
    <automated>cargo test -p reposix-fuse --locked tree:: &amp;&amp; cargo clippy -p reposix-fuse --all-targets --locked -- -D warnings &amp;&amp; cargo test -p reposix-fuse --locked tree::tests 2>&amp;1 | grep -oE 'test result: ok\. [0-9]+ passed' | head -1 | awk '{ exit ($4 &gt;= 12 ? 0 : 1) }'</automated>
  </verify>
  <done>
    `tree.rs` exists, compiles, and passes ≥ 12 unit tests. All T-13-03 (cycle), T-13-04 (collision), T-13-05 (symlink escape) invariants asserted in tests. Zero `fuser::` imports in `tree.rs` (grep to confirm). Module is re-exported from `lib.rs`. Commit: `feat(13-B2): add reposix-fuse::tree::TreeSnapshot + build + cycle-safe resolver`.
  </done>
</task>

<task type="auto">
  <name>Task 2: Workspace-wide green check</name>
  <files>
    (no file edits; validation only)
  </files>
  <action>
    ```bash
    cargo fmt --all --check
    cargo clippy --workspace --all-targets --locked -- -D warnings
    cargo test --workspace --locked
    ```
    B2 should be fully isolated — no downstream crate consumes it yet (C does, in the next wave).
  </action>
  <verify>
    <automated>cargo fmt --all --check &amp;&amp; cargo clippy --workspace --all-targets --locked -- -D warnings &amp;&amp; cargo test --workspace --locked</automated>
  </verify>
  <done>
    Full workspace still green. New `tree::` tests appear in the output.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Tainted parent_id → tree structure | Adversarial parent_id values can describe cycles or point outside the mounted set. Neither must wedge the builder. |
| Tainted title → dir/symlink name | `slugify_title` from Wave A sanitizes to `[a-z0-9-]`, so by the time we use the slug, it's trusted. But the Wave-A tests must already be green as a precondition. |
| Tree → FUSE kernel replies | Symlink target strings cross the FUSE boundary. They must never contain `\0`, must not be absolute paths, must not escape the mount via extra `..` components. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|---|---|---|---|---|
| T-13-03 | DoS | Cycle in parent chain causing infinite recursion | mitigate | Iterative DFS + visited HashSet in `effective_parent_of`. Test `cycle_is_broken_and_warned` + `three_way_cycle_terminates` prove termination. Tracing warn emitted for audit. |
| T-13-04 | Tampering | Sibling slug collision (post-truncation) | mitigate | Delegated to `dedupe_siblings` from Wave A. B2's test `sibling_collision_is_deduped` proves integration wired correctly. |
| T-13-05 | Tampering | Symlink target escapes mount (`../../../../etc/passwd`) | mitigate | Target is constructed from constants (bucket, padded_id, depth) — all internal. Property test `symlink_target_never_escapes_mount_property` validates regex `^(\.\./)+<bucket>/\d{11}\.md$` over every produced target in every test. |
| T-13-DOS1 | DoS | Pathological deep chain (10k-deep) | accept + test | Iterative algorithm is O(n), no stack cost. Test `deep_linear_chain_1000_deep` asserts <100ms. Phase-11 already enforces 500-page cap at `list_issues`, so 10k-deep is not reachable in practice. |

**Block-on-high:** T-13-05 test is mandatory — commit blocked if it fails.
</threat_model>

<verification>
Nyquist coverage:
- **Unit (in-module):** ≥12 tests covering tree shape, depth, collision, cycles, orphan parents, inode ranges, target correctness, stability.
- **Property:** `symlink_target_never_escapes_mount_property` runs over every symlink in every fixture and asserts the regex invariant.
- **Optional proptest:** agent discretion — if added, shrink-minimized counterexamples for the collision+cycle case.
- **Isolation:** `grep -c "fuser::" crates/reposix-fuse/src/tree.rs` returns 0 (module is FUSE-independent).
</verification>

<success_criteria>
Each a Bash assertion runnable from repo root:

1. `test -f crates/reposix-fuse/src/tree.rs` exits 0.
2. `grep -qE 'pub mod tree;' crates/reposix-fuse/src/lib.rs` exits 0.
3. `grep -qE 'pub struct TreeSnapshot' crates/reposix-fuse/src/tree.rs` exits 0.
4. `grep -qE 'pub fn build' crates/reposix-fuse/src/tree.rs` exits 0.
5. `grep -qE 'pub const TREE_DIR_INO_BASE: u64 = 0x8_0000_0000' crates/reposix-fuse/src/tree.rs` exits 0.
6. `grep -qE 'pub const TREE_SYMLINK_INO_BASE: u64 = 0xC_0000_0000' crates/reposix-fuse/src/tree.rs` exits 0.
7. `grep -c 'fuser::' crates/reposix-fuse/src/tree.rs | awk '{ exit ($1 == 0 ? 0 : 1) }'` exits 0 (module does not import fuser).
8. `cargo test -p reposix-fuse --locked tree::` exits 0.
9. `cargo test -p reposix-fuse --locked tree::tests 2>&1 | grep -oE 'test result: ok\. [0-9]+ passed' | head -1 | awk '{print $4}'` returns ≥ 12.
10. `cargo clippy --workspace --all-targets --locked -- -D warnings` exits 0.
11. `cargo test --workspace --locked` exits 0.
</success_criteria>

<output>
After completion, create `.planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-B2-SUMMARY.md` documenting:
- Final tree-module test count
- Whether proptest was adopted (agent's discretion per CONTEXT.md)
- The inode-range layout chosen and any deviations from the suggested 0x8_0000_0000 / 0xC_0000_0000 bases
- Confirmation that `tree.rs` has zero `fuser::` imports (paste the grep output)
- T-13-03 + T-13-05 test output evidence
</output>
