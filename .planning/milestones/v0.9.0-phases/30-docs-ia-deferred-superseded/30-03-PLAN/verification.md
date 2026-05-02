← [back to index](./index.md)

# Verification, success criteria, and output

<verification>
1. `~/.local/bin/vale --config=.vale.ini docs/index.md docs/mental-model.md docs/vs-mcp-sdks.md` exits 0.
2. `python3 scripts/check_phase_30_structure.py` — errors reduced: index.md P1/P2 now clean (previously flagged); mental-model 3-H2 check passes; vs-mcp-sdks complement check passes.
3. `mkdocs build --strict` — may still fail due to nav entries (plan 30-04 fixes); must NOT fail due to broken internal links FROM docs/index.md, docs/mental-model.md, or docs/vs-mcp-sdks.md.
4. `git diff --stat docs/index.md docs/mental-model.md docs/vs-mcp-sdks.md` — index.md should show >=50 lines removed, >=80 lines added (full rewrite); mental-model + vs-mcp-sdks show significant content added.
</verification>

<success_criteria>
- docs/index.md is a Layer-1 hero with the V1 before/after code pair, the exact complement blockquote, a three-up value props grid, and a "Where to go next" grid pointing at all four next-step pages.
- docs/mental-model.md is 300-400 words with three locked H2s.
- docs/vs-mcp-sdks.md contains comparison table + P1 paragraph + benchmark citation.
- Vale clean on all three files.
- Creative-bans enforced (no stock-photo refs, no banned marketing vocab).
</success_criteria>

<output>
After completion, create `.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-03-SUMMARY.md` documenting:
- Word counts for all three pages
- Which competitor patterns were ultimately woven into the hero (B/D/G expected)
- Vale output (should be empty — all clean)
- Preview of the `mkdocs serve` rendered hero (1-2 sentence description)
</output>
