← [back to index](./index.md) · phase 30 research

## Scope Split Decision

**Question:** Given 9 DOCS requirements + 8+ new/modified pages + theme tuning + linter + tutorial + 3 diagrams, does this fit one phase?

**Answer:** **Yes, one phase — do NOT split.**

**Reasoning:**

1. **The artifacts are mutually reinforcing.** The hero (DOCS-01), the how-it-works pages (DOCS-02), the mental model (DOCS-03), and the tutorial (DOCS-06) all reference each other. Splitting into phase 30a (copy+IA) and 30b (diagrams+tutorial+linter) creates cross-phase dependencies — the 30a copy subagent would write placeholders for 30b content, then 30b would correct them, doubling work.
2. **The subagent fanout the source-of-truth note proposes is already parallelism.** Explore + Copy + IA + Diagrams + Tutorial are 5 parallel work-streams. With 5 parallel subagents, one phase IS the split.
3. **The linter (DOCS-09) is independent of narrative copy and blocks nothing.** It can run in parallel with copy/diagrams. Adding it to the same phase is cheap.
4. **Theme tuning (DOCS-08) is 15 lines of mkdocs.yml.** Not a phase unto itself.
5. **Tutorial (DOCS-06) depends on nothing upstream in this phase.** It can start immediately — the simulator already works, demo.md already has accurate content.
6. **Verification costs are lower for one big phase.** Playwright screenshots of the whole site, one doc-clarity-review pass on the rendered index, one CI green gate — vs doing these twice for two smaller phases.

**The risk:** phase scope creep. To mitigate:
- The plan MUST list exact file counts and a done-checklist per requirement.
- If any subagent returns BLOCKED, the plan must be able to continue the other 4 workstreams.
- The Troubleshooting page ships as an explicit stub (~3 entries). Do not let it balloon — it's post-launch growth.

**Recommended subagent structure (confirming the source-of-truth note's suggestion):**

1. **Explore subagent** — competitor narrative scan + hero-line drafting. Picks 3 of the 8 patterns from this research's Competitor Narrative Scan section. Outputs 3 candidate hero-lines.
2. **Copy subagent** — constrained by P1/P2 banned-word list. Writes: `index.md` hero + complement line + three-up; `mental-model.md`; `vs-mcp-sdks.md`. Runs Vale against its own output before submitting.
3. **IA subagent** — two competing `mkdocs.yml` nav structures scored against Diátaxis + personas. Outputs the winner.
4. **Diagrams subagent** — three mermaid diagrams (filesystem / git / trust-model). Uses the three specs from this research's Code Examples §Example 6 as starting points. Runs `mkdocs serve` + playwright screenshot for visual review.
5. **Tutorial subagent** — authors `tutorial.md`. Runs it end-to-end against `reposix-sim`. Attaches a transcript. Screenshots each step via playwright MCP.
6. **Linter subagent (can run last or in parallel)** — writes `.vale.ini` + `.vale-styles/Reposix/*.yml` + `scripts/hooks/pre-commit-docs` + CI diff. Runs the linter against all new/modified docs and fixes violations until clean.
7. **How-it-works carver subagent** — carves `architecture.md` + `security.md` into three pages. Updates all internal links. Deletes old files in the final wave.

**Wave structure (planner decides exact split):**

- **Wave 0:** Plan + agree on IA + hero-line candidates (humans / orchestrator approve copy direction).
- **Wave 1 (parallel):** Copy, IA, Diagrams, Tutorial, Linter subagents.
- **Wave 2 (parallel):** How-it-works carver subagent; guide author for agent-integration and troubleshooting.
- **Wave 3 (serial):** grep-audit for dangling references; delete obsolete files; update README.md.
- **Wave 4 (serial):** `mkdocs build --strict` green; Vale green; doc-clarity-review pass; playwright screenshots committed; CHANGELOG entry; `30-SUMMARY.md`.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|-------------|-----------|---------|----------|
| mkdocs | All doc builds | ✓ | 1.6.1 | — |
| mkdocs-material | Theme | ✓ | 9.7.1 | — |
| mkdocs-material-extensions | Theme | ✓ | 1.3.1 | — |
| mkdocs-minify-plugin | Already in mkdocs.yml | ✓ | 0.8.0 | — |
| CairoSVG (for social cards) | DOCS-08 theme tuning | ✓ | 2.7.1 | If CI fails: `pip install "mkdocs-material[imaging]"` |
| pillow (for social cards) | DOCS-08 | ✓ | 10.4.0 | Same |
| mmdc (mermaid-cli) | Offline mermaid render (not strictly needed if client-side render is kept) | ✓ | 11.12.0 | Client-side render is default; no fallback needed |
| Playwright Chromium | Screenshot verification | ✓ | 1217 cached | Fall back to system chromium if MCP fails |
| Vale | DOCS-09 linter | ✗ | — | Install in CI per Code Examples §Example 3; locally via tarball (~15MB) |
| doc-clarity-review skill | 10-second value-prop validation | ✓ | (at `~/.claude/skills/doc-clarity-review/`) | — |
| Claude subprocess (`claude -p ...`) | doc-clarity-review backend | Presumed available (user environment) | — | Manual review by orchestrator |
| `mkdocs serve` local port 8000 | Screenshot verification | ✓ | — | — |
| fuse3 | Tutorial will reference but not require during authoring | ✓ | (system) | Tutorial runs against simulator + mount on Linux |
| cargo / rustc | Tutorial build steps | ✓ | (project toolchain) | — |

**Missing dependencies with no fallback:** None.

**Missing dependencies with fallback:** Vale — fallback is a custom ~40-line Python linter, but this adds undocumented edge cases. Strongly recommend installing Vale.
