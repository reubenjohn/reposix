---
phase: 13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren
plan: D3
subsystem: release-scripts-and-demos
tags: [release, demo, confluence, tier-5, v0.4.0]
requires:
  - tag-v0.3.0.sh: existed at scripts/tag-v0.3.0.sh as the template to clone
  - 06-mount-real-confluence.sh: existed at scripts/demos/06-mount-real-confluence.sh as the skip-gate + cleanup template
  - _lib.sh: existed at scripts/demos/_lib.sh with section/require/cleanup_trap/wait helpers
provides:
  - scripts/tag-v0.4.0.sh: annotated-tag-and-push script for the v0.4.0 release with 7 safety guards
  - scripts/demos/07-mount-real-confluence-tree.sh: tier-5 live Confluence tree overlay demo (hero.png flow)
affects:
  - none (strictly additive; no existing files touched)
tech-stack:
  added: []
  patterns:
    - "skip-if-no-creds gate (identical to 06): four-env-var check before any network op"
    - "tree/ overlay hero-flow: ls root -> cat .gitignore -> ls pages/ -> cd tree/<slug>/ -> readlink -> cat"
    - "v0.4.0 tag script adds a 7th guard (Cargo.toml workspace version == 0.4.0) beyond the six v0.3 guards"
key-files:
  created:
    - scripts/tag-v0.4.0.sh
    - scripts/demos/07-mount-real-confluence-tree.sh
  modified: []
decisions:
  - "Do not add demo 07 to smoke.sh: requires Atlassian creds; smoke must remain sim-only-4/4"
  - "Do not touch _lib.sh: parallel execution with D1 means any shared-file edit risks a merge conflict"
  - "Add a 7th tag-script guard (Cargo.toml version == 0.4.0) as a preflight check — do NOT bump the version from within the script (caller does that in a pre-tag commit)"
  - "Reference a reposix.md content marker (_self.md) in the demo's hero-dir assertion — catches regressions if the tree/ layout regresses to not emitting parent self-links"
metrics:
  duration_min: 3
  tasks: 3
  commits: 2
  files_created: 2
  files_modified: 0
completed: 2026-04-14
---

# Phase 13 Plan D3: Release scripts and demo — Summary

One-liner: Shipped `scripts/tag-v0.4.0.sh` (7-guard tag+push script cloned from v0.3 with a new Cargo.toml-version preflight) and `scripts/demos/07-mount-real-confluence-tree.sh` (Tier-5 live-Confluence demo exercising the new three-entry root layout `.gitignore` + `pages/` + `tree/` and the symlink-walk hero flow).

## Tasks Completed

| # | Task | Commit | Files |
|---|------|--------|-------|
| 1 | `scripts/tag-v0.4.0.sh` release script | `06035ea` | scripts/tag-v0.4.0.sh (new, +103 lines) |
| 2 | `scripts/demos/07-mount-real-confluence-tree.sh` tier-5 demo | `82f11c7` | scripts/demos/07-mount-real-confluence-tree.sh (new, +206 lines) |
| 3 | Workspace-green spot-check (verification only) | — | (no file changes) |

## Artifact 1: `scripts/tag-v0.4.0.sh` — diff vs `scripts/tag-v0.3.0.sh`

The v0.4 script is a structural clone of v0.3 with:

1. Every `v0.3.0` / `0.3.0` / `v0.3` string bumped to `v0.4.0` / `0.4.0` / `v0.4`.
2. Header comment line 5 changed from "You've at least skimmed MORNING-BRIEF-v0.3.md" to "Cargo.toml workspace version has been bumped to 0.4.0" — v0.4 doesn't have a morning brief and does have a version-bump prereq.
3. A NEW guard 6 (renumbering the old `green` guard 6 → 7): preflight check for `version = "0.4.0"` in `Cargo.toml`. The script does NOT edit Cargo.toml — it only asserts. The actual bump is a pre-tag commit by the user or Wave E.
4. Tag annotation message line changed from `"reposix $TAG — read-only Confluence Cloud adapter"` to `"reposix $TAG — nested mount layout + Confluence tree/ overlay"` to reflect the v0.4 feature scope.

Full unified diff (trimmed of trivial `N/6 → N/7` renumbering):

