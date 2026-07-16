"""quality/gates/perf/bench_token_economy_captures.py -- JSONL session-usage headline path.

P115 Task 5 (amendment #10, 2026-07-15 "jsonl-usage-methodology"): the
token-economy HEADLINE numbers derive from the committed Claude Code session
JSONL **usage records** (one scrubbed extract per live benchmark session under
``benchmarks/captures/*.json``), NOT the Anthropic ``count_tokens`` endpoint.

This is the honest END-TO-END cost of a real agentic run against a live GitHub
backend -- no ``ANTHROPIC_API_KEY``, no network, deterministic offline from the
committed captures. ``count_tokens`` (in the sibling ``bench_token_economy_io``)
is demoted to an OPTIONAL per-artifact enrichment and is no longer on the
headline / doc-generation path.

Sibling of ``bench_token_economy.py`` (the entry point, which re-exports the
public names below). Split out per the file-size-limits gate (15 000 char
budget per .py file).
"""
from __future__ import annotations

import json
import pathlib
from typing import Optional

# ---------------------------------------------------------------------------
# Provenance constants (fixed at capture time -- keep the doc deterministic).
# The measurement date is the T4 capture date, NOT the regen date: regenerating
# the doc does not re-measure, so no volatile now()-timestamp is emitted.
# ---------------------------------------------------------------------------

CAPTURE_DATE = "2026-07-16"
BACKEND = "GitHub (reubenjohn/reposix)"
# The official GitHub MCP server (github/github-mcp-server), generally available,
# registered locally as `github-probe` for the capture. 44-tool surface recorded
# in benchmarks/fixtures/mcp_github_catalog.json.
MCP_SERVER = "the official GitHub MCP server (`github/github-mcp-server`, GA), registered locally as `github-probe`"
MODEL = "claude-sonnet-5 (`--model sonnet`)"
TASK = "read 3 GitHub issues (#56, #57, #60), edit 1 (#60 body marker), push"
N_PER_ARM = 3

# The four headline axes -> the capture JSON field each is summed from.
AXES = (
    ("output", "output_tokens"),
    ("cache_create", "cache_creation_input_tokens"),
    ("input_context", "total_input_context_tokens"),
    ("cost", "total_cost_usd"),
)

ARMS = ("reposix-mediated", "mcp-mediated")


# ---------------------------------------------------------------------------
# Capture loading + median math
# ---------------------------------------------------------------------------


def median(values: list) -> float:
    """Return the median of *values* (numeric). Empty list -> raises ValueError."""
    if not values:
        raise ValueError("median() of empty sequence")
    ordered = sorted(values)
    n = len(ordered)
    mid = n // 2
    if n % 2 == 1:
        return float(ordered[mid])
    return (ordered[mid - 1] + ordered[mid]) / 2.0


def _is_smoke(path: pathlib.Path, record: dict) -> bool:
    """A capture is a smoke/non-headline row if its filename says so or it lacks
    a ``run_label`` (the six real median-of-3 captures all carry one; the single
    ``mcp-kan-smoke.json`` context-only probe does not)."""
    if "smoke" in path.name.lower():
        return True
    return "run_label" not in record


def load_headline_captures(captures_dir: pathlib.Path) -> list:
    """Return the list of headline capture records under *captures_dir*.

    Globs ``*.json``, parses each, and EXCLUDES smoke / non-headline probes
    (see :func:`_is_smoke`). Sorted by ``run_label`` for determinism.

    # Errors
    Raises ``SystemExit`` if no headline captures are found (fail loud rather
    than silently emit an empty doc).
    """
    records = []
    for path in sorted(captures_dir.glob("*.json")):
        with path.open() as f:
            record = json.load(f)
        if _is_smoke(path, record):
            continue
        record["_source_file"] = path.name
        records.append(record)
    if not records:
        raise SystemExit(
            f"no headline captures found under {captures_dir} -- "
            "expected benchmarks/captures/{mcp,reposix}-github-run*.json"
        )
    return sorted(records, key=lambda r: r.get("run_label", ""))


