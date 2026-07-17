#!/usr/bin/env bash
# quality/gates/structure/social-freshness.selftest.sh
#
# Self-test for social-freshness.sh. Builds a throwaway git repo under /tmp
# (never the shared repo — leaf isolation), seeds fixture docs/social/*.md
# files, runs the gate against it, and asserts:
#   (a) a clean fixture (no dead terms) → exit 0 (PASS path)
#   (b) a planted `FUSE` line → exit 1, names file:line (BLOCK path)
#   (c) a planted `/mnt/` line → exit 1 (BLOCK path, second term)
#   (d) a planted bare `mount` line → exit 1 (BLOCK path, third term)
#   (e) an "amount"/"paramount" line → exit 0 (word-boundary anchor holds, T-117-08)
#   (f) a planted `FUSE` line WITH the allowlist marker → exit 0 (escape hatch works)
#
# Run: bash quality/gates/structure/social-freshness.selftest.sh
# Exit 0 = all assertions pass; exit 1 = a regression.
set -euo pipefail

GATE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/social-freshness.sh"
[[ -x "$GATE" ]] || { echo "FATAL: gate not found/executable at $GATE" >&2; exit 1; }

WORK="$(mktemp -d "${TMPDIR:-/tmp}/social-freshness-selftest.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT

# Throwaway repo — non-fixture identity so the leaf-isolation guard is a no-op.
git -C "$WORK" init -q
git -C "$WORK" config core.hooksPath /dev/null
git -C "$WORK" config user.name "selftest"
git -C "$WORK" config user.email "selftest@example.invalid"
mkdir -p "$WORK/docs/social"

run() { ( cd "$WORK" && "$GATE" ) 2>"$WORK/.err"; }  # returns gate exit; stderr→.err

pass=0; fail=0
check() { # check <label> <actual> <expected>
  if [[ "$2" == "$3" ]]; then echo "  PASS: $1 (got $2)"; pass=$((pass+1))
  else echo "  FAIL: $1 (expected $3, got $2)"; fail=$((fail+1)); fi
}

echo "== Case (a): clean fixture -> exit 0 =="
cat > "$WORK/docs/social/twitter.md" <<'EOF'
# Twitter draft

reposix exposes REST-backed systems of record as a git-native partial clone.
No mounting, no daemons — just `git checkout` and `cat`.
EOF
set +e; run; rc=$?; set -e
check "clean fixture exit code" "$rc" 0

echo "== Case (b): planted FUSE line -> exit 1, names file:line =="
cat >> "$WORK/docs/social/twitter.md" <<'EOF'

Under the hood it's really a FUSE filesystem.
EOF
set +e; run; rc=$?; set -e
check "FUSE-planted exit code" "$rc" 1
grep -q "docs/social/twitter.md:.*FUSE" "$WORK/.err" && f=yes || f=no
check "FUSE offender names file:line" "$f" "yes"
git -C "$WORK" checkout -q -- docs/social/twitter.md 2>/dev/null || true
printf '%s\n' "# Twitter draft" > "$WORK/docs/social/twitter.md"
printf '%s\n' "reposix exposes REST-backed systems of record as a git-native partial clone." >> "$WORK/docs/social/twitter.md"

echo "== Case (c): planted /mnt/ path -> exit 1 =="
printf '%s\n' 'Old docs said reach it under /mnt/reposix/ — no longer true.' >> "$WORK/docs/social/twitter.md"
set +e; run; rc=$?; set -e
check "/mnt/-planted exit code" "$rc" 1
printf '%s\n' "# Twitter draft" > "$WORK/docs/social/twitter.md"
printf '%s\n' "reposix exposes REST-backed systems of record as a git-native partial clone." >> "$WORK/docs/social/twitter.md"

echo "== Case (d): planted bare 'mount' word -> exit 1 =="
printf '%s\n' 'You used to mount the repo before reading it.' >> "$WORK/docs/social/twitter.md"
set +e; run; rc=$?; set -e
check "bare-mount-planted exit code" "$rc" 1
printf '%s\n' "# Twitter draft" > "$WORK/docs/social/twitter.md"
printf '%s\n' "reposix exposes REST-backed systems of record as a git-native partial clone." >> "$WORK/docs/social/twitter.md"

echo "== Case (e): 'amount'/'paramount' -> exit 0 (word-boundary holds, T-117-08) =="
printf '%s\n' 'A paramount concern: keep the token amount low.' >> "$WORK/docs/social/twitter.md"
set +e; run; rc=$?; set -e
check "amount/paramount exit code" "$rc" 0
printf '%s\n' "# Twitter draft" > "$WORK/docs/social/twitter.md"
printf '%s\n' "reposix exposes REST-backed systems of record as a git-native partial clone." >> "$WORK/docs/social/twitter.md"

echo "== Case (f): planted FUSE line WITH allowlist marker -> exit 0 =="
printf '%s\n' 'v0.4 shipped a FUSE mount at launch. <!-- social-freshness: ok — historical recap -->' >> "$WORK/docs/social/twitter.md"
set +e; run; rc=$?; set -e
check "allowlist-marker exit code" "$rc" 0

echo
echo "RESULT: $pass passed, $fail failed"
[[ $fail -eq 0 ]] || exit 1
