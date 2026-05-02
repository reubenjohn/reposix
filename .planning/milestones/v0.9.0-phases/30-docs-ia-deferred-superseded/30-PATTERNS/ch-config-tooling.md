# Pattern Assignments — Config and tooling

← [back to index](./index.md)

### `mkdocs.yml` (MODIFY — nav + theme + plugins)

**Analog (baseline):** `mkdocs.yml` current state at `/home/reuben/workspace/reposix/mkdocs.yml` (103 lines).

**Current theme block (lines 8–36, keep palette; modify features):**

```yaml
theme:
  name: material
  palette:
    - scheme: default
      primary: deep purple
      accent: amber
      toggle:
        icon: material/brightness-7
        name: Switch to dark mode
    - scheme: slate
      primary: deep purple
      accent: amber
      toggle:
        icon: material/brightness-4
        name: Switch to light mode
  features:
    - navigation.instant
    - navigation.tracking
    - navigation.sections
    - navigation.expand       # REMOVE per research — conflicts with sections
    - navigation.top
    - search.suggest
    - search.highlight
    - content.code.copy
    - content.code.annotate
    - content.tabs.link
  icon:
    repo: fontawesome/brands/github
```

**Target features (research §"mkdocs-material Theme Tuning" lines 833–862):** add `navigation.footer`, `navigation.tabs`, `content.action.edit`, `content.action.view`; optional `announce.dismiss`; REMOVE `navigation.expand`.

**Current plugins (lines 40–43):**

```yaml
plugins:
  - search
  - minify:
      minify_html: true
```

**Target plugins (add social):**

```yaml
plugins:
  - search
  - social   # ADD — generates per-page social cards (CairoSVG + pillow already present)
  - minify:
      minify_html: true
```

**Current markdown extensions (lines 45–70, keep AS-IS — already has mermaid fence wiring):**

```yaml
markdown_extensions:
  - pymdownx.highlight:
      anchor_linenums: true
      line_spans: __span
      pygments_lang_class: true
  - pymdownx.superfences:
      custom_fences:
        - name: mermaid
          class: mermaid
          format: !!python/name:pymdownx.superfences.fence_code_format
  ...
```

**Current nav (lines 77–103):**

```yaml
nav:
  - Home: index.md
  - Why reposix: why.md
  - Architecture: architecture.md
  - Demos:
      - Overview: demos/index.md
      - Tier 2 walkthrough: demo.md
  - Security: security.md
  - Decisions:
      - ADR-001 GitHub state mapping: decisions/001-github-state-mapping.md
      ...
  - Reference:
      - Crates overview: reference/crates.md
      - CLI: reference/cli.md
      - HTTP API (simulator): reference/http-api.md
      - git remote helper: reference/git-remote.md
      - Confluence backend: reference/confluence.md
  - Connectors:
      - Building your own: connectors/guide.md
  - Development:
      - Contributing: development/contributing.md
      - Roadmap (v0.2+): development/roadmap.md
```

**Target nav (source-of-truth §"Proposed nav", matches RESEARCH.md §"Recommended docs/ tree structure"):**

```yaml
nav:
  - Home: index.md
  - Why reposix: why.md
  - Mental model: mental-model.md
  - reposix vs MCP / SDKs: vs-mcp-sdks.md
  - Try it in 5 minutes: tutorial.md
  - How it works:
      - Overview: how-it-works/index.md
      - The filesystem layer: how-it-works/filesystem.md
      - The git layer: how-it-works/git.md
      - The trust model: how-it-works/trust-model.md
  - Guides:
      - Connect to GitHub: guides/connect-github.md
      - Connect to Jira: guides/connect-jira.md
      - Connect to Confluence: guides/connect-confluence.md
      - Write your own connector: guides/write-your-own-connector.md
      - Integrate with your agent: guides/integrate-with-your-agent.md
      - Troubleshooting: guides/troubleshooting.md
  - Reference:
      - Crates overview: reference/crates.md
      - CLI: reference/cli.md
      - HTTP API (simulator): reference/http-api.md
      - Simulator: reference/simulator.md
      - git remote helper: reference/git-remote.md
      - Confluence backend: reference/confluence.md
  - Decisions:
      - ADR-001 GitHub state mapping: decisions/001-github-state-mapping.md
      ...
  - Research:
      - Architectural argument: research/initial-report.md
      - Agentic engineering reference: research/agentic-engineering-reference.md
  - Development:
      - Contributing: development/contributing.md
      - Roadmap (v0.2+): development/roadmap.md
```

