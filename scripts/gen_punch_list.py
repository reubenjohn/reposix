#!/usr/bin/env python3
"""Generate PUNCH-LIST.md from a populated docs-alignment catalog.

Clusters MISSING_TEST and RETIRE_PROPOSED rows by source-file prefix so
the human review can see "Confluence backend parity", "JIRA shape",
"benchmark numbers", "guides" etc. as named clusters.

Usage:
    python3 scripts/gen_punch_list.py <run-dir>

where <run-dir> is the backfill timestamped directory (e.g.
quality/reports/doc-alignment/backfill-20260428T085523Z/).
The catalog is read from quality/catalogs/doc-alignment.json.
The output is written to <run-dir>/PUNCH-LIST.md.
"""
from __future__ import annotations

import json
import sys
from collections import defaultdict
from pathlib import Path


def cluster_key(source_file: str) -> str:
    """Map a source path to a human-readable cluster name."""
    parts = source_file.split("/")
    if source_file == "README.md":
        return "README headline + install + dark-factory loop"
    if parts[0] == ".planning":
        # archived REQUIREMENTS.md from milestones
        if "milestones" in parts:
            for p in parts:
                if p.startswith("v0.") and p.endswith("-phases"):
                    return f"Archived REQUIREMENTS.md ({p})"
        return "Archived planning"
    if parts[0] != "docs":
        return "Other"
    if len(parts) < 2:
        return "docs/ root"
    sub = parts[1]
    if sub == "reference":
        if len(parts) >= 3:
            return f"Reference: {parts[2].split('.md')[0]}"
        return "Reference"
    if sub == "decisions":
        if len(parts) >= 3:
            slug = parts[2].split(".md")[0]
            # group ADRs into thematic buckets
            if "github" in slug:
                return "ADR: GitHub state mapping"
            if "confluence" in slug:
                return "ADR: Confluence page mapping"
            if "jira" in slug:
                return "ADR: JIRA issue mapping"
            if "nested-mount" in slug or "mount-layout" in slug:
                return "ADR: nested mount layout (FUSE-era)"
            if "time-travel" in slug:
                return "ADR: time-travel via git tags"
            if "helper-backend" in slug:
                return "ADR: helper backend dispatch"
            if "stability" in slug:
                return "ADR: stability commitments"
            return f"ADR: {slug}"
        return "ADR (unspecified)"
    if sub == "benchmarks":
        return "Benchmarks: headline numbers"
    if sub == "concepts":
        return "Concepts (mental model + positioning)"
    if sub == "connectors":
        return "Connector authoring guide"
    if sub == "development":
        return "Developer workflow + invariants"
    if sub == "guides":
        return "User guides (integration, troubleshooting, write-connector)"
    if sub == "how-it-works":
        return "Internals (filesystem / git / time-travel / trust)"
    if sub == "research":
        return "Research notes"
    if sub == "social":
        return "Social posts (marketing copy)"
    if sub == "tutorials":
        return "Tutorials (first-run flow)"
    return f"docs/{sub}"


def fmt_source(source: dict | str) -> str:
    if isinstance(source, str):
        return source
    if isinstance(source, dict):
        return f"{source['file']}:{source['line_start']}-{source['line_end']}"
    if isinstance(source, list):
        return ", ".join(fmt_source(s) for s in source)
    return str(source)


def source_file(source: dict | str) -> str:
    if isinstance(source, str):
        return source.split(":")[0]
    if isinstance(source, dict):
        return source["file"]
    if isinstance(source, list) and source:
        return source_file(source[0])
    return ""