```diff
--- scripts/tag-v0.3.0.sh
+++ scripts/tag-v0.4.0.sh
@@
-# scripts/tag-v0.3.0.sh — create and push the v0.3.0 annotated tag.
+# scripts/tag-v0.4.0.sh — create and push the v0.4.0 annotated tag.
@@
-#   4. You've eyeballed the CHANGELOG [v0.3.0] section
-#   5. You've at least skimmed MORNING-BRIEF-v0.3.md
+#   4. You've eyeballed the CHANGELOG [v0.4.0] section
+#   5. Cargo.toml workspace version has been bumped to 0.4.0
@@
-# Safety guards (this script enforces all six):
+# Safety guards (this script enforces all seven):
@@
-#   3. v0.3.0 tag must NOT already exist locally
-#   4. v0.3.0 tag must NOT already exist on origin
-#   5. CHANGELOG.md must contain a `## [v0.3.0]` header
-#   6. cargo test --workspace --locked green + scripts/demos/smoke.sh green
+#   3. v0.4.0 tag must NOT already exist locally
+#   4. v0.4.0 tag must NOT already exist on origin
+#   5. CHANGELOG.md must contain a `## [v0.4.0]` header
+#   6. Cargo.toml workspace version must be 0.4.0
+#   7. cargo test --workspace --locked green + scripts/demos/smoke.sh green
@@
-TAG="v0.3.0"
+TAG="v0.4.0"
@@
-if ! grep -qE '^## \[v0\.3\.0\]' CHANGELOG.md; then
-    echo "ERROR: CHANGELOG.md has no '## [v0.3.0]' section" >&2
+if ! grep -qE '^## \[v0\.4\.0\]' CHANGELOG.md; then
+    echo "ERROR: CHANGELOG.md has no '## [v0.4.0]' section" >&2
@@
+# 6. Cargo.toml workspace version bumped
+#    The actual bump is a pre-tag commit the user makes (or a separate
+#    small commit in Wave E). This script only checks; it does NOT edit.
+if ! grep -qE '^version = "0\.4\.0"' Cargo.toml; then
+    echo "ERROR: Cargo.toml workspace version is not 0.4.0. Bump it before tagging." >&2
+    exit 1
+fi
+echo "[guard 6/7] Cargo.toml workspace version = 0.4.0"
@@
-CHANGELOG_BODY="$(sed -n '/^## \[v0.3.0\]/,/^## \[/p' CHANGELOG.md | sed '$d')"
+CHANGELOG_BODY="$(sed -n '/^## \[v0.4.0\]/,/^## \[/p' CHANGELOG.md | sed '$d')"
@@
-git tag -a "$TAG" -m "reposix $TAG — read-only Confluence Cloud adapter
+git tag -a "$TAG" -m "reposix $TAG — nested mount layout + Confluence tree/ overlay
```

Verification:
- `test -x scripts/tag-v0.4.0.sh` → ok
- `bash -n scripts/tag-v0.4.0.sh` → ok
- `grep -c 'v0\.4\.0' scripts/tag-v0.4.0.sh` → 12 occurrences
- `! grep -q 'v0\.3\.0' scripts/tag-v0.4.0.sh` → ok (zero v0.3.0 leftovers)
- shellcheck: not installed in dev host; manual spot-check: all `set -euo pipefail`, all vars properly quoted, `"$TAG"` / `"$CURRENT_BRANCH"` / `"$CHANGELOG_BODY"` all quoted, no SC2086-style splits.
- The script is NOT executed end-to-end from this plan (it would push a real tag).

## Artifact 2: `scripts/demos/07-mount-real-confluence-tree.sh`

Six-step narrative, modeled after 06-mount-real-confluence's four-step narrative but expanded for the three-way root layout:

| Step | Banner | Verb |
|------|--------|------|
| 1/6 | mount real Confluence at `$MOUNT_PATH` | spawn `reposix mount --backend confluence` in background, wait up to 30s for `pages/*.md` to appear |
| 2/6 | ls mount root — expect `.gitignore`, `pages/`, `tree/` | `ls -la $MOUNT_PATH`, assert all three entries present via `ls -A ... grep -qx` |
| 3/6 | cat `.gitignore` — expect `/tree/` | `cat $MOUNT_PATH/.gitignore`, assert the exact `/tree/` literal via `grep -q '^/tree/$'` |
| 4/6 | flat view — ls `pages/` and cat the first page | `ls $MOUNT_PATH/pages`, `head -5` on the first entry |
| 5/6 | hierarchical view — cd tree/ and readlink the hero file | `find $MOUNT_PATH/tree -mindepth 1 -maxdepth 1 -type d` → cd into the homepage slug dir, assert `_self.md` is a symlink, `readlink _self.md`, `head -5 _self.md`, then `readlink` + `head -5` on one sibling leaf |
| 6/6 | unmount | `fusermount3 -u`, wait up to 3s, assert `mountpoint -q` returns non-zero |

Divergences from 06-mount-real-confluence.sh (minimal, all justified):

| # | Divergence | Why |
|---|-----------|-----|
| 1 | `exec timeout 120` instead of `exec timeout 90` | Extra 30s covers the tree-walk (find + readlink + head) that 06 does not do |
| 2 | `wait_for_mount` not used; inline spin on `ls $MOUNT_PATH/pages` | `wait_for_mount` from `_lib.sh` checks the top level for `*.md`, but under Phase 13 the top level contains `.gitignore`/`pages`/`tree` with no `.md` — the check has to descend into `pages/` |
| 3 | 6 sections vs 4 | Added sections 2 (root layout) and 3 (.gitignore) before the flat-view step; step 5 expanded into a cd + readlink walk |
| 4 | Asserts for `.gitignore` content, `_self.md` existence, presence of all three root entries | Catches regressions to the nested-mount layout guarantees that 06 doesn't have to care about |

Skip-path output (verified on dev host with all four env vars unset and `PATH="$PWD/target/release:$PATH"`):

```
SKIP: env vars unset: ATLASSIAN_API_KEY ATLASSIAN_EMAIL REPOSIX_CONFLUENCE_TENANT REPOSIX_CONFLUENCE_SPACE
      Set them (see .env.example) to run this demo against the
      real REPOSIX space on reuben-john.atlassian.net.
== DEMO COMPLETE ==
```

Exit code: 0. Matches the contract.

Note: like demo 06, the skip path first runs `require reposix` / `require fusermount3`. If `reposix` binary is not on PATH the script exits 2 before reaching the skip check. This matches 06's contract and the smoke-runner convention (release binaries must be on PATH before any Tier-5 demo runs).

## Verification (success criteria)

| SC | Assertion | Result |
|----|-----------|--------|
| 1 | `test -x scripts/tag-v0.4.0.sh` | PASS |
| 2 | `bash -n scripts/tag-v0.4.0.sh` | PASS |
| 3 | `grep -qE 'v0\.4\.0' scripts/tag-v0.4.0.sh` | PASS (12 occurrences) |
| 4 | `! grep -qE 'v0\.3\.0' scripts/tag-v0.4.0.sh` | PASS |
| 5 | `test -x scripts/demos/07-mount-real-confluence-tree.sh` | PASS |
| 6 | `bash -n scripts/demos/07-mount-real-confluence-tree.sh` | PASS |
| 7 | `grep -q 'ATLASSIAN_API_KEY' scripts/demos/07-mount-real-confluence-tree.sh` | PASS |
| 8 | `grep -q 'readlink' scripts/demos/07-mount-real-confluence-tree.sh` | PASS |
| 9 | `(unset ATLASSIAN_API_KEY; bash scripts/demos/07-mount-real-confluence-tree.sh)` exits 0 | PASS (with reposix on PATH) |
| 10 | `bash scripts/demos/smoke.sh` exits 0 | NOT YET — blocked on Wave D1 |

## Known Issues / Handoffs

### SC10: smoke.sh is red, blocked on Wave D1

`bash scripts/demos/smoke.sh` fails at demo 01-edit-and-push.sh because the sim-backend demos still reference the old flat-layout path `$MNT/0001.md` at the mount root. The Phase-13 layout change (Waves A+B+C, landed in commits `0c7ee19`..`171c83f`) moved real files to `$MNT/issues/<id>.md` for the sim backend. Fixing every demo script + test + doc reference is precisely Wave D1's scope (`13-D1-breaking-migration-sweep.md`).

D3 executed in parallel with D1 and D2 per the plan's wave graph. My two new files are strictly additive — they cannot have regressed smoke. Once D1 lands its sweep and demo 01-04 read the new `issues/<id>.md` path, smoke goes back to 4/4 green and guard 7 of `tag-v0.4.0.sh` fires correctly on the pre-release run.

### Guard 6 (Cargo.toml version)

Cargo.toml workspace version is currently `0.1.0`. Running `tag-v0.4.0.sh` today would trip guard 6. This is intentional: the bump is a human (or Wave E) pre-tag commit. `tag-v0.4.0.sh` is a checker, not an editor.

## Deviations from Plan

**None — plan executed as written.** The plan explicitly allowed a preflight Cargo.toml version check as a new guard and I added it exactly per the plan's action block.

One note on smoke-list membership: the plan's must_have row says demo 07 "is NOT added to smoke.sh". I did not add it; smoke remains the 4-demo sim-only suite. Demo 07 is discoverable at `scripts/demos/07-mount-real-confluence-tree.sh` as a standalone Tier-5 demo, same pattern as demo 06.

## Threat Flags

None. Both artifacts follow the threat-model disposition from the plan:
- `T-13-D3-1` (token leakage via echo): mitigated by pattern inheritance from demo 06 — only non-secret identifiers (tenant subdomain, space key) appear on stdout; `ATLASSIAN_API_KEY` / `ATLASSIAN_EMAIL` are read but never echoed.
- `T-13-D3-2` (tag pushed to wrong remote): accepted per plan; user's `git remote -v` is their check.

No new threat surface introduced vs. the plan's threat register.

## Self-Check: PASSED

Files present:
- FOUND: scripts/tag-v0.4.0.sh (103 lines, +x)
- FOUND: scripts/demos/07-mount-real-confluence-tree.sh (206 lines, +x)

Commits present on main:
- FOUND: 06035ea chore(13-D3-1): add scripts/tag-v0.4.0.sh release script
- FOUND: 82f11c7 feat(13-D3-2): scripts/demos/07-mount-real-confluence-tree.sh tier-5 demo
