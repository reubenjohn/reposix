# 115-UNWAIVE-PATH — inventory + exit route for every row T6 owes

Enumerated directly from the live catalogs (`quality/catalogs/doc-alignment.json`,
`quality/catalogs/perf-targets.json`) on 2026-07-16, T6 Wave 2 item 2. Every row below
was re-read from disk at inventory time — this is not a re-statement of the plan's
expected count, it is the actual grep result, cross-checked against the expected count.

## Result vs expected count

**Expected:** ~19 doc-alignment rows + 2 perf rows.
**Found:** exactly **19** doc-alignment rows waived until `2026-08-15T00:00:00Z` that
trace to this T6 lineage, + exactly **2** perf-targets rows this plan is responsible for
un-waiving. Total **21**, matching the expected count exactly. No discrepancy in row
*count*. One discrepancy in row *state description* is flagged below (§ Group A note) —
the plan's shorthand "WAIVED-MISSING_TEST" undersells that 4 of the 8 hero rows are
actually `STALE_DOCS_DRIFT`, a stronger signal.

Method: `grep -n '"tracked_in"' quality/catalogs/*.json` for `"P115 T6"` /
`"115-UNWAIVE-PATH"` literal text found 11 doc-alignment rows + 1 perf row directly
tagged. The remaining 8 doc-alignment rows (Group A below) carry an OLDER `tracked_in`
pointing at `.planning/milestones/audits/2026-07-12-reality-check.md §5 Q1` — they were
not re-tagged when T6 inherited them. A second pass filtering all doc-alignment rows by
`waiver.until == "2026-08-15"` (regardless of `tracked_in` text) surfaced all 19,
confirming the plan's expected inventory is correct even though 8 of the 19 rows don't
literally say "P115 T6" in their catalog `tracked_in` field. This is itself worth noting
(see Noticing below) — future re-inventories should filter by waiver expiry + source
file, not by `tracked_in` text, or they will silently miss these 8.

---

## Group A — 8 hero-number rows (docs/index.md + README.md)

All waived until **2026-08-15T00:00:00Z**. `tracked_in` (as written in the catalog) is
`.planning/milestones/audits/2026-07-12-reality-check.md §5 Q1`, not literally "P115 T6"
— but the plan's charter and `PROGRESS.md` NOW-item-1/6 explicitly assign these 8 to T6
item 6 for un-waive, so they are owed by this phase regardless of the catalog's stale
`tracked_in` label.

| Row id | Claim | Actual state (live) | Exit route | Who acts |
|---|---|---|---|---|
| `docs/index/latency-24ms-cold-init` | 27ms cold init (sim) | `MISSING_TEST` | T6 item 6: `headline-numbers-cross-check.py` binds the 27ms figure to `latency-bench.sh`'s `SIM_INIT_MS` output | agent (item 6) |
| `docs/index/latency-8ms-read` | 8ms cached read (sim) | `MISSING_TEST` | T6 item 6: same cross-check gate binds the 8ms figure | agent (item 6) |
| `docs/index/token-reduction-89-percent` | 89.1% token reduction | `STALE_DOCS_DRIFT` | Content already superseded — docs/index.md:17 now reads "~94% fewer output tokens", not 89.1%. Exit is a doc-alignment **re-grade/retire**, not a test-bind (the claim itself is gone from the page) | agent (item 6, retire this row rather than bind it) |
| `latency-hero-24ms-mismatch` | Homepage hero 27ms cold init | `MISSING_TEST` | T6 item 6 cross-check gate | agent (item 6) |
| `README-md/token-89-percent` | Input-context reduction 89.1% | `STALE_DOCS_DRIFT` | Content already superseded — README.md:27 now reads "~94% fewer output tokens". Retire, not bind | agent (item 6, retire) |
| `README-md/latency-8ms` | 8ms cache read | `STALE_DOCS_DRIFT` | Structure shifted (an italic caveat line was inserted above the bullets in the hero reframe); re-cite to the current line, then bind via cross-check gate | agent (item 6, re-cite + bind) |
| `README-md/init-24ms` | 27ms cold bootstrap | `MISSING_TEST` | T6 item 6 cross-check gate | agent (item 6) |
| `docs/why/token-economy-89-1-percent` | 89.1% token reduction | `STALE_DOCS_DRIFT` | Content already superseded (same docs/index.md:17 citation as `token-reduction-89-percent` above — this row may be a near-duplicate of that one, both citing the same line; worth deduping when item 6 touches this group) | agent (item 6, retire) |

