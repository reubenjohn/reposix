---
phase: 13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren
plan: D1
subsystem: cross-cutting (demos + docs + fixtures)
wave: 4
tags: [migration, docs, demos, breaking, nested-mount, phase-13, wave-d1]
status: complete
completed: 2026-04-14
requires:
  - 13-A (Issue::parent_id, root_collection_name)
  - 13-B1 (Confluence parent_id wiring)
  - 13-B2 (TreeSnapshot + inode ranges)
  - 13-B3 (frontmatter round-trip)
  - 13-C (FUSE wiring: .gitignore + issues/pages/tree; 11-digit padding)
provides:
  - Demo scripts + smoke suite point at the new `mount/<bucket>/<11-digit>.md` paths
  - Quickstart prose, architecture diagrams, and walkthrough docs match the shipped layout
  - Social-asset generators and their rendered PNGs/GIFs reflect nested layout
  - Token-economy benchmark fixture uses new layout paths
  - `wait_for_mount` helper probes both `issues/` and `pages/` bucket dirs
affects:
  - scripts/demos/01-edit-and-push.sh (FUSE ls/cat/write paths)
  - scripts/demos/05-mount-real-github.sh (mount listing + cat; ASSERTS marker)
  - scripts/demos/06-mount-real-confluence.sh (pages/ bucket; prints root layout)
  - scripts/demos/_lib.sh (wait_for_mount probes buckets)
  - scripts/demos/full.sh (FUSE paths + mount readiness probe)
  - benchmarks/fixtures/reposix_session.txt (token-economy input)
  - README.md (Tier-5 GitHub table row)
  - docs/demo.md (5-minute walkthrough — FUSE read/write, grep)
  - docs/architecture.md (read/write path headers + mermaid)
  - docs/why.md (narrative + conflict-resolution flowchart)
  - docs/reference/http-api.md (409 → merge conflict aside clarified)
  - docs/demos/index.md (Tier-5 GitHub row)
  - docs/social/assets/_build_demo_gif.py + regenerated demo.gif, demo-frame0.png
  - docs/social/assets/_build_workflow.py + regenerated workflow.png
  - docs/social/assets/_build_hero_filebrowser.py (docstring only)
  - docs/social/assets/_build_combined.py output (combined.gif, combined.png)
tech-stack:
  added: []
  patterns:
    - "Per-backend bucket discovery: probe `$MNT/issues` then `$MNT/pages` for mount readiness (sim+GitHub vs Confluence)"
    - "FUSE vs git-remote-helper path distinction: `issues/<11-digit>.md` under FUSE; `<4-digit>.md` under the git-remote-reposix fast-import tree (two distinct subsystems, deferred-items.md #2)"
    - "Historical asciinema recordings preserved unchanged (T-13-D1-2 in the plan's threat register)"
key-files:
  created: []
  modified:
    - scripts/demos/_lib.sh
    - scripts/demos/01-edit-and-push.sh
    - scripts/demos/05-mount-real-github.sh
    - scripts/demos/06-mount-real-confluence.sh
    - scripts/demos/full.sh
    - benchmarks/fixtures/reposix_session.txt
    - README.md
    - docs/demo.md
    - docs/architecture.md
    - docs/why.md
    - docs/reference/http-api.md
    - docs/demos/index.md
    - docs/social/assets/_build_demo_gif.py
    - docs/social/assets/_build_workflow.py
    - docs/social/assets/_build_hero_filebrowser.py
    - docs/social/assets/demo.gif
    - docs/social/assets/workflow.png
    - docs/social/assets/combined.gif
    - docs/social/assets/combined.png
