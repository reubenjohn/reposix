#!/usr/bin/env python3
"""quality/gates/perf/bench_token_economy.py -- perf dimension token-economy benchmark.

MIGRATED FROM: scripts/bench_token_economy.py per SIMPLIFY-11 (P59) -- file move only.
SPLIT:         IO helpers + table renderers extracted to bench_token_economy_io.py
               per the file-size-limits gate (15 000 char budget per .py file).
CATALOG ROW:   quality/catalogs/perf-targets.json -> perf/token-economy-bench (WAIVED until 2026-07-26)
CADENCE:       weekly (per docs/benchmarks/token-economy.md; ~1min wall time)
STATUS:        v0.12.0 file-relocate stub; full gate logic deferred to v0.12.1 via MIGRATE-03.

Wave E chose Option B (underscore filename) so the test file at
quality/gates/perf/test_bench_token_economy.py can keep its `import
bench_token_economy as bench` import unchanged. The test surface
(``bench.get_or_count``, ``bench._cache_path``, ``bench.COUNT_MODEL``,
``bench.FIXTURES`` etc.) remains intact via the re-exports below; the
hyphenated convention (used by other dimensions for entry-point scripts
that are never imported) does not apply here. See SURPRISES.md
2026-04-27 P59.

Predecessor preserved as scripts/bench_token_economy.py shim per OP-5
reversibility; P63 SIMPLIFY-12 audit may delete the shim.

What it does:

Measure the reposix token-economy claim using real Anthropic token counts.
Compares byte- and real-token-cost an LLM agent ingests for the same
task under MCP-mediated vs reposix scenarios. Also compares raw-JSON
costs across backends (BENCH-02): GitHub, Confluence, Jira (real
adapter placeholder).

Emits a Markdown table to docs/benchmarks/token-economy.md and prints
the same table to stdout. Token counts are produced by Anthropic's
count_tokens endpoint; results are cached in
benchmarks/fixtures/*.tokens.json (SHA-256 content hash as cache key)
so subsequent runs -- including CI --offline -- require no API key.

Usage:
  # First run (populates cache):
  ANTHROPIC_API_KEY=<key> python3 quality/gates/perf/bench_token_economy.py

  # Subsequent runs (cache hit, no network):
  python3 quality/gates/perf/bench_token_economy.py --offline
"""
from __future__ import annotations

import argparse
import datetime
import json
import pathlib
import sys
from typing import Optional

# Re-export from the IO sibling so tests doing `import bench_token_economy as bench`
# can continue to access these symbols via `bench.<name>` (test surface contract).
from bench_token_economy_io import (  # noqa: F401
    COUNT_MODEL,
    JIRA_REAL_PLACEHOLDER,
    _cache_path,
    _get_client,
    _sha256,
    count_tokens_api,
    get_or_count,
    load_raw_text,
    render_per_backend_table,
    render_results_markdown,
    require_api_key_or_cached,
    verify_fixture_cache_integrity,
)

# ---------------------------------------------------------------------------
# Module-level constants (kept here so tests' monkeypatch.setattr(bench, ...)
# continues to redirect FIXTURES / BENCH_DIR / RESULTS in main()).
# ---------------------------------------------------------------------------

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


# ---------------------------------------------------------------------------
# FIXTURES-dependent loaders (kept in the entry point so monkeypatching
# `bench.FIXTURES` redirects them, matching the predecessor behaviour).
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

    md = render_results_markdown(
        now=now,
        mcp_chars=mcp_chars,
        mcp_tokens=mcp_tokens,
        reposix_chars=reposix_chars,
        reposix_tokens=reposix_tokens,
        reduction_pct=reduction_pct,
        ratio=ratio,
        per_backend_table=per_backend_table,
    )

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
