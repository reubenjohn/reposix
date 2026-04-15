# Phase 22: Honest-tokenizer benchmarks (OP-8) - Research

**Researched:** 2026-04-15
**Domain:** Python scripting, Anthropic SDK token-counting API, benchmark methodology
**Confidence:** HIGH

## Summary

Phase 22 upgrades a single Python script (`scripts/bench_token_economy.py`) from a `len(text)/4` approximation to real Anthropic SDK `client.messages.count_tokens()` API calls, caches results in `benchmarks/fixtures/*.tokens.json`, adds per-backend and matrix benchmark sections, and re-states the headline number in `docs/why.md`.

The codebase is a Rust workspace; this phase touches only Python (one script) and Markdown (one doc). No Rust changes. No new crates. The Anthropic Python SDK (`anthropic`) is not currently installed in the dev environment — installing it is Wave 0 work.

The current `benchmarks/RESULTS.md` reports **91.6% token reduction** using the `len/4` heuristic. The new number (post-`count_tokens`) will be authoritative but may differ. The CONTEXT.md is explicit: "If it's lower, say so."

**Primary recommendation:** Keep the upgrade fully self-contained in `scripts/bench_token_economy.py`. Use the `client.messages.count_tokens()` GA endpoint (not `beta.messages.count_tokens`). Cache with SHA-256 content hash. Gate `ANTHROPIC_API_KEY` with `sys.exit()`. Guard live-backend matrix cells behind `REPOSIX_BENCH_LIVE=1`.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Upgrade `scripts/bench_token_economy.py` in place — do not create a new script.
- Use `client.messages.count_tokens()` from the Anthropic SDK (not `len/4`).
- Cache token counts in `benchmarks/fixtures/*.tokens.json` with content hash as cache key.
- Guard `ANTHROPIC_API_KEY` with `if not os.environ.get("ANTHROPIC_API_KEY"): sys.exit(...)`.
- Gate 500-issue matrix cells for real backends behind `REPOSIX_BENCH_LIVE=1`.
- Jira row must appear in per-backend table as `N/A (adapter not yet implemented)`.
- Re-state the headline reduction in `docs/why.md`; do not soften or omit if number is lower.

### Claude's Discretion
- Exact cache filename scheme (within `benchmarks/fixtures/`).
- Whether to use `hashlib.sha256` or another hash for cache keys.
- Whether `count_tokens()` calls are synchronous or async (synchronous is simpler).
- Format of `benchmarks/fixtures/*.tokens.json` beyond including the content hash.
- How the script structures the per-backend table output (Markdown table in RESULTS.md).
- Whether the cold-mount timing section is a separate script or added to the existing one.

### Deferred Ideas (OUT OF SCOPE)
- Real Jira adapter (Phase 12 or later).
- Git-push round-trip latency benchmark (mentioned in CONTEXT.md §OP-8 as a BENCH item but NOT in REQUIREMENTS.md BENCH-01..04 — defer or treat as optional).
- Any Rust code changes.
- Any changes to `reposix-core`, `reposix-fuse`, `reposix-sim`, `reposix-remote`, or `reposix-cli`.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| BENCH-01 | `bench_token_economy.py` uses `client.messages.count_tokens()` instead of `len(text)/4`; results cached in `benchmarks/fixtures/*.tokens.json` | Anthropic SDK GA `count_tokens` endpoint verified; cache pattern with SHA-256 hash documented below |
| BENCH-02 | Per-backend comparison table (sim, github, confluence) for token reduction vs raw JSON API | Requires fixture files for github and confluence JSON payloads; sim fixture already exists; Jira row = N/A placeholder |
| BENCH-03 | Cold-mount time-to-first-ls matrix: 4 backends × [10, 100, 500] issues | Gated behind `REPOSIX_BENCH_LIVE=1` for real backends; sim-only cells always run |
| BENCH-04 | `docs/why.md` honest-framing section updated with real tokenization numbers | `docs/why.md` exists at line 42-53; headline "92.3%" and table rows are the update targets |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Token counting | Python script (offline) | Anthropic API (online) | The script is the consumer; the API is the measurement provider |
| Result caching | Python script writes JSON to `benchmarks/fixtures/` | — | Keeps benchmark offline-reproducible after first run |
| Per-backend fixture data | Static fixture files in `benchmarks/fixtures/` | — | Fixtures are the inputs; scripts compute from them |
| Cold-mount timing | Python `subprocess` + `time.monotonic()` | `reposix-fuse` binary (already built) | Script shells out to the binary; binary logic unchanged |
| Docs update | Manual edit of `docs/why.md` | — | Numbers come from RESULTS.md output; copyedit is human or script |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `anthropic` (Python SDK) | 0.72.0 (latest as of 2026-04-15) | `count_tokens()` API calls | Official Anthropic SDK; only way to call the token-count endpoint cleanly |
| `hashlib` (stdlib) | stdlib | SHA-256 content hash for cache keys | No extra dependency |
| `json` (stdlib) | stdlib | Read/write `.tokens.json` cache files | No extra dependency |
| `pathlib` (stdlib) | stdlib | Already used by the script | Consistent with existing code |
| `subprocess` (stdlib) | stdlib | Shell out for cold-mount timing matrix | No extra dependency |

