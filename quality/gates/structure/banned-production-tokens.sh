#!/usr/bin/env bash
# quality/gates/structure/banned-production-tokens.sh — RBF-FW-04
# Scans crates/**/*.rs for phase-ID tokens (\bP\d{2,3}-\d+\b) that
# leak internal phase numbers into production source. Implements
# catalog row structure/banned-production-tokens.
#
# ─────────────────────────────────────────────────────────────────────────
# Banned-token regex scope (mirrored in CLAUDE.md
# § "Quality Gates" via 89-08, and quality/PROTOCOL.md):
#
#   CATCHES:    v0.13+ phase numbers — \bP\d{2,3}-\d+\b
#               (e.g. P79-02, P83-01, P150-01)
#
#   INTENTIONALLY MISSES:
#               - v0.8/v0.9-era audit IDs — P\d-\d
#                 (e.g. P1-1, P1-5, P0-2 in crates/reposix-core/src/error.rs)
#               - generic phrases that happen to contain "P1-1 not implemented"
#                 in stderr — would require an NLP-grade classifier
#
#   FORWARD CONVENTION: future audit-ID conventions in this repo SHOULD
#               either (a) adopt P\d{2,3}- numbering (so the linter catches
#               them) OR (b) use a different prefix entirely (e.g. AUD-1,
#               QA-2-3) so the framework does not accidentally ban them.
#
#   This trade-off is intentional. The linter's
#   purpose is to block accidental v0.13+ phase-ID leaks (the F-K class
#   reposix's owner repeatedly catches in user-facing stderr); broader
#   phase-prefix scrubbing is out of scope and would require a different
#   regex shape + an allowlist of legitimate non-phase usages.
# ─────────────────────────────────────────────────────────────────────────
#
# Tightened from CONTEXT D-04c regex \bP[0-9]+-[0-9]+\b → \bP\d{2,3}-\d+\b
# per 89-CONTEXT.md (Q-DEFERRAL-1) finding (avoids false positives on legitimate
# P1-1 / P0-2 code-quality audit IDs in crates/reposix-core/src/error.rs).
#
# Allowlist marker: append `// banned-words: ok` to a line for justified exceptions.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "$REPO_ROOT"

ALLOWLIST_MARKER='// banned-words: ok'
PATTERN='\bP[0-9]{2,3}-[0-9]+\b'

violations=0
matched_files=()

while IFS= read -r -d '' file; do
  while IFS=: read -r path lineno content; do
    [[ "$content" == *"$ALLOWLIST_MARKER"* ]] && continue
    printf '✖ banned production token: %s:%s: %s\n' "$path" "$lineno" "$content" >&2
    violations=1
    matched_files+=("$path:$lineno")
  done < <(grep -nHE "$PATTERN" "$file" 2>/dev/null || true)
done < <(find crates -type f -name '*.rs' \
  ! -path '*/tests/*' ! -path '*/target/*' -print0)

if [[ $violations -eq 1 ]]; then
  echo "" >&2
  echo "owner_hint: rename or remove the phase-ID token; or add '$ALLOWLIST_MARKER' to the line" >&2
  echo "see: quality/catalogs/freshness-invariants.json row structure/banned-production-tokens" >&2
  exit 1
fi
echo "PASS: no banned production-error tokens in crates/**/*.rs"