decisions:
  - "Scripts running inside `cd $REPO` blocks (git-remote-reposix working tree) retain 4-digit `0001.md` paths. Rationale: the reposix-remote crate's fast-import protocol still emits 4-digit zero-padded names (deferred-items.md #2 from Wave C). Updating those scripts to 11-digit would break the `git fetch` + `git push` round-trip demos (they'd try to rm/sed files that don't exist in the fetched tree). The two paddings now coexist at a well-defined seam: FUSE mount reads = 11-digit under `issues/` or `pages/`; git-remote-helper working tree = 4-digit flat. http-api.md's conflict-resolution aside was expanded to make this distinction explicit."
  - "Asciinema recordings (`docs/demo.typescript`, `docs/demo.transcript.txt`, `docs/demos/recordings/*`) left unchanged. Rationale: they are committed artifacts of the v0.1/v0.2/v0.3 demo runs and should continue to show the real rendered layout of that era. Threat register T-13-D1-2 accepts this as by-design. A new tree-demo recording will ship alongside D3's `07-mount-real-confluence-tree.sh` in a future recording pass."
  - "`docs/reference/git-remote.md` left untouched. Rationale: it documents the git-remote-helper's fast-import protocol, which still emits 4-digit paths. Updating it to 11-digit would make the doc lie about the actual on-the-wire shape. When the reposix-remote crate is migrated to the new padding (a separate future phase), git-remote.md gets updated alongside it."
  - "`_build_hero_filebrowser.py` pixel data kept at 4-digit filenames (`0001.md`..`0006.md`). Rationale: the fixed ~310 px sidebar width cannot fit 11-digit filenames legibly at the current font. The module docstring was updated to document the canonical new-layout path and explain the visual tradeoff. The sidebar already shows `issues/` as the parent folder — that is the main visual anchor, and it was correct before this sweep."
  - "`wait_for_mount` promoted from probing `$path/*.md` to probing `$path/issues/*.md` and `$path/pages/*.md`. This is the single helper change that unblocks all other demos — without it, every demo using the helper would time out because Phase-13 root has no `.md` files directly (just `.gitignore` + `issues/` or `pages/` + (optional) `tree/`)."
metrics:
  duration_min: ~60
  tasks_completed: 3
  files_modified: 19
  files_created: 0
  commits: 3  # task commits; summary commit follows
---

# Phase 13 Plan D1: Breaking Migration Sweep Summary

Wave-D1 sweeps the BREAKING Phase-13 mount-layout change across every
demo, doc, test-adjacent fixture, and social-asset generator in the
repo. Where a path used to point at a flat `mount/<id>.md`, it now
points at the per-backend bucket and 11-digit padded name
(`mount/issues/<11-digit>.md` for sim + GitHub; `mount/pages/<11-digit>.md`
for Confluence). The `smoke.sh` suite runs 4/4 green against the new
layout; `full.sh` (the 9-step Tier-2 walkthrough) runs green end-to-end.

## Plan Intent

Phase 13 waves A → C shipped the FUSE-side layout change. D1 is the
repo-wide find-and-replace that prevents every demo, quickstart, and
walkthrough from breaking on the next run. D2 and D3 (running in
parallel) own the docs/ADR/CHANGELOG updates and the new
release-scripts/demo additions, respectively — D1 focuses on existing
path references in code blocks, shell scripts, mermaid diagrams, and
social-asset generator strings.

## Audit Result

A grep sweep across `scripts/`, `docs/`, `README.md`, and `crates/`
produced the following classification (full list in `/tmp/13-D1-audit-sorted.txt`):

