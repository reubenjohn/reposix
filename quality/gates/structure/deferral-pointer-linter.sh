#!/usr/bin/env bash
# quality/gates/structure/deferral-pointer-linter.sh — RBF-FW-05
# Greps crates/ for three deferral-pointer regex patterns and BLOCKs if
# the named phase number has no PLAN artifact under .planning/phases/N-*/
# (the deferred work has nowhere to land —
# substrate-gap-deferred-but-no-substrate).
#
# ─────────────────────────────────────────────────────────────────────────
# A deferral-pattern match that yields ZERO extracted phase numbers (e.g.,
# a comment `// substrate-gap-deferred` with no PNN suffix) ALSO BLOCKs.
# The linter's contract is "every deferral pointer cross-references a real
# downstream PLAN", which a no-PNN deferral cannot satisfy. Two of the
# three current deferral patterns DO require a P\d+ suffix; the BLOCK
# guards against FUTURE drift where a developer adds a bare
# `// substrate-gap-deferred` without naming a phase.
#
# PNN extraction is PHRASE-SCOPED, not line-scoped (design choice):
# phase numbers are extracted from the pattern-matched FRAGMENT only
# (`grep -oE "$pat" <<< "$content"`), never from the whole line. Why:
# lines can carry ADJACENT phase-number-bearing text that is NOT a
# deferral target — e.g. the 89-02 allowlist marker
# `// banned-words: ok — P91 RBF-A-03 will remove this string` on
# crates/reposix-cli/src/attach.rs:163. Line-scoped extraction would
# misread that P91 as a deferral pointer and falsely BLOCK. For the
# `substrate-gap-deferred` pattern (which carries no PNN itself), the
# phase number is extracted from the remainder of the line AFTER the
# match (`substrate-gap-deferred until P95 lands` → P95); a bare match
# with nothing extractable after it → no-PNN BLOCK.
#
# PLAN-existence check accepts BOTH plan layouts (post-split reality):
#   - flat files:  .planning/phases/89-*/89-05-PLAN.md
#   - directories: .planning/phases/79-*/79-01-PLAN/{index,t01,...}.md
# via `find <phase-dir> -path '*PLAN*' -name '*.md' -print -quit`.
# ─────────────────────────────────────────────────────────────────────────
#
# Existence-only check per CONTEXT D-05c "Default for P89"; content
# cross-reference is a P90/P95 polish.
#
# Implements catalog row structure/deferral-pointer-linter.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "$REPO_ROOT"

# Pattern 2 (`lands? (alongside|in) ... P[0-9]+`) tolerates a bounded run of
# up to 5 intervening words between the verb phrase and the phase number
# (GOOD-TO-HAVES-02, RESOLVED P97). F-K6-verbatim required the PNN to
# immediately follow the verb, so a phrasing like
# `// github/confluence/jira land alongside the integration tests in P79-03`
# was INVISIBLE to the linter — neither the orphan-PNN nor no-PNN BLOCK fired.
# The intervening class is `[a-zA-Z-]+ ` (letters/hyphens + trailing space):
# it cannot traverse a PNN (the digits in `P79-03` break `[a-zA-Z-]+`), so
# `grep -oE` always stops at the FIRST phase number in the fragment and never
# swallows an adjacent allowlist-marker PNN (e.g. a trailing
# `// banned-words: ok — P91 ...` on the same line stays out of the match).
# Synthetic BLOCK scenario (proves the widened shape catches it):
#   `// this lands alongside the follow-up work in P999` → orphan-PNN BLOCK
#   (no .planning/phases/999-*/ PLAN artifact exists).
PATTERNS=(
  'not yet wired in P[0-9]+'
  'lands? (alongside|in) ([a-zA-Z-]+ ){0,5}P[0-9]+'
  'substrate-gap-deferred'
)

# True iff at least one .planning/phases/<N>-*/ dir contains a PLAN
# markdown artifact (flat *PLAN*.md file OR *PLAN*/ dir with .md inside).
phase_has_plan() {
  local n="$1" dir hit
  for dir in .planning/phases/"${n}"-*/; do
    [[ -d "$dir" ]] || continue
    hit="$(find "$dir" -path '*PLAN*' -name '*.md' -print -quit 2>/dev/null)"
    [[ -n "$hit" ]] && return 0
  done
  return 1
}

violations=0
total_matches=0

for pat in "${PATTERNS[@]}"; do
  while IFS=: read -r path lineno content; do
    total_matches=$((total_matches + 1))

    # Phrase-scoped phase-number extraction (see header). For the two
    # PNN-carrying patterns, extract from the matched fragment only; for
    # substrate-gap-deferred, extract from the line remainder AFTER the
    # match. Count extractions — if zero, BLOCK explicitly.
    if [[ "$pat" == 'substrate-gap-deferred' ]]; then
      remainder="${content#*substrate-gap-deferred}"
      extracted_phases="$(grep -oE 'P[0-9]+' <<< "$remainder" | sed 's/^P//' | sort -u || true)"
    else
      extracted_phases="$(grep -oE "$pat" <<< "$content" | grep -oE 'P[0-9]+' | sed 's/^P//' | sort -u || true)"
    fi
    extracted_count="$(grep -c '^[0-9]' <<< "$extracted_phases" || true)"

    if [[ "$extracted_count" -eq 0 ]]; then
      # Bare deferral pattern with NO phase number — a silent-pass false
      # negative if unguarded. Block explicitly.
      printf '✖ %s:%s deferral pattern matched but no PNN suffix found\n' \
        "$path" "$lineno" >&2
      printf '   matched-line: %s\n' "$content" >&2
      printf '   pattern: %s\n' "$pat" >&2
      printf '   owner_hint: append the target phase number (e.g. `... until P95 lands`) OR scrub the deferral pointer\n' >&2
      violations=1
      continue
    fi

    # ≥1 phase extracted — verify each resolves to an existing PLAN artifact.
    while IFS= read -r N; do
      [[ -z "$N" ]] && continue
      if ! phase_has_plan "$N"; then
        printf '✖ %s:%s references P%s but no PLAN artifact exists under .planning/phases/%s-*/\n' \
          "$path" "$lineno" "$N" "$N" >&2
        printf '   matched-line: %s\n' "$content" >&2
        violations=1
      fi
    done <<< "$extracted_phases"
  done < <(grep -rnHE "$pat" crates/ 2>/dev/null || true)
done

if [[ $violations -eq 1 ]]; then
  echo "" >&2
  echo "owner_hint: create the named phase's PLAN artifact under .planning/phases/N-*/ (flat N-NN-PLAN.md or N-NN-PLAN/ dir)" >&2
  echo "            OR scrub the deferral pointer if the deferred work shipped" >&2
  echo "            OR append the missing PNN suffix if the pattern is bare" >&2
  echo "see: quality/catalogs/freshness-invariants.json row structure/deferral-pointer-linter" >&2
  exit 1
fi
echo "PASS: ${total_matches} deferral-pointer match(es) in crates/ all resolve to existing phase dirs with PLAN files (and all carry PNN suffixes)"