**Divergence from analog:** Research §"mkdocs-material Theme Tuning" is the authority on theme features. Research §"Recommended docs/ tree structure" (RESEARCH.md lines 211–262) is the authority on nav structure. The current `Architecture` / `Security` / `Demos` entries are REMOVED (content carved and source files deleted). The current `Connectors` section is REMOVED (folded into `Guides`). Edit carefully per Pitfall 7 — every removed page must have no dangling links when `mkdocs build --strict` runs.

---

### `.vale.ini` (NEW — Vale linter config)

**Analog:** None in the repo. Use research §Example 1 (RESEARCH.md lines 433–464) verbatim.

**Template pattern:**

```ini
# .vale.ini
StylesPath = .vale-styles
MinAlertLevel = warning

Vocab = Reposix

# Exclude code from prose linting (critical — avoids flagging bash snippets)
IgnoredScopes = code, code_block

# By default, apply the banned-word rule to all markdown
[*.md]
BasedOnStyles = Reposix

# ESCAPE HATCH: how-it-works/, reference/, decisions/, research/ are Layer 3+
# and MAY use the banned terms. Opt-out per-glob.
[docs/how-it-works/**]
Reposix.ProgressiveDisclosure = NO
[docs/reference/**]
Reposix.ProgressiveDisclosure = NO
[docs/decisions/**]
Reposix.ProgressiveDisclosure = NO
[docs/research/**]
Reposix.ProgressiveDisclosure = NO
[docs/development/**]
Reposix.ProgressiveDisclosure = NO

# The hero-ban on "replace" is a different rule, applied EVERYWHERE on the landing
[docs/index.md]
BasedOnStyles = Reposix
Reposix.NoReplace = YES
```

**Divergence from analog:** No repo analog for `.ini` format. The above is verbatim-citable. Planner may need to add a `[docs/mental-model.md]` exception specifically for the locked "mount = git working tree" H2 (see `docs/mental-model.md` divergence note above).

---

### `.vale-styles/Reposix/ProgressiveDisclosure.yml` (NEW — P2 rule)

**Analog:** None in the repo. Use research §Example 1 (RESEARCH.md lines 468–482) verbatim.

```yaml
extends: existence
message: "P2 violation: '%s' is a Layer 3 term — banned on Layer 1/2 pages (index, mental-model, vs-mcp-sdks, tutorial, guides, home-adjacent). Move it into docs/how-it-works/ or rephrase in user-experience language."
level: error
scope: text
ignorecase: true
tokens:
  - FUSE
  - inode
  - daemon
  - \bhelper\b       # boundaries — "Jupyter helper" generic is fine, but flag bare
  - kernel
  - \bmount\b        # noun/verb — flag any bare occurrence; authors rephrase
  - syscall
```

**Divergence:** none — template is canonical.

---

### `.vale-styles/Reposix/NoReplace.yml` (NEW — P1 rule)

**Analog:** None in the repo. Use research §Example 1 (RESEARCH.md lines 486–497) verbatim.

```yaml
extends: existence
message: "P1 violation: 'replace' is banned in hero/value-prop copy. Use 'complement, absorb, subsume, lift, erase the ceremony' instead."
level: error
scope: text
ignorecase: true
tokens:
  - replace
  - replaces
  - replacing
  - replacement
```

**Divergence:** none — template is canonical.

---

### `scripts/hooks/pre-commit-docs` (NEW — doc-lint git hook)

**Analog:** `scripts/hooks/pre-push` (155 lines) — mirror bash-hook structure, colored logging, set flags, and `REPOSIX_HOOKS_QUIET` gating.

**Analog header + logging pattern (`scripts/hooks/pre-push` lines 1–33):**

