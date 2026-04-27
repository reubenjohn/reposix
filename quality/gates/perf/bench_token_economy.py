#!/usr/bin/env python3
"""quality/gates/perf/bench_token_economy.py -- perf dimension token-economy benchmark.

MIGRATED FROM: scripts/bench_token_economy.py per SIMPLIFY-11 (P59) -- file move only.
CATALOG ROW:   quality/catalogs/perf-targets.json -> perf/token-economy-bench (WAIVED until 2026-07-26)
CADENCE:       weekly (per docs/benchmarks/token-economy.md; ~1min wall time)
STATUS:        v0.12.0 file-relocate stub; full gate logic deferred to v0.12.1 via MIGRATE-03.

Wave E chose Option B (underscore filename) so the test file at
quality/gates/perf/test_bench_token_economy.py can keep its `import
bench_token_economy as bench` import unchanged. The hyphenated
convention (used by other dimensions for entry-point scripts that are
never imported) does not apply here. See SURPRISES.md 2026-04-27 P59.

Predecessor preserved as scripts/bench_token_economy.py shim per OP-5
reversibility; P63 SIMPLIFY-12 audit may delete the shim.

Original docstring follows:

Measure the reposix token-economy claim using real Anthropic token counts.

Compares the byte- and real-token-cost an LLM agent ingests for the same
task under two scenarios:

1. MCP-mediated: the agent loads a tool catalog + per-tool JSON schemas before
   calling anything. Fixture: benchmarks/fixtures/mcp_jira_catalog.json
2. reposix: the agent reads the bytes of its own shell session. Fixture:
   benchmarks/fixtures/reposix_session.txt

Also compares raw-JSON token costs across backends (BENCH-02):
- GitHub: benchmarks/fixtures/github_issues.json
- Confluence: benchmarks/fixtures/confluence_pages.json
- Jira (real adapter): not yet implemented

Emits a Markdown table to docs/benchmarks/token-economy.md. Prints the same
table to stdout.

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

# Workspace root is three levels up from quality/gates/perf/ (was one
# level up from scripts/ in the predecessor).
REPO_ROOT = pathlib.Path(__file__).resolve().parents[3]
BENCH_DIR = REPO_ROOT / "benchmarks"
FIXTURES = BENCH_DIR / "fixtures"
# RESULTS.md was renamed and relocated to docs/benchmarks/token-economy.md so
# mkdocs can publish it directly (§0.4 of HANDOVER.md). The file is still
# regenerated in place by this script.
RESULTS = REPO_ROOT / "docs" / "benchmarks" / "token-economy.md"

GITHUB_FIXTURE = FIXTURES / "github_issues.json"
CONFLUENCE_FIXTURE = FIXTURES / "confluence_pages.json"
JIRA_REAL_PLACEHOLDER = "N/A (adapter not yet implemented)"

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
            # Hash must be computed from the processed text (after JSON round-trip),
            # matching what get_or_count stores — NOT the raw bytes on disk.
            processed_text, _ = load_raw_text(fixture_path)
            actual_hash = _sha256(processed_text)
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


def load_raw_text(path: pathlib.Path) -> tuple:
    """Return ``(serialized_text, path)`` for a raw fixture file.

    For JSON files: parse, drop ``_note`` key if present, reserialize with
    compact-with-spaces format (``json.dumps(data, separators=(", ", ": "))``).
    This matches the compact shape ``load_mcp_bytes`` uses, making GitHub/
    Confluence rows directly comparable to MCP.

    For text files: return the raw text unchanged.

    Parameters
    ----------
    path:
        Fixture file path. Must exist.

    Returns
    -------
    tuple[str, pathlib.Path]
        ``(serialized_text, path)``
    """
    if path.suffix == ".json":
        with path.open() as f:
            data = json.load(f)
        # Drop _note only if data is a dict (MCP catalog shape); GitHub fixture is a list.
        if isinstance(data, dict):
            data.pop("_note", None)
        serialized = json.dumps(data, separators=(", ", ": "))
        return serialized, path
    # Plain text (e.g. .txt)
    return path.read_text(), path


# ---------------------------------------------------------------------------
# Table renderers
# ---------------------------------------------------------------------------


def render_per_backend_table(rows: list) -> str:
    """Render the BENCH-02 per-backend comparison pipe table.

    Parameters
    ----------
    rows:
        List of dicts, each with keys:
        ``backend``, ``fixture``, ``chars``, ``raw_tokens``,
        ``reposix_tokens``, ``reduction_pct``.

        For the Jira-real placeholder row, pass ``raw_tokens=None``,
        ``chars=None``, ``reduction_pct=None`` — those cells will render
        as ``N/A (adapter not yet implemented)``.

    Returns
    -------
    str
        A Markdown pipe table string (no trailing newline).
    """
    header = (
        "| Backend | Raw-API fixture | Characters | Real tokens | reposix tokens | Reduction |\n"
        "|---------|-----------------|-----------:|------------:|---------------:|----------:|"
    )
    table_rows = []
    for row in rows:
        backend = row["backend"]
        fixture = row["fixture"]
        if row.get("raw_tokens") is None:
            # Placeholder row (Jira real adapter not yet implemented)
            chars_cell = "—"
            raw_tokens_cell = "—"
            reposix_tokens_cell = "—"
            reduction_cell = JIRA_REAL_PLACEHOLDER
        else:
            chars_cell = f"{row['chars']:,}"
            raw_tokens_cell = f"{row['raw_tokens']:,}"
            reposix_tokens_cell = f"{row['reposix_tokens']:,}"
            reduction_cell = f"{row['reduction_pct']:.1f}%"
        table_rows.append(
            f"| {backend} | {fixture} | {chars_cell} | {raw_tokens_cell} | {reposix_tokens_cell} | {reduction_cell} |"
        )
    return header + "\n" + "\n".join(table_rows)


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
    """Run the benchmark and write docs/benchmarks/token-economy.md.

    # Errors
    Calls ``sys.exit()`` if cache is missing and ``ANTHROPIC_API_KEY`` is
    unset (unless ``--offline`` is passed).
    """
    args = _parse_args(argv)

    mcp_text, mcp_path = load_mcp_bytes()
    reposix_text, reposix_path = load_reposix_bytes()

    # Per-backend fixtures (optional — skip gracefully if absent).
    # Resolve dynamically from FIXTURES so monkeypatching FIXTURES in tests works.
    gh_path = FIXTURES / "github_issues.json"
    conf_path = FIXTURES / "confluence_pages.json"
    gh_available = gh_path.exists()
    conf_available = conf_path.exists()

    gh_text: Optional[str] = None
    conf_text: Optional[str] = None
    if gh_available:
        gh_text, _ = load_raw_text(gh_path)
    if conf_available:
        conf_text, _ = load_raw_text(conf_path)

    # Baseline fixture list (always required)
    baseline_fixture_paths = [mcp_path, reposix_path]
    # Per-backend fixtures (include only if present)
    per_backend_fixture_paths = []
    if gh_available:
        per_backend_fixture_paths.append(gh_path)
    if conf_available:
        per_backend_fixture_paths.append(conf_path)

    all_fixture_paths = baseline_fixture_paths + per_backend_fixture_paths

    # Stale-cache integrity check (warn, do not exit)
    integrity_warnings = verify_fixture_cache_integrity(all_fixture_paths)
    for w in integrity_warnings:
        print(f"WARN: {w}", file=sys.stderr)

    if not args.offline:
        require_api_key_or_cached(all_fixture_paths)

    # Count tokens for all fixtures
    mcp_tokens = get_or_count(mcp_text, mcp_path, offline=args.offline)
    reposix_tokens = get_or_count(reposix_text, reposix_path, offline=args.offline)

    gh_tokens: Optional[int] = None
    conf_tokens: Optional[int] = None
    if gh_available and gh_text is not None:
        gh_tokens = get_or_count(gh_text, gh_path, offline=args.offline)
    if conf_available and conf_text is not None:
        conf_tokens = get_or_count(conf_text, conf_path, offline=args.offline)

    mcp_chars = len(mcp_text)
    reposix_chars = len(reposix_text)

    ratio = mcp_tokens / reposix_tokens if reposix_tokens else float("inf")
    reduction_pct = 100 * (1 - reposix_tokens / mcp_tokens) if mcp_tokens else 0.0

    now = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%d %H:%M UTC")

    # Build per-backend table rows
    per_backend_rows = [
        {
            "backend": "Jira (MCP)",
            "fixture": "mcp_jira_catalog.json",
            "chars": mcp_chars,
            "raw_tokens": mcp_tokens,
            "reposix_tokens": reposix_tokens,
            "reduction_pct": reduction_pct,
        },
    ]
    if gh_available and gh_tokens is not None and gh_text is not None:
        gh_chars = len(gh_text)
        gh_reduction = 100 * (1 - reposix_tokens / gh_tokens) if gh_tokens else 0.0
        per_backend_rows.append(
            {
                "backend": "GitHub",
                "fixture": "github_issues.json",
                "chars": gh_chars,
                "raw_tokens": gh_tokens,
                "reposix_tokens": reposix_tokens,
                "reduction_pct": gh_reduction,
            }
        )
    if conf_available and conf_tokens is not None and conf_text is not None:
        conf_chars = len(conf_text)
        conf_reduction = 100 * (1 - reposix_tokens / conf_tokens) if conf_tokens else 0.0
        per_backend_rows.append(
            {
                "backend": "Confluence",
                "fixture": "confluence_pages.json",
                "chars": conf_chars,
                "raw_tokens": conf_tokens,
                "reposix_tokens": reposix_tokens,
                "reduction_pct": conf_reduction,
            }
        )
    # Jira real adapter placeholder (always present)
    per_backend_rows.append(
        {
            "backend": "Jira (real adapter)",
            "fixture": "—",
            "chars": None,
            "raw_tokens": None,
            "reposix_tokens": None,
            "reduction_pct": None,
        }
    )

    per_backend_table = render_per_backend_table(per_backend_rows)

    md = f"""# Benchmark results -- token economy

