#!/usr/bin/env bash
# scripts/demos/_record.sh — record a Tier 1 demo as a typescript.
#
# Usage:
#   bash scripts/demos/_record.sh <demo-script> <output-basename>
#
# Produces:
#   docs/demos/recordings/<basename>.typescript
#   docs/demos/recordings/<basename>.transcript.txt
#
# The .typescript is the raw `script(1)` output (ANSI escapes intact).
# The .transcript.txt strips escape sequences for GitHub rendering.
#
# Prereqs: release binaries on PATH. Callers should do:
#   cargo build --release --workspace --bins
#   PATH="$PWD/target/release:$PATH" bash scripts/demos/_record.sh <demo> <name>

set -euo pipefail

if [[ $# -lt 2 ]]; then
    echo "Usage: $0 <demo-script> <output-basename>" >&2
    exit 2
fi

DEMO="$1"
BASENAME="$2"
if [[ ! -f "$DEMO" ]]; then
    echo "ERROR: demo not found: $DEMO" >&2
    exit 2
fi

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="${REPO_ROOT}/docs/demos/recordings"
mkdir -p "$OUT_DIR"
TS="${OUT_DIR}/${BASENAME}.typescript"
TXT="${OUT_DIR}/${BASENAME}.transcript.txt"

echo "== recording $DEMO -> $TS"
# -q: quiet start/end banner. -c: command to run under the pty.
script -q -c "bash '$DEMO'" "$TS"

# Strip ANSI escape sequences for the github-rendered transcript.
# Reference: perl regex matches CSI (ESC [ ... cmd) and OSC (ESC ] ... BEL).
perl -pe 's/\e\[[0-9;?]*[a-zA-Z]//g; s/\e\][^\a]*\a//g' "$TS" > "$TXT"
# Drop `script` start/end banners from the transcript.
sed -i '/^Script started on/d; /^Script done on/d; /^Script started,/d; /^Script done,/d' "$TXT"

# Report sizes.
ts_size=$(wc -c < "$TS")
ts_lines=$(wc -l < "$TS")
txt_size=$(wc -c < "$TXT")
txt_lines=$(wc -l < "$TXT")
echo "   typescript:  ${ts_size} bytes, ${ts_lines} lines"
echo "   transcript:  ${txt_size} bytes, ${txt_lines} lines"
