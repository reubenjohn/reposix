---
phase: 13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren
plan: C
subsystem: reposix-fuse
wave: 3
tags: [fuse, integration, readlink, nested-layout, phase-13, wave-c]
status: complete
completed: 2026-04-14
requires:
  - 13-A (Issue::parent_id, BackendFeature::Hierarchy, root_collection_name)
  - 13-B1 (Confluence adapter populates parent_id + supports Hierarchy)
  - 13-B2 (reposix_fuse::tree::TreeSnapshot + inode-range constants)
  - 13-B3 (frontmatter parent_id round-trip tests)
provides:
  - FUSE mount root layout: .gitignore + <bucket>/ + (optional) tree/
  - readlink callback resolving tree-symlink inodes via TreeSnapshot
  - Inode-range classifier (InodeKind::classify) used by every callback
  - 11-digit zero-padded on-disk filenames (match B2 symlink targets)
  - 5 new FUSE integration tests under #[ignore] flag
affects:
  - crates/reposix-fuse/src/fs.rs (major rewrite; +~460 / −275 LOC)
  - crates/reposix-fuse/src/inode.rs (new fixed-inode consts + test)
  - crates/reposix-fuse/tests/readdir.rs (new root layout assertions)
  - crates/reposix-fuse/tests/sim_death_no_hang.rs (path update)
  - crates/reposix-fuse/tests/nested_layout.rs (NEW, 598 LOC, 5 tests)
tech-stack:
  added: []  # zero new deps; std::sync::RwLock + existing reposix-core/tree
  patterns:
    - "u64 inode-range dispatch via enum classifier before any HashMap lookup"
    - "Shared std::sync::RwLock<TreeSnapshot> rebuilt on each list_issues refresh"
    - "Synthesized read-only .gitignore: compile-time const bytes, 0o444 perm"
    - "Symlink attr size = target.len() (T-13-05; restic-bug avoidance)"
    - "Inode write path gated to InodeKind::RealFile; everything else EROFS/EPERM"
    - "Dispatch on classify(parent) for lookup/readdir to split root / bucket / tree"
key-files:
  created:
    - crates/reposix-fuse/tests/nested_layout.rs
  modified:
    - crates/reposix-fuse/src/fs.rs
    - crates/reposix-fuse/src/inode.rs
    - crates/reposix-fuse/tests/readdir.rs
    - crates/reposix-fuse/tests/sim_death_no_hang.rs
decisions:
  - "Store TreeSnapshot inside an Arc<RwLock<...>> so readdir(bucket) and readdir(root) can both rebuild it from refresh_issues() without contention. Wave B2 ships the snapshot as pure data, so a plain RwLock<> is sufficient — no interior mutability on the snapshot itself."
  - "Added InodeKind::classify as a tiny helper enum. Preferred over a bare range-match inside each callback because the dispatch also needs to run inside setattr/write/release/create/unlink for EROFS gating, and the enum keeps the invariant (root+bucket+tree_root+gitignore BELOW FIRST_ISSUE_INODE BELOW tree_dir_base BELOW tree_symlink_base) in one readable place."
  - "Chose 11-digit zero-padded filenames in the bucket to match B2's symlink target construction exactly. The alternative — patching B2 to use 4-digit padding — would have broken the T-13-05 target-escape property test (which checks for 11-digit suffixes). 11-digit accommodates astronomical Confluence page ids without overflow."
  - "Root readdir calls refresh_issues() once so the tree snapshot is ready before the kernel's follow-up lookup('tree') call. Without this, the first ls mount/ on Confluence would miss tree/ and require a second readdir to see it."
  - "Gitignore bytes are a compile-time `const GITIGNORE_BYTES: &[u8] = b\"/tree/\\n\";` — no runtime input, no format! call. Mitigates T-13-C2 (content injection) by construction."
  - "Set perm: 0o444 on .gitignore (and FileType::RegularFile) — the kernel will refuse write() at the VFS layer, but we also EROFS-reject write() on InodeKind::Gitignore in our own dispatch as belt-and-suspenders."
  - "The `_self.md` FileType is Symlink (not RegularFile) because it's allocated from TREE_SYMLINK_INO_BASE — the classifier doesn't distinguish leaf symlinks from _self.md by inode range, and the TreeSnapshot::resolve_symlink API returns them interchangeably."
  - "setattr on a tree dir or tree root returns the current dir attrs (not EROFS). Rationale: linux tools like `rm -rf` issue setattr(mode) as a sanity step before unlink; returning the dir unchanged is harmless and avoids a noisy error log."
  - "Cycle-safe behavior (T-13-03) is entirely delegated to Wave B2's TreeSnapshot::build. Wave C's cycle test runs FUSE-side and asserts the mount stays responsive within a 5s budget — NOT that the cycle diagnostic log fires."
