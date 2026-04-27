---
phase: 59-docs-repro-dimension-agent-ux-thin-home-perf-relocate
plan: 05
wave: E
status: shipped
duration_min: 8
commit: 1fbf59d
shipped_at: "2026-04-27T22:35:00Z"
---

# Wave E — SIMPLIFY-11 perf relocate (file-relocate-only stub)

## Outcome

GREEN. 3 files relocated via git mv (history-preserved); 2 shims at old paths; 1 deletion (`scripts/test_bench_token_economy.py` — pytest auto-discovers from new location, no external callers). Catalog row corrected hyphen→underscore (4-char edit per Wave E pivot rule). 9/9 tests pass at new location.

## BEFORE-state (Task 1)

```
$ python3 -m pytest scripts/test_bench_token_economy.py
9 passed in 0.07s
```

`.github/workflows/bench-latency-cron.yml` exists; both ci.yml and bench-latency-cron.yml invoke `scripts/latency-bench.sh` directly.

`scripts/bench_token_economy.py` and `scripts/latency-bench.sh` use one-level path arithmetic (`parent.parent` / `SCRIPT_DIR/..`) to derive REPO_ROOT — needs 3-level fix at new home.

## Option B vs Option A (Task 2)

Chose **Option B** (underscore) for the perf-dimension Python files:
- The test file imports the production module: `import bench_token_economy as bench` (line 19). Python module syntax forbids hyphens.
- Wave A's catalog row at `perf/token-economy-bench` specified hyphenated path — corrected to underscore in same Wave E commit (4-char edit).
- 9/9 tests pass at new location with `sys.path.insert(parent)` sibling-import unchanged.

## AFTER-state

3 file moves via `git mv` (history-preserved):
- `scripts/latency-bench.sh` → `quality/gates/perf/latency-bench.sh` (532 lines; 18-line SIMPLIFY-11 lineage header; `WORKSPACE_ROOT` adjusted to `cd "${SCRIPT_DIR}/../../.."`).
- `scripts/bench_token_economy.py` → `quality/gates/perf/bench_token_economy.py` (Option B underscore; 16-line lineage header; `REPO_ROOT = parents[3]`).
- `scripts/test_bench_token_economy.py` → `quality/gates/perf/test_bench_token_economy.py` (header docstring updated; sibling-import preserved).

Old paths (2 shims + 1 deletion):
- `scripts/latency-bench.sh` — 6-line bash shim.
- `scripts/bench_token_economy.py` — 12-line Python shim (subprocess.run-invokes canonical).
- `scripts/test_bench_token_economy.py` — DELETED (no external callers; pytest auto-discovers).

CI updates per OP-1:
- `.github/workflows/ci.yml:240` — `bash scripts/latency-bench.sh` → `bash quality/gates/perf/latency-bench.sh`.
- `.github/workflows/bench-latency-cron.yml:60` — same edit.

Catalog correction:
- `quality/catalogs/perf-targets.json` row `perf/token-economy-bench` `verifier.script` flipped `bench-token-economy.py` → `bench_token_economy.py` (4-char edit per Wave E pivot rule).

`benchmarks/README.md` — 11-line append documenting the SIMPLIFY-11 move (fixtures stay in place; gates moved to canonical home).

## Runner state at close

```
$ python3 quality/runners/run.py --cadence weekly
summary: 14 PASS, 0 FAIL, 0 PARTIAL, 3 WAIVED, 2 NOT-VERIFIED -> exit=0
$ python3 quality/runners/run.py --cadence pre-push
summary: 11 PASS, 0 FAIL, 0 PARTIAL, 1 WAIVED, 0 NOT-VERIFIED -> exit=0
```

3 perf rows still WAIVED until 2026-07-26 per Wave A design (file-relocate stub; full gate logic v0.12.1 per MIGRATE-03).

## Commit

`1fbf59d` — `feat(p59): perf-dimension file relocate (SIMPLIFY-11 v0.12.0 stub)`. Pre-push hook GREEN; pushed to origin/main.

## SURPRISES.md entries appended

1. Wave E Option B underscore + 4-char catalog correction (combined entry).
2. Wave E REPO_ROOT path arithmetic (parents[3] / `../../..` for new home depth).
