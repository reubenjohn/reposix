#!/usr/bin/env python3
# KEEP-AS-CANONICAL (P63 SIMPLIFY-12): no canonical home under quality/gates/. Per-FILE planning aid; meta-helper distinct from quality/catalogs/ per-CHECK enforcement (SIMPLIFY-03 boundary).
"""Per-file LIVING tracker for v0.11.1.

JSON at .planning/v0.11.1-catalog.json is the source of truth.
Markdown view at .planning/research/v0.11.1-CATALOG-v3.md is auto-rendered
(do not hand-edit). See HANDOVER.md §7-C2 for design rationale.

Stdlib only. Subcommands: init / set / coverage / render / query / stats.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
CATALOG_JSON = REPO_ROOT / ".planning" / "v0.11.1-catalog.json"
CATALOG_MD = REPO_ROOT / ".planning" / "research" / "v0.11.1-CATALOG-v3.md"
SCHEMA_VERSION = 1

VALID_STATUSES = {"KEEP", "TODO", "DONE", "REVIEW", "DELETE", "REFACTOR"}

# Audits to scan for prior verdicts during init.
AUDIT_FILES = [
    (".planning/research/v0.11.0-CATALOG-v2.md", "catalog-v2"),
    (".planning/research/v0.11.1-repo-organization-gaps.md", "repo-org-gaps"),
    (".planning/research/v0.11.1-code-quality-gaps.md", "code-quality-gaps"),
]

# Files we never want to actively track in the catalog (generated / archival /
# milestone log files). Skipped from JSON entirely.
SKIP_PATH_PATTERNS = [
    re.compile(r"^Cargo\.lock$"),
    re.compile(r"^target/"),
    re.compile(r"^runtime/"),
    re.compile(r"^\.planning/milestones/v0\.[0-9]+\.[0-9]+-phases/"),
]


def now_iso() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def git_ls_files() -> list[str]:
    out = subprocess.run(
        ["git", "ls-files"],
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return [line for line in out.stdout.splitlines() if line.strip()]


def should_skip(path: str) -> bool:
    return any(p.search(path) for p in SKIP_PATH_PATTERNS)


def load_audit_text(rel_path: str) -> str:
    p = REPO_ROOT / rel_path
    if not p.exists():
        return ""
    return p.read_text(encoding="utf-8", errors="replace")


# Audit verdict scanning. Each audit file mentions paths in markdown tables /
# bullets. We use a coarse heuristic: for each file path in git ls-files, look
# for occurrences in the audit text under a verdict-bearing context.
VERDICT_PATTERNS = [
    (re.compile(r"\bDELETE\b", re.IGNORECASE), "DELETE"),
    (re.compile(r"\bREFACTOR\b", re.IGNORECASE), "REFACTOR"),
    (re.compile(r"\bREVIEW\b", re.IGNORECASE), "REVIEW"),
    (re.compile(r"\bARCHIVE\b", re.IGNORECASE), "DELETE"),
    (re.compile(r"\bREWRITE\b", re.IGNORECASE), "REFACTOR"),
    (re.compile(r"\bCONDENSE\b", re.IGNORECASE), "REFACTOR"),
]


def scan_audit_for_path(audit_text: str, path: str) -> tuple[str | None, str | None]:
    """Return (status, plan_excerpt) if the path is mentioned in the audit text
    near a verdict keyword. None if path absent.
    """
    if not audit_text or not path:
        return None, None
    # Match only on the full repo-relative path (with optional repo-root
    # prefix). Basename matching produced too many false positives (any
    # mention of "CHANGELOG.md" in passing flagged the file). The audit
    # files are structured: every actionable row mentions the full path
    # at least once, often as `/home/reuben/workspace/reposix/<path>` or
    # backticked `<path>`.
    candidates = [path, f"/home/reuben/workspace/reposix/{path}"]
    matched_status: str | None = None
    matched_plan: str | None = None
    for line in audit_text.splitlines():
        if not any(c in line for c in candidates):
            continue
        if True:
            for pat, status in VERDICT_PATTERNS:
                if pat.search(line):
                    # Prefer DELETE > REFACTOR > REVIEW for severity.
                    rank = {"DELETE": 3, "REFACTOR": 2, "REVIEW": 1}
                    if matched_status is None or rank[status] > rank[matched_status]:
                        matched_status = status
                        # Trim the line as plan summary.
                        cleaned = line.strip().lstrip("|").strip()
                        if len(cleaned) > 240:
                            cleaned = cleaned[:237] + "..."
                        matched_plan = cleaned
    return matched_status, matched_plan


def initial_status_for(path: str) -> tuple[str, list[str], str | None]:
    """Cross-reference a path against all audit files. Return
    (status, prior_refs, plan).
    """
    prior: list[str] = []
    final_status = "KEEP"
    plan: str | None = None
    rank = {"DELETE": 3, "REFACTOR": 2, "REVIEW": 1, "KEEP": 0, "TODO": 0, "DONE": 0}
    for rel, tag in AUDIT_FILES:
        text = load_audit_text(rel)
        status, line = scan_audit_for_path(text, path)
        if status is None:
            continue
        prior.append(f"{tag}:{status}")
        if rank[status] > rank[final_status]:
            final_status = status
            plan = f"[{tag}] {line}" if line else None
    # In v0.11.1 we treat any flagged file as TODO with the audit's plan.
    # The original status (DELETE/REFACTOR/REVIEW) goes into the prior tags.
    if final_status in {"DELETE", "REFACTOR", "REVIEW"}:
        return "TODO", prior, plan
    return "KEEP", prior, None


def cmd_init(_args: argparse.Namespace) -> int:
    files = git_ls_files()
    existing: dict[str, dict] = {}
    if CATALOG_JSON.exists():
        data = json.loads(CATALOG_JSON.read_text())
        for row in data.get("files", []):
            existing[row["path"]] = row

    rows: list[dict] = []
    added = 0
    skipped = 0
    kept = 0
    for path in files:
        if should_skip(path):
            skipped += 1
            continue
        if path in existing:
            rows.append(existing[path])
            kept += 1
            continue
        status, prior, plan = initial_status_for(path)
        rows.append(
            {
                "path": path,
                "status": status,
                "prior": prior,
                "plan": plan,
                "note": "",
                "updated_at": now_iso(),
            }
        )
        added += 1

    rows.sort(key=lambda r: r["path"])
    payload = {
        "generated_at": now_iso(),
        "schema_version": SCHEMA_VERSION,
        "files": rows,
    }
    CATALOG_JSON.parent.mkdir(parents=True, exist_ok=True)
    CATALOG_JSON.write_text(json.dumps(payload, indent=2, sort_keys=False) + "\n")
    print(
        f"init complete: {len(rows)} rows ({added} added, {kept} kept, {skipped} skipped). "
        f"Wrote {CATALOG_JSON}."
    )
    return 0


def load_catalog() -> dict:
    if not CATALOG_JSON.exists():
        print(f"error: {CATALOG_JSON} does not exist; run `init` first.", file=sys.stderr)
        sys.exit(2)
    return json.loads(CATALOG_JSON.read_text())


def save_catalog(data: dict) -> None:
    data["generated_at"] = now_iso()
    CATALOG_JSON.write_text(json.dumps(data, indent=2, sort_keys=False) + "\n")


def cmd_set(args: argparse.Namespace) -> int:
    if args.status not in VALID_STATUSES:
        print(
            f"error: invalid status {args.status!r}; must be one of {sorted(VALID_STATUSES)}",
            file=sys.stderr,
        )
        return 2
    data = load_catalog()
    target = None
    for row in data["files"]:
        if row["path"] == args.path:
            target = row
            break
    if target is None:
        print(f"error: path {args.path!r} not found in catalog.", file=sys.stderr)
        return 2
    target["status"] = args.status
    if args.note is not None:
        target["note"] = args.note
    if args.plan is not None:
        target["plan"] = args.plan
    target["updated_at"] = now_iso()
    save_catalog(data)
    print(f"set {args.path} → {args.status}")
    return 0


def cmd_coverage(_args: argparse.Namespace) -> int:
    data = load_catalog()
    offenders = [
        r for r in data["files"]
        if r["status"] == "TODO" and not (r.get("plan") and r["plan"].strip())
    ]
    if offenders:
        print(
            f"coverage: {len(offenders)} TODO row(s) without a plan:",
            file=sys.stderr,
        )
        for r in offenders[:50]:
            print(f"  - {r['path']}", file=sys.stderr)
        if len(offenders) > 50:
            print(f"  ... and {len(offenders) - 50} more", file=sys.stderr)
        return 1
    print("coverage: ok (every TODO has a plan)")
    return 0


def cmd_stats(_args: argparse.Namespace) -> int:
    data = load_catalog()
    counts: dict[str, int] = {s: 0 for s in VALID_STATUSES}
    for r in data["files"]:
        counts[r["status"]] = counts.get(r["status"], 0) + 1
    total = len(data["files"])
    print(f"stats: {total} files tracked")
    for status in sorted(VALID_STATUSES):
        print(f"  {status:8s} {counts.get(status, 0):>5d}")
    return 0


def cmd_query(args: argparse.Namespace) -> int:
    try:
        proc = subprocess.run(
            ["jq", args.expression, str(CATALOG_JSON)],
            check=False,
        )
    except FileNotFoundError:
        print(
            "error: jq not installed. Install it (e.g. `apt install jq`) or use `stats`.",
            file=sys.stderr,
        )
        return 127
    return proc.returncode


# ----- Render -------------------------------------------------------------


def _md_escape(s: str) -> str:
    if s is None:
        return ""
    return str(s).replace("|", "\\|").replace("\n", " ").strip()


def _table(headers: list[str], rows: list[list[str]]) -> str:
    if not rows:
        return "_(none)_\n"
    out = ["| " + " | ".join(headers) + " |"]
    out.append("|" + "|".join("---" for _ in headers) + "|")
    for r in rows:
        out.append("| " + " | ".join(_md_escape(c) for c in r) + " |")
    return "\n".join(out) + "\n"


def cmd_render(_args: argparse.Namespace) -> int:
    data = load_catalog()
    rows = sorted(data["files"], key=lambda r: r["path"])
    counts: dict[str, int] = {s: 0 for s in VALID_STATUSES}
    per_dir: dict[str, dict[str, int]] = {}
    for r in rows:
        counts[r["status"]] = counts.get(r["status"], 0) + 1
        top = r["path"].split("/", 1)[0]
        default_entry = {s: 0 for s in VALID_STATUSES}
        default_entry["_total"] = 0
        d = per_dir.setdefault(top, default_entry)
        d[r["status"]] = d.get(r["status"], 0) + 1
        d["_total"] = d.get("_total", 0) + 1

    def by_status(status: str) -> list[list[str]]:
        out = []
        for r in rows:
            if r["status"] == status:
                out.append(
                    [
                        r["path"],
                        r.get("plan") or "",
                        ", ".join(r.get("prior") or []),
                        r.get("note") or "",
                        r.get("updated_at") or "",
                    ]
                )
        return out

    done_recent = sorted(
        (r for r in rows if r["status"] == "DONE"),
        key=lambda r: r.get("updated_at") or "",
        reverse=True,
    )[:20]

    md_lines: list[str] = []
    md_lines.append("# CATALOG-v3 — v0.11.1 LIVING per-file tracker\n")
    md_lines.append(
        "*Auto-generated by `scripts/catalog.py render` from `.planning/v0.11.1-catalog.json`. "
        "Do NOT hand-edit.*\n"
    )
    md_lines.append(
        f"*Generated at: {data.get('generated_at', now_iso())}. "
        f"Total files tracked: {len(rows)}.*\n"
    )

    md_lines.append("\n## Status counts\n")
    md_lines.append(
        _table(
            ["Status", "Count"],
            [[s, str(counts.get(s, 0))] for s in sorted(VALID_STATUSES)],
        )
    )

    md_lines.append("\n## Per-directory summary\n")
    dir_rows = []
    for d in sorted(per_dir.keys()):
        entry = per_dir[d]
        dir_rows.append([
            d,
            str(entry["_total"]),
            str(entry.get("TODO", 0)),
            str(entry.get("DONE", 0)),
            str(entry.get("REFACTOR", 0)),
            str(entry.get("DELETE", 0)),
        ])
    md_lines.append(_table(["dir", "total", "TODO", "DONE", "REFACTOR", "DELETE"], dir_rows))

    md_lines.append("\n## Open work — TODO\n")
    md_lines.append(_table(["path", "plan", "prior", "note", "updated_at"], by_status("TODO")))

    md_lines.append("\n## Pending refactors — REFACTOR\n")
    md_lines.append(_table(["path", "plan", "prior", "note", "updated_at"], by_status("REFACTOR")))

    md_lines.append("\n## Pending deletes — DELETE\n")
    md_lines.append(_table(["path", "plan", "prior", "note", "updated_at"], by_status("DELETE")))

    md_lines.append("\n## Under review — REVIEW\n")
    md_lines.append(_table(["path", "plan", "prior", "note", "updated_at"], by_status("REVIEW")))

    md_lines.append("\n## Recently completed — DONE (last 20 by updated_at desc)\n")
    md_lines.append(
        _table(
            ["path", "plan", "prior", "note", "updated_at"],
            [
                [
                    r["path"],
                    r.get("plan") or "",
                    ", ".join(r.get("prior") or []),
                    r.get("note") or "",
                    r.get("updated_at") or "",
                ]
                for r in done_recent
            ],
        )
    )

    CATALOG_MD.parent.mkdir(parents=True, exist_ok=True)
    CATALOG_MD.write_text("\n".join(md_lines))
    print(f"render complete: wrote {CATALOG_MD}")
    return 0


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0] if __doc__ else "")
    sub = p.add_subparsers(dest="cmd", required=True)

    sp_init = sub.add_parser("init", help="bootstrap or top-up the JSON tracker")
    sp_init.set_defaults(func=cmd_init)

    sp_set = sub.add_parser("set", help="set status of one file")
    sp_set.add_argument("path")
    sp_set.add_argument("status")
    sp_set.add_argument("--note", default=None)
    sp_set.add_argument("--plan", default=None)
    sp_set.set_defaults(func=cmd_set)

    sp_cov = sub.add_parser("coverage", help="exit 1 if any TODO row has no plan")
    sp_cov.set_defaults(func=cmd_coverage)

    sp_stats = sub.add_parser("stats", help="counts by status")
    sp_stats.set_defaults(func=cmd_stats)

    sp_render = sub.add_parser("render", help="regenerate the auto-rendered MD view")
    sp_render.set_defaults(func=cmd_render)

    sp_query = sub.add_parser("query", help="passthrough to jq for spot-checks")
    sp_query.add_argument("expression")
    sp_query.set_defaults(func=cmd_query)

    args = p.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
