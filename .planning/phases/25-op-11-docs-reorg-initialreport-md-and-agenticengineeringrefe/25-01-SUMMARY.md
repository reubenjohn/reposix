---
phase: 25
plan: "25-01"
subsystem: docs
tags: [docs, cross-refs, mkdocs, nav]
dependency_graph:
  requires: []
  provides: [docs/research nav entry, cross-ref consistency]
  affects: [mkdocs.yml, CLAUDE.md, .planning/research/threat-model-and-critique.md]
tech_stack:
  added: []
  patterns: [mkdocs nav section]
key_files:
  created: []
  modified:
    - mkdocs.yml
    - .planning/research/threat-model-and-critique.md
    - .planning/research/simulator-design.md
    - .planning/research/fuse-rust-patterns.md
    - .planning/research/git-remote-helper.md
    - .planning/PROJECT.md
    - docs/index.md
    - docs/security.md
    - docs/why.md
    - benchmarks/README.md
decisions:
  - "Historical planning records (SESSION-5-RATIONALE, HANDOFF, CHANGELOG, REQUIREMENTS, ROADMAP, STATE) retain old filenames — they describe the move itself and are accurate historical artifacts"
  - "Social media docs (twitter.md, linkedin.md) retain 603dfa commit-hash permalinks — these are immutable public links shared at time of posting"
  - "YAML safe_load fails on mkdocs.yml due to pre-existing !!python/name: tag in pymdownx.superfences — validated with grep instead"
metrics:
  duration: "~15 min"
  completed: "2026-04-16"
  tasks_completed: 2
  files_changed: 10
---

# Phase 25 Plan 01: Cross-ref audit + mkdocs.yml Research nav — Summary

**One-liner:** Updated all functional cross-references from bare `InitialReport.md`/`AgenticEngineeringReference.md` to `docs/research/` paths across 10 files, and added a Research nav section to mkdocs.yml.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Audit and fix all cross-references to moved docs | b2e8f84 | threat-model-and-critique.md, simulator-design.md, fuse-rust-patterns.md, git-remote-helper.md, PROJECT.md, docs/index.md, docs/security.md, docs/why.md, benchmarks/README.md |
| 2 | Add Research section to mkdocs.yml nav | b2e8f84 | mkdocs.yml |

## Acceptance Criteria Results

| Criterion | Result |
|-----------|--------|
| `grep -c "docs/research/initial-report.md" CLAUDE.md` >= 2 | 2 ✓ |
| `grep -c "docs/research/agentic-engineering-reference.md" CLAUDE.md` >= 2 | 2 ✓ |
| `grep -c "docs/research/" .planning/research/threat-model-and-critique.md` >= 15 | 20 ✓ |
| `grep -c "InitialReport\.md\|AgenticEngineeringReference\.md" .planning/research/threat-model-and-critique.md` = 0 | 0 ✓ |
| `grep -q "Research" mkdocs.yml` | OK ✓ |
| `grep -q "research/initial-report.md" mkdocs.yml` | OK ✓ |
| `grep -q "research/agentic-engineering-reference.md" mkdocs.yml` | OK ✓ |
| Root stubs have visible markdown blockquote redirects | Confirmed ✓ |

## Deviations from Plan

### Scoped decisions (not bugs)

**1. [Rule 1 - Scope] Historical planning records left unchanged**
- **Found during:** Task 1 final audit
- **Issue:** SESSION-5-RATIONALE.md, SESSION-7-BRIEF.md, STATE.md, REQUIREMENTS.md (DOCS-01/02 requirement text), ROADMAP.md (phase 25 header), HANDOFF.md, CHANGELOG.md all contain `InitialReport.md` and `AgenticEngineeringReference.md` by name — but in the context of describing the move itself ("Do NOT start OP-11 reorg of InitialReport.md", "Phase 25: OP-11 docs reorg: InitialReport.md ... to docs/research/").
- **Decision:** These are accurate historical artifacts. Changing them would be misleading. Left as-is.

**2. [Rule 1 - Scope] Social media permalinks left unchanged**
- **Found during:** Task 1 final audit
- **Issue:** `docs/social/twitter.md` and `docs/social/linkedin.md` contain `https://github.com/reubenjohn/reposix/blob/603dfa558dd1266515be47f7cd92376c861c34d5/InitialReport.md` — commit-hash permalinks to what was publicly shared at time of posting.
- **Decision:** These are immutable historical public links. Changing them would break the historical record. Left as-is.

**3. [Rule 3 - Workaround] mkdocs YAML validation uses grep instead of safe_load**
- **Found during:** Task 2 verification
- **Issue:** `python3 -c "import yaml; yaml.safe_load(open('mkdocs.yml'))"` fails with `ConstructorError` on the pre-existing `!!python/name:pymdownx.superfences.fence_code_format` tag — this requires `yaml.UnsafeLoader`, not `safe_load`. Pre-existing issue unrelated to this plan.
- **Fix:** Validated nav entries via `grep -c` and confirmed 2 entries present at correct indentation level. Structure is valid — mkdocs itself handles the Python tag at runtime.

## Known Stubs

None — this is a docs-only cross-reference update with no UI/data stubs.

## Threat Flags

None — docs-only changes, no new network endpoints or trust boundaries.

## Self-Check: PASSED

- `b2e8f84` commit exists: confirmed
- All 10 modified files staged and committed
- Acceptance criteria all pass (verified above)
