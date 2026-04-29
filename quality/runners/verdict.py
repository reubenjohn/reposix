#!/usr/bin/env python3
"""Quality Gates verdict — collate artifacts -> markdown verdict + shields.io badge JSON.

Per .planning/research/v0.12.0/naming-and-architecture.md § runner-contract.
Stdlib only. Reads quality/catalogs/*.json + quality/reports/verifications/<dim>/*.json,
writes quality/reports/verdicts/<cadence-or-phase>/<ts>.md + quality/reports/badge.json.

QG-09 P57 emit scope: badge.json in shields.io endpoint format. P60 publishes
it via mkdocs as docs/badge.json -> https://reubenjohn.github.io/reposix/badge.json.

Usage:
  python3 quality/runners/verdict.py --cadence pre-push
  python3 quality/runners/verdict.py --phase 57
  python3 quality/runners/verdict.py session-end   # roll up across every cadence; emit SESSION-VERDICT.md
  python3 quality/runners/verdict.py            # all cadences

Exit codes:
  0 — verdict is GREEN.
  1 — verdict is RED.
"""

from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
CATALOG_DIR = REPO_ROOT / "quality" / "catalogs"
REPORTS_DIR = REPO_ROOT / "quality" / "reports"
VERDICTS_DIR = REPORTS_DIR / "verdicts"
BADGE_PATH = REPORTS_DIR / "badge.json"

CADENCES = ("pre-push", "pre-pr", "weekly", "pre-release", "post-release", "on-demand")


def now_iso() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def now_iso_filename() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H-%M-%SZ")


def discover_catalogs() -> list[Path]:
    out: list[Path] = []
    for p in sorted(CATALOG_DIR.glob("*.json")):
        if p.stem == "orphan-scripts":
            continue
        out.append(p)
    return out


