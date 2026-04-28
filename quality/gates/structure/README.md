# quality/gates/structure/ — structure-dimension verifiers

Verifiers backing the catalog rows in `quality/catalogs/freshness-invariants.json`.

## Verifiers

| Script | Catalog rows | Cadence |
|---|---|---|
| `freshness-invariants.py` | 10 structure rows + 3 P62 rows (see below) | pre-push |
| `banned-words.sh` | structure/banned-words | pre-push |
| `cred-hygiene.sh` | structure/cred-hygiene | pre-push |

## Conventions

- Python stdlib for cross-platform mechanical checks; bash for thin pre-existing wrappers.
- Exit 0 = PASS, 1 = FAIL (per `quality/runners/run.py`).
- Artifacts at `quality/reports/verifications/structure/<row-slug>.json`.

## P62 — repo-org recurrence guards

| Row id | What it asserts | Owner hint |
|---|---|---|
| `structure/no-loose-top-level-planning-audits` | `find .planning -maxdepth 1 -type f \( -name '*MILESTONE-AUDIT*.md' -o -name 'SESSION-END-STATE*' \) \| grep -v archive` returns empty | new milestone-audit/session-end-state docs land under `.planning/milestones/audits/` or `.planning/archive/`, never `.planning/` top level |
| `structure/no-pre-pivot-doc-stubs` | every `docs/*.md` stub <500 bytes is in `mkdocs.yml` `nav:` OR in `plugins.redirects.redirect_maps` | new top-level `docs/<slug>.md` stubs <500 bytes — add to redirect map or remove |
| `structure/repo-org-audit-artifact-present` | `quality/reports/audits/repo-org-gaps.md` exists with a closure-path table per gap | future repo-org audits land at the same canonical path |

These 3 rows ship in the P62 Wave 1 catalog-first commit. The verifier branches in `quality/gates/structure/freshness-invariants.py` are extended in P62 Wave 3 to handle the new `--row-id` values. Until Wave 3 lands, the rows wear a short-lived waiver until 2026-05-15. After Wave 3 + Wave 6, the runner re-grades to PASS and the waivers expire harmlessly.

See `quality/PROTOCOL.md` for runtime details (waiver semantics, cadences, dispatch).

## Cross-references

- `quality/catalogs/freshness-invariants.json` — rows backed by these verifiers.
- `quality/PROTOCOL.md` — runner + verdict + waiver contract.
- `CLAUDE.md` § "Freshness invariants" — the user-facing rules these gates enforce.
