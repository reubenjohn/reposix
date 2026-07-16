#!/usr/bin/env python3
"""quality/gates/perf/bench_token_economy.py -- perf dimension token-economy benchmark.

HEADLINE METHODOLOGY (P115 Task 5, amendment #10 "jsonl-usage-methodology"):

The token-economy headline is computed from the committed Claude Code session
JSONL **usage records** under ``benchmarks/captures/*.json`` -- the honest
end-to-end cost of a real agentic run against a live GitHub backend
(reubenjohn/reposix), median-of-3 per arm, two arms (reposix-mediated vs
MCP-mediated). It requires **NO ANTHROPIC_API_KEY** and no network: the doc is
regenerated deterministically + offline from the committed captures.

  python3 quality/gates/perf/bench_token_economy.py --offline   # regen the doc

The ``--offline`` flag is accepted for backward-compatibility and CI symmetry;
the headline path is always offline, so with or without it the result is
identical.

The prior methodology counted Anthropic ``count_tokens`` over static fixtures
(``benchmarks/fixtures/*.tokens.json``). That is retired as the headline source
and DEMOTED to an optional per-artifact enrichment. Its IO helpers
(``get_or_count``, ``_cache_path``, ``COUNT_MODEL`` ...) remain importable from
``bench_token_economy_io`` and re-exported below so the enrichment path and the
existing unit tests keep working; ``main()`` no longer calls them.

Emits a Markdown table to docs/benchmarks/token-economy.md and prints the same
table to stdout.

MIGRATED FROM: scripts/bench_token_economy.py per SIMPLIFY-11 (P59); shim kept.
CATALOG ROW:   quality/catalogs/perf-targets.json -> perf/token-economy-bench.
"""
from __future__ import annotations

import argparse
import pathlib
import sys
from typing import Optional

# Re-export the count_tokens / cache IO surface (retired as headline source,
# kept for the optional enrichment path + the existing unit-test contract that
# does `import bench_token_economy as bench` then `bench.<name>`).
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

# Re-export the JSONL-usage headline path (the live methodology).
from bench_token_economy_captures import (  # noqa: F401
    AXES,
    compute_arm_medians,
    compute_reductions,
    load_headline_captures,
    median,
    render_token_economy_markdown,
)

# ---------------------------------------------------------------------------
# Module-level constants (kept here so tests' monkeypatch.setattr(bench, ...)
# continues to redirect CAPTURES / BENCH_DIR / RESULTS / FIXTURES in main()).
# ---------------------------------------------------------------------------

# Workspace root is three levels up from quality/gates/perf/.
REPO_ROOT = pathlib.Path(__file__).resolve().parents[3]
BENCH_DIR = REPO_ROOT / "benchmarks"
# JSONL session-usage captures (the headline source).
CAPTURES = BENCH_DIR / "captures"
# Retired count_tokens fixtures (kept for the optional enrichment path only).
FIXTURES = BENCH_DIR / "fixtures"
# Results doc; regenerated in place, published by mkdocs.
RESULTS = REPO_ROOT / "docs" / "benchmarks" / "token-economy.md"


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
            "Accepted for backward-compat + CI symmetry. The JSONL-usage headline "
            "path is always offline (reads committed benchmarks/captures/*.json; "
            "never calls any API), so this flag does not change the result."
        ),
    )
    return parser.parse_args(argv)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def main(argv: Optional[list] = None) -> int:
    """Regenerate docs/benchmarks/token-economy.md from the committed captures.

    Deterministic + offline: parses the session-usage records, computes the
    per-axis medians per arm and the reposix-vs-MCP reductions, and writes the
    doc byte-for-byte reproducibly. No ANTHROPIC_API_KEY, no network.
    """
    _parse_args(argv)  # parse for --help / validation; flag is a no-op by design

    records = load_headline_captures(CAPTURES)
    arm_medians = compute_arm_medians(records)
    reductions = compute_reductions(arm_medians)

    md = render_token_economy_markdown(
        arm_medians=arm_medians,
        reductions=reductions,
        n_sessions=len(records),
    )

    existing = RESULTS.read_text() if RESULTS.exists() else ""
    if existing == md:
        print(md)
        print(f"(unchanged -- {RESULTS.relative_to(BENCH_DIR.parent)} already current)")
    else:
        RESULTS.write_text(md)
        print(md)
        print(f"(written to {RESULTS.relative_to(BENCH_DIR.parent)})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
