#!/usr/bin/env bash
# quality/gates/structure/social-freshness.sh — P117-06 fix-twice backstop.
#
# docs/social/* is `not_in_nav` (mkdocs.yml) so mkdocs-strict / link-resolution /
# doc-alignment / banned-words NEVER scan it — that gap is exactly why a stale
# current-tense FUSE line survived to launch in the SC5b bug this phase purged
# (PATTERNS.md NOTICED #3). This is the minimal grep-based backstop: scan every
# docs/social/**/*.md for known-dead architecture terms and BLOCK if any survive.
#
# Terms (case-insensitive, word-boundary anchored per T-117-08 to avoid false
# hits on "amount"/"paramount"):
#   \bFUSE\b   -- the pre-v0.9.0 FUSE-mount architecture, retired at the
#                 git-native pivot (docs/research/architecture-pivot-summary).
#   /mnt/      -- FUSE mount-path form.
#   \bmount\b  -- generic "mount" vocabulary tied to the retired FUSE model.
#
# Allowlist marker (mirrors docs/.banned-words.toml's own `<!-- banned-words:
# ok -->` convention): a line containing `social-freshness: ok` inside an
# HTML comment (reason text may follow inline, e.g. `<!-- social-freshness:
# ok — reason -->`) is skipped. This is for LEGITIMATE past-tense history
# (e.g. "v0.4 shipped a
# FUSE mount" — the same framing docs/development/roadmap.md and the ADRs use
# unrestricted, since Reference/Research is an unrestricted layer) — the bug
# class this gate closes is a CURRENT-tense claim, not the word's mere
# presence. Use sparingly; each marker should be paired with a reason in the
# same commit.
#
# Catalog row: quality/catalogs/freshness-invariants.json structure/social-freshness.
set -euo pipefail
# Resolve relative to the CALLER's cwd (git-toplevel), not the script's own
# location — this is what lets a selftest `cd` into a throwaway /tmp repo and
# invoke this gate against THAT tree (precedent: file-size-limits.sh).
cd "$(git rev-parse --show-toplevel)"

shopt -s nullglob globstar
FILES=(docs/social/**/*.md)

if [[ ${#FILES[@]} -eq 0 ]]; then
  echo "PASS: no docs/social/**/*.md files to scan"
  exit 0
fi

# Substring (not exact-string) match on purpose: the marker's reason text
# commonly lives inline within the same HTML comment (e.g. "<!-- social-
# freshness: ok — reason -->"), so anchoring on the "social-freshness: ok"
# prefix catches every variant without forcing a bare, reason-less marker.
ALLOWLIST_MARKER='social-freshness: ok'
OFFENDERS=$(grep -HinE '\bFUSE\b|/mnt/|\bmount\b' "${FILES[@]}" 2>/dev/null | grep -Fv -- "${ALLOWLIST_MARKER}" || true)

if [[ -n "${OFFENDERS}" ]]; then
  echo "FAIL: docs/social/**/*.md contains a known-dead architecture term:" >&2
  echo "${OFFENDERS}" >&2
  echo "owner_hint: docs/social/* is not_in_nav (mkdocs.yml) so no other docs gate scans it — reword to the git-native partial-clone model (no FUSE, no mount path). If this is deliberate PAST-tense history (matching docs/development/roadmap.md's framing), append '<!-- ${ALLOWLIST_MARKER} — <reason> -->' to the line." >&2
  exit 1
fi

echo "PASS: no docs/social/**/*.md contains a known-dead architecture term (FUSE, /mnt/, mount)"
exit 0