```bash
#!/usr/bin/env bash
#
# Credential-hygiene pre-push hook (OP-7 from HANDOFF.md).
#
# Rejects a push if any committed file on the outgoing range contains
# a literal Atlassian API token (prefix `ATATT3`) or GitHub PAT (prefix
# `ghp_` or `github_pat_`). ...
#
# To install:
#   bash scripts/install-hooks.sh
#
# Environment:
#   REPOSIX_HOOKS_QUIET=1   suppress informational output; still rejects on hit.

set -euo pipefail

readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly NC='\033[0m'

quiet="${REPOSIX_HOOKS_QUIET:-0}"

log() {
  if [[ "$quiet" != "1" ]]; then
    printf '%b\n' "$*" >&2
  fi
}
```

**Analog exit-discipline pattern (`scripts/hooks/pre-push` lines 124–135):**

```bash
if [[ "$hit" -gt 0 ]]; then
  printf '%b\n' "${RED}✖ pre-push rejected:${NC} ${hit} credential-prefix hit(s) above." >&2
  printf '%b\n' "${YELLOW}→${NC} If this is a false positive ..." >&2
  exit 1
fi

log "${GREEN}✓${NC} no credential prefixes detected."
```

**Target sketch (RESEARCH.md §Example 2 lines 505–524):**

```bash
#!/usr/bin/env bash
# Doc-lint gate — runs Vale on docs/**.md.
# Mirrors scripts/hooks/pre-push pattern (HARD-00).

set -euo pipefail

CHANGED=$(git diff --cached --name-only --diff-filter=ACM | grep -E '^docs/.*\.md$' || true)

if [ -z "$CHANGED" ]; then
    exit 0
fi

if ! command -v vale >/dev/null 2>&1; then
    echo "error: vale not installed. See .planning/phases/30-.../RESEARCH.md §Standard Stack." >&2
    exit 1
fi

echo "==> Vale lint on $CHANGED"
echo "$CHANGED" | xargs vale --config=.vale.ini
```

**Divergence from analog:** `pre-push` operates on ref ranges + git-diff between shas (push-time); `pre-commit-docs` operates on `--cached` staged files (commit-time). Scope is narrower (only `docs/**.md`). Full color/log/quiet pattern should be adopted for consistency — the research example is minimal; the planner should extend it with the `readonly RED/GREEN/YELLOW/NC` + `log()` helper per `pre-push` analog.

**Install path:** `scripts/install-hooks.sh` (lines 28–42) already loops `scripts/hooks/*` and symlinks every executable file. **NO change** needed to `install-hooks.sh` — new hook auto-registers on re-run. Confirmed by reading `install-hooks.sh`.

---

### `scripts/hooks/test-pre-commit-docs.sh` (NEW — hook test)

**Analog:** `scripts/hooks/test-pre-push.sh` (146 lines) — same detached-HEAD, stage-fixture, assert-exit-code pattern.

**Analog setup+cleanup pattern (`scripts/hooks/test-pre-push.sh` lines 36–52):**

```bash
readonly orig_head="$(git rev-parse HEAD)"
readonly orig_branch="$(git symbolic-ref --short -q HEAD || echo '')"
readonly tmp_branch="test-pre-push-$$-$RANDOM"

cleanup() {
  git reset -q --hard "$orig_head" 2>/dev/null || true
  if [[ -n "$orig_branch" ]]; then
    git checkout -q "$orig_branch" 2>/dev/null || true
  else
    git checkout -q "$orig_head" 2>/dev/null || true
  fi
  git branch -D "$tmp_branch" 2>/dev/null || true
  rm -f "${repo_root}/.test-pre-push-fixture.txt"
}
trap cleanup EXIT
```

**Analog test-harness pattern (`scripts/hooks/test-pre-push.sh` lines 54–70):**

```bash
run_and_check() {
  local label="$1"
  local expected="$2"
  local actual=0
  echo "refs/heads/test HEAD HEAD^{commit}~1 $(git rev-parse HEAD^)" \
    | bash "$hook" > /tmp/test-pre-push.out 2>&1 || actual=$?
  if [[ "$actual" == "$expected" ]]; then
    printf '%b\n' "${GREEN}✓${NC} ${label} (exit=${actual})"
    return 0
  else
    printf '%b\n' "${RED}✖ ${label}: expected exit=${expected}, got ${actual}${NC}" >&2
    sed 's/^/    /' /tmp/test-pre-push.out >&2
    return 1
  fi
}
```