metrics:
  duration_min: ~45
  tasks_completed: 4  # inode const (1), fs.rs wiring (2), nested_layout.rs tests (3), workspace green (4)
  files_modified: 4
  files_created: 1
  commits: 3  # task commits; summary commit follows
  tests_added: 6  # 1 inode:: + 5 nested_layout::
---

# Phase 13 Plan C: FUSE Wiring Summary

Wave-C integrator ships the Phase-13 nested mount layout end-to-end. The FUSE
daemon now synthesizes `.gitignore`, the per-backend `<bucket>/` directory,
and the conditional `tree/` overlay at the mount root. A new `readlink`
callback resolves tree-symlink inodes via the `TreeSnapshot` built in B2.
Every `lookup`/`getattr`/`readdir`/`read`/`readlink` callback runs a
`InodeKind::classify(ino)` range dispatch before any HashMap lookup, and
write-path callbacks are gated to real-file inodes only.

## Plan Intent

Wire `TreeSnapshot` (Wave B2) into `ReposixFs` (`fs.rs`) with the minimum
surface area: six new struct fields, one classifier enum, one `readlink`
impl, and dispatch rewrites for all existing Filesystem-trait methods. Flip
the on-disk filename padding from the legacy 4-digit `{:04}.md` to the
B2-compatible 11-digit `{:011}.md`, and update the sim-backend integration
tests to match. Land a new wiremock-Confluence + FUSE-mount integration
test exercising 3-level hierarchy, collision dedup, cycle safety, gitignore
content, and symlink-depth correctness.

## Final Inode Layout (module doc of `inode.rs`, copy-paste)

| Range | Purpose |
|-------|---------|
| `1` | Mount root (FUSE convention; `ROOT_INO`). |
| `2` | `BUCKET_DIR_INO` — the per-backend collection directory (`pages/` or `issues/`). |
| `3` | `TREE_ROOT_INO` — the synthesized `tree/` overlay root. |
| `4` | `GITIGNORE_INO` — the synthesized `/tree/\n` `.gitignore` file. |
| `5..=0xFFFF` | Reserved for future synthetic files. Never allocated. |
| `0x1_0000..` | Real issue/page files under `<bucket>/<padded-id>.md`. |
| `0x8_0000_0000..0xC_0000_0000` | `tree/` interior directories. |
| `0xC_0000_0000..u64::MAX` | `tree/` leaf symlinks and `_self.md` entries. |

## Tasks Executed

### Task 1 — Fixed inode constants

Added to `crates/reposix-fuse/src/inode.rs`:

- `pub const ROOT_INO: u64 = 1;`
- `pub const BUCKET_DIR_INO: u64 = 2;`
- `pub const TREE_ROOT_INO: u64 = 3;`
- `pub const GITIGNORE_INO: u64 = 4;`

Updated `FIRST_ISSUE_INODE` doc to cite 5..=0xFFFF as reserved (was
2..=0xFFFF). Updated the `reserved_range_is_unmapped` test to scan
5..=0xFFFF (1..=4 are now fixed synthetic slots, also never allocated by
the dynamic registry). Added new `fixed_inodes_are_disjoint_from_dynamic_ranges`
test that pins:

