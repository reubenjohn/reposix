---
phase: 13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren
plan: D2
subsystem: docs
wave: 4
tags: [docs, adr, changelog, phase-13, wave-d2, op-1, op-12]
status: complete
completed: 2026-04-14
requires:
  - 13-C (FUSE wiring shipped; ADR-003 can cite concrete inode layout + readlink proof)
provides:
  - ADR-003 design doc for pages/ + tree/ symlink overlay (supersedes ADR-002 layout section)
  - ADR-002 'Superseded in part' banner pointing at ADR-003
  - CHANGELOG [v0.4.0] block with BREAKING + Added + Changed + Migration sections
  - CLAUDE.md fuser version correction (0.15 → 0.17) + refreshed Operating Principle #1
  - README Quickstart leading with prebuilt-binaries path (OP-12 fold-in)
  - README top-level 'Folder-structure mount (v0.4+)' section linking to ADR-003
affects:
  - docs/decisions/003-nested-mount-layout.md (NEW, 234 lines)
  - docs/decisions/002-confluence-page-mapping.md (superseded banner)
  - CHANGELOG.md ([v0.4.0] block prepended, footer compare links updated)
  - CLAUDE.md (fuser version + OP #1 wording)
  - README.md (Quickstart split + Folder-structure section + Phase 13 row + path-scheme example fix)
  - mkdocs.yml (nav entry for ADR-003)
tech-stack:
  added: []
  patterns: []
key-files:
  created:
    - docs/decisions/003-nested-mount-layout.md
    - .planning/phases/13-nested-mount-layout-pages-tree-symlinks-for-confluence-paren/13-D2-SUMMARY.md
  modified:
    - docs/decisions/002-confluence-page-mapping.md
    - CHANGELOG.md
    - CLAUDE.md
    - README.md
    - mkdocs.yml
    - .planning/ROADMAP.md
decisions:
  - "ADR-003 explicitly supersedes only ADR-002's §Options layout decision (row A). The field-mapping, status-mapping, auth, pagination, and rate-limit sections of ADR-002 remain authoritative — called out in both the new ADR's frontmatter and ADR-002's top-of-file banner. Rationale: ADR-002's non-layout content is still correct and is cited by the Confluence adapter code; wholesale supersession would orphan the field-mapping reference."
  - "Folded in the OP-12 prebuilt-binaries work mid-plan. The README Quickstart now leads with 'Install prebuilt binaries (recommended)' (curl + tar against releases/latest) and demotes 'Build from source' to a contributors-only secondary subsection. The CHANGELOG [v0.4.0] Added section also documents prebuilt binaries as a user-facing addition, correcting the OP-12-flagged gap (they shipped silently in v0.3 but were never CHANGELOG-mentioned)."
  - "CLAUDE.md Operating Principle #1 was reworded rather than just patched for the version string. The original 'until v0.2 explicitly opens this gate' language has been stale for three releases — v0.2 shipped real GitHub, v0.3 shipped real Confluence, v0.4 ships tree/. New wording captures the enduring invariant: sim is the default, real backends are guarded by allowlist + explicit creds, autonomous mode never hits real backends unless both are set. Addresses T-13-D2-1 from the threat register."
  - "Did NOT touch docs/index.md, docs/security.md, or docs/demo.md this plan. Plan called those out as NICE-TO-HAVE and the MUST-HAVE scope (ADR-003 + CHANGELOG + CLAUDE.md + README) consumed the budget cleanly. Those three docs still carry v0.2-era framing (e.g., 'v0.1 alpha' in demo.md, 'v0.2 deferred' in security.md); recommend a future OP-6 sweep or a subsequent /gsd-quick plan to fold them into v0.4 framing alongside the D1 path-scheme sweep."
metrics:
  duration_min: ~15
  tasks_completed: 4  # ADR-003 + ADR-002 banner (1), CHANGELOG (2), CLAUDE.md (3), README (4)
  files_modified: 5
  files_created: 1
  commits: 4  # task commits; summary commit follows
---

# Phase 13 Plan D2: Docs + ADR Summary

Wave-D2 lands the documentation surface of Phase 13. ADR-003 documents the
nested mount layout design (pages/ + tree/ symlinks) and supersedes
ADR-002's flat-layout decision. CHANGELOG gains a `[v0.4.0]` block with
BREAKING + Added + Changed + Migration sections. CLAUDE.md gets a fuser
version correction and an Operating Principle refresh. README leads
Quickstart with the prebuilt-binaries path (OP-12 fold-in) and gains a
top-level 'Folder-structure mount (v0.4+)' section. `mkdocs build
--strict` stays green.

## Plan Intent

Pure documentation — no code, no tests, no scripts. Land the four
must-have artifacts called out in the plan spec, fold in the OP-12
prebuilt-binaries install-docs work, and leave docs/index.md /
docs/security.md / docs/demo.md for a future OP-6 sweep per the plan's
NICE-TO-HAVE carve-out.

## ADR-003 Summary

**File:** `docs/decisions/003-nested-mount-layout.md` (234 lines).

**Sections:**

1. Context (ADR-002's option analysis + why we reopened the layout
   question)
2. Decision (the three mount-root entries: bucket, tree/, .gitignore)
3. Trade-off accepted (symlinks vs hard-links vs bind-mounts)
4. Alternatives rejected (duplicate-content, ID-only tree paths,
   git-tracked tree/)
5. Slug algorithm (5-step algorithm; fallback)
6. Collision resolution (ascending-IssueId deterministic dedup)
7. Cycle handling (DFS visited-set; break at deepest repeated ancestor)
8. `_self.md` convention (reserved by construction via slug step 2)
9. `.gitignore` emission (compile-time const bytes; 0o444; inode 4)
10. Known limitations (NFC gap T-13-06, tree staleness, Confluence-only,
    500-page cap, no tree-rename support)
11. Consequences (breaking path change, 11-digit padding, additive core
    field, write-through-symlink semantics, git-merge-hell dissolved)
12. References

**All 9 required sections present:** Decision, Slug algorithm, Collision
resolution, Cycle handling, `_self.md`, `.gitignore`, Known limitations
(including NFC / T-13-06), Consequences, References. Context + Alternatives
rejected added on top of the required minimum.

**Frontmatter** explicitly names the supersession target:
`supersedes: docs/decisions/002-confluence-page-mapping.md (§"Option A:
flat" layout decision; everything else in ADR-002 remains valid)`.

## ADR-002 Superseded Banner

Prepended to ADR-002 right after the title, before the status block:

```markdown
> **Superseded in part (2026-04-14).** The flat-layout decision in this
> ADR's §"Options" table (row A, chosen in v0.3) was replaced by
> [ADR-003](003-nested-mount-layout.md) (`pages/` bucket + `tree/` symlink
> overlay). The field-mapping, status-mapping, auth, pagination, and
> rate-limit decision sections below remain authoritative.
```

Status line updated: `Accepted` → `Accepted (layout section superseded by
ADR-003)`. `Superseded by` line now points at ADR-003.

## CHANGELOG Block Inserted (first 10 lines)

```markdown
## [v0.4.0] — 2026-04-14

The "nested mount layout" cut. OP-1 from the v0.3 HANDOFF.md (the
"folder structure inside the mount" ask) ships, and Confluence's native
`parentId` hierarchy becomes a navigable directory tree backed by FUSE
symlinks. The v0.3 flat `<padded-id>.md` layout is retained under a
per-backend bucket (`pages/` or `issues/`), and a synthesized
read-only `tree/` overlay appears alongside when the backend exposes
hierarchy.

### BREAKING
```

BREAKING callouts: layout reshuffle (pages/|issues/) + 11-digit id
padding (was 4-digit). Added section enumerates 9 items including
ADR-003, prebuilt binaries (OP-12 fold-in), tree/ overlay, slugify
helper, and `_self.md` convention. Changed section calls out the
CLAUDE.md + README updates. Migration block shows before/after shell
examples. Footer compare-links updated: `[v0.4.0]` link added,
`[Unreleased]` target rebased to v0.4.0.

## CLAUDE.md Diff for fuser Version Line

Before (line 40):

```
- FUSE: `fuser` 0.15 with `default-features = false`. **Reason:** …
```

After (line 40):

```
- FUSE: `fuser` 0.17 with `default-features = false`. **Reason:** …
```

Reason-comment byte-for-byte unchanged. Verified against `Cargo.lock`:
`fuser = "0.17"` is the workspace-resolved version, confirmed in
13-RESEARCH.md §Core stack.

## CLAUDE.md Operating Principle #1 — Before / After

**Before:**

> Simulator before real backend. Until v0.2 explicitly opens this gate,
> no code in this repo authenticates to a real GitHub/Jira/Confluence
> instance. The simulator at `crates/reposix-sim/` is the only backend.
> This is both a security constraint (no creds in autonomous mode) and
> the StrongDM dark-factory pattern.

**After:**

> Simulator is the default / testing backend. The simulator at
> `crates/reposix-sim/` is the default backend for every demo, unit
> test, and autonomous agent loop. Real backends (GitHub via
> `reposix-github`, Confluence via `reposix-confluence`) are guarded by
> the `REPOSIX_ALLOWED_ORIGINS` egress allowlist and require explicit
> credential env vars (`GITHUB_TOKEN`, `ATLASSIAN_API_KEY` +
> `ATLASSIAN_EMAIL` + `REPOSIX_CONFLUENCE_TENANT`). Autonomous mode
> never hits a real backend unless the user has put real creds in `.env`
> AND set a non-default allowlist. This is both a security constraint
> (fail-closed by default) and the StrongDM dark-factory pattern.

Captures the enduring invariant (default-to-sim + allowlist + explicit
creds) rather than the now-stale v0.2 gate wording.

## README Section Title + First 3 Lines

```markdown
## Folder-structure mount (v0.4+)

Mounting a Confluence space exposes the page hierarchy as a navigable directory tree:
```

Section placed immediately after `## Quickstart`, before `## Architecture`.
Links to `docs/decisions/003-nested-mount-layout.md`. Includes a worked
shell example against the REPOSIX demo space showing `.gitignore`,
`pages/`, `tree/`, `_self.md`, symlink resolution, and the
`readlink ../../pages/...` output.

Also inside Quickstart, split into two subsections:

- **Install prebuilt binaries (recommended)** — `curl -fsSLO` against
  `releases/latest/download/reposix-v0.4.0-x86_64-unknown-linux-gnu.tar.gz`
  + `tar -xzf` + `export PATH`. Mentions SHA256SUMS verification. Matches
  the tarball-naming convention in `.github/workflows/release.yml`
  (`reposix-${TAG}-${target}.tar.gz`).
- **Build from source (contributors)** — the old `git clone && bash
  scripts/demo.sh` path, demoted to contributors-only.

Phase 13 row added to the status table. HANDOFF forward-reference updated
from `v0.4 direction` to `v0.5+ direction` now that OP-1 has landed.

## mkdocs Strict Build — Success Line

```
INFO    -  Documentation built in 0.86 seconds
```

`mkdocs build --strict` exits 0. The two `social/linkedin.md` +
`social/twitter.md` not-in-nav INFO lines are pre-existing (they're social
share stubs, not real docs) — mkdocs' strict mode correctly treats INFOs
as non-fatal.

ADR-003 rendered cleanly; nav entry `ADR-003 Nested mount layout:
decisions/003-nested-mount-layout.md` added to `mkdocs.yml`.

## Success Criteria Map

| SC  | Assertion                                                                                       | Status |
|-----|-------------------------------------------------------------------------------------------------|--------|
| 1   | `test -f docs/decisions/003-nested-mount-layout.md`                                             | PASS   |
| 2   | `wc -l docs/decisions/003-nested-mount-layout.md` ≥ 120                                         | PASS (234) |
| 3   | `grep -q '## Decision' docs/decisions/003-nested-mount-layout.md`                               | PASS   |
| 4   | `grep -q 'Slug algorithm' docs/decisions/003-nested-mount-layout.md`                            | PASS   |
| 5   | `grep -q 'Known limitations' docs/decisions/003-nested-mount-layout.md`                         | PASS   |
| 6   | `grep -q 'NFC' docs/decisions/003-nested-mount-layout.md` (T-13-06 documented)                  | PASS   |
| 7   | `grep -q 'Superseded in part' docs/decisions/002-confluence-page-mapping.md`                    | PASS   |
| 8   | `grep -q '## \[v0.4.0\]' CHANGELOG.md`                                                          | PASS   |
| 9   | `grep -q '### BREAKING' CHANGELOG.md`                                                           | PASS   |
| 10  | `grep -qE 'fuser.*0\.17' CLAUDE.md && ! grep -qE 'fuser.*0\.15' CLAUDE.md`                      | PASS   |
| 11  | `grep -q 'Folder-structure mount' README.md`                                                    | PASS   |
| 12  | `grep -q '003-nested-mount-layout' README.md`                                                   | PASS   |
| 13  | `mkdocs build --strict` exits 0                                                                 | PASS   |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 — Missing critical functionality] OP-12 install-docs fold-in included in scope**