**Version verification:** `pip install --dry-run anthropic` resolves to `0.72.0` [VERIFIED: pip dry-run, 2026-04-15].

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `os` (stdlib) | stdlib | Read env vars (`ANTHROPIC_API_KEY`, `REPOSIX_BENCH_LIVE`) | Already standard in the script |
| `time` (stdlib) | stdlib | `time.monotonic()` for wall-clock timing matrix | Cold-mount measurements |
| `datetime` (stdlib) | stdlib | Timestamp header in RESULTS.md | Already used in script |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `anthropic` SDK | Raw `requests` to `/v1/messages/count_tokens` | More code for same result; SDK handles retries and auth |
| `hashlib.sha256` | `hashlib.md5` | SHA-256 is collision-resistant; no practical difference here, but SHA-256 is safer by convention |
| Synchronous `anthropic.Anthropic()` | `anthropic.AsyncAnthropic()` | Async adds complexity for a CLI script; sync is appropriate |

**Installation:**
```bash
pip3 install anthropic
```

Python 3.8.10 is installed and is the runtime. `anthropic` 0.72.0 supports Python 3.8+. [VERIFIED: pip dry-run shows no Python version error]

## Architecture Patterns

### System Architecture Diagram

```
benchmarks/fixtures/
  mcp_jira_catalog.json    ──► count_tokens() ──► *.tokens.json cache
  reposix_session.txt      ──► count_tokens() ──► *.tokens.json cache
  github_issues.json       ──► count_tokens() ──► *.tokens.json cache  (new fixture)
  confluence_pages.json    ──► count_tokens() ──► *.tokens.json cache  (new fixture)

scripts/bench_token_economy.py
  ┌─────────────────────────────┐
  │  load_fixture(path)         │
  │  ↓                          │
  │  cache_key = sha256(text)   │
  │  ↓                          │
  │  if hit: read *.tokens.json │
  │  else:   count_tokens()     │
  │          write *.tokens.json│
  │  ↓                          │
  │  emit per-backend table     │
  │  emit cold-mount matrix     │─── subprocess: reposix mount + ls
  │  ↓                          │
  │  write benchmarks/RESULTS.md│
  └─────────────────────────────┘
          ↓
  docs/why.md  ← human updates headline after seeing RESULTS.md
```

### Recommended Project Structure
```
benchmarks/
├── fixtures/
│   ├── mcp_jira_catalog.json        # existing
│   ├── reposix_session.txt          # existing
│   ├── github_issues.json           # new (BENCH-02 github row)
│   ├── confluence_pages.json        # new (BENCH-02 confluence row)
│   ├── mcp_jira_catalog.tokens.json # new (cache, gitignore-able or committed)
│   └── reposix_session.tokens.json  # new (cache, same)
├── RESULTS.md                       # updated by script
└── README.md                        # existing; may note new cache files
scripts/
└── bench_token_economy.py           # upgraded in place
docs/
└── why.md                           # BENCH-04: update headline number
```