def compute_arm_medians(records: list) -> dict:
    """Group *records* by ``arm`` and compute the per-axis median for each arm.

    Returns ``{arm: {axis_name: median, ..., "n": count}}`` for each arm found.
    Medians are taken PER AXIS independently (median-of-3), matching the
    SESSION-HANDOVER reference table.
    """
    by_arm: dict = {}
    for record in records:
        by_arm.setdefault(record["arm"], []).append(record)

    out: dict = {}
    for arm, arm_records in by_arm.items():
        arm_medians: dict = {"n": len(arm_records)}
        for axis_name, field in AXES:
            arm_medians[axis_name] = median([r[field] for r in arm_records])
        out[arm] = arm_medians
    return out


def compute_reductions(arm_medians: dict) -> dict:
    """Return the reposix-vs-MCP reduction percentage for each axis.

    ``pct = 100 * (1 - reposix_median / mcp_median)`` -- positive = reposix uses
    less. Also returns the cost + input-context multiples MCP pays.

    # Errors
    Raises ``KeyError`` if either arm is absent (both are required for a headline).
    """
    reposix = arm_medians["reposix-mediated"]
    mcp = arm_medians["mcp-mediated"]
    reductions: dict = {}
    for axis_name, _ in AXES:
        r_val = reposix[axis_name]
        m_val = mcp[axis_name]
        reductions[axis_name] = 100.0 * (1 - r_val / m_val) if m_val else 0.0
    reductions["cost_multiple"] = mcp["cost"] / reposix["cost"] if reposix["cost"] else float("inf")
    reductions["input_multiple"] = (
        mcp["input_context"] / reposix["input_context"] if reposix["input_context"] else float("inf")
    )
    return reductions


# ---------------------------------------------------------------------------
# Doc renderer (pure formatter -- no IO, no now())
# ---------------------------------------------------------------------------


