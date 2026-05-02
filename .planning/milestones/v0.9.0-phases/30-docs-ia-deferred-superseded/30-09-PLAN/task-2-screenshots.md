← [back to index](./index.md)

# Task 2: Capture Playwright screenshots for 7 pages × 2 viewports

<task type="auto">
  <name>Task 2: Capture Playwright screenshots for 7 pages × 2 viewports</name>
  <files>.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/screenshots/</files>
  <read_first>
    - `scripts/screenshot-docs.sh` (plan 30-01 — generates the JSON manifest; orchestrator reads it and invokes Playwright MCP)
    - `.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-RESEARCH.md` §Example 5 (Playwright screenshot workflow)
    - `.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-VALIDATION.md` §"Manual-Only Verifications" (diagrams aesthetic check)
  </read_first>
  <action>
    Step 1 — Start `mkdocs serve` in the background on port 8000:

```bash
mkdocs serve -a 127.0.0.1:8000 > /tmp/30-09-mkdocs-serve.log 2>&1 &
MKDOCS_PID=$!
# wait for readiness
for i in {1..30}; do curl -sf http://127.0.0.1:8000/ > /dev/null && break; sleep 0.2; done
```

Step 2 — Generate the screenshot manifest:

```bash
bash scripts/screenshot-docs.sh 8000 .planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/screenshots > /tmp/30-09-screenshot-manifest.json
cat /tmp/30-09-screenshot-manifest.json
```

Expected: a JSON array of 14 entries (7 pages × 2 viewports each). The orchestrator parses this file.

Step 3 — For each manifest entry, invoke Playwright MCP to navigate + resize + screenshot. This is executed by the EXECUTOR agent (which has MCP access) via:

- `mcp__playwright__browser_navigate { url: "<url>" }`
- `mcp__playwright__browser_resize { width: <w>, height: <h> }`
- `mcp__playwright__browser_take_screenshot { filename: "<output>" }`

After all 14 screenshots are taken, verify the directory:

```bash
ls -la .planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/screenshots/
# expected: 14 *.png files plus .gitkeep
# expected filenames (per scripts/screenshot-docs.sh slug logic):
#   home-desktop.png, home-mobile.png
#   mental-model-desktop.png, mental-model-mobile.png
#   vs-mcp-sdks-desktop.png, vs-mcp-sdks-mobile.png
#   tutorial-desktop.png, tutorial-mobile.png
#   how-it-works-filesystem-desktop.png, how-it-works-filesystem-mobile.png
#   how-it-works-git-desktop.png, how-it-works-git-mobile.png
#   how-it-works-trust-model-desktop.png, how-it-works-trust-model-mobile.png
```

Step 4 — Visual review each screenshot per user global CLAUDE.md OP #1 checklist:

- Layout (sidebar shows 11 top-level entries; Home highlighted)
- Contrast (text readable in both light + dark mode — take one pair each)
- Broken links (no red underlines in mkdocs-material default theme)
- Mobile width (text reflows; no horizontal scroll)
- For diagram pages: spaghetti edges, overlapping labels, unreadable node text, subgraph grouping coherent, arrows not crossing through boxes (Mermaid diagrams aesthetic check per RESEARCH.md §Example 5).

If any screenshot reveals an issue, record the finding. Minor cosmetic issues do not block the phase; major issues (unreadable diagram, broken layout on mobile) require a Wave-4 revision (see Task 3 conditional path).

Step 5 — Clean up:

```bash
kill $MKDOCS_PID 2>/dev/null || true
```

Step 6 — Git-add the screenshots:

```bash
git add .planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/screenshots/*.png
git status --short .planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/screenshots/
```
  </action>
  <verify>
    <automated>ls .planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/screenshots/*.png | wc -l | awk '{exit !($1 == 14)}' && ls .planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/screenshots/ | grep -q 'home-desktop.png' && ls .planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/screenshots/ | grep -q 'how-it-works-trust-model-mobile.png'</automated>
  </verify>
  <acceptance_criteria>
    - 14 PNG files exist under `.planning/phases/30-.../screenshots/` (7 pages × 2 viewports).
    - Filenames follow the `{slug}-{desktop|mobile}.png` convention.
    - `home-desktop.png` file size > 20KB (non-trivial screenshot, not a 1×1 pixel error).
    - `home-mobile.png` file size > 10KB.
    - All `how-it-works-*.png` file sizes > 20KB (diagrams present).
    - Manual review per user CLAUDE.md OP #1 passes on all 14 (documented in SUMMARY; no critical issues).
    - `mkdocs serve` process cleaned up (no zombie on port 8000).
  </acceptance_criteria>
  <done>
    14 screenshots captured + committed. Visual review passes. Rendered hero + how-it-works + tutorial all pass layout + contrast checks.
  </done>
</task>
