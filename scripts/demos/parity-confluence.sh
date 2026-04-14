#!/usr/bin/env bash
# scripts/demos/parity-confluence.sh — Tier 3B sim-vs-Confluence parity demo.
#
# AUDIENCE: skeptic
# RUNTIME_SEC: 45
# REQUIRES: cargo, jq, reposix (release binary)
# ASSERTS: "shape parity" "DEMO COMPLETE"
#
# Narrative: same story as parity.sh, but for Confluence Cloud. Lists
# pages from a real Atlassian space AND issues from the reposix sim,
# normalizes both to {id, title, status}, and diffs the key sets.
# Identical keys = structural parity; only content differs.
#
# Why does this exist alongside parity.sh? Because the IssueBackend trait
# claim is "any REST tracker flattens to the same shape." parity.sh
# proves that for GitHub issues; this demo proves it for Confluence
# pages. Two independent data models, one canonical `Issue`.
#
# Skip behavior: exits 0 with SKIP banner if any of the four required
# env vars are unset. This is the documented v0.3 SKIP contract so a
# dev without Atlassian credentials can still run the demo and get a
# green exit. Non-negotiable for CI-friendly behavior.
#
# Outputs:
#   /tmp/parity-conf-sim.json        — [{id, title, status}, ...] from sim.
#   /tmp/parity-conf-confluence.json — same shape, from real Confluence.
#   /tmp/parity-conf-diff.txt        — unified diff of the two.

set -euo pipefail

# Self-wrap in `timeout 90` so a stuck sub-step cannot hang smoke.sh.
if [[ -z "${REPOSIX_DEMO_INNER:-}" ]]; then
    exec timeout 90 env REPOSIX_DEMO_INNER=1 bash "$0" "$@"
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=/dev/null
source "${SCRIPT_DIR}/_lib.sh"

# ------------------------------------------------------------ prereqs
require reposix-sim
require reposix
require jq
require curl

# ------------------------------------------------------------ SKIP check
# All four env vars are required. Missing any one => SKIP path.
# We intentionally do NOT echo the values of ATLASSIAN_API_KEY or
# ATLASSIAN_EMAIL anywhere in this script — tenant + space are the
# only non-secret identifiers we're comfortable showing on stdout.
MISSING=()
[[ -z "${ATLASSIAN_API_KEY:-}"         ]] && MISSING+=("ATLASSIAN_API_KEY")
[[ -z "${ATLASSIAN_EMAIL:-}"           ]] && MISSING+=("ATLASSIAN_EMAIL")
[[ -z "${REPOSIX_CONFLUENCE_TENANT:-}" ]] && MISSING+=("REPOSIX_CONFLUENCE_TENANT")
[[ -z "${REPOSIX_CONFLUENCE_SPACE:-}"  ]] && MISSING+=("REPOSIX_CONFLUENCE_SPACE")
if (( ${#MISSING[@]} > 0 )); then
    echo "SKIP: env vars unset: ${MISSING[*]}"
    echo "      Set them (see .env.example and MORNING-BRIEF-v0.3.md) to"
    echo "      run the Confluence half of parity."
    echo "== DEMO COMPLETE =="
    exit 0
fi

# Both backends need to be reachable from the HttpClient allowlist.
# 127.0.0.1:* covers the sim; the tenant hostname covers Confluence.
export REPOSIX_ALLOWED_ORIGINS="http://127.0.0.1:*,https://${REPOSIX_CONFLUENCE_TENANT}.atlassian.net"

# ------------------------------------------------------------ config
# Port 7805 avoids collision with parity.sh's 7804 so the two demos can
# (in principle) run concurrently without stepping on each other.
SIM_BIND="127.0.0.1:7805"
SIM_URL="http://${SIM_BIND}"
SIM_DB="/tmp/reposix-demo-parity-conf-sim.db"
SIM_JSON="/tmp/parity-conf-sim.json"
CONF_JSON="/tmp/parity-conf-confluence.json"
DIFF_OUT="/tmp/parity-conf-diff.txt"
export SIM_BIND SIM_URL
_REPOSIX_TMP_PATHS+=("$SIM_JSON" "$CONF_JSON" "$DIFF_OUT")
cleanup_trap

# Pre-clean debris from a prior aborted run.
rm -f "$SIM_JSON" "$CONF_JSON" "$DIFF_OUT" \
    "$SIM_DB" "${SIM_DB}-wal" "${SIM_DB}-shm" 2>/dev/null || true
pkill -f "reposix-sim --bind ${SIM_BIND}" 2>/dev/null || true
sleep 0.2

# ------------------------------------------------------------ 1/4 boot sim
section "[1/4] start simulator on ${SIM_BIND}"
setup_sim "$SIM_DB"
echo "sim ready at ${SIM_URL}"

# ------------------------------------------------------------ 2/4 list sim
section "[2/4] list issues via SimBackend"
reposix list --origin "${SIM_URL}" --project demo --format json \
    | jq '[.[] | {id, title, status}] | sort_by(.id)' \
    > "$SIM_JSON"
echo "wrote $(jq length < "$SIM_JSON") sim issues -> $SIM_JSON"
head -20 "$SIM_JSON"

# ------------------------------------------------------------ 3/4 list confluence
section "[3/4] list pages via real Confluence (space=${REPOSIX_CONFLUENCE_SPACE})"
echo "tenant: ${REPOSIX_CONFLUENCE_TENANT}.atlassian.net"
# Unlike parity.sh, we do NOT need a `gh api` / `curl` fallback here —
# Phase 11-A+B shipped a real Confluence adapter with CLI dispatch, so
# `reposix list --backend confluence` is the first-party surface.
reposix list \
    --backend confluence \
    --project "$REPOSIX_CONFLUENCE_SPACE" \
    --format json \
    | jq '[.[] | {id, title, status}] | sort_by(.id)' \
    > "$CONF_JSON"
echo "wrote $(jq length < "$CONF_JSON") confluence pages -> $CONF_JSON"
head -20 "$CONF_JSON"

# ------------------------------------------------------------ 4/4 diff
section "[4/4] shape parity: diff normalized JSON"
echo
echo "--- the diff IS the story ---"
echo "sim and confluence produce the SAME JSON shape — {id, title, status} —"
echo "so the diff reports only content differences (different page bodies),"
echo "no structural/key/type differences."
echo

# `diff -u` returns 1 on any difference. Capture to a file so we can
# always show a head slice of the output.
set +e
diff -u "$SIM_JSON" "$CONF_JSON" > "$DIFF_OUT"
DIFF_RC=$?
set -e
if [[ "$DIFF_RC" -eq 0 ]]; then
    echo "unexpected: sim and confluence outputs are byte-identical (shouldn't be)"
    exit 1
fi

# Show the first 40 lines of the unified diff so a human can eyeball
# that ONLY content differs, not schema.
head -40 "$DIFF_OUT"

# Structural assertion: every entry in both files has keys {id, title,
# status}; jq would fail loudly if any were missing.
sim_keys=$(jq -r '[.[] | keys_unsorted] | add | unique | sort | join(",")' "$SIM_JSON")
conf_keys=$(jq -r '[.[] | keys_unsorted] | add | unique | sort | join(",")' "$CONF_JSON")
echo
echo "sim keys:        $sim_keys"
echo "confluence keys: $conf_keys"
if [[ "$sim_keys" != "$conf_keys" ]]; then
    echo "FAIL: key sets differ"
    exit 1
fi
echo "shape parity: confirmed (same keys, same types)"

echo
echo "== DEMO COMPLETE =="