- All four fixed slots < `FIRST_ISSUE_INODE`.
- `FIRST_ISSUE_INODE < tree::TREE_DIR_INO_BASE < tree::TREE_SYMLINK_INO_BASE`.
- Pairwise distinctness of the four fixed slots.
- `inode::TREE_ROOT_INO == tree::TREE_ROOT_INO`.

### Task 2 — FUSE wiring in `fs.rs`

The core integrator. Major changes:

1. **New struct fields** (`ReposixFs`):
   - `bucket: &'static str` — set at construction from
     `backend.root_collection_name()`.
   - `hierarchy_feature: bool` — set from
     `backend.supports(BackendFeature::Hierarchy)`.
   - `tree: Arc<RwLock<TreeSnapshot>>` — rebuilt on each
     `refresh_issues()` call.
   - `bucket_attr`, `tree_attr`, `gitignore_attr: FileAttr` — cached
     to avoid recomputation on every callback.

2. **`InodeKind::classify(ino) -> InodeKind`** — small enum with 8
   variants that every callback branches on as its first action.

3. **Root-level synthesis** (`lookup` + `readdir` on `ROOT_INO`):
   - `.gitignore` → inode 4, RegularFile, 7 bytes.
   - `<bucket>` → inode 2, Directory.
   - `tree` → inode 3, Directory, conditional on `should_emit_tree()`.

4. **Bucket-level dispatch** (parent == `BUCKET_DIR_INO`):
   - `lookup(name)` passes through existing `resolve_name` +
     registry + frontmatter render path.
   - `readdir` emits one `<padded-id>.md` entry per issue at
     11-digit zero-padded.
   - `create` / `unlink` still land here and still drive the
     existing PATCH/POST write paths.

5. **Tree-level dispatch** (`TreeRoot` + `TreeDir`):
   - `lookup` scans `TreeEntry`s of the parent dir, returns either
     a Directory (for `TreeEntry::Dir`) or a Symlink (for
     `TreeEntry::Symlink`). Symlink FileAttr has `size =
     target.len()` per T-13-05 / restic-bug avoidance.
   - `readdir` flattens entries via `collect_tree_entries`.

6. **`readlink` (NEW)**:

   ```rust
   fn readlink(&self, _req: &Request, ino: INodeNo, reply: ReplyData) {
       match InodeKind::classify(ino.0) {
           InodeKind::TreeSymlink => {
               let Ok(snap) = self.tree.read() else {
                   reply.error(fuser::Errno::from_i32(libc::EIO));
                   return;
               };
               match snap.resolve_symlink(ino.0) {
                   Some(target) => reply.data(target.as_bytes()),
                   None => reply.error(fuser::Errno::from_i32(libc::ENOENT)),
               }
           }
           _ => reply.error(fuser::Errno::from_i32(libc::EINVAL)),
       }
   }
   ```

7. **Write-path gating**: `setattr` / `write` / `release` /
   `create` / `unlink` all check `InodeKind::classify` at entry.
   Real-file writes pass through the existing Phase-S machinery.
   Everything else rejects with `EROFS` / `EPERM` / `EISDIR` /
   `ENOTDIR` as appropriate.

8. **Filename padding normalized to 11 digits** in
   `issue_filename(id) = format!("{:011}.md", id.0)`. Matches Wave B2's
   `symlink_target()` which constructs `../<bucket>/<11-digit>.md`
   so every symlink target resolves to a real file.

Updated `tests/readdir.rs` (sim backend):

- `ls mount/` → `.gitignore` + `issues/` (NO `tree/` because sim
  doesn't advertise Hierarchy and fixture issues have no
  `parent_id`).
- `cat mount/.gitignore` → exactly `/tree/\n` (7 bytes).
- `ls mount/issues/` → `00000000001.md` / `00000000002.md` /
  `00000000003.md`.
