#!/usr/bin/env python3
"""quality/gates/perf/headline-numbers-cross-check.py -- perf-dimension headline cross-check.

Cross-checks the headline numbers on the HERO SURFACES (docs/index.md, README.md,
docs/concepts/reposix-vs-mcp-and-sdks.md) against their CANONICAL SOURCES, so a
benchmark re-measurement can never silently diverge from the marketing copy:

    docs/benchmarks/latency.md (CI-canonical sim table):
        Get one record / List records / `reposix init` cold (sim)
        -> the "cached read" / "list" / "cold init" (278 ms) hero figures
    docs/benchmarks/token-economy.md (live GitHub-capture medians):
        Output/Cost reductions -> "~94% fewer output tokens" / "~75% cheaper"
        Output-token medians   -> the mermaid loop figures ("~1.2k" / "~21k")

The gate NEVER hardcodes the numbers: it PARSES them from the canonical docs at
run time. On drift it exits 1 and names the surface line, the stale value, the
canonical value, and a copy-paste fix. Exit 0 = every hero headline matches.
Deterministic + offline (committed markdown only; no network, no API key).

CATALOG ROW: quality/catalogs/perf-targets.json -> perf/headline-numbers-cross-check.
"""
from __future__ import annotations

import pathlib
import re
import sys
from typing import Optional

# Workspace root is three levels up from quality/gates/perf/.
REPO_ROOT = pathlib.Path(__file__).resolve().parents[3]

# Canonical sources of truth.
LATENCY_DOC = "docs/benchmarks/latency.md"
TOKEN_DOC = "docs/benchmarks/token-economy.md"

# Hero surfaces cross-checked.
HERO_INDEX = "docs/index.md"
HERO_README = "README.md"
HERO_CONCEPTS = "docs/concepts/reposix-vs-mcp-and-sdks.md"


# ---------------------------------------------------------------------------
# Canonical parsing (fail loud + structured if the doc shape changes -- the
# gate refuses to grade against a source it cannot read, per PROTOCOL Principle B).
# ---------------------------------------------------------------------------


def _table_rows(text: str):
    """Yield [cell, cell, ...] for each markdown table row in *text*."""
    for line in text.splitlines():
        stripped = line.strip()
        if not stripped.startswith("|"):
            continue
        cells = [c.strip() for c in stripped.strip("|").split("|")]
        if cells:
            yield cells


def parse_latency_canonical(text: str) -> dict:
    """Extract the sim-column 'Get one record', 'List records', and cold-init figures."""
    get_ms: Optional[int] = None
    list_ms: Optional[int] = None
    init_ms: Optional[int] = None
    for cells in _table_rows(text):
        if len(cells) < 2:
            continue
        label = cells[0].lower()
        m = re.search(r"(\d+)\s*ms", cells[1])  # cells[1] == sim column
        if not m:
            continue
        if "get one record" in label:
            get_ms = int(m.group(1))
        elif "list records" in label:
            list_ms = int(m.group(1))
        elif "reposix init" in label and "cold" in label:
            init_ms = int(m.group(1))
    if get_ms is None or list_ms is None or init_ms is None:
        raise SystemExit(
            f"headline-cross-check: could not parse canonical latency figures from "
            f"{LATENCY_DOC} (need a 'Get one record', a 'List records', and a "
            f"'reposix init cold' table row with a sim-column 'N ms' cell). Got "
            f"get={get_ms}, list={list_ms}, init={init_ms}. "
            f"If the table shape changed, update parse_latency_canonical()."
        )
    return {"get": get_ms, "list": list_ms, "init": init_ms}


def _int_cell(cell: str) -> Optional[int]:
    """Parse a comma-grouped integer token count out of a table cell (e.g. '21,171')."""
    m = re.search(r"(\d[\d,]*)", cell)
    return int(m.group(1).replace(",", "")) if m else None


def parse_token_canonical(text: str) -> dict:
    """Extract the output-token / cost reduction percentages AND the absolute
    output-token medians (reposix + MCP) for the loop-figure cross-check.

    Table shape (docs/benchmarks/token-economy.md):
        | Axis | reposix (median) | GitHub MCP (median) | reposix advantage |
        | Output tokens ... | 1,213 | 21,171 | **~94.3% fewer** |
    """
    output_pct: Optional[float] = None
    cost_pct: Optional[float] = None
    output_reposix: Optional[int] = None
    output_mcp: Optional[int] = None
    for cells in _table_rows(text):
        if len(cells) < 2:
            continue
        label = cells[0].lower()
        m = re.search(r"(\d+(?:\.\d+)?)\s*%", cells[-1])  # reduction lives in the last column
        if not m:
            continue
        if "output tokens" in label:
            output_pct = float(m.group(1))
            if len(cells) >= 4:  # reposix median = col 1, MCP median = col 2
                output_reposix = _int_cell(cells[1])
                output_mcp = _int_cell(cells[2])
        elif "cost per session" in label:
            cost_pct = float(m.group(1))
    if output_pct is None or cost_pct is None or output_reposix is None or output_mcp is None:
        raise SystemExit(
            f"headline-cross-check: could not parse canonical token figures from "
            f"{TOKEN_DOC} (need an 'Output tokens ...' row with reposix + MCP median "
            f"columns and a trailing 'N% ...' reduction cell, plus a 'Cost per session "
            f"...' reduction row). Got output={output_pct}, cost={cost_pct}, "
            f"output_reposix={output_reposix}, output_mcp={output_mcp}. If the table "
            f"shape changed, update parse_token_canonical()."
        )
    return {
        "output": output_pct,
        "cost": cost_pct,
        "output_reposix": output_reposix,
        "output_mcp": output_mcp,
    }


