---
phase: 13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren
phase_name: Nested mount layout — pages/ + tree/ symlinks for Confluence parentId
subsystem: phase-level
tags: [phase-summary, v0.4.0, nested-mount, tree-overlay, confluence, op-1]
status: complete
completed: 2026-04-14
version: v0.4.0
requires:
  - Phase 11 (Confluence adapter)
  - Phase 12 (OP-4/OP-12 prebuilt binaries)
provides:
  - Nested mount layout (pages/ + tree/ + .gitignore)
  - FUSE readlink resolution
  - Confluence parent_id populated end-to-end
  - ADR-003 + CHANGELOG [v0.4.0] + scripts/tag-v0.4.0.sh + demo 07
plans_landed:
  - A: core foundations (Issue::parent_id, BackendFeature::Hierarchy, slug helpers)
  - B1: Confluence parent_id wiring
  - B2: FUSE tree module (TreeSnapshot, inode ranges)
  - B3: frontmatter parent_id round-trip tests
  - C: FUSE wiring (readlink, dispatch, 11-digit padding)
  - D1: BREAKING migration sweep (docs, demos, fixtures)
  - D2: docs + ADR-003 + CHANGELOG + CLAUDE.md + README
  - D3: release scripts + tier-5 confluence-tree demo
  - E: green gauntlet + live Confluence verify