- **Found during:** Plan reading — the prompt explicitly instructs folding OP-12 prebuilt-binaries install docs into this plan.
- **Issue:** The plan spec as written only covers ADR-003 + CHANGELOG + CLAUDE.md + a single README Folder-structure section. The prompt wraps an additional OP-12 ask: update README Quickstart to lead with prebuilt binaries (OP-12 from HANDOFF.md).
- **Fix:** Added the `### Install prebuilt binaries (recommended)` subsection to README Quickstart (curl + tar against `releases/latest/download/reposix-v0.4.0-x86_64-unknown-linux-gnu.tar.gz`, matching the `.github/workflows/release.yml` tarball naming). Also updated the CHANGELOG [v0.4.0] Added section to call out prebuilt binaries as a user-facing addition (correcting the OP-12 CHANGELOG gap).
- **Files modified:** `README.md`, `CHANGELOG.md`.
- **Commits:** `19e25b0` (README), `4bebde1` (CHANGELOG).

**2. [Rule 2 — Missing critical functionality] README Tier-5 path-scheme example updated**

- **Found during:** README editing — the Tier-5 GitHub mount example under `## Demo` still used the flat `cat /tmp/reposix-gh-mnt/0001.md` path.
- **Issue:** Plan spec told D1 to do repo-wide path-scheme sweeps but explicitly called out that D2 should update README examples itself because D2 owns the structural README rewrite.
- **Fix:** Updated `cat /tmp/reposix-gh-mnt/0001.md` → `cat /tmp/reposix-gh-mnt/issues/00000000001.md` in the Tier-5 demo example. Left other path references in Tier-1..Tier-4 demo descriptions alone — those reference shell scripts under `scripts/demos/` which D1 owns.
- **Files modified:** `README.md`.
- **Commit:** `19e25b0`.