**Analog specific-test pattern (`scripts/hooks/test-pre-push.sh` lines 84–91) — fixture injection + staging + assertion:**

```bash
git checkout -q --detach HEAD
echo "${FAKE_ATATT3_TOKEN}" > .test-pre-push-fixture.txt   # var holds a fake ATATT3… test token
git add .test-pre-push-fixture.txt
git -c user.email=test@test -c user.name=test commit -q -m "test: inject fake ATATT3 token"
if ! run_and_check "ATATT3 token rejected" 1; then
  fails=$((fails + 1))
fi
git reset -q --hard HEAD^  # pop the fake commit
```

**Divergence from analog:** The pre-commit-docs hook reads from `git diff --cached` (staged), not push-refs. Test harness needs to `git add` a fixture `.md` file with a banned word (e.g. `docs/_test_banned.md` containing "replace" on a home-path file), then spawn the hook with the staged-file context. Cleanup must unstage + remove fixture. Suggested tests per RESEARCH.md §Wave 0 Gaps (line 1097):
1. Clean docs commit passes.
2. `docs/index.md` with "replace" is rejected.
3. `docs/how-it-works/filesystem.md` with "FUSE" passes (scope exemption).
4. `docs/tutorial.md` with "mount" (bare) is rejected.
5. Code-block-only "FUSE" (fenced) is NOT flagged (Vale `IgnoredScopes` check).

---

### `scripts/check_phase_30_structure.py` (NEW — validation script)

**Analog:** `scripts/check_fixtures.py` — Python validation script with module-level docstring, stdlib-only imports, `check_*` functions, and exit-with-nonzero-on-failure pattern.

**Analog header pattern (`scripts/check_fixtures.py` lines 1–11):**

```python
#!/usr/bin/env python3
"""Validate benchmark fixture files for shape, size, and content safety.

Checks:
  - github_issues.json: JSON array, >=3 issues, required keys, 4-12 KB, no secrets.
  ...

Run from the repository root:
    python3 scripts/check_fixtures.py
"""

from __future__ import annotations

import json
import pathlib
import re
import sys
```

**Analog check-function pattern (`scripts/check_fixtures.py` lines 39–40):**

```python
def check_github() -> list[str]:
    """Check benchmarks/fixtures/github_issues.json.
```

**Divergence from analog:** Same script shape — stdlib-only, one check function per requirement group, returning `list[str]` of error messages. Targets per RESEARCH.md §"Phase Requirements → Test Map" (lines 1063–1077):
- `test -f docs/how-it-works/{filesystem,git,trust-model}.md`
- `grep -c '```mermaid' docs/how-it-works/*.md` ≥ 1 each
- `grep -c '^## ' docs/mental-model.md` == 3
- `grep -iE 'complement|absorb|subsume' docs/vs-mcp-sdks.md` ≥ 1
- `grep -iE '\breplace\b' docs/index.md` == 0
- `grep -c 'social' mkdocs.yml` ≥ 1
- `grep -c 'navigation.footer' mkdocs.yml` ≥ 1

Use `pathlib.Path.read_text()` + string search (not shelling to `grep`), consistent with `check_fixtures.py`.

---

### `scripts/test_phase_30_tutorial.sh` (NEW — end-to-end tutorial check)

**Analog:** `scripts/hooks/test-pre-push.sh` (overall shell-test structure) + `scripts/demo.sh` → `scripts/demos/full.sh` (actual tutorial command sequence).

**Analog shim pattern (`scripts/demo.sh` entire file):**

```bash
#!/usr/bin/env bash
# scripts/demo.sh — backwards-compatible shim. The demo lives at
# scripts/demos/full.sh as of Phase 8-A; this file exists so existing
# "bash scripts/demo.sh" references in docs, README, and user muscle
# memory keep working.
exec bash "$(dirname "$0")/demos/full.sh" "$@"
```

