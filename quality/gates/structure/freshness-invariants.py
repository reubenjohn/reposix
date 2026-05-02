#!/usr/bin/env python3
"""Quality Gates structure-dimension verifier — freshness invariants (dispatcher).

Per .planning/research/v0.12.0/naming-and-architecture.md § "Per-dimension catalog files"
+ quality/catalogs/freshness-invariants.json. Stdlib only.

This file is a thin --row-id dispatcher. The per-invariant logic lives in
the `freshness/` package (sibling directory). Each verify_<slug>(row, repo_root)
function:

1. Performs the check named in row.expected.asserts.
2. Writes the artifact JSON with asserts_passed + asserts_failed populated.
3. Returns exit code: 0 PASS, 1 FAIL, 2 PARTIAL.

Dispatch:
  python3 quality/gates/structure/freshness-invariants.py --row-id <row.id>

Anti-bloat: per-invariant modules in `freshness/`. Adding a new row means
adding a function in the right submodule and registering it in DISPATCH below.
New dimensions get their own quality/gates/<dim>/<verifier>.py.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

# Make `freshness` importable regardless of CWD.
sys.path.insert(0, str(Path(__file__).resolve().parent))

from freshness._shared import REPO_ROOT, load_row  # noqa: E402
from freshness.doc_alignment import (  # noqa: E402
    verify_doc_alignment_catalog_present,
    verify_doc_alignment_floor_not_decreased,
    verify_doc_alignment_summary_block_valid,
)
from freshness.install_paths import (  # noqa: E402
    verify_install_leads_with_pkg_mgr_docs_index,
    verify_install_leads_with_pkg_mgr_readme,
)
from freshness.p62_repo_org import (  # noqa: E402
    verify_no_loose_top_level_planning_audits,
    verify_no_pre_pivot_doc_stubs,
    verify_repo_org_audit_artifact_present,
)
from freshness.structure_misc import (  # noqa: E402
    verify_badges_resolve,
    verify_benchmarks_in_mkdocs_nav,
    verify_no_loose_roadmap_or_requirements,
    verify_no_orphan_docs,
    verify_no_version_pinned_filenames,
    verify_top_level_requirements_roadmap_scope,
)


DISPATCH = {
    "structure/no-version-pinned-filenames": verify_no_version_pinned_filenames,
    "structure/install-leads-with-pkg-mgr-docs-index": verify_install_leads_with_pkg_mgr_docs_index,
    "structure/install-leads-with-pkg-mgr-readme": verify_install_leads_with_pkg_mgr_readme,
    "structure/benchmarks-in-mkdocs-nav": verify_benchmarks_in_mkdocs_nav,
    "structure/no-loose-roadmap-or-requirements": verify_no_loose_roadmap_or_requirements,
    "structure/no-orphan-docs": verify_no_orphan_docs,
    "structure/top-level-requirements-roadmap-scope": verify_top_level_requirements_roadmap_scope,
    "structure/badges-resolve": verify_badges_resolve,
    "structure/no-loose-top-level-planning-audits": verify_no_loose_top_level_planning_audits,
    "structure/no-pre-pivot-doc-stubs": verify_no_pre_pivot_doc_stubs,
    "structure/repo-org-audit-artifact-present": verify_repo_org_audit_artifact_present,
    "structure/doc-alignment-catalog-present": verify_doc_alignment_catalog_present,
    "structure/doc-alignment-summary-block-valid": verify_doc_alignment_summary_block_valid,
    "structure/doc-alignment-floor-not-decreased": verify_doc_alignment_floor_not_decreased,
}


def main() -> int:
    parser = argparse.ArgumentParser(description="structure-dim freshness invariants verifier")
    parser.add_argument("--row-id", required=True)
    args = parser.parse_args()
    row = load_row(args.row_id)
    fn = DISPATCH.get(args.row_id)
    if fn is None:
        print(f"unknown row_id: {args.row_id}", file=sys.stderr)
        return 1
    return fn(row, REPO_ROOT)


if __name__ == "__main__":
    raise SystemExit(main())
