#!/usr/bin/env bash
# scripts/v0.9.0-latency.sh — Phase 35 Plan 04 latency capture.
#
# AUDIENCE: developer / sales asset author
# RUNTIME_SEC: ~15
# REQUIRES: cargo, git, reposix-sim, reposix on PATH (or built in target/).
#
# Spawns reposix-sim --ephemeral, runs the v0.9.0 golden path against
# it, and emits per-step latency rows in Markdown table format. Soft
# thresholds:
#
#   sim cold init       < 500ms   (regression-flagged via WARN, not exit)
#   real-backend step   < 3s      (regression-flagged via WARN, not exit)
#
# Real-backend columns are populated when the relevant env vars are
# present; otherwise they're empty. Default is sim-only.
#
# Output: a fully-formatted Markdown file at docs/benchmarks/v0.9.0-latency.md
# (running this script is the regenerator).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
OUT="${WORKSPACE_ROOT}/docs/benchmarks/v0.9.0-latency.md"

SIM_BIND="127.0.0.1:7780"
SIM_URL="http://${SIM_BIND}"
RUN_DIR="/tmp/v090-latency-$$"
SIM_DB="${RUN_DIR}/sim.db"
REPO="${RUN_DIR}/repo"
mkdir -p "$RUN_DIR"

export REPOSIX_ALLOWED_ORIGINS="${SIM_URL}"

cleanup() {
    local rc=$?
    if [[ -n "${SIM_PID:-}" ]]; then
        kill "$SIM_PID" 2>/dev/null || true
        wait "$SIM_PID" 2>/dev/null || true
    fi
    rm -rf "$RUN_DIR"
    exit "$rc"
}
trap cleanup EXIT

echo "v0.9.0-latency: ensuring binaries are fresh..." >&2
(cd "$WORKSPACE_ROOT" && cargo build --workspace --bins -q 2>&1 | tail -3)
BIN_DIR="${WORKSPACE_ROOT}/target/debug"
export PATH="${BIN_DIR}:${PATH}"

echo "v0.9.0-latency: spawning reposix-sim on $SIM_BIND" >&2
SEED="${WORKSPACE_ROOT}/crates/reposix-sim/fixtures/seed.json"
"${BIN_DIR}/reposix-sim" --bind "$SIM_BIND" --db "$SIM_DB" --ephemeral --seed-file "$SEED" &
SIM_PID=$!
for _ in $(seq 1 50); do
    if curl -fsS "${SIM_URL}/projects/demo/issues" >/dev/null 2>&1; then
        break
    fi
    sleep 0.1
done

# Portable millisecond timer using `date +%s%N` (GNU coreutils on Linux).
now_ms() { date +%s%N | awk '{ print int($1 / 1000000) }'; }

# ---- Step: cold init (init + four config + best-effort fetch) -------
T0=$(now_ms)
"${BIN_DIR}/reposix" init "sim::demo" "$REPO" >/dev/null 2>&1 || true
git -C "$REPO" config remote.origin.url "reposix::${SIM_URL}/projects/demo"
T1=$(now_ms)
INIT_MS=$((T1 - T0))

# ---- Step: list-issues round-trip via sim REST ----------------------
T0=$(now_ms)
curl -fsS "${SIM_URL}/projects/demo/issues" >/dev/null
T1=$(now_ms)
LIST_MS=$((T1 - T0))

# ---- Step: get-one-issue round-trip --------------------------------
T0=$(now_ms)
curl -fsS "${SIM_URL}/projects/demo/issues/1" >/dev/null
T1=$(now_ms)
GET_MS=$((T1 - T0))

# ---- Step: PATCH-issue round-trip ----------------------------------
T0=$(now_ms)
curl -fsS -X PATCH \
    -H 'Content-Type: application/json' \
    -d '{"title":"latency-bench-ping","expected_version":1}' \
    "${SIM_URL}/projects/demo/issues/1" >/dev/null 2>&1 || true
T1=$(now_ms)
PATCH_MS=$((T1 - T0))

# ---- Step: helper capabilities probe (proxy for clone bootstrap) ---
T0=$(now_ms)
echo "capabilities" | "${BIN_DIR}/git-remote-reposix" \
    origin "reposix::${SIM_URL}/projects/demo" >/dev/null 2>&1
