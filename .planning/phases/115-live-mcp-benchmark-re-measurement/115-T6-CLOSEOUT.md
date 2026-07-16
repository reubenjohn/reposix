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

---

## Wave 2 — item 5

**Scope:** Regen-clobber guard — `emit-markdown.sh` must not overwrite the CI-canonical
sections of `docs/benchmarks/latency.md`. Executed 2026-07-16 by the Wave-2 tree-writer
(gsd-executor).

**Reality found before fixing:** `quality/gates/perf/latency-bench/emit-markdown.sh`
always does an unconditional `cat > "$OUT" <<MARKDOWN …`, stamping its own fixed template
over whatever is at `$OUT`. That template has no "Corrected" banner, no "Provenance &
methodology" section, no "PATCH figures — known caveat" section — all hand-curated
content the current `docs/benchmarks/latency.md` carries on top of the CI-measured table.
The doc's own "Reproduce" prose claimed "it does not overwrite the CI-sourced
real-backend figures unless run with credentials for those backends" — that claim was
**false**: any bare local run (sim-only, no creds) would blank the github/confluence/jira
cells and delete the curated sections outright. Noted as a lying-doc-claim finding (see
Noticing below); fixed by making the guard's refusal the actual behavior instead of
rewriting prose to match a bug.

**Guard design chosen:** refuse-and-explain, gated on marker presence, not a merge.
1. New factored module `quality/gates/perf/latency-bench/regen-guard.sh` (follows this
   dir's existing per-concern layout: `lib.sh`/`sim.sh`/`github.sh`/…) defines
   `regen_guard_check <out_path>`: returns 0 if `$out_path` doesn't exist or has no
   `reposix:regen-guard:protected-begin` marker; else refuses (rc=1, teaching error to
   stderr) unless `REPOSIX_LATENCY_BENCH_ALLOW_CANONICAL_OVERWRITE=1` is set.
2. `emit-markdown.sh` sources it and calls `regen_guard_check "$OUT" || exit 1` as its
   first executable lines (fail fast, before any cell formatting work).
3. `docs/benchmarks/latency.md` carries the marker comment (both `-begin`/`-end` lines
   together) **at end-of-file, not wrapping the content above** — an earlier draft that
   wrapped the whole body (marker inserted right after frontmatter) shifted every line
   below by 9, which tripped `STALE_DOCS_DRIFT` on all 14 doc-alignment rows bound into
   this file's table/soft-threshold sections (citations are line-anchored). Caught by
   running `quality/gates/docs-alignment/walk.sh` before committing; fixed by moving the
   marker to end-of-file only — zero lines shift above it, walk re-run clean (rc=0, no
   `latency.md` rows in output).
4. `quality/gates/perf/latency-bench.sh`'s `OUT` var made overridable
   (`OUT="${OUT:-...}"`) so the guard's own recovery command #1 (`OUT=/tmp/... bash
   quality/gates/perf/latency-bench.sh`) is truthful, not aspirational.
5. Teaching error covers all three Rust-compiler-grade UX elements: **what** was
   protected (names the marker + the curated sections at risk), **why** (CI-reviewed
   numbers vs. unreviewed laptop noise), **recovery** (two copy-paste commands: preview
   via `OUT=` redirect, or explicit override + manual restore of the curated prose).

**Verify against reality (all trials against `/tmp` destinations — the real
`docs/benchmarks/latency.md` was only ever touched via reviewed `Edit` calls, never by
running the generator against it):**
- `regen_guard_check` unit-level (12 assertions, `regen-guard.selftest.sh`): no-file →
  safe; unmarked file → safe; marked file no override → refuse + file byte-identical
  before/after (sha256 compared) + all 4 teaching-content greps pass; marked file +
  override → safe; **the real committed `docs/benchmarks/latency.md` itself trips the
  guard** (rc=1) — confirms the marker is live in the shipped file, not just a synthetic
  fixture. `bash quality/gates/perf/latency-bench/regen-guard.selftest.sh` → 12 passed, 0
  failed.
- Full `emit-markdown.sh` sourced directly (real script, stubbed timing vars, no
  cargo/sim needed): (a) fresh `/tmp` destination → regenerates normally, rc=0; (b)
  `/tmp` copy of the real `docs/benchmarks/latency.md` as destination → refuses, rc=1,
  sha256 identical before/after; (c) same + explicit override env → proceeds, rc=0,
  content changes. `git status --short -- docs/benchmarks/latency.md` after all trials
  shows only this lane's own intentional edits — no generator output ever landed there.
- `bash quality/gates/docs-alignment/walk.sh` → rc=0 (post end-of-file-marker fix; the 14
  previously-drifted `docs/benchmarks/latency/*` rows are absent from the output).
- `bash scripts/banned-words-lint.sh --all` → PASS. `bash
  quality/gates/docs-build/mkdocs-strict.sh` → OK. `bash
  quality/gates/docs-build/mermaid-renders.sh` → 7/7 OK.

**Test added:** `quality/gates/perf/latency-bench/regen-guard.selftest.sh` (new,
3874 chars), following the `quality/gates/structure/file-size-limits.selftest.sh`
convention (throwaway `/tmp` fixtures, pass/fail counters, non-zero exit on any
regression) — not wired into the kcov shell-coverage harness (neither is its
precedent file), so it sits in the same "standalone, run-on-demand" bucket as that
exemplar; not a new gap.

**Files touched:** `quality/gates/perf/latency-bench/regen-guard.sh` (new, 4216 chars),
`quality/gates/perf/latency-bench/regen-guard.selftest.sh` (new, 3874 chars),
`quality/gates/perf/latency-bench/emit-markdown.sh` (+7/-1 lines: source + guard call),
`quality/gates/perf/latency-bench.sh` (+4/-1: overridable `OUT`), `docs/benchmarks/
latency.md` (+33/-8: guard marker + updated Reproduce prose; zero bytes changed above
line 128, confirmed by `git diff` — the table/Provenance/PATCH-caveat/soft-threshold
sections are untouched).

**Noticing:**
1. **Lying doc claim (fixed):** the pre-existing Reproduce prose asserted the script
   "does not overwrite the CI-sourced real-backend figures unless run with credentials"
   — false for the shipped code (see "Reality found" above). Now true, because the guard
   enforces it instead of the prose merely claiming it.
2. **Line-anchored doc-alignment citations are a sharp edge for ANY future doc edit
   in this file** (not unique to this task): inserting content before a bound section
   silently trips `STALE_DOCS_DRIFT` on every citation below the insertion point, even
   when the actual cited *values* didn't change. Worth a one-line callout in
   `quality/CLAUDE.md` § docs-alignment for the next editor of a heavily-bound doc —
   filed as a GOOD-TO-HAVE rather than fixed here (touches a shared doc outside this
   item's scope; `.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md`).
3. The guard is a refuse-and-explain shim, not a merge: even the "override" path still
   can't make `emit-markdown.sh`'s template reproduce the Provenance/PATCH-caveat prose.
   A future item (not this one) could teach the template to preserve those sections by
   splicing rather than full-file replace — out of scope here, flagged in the script's
   own header comment so it isn't lost.

**Owner confirm-retire mid-lane:** re-checked via `git pull --rebase origin main`
immediately pre-push — see final push report for whether anything landed.

---

## Wave 2 — item 7

**Scope:** Delete all 5 [SELF] decision entries from `.planning/CONSULT-DECISIONS.md` 
per the T6 closeout plan + companion note cleanup. Executed 2026-07-16 by the Wave-2 
tree-writer (gsd-executor).

**Entries deleted (confirmed count = 5):**
- Line 71: A1 — "one benchmark session (≤50 ceiling)" definition
- Line 96: P115-T6 — headline framing honest CI-canonical reframe
- Line 114: P115-T2 — canonical latency measurement environment
- Line 123: P115-T5 — token-economy JSONL methodology  
- Line 153: P115-T4 — pivot GitHub backend from Jira/atlassian-rovo

**Companion note deleted:** Line 159 reference "Delete this entry when T4 captures land 
(folds into the T6 [SELF]-deletion sweep)" removed as part of T4 entry deletion.

**Verify against reality:**
- `grep -n '\[SELF\]' .planning/CONSULT-DECISIONS.md` → 1 line: the definition at line 6 
  (`[SELF]` = decided under the escalation-valve bar) — all 5 decision entries gone.
- File structure intact: RBF-LR-03 entry ends cleanly at line 69 + blank line 70; file 
  EOF at line 70 (no orphaned list markers, no broken headers).
- No regression in nearby content: read lines 60–70 confirmed boundary intact, no 
  extraneous deletions.

---

## Wave 2 — item 6a

**Scope:** (1) write the missing `quality/gates/perf/headline-numbers-cross-check.py`
verifier + its offline pytest; (2) reconcile the "8 ms" hero prose to the canonical
"6 ms get / 7 ms list" (`docs/benchmarks/latency.md` sim column); (3) repair + un-waive
the EXISTING P90-era `perf/headline-numbers-cross-check` catalog row (a dangling-verifier
row, NOT absent — no duplicate created); (4) rebind the two `MISSING_TEST` latency rows
whose claim changed + re-cite the two BOUND rows whose cited lines shifted. Executed
2026-07-16 by the Wave-2 item-6a tree-writer (gsd-executor). Sibling lane 6b follows.

**Gate written + proven (verify against reality):**
- `quality/gates/perf/headline-numbers-cross-check.py` — PARSES canonical get/list ms from
  `latency.md` + the four-axis reductions from `token-economy.md` (never hardcodes the
  numbers), cross-checks 6 latency + 6 token hero headlines across the 3 hero surfaces;
  teaching output names surface:line + the stale value + the canonical value + source + a
  copy-paste fix.
- **PRE-edit run: FAIL, exit 1, 6 drifts detected** (5×`8 ms` get + 1×`9 ms` list) —
  proves the gate actually fires. **POST-edit run: `PASS — all 6 latency + 6 token hero
  headlines match their canonical sources`, exit 0.**
- `quality/gates/perf/test_headline_numbers_cross_check.py` — **12 tests, all pass offline**
  (canonical parse + missing-structure raise, GREEN path, each RED drift class, teaching
  assertions, and `test_hero_surfaces_match_canonical` integration against the real docs —
  the fn the two rebound rows point at conceptually via the gate/test binding).
- **Minted via `run.py --cadence weekly --persist`:** row `perf/headline-numbers-cross-check`
  flipped `WAIVED → PASS`. Validate-only preview first confirmed **only `perf-targets.json`
  flips** (every `release-assets.json` row stayed PASS — surgical, no cross-catalog churn).
  Row now: `status=PASS, waiver=null, minted_at=2026-07-16, transport_claim=false,
  coverage_kind=mechanical`, `sources`+`tracked_in` refreshed to `P115 T6
  (115-UNWAIVE-PATH.md)`. (`transport_claim:false` is honest — this gate reads committed
  markdown and makes no transport/latency claim of its own; `coverage_kind:mechanical`.)

**Prose reconciled (6 edits, 3 hero surfaces):**
- `docs/index.md`: `:18` `8 ms`→`6 ms` (hero card cached read); `:87` `8 ms`→`6 ms`
  (Tested-against cache read); `:93` `9 ms`→`7 ms` (list-issues).
- `README.md`: `:23` `8 ms`→`6 ms` (simulator-measured caveat); `:25` `8 ms`→`6 ms`
  (read-one-issue bullet).
- `docs/concepts/reposix-vs-mcp-and-sdks.md`: `:14` `8 ms`→`6 ms` (Latency, cached read row).
- The `27 ms` cold-init figures were LEFT (out of scope; canonical init is `278 ms` — a
  separate, larger reconciliation, framed "simulator, interim / pending re-measurement" on
  the page). The gate deliberately does NOT check cold-init.

**Doc-alignment rows (via `reposix-quality doc-alignment bind`/`unwaive`):**
- **REBOUND** (claim `8 ms`→`6 ms`, bound to `headline-numbers-cross-check.py` +
  `test_headline_numbers_cross_check.py` AND-drift, MISSING_TEST waiver CLEARED via
  `unwaive`): `docs/index/latency-8ms-read` (`docs/index.md:18`), `latency-cached-read-8ms`
  (`docs/concepts/reposix-vs-mcp-and-sdks.md:14`) — both now `BOUND`, no waiver.
- **RE-CITED** (line-shift hash refresh, claim + test binding unchanged):
  `docs/index/tested-three-backends` (86-91 range — line 87's figure changed within it),
  `docs/index/soft-threshold-24ms` (line 93 — list figure changed; the cold-init soft-
  threshold claim is unchanged) — both stay `BOUND`.
- `bash quality/gates/docs-alignment/walk.sh` → **rc=0, zero blocking lines**.

**Expected side-effect (RAISED for 6b):** the `:18` edit changed the cited content of two
WAIVED cold-init rows (`docs/index/latency-24ms-cold-init`, `latency-hero-24ms-mismatch`) —
they re-grade `WAIVED-STALE_DOCS_DRIFT` (was `MISSING_TEST`), still non-blocking, waivers
untouched (not my scope). Their `115-UNWAIVE-PATH.md` exit route ("bind the 27 ms figure to
the cross-check gate") **no longer holds**: this gate deliberately does not cover cold-init
(canonical init `278 ms` ≠ hero `27 ms`). 6b must reconcile `27→278 ms` separately or extend
the gate — it cannot simply bind these rows to `headline-numbers-cross-check`.

**Gates:** headline gate GREEN (exit 0); pytest 12/12; walk rc=0; `banned-words-lint --all`
PASS; `mkdocs-strict` OK; `mermaid-renders` 7/7 OK.

**P0 independently discovered — main CI was RED — already fixed by a concurrent lane (NOT
me).** While inspecting the tree I found main's latest CI run (`29495972731`) had FAILED on
`bench-latency-v09` → step "Run latency bench against sim": item 5's regen-guard refuses to
overwrite the tracked `docs/benchmarks/latency.md`, and `.github/workflows/ci.yml` still ran
`latency-bench.sh` with the default OUT. **A concurrent CI-hotfix lane had already landed
the complete fix as local commit `3eacb53` ("fix(115-T6): let CI/cron bench through the
regen-clobber guard")** — ci.yml regen-to-scratch + the cron `ALLOW_CANONICAL_OVERWRITE`
producer path + a `regen-guard.selftest.sh` case (f) regression net (see the "Wave 2 — CI
hotfix" section below, authored by that lane). **I did NOT touch `ci.yml`** — I verified the
`OUT` override is wired end-to-end (`latency-bench.sh:53` default-override +
`emit-markdown.sh:11` `regen_guard_check "$OUT"` keys off the output path) and confirmed
`3eacb53` did not touch any of my files. `3eacb53` is local (1 ahead of `origin/main`) and
will ride out on this lane's push, turning main green — noted per the charter's mid-lane
landing clause.

**Concurrent-writer / owner commit mid-lane:** YES — `3eacb53` (CI hotfix) landed on local
HEAD during this lane; it is orthogonal to 6a (no file overlap) and left my gate GREEN +
catalog rows intact. Re-checked via `git pull --rebase origin main` immediately pre-push —
see final push report.

## Wave 2 — CI hotfix: bench-latency-v09 vs regen-clobber guard

- RED: run 29495972731 (e7a1fd2) — the item-5 guard fired inside CI;
  latency-bench.sh defaults OUT to the tracked latency.md (line 53).
- Fix (a) ci.yml: job is artifact-only (never publishes the file) — run
  step exports OUT="${RUNNER_TEMP}/latency-preview.md", uploads that path.
- Fix (b) cron: it IS the sanctioned producer (cp/diff/PR of the tracked
  file) — env sets REPOSIX_LATENCY_BENCH_ALLOW_CANONICAL_OVERWRITE="1".
- Regression net: regen-guard.selftest.sh case (f) greps both workflow
  wirings — 15/15 PASS locally. Green-run id: fixing lane report.

---

## Wave 2 — item 6b (FINAL T6 lane, 2026-07-16)

Terse (file over 20k warn — full row table lives in `115-UNWAIVE-PATH.md` § Wave 2 item 6b).

**Verdict: GREEN.** Walk rc=0, headline gate exit 0, perf pytest 26/26, banned-words/mkdocs/
mermaid all pass.

- **T1 cold-init reconcile:** hero 27 ms → canonical **278 ms** (latency.md `reposix init` cold
  sim; the 24/27 ms were superseded dev-machine artifacts per latency.md § Provenance — same
  operation, different env → fix-to-canonical). Extended `headline-numbers-cross-check.py`:
  +cold-init axis, +4 cold-init hero claims, +2 absolute loop-figure claims (18 headlines,
  all match). Bound+unwaived `latency-24ms-cold-init`, `latency-hero-24ms-mismatch`,
  `README-md/init-24ms`.
- **T2/T3 retire+dedupe:** propose-retired the 3 superseded 89.1% rows (`docs/index/
  token-reduction-89-percent`, `docs/why/token-economy-89-1-percent` [true dup — identical
  docs/index.md:17 source_hash], `README-md/token-89-percent`); re-waived with fresh
  P115-T6 attribution. (The charter's "4th STALE_DOCS_DRIFT hero row" = `README-md/latency-8ms`,
  an 8 ms LATENCY row → rebound to 6 ms + gate, NOT retired.)
- **T4 loop rows:** claims live post-reframe (d2fd85c) → bound+unwaived `mcp-loop-4883`
  (~21k, gate parses MCP median 21,171) + `reposix-loop-531` (~1.2k, reposix median 1,213).
- **T5 perf bench:** `bench_token_economy.py` main() now `_assert_headline_reduction()`
  (94.3% ±1.0pp; computed 94.27%); un-waived + minted `perf/token-economy-bench` PASS.
- **T6 validate-only flips:** BENIGN (code/shell-coverage FAIL→PASS stale kcov; security/
  cargo-audit NOT-VERIFIED→PASS) — persisted surgically (diffs = status+timestamp only).
- **T7 non-hero 8 ms:** mental-model:69 (8→6, 24→278), filesystem-layer:42 (8→6, re-cited),
  concepts-vs-mcp:15 (24→278). LEFT simulator.md:18 (dev-host framing) + mental-model:21
  (3 STALE_TEST_DRIFT rows → GTH-V15-33).
- **Re-cites (my edits shifted cited bytes):** `latency-8ms-read`, `soft-threshold-24ms`,
  `bootstrap-latency-24ms`, `filesystem-layer/blob-lazy-first-cat` — all stay BOUND.
- **Human relay updated:** confirm-retire batch is now **11 rows** (8 pre-existing + my 3).

### Human relay — action pending (SUPERSEDES the Wave-1 "8 rows" total)

`confirm-retire` (HUMAN-ONLY, TTY-guarded) is owed on **11 rows**: the 6 `docs/benchmarks/
token-economy/*` + 2 `token-baseline-{mcp-4883,reposix-531}` (Waves 1/prior) **plus** the 3
new 89.1%-era hero rows retired here (`docs/index/token-reduction-89-percent`,
`docs/why/token-economy-89-1-percent`, `README-md/token-89-percent`). One batch, per row:

```
target/release/reposix-quality doc-alignment confirm-retire --row-id <ID>
```

**Paper cuts filed:** GTH-V15-29..33 (`GOOD-TO-HAVES.md`); closeout-size split candidate
(`deferred-items.md`).
