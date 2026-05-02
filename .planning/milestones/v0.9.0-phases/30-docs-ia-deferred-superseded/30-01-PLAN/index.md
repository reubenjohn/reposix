---
phase: 30
plan: 01
type: execute
wave: 0
depends_on: []
files_modified:
  - scripts/install-vale.sh
  - scripts/render-mermaid.sh
  - scripts/screenshot-docs.sh
  - scripts/check_phase_30_structure.py
  - scripts/hooks/pre-commit-docs
  - scripts/hooks/test-pre-commit-docs.sh
  - .vale.ini
  - .vale-styles/Reposix/ProgressiveDisclosure.yml
  - .vale-styles/Reposix/NoReplace.yml
  - .vale-styles/config/vocabularies/Reposix/accept.txt
  - .github/workflows/docs.yml
autonomous: true
requirements: [DOCS-09]
must_haves:
  truths:
    - "Vale binary is installed locally (Linux x86_64) and via CI on ubuntu-latest."
    - "`.vale.ini` + rule files enforce P2 (banned Layer-3 terms) and P1 (banned 'replace') with per-file scope exceptions for `docs/how-it-works/**`, `docs/reference/**`, `docs/decisions/**`, `docs/research/**`, `docs/development/**`, and `docs/mental-model.md`."
    - "`scripts/hooks/pre-commit-docs` rejects commits that stage P1/P2 violations above Layer 3 and is auto-installed by `scripts/install-hooks.sh`."
    - "`scripts/render-mermaid.sh` calls `mmdc` to turn one `.md` mermaid fence into an SVG at `docs/assets/diagrams/<name>.svg` (used only for reproducibility; mkdocs-material renders client-side)."
    - "`scripts/screenshot-docs.sh` uses playwright MCP to capture desktop (1280x800) + mobile (375x667) screenshots of listed pages against `http://127.0.0.1:8000` (localhost-only allowlist — SEC)."
    - "`scripts/check_phase_30_structure.py` runs grep-based structural invariants (page exists, mermaid fences count, banned-word absence) and exits non-zero on any failure."
    - "CI `.github/workflows/docs.yml` installs Vale and runs `vale --config=.vale.ini docs/` before `mkdocs build --strict`; both gate the Docs workflow."
  artifacts:
    - path: "scripts/install-vale.sh"
      provides: "Local + CI Vale installer pinned to v3.10.0"
      min_lines: 20
    - path: "scripts/render-mermaid.sh"
      provides: "`mmdc` wrapper rendering a single `.md` mermaid fence to SVG"
      min_lines: 15
    - path: "scripts/screenshot-docs.sh"
      provides: "Playwright screenshot driver (localhost-only)"
      min_lines: 25
    - path: "scripts/check_phase_30_structure.py"
      provides: "Structural invariants linter"
      min_lines: 60
    - path: "scripts/hooks/pre-commit-docs"
      provides: "Vale pre-commit gate on docs/**.md"
      min_lines: 40
    - path: "scripts/hooks/test-pre-commit-docs.sh"
      provides: "5 test cases for the hook (clean/replace/FUSE-above/FUSE-below/code-fence exempt)"
      min_lines: 80
    - path: ".vale.ini"
      provides: "Vale config with per-glob rule scoping"
      contains: "IgnoredScopes = code, code_block"
    - path: ".vale-styles/Reposix/ProgressiveDisclosure.yml"
      provides: "P2 banned-term rule"
      contains: "- FUSE"
    - path: ".vale-styles/Reposix/NoReplace.yml"
      provides: "P1 banned-term rule"
      contains: "- replace"
    - path: ".vale-styles/config/vocabularies/Reposix/accept.txt"
      provides: "Vocabulary allowlist so 'reposix', 'mkdocs', 'jq' are not unknown"
      min_lines: 5
    - path: ".github/workflows/docs.yml"
      provides: "Vale CI step + build strict + gh-deploy"
      contains: "vale --config=.vale.ini"
  key_links:
    - from: "scripts/hooks/pre-commit-docs"
      to: ".vale.ini"
      via: "vale --config=.vale.ini"
      pattern: "vale --config=\\.vale\\.ini"
    - from: ".github/workflows/docs.yml"
      to: "scripts/install-vale.sh"
      via: "bash scripts/install-vale.sh"
      pattern: "install-vale\\.sh"
    - from: "scripts/install-hooks.sh"
      to: "scripts/hooks/pre-commit-docs"
      via: "glob loop — no code change needed, hook auto-registers"
      pattern: "scripts/hooks/\\*"
---

<objective>
Ship the Wave 0 tooling and enforcement backbone that every later plan depends on. After this plan lands:

- A developer can run `bash scripts/install-vale.sh` locally and `vale --config=.vale.ini docs/` returns 0 on the (still-unchanged) docs tree.
- CI runs Vale and fails the Docs workflow on any P1/P2 violation.
- `scripts/hooks/pre-commit-docs` rejects commits that stage a docs file containing "replace" on `docs/index.md` or "FUSE" on `docs/mental-model.md`, and passes commits to `docs/how-it-works/**` that mention FUSE.
- `scripts/screenshot-docs.sh` + `scripts/render-mermaid.sh` + `scripts/check_phase_30_structure.py` exist, are executable, and emit usage text when invoked with no args. They are not yet wired to concrete inputs (that happens in Wave 1/2/4).

Purpose: OP #4 (self-improving infrastructure) + Nyquist gate — every subsequent wave depends on Vale being live, screenshots being runnable, and the structural linter being callable. If Wave 0 is not complete, Wave 1 copy merges cannot be gated on P1/P2 rules.