- `cat mount/issues/00000000001.md` → starts with `---\n`, contains
  `id: 1`.

Updated `tests/sim_death_no_hang.rs` to poll `mount/issues/` and stat
`mount/issues/00000000001.md` for the post-backend-death path.

### Task 3 — `tests/nested_layout.rs` (NEW)

Five `#[ignore]`-gated integration tests run via
`cargo test -p reposix-fuse --release -- --ignored --test-threads=1 nested_layout`:

1. **`nested_layout_three_level_hierarchy`** — 4-page Confluence demo
   fixture (homepage + 3 children). Asserts full root layout, 4
   padded-id entries in `pages/`, 1 root dir in `tree/` with
   `_self.md` + 3 children inside. Verifies exact readlink target
   strings and that `cat mount/tree/.../welcome-to-reposix.md`
   yields identical bytes to `cat mount/pages/00000131192.md`.

2. **`nested_layout_collision_gets_suffixed`** — 3 siblings named
   "Same Title" under a common parent. Confirms dedup ordering:
   ascending-`IssueId` keeps the bare slug; next two get `-2`,
   `-3` suffixes. Readlink targets checked byte-for-byte.

3. **`nested_layout_cycle_does_not_hang`** — Two-page parent_id
   cycle (`A.parent=B, B.parent=A`). Asserts `Mount::open` returns
   within 5s and `readdir tree/` returns within 3s. Both pages
   surface as tree roots.

4. **`nested_layout_gitignore_content_exact`** — Byte-for-byte
   check: `read(".gitignore") == b"/tree/\n"`, `len() == 7`,
   `metadata().len() == 7`, `metadata().is_file()`.

5. **`nested_layout_readlink_target_depth_is_correct`** — 3-level
   chain (grandparent → parent → child). Asserts each symlink's
   target has `depth + 1` leading `../` components:

   ```
   tree/grandparent/_self.md       -> ../../pages/00000000001.md   (depth 1, 2 ../)
   tree/grandparent/parent/_self.md -> ../../../pages/00000000002.md (depth 2, 3 ../)
   tree/grandparent/parent/child.md -> ../../../pages/00000000003.md (depth 2, 3 ../)
   ```

All 5 pass on fusermount3 3.9.0.

### Task 4 — Workspace-wide green check

- `cargo fmt --all --check` — clean (one pass of `cargo fmt --all`
  normalized three cosmetic nits in newly-written code).
- `cargo clippy --workspace --all-targets --locked -- -D warnings` —
  clean.
- `cargo test --workspace --locked` — all green.
- `cargo test --workspace --release --locked -- --ignored --test-threads=1` —
  green EXCEPT `contract_github` (env-gated live test, pre-existing
  behavior — documented in `deferred-items.md`).
- `mount | grep reposix` — returns empty (no leaked FUSE mounts).

## T-13-05 Integration Proof

`nested_layout_readlink_target_depth_is_correct` passes. One sample
assertion line from the test (line 578 of `nested_layout.rs`):

```rust
assert_eq!(
    child.to_string_lossy(),
    "../../../pages/00000000003.md",
    "child target"
);
```

Run output:

```
running 1 test
test nested_layout_readlink_target_depth_is_correct ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.01s
```

The target is byte-for-byte equal to the expected `../../../pages/00000000003.md`.
The "no double slash, no absolute path" invariant established in Wave B2's
`readlink_target_never_contains_double_slash_or_absolute_path` unit test
rides through unchanged — Wave C only consumes the target string from
`TreeSnapshot::resolve_symlink`.

## Mount Cleanliness Proof

After the `--ignored` run completes:

```
$ mount | grep reposix
(no output)
```