**3. [Rule 2 — Missing critical functionality] Phase 13 row added to README status table**

- **Found during:** README editing.
- **Issue:** Status table ended at Phase 11 (v0.3); v0.4 ships Phase 13 but the table didn't reflect it.
- **Fix:** Added `| Phase 13 — Nested mount layout (v0.4) | shipped: pages/ + tree/ symlink overlay for Confluence hierarchy, Issue::parent_id, _self.md convention, ADR-003 |` row. Updated HANDOFF forward-reference from "v0.4 direction" to "v0.5+ direction" (OP-1 is now shipped; remaining OPs are OP-2 INDEX.md, OP-3 git-pull cache refresh, etc.).
- **Files modified:** `README.md`.
- **Commit:** `19e25b0`.

**4. [Rule 3 — Blocking] mkdocs.yml nav entry added**

- **Found during:** First `mkdocs build --strict` run.
- **Issue:** `mkdocs.yml` has an explicit `nav:` block listing each ADR individually. Dropping ADR-003 at `docs/decisions/003-nested-mount-layout.md` without a nav entry would have emitted an INFO about an undeclared page (harmless in strict mode but sloppy).
- **Fix:** Added `- ADR-003 Nested mount layout: decisions/003-nested-mount-layout.md` to the Decisions subsection of the nav.
- **Files modified:** `mkdocs.yml`.
- **Commit:** `ff8bde2` (Task 1).

