# Shared Patterns, No Analog Found, and Metadata

← [back to index](./index.md)

## Shared Patterns

### Mermaid diagrams (render + review)
**Source:** `docs/architecture.md` (multiple) + `docs/why.md` lines 104–115 + `docs/index.md` lines 16–26.
**Apply to:** All three `docs/how-it-works/*.md` pages (one diagram each).

Pattern conventions already set by the codebase:
- Use `flowchart LR` for left-to-right dataflows; `flowchart TB` for subgraph-heavy security perimeter; `sequenceDiagram` with `autonumber` for read/write paths.
- Label nodes in quoted strings if they contain HTML (`<br/>`): `A["LLM agent<br/>(Claude Code / shell)"]`.
- Color palette is uniform across diagrams: purple (`#6a1b9a`) = agent/attacker-origin; teal (`#00897b`) = reposix core; orange (`#ef6c00`) = egress/danger-of-data-loss; red (`#d32f2f`) = tainted/hostile.
- Style lines ALWAYS at bottom of the block: `style A fill:#... ,stroke:#fff,color:#fff`.

**Rendering verification:** `mkdocs-material` auto-themes flowchart/sequence/class/state/ER diagrams; client-side JS renders at page load. Screenshot verification via Playwright MCP per RESEARCH.md §Example 5. Dark-mode check is mandatory (Pitfall 3).

### Frontmatter on markdown pages
**Source:** no docs page in the repo currently uses YAML frontmatter (confirmed — `docs/index.md`, `docs/why.md`, etc. all start with `# Title`). mkdocs-material uses `hide:` / `template:` only for custom templates.
**Apply to:** All new pages — NO frontmatter unless the page needs `template: home.html` (research recommends against that path).

### Internal-link conventions
**Source:** `docs/connectors/guide.md` lines 64–65, 451–464; `docs/architecture.md` line 254.
**Apply to:** All new pages.

Pattern:
- Internal markdown links use relative paths from the current file: `[text](../decisions/...)`, `[text](reference/cli.md)`, `[text](why.md#anchor)`.
- Cross-repository-root references (to `crates/`, `.planning/`) use full GitHub URLs: `[path](https://github.com/reubenjohn/reposix/blob/main/crates/reposix-core/src/backend.rs)`.
- `mkdocs --strict` gate catches dangling internal links — verify before merge.

### Admonition voice
**Source:** `docs/index.md` line 11 (`!!! success`) + `docs/why.md` line 41 (`!!! success "Measured, not claimed"`) + `docs/development/contributing.md` line 3 (`!!! note`).
**Apply to:** Announcement banners, measured-claim callouts, non-obvious-caveat notes.

Pattern: `!!! <type> "<title>"` with 4-space indented body. Types used: `success`, `note`. Avoid decorative use.

### Footnote-for-cited-source
**Source:** `docs/security.md` lines 5, 11 (`[^1]: Simon Willison ...`).
**Apply to:** `docs/vs-mcp-sdks.md`, `docs/how-it-works/trust-model.md`.

Pattern:

```markdown
Every deployment of reposix is a textbook **lethal trifecta**[^1]:
...
[^1]: [Simon Willison, "The lethal trifecta for AI agents"](https://simonwillison.net/...), revised April 2026.
```

### Bash hook structure
**Source:** `scripts/hooks/pre-push` + `scripts/hooks/test-pre-push.sh`.
**Apply to:** `scripts/hooks/pre-commit-docs` + `scripts/hooks/test-pre-commit-docs.sh`.

Pattern: shebang + long-form comment header (purpose, install, bypass, env); `set -euo pipefail`; `readonly` color codes + NC; `log()` helper gated on `REPOSIX_HOOKS_QUIET`; return 0 on no-match, non-zero on hit with helpful stderr message ending in a suggested next action ("rotate the token," "rephrase the sentence"); cleanup trap for tests.

### Python validation script
**Source:** `scripts/check_fixtures.py`.
**Apply to:** `scripts/check_phase_30_structure.py`.

Pattern: `#!/usr/bin/env python3` shebang; module docstring explaining checks + usage; `from __future__ import annotations`; stdlib-only imports; `check_*() -> list[str]` functions each returning error messages; `main()` aggregates and exits non-zero if any check failed.

### GitHub Actions tool-install step
**Source:** `.github/workflows/docs.yml` lines 27–34 + `.github/workflows/ci.yml` line 39 (`Swatinem/rust-cache@v2` pattern for Rust; `sudo apt-get install` for system binaries).
**Apply to:** New Vale-install step in `docs.yml`.

Pattern: explicit `- name:` per step; pin versions where possible; use `curl -L | tar -xz` for binary releases (per research §Standard Stack); verify with `--version` call immediately after install.

---

## No Analog Found

Files with no close existing match in the reposix codebase. Planner should use RESEARCH.md-provided templates directly.

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `.vale.ini` | linter config | new-from-scratch | Vale has never been used in this repo; format is `.ini`, which exists nowhere else. Use RESEARCH.md §Example 1 verbatim. |
| `.vale-styles/Reposix/ProgressiveDisclosure.yml` | Vale rule | new-from-scratch | No Vale rules yet. Use RESEARCH.md §Example 1 verbatim. |
| `.vale-styles/Reposix/NoReplace.yml` | Vale rule | new-from-scratch | Same as above. |
| `docs/how-it-works/index.md` | section landing | new-from-scratch | No existing section-landing page in `docs/` (decisions/, development/ have no index.md). The `docs/reference/crates.md` file is the closest shape-wise, but it's a page-in-nav, not a section-landing. Pattern is "1-paragraph bridge + grid-cards TOC." |

---

## Metadata

**Analog search scope:**
- `docs/` (all files read or surveyed) — 25 files
- `docs/reference/`, `docs/decisions/`, `docs/development/`, `docs/research/`, `docs/connectors/` — 100% coverage
- `mkdocs.yml` — read in full
- `scripts/hooks/` — `pre-push`, `test-pre-push.sh`, `install-hooks.sh` all read
- `scripts/` — `check_fixtures.py`, `demo.sh` sampled as Python/shell analogs
- `.github/workflows/` — `ci.yml`, `docs.yml`, `release.yml` listed; `docs.yml` + `ci.yml` read in full

**Files scanned:** 15 existing files read directly; 25+ files surveyed via directory listings.

**Pattern extraction date:** 2026-04-17

**Verified tools / already-present dependencies (from RESEARCH.md §Environment Availability, lines 1024–1041):** mkdocs 1.6.1, mkdocs-material 9.7.1, pymdownx.superfences with mermaid fence already wired (line 50–54 of mkdocs.yml), CairoSVG + pillow (social-cards-ready), mmdc 11.12.0, Playwright Chromium cached. Vale is the only net-new install.

## PATTERN MAPPING COMPLETE
