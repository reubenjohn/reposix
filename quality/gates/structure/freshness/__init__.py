"""Per-invariant verifier modules backing freshness-invariants.py dispatch.

Split from a single 26k-char file in 2026-05-01 per the file-size-limits
gate (`*.py` budget = 15k chars). Each module owns one logical group of
related verifier functions; the parent `freshness-invariants.py` is the
thin `--row-id` dispatcher.

Naming:
- `_shared` — common helpers (artifact write, row load, bash_check).
- `install_paths` — README/docs-index pkg-mgr-leads-install-snippet asserts.
- `structure_misc` — version-pinned filenames, mkdocs nav, ROADMAP scope, etc.
- `p62_repo_org` — Python regression branches retained behind shipped .sh verifiers.
- `doc_alignment` — quality/catalogs/doc-alignment.json schema + floor walk.
"""