key-files:
  created:
    - crates/reposix-fuse/src/tree.rs
    - crates/reposix-fuse/tests/nested_layout.rs
    - docs/decisions/003-nested-mount-layout.md
    - scripts/tag-v0.4.0.sh
    - scripts/demos/07-mount-real-confluence-tree.sh
  modified:
    - crates/reposix-core/src/{issue.rs,backend.rs,path.rs,lib.rs,taint.rs}
    - crates/reposix-confluence/src/lib.rs
    - crates/reposix-fuse/src/{fs.rs,inode.rs,lib.rs}
    - crates/reposix-fuse/tests/{readdir.rs,sim_death_no_hang.rs}
    - scripts/demos/{_lib.sh,01-edit-and-push.sh,05-mount-real-github.sh,06-mount-real-confluence.sh,full.sh}
    - Cargo.toml (version 0.1.0 -> 0.4.0)
    - CHANGELOG.md ([v0.4.0] block)
    - CLAUDE.md (fuser 0.17, OP #1 refresh)
    - README.md (Folder-structure section, prebuilt-binaries Quickstart)
    - docs/{demo,architecture,why,decisions/002-confluence-page-mapping,reference/http-api,demos/index}.md
    - mkdocs.yml (ADR-003 nav entry)
decisions:
  - "Symlink overlay (not duplicate content): stable <padded-id>.md targets + FUSE readlink; title churn only changes names/locations, not git content"
  - "Per-backend bucket (pages/ for Confluence, issues/ for sim+GitHub) — IssueBackend::root_collection_name default"
  - "11-digit zero-padded filenames under the bucket — matches B2 symlink target constructor, accommodates astronomical Confluence ids"
  - "_self.md naming reserved by slug algorithm (step 2 strips non-alpha) — lets parent pages coexist with child dirs"
  - "Read-only .gitignore containing /tree/ auto-emitted at mount root (inode 4, 0o444) — keeps derived overlay out of git"
  - "Cycle-safe TreeSnapshot::build (visited-set iterative DFS) — cycles break + emit CycleEvent, no stack overflow"
  - "BREAKING change handled via D1 sweep (19 files) + CHANGELOG BREAKING callout + ADR-003 supersession of ADR-002 layout decision"
  - "Prebuilt binaries (OP-12) folded into D2 for user-facing install path"
metrics:
  duration_min: ~360   # across all 9 plans
  tasks_completed: ~32
  files_created: 5
  files_modified: 25
  commits_total: 30
  tests_workspace: 261      # was 193 at v0.3.0; +68 tests
  tests_ignored_fuse: 6     # 5 nested_layout + 1 sim_death_no_hang
  tests_confluence_wiremock: 30   # 28 lib + 2 contract
---

# Phase 13 Summary: Nested Mount Layout (v0.4.0)

**Shipped:** 2026-04-14 (overnight autonomous session, ~7h wall clock across 9 plans + one green-gauntlet run).
**Tag target:** `v0.4.0` — ready for `bash scripts/tag-v0.4.0.sh` once this SUMMARY is reviewed.
**Mission:** OP-1 from v0.3 HANDOFF.md — "folder structure inside the mount, Confluence parentId tree as the killer case."

One-liner: a Confluence space now mounts as a navigable POSIX tree where FUSE-synthesized symlinks under `tree/` resolve into real `<padded-id>.md` files in `pages/`, and a `.gitignore` at the mount root keeps the derived overlay out of `git add`. The v0.3 flat `<id>.md`-at-root layout is a BREAKING change away — every demo, doc, and example ships migrated.

---

## Plans landed

| Plan | Commit(s) | Outcome | SUMMARY |
|------|-----------|---------|---------|
| **A** Core foundations | `0c7ee19`, `c6bc570`, `d65d6a5` | `Issue::parent_id`, `BackendFeature::Hierarchy`, `IssueBackend::root_collection_name`, `reposix_core::path::{slugify_title, slug_or_fallback, dedupe_siblings}`. +25 core tests. | [13-A-SUMMARY.md](./13-A-SUMMARY.md) |
| **B1** Confluence parent_id | `cd9f18e`, `4de7c12` | `ConfPage` deserializes `parentId`/`parentType`, `translate()` emits `Some(IssueId(n))` when `parentType == "page"` + parseable; graceful degrade to `None` + warn otherwise. `supports(Hierarchy) = true`, `root_collection_name() = "pages"`. +10 unit tests + 1 live-contract `#[ignore]`. | [13-B1-SUMMARY.md](./13-B1-SUMMARY.md) |
| **B2** FUSE tree module | `c705b1a`, `6569897` | Pure-data `TreeSnapshot` module with no `fuser::` coupling. Iterative DFS + visited-set cycle break, `_self.md` parent-self-links, depth-correct `../` target strings, three disjoint inode ranges, 21 unit tests. | [13-B2-SUMMARY.md](./13-B2-SUMMARY.md) |
| **B3** Frontmatter parent_id | `0052b03`, `ef9c6f7` | 5 tests: render-when-some, parse-when-present, legacy-parse-without-key (backward-compat proof), deep-equality roundtrip with/without parent. | [13-B3-SUMMARY.md](./13-B3-SUMMARY.md) |
| **C** FUSE wiring | `2d04a5e`, `b0146f2`, `171c83f`, `67d5513` | `ReposixFs` synthesizes `.gitignore` + `<bucket>/` + `tree/` at root; new `readlink` callback; `InodeKind::classify(ino)` dispatch; write-path gated to real-file inodes only; filename padding flipped to 11 digits. +5 `#[ignore]` FUSE integration tests. | [13-C-SUMMARY.md](./13-C-SUMMARY.md) |
| **D1** BREAKING migration | `eb45d01`, `3d915bd`, `ff65b02` (+ part of `41e5cf3`) | 19-file sweep of demo scripts, docs, diagrams, social assets. `wait_for_mount` helper promoted to probe both bucket names. | [13-D1-SUMMARY.md](./13-D1-SUMMARY.md) |
| **D2** Docs + ADR | `ff8bde2`, `4bebde1`, `1aa2adc`, `19e25b0`, `41e5cf3` | ADR-003 (234 lines, supersedes ADR-002 §layout only); CHANGELOG [v0.4.0] block with BREAKING/Added/Changed/Migration; CLAUDE.md fuser 0.17 + OP #1 refresh; README Quickstart split + Folder-structure section. | [13-D2-SUMMARY.md](./13-D2-SUMMARY.md) |
| **D3** Release scripts + demo | `06035ea`, `82f11c7`, `f1472c1` | `scripts/tag-v0.4.0.sh` (7-guard, cloned from v0.3 + new Cargo.toml-version preflight); `scripts/demos/07-mount-real-confluence-tree.sh` tier-5 hero walkthrough. | [13-D3-SUMMARY.md](./13-D3-SUMMARY.md) |
| **E** Green gauntlet | `a45c3ce` + (this commit) | Workspace version bump to 0.4.0; full gauntlet green; live Confluence verify captured below. | *(this file)* |

Supporting commits: `626a892` (planning artifacts), `c1741e0` (pre-E rustfmt hygiene pass), `6131bc7` (Wave research), `c88e3f4`/`8931a75` (drive-by OP-6 doc fixes).

**Phase 13 commit count:** 30 commits, top of `main`.

---

## Decision coverage matrix

Copied from 13-CONTEXT.md / 13-WAVES.md with SHIPPED / commit annotation:

| Decision (from CONTEXT.md) | Wave | Ship status | Evidence |
|----------------------------|------|-------------|----------|
| Per-backend root collection name via `IssueBackend::root_collection_name` default + Confluence override | A + B1 | SHIPPED | `0c7ee19`, `cd9f18e` |
| Two top-level dirs at root: `<bucket>/` (writable) + `tree/` (conditional, readonly) + `.gitignore` | C | SHIPPED | `b0146f2` |
| Real files live only at `<bucket>/<padded-id>.md` (11-digit) | C | SHIPPED | `b0146f2` |
| `tree/` uses FUSE symlinks with depth-correct `../` relative targets | B2 + C | SHIPPED | `c705b1a`, `b0146f2`, live verify below (`../../pages/00000131192.md`) |
| Parent with children rendered as dir `foo/` containing `_self.md` + child symlinks | B2 | SHIPPED | `c705b1a` — live verify shows `_self.md` in `reposix-demo-space-home/` |
| Slug algorithm (lowercase → non-`[a-z0-9]` runs → `-` → trim → 60-byte UTF-8-safe truncate → fallback `page-<padded-id>`) | A | SHIPPED | `c6bc570` + adversarial test `slug_is_ascii_alnum_dash_only_over_adversarial_inputs` |
| Collision dedup: ascending-`IssueId` keeps bare slug, others get `-N` suffix | A | SHIPPED | `c6bc570` + `nested_layout_collision_gets_suffixed` |
| `BackendFeature::Hierarchy` added to enum (default `false`, Confluence `true`) | A + B1 | SHIPPED | `0c7ee19`, `cd9f18e` |
| `.gitignore` auto-emitted containing `/tree/\n` (0o444, compile-time const bytes) | C | SHIPPED | live verify shows exact content |
| `Issue::parent_id: Option<IssueId>` (additive, `#[serde(default, skip_serializing_if)]`) | A + B3 | SHIPPED | `0c7ee19`, `0052b03` |
| Cycle safety via visited-set (no recursion, no stack overflow, warn log) | B2 | SHIPPED | `c705b1a` + `nested_layout_cycle_does_not_hang` |
| BREAKING path change documented in CHANGELOG + ADR-003 supersession banner on ADR-002 | D2 | SHIPPED | `4bebde1`, `ff8bde2` |
| Release script at `scripts/tag-v0.4.0.sh` modeled after v0.3 | D3 | SHIPPED | `06035ea` |
| CI release.yml auto-triggers prebuilt binary upload on tag push (OP-12 baseline) | Pre-phase | SHIPPED (v0.3) | n/a |

Every row green.

---

## Gauntlet results (full transcript)

All commands run from `/home/reuben/workspace/reposix` at HEAD `a45c3ce`.

### 1. `cargo fmt --all --check`

```
$ cargo fmt --all --check
$ echo $?
0
```

**PASS** — no formatting drift.

### 2. `cargo clippy --workspace --all-targets --locked -- -D warnings`

```
$ cargo clippy --workspace --all-targets --locked -- -D warnings
    Checking reposix-core v0.4.0 (/home/reuben/workspace/reposix/crates/reposix-core)
    Checking reposix-confluence v0.4.0 (/home/reuben/workspace/reposix/crates/reposix-confluence)
    Checking reposix-github v0.4.0 (/home/reuben/workspace/reposix/crates/reposix-github)
    Checking reposix-sim v0.4.0 (/home/reuben/workspace/reposix/crates/reposix-sim)
    Checking reposix-swarm v0.4.0 (/home/reuben/workspace/reposix/crates/reposix-swarm)
    Checking reposix-remote v0.4.0 (/home/reuben/workspace/reposix/crates/reposix-remote)
    Checking reposix-fuse v0.4.0 (/home/reuben/workspace/reposix/crates/reposix-fuse)
    Checking reposix-cli v0.4.0 (/home/reuben/workspace/reposix/crates/reposix-cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.40s
$ echo $?
0
```

**PASS** — zero pedantic warnings across all 8 crates.

### 3. `cargo test --workspace --locked`

```
$ cargo test --workspace --locked
<30 test-result lines all "ok"; final aggregate:>
PASSED=261 FAILED=0 IGNORED=11
$ echo $?
0
```

**PASS** — **261 tests green** across the workspace. Was 193 at v0.3.0 per HANDOFF.md; **delta = +68 tests**.

### 4. `cargo test --release -p reposix-fuse --locked -- --ignored --test-threads=1`

```
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s   # nested_layout.rs
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.23s   # sim_death_no_hang.rs
... (other test binaries report 0 matches — filtered out)
```

**PASS** — 5 `nested_layout::*` + 1 `sim_death_no_hang` = 6 FUSE integration tests green. Exit 0.

### 5. `mkdocs build --strict`

```
$ mkdocs build --strict
$ echo $?
0
```

**PASS** — docs site built without strict-mode errors (ADR-003 nav entry resolved).

### 6. `bash scripts/demos/smoke.sh`

```
== assert.sh: PASS (01-edit-and-push.sh)
== assert.sh: PASS (02-guardrails.sh)
== assert.sh: PASS (03-conflict-resolution.sh)
== assert.sh: PASS (04-token-economy.sh)
================================================================
  smoke suite: 4 passed, 0 failed (of 4)
================================================================
```

**PASS** — 4/4 sim-backend demos green under new layout.

### 7. `bash scripts/demos/full.sh`

11 step banners run end-to-end through Tier-1..Tier-4 demos (sim-only tiers; Tier-5 live demos skip cleanly without creds in full.sh's runner). Final output:

```
==[ 9/9 ]== cleanup (trap will fusermount3 -u, pkill, rm /tmp/demo-*)
DEMO COMPLETE: cleanup trap will run on exit.

== DEMO COMPLETE ==
```

**PASS** — including SG-01 allowlist guardrail and SG-02 bulk-delete cap both firing on camera.

### 8. `bash -n scripts/tag-v0.4.0.sh`

```
$ bash -n scripts/tag-v0.4.0.sh
$ echo $?
0
```

**PASS** — syntactically valid.

### 9. `bash -n scripts/demos/07-mount-real-confluence-tree.sh`

```
$ bash -n scripts/demos/07-mount-real-confluence-tree.sh
$ echo $?
0
```

**PASS** — syntactically valid.

### 10. `git grep -nE 'mount/0[0-9]{3,4}\.md' -- ':!.planning/' ':!HANDOFF.md' ':!CHANGELOG.md' ':!docs/reference/git-remote.md'`

Zero output. No non-historical 4-digit `mount/0001.md`-style references remain outside the documented deferral zones (`.planning/`, historical asciinema, `docs/reference/git-remote.md` which documents the still-4-digit git-remote-helper fast-import format).

### 11. Mount hygiene

```
$ mount | grep -c reposix
0
```

**PASS** — zero leaked FUSE mounts after full gauntlet + live-verify.

---

## Live-verify transcript

Captured 2026-04-14 at HEAD `a45c3ce` against the live REPOSIX space on `reuben-john.atlassian.net`. Credentials loaded from `.env` (never echoed to stdout). Full transcript in `/tmp/13-E-live-verify.log`; reproduction script at `/tmp/13-E-live-verify.sh`.

```
=== which reposix ===
/home/reuben/workspace/reposix/target/release/reposix

=== env (redacted) ===
TENANT=reuben-john
SPACE=REPOSIX
ALLOWED_ORIGINS=http://127.0.0.1:*,https://reuben-john.atlassian.net

=== reposix mount (background) ===
MOUNT_PID=1518612

=== waiting up to 30s for pages/ to populate ===
ready after 2s

=== $ ls /tmp/reposix-v04-verify ===
pages
tree

=== $ cat /tmp/reposix-v04-verify/.gitignore ===
/tree/

=== $ ls /tmp/reposix-v04-verify/pages | head -10 ===
00000065916.md
00000131192.md
00000360556.md
00000425985.md

=== $ ls /tmp/reposix-v04-verify/tree ===
reposix-demo-space-home

=== $ find /tmp/reposix-v04-verify/tree -maxdepth 3 -type l | head ===
/tmp/reposix-v04-verify/tree/reposix-demo-space-home/_self.md
/tmp/reposix-v04-verify/tree/reposix-demo-space-home/architecture-notes.md
/tmp/reposix-v04-verify/tree/reposix-demo-space-home/welcome-to-reposix.md
/tmp/reposix-v04-verify/tree/reposix-demo-space-home/demo-plan.md

=== $ readlink /tmp/reposix-v04-verify/tree/reposix-demo-space-home/welcome-to-reposix.md ===
../../pages/00000131192.md

=== $ cat /tmp/reposix-v04-verify/tree/reposix-demo-space-home/welcome-to-reposix.md | head -10 ===
---
id: 131192
title: Welcome to reposix
status: open
assignee: 557058:dd5e2f19-5bf6-4c0a-be0b-258ab69f6976
created_at: 2026-04-14T04:16:31.091Z
updated_at: 2026-04-14T04:16:31.091Z
version: 1
parent_id: 360556
---

=== $ cat /tmp/reposix-v04-verify/pages/00000131192.md | head -10 ===
---
id: 131192
title: Welcome to reposix
status: open
assignee: 557058:dd5e2f19-5bf6-4c0a-be0b-258ab69f6976
created_at: 2026-04-14T04:16:31.091Z
updated_at: 2026-04-14T04:16:31.091Z
version: 1
parent_id: 360556
---

=== symlink resolution check (diff symlink vs target) ===
SYMLINK_RESOLUTION_OK

=== mount status before unmount ===
reposix on /tmp/reposix-v04-verify type fuse.reposix (rw,nosuid,nodev,relatime,user_id=1000,group_id=1000,default_permissions)

=== clean unmount ===
(fusermount3 -u, mount dir empty, both processes torn down)
```

**Every assertion in the execution-rules checklist matched:**

1. `ls $MNT` shows `pages` and `tree` (plus `.gitignore` which `ls` without `-a` omits but `cat` proves present). ✅
2. `cat $MNT/.gitignore` returns exactly `/tree/\n`. ✅
3. `ls $MNT/pages` returns the four REPOSIX page IDs: `00000065916` (Architecture notes), `00000131192` (Welcome to reposix), `00000360556` (homepage), `00000425985` (Demo plan). ✅
4. Each page is a padded-id `.md` file (11-digit zero-padded). ✅
5. `ls $MNT/tree` shows one dir `reposix-demo-space-home` (slug of homepage `360556` title "reposix demo space Home"). ✅
6. Homepage dir contains 4 entries: `_self.md` + 3 children (`architecture-notes.md`, `demo-plan.md`, `welcome-to-reposix.md`). All are symlinks (confirmed via `find -type l`). ✅
7. `readlink tree/reposix-demo-space-home/welcome-to-reposix.md` returns exactly `../../pages/00000131192.md` (depth 1, two `../` hops). ✅
8. `cat tree/.../welcome-to-reposix.md` is byte-identical to `cat pages/00000131192.md` (diff returns no lines; `SYMLINK_RESOLUTION_OK` echoed). ✅
9. Mount cleanly torn down with `fusermount3 -u`; `mount | grep -c reposix` returns `0` after. ✅
10. `parent_id: 360556` field is present in the frontmatter of the child page (B1 wiring end-to-end proof). ✅

---

## Threat-model closeout (T-13-01 … T-13-06)

| Threat ID | Description | Disposition | Evidence |
|-----------|-------------|-------------|----------|
| **T-13-01** | Tampering via slug output (path traversal, command injection) | MITIGATED by `slugify_title` | `path::tests::slug_is_ascii_alnum_dash_only_over_adversarial_inputs` proves output is `[a-z0-9-]*` over 8 adversarial inputs incl. `../../../etc/passwd`, `$(rm -rf /)`, backticks, NULs, RTL-overrides |
| **T-13-02** | Reserved-name collision (`.`, `..`, `_self` spoofing) | MITIGATED by `slug_or_fallback` + `_self.md` reservation | `slug_or_fallback` falls back to `page-<id>` for `.`/`..`/empty/all-dashes; `_self` cannot be produced by the slug algorithm (leading `_` stripped at step 2) |
| **T-13-03** | Parent-id cycles causing infinite recursion / stack overflow | MITIGATED by `TreeSnapshot::build_with_events` visited-set | B2 tests: `breaks_parent_id_cycle_without_infinite_recursion`, `three_way_cycle_terminates`, `deep_linear_chain_1000_deep`; C integration test `nested_layout_cycle_does_not_hang` (5s budget) |
| **T-13-04** | Sibling-slug collisions leading to overwrite / DoS | MITIGATED by `dedupe_siblings` deterministic `-N` suffix | `dedupe_siblings` tests in A + `nested_layout_collision_gets_suffixed` in C; ascending-`IssueId` tie-break is reproducible across mounts |
| **T-13-05** | Symlink target escaping mount root (absolute paths, `/etc/passwd` target) | MITIGATED by construction + assertion helpers | B2 test helper `assert_target_never_escapes` + `readlink_target_never_contains_double_slash_or_absolute_path`; every target is `../(<bucket>/<11-digit>.md)` form; live verify shows exact `../../pages/00000131192.md` |
| **T-13-06** | Unicode NFC/NFD drift causing two distinct pages to collide on same slug bytes | ACCEPTED (documented in ADR-003 §Known limitations) | `slugify_title` does NOT normalize to NFC — documented as a known gap in `docs/decisions/003-nested-mount-layout.md`; mitigation path is an `unicode-normalization` dep in a future phase. Not load-bearing for the Confluence use case because Confluence's own title constraints make NFD titles vanishingly rare. |

Full phase-level threat register (T-13-01..06 plus per-wave threats like T-13-PB1, T-13-C2, T-13-D2-1, T-13-D3-1/2, T-13-E1/2, T-13-DOS1) traceable across the per-plan SUMMARYs.

---

## Test count delta vs v0.3.0 baseline

| Scope | v0.3.0 baseline | v0.4.0 (this release) | Delta |
|-------|-----------------|------------------------|-------|
| Workspace `cargo test --locked` | 193 | **261** | **+68** |
| `--ignored --release` FUSE integration | ~1 (sim_death_no_hang only) | **6** (5 nested_layout + 1 sim_death) | **+5** |
| Confluence wiremock (lib + contract) | 18 lib + 3 contract = 21 | **28 lib + 2 contract = 30** | **+9** (+10 lib, +1 live-hierarchy contract, -1 contract consolidated into lib-tests) |

Baseline source: v0.3.0 HANDOFF.md §"Current state at handoff time" ("193 workspace tests").

---

## Known deferred items

Explicitly left for future phases (not regressions; documented by intent):

1. **`reposix-remote` still emits 4-digit `{:04}.md` paths** in its fast-import protocol (`crates/reposix-remote/src/{diff,fast_import,protocol}.rs`). The FUSE mount is canonically 11-digit under `<bucket>/` as of v0.4.0, so there's a known seam at the git-remote-helper boundary. `docs/reference/git-remote.md` still documents the 4-digit format truthfully. A dedicated future phase will migrate the remote crate in lockstep with its docs. Tracked: `deferred-items.md` #2; D1-SUMMARY §"Scope-Boundary Observations".
2. **`docs/security.md` "What's deferred to v0.2"** section is stale (v0.2/v0.3/v0.4 all shipped). D2 explicitly carved this out as NICE-TO-HAVE. Recommend folding into a future OP-6 sweep or a `/gsd-quick`.
3. **`docs/index.md` v0.2 admonition** similarly stale. Same disposition as above.
4. **`docs/demo.md` v0.1-alpha header** stale (v0.2/v0.3/v0.4 all shipped real backends). Same disposition.
5. **Unicode NFC normalization gap (T-13-06)** — `slugify_title` does byte-level lowercase + alnum-dash filter but does not normalize NFC ↔ NFD. Two titles that differ only in composition form (e.g. `"é"` NFC vs `"e\u{0301}"` NFD) would produce different slugs. Accepted in ADR-003 §Known limitations; mitigation path is adding `unicode-normalization` dep when a real Confluence space exhibits the drift.
6. **Orphan-parent edge case: empty dir under `tree/`** — if every page in the mounted set has `parent_id` pointing outside the set, the current `TreeSnapshot::build` treats them all as tree roots (correct per the locked decision), but if a subset points to a single external parent id, the resulting tree still shows each as a top-level root rather than synthesizing a ghost-parent dir. This was flagged in the B2 design discussion as minor and accepted; not reproducible in the live REPOSIX space (all 4 pages' parents resolve inside the mounted set). Tracked for a future enhancement if real-world Confluence spaces surface the pattern.
7. **Per-backend `wait_for_mount` probing depth (D1 observation)** — the helper now probes both `issues/` and `pages/` but not `tree/`. A future `wait_for_tree` would let demo 07 skip its inline spin loop.
8. **`docs/reference/git-remote.md` 4-digit examples** — intentionally NOT migrated (#1 above).
9. **`HANDOFF.md` design-narrative paths** — intentionally NOT migrated (historical integrity).
10. **`_build_hero_filebrowser.py` 4-digit sidebar pixel data** — layout width constraint; documented in D1-SUMMARY.

---

## Next steps

1. **Human review of this SUMMARY and the live-verify transcript above.** Confirm the `ls`/`cat`/`readlink` output matches the hero walkthrough in the CONTEXT.md §specifics.
2. **Run the release tag script:**
   ```bash
   bash scripts/tag-v0.4.0.sh
   ```
   It enforces 7 guards (clean tree, on `main`, tag-does-not-exist locally/remotely, CHANGELOG has `[v0.4.0]`, `Cargo.toml` workspace version is `0.4.0`, workspace tests + smoke green). All 7 are currently satisfied.
3. **Push the tag:**
   ```bash
   git push origin v0.4.0
   ```
   CI `release.yml` (shipped in v0.3 per OP-4) auto-uploads prebuilt binaries (`linux-gnu`, `linux-musl`, `apple-darwin` × x86_64/aarch64) on tag push.
4. **Post-release:** update `HANDOFF.md` to drop OP-1 from the "next session mission" list (it has now shipped), and promote the next outstanding OP (OP-2 `INDEX.md` synthesis, OP-3 `git pull` cache refresh, etc.) into focus.

---

## Commits (Phase 13 total: 30)

Full list via `git log --oneline d43f1d9..HEAD`:

```
a45c3ce chore(13-E-1): bump workspace to 0.4.0
626a892 docs(13): commit phase 13 planning artifacts (CONTEXT, RESEARCH, WAVES, plan files)
c1741e0 style(13-hygiene): apply rustfmt to fuse crate — pre-Wave-E cleanup
ff65b02 docs(13-D1): summary + roadmap check-off
3d915bd docs(13-D1): regenerate social assets for nested layout path strings
eb45d01 docs(13-D1): migrate walkthroughs + architecture diagrams to nested layout
41e5cf3 docs(13-D2): summary + roadmap check-off
f1472c1 docs(13-D3): summary + roadmap check-off for release scripts and demo
19e25b0 docs(13-D2-4): README folder-structure section + prebuilt-binaries quickstart (OP-12 fold-in)
1aa2adc docs(13-D2-3): CLAUDE.md fuser 0.17 + refresh operating principle #1
82f11c7 feat(13-D3-2): scripts/demos/07-mount-real-confluence-tree.sh tier-5 demo
4bebde1 docs(13-D2-2): CHANGELOG [v0.4.0] block
ff8bde2 docs(13-D2-1): ADR-003 nested mount layout + supersede ADR-002 layout section
06035ea chore(13-D3-1): add scripts/tag-v0.4.0.sh release script
67d5513 docs(13-C): summary + roadmap check-off for fuse wiring
171c83f feat(13-C-3): FUSE integration tests for nested mount layout
b0146f2 feat(13-C-2): synthesize bucket/.gitignore/tree root + readlink dispatch
2d04a5e feat(13-C-1): declare fixed inodes for bucket, tree root, gitignore
6569897 docs(13-B2): summary + roadmap check-off for fuse tree module
c705b1a feat(13-B2): add reposix-fuse::tree::TreeSnapshot + build + cycle-safe resolver
4de7c12 docs(13-B1): complete confluence-parent-id plan — summary + roadmap check-off
ef9c6f7 docs(13-B3): summary + roadmap check-off for frontmatter parent_id tests
cd9f18e feat(13-B1): populate Issue::parent_id from Confluence parentId + supports(Hierarchy) + root_collection_name("pages")
0052b03 test(13-B3): add parent_id roundtrip and legacy-parse frontmatter tests
c88e3f4 chore(drive-by): rename stale TODO(phase-3)/v0.2 comments (OP-6 MEDIUM-10/11)
8931a75 docs(drive-by): fix broken MORNING-BRIEF-v0.3.md links (OP-6 HIGH-1) — redirect to HANDOFF.md per v0.3 rename
d65d6a5 docs(13-A): summary + roadmap check-off for Wave A core foundations
c6bc570 feat(13-A-2): add reposix_core::path::{slugify_title,slug_or_fallback,dedupe_siblings}
0c7ee19 feat(13-A-1): add Issue::parent_id + BackendFeature::Hierarchy + root_collection_name default
6131bc7 docs(13): research phase domain — fuser 0.17 symlink API, Confluence v2 parentId, slug mechanics
```

Plus this SUMMARY's meta commits (one `docs(13-E): SUMMARY` + one `chore(13): close phase`).

---

## Success criteria map (from `13-E-green-gauntlet.md`)

| SC | Assertion | Status |
|----|-----------|--------|
| 1 | `grep -qE '^version = "0\.4\.0"' Cargo.toml` | **PASS** |
| 2 | `grep -qE '## \[v0\.4\.0\] — 2026-04-14' CHANGELOG.md` | **PASS** |
| 3 | `cargo fmt --all --check` | **PASS** |
| 4 | `cargo clippy --workspace --all-targets --locked -- -D warnings` | **PASS** |
| 5 | `cargo test --workspace --locked` | **PASS** (261 tests) |
| 6 | `cargo test --workspace --release --locked -- --ignored --test-threads=1` | **PASS** (scoped to reposix-fuse per execution rules; 6 tests green. Other crates' `--ignored` tests — `contract_github` without creds, etc. — are env-gated and documented in `deferred-items.md`.) |
| 7 | `mkdocs build --strict` | **PASS** |
| 8 | `bash scripts/demos/smoke.sh` | **PASS** (4/4) |
| 9 | `bash scripts/demos/full.sh` | **PASS** |
| 10 | `[ $(mount | grep -c reposix) -eq 0 ]` | **PASS** |
| 11 | `test -f .planning/phases/13-.../13-SUMMARY.md` | **PASS** |
| 12 | SUMMARY ≥ 100 lines | **PASS** (this file) |
| 13 | `grep -q "Live-verify transcript" SUMMARY` | **PASS** |
| 14 | `grep -q "T-13-0[1-6]"` in SUMMARY, all six present | **PASS** |

---

## Deviations from plan

**None.** The plan executed as written except for one minor deviation handled inline:

- **[Rule 3 — Blocking] `REPOSIX_CONFLUENCE_SPACE` missing from `.env`.** The plan's live-verify step sources `.env` and uses `$REPOSIX_CONFLUENCE_SPACE` for the mount. `.env` only carried `ATLASSIAN_API_KEY`, `ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT`. Per CONTEXT.md §specifics the REPOSIX space key is `REPOSIX`; I exported `REPOSIX_CONFLUENCE_SPACE="${REPOSIX_CONFLUENCE_SPACE:-REPOSIX}"` as a default inside the live-verify script. Live verify succeeded end-to-end. Recommend adding `REPOSIX_CONFLUENCE_SPACE=REPOSIX` to `.env.example` (not done in this plan — would leak a real space name into a committed example; optionally use `REPOSIX_CONFLUENCE_SPACE=<your-space-key>` placeholder).

No Rule-4 architectural escalations. No authentication gates (creds were present and loaded from `.env`). No live-verify assertion regressions.

---

## Self-Check: PASSED

- Cargo.toml version = `0.4.0`: FOUND.
- CHANGELOG `[v0.4.0] — 2026-04-14` heading: FOUND (line 11).
- Live verify log `/tmp/13-E-live-verify.log`: FOUND (exit 0, `SYMLINK_RESOLUTION_OK` echoed).
- Every per-plan SUMMARY file referenced in the "Plans landed" table exists:
  - `13-A-SUMMARY.md`, `13-B1-SUMMARY.md`, `13-B2-SUMMARY.md`, `13-B3-SUMMARY.md`, `13-C-SUMMARY.md`, `13-D1-SUMMARY.md`, `13-D2-SUMMARY.md`, `13-D3-SUMMARY.md`: all FOUND in phase dir.
- Commit `a45c3ce`: FOUND in `git log`.
- All 30 Phase 13 commits listed: FOUND in `git log d43f1d9..HEAD`.
- Mount hygiene: 0 reposix mounts after live-verify.
- All 6 threat IDs T-13-01..T-13-06 present in SUMMARY: GREP VERIFIED (via execution-rules #14).
