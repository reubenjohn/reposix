#!/usr/bin/env python3
"""Measure the reposix token-economy claim using real Anthropic token counts.

Compares the byte- and real-token-cost an LLM agent ingests for the same
task under two scenarios:

1. MCP-mediated: the agent loads a tool catalog + per-tool JSON schemas before
   calling anything. Fixture: benchmarks/fixtures/mcp_jira_catalog.json
2. reposix: the agent reads the bytes of its own shell session. Fixture:
   benchmarks/fixtures/reposix_session.txt

Emits a Markdown table to benchmarks/RESULTS.md. Prints the same table to
stdout.

Token counts are produced by Anthropic's count_tokens endpoint (see
requirements-bench.txt for the SDK pin). Results are cached in
benchmarks/fixtures/*.tokens.json (SHA-256 content hash as cache key) so
subsequent runs -- including CI --offline -- require no API key.

Usage:
  # First run (populates cache):
  ANTHROPIC_API_KEY=<key> python3 scripts/bench_token_economy.py

  # Subsequent runs (cache hit, no network):
  python3 scripts/bench_token_economy.py --offline
"""
from __future__ import annotations

import argparse
import datetime
import hashlib
import json
import os
import pathlib
import sys
from typing import Optional

# ---------------------------------------------------------------------------
# Module-level constants
# ---------------------------------------------------------------------------

# rationale: token counts are tokenizer-shared across Claude 3 text inputs;
# haiku is the cheapest stable model alias -- see 22-RESEARCH.md Pitfall 3.
COUNT_MODEL = "claude-3-haiku-20240307"

BENCH_DIR = pathlib.Path(__file__).resolve().parent.parent / "benchmarks"
FIXTURES = BENCH_DIR / "fixtures"
RESULTS = BENCH_DIR / "RESULTS.md"

# Lazy-initialised Anthropic client (avoids import at module scope so the test
# suite and --offline path can run without the package installed).
_CLIENT = None


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _sha256(text: str) -> str:
    """Return hex SHA-256 digest of *text* encoded as UTF-8."""
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def _cache_path(fixture_path: pathlib.Path) -> pathlib.Path:
    """Return the sidecar cache path for *fixture_path*.

    ``foo.json``  ->  ``foo.json.tokens.json``
    ``foo.txt``   ->  ``foo.txt.tokens.json``

    The double-suffix avoids collision with the original ``.json`` extension.
    """
    return fixture_path.with_suffix(fixture_path.suffix + ".tokens.json")


def _get_client():  # noqa: ANN201  (returns anthropic.Anthropic; not type-annotated to avoid import)
    """Lazily import and return a cached Anthropic client."""
    global _CLIENT  # noqa: PLW0603
    if _CLIENT is None:
        import anthropic  # noqa: PLC0415 (intentional lazy import)
        _CLIENT = anthropic.Anthropic()
    return _CLIENT


def count_tokens_api(text: str, client) -> int:  # noqa: ANN001
    """Return real token count for *text* via Anthropic count_tokens API.

    # Errors
    Propagates any ``anthropic.APIError`` raised by the SDK.
    """
    response = client.messages.count_tokens(
        model=COUNT_MODEL,
        messages=[{"role": "user", "content": text}],
    )
    return response.input_tokens