Output: one installed binary (Vale), five committed scripts, four committed config/rule files, one CI workflow update.

**Locked decisions honored:**
- DOCS-09 fully implemented here.
- Vale binary choice (per RESEARCH.md §Linter Choice) — scope-exception capability is non-negotiable.
- Vale IgnoredScopes = code, code_block — Pitfall 1 mitigation.
- Pre-commit hook mirrors `scripts/hooks/pre-push` pattern (readonly color codes, `REPOSIX_HOOKS_QUIET`, log() helper) — PATTERNS.md §"Bash hook structure".
- `scripts/install-hooks.sh` already loops `scripts/hooks/*` and symlinks each — no edit to it needed (confirmed by reading file).
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@.planning/ROADMAP.md
@.planning/REQUIREMENTS.md
@.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/CONTEXT.md
@.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-RESEARCH.md
@.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-PATTERNS.md
@.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-VALIDATION.md
@.planning/notes/phase-30-narrative-vignettes.md

@scripts/hooks/pre-push
@scripts/install-hooks.sh
@scripts/check_fixtures.py
@.github/workflows/docs.yml

<interfaces>
Existing tooling the executor leverages (no change to these):

- `scripts/install-hooks.sh` lines 28-42 loop every executable file in `scripts/hooks/` and symlink it. New hook auto-registers. Verified by reading the file.
- Vale CLI contract: `vale --config=<path> <file-or-dir>`, exit 0 on clean, non-zero on violations at `level: error`.
- Playwright MCP tool call shape: `browser_navigate` -> `browser_take_screenshot(viewport={width, height})`.
- `mmdc` CLI: `mmdc -i input.mmd -o output.svg` for one diagram per invocation.
</interfaces>
</context>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| CI runner -> internet (Vale binary download) | Fetches a signed tarball from github.com/errata-ai/vale/releases — pin version + verify presence of `vale` executable after extract. |
| Playwright screenshot script -> network | MUST only navigate to `http://127.0.0.1:*` (localhost mkdocs serve). External URLs rejected. |
| Developer commit -> pre-commit hook -> Vale | Hook must be bypass-discouraged (bypass = `git commit --no-verify`, documented but unused in CI). |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-30-01-01 | Tampering | Vale rule files | mitigate | `.vale-styles/` checked into git; any change visible in PR review. Rule files under ~50 lines — reviewable at a glance. |
| T-30-01-02 | Elevation of Privilege | pre-commit-docs hook | mitigate | Hook shipped as `scripts/hooks/pre-commit-docs` (symlinked via `scripts/install-hooks.sh`); CI runs `vale` independently so a developer skipping the hook locally still fails at CI. |
| T-30-01-03 | Information disclosure | `scripts/screenshot-docs.sh` | mitigate | Hardcode URL allowlist to `http://127.0.0.1:*`; reject any argument that does not match. Per CLAUDE.md project OP #5 (reversibility) + source-of-truth SEC constraint. |
| T-30-01-04 | Denial of Service | Vale on massive doc tree | accept | `docs/` is ~20 files; `vale` runs in ~3s per RESEARCH.md §Sampling Rate. Not a realistic DoS vector. |
| T-30-01-05 | Repudiation | CI install step does not verify download checksum | mitigate | Pin `VALE_VERSION=3.10.0`; include `vale --version` check after install that fails pipeline if version drift. Full checksum verification deferred (low-risk, single-maintainer Go binary with widely-observed releases). |
</threat_model>

## Chapters

- [Task 1: Install Vale locally + create installer script + CI integration](./task-1.md)
- [Task 2: Create Vale config + P1/P2 rule files + vocabulary](./task-2.md)
- [Task 3: Pre-commit docs hook + hook test + helper scripts (mermaid, screenshot, structure)](./task-3.md)

<verification>
1. `bash scripts/install-vale.sh && ~/.local/bin/vale --version | grep v3.10.0`.
2. `~/.local/bin/vale --config=.vale.ini docs/reference/cli.md` exits 0; `~/.local/bin/vale --config=.vale.ini docs/index.md` exits non-zero.
3. `bash scripts/hooks/test-pre-commit-docs.sh` — all 5 cases pass.
4. `scripts/check_phase_30_structure.py` runs; errors are limited to "missing" new pages (expected until Wave 1+).
5. `git grep -n 'install-vale.sh\|vale --config' .github/workflows/docs.yml` returns 3+ matches.
6. `bash scripts/install-hooks.sh` lists `pre-commit-docs` in its output.
</verification>

<success_criteria>
- Vale installed on developer host; CI workflow installs it too (via `install-vale.sh`).
- `.vale.ini` + two rule files + vocabulary checked in.
- Pre-commit hook + test committed and test suite green.
- Three helper scripts (mermaid render, screenshot driver, structure linter) committed executable with usage text and SEC-localhost validation.
- CI workflow updated to add Vale lint step before mkdocs build.
- No change needed to `scripts/install-hooks.sh` (auto-registration loop already works).
</success_criteria>

<output>
After completion, create `.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-01-SUMMARY.md` documenting:
- Vale version pinned (3.10.0) and CI integration shape
- Vale rule scope map (which dirs exempt, which per-file exception)
- The 5 pre-commit hook test cases (reference for future doc authors)
- Any calibration observations — e.g., "current docs/index.md fails Vale by design; Wave 1 hero rewrite will fix"
</output>
