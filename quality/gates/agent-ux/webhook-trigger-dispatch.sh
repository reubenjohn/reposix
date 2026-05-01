#!/usr/bin/env bash
# quality/gates/agent-ux/webhook-trigger-dispatch.sh — agent-ux
# verifier for catalog row `agent-ux/webhook-trigger-dispatch`.
#
# CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/webhook-trigger-dispatch
# CADENCE:     pre-pr (~15s wall time)
# INVARIANT:   Workflow YAML exists at BOTH:
#              (a) docs/guides/dvcs-mirror-setup-template.yml (canonical repo, this checkout)
#              (b) reubenjohn/reposix-tokenworld-mirror:.github/workflows/reposix-mirror-sync.yml (live; via gh api)
#              The two are byte-equal modulo whitespace (diff -w returns zero).
#              YAML structure: repository_dispatch types=[reposix-mirror-sync] AND
#                              cargo binstall reposix-cli (NOT bare 'reposix') AND
#                              NO github.event.client_payload references.
#
# Status until P84-01 T06: FAIL.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

TEMPLATE="docs/guides/dvcs-mirror-setup-template.yml"
MIRROR_REPO="reubenjohn/reposix-tokenworld-mirror"
LIVE_PATH=".github/workflows/reposix-mirror-sync.yml"

# 1. Template copy must exist + parse as YAML.
test -f "$TEMPLATE" || { echo "FAIL: $TEMPLATE missing"; exit 1; }
python3 -c "import yaml,sys; yaml.safe_load(open(sys.argv[1]))" "$TEMPLATE" \
  || { echo "FAIL: $TEMPLATE does not parse as YAML"; exit 1; }

# 2. Live copy must exist via gh api.
command -v gh >/dev/null \
  || { echo "FAIL: gh not on PATH"; exit 1; }
LIVE_CONTENT=$(mktemp)
trap 'rm -f "$LIVE_CONTENT"' EXIT
gh api "repos/${MIRROR_REPO}/contents/${LIVE_PATH}" -q .content 2>/dev/null \
  | base64 -d > "$LIVE_CONTENT" \
  || { echo "FAIL: live copy ${MIRROR_REPO}:${LIVE_PATH} unreachable"; exit 1; }
test -s "$LIVE_CONTENT" \
  || { echo "FAIL: live copy is empty"; exit 1; }

# 3. Two copies are byte-equal modulo whitespace.
diff -w "$TEMPLATE" "$LIVE_CONTENT" >/dev/null \
  || { echo "FAIL: template and live copies differ (modulo whitespace)"; \
       diff -w "$TEMPLATE" "$LIVE_CONTENT" | head -20; exit 1; }

# 4. Structural greps on the YAML.
grep -q "repository_dispatch:" "$TEMPLATE" \
  || { echo "FAIL: missing repository_dispatch trigger"; exit 1; }
grep -qE "types:\s*\[\s*reposix-mirror-sync\s*\]" "$TEMPLATE" \
  || { echo "FAIL: missing types: [reposix-mirror-sync]"; exit 1; }
# Match `cargo binstall [...flags...] reposix-cli` — flags like
# `--no-confirm` are commonly interleaved between `binstall` and the
# crate name, so allow any whitespace-delimited tokens between them.
grep -qE "cargo binstall( +[^[:space:]]+)* +reposix-cli\b" "$TEMPLATE" \
  || { echo "FAIL: missing 'cargo binstall ... reposix-cli' (D-05; must NOT be bare 'reposix')"; exit 1; }
# Ensure no bare `cargo binstall ... reposix` (without -cli suffix) on
# any line (with or without intervening flags).
if grep -nE "cargo binstall( +[^[:space:]]+)* +reposix($|[^-[:alnum:]])" "$TEMPLATE" \
     | grep -v "reposix-cli" >/dev/null 2>&1; then
  echo "FAIL: bare 'cargo binstall reposix' detected (use reposix-cli per D-05)"
  exit 1
fi
if grep -q "github.event.client_payload" "$TEMPLATE"; then
  echo "FAIL: github.event.client_payload reference detected (S2/T-84-01 violation)"
  exit 1
fi
if grep -q "set -x" "$TEMPLATE"; then
  echo "FAIL: 'set -x' detected (T-84-02 secret-leak risk)"
  exit 1
fi

echo "PASS: workflow YAML present in both copies, byte-equal mod whitespace, structural invariants hold"
exit 0
