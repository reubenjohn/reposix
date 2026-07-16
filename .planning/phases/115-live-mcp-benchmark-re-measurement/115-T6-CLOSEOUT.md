# P115 T6 — Closeout evidence

Living evidence log for the T6 headline-reframe cleanup. Each wave/item appends its
own section with real artifacts (row states, citations, walk exit codes), not prose
claims. The authoritative record of "what landed" lives here, not in chat.

---

## Wave 1 — item 3 agent-side

**Scope:** Retire the 6 synthetic `count_tokens`-over-fixture doc-alignment rows for
`docs/benchmarks/token-economy.md` (agent-side propose-retire only), and BIND replacement
rows for the LIVE four-axis GitHub-capture figures. Executed 2026-07-16 by the
Wave-1 tree-writer (gsd-executor). Single push: catalog + this closeout + `PROGRESS.md`.

**HARD LIMIT respected:** `confirm-retire` is HUMAN-ONLY and was NOT run. All 6 rows sit
at `RETIRE_PROPOSED` awaiting a human `confirm-retire` from a real TTY. No row was
hand-edited to fake a retired state; the env-guard was never worked around.

### (a) propose-retire — 6 rows, all `RETIRE_PROPOSED`, waivers preserved

Each row transitioned `WAIVED-STALE_DOCS_DRIFT` → `RETIRE_PROPOSED`
(`next_action=RETIRE_FEATURE`, `last_extracted_by=propose-retire-call`). The pre-existing
`until=2026-08-15` waiver carried over (propose-retire does not touch `waiver`), so none
flipped into an un-waived blocking state.

| Row id | Retired figure | Post state |
|---|---|---|
| `docs/benchmarks/token-economy/confluence-reduction-76-percent` | 76.4% | RETIRE_PROPOSED (waived) |
| `docs/benchmarks/token-economy/github-reduction-85-percent` | 85.5% | RETIRE_PROPOSED (waived) |
| `docs/benchmarks/token-economy/jira-real-adapter-not-implemented` | jira-real-adapter | RETIRE_PROPOSED (waived) |
| `docs/benchmarks/token-economy/mcp-mediated-baseline-4883-tokens` | 4,883 | RETIRE_PROPOSED (waived) |
| `docs/benchmarks/token-economy/reposix-baseline-531-tokens` | 531 | RETIRE_PROPOSED (waived) |
| `docs/benchmarks/token-economy/reduction-89-percent` | 89.1% | RETIRE_PROPOSED (waived) |

### (b) BIND — 3 replacement rows for the LIVE four-axis figures (all `BOUND`/GREEN)

Citations verified by hand against the current `docs/benchmarks/token-economy.md`
(read the exact lines; hashes computed by the `bind` verb at bind time). Each row binds
to BOTH the regenerator and its offline test suite (AND-drift watch — stronger than the
single-file binding the retired rows carried).

| New row id | Claim (live figure) | Source citation | Test bindings |
|---|---|---|---|
| `docs/benchmarks/token-economy/output-reduction-94-percent` | ~94.3% fewer output tokens (1,213 vs 21,171 median-of-3) | `docs/benchmarks/token-economy.md:37` | `quality/gates/perf/bench_token_economy.py` + `quality/gates/perf/test_bench_token_economy.py` |
| `docs/benchmarks/token-economy/cost-reduction-75-percent` | ~74.9% cheaper per session ($0.2076 vs $0.8278 median-of-3) | `docs/benchmarks/token-economy.md:40` | `quality/gates/perf/bench_token_economy.py` + `quality/gates/perf/test_bench_token_economy.py` |
| `docs/benchmarks/token-economy/live-github-capture-methodology` | live GitHub-capture Claude Code JSONL session-usage records, median-of-3 per arm, offline-reproducible | `docs/benchmarks/token-economy.md:8-13` | `quality/gates/perf/bench_token_economy.py` + `quality/gates/perf/test_bench_token_economy.py` |

**Binding rationale (why a test that asserts 90.0% backs a 94.3% doc figure):** the tests
prove the *regenerator is correct*, not a hardcoded value.
`test_compute_arm_medians_and_reductions` asserts the per-axis median-of-3 reduction math
across all four axes (output/cache_create/input_context/cost); `test_main_offline_regenerates_doc_from_captures`
asserts `main(['--offline'])` reads the captures and regenerates the doc deterministically
(byte-identical second run). The published 94.3% / 74.9% figures are simply that verified
regenerator's output over the *real committed* `benchmarks/captures/*.json`. This mirrors
the pattern the retired rows used ("`bench_token_economy.py` … IS the regenerator").

### Verify-against-reality artifacts

- `python3 -m pytest quality/gates/perf/test_bench_token_economy.py -q` → **9 passed** (offline; no `ANTHROPIC_API_KEY`, no network).
- `python3 quality/gates/perf/bench_token_economy.py --offline` → doc regenerates **byte-for-byte** (empty `git diff` on `token-economy.md`), confirming the "offline-reproducible" methodology binding is truthful.
- Catalog delta HEAD→working: **+3 rows, 0 removed**; `claims_bound` 263→266; `alignment_ratio` 0.7827→0.7847; `claims_retired` unchanged at 57 (correct — `RETIRE_PROPOSED` ≠ `RETIRE_CONFIRMED`).

### (c) Pre-push walk verification

`bash quality/gates/docs-alignment/walk.sh` (GRADE mode, read-only /tmp copy) → **rc=0**,
**zero blocking lines**. The 6 retired rows print `WAIVED-RETIRE_PROPOSED` (loud, non-blocking);
the 3 new rows are clean `BOUND` (absent from walk output = no drift). No new blocking
state was created, so the pre-authorized `waive` fix was not *required*.

### Waives applied (reason-refresh only, not a new deferral)

The 6 pre-existing waivers still covered `RETIRE_PROPOSED`, but their `reason` text was
stale (it described the rebind as a *future* T6 job — now done). Re-`waive`d all 6 with the
pre-authorized values (`until=2026-08-15T00:00:00Z`, `tracked_in="P115 T6 (115-UNWAIVE-PATH.md)"`,
precedent T5 `c9c2aee`) and an accurate reason: replacement rows already BOUND, only
HUMAN-ONLY confirm-retire remains. Walk re-run after the refresh: still **rc=0**.

### Human relay — action pending

`confirm-retire` (HUMAN-ONLY, TTY-guarded) is still owed on these 6 rows **plus** the 2
concepts-page rows `9a2b6f1` marked `RETIRE_PROPOSED` (`token-baseline-mcp-4883`,
`token-baseline-reposix-531`). Batch all 8 into one manager `confirm-retire` ask
(PROGRESS.md item 3 / manager w1:p7). Command shape for the human, per row:

```
target/release/reposix-quality doc-alignment confirm-retire --row-id <ID>
```

(run from a real terminal — refuses under `CLAUDE_AGENT_CONTEXT` / non-TTY stdin).
