---
phase: 22-op-8-honest-tokenizer-benchmarks
reviewed: 2026-04-15T00:00:00Z
depth: quick
files_reviewed: 5
files_reviewed_list:
  - scripts/bench_token_economy.py
  - scripts/test_bench_token_economy.py
  - scripts/check_fixtures.py
  - requirements-bench.txt
  - benchmarks/fixtures/README.md
findings:
  critical: 0
  warning: 2
  info: 2
  total: 4
status: issues_found
---

# Phase 22: Code Review Report

**Reviewed:** 2026-04-15T00:00:00Z
**Depth:** quick
**Files Reviewed:** 5
**Status:** issues_found

## Summary

Reviewed the five files that make up the OP-8 honest tokenizer benchmark addition: the
benchmark script, its test suite, the fixture validator, the pinned requirements file, and
the fixtures README. No critical security or data-loss issues were found. The code is
well-structured with a solid SHA-256 cache-integrity story and good offline-reproducibility
guarantees. Two warnings and two info items follow.

## Warnings

### WR-01: `check_fixtures.py` uses a relative `FIXTURES` path — breaks when run from any directory other than repo root

**File:** `scripts/check_fixtures.py:20`
**Issue:** `FIXTURES = pathlib.Path("benchmarks/fixtures")` is a relative path. The script's
own docstring says "Run from the repository root", but relative paths in a module-level
constant are fragile: pytest, CI, or a developer `cd`-ing to `scripts/` before running the
script will silently resolve the path against the wrong working directory, causing all three
check functions to return `MISSING` errors rather than actual fixture content. The peer
script `bench_token_economy.py` correctly anchors its path with
`pathlib.Path(__file__).resolve().parent.parent / "benchmarks"`.

**Fix:**
```python
# Replace line 20 in check_fixtures.py
FIXTURES = pathlib.Path(__file__).resolve().parent.parent / "benchmarks" / "fixtures"
```

### WR-02: `load_mcp_bytes` and `load_raw_text` use `separators=(", ", ": ")` — misleadingly named "compact" but is actually default format; hash-sensitive if ever corrected

**File:** `scripts/bench_token_economy.py:252,289`
**Issue:** The docstring for `load_raw_text` (line 267) describes the format as
"compact-with-spaces", and both functions pass `separators=(", ", ": ")` to `json.dumps`.
However, this produces output *identical* to the Python default (no `separators` argument),
not compact JSON (which would be `separators=(",", ":")`). This is not currently a bug
because both functions use the same separators and the SHA-256 cache keys are consistent
within a single run. However, if a future developer corrects the separators to true compact
(`","` / `":"`) to actually reduce token counts, all committed `*.tokens.json` sidecars
will immediately have stale `content_hash` values, requiring full API regeneration with no
warning beyond the stale-cache check at runtime.

**Fix:** Either rename the format to "default" in the docstring and leave the separators
unchanged (preferred, zero risk), or truly compact the output and regenerate all cached
sidecars:
```python
# Option A — correct the docstring (no code change needed)
# "json.dumps with default separators (comma-space, colon-space)"

# Option B — true compact (requires sidecar regeneration)
serialized = json.dumps(data, separators=(",", ":"))
```

## Info

### IN-01: Module-level constants `GITHUB_FIXTURE` and `CONFLUENCE_FIXTURE` are dead code

**File:** `scripts/bench_token_economy.py:55-56`
**Issue:** `GITHUB_FIXTURE` and `CONFLUENCE_FIXTURE` are defined at module scope but `main()`
never references them — it recomputes `gh_path` and `conf_path` from `FIXTURES` directly
(lines 386-387). The constants exist solely so test code can monkeypatch them
(`monkeypatch.setattr(bench, "GITHUB_FIXTURE", ...)`), but the monkeypatching has no effect
on the actual `main()` logic because `main()` reads `FIXTURES` instead. Tests that monkeypatch
these constants (e.g. `test_per_backend_table_rendered_with_all_four_rows`) appear to work
only because they also monkeypatch `FIXTURES`, making the `GITHUB_FIXTURE`/`CONFLUENCE_FIXTURE`
patches redundant rather than harmful.

**Fix:** Remove the two module-level constants and delete the corresponding monkeypatch lines
from the test cases, or add a usage in `main()` if the intent was to allow external override:
```python
# Remove from bench_token_economy.py:
# GITHUB_FIXTURE = FIXTURES / "github_issues.json"
# CONFLUENCE_FIXTURE = FIXTURES / "confluence_pages.json"

# Remove from test files:
# monkeypatch.setattr(bench, "GITHUB_FIXTURE", ...)
# monkeypatch.setattr(bench, "CONFLUENCE_FIXTURE", ...)
```

### IN-02: `requirements-bench.txt` pins only `anthropic==0.72.0` — no transitive pins

**File:** `requirements-bench.txt:1`
**Issue:** The single-line pin captures the direct dependency but not its transitive
dependencies (httpx, anyio, etc.). This means `pip install -r requirements-bench.txt` on a
fresh environment may resolve different transitive versions across machines, undermining
reproducibility. For a benchmark whose headline claim depends on deterministic token counts,
this is worth noting.

**Fix:** Generate a fully-pinned lockfile (e.g. `pip-compile requirements-bench.in > requirements-bench.txt`) or document that transitive resolution is intentionally left floating because `anthropic`'s SDK guarantees token-count stability across its transitive deps.

---

_Reviewed: 2026-04-15T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: quick_
