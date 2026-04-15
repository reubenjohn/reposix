---
phase: 22
plan: B
type: execute
wave: 1
depends_on: []
files_modified:
  - benchmarks/fixtures/github_issues.json
  - benchmarks/fixtures/confluence_pages.json
  - benchmarks/fixtures/README.md
files_read_only:
  - scripts/bench_token_economy.py
autonomous: true
requirements:
  - BENCH-02
  - BENCH-03
user_setup: []

must_haves:
  truths:
    - "`benchmarks/fixtures/github_issues.json` exists and parses as JSON, contains at least 3 issue records with the shape GitHub's REST v3 `/repos/{owner}/{repo}/issues` returns (array of objects each with `id`, `number`, `title`, `body`, `state`, `user`, `labels`, `created_at`, `updated_at`)."
    - "`benchmarks/fixtures/confluence_pages.json` exists and parses as JSON, contains at least 3 page records shaped like Confluence `/wiki/api/v2/pages?limit=N` responses (top-level object with `results` array, each entry containing `id`, `title`, `body.atlas_doc_format.value`, `version.number`, `createdAt`)."
    - "`benchmarks/fixtures/README.md` exists and documents each fixture's provenance, shape, size, and the offline-reproducibility contract (cache files committed, no live API calls in CI)."
  artifacts:
    - path: "benchmarks/fixtures/github_issues.json"
      provides: "Representative GitHub issues REST v3 JSON payload for BENCH-02 github row"
      min_lines: 30
    - path: "benchmarks/fixtures/confluence_pages.json"
      provides: "Representative Confluence v2 pages JSON payload for BENCH-02 confluence row"
      min_lines: 30
    - path: "benchmarks/fixtures/README.md"
      provides: "Fixture provenance + offline contract"
      contains: "offline"
  key_links:
    - from: "benchmarks/fixtures/github_issues.json"
      to: "scripts/bench_token_economy.py (read by Plan 22-C)"
      via: "hard-coded fixture path loaded in main()"
      pattern: "github_issues\\.json"
    - from: "benchmarks/fixtures/confluence_pages.json"
      to: "scripts/bench_token_economy.py (read by Plan 22-C)"
      via: "hard-coded fixture path loaded in main()"
      pattern: "confluence_pages\\.json"
---

<objective>
Create the two new per-backend fixture files (`github_issues.json` + `confluence_pages.json`) and a `benchmarks/fixtures/README.md` documenting provenance + the offline cache contract. These fixtures are the **inputs** consumed by the per-backend comparison table that Plan 22-C wires into the script and into `docs/why.md`.

Purpose: BENCH-02 ("per-backend comparison table for sim, github, confluence") is impossible without representative JSON payloads to measure. The research (22-RESEARCH.md Pitfall 4) flagged the absence of these fixtures explicitly. BENCH-03 ("cold-mount matrix") is a stretch goal noted in the user-supplied wave suggestion; this plan emits a placeholder row contract (the matrix data files) so Plan 22-C can decide whether to ship or defer the matrix portion based on `REPOSIX_BENCH_LIVE` availability.