def get_or_count(
    text: str,
    fixture_path: pathlib.Path,
    *,
    offline: bool,
    counter=None,  # noqa: ANN001
) -> int:
    """Return cached token count or call the API and write the cache.

    Parameters
    ----------
    text:
        The fixture content whose tokens we want to count.
    fixture_path:
        Path to the source fixture file (used to derive the cache path and
        the ``source`` field in the cache JSON).
    offline:
        If True, raise ``SystemExit`` on cache miss instead of calling the API.
    counter:
        Optional callable ``(text: str, client) -> int`` used in place of
        :func:`count_tokens_api`. Lets tests inject a stub without the
        ``anthropic`` package installed.

    # Errors
    Raises ``SystemExit`` on cache miss when ``offline=True``.
    """
    content_hash = _sha256(text)
    cache_path = _cache_path(fixture_path)

    if cache_path.exists():
        try:
            cached = json.loads(cache_path.read_text())
            if cached.get("content_hash") == content_hash:
                return int(cached["input_tokens"])
        except (json.JSONDecodeError, KeyError):
            pass  # treat corrupt cache as a miss

    # Cache miss
    if offline:
        raise SystemExit(
            f"--offline: cache miss for {fixture_path.name}; "
            "run once with ANTHROPIC_API_KEY set to populate."
        )

    # Live API call (or stub for tests)
    if counter is not None:
        input_tokens = counter(text, None)
    else:
        input_tokens = count_tokens_api(text, _get_client())

    counted_at = datetime.datetime.now(datetime.timezone.utc).strftime(
        "%Y-%m-%dT%H:%M:%SZ"
    )
    cache_data = {
        "content_hash": content_hash,
        "input_tokens": input_tokens,
        "source": fixture_path.name,
        "model": COUNT_MODEL,
        "counted_at": counted_at,
    }
    cache_path.write_text(json.dumps(cache_data, indent=2))
    return input_tokens


def require_api_key_or_cached(fixture_paths: list) -> bool:
    """Exit with a named-variable message if the API key is missing and cache is incomplete.

    Parameters
    ----------
    fixture_paths:
        List of fixture ``pathlib.Path`` objects whose caches must exist when
        no API key is present.

    Returns
    -------
    bool
        ``True`` if ``ANTHROPIC_API_KEY`` is set, ``False`` if all fixtures
        are cached (and no key is needed).

    # Errors
    Calls ``sys.exit()`` with a message naming ``ANTHROPIC_API_KEY`` (never
    its value) when the key is absent and at least one fixture has no cache.
    """
    all_cached = all(_cache_path(p).exists() for p in fixture_paths)
    if not all_cached and not os.environ.get("ANTHROPIC_API_KEY"):
        sys.exit(
            "ANTHROPIC_API_KEY is required when cache is missing.\n"
            "Set it or commit benchmarks/fixtures/*.tokens.json for offline reproducibility.\n"
            "(See benchmarks/README.md for the offline contract.)"
        )
    return bool(os.environ.get("ANTHROPIC_API_KEY"))


def verify_fixture_cache_integrity(fixture_paths: list) -> list:
    """Return human-readable warnings for each cache file with a stale content_hash.

    Parameters
    ----------
    fixture_paths:
        List of fixture ``pathlib.Path`` objects to check.

    Returns
    -------
    list[str]
        Warning strings (may be empty if all caches are fresh).
    """
    warnings = []
    for fixture_path in fixture_paths:
        cache_path = _cache_path(fixture_path)
        if not cache_path.exists():
            continue
        if not fixture_path.exists():
            continue
        try:
            cached = json.loads(cache_path.read_text())
            fixture_bytes = fixture_path.read_bytes()
            actual_hash = _sha256(fixture_bytes.decode("utf-8"))
            if cached.get("content_hash") != actual_hash:
                warnings.append(
                    f"{fixture_path.name}: cache hash mismatch "
                    f"(cached={cached.get('content_hash', 'missing')[:12]}..., "
                    f"actual={actual_hash[:12]}...)"
                )
        except (json.JSONDecodeError, UnicodeDecodeError, KeyError):
            warnings.append(f"{fixture_path.name}: cache file is unreadable or corrupt")
    return warnings


# ---------------------------------------------------------------------------
# Fixture loaders
# ---------------------------------------------------------------------------


def load_mcp_bytes() -> tuple:
    """Return ``(serialized_text, fixture_path)`` for the MCP Jira catalog fixture.

    Serialises compactly (no indentation) to match real wire-format JSON,
    stripping the internal ``_note`` field.
    """
    path = FIXTURES / "mcp_jira_catalog.json"
    with path.open() as f:
        data = json.load(f)
    data.pop("_note", None)
    serialized = json.dumps(data, separators=(", ", ": "))
    return serialized, path


def load_reposix_bytes() -> tuple:
    """Return ``(text, fixture_path)`` for the reposix session transcript fixture."""
    path = FIXTURES / "reposix_session.txt"
    text = path.read_text()
    return text, path


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------