def main() -> int:
    if len(sys.argv) != 2:
        print(__doc__, file=sys.stderr)
        return 2

    run_dir = Path(sys.argv[1])
    if not run_dir.is_dir():
        print(f"error: not a directory: {run_dir}", file=sys.stderr)
        return 2

    catalog_path = Path("quality/catalogs/doc-alignment.json")
    catalog = json.loads(catalog_path.read_text(encoding="utf-8"))

    summary = catalog["summary"]
    rows = catalog["rows"]

    missing: dict[str, list[dict]] = defaultdict(list)
    retire: dict[str, list[dict]] = defaultdict(list)
    bound: dict[str, int] = defaultdict(int)

    for row in rows:
        cluster = cluster_key(source_file(row.get("source", "")))
        verdict = row.get("last_verdict")
        if verdict == "MISSING_TEST":
            missing[cluster].append(row)
        elif verdict == "RETIRE_PROPOSED":
            retire[cluster].append(row)
        elif verdict == "BOUND":
            bound[cluster] += 1

    out = []
    out.append("# v0.12.0 P65 — Docs-alignment backfill PUNCH-LIST")
    out.append("")
    out.append(f"**Run dir:** `{run_dir.as_posix()}/`")
    out.append("**Backfill date:** 2026-04-28")
    out.append("**Catalog:** `quality/catalogs/doc-alignment.json`")
    out.append("")
    out.append("## Top-line numbers")
    out.append("")
    out.append(f"- claims_total: **{summary['claims_total']}**")
    out.append(f"- claims_bound: **{summary['claims_bound']}**")
    out.append(f"- claims_missing_test: **{summary['claims_missing_test']}**")
    out.append(f"- claims_retire_proposed: **{summary['claims_retire_proposed']}**")
    out.append(f"- alignment_ratio: **{summary['alignment_ratio']:.3f}** (floor 0.50; floor_waiver until 2026-07-31)")
    out.append("")
    out.append("## How to read this list")
    out.append("")
    out.append(
        "Each cluster maps to a user-facing surface. **MISSING_TEST** rows are claims with no test asserting them — these are the gap-closure work for v0.12.1 (P71+). **RETIRE_PROPOSED** rows are claims an extractor flagged as superseded by a documented architecture decision; the human (`reuben`) confirms each retirement explicitly via `reposix-quality doc-alignment confirm-retire --row-id X` from a TTY (env-guarded against agent contexts)."
    )
    out.append("")
    out.append(
        "**Important caveat on cluster sizes.** A few clusters are over-extracted: `Reference: glossary` shows 24 RETIRE_PROPOSED rows because the extractor treated each glossary term as a claim and proposed retiring them all. Treat such clusters as a bulk-confirm review item (the human can `confirm-retire` all 24 in one sitting), not 24 individual investigation tickets."
    )
    out.append("")
    out.append("## Clusters by user-facing surface")
    out.append("")

    cluster_names = sorted(set(missing.keys()) | set(retire.keys()))
    for cname in cluster_names:
        m_count = len(missing.get(cname, []))
        r_count = len(retire.get(cname, []))
        b_count = bound.get(cname, 0)
        if m_count == 0 and r_count == 0:
            continue
        out.append(f"### {cname}")
        out.append("")
        out.append(f"- BOUND: {b_count}")
        out.append(f"- MISSING_TEST: {m_count}")
        out.append(f"- RETIRE_PROPOSED: {r_count}")
        out.append("")
        if m_count:
            out.append(f"<details><summary>{m_count} MISSING_TEST</summary>")
            out.append("")
            for row in missing[cname]:
                out.append(f"- **{row['id']}** — {row.get('claim','(no claim text)')} <br/>source: `{fmt_source(row.get('source',''))}`")
            out.append("")
            out.append("</details>")
            out.append("")
        if r_count:
            out.append(f"<details><summary>{r_count} RETIRE_PROPOSED</summary>")
            out.append("")
            for row in retire[cname]:
                out.append(f"- **{row['id']}** — {row.get('claim','(no claim text)')} <br/>source: `{fmt_source(row.get('source',''))}`")
            out.append("")
            out.append("</details>")
            out.append("")

    out.append("## v0.12.1 carry-forward")
    out.append("")
    out.append(
        "v0.12.1 milestone phases P71+ close clusters above. Suggested phase scoping (1 cluster per phase or one phase per related-cluster group):"
    )
    out.append("")
    out.append("- **P71** — Confluence backend parity (close `Reference: confluence` MISSING_TEST + `Archived REQUIREMENTS.md (v0.8.0-phases)` Confluence rows). The smoking-gun page-tree-symlink regression originates here.")
    out.append("- **P72** — JIRA shape (close `ADR: JIRA issue mapping` + `Reference: jira` MISSING_TEST).")
    out.append("- **P73** — Benchmark numbers (close `Benchmarks: headline numbers` MISSING_TEST). Includes drift fix: README/index/social copy headline numbers vs measured benchmarks.")
    out.append("- **P74** — Connector authoring guide (close `Connector authoring guide` MISSING_TEST).")
    out.append("- **P75** — Tutorial integration coverage (close `Tutorials (first-run flow)` MISSING_TEST steps 4-8).")
    out.append("- **P76** — Glossary retirement bulk confirm (`Reference: glossary` 24 RETIRE_PROPOSED rows).")
    out.append("- **P77** — Developer workflow + invariants (close `Developer workflow + invariants` MISSING_TEST).")
    out.append("- **P78** — Concepts (close mental-model + reposix-vs-mcp MISSING_TEST).")
    out.append("- **P79** — Guides MISSING_TEST (REPOSIX_BLOB_LIMIT env override + others).")
    out.append("- **P80** — FUSE-era ADR retirement (`ADR: nested mount layout` etc.; cite v0.9.0 architecture pivot).")
    out.append("")
    out.append("Numbering is suggestive; the human re-prioritizes when v0.12.1 opens.")
    out.append("")
    out.append("## Floor-waiver expiry")
    out.append("")
    out.append("`summary.floor_waiver.until` and `quality/catalogs/freshness-invariants.json` row `docs-alignment/walk` waiver both expire **2026-07-31**. Before that date, v0.12.1 must close enough clusters to lift `alignment_ratio` above 0.50 OR the human ratchets the floor + waiver explicitly.")
    out.append("")
    out.append("---")
    out.append("")
    out.append("*Generated by `scripts/gen_punch_list.py` from the populated docs-alignment catalog at the end of P65.*")
    out.append("")

    output_path = run_dir / "PUNCH-LIST.md"
    output_path.write_text("\n".join(out), encoding="utf-8")
    print(f"wrote {output_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