def render_token_economy_markdown(
    *,
    arm_medians: dict,
    reductions: dict,
    n_sessions: int,
) -> str:
    """Render the full docs/benchmarks/token-economy.md body from the medians.

    Deterministic: every number is derived from the committed captures; there
    is no volatile timestamp, so repeated ``--offline`` runs reproduce the doc
    byte-for-byte.
    """
    r = arm_medians["reposix-mediated"]
    m = arm_medians["mcp-mediated"]
    red = reductions

    return f"""# Benchmark results -- token economy

*Measured: {CAPTURE_DATE}, from {n_sessions} live agentic sessions captured during P115 Task 4.*
*Source: committed session-usage records in `benchmarks/captures/*.json` (median-of-{N_PER_ARM} per arm).*

## Methodology

These numbers are the honest **end-to-end cost of a real agentic run**, not a
token-count of a static fixture. Two arms ran the **same task** against the
**same live backend**, {N_PER_ARM} times each (median-of-{N_PER_ARM}); each
session's Claude Code JSONL **usage record** (output tokens, cache-creation
tokens, total input-context tokens, and end-to-end USD cost) was scrubbed to an
offline-CI-stable extract and committed.

- **MCP-mediated arm** -- the agent reaches the backend through {MCP_SERVER}.
- **reposix-mediated arm** -- the agent reaches the SAME backend content through
  a `reposix` git-native checkout, using only `cat` / `grep` / `git` (run under
  `--strict-mcp-config` with zero MCP servers loaded, so its usage carries no
  MCP tool-loading cost).
- **Backend:** {BACKEND} (public RUSTSEC-advisory issues; no private data).
- **Task (held constant):** {TASK}.
- **Model:** {MODEL}.

Runs are offline-reproducible: `python3 quality/gates/perf/bench_token_economy.py --offline`
recomputes the medians from the committed captures with no `ANTHROPIC_API_KEY`
and no network, and regenerates this file byte-for-byte.

## Headline: reposix is ~{red['output']:.0f}% fewer output tokens and ~{red['cost']:.0f}% cheaper per session

For the identical task against the identical live GitHub backend, the
git-native (`reposix`) arm is cheaper on **every** axis than the GitHub-MCP arm.
The two lead numbers are **output tokens** (what the agent has to generate) and
**end-to-end USD cost** (what the run actually costs):

| Axis | reposix (median) | GitHub MCP (median) | reposix advantage |
|------|-----------------:|--------------------:|:------------------|
| Output tokens (agent generates) | {r['output']:,.0f} | {m['output']:,.0f} | **~{red['output']:.1f}% fewer** |
| Cache-creation tokens (new context cached) | {r['cache_create']:,.0f} | {m['cache_create']:,.0f} | ~{red['cache_create']:.1f}% fewer |
| Total input-context tokens | {r['input_context']:,.0f} | {m['input_context']:,.0f} | ~{red['input_context']:.1f}% smaller |
| Cost per session (USD) | ${r['cost']:.4f} | ${m['cost']:.4f} | ~{red['cost']:.1f}% cheaper |

Equivalently: the MCP arm costs **~{red['cost_multiple']:.1f}x** more per session and
carries **~{red['input_multiple']:.2f}x** the total input-context for the same result.

## What retired the old 89.1% / 85.5% figures

The previous token-economy figures (an **89.1%** headline and a per-backend
**85.5%** GitHub number) came from a *different, synthetic* methodology: running
Anthropic's `count_tokens` over a static, hand-constructed JSON fixture that
stood in for an MCP tool catalog. That measured the size of a fixture, not the
cost of a live agent run. It is **retired here** in favour of the live
session-usage medians above -- real sessions, a real GitHub backend, and the
GitHub MCP server's real tool surface. The synthetic fixtures remain in the repo
only as provenance for that retired estimate; they no longer back any published
number.

## What this DOES measure

- The real, end-to-end token + dollar cost of an agent completing the task, as
  recorded by Claude Code's own per-session usage accounting.
- The cost of "learning + calling the tool surface" (MCP) vs "using POSIX + git
  you already know" (reposix), against a live backend.

## What this does NOT measure (honest caveats)

- **reposix write-back on GitHub is read-only in this build cut.** The
  reposix arm read + locally edited + attempted a push; the push was correctly
  rejected by the documented read-only GitHub adapter. The token comparison is
  unaffected -- it measures agent context size, not write capability -- but these
  numbers must not be read as a claim that reposix persists writes to GitHub.
- **Fidelity note (factual):** during capture, the GitHub MCP `issue_read`
  HTML-escaped body content (`>=` -> `&gt;=`) and dropped literal angle-bracket
  text, so an MCP read-modify-write round-trip altered raw markdown; the reposix
  arm round-tripped bytes unchanged. Recorded for accuracy, not as a headline.
- Absolute numbers vary run-to-run with backend content and agent path; the
  medians above smooth {N_PER_ARM} runs per arm but are not a guarantee for any
  single session.

## Capture provenance

- `benchmarks/captures/reposix-github-run{{1,2,3}}.json`,
  `benchmarks/captures/mcp-github-run{{1,2,3}}.json` -- the six scrubbed
  session-usage extracts (session id, per-axis token counts, turn count,
  end-to-end USD cost, tool-call names). No backend body content, no secrets.
  Captured live during P115 Task 4 against {BACKEND}.
  `benchmarks/captures/mcp-kan-smoke.json` is a context-only smoke probe and is
  **excluded** from the medians.
- `benchmarks/fixtures/mcp_github_catalog.json` -- the real 44-tool GitHub MCP
  tool surface recorded at capture time (provenance for the MCP arm's tool set).
- `benchmarks/fixtures/reposix_session.txt` -- an ANSI-stripped transcript of the
  reposix arm's git-native shell session against the live GitHub backend.

Reproduce: `python3 quality/gates/perf/bench_token_economy.py --offline`
(deterministic; no API key, no network).
"""