*Measured: {now}*
*Tokenizer: Anthropic count_tokens API (requirements-bench.txt pins anthropic==0.72.0)*

Task held constant across both scenarios: **read 3 issues, edit 1, push the
change**. What differs is only the context the agent must ingest to get
started.

## Baseline comparison (MCP-mediated vs reposix)

| Scenario | Characters | Real tokens (`count_tokens`) |
|----------|-----------:|-----------------------------:|
| MCP-mediated (tool catalog + schemas) | {mcp_chars:>10,} | {mcp_tokens:>10,} |
| **reposix** (shell session transcript) | {reposix_chars:>10,} | **{reposix_tokens:>10,}** |

**Reduction:** `reposix` uses **{reduction_pct:.1f}%** fewer tokens than the
MCP-mediated baseline for the same task. Equivalently, MCP costs
**~{ratio:.1f}x** more context.

## Per-backend raw-JSON comparison (BENCH-02)

{per_backend_table}

## What this does NOT measure

- Actual inference cost (token price depends on the frontier model).
- The agent's own reasoning tokens (they cancel out -- the task is identical).
- Tool-call output tokens (small and comparable).
- Re-fetch of schemas if the agent's context is compacted mid-session.

## What this DOES measure

- The raw bytes the agent's context window has to hold in order to be
  productive at minute 0.
- The cost of "learning the tool" vs "using what you already know".
- Token counts are produced by Anthropic's `count_tokens` endpoint (SDK pinned in `requirements-bench.txt`).

## Fixture provenance

- `benchmarks/fixtures/mcp_jira_catalog.json` -- a representative manifest of
  35 Jira tools, modeled on the public Atlassian Forge surface and the schemas
  produced by the `mcp-atlassian` server. Full schemas for each tool, shaped
  like real JSON-Schema input descriptors.
- `benchmarks/fixtures/reposix_session.txt` -- the ANSI-stripped excerpt of
  what an agent's shell actually contains after running the equivalent
  workflow through `scripts/demo.sh`.
- `benchmarks/fixtures/github_issues.json` -- a synthetic GitHub REST v3
  `/repos/{{owner}}/{{repo}}/issues` payload with 3 representative issues.
- `benchmarks/fixtures/confluence_pages.json` -- a synthetic Confluence v2
  `/wiki/api/v2/pages` payload with 3 pages including full ADF body content.

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