Confirms every test's `unmount_and_wait()` helper drops the mount cleanly
via `BackgroundSession::drop` (fuser's `UmountOnDrop`) and waits up to 3s
for the kernel to release the tempdir.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 — Clippy] Match-on-`self.tree.read()` → `let...else`**
- **Found during:** Task 2 clippy gauntlet.
- **Issue:** Seven identical `match self.tree.read() { Ok(g) => g, Err(_) => { reply.error(...); return; } }` blocks were flagged by `clippy::match_single_binding` / `clippy::collapsible_match`.
- **Fix:** Single `Edit` with `replace_all=true` converted all seven to `let Ok(snap) = self.tree.read() else { reply.error(...); return; };`. Semantically identical; less noise.

**2. [Rule 1 — Clippy] `target.as_bytes().len()` → `target.len()`**
- **Found during:** Task 2 clippy gauntlet.
- **Issue:** `target.as_bytes().len()` on a `&str` is redundant per `clippy::needless_call_to_as_bytes`.
- **Fix:** Dropped `.as_bytes()` call; `&str::len()` returns byte length directly.

**3. [Rule 1 — Clippy] `unused_self` on `symlink_attr`**
- **Found during:** Task 2 clippy gauntlet.
- **Issue:** `symlink_attr` doesn't use `self` (only reads compile-time uid/gid/ino helpers).
- **Fix:** Added `#[allow(clippy::unused_self)]` with the rationale that keeping it as a method preserves symmetry with `file_attr` and `tree_dir_attr` (both of which might grow per-fs state in later phases).

**4. [Rule 1 — Clippy] `assertions_on_constants` in inode test**
- **Found during:** Task 1's test addition.
- **Issue:** The six `assert!(CONST_A < CONST_B)` lines in `fixed_inodes_are_disjoint_from_dynamic_ranges` trigger `clippy::assertions_on_constants` under pedantic — every comparison is a const-fold.
- **Fix:** Added a scoped `#[allow]` with a verbose `reason = "..."` explaining that the test IS the compile-time disjointness proof, and the lint's suggested `const { assert!(..) }` rewrite would require hoisting six comparisons out of the `#[test]` discovery path (making the invariant harder to grep for).

**5. [Rule 3 — Blocking] sim_death path update**
- **Found during:** Task 2 (running sim_death_no_hang after updating path padding to 11 digits).
- **Issue:** The test polls for `0001.md` at the old flat-root path; after 13-C, real files live at `mount/issues/00000000001.md`.
- **Fix:** Updated the polling closure to `read_dir(mount_path.join("issues"))` and the stat target to `mount_path.join("issues/00000000001.md")`. No semantic change to what the test proves.

No Rule-4 architectural escalations. No authentication gates encountered
during Wave-C execution. (The `contract_github` env-gated failure during
workspace `--ignored` is documented in deferred-items.md — it's a
pre-existing test hygiene issue in `reposix-github/tests/contract.rs`,
not a Wave-C regression.)

### Scope-Boundary Observations (deferred to future plans)

Documented in `.planning/phases/13-.../deferred-items.md`:

1. **`contract_github` test requires `REPOSIX_ALLOWED_ORIGINS`**:
   `crates/reposix-github/tests/contract.rs::contract_github` is
   `#[ignore]`-gated but hard-panics on `--ignored` without env. The
   Confluence sibling uses `skip_if_no_env!` — GitHub doesn't.
   Out-of-scope: file not in 13-C's list; failure pre-exists
   Wave-C changes.

2. **`reposix-remote` emits 4-digit paths**: the git remote helper
   produces a fast-import stream with `{:04}.md` blobs (3 sites in
   `diff.rs` / `fast_import.rs` / `protocol.rs`). The FUSE mount
   post-13-C surfaces `<bucket>/<11-digit>.md`. Wave D1's migration
   sweep is planned to unify these. Out-of-scope: `crates/reposix-remote/`
   is not in 13-C's file list.

## Commits

