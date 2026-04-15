---
phase: 22
plan: C
type: execute
wave: 2
depends_on:
  - 22-A
  - 22-B
files_modified:
  - scripts/bench_token_economy.py
  - scripts/test_bench_token_economy.py
  - benchmarks/RESULTS.md
  - benchmarks/README.md
  - benchmarks/fixtures/mcp_jira_catalog.json.tokens.json
  - benchmarks/fixtures/reposix_session.txt.tokens.json
  - benchmarks/fixtures/github_issues.json.tokens.json
  - benchmarks/fixtures/confluence_pages.json.tokens.json
  - docs/why.md
  - CHANGELOG.md
  - .planning/phases/22-op-8-honest-tokenizer-benchmarks-replace-len-div-4-with-coun/22-SUMMARY.md
autonomous: false
requirements:
  - BENCH-02
  - BENCH-03
  - BENCH-04
user_setup:
  - service: anthropic
    why: "Populate *.tokens.json cache once (one-shot; committed for offline CI thereafter)"
    env_vars:
      - name: ANTHROPIC_API_KEY
        source: "https://console.anthropic.com/settings/keys"

must_haves:
  truths:
    - "`scripts/bench_token_economy.py` emits a per-backend comparison table in `benchmarks/RESULTS.md` with rows for MCP, reposix, GitHub (raw), Confluence (raw), and Jira = `N/A (adapter not yet implemented)`."
    - "Committed `*.tokens.json` cache files for all four fixtures allow `python3 scripts/bench_token_economy.py --offline` to run end-to-end with zero network calls."
    - "`docs/why.md` headline number matches (within one decimal place) the reduction percentage printed by `scripts/bench_token_economy.py --offline` — no drift between docs and RESULTS.md."
    - "`docs/why.md` §Token-economy benchmark names the tokenizer as Anthropic's `count_tokens` API, not the `len/4` heuristic."
    - "`CHANGELOG.md [Unreleased]` records the BENCH-01..04 closure and the resulting reduction percentage."
  artifacts:
    - path: "benchmarks/RESULTS.md"
      provides: "Real-tokenizer per-backend table + optional cold-mount matrix"
      contains: "count_tokens"
    - path: "docs/why.md"
      provides: "Honest headline reduction number with Anthropic count_tokens citation"
      contains: "count_tokens"
    - path: "CHANGELOG.md"
      provides: "Phase 22 entry under [Unreleased]"
      contains: "BENCH-01"
    - path: "benchmarks/fixtures/github_issues.json.tokens.json"
      provides: "Cached token count for github fixture"
      contains: "content_hash"
    - path: "benchmarks/fixtures/confluence_pages.json.tokens.json"
      provides: "Cached token count for confluence fixture"
      contains: "content_hash"
  key_links:
    - from: "scripts/bench_token_economy.py"
      to: "benchmarks/fixtures/{mcp_jira_catalog.json,reposix_session.txt,github_issues.json,confluence_pages.json}"
      via: "main() loads all four fixtures and emits per-backend rows"
      pattern: "github_issues\\.json|confluence_pages\\.json"
    - from: "benchmarks/RESULTS.md"
      to: "docs/why.md"
      via: "headline reduction % copy-paste; must stay numerically consistent"
      pattern: "[0-9]+\\.[0-9]+%"
---

<objective>
Close **BENCH-02 / BENCH-03 / BENCH-04** by wiring the per-backend comparison into `scripts/bench_token_economy.py`, running the script once with a real `ANTHROPIC_API_KEY` to populate and commit `*.tokens.json` cache files, updating `benchmarks/RESULTS.md` + `docs/why.md` with the honest headline number, adding a CHANGELOG entry, and writing the phase SUMMARY.

Purpose: Plans 22-A and 22-B set up the script skeleton and the fixtures. This plan actually *runs the benchmark* and commits the result. A checkpoint gate in the middle ensures the user sees the new headline number (which may differ from 91.6%) before it replaces the old one in public docs — per CONTEXT.md: "If it's lower, say so. Dishonest-but-flattering beats honest only if you don't care about the project."

