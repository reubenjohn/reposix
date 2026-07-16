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

---

## Wave 2 — item 2

**Scope:** Write `115-UNWAIVE-PATH.md` (inventory + exit route for every currently-waived
doc-alignment/perf row owed to T6), file the pre-push wall-time-creep intake row, commit
+ push. Executed 2026-07-16 by the Wave-2 tree-writer (gsd-executor).

**Row count found vs expected:** live-grepped `quality/catalogs/doc-alignment.json`
(all 396 rows) filtered by `waiver.until == "2026-08-15T00:00:00Z"` → exactly **19** rows.
Cross-checked `quality/catalogs/perf-targets.json` (all 4 rows) for the 2 rows this plan
owns → exactly **2**. Total **21**, matching the plan's expected "~19 + 2" exactly — no
count discrepancy. Full breakdown: `115-UNWAIVE-PATH.md`.

**Discrepancies found (state description, not count):**
1. The plan's shorthand "8 hero-number rows ... WAIVED-MISSING_TEST" undersells reality —
   live data is a 4/4 split: 4 `MISSING_TEST` (latency figures, still-live claims) + 4
   `STALE_DOCS_DRIFT` (89.1%-token claims, already superseded by the ~94% reframe on the
   page). Practical effect: those 4 `STALE_DOCS_DRIFT` rows are item-6 **retire**
   candidates, not bind candidates — the old claim text no longer exists on the page to
   bind a test to.
2. 8 of the 19 doc-alignment rows (the hero group) carry a stale `tracked_in` pointing at
   the 2026-07-12 reality-check audit, not literally "P115 T6" — they were never
   re-tagged when T6 inherited them. Confirmed owed to T6 via `PROGRESS.md` NOW-item-6,
   not via the catalog's own `tracked_in` text.
3. `perf/headline-numbers-cross-check`'s `tracked_in` still says `P97`, not `P115 T6` —
   same class of stale-tag drift as (2). Row confirmed to exist (P90-era) with its
   verifier script confirmed ABSENT from `quality/gates/perf/` (directory listing run,
   no `headline-numbers-cross-check.py` present) — matches the plan's expectation, no
   duplicate row needed.
4. Possible row-ID duplication: `docs/why/token-economy-89-1-percent` and
   `docs/index/token-reduction-89-percent` cite the identical source line
   (`docs/index.md:17`) with near-identical claim text — flagged for item 6 to dedupe,
   not fixed here (catalogs read-only in this lane).

**Intake row filed:** `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md`,
2026-07-16 12:00 entry — third corroborating data point (141s at `d7da383`, 128s the push
before) on the pre-push wall-time creep already tracked by the `2026-07-15 06:35` and
`2026-07-15 17:18` entries; filed as a cross-referencing addition, not a duplicate.

**Owner confirm-retire mid-lane:** none landed during this lane's execution window (see
final commit/push report for the immediately-pre-push state).
