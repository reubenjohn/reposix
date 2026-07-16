#!/usr/bin/env python3
"""quality/gates/perf/headline-numbers-cross-check.py -- perf-dimension headline cross-check.

Cross-checks the headline numbers printed on the reposix HERO SURFACES against
their CANONICAL SOURCES, so a re-measurement of the benchmark docs can never
silently diverge from the marketing copy (or vice-versa).

  Hero surfaces (the pages a first-time reader lands on):
    * docs/index.md
    * README.md
    * docs/concepts/reposix-vs-mcp-and-sdks.md

  Canonical sources of truth (regenerated from measured data):
    * docs/benchmarks/latency.md      -- CI-canonical sim latency table
        Get one record (sim)  -> the "cached read" hero figure
        List records   (sim)  -> the "list" hero figure
    * docs/benchmarks/token-economy.md -- live GitHub-capture four-axis medians
        Output tokens reduction  -> the "~94% fewer output tokens" headline
        Cost per session reduction -> the "~75% cheaper" headline

The gate NEVER hardcodes the numbers: it PARSES them out of the canonical docs
at run time and compares. If someone re-measures latency.md to a new figure but
forgets the hero prose (the exact drift this closes -- an "8 ms" hero figure
lingering after latency.md moved to 6 ms / 7 ms), this gate goes RED and names
the surface line, the stale value, the canonical value, and the copy-paste fix.

Exit 0 = every hero headline matches its canonical source.
Exit 1 = at least one drift (teaching output on stderr/stdout).

Deterministic + offline: reads committed markdown only. No network, no API key.

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
    """Extract the sim-column 'Get one record' and 'List records' ms figures."""
    get_ms: Optional[int] = None
    list_ms: Optional[int] = None
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
    if get_ms is None or list_ms is None:
        raise SystemExit(
            f"headline-cross-check: could not parse canonical latency figures from "
            f"{LATENCY_DOC} (need a 'Get one record' and a 'List records' table row "
            f"with a sim-column 'N ms' cell). Got get={get_ms}, list={list_ms}. "
            f"If the table shape changed, update parse_latency_canonical()."
        )
    return {"get": get_ms, "list": list_ms}


def parse_token_canonical(text: str) -> dict:
    """Extract the output-token and cost reduction percentages (last table column)."""
    output_pct: Optional[float] = None
    cost_pct: Optional[float] = None
    for cells in _table_rows(text):
        if len(cells) < 2:
            continue
        label = cells[0].lower()
        m = re.search(r"(\d+(?:\.\d+)?)\s*%", cells[-1])  # reduction lives in the last column
        if not m:
            continue
        if "output tokens" in label:
            output_pct = float(m.group(1))
        elif "cost per session" in label:
            cost_pct = float(m.group(1))
    if output_pct is None or cost_pct is None:
        raise SystemExit(
            f"headline-cross-check: could not parse canonical token figures from "
            f"{TOKEN_DOC} (need an 'Output tokens ...' and a 'Cost per session ...' "
            f"table row with a trailing 'N% ...' reduction cell). Got "
            f"output={output_pct}, cost={cost_pct}. If the table shape changed, "
            f"update parse_token_canonical()."
        )
    return {"output": output_pct, "cost": cost_pct}


# ---------------------------------------------------------------------------
# Hero-surface claim registries.
#
# LATENCY_CLAIMS: each entry names a hero line whose `N ms` figure MUST equal a
# canonical latency axis. The regex has exactly one capture group (the integer
# ms). A claim that no longer matches its regex is itself a failure (the prose
# was restructured or the figure format changed) -- the gate refuses to silently
# pass a claim it can no longer locate.
# ---------------------------------------------------------------------------

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

    return failures, latency, token


def main(argv: Optional[list] = None) -> int:
    failures, latency, token = run_cross_check(REPO_ROOT)

    print("headline-numbers-cross-check — hero surfaces vs canonical sources")
    print(f"  canonical latency ({LATENCY_DOC}): get={latency['get']} ms, list={latency['list']} ms")
    print(
        f"  canonical token ({TOKEN_DOC}): output -{token['output']}%, cost -{token['cost']}%"
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
        f"\nPASS — all {len(LATENCY_CLAIMS)} latency + {len(TOKEN_CLAIMS)} token hero "
        f"headlines match their canonical sources."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