def load_catalog(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def load_artifact(row: dict, repo_root: Path) -> dict | None:
    art_path = row.get("artifact")
    if not art_path:
        return None
    abs_path = repo_root / art_path
    if not abs_path.exists():
        return None
    try:
        return json.loads(abs_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return None


def in_scope_filter(rows: list[dict], cadence: str | None) -> list[dict]:
    if cadence is None:
        return list(rows)
    return [r for r in rows if r.get("cadence") == cadence]


def compute_status_counts(rows: list[dict]) -> dict[str, int]:
    counts = {"PASS": 0, "FAIL": 0, "PARTIAL": 0, "WAIVED": 0, "NOT-VERIFIED": 0}
    for r in rows:
        s = r.get("status", "NOT-VERIFIED")
        counts[s] = counts.get(s, 0) + 1
    return counts


def is_p0_p1(row: dict) -> bool:
    return row.get("blast_radius") in ("P0", "P1")


def is_green_status(status: str) -> bool:
    return status in ("PASS", "WAIVED")


def compute_color(rows: list[dict]) -> str:
    """brightgreen if all P0+P1 are PASS or WAIVED; yellow if any P2 RED but no P0+P1 RED; red if any P0+P1 RED."""
    p0p1_red = any(is_p0_p1(r) and not is_green_status(r.get("status", "")) for r in rows)
    if p0p1_red:
        return "red"
    p2_red = any(
        r.get("blast_radius") == "P2" and not is_green_status(r.get("status", ""))
        for r in rows
    )
    if p2_red:
        return "yellow"
    return "brightgreen"


def compute_badge_message(rows: list[dict]) -> str:
    p0p1 = [r for r in rows if is_p0_p1(r)]
    if not p0p1:
        return "0/0 GREEN"
    green = sum(1 for r in p0p1 if is_green_status(r.get("status", "")))
    return f"{green}/{len(p0p1)} GREEN"


def emit_shields_endpoint_json(rows: list[dict], out_path: Path) -> None:
    badge = {
        "schemaVersion": 1,
        "label": "quality gates",
        "message": compute_badge_message(rows),
        "color": compute_color(rows),
    }
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(badge, indent=2) + "\n", encoding="utf-8")


def emit_markdown_verdict(rows: list[dict], cadence_or_phase: str, out_path: Path, repo_root: Path) -> None:
    counts = compute_status_counts(rows)
    color = compute_color(rows)
    verdict = "GREEN" if color == "brightgreen" else "RED"
    out_path.parent.mkdir(parents=True, exist_ok=True)

    lines: list[str] = []
    lines.append(f"# Quality Gates verdict — {cadence_or_phase} — {now_iso()}")
    lines.append("")
    lines.append(f"- Verdict: **{verdict}**")
    lines.append(f"- Generated at: `{now_iso()}`")
    lines.append("- Badge: `quality/reports/badge.json` (shields.io endpoint format)")
    lines.append(f"- Color: `{color}` ({compute_badge_message(rows)})")
    lines.append("")
    lines.append("## Status counts")
    lines.append("")
    lines.append("| status | count |")
    lines.append("|---|---|")
    for status in ("PASS", "FAIL", "PARTIAL", "WAIVED", "NOT-VERIFIED"):
        lines.append(f"| {status} | {counts.get(status, 0)} |")
    lines.append("")

    failures = [r for r in rows if r.get("status") in ("FAIL", "PARTIAL")]
    lines.append("## Failures (if any)")
    lines.append("")
    if not failures:
        lines.append("_None._")
    else:
        for r in failures:
            artifact = load_artifact(r, repo_root) or {}
            failed = artifact.get("asserts_failed", []) or []
            lines.append(f"- `{r.get('id')}` (P{r.get('blast_radius')[-1]}, dim={r.get('dimension')}) — {r.get('owner_hint', '')}")
            lines.append(f"  - artifact: `{r.get('artifact')}`")
            lines.append(f"  - asserts_failed: {failed!r}")
    lines.append("")

    waived = [r for r in rows if r.get("status") == "WAIVED"]
    lines.append("## Waivers (if any)")
    lines.append("")
    if not waived:
        lines.append("_None._")
    else:
        for r in waived:
            w = r.get("waiver") or {}
            lines.append(
                f"- `{r.get('id')}` — {w.get('reason', '')} (until `{w.get('until', '?')}`, tracked_in `{w.get('tracked_in', '?')}`)"
            )
    lines.append("")

    not_verified = [r for r in rows if r.get("status") == "NOT-VERIFIED"]
    lines.append("## NOT-VERIFIED (if any)")
    lines.append("")
    if not not_verified:
        lines.append("_None._")
    else:
        for r in not_verified:
            lv = r.get("last_verified") or "never"
            ttl = r.get("freshness_ttl") or "(mechanical)"
            lines.append(
                f"- `{r.get('id')}` (P{r.get('blast_radius', '?')[-1]}, dim={r.get('dimension')}) — last_verified: `{lv}`, freshness_ttl: `{ttl}`"
            )
    lines.append("")

    # P61 SUBJ-03: STALE sub-table inside the NOT-VERIFIED section. A row only
    # appears here if its artifact JSON has stale=True (the freshness branch
    # in run.py wrote it). Provides the days-expired summary the human needs.
    stale_rows: list[tuple[dict, dict]] = []
    for r in not_verified:
        art = load_artifact(r, repo_root) or {}
        if art.get("stale") is True:
            stale_rows.append((r, art))
    lines.append("### STALE (subset of NOT-VERIFIED)")
    lines.append("")
    if not stale_rows:
        lines.append("_None._")
    else:
        lines.append("| Row ID | last_verified | freshness_ttl | days expired |")
        lines.append("|---|---|---|---|")
        now_dt = datetime.now(timezone.utc)
        for r, art in stale_rows:
            lv_input = art.get("last_verified_input") or "?"
            ttl = art.get("freshness_ttl") or r.get("freshness_ttl") or "?"
            try:
                from run import parse_duration as _pd
                from run import parse_rfc3339 as _pr
                expired_at = _pr(lv_input) + _pd(ttl)
                days_expired = (now_dt - expired_at).days
            except (ValueError, TypeError):
                days_expired = "?"
            lines.append(f"| `{r.get('id')}` | `{lv_input}` | `{ttl}` | {days_expired} |")
    lines.append("")

    passed = [r for r in rows if r.get("status") == "PASS"]
    lines.append("## PASS (collapsed)")
    lines.append("")
    lines.append(f"<details><summary>{len(passed)} PASS rows</summary>")
    lines.append("")
    for r in passed:
        lines.append(f"- `{r.get('id')}` (P{r.get('blast_radius', '?')[-1]}, dim={r.get('dimension')})")
    lines.append("")
    lines.append("</details>")
    lines.append("")
    lines.append("---")
    lines.append("*Generated by `python3 quality/runners/verdict.py`. See `quality/PROTOCOL.md` § \"runner contract\".*")

    out_path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def collate_rows(cadence: str | None) -> list[dict]:
    """Read every catalog file, return rows filtered by cadence (or all rows if None)."""
    rows: list[dict] = []
    for cat_path in discover_catalogs():
        data = load_catalog(cat_path)
        for r in data.get("rows", []):
            if cadence is None or r.get("cadence") == cadence:
                rows.append(r)
    return rows


def emit_session_end_rollup(repo_root: Path, args: list[str]) -> int:
    """Run rollup across every cadence; emit timestamped + stable SESSION-VERDICT.md."""
    sub = argparse.ArgumentParser(prog="verdict.py session-end")
    sub.add_argument("--phase", type=int, default=None)
    sub_args = sub.parse_args(args)

    # Union rows across every cadence (each row reports its primary cadence).
    rows = collate_rows(cadence=None)
    if sub_args.phase is not None:
        # No phase tagging on rows yet (P57 hasn't catalogged that field); accept all rows.
        pass

    ts = now_iso_filename()
    timestamped = VERDICTS_DIR / "session-end" / f"{ts}.md"
    stable = VERDICTS_DIR / "session-end" / "SESSION-VERDICT.md"
    emit_markdown_verdict(rows, "session-end", timestamped, repo_root)
    # Stable copy: rewrite via a second emit so timestamps in the body are fresh.
    emit_markdown_verdict(rows, "session-end", stable, repo_root)
    emit_shields_endpoint_json(rows, BADGE_PATH)

    color = compute_color(rows)
    return 0 if color == "brightgreen" else 1


def main() -> int:
    # Handle the `session-end` positional subcommand BEFORE argparse flag parsing
    if len(sys.argv) > 1 and sys.argv[1] == "session-end":
        return emit_session_end_rollup(REPO_ROOT, sys.argv[2:])

    parser = argparse.ArgumentParser(description="Quality Gates verdict generator")
    parser.add_argument("--cadence", choices=CADENCES, default=None)
    parser.add_argument("--phase", type=int, default=None)
    args = parser.parse_args()

    rows = collate_rows(args.cadence)
    if args.phase is not None:
        scope = f"p{args.phase}"
    elif args.cadence is not None:
        scope = args.cadence
    else:
        scope = "all"

    ts = now_iso_filename()
    out_path = VERDICTS_DIR / scope / f"{ts}.md"
    emit_markdown_verdict(rows, scope, out_path, REPO_ROOT)
    emit_shields_endpoint_json(rows, BADGE_PATH)

    color = compute_color(rows)
    print(f"verdict: {color} ({compute_badge_message(rows)}) -> {out_path.relative_to(REPO_ROOT)}")
    return 0 if color == "brightgreen" else 1


if __name__ == "__main__":
    raise SystemExit(main())
