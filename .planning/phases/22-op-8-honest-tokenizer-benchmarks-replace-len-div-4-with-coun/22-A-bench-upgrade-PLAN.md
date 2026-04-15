---
phase: 22
plan: A
type: execute
wave: 1
depends_on: []
files_modified:
  - scripts/bench_token_economy.py
  - requirements-bench.txt
  - .gitignore
autonomous: true
requirements:
  - BENCH-01
user_setup:
  - service: anthropic
    why: "count_tokens() API calls; optional after cache committed"
    env_vars:
      - name: ANTHROPIC_API_KEY
        source: "https://console.anthropic.com/settings/keys"
    dashboard_config: []

must_haves:
  truths:
    - "Running `python3 scripts/bench_token_economy.py --offline` with committed cache files produces a RESULTS.md without ever calling the network."
    - "Running with a valid ANTHROPIC_API_KEY and no cache writes one `*.tokens.json` sidecar per input fixture whose `content_hash` matches `sha256(fixture_bytes)`."
    - "The string `len(text) // 4` and the function name `estimate_tokens` are gone from `scripts/bench_token_economy.py` (or reduced to a clearly-labelled offline fallback that is NOT the default path)."
    - "The script exits with a non-zero status and a message that names `ANTHROPIC_API_KEY` (without printing its value) when a fixture has no cache entry and the key is unset."
  artifacts:
    - path: "scripts/bench_token_economy.py"
      provides: "Upgraded benchmark script with count_tokens + SHA-256 cache + --offline flag"
      contains: "count_tokens"
    - path: "requirements-bench.txt"
      provides: "Pinned anthropic SDK dependency for the benchmark script"
      contains: "anthropic"
  key_links:
    - from: "scripts/bench_token_economy.py"
      to: "anthropic.Anthropic.messages.count_tokens"
      via: "synchronous SDK call gated on ANTHROPIC_API_KEY"
      pattern: "client\\.messages\\.count_tokens"
    - from: "scripts/bench_token_economy.py"
      to: "benchmarks/fixtures/*.tokens.json"
      via: "content-hash cache read/write"
      pattern: "content_hash.*sha256|sha256.*content_hash"
---

<objective>
Upgrade `scripts/bench_token_economy.py` from the `len(text) // 4` heuristic to real Anthropic SDK `client.messages.count_tokens()` calls, backed by a content-hash (SHA-256) cache in `benchmarks/fixtures/*.tokens.json`, with an `--offline` flag that forces cache-only mode for CI.

Purpose: Closes **BENCH-01** (real tokenizer) and lays the fixture-loading infrastructure that Plans B and C depend on. Honest measurement is the entire point of OP-8 — `len/4` is ±10% and unacceptable for a published headline number.

Output: Rewritten benchmark script + pinned `anthropic` dependency file + `.gitignore` hygiene for cache reproducibility. First run with `ANTHROPIC_API_KEY` populates the cache; subsequent runs (including CI `--offline`) read the cache and never touch the network.
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
@CLAUDE.md
@scripts/bench_token_economy.py
@benchmarks/README.md
@benchmarks/RESULTS.md

<interfaces>
<!-- Anthropic SDK surface (verified in 22-RESEARCH.md). Executor MUST use these call shapes verbatim. -->

From `anthropic` 0.72.0 (pip install via requirements-bench.txt):
```python
import anthropic

client = anthropic.Anthropic()  # reads ANTHROPIC_API_KEY from env automatically
result = client.messages.count_tokens(
    model="claude-3-haiku-20240307",  # stable; token counts are tokenizer-shared across Claude 3 text inputs
    messages=[{"role": "user", "content": text}],
)
# result.input_tokens is an int
```

Cache sidecar file format (JSON, written next to each fixture):
```json
{
  "content_hash": "<hex sha256 of fixture bytes>",
  "input_tokens": 4201,
  "source": "mcp_jira_catalog.json",
  "model": "claude-3-haiku-20240307",
  "counted_at": "2026-04-15T18:30:00Z"
}
```