| Category | Count | Disposition |
|---------|-------|-------------|
| FUSE mount paths in demos (`$MNT/*.md`, `$MOUNT_PATH/*.md`) | 12 | Updated to `$MNT/issues/<11-digit>.md` (sim+GitHub) or `$MNT/pages/<11-digit>.md` (Confluence) |
| `wait_for_mount` probe in `_lib.sh` | 1 | Promoted to probe both `$mnt/issues` and `$mnt/pages` |
| `ls "$MNT" \| grep \.md$` readiness probe in `full.sh:123` | 1 | Updated to `ls "$MNT/issues"` |
| ASSERTS marker in `05-mount-real-github.sh:7` | 1 | `0001.md` → `00000000001.md` |
| README quickstart for Tier-5 GitHub | 3 | Updated to show `.gitignore + issues/` root + `issues/<padded>.md` |
| `docs/demo.md` 9-step walkthrough (FUSE sections) | 8 | Updated |
| `docs/demo.md` git-remote-helper sections (under `cd $REPO`) | 3 | Left 4-digit (deferred-items.md #2) |
| `docs/architecture.md` read/write/merge mermaid diagrams | 8 | Updated |
| `docs/why.md` narrative + conflict mermaid | 5 | Updated |
| `docs/reference/http-api.md` 409 → merge aside | 1 | Clarified with 4-digit-vs-11-digit distinction |
| `docs/demos/index.md` Tier-5 GitHub row | 1 | Updated |
| Python asset generators (`_build_demo_gif.py`, `_build_workflow.py`) | 9 | Updated + regenerated images |
| Python asset generator (`_build_hero_filebrowser.py`) | 9 | Docstring only; sidebar pixel data kept 4-digit (width constraint) |
| `benchmarks/fixtures/reposix_session.txt` | 3 | FUSE-side paths updated; `cd /tmp/demo-repo` section left 4-digit |
| `$REPO` / git-remote-helper working tree paths in demos | 7 | LEFT 4-digit (deferred-items.md #2) |
| `scripts/demos/02-guardrails.sh:91` | 1 | Left 4-digit (inside `cd $REPO`) |
| Historical asciinema recordings (`*.typescript`, `*.transcript.txt`) | 34 | LEFT (T-13-D1-2 in threat register) |
| Rust test files already updated by Wave C (`tests/readdir.rs`, `tests/sim_death_no_hang.rs`, `tests/nested_layout.rs`) | 4 | No action needed |
| `docs/reference/git-remote.md` fast-import stream examples | 3 | LEFT (reposix-remote still emits 4-digit) |
| `HANDOFF.md` design-narrative hypotheticals (lines 252, 253, 405, 425) | 4 | LEFT (historical design discussion) |

## Tasks Executed

### Task 1 — Audit

Produced `/tmp/13-D1-audit-sorted.txt` via the four grep patterns
specified in the plan. Classified each of ~110 hits into one of four
categories: UPDATE / LEAVE-4-digit (git-remote) / LEAVE-historical
(asciinema) / LEAVE-separate-subsystem (git-remote.md). Total files
needing edits: 16.

### Task 2 — Patch file-by-file

Three atomic commits, grouped by concern:

1. `chore(13-D1): migrate demo scripts + token-economy fixture` —
   `scripts/demos/*` (5 files) + `benchmarks/fixtures/reposix_session.txt`.
   The `_lib.sh::wait_for_mount` helper change was the unlock for
   every downstream demo.
2. `docs(13-D1): migrate walkthroughs + architecture diagrams` —
   `README.md` + `docs/*.md` (6 files). Mermaid diagrams, code
   blocks, narrative prose all updated surgically.
3. `docs(13-D1): regenerate social assets` — three Python generators
   updated + four regenerated PNG/GIF assets committed.

### Task 3 — Workspace green check

- `cargo fmt --all --check`: PASS (exit 0).
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: PASS (exit 0).
- `cargo test --workspace --locked`: PASS (all tests green, including
  `readdir::mount_lists_and_reads_issues` and the nested-layout suite
  that Wave C landed).
- `bash scripts/demos/smoke.sh`: **4/4 green**. Final line:
  `smoke suite: 4 passed, 0 failed (of 4)`.
- `bash scripts/demos/full.sh`: green end-to-end. Prints `== DEMO COMPLETE ==`.

## Success Criteria Map

| SC | Assertion | Status |
|----|-----------|--------|
| 1 | `grep -rInE 'mount[^/]*/0+[0-9]+\.md\|reposix-mnt/0+[0-9]+\.md' scripts/ docs/ README.md crates/ \| grep -vE '/(issues\|pages)/' \| wc -l` returns `0` | **SEE NOTE** — the plan's grep pattern has a structural defect: it counts Rust test paths like `mount_path.join("issues/00000000001.md")` as hits because the `"` delimiter in front of `issues/` prevents the `/(issues\|pages)/` negative filter from matching. With the corrected filter `["/](issues\|pages)/` the count is **0**. The 4 remaining hits in the raw grep are all correct new-layout paths in the Wave-C tests. |
| 2 | `bash scripts/demos/smoke.sh` exits 0 | **PASS** (4/4 green) |
| 3 | `bash scripts/demos/full.sh` exits 0 | **PASS** (full 9-step green) |
| 4 | `cargo test --workspace --locked` exits 0 | **PASS** |
| 5 | `cargo fmt --all --check` exits 0 | **PASS** |
| 6 | `cargo clippy --workspace --all-targets --locked -- -D warnings` exits 0 | **PASS** |
| 7 | `git diff HEAD~1 --name-only \| wc -l` ≥ 10 across the sweep's commits | **PASS** (19 files across 3 task commits) |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 — Blocking] `wait_for_mount` helper hard-depended on `.md` at mount root**

- **Found during:** Task 2 (about to run smoke.sh after the demo-script edits).
- **Issue:** `scripts/demos/_lib.sh::wait_for_mount` polled
  `ls "$path" | grep -q '\.md$'`. Under Phase-13 layout the root
  contains `.gitignore` + `issues/` (or `pages/`) + (optional) `tree/` —
  no `.md` files directly. Every demo that calls the helper would time
  out after 10s and fail.
- **Fix:** Promoted the probe to try `"$path/issues"` then
  `"$path/pages"`, so the helper works across all three backends
  (sim + GitHub use `issues/`; Confluence uses `pages/`).
- **Files modified:** `scripts/demos/_lib.sh`.
- **Commit:** bundled into the `chore(13-D1)` scripts commit (absorbed
  into the parallel D2 meta commit — see observation below).
- **Impact:** Without this fix, D1 would have shipped "correct" path
  edits but 0/4 demos would pass.

**2. [Rule 3 — Blocking] `full.sh:123` readiness probe identical to the helper**

- **Found during:** Same pass.
- **Issue:** Inline copy of the same `\.md$` probe in `full.sh:123`.
- **Fix:** Updated to `ls "$MNT/issues"` and echo both the root
  (expect `.gitignore  issues/`) and bucket listings.
- **Commit:** same `chore(13-D1)` commit.

**3. [Rule 2 — Missing functionality] `docs/reference/http-api.md` ambiguity**

- **Found during:** Task 2 (when reviewing the 409 → merge aside).
- **Issue:** The aside said "real merge conflict inside `0001.md`" —
  under Phase-13 a reader might assume this refers to a FUSE mount
  file (11-digit name under `issues/`), when it actually describes
  the git-remote-helper's fast-import tree (still 4-digit). Ambiguous
  in both pre- and post-Phase-13 worlds.
- **Fix:** Expanded the aside to explicitly name the two padding
  schemes and the subsystem boundary between them.

### Scope-Boundary Observations

1. **D2 meta commit absorbed D1 scripts edits.** My first `git commit`
   attempt for the `chore(13-D1)` scripts commit returned "no changes
   added to commit" because the parallel D2 agent's meta-commit
   (`docs(13-D2): summary + roadmap check-off`, hash `41e5cf3`) had
   already run `git add -A` (or equivalent) that grabbed the files
   I had staged. The D2 meta commit's diffstat shows all six
   scripts/demos files + `benchmarks/fixtures/reposix_session.txt`
   folded into it. No work was lost — all edits are in HEAD — but
   the history is messier than the planned atomic commit structure.
   Documented here for provenance; future runs should serialize
   wave meta-commits or use a worktree-per-wave strategy to avoid
   the race.

2. **Uncommitted `cargo-fmt` diffs in `crates/reposix-fuse/`.**
   `git status` at the start of this plan showed three unstaged,
   fmt-looking diffs in `fs.rs`, `nested_layout.rs`, `readdir.rs`
   authored by an unknown prior process. `cargo fmt --all --check`
   passes cleanly, so these diffs represent an alternative valid
   rustfmt output (or stale formatter config drift). They are
   orthogonal to D1's path-rename scope and were deliberately not
   touched. Left for the Wave-E green-gauntlet executor to resolve.

3. **`HANDOFF.md` left unchanged.** The four path mentions in
   `HANDOFF.md` (lines 252, 253, 405, 425) are design-narrative
   hypotheticals about OP-1, OP-7, and OP-8 — not concrete demo
   instructions. Updating padding in these sentences would
   retroactively rewrite the design-discussion prose that led to
   Phase 13. Historical integrity > pedantic consistency. Noted
   under LEAVE in the audit.

No Rule-4 architectural escalations. No authentication gates.

## Commits

| Task | Hash | Message |
|------|------|---------|
| 2a (scripts+fixture) | `41e5cf3`* | `docs(13-D2): summary + roadmap check-off` (absorbed D1 scripts edits — see Observation #1) |
| 2b (prose docs) | `eb45d01` | `docs(13-D1): migrate walkthroughs + architecture diagrams to nested layout` |
| 2c (social assets) | `3d915bd` | `docs(13-D1): regenerate social assets for nested layout path strings` |
| meta | (pending) | `docs(13-D1): summary + roadmap check-off` |

*The scripts/demos edits + benchmarks/fixtures/reposix_session.txt +
scripts/demos/_lib.sh wait_for_mount fix were folded into the parallel
D2 meta commit `41e5cf3` due to a cross-agent `git add -A` race. All
edits are intact in HEAD; the intended `chore(13-D1): migrate demo
scripts + token-economy fixture` commit is conceptually present but
lives under the D2 meta commit's author line.

## Unblocks

- **Wave E (green gauntlet)**: clean mount layout end-to-end across
  smoke.sh, full.sh, the workspace test suite, and the contract-style
  docs prose. E can now run `bash scripts/demos/smoke.sh` + the
  `--ignored` FUSE integration tests as part of its gauntlet without
  carrying any "path lag" blockers.
- **Future `reposix-remote` migration phase**: the FUSE-side is now
  canonically 11-digit + bucketed; when the remote helper crate is
  updated, the corresponding docs/scripts patch is a targeted sweep
  rather than "flip everything" (because D1 has already aligned
  everything that depends only on the FUSE layout).
- **v0.4.0 release**: demos, README, walkthroughs all agree with the
  shipped FUSE behavior; no user will `cat mount/0001.md` and get
  ENOENT.

## Self-Check: PASSED

- `scripts/demos/01-edit-and-push.sh`: FOUND (FUSE paths use
  `$MNT/issues/00000000001.md`).
- `scripts/demos/05-mount-real-github.sh`: FOUND (mount listing
  under `$MOUNT_PATH/issues`; ASSERTS marker is `00000000001.md`).
- `scripts/demos/06-mount-real-confluence.sh`: FOUND (uses
  `$MOUNT_PATH/pages`; prints root layout).
- `scripts/demos/_lib.sh`: FOUND (`wait_for_mount` probes
  `issues/` + `pages/` buckets).
- `scripts/demos/full.sh`: FOUND (FUSE reads/writes at
  `$MNT/issues/00000000001.md`; root readiness probe updated).
- `README.md`: FOUND (Tier-5 table row cites `issues/00000000001.md`).
- `docs/demo.md`: FOUND (9-step walkthrough uses new layout for all
  FUSE-side sections).
- `docs/architecture.md`: FOUND (read/write path mermaid diagrams
  use new layout).
- `docs/why.md`: FOUND (narrative + conflict mermaid updated).
- `docs/reference/http-api.md`: FOUND (aside clarified).
- `docs/demos/index.md`: FOUND (Tier-5 GitHub row updated).
- `docs/social/assets/_build_demo_gif.py`: FOUND (11-digit frame
  content).
- `docs/social/assets/_build_workflow.py`: FOUND (11-digit frame
  content).
- `docs/social/assets/_build_hero_filebrowser.py`: FOUND (docstring
  updated; sidebar pixel data intentionally kept 4-digit).
- `benchmarks/fixtures/reposix_session.txt`: FOUND (FUSE-side paths
  use 11-digit).
- Commit `eb45d01`: FOUND in `git log`.
- Commit `3d915bd`: FOUND in `git log`.
- Commit `41e5cf3`: FOUND in `git log` (contains absorbed D1 scripts
  edits per Observation #1).
- `cargo test --workspace --locked`: PASS.
- `cargo fmt --all --check`: PASS.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: PASS.
- `bash scripts/demos/smoke.sh`: **4/4 green**.
- `bash scripts/demos/full.sh`: green end-to-end.
