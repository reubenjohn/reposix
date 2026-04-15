---
phase: 22
plan: C (phase SUMMARY — covers all waves A+B+C)
subsystem: benchmarks
tags: [python, benchmark, tokenizer, anthropic-sdk, sha256-cache, offline, fixtures, docs]
dependency_graph:
  requires:
    - scripts/bench_token_economy.py (Plan 22-A output)
    - benchmarks/fixtures/github_issues.json (Plan 22-B output)
    - benchmarks/fixtures/confluence_pages.json (Plan 22-B output)
  provides:
    - benchmarks/RESULTS.md (real-tokenizer per-backend table, committed)
    - benchmarks/fixtures/mcp_jira_catalog.json.tokens.json
    - benchmarks/fixtures/reposix_session.txt.tokens.json
    - benchmarks/fixtures/github_issues.json.tokens.json
    - benchmarks/fixtures/confluence_pages.json.tokens.json
    - docs/why.md (headline updated to real count_tokens number)
    - CHANGELOG.md (BENCH-01..04 entry under [Unreleased])
  affects:
    - docs/why.md (headline % now matches RESULTS.md to 1 decimal)
    - benchmarks/README.md (offline contract documented)
tech_stack:
  added:
    - anthropic==0.72.0 (pip, requirements-bench.txt — Plan 22-A)
  patterns:
    - SHA-256 content-hash cache for deterministic offline reproducibility
    - Lazy anthropic import to allow test suite without SDK installed
    - counter= stub injection for unit-testable API calls
    - Per-backend table render (render_per_backend_table) with N/A placeholder row
    - load_raw_text() handles both JSON (list+dict) and .txt fixtures
key_files:
  created:
    - benchmarks/fixtures/mcp_jira_catalog.json.tokens.json
    - benchmarks/fixtures/reposix_session.txt.tokens.json
    - benchmarks/fixtures/github_issues.json.tokens.json
    - benchmarks/fixtures/confluence_pages.json.tokens.json
  modified:
    - scripts/bench_token_economy.py
    - scripts/test_bench_token_economy.py
    - benchmarks/RESULTS.md
    - benchmarks/README.md
    - docs/why.md
    - CHANGELOG.md
decisions:
  - "GITHUB_FIXTURE and CONFLUENCE_FIXTURE computed dynamically in main() from FIXTURES (not as module-level constants) so monkeypatching FIXTURES in tests covers per-backend paths automatically"
  - "load_raw_text() checks isinstance(data, dict) before .pop('_note') — github_issues.json is a list, not a dict"
  - "render_per_backend_table() takes rows list with None sentinel for chars/raw_tokens/reduction_pct to distinguish placeholder rows from zero-token rows"
  - "Auto-approved checkpoint C2 (dark-factory mode): 89.1% is within 70-99% range; four sidecars verified; --offline is a no-op"
  - "BENCH-03 (cold-mount time matrix) not shipped in Phase 22 — deferred; neither script nor plan included sim-cell matrix"
metrics:
  duration: ~30m (C1+C2+C3)
  completed: "2026-04-15"
  tasks_completed: 3
  files_changed: 9
---

# Phase 22 Summary: OP-8 — Honest Tokenizer Benchmarks

**One-liner:** Replaced `len/4` heuristic with Anthropic SDK `count_tokens` (89.1% real reduction vs 91.6% estimated); per-backend table for MCP/GitHub/Confluence; four committed `*.tokens.json` sidecars for offline CI; `docs/why.md` headline updated.

## Headline Numbers (before → after)

| Metric | Before Phase 22 | After Phase 22 |
|--------|----------------|----------------|
| Tokenizer method | `len(text) // 4` heuristic | Anthropic `count_tokens` API |
| MCP tokens | ~4,068 (estimate) | 4,883 (real) |
| reposix tokens | ~343 (estimate) | 531 (real) |
| Reduction % | 91.6% (estimated) | **89.1% (real)** |
| Ratio | ~11.9× (estimated) | **~9.2×** (real) |

The new number (89.1%) is lower than the old estimate (91.6%) by 2.5 percentage points — within the expected range given that `len/4` systematically undercounts tokens for JSON content (symbols, brackets, and punctuation tokenize to fewer chars/token than English prose). Per CONTEXT.md: "If it's lower, say so."

## Per-Backend Results (BENCH-02)

| Backend | Fixture | Real tokens | reposix tokens | Reduction |
|---------|---------|-------------|----------------|-----------|
| Jira (MCP) | mcp_jira_catalog.json | 4,883 | 531 | 89.1% |
| GitHub | github_issues.json | 3,661 | 531 | 85.5% |
| Confluence | confluence_pages.json | 2,251 | 531 | 76.4% |
| Jira (real adapter) | — | — | — | N/A (adapter not yet implemented) |

All backends show substantial reductions (76–89%) even against comparatively smaller raw-JSON payloads.

## Wave Commits

### Wave 1 — Plan A (bench script upgrade)
| Task | Commit | Description |
|------|--------|-------------|
| A1+A2 | `c804625` | SHA-256 cache + offline flag + 6 pytest tests + requirements-bench.txt |

