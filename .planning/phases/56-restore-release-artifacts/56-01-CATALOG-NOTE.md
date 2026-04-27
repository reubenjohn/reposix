# 56-01 — Catalog temporary-home rationale

## 1. Why the catalog rows live in `.planning/docs_reproducible_catalog.json` for P56

This phase refreshes the 5 install rows in
`.planning/docs_reproducible_catalog.json` rather than creating a new
`quality/catalogs/release-assets.json`. Reasons (cross-references:
REQUIREMENTS.md `## v0.12.0` § "Release dimension — close the immediate
breakage"; ROADMAP.md `## v0.12.0` Phase 57; CLAUDE.md "Build memory budget" /
"Subagent delegation rules"):

- **`quality/` does not exist yet.** P57 ships the framework skeleton (QG-01).
  Creating a one-row `quality/catalogs/` outside that scaffolded directory
  pre-empts the framework's own first commit and risks shape-drift between
  the v0 row P56 lands and the unified-schema rows P57 lands.
- **The v0 catalog already has the 5 install rows** with field semantics that
  map 1:1 onto the unified schema (`sources` ↔ `sources`, `verifier` ↔
  `verifier`, `expected_outcome.asserts` ↔ `verifier.expected_asserts`,
  `blast_radius` ↔ `blast_radius`, `automation_status` ↔ `cadence`).
- **P57 / P58 / P59 already plan the migration.** P58 ships
  `quality/catalogs/install-paths.json` (the install rows split out at that
  point per SIMPLIFY work); P59 ports the DRAFT seed to
  `quality/catalogs/docs-reproducible.json` (DOCS-REPRO-04). Forcing a half-
  step in P56 doubles the migration churn.

## 2. What P57 / P58 must do to migrate cleanly

Concrete checklist for the next phase agents:

- The 5 install rows split — copy them to
  `quality/catalogs/install-paths.json` with unified-schema fields (P58
  SIMPLIFY work). The docs-tutorial + benchmark + troubleshooting rows stay
  in `docs_reproducible_catalog.json` and migrate to
  `quality/catalogs/docs-reproducible.json` (P59 DOCS-REPRO-04).
- Preserve the `phase: "p56"` provenance tag on every migrated row so the
  audit-trail "which phase declared this contract first" is recoverable.
- Translate the v0 fields to the unified schema (`automation_status` →
  `cadence`; `blast_radius` retained verbatim; `verifier.kind` →
  `verifier.kind`; `expected_outcome.asserts` →
  `verifier.expected_asserts`).

## 3. Pivot rule for Wave B (Option A → Option B)

Restated verbatim from `.planning/research/v0.12.0-install-regression-
diagnosis.md`: if `${{ github.ref }}` substitutions in `release.yml` cannot
be cleanly mapped from `reposix-cli-v0.11.3` to a `version=0.11.3` for
archive filenames (current `reposix-${TAG}-${target}.tar.gz` would become
`reposix-reposix-cli-v0.11.3-…`), pivot to Option B per
`.planning/research/v0.12.0-install-regression-diagnosis.md` § "Option B".
Append the pivot to `quality/SURPRISES.md` (Wave D creates the file).

## 4. Anti-bloat note

This file is single-purpose. Do not grow it. P57 deletes it as part of the
migration close-out.

Cross-references: REQUIREMENTS.md ## v0.12.0 RELEASE-01..03; ROADMAP.md ## Phase 56; .planning/research/v0.12.0-install-regression-diagnosis.md.