# Hero-surface claim registries. LATENCY_CLAIMS: each entry names a hero line
# whose `N ms` figure MUST equal a canonical latency axis; the regex has one
# capture group (the integer ms). A claim whose regex no longer matches is itself
# a failure (prose restructured / format changed) -- the gate never silently
# passes a claim it cannot locate.

LATENCY_CLAIMS = [
    {
        "file": HERO_INDEX,
        "label": "hero card 'cached read'",
        "regex": r"\*\*`(\d+) ms`\*\* cached read",
        "axis": "get",
    },
    {
        "file": HERO_INDEX,
        "label": "'Tested against' cache read",
        "regex": r"reposix's `(\d+) ms` cache read",
        "axis": "get",
    },
    {
        "file": HERO_INDEX,
        "label": "'Tested against' list-issues",
        "regex": r"list-issues `(\d+) ms`",
        "axis": "list",
    },
    {
        "file": HERO_README,
        "label": "'Three measured numbers' simulator-measured caveat",
        "regex": r"`(\d+) ms` / `\d+ ms` are simulator-measured",
        "axis": "get",
    },
    {
        "file": HERO_README,
        "label": "'read one issue from the local cache' bullet",
        "regex": r"\*\*`(\d+) ms`\*\* — read one issue",
        "axis": "get",
    },
    {
        "file": HERO_CONCEPTS,
        "label": "'Latency, cached read' comparison-table row",
        "regex": r"\*\*Latency, cached read\*\* \| `(\d+) ms`",
        "axis": "get",
    },
    # --- cold-init (canonical: latency.md 'reposix init' cold sim, currently 278 ms) ---
    {
        "file": HERO_INDEX,
        "label": "hero card 'cold init'",
        "regex": r"\*\*`(\d+) ms`\*\* cold init",
        "axis": "init",
    },
    {
        "file": HERO_INDEX,
        "label": "'Tested against' sim cold init",
        "regex": r"Sim cold init is `(\d+) ms`",
        "axis": "init",
    },
    {
        "file": HERO_README,
        "label": "'reposix init cold bootstrap' bullet",
        "regex": r"\*\*`(\d+) ms`\*\* — `reposix init` cold bootstrap",
        "axis": "init",
    },
    {
        "file": HERO_CONCEPTS,
        "label": "'Latency, cold init' comparison-table row",
        "regex": r"\*\*Latency, cold init / first call\*\* \| `(\d+) ms` cold init",
        "axis": "init",
    },
]

# TOKEN_LOOP_CLAIMS: the two homepage mermaid-note loop figures assert the ABSOLUTE
# per-arm output-token medians. `{reposix_k_1dp}` = reposix median / 1000 to 1dp
# (1,213 -> "1.2"); `{mcp_k_int}` = MCP median / 1000 rounded (21,171 -> "21").
TOKEN_LOOP_CLAIMS = [
    {
        "file": HERO_INDEX,
        "template": "git-native loop · ~{reposix_k_1dp}k output tokens (live)",
        "axis": "output_reposix",
    },
    {
        "file": HERO_INDEX,
        "template": "MCP tool loop · ~{mcp_k_int}k output tokens (live)",
        "axis": "output_mcp",
    },
]

# TOKEN_CLAIMS: each entry asserts a canonical-derived substring is present on a
# hero surface. `{out_round}`/`{cost_round}` are the nearest-integer reductions
# (the rounded "~94%"/"~75%" headline style); `{out_1dp}`/`{cost_1dp}` are the
# one-decimal figures (the precise "~94.3%"/"~74.9%" style the concepts page uses).
TOKEN_CLAIMS = [
    {"file": HERO_INDEX, "template": "~{out_round}% fewer output tokens", "axis": "output"},
    {"file": HERO_INDEX, "template": "~{cost_round}% cheaper", "axis": "cost"},
    {"file": HERO_README, "template": "~{out_round}% fewer output tokens", "axis": "output"},
    {"file": HERO_README, "template": "~{cost_round}% cheaper per session", "axis": "cost"},
    {"file": HERO_CONCEPTS, "template": "~{out_1dp}% fewer output", "axis": "output"},
    {"file": HERO_CONCEPTS, "template": "~{cost_1dp}% cheaper", "axis": "cost"},
]


def _fmt1(value: float) -> str:
    return f"{value:.1f}"


def _find_claim(path: pathlib.Path, regex: str):
    """Return (lineno, int_value) of the first regex match, or None."""
    pat = re.compile(regex)
    for lineno, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        m = pat.search(line)
        if m:
            return lineno, int(m.group(1))
    return None