Output: Per-backend table in RESULTS.md + matching number in `docs/why.md` + committed cache files + CHANGELOG + phase SUMMARY. This is the shipping wave.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/STATE.md
@.planning/phases/22-op-8-honest-tokenizer-benchmarks-replace-len-div-4-with-coun/CONTEXT.md
@.planning/phases/22-op-8-honest-tokenizer-benchmarks-replace-len-div-4-with-coun/22-RESEARCH.md
@.planning/phases/22-op-8-honest-tokenizer-benchmarks-replace-len-div-4-with-coun/22-VALIDATION.md
@.planning/phases/22-op-8-honest-tokenizer-benchmarks-replace-len-div-4-with-coun/22-A-SUMMARY.md
@.planning/phases/22-op-8-honest-tokenizer-benchmarks-replace-len-div-4-with-coun/22-B-SUMMARY.md
@CLAUDE.md
@CHANGELOG.md
@docs/why.md
@benchmarks/README.md
@benchmarks/RESULTS.md
@scripts/bench_token_economy.py
@benchmarks/fixtures/README.md

<interfaces>
<!-- After this plan, scripts/bench_token_economy.py emits a RESULTS.md with this exact shape. -->

Target RESULTS.md structure (Task C1 must produce this in the script output):
```markdown
# Benchmark results — token economy

*Measured: <ISO8601 UTC>*
*Tokenizer: Anthropic count_tokens API (requirements-bench.txt pins anthropic==0.72.0)*

Task held constant: **read 3 issues, edit 1, push the change**.

## Baseline comparison (MCP-mediated vs reposix)

| Scenario | Characters | Real tokens (`count_tokens`) |
|----------|-----------:|-----------------------------:|
| MCP-mediated (tool catalog + schemas) | <N> | <N> |
| **reposix** (shell session transcript) | <N> | **<N>** |

**Reduction:** reposix uses **<X.X>%** fewer tokens. Equivalently, MCP costs **~<Y.Y>×** more context.

## Per-backend raw-JSON comparison (BENCH-02)

| Backend | Raw-API fixture | Characters | Real tokens | reposix tokens | Reduction |
|---------|-----------------|-----------:|------------:|---------------:|----------:|
| Jira (MCP) | mcp_jira_catalog.json | <N> | <N> | <N> | <X.X>% |
| GitHub | github_issues.json | <N> | <N> | <N> | <X.X>% |
| Confluence | confluence_pages.json | <N> | <N> | <N> | <X.X>% |
| Jira (real adapter) | — | — | — | — | N/A (adapter not yet implemented) |

(... "What this does / does NOT measure" sections preserved from existing RESULTS.md, with one bullet updated to cite count_tokens instead of len/4 ...)
```
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task C1: Extend bench script with per-backend table + update RESULTS.md contract</name>
  <files>scripts/bench_token_economy.py, scripts/test_bench_token_economy.py</files>
  <read_first>
    - scripts/bench_token_economy.py (post-Plan-22-A state: `main(argv)`, `get_or_count`, `verify_fixture_cache_integrity`)
    - benchmarks/fixtures/README.md (Plan 22-B output — fixture inventory table to mirror)
    - benchmarks/fixtures/github_issues.json, confluence_pages.json (Plan 22-B outputs)
    - 22-RESEARCH.md §"Per-Backend Comparison" portion (BENCH-02 requirement + Jira N/A row)
    - benchmarks/RESULTS.md (current shape — the "What this DOES/DOESN'T measure" sections are preserved)
  </read_first>
  <behavior>
    - Test 7 (`test_per_backend_table_rendered_with_all_four_rows`): with all four cache files present in `tmp_path`, `main(["--offline"])` produces RESULTS.md containing a markdown table where each of these strings appears in a row: `mcp_jira_catalog.json`, `github_issues.json`, `confluence_pages.json`, and the literal `N/A (adapter not yet implemented)` as the Jira-real row.
    - Test 8 (`test_per_backend_table_jira_real_row_has_no_fake_numbers`): the Jira (real adapter) row MUST NOT contain a percentage value or token count — only the `N/A` placeholder.
    - Test 9 (`test_headline_reduction_matches_baseline_pair`): the "baseline comparison" reduction percentage in RESULTS.md equals `round(100 * (1 - reposix_tokens / mcp_tokens), 1)` computed from the stored cache values for mcp and reposix. This is the number Plan 22-C Task 3 will paste into docs/why.md.
  </behavior>
  <action>
    1. Add these constants to `scripts/bench_token_economy.py`:
       - `GITHUB_FIXTURE = FIXTURES / "github_issues.json"`
       - `CONFLUENCE_FIXTURE = FIXTURES / "confluence_pages.json"`
       - `JIRA_REAL_PLACEHOLDER = "N/A (adapter not yet implemented)"`
    2. Add `load_raw_text(path: pathlib.Path) -> tuple[str, pathlib.Path]`:
       - If `path.suffix == ".json"`: parse, drop `_note` key if present (mirrors `load_mcp_bytes`), reserialize with `json.dumps(data, separators=(", ", ": "))` (same compact-with-spaces shape the existing MCP loader uses — keeps the GitHub/Confluence rows comparable to MCP).
       - If `path.suffix == ".txt"`: `path.read_text()` straight through.
       - Return `(serialized_text, path)`.
    3. Add `render_per_backend_table(rows: list[dict]) -> str` that produces the BENCH-02 pipe table per the Interfaces block. Each `row` dict has keys `backend`, `fixture`, `chars`, `raw_tokens`, `reposix_tokens`, `reduction_pct`. The Jira-real row is special: pass `raw_tokens=None`, `chars=None`, `reduction_pct=None` → render as `N/A (adapter not yet implemented)` in each numeric cell.
    4. Refactor `main()` to:
       - Load all four fixtures: `(mcp_text, mcp_path)`, `(reposix_text, reposix_path)`, `(gh_text, gh_path)`, `(conf_text, conf_path)`.
       - Require API key or cached-for-all-four.
       - `tokens = { ... }` — counts for all four via `get_or_count`.
       - Compute baseline pair reduction (unchanged formula: `100 * (1 - reposix_tokens / mcp_tokens)`).
       - Build per-backend rows: `{backend: "Jira (MCP)", fixture: "mcp_jira_catalog.json", chars: len(mcp_text), raw_tokens: mcp_tokens, reposix_tokens: reposix_tokens, reduction_pct: 100*(1 - reposix_tokens/mcp_tokens)}` plus equivalent rows for GitHub and Confluence, plus the Jira-real `N/A` row.
       - Preserve the existing "Fixture provenance" + "What this DOES/DOESN'T measure" sections but UPDATE the DOES section to cite Anthropic count_tokens instead of len/4. Remove the paragraph about `len / 4` being "within ~10% of Claude's real tokenizer" — replace with a one-liner: "Token counts are produced by Anthropic's `count_tokens` endpoint (SDK pinned in `requirements-bench.txt`)."
       - Preserve the "unchanged" detection (`_normalize` strips the `*Measured:` line) so repeat offline runs don't dirty the tree.
    5. Add Tests 7, 8, 9 to `scripts/test_bench_token_economy.py`. These tests use `tmp_path` with real synthetic fixture files (can be tiny — just need valid JSON shape) and pre-seeded `.tokens.json` sidecars with stub counts. Monkeypatch `bench.FIXTURES` and `bench.BENCH_DIR` so the test is fully hermetic.

    Forbidden: using `len/4` anywhere in the updated script body or RESULTS.md output. Forbidden: softening the Jira (real adapter) row to something less than `N/A (adapter not yet implemented)` — the exact string is required (CONTEXT.md design question 3 + research).
  </action>
  <acceptance_criteria>
    - `python3 -m pytest scripts/test_bench_token_economy.py -x -q`  (9/9 tests pass total)
    - `grep -q 'github_issues.json' scripts/bench_token_economy.py`
    - `grep -q 'confluence_pages.json' scripts/bench_token_economy.py`
    - `grep -q 'N/A (adapter not yet implemented)' scripts/bench_token_economy.py`
    - `grep -q 'render_per_backend_table' scripts/bench_token_economy.py`
    - `! grep -E 'len\([^)]*\) *// *4' scripts/bench_token_economy.py`  (still no live len/4)
    - `! grep -E "within ~10%" scripts/bench_token_economy.py`  (old len/4 apologia is gone)
  </acceptance_criteria>
  <verify>
    <automated>python3 -m pytest scripts/test_bench_token_economy.py -x -q && grep -q 'render_per_backend_table' scripts/bench_token_economy.py && grep -q 'N/A (adapter not yet implemented)' scripts/bench_token_economy.py</automated>
  </verify>
  <done>Script emits 5-row per-backend table (MCP, reposix, GitHub, Confluence, Jira-real). All 9 pytest cases pass. `len/4` and the old apologia are removed.</done>
