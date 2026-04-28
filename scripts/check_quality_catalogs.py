#!/usr/bin/env python3
"""Validate quality/catalogs/*.json structural invariants.

Per quality/PROTOCOL.md Step 3 (catalog-first), the catalog files define
end-state contracts that downstream waves and the unbiased verifier
subagent grade against. This script enforces the structural invariants
so a hand-edit can't silently drift the contract.

Stdlib only. Cross-platform.

Usage:
  python3 scripts/check_quality_catalogs.py            # check every catalog
  python3 scripts/check_quality_catalogs.py --catalog release-assets

Exit codes:
  0 — every checked catalog matches the contract.
  1 — at least one invariant violated.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
CATALOG_DIR = REPO_ROOT / "quality" / "catalogs"

# (catalog stem, dimension, expected row count, required row ids,
#  allowed cadences, post-validate hook)
CONTRACTS = {
    "release-assets": {
        "dimension": "release",
        # P58 Wave A removed reposix-swarm (publish=false) -> 15 rows.
        "row_count": 15,
        "required_ids": {
            "install/curl-installer-sh",
            "install/powershell-installer-ps1",
            "install/homebrew",
            "install/build-from-source",
            "release/crates-io-max-version/reposix-cli",
            "release/crates-io-max-version/reposix-remote",
            "release/crates-io-max-version/reposix-core",
            "release/crates-io-max-version/reposix-cache",
            "release/crates-io-max-version/reposix-sim",
            "release/crates-io-max-version/reposix-github",
            "release/crates-io-max-version/reposix-confluence",
            "release/crates-io-max-version/reposix-jira",
            "release/gh-assets-present",
            "release/brew-formula-current",
            "release/cargo-binstall-resolves",
        },
        "allowed_cadences": {"weekly", "post-release"},
        "all_status": "NOT-VERIFIED",
        "binstall_check": True,
    },
    "code": {
        "dimension": "code",
        # P58 Wave C + P60 added clippy-warnings, fmt-check, fixtures-valid.
        # Required-ids enforces presence of POLISH-CODE rows; extras allowed.
        "row_count": 6,
        "required_ids": {
            "code/clippy-lint-loaded",
            "code/cargo-test-pass",
            "code/cargo-fmt-clean",
            "code/fixtures-valid",
            "code/cargo-fmt-check",
            "code/cargo-clippy-warnings",
        },
        "allowed_cadences": {"pre-push", "pre-pr"},
        "all_status": "NOT-VERIFIED",
        "code_waivers_check": True,
    },
    "orphan-scripts": {
        "dimension": "meta",
        # P58 Wave E removed the release/crates-io-max-version waiver row;
        # P63 Wave 1 locks the schema with rows=[] (Wave 2 populates after
        # caller-scan).
        "row_count": 0,
        "required_ids": set(),
        "allowed_cadences": {"pre-push"},
        "all_status": "WAIVED",
        "orphan_lineage_check": False,
    },
}

REQUIRED_ROW_FIELDS = (
    "id", "dimension", "cadence", "kind", "sources", "command", "expected",
    "verifier", "artifact", "status", "last_verified", "freshness_ttl",
    "blast_radius", "owner_hint", "waiver",
)


def fail(msg: str, errors: list) -> None:
    errors.append(msg)


def check_catalog(stem: str, contract: dict, errors: list) -> None:
    path = CATALOG_DIR / f"{stem}.json"
    if not path.exists():
        fail(f"{stem}: file does not exist at {path}", errors)
        return
    try:
        d = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as e:
        fail(f"{stem}: invalid JSON: {e}", errors)
        return

    if d.get("dimension") != contract["dimension"]:
        fail(f"{stem}: wrapper dimension={d.get('dimension')!r} expected={contract['dimension']!r}", errors)

    rows = d.get("rows") or []
    if len(rows) != contract["row_count"]:
        fail(f"{stem}: expected {contract['row_count']} rows, got {len(rows)}", errors)

    ids = {r.get("id") for r in rows}
    missing = contract["required_ids"] - ids
    extra = ids - contract["required_ids"]
    if missing:
        fail(f"{stem}: missing row ids: {sorted(missing)}", errors)
    if extra:
        fail(f"{stem}: unexpected row ids: {sorted(extra)}", errors)

    cadences = {r.get("cadence") for r in rows}
    bad_cadences = cadences - contract["allowed_cadences"]
    if bad_cadences:
        fail(f"{stem}: unexpected cadences: {sorted(bad_cadences)}", errors)

    valid_statuses = {"NOT-VERIFIED", "PASS", "FAIL", "PARTIAL", "WAIVED"}
    for r in rows:
        for field in REQUIRED_ROW_FIELDS:
            if field not in r:
                fail(f"{stem}: row {r.get('id')!r} missing field {field!r}", errors)
        status = r.get("status")
        if status not in valid_statuses:
            fail(
                f"{stem}: row {r.get('id')!r} status={status!r} "
                f"not in {sorted(valid_statuses)}",
                errors,
            )

    if contract.get("binstall_check"):
        binstall = next((r for r in rows if r.get("id") == "release/cargo-binstall-resolves"), None)
        if not binstall:
            fail(f"{stem}: release/cargo-binstall-resolves row missing", errors)
        else:
            if binstall.get("cadence") != "post-release":
                fail(f"{stem}: cargo-binstall-resolves cadence must be post-release", errors)
            if binstall.get("kind") != "container":
                fail(f"{stem}: cargo-binstall-resolves kind must be container", errors)

    if contract.get("code_waivers_check"):
        clippy = next((r for r in rows if r.get("id") == "code/clippy-lint-loaded"), None)
        if clippy:
            w = clippy.get("waiver")
            # P58 Wave C closed; clippy must be actively enforced (waiver=null).
            if w is not None:
                fail(
                    f"{stem}: code/clippy-lint-loaded waiver must be null "
                    f"(active enforcement post-P58 Wave C)",
                    errors,
                )
        # P63 Wave 1 wiring contract:
        # - cargo-fmt-clean: waiver MUST be null (Wave 3 wires direct cargo fmt).
        # - cargo-test-pass: waiver MUST be present + tracked_in MUST start
        #   with 'v0.12.1 MIGRATE-03' (memory-budget rationale; CI is
        #   canonical enforcement venue).
        cf = next((r for r in rows if r.get("id") == "code/cargo-fmt-clean"), None)
        if cf is not None and cf.get("waiver") is not None:
            fail(
                f"{stem}: code/cargo-fmt-clean waiver must be null "
                f"(P63 wired direct cargo fmt -- read-only ~5s, ONE cargo at a time safe)",
                errors,
            )
        ct = next((r for r in rows if r.get("id") == "code/cargo-test-pass"), None)
        if ct is not None:
            w = ct.get("waiver")
            if not w or not str(w.get("tracked_in", "")).startswith("v0.12.1 MIGRATE-03"):
                fail(
                    f"{stem}: code/cargo-test-pass waiver.tracked_in must start with "
                    f"'v0.12.1 MIGRATE-03' (per-row local cargo enforcement deferred)",
                    errors,
                )

    if contract.get("orphan_lineage_check"):
        if not rows:
            return
        r = rows[0]
        oh = r.get("owner_hint", "")
        if "P58 Wave A" not in oh:
            fail(f"{stem}: owner_hint missing 'P58 Wave A' breadcrumb", errors)
        if "Wave E flips this waiver to resolved" not in oh:
            fail(f"{stem}: owner_hint missing 'Wave E flips this waiver to resolved' breadcrumb", errors)
        w = r.get("waiver") or {}
        if w.get("until") != "2026-05-15T00:00:00Z":
            fail(f"{stem}: waiver.until must be '2026-05-15T00:00:00Z' (preserved from P57)", errors)
        if r.get("status") != "WAIVED":
            fail(f"{stem}: status must remain WAIVED", errors)


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate quality/catalogs/*.json invariants")
    parser.add_argument(
        "--catalog",
        choices=tuple(CONTRACTS.keys()),
        default=None,
        help="check a single catalog by stem (default: check all)",
    )
    args = parser.parse_args()

    targets = [args.catalog] if args.catalog else list(CONTRACTS.keys())
    errors: list = []
    for stem in targets:
        check_catalog(stem, CONTRACTS[stem], errors)

    if errors:
        print("FAIL — quality catalog invariants violated:", file=sys.stderr)
        for e in errors:
            print(f"  - {e}", file=sys.stderr)
        return 1
    counts = {stem: CONTRACTS[stem]["row_count"] for stem in targets}
    print(f"PASS — quality catalog invariants OK ({counts})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