No Rule-4 architectural escalations. No authentication gates encountered.

### Scope-Boundary Observations (deferred)

Not addressed this plan; noted for a future `/gsd-quick` or OP-6 sweep:

1. **`docs/index.md`** — "v0.2 autonomously built" admonition is now stale; v0.3 + v0.4 have both shipped. Worth a one-admonition update.
2. **`docs/security.md`** — "What's deferred to v0.2" section is entirely stale (v0.2, v0.3, v0.4 all shipped). Whole section should be rewritten to reflect the actual v0.4 deferred items.
3. **`docs/demo.md`** — "v0.1 alpha, built autonomously overnight on 2026-04-13" header is stale; same for the "No real backend. Simulator-only." limitation bullet (v0.2/v0.3/v0.4 all ship real backends).

None of these are in the D2 must-have set; plan explicitly flagged them as NICE-TO-HAVE with a "leave for future OP-6 sweep" escape hatch.

## Commits

| Task | Hash      | Message                                                                                 |
|------|-----------|-----------------------------------------------------------------------------------------|
| 1    | `ff8bde2` | `docs(13-D2-1): ADR-003 nested mount layout + supersede ADR-002 layout section`         |
| 2    | `4bebde1` | `docs(13-D2-2): CHANGELOG [v0.4.0] block`                                               |
| 3    | `1aa2adc` | `docs(13-D2-3): CLAUDE.md fuser 0.17 + refresh operating principle #1`                  |
| 4    | `19e25b0` | `docs(13-D2-4): README folder-structure section + prebuilt-binaries quickstart (OP-12 fold-in)` |
| meta | (pending) | `docs(13-D2): summary + roadmap check-off`                                              |