def _parse_args(argv: Optional[list] = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "--offline",
        action="store_true",
        default=False,
        help=(
            "Refuse to call the Anthropic API; read cache only. "
            "For CI and offline builds. "
            "Default: allow API calls when cache is missing and ANTHROPIC_API_KEY is set."
        ),
    )
    return parser.parse_args(argv)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def main(argv: Optional[list] = None) -> int:
    """Run the benchmark and write benchmarks/RESULTS.md.

    # Errors
    Calls ``sys.exit()`` if cache is missing and ``ANTHROPIC_API_KEY`` is
    unset (unless ``--offline`` is passed).
    """
    args = _parse_args(argv)

    mcp_text, mcp_path = load_mcp_bytes()
    reposix_text, reposix_path = load_reposix_bytes()

    fixture_paths = [mcp_path, reposix_path]

    # Stale-cache integrity check (warn, do not exit)
    integrity_warnings = verify_fixture_cache_integrity(fixture_paths)
    for w in integrity_warnings:
        print(f"WARN: {w}", file=sys.stderr)

    if not args.offline:
        require_api_key_or_cached(fixture_paths)

    mcp_tokens = get_or_count(mcp_text, mcp_path, offline=args.offline)
    reposix_tokens = get_or_count(reposix_text, reposix_path, offline=args.offline)

    mcp_chars = len(mcp_text)
    reposix_chars = len(reposix_text)

    ratio = mcp_tokens / reposix_tokens if reposix_tokens else float("inf")
    reduction_pct = 100 * (1 - reposix_tokens / mcp_tokens) if mcp_tokens else 0.0

    now = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%d %H:%M UTC")

    md = f"""# Benchmark results -- token economy

*Measured: {now}*

Task held constant across both scenarios: **read 3 issues, edit 1, push the
change**. What differs is only the context the agent must ingest to get
started.

| Scenario | Characters | Real tokens (Anthropic `count_tokens`) |
|----------|-----------:|---------------------------------------:|
| MCP-mediated (tool catalog + schemas) | {mcp_chars:>10,} | {mcp_tokens:>10,} |
| **reposix** (shell session transcript) | {reposix_chars:>10,} | **{reposix_tokens:>10,}** |

**Reduction:** `reposix` uses **{reduction_pct:.1f}%** fewer tokens than the
MCP-mediated baseline for the same task. Equivalently, MCP costs
**~{ratio:.1f}x** more context.

## What this does NOT measure

- Actual inference cost (token price depends on the frontier model).
- The agent's own reasoning tokens (they cancel out -- the task is identical).
- Tool-call output tokens (small and comparable).
- Re-fetch of schemas if the agent's context is compacted mid-session.

## What this DOES measure

- The raw bytes the agent's context window has to hold in order to be
  productive at minute 0.
- The cost of "learning the tool" vs "using what you already know".
- Token counts are produced by Anthropic's count_tokens endpoint (see requirements-bench.txt for the SDK pin).

## Fixture provenance

- `benchmarks/fixtures/mcp_jira_catalog.json` -- a representative manifest of
  35 Jira tools, modeled on the public Atlassian Forge surface and the schemas
  produced by the `mcp-atlassian` server. Full schemas for each tool, shaped
  like real JSON-Schema input descriptors.
- `benchmarks/fixtures/reposix_session.txt` -- the ANSI-stripped excerpt of
  what an agent's shell actually contains after running the equivalent
  workflow through `scripts/demo.sh`.

Reproduce: `python3 scripts/bench_token_economy.py --offline` (cache must be populated first)
"""

    def _normalize(text: str) -> str:
        return "\n".join(
            line for line in text.splitlines() if not line.startswith("*Measured:")
        )

    existing = RESULTS.read_text() if RESULTS.exists() else ""
    if _normalize(existing) == _normalize(md):
        print(md)
        print(f"(unchanged -- {RESULTS.relative_to(BENCH_DIR.parent)} already current)")
    else:
        RESULTS.write_text(md)
        print(md)
        print(f"(written to {RESULTS.relative_to(BENCH_DIR.parent)})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