def run_cross_check(repo_root: pathlib.Path):
    """Return (failures, latency, token). failures is a list of teaching strings."""
    failures: list[str] = []
    latency = parse_latency_canonical((repo_root / LATENCY_DOC).read_text(encoding="utf-8"))
    token = parse_token_canonical((repo_root / TOKEN_DOC).read_text(encoding="utf-8"))

    axis_source = {
        "get": f"{LATENCY_DOC} 'Get one record' (sim)",
        "list": f"{LATENCY_DOC} 'List records' (sim)",
        "init": f"{LATENCY_DOC} 'reposix init' cold (sim)",
    }

    # --- Latency cross-check ---
    for claim in LATENCY_CLAIMS:
        path = repo_root / claim["file"]
        canonical = latency[claim["axis"]]
        hit = _find_claim(path, claim["regex"])
        if hit is None:
            failures.append(
                f"{claim['file']} — could NOT locate the {claim['label']} latency "
                f"figure (regex {claim['regex']!r} matched nothing). Either the prose "
                f"was restructured or the `N ms` format changed; the canonical value is "
                f"`{canonical} ms` from {axis_source[claim['axis']]}. Fix: restore a "
                f"`{canonical} ms` figure matching the expected phrasing, or update this "
                f"gate's regex if the copy was intentionally reworded."
            )
            continue
        lineno, value = hit
        if value != canonical:
            failures.append(
                f"{claim['file']}:{lineno} — {claim['label']} shows `{value} ms` but "
                f"canonical is `{canonical} ms` ({axis_source[claim['axis']]}). "
                f"Fix: change the `{value} ms` figure on {claim['file']}:{lineno} to "
                f"`{canonical} ms` (or re-measure and regenerate {LATENCY_DOC} if the "
                f"canonical figure genuinely moved)."
            )

    # --- Token-economy cross-check ---
    fmt = {
        "out_round": round(token["output"]),
        "cost_round": round(token["cost"]),
        "out_1dp": _fmt1(token["output"]),
        "cost_1dp": _fmt1(token["cost"]),
    }
    axis_token_source = {
        "output": f"{TOKEN_DOC} 'Output tokens' reduction ({token['output']}%)",
        "cost": f"{TOKEN_DOC} 'Cost per session' reduction ({token['cost']}%)",
    }
    for claim in TOKEN_CLAIMS:
        expected = claim["template"].format(**fmt)
        text = (repo_root / claim["file"]).read_text(encoding="utf-8")
        if expected not in text:
            failures.append(
                f"{claim['file']} — expected the canonical token headline substring "
                f"{expected!r} (derived from {axis_token_source[claim['axis']]}) but it "
                f"is absent. Fix: update the token headline on {claim['file']} to match "
                f"the canonical figure, or regenerate {TOKEN_DOC} if it genuinely moved."
            )

    # --- Loop-figure cross-check (absolute output-token medians) ---
    loop_fmt = {
        "reposix_k_1dp": _fmt1(token["output_reposix"] / 1000),
        "mcp_k_int": round(token["output_mcp"] / 1000),
    }
    loop_source = {
        "output_reposix": f"{TOKEN_DOC} 'Output tokens' reposix median ({token['output_reposix']:,})",
        "output_mcp": f"{TOKEN_DOC} 'Output tokens' MCP median ({token['output_mcp']:,})",
    }
    for claim in TOKEN_LOOP_CLAIMS:
        expected = claim["template"].format(**loop_fmt)
        text = (repo_root / claim["file"]).read_text(encoding="utf-8")
        if expected not in text:
            failures.append(
                f"{claim['file']} — expected the canonical loop-figure substring "
                f"{expected!r} (derived from {loop_source[claim['axis']]}) but it is "
                f"absent. Fix: update the loop figure on {claim['file']} to match the "
                f"captures-computed median, or regenerate {TOKEN_DOC} if it genuinely moved."
            )

    return failures, latency, token


def main(argv: Optional[list] = None) -> int:
    failures, latency, token = run_cross_check(REPO_ROOT)

    print("headline-numbers-cross-check — hero surfaces vs canonical sources")
    print(
        f"  canonical latency ({LATENCY_DOC}): get={latency['get']} ms, "
        f"list={latency['list']} ms, init={latency['init']} ms"
    )
    print(
        f"  canonical token ({TOKEN_DOC}): output -{token['output']}%, cost -{token['cost']}%; "
        f"medians reposix={token['output_reposix']:,} MCP={token['output_mcp']:,} output tokens"
    )
    print(
        f"  hero surfaces checked: {HERO_INDEX}, {HERO_README}, {HERO_CONCEPTS}"
    )

    if failures:
        print(f"\nFAIL — {len(failures)} headline drift(s) detected:\n", file=sys.stderr)
        for i, f in enumerate(failures, start=1):
            print(f"  [{i}] {f}\n", file=sys.stderr)
        return 1

    print(
        f"\nPASS — all {len(LATENCY_CLAIMS)} latency + {len(TOKEN_CLAIMS)} token + "
        f"{len(TOKEN_LOOP_CLAIMS)} loop-figure hero headlines match their canonical sources."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
