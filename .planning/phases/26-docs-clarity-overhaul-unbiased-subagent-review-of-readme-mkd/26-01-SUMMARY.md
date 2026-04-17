---
phase: 26
plan: "26-01"
subsystem: docs
tags: [housekeeping, readme, archive, version-fix]
dependency_graph:
  requires: [phase-25]
  provides: [clean-root, accurate-readme-version]
  affects: [README.md, mkdocs.yml]
tech_stack:
  added: []
  patterns: [docs/archive/ for obsolete project-era files]
key_files:
  created:
    - docs/archive/MORNING-BRIEF.md
    - docs/archive/PROJECT-STATUS.md
  modified:
    - README.md
    - mkdocs.yml
  deleted:
    - AgenticEngineeringReference.md
    - InitialReport.md
    - MORNING-BRIEF.md (moved to docs/archive/)
    - PROJECT-STATUS.md (moved to docs/archive/)
decisions:
  - "Used not_in_nav mkdocs pattern (not exclude_docs) to suppress orphan page warning for docs/archive/"
  - "Removed v0.3 Confluence quickstart version qualifier rather than updating to v0.7 — removes churn, quickstart still accurate"
metrics:
  duration: ~10 minutes
  completed: "2026-04-17"
---

# Phase 26 Plan 01: Root-level housekeeping — delete stubs, archive obsolete files, fix README version

One-liner: Deleted root redirect stubs, archived v0.1/v0.2 era docs to `docs/archive/`, and updated README.md version references from v0.3.0 to v0.7.0 throughout.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Delete root stubs, archive obsolete docs | dc24ce8 | AgenticEngineeringReference.md (deleted), InitialReport.md (deleted), MORNING-BRIEF.md→docs/archive/, PROJECT-STATUS.md→docs/archive/, mkdocs.yml |
| 2 | Fix README.md version references and phase table | 3e59993 | README.md |

## Verification Results

```
stub 1 gone          (AgenticEngineeringReference.md deleted)
stub 2 gone          (InitialReport.md deleted)
archive 1 exists     (docs/archive/MORNING-BRIEF.md)
archive 2 exists     (docs/archive/PROJECT-STATUS.md)
v0.7 matches: 8      (grep "v0\.7" README.md | wc -l)
v0.3.0 alpha: 0      (grep -c "v0\.3\.0 alpha" README.md)
workspace version = "0.7.0"   (Cargo.toml)
cargo check --workspace: Finished (clean)
```

## Changes Made

### Task 1
- `git rm AgenticEngineeringReference.md` — redirect stub only (single blockquote line); canonical copy in `docs/research/agentic-engineering-reference.md`
- `git rm InitialReport.md` — redirect stub only; canonical copy in `docs/research/initial-report.md`
- `git mv MORNING-BRIEF.md docs/archive/MORNING-BRIEF.md` — archived v0.1/v0.2 morning brief with archive notice prepended
- `git mv PROJECT-STATUS.md docs/archive/PROJECT-STATUS.md` — archived v0.1/v0.2 project status with archive notice prepended
- `mkdocs.yml`: added `not_in_nav: archive/*` block to suppress orphan page warnings; archive pages do not appear in nav

### Task 2
- Status block: `v0.3.0 alpha` → `v0.7.0`, test count `193` → `317+`, date range updated
- Phase table replaced with release table covering v0.1 through v0.7
- Tracking artifacts sentence: updated to v0.8+ direction (JIRA Cloud integration)
- Confluence quickstart: removed stale `(read-only, v0.3)` qualifier
- Binary download URLs: `v0.4.0-x86_64` → `v0.7.0-x86_64` (all 3 occurrences)
- Security section heading: `what's enforced today (v0.3)` → `(v0.7)`
- Security prose: updated to reflect v0.3/v0.6/v0.7 mitigations accurately
- Deferred section: heading `v0.4 / future` → `v0.8 / future`; content updated (GitHub write still deferred, Confluence write shipped in v0.6, JIRA added, macOS updated for v0.7 partial)

## Deviations from Plan

None — plan executed exactly as written. The Confluence quickstart version qualifier was removed (option mentioned in the plan as acceptable) rather than updated to v0.7, since the command itself is version-agnostic.

## Known Stubs

None — this plan is docs-only housekeeping with no data stubs.

## Threat Flags

None — docs-only changes, no new network endpoints, auth paths, or schema changes introduced.

## Self-Check: PASSED

- [x] `docs/archive/MORNING-BRIEF.md` exists
- [x] `docs/archive/PROJECT-STATUS.md` exists
- [x] `AgenticEngineeringReference.md` does not exist
- [x] `InitialReport.md` does not exist
- [x] Commit dc24ce8 exists
- [x] Commit 3e59993 exists
