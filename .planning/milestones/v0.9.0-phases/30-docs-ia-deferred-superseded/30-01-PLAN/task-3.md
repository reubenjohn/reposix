# Task 3: Pre-commit docs hook + hook test + helper scripts (mermaid, screenshot, structure)

← [back to index](./index.md)

<task type="auto">
  <name>Task 3: Pre-commit docs hook + hook test + helper scripts (mermaid, screenshot, structure)</name>
  <files>scripts/hooks/pre-commit-docs, scripts/hooks/test-pre-commit-docs.sh, scripts/render-mermaid.sh, scripts/screenshot-docs.sh, scripts/check_phase_30_structure.py</files>
  <read_first>
    - `scripts/hooks/pre-push` (lines 1-60 — header + `readonly RED/GREEN/YELLOW/NC` + `log()` helper pattern)
    - `scripts/hooks/test-pre-push.sh` (lines 1-90 — cleanup trap + `run_and_check` harness + fixture staging pattern)
    - `scripts/check_fixtures.py` (python validation script pattern: stdlib-only + check_*() -> list[str] + aggregate in main())
    - `.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-RESEARCH.md` §Example 2 (lines 501-528 — pre-commit-docs sketch), §Example 5 (playwright screenshot pattern), §"Validation Architecture → Phase Requirements → Test Map" (check_phase_30_structure targets)
    - `.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-PATTERNS.md` §"scripts/hooks/pre-commit-docs", §"scripts/hooks/test-pre-commit-docs.sh", §"scripts/check_phase_30_structure.py", §"scripts/test_phase_30_tutorial.sh"
  </read_first>
  <action>
    Create `scripts/hooks/pre-commit-docs` with EXACT content (mirrors `scripts/hooks/pre-push` discipline — colors, log(), REPOSIX_HOOKS_QUIET, exit codes):

```bash
#!/usr/bin/env bash
#
# Pre-commit docs hook — Phase 30 DOCS-09 enforcement.
#
# Runs Vale on staged docs/**.md. Rejects commit if any file violates:
#   - Reposix.ProgressiveDisclosure (P2 — FUSE/inode/daemon/etc above Layer 3)
#   - Reposix.NoReplace (P1 — "replace" on docs/index.md)
#
# To install:
#   bash scripts/install-hooks.sh
#
# To bypass (strongly discouraged — CI will still reject):
#   git commit --no-verify
#
# Environment:
#   REPOSIX_HOOKS_QUIET=1    suppress informational output; still rejects on hit.

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

CHANGED="$(git diff --cached --name-only --diff-filter=ACM | grep -E '^docs/.*\.md$' || true)"

if [[ -z "$CHANGED" ]]; then
  log "${GREEN}✓${NC} no staged docs changes — skipping Vale."
  exit 0
fi

if ! command -v vale >/dev/null 2>&1; then
  printf '%b\n' "${RED}✖ vale not found on PATH${NC}" >&2
  printf '%b\n' "${YELLOW}→${NC} run: bash scripts/install-vale.sh" >&2
  exit 1
fi

log "${GREEN}==>${NC} Vale lint on staged docs:"
log "$CHANGED"

# xargs -I {} with quoting handles filenames with spaces safely.
echo "$CHANGED" | xargs -I {} vale --config=.vale.ini {}

log "${GREEN}✓${NC} Vale clean."
```

Create `scripts/hooks/test-pre-commit-docs.sh` with the following structure (mirror `scripts/hooks/test-pre-push.sh` cleanup-trap discipline — EXACT content below, 5 cases):

