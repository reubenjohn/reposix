---
phase: 26
plan: "26-04"
subsystem: docs
tags: [docs, clarity, reference, connectors, decisions, adr]
dependency_graph:
  requires: [26-01]
  provides: [reference-docs-clear, connector-guide-clear, adr-002-scope-clarified]
  affects: [docs/reference, docs/connectors, docs/decisions]
tech_stack:
  added: []
  patterns: [doc-clarity-review, adr-scope-note]
key_files:
  created: []
  modified:
    - docs/reference/cli.md
    - docs/reference/confluence.md
    - docs/reference/crates.md
    - docs/connectors/guide.md
    - docs/decisions/002-confluence-page-mapping.md
decisions:
  - "ADR-002 scope note uses 'Active — with scope note' wording; existing superseded blockquote replaced (not additive) to avoid duplicate status signals"
  - "crates.md updated from 'five crates' to 'eight crates'; added reposix-github, reposix-confluence, reposix-swarm sections"
  - "guide.md stale 'ADR-003' forward reference (for subprocess ABI) fixed to 'a future ADR' with note that ADR-003 is the nested mount layout"
metrics:
  duration: "~20 minutes"
  completed: "2026-04-17T02:42:41Z"
  tasks_completed: 2
  files_modified: 5
---

# Phase 26 Plan 04: Clarity review — reference/, connectors/, decisions/ docs Summary

Reviewed and fixed 9 docs across `docs/reference/`, `docs/connectors/`, and `docs/decisions/`. All critical gaps addressed; ADR-002 scope-clarified per plan spec.

## One-liner

Reference doc clarity sweep: added missing CLI subcommands (spaces, list, --no-truncate), FUSE mount layout section, three missing crate entries, stale ADR-003 forward-reference fix, and ADR-002 scope note + See Also per plan.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Review + fix reference/ docs | 0a54e24 | cli.md, confluence.md, crates.md |
| 2 | Review + fix connectors/guide.md and decisions/ ADRs | 6a0b9aa | guide.md, 002-confluence-page-mapping.md |

## Friction Points Fixed

### docs/reference/cli.md
- **CRITICAL — Gap:** `reposix list` subcommand entirely absent from the CLI reference, despite being the primary query command referenced in confluence.md and throughout the docs.
- **CRITICAL — Gap:** `reposix spaces` subcommand (Phase 23) absent from usage block and reference.
- **Fix:** Added `list` and `spaces` to the usage block; added full `## reposix list` and `## reposix spaces` sections with flag tables.

### docs/reference/confluence.md
- **CRITICAL — Gap:** `--no-truncate` flag (Phase 21 HARD-02) not documented anywhere in the Confluence reference.
- **CRITICAL — Gap:** `.comments/` FUSE overlay (Phase 23) not mentioned anywhere — readers have no way to discover this capability.
- **CRITICAL — Gap:** No `reposix spaces` command shown in CLI surface examples.
- **Fix:** Added `reposix spaces` and `--no-truncate` to CLI surface section; added `--no-truncate` explanation paragraph; added full "FUSE mount layout (v0.4+)" section with ASCII directory tree showing `pages/`, `pages/<id>.comments/`, `tree/`, and `.gitignore`.

### docs/reference/crates.md
- **CRITICAL — Error:** Doc says "five crates" but workspace has eight (`reposix-github`, `reposix-confluence`, `reposix-swarm` are present and undocumented).
- **Fix:** Updated count to "eight crates"; added sections for `reposix-github` (v0.2 read-only GitHub adapter), `reposix-confluence` (v0.3–v0.5 capabilities table, key methods), and `reposix-swarm` (adversarial swarm harness).

### docs/connectors/guide.md
- **MINOR — Stale reference:** Guide says "when Phase 12 lands, a new ADR-003 will document the migration path" — but ADR-003 already exists (nested mount layout, Phase 13). Cold reader would be confused finding a different ADR-003.
- **Fix:** Changed "a new ADR-003" to "a future ADR" with an inline note that ADR-003 is the nested mount layout.

### docs/decisions/002-confluence-page-mapping.md
- **MINOR — Clarity:** Existing blockquote said "Superseded in part" which could mislead readers into thinking the whole ADR is stale. Plan required specific "Active — with scope note" wording clarifying which sections remain authoritative.
- **Fix:** Replaced blockquote with plan-specified wording. Added "See Also" section at bottom linking to ADR-003.

### docs/reference/git-remote.md
- No critical issues found. Authentication section correctly labeled as v0.2 future work. Verdict: CLEAR.

### docs/reference/http-api.md
- No critical issues found. Covers simulator API (not Confluence REST v2 — correct scope). Verdict: CLEAR.

### docs/decisions/001-github-state-mapping.md
- No changes needed. Well-structured, scope clearly stated, decision table self-contained. Verdict: CLEAR.

### docs/decisions/003-nested-mount-layout.md
- No changes needed. Comprehensive, well-referenced, consequences documented. Verdict: CLEAR.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Stale reference] guide.md "ADR-003" forward reference**
- **Found during:** Task 2 review of connectors/guide.md
- **Issue:** Guide says Phase 12 subprocess ABI will be documented in "a new ADR-003". ADR-003 already exists as the nested mount layout (Phase 13). A cold reader following this would find the wrong ADR.
- **Fix:** Replaced "a new ADR-003" with "a future ADR" plus an inline clarification note.
- **Files modified:** docs/connectors/guide.md
- **Commit:** 6a0b9aa

**2. [Rule 2 - Missing critical info] crates.md missing three crates**
- **Found during:** Task 1 review of crates.md
- **Issue:** Doc says "five crates" but workspace has eight. reposix-github, reposix-confluence, and reposix-swarm — all shipping crates — were entirely undocumented.
- **Fix:** Updated count, added sections for all three missing crates.
- **Files modified:** docs/reference/crates.md
- **Commit:** 0a54e24

## ADR Editing Compliance

- **ADR-001:** No changes.
- **ADR-002:** Only modified the status blockquote (replaced superseded note with plan-required "Active — with scope note" wording) and added "See Also" section at bottom. The Context, Decision, Consequences, and References sections are unmodified.
- **ADR-003:** No changes.

## Known Stubs

None — all docs updated reflect implemented features. The Phase 12 subprocess ABI section in guide.md is correctly labeled as future work, not a stub.

## Self-Check

Commits exist: 0a54e24, 6a0b9aa
