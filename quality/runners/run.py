#!/usr/bin/env python3
"""Quality Gates runner — single entry point for cadence-tagged gates.

Per .planning/research/v0.12.0/naming-and-architecture.md § runner-contract.
Stdlib only. Cross-platform (linux + macOS).
Anti-bloat: this file does not grow beyond the runner contract. Per-dimension
verifiers live under quality/gates/<dim>/.

Usage:
  python3 quality/runners/run.py --cadence <pre-push|pre-pr|weekly|pre-release|post-release|on-demand>

Exit codes:
  0 — every P0+P1 row in scope is PASS or WAIVED.
  1 — any P0+P1 row in scope is FAIL/PARTIAL/NOT-VERIFIED.

Cadence-specific notes (P61 SUBJ-03):
  - pre-release: this is the cadence where freshness-TTL enforcement
    materially gates a release. STALE subagent-graded rows (kind=
    subagent-graded with expired freshness_ttl) flip to NOT-VERIFIED;
    compute_exit_code treats P0+P1 NOT-VERIFIED as RED. The pre-release
    workflow at .github/workflows/quality-pre-release.yml fails the
    release with a hint pointing the maintainer at the dispatcher
    (`bash .claude/skills/reposix-quality-review/dispatch.sh --all-stale
    --force`). Auto-dispatch from CI (would require Anthropic API auth
    on GH Actions runners) is a v0.12.1 carry-forward via MIGRATE-03.
  - weekly: STALE rows raise visibility but P2 rows do not block exit
    per compute_exit_code's existing P0+P1-only gating.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

# P61 SUBJ-03: freshness helpers extracted to a sibling module per the
# Wave B pivot rule (anti-bloat cap on run.py).
from _freshness import parse_duration as _parse_duration_impl
from _freshness import is_stale as _is_stale_impl

# Re-export so existing callers and tests can keep doing `from run import parse_duration`.
parse_duration = _parse_duration_impl

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
CATALOG_DIR = REPO_ROOT / "quality" / "catalogs"
REPORTS_DIR = REPO_ROOT / "quality" / "reports"
BLAST_RADIUS_ORDER = {"P0": 0, "P1": 1, "P2": 2}

VALID_CADENCES = (
    "pre-push", "pre-pr", "weekly", "pre-release", "post-release", "on-demand",
)


def now_iso() -> str:
    """Return current UTC time as RFC3339 with Z suffix."""
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def parse_rfc3339(s: str) -> datetime:
    """Parse RFC3339 with or without Z; return tz-aware datetime."""
    if s.endswith("Z"):
        s = s[:-1] + "+00:00"
    return datetime.fromisoformat(s)


def discover_catalogs() -> list[Path]:
    """Glob catalog files. Skip orphan-scripts.json + allow-list sidecars + README.md."""
    out: list[Path] = []
    for p in sorted(CATALOG_DIR.glob("*.json")):
        if p.stem == "orphan-scripts" or p.stem.endswith("-allowlist"):
            continue
        out.append(p)
    return out


def load_catalog(path: Path) -> dict:
    """Load one catalog file; verify wrapper has dimension + rows keys."""
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as e:
        raise SystemExit(f"FAIL: {path}: invalid JSON: {e}") from e
    for k in ("dimension", "rows"):
        if k not in data:
            raise SystemExit(f"FAIL: {path}: wrapper missing {k!r}")
    return data


def save_catalog(path: Path, data: dict) -> None:
    """Write catalog back; preserve wrapper field order ($schema, comment, dimension, rows).

    Uses ensure_ascii=False so em-dashes and other Unicode survive round-trips.
    Caller decides whether to invoke save_catalog() — see catalog_dirty().
    """
    ordered: dict[str, Any] = {}
    for key in ("$schema", "comment", "dimension", "rows"):
        if key in data:
            ordered[key] = data[key]
    for key, val in data.items():
        if key not in ordered:
            ordered[key] = val
    path.write_text(
        json.dumps(ordered, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )


def catalog_dirty(original: dict, updated: dict) -> bool:
    """True iff any row's status changed. Timestamp-only updates do not count
    as a semantic change — they belong in the artifact, not the committed
    catalog. This keeps `git status` clean across pre-push runs that just
    re-confirm an already-known status."""
    orig_rows = {r["id"]: r.get("status") for r in original.get("rows", [])}
    new_rows = {r["id"]: r.get("status") for r in updated.get("rows", [])}
    return orig_rows != new_rows


def is_in_scope(row: dict, cadence: str, now: datetime) -> bool:
    """Row applies to this cadence AND any waiver is still in effect (or absent)."""
    if row.get("cadence") != cadence:
        return False
    return True  # waiver doesn't drop scope; it just changes status to WAIVED


def waiver_active(row: dict, now: datetime) -> bool:
    waiver = row.get("waiver")
    if waiver is None:
        return False
    until = waiver.get("until")
    if not until:
        return False
    try:
        return parse_rfc3339(until) > now
    except (ValueError, TypeError):
        return False


def is_stale(row: dict, now: datetime) -> bool:
    """Thin wrapper around _freshness.is_stale; injects parse_rfc3339."""
    return _is_stale_impl(row, now, parse_rfc3339)


def sort_by_blast_radius(rows: list[dict]) -> list[dict]:
    return sorted(rows, key=lambda r: BLAST_RADIUS_ORDER.get(r.get("blast_radius", "P2"), 99))


def write_artifact(path: Path, data: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")


def run_row(row: dict, repo_root: Path, now: datetime) -> tuple[dict, float]:
    """Invoke verifier (or short-circuit), write artifact, return (updated row, elapsed_s).

    Status mutation policy: the runner sets the in-memory `status` and
    `last_verified` for use by the immediate caller (verdict generation).
    Whether those changes get persisted back to the catalog file is
    decided by `catalog_dirty()` in `main()` — only meaningful status
    flips persist; per-run timestamp churn lives in the artifact, not
    the catalog. This keeps `git status` clean across repeated pre-push
    runs that just re-confirm an already-known status.
    """
    started = time.monotonic()
    artifact_path = repo_root / row["artifact"] if row.get("artifact") else None

    # WAIVED case
    if waiver_active(row, now):
        artifact = {
            "ts": now_iso(),
            "row_id": row["id"],
            "exit_code": 0,
            "waiver": row["waiver"],
            "asserts_passed": [],
            "asserts_failed": [],
        }
        if artifact_path:
            write_artifact(artifact_path, artifact)
        row["status"] = "WAIVED"
        row["last_verified"] = artifact["ts"]
        return row, time.monotonic() - started

    # STALE case (freshness TTL expired). Per P61 SUBJ-03: STALE is a flavor
    # of NOT-VERIFIED with a clearer label; compute_exit_code semantics
    # unchanged (P0+P1 NOT-VERIFIED -> exit 1; P2 NOT-VERIFIED -> exit 0).
    if is_stale(row, now):
        artifact = {
            "ts": now_iso(),
            "row_id": row["id"],
            "exit_code": None,
            "stale": True,
            "freshness_ttl": row["freshness_ttl"],
            "last_verified_input": row["last_verified"],
            "asserts_passed": [],
            "asserts_failed": [
                f"freshness expired: last_verified={row['last_verified']} + ttl={row['freshness_ttl']} < now"
            ],
        }
        if artifact_path:
            write_artifact(artifact_path, artifact)
        row["status"] = "NOT-VERIFIED"
        row["_stale"] = True  # transient flag for print_row_summary; not persisted
        row["last_verified"] = artifact["ts"]
        return row, time.monotonic() - started

    # Verifier-not-found case
    script_rel = row.get("verifier", {}).get("script")
    script_abs = (repo_root / script_rel) if script_rel else None
    if not script_abs or not script_abs.exists():
        artifact = {
            "ts": now_iso(),
            "row_id": row["id"],
            "exit_code": None,
            "error": f"verifier not found at {script_rel}",
            "asserts_passed": [],
            "asserts_failed": [],
        }
        if artifact_path:
            write_artifact(artifact_path, artifact)
        # Status stays whatever the catalog had (likely NOT-VERIFIED already).
        # Don't flip from PASS->NOT-VERIFIED on a missing verifier — that
        # would be a regression-on-deploy disguised as runner output.
        if row.get("status") not in ("PASS", "FAIL", "PARTIAL"):
            row["status"] = "NOT-VERIFIED"
        row["last_verified"] = artifact["ts"]
        return row, time.monotonic() - started

    # Subprocess case
    args = row.get("verifier", {}).get("args", []) or []
    timeout_s = int(row.get("verifier", {}).get("timeout_s", 30))
    suffix = script_abs.suffix
    if suffix == ".py":
        cmd = [sys.executable, str(script_abs), *args]
    elif suffix in (".sh", ""):
        cmd = ["bash", str(script_abs), *args]
    else:
        cmd = [str(script_abs), *args]

    timed_out = False
    stdout = ""
    stderr = ""
    exit_code: int | None = None
    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=timeout_s,
            cwd=str(repo_root),
            shell=False,
            check=False,
        )
        exit_code = result.returncode
        stdout = result.stdout
        stderr = result.stderr
    except subprocess.TimeoutExpired as e:
        timed_out = True
        stdout = e.stdout.decode("utf-8", "replace") if isinstance(e.stdout, (bytes, bytearray)) else (e.stdout or "")
        stderr = e.stderr.decode("utf-8", "replace") if isinstance(e.stderr, (bytes, bytearray)) else (e.stderr or "")

    # Try to parse the artifact the verifier may have written (it has authoritative
    # asserts_passed / asserts_failed). If verifier wrote it, we keep its body and
    # only annotate top-level metadata. Otherwise we synthesize.
    artifact: dict[str, Any] = {}
    if artifact_path and artifact_path.exists():
        try:
            artifact = json.loads(artifact_path.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            artifact = {}
    artifact.setdefault("ts", now_iso())
    artifact["row_id"] = row["id"]
    artifact["exit_code"] = None if timed_out else exit_code
    artifact["timed_out"] = timed_out
    artifact.setdefault("stdout", stdout)
    artifact.setdefault("stderr", stderr)
    artifact.setdefault("asserts_passed", [])
    artifact.setdefault("asserts_failed", [])
    if artifact_path:
        write_artifact(artifact_path, artifact)

    if timed_out:
        row["status"] = "FAIL"
    elif exit_code == 0:
        row["status"] = "PASS"
    elif exit_code == 2:
        row["status"] = "PARTIAL"
    else:
        row["status"] = "FAIL"
    row["last_verified"] = artifact["ts"]
    return row, time.monotonic() - started


def compute_exit_code(rows: list[dict]) -> int:
    """Exit 0 iff every P0+P1 row is PASS or WAIVED."""
    for r in rows:
        blast = r.get("blast_radius", "P2")
        if blast in ("P0", "P1") and r.get("status") not in ("PASS", "WAIVED"):
            return 1
    return 0


def print_row_summary(row: dict, elapsed_s: float, extra: str = "") -> None:
    status = row.get("status", "?")
    # P61 SUBJ-03: a NOT-VERIFIED row that flipped via the freshness branch
    # carries a transient _stale=True flag; render as [STALE] for clarity.
    label = "STALE" if (status == "NOT-VERIFIED" and row.get("_stale")) else status
    blast = row.get("blast_radius", "?")
    rid = row.get("id", "?")
    suffix = f" -> {extra}" if extra else ""
    print(f"    [{label:<13}] {rid}  ({blast}, {elapsed_s:.2f}s){suffix}")


def main() -> int:
    parser = argparse.ArgumentParser(description="Quality Gates runner")
    parser.add_argument("--cadence", required=True, choices=VALID_CADENCES)
    args = parser.parse_args()

    print(f"quality/runners/run.py --cadence {args.cadence}")
    now = datetime.now(timezone.utc)

    catalogs = discover_catalogs()
    if not catalogs:
        print(f"  (no catalogs under {CATALOG_DIR})")
        return 0

    all_rows: list[dict] = []
    counts = {"PASS": 0, "FAIL": 0, "PARTIAL": 0, "WAIVED": 0, "NOT-VERIFIED": 0}

    for cat_path in catalogs:
        original = load_catalog(cat_path)
        # Deep-copy via JSON round-trip so we can compare status before/after
        # without sharing references that mutate during run_row().
        data = json.loads(json.dumps(original))
        in_scope = [r for r in data["rows"] if is_in_scope(r, args.cadence, now)]
        if not in_scope:
            continue
        in_scope_sorted = sort_by_blast_radius(in_scope)
        print(f"  catalog: {cat_path.name} ({len(data['rows'])} rows; {len(in_scope_sorted)} in scope)")
        # Map id -> original status for last_verified rollback below.
        orig_status_by_id = {r["id"]: r.get("status") for r in original["rows"]}
        orig_lv_by_id = {r["id"]: r.get("last_verified") for r in original["rows"]}
        for row in in_scope_sorted:
            updated, elapsed = run_row(row, REPO_ROOT, now)
            counts[updated.get("status", "NOT-VERIFIED")] += 1
            extra = ""
            if updated.get("status") == "NOT-VERIFIED":
                if updated.get("_stale"):
                    extra = (
                        f"freshness expired: last_verified+{row.get('freshness_ttl')} < now "
                        f"(input last_verified={orig_lv_by_id.get(updated['id'])})"
                    )
                else:
                    extra = f"verifier not found at {row.get('verifier', {}).get('script')}"
            elif updated.get("status") == "WAIVED":
                w = updated.get("waiver") or {}
                extra = f"waived until {w.get('until', '?')} — {w.get('reason', '')[:60]}"
            print_row_summary(updated, elapsed, extra)
            all_rows.append(updated)
        # For rows whose status did NOT change, roll back last_verified to its
        # original value. The runner mutated it for in-memory display; the
        # catalog should NOT persist per-run timestamp churn — it would leave
        # a dirty file across every pre-push run.
        for row in data["rows"]:
            rid = row.get("id")
            if rid in orig_status_by_id and row.get("status") == orig_status_by_id[rid]:
                row["last_verified"] = orig_lv_by_id[rid]
        # Persist mutations back to catalog ONLY if any row's status actually
        # changed. Timestamp-only updates belong in the artifact, not the
        # committed catalog — otherwise every pre-push run leaves a dirty
        # catalog file with formatter noise (em-dashes, array layout).
        if catalog_dirty(original, data):
            save_catalog(cat_path, data)

    exit_code = compute_exit_code(all_rows)
    summary = (
        f"summary: {counts['PASS']} PASS, {counts['FAIL']} FAIL, "
        f"{counts['PARTIAL']} PARTIAL, {counts['WAIVED']} WAIVED, "
        f"{counts['NOT-VERIFIED']} NOT-VERIFIED -> exit={exit_code}"
    )
    print(summary)
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
