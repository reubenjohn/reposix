---
phase: 25
plan: "25-01 + 25-02"
subsystem: docs
tags: [docs, cross-refs, mkdocs, nav, version-bump, changelog]
dependency_graph:
  requires: [Phase 24]
  provides: [docs/research nav, cross-ref consistency, v0.7.0 release]
  affects: [mkdocs.yml, CLAUDE.md, README.md, .planning/research/threat-model-and-critique.md, Cargo.toml, Cargo.lock, CHANGELOG.md]
tech_stack:
  added: []
  patterns: [mkdocs nav section, visible markdown redirect stub]
key_files:
  created:
    - docs/research/initial-report.md
    - docs/research/agentic-engineering-reference.md
    - InitialReport.md (root stub with visible blockquote redirect)
    - AgenticEngineeringReference.md (root stub with visible blockquote redirect)
  modified:
    - mkdocs.yml
    - CLAUDE.md
    - README.md
    - .planning/research/threat-model-and-critique.md
    - .planning/research/simulator-design.md
    - .planning/research/fuse-rust-patterns.md
    - .planning/research/git-remote-helper.md
    - .planning/PROJECT.md
    - docs/index.md
    - docs/security.md
    - docs/why.md
    - benchmarks/README.md
    - Cargo.toml
    - Cargo.lock
    - CHANGELOG.md
decisions:
  - "Root stub format: visible markdown blockquote redirect (not invisible HTML comment) — GitHub renders it, users who land on the old URL see a clear forwarding link"
  - "mkdocs.yml Research section inserted between Decisions and Reference in the nav"
  - "Historical planning records (SESSION files, HANDOFF, CHANGELOG, REQUIREMENTS) retain old filenames when describing the move itself — changing them would be historically misleading"
  - "Social media docs retain commit-hash permalinks — immutable public links shared at time of posting"
  - "YAML safe_load fails on mkdocs.yml due to pre-existing !!python/name: tag — validated with grep instead"
metrics:
  duration: "~25 min"
  completed: "2026-04-16"
  tasks_completed: 4
  files_changed: 15
---

# Phase 25 Summary: OP-11 Docs Reorg + v0.7.0 Release

**Status: SHIPPED**

**One-liner:** Moved `InitialReport.md` and `AgenticEngineeringReference.md` to `docs/research/` with visible redirect stubs at root; updated all cross-references across 10 files; added Research nav section to mkdocs.yml; bumped workspace to v0.7.0 and promoted CHANGELOG.

## Plans Executed

| Plan | Title | Commit | Status |
|------|-------|--------|--------|
| 25-01 | Cross-ref audit + mkdocs.yml Research nav | b2e8f84 | SHIPPED |
| 25-02 | v0.7.0 version bump, CHANGELOG promotion, STATE.md, SUMMARY | 3a92e02 | SHIPPED |

## Artifacts Produced

| Artifact | Description |
|----------|-------------|
| `docs/research/initial-report.md` | Full architectural argument for FUSE + git-remote-helper (moved from root) |
| `docs/research/agentic-engineering-reference.md` | Dark-factory pattern, lethal trifecta, simulator-first (moved from root) |
| `InitialReport.md` | Root stub with visible markdown blockquote redirect to docs/research/ |
| `AgenticEngineeringReference.md` | Root stub with visible markdown blockquote redirect to docs/research/ |
| `mkdocs.yml` | Research nav section added between Decisions and Reference |
| `CLAUDE.md` | Cross-refs updated to docs/research/ paths (Quick links section) |
| `.planning/research/threat-model-and-critique.md` | ~20 bare cross-refs updated to docs/research/ paths |
| `CHANGELOG.md` | [v0.7.0] promoted from [Unreleased]; new empty [Unreleased] section added |
| `Cargo.toml` | Workspace version 0.6.0 → 0.7.0 |
| `Cargo.lock` | Regenerated via `cargo check --workspace` |

## Acceptance Criteria Results

| Criterion | Result |
|-----------|--------|
| `grep -c "docs/research/initial-report.md" CLAUDE.md` >= 2 | 2 ✓ |
| `grep -c "docs/research/agentic-engineering-reference.md" CLAUDE.md` >= 2 | 2 ✓ |
| `grep -c "docs/research/" .planning/research/threat-model-and-critique.md` >= 15 | 20 ✓ |
| `grep -c "InitialReport\.md\|AgenticEngineeringReference\.md" threat-model` = 0 | 0 ✓ |
| `grep -q "Research" mkdocs.yml` | OK ✓ |
| `grep -q "research/initial-report.md" mkdocs.yml` | OK ✓ |
| `grep -q "research/agentic-engineering-reference.md" mkdocs.yml` | OK ✓ |
| Root stubs have visible markdown blockquote redirects | Confirmed ✓ |
| `grep 'version = "0.7.0"' Cargo.toml` | 1 match ✓ |
| `grep '\[v0\.7\.0\]' CHANGELOG.md` | 2 matches (heading + URL) ✓ |
| `grep '\[Unreleased\]' CHANGELOG.md` | Present ✓ |
| `grep 'v0\.6\.0\.\.\.v0\.7\.0' CHANGELOG.md` | Present ✓ |
| `cargo check --workspace` errors | 0 ✓ |
| `grep "Phase 25 SHIPPED" .planning/STATE.md` | Present ✓ |
| `grep "Phase 26" .planning/STATE.md` | Present ✓ |

## Deviations from Plan

### Scoped decisions (not bugs)

**1. [Rule 1 - Scope] Historical planning records left unchanged**
- **Found during:** Task 1 (25-01) final audit
- **Issue:** SESSION files, HANDOFF.md, REQUIREMENTS.md, ROADMAP.md, CHANGELOG.md contain `InitialReport.md` and `AgenticEngineeringReference.md` by name — in the context of describing the move itself.
- **Decision:** These are accurate historical artifacts. Changing them would be misleading. Left as-is.

**2. [Rule 1 - Scope] Social media permalinks left unchanged**
- **Found during:** Task 1 (25-01) final audit
- **Issue:** `docs/social/twitter.md` and `docs/social/linkedin.md` contain commit-hash permalinks to the old root locations — immutable public links shared at time of posting.
- **Decision:** Left as-is to preserve the historical record.

**3. [Rule 3 - Workaround] mkdocs YAML validation uses grep instead of safe_load**
- **Found during:** Task 2 (25-01) verification
- **Issue:** `python3 -c "import yaml; yaml.safe_load(...)"` fails on the pre-existing `!!python/name:pymdownx.superfences.fence_code_format` tag.
- **Fix:** Validated nav entries via `grep -c` instead. Pre-existing issue unrelated to this plan.

## Known Stubs

None — docs-only changes with no UI/data stubs.

## Threat Flags

None — docs-only changes, no new network endpoints or trust boundaries.

## Next Phase

Phase 26 — Docs clarity overhaul: unbiased subagent review of all user-facing Markdown docs (README.md, all docs/ pages, root-level orphan cleanup).

## Self-Check: PASSED

- Commit `b2e8f84` (25-01): confirmed in git log
- Commit `3a92e02` (25-02): confirmed in git log
- `docs/research/initial-report.md`: exists
- `docs/research/agentic-engineering-reference.md`: exists
- `Cargo.toml` version = "0.7.0": confirmed
- `CHANGELOG.md` [v0.7.0]: confirmed
- `STATE.md` "Phase 25 SHIPPED": confirmed
- `ROADMAP.md` Phase 25 plan list both checked: confirmed
