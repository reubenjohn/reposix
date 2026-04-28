#!/usr/bin/env python3
# KEEP-AS-CANONICAL (P63 SIMPLIFY-12): canonical for own domain (P56 install-evidence validator); CLAUDE.md P56 section names this exact path.
"""P56 RELEASE-01..03 — install-evidence JSON validator.

Validates each `.planning/verifications/p56/install-paths/<row>.json` file
against the schema implied by Plan 56-03's Task verifier-blocks. This is the
gate runner the verifier subagent (Wave 4) will re-run with zero session
context — same script as the executing agent's gate, no narrative drift.

Why a script not an inline `python3 -c '...'`: the agent-context one-liner
is repeated five times (one per install-path row) and is itself the per-row
asserts. Encoding it as a script means:
  1. Wave 4 can rerun the validator with a single command.
  2. The asserts are reviewed once, not by reading five inline strings.
  3. Future P56-equivalent gates (e.g. v0.12.1 RELEASE-04) reuse the table.

Usage:
    python3 scripts/p56-validate-install-evidence.py

    python3 scripts/p56-validate-install-evidence.py \\
        --row install/curl-installer-sh

Exit codes:
    0 = all rows in scope PASS (or PARTIAL where the table allows it)
    1 = at least one row FAIL or schema-broken
    2 = at least one row missing
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE_DIR = ROOT / ".planning" / "verifications" / "p56" / "install-paths"

# Per-row gate spec, derived from Plan 56-03's Task verifier-blocks.
# `path_under_evidence_dir`: relative file
# `required_status_set`: terminal statuses that count as a gate-pass
# `asserts`: dotted-path → required-value mapping. Use bool / str / lambda.
GATES = {
    "install/curl-installer-sh": {
        "path": "curl-installer-sh.json",
        "required_status_set": {"PASS"},
        "asserts": {
            "claim_id": "install/curl-installer-sh",
            "asserts.container_oneliner_exit_zero": True,
            "asserts.command_v_reposix_exit_zero": True,
            "asserts.command_v_git_remote_reposix_exit_zero": True,
            "asserts.reposix_version_matches_release_tag": True,
        },
    },
    "install/powershell-installer-ps1": {
        "path": "powershell-installer-ps1.json",
        "required_status_set": {"PASS"},
        "asserts": {
            "claim_id": "install/powershell-installer-ps1",
            "asserts.http_200": True,
            "asserts.content_length_gte_1024": True,
        },
    },
    "install/cargo-binstall": {
        "path": "cargo-binstall.json",
        # Catalog row baseline (.planning/docs_reproducible_catalog.json
        # claim install/cargo-binstall) is PARTIAL with blast_radius P1
        # (works, just slow). The validator admits both PASS (clean lift)
        # and PARTIAL (no regression vs catalog baseline).
        "required_status_set": {"PASS", "PARTIAL"},
        # No per-assertion truth requirements: the row is allowed to
        # PARTIAL with all asserts false because the catalog already
        # marked this row P1 with a documented fall-back behaviour. The
        # gate_disposition_note + failure_mode + recovery blocks in the
        # JSON carry the verifier signal.
        "asserts": {
            "claim_id": "install/cargo-binstall",
        },
    },
    "install/homebrew": {
        "path": "homebrew.json",
        "required_status_set": {"PASS"},
        "asserts": {
            "claim_id": "install/homebrew",
            "asserts.formula_version_equals_release_version": True,
            "asserts.formula_has_three_sha256_strings": True,
        },
    },
    "install/build-from-source": {
        "path": "build-from-source.json",
        "required_status_set": {"PASS"},
        "asserts": {
            "claim_id": "install/build-from-source",
            "asserts.ci_test_job_green_on_main": True,
        },
    },
}


def get_path(d: dict, dotted: str):
    cur = d
    for part in dotted.split("."):
        if not isinstance(cur, dict) or part not in cur:
            return None
        cur = cur[part]
    return cur


def grade(row_id: str) -> tuple[str, list[str]]:
    spec = GATES[row_id]
    f = EVIDENCE_DIR / spec["path"]
    if not f.exists():
        return "MISSING", [f"file not found: {f.relative_to(ROOT)}"]
    try:
        d = json.loads(f.read_text())
    except (json.JSONDecodeError, OSError) as exc:
        return "BROKEN", [f"json parse failure: {exc}"]
    failures: list[str] = []
    status = d.get("status")
    if status not in spec["required_status_set"]:
        failures.append(
            f"status={status!r} not in {sorted(spec['required_status_set'])}"
        )
    for dotted, expected in spec["asserts"].items():
        actual = get_path(d, dotted)
        if actual != expected:
            failures.append(f"{dotted}={actual!r} (want {expected!r})")
    if failures:
        return "FAIL", failures
    return "PASS", []


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--row",
        action="append",
        choices=sorted(GATES.keys()),
        help="grade only the named row (default: all)",
    )
    args = parser.parse_args()
    rows = args.row or sorted(GATES.keys())

    rc = 0
    print(f"P56 install-evidence validator — {len(rows)} row(s)")
    print(f"evidence dir: {EVIDENCE_DIR.relative_to(ROOT)}")
    print()
    for row_id in rows:
        outcome, failures = grade(row_id)
        if outcome == "PASS":
            print(f"  PASS  {row_id}")
        elif outcome == "MISSING":
            print(f"  MISS  {row_id}")
            for f in failures:
                print(f"          - {f}")
            rc = max(rc, 2)
        else:
            print(f"  FAIL  {row_id}  ({outcome})")
            for f in failures:
                print(f"          - {f}")
            rc = max(rc, 1)
    print()
    if rc == 0:
        print("OK — all rows pass their gate")
    else:
        print(f"FAIL — exit code {rc}")
    return rc


if __name__ == "__main__":
    sys.exit(main())