Cache files (`*.tokens.json`) can be committed to allow CI to run the script without an API key (cache hit = no API call). This is the offline-reproducibility guarantee mentioned in CONTEXT.md.

### Pattern 1: count_tokens() API call
**What:** Wraps a text string in a single-message list and counts tokens via the API.
**When to use:** Any time we need a real token count for a fixture string.

```python
# Source: https://platform.claude.com/docs/en/api/messages-count-tokens
import anthropic

def count_tokens_api(text: str, client: anthropic.Anthropic) -> int:
    """Return real token count for text via Anthropic count_tokens API."""
    response = client.messages.count_tokens(
        model="claude-haiku-3-5-20241022",  # cheapest model; token count is model-independent
        messages=[{"role": "user", "content": text}],
    )
    return response.input_tokens
```

The endpoint returns `MessageTokensCount` with a single field `input_tokens: int` (the total token count across messages + system prompt + tools). [VERIFIED: platform.claude.com/docs/en/api/messages-count-tokens]

### Pattern 2: Content-hash cache
**What:** Hash the input text; read/write a sidecar `.tokens.json` in `benchmarks/fixtures/`.
**When to use:** Before every `count_tokens()` call.

```python
import hashlib, json, pathlib

def get_or_count(text: str, fixture_path: pathlib.Path,
                 client: anthropic.Anthropic) -> int:
    """Return cached token count or call API and cache result."""
    content_hash = hashlib.sha256(text.encode()).hexdigest()
    cache_path = fixture_path.with_suffix(".tokens.json")
    if cache_path.exists():
        cached = json.loads(cache_path.read_text())
        if cached.get("content_hash") == content_hash:
            return cached["input_tokens"]
    # Cache miss or stale
    token_count = count_tokens_api(text, client)
    cache_path.write_text(json.dumps({
        "content_hash": content_hash,
        "input_tokens": token_count,
        "source": str(fixture_path.name),
    }, indent=2))
    return token_count
```

### Pattern 3: API key guard
**What:** Exit early with a clear error if `ANTHROPIC_API_KEY` is missing and no cache hit.
**When to use:** At the top of `main()`, before constructing the Anthropic client.

```python
import os, sys

def require_api_key_or_cached(fixtures: list[pathlib.Path]) -> bool:
    """Return True if API key present, False if all fixtures are cached (no key needed)."""
    all_cached = all(p.with_suffix(".tokens.json").exists() for p in fixtures)
    if not all_cached and not os.environ.get("ANTHROPIC_API_KEY"):
        sys.exit(
            "ANTHROPIC_API_KEY is required when cache is missing.\n"
            "Set it or commit benchmarks/fixtures/*.tokens.json for offline use."
        )
    return bool(os.environ.get("ANTHROPIC_API_KEY"))
```

### Anti-Patterns to Avoid
- **Calling `count_tokens()` on every run without caching:** Each call costs a network round-trip and burns API quota; fixture content rarely changes.
- **Using `beta.messages.count_tokens()`:** The GA endpoint `client.messages.count_tokens()` is stable as of SDK 0.72.0 and requires no `betas=` flag for text-only content. [VERIFIED: official docs]
- **Using `model="claude-3-5-sonnet-20241022"` for counting:** Token counts are model-agnostic for text (tokenizer is shared); any model string is valid. Use a cheap/fast alias like `claude-haiku-3-5-20241022` to avoid confusion.
- **Hardcoding a reduction percentage before running:** The script output is the truth; `docs/why.md` must be updated after the script runs, not before.
- **Gating the entire script on `REPOSIX_BENCH_LIVE=1`:** Only the 500-issue matrix cells for real backends need that gate. The token-economy portion should run offline against cached fixtures.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Token counting | Custom tokenizer / `len/4` heuristic | `client.messages.count_tokens()` | ±10% error is too large for published benchmarks; real API is accurate and cheap |
| HTTP auth + retries | Custom `requests` wrapper | `anthropic.Anthropic()` client | SDK handles `x-api-key` header, exponential backoff, and response parsing |
| Cache invalidation logic | Time-based TTL | Content hash (SHA-256) | Content-based cache never expires spuriously; deterministic for the same fixture |

