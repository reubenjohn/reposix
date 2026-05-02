← [back to index](./index.md) · phase 30 research

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DOCS-01 | Value prop lands in 10 seconds (hero vignette + three-up + no "replace") | Competitor pattern scan (§Competitor Narrative Scan); validation via `doc-clarity-review` skill (§Validation Architecture); banned-word linter for "replace" (§Linter Choice) |
| DOCS-02 | "How it works" with three pages + one diagram each, playwright-verified | Mermaid integration is already wired in `mkdocs.yml`; `mmdc` 11.12.0 on `$PATH`; Playwright Chromium cached at `~/.cache/ms-playwright/`; architecture.md source material already exists and is rich (§Source Content Audit) |
| DOCS-03 | Mental model + reposix-vs-MCP/SDKs comparison page | Turso `/concepts` and Stripe quickstart set the length/structure precedent (~300-400 words, card-based); comparison pattern from Tailscale (§Competitor Narrative Scan) |
| DOCS-04 | Three guides — connector, agent integration, troubleshooting | `docs/connectors/guide.md` already exists as rich source for "Write your own connector"; "Integrate with your agent" is greenfield; Troubleshooting is stub |
| DOCS-05 | Simulator page moves to Reference | Mechanical nav edit + file move; content already exists in `docs/architecture.md` + `docs/reference/http-api.md` (§Files to Read/Create/Modify) |
| DOCS-06 | 5-min first-run tutorial against simulator | `scripts/demo.sh` is the canonical demo script — tutorial will narrate its first few steps; simulator is simulator-first per CLAUDE.md OP #1 (§Tutorial Pattern) |
| DOCS-07 | Nav restructured per Diátaxis + P2 banned terms not above Layer 3 | Diátaxis confirmed as the right framework (§Diátaxis Validation); linter enforces P2 (§Linter Choice) |
| DOCS-08 | Theme tuned — palette, hero features, social cards | `material[imaging]` deps (CairoSVG, pillow) already installed; `mkdocs-material` 9.7.1 supports `social` plugin; `grid cards` component ships OOB (§mkdocs-material Tuning) |
| DOCS-09 | Banned-word linter runs on every doc commit | Vale is the recommended tool (§Linter Choice); can be scoped by file glob via `.vale.ini` `[*.md]` blocks; pre-commit hook pattern matches existing `scripts/hooks/` (§Linter Integration) |
</phase_requirements>