| Task | Hash | Message |
|------|------|---------|
| 1 | `2d04a5e` | `feat(13-C-1): declare fixed inodes for bucket, tree root, gitignore` |
| 2 | `b0146f2` | `feat(13-C-2): synthesize bucket/.gitignore/tree root + readlink dispatch` |
| 3 | `171c83f` | `feat(13-C-3): FUSE integration tests for nested mount layout` |
| meta | (pending) | `docs(13-C): summary + roadmap check-off` |

## Success Criteria Map

| SC | Assertion | Status |
|----|-----------|--------|
| 1 | `grep -qE 'pub const BUCKET_DIR_INO: u64 = 2' crates/reposix-fuse/src/inode.rs` | PASS |
| 2 | `grep -qE 'pub const GITIGNORE_INO: u64 = 4' crates/reposix-fuse/src/inode.rs` | PASS |
| 3 | `grep -qE 'fn readlink' crates/reposix-fuse/src/fs.rs` | PASS |
| 4 | `grep -qE 'TreeSnapshot' crates/reposix-fuse/src/fs.rs` | PASS |
| 5 | `grep -qE 'root_collection_name' crates/reposix-fuse/src/fs.rs` | PASS |
| 6 | `grep -qE 'BackendFeature::Hierarchy' crates/reposix-fuse/src/fs.rs` | PASS |
| 7 | `grep -qE '/tree/\\\\n' crates/reposix-fuse/src/fs.rs` | PASS (as `b"/tree/\n"`) |
| 8 | `test -f crates/reposix-fuse/tests/nested_layout.rs` | PASS |
| 9 | `cargo test -p reposix-fuse --locked` exits 0 | PASS |
| 10 | `cargo test -p reposix-fuse --release --locked -- --ignored --test-threads=1 nested_layout` exits 0 with ≥5 tests passing | PASS (5 passed) |
| 11 | `cargo test -p reposix-fuse --release --locked -- --ignored --test-threads=1 readdir` exits 0 | PASS (readdir is a non-ignored test; runs under both debug + release-ignored flow and passes) |
| 12 | `cargo clippy --workspace --all-targets --locked -- -D warnings` exits 0 | PASS |
| 13 | `mount | grep -c reposix` returns `0` after suite | PASS |

## Unblocks

- **Wave D1 (BREAKING migration sweep)**: Now has a green mount-root
  layout to sweep across docs, demos, the `reposix-cli::demo`
  command, README quickstart, and the `reposix-remote` crate's
  padding to 11-digit.
- **Wave D2 (docs / ADR-003)**: Can reference the full inode layout
  table and the T-13-05 `readlink` proof in the ADR.
- **Wave D3 (release script + demo.sh)**: Can pin the `tree/` →
  `pages/` readlink assertion as a smoke test.
- **Wave E (green gauntlet)**: `contract_github` env-gating is the
  sole known red in the `--ignored` matrix; fixing it gives E a
  clean slate.

## Self-Check: PASSED

- `crates/reposix-fuse/src/fs.rs`: FOUND (major rewrite; readlink
  present, classify() present, TreeSnapshot wired, bucket/hierarchy
  fields wired).
- `crates/reposix-fuse/src/inode.rs`: FOUND (BUCKET_DIR_INO /
  TREE_ROOT_INO / GITIGNORE_INO consts present; test
  `fixed_inodes_are_disjoint_from_dynamic_ranges` green).
- `crates/reposix-fuse/tests/readdir.rs`: FOUND (updated for new
  root layout; 1 test green).
- `crates/reposix-fuse/tests/nested_layout.rs`: FOUND (598 LOC; 5
  tests green under `--ignored`).
- Commit `2d04a5e`: FOUND in `git log`.
- Commit `b0146f2`: FOUND in `git log`.
- Commit `171c83f`: FOUND in `git log`.
- `cargo test -p reposix-fuse --locked`: PASS.
- `cargo test -p reposix-fuse --release --locked -- --ignored --test-threads=1`: PASS (6 ignored tests total: 5 nested_layout + 1 sim_death_no_hang).
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: PASS.
- `cargo fmt --all --check`: PASS.
- `mount | grep reposix`: empty.
