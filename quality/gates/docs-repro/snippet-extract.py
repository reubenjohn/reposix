#!/usr/bin/env python3
"""snippet-extract.py -- fenced-code-block enumeration + drift detector.

DOCS-REPRO-01. Stdlib only. <=250 lines (see quality/gates/docs-repro/README.md).
Modes: --check (default; writes snippet-coverage.json) | --list | --write-template
<derived-id>. Pivot rule: blocks > PIVOT_THRESHOLD flags allow-list mode
per quality/PROTOCOL.md.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent.parent
CATALOG_DIR = REPO_ROOT / "quality" / "catalogs"
CATALOG_PATH = CATALOG_DIR / "docs-reproducible.json"
ALLOWLIST_PATH = CATALOG_DIR / "docs-reproducible-allowlist.json"
ARTIFACT_PATH = REPO_ROOT / "quality" / "reports" / "verifications" / "docs-repro" / "snippet-coverage.json"
DOC_GLOBS = (("README.md",), ("docs/index.md",), ("docs/tutorials", "*.md"), ("docs/guides", "*.md"))
FENCE_RE = re.compile(r"^```(\w*)\s*$")
PIVOT_THRESHOLD = 50


def now_iso() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def discover_docs() -> list[Path]:
    out: list[Path] = []
    for spec in DOC_GLOBS:
        if len(spec) == 1:
            p = REPO_ROOT / spec[0]
            if p.exists():
                out.append(p)
        else:
            base = REPO_ROOT / spec[0]
            if base.exists():
                out.extend(sorted(base.glob(spec[1])))
    return out


def extract_blocks(path: Path) -> list[dict]:
    """Walk one markdown file; return one dict per fenced code block."""
    try:
        text = path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError):
        return []
    blocks: list[dict] = []
    in_block = False
    start = 0
    lang = ""
    buf: list[str] = []
    rel = str(path.relative_to(REPO_ROOT))
    for n, line in enumerate(text.splitlines(), start=1):
        m = FENCE_RE.match(line)
        if m and not in_block:
            in_block, start, lang, buf = True, n, m.group(1), []
        elif m and in_block:
            in_block = False
            content = "\n".join(buf)
            sha = hashlib.sha256(content.encode("utf-8")).hexdigest()[:16]
            blocks.append({
                "file": rel, "start_line": start, "end_line": n, "lang": lang,
                "content": content, "sha256": sha,
                "derived_id": f"snippet/{rel}:{start}-{n}",
            })
        elif in_block:
            buf.append(line)
    return blocks


def all_blocks() -> list[dict]:
    out: list[dict] = []
    for p in discover_docs():
        out.extend(extract_blocks(p))
    return out


def load_catalog() -> dict:
    """Aggregate every quality/catalogs/*.json -- cross-catalog source citations
    (e.g. release-assets rows citing README.md install lines) cover doc blocks."""
    rows: list = []
    for p in sorted(CATALOG_DIR.glob("*.json")):
        try:
            rows.extend(json.loads(p.read_text(encoding="utf-8")).get("rows", []) or [])
        except (json.JSONDecodeError, OSError):
            continue
    return {"rows": rows}


def load_allowlist() -> set[str]:
    if not ALLOWLIST_PATH.exists():
        return set()
    try:
        return set(json.loads(ALLOWLIST_PATH.read_text(encoding="utf-8")).get("ids", []))
    except json.JSONDecodeError:
        return set()


def _src_covers(src: str, block: dict) -> bool:
    """True if a row source path matches the block file + line range."""
    if not isinstance(src, str) or not src.startswith(block["file"]):
        return False
    if src == block["file"]:
        return True
    tail = src[len(block["file"]):]
    if not tail.startswith(":"):
        return False
    spec = tail[1:]
    try:
        if "-" in spec:
            a, b = (int(x) for x in spec.split("-", 1))
            return a <= block["end_line"] and b >= block["start_line"]
        n = int(spec)
        return block["start_line"] <= n <= block["end_line"]
    except ValueError:
        return False


def block_covered(block: dict, rows: list[dict], allowlist: set[str]) -> bool:
    if block["derived_id"] in allowlist:
        return True
    return any(_src_covers(s, block) for r in rows for s in (r.get("sources") or []))


def cmd_list() -> int:
    blocks = all_blocks()
    print(json.dumps({
        "scope": [str(p.relative_to(REPO_ROOT)) for p in discover_docs()],
        "total": len(blocks),
        "blocks": blocks,
    }, indent=2))
    return 0


def _drift_failures(rows: list[dict], blocks: list[dict]) -> list[str]:
    """Compare row.expected_content_sha256 against current block hashes."""
    drift: list[str] = []
    for row in rows:
        expected = row.get("expected_content_sha256")
        if not expected:
            continue
        for src in row.get("sources") or []:
            if not isinstance(src, str) or ":" not in src:
                continue
            file_part, _, _ = src.partition(":")
            for b in blocks:
                if b["file"] != file_part:
                    continue
                if src.endswith(f"{b['start_line']}-{b['end_line']}") or str(b["start_line"]) in src:
                    if b["sha256"] != expected:
                        drift.append(
                            f"row {row['id']} expected sha256={expected} but {src} hashes to {b['sha256']}"
                        )
    return drift


def cmd_check() -> int:
    blocks = all_blocks()
    rows = load_catalog().get("rows", [])
    allowlist = load_allowlist()
    passed: list[str] = [f"{len(blocks)} fenced code blocks scanned across {len(discover_docs())} files"]
    failed: list[str] = []

    if len(blocks) > PIVOT_THRESHOLD:
        failed.append(
            f"{len(blocks)} blocks exceed threshold ({PIVOT_THRESHOLD}); switch to allow-list mode "
            "per quality/gates/docs-repro/README.md pivot rules"
        )

    uncovered = [b for b in blocks if not block_covered(b, rows, allowlist)]
    if uncovered:
        for b in uncovered[:10]:
            failed.append(
                f"{b['file']}:{b['start_line']}-{b['end_line']} ({b['lang'] or 'no-lang'}) "
                f"has no catalog row; suggest --write-template {b['derived_id']}"
            )
        if len(uncovered) > 10:
            failed.append(f"... and {len(uncovered) - 10} more uncatalogued blocks")
    else:
        passed.append(
            "every fenced block has a catalog row in docs-reproducible.json or matches an allow-list entry"
        )

    drift = _drift_failures(rows, blocks)
    if drift:
        failed.extend(drift)
    else:
        passed.append("no drift detected: every row's expected_content_sha256 matches its source")

    exit_code = 1 if failed else 0
    ARTIFACT_PATH.parent.mkdir(parents=True, exist_ok=True)
    ARTIFACT_PATH.write_text(json.dumps({
        "ts": now_iso(),
        "row_id": "docs-repro/snippet-coverage",
        "exit_code": exit_code,
        "asserts_passed": passed,
        "asserts_failed": failed,
        "block_count": len(blocks),
        "uncovered_count": len(uncovered),
    }, indent=2) + "\n", encoding="utf-8")
    return exit_code


def cmd_write_template(derived_id: str) -> int:
    target = next((b for b in all_blocks() if b["derived_id"] == derived_id), None)
    if target is None:
        print(f"snippet-extract: unknown derived-id {derived_id!r}", file=sys.stderr)
        return 2
    slug = derived_id.replace("/", "-").replace(":", "-")
    first = target["content"].splitlines()[0] if target["content"] else None
    print(json.dumps({
        "id": derived_id, "dimension": "docs-repro", "cadence": "post-release",
        "kind": "container",
        "sources": [f"{target['file']}:{target['start_line']}-{target['end_line']}"],
        "command": first,
        "expected": {"asserts": ["TBD: rewrite this placeholder for the snippet's intent"]},
        "expected_content_sha256": target["sha256"],
        "verifier": {"script": "quality/gates/docs-repro/container-rehearse.sh",
                     "args": [derived_id], "timeout_s": 180, "container": "ubuntu:24.04"},
        "artifact": f"quality/reports/verifications/docs-repro/{slug}.json",
        "status": "NOT-VERIFIED", "last_verified": None, "freshness_ttl": None,
        "blast_radius": "P2", "owner_hint": "TBD", "waiver": None,
    }, indent=2))
    return 0


def main() -> int:
    p = argparse.ArgumentParser(description="Fenced-code-block drift detector for docs-repro.")
    g = p.add_mutually_exclusive_group()
    g.add_argument("--list", action="store_true", help="enumerate fenced blocks as JSON")
    g.add_argument("--check", action="store_true", help="drift detector (default)")
    g.add_argument("--write-template", metavar="DERIVED_ID", help="emit catalog-row stub")
    args = p.parse_args()
    if args.list:
        return cmd_list()
    if args.write_template:
        return cmd_write_template(args.write_template)
    return cmd_check()


if __name__ == "__main__":
    raise SystemExit(main())
