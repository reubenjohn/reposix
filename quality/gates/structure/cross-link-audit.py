#!/usr/bin/env python3
# KEEP-AS-CANONICAL (P63 MIGRATE-02): meta-helper / cross-link audit verifier; no canonical home.
"""cross-link-audit.py -- MIGRATE-02 cohesion pass verifier (P63).

Walks CLAUDE.md + quality/PROTOCOL.md + every quality/gates/<dim>/README.md
and extracts relative repo-path mentions. Asserts each path resolves to an
existing file or directory.

Stale-by-design exception: a line containing `<!-- planned: v0.12.1 ... -->`
is skipped (planned future paths are valid stubs).

Stdlib only.

Modes:
  python3 quality/gates/structure/cross-link-audit.py          # all defaults
  python3 quality/gates/structure/cross-link-audit.py --json <path>
  python3 quality/gates/structure/cross-link-audit.py --report <path>

Exit codes: 0 PASS (zero stale), 1 FAIL (>=1 stale).
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent.parent
DEFAULT_DOCS = [
    REPO_ROOT / "CLAUDE.md",
    REPO_ROOT / "quality" / "PROTOCOL.md",
    REPO_ROOT / "quality" / "gates" / "code" / "README.md",
    REPO_ROOT / "quality" / "gates" / "release" / "README.md",
    REPO_ROOT / "quality" / "gates" / "docs-build" / "README.md",
    REPO_ROOT / "quality" / "gates" / "docs-repro" / "README.md",
    REPO_ROOT / "quality" / "gates" / "agent-ux" / "README.md",
    REPO_ROOT / "quality" / "gates" / "perf" / "README.md",
    REPO_ROOT / "quality" / "gates" / "structure" / "README.md",
    REPO_ROOT / "quality" / "gates" / "security" / "README.md",
]

# Regex captures candidate repo-relative paths. Anchored with prefix-segment
# so we only match repo-relative paths (not arbitrary URLs / unix paths).
# Known historical / planned paths that are intentionally referenced in
# narrative without existing on disk. These ARE valid mentions (they document
# project history or planned future work); the audit must not flag them.
KNOWN_HISTORICAL_OR_PLANNED = {
    # Truncated/template paths that the path regex catches
    ".planning/milestones/v",
    "docs/tutorial/benchmark",
    "quality/reports/verdicts/p",
    # Planned future paths (P58 split, v0.12.1 carry-forward)
    "quality/catalogs/install-paths.json",
    "quality/runners/_cache.py",
    # Retired scripts referenced as historical lineage in CLAUDE.md / READMEs
    "scripts/check_clippy_lint_loaded.sh",
    "scripts/check_fixtures.py",
    "scripts/repro-quickstart.sh",
    "scripts/test_bench_token_economy.py",
    "scripts/dark-factory-test.sh",
}

PATH_PREFIXES = (
    "quality/gates",
    "quality/catalogs",
    "quality/runners",
    "quality/reports",
    "quality/PROTOCOL.md",
    "quality/SURPRISES",
    ".planning/phases",
    ".planning/milestones",
    ".planning/research",
    ".planning/REQUIREMENTS.md",
    ".planning/ROADMAP.md",
    ".planning/STATE.md",
    ".planning/PROJECT.md",
    "scripts/",
    "crates/",
    "docs/",
    "examples/",
    ".github/",
    "benchmarks/",
)
# Anything matching one of those prefixes followed by a path-component run.
PATH_RE = re.compile(
    r"(?P<path>(?:" + "|".join(re.escape(p) for p in PATH_PREFIXES) + r")[A-Za-z0-9_./-]+)"
)
TRAILING = ".,;:)\"'`*"
PLANNED_MARKER = re.compile(r"<!--\s*planned\s*:")


def now_rfc3339() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def is_glob(p: str) -> bool:
    return "*" in p or "{" in p or "..." in p


def looks_like_doc_anchor(p: str) -> bool:
    # paths used as anchors / fragments / shell-template often end with -<wildcard>
    # Heuristic: skip if it contains explicit placeholder syntax.
    if "<" in p or ">" in p or "#" in p:
        return True
    # Template placeholders (v0.X.0, YYYY-QN, p<N>, <ts>, ...)
    if re.search(r"v0\.X\.|YYYY|<[A-Za-z]|/p[A-Z]+/|p<N>", p):
        return True
    # Trailing dash indicates a truncated/template path (e.g. "scripts/check-")
    if p.endswith(("-", "_", "/")):
        return True
    # Single-letter or two-letter trailing segment after a dash often indicates
    # a truncated reference (e.g. "v0.12.0-").
    return False


def extract_paths(text: str) -> set[str]:
    out: set[str] = set()
    for line in text.splitlines():
        if PLANNED_MARKER.search(line):
            continue
        for m in PATH_RE.finditer(line):
            raw = m.group("path")
            # Strip trailing punctuation
            while raw and raw[-1] in TRAILING:
                raw = raw[:-1]
            if not raw:
                continue
            if is_glob(raw):
                continue
            if looks_like_doc_anchor(raw):
                continue
            # Strip backticks etc.
            raw = raw.strip("`")
            if not raw:
                continue
            out.add(raw)
    return out


def check_paths(paths: set[str]) -> tuple[set[str], set[str]]:
    """Return (resolved, stale). Known-historical / planned paths are
    treated as resolved (they are valid narrative references)."""
    resolved: set[str] = set()
    stale: set[str] = set()
    for p in paths:
        if p in KNOWN_HISTORICAL_OR_PLANNED:
            resolved.add(p)
            continue
        target = REPO_ROOT / p
        if target.exists():
            resolved.add(p)
        else:
            stale.add(p)
    return resolved, stale


def write_report(report_path: Path, per_doc: dict[str, dict]) -> None:
    report_path.parent.mkdir(parents=True, exist_ok=True)
    lines: list[str] = []
    lines.append("# Cross-link audit (P63 MIGRATE-02)")
    lines.append("")
    lines.append(f"_Generated {now_rfc3339()} by `quality/gates/structure/cross-link-audit.py`._")
    lines.append("")
    total = sum(len(v["paths"]) for v in per_doc.values())
    stale_total = sum(len(v["stale"]) for v in per_doc.values())
    lines.append(f"**Summary:** {total} unique paths audited across {len(per_doc)} docs; {stale_total} stale.")
    lines.append("")
    for doc, info in per_doc.items():
        lines.append(f"## {doc}")
        lines.append(f"- paths: {len(info['paths'])}")
        lines.append(f"- stale: {len(info['stale'])}")
        if info["stale"]:
            lines.append("")
            lines.append("Stale paths:")
            for s in sorted(info["stale"]):
                lines.append(f"  - `{s}`")
        lines.append("")
    report_path.write_text("\n".join(lines), encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description="MIGRATE-02 cross-link audit verifier (P63).")
    parser.add_argument("--json", default=str(REPO_ROOT / "quality" / "reports" / "verifications" / "structure" / "cross-link-audit-p63.json"))
    parser.add_argument("--report", default=str(REPO_ROOT / "quality" / "reports" / "audits" / "cross-link-audit-p63.md"))
    args = parser.parse_args()

    per_doc: dict[str, dict] = {}
    all_stale: set[str] = set()
    all_paths: set[str] = set()
    for doc in DEFAULT_DOCS:
        if not doc.is_file():
            per_doc[str(doc.relative_to(REPO_ROOT))] = {"paths": set(), "stale": {f"<doc-missing>{doc}"}}
            all_stale.add(f"<doc-missing>{doc}")
            continue
        text = doc.read_text(encoding="utf-8", errors="ignore")
        paths = extract_paths(text)
        resolved, stale = check_paths(paths)
        rel = str(doc.relative_to(REPO_ROOT))
        per_doc[rel] = {"paths": paths, "stale": stale, "resolved": resolved}
        all_paths.update(paths)
        all_stale.update(stale)

    status = "PASS" if not all_stale else "FAIL"
    payload = {
        "claim_id": "structure/cross-link-audit-p63",
        "phase": "p63",
        "verifier_kind": "mechanical",
        "verified_at": now_rfc3339(),
        "verifier_script": "quality/gates/structure/cross-link-audit.py",
        "audited_docs": list(per_doc.keys()),
        "paths_total": len(all_paths),
        "paths_stale": len(all_stale),
        "stale_list": sorted(all_stale),
        "status": status,
    }
    json_path = Path(args.json)
    json_path.parent.mkdir(parents=True, exist_ok=True)
    json_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    write_report(Path(args.report), per_doc)

    print(f"[{status}] cross-link-audit: {len(all_paths)} paths, {len(all_stale)} stale -> {json_path.relative_to(REPO_ROOT)}")
    if all_stale:
        for s in sorted(all_stale)[:50]:
            print(f"  STALE: {s}", file=sys.stderr)
        if len(all_stale) > 50:
            print(f"  ... ({len(all_stale) - 50} more)", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
