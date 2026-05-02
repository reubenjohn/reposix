← [back to index](./index.md)

# Verification, success criteria, and output

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Carved prose -> published Layer-3 pages | FUSE / kernel / daemon terms are permitted here (per P2 layer model) but must remain accurate to code; stale claims about SG-01..08 rows would mislead. |
| Sentinel blocks -> transitional state | docs/architecture.md + docs/security.md stay on disk briefly; sentinel must make it clear content is elsewhere to avoid reader confusion. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-30-05-01 | Tampering | SG-01..08 rows paraphrased inaccurately | mitigate | Rows copied verbatim from docs/security.md lines 21-30; acceptance criteria grep checks for exact SG-IDs present. |
| T-30-05-02 | Information Disclosure | Evidence file:line pointers leak non-public source paths | accept | All cited paths under `crates/**` are public in the repo. |
| T-30-05-03 | Repudiation | Stale "content carved" sentinel points at wrong destination | mitigate | Sentinel lists all three how-it-works target files with relative links; acceptance criteria grep checks links resolve. |
</threat_model>

<verification>
1. `mkdocs build --strict` exits 0.
2. `python3 scripts/check_phase_30_structure.py` — mermaid-counts pass; how-it-works pages fill content; deleted-paths still present (expected — Wave 3 deletes).
3. Read `docs/how-it-works/trust-model.md` end-to-end — every SG-* row has a file:line pointer; no new claims.
4. `grep -rc 'architecture.md\|security.md' docs/ | grep -v ':0$'` — note which pages still link to the soon-deleted files. Wave 3 plan 30-08 updates these.
</verification>

<success_criteria>
- Three how-it-works sub-pages filled with carved content; one mermaid each.
- how-it-works/index.md is final (stub marker removed).
- architecture.md + security.md flagged for deletion via sentinel blocks.
- mkdocs build --strict green.
- SG-01..08 table transcribed verbatim (every file:line pointer preserved).
</success_criteria>

<output>
After completion, create `.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-05-SUMMARY.md` documenting:
- Line counts per how-it-works sub-page
- SG-* rows validated verbatim against docs/security.md
- Which architecture.md sections were carved vs dropped
- Inventory of cross-page links now pointing at how-it-works/* (useful for Wave 3 audit)
</output>
