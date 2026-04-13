#!/usr/bin/env python3
"""Measure the reposix token-economy claim.

Compares the byte- and estimated-token-cost an LLM agent ingests for the same
task under two scenarios:

1. MCP-mediated: the agent loads a tool catalog + per-tool JSON schemas before
   calling anything. Fixture: benchmarks/fixtures/mcp_jira_catalog.json
2. reposix: the agent reads the bytes of its own shell session. Fixture:
   benchmarks/fixtures/reposix_session.txt

Emits a Markdown table to benchmarks/RESULTS.md. Prints the same table to
stdout.

Tokens are estimated via `len(text) / 4` — the standard heuristic used by
simonw/llm, theact, and matching Claude's real tokenizer within ~10% on
English + code. No tiktoken dependency.
"""
from __future__ import annotations
import datetime
import json
import pathlib
import sys


BENCH_DIR = pathlib.Path(__file__).resolve().parent.parent / "benchmarks"
FIXTURES = BENCH_DIR / "fixtures"
RESULTS = BENCH_DIR / "RESULTS.md"


def estimate_tokens(text: str) -> int:
    """Standard char/4 heuristic."""
    return len(text) // 4


def load_mcp_bytes() -> tuple[str, int]:
    """Return (serialized JSON, char length).

    We serialize compactly (no indentation) because that's how an MCP server
    transmits its manifest to a client — but we still include one space per
    separator so it's not pathologically terse. This matches the shape of a
    real `tools/list` response.
    """
    path = FIXTURES / "mcp_jira_catalog.json"
    with path.open() as f:
        data = json.load(f)
    # strip the _note field; it's internal to this fixture, not wire bytes
    data.pop("_note", None)
    serialized = json.dumps(data, separators=(", ", ": "))
    return serialized, len(serialized)


def load_reposix_bytes() -> tuple[str, int]:
    path = FIXTURES / "reposix_session.txt"
    text = path.read_text()
    return text, len(text)


def main() -> int:
    mcp_text, mcp_chars = load_mcp_bytes()
    mcp_tokens = estimate_tokens(mcp_text)

    reposix_text, reposix_chars = load_reposix_bytes()
    reposix_tokens = estimate_tokens(reposix_text)

    ratio = mcp_tokens / reposix_tokens if reposix_tokens else float("inf")
    reduction_pct = 100 * (1 - reposix_tokens / mcp_tokens) if mcp_tokens else 0.0

    now = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%d %H:%M UTC")

    md = f"""# Benchmark results — token economy

*Measured: {now}*

Task held constant across both scenarios: **read 3 issues, edit 1, push the
change**. What differs is only the context the agent must ingest to get
started.

| Scenario | Characters | Estimated tokens (`chars / 4`) |
|----------|-----------:|-------------------------------:|
| MCP-mediated (tool catalog + schemas) | {mcp_chars:>10,} | {mcp_tokens:>10,} |
| **reposix** (shell session transcript) | {reposix_chars:>10,} | **{reposix_tokens:>10,}** |

**Reduction:** `reposix` uses **{reduction_pct:.1f}%** fewer tokens than the
MCP-mediated baseline for the same task. Equivalently, MCP costs
**~{ratio:.1f}×** more context.

## What this does NOT measure

- Actual inference cost (token price depends on the frontier model).
- The agent's own reasoning tokens (they cancel out — the task is identical).
- Tool-call output tokens (small and comparable).
- Re-fetch of schemas if the agent's context is compacted mid-session.

## What this DOES measure

- The raw bytes the agent's context window has to hold in order to be
  productive at minute 0.
- The cost of "learning the tool" vs "using what you already know".

## Fixture provenance

- `benchmarks/fixtures/mcp_jira_catalog.json` — a representative manifest of
  35 Jira tools, modeled on the public Atlassian Forge surface and the schemas
  produced by the `mcp-atlassian` server. Full schemas for each tool, shaped
  like real JSON-Schema input descriptors.
- `benchmarks/fixtures/reposix_session.txt` — the ANSI-stripped excerpt of
  what an agent's shell actually contains after running the equivalent
  workflow through `scripts/demo.sh`.

Reproduce: `python3 scripts/bench_token_economy.py`
"""

    RESULTS.write_text(md)
    print(md)
    print(f"(written to {RESULTS.relative_to(BENCH_DIR.parent)})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
