---
phase: 60-docs-build-migration
plan: 05
subsystem: quality-gates
tags: [simplify-10, pre-push, hook, runner, cred-hygiene]
requires:
  - quality/runners/run.py
  - quality/gates/structure/cred-hygiene.sh
provides:
  - scripts/hooks/pre-push one-liner (SIMPLIFY-10 closure)
affects:
  - every developer push (the hook is what runs on `git push`)
key-files:
  modified:
    - scripts/hooks/pre-push
  unchanged:
    - scripts/hooks/test-pre-push.sh
    - scripts/install-hooks.sh
decisions:
  - "NO PIVOT: warm-cache runner time 5.3s well under 60s budget"
  - "cred-hygiene stays IN the hook body because the runner does not pipe stdin"
  - "hook body 10 lines (cap was <=30); 40 lines total including header"
metrics:
  warm_cache_runner_time_first: 7.0s
  warm_cache_runner_time_second: 5.3s
  hook_body_lines: 10
  hook_total_lines: 40
  tests_passed: 6/6
  duration_minutes: 8
  completed_date: "2026-04-27"
---

# Phase 60 Plan 05: SIMPLIFY-10 pre-push hook one-liner

## One-liner

`scripts/hooks/pre-push` body collapsed to one runner invocation + the cred-hygiene fail-fast wrapper; SIMPLIFY-10 closed.

## Profile

```
{ time python3 quality/runners/run.py --cadence pre-push; }
# Run 1: 7.02s wall (cold artifact dir)
# Run 2: 5.31s wall (warm)
```

19 PASS rows; cargo clippy hot in 0.23s thanks to incremental cache. Decision: **NO PIVOT** -- carve-out unnecessary.

## Hook shape

Header (~25 lines) explaining lineage + bypass + install. Body proper (10 lines):

```bash
set -euo pipefail
readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

stdin_buf=$(cat)
if [[ -n "$stdin_buf" ]]; then
  if ! printf '%s\n' "$stdin_buf" | bash "$REPO_ROOT/quality/gates/structure/cred-hygiene.sh"; then
    exit 1
  fi
fi

exec python3 "$REPO_ROOT/quality/runners/run.py" --cadence pre-push
```

The pre-Wave-E hook (229 lines, ~85 body lines) chained 6 distinct verifiers + the parallel-migration runner block. Every surface is now a catalog row graded by the runner; the cred-hygiene wrapper is the only fail-fast carve-out (P0 security gate; runner does not pipe stdin).

## Test harness

`scripts/hooks/test-pre-push.sh` ran 6/6 PASS against the new hook (after commit so `git reset --hard` in the harness preserves the new hook in HEAD). No edits to the test harness were needed.

```
✓ clean commit passes (exit=0)
✓ ATATT3 token rejected (exit=1)
✓ Bearer ATATT3 rejected (exit=1)
✓ ghp_ GitHub PAT rejected (exit=1)
✓ github_pat_ rejected (exit=1)
✓ hook self-scan exclusion honored (exit=0)
```

## Files

- **modified:** `scripts/hooks/pre-push` (229 → 40 lines; -189 lines)
- **untouched (per plan):** `scripts/hooks/test-pre-push.sh`, `scripts/install-hooks.sh`

## Commits

- `f00affc` -- refactor(p60): pre-push hook one-liner per SIMPLIFY-10

## Self-Check: PASSED

- File `scripts/hooks/pre-push` FOUND, executable, 40 lines.
- Commit `f00affc` FOUND in git log.
- Test harness 6/6 PASS.
- `scripts/install-hooks.sh` diff bytes = 0.
- Banned word `replace` absent.
- `quality/runners/run.py` referenced in hook body.
- `cred-hygiene.sh` referenced in hook body.