**Group A note (discrepancy vs plan wording):** the plan describes this group as
"8 hero-number rows ... WAIVED-MISSING_TEST until 2026-08-15." Live data shows a
**4/4 split**: 4 rows are `MISSING_TEST` (the latency figures — content unchanged,
just never asserted) and 4 rows are `STALE_DOCS_DRIFT` (the 89.1% token figures — content
has ALREADY moved to ~94% under the T5/T6 reframe, so these rows are stale references to
copy that no longer exists on the page). Verified by reading current `docs/index.md:17-18`
and `README.md:25-27` directly: the 8ms/27ms latency claims are still live and accurate
(pending re-measurement); the 89.1% token claims are gone, replaced by ~94%. Practical
consequence for item 6: the 4 `STALE_DOCS_DRIFT` rows should be **retired** (the old claim
no longer exists to bind), not test-bound — binding a test to a claim the page no longer
makes would be a no-op. Only the 4 `MISSING_TEST` latency rows are real bind targets for
`headline-numbers-cross-check.py`.

**Also noticed:** `docs/why/token-economy-89-1-percent` and
`docs/index/token-reduction-89-percent` cite the *identical* source location
(`docs/index.md:17`) with near-identical claim text. This looks like a row-ID duplication
left over from an earlier catalog-authoring pass rather than two distinct claims. Not
fixed here (catalogs are read-only in this lane) — flagged for item 6 to dedupe rather
than carry two rows for one sentence.

---

## Group B — 6 docs/benchmarks/token-economy/\* rows (synthetic figures, RETIRE_PROPOSED)

