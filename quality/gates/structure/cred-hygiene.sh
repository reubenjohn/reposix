#!/usr/bin/env bash
# quality/gates/structure/cred-hygiene.sh -- structure/cred-hygiene verifier (P60 Wave D).
#
# Migrated VERBATIM from scripts/hooks/pre-push:41-122 lines (the
# credential-prefix scan body). The scan logic is a P0 security gate;
# any logic change without explicit threat-model review is silent
# downgrade per quality/PROTOCOL.md failure modes.
#
# Stdin: push-ref-range lines (matches the existing pre-push hook stdin
# contract). Empty stdin = no ranges to scan = exit 0 (correct).
# Non-empty stdin = iterate ranges, grep for credential prefixes.
#
# Exits 0 = PASS (no hits), 1 = FAIL (any credential prefix matched).
#
# Honors --row-id <id> (defaults to structure/cred-hygiene).

set -euo pipefail

readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
readonly ARTIFACT_DIR="${REPO_ROOT}/quality/reports/verifications/structure"
readonly ARTIFACT="${ARTIFACT_DIR}/cred-hygiene.json"

row_id="structure/cred-hygiene"
if [[ "${1:-}" == "--row-id" && -n "${2:-}" ]]; then
  row_id="$2"
fi

cd "$REPO_ROOT"
mkdir -p "$ARTIFACT_DIR"

# VERBATIM PATTERNS + EXCLUDE_DIRS from scripts/hooks/pre-push:41-57.
# Google API key pattern added 2026-07-04 after a secret-scanning alert
# (#1) flagged an AIza-shaped string in a bootstrap-seed .playwright-mcp
# log. The owner triaged that alert as a FALSE POSITIVE (no real key, no
# rotation, no history purge) — but the coverage gap it exposed was real:
# this committed gate greps only these prefixes, so a Google `AIza...` key
# would have been invisible to it and caught only server-side. ADDING a
# pattern is a security UPGRADE (broader coverage), not the
# threat-model-review downgrade the header warns against.
#
# AWS/Slack/OpenAI/PEM patterns added 2026-07-04 (D-CONV-4,
# quality/SURPRISES.md "Quality Convergence"): the gate's prior five
# prefixes were the known blind spots. These four are the highest-value
# additions, matched by the gitleaks CI backstop too (Layer 2) but worth
# catching in the zero-dep local layer so a fresh clone with hooks
# installed is protected without the gitleaks binary. Adding patterns is a
# security UPGRADE.
#
# OpenAI note: the "obvious" pattern `sk-[A-Za-z0-9_-]{20,}` collides with
# kebab-case doc slugs — `.planning/.../task-4-summary-changelog` yields a
# 19-char `sk-…` run (ta[sk-4-summary-changelog]), one char shy of a false
# hit, and a slightly longer slug WOULD trip it. Real OpenAI keys have a
# pure-alphanumeric body run (legacy `sk-`+48 alnum; project `sk-proj-`+
# alnum) — so we match an alnum RUN, which kebab-case (separator-broken)
# can never produce at length 20+. This keeps detection while dropping the
# slug-collision vector (verified: zero matches across the current tree).
readonly PATTERNS=(
  'ATATT3[A-Za-z0-9_+/=-]{20,}'
  'Bearer[[:space:]]+ATATT3[A-Za-z0-9_+/=-]{20,}'
  'ghp_[A-Za-z0-9]{20,}'
  'github_pat_[A-Za-z0-9_]{20,}'
  'AIza[0-9A-Za-z_-]{35}'
  'AKIA[0-9A-Z]{16}'
  'xox[baprs]-[0-9A-Za-z-]{10,}'
  'sk-(proj-)?[A-Za-z0-9]{20,}'
  '-----BEGIN( RSA| EC| OPENSSH)? PRIVATE KEY-----'
)
readonly EXCLUDE_DIRS=(
  '.git'
  'target'
  'node_modules'
  '.githooks'
  'quality/gates/structure'
)

# Read stdin ref ranges (push-ref-line format from git).
scan_refs=()
while read -r local_ref local_sha remote_ref remote_sha; do
  if [[ -z "${local_sha:-}" ]]; then continue; fi
  if [[ "$local_sha" == "0000000000000000000000000000000000000000" ]]; then continue; fi
  if [[ "$remote_sha" == "0000000000000000000000000000000000000000" ]]; then
    base=$(git rev-list --max-parents=0 "$local_sha" 2>/dev/null | head -1)
    if [[ -n "$base" ]]; then
      scan_refs+=("${base}..${local_sha}")
    else
      scan_refs+=("$local_sha")
    fi
  else
    scan_refs+=("${remote_sha}..${local_sha}")
  fi
done

ts=$(date -u +%Y-%m-%dT%H:%M:%SZ)
hit=0
hits_log=""

if [ "${#scan_refs[@]}" -gt 0 ]; then
  for range in "${scan_refs[@]}"; do
    files=$(git diff --name-only "$range" 2>/dev/null || true)
    for file in $files; do
      skip=0
      for excl in "${EXCLUDE_DIRS[@]}"; do
        if [[ "$file" == "$excl"* ]]; then skip=1; break; fi
      done
      [[ "$skip" -eq 1 ]] && continue
      tip_sha="${range##*..}"
      if ! git cat-file -e "${tip_sha}:${file}" 2>/dev/null; then continue; fi
      content=$(git show "${tip_sha}:${file}" 2>/dev/null || true)
      for pattern in "${PATTERNS[@]}"; do
        if printf '%s' "$content" | grep -qE "$pattern"; then
          printf 'credential-prefix match in %s: pattern %s\n' "$file" "$pattern" >&2
          hit=$((hit + 1))
          hits_log="${hits_log}${file}:${pattern} "
        fi
      done
    done
  done
fi

# Build artifact JSON.
exit_code=0
if [ "$hit" -gt 0 ]; then
  exit_code=1
  asserts_passed='[]'
  asserts_failed='["'"$hit"' credential-prefix match(es) on outgoing ref-range"]'
else
  asserts_passed='["no credential-prefix matches in '"${#scan_refs[@]}"' ref range(s) -- patterns set: '"${#PATTERNS[@]}"'"]'
  asserts_failed='[]'
fi

cat > "$ARTIFACT" <<EOF
{
  "ts": "$ts",
  "row_id": "$row_id",
  "exit_code": $exit_code,
  "ranges_scanned": ${#scan_refs[@]},
  "patterns_count": ${#PATTERNS[@]},
  "hits": $hit,
  "asserts_passed": $asserts_passed,
  "asserts_failed": $asserts_failed
}
EOF

exit "$exit_code"