**Key insight:** The entire complexity of this phase is at the API boundary (one new function) and the cache layer (one JSON sidecar per fixture). Everything else is string formatting already present in the script.

## Common Pitfalls

### Pitfall 1: Stale cache after fixture edit
**What goes wrong:** Developer edits `mcp_jira_catalog.json` but the cached `.tokens.json` still has the old count. The reduction number is silently wrong.
**Why it happens:** Cache was committed; the hash check is skipped if `content_hash` key is missing or the implementation has a bug.
**How to avoid:** Always compare `sha256(text) == cached["content_hash"]` before trusting the cached value. If mismatch: re-call the API.
**Warning signs:** `benchmarks/fixtures/*.tokens.json` has a `content_hash` that doesn't match `sha256` of the fixture file.

### Pitfall 2: API key in CI leaks to logs
**What goes wrong:** `sys.exit()` message or exception traceback prints the key value.
**Why it happens:** Passing the key as a string in an exception or printing `os.environ`.
**How to avoid:** Never print `ANTHROPIC_API_KEY` value. The guard message should only say the variable name.

### Pitfall 3: count_tokens() model parameter confusion
**What goes wrong:** Using an invalid or deprecated model string raises `InvalidRequestError`.
**Why it happens:** Copy-paste from messages API examples that use versioned model IDs.
**How to avoid:** Use a known current model ID. Token counts are tokenizer-identical across Claude 3 models for text-only inputs — use `claude-3-haiku-20240307` or `claude-haiku-3-5-20241022` as a stable, cheap model alias.
**Warning signs:** `anthropic.BadRequestError: model not found`.

### Pitfall 4: Per-backend fixtures don't exist yet
**What goes wrong:** BENCH-02 requires `github_issues.json` and `confluence_pages.json` fixtures, but neither exists in `benchmarks/fixtures/` today.
**Why it happens:** The current script only benchmarks sim (MCP Jira catalog vs reposix session). GitHub and Confluence fixtures need to be created.
**How to avoid:** Wave 0 must create representative fixture files. Use `gh api /repos/owner/repo/issues?per_page=3` output for github, and a comparable `/wiki/api/v2/pages?limit=3` JSON for confluence. These can be synthetic/anonymized.
**Warning signs:** `FileNotFoundError` when script tries to load `github_issues.json`.

### Pitfall 5: Cold-mount timing requires a built binary
**What goes wrong:** BENCH-03 shells out to `reposix mount`; if the binary is not built or the path is wrong, all timing cells are errors.
**Why it happens:** The Python script doesn't know how to find the Cargo build output path automatically.
**How to avoid:** The script should `cargo build -p reposix-cli --release` first (or accept a `--bin-path` flag), then use the explicit path. Gate the entire BENCH-03 section behind a binary-exists check.
**Warning signs:** `FileNotFoundError` or `subprocess.CalledProcessError` on first timing call.

## Code Examples

### Minimal count_tokens call
```python
# Source: https://platform.claude.com/docs/en/api/messages-count-tokens
import anthropic

client = anthropic.Anthropic()  # reads ANTHROPIC_API_KEY from env
result = client.messages.count_tokens(
    model="claude-3-haiku-20240307",
    messages=[{"role": "user", "content": text}],
)
print(result.input_tokens)  # int
```

### Cache file format (`.tokens.json`)
```json
{
  "content_hash": "sha256hexdigest",
  "input_tokens": 4201,
  "source": "mcp_jira_catalog.json"
}
```