## Unblocks

- **Wave D1 (BREAKING migration sweep):** runs in parallel with D2; D2's README path-scheme fixes to the Tier-5 example are the pattern D1 applies elsewhere.
- **Wave D3 (release scripts + demo):** ADR-003 + CHANGELOG [v0.4.0] are now canonical references for the tag-v0.4.0 release body.
- **Wave E (green gauntlet):** mkdocs-strict is green, so the final gauntlet sweep doesn't need to rebuild it.
- **HANDOFF OP-1 + OP-12:** both now have user-facing documentation. The next session's HANDOFF augmentation can drop the OP-12 entry.

## Self-Check: PASSED

- `docs/decisions/003-nested-mount-layout.md`: FOUND (234 lines, all 9 required sections present).
- `docs/decisions/002-confluence-page-mapping.md`: FOUND + superseded banner at top.
- `CHANGELOG.md`: FOUND + `[v0.4.0]` block with BREAKING + Added + Changed + Migration sections.
- `CLAUDE.md`: FOUND + fuser version `0.17` (not 0.15) + refreshed OP #1.
- `README.md`: FOUND + `## Folder-structure mount (v0.4+)` section + ADR-003 link + prebuilt-binaries Quickstart + Phase 13 status row.
- `mkdocs.yml`: FOUND + ADR-003 nav entry.
- `.planning/ROADMAP.md`: FOUND + `- [x] 13-D2-docs-and-adr`.
- Commits `ff8bde2`, `4bebde1`, `1aa2adc`, `19e25b0`: FOUND in `git log`.
- `mkdocs build --strict`: PASS (exits 0, ADR-003 rendered).
- All 13 success-criteria assertions: PASS.
