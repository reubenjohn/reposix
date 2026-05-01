#!/usr/bin/env bash
# quality/gates/structure/file-size-limits.sh
#
# Progressive disclosure / readability gate. Refuses files past
# extension-specific size budgets. Tagged at pre-commit + pre-push +
# pre-pr so it fires for every trigger; contributors without local
# hooks still get gated by CI.
#
# Default: exit 1 on any violation (BLOCK).
# --warn-only: prints violations to stderr but exits 0 (transitional).
#
# Limits live in this file (not the catalog row) so the check is
# self-contained and reviewable in one diff.
set -euo pipefail

WARN_ONLY=0
for arg in "$@"; do
  case "$arg" in
    --warn-only) WARN_ONLY=1 ;;
    *) echo "unknown flag: $arg" >&2; exit 2 ;;
  esac
done

cd "$(git rev-parse --show-toplevel)"

# Final exclusion list (per user direction):
# - auto-generated / lockfiles / catalog state
# - non-source test fixtures
# - .rs source under crates/ DEFERRED to next milestone (bulk refactor)
# Everything else IS in scope: .planning/, docs/research/, quality/gates/,
# quality/runners/, scripts/, root *.md, .claude/skills/, etc.
EXCLUDED_PATTERNS=(
  '^Cargo\.lock$'
  '^quality/catalogs/.*\.json$'
  '^quality/reports/'
  '^crates/.*/tests/fixtures/'
  '^CHANGELOG\.md$'
  '^crates/.*\.rs$'
)

violations=()
total_scanned=0

while IFS= read -r file; do
  skip=0
  for pattern in "${EXCLUDED_PATTERNS[@]}"; do
    if [[ "$file" =~ $pattern ]]; then skip=1; break; fi
  done
  [[ $skip -eq 1 ]] && continue

  limit=
  hint=
  base=$(basename -- "$file")
  if [[ "$base" == "CLAUDE.md" ]]; then
    limit=40000
    hint='move detail to .claude/skills/ or linked docs'
  elif [[ "$file" == .claude/skills/* ]]; then
    limit=10000
    hint='split skill into smaller files or linked pages'
  else
    case "$file" in
      *.md)         limit=20000; hint='split into smaller files (child pages, linked docs)' ;;
      *.rs)         limit=20000; hint='split modules or improve boundaries' ;;
      *.py)         limit=15000; hint='split into separate modules' ;;
      *.sh|*.bash)  limit=10000; hint='factor into composable scripts' ;;
    esac
  fi

  [[ -z "$limit" ]] && continue
  total_scanned=$((total_scanned + 1))

  size=$(wc -c < "$file" | tr -d ' ')
  if [[ "$size" -gt "$limit" ]]; then
    violations+=("$file is $size chars (limit: $limit) — $hint")
  fi
done < <(git ls-files)

if [[ "${#violations[@]}" -gt 0 ]]; then
  echo "file-size-limits: ${#violations[@]} file(s) over budget (scanned: $total_scanned)" >&2
  for v in "${violations[@]}"; do
    echo "  $v" >&2
  done
  if [[ $WARN_ONLY -eq 1 ]]; then
    echo "(--warn-only mode; exiting 0)" >&2
    exit 0
  fi
  exit 1
fi

echo "PASS: file-size-limits — $total_scanned tracked files within budgets"