Fixtures the script must be prepared to load (Plan 22-B creates the two new ones):
- `benchmarks/fixtures/mcp_jira_catalog.json` (exists, 19362 bytes)
- `benchmarks/fixtures/reposix_session.txt` (exists, 1372 bytes)
- `benchmarks/fixtures/github_issues.json` (created by 22-B, Wave 1)
- `benchmarks/fixtures/confluence_pages.json` (created by 22-B, Wave 1)
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task A1: Add anthropic dep + .gitignore hygiene + cache/fixture helpers (RED + GREEN for SHA-256 cache roundtrip)</name>
  <files>requirements-bench.txt, .gitignore, scripts/bench_token_economy.py, scripts/test_bench_token_economy.py</files>
  <read_first>
    - scripts/bench_token_economy.py (current shape: 134 lines, sync, uses estimate_tokens = len(text)//4)
    - benchmarks/README.md (honest-caveats section — will be contradicted after this phase; do not edit here, that is Plan 22-C's job)
    - 22-RESEARCH.md §Pattern 2 (content-hash cache code example) and §Pitfall 1 (stale cache detection)
    - 22-RESEARCH.md §"API key in CI leaks to logs" (Pitfall 2) — the guard MUST NOT print the key value
  </read_first>
  <behavior>
    - Test 1 (`test_cache_roundtrip_hits_on_identical_content`): write fixture "abc" → `get_or_count` with a stub that returns 7 tokens writes `<fixture>.tokens.json` with `content_hash == sha256("abc")` and `input_tokens == 7`. A second call with the same fixture reads the cache and does NOT invoke the stub counter.
    - Test 2 (`test_cache_miss_on_hash_change_calls_counter`): write fixture "abc" with cache claiming `content_hash == sha256("xyz")` → `get_or_count` detects mismatch, invokes the counter stub, rewrites cache with fresh hash.
    - Test 3 (`test_missing_cache_without_api_key_exits_with_named_variable`): with cache absent and `ANTHROPIC_API_KEY` unset in env, `require_api_key_or_cached([fixture])` calls `sys.exit(msg)` where `msg` contains the string `ANTHROPIC_API_KEY` and does NOT contain any value (stubbed or otherwise) of that variable.
    - Test 4 (`test_offline_mode_refuses_api_call_on_cache_miss`): when `--offline` is passed and a fixture has no cache file, the script exits non-zero WITHOUT calling the Anthropic client at all (monkeypatch client.messages.count_tokens to `raise AssertionError`).
  </behavior>
  <action>
    1. Create `requirements-bench.txt` at repo root with a single pinned line: `anthropic==0.72.0`  (rationale: SDK verified in 22-RESEARCH.md §Standard Stack; pinning protects against silent API shape drift).
    2. Append to `.gitignore` (if not already ignored there): `# Byproducts of scripts/bench_token_economy.py not involving fixtures:` and the single entry `runtime/bench-*.log` — do NOT add `benchmarks/fixtures/*.tokens.json` to `.gitignore`; per 22-RESEARCH.md §Open Questions #1 we commit the cache files so CI can run `--offline` with zero secrets.
    3. Rewrite `scripts/bench_token_economy.py` per the structure below. Preserve the file-header docstring's project voice but UPDATE it to describe the real-tokenizer approach.
       - Top of file imports: `from __future__ import annotations`, `argparse`, `datetime`, `hashlib`, `json`, `os`, `pathlib`, `sys`, `typing.Optional`.
       - DO NOT import `anthropic` at module scope. Import it lazily inside `_get_client()` so the test suite and `--offline` path can run without the package installed.
       - Add three constants at module scope:
         - `COUNT_MODEL = "claude-3-haiku-20240307"` with a `# rationale:` comment citing 22-RESEARCH.md Pitfall 3 (token counts are tokenizer-shared; cheap stable model)
         - `BENCH_DIR = pathlib.Path(__file__).resolve().parent.parent / "benchmarks"`
         - `FIXTURES = BENCH_DIR / "fixtures"`
       - Keep `load_mcp_bytes()` and `load_reposix_bytes()` essentially as-is but return `(text, path)` tuples — they still strip the `_note` field for MCP (matches existing behavior).
       - Add `_sha256(text: str) -> str` returning `hashlib.sha256(text.encode("utf-8")).hexdigest()`.
       - Add `_cache_path(fixture_path: pathlib.Path) -> pathlib.Path` returning `fixture_path.with_suffix(fixture_path.suffix + ".tokens.json")` (i.e. `foo.json` → `foo.json.tokens.json`, `foo.txt` → `foo.txt.tokens.json`). This avoids collision with the original fixture's own `.json` suffix.
       - Add `_get_client()` that lazily imports `anthropic` and returns `anthropic.Anthropic()`; caches on a module-level `_CLIENT = None`.
       - Add `count_tokens_api(text: str, client) -> int`:
         ```python
         response = client.messages.count_tokens(
             model=COUNT_MODEL,
             messages=[{"role": "user", "content": text}],
         )
         return response.input_tokens
         ```
       - Add `get_or_count(text: str, fixture_path: pathlib.Path, *, offline: bool, counter=None) -> int`:
         1. Compute `content_hash = _sha256(text)`.
         2. `cache_path = _cache_path(fixture_path)`.
         3. If `cache_path.exists()`: parse JSON; if `cached["content_hash"] == content_hash`: return `cached["input_tokens"]`.
         4. On cache miss: if `offline` is True: raise `SystemExit(f"--offline: cache miss for {fixture_path.name}; run once with ANTHROPIC_API_KEY set to populate.")`.
         5. Else: `counter = counter or count_tokens_api(text, _get_client())` — note the parameter lets tests inject a stub that returns a fixed int. Write cache JSON with keys `content_hash`, `input_tokens`, `source=fixture_path.name`, `model=COUNT_MODEL`, `counted_at=<utcnow ISO8601>`. Return `input_tokens`.
       - Add `require_api_key_or_cached(fixture_paths: list[pathlib.Path]) -> bool`:
         - `all_cached = all(_cache_path(p).exists() for p in fixture_paths)`.
         - If not `all_cached` and `not os.environ.get("ANTHROPIC_API_KEY")`: `sys.exit("ANTHROPIC_API_KEY is required when cache is missing.\nSet it or commit benchmarks/fixtures/*.tokens.json for offline reproducibility.\n(See benchmarks/README.md for the offline contract.)")`. The exit message MUST name the variable but MUST NOT print its value.
         - Return `bool(os.environ.get("ANTHROPIC_API_KEY"))`.
       - Add `_parse_args()` returning `argparse.Namespace` with `--offline` (store_true; default False). `--help` text must say: "Refuse to call the Anthropic API; read cache only. For CI and offline builds. Default: allow API calls when cache is missing and ANTHROPIC_API_KEY is set."
       - Refactor `main()` to:
         1. `args = _parse_args()`.
         2. Build `fixture_pairs = [(load_mcp_bytes(), "mcp"), (load_reposix_bytes(), "reposix")]` — load_mcp_bytes now returns `(serialized_text, fixture_path)`; load_reposix_bytes returns `(text, fixture_path)`.
         3. If `not args.offline`: `require_api_key_or_cached([pair[0][1] for pair in fixture_pairs])`.
         4. For each `((text, fixture_path), label)`: `tokens = get_or_count(text, fixture_path, offline=args.offline)`; collect `(label, len(text), tokens)`.
         5. Compute ratio + reduction_pct from mcp_tokens / reposix_tokens exactly as before (formulas unchanged; numerator/denominator unchanged).
         6. Emit the same RESULTS.md structure, but:
            - Change the column header from `Estimated tokens (\`chars / 4\`)` to `Real tokens (Anthropic \`count_tokens\`)`.
            - Add a paragraph under "What this DOES measure" noting the switch from len/4 to real tokenizer (the exact prose is Plan 22-C's scope; here just leave a single placeholder sentence: `"Token counts are produced by Anthropic's count_tokens endpoint (see requirements-bench.txt for the SDK pin)."`)
            - Preserve the existing "unchanged" detection (normalize by stripping the `*Measured:` line) so repeat smoke runs do not dirty the tree.
       - Keep `if __name__ == "__main__": sys.exit(main())`.
    4. Create `scripts/test_bench_token_economy.py` with `pytest`-compatible tests covering the four behaviors in `<behavior>`. Use `tmp_path` for fixture and cache files; import the script via `sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))` then `import bench_token_economy as bench`. For the client stub, use `unittest.mock.MagicMock()` — DO NOT depend on `anthropic` being installed in the test environment (use the `counter=` keyword in `get_or_count` for the stub-injection tests).
    5. DO NOT edit `benchmarks/fixtures/` in this plan. DO NOT edit `docs/why.md`. DO NOT edit `benchmarks/README.md`. DO NOT run the script against real fixtures in this task — `cargo`, `reposix`, and network are all out-of-scope here.

    Forbidden: adding `benchmarks/fixtures/*.tokens.json` to `.gitignore` (per 22-RESEARCH.md we commit the cache so CI can run `--offline`). Forbidden: printing the value of `ANTHROPIC_API_KEY` anywhere (22-RESEARCH.md Pitfall 2). Forbidden: scope-reducing BENCH-01 to "still-uses-len/4-as-fallback by default" — the default path MUST be the real count_tokens API with SHA-256-keyed cache.
  </action>
  <acceptance_criteria>
    - `grep -q 'anthropic==0.72.0' requirements-bench.txt`
    - `grep -q 'client.messages.count_tokens' scripts/bench_token_economy.py`
    - `grep -q 'hashlib.sha256' scripts/bench_token_economy.py`
    - `grep -q 'argparse' scripts/bench_token_economy.py`
    - `grep -q -- '--offline' scripts/bench_token_economy.py`
    - `! grep -E 'len\(.*\) *// *4' scripts/bench_token_economy.py`  (the old heuristic is gone from the module body; it MAY still appear only as a comment explaining the deprecation)
    - `! grep -n 'estimate_tokens(' scripts/bench_token_economy.py | grep -v '^[[:space:]]*#'`  (no uses of the old function in live code)
    - `python3 -m py_compile scripts/bench_token_economy.py`  (syntax check without executing)
    - `python3 -m py_compile scripts/test_bench_token_economy.py`
    - `python3 -m pytest scripts/test_bench_token_economy.py -x -q`  (4/4 tests pass; no anthropic import required)
    - `! grep -E 'fixtures/.*\.tokens\.json' .gitignore`  (cache files are NOT gitignored)
  </acceptance_criteria>
  <verify>
    <automated>python3 -m pytest scripts/test_bench_token_economy.py -x -q && python3 -m py_compile scripts/bench_token_economy.py && grep -q 'count_tokens' scripts/bench_token_economy.py && ! grep -E 'len\([^)]*\) *// *4' scripts/bench_token_economy.py</automated>
  </verify>
  <done>All four behavior tests pass. The script compiles. `count_tokens` appears, `len(text) // 4` does not (except possibly in a comment). `--offline` and SHA-256 caching are present. `.gitignore` does not hide the cache files. `requirements-bench.txt` pins `anthropic==0.72.0`.</done>
</task>

<task type="auto" tdd="true">
  <name>Task A2: Wire fixture-hash self-verification into script + end-to-end dry-run smoke without fixtures_b</name>
  <files>scripts/bench_token_economy.py, scripts/test_bench_token_economy.py</files>
  <read_first>
    - scripts/bench_token_economy.py (after Task A1 — the helpers are in place)
    - 22-VALIDATION.md Per-Task Verification Map (22-bench-01 and 22-cache-01 requirements)
    - 22-RESEARCH.md §"Common Pitfalls" #1 (stale cache) and #4 (missing per-backend fixtures)
  </read_first>
  <behavior>
    - Test 5 (`test_main_offline_with_mcp_and_reposix_cache_writes_results`): when `benchmarks/fixtures/mcp_jira_catalog.json.tokens.json` and `benchmarks/fixtures/reposix_session.txt.tokens.json` both exist with matching hashes, running `main(["--offline"])` writes a RESULTS.md that contains "MCP-mediated", "reposix", a reduction percentage matching `100 * (1 - reposix_tokens/mcp_tokens)`, and the exact string "Anthropic `count_tokens`" (column header). Must NOT contain the string "chars / 4" in the table header.
    - Test 6 (`test_main_falls_back_gracefully_when_per_backend_fixtures_absent`): when `github_issues.json` and `confluence_pages.json` do NOT yet exist (this plan is Wave 1; fixtures are 22-B Wave 1), `main(["--offline"])` still succeeds on the base two fixtures and emits a RESULTS.md without crashing — per-backend expansion is Plan 22-B's job; this task only ensures the core path still works.
  </behavior>
  <action>
    1. In `scripts/bench_token_economy.py`, expose `main(argv: Optional[list[str]] = None) -> int` by threading `argv` through `_parse_args`: `args = _parse_args(argv)`. `_parse_args` must call `argparse.ArgumentParser.parse_args(argv)` so tests can inject args. CLI behavior (`python3 scripts/bench_token_economy.py --offline`) is unchanged.
    2. Add a `verify_fixture_cache_integrity(fixture_paths: list[pathlib.Path]) -> list[str]` helper that returns a list of human-readable warning strings for each cache file whose `content_hash` disagrees with `_sha256(fixture_bytes)`. This is called from `main()` immediately after fixture load; any warnings are printed to `stderr` prefixed with `WARN:` (per-BENCH-01 stale-cache pitfall mitigation). Do not `sys.exit` on mismatch — the cache miss path already handles it by re-calling the API (or failing under `--offline`, matching Task A1 Test 4).
    3. Add the `Optional[list[str]]` type alias via `typing.Optional` import (already imported in A1). Rename no existing functions; only add the argv thread.
    4. Extend `scripts/test_bench_token_economy.py` with Tests 5 and 6. For Test 5, the test writes real fixture files into `tmp_path`, then monkeypatches `bench.FIXTURES = tmp_path` and `bench.BENCH_DIR = tmp_path.parent` before calling `bench.main(["--offline"])`. For Test 6 the test deliberately does NOT write `github_issues.json` / `confluence_pages.json` — the base script path must not reference them yet (Plan 22-B will add those references).
    5. DO NOT add per-backend (github/confluence) logic to the script in this task — that is Plan 22-B's exclusive scope. This task keeps A1's 2-fixture path working end-to-end and adds the stale-cache warning plumbing.

    Forbidden: adding `pathlib.Path("benchmarks/fixtures/github_issues.json")` or `.../confluence_pages.json` to the script in this task. Those references enter via Plan 22-B.
  </action>
  <acceptance_criteria>
    - `python3 -m pytest scripts/test_bench_token_economy.py -x -q`  (6/6 tests pass total — 4 from A1 + 2 from A2)
    - `grep -q 'verify_fixture_cache_integrity' scripts/bench_token_economy.py`
    - `grep -q 'def main(argv' scripts/bench_token_economy.py`
    - `! grep -n 'github_issues.json\|confluence_pages.json' scripts/bench_token_economy.py`  (per-backend fixtures NOT yet referenced; that is Plan 22-B's scope)
    - `python3 scripts/bench_token_economy.py --help 2>&1 | grep -q -- '--offline'`  (CLI help renders without crashing and shows the flag)
  </acceptance_criteria>
  <verify>
    <automated>python3 -m pytest scripts/test_bench_token_economy.py -x -q && python3 scripts/bench_token_economy.py --help 2>&1 | grep -q -- '--offline'</automated>
  </verify>
  <done>6/6 pytest cases green. CLI `--help` renders. Per-backend fixtures are NOT yet referenced by the script (they land in Plan 22-B). Stale-cache `WARN:` path exists and is tested.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| dev laptop → `api.anthropic.com` | HTTPS egress to count tokens; carries `ANTHROPIC_API_KEY` in header |
| benchmarks/fixtures/*.tokens.json → git history | Cache files are committed; they encode token counts derived from public fixture text + the current tokenizer state |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-22-A-01 | Information Disclosure | Error / exit messages in `require_api_key_or_cached` | mitigate | Exit message names the variable `ANTHROPIC_API_KEY` but MUST NOT print its value; explicit Test 3 asserts the value is absent from `SystemExit.code` text. |
| T-22-A-02 | Tampering | Cache file `*.tokens.json` with stale `content_hash` | mitigate | `get_or_count` always recomputes `_sha256(text)` and compares before trusting cached `input_tokens`; `verify_fixture_cache_integrity` emits `WARN:` on mismatch; 22-RESEARCH.md Pitfall 1 is the reference threat model. |
| T-22-A-03 | Denial of Service | `count_tokens` rate-limit or quota exhaustion in CI | mitigate | `--offline` flag bypasses all network calls; cache files committed; CI uses `--offline` (contract documented in Plan 22-C's README edit). |
| T-22-A-04 | Repudiation | Which model produced a given count | accept | Cache writes `model` + `counted_at` fields; sufficient audit trail for a research-quality benchmark. Not a production signing concern. |
| T-22-A-05 | Spoofing | Malicious `requirements-bench.txt` pulling in a typosquatted SDK | mitigate | Pin `anthropic==0.72.0` (exact ==); no wildcards. Developer-only runtime, not shipped to end users. |

**Per CLAUDE.md project constraints:** no FUSE mount, no audit log, no `REPOSIX_ALLOWED_ORIGINS` involvement — this script calls `api.anthropic.com` directly via SDK and is out-of-scope for the egress-allowlist guardrail (which protects the FUSE daemon and remote helper, not developer scripts).
</threat_model>

<verification>
After this plan lands:
1. `python3 -m pytest scripts/test_bench_token_economy.py -x -q` → 6/6 green.
2. `python3 scripts/bench_token_economy.py --help` → exits 0, shows `--offline`.
3. `grep -q 'count_tokens' scripts/bench_token_economy.py && grep -q 'hashlib.sha256' scripts/bench_token_economy.py` → both true.
4. `! grep -E 'len\([^)]*\) *// *4' scripts/bench_token_economy.py` → no live len/4.
5. `grep -q 'anthropic==0.72.0' requirements-bench.txt` → dependency pinned.
6. Running `ANTHROPIC_API_KEY=<key> python3 scripts/bench_token_economy.py` (MANUAL — ANTHROPIC_API_KEY required once to populate cache) produces `benchmarks/fixtures/mcp_jira_catalog.json.tokens.json` and `benchmarks/fixtures/reposix_session.txt.tokens.json` with valid SHA-256 `content_hash` fields. This is explicitly a manual step gated on the user's API key; Plan 22-C's checkpoint handles the visual confirmation.
</verification>

<success_criteria>
- BENCH-01 partially closed: the script uses `client.messages.count_tokens()` with an SHA-256-keyed cache. Final closure (the cache files committed to `benchmarks/fixtures/` from a real run) arrives in Plan 22-C.
- All behavior tests in `scripts/test_bench_token_economy.py` pass offline with no `anthropic` package installed (stub-injection works).
- The `--offline` flag exists and refuses all network calls.
- The API-key exit path names the variable without leaking its value.
</success_criteria>

<output>
After completion, create `.planning/phases/22-op-8-honest-tokenizer-benchmarks-replace-len-div-4-with-coun/22-A-SUMMARY.md` following `$HOME/.claude/get-shit-done/templates/summary.md`. Record: exact commit SHAs, pytest pass count, which lines of `bench_token_economy.py` changed, and whether the SDK was installed to the dev host (for Plan 22-C's CHANGELOG).
</output>