### Wave 1 — Plan B (fixtures)
| Task | Commit | Description |
|------|--------|-------------|
| B1 | `2fdbcac` | github_issues.json fixture + check_fixtures.py |
| B2 | `0cd35ab` | confluence_pages.json fixture |
| B3 | `372f9ba` | fixtures/README.md with provenance + offline contract |

### Wave 2 — Plan C (wire + ship)
| Task | Commit | Description |
|------|--------|-------------|
| C1 | `e422ba3` | per-backend table + load_raw_text + render_per_backend_table + tests 7/8/9 |
| C2 | (auto-approved) | API run — four *.tokens.json sidecars populated |
| C3 | (this commit) | sidecars committed + docs/why.md + README + CHANGELOG + SUMMARY |

## Requirements Closed

| Requirement | Status | Notes |
|-------------|--------|-------|
| BENCH-01 | **Closed** | count_tokens replaces len/4; cache written + committed |
| BENCH-02 | **Closed** | Per-backend table: MCP, GitHub, Confluence rows + Jira-real N/A |
| BENCH-03 | **Deferred** | Cold-mount time-to-first-ls matrix not shipped; stretch goal per 22-RESEARCH.md |
| BENCH-04 | **Closed** | docs/why.md headline = 89.1% (real), old 92.3% gone, count_tokens cited |

## Test Results

```
9 passed in 0.04s
```

All 9 tests pass offline with no `anthropic` package required:
- Tests 1–4: SHA-256 cache + offline guard (Plan 22-A)
- Tests 5–6: end-to-end main() with monkeypatched FIXTURES (Plan 22-A)
- Test 7: per-backend table has all four fixture rows including N/A
- Test 8: Jira-real row has no fake percentage or token count
- Test 9: headline reduction matches `round(100*(1-reposix/mcp), 1)` from cache

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] load_raw_text() TypeError on list-shaped JSON**
- **Found during:** Task C1 — pytest run
- **Issue:** `data.pop("_note", None)` raises `TypeError` when `data` is a list (github_issues.json is a JSON array, not an object)
- **Fix:** Added `isinstance(data, dict)` guard before `.pop()`
- **Files modified:** `scripts/bench_token_economy.py`
- **Commit:** `e422ba3`

**2. [Rule 1 - Bug] GITHUB_FIXTURE/CONFLUENCE_FIXTURE module-level constants not updated by monkeypatch**
- **Found during:** Task C1 — test 6 (`test_main_falls_back_gracefully_when_per_backend_fixtures_absent`) failed because `gh_path.exists()` resolved to the real fixture on disk instead of the tmp_path fixture dir
- **Fix:** Changed `main()` to resolve `gh_path = FIXTURES / "github_issues.json"` dynamically at call time (from the current `FIXTURES` value), so monkeypatching `FIXTURES` automatically covers per-backend paths. Module-level `GITHUB_FIXTURE` and `CONFLUENCE_FIXTURE` constants retained for documentation/import purposes.
- **Files modified:** `scripts/bench_token_economy.py`
- **Commit:** `e422ba3`

## Follow-ups

- **BENCH-03 deferred:** Cold-mount time-to-first-ls matrix (sim vs GitHub vs Confluence) was a stretch goal in 22-RESEARCH.md. Neither the plan nor the script implemented it. A future OP-8 follow-up phase can add this with `REPOSIX_BENCH_LIVE=1` gating.

## Known Stubs

None. All four `*.tokens.json` sidecars are committed with real API-measured counts. `docs/why.md` and `benchmarks/RESULTS.md` are numerically consistent to 1 decimal.

## Threat Mitigations Applied

| Threat | Mitigation | Verified |
|--------|-----------|---------|
| T-22-C-01: ANTHROPIC_API_KEY leak | Used `"$ANTHROPIC_API_KEY"` env expansion; never echoed value | Manual review |
| T-22-C-02: Numerical drift docs vs RESULTS | Cross-file diff check in acceptance criteria | `diff` spot-check passed (89.1% = 89.1%) |
| T-22-C-03: Tokenizer provenance opaque | sidecars record `model` + `counted_at`; RESULTS.md header cites API | grep confirmed |
| T-22-C-04: Anthropic rate limit | 4 API calls total; well under free tier | No 429s observed |
| T-22-C-05: Secret in committed fixture | Plan 22-B acceptance checked; sidecars contain only hash+count | check_fixtures.py confirmed |

## Self-Check: PASSED

Files created/modified:
- [x] `benchmarks/fixtures/mcp_jira_catalog.json.tokens.json` — exists, has content_hash + input_tokens
- [x] `benchmarks/fixtures/reposix_session.txt.tokens.json` — exists, has content_hash + input_tokens
- [x] `benchmarks/fixtures/github_issues.json.tokens.json` — exists, has content_hash + input_tokens
- [x] `benchmarks/fixtures/confluence_pages.json.tokens.json` — exists, has content_hash + input_tokens
- [x] `benchmarks/RESULTS.md` — contains count_tokens, 89.1%, per-backend table
- [x] `docs/why.md` — contains count_tokens, 89.1%, no len/4, no 92.3%
- [x] `benchmarks/README.md` — contains count_tokens, offline instructions
- [x] `CHANGELOG.md` — contains BENCH-01, BENCH-02, BENCH-04

Commits:
- [x] `e422ba3` (C1) — exists in git log
