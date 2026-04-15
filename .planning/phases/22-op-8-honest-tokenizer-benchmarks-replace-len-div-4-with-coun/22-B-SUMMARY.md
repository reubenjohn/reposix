---
phase: 22
plan: B
subsystem: benchmarks/fixtures
tags: [fixtures, benchmarks, github, confluence, offline]
dependency_graph:
  requires: []
  provides:
    - benchmarks/fixtures/github_issues.json
    - benchmarks/fixtures/confluence_pages.json
    - benchmarks/fixtures/README.md
    - scripts/check_fixtures.py
  affects:
    - scripts/bench_token_economy.py (Plan 22-C reads these fixtures)
tech_stack:
  added: []
  patterns:
    - Committed fixture files as offline benchmark inputs
    - SHA-256 sidecar cache contract (*.tokens.json)
    - check_fixtures.py as committed validation artifact replacing ad-hoc pipelines
key_files:
  created:
    - benchmarks/fixtures/github_issues.json
    - benchmarks/fixtures/confluence_pages.json
    - benchmarks/fixtures/README.md
    - scripts/check_fixtures.py
  modified: []
decisions:
  - scripts/check_fixtures.py promoted from ad-hoc pipeline to committed artifact per CLAUDE.md §4
  - github_issues.json: 3 issues at 11579 bytes (within 4-12 KB spec)
  - confluence_pages.json: 3 pages at 7281 bytes (within 6-16 KB spec)
  - No shape deviations from the Interfaces block in the plan
metrics:
  duration: ~15 minutes
  completed: "2026-04-15"
  tasks: 3
  files_created: 4
---

# Phase 22 Plan B: Fixtures and Table SUMMARY

**One-liner:** GitHub REST v3 and Confluence v2 synthetic fixtures for BENCH-02 per-backend token comparison, with offline cache contract README and committed validator.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| B1 | Create github_issues.json fixture | `2fdbcac` | `benchmarks/fixtures/github_issues.json`, `scripts/check_fixtures.py` |
| B2 | Create confluence_pages.json fixture | `0cd35ab` | `benchmarks/fixtures/confluence_pages.json` |
| B3 | Fixture provenance README | `372f9ba` | `benchmarks/fixtures/README.md` |

## Fixture Measurements

| File | wc -c bytes | Spec range | Status |
|------|------------|-----------|--------|
| `github_issues.json` | 11579 | 4000–12000 | PASS |
| `confluence_pages.json` | 7281 | 6000–16000 | PASS |
| `README.md` | 4311 | — | created |
| `check_fixtures.py` | ~5400 | — | validation artifact |

## Shape Conformance

**github_issues.json:** Array of 3 issues. All required keys present on every issue:
`id`, `node_id`, `number`, `title`, `user` (with `login`/`id`/`type`), `labels` (with `id`/`name`/`color`), `state`, `assignee`, `assignees`, `milestone`, `comments`, `created_at`, `updated_at`, `closed_at`, `author_association`, `body`, `reactions` (with all per-emoji counts), `pull_request`, `url`, `repository_url`, `labels_url`, `comments_url`, `events_url`, `html_url`. No fields omitted.

Issue content:
1. #42: FUSE hang when sim dies — bug, open, 2 comments
2. #43: `--offline` flag for bench script — enhancement+benchmarks, open, milestone set
3. #44: ADF table cell alignment round-trip — bug+confluence, closed, `closed_at` populated

**confluence_pages.json:** Object with `results[]` (3 pages) and top-level `_links.base`. All required keys present on every page: `id`, `status`, `title`, `spaceId`, `parentId`, `parentType`, `position`, `authorId`, `ownerId`, `createdAt`, `version{}`, `body.atlas_doc_format{value, representation}`, `_links`. Every `atlas_doc_format.value` is a JSON-stringified ADF doc (`{"type":"doc",...}`) that re-parses correctly.

Page content:
1. Engineering Runbook — heading + 2 paragraphs + codeBlock
2. Security Review 2026-Q2 — heading + paragraph + bulletList (4 items) + paragraph
3. Onboarding Guide (draft) — heading + 2 paragraphs + table (2 rows × 3 cols), version 4

## Security Check

- `github_issues.json`: no `GITHUB_TOKEN`, no `password`, no `secret`, no `bearer` credentials. Uses `example-org/example-repo`.
- `confluence_pages.json`: no `ATLASSIAN_API_KEY`, no `password`, no `reuben` PII. Uses `example.atlassian.net`.
- All usernames are fictional: `alice-bot`, `benchmark-ci`, `fuse-agent`, invented Atlassian authorIds.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical functionality] check_fixtures.py promoted from ad-hoc pipeline to committed artifact**
- **Found during:** Task B1 verification
- **Issue:** The acceptance-criteria commands from the plan spec were multi-stage pipelines over 300 chars, blocked by CLAUDE.md §4 (ad-hoc bash is a missing-tool signal).
- **Fix:** Created `scripts/check_fixtures.py` — a committed validation script covering all three fixture checks (shape, size, secret scan, ADF re-parse). This is grounding infrastructure that the next agent and CI can use directly.
- **Files modified:** `scripts/check_fixtures.py` (new)
- **Commit:** `2fdbcac`

No other deviations. All fixture shapes match the Interfaces block exactly. No plan files were modified. No Python or Rust source files were touched (pure fixture + docs plan as specified).

## Known Stubs

None. Both fixture files are complete and ready for Plan 22-C to consume via `(FIXTURES / "github_issues.json").read_text()` and `(FIXTURES / "confluence_pages.json").read_text()`.

## Threat Flags

None. Both fixtures are author-authored synthetic bytes (not remote-sourced). T-22-B-01 mitigation applied: acceptance criteria confirmed no credential-shaped strings. T-22-B-03 mitigation applied: README disclaims synthetic provenance prominently.

## Self-Check: PASSED

- `benchmarks/fixtures/github_issues.json` exists: FOUND
- `benchmarks/fixtures/confluence_pages.json` exists: FOUND
- `benchmarks/fixtures/README.md` exists: FOUND
- `scripts/check_fixtures.py` exists: FOUND
- Commit `2fdbcac` (B1): FOUND
- Commit `0cd35ab` (B2): FOUND
- Commit `372f9ba` (B3): FOUND
- `python3 scripts/check_fixtures.py` exits 0: CONFIRMED