### BENCH-03 cold-mount timing cell (pseudocode)
```python
import subprocess, time

def time_to_first_ls(bin_path: str, backend: str, issue_count: int,
                     mount_dir: str) -> float | None:
    """Wall-clock seconds from mount spawn to first non-empty ls."""
    t0 = time.monotonic()
    proc = subprocess.Popen([bin_path, "mount", "--backend", backend, mount_dir])
    # Poll until ls returns output or timeout
    for _ in range(50):
        out = subprocess.run(["ls", mount_dir], capture_output=True, text=True)
        if out.stdout.strip():
            proc.terminate()
            return time.monotonic() - t0
        time.sleep(0.05)
    proc.terminate()
    return None  # timeout
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `len(text) / 4` heuristic | `client.messages.count_tokens()` API | Anthropic released token-count API ~Nov 2024 | ±10% error → exact count |
| No per-backend comparison | Sim only (MCP Jira vs reposix session) | Phase 22 | Adds GitHub + Confluence rows |
| No cold-mount timing | N/A | Phase 22 | BENCH-03 new section |

**Deprecated/outdated:**
- `estimate_tokens(text: str) -> int` function in `bench_token_economy.py`: replaced by `get_or_count()` with API call + cache. Remove or keep as offline fallback clearly labelled as approximate.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Token count is model-independent for plain text — any valid model string works for `count_tokens()` | Code Examples | If tokenizer varies by model, the chosen model must match what the user's agent will actually use; negligible difference in practice |
| A2 | `anthropic` 0.72.0 supports Python 3.8.10 | Standard Stack | If minimum Python version is 3.9+, dev host needs a Python upgrade before running the script |
| A3 | `benchmarks/fixtures/github_issues.json` and `confluence_pages.json` need to be created as new fixtures | Common Pitfalls | If synthetic fixtures are unrealistic, the per-backend comparison numbers will be misleading |
| A4 | Cold-mount timing for sim cells is independent of `REPOSIX_BENCH_LIVE` | Architecture Patterns | Correct per CONTEXT.md design question 4: only 500-issue real-backend cells need the gate |

**If this table is empty:** All claims in this research were verified or cited — no user confirmation needed.

## Open Questions

1. **Should `*.tokens.json` cache files be committed to git?**
   - What we know: Committing them makes the benchmark offline-reproducible without an API key (beneficial for CI and contributors without keys).
   - What's unclear: Whether cache files in `benchmarks/fixtures/` should be `.gitignore`d or committed.
   - Recommendation: Commit them. They are deterministic (same content → same hash → same count), small (< 1 KB each), and enable CI to run the script without a secret. Document in `benchmarks/README.md`.

2. **Should the git-push round-trip latency section (mentioned in CONTEXT.md §OP-8 but absent from BENCH-01..04) be included?**
   - What we know: REQUIREMENTS.md BENCH-01..04 does not include a BENCH-05 for git-push latency.
   - What's unclear: Whether the CONTEXT.md mention is aspirational or required for this phase.
   - Recommendation: Treat as optional / stretch. If time permits after BENCH-01..04, add as a `## Git-push round-trip (optional)` section in RESULTS.md.

3. **What model string to use for `count_tokens()`?**
   - What we know: The API accepts any valid model ID; token counts are effectively model-independent for text on Claude 3+.
   - What's unclear: Whether to hard-code a model string or let the user override via env var.
   - Recommendation: Hard-code `claude-3-haiku-20240307` as a stable, cheap alias. Document in `--help` text.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Python 3 | Script runtime | Yes | 3.8.10 | — |
| `anthropic` SDK | BENCH-01 count_tokens | No (not installed) | 0.72.0 available via pip | Install in Wave 0 |
| `ANTHROPIC_API_KEY` | BENCH-01 (live call) | Unknown (secret) | — | Committed cache files bypass requirement |
| `reposix` binary | BENCH-03 cold-mount timing | Requires `cargo build` | current | Gate section behind binary-exists check |
| `fusermount3` | BENCH-03 (FUSE mount) | Likely (Ubuntu dev host) | — | Skip BENCH-03 if missing |
| `REPOSIX_BENCH_LIVE=1` | BENCH-03 real backends | Not set (default) | — | Skip real-backend cells; run sim only |

**Missing dependencies with no fallback:**
- None that block BENCH-01/02/04. BENCH-03 requires `reposix` binary + FUSE tools but is self-gating.

