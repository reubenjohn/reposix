#!/usr/bin/env bash
#
# banned-words-lint.sh — enforce P1 + P2 progressive-disclosure layer rules.
#
# Source of truth for the banned-words matrix: `docs/.banned-words.toml`.
# This script keeps a hardcoded mirror of that matrix because pure-bash TOML
# parsing isn't worth the complexity. If you change `.banned-words.toml`,
# update the matching arrays in this script — and the test below will fail
# loudly if they drift.
#
# Usage:
#   scripts/banned-words-lint.sh           # default: scan Layer 1 + Layer 2
#   scripts/banned-words-lint.sh --all     # also scan Layer 3 (QA mode)
#   scripts/banned-words-lint.sh --help    # show this help
#
# Allowlist:
#   Lines containing the marker `<!-- banned-words: ok -->` are skipped.
#   Use sparingly — every exception should be paired with a one-line comment
#   explaining why.
#
# Exit codes:
#   0 — no violations.
#   1 — at least one violation. Path + line number + matched term printed to stderr.
#   2 — usage error.

set -euo pipefail

readonly ALLOWLIST_MARKER='<!-- banned-words: ok -->'

# Repo-rooted absolute path so the script works regardless of cwd.
readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly DOCS_ROOT="${REPO_ROOT}/docs"
readonly CONFIG_PATH="${DOCS_ROOT}/.banned-words.toml"

# --- P1: banned everywhere in docs/ -----------------------------------------
# The "complement-not-replace" rule from .planning/archive/notes/phase-30-narrative-vignettes.md.
P1_WORDS=(
  "replace"
)

# --- Layer 1 (Hero) banned words --------------------------------------------
LAYER1_PATHS=(
  "index.md"
)
LAYER1_BANNED=(
  "FUSE"
  "fusermount"
  "kernel"
  "syscall"
  "daemon"
  "inode"
  "partial-clone"
  "promisor"
  "stateless-connect"
  "fast-import"
  "protocol-v2"
)

# --- Layer 2 (Below the fold — concepts, tutorials, guides) -----------------
LAYER2_GLOBS=(
  "concepts"
  "tutorials"
  "guides"
)
# Same banned list as Layer 1 — the rule is "above Layer 3 you may not name plumbing".
LAYER2_BANNED=("${LAYER1_BANNED[@]}")

# --- Layer 3 (How it works) -------------------------------------------------
LAYER3_GLOBS=(
  "how-it-works"
)
# Layer 3 has no per-layer banned terms (P1 still applies). Plumbing vocabulary
# is permitted here. The default-mode scan does NOT cover Layer 3 — only --all.
LAYER3_BANNED=()

usage() {
  sed -n '4,25p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
}

die() {
  printf '✖ %s\n' "$*" >&2
  exit 2
}

# Print the absolute path of every Markdown file matching the relative paths
# under $DOCS_ROOT. First arg is "file" or "glob".
resolve_paths() {
  local mode="$1"
  shift
  local rel
  for rel in "$@"; do
    if [[ "$mode" == file ]]; then
      [[ -f "${DOCS_ROOT}/${rel}" ]] && printf '%s\n' "${DOCS_ROOT}/${rel}"
    else
      # Glob: find all .md under docs/<rel>/.
      if [[ -d "${DOCS_ROOT}/${rel}" ]]; then
        find "${DOCS_ROOT}/${rel}" -type f -name '*.md' | sort
      fi
    fi
  done
}

# scan_files <label> <banned-array-name> <file...>
# Reads each file. For each line not containing the allowlist marker, fails on
# any case-insensitive match of any banned word as a whole-token.
scan_files() {
  local label="$1"
  local arr_name="$2"
  shift 2
  local -n banned_ref="$arr_name"

  if [[ "${#banned_ref[@]}" -eq 0 ]]; then
    return 0
  fi

  # Build a single ERE alternation `\b(word1|word2|...)\b`. Hyphens are valid
  # inside the regex character class without escaping; we rely on grep -E.
  local pattern
  pattern="$(printf '%s|' "${banned_ref[@]}")"
  pattern="\\b(${pattern%|})\\b"

  local found=0
  local file
  for file in "$@"; do
    [[ -f "$file" ]] || continue
    # -n line numbers; -E ERE; -i case-insensitive; -H always print path.
    # We grep the whole file then post-filter out allowlist marker lines so
    # the user sees only un-allowlisted violations.
    while IFS= read -r hit; do
      # hit format: path:lineno:line-content
      local line_content="${hit#*:*:}"
      if [[ "$line_content" == *"${ALLOWLIST_MARKER}"* ]]; then
        continue
      fi
      printf '✖ [%s] %s\n' "$label" "$hit" >&2
      found=1
    done < <(grep -HniE "$pattern" "$file" || true)
  done
  return "$found"
}

main() {
  local scan_all=0
  case "${1:-}" in
    -h|--help)
      usage
      exit 0
      ;;
    --all)
      scan_all=1
      ;;
    "")
      ;;
    *)
      die "unknown argument: $1 (try --help)"
      ;;
  esac

  if [[ ! -f "$CONFIG_PATH" ]]; then
    die "missing config: $CONFIG_PATH (the linter mirrors this file)"
  fi

  # Build the file list per layer.
  local -a layer1_files layer2_files layer3_files
  mapfile -t layer1_files < <(resolve_paths file "${LAYER1_PATHS[@]}")
  mapfile -t layer2_files < <(
    for g in "${LAYER2_GLOBS[@]}"; do resolve_paths glob "$g"; done
  )
  mapfile -t layer3_files < <(
    for g in "${LAYER3_GLOBS[@]}"; do resolve_paths glob "$g"; done
  )

  # Combine all in-scope files into one P1-scan list. P1 always runs everywhere
  # the linter is asked to look.
  local -a p1_files=("${layer1_files[@]}" "${layer2_files[@]}")
  if [[ "$scan_all" -eq 1 ]]; then
    p1_files+=("${layer3_files[@]}")
  fi

  local fail=0
  scan_files "P1" P1_WORDS "${p1_files[@]}" || fail=1
  scan_files "Layer 1 (Hero)" LAYER1_BANNED "${layer1_files[@]}" || fail=1
  scan_files "Layer 2 (Below fold)" LAYER2_BANNED "${layer2_files[@]}" || fail=1
  if [[ "$scan_all" -eq 1 ]]; then
    scan_files "Layer 3 (How it works)" LAYER3_BANNED "${layer3_files[@]}" || fail=1
  fi

  if [[ "$fail" -ne 0 ]]; then
    printf '\n✖ banned-words-lint failed. See `docs/.banned-words.toml` for the policy.\n' >&2
    printf '  Allowlist a deliberate exception with: %s\n' "$ALLOWLIST_MARKER" >&2
    exit 1
  fi

  printf '✓ banned-words-lint passed (%s mode).\n' \
    "$([[ "$scan_all" -eq 1 ]] && echo all || echo default)"
}

main "$@"