Output: Two new fixture JSON files in `benchmarks/fixtures/` + a README documenting the fixture set and offline contract. This plan writes NO Python code and modifies NO existing files (disjoint from Plan 22-A's `scripts/bench_token_economy.py` work → parallelizable in Wave 1).
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
@benchmarks/README.md
@benchmarks/fixtures/mcp_jira_catalog.json

<interfaces>
<!-- Real API response shapes. Fixtures must reproduce these contracts faithfully. -->

GitHub REST v3 `/repos/{owner}/{repo}/issues` (array; documented at docs.github.com/en/rest/issues/issues#list-repository-issues):
```json
[
  {
    "id": 1347,
    "node_id": "MDU6SXNzdWUx",
    "number": 1347,
    "title": "Found a bug",
    "user": { "login": "octocat", "id": 1, "type": "User" },
    "labels": [{ "id": 208045946, "name": "bug", "color": "d73a4a" }],
    "state": "open",
    "assignee": null,
    "assignees": [],
    "milestone": null,
    "comments": 0,
    "created_at": "2011-04-22T13:33:48Z",
    "updated_at": "2011-04-22T13:33:48Z",
    "closed_at": null,
    "author_association": "COLLABORATOR",
    "body": "I'm having a problem with this.",
    "reactions": { "url": "…", "total_count": 0 },
    "pull_request": null
  }
]
```

Confluence v2 `/wiki/api/v2/pages?limit=N` (object with `results` array; documented at developer.atlassian.com/cloud/confluence/rest/v2/api-group-page):
```json
{
  "results": [
    {
      "id": "393219",
      "status": "current",
      "title": "Welcome to Confluence",
      "spaceId": "98306",
      "parentId": null,
      "parentType": null,
      "position": 1,
      "authorId": "5b10a2844c20165700ede21g",
      "ownerId": "5b10a2844c20165700ede21g",
      "createdAt": "2024-01-15T09:00:00.000Z",
      "version": { "number": 1, "message": "", "minorEdit": false, "authorId": "…", "createdAt": "2024-01-15T09:00:00.000Z" },
      "body": {
        "atlas_doc_format": {
          "value": "{\"type\":\"doc\",\"version\":1,\"content\":[…]}",
          "representation": "atlas_doc_format"
        }
      },
      "_links": { "webui": "/spaces/X/pages/393219", "edittext": "/pages/resumedraft.action?draftId=393219" }
    }
  ],
  "_links": { "base": "https://example.atlassian.net/wiki" }
}
```

Downstream consumer (for reference only — not modified in this plan):
- `scripts/bench_token_economy.py` will load these two files in Plan 22-C via `(FIXTURES / "github_issues.json").read_text()` and `(FIXTURES / "confluence_pages.json").read_text()`.
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task B1: Create github_issues.json fixture</name>
  <files>benchmarks/fixtures/github_issues.json</files>
  <read_first>
    - benchmarks/fixtures/mcp_jira_catalog.json (reference for scale — 19362 bytes, 35 tools; our fixture is 3 issues so should be ~1-3KB)
    - CLAUDE.md §"Tainted by default" (any bytes derived from a remote are tainted; this fixture is explicitly anonymized synthetic data, tagged as such in the README)
    - 22-RESEARCH.md §Pitfall 4 (realistic shape or misleading comparison numbers)
  </read_first>
  <action>
    Write `benchmarks/fixtures/github_issues.json` as a JSON **array** of exactly 3 issue objects shaped like the GitHub REST v3 `/issues` response. Each object MUST include every field in the Interfaces block above (do not cherry-pick — verbose JSON IS the point of the BENCH-02 comparison).

    Use synthetic-but-realistic content inspired by reposix's own domain (no real user data, no scraped production text). Suggested contents:
    1. Issue #42: title="FUSE mount hangs when simulator dies", body ~150 chars describing the symptom with a stack-trace fragment in a fenced code block, label "bug", state "open".
    2. Issue #43: title="Add --offline flag to bench_token_economy.py", body ~200 chars describing the BENCH-01 upgrade, labels ["enhancement", "benchmarks"], state "open".
    3. Issue #44: title="Confluence ADF round-trip drops table cell alignment", body ~250 chars with a small Markdown table example inside, labels ["bug", "confluence"], state "closed", closed_at populated.

    Populate every field from the Interfaces block:
    - `id`, `node_id`, `number` — monotonically increasing integers (1000+1..3) and matching `number`.
    - `user` object with `login`, `id`, `type: "User"` — use fictitious usernames (e.g. "alice-bot", "benchmark-ci", "fuse-agent").
    - `labels` array with `id`, `name`, `color` — realistic hex colors.
    - `assignee: null`, `assignees: []`, `milestone: null` on at least one issue; populate on the others with fictitious entries.
    - `comments` integer (e.g. 0, 2, 5).
    - `created_at`, `updated_at`, `closed_at` — ISO8601 Zulu timestamps.
    - `author_association` — one of `OWNER`, `COLLABORATOR`, `CONTRIBUTOR`, `NONE`.
    - `body` — Markdown string with enough tokens to make the BENCH-02 reduction meaningful (~150-250 chars each).
    - `reactions` object with `url`, `total_count`, and per-emoji counts (`"+1"`, `"-1"`, `"laugh"`, `"hooray"`, `"confused"`, `"heart"`, `"rocket"`, `"eyes"`).
    - `pull_request: null` (we're not benchmarking PRs here).
    - `url`, `repository_url`, `labels_url`, `comments_url`, `events_url`, `html_url` — fictitious but valid-shaped URLs under `https://api.github.com/repos/example-org/example-repo/issues/{number}` and sibling paths.

    Final file size expectation: between 4 KB and 12 KB. If under 2 KB, the fixture is too sparse to match real API verbosity; if over 20 KB, it's gone past representative.

    Format: pretty-printed JSON with 2-space indent (readable in diffs), terminating newline.

    Forbidden:
    - No real user data (no scraped GitHub text). All content is synthetic.
    - No secrets / tokens / PII in any field.
    - Do NOT reference the `reposix/reposix` real repo — use `example-org/example-repo` so this fixture is obviously synthetic.
    - Do NOT omit fields to shrink the file — verbose JSON shape IS the measured quantity.
  </action>
  <acceptance_criteria>
    - `python3 -c "import json, pathlib; data = json.loads(pathlib.Path('benchmarks/fixtures/github_issues.json').read_text()); assert isinstance(data, list) and len(data) >= 3"` exits 0
    - `python3 -c "import json, pathlib; [issue for issue in json.loads(pathlib.Path('benchmarks/fixtures/github_issues.json').read_text()) if all(k in issue for k in ['id','number','title','body','state','user','labels','created_at','updated_at','reactions','author_association'])]"` — executes without assertion failure (every issue has the required keys)
    - `test $(wc -c < benchmarks/fixtures/github_issues.json) -ge 4000 && test $(wc -c < benchmarks/fixtures/github_issues.json) -le 12000`  (size sanity: 4–12 KB)
    - `! grep -iE 'token|api[_-]?key|password|secret|bearer|GITHUB_TOKEN' benchmarks/fixtures/github_issues.json`  (no credential-shaped strings)
  </acceptance_criteria>
  <verify>
    <automated>python3 -c "import json, pathlib, sys; data = json.loads(pathlib.Path('benchmarks/fixtures/github_issues.json').read_text()); assert isinstance(data, list) and len(data) >= 3, 'need >=3 issues'; required = {'id','number','title','body','state','user','labels','created_at','updated_at','reactions','author_association'}; [sys.exit(f'issue {i} missing {required - set(issue.keys())}') for i, issue in enumerate(data) if not required.issubset(issue)]; print(f'OK: {len(data)} issues, {pathlib.Path(\"benchmarks/fixtures/github_issues.json\").stat().st_size} bytes')"</automated>
  </verify>
  <done>`benchmarks/fixtures/github_issues.json` exists, parses as JSON array of ≥3 issues, each issue has the full required key set, size is 4–12 KB, no credential strings.</done>
</task>

<task type="auto">
  <name>Task B2: Create confluence_pages.json fixture</name>
  <files>benchmarks/fixtures/confluence_pages.json</files>
  <read_first>
    - benchmarks/fixtures/mcp_jira_catalog.json (reference for scale)
    - crates/reposix-confluence/src/ (grep for ADF shape examples used in tests — helpful to produce a realistic `atlas_doc_format.value` string)
    - 22-RESEARCH.md §Pitfall 4 (realistic shape requirement)
  </read_first>
  <action>
    Write `benchmarks/fixtures/confluence_pages.json` as a JSON **object** with `results` + `_links` top-level keys, shaped like a real `/wiki/api/v2/pages?limit=3` response. `results` MUST contain exactly 3 page objects.

    For each page, include every field in the Interfaces block:
    - `id` (string, e.g. "393219"), `status` ("current"), `title`, `spaceId` (string), `parentId` (null or string), `parentType` (null or "page"), `position` (integer), `authorId`, `ownerId`, `createdAt` (ISO8601).
    - `version` object: `number`, `message`, `minorEdit`, `authorId`, `createdAt`.
    - `body.atlas_doc_format`: the `value` field MUST be a **JSON-stringified** ADF document (a string whose value is valid JSON representing a `{"type":"doc","version":1,"content":[…]}` tree). Each document should have 3–5 content nodes mixing `paragraph`, `heading`, and `codeBlock`. Per-page body string length target: 400–1200 chars (post-stringification).
    - `body.atlas_doc_format.representation`: `"atlas_doc_format"`.
    - `_links` per-page: `webui`, `edittext` with plausible path shapes.

    Top-level `_links.base` = `"https://example.atlassian.net/wiki"`.

    Page content suggestions (synthetic, domain-relevant):
    1. Page 1: title="Engineering Runbook", 2 paragraphs + 1 heading (level 2) + 1 fenced code block with 3 lines of bash.
    2. Page 2: title="Security Review — 2026-Q2", 1 heading (level 1) + 1 bullet-list node with 4 items + 1 paragraph.
    3. Page 3: title="Onboarding Guide (draft)", 3 paragraphs + 1 heading + 1 table node (ADF `table` with 2 rows × 3 cols). `status` stays "current" but `version.number` = 4 (simulating history).

    File size expectation: 6 KB to 16 KB. The Confluence ADF shape is intentionally more verbose than GitHub because ADF carries every paragraph as a node tree — that verbosity IS the benchmark story.

    Format: pretty-printed JSON with 2-space indent, terminating newline. Note that the `atlas_doc_format.value` field is a string containing JSON — inside that string, escape quotes properly so the outer JSON still parses.

    Forbidden:
    - No real user data, no real tenant references beyond `example.atlassian.net`.
    - No secrets / keys / PII.
    - Do NOT stringify the entire response — only the `body.atlas_doc_format.value` field is string-wrapped (that matches the real API; the rest is structured).
  </action>
  <acceptance_criteria>
    - `python3 -c "import json, pathlib; data = json.loads(pathlib.Path('benchmarks/fixtures/confluence_pages.json').read_text()); assert 'results' in data and len(data['results']) >= 3"` exits 0
    - `python3 -c "import json, pathlib; data = json.loads(pathlib.Path('benchmarks/fixtures/confluence_pages.json').read_text()); [json.loads(p['body']['atlas_doc_format']['value']) for p in data['results']]"` — every page's ADF value re-parses as valid JSON (string-containing-JSON contract holds)
    - `python3 -c "import json, pathlib; data = json.loads(pathlib.Path('benchmarks/fixtures/confluence_pages.json').read_text()); [p for p in data['results'] if all(k in p for k in ['id','status','title','spaceId','createdAt','version','body'])]"` — every page has required top-level keys
    - `test $(wc -c < benchmarks/fixtures/confluence_pages.json) -ge 6000 && test $(wc -c < benchmarks/fixtures/confluence_pages.json) -le 16000`
    - `! grep -iE 'ATLASSIAN_API_KEY|bearer|password|secret|reuben' benchmarks/fixtures/confluence_pages.json`
  </acceptance_criteria>
  <verify>
    <automated>python3 -c "import json, pathlib, sys; data = json.loads(pathlib.Path('benchmarks/fixtures/confluence_pages.json').read_text()); assert 'results' in data and len(data['results']) >= 3, 'need results[] with >=3'; [json.loads(p['body']['atlas_doc_format']['value']) for p in data['results']]; required = {'id','status','title','spaceId','createdAt','version','body'}; [sys.exit(f'page {i} missing {required - set(p.keys())}') for i, p in enumerate(data['results']) if not required.issubset(p)]; print(f'OK: {len(data[\"results\"])} pages, {pathlib.Path(\"benchmarks/fixtures/confluence_pages.json\").stat().st_size} bytes')"</automated>
  </verify>
  <done>`benchmarks/fixtures/confluence_pages.json` exists, parses as JSON object with ≥3-entry `results`, every ADF `value` is parseable-JSON-inside-a-string, every page has required keys, size is 6–16 KB, no secrets.</done>
</task>

<task type="auto">
  <name>Task B3: Fixture provenance README</name>
  <files>benchmarks/fixtures/README.md</files>
  <read_first>
    - benchmarks/README.md (top-level honest-caveats section — style reference)
    - benchmarks/fixtures/github_issues.json (from Task B1 — include its measured char count)
    - benchmarks/fixtures/confluence_pages.json (from Task B2 — include its measured char count)
    - 22-RESEARCH.md §Open Questions #1 (the offline-reproducibility contract for committing *.tokens.json)
  </read_first>
  <action>
    Create `benchmarks/fixtures/README.md` with these sections (use pipe-table for the fixture inventory so each fixture has a measured char count + purpose):

    1. **Header:** `# Benchmarks fixtures — provenance + offline contract`
    2. **One-paragraph intro** explaining these are inputs consumed by `scripts/bench_token_economy.py`. Synthetic, anonymized, deterministic.
    3. **Fixture inventory table** with columns: `File` | `Size (bytes)` | `Shape` | `Purpose` | `Backend row`. Rows (fill in the **actual** measured sizes from `wc -c`):
       - `mcp_jira_catalog.json` | <size> | JSON object with `tools[]` | MCP-mediated baseline (35-tool Jira manifest) | MCP
       - `reposix_session.txt` | <size> | ANSI-stripped shell transcript | reposix POSIX session (read 3 issues, edit 1) | reposix
       - `github_issues.json` | <size> | JSON array, GitHub REST v3 | GitHub `/issues` raw payload | github (BENCH-02)
       - `confluence_pages.json` | <size> | JSON object with `results[]`, v2 shape | Confluence `/wiki/api/v2/pages` raw payload | confluence (BENCH-02)
    4. **Synthetic data disclaimer** (one paragraph): every fixture is constructed, not scraped. No real user data. No real tenant references. `example-org/example-repo` and `example.atlassian.net` are deliberately fake. Anyone tracing a token reduction claim back to these files must understand they represent *shape*, not *size of any real production payload*.
    5. **Offline-reproducibility contract** (one paragraph):
       - First run: `ANTHROPIC_API_KEY=<key> python3 scripts/bench_token_economy.py` populates `*.tokens.json` sidecar files in this directory.
       - Sidecar contents: `{"content_hash": sha256, "input_tokens": int, "source": <filename>, "model": "claude-3-haiku-20240307", "counted_at": <iso8601>}`.
       - Sidecars ARE committed to git (per 22-RESEARCH.md §Open Questions #1 resolution).
       - Subsequent runs (including CI): `python3 scripts/bench_token_economy.py --offline` reads cached counts and never calls the Anthropic API.
       - If a fixture's bytes change, its `sha256` will mismatch the cached `content_hash` and the script will refuse the cached value — regenerate with API key.
    6. **Adding a new fixture** (3-step recipe): create `<name>.{json,txt}` → run with `ANTHROPIC_API_KEY` to populate `<name>.{json,txt}.tokens.json` → commit both files. Reference `scripts/bench_token_economy.py` as the consumer.

    Forbidden:
    - Don't claim the fixtures mirror *any* real production payload size (they are *shape*-representative, not *magnitude*-representative).
    - Don't provide instructions that bypass the offline contract (no "just use mock token counts" fallback).
    - Don't commit a placeholder for a fixture that doesn't yet exist in this plan's scope.
  </action>
  <acceptance_criteria>
    - `test -f benchmarks/fixtures/README.md`
    - `grep -q 'offline' benchmarks/fixtures/README.md`
    - `grep -q 'github_issues.json' benchmarks/fixtures/README.md`
    - `grep -q 'confluence_pages.json' benchmarks/fixtures/README.md`
    - `grep -q 'content_hash' benchmarks/fixtures/README.md`
    - `grep -q 'synthetic' benchmarks/fixtures/README.md`
    - The measured sizes in the table are within ±5% of `wc -c` output for each fixture (spot-checkable by the checker; the task output will include a one-liner confirming this).
  </acceptance_criteria>
  <verify>
    <automated>test -f benchmarks/fixtures/README.md && grep -q 'offline' benchmarks/fixtures/README.md && grep -q 'content_hash' benchmarks/fixtures/README.md && grep -q 'github_issues.json' benchmarks/fixtures/README.md && grep -q 'confluence_pages.json' benchmarks/fixtures/README.md && grep -q 'synthetic' benchmarks/fixtures/README.md</automated>
  </verify>
  <done>Fixture README exists, names both new fixtures, documents the offline cache contract, labels fixtures as synthetic.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| committed fixture → bench script reads | Fixture content is author-controlled at commit time; treated as trusted input by the script (no untaint ceremony beyond JSON parsing). |
| committed fixture → public git history | Fixtures ship to every `git clone` of reposix; content is permanent. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-22-B-01 | Information Disclosure | Fixture content committed to public repo | mitigate | Every field is synthetic; user names are fictional (`alice-bot`, `benchmark-ci`); tenant is `example.atlassian.net`; acceptance criteria greps block `token/api_key/password/secret/GITHUB_TOKEN/ATLASSIAN_API_KEY/reuben` strings. |
| T-22-B-02 | Tampering | Someone edits a fixture without regenerating the `*.tokens.json` cache | mitigate (downstream) | Plan 22-A's `verify_fixture_cache_integrity` emits `WARN:` on `content_hash` mismatch; under `--offline` the script exits non-zero. Covered by Plan 22-A Test 4. |
| T-22-B-03 | Spoofing | Synthetic fixture mistaken for real production data | mitigate | `benchmarks/fixtures/README.md` explicit "synthetic, not scraped" disclaimer; `example-org/example-repo` naming convention makes provenance obvious at a glance. |

Per CLAUDE.md "Tainted by default" — this plan commits author-authored bytes, not remote-sourced bytes. The fixtures are trusted-by-construction (synthetic) and explicitly labelled as such. No SG-01 allowlist concerns (no network calls in this plan).
</threat_model>

<verification>
After this plan lands:
1. `python3 -c "import json; json.loads(open('benchmarks/fixtures/github_issues.json').read())"` → parses.
2. `python3 -c "import json; json.loads(open('benchmarks/fixtures/confluence_pages.json').read())"` → parses.
3. `test -f benchmarks/fixtures/README.md && grep -q offline benchmarks/fixtures/README.md` → both true.
4. Plan 22-A's existing tests still pass (this plan does not touch the script): `python3 -m pytest scripts/test_bench_token_economy.py -x -q`.
</verification>

<success_criteria>
- BENCH-02 prerequisites satisfied: the two missing fixture files exist with realistic API shapes.
- BENCH-03 prerequisites partially scoped: the per-backend fixture set is complete; the cold-mount timing matrix (live-binary timing) remains Plan 22-C's scope and may be deferred per `REPOSIX_BENCH_LIVE` gating (research confirms this is acceptable).
- No Python or Rust source files modified — pure fixture + docs plan.
- Fixtures are labelled synthetic and contain no secrets.
</success_criteria>

<output>
After completion, create `.planning/phases/22-op-8-honest-tokenizer-benchmarks-replace-len-div-4-with-coun/22-B-SUMMARY.md`. Record exact `wc -c` sizes for both fixtures + whether any shape deviations were made from the Interfaces block (and why).
</output>