**Missing dependencies with fallback:**
- `anthropic` SDK: install via `pip3 install anthropic` (Wave 0 task). Cache files committed after first run mean subsequent runs are key-free.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | pytest (if added) or direct script invocation |
| Config file | none — script is standalone; no pytest.ini |
| Quick run command | `python3 scripts/bench_token_economy.py` (with cache = no API call) |
| Full suite command | `ANTHROPIC_API_KEY=... python3 scripts/bench_token_economy.py` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| BENCH-01 | Script uses count_tokens() not len/4; cache written | smoke (script run + assert cache file exists) | `python3 scripts/bench_token_economy.py && test -f benchmarks/fixtures/mcp_jira_catalog.tokens.json` | Wave 0 |
| BENCH-02 | Per-backend table appears in RESULTS.md | smoke (grep output) | `python3 scripts/bench_token_economy.py && grep -q 'github' benchmarks/RESULTS.md` | Wave 0 |
| BENCH-03 | Cold-mount matrix rows in RESULTS.md | smoke (sim cells only, offline) | `python3 scripts/bench_token_economy.py && grep -q '10 issues' benchmarks/RESULTS.md` | Wave 0 |
| BENCH-04 | `docs/why.md` headline number updated | manual verification | Eyeball `docs/why.md` after script run | existing file |

### Sampling Rate
- **Per task commit:** `python3 scripts/bench_token_economy.py` (cache hit = fast, no API call)
- **Per wave merge:** Same + verify `benchmarks/RESULTS.md` updated
- **Phase gate:** Full suite green (script runs clean, RESULTS.md contains all four sections, `docs/why.md` updated)

### Wave 0 Gaps
- [ ] `pip3 install anthropic` — must be done before first script run
- [ ] `benchmarks/fixtures/github_issues.json` — new fixture file for BENCH-02
- [ ] `benchmarks/fixtures/confluence_pages.json` — new fixture file for BENCH-02
- [ ] Smoke assertion for cache file presence after script run

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes (fixture files) | Fixture content is researcher-controlled; no untrusted input |
| V6 Cryptography | no | SHA-256 used for integrity only, not secrecy |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| API key leak in log output | Information Disclosure | Never print `ANTHROPIC_API_KEY` value in error messages |
| Fixture file path traversal | Tampering | Use `pathlib.Path` with explicit `BENCH_DIR` prefix; no user-supplied paths |
| Cache poisoning | Tampering | Content hash (`sha256`) ties cache to exact fixture content; mismatched hash → API re-call |

The threat surface here is minimal — this is a local developer script, not a server.

## Project Constraints (from CLAUDE.md)

- **No Rust changes.** This phase is Python + Markdown only.
- **Audit log not relevant.** The benchmark script does not mount FUSE or hit the sim.
- **`REPOSIX_ALLOWED_ORIGINS` not relevant.** Script calls `api.anthropic.com` directly via SDK, not through the reposix HTTP allowlist machinery (that allowlist is for the FUSE/remote-helper, not for Python scripts).
- **`#![forbid(unsafe_code)]`** — not applicable to Python.
- **GSD workflow**: enter via `/gsd-execute-phase 22` — no hand-edits outside a GSD-tracked phase.

## Sources

### Primary (HIGH confidence)
- `platform.claude.com/docs/en/api/messages-count-tokens` — exact API signature, response shape, `input_tokens` field [VERIFIED: WebFetch 2026-04-15]
- `scripts/bench_token_economy.py` — current script state (uses `len/4`, has `estimate_tokens()` function) [VERIFIED: Read tool]
- `docs/why.md` — current headline ("92.3% reduction"), table values (~7700 tokens vs ~595) [VERIFIED: Read tool]
- `benchmarks/RESULTS.md` — current measured reduction (91.6%, chars 16274/1372) [VERIFIED: Read tool]
- `benchmarks/fixtures/` — only `mcp_jira_catalog.json` (19362 bytes) and `reposix_session.txt` (1372 bytes) exist today [VERIFIED: ls + wc]
- `pip dry-run` — anthropic 0.72.0 available for Python 3.8 [VERIFIED: Bash]

### Secondary (MEDIUM confidence)
- WebSearch result confirming `client.messages.count_tokens()` GA signature [VERIFIED against official docs above]

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — SDK version verified via pip dry-run; API signature verified via official docs
- Architecture: HIGH — existing script fully read; design constraints from CONTEXT.md are explicit
- Pitfalls: MEDIUM — A3 (missing fixtures) is inferred from directory listing, not a spec

**Research date:** 2026-04-15
**Valid until:** 2026-05-15 (SDK version may change; API shape is stable)
