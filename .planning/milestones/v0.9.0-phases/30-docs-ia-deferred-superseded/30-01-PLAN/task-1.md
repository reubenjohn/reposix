# Task 1: Install Vale locally + create installer script + CI integration

← [back to index](./index.md)

<task type="auto">
  <name>Task 1: Install Vale locally + create installer script + CI integration</name>
  <files>scripts/install-vale.sh, .github/workflows/docs.yml</files>
  <read_first>
    - `.github/workflows/docs.yml` (current state — see lines 31-37 for where Vale install step must be inserted before `mkdocs build --strict`)
    - `.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-RESEARCH.md` §Standard Stack (lines 102-136) and §Example 3 (GitHub Actions step for Vale)
    - `.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-PATTERNS.md` §".github/workflows/docs.yml" analog reference
    - `.planning/notes/phase-30-narrative-vignettes.md` (for the LOCKED P1/P2 ban rules this installer supports)
  </read_first>
  <action>
    Create `scripts/install-vale.sh` with the following exact content (use this verbatim — DO NOT substitute version or URL):

```bash
#!/usr/bin/env bash
#
# Install Vale — Phase 30 banned-word linter (DOCS-09).
#
# Usage:
#   bash scripts/install-vale.sh                 # default install to ~/.local/bin
#   VALE_PREFIX=/usr/local/bin bash scripts/install-vale.sh   # CI install to system path
#
# Idempotent: if `vale --version` already prints the pinned version, exits 0.

set -euo pipefail

readonly VALE_VERSION="3.10.0"
readonly PREFIX="${VALE_PREFIX:-${HOME}/.local/bin}"

mkdir -p "$PREFIX"

if command -v vale >/dev/null 2>&1; then
  if vale --version 2>/dev/null | grep -q "v${VALE_VERSION}"; then
    echo "vale v${VALE_VERSION} already installed at $(command -v vale)"
    exit 0
  fi
fi

readonly OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
readonly ARCH="64-bit"   # x86_64 Linux / macOS matrix — Vale release asset naming
readonly URL="https://github.com/errata-ai/vale/releases/download/v${VALE_VERSION}/vale_${VALE_VERSION}_Linux_${ARCH}.tar.gz"

echo "Downloading Vale v${VALE_VERSION} from ${URL}..."
curl -fsSL "$URL" | tar xz -C "$PREFIX" vale
chmod +x "${PREFIX}/vale"

# Verify
"${PREFIX}/vale" --version
echo "Vale installed at ${PREFIX}/vale"
```

Then update `.github/workflows/docs.yml` — insert three new steps between the existing "Install MkDocs + Material" step (line 31-34) and "Build strict" step (line 36-37). Also change the pip install line to include `[imaging]` per RESEARCH.md §Environment Availability recommendations. Also add `.vale.ini` + `.vale-styles/**` to the `paths:` trigger filter (lines 6-9).

Exact diff to `.github/workflows/docs.yml`:

```yaml
on:
  push:
    branches: [main]
    paths:
      - 'docs/**'
      - 'mkdocs.yml'
      - '.github/workflows/docs.yml'
      - '.vale.ini'
      - '.vale-styles/**'
      - 'scripts/install-vale.sh'
  workflow_dispatch:
```

And within the `build-and-deploy` steps list (after the "Install MkDocs + Material" step and before "Build strict"):

```yaml
      - name: Install MkDocs + Material (with imaging for social cards)
        run: |
          python -m pip install --upgrade pip
          python -m pip install "mkdocs-material[imaging]" mkdocs-minify-plugin

      - name: Install Vale (DOCS-09)
        run: |
          VALE_PREFIX=/usr/local/bin bash scripts/install-vale.sh

      - name: Lint docs with Vale (P1/P2 banned-word rules)
        run: vale --config=.vale.ini docs/

      - name: Build strict (fail on broken links)
        run: mkdocs build --strict
```

Replace the existing "Install MkDocs + Material" + "Build strict" steps with the block above. Keep `- name: Deploy to gh-pages` unchanged.

Run the installer locally to confirm it works: `bash scripts/install-vale.sh` then `~/.local/bin/vale --version` should emit `vale version v3.10.0`.
  </action>
  <verify>
    <automated>bash scripts/install-vale.sh && ~/.local/bin/vale --version | grep -q "v3.10.0" && grep -q "install-vale.sh" .github/workflows/docs.yml && grep -q "vale --config=.vale.ini docs/" .github/workflows/docs.yml && grep -q 'mkdocs-material\[imaging\]' .github/workflows/docs.yml</automated>
  </verify>
  <acceptance_criteria>
    - `test -x scripts/install-vale.sh` returns 0 (file exists and is executable).
    - Running `bash scripts/install-vale.sh` exits 0 on Linux x86_64.
    - After running, `~/.local/bin/vale --version` outputs a line containing `v3.10.0`.
    - `grep -c "install-vale.sh" .github/workflows/docs.yml` returns `>= 1`.
    - `grep -c 'vale --config=.vale.ini docs/' .github/workflows/docs.yml` returns `1`.
    - `grep -c 'mkdocs-material\[imaging\]' .github/workflows/docs.yml` returns `1`.
    - `grep -c '.vale-styles/\*\*' .github/workflows/docs.yml` returns `1` (paths trigger).
    - `grep -c 'VALE_VERSION="3.10.0"' scripts/install-vale.sh` returns `1` (pinned).
  </acceptance_criteria>
  <done>
    Vale installed locally, `scripts/install-vale.sh` committed executable, `.github/workflows/docs.yml` updated and diff reviewed — no secrets leaked. Workflow is still green against zero docs changes (Vale will find zero P1/P2 violations in current docs/ because those rules are not yet configured — that happens in Task 2).
  </done>
</task>