</task>

<task type="checkpoint:human-verify" gate="blocking">
  <name>Task C2: CHECKPOINT — run script once with ANTHROPIC_API_KEY; confirm honest headline</name>
  <files>benchmarks/fixtures/mcp_jira_catalog.json.tokens.json, benchmarks/fixtures/reposix_session.txt.tokens.json, benchmarks/fixtures/github_issues.json.tokens.json, benchmarks/fixtures/confluence_pages.json.tokens.json, benchmarks/RESULTS.md</files>
  <read_first>
    - scripts/bench_token_economy.py (post-Task C1)
    - benchmarks/fixtures/*.json + *.txt (fixtures from Plans 22-A baseline + 22-B per-backend)
    - 22-RESEARCH.md §"API key guard" and §Pitfall 2 (never echo the key)
    - CONTEXT.md "If it's lower, say so"
  </read_first>
  <action>
    Run the one-shot cache-populating command documented in `<how-to-verify>` below. This is the ONLY network-touching step in Phase 22 — four Anthropic `count_tokens` calls that produce the sidecars. Executor MUST:
    (1) confirm `ANTHROPIC_API_KEY` is present in env without echoing its value,
    (2) `pip3 install --user -r requirements-bench.txt`,
    (3) `python3 scripts/bench_token_economy.py` (populates cache + rewrites RESULTS.md),
    (4) display the four sidecars + regenerated RESULTS.md,
    (5) confirm `--offline` re-run is a no-op,
    (6) HALT and wait for user sign-off on the new headline percentage.
    Do NOT proceed to Task C3 until the user has explicitly approved the percentage via `approved <PCT>` resume-signal.
  </action>
  <what-built>
    Task C1 produced a script that emits the real-tokenizer RESULTS.md shape. This checkpoint is where the user:
    (a) provides an `ANTHROPIC_API_KEY` (or confirms one is already in the shell env) for the one-shot cache-populating run,
    (b) inspects the honest headline number that falls out,
    (c) either accepts the number or asks for RESULTS.md / docs/why.md prose adjustments before Task C3 wires it into public docs.
  </what-built>
  <how-to-verify>
    Executor (Claude) runs:
    ```bash
    # 1. Install the SDK (one-shot; not tracked in workspace venv since this is a dev-host script).
    pip3 install --user -r requirements-bench.txt

    # 2. Populate cache with real API calls. This call is the ONLY network-touching step in Phase 22.
    ANTHROPIC_API_KEY="$ANTHROPIC_API_KEY" python3 scripts/bench_token_economy.py

    # 3. Show the four cache sidecars and their content hashes:
    for f in benchmarks/fixtures/*.tokens.json; do
      echo "=== $f ==="
      cat "$f"
      echo
    done

    # 4. Show the regenerated RESULTS.md:
    cat benchmarks/RESULTS.md

    # 5. Confirm offline reproducibility:
    python3 scripts/bench_token_economy.py --offline
    # (should print "(unchanged — …)" on stdout; no diff to benchmarks/RESULTS.md)
    ```

    User reviews:
    1. Are the four `*.tokens.json` cache files committed-worthy (no secrets, no unexpected fields)?
    2. Is the baseline reduction percentage reasonable? (Expectation from 22-RESEARCH.md: 85-98%; if wildly outside that range, fixture is broken.)
    3. Is the per-backend table complete with Jira-real showing `N/A (adapter not yet implemented)`?
    4. Does `--offline` run produce no file changes (proves the cache contract holds)?

    If the new headline is (e.g.) 89% instead of the old 91.6%: the user decides whether the prose in Task C3 should frame it as "recalibrated with real tokenizer, down from the len/4 estimate" or some other honest wording. The NUMBER itself is not negotiable — CONTEXT.md requires the real one.
  </how-to-verify>
  <acceptance_criteria>
    - All four `benchmarks/fixtures/*.tokens.json` sidecars exist and contain a `content_hash` field that matches `sha256` of the matching fixture bytes.
    - `benchmarks/RESULTS.md` contains a literal percentage `[0-9]+\.[0-9]+%` AND the string `count_tokens`.
    - `python3 scripts/bench_token_economy.py --offline` re-run prints `(unchanged — …)` and leaves the tree clean (no file diffs).
    - User has replied with `approved <PCT>` (copying the exact percentage from RESULTS.md) before Task C3 starts.
  </acceptance_criteria>
  <verify>
    <automated>test -f benchmarks/fixtures/mcp_jira_catalog.json.tokens.json && test -f benchmarks/fixtures/reposix_session.txt.tokens.json && test -f benchmarks/fixtures/github_issues.json.tokens.json && test -f benchmarks/fixtures/confluence_pages.json.tokens.json && grep -q 'count_tokens' benchmarks/RESULTS.md && grep -qE '[0-9]+\.[0-9]+%' benchmarks/RESULTS.md</automated>
  </verify>
  <done>Four sidecar `*.tokens.json` files exist with matching `content_hash` fields; `benchmarks/RESULTS.md` carries the real-tokenizer percentage; `--offline` re-run is a no-op; user has approved the headline percentage.</done>
  <resume-signal>
    Reply with one of:
    - `approved <PCT>` — confirm the new headline percentage (copy from RESULTS.md) for Task C3 to use.
    - `regenerate fixture <name>` — if a fixture shape looks wrong, return to Plan 22-B to fix it.
    - `script issue: <describe>` — if the output shape is wrong, return to Task C1.
  </resume-signal>
</task>

<task type="auto">
  <name>Task C3: Commit cache files + update docs/why.md + benchmarks/README.md + CHANGELOG + phase SUMMARY</name>
  <files>benchmarks/fixtures/mcp_jira_catalog.json.tokens.json, benchmarks/fixtures/reposix_session.txt.tokens.json, benchmarks/fixtures/github_issues.json.tokens.json, benchmarks/fixtures/confluence_pages.json.tokens.json, benchmarks/RESULTS.md, benchmarks/README.md, docs/why.md, CHANGELOG.md, .planning/phases/22-op-8-honest-tokenizer-benchmarks-replace-len-div-4-with-coun/22-SUMMARY.md</files>
  <read_first>
    - benchmarks/RESULTS.md (freshly regenerated in Task C2 — source of truth for all numbers)
    - benchmarks/fixtures/*.tokens.json (generated in Task C2 — commit these to git)
    - docs/why.md (current headline: "92.3%", "~12.9×" — both to be replaced with values from RESULTS.md)
    - CHANGELOG.md (find `[Unreleased]` section; add entry)
    - 22-VALIDATION.md Per-Task Verification Map (22-docs-01: `grep -qE "[0-9]+\.[0-9]+%" docs/why.md`)
    - The user-approved percentage from Task C2's `resume-signal`
  </read_first>
  <action>
    Numbers-from-RESULTS.md substitution throughout this task: whenever this plan references `<NEW_PCT>`, `<NEW_RATIO>`, `<MCP_TOKENS>`, `<REPOSIX_TOKENS>`, replace with the exact values from the freshly-generated `benchmarks/RESULTS.md`. Do not round differently from RESULTS.md — consistency between files is a must-have.

    1. **Commit cache sidecars.** The four `*.tokens.json` files produced in Task C2 are already on disk. This task only `git add`s them alongside the other file edits. Do not modify their contents.

    2. **Update `docs/why.md`** (the heart of BENCH-04):
       - Replace the line `> The architecture paper[^1] projected a ~98% reduction. We measured it against a representative fixture corpus. Result: **92.3% reduction** — reposix ingests **~12.9× less context** than an MCP-mediated baseline for the same task.` with the new measured values:
         `> The architecture paper[^1] projected a ~98% reduction. We measured it against a representative fixture corpus using Anthropic's \`count_tokens\` API (no more \`len/4\` heuristic). Result: **<NEW_PCT>% reduction** — reposix ingests **~<NEW_RATIO>× less context** than an MCP-mediated baseline for the same task.`
       - Replace the §"How we measure it" paragraph that says `…computes character counts and token estimates (via \`len/4\` — the standard heuristic that tracks Claude's real tokenizer within ~10%) and emits a Markdown table.` with: `…computes character counts and real token counts (via Anthropic's \`client.messages.count_tokens()\` API; results cached in \`benchmarks/fixtures/*.tokens.json\` for offline reproducibility) and emits a Markdown table.`
       - Update the 2-row table values:
         | MCP-mediated (tool catalog + schemas) | ~<MCP_TOKENS> |
         | **reposix** (shell session transcript) | **~<REPOSIX_TOKENS>** |
       - Replace the paragraph beginning `The paper's 98.7% number assumes…` with prose that compares (a) the old `len/4` estimate (91.6% historical) with (b) the new real-tokenizer number. Use the user's chosen framing from Task C2's resume signal. If `<NEW_PCT>` differs from 91.6% by more than ±2 points, the paragraph MUST explicitly acknowledge the change: "Prior to Phase 22 we published <OLD_PCT>% based on a `len/4` heuristic; with real tokenization the number is <NEW_PCT>%. We keep both on file in \`benchmarks/RESULTS.md\` git history."
       - Update the §"What the measurement does NOT capture" bullet that says `Real-world tokenizer quirks (our estimate is \`len / 4\`; Claude's tokenizer is within ~10% on English+code).` — DELETE this bullet entirely. It no longer applies; we now use the real tokenizer.
       - Add (if not already present in that section) a new bullet: `Fixture representativeness — our GitHub and Confluence fixtures are synthetic (see \`benchmarks/fixtures/README.md\`); real production payloads can be larger, which would push the reduction higher.`

    3. **Update `benchmarks/README.md`** (the fixture README at the top-level benchmarks dir):
       - §"Honest caveats" bullet `Token estimate uses \`len(text) / 4\`…` — rewrite to: `Token counts come from Anthropic's \`count_tokens\` API, cached in \`benchmarks/fixtures/*.tokens.json\` (committed). Offline CI runs via \`python3 scripts/bench_token_economy.py --offline\`.`
       - §"Running the benchmark" — add a second code block for the offline path:
         ```bash
         # One-shot (requires ANTHROPIC_API_KEY; populates cache):
         ANTHROPIC_API_KEY=<key> python3 scripts/bench_token_economy.py

         # Offline (reads committed cache; zero network):
         python3 scripts/bench_token_economy.py --offline
         ```

    4. **Append to `CHANGELOG.md` under `[Unreleased]`** (create the `[Unreleased]` section if absent):
       Under `### Changed`:
       - `bench_token_economy.py: token counts now produced by Anthropic's \`count_tokens\` API instead of the \`len(text) // 4\` heuristic. Cached in \`benchmarks/fixtures/*.tokens.json\` for offline reproducibility. Closes BENCH-01.`
       Under `### Added`:
       - `benchmarks/fixtures/github_issues.json + confluence_pages.json + fixtures README — per-backend token-economy comparison. Closes BENCH-02.`
       - `docs/why.md headline number recalibrated from len/4 estimate (91.6%) to real tokenization (<NEW_PCT>%). Closes BENCH-04.`
       - `requirements-bench.txt pinning anthropic==0.72.0 (dev-script dependency; not in the Rust workspace).`
       - (Optional, only if BENCH-03 matrix was shipped in Task C1) `Cold-mount time-to-first-ls matrix in RESULTS.md (sim cells offline; github/confluence gated on REPOSIX_BENCH_LIVE=1). Closes BENCH-03 (sim cells); remaining cells deferred.`

    5. **Write phase SUMMARY** at `.planning/phases/22-op-8-honest-tokenizer-benchmarks-replace-len-div-4-with-coun/22-SUMMARY.md` following `$HOME/.claude/get-shit-done/templates/summary.md`. Must include:
       - Exact before/after headline numbers (91.6% → <NEW_PCT>%) with the ratio.
       - Which requirements closed (BENCH-01 fully; BENCH-02 fully; BENCH-04 fully; BENCH-03 scoped to sim-only unless REPOSIX_BENCH_LIVE was used).
       - Commit SHAs for each plan wave.
       - The user-chosen framing language from Task C2 resume-signal.
       - A "Follow-ups" bullet if BENCH-03 real-backend cells were deferred.

    **Scope-reduction prohibition reminder:** Do NOT write `<NEW_PCT>` as "an estimate" or "approximately". RESULTS.md is authoritative to one decimal; `docs/why.md` must match to one decimal.

    Forbidden: leaving `92.3%` or `11.9×` or `~12.9×` anywhere in `docs/why.md` outside a `### History` subsection that explicitly attributes them to the pre-Phase-22 measurement. Forbidden: re-introducing `len/4` prose in `docs/why.md` except in a "prior approach" historical footnote.
  </action>
  <acceptance_criteria>
    - `test -f benchmarks/fixtures/mcp_jira_catalog.json.tokens.json && test -f benchmarks/fixtures/reposix_session.txt.tokens.json && test -f benchmarks/fixtures/github_issues.json.tokens.json && test -f benchmarks/fixtures/confluence_pages.json.tokens.json`
    - `for f in benchmarks/fixtures/*.tokens.json; do python3 -c "import json,sys; d = json.load(open('$f')); assert 'content_hash' in d and 'input_tokens' in d, '$f missing fields'"; done` — all four sidecars have required fields
    - `grep -qE "count_tokens" docs/why.md`
    - `! grep -qE "len *\( *text *\) */ *4" docs/why.md`  (old heuristic gone from docs)
    - `! grep -q "92.3%" docs/why.md`  (old headline gone — unless the new number happens to also be 92.3%, in which case this acceptance may be waived with justification)
    - `grep -qE "[0-9]+\.[0-9]+%" docs/why.md`  (a new percentage is present)
    - `grep -qE "count_tokens|Anthropic" benchmarks/README.md`
    - `grep -qE "offline" benchmarks/README.md`
    - `grep -qE "BENCH-0[124]" CHANGELOG.md`  (at least one of BENCH-01/02/04 mentioned)
    - `test -f .planning/phases/22-op-8-honest-tokenizer-benchmarks-replace-len-div-4-with-coun/22-SUMMARY.md`
    - `python3 scripts/bench_token_economy.py --offline`  (re-run: produces no file changes; cache hits on all four sidecars — no network)
    - `python3 -m pytest scripts/test_bench_token_economy.py -x -q`  (9/9 still green)
    - Cross-file consistency: the percentage in `docs/why.md` is the same (to 1 decimal) as the baseline reduction in `benchmarks/RESULTS.md`. Spot-check:
      `diff <(grep -oE '[0-9]+\.[0-9]+%' docs/why.md | head -1) <(grep -oE '[0-9]+\.[0-9]+%' benchmarks/RESULTS.md | head -1)`
  </acceptance_criteria>
  <verify>
    <automated>test -f benchmarks/fixtures/github_issues.json.tokens.json && test -f benchmarks/fixtures/confluence_pages.json.tokens.json && grep -qE "count_tokens" docs/why.md && ! grep -qE "len *\( *text *\) */ *4" docs/why.md && grep -qE "BENCH-0[124]" CHANGELOG.md && test -f .planning/phases/22-op-8-honest-tokenizer-benchmarks-replace-len-div-4-with-coun/22-SUMMARY.md && python3 scripts/bench_token_economy.py --offline && python3 -m pytest scripts/test_bench_token_economy.py -x -q</automated>
  </verify>
  <done>Four cache sidecars committed. `docs/why.md` headline cites real tokenizer with the new percentage; old 92.3% gone (or honestly historicized). `benchmarks/README.md` documents the offline contract. CHANGELOG [Unreleased] names BENCH-01/02/04. Phase SUMMARY written. Offline re-run is a no-op (cache hits). All pytest cases green.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| dev laptop → `api.anthropic.com` | HTTPS egress carrying ANTHROPIC_API_KEY during the one-shot cache population in Task C2. |
