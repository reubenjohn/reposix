← [back to index](./index.md) · phase 30 research

## Standard Stack

### Core (already installed, no new installs needed)

| Tool | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| mkdocs | 1.6.1 | Static site generator | Project standard since v0.1; `docs.yml` CI job already builds with `--strict` |
| mkdocs-material | 9.7.1 | Theme | Already configured; widest community support; palette + hero + grid cards all native |
| mkdocs-material-extensions | 1.3.1 | Material-specific markdown extensions | Dependency of theme; already present |
| mkdocs-minify-plugin | 0.8.0 | HTML/JS/CSS minification | Already configured |
| CairoSVG | 2.7.1 | SVG → PNG for social cards | Already installed; used by `material[imaging]` |
| pillow | 10.4.0 | Image processing for social cards | Already installed; used by `material[imaging]` |
| mmdc (@mermaid-js/mermaid-cli) | 11.12.0 | Mermaid → SVG/PNG CLI | Already on `$PATH` via nvm; used for offline diagram render / verification |
| Playwright Chromium | (cached) | Screenshot verification | Already at `~/.cache/ms-playwright/chromium-1217`; invoked via playwright MCP |

### To install (new)

| Tool | Version | Purpose | Install command | Verify |
|------|---------|---------|-----------------|--------|
| Vale | 3.x (latest) | Prose linter for banned-words rules | `uv tool install vale` OR download binary from `github.com/errata-ai/vale/releases/latest` (there is no PyPI package; Vale is a Go binary) | `vale --version` |

**Version verification note:** Vale is published as a statically-linked Go binary (no Python package). The CI install step should be `curl -L https://github.com/errata-ai/vale/releases/latest/download/vale_<version>_Linux_64-bit.tar.gz | tar xz -C ~/.local/bin vale` OR use `errata-ai/vale-action@latest` GitHub Action. Both approaches `[VERIFIED: vale.sh/docs/install]` and `[VERIFIED: github.com/errata-ai/vale-action]` as of April 2026.

**Installation command for the whole phase (no-op for items already present):**

```bash
# Social cards imaging — already satisfied on this host, included for CI parity:
python3 -m pip install --upgrade "mkdocs-material[imaging]"

# Vale — new:
# Preferred: pin a version to avoid surprise upgrades in CI.
VALE_VERSION=3.10.0  # pick latest at plan time; check https://github.com/errata-ai/vale/releases
curl -L "https://github.com/errata-ai/vale/releases/download/v${VALE_VERSION}/vale_${VALE_VERSION}_Linux_64-bit.tar.gz" \
    | tar xz -C ~/.local/bin/ vale
vale --version
```

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Vale | proselint | proselint is Python-based and simpler but cannot do per-file-glob scoping, which we need (banned terms allowed in `docs/how-it-works/` but not elsewhere). Vale wins on scoping. `[VERIFIED: vale.sh/docs/styles]` |
| Vale | Custom Python regex script | Custom script is ~40 lines and testable via pytest — but it duplicates Vale's `IgnoredScopes` handling (code fences, inline code must be excluded from word matching, or `cat FUSE.md` in a bash snippet would false-positive). Vale's `IgnoredScopes = code, code_block` gets this right out of the box. Custom script wins on zero new install, loses on edge cases. **Recommend Vale**, but if the planner prefers zero-install, a custom script is viable with the IgnoredScopes caveat — see §Linter Choice for the tradeoff analysis. |
| Vale | pre-commit `grep` regex | Works for simple "word X nowhere" — but fails the "word X allowed below layer 3" scoping. Rejected. |
| Client-side mermaid (current) | `mmdc` pre-render to SVG checked into `docs/assets/diagrams/` | Pre-render is more reproducible and works with dark-mode-disabled static snapshots, but costs a build step and loses the "single source of truth in the markdown" virtue. **Recommend keep client-side** — it already works, `mkdocs-material` colors flowcharts/sequence diagrams automatically `[VERIFIED: mkdocs-material diagrams reference]`. |
| Material `grid cards` component | Custom `overrides/home.html` Jinja hero template | Custom template is needed if we want scroll-animation / parallax. The creative-license notes explicitly prefer "precise, dry, earned" over marketing flash. Grid cards + markdown-native admonitions gets us there without Jinja. **Recommend markdown-first.** |