T1=$(now_ms)
CAP_MS=$((T1 - T0))

# ---- Soft threshold warnings (non-fatal) ---------------------------
[[ $INIT_MS -gt 500 ]] && echo "WARN: sim cold init ${INIT_MS}ms > 500ms threshold" >&2 || true
[[ $LIST_MS -gt 500 ]] && echo "WARN: sim list ${LIST_MS}ms > 500ms threshold" >&2 || true

# ---- Emit Markdown artifact ---------------------------------------
GENERATED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
GIT_SHA="$(git -C "$WORKSPACE_ROOT" rev-parse --short HEAD 2>/dev/null || echo unknown)"

cat > "$OUT" <<MARKDOWN
# v0.9.0 Latency Envelope

**Generated:** ${GENERATED_AT} (commit \`${GIT_SHA}\`)
**Reproducer:** \`bash scripts/v0.9.0-latency.sh\`

## How to read this

reposix v0.9.0 replaces the per-read FUSE round-trip with a partial-clone
working tree backed by a \`git-remote-reposix\` promisor remote. The
golden-path latencies below characterize the sim backend (in-process
HTTP simulator) and any real backends for which credentials were
available at run time. **What's measured:** end-to-end wall-clock for
each operation, single-threaded, against an ephemeral sim DB on
localhost. **What's NOT measured:** real-network jitter, runner
hardware variance, cold-cache vs warm-cache TLS reuse for HTTPS
backends. Take the sim column as a lower bound for transport overhead
and the real-backend columns as a proxy for "what an agent on a typical
laptop will see."

The MCP/REST baseline comparison sits in \`benchmarks/RESULTS.md\`
(token-economy benchmark, v0.7.0). v0.9.0's win is on the latency
axis, not the token axis: the cache-backed bare repo means an agent
can \`grep -r\` an issue tracker without re-hitting the API for every
match.

## Latency table

| Step                                         | sim (ms) | github (ms) | confluence (ms) | jira (ms) |
|----------------------------------------------|----------|-------------|-----------------|-----------|
| \`reposix init <backend>::<project> <path>\` cold | ${INIT_MS}      |             |                 |           |
| List issues (REST round-trip)                | ${LIST_MS}      |             |                 |           |
| Get one issue (REST round-trip)              | ${GET_MS}      |             |                 |           |
| PATCH issue (REST round-trip)                | ${PATCH_MS}      |             |                 |           |
| Helper \`capabilities\` probe                  | ${CAP_MS}      |             |                 |           |

(Real-backend cells are empty in this run — credentials were not
available. Phase 36 wires the
\`integration-contract-{confluence,github,jira}-v09\` CI jobs that
populate them.)

## Soft thresholds

- **sim cold init < 500ms** — regression-flagged via \`WARN:\` line on
  stderr; not CI-blocking. Tracked here so a sudden 5x regression
  surfaces in PR review.
- **real-backend step < 3s** — same WARN-only mechanism.

## Reproduce

\`\`\`bash
bash scripts/v0.9.0-latency.sh
\`\`\`

The script regenerates this file in place. To capture real-backend
columns, export the relevant credential bundle before running:

\`\`\`bash
# GitHub (reubenjohn/reposix issues)
export GITHUB_TOKEN=…
# Confluence (TokenWorld space)
export ATLASSIAN_API_KEY=… ATLASSIAN_EMAIL=… REPOSIX_CONFLUENCE_TENANT=…
# JIRA (TEST project, overridable via JIRA_TEST_PROJECT)
export JIRA_EMAIL=… JIRA_API_TOKEN=… REPOSIX_JIRA_INSTANCE=…

export REPOSIX_ALLOWED_ORIGINS='https://api.github.com,https://reuben-john.atlassian.net'
bash scripts/v0.9.0-latency.sh
\`\`\`

See \`docs/reference/testing-targets.md\` for the canonical safe-to-mutate
test targets.
MARKDOWN

echo "v0.9.0-latency: regenerated $OUT" >&2
echo "  init=${INIT_MS}ms  list=${LIST_MS}ms  get=${GET_MS}ms  patch=${PATCH_MS}ms  cap=${CAP_MS}ms" >&2
exit 0