All waived until **2026-08-15T00:00:00Z**, `tracked_in: "P115 T6 (115-UNWAIVE-PATH.md)"`
verbatim in the catalog. **State transition confirmed:** these 6 rows were
`WAIVED-STALE_DOCS_DRIFT` before Wave 1 item 3 (per `115-T6-CLOSEOUT.md` § Wave 1 —
item 3(a): "Each row transitioned `WAIVED-STALE_DOCS_DRIFT` → `RETIRE_PROPOSED`
(`next_action=RETIRE_FEATURE`, `last_extracted_by=propose-retire-call`)"), landed in
commit `d7da383`. They are currently `RETIRE_PROPOSED`, not `STALE_DOCS_DRIFT`.

**CRITICAL — exit route is HUMAN-ONLY, not test-rebind.** `confirm-retire` requires a
real TTY (refuses under `CLAUDE_AGENT_CONTEXT` / non-TTY stdin) and is explicitly
HUMAN-ONLY per the tool's own guard. The live four-axis replacement rows are already
`BOUND`/GREEN (see below) — the only remaining action on these 6 rows is the owner
running `confirm-retire` from a real terminal, batched with the 2 Group C
`RETIRE_PROPOSED` rows into one ask (manager w1:p7, per `PROGRESS.md` item 3).

| Row id | Retired figure | Live state | Exit route | Who acts |
|---|---|---|---|---|
| `docs/benchmarks/token-economy/confluence-reduction-76-percent` | 76.4% | `RETIRE_PROPOSED` | HUMAN confirm-retire (`target/release/reposix-quality doc-alignment confirm-retire --row-id <ID>`, real TTY) | **human (owner)** |
| `docs/benchmarks/token-economy/github-reduction-85-percent` | 85.5% | `RETIRE_PROPOSED` | HUMAN confirm-retire | **human (owner)** |
| `docs/benchmarks/token-economy/jira-real-adapter-not-implemented` | jira-real-adapter placeholder | `RETIRE_PROPOSED` | HUMAN confirm-retire | **human (owner)** |
| `docs/benchmarks/token-economy/mcp-mediated-baseline-4883-tokens` | 4,883 tokens | `RETIRE_PROPOSED` | HUMAN confirm-retire | **human (owner)** |
| `docs/benchmarks/token-economy/reposix-baseline-531-tokens` | 531 tokens | `RETIRE_PROPOSED` | HUMAN confirm-retire | **human (owner)** |
| `docs/benchmarks/token-economy/reduction-89-percent` | 89.1% | `RETIRE_PROPOSED` | HUMAN confirm-retire | **human (owner)** |

**Already-BOUND replacements** (context, not part of the waived inventory — listed so
the reader doesn't mistake these for still-open rows): `output-reduction-94-percent`,
`cost-reduction-75-percent`, `live-github-capture-methodology` — all `last_verdict:
BOUND`, no waiver. These are the live four-axis figures that make the 6 retiring rows
safe to retire (the claim they made has a verified replacement, not a gap).

---

## Group C — 5 rows from `c9c2aee` (post-refresh time-boxed waivers)

All waived until **2026-08-15T00:00:00Z**, `tracked_in: "P115 T6 (115-UNWAIVE-PATH.md)"`
verbatim.

| Row id | Live state | Claim | Exit route | Who acts |
|---|---|---|---|---|
| `docs/index/mcp-loop-4883-tokens` | `MISSING_TEST` | MCP loop ~21k output tokens (live) | T6 item 6: bind to `headline-numbers-cross-check.py` against the captures-computed GitHub-MCP median (~21,171) | agent (item 6) |
| `docs/index/reposix-loop-531-tokens` | `MISSING_TEST` | reposix loop ~1.2k output tokens (live) | T6 item 6: bind to same cross-check gate against the captures-computed reposix median (~1,213) | agent (item 6) |
| `latency-cached-read-8ms` | `MISSING_TEST` | reposix cached read = 8ms (sim), comparison-page mirror of the canonical docs/index figure | T6 item 6 cross-check gate — **note:** canonical `docs/benchmarks/latency.md` now reads 6ms get / 7ms list vs the 8ms hero prose; item 6 must reconcile this mismatch, not just bind a test to the (possibly wrong) 8ms figure | agent (item 6) |
| `token-baseline-mcp-4883` | `RETIRE_PROPOSED` | MCP synthesized fixture = 4,883 tokens / 89.1% reduction triple | HUMAN confirm-retire (same batch as Group B, per `9a2b6f1` re-grade) | **human (owner)** |
| `token-baseline-reposix-531` | `RETIRE_PROPOSED` | reposix loop = 531 tokens (simulator) | HUMAN confirm-retire (same batch) | **human (owner)** |

---

## Group D — 2 perf rows

| Row id | Status | Waive expiry | tracked_in (as written) | Exit route | Who acts |
|---|---|---|---|---|---|
| `perf/token-economy-bench` | `WAIVED` | 2026-09-15T00:00:00Z | `"P115 T6 (115-UNWAIVE-PATH.md) -> a future code phase adds the headline assertion"` | T6 item 6 adds the ~94% output-token-reduction assertion to `quality/gates/perf/bench_token_economy.py` (currently regenerates + emits the four-axis medians but `main()` returns 0 unconditionally — a wrong-but-present headline would not fail the gate) | agent (item 6) |
| `perf/headline-numbers-cross-check` | `WAIVED` | 2026-09-15T00:00:00Z | `"P97 (Good-to-haves polish + milestone close, launch-readiness slot) -- perf-dimension full implementation incl. this row's missing verifier script"` | Row **already exists** (P90-era, confirmed dangling-verifier: `quality/gates/perf/headline-numbers-cross-check.py` confirmed absent from `quality/gates/perf/` — verified by directory listing, no such file). T6 item 6 writes the missing verifier script; do NOT create a duplicate row — bind/un-waive this existing one | agent (item 6) |

**Group D note (tracked_in discrepancy):** `perf/headline-numbers-cross-check`'s catalog
`tracked_in` field officially still says `P97`, not `P115 T6` — it was never re-tagged
when T6 absorbed the un-waive obligation. The plan (and `PROGRESS.md` item 6) explicitly
assigns writing the missing verifier script to T6 item 6, so functionally this row is
owed here even though the catalog metadata hasn't caught up. Same class of drift as
Group A's stale `tracked_in`. Not fixed here (catalogs read-only in this lane) — item 6
should re-`waive`/re-tag this row's `tracked_in` to `P115 T6` when it lands the script,
so a future re-inventory doesn't have to cross-reference PROGRESS.md to find it.

---

## Summary table (all 21 rows, one line each)

| # | Row id | State | Expiry | Exit route |
|---|---|---|---|---|
| 1 | `docs/index/latency-24ms-cold-init` | MISSING_TEST | 2026-08-15 | item 6 bind |
| 2 | `docs/index/latency-8ms-read` | MISSING_TEST | 2026-08-15 | item 6 bind |
| 3 | `docs/index/token-reduction-89-percent` | STALE_DOCS_DRIFT | 2026-08-15 | item 6 retire (claim gone) |
| 4 | `latency-hero-24ms-mismatch` | MISSING_TEST | 2026-08-15 | item 6 bind |
| 5 | `README-md/token-89-percent` | STALE_DOCS_DRIFT | 2026-08-15 | item 6 retire (claim gone) |
| 6 | `README-md/latency-8ms` | STALE_DOCS_DRIFT | 2026-08-15 | item 6 re-cite + bind |
| 7 | `README-md/init-24ms` | MISSING_TEST | 2026-08-15 | item 6 bind |
| 8 | `docs/why/token-economy-89-1-percent` | STALE_DOCS_DRIFT | 2026-08-15 | item 6 retire (claim gone; likely dup of #3) |
| 9 | `docs/benchmarks/token-economy/confluence-reduction-76-percent` | RETIRE_PROPOSED | 2026-08-15 | HUMAN confirm-retire |
| 10 | `docs/benchmarks/token-economy/github-reduction-85-percent` | RETIRE_PROPOSED | 2026-08-15 | HUMAN confirm-retire |
| 11 | `docs/benchmarks/token-economy/jira-real-adapter-not-implemented` | RETIRE_PROPOSED | 2026-08-15 | HUMAN confirm-retire |
| 12 | `docs/benchmarks/token-economy/mcp-mediated-baseline-4883-tokens` | RETIRE_PROPOSED | 2026-08-15 | HUMAN confirm-retire |
| 13 | `docs/benchmarks/token-economy/reposix-baseline-531-tokens` | RETIRE_PROPOSED | 2026-08-15 | HUMAN confirm-retire |
| 14 | `docs/benchmarks/token-economy/reduction-89-percent` | RETIRE_PROPOSED | 2026-08-15 | HUMAN confirm-retire |
| 15 | `docs/index/mcp-loop-4883-tokens` | MISSING_TEST | 2026-08-15 | item 6 bind |
| 16 | `docs/index/reposix-loop-531-tokens` | MISSING_TEST | 2026-08-15 | item 6 bind |
| 17 | `latency-cached-read-8ms` | MISSING_TEST | 2026-08-15 | item 6 bind (+ reconcile 8ms vs 6/7ms) |
| 18 | `token-baseline-mcp-4883` | RETIRE_PROPOSED | 2026-08-15 | HUMAN confirm-retire |
| 19 | `token-baseline-reposix-531` | RETIRE_PROPOSED | 2026-08-15 | HUMAN confirm-retire |
| 20 | `perf/token-economy-bench` | WAIVED | 2026-09-15 | item 6 adds ~94% assertion |
| 21 | `perf/headline-numbers-cross-check` | WAIVED | 2026-09-15 | item 6 writes missing verifier script |

**Tally:** 8 rows need HUMAN confirm-retire (rows 9-14, 18-19 = 8 rows across Groups B+C
— batch into one manager ask, per `115-T6-CLOSEOUT.md` § Human relay). 4 rows need item 6
to retire (claim already superseded, rows 3, 5, 6\*, 8). 9 rows need item 6 to bind a test
(rows 1, 2, 4, 6\*, 7, 15, 16, 17 — note row 6 needs BOTH a re-cite and a bind since its
citation line shifted). 2 rows need item 6 to add/write an assertion or verifier script
(rows 20, 21).

\* Row 6 (`README-md/latency-8ms`) appears in both counts: its citation is stale (line
shifted) but the underlying 8ms claim is still live, so it needs a re-cite THEN a bind,
not a retire.

---

## Wave 2 item 6a — status update (2026-07-16)

Rows resolved by lane 6a (evidence: `115-T6-CLOSEOUT.md` § Wave 2 — item 6a):

| Row id | Was | Now | How |
|---|---|---|---|
| #2 `docs/index/latency-8ms-read` | WAIVED-MISSING_TEST | **BOUND, no waiver** | claim `8 ms`→`6 ms`; bound to `headline-numbers-cross-check.py` + its test; `unwaive`d |
| #17 `latency-cached-read-8ms` | WAIVED-MISSING_TEST | **BOUND, no waiver** | claim `8 ms`→`6 ms`; bound to the same gate+test; `unwaive`d |
| #21 `perf/headline-numbers-cross-check` | WAIVED (dangling verifier) | **PASS, no waiver** | wrote `headline-numbers-cross-check.py` + test; minted via `run.py --cadence weekly --persist`; `tracked_in`→`P115 T6`; `transport_claim:false`, `coverage_kind:mechanical` |

Also re-cited (line-shift hash refresh, not in the 21-row inventory — were already BOUND):
`docs/index/tested-three-backends` (86-91), `docs/index/soft-threshold-24ms` (93).

**Exit-route correction for 6b (rows #1 `docs/index/latency-24ms-cold-init` and #4
`latency-hero-24ms-mismatch`):** 6a's `docs/index.md:18` edit re-graded these two WAIVED
cold-init rows `MISSING_TEST → STALE_DOCS_DRIFT` (still non-blocking, waivers untouched).
Their exit route as written ("bind the 27 ms figure to the cross-check gate") **is no longer
valid**: `headline-numbers-cross-check.py` deliberately does NOT check cold-init — canonical
`latency.md` init is `278 ms`, not the hero `27 ms`, so binding the 27 ms hero figure to
this gate would be false. 6b must reconcile `27 ms → 278 ms` (or the interim framing)
separately, or extend the gate to cover cold-init, before those rows can bind. Row #7
`README-md/init-24ms` (README.md:26) was NOT touched by 6a and is unchanged.

Remaining for 6b: rows #3, #5, #6, #8 (STALE_DOCS_DRIFT hero retire candidates), #15/#16
(mcp-loop/reposix-loop), #20 (`perf/token-economy-bench` assertion), the 8 HUMAN
confirm-retire rows, and the cold-init reconciliation above.

## Wave 2 — item 6b (FINAL — closes the agent-side T6 un-waive path)

Executed 2026-07-16 by the Wave-2 item-6b tree-writer. Evidence: `115-T6-CLOSEOUT.md`
§ Wave 2 — item 6b. Every row from the 21-row inventory is now at its terminal
agent-side state; the ONLY remaining waived rows are the human-confirm-retire batch.

**Cold-init reconcile (Task 1):** hero 27 ms → canonical **278 ms** (docs/benchmarks/
latency.md `reposix init` cold sim, CI-canonical; the 24/27 ms figures were superseded
dev-machine artifacts per latency.md § Provenance — SAME operation, different env, so
"fix to canonical"). Extended `headline-numbers-cross-check.py` with a cold-init axis +
4 hero cold-init claims + 2 absolute loop-figure claims.

| Row | Was | Now |
|---|---|---|
| #1 `docs/index/latency-24ms-cold-init` | WAIVED-MISSING_TEST→STALE_DOCS_DRIFT | **BOUND** (278 ms, gate+test); unwaived |
| #4 `latency-hero-24ms-mismatch` | WAIVED-STALE_DOCS_DRIFT | **BOUND** (278 ms, gate+test); unwaived |
| #7 `README-md/init-24ms` | WAIVED-MISSING_TEST | **BOUND** (278 ms, gate+test); unwaived |
| #15 `docs/index/mcp-loop-4883-tokens` | WAIVED-MISSING_TEST | **BOUND** (~21k live, gate parses MCP median 21,171); unwaived |
| #16 `docs/index/reposix-loop-531-tokens` | WAIVED-MISSING_TEST | **BOUND** (~1.2k live, gate parses reposix median 1,213); unwaived |
| #6 `README-md/latency-8ms` | WAIVED-STALE_DOCS_DRIFT | **BOUND** — re-cited README.md:23→:25, 8→6 ms (gate+test); unwaived |
| #3 `docs/index/token-reduction-89-percent` | WAIVED-STALE_DOCS_DRIFT | **RETIRE_PROPOSED** (waiver reason refreshed→P115 T6); HUMAN confirm-retire |
| #8 `docs/why/token-economy-89-1-percent` | WAIVED-STALE_DOCS_DRIFT | **RETIRE_PROPOSED** (dup of #3, same docs/index.md:17); HUMAN confirm-retire |
| #5 `README-md/token-89-percent` | WAIVED-STALE_DOCS_DRIFT | **RETIRE_PROPOSED** (reason refreshed→P115 T6); HUMAN confirm-retire |
| #20 `perf/token-economy-bench` | WAIVED (2026-09-15) | **PASS** — bench main() now asserts ~94.3% ±1.0pp; unwaived + minted |
| #21 `perf/headline-numbers-cross-check` | (6a) PASS | PASS — gate extended (cold-init + loop), still PASS |

**Re-cite side-effects (my index.md 27→278 / README 27→278 / filesystem 8→6 edits shifted
cited bytes):** re-bound `docs/index/latency-8ms-read` (L18, claim unchanged), `docs/index/
soft-threshold-24ms` (L93, claim→278, +cross-check binding), `docs/index/bootstrap-latency-24ms`
(L134, claim→278), `filesystem-layer/blob-lazy-first-cat` (L42, claim unchanged) — all stay BOUND.

**Dedupe (Task 3):** #3 and #8 cite the IDENTICAL docs/index.md:17 line (same source_hash) —
a true duplicate (the `docs/why/*` id was mis-attributed to docs/index). Both retired together;
no distinct claim lost.

**Task 5:** `bench_token_economy.py` main() now calls `_assert_headline_reduction()` (94.3% ±1.0pp,
computed 94.27%); un-waived + minted PASS via `run.py --cadence weekly --persist` (only
perf-targets.json flipped).

**Task 6 (validate-only flips):** BENIGN stale state, not regressions — `code/shell-coverage`
on-disk FAIL→fresh PASS (kcov 70s, above floor; stale FAIL from a prior sub-floor run) and
`security/cargo-audit-rustsec-posture` NOT-VERIFIED→PASS. Persisted surgically via
`run.py --cadence pre-push --persist` (diffs = status + timestamp only; 61 PASS, 0 FAIL).

**Task 7 (non-hero 8 ms):** `mental-model-in-60-seconds.md:69` 8→6 ms + 24→278 ms (no bound
row on L69, safe); `how-it-works/filesystem-layer.md:42` 8→6 ms (re-cited); `docs/concepts/
reposix-vs-mcp-and-sdks.md:15` 24→278 ms cold init (hero surface, no bound row). LEFT:
`docs/reference/simulator.md:18` (explicit "on the dev host" framing — a legitimately
different env from CI-canonical, not a hero surface, no bound row) and `mental-model:21`
(bound to 3 STALE_TEST_DRIFT rows — filed GTH-V15-33, out of scope to avoid flipping
non-blocking drift to blocking).

### Remaining waived (expected count = 11, ALL RETIRE_PROPOSED = the human-confirm-retire batch)

The 8 pre-existing (Groups B + C: token-economy/* ×6, token-baseline-{mcp-4883,reposix-531})
+ my 3 new 89.1% rows (#3, #5, #8) = **11 rows**, all `WAIVED-RETIRE_PROPOSED`, all owed a
single HUMAN-ONLY `confirm-retire` from a real TTY. NO other T6-owned row remains waived
(`perf/token-economy-bench` un-waived; `perf/latency-bench` stays waited-until-2026-09-15 but
is NOT a T6 obligation). Walk rc=0, headline gate exit 0, perf pytest 26/26.

## Cross-references

- Group B state transition + human-relay batch: `115-T6-CLOSEOUT.md` § Wave 1 — item 3.
- Item 6 charter (un-waive obligations): `PROGRESS.md` § NOW, item 4/6, and
  `.planning/SESSION-HANDOVER.md` §5.
- `perf/token-economy-bench` waiver text names this file as the un-waive path
  (`quality/catalogs/perf-targets.json`, row `perf/token-economy-bench`).
