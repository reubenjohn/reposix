#!/usr/bin/env bash
# scripts/demos/assert.sh — run a demo, capture output, grep ASSERTS markers.
#
# Usage:
#   bash scripts/demos/assert.sh scripts/demos/01-edit-and-push.sh
#
# The target demo script must have a header comment of the form:
#   # ASSERTS: "marker one" "marker two" "DEMO COMPLETE"
#
# Each marker is searched literally (grep -F) against the captured
# stdout+stderr. Exit 0 iff every marker matches; exit 1 otherwise.
# This turns a demo into a self-asserting integration test: the demo
# says what success looks like, and assert.sh enforces it.

set -euo pipefail

if [[ $# -lt 1 ]]; then
    echo "Usage: $0 <demo-script> [args...]" >&2
    exit 2
fi

DEMO="$1"
shift
if [[ ! -f "$DEMO" ]]; then
    echo "ERROR: demo script not found: $DEMO" >&2
    exit 2
fi

# Extract the ASSERTS line from the header comment.
ASSERTS_LINE=$(grep -m1 -E '^# ASSERTS:' "$DEMO" || true)
if [[ -z "$ASSERTS_LINE" ]]; then
    echo "ERROR: $DEMO has no '# ASSERTS:' header comment" >&2
    exit 2
fi

# Strip the "# ASSERTS:" prefix. The remainder is a list of space-
# separated double-quoted strings. Parse into a bash array.
ASSERTS_RAW="${ASSERTS_LINE#\# ASSERTS:}"
# shellcheck disable=SC2206
eval "MARKERS=($ASSERTS_RAW)"
if [[ ${#MARKERS[@]} -eq 0 ]]; then
    echo "ERROR: ASSERTS header in $DEMO parsed to zero markers" >&2
    exit 2
fi

echo "== assert.sh: running $DEMO"
echo "   markers to assert: ${#MARKERS[@]}"
for m in "${MARKERS[@]}"; do
    echo "     - $m"
done

# Capture combined stdout+stderr so ASSERTS can match log lines that
# the demo emits on either stream. Preserve the exit code so we can
# report it alongside the grep results.
TMP_OUT=$(mktemp)
trap 'rm -f "$TMP_OUT"' EXIT

set +e
bash "$DEMO" "$@" >"$TMP_OUT" 2>&1
DEMO_RC=$?
set -e

echo
echo "== assert.sh: demo exited with rc=$DEMO_RC"

# Grep each marker. Use grep -Fi (literal + case-insensitive) so markers
# like "reduction" match "**Reduction:**" and "status: in_progress"
# doesn't need regex escaping.
FAILED=0
for m in "${MARKERS[@]}"; do
    if grep -Fiq -- "$m" "$TMP_OUT"; then
        echo "   OK   marker matched: $m"
    else
        echo "   FAIL marker missing: $m" >&2
        FAILED=$((FAILED + 1))
    fi
done

if [[ $DEMO_RC -ne 0 ]]; then
    echo "   FAIL demo script exited non-zero ($DEMO_RC)" >&2
    FAILED=$((FAILED + 1))
fi

if [[ $FAILED -gt 0 ]]; then
    echo
    echo "== assert.sh: FAIL ($FAILED marker(s) missing or demo exited non-zero)"
    echo "   last 40 lines of demo output:"
    tail -40 "$TMP_OUT" | sed 's/^/     /'
    exit 1
fi

echo
echo "== assert.sh: PASS ($(basename "$DEMO"))"
exit 0