**Divergence from analog:** `test_phase_30_tutorial.sh` must actually RUN the tutorial end-to-end against the simulator (per RESEARCH.md line 1099 — "Promote the ad-hoc bash per OP #4"). It spawns `reposix-sim`, mounts, executes each tutorial command, asserts `$?` per command, and tears down. Pattern = `test-pre-push.sh` cleanup-trap discipline. Source for actual command sequence: `docs/demo.md` lines 60–175 (the simulator-facing subset of the 9-step demo).

---

### `.github/workflows/docs.yml` (MODIFY — add Vale step)

**Analog (baseline):** `.github/workflows/docs.yml` current state at `/home/reuben/workspace/reposix/.github/workflows/docs.yml` (45 lines).

**Current install step (lines 31–34):**

```yaml
      - name: Install MkDocs + Material
        run: |
          python -m pip install --upgrade pip
          python -m pip install mkdocs-material mkdocs-minify-plugin
```

**Analog hook-test pattern (`.github/workflows/ci.yml` lines 54–55):**

```yaml
      - name: Test pre-push credential hook
        run: bash scripts/hooks/test-pre-push.sh
```

**Target additions (RESEARCH.md §Example 3 lines 533–545):**

```yaml
      - name: Install Vale
        run: |
          VALE_VERSION=3.10.0  # pin; bump deliberately
          curl -L "https://github.com/errata-ai/vale/releases/download/v${VALE_VERSION}/vale_${VALE_VERSION}_Linux_64-bit.tar.gz" \
              | tar xz -C /usr/local/bin vale
          vale --version

      - name: Lint docs with Vale (banned-word + progressive-disclosure rules)
        run: vale --config=.vale.ini docs/

      - name: Build strict (fail on broken links)
        run: mkdocs build --strict
```

**Divergence from analog:** Current workflow installs bare `mkdocs-material`. Research recommends upgrading to `mkdocs-material[imaging]` for social cards. Also: the current workflow has paths-filter on `docs/**` + `mkdocs.yml` — add `.vale.ini` + `.vale-styles/**` to the filter so doc-relevant config changes also trigger the build. Verify via `gh run view` post-push per OP #1.

---

### `docs/screenshots/phase-30/*.png` (NEW — Playwright outputs)

**Analog:** `docs/screenshots/` current contents — naming convention `site-<page>.png` and `gh-pages-<page>.png`.

**Analog naming pattern (current dir listing):**

```
docs/screenshots/gh-pages-home-v0.2.png
docs/screenshots/gh-pages-home.png
docs/screenshots/site-architecture.png
docs/screenshots/site-home.png
docs/screenshots/site-security.png
```

**Target names per RESEARCH.md §Example 5 (lines 582–590):**

```
docs/screenshots/phase-30/home-desktop.png
docs/screenshots/phase-30/home-mobile.png
docs/screenshots/phase-30/how-it-works-filesystem-desktop.png
docs/screenshots/phase-30/how-it-works-filesystem-mobile.png
docs/screenshots/phase-30/how-it-works-git-desktop.png
docs/screenshots/phase-30/how-it-works-git-mobile.png
docs/screenshots/phase-30/how-it-works-trust-model-desktop.png
docs/screenshots/phase-30/how-it-works-trust-model-mobile.png
docs/screenshots/phase-30/tutorial-desktop.png
docs/screenshots/phase-30/tutorial-mobile.png
```

**Divergence from analog:** Existing screenshots are flat in `docs/screenshots/`. Phase 30 organizes under a `phase-30/` subdirectory (scales better for future phases; matches `.planning/phases/` directory-per-milestone convention). Resolution/sizing per research: desktop 1280×800, mobile 375×667 via Playwright MCP.

---

### `README.md` (MODIFY — link audit only)

**Analog:** existing `README.md` at repo root.

**Divergence from analog:** This is a minimal mechanical change. The README likely contains links to `docs/architecture.md` and/or `docs/security.md` — both being deleted. Per RESEARCH.md §"Runtime State Inventory" lines 377–378, README.md MUST be grep-audited for dangling links and updated. Use the find-and-replace pattern from research (RESEARCH.md Pitfall 7, lines 421–425) — don't delete the source pages until the audit is clean.
