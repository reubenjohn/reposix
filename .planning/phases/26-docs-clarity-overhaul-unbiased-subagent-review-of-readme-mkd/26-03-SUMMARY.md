---
phase: 26
plan: "26-03"
subsystem: docs
tags: [docs, clarity, cold-reader-review, security, architecture, demo, contributing]
dependency_graph:
  requires: [26-01]
  provides: [clarity-reviewed-core-docs]
  affects: [docs/architecture.md, docs/why.md, docs/security.md, docs/demo.md, docs/demos/index.md, docs/development/contributing.md]
tech_stack:
  added: []
  patterns: [doc-clarity-review, isolated-cold-reader-review]
key_files:
  modified:
    - docs/architecture.md
    - docs/why.md
    - docs/security.md
    - docs/demo.md
    - docs/development/contributing.md
decisions:
  - "Kept 89.1% (count_tokens) in why.md table for accuracy; added prose explaining 92.3% (chars/4 heuristic) vs 89.1% (API-measured) — both support the same conclusion"
  - "Phase 21 HARD-* items added to security.md shipped section; 500-page truncation moved from deferred to shipped (HARD-02 closed)"
  - "docs/demos/index.md had no critical friction points — no edits needed (CLEAR on first review)"
metrics:
  duration: "~25 minutes"
  completed: "2026-04-16"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 5
---

# Phase 26 Plan 03: Clarity Review — Core Docs Pages Summary

**One-liner:** Cold-reader review of six core docs pages; five files patched for clarity (SG-* orientation, token-economy consistency, Phase 21 hardening items, git-vs-FUSE layout disambiguation, missing cargo check command); docs/demos/index.md was already CLEAR.

## What Was Done

Performed isolated cold-reader review of six docs pages using the doc-clarity-review skill's DEFAULT_PROMPT criteria (Friction Points, Unanswered Questions, Over-Explained, Under-Explained, Missing References, CLEAR/NEEDS WORK verdict). Applied targeted fixes to CRITICAL friction points only.

### Task 1: docs/architecture.md, docs/why.md, docs/security.md

**docs/architecture.md — NEEDS WORK → CLEAR**

Friction points found:
- SG-* codes used throughout without explanation — cold reader has no idea what SG-01 means
- "Phase 10 rewired reads; Phase 14 rewired writes" — meaningless internal references
- IssueBackend trait mentioned without explanation of what adding a new backend requires

Fixes applied:
- Replaced phase references with substantive explanation of the IssueBackend trait contract
- Added sentence after security perimeter diagram explaining SG-* codes point to the security page guardrails table
- Added one sentence explaining what implementing IssueBackend means for new backend authors

**docs/why.md — NEEDS WORK → CLEAR**

Friction points found:
- Token-economy callout said "89.1% reduction" but the demo recording, benchmark chart SVG, and demos/index.md all show "92.3%"
- Table showed 531/4,883 tokens (89.1%) with no explanation of why the headline differs
- Internal inconsistency between callout, table, prose, and linked assets

Fixes applied:
- Updated callout to say "92.3% reduction" and "~13× less context" (matching demo assets)
- Replaced confusing prose paragraph with clear explanation: 92.3% = original chars/4 heuristic (basis for demo recording); 89.1% = Phase 22 count_tokens API (more rigorous); both support the same conclusion

**docs/security.md — NEEDS WORK → CLEAR**

Friction points found:
- Phase 21 HARD-00..05 hardening items completely absent from "What shipped after v0.1" section
- 500-page truncation still listed as *deferred* in the deferred section — Phase 21 (HARD-02) shipped the fix
- Cold reader gets false picture: thinks truncation is still a known gap when it was closed

Fixes applied:
- Added "OP-7 hardening bundle (Phase 21, HARD-00..05)" entry with all six items to the shipped section
- Updated 500-page truncation deferred item to note it was shipped in Phase 21 (HARD-02)

### Task 2: docs/demo.md, docs/demos/index.md, docs/development/contributing.md

**docs/demo.md — NEEDS WORK → CLEAR**

Friction points found:
- Step 7 initializes a git repo at `/tmp/demo-repo` and shows `ls` with `0001.md...0006.md` — after step 4 showed `issues/00000000001.md` in the FUSE mount. No explanation that these are different layouts for different access paths.
- Limitations section says "Those are deferred to v0.2" — stale, project is now on v0.4+

Fixes applied:
- Added orientation paragraph before the step 7 code block explaining: this is a separate git repo from the FUSE mount; the git-remote helper uses a different (root-level, short-form) file layout than the FUSE daemon's `issues/` bucket
- Updated "deferred to v0.2" limitation to point to current security.md deferred section

**docs/demos/index.md — CLEAR (no edits needed)**

Review found minor items (Tier 4 swarm binary not oriented, "sim-direct / fuse mode split" jargon) but none CRITICAL. The document successfully orients a cold reader. No changes made.

**docs/development/contributing.md — NEEDS WORK → CLEAR**

Friction points found:
- `cargo check --workspace` (the fast type-check command, recommended in CLAUDE.md as the "local dev loop" entry point) was missing from the Quickstart dev commands
- A contributor would go straight to `cargo build` (slow) without knowing the fast feedback loop

Fixes applied:
- Added `cargo check --workspace` with explanatory comment as the first command in the Quickstart block, matching CLAUDE.md's dev loop order

## Deviations from Plan

### Auto-applied fixes (no deviations from plan intent)

None. All fixes were targeted clarity improvements within plan scope.

### Token-economy number reconciliation

The plan said "confirm '92.3%' is present; if it says '91.6%' or '~91%', update to 92.3%." The doc said 89.1% (a third value — Phase 22's actual count_tokens measurement). Rather than blindly replacing 89.1% with 92.3% in the table (which would create a factually wrong table), the fix added prose explaining both measurements and updated the headline callout to 92.3% (matching all external assets). This is a Rule 2 (correctness) improvement — internal inconsistency was the clarity bug.

## Known Stubs

None. All six docs contain real content wired to actual implementation.

## Threat Flags

None. No new network endpoints, auth paths, or schema changes introduced.

## Self-Check

- [x] docs/architecture.md exists and modified
- [x] docs/why.md contains "92.3%"
- [x] docs/security.md contains HARD-00 through HARD-05
- [x] docs/demo.md clarifies git-repo vs FUSE-mount layout in step 7
- [x] docs/development/contributing.md contains `cargo check --workspace`
- [x] docs/demos/index.md reviewed (no edits needed — already CLEAR)
- [x] Commit 18d8af4 exists

## Self-Check: PASSED