| `*.tokens.json` cache files → public git history | Committed sidecars encode token counts for synthetic fixtures; no secret material. |
| `docs/why.md` → public docs site | Published to readers of the reposix documentation; numerical claim is permanent until a future bench run updates it. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-22-C-01 | Information Disclosure | `ANTHROPIC_API_KEY` leaking into shell history / CI logs during Task C2 | mitigate | Checkpoint task's shell snippet uses `"$ANTHROPIC_API_KEY"` env expansion (not literal); instructions tell executor never to echo the key; pip install --user avoids global pollution. |
| T-22-C-02 | Tampering | Numerical drift between `benchmarks/RESULTS.md` and `docs/why.md` | mitigate | Acceptance criterion requires both files to carry the same percentage to one decimal; `diff` spot-check in the automated verify. |
| T-22-C-03 | Repudiation | Future reader can't tell which tokenizer produced a given reduction number | mitigate | `*.tokens.json` sidecars record `model` and `counted_at`; RESULTS.md header cites "Anthropic count_tokens API"; SDK version is pinned in `requirements-bench.txt` and CHANGELOG-logged. |
| T-22-C-04 | Denial of Service | Anthropic rate limit during the one-shot cache population | accept | Four API calls total for Phase 22 (one per fixture); well under any documented free-tier limit. Script exits gracefully on HTTP 429 (SDK-level retry; ultimate failure surfaces as non-zero exit). |
| T-22-C-05 | Elevation of Privilege | Committed fixture with secret-shaped content | mitigate | Plan 22-B acceptance criteria blocked credential-shaped strings; this plan does not introduce new fixture content. Cache sidecars contain only `content_hash` + `input_tokens` + fixture name. |

