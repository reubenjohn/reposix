#!/usr/bin/env python3
"""Validate the P84 webhook-latency.json artifact shape.

Required fields per plan's <must_haves> JSON template + the
T05-shipped `note` field (deviation Rule 4 — see SURPRISES-INTAKE
§ 2026-05-01 16:43 for the deferred-real-measurement framing).
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
ARTIFACT = REPO_ROOT / "quality" / "reports" / "verifications" / "perf" / "webhook-latency.json"


def main() -> int:
    d = json.loads(ARTIFACT.read_text())
    required = {
        "measured_at",
        "method",
        "n",
        "p50_seconds",
        "p95_seconds",
        "max_seconds",
        "target_seconds",
        "verdict",
    }
    missing = required - d.keys()
    assert not missing, f"missing fields: {missing}"
    assert d["p95_seconds"] <= 120, f"p95={d['p95_seconds']} > 120s threshold"
    assert d["verdict"] in ("PASS", "FAIL")
    assert d["method"] in (
        "synthetic-dispatch",
        "real-tokenworld-manual-edit",
    ), f"unexpected method: {d['method']}"
    assert d["n"] >= 1, f"n={d['n']} (expected >= 1)"
    print(
        f"webhook-latency.json valid: method={d['method']} n={d['n']} "
        f"p95={d['p95_seconds']}s verdict={d['verdict']}"
    )
    if "note" in d:
        print(f"NOTE: {d['note'][:140]}...")
    return 0


if __name__ == "__main__":
    sys.exit(main())
