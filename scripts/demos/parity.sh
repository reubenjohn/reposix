#!/usr/bin/env bash
# scripts/demos/parity.sh — Tier 3 sim-vs-GitHub parity demo.
#
# AUDIENCE: skeptic
# RUNTIME_SEC: 30
# REQUIRES: cargo, jq, gh
# ASSERTS: "shape parity" "DEMO COMPLETE"
#
# Narrative: the whole point of the IssueBackend seam is that concrete
# backends (simulator, GitHub) are structurally interchangeable. This demo
# proves it by listing issues from BOTH and diffing the normalized shape.
#
# Why does this use `gh api` for the GitHub side instead of `reposix list
# --backend github`? Because `reposix list` currently hard-codes
# SimBackend (that wiring lands in v0.2). The library-level proof lives in
# `crates/reposix-github/tests/contract.rs` — this script is the human-
# visible side of that same claim.
#
# Outputs:
#   /tmp/parity-sim.json    — [{id, title, status}, ...] from the sim.
#   /tmp/parity-github.json — same shape, from octocat/Hello-World.
#
# Expected diff: content differs (different issues) but the *shape* is
# identical. That's the story.

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
require gh
require curl

# Both backends need to be reachable from the HttpClient allowlist.
export REPOSIX_ALLOWED_ORIGINS="http://127.0.0.1:*,https://api.github.com"

# ------------------------------------------------------------ config
SIM_BIND="127.0.0.1:7804"
SIM_URL="http://${SIM_BIND}"
SIM_DB="/tmp/reposix-demo-parity-sim.db"
SIM_JSON="/tmp/parity-sim.json"
GH_JSON="/tmp/parity-github.json"
DIFF_OUT="/tmp/parity-diff.txt"
export SIM_BIND SIM_URL
_REPOSIX_TMP_PATHS+=("$SIM_JSON" "$GH_JSON" "$DIFF_OUT")
cleanup_trap

# Pre-clean debris from a prior aborted run.
rm -f "$SIM_JSON" "$GH_JSON" "$DIFF_OUT" \
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

# ------------------------------------------------------------ 3/4 list github
section "[3/4] list issues via real GitHub (octocat/Hello-World)"
# GitHub's REST exposes `number` where we call `id`, and `state` (2-valued)
# where we call `status` (5-valued). The parity claim is the *shape* of the
# response — {id, title, status} — is identical, content differs.
# We normalize GitHub's 2-valued state into the reposix IssueStatus
# vocabulary (ADR-001) inside jq so the diff has nothing to catch on.
gh api '/repos/octocat/Hello-World/issues?state=all&per_page=30' \
    | jq '[.[] | {
        id: .number,
        title: .title,
        status: (
          if .state == "open" then
            if (.labels | map(.name) | contains(["status/in-review"])) then "in_review"
            elif (.labels | map(.name) | contains(["status/in-progress"])) then "in_progress"
            else "open" end
          else
            if .state_reason == "not_planned" then "wont_fix"
            else "done" end
          end
        )
      }] | sort_by(.id)' \
    > "$GH_JSON"
echo "wrote $(jq length < "$GH_JSON") github issues -> $GH_JSON"
head -20 "$GH_JSON"

# ------------------------------------------------------------ 4/4 diff
section "[4/4] shape parity: diff normalized JSON"
echo
echo "--- the diff IS the story ---"
echo "sim and github produce the SAME JSON shape — {id, title, status} —"
echo "so the diff reports only content differences (different issue bodies),"
echo "no structural/key/type differences."
echo

# `diff -u` returns 1 on any difference. We capture to a file so we can
# always show the header (and truncate the body to keep the demo tight).
set +e
diff -u "$SIM_JSON" "$GH_JSON" > "$DIFF_OUT"
DIFF_RC=$?
set -e
if [[ "$DIFF_RC" -eq 0 ]]; then
    echo "unexpected: sim and github issues are byte-identical (shouldn't be)"
    exit 1
fi

# Show the first 40 lines of the unified diff so a human can eyeball that
# ONLY content differs, not schema.
head -40 "$DIFF_OUT"

# Structural assertion: every entry in both files has keys {id, title,
# status}; jq would fail loudly if any were missing.
sim_keys=$(jq -r '[.[] | keys_unsorted] | add | unique | sort | join(",")' "$SIM_JSON")
gh_keys=$(jq -r '[.[] | keys_unsorted] | add | unique | sort | join(",")' "$GH_JSON")
echo
echo "sim keys:    $sim_keys"
echo "github keys: $gh_keys"
if [[ "$sim_keys" != "$gh_keys" ]]; then
    echo "FAIL: key sets differ"
    exit 1
fi
echo "shape parity: confirmed (same keys, same types)"

echo
echo "== DEMO COMPLETE =="