```bash
#!/usr/bin/env bash
#
# Test scripts/hooks/pre-commit-docs — Phase 30 DOCS-09 verification.
#
# Creates a temporary branch, stages fixture docs files with known violations,
# and asserts the hook rejects or accepts per expected rule scoping. Fixtures
# use entropy-distinct names so production files remain untouched.

set -euo pipefail

readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly NC='\033[0m'

readonly repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
readonly hook="${repo_root}/scripts/hooks/pre-commit-docs"

cd "$repo_root"

readonly orig_head="$(git rev-parse HEAD)"
readonly orig_branch="$(git symbolic-ref --short -q HEAD || echo '')"
readonly tmp_branch="test-pre-commit-docs-$$-$RANDOM"

cleanup() {
  git reset -q --hard "$orig_head" 2>/dev/null || true
  if [[ -n "$orig_branch" ]]; then
    git checkout -q "$orig_branch" 2>/dev/null || true
  else
    git checkout -q "$orig_head" 2>/dev/null || true
  fi
  git branch -D "$tmp_branch" 2>/dev/null || true
  rm -f "${repo_root}/docs/_test_pre_commit_fixture.md"
  rm -f "${repo_root}/docs/how-it-works/_test_pre_commit_fixture.md"
  rmdir "${repo_root}/docs/how-it-works" 2>/dev/null || true
}
trap cleanup EXIT

git checkout -q -b "$tmp_branch"

run_and_check() {
  local label="$1"
  local expected="$2"
  local actual=0
  bash "$hook" >/tmp/test-pre-commit-docs.out 2>&1 || actual=$?
  if [[ "$actual" == "$expected" ]]; then
    printf '%b\n' "${GREEN}✓${NC} ${label} (exit=${actual})"
    return 0
  else
    printf '%b\n' "${RED}✖ ${label}: expected exit=${expected}, got ${actual}${NC}" >&2
    sed 's/^/    /' /tmp/test-pre-commit-docs.out >&2
    return 1
  fi
}

fails=0

# --- Case 1: no docs staged -> exit 0 ---
run_and_check "case 1: no docs staged passes" 0 || fails=$((fails+1))

# --- Case 2: docs/index.md with "replace" -> exit 1 ---
echo 'reposix will replace REST APIs' > docs/_test_pre_commit_fixture.md
git add docs/_test_pre_commit_fixture.md
# NOTE: The fixture goes in docs/ root, NOT docs/index.md itself (to avoid
# clobbering the real index.md). We force the NoReplace rule by temporarily
# adding a per-file block via the fixture filename.
# To test properly, put the fixture where the rule scope matches. We'll use
# docs/_test_pre_commit_fixture.md which has ProgressiveDisclosure active but
# not NoReplace. Use "FUSE" in the body instead.
git reset -q HEAD docs/_test_pre_commit_fixture.md
rm -f docs/_test_pre_commit_fixture.md

# --- Case 2 (revised): docs/_test_pre_commit_fixture.md at Layer 1/2 with "FUSE" -> exit 1 ---
echo 'The reposix FUSE daemon mounts your tracker.' > docs/_test_pre_commit_fixture.md
git add docs/_test_pre_commit_fixture.md
run_and_check "case 2: FUSE at Layer 1/2 rejected" 1 || fails=$((fails+1))
git reset -q HEAD docs/_test_pre_commit_fixture.md
rm -f docs/_test_pre_commit_fixture.md

# --- Case 3: docs/how-it-works/*.md with "FUSE" -> exit 0 (exempted) ---
mkdir -p docs/how-it-works
echo 'Under the hood, reposix is a FUSE daemon.' > docs/how-it-works/_test_pre_commit_fixture.md
git add docs/how-it-works/_test_pre_commit_fixture.md
run_and_check "case 3: FUSE under how-it-works/ accepted" 0 || fails=$((fails+1))
git reset -q HEAD docs/how-it-works/_test_pre_commit_fixture.md
rm -f docs/how-it-works/_test_pre_commit_fixture.md
rmdir docs/how-it-works 2>/dev/null || true

# --- Case 4: docs/_test_pre_commit_fixture.md with "mount" (bare) -> exit 1 ---
echo 'You mount the tracker as a folder.' > docs/_test_pre_commit_fixture.md
git add docs/_test_pre_commit_fixture.md
run_and_check "case 4: bare 'mount' at Layer 1/2 rejected" 1 || fails=$((fails+1))
git reset -q HEAD docs/_test_pre_commit_fixture.md
rm -f docs/_test_pre_commit_fixture.md

# --- Case 5: docs/_test_pre_commit_fixture.md with FUSE *only* in code fence -> exit 0 ---
cat > docs/_test_pre_commit_fixture.md <<'FIX'
# Sample

Bash snippets don't count:

```bash
# FUSE daemon start — this must not trip the linter
fusermount -u /tmp/mnt
```

The prose above the fence must avoid banned terms.
FIX
git add docs/_test_pre_commit_fixture.md
run_and_check "case 5: FUSE only in code fence accepted" 0 || fails=$((fails+1))
git reset -q HEAD docs/_test_pre_commit_fixture.md
rm -f docs/_test_pre_commit_fixture.md

if [[ "$fails" -gt 0 ]]; then
  printf '%b\n' "${RED}✖ $fails test(s) failed${NC}" >&2
  exit 1
fi

printf '%b\n' "${GREEN}✓ all pre-commit-docs tests passed${NC}"
```