Per CLAUDE.md "Tainted by default" — network egress in Task C2 is exactly one host (`api.anthropic.com`) carrying synthetic fixture text. This is *developer-script* egress, not *product* egress (REPOSIX_ALLOWED_ORIGINS governs only the FUSE daemon and remote helper). Acknowledged.
</threat_model>

<verification>
End-of-phase check (run after Task C3):
1. `python3 -m pytest scripts/test_bench_token_economy.py -x -q` → 9/9 green.
2. `python3 scripts/bench_token_economy.py --offline` → prints "unchanged" or rewrites RESULTS.md with identical content; no network.
3. `grep -qE '[0-9]+\.[0-9]+%' docs/why.md && grep -q 'count_tokens' docs/why.md` → both true.
4. `grep -q 'BENCH-01' CHANGELOG.md` → true.
5. `ls benchmarks/fixtures/*.tokens.json | wc -l` → 4.
6. Cross-file percentage consistency (see Task C3 acceptance).
7. `cargo test --workspace` → still green (Phase 22 touches no Rust; regression check).
</verification>

<success_criteria>
- BENCH-01: script uses `count_tokens`; cache written + committed. Fully closed.
- BENCH-02: per-backend table with MCP, GitHub, Confluence rows + Jira-real `N/A`. Fully closed.
- BENCH-03: cold-mount matrix — either (a) sim-only cells shipped in Task C1 with REAL-backend cells gated on `REPOSIX_BENCH_LIVE` and documented as deferred, OR (b) BENCH-03 explicitly called out in the SUMMARY as not-shipped-this-phase (stretch goal per 22-RESEARCH.md). Either resolution must be documented.
- BENCH-04: `docs/why.md` headline matches the real count_tokens reduction and cites the API; old 92.3% is gone or historicized. Fully closed.
</success_criteria>

<output>
`.planning/phases/22-op-8-honest-tokenizer-benchmarks-replace-len-div-4-with-coun/22-SUMMARY.md` is the phase-wide SUMMARY (not per-plan) and replaces what would otherwise be `22-C-SUMMARY.md`. It captures the whole OP-8 story end-to-end so future planning phases can absorb it from the history digest.
</output>