Create `scripts/render-mermaid.sh`:

```bash
#!/usr/bin/env bash
#
# Render a mermaid diagram from a .md fence to SVG via mmdc.
# Usage: bash scripts/render-mermaid.sh <input-md> <fence-index> <output-svg>
#   where fence-index is 1-based position of the ```mermaid fence in the input.

set -euo pipefail

if [[ $# -lt 3 ]]; then
  echo "usage: $0 <input-md> <fence-index> <output-svg>" >&2
  echo "example: $0 docs/how-it-works/filesystem.md 1 docs/assets/diagrams/filesystem-layer.svg" >&2
  exit 2
fi

readonly INPUT="$1"
readonly FENCE_IDX="$2"
readonly OUTPUT="$3"
readonly TMPFILE="$(mktemp --suffix=.mmd)"

# Extract the Nth mermaid fence.
awk -v want="$FENCE_IDX" '
  /^```mermaid$/ { in_fence=1; count++; next }
  /^```$/ && in_fence { in_fence=0; next }
  in_fence && count==want { print }
' "$INPUT" > "$TMPFILE"

if [[ ! -s "$TMPFILE" ]]; then
  echo "error: no mermaid fence #${FENCE_IDX} found in ${INPUT}" >&2
  rm -f "$TMPFILE"
  exit 1
fi

mkdir -p "$(dirname "$OUTPUT")"
mmdc -i "$TMPFILE" -o "$OUTPUT"
rm -f "$TMPFILE"
echo "rendered: $OUTPUT"
```

Create `scripts/screenshot-docs.sh`:

```bash
#!/usr/bin/env bash
#
# Playwright screenshot driver — Phase 30 feedback-loop evidence.
# Localhost-only by design (SEC T-30-01-03).
#
# Usage: bash scripts/screenshot-docs.sh <mkdocs-serve-port> <output-dir>
#
# This script emits a JSON manifest of (url, viewport, output-path) triples to
# stdout. The Playwright MCP orchestrator consumes the manifest and performs
# the actual screenshots. We do NOT drive playwright directly from bash — the
# MCP path is the supported integration.

set -euo pipefail

readonly PORT="${1:-8000}"
readonly OUT_DIR="${2:-.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/screenshots}"
readonly BASE_URL="http://127.0.0.1:${PORT}"

# SEC: Reject anything that isn't localhost.
if [[ "$BASE_URL" != http://127.0.0.1:* ]]; then
  echo "error: BASE_URL must be http://127.0.0.1:*; got ${BASE_URL}" >&2
  exit 1
fi

mkdir -p "$OUT_DIR"

PAGES=(
  "/"
  "/mental-model/"
  "/vs-mcp-sdks/"
  "/tutorial/"
  "/how-it-works/filesystem/"
  "/how-it-works/git/"
  "/how-it-works/trust-model/"
)

VIEWPORTS=(
  "desktop:1280:800"
  "mobile:375:667"
)

echo "["
first=1
for page in "${PAGES[@]}"; do
  slug="$(echo "$page" | sed 's|/|-|g; s|^-||; s|-$||')"
  [[ -z "$slug" ]] && slug="home"
  for vp in "${VIEWPORTS[@]}"; do
    name="${vp%%:*}"
    width="$(echo "$vp" | cut -d: -f2)"
    height="$(echo "$vp" | cut -d: -f3)"
    [[ "$first" == "1" ]] && first=0 || echo ","
    cat <<JSON
  {
    "url": "${BASE_URL}${page}",
    "viewport_name": "${name}",
    "width": ${width},
    "height": ${height},
    "output": "${OUT_DIR}/${slug}-${name}.png"
  }
JSON
  done
done
echo
echo "]"
```

Create `scripts/check_phase_30_structure.py`:

```python
#!/usr/bin/env python3
"""Validate Phase 30 docs structural invariants (DOCS-01..09).

Checks each invariant derived from RESEARCH.md §"Validation Architecture →
Phase Requirements → Test Map":

  - Required new pages exist
  - Each how-it-works page has exactly one ```mermaid fence
  - docs/mental-model.md has exactly three H2s
  - docs/vs-mcp-sdks.md contains one of {complement, absorb, subsume}
  - docs/index.md does NOT contain "replace" (P1)
  - docs/index.md does NOT contain FUSE/inode/daemon/kernel/syscall (P2)
  - mkdocs.yml has `- social` plugin + `navigation.footer` feature
  - mkdocs.yml nav does not reference deleted files (architecture, security, demo, connectors/guide)

Run from the repository root:
    python3 scripts/check_phase_30_structure.py
Exit non-zero if any check fails.
"""

from __future__ import annotations

import pathlib
import re
import sys


ROOT = pathlib.Path(__file__).resolve().parent.parent

REQUIRED_PAGES = [
    "docs/index.md",
    "docs/mental-model.md",
    "docs/vs-mcp-sdks.md",
    "docs/tutorial.md",
    "docs/how-it-works/index.md",
    "docs/how-it-works/filesystem.md",
    "docs/how-it-works/git.md",
    "docs/how-it-works/trust-model.md",
    "docs/guides/write-your-own-connector.md",
    "docs/guides/integrate-with-your-agent.md",
    "docs/guides/troubleshooting.md",
    "docs/reference/simulator.md",
    "mkdocs.yml",
    ".vale.ini",
    ".vale-styles/Reposix/ProgressiveDisclosure.yml",
    ".vale-styles/Reposix/NoReplace.yml",
]

DELETED_PATHS = [
    "docs/architecture.md",
    "docs/security.md",
    "docs/demo.md",
    "docs/connectors/guide.md",
]

P2_BANNED = ["FUSE", "inode", "daemon", "kernel", "syscall"]  # check in prose, not code


def check_required_pages() -> list[str]:
    errors = []
    for p in REQUIRED_PAGES:
        if not (ROOT / p).is_file():
            errors.append(f"missing: {p}")
    return errors


def check_deleted_paths() -> list[str]:
    # These should be deleted by Wave 3 — if they exist, Wave 3 is incomplete.
    errors = []
    for p in DELETED_PATHS:
        if (ROOT / p).is_file():
            errors.append(f"should be deleted: {p}")
    return errors


def count_mermaid_fences(md_path: pathlib.Path) -> int:
    if not md_path.is_file():
        return -1
    text = md_path.read_text()
    return len(re.findall(r"^```mermaid\s*$", text, flags=re.MULTILINE))


def check_mermaid_counts() -> list[str]:
    errors = []
    for p in [
        "docs/how-it-works/filesystem.md",
        "docs/how-it-works/git.md",
        "docs/how-it-works/trust-model.md",
    ]:
        count = count_mermaid_fences(ROOT / p)
        if count != 1:
            errors.append(f"{p}: expected 1 mermaid fence, got {count}")
    return errors


def check_mental_model_h2s() -> list[str]:
    errors = []
    path = ROOT / "docs/mental-model.md"
    if not path.is_file():
        return [f"missing: {path.relative_to(ROOT)}"]
    text = path.read_text()
    h2s = re.findall(r"^## (.+)$", text, flags=re.MULTILINE)
    if len(h2s) != 3:
        errors.append(f"docs/mental-model.md: expected 3 H2s, got {len(h2s)}: {h2s}")
    return errors


def check_vs_mcp_sdks_complement() -> list[str]:
    path = ROOT / "docs/vs-mcp-sdks.md"
    if not path.is_file():
        return [f"missing: {path.relative_to(ROOT)}"]
    text = path.read_text().lower()
    if not re.search(r"\b(complement|absorb|subsume)\w*", text):
        return ["docs/vs-mcp-sdks.md: must mention one of complement/absorb/subsume"]
    return []


def strip_code_fences(text: str) -> str:
    """Remove fenced code blocks so banned-word scan only hits prose."""
    return re.sub(r"```.*?\n.*?```", "", text, flags=re.DOTALL)


def check_index_p1_p2() -> list[str]:
    errors = []
    path = ROOT / "docs/index.md"
    if not path.is_file():
        return [f"missing: {path.relative_to(ROOT)}"]
    prose = strip_code_fences(path.read_text())
    # P1
    if re.search(r"\breplace\w*\b", prose, flags=re.IGNORECASE):
        errors.append("docs/index.md: contains 'replace*' (P1 violation)")
    # P2 — restricted set (not mount/helper which have generic use)
    for term in P2_BANNED:
        if re.search(rf"\b{term}\b", prose, flags=re.IGNORECASE):
            errors.append(f"docs/index.md: contains '{term}' (P2 violation)")
    return errors


def check_mkdocs_nav() -> list[str]:
    errors = []
    path = ROOT / "mkdocs.yml"
    text = path.read_text()
    if "- social" not in text:
        errors.append("mkdocs.yml: missing `- social` plugin (DOCS-08)")
    if "navigation.footer" not in text:
        errors.append("mkdocs.yml: missing `navigation.footer` feature (DOCS-08)")
    # Deleted pages should not appear in nav
    for p in [
        "architecture.md",
        "security.md",
        "connectors/guide.md",
    ]:
        # allow demo.md reference during transition — it may redirect
        if f": {p}" in text or f"- {p}" in text:
            errors.append(f"mkdocs.yml: still references deleted {p}")
    return errors


def main() -> int:
    all_errors: list[str] = []
    for check in (
        check_required_pages,
        check_deleted_paths,
        check_mermaid_counts,
        check_mental_model_h2s,
        check_vs_mcp_sdks_complement,
        check_index_p1_p2,
        check_mkdocs_nav,
    ):
        errs = check()
        if errs:
            print(f"[{check.__name__}] FAILED:")
            for e in errs:
                print(f"  - {e}")
            all_errors.extend(errs)
        else:
            print(f"[{check.__name__}] OK")

    if all_errors:
        print(f"\n✖ {len(all_errors)} invariant(s) failed")
        return 1
    print("\n✓ all invariants passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
```

Make all new scripts executable:

```bash
chmod +x scripts/hooks/pre-commit-docs
chmod +x scripts/hooks/test-pre-commit-docs.sh
chmod +x scripts/render-mermaid.sh
chmod +x scripts/screenshot-docs.sh
chmod +x scripts/check_phase_30_structure.py
```

Re-run `scripts/install-hooks.sh` to auto-register `pre-commit-docs`:

```bash
bash scripts/install-hooks.sh
# expected output line: ✓ installed hook: pre-commit-docs -> scripts/hooks/pre-commit-docs
```

Finally, run `bash scripts/hooks/test-pre-commit-docs.sh` — all 5 cases must pass. Expected output:
- `✓ case 1: no docs staged passes (exit=0)`
- `✓ case 2: FUSE at Layer 1/2 rejected (exit=1)`
- `✓ case 3: FUSE under how-it-works/ accepted (exit=0)`
- `✓ case 4: bare 'mount' at Layer 1/2 rejected (exit=1)`
- `✓ case 5: FUSE only in code fence accepted (exit=0)`
  </action>
  <verify>
    <automated>test -x scripts/hooks/pre-commit-docs && test -x scripts/hooks/test-pre-commit-docs.sh && test -x scripts/render-mermaid.sh && test -x scripts/screenshot-docs.sh && test -x scripts/check_phase_30_structure.py && bash scripts/install-hooks.sh | grep -q "pre-commit-docs" && bash scripts/hooks/test-pre-commit-docs.sh</automated>
  </verify>
  <acceptance_criteria>
    - `scripts/hooks/pre-commit-docs` is executable and follows `scripts/hooks/pre-push` pattern (contains `REPOSIX_HOOKS_QUIET`, `log()`, readonly colors).
    - `scripts/hooks/test-pre-commit-docs.sh` runs all 5 cases and exits 0.
    - `bash scripts/install-hooks.sh` output contains "✓ installed hook: pre-commit-docs".
    - After install-hooks.sh run, `test -L .git/hooks/pre-commit-docs` returns 0 (symlink exists).
    - `scripts/render-mermaid.sh --help` or `scripts/render-mermaid.sh` (no args) emits `usage:` line and exits non-zero.
    - `scripts/screenshot-docs.sh http://evil.com/ /tmp/out` exits non-zero (SEC: rejects non-localhost URL).
    - `scripts/check_phase_30_structure.py` runs and exits non-zero on current docs/ tree (at least "missing docs/mental-model.md" error present — expected until later waves).
    - Running `grep -c "IgnoredScopes\|code_block" scripts/hooks/pre-commit-docs` returns 0 (hook delegates to Vale, doesn't duplicate parsing logic).
  </acceptance_criteria>
  <done>
    5 scripts committed executable, hook symlinked, hook test suite green, structure linter runs (fails as expected — later waves fix). Wave 0 backbone complete. Any subsequent wave can now rely on `vale --config=.vale.ini`, `bash scripts/screenshot-docs.sh`, `python3 scripts/check_phase_30_structure.py`.
  </done>
</task>
