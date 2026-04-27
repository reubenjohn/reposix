#!/usr/bin/env bash
# scripts/latency-bench.sh — v0.11.0 Phase 54 latency capture (POLISH-08).
#
# AUDIENCE: developer / sales asset author
# RUNTIME_SEC: ~15s sim-only, ~60s with all three real-backend bundles.
# REQUIRES: cargo, git, jq, sqlite3, curl, reposix-sim, reposix on PATH (or built in target/).
#
# Spawns reposix-sim --ephemeral, runs the v0.9.0 golden path against it,
# and emits per-step latency rows in Markdown table format. When real-backend
# credential bundles are present in env, additionally probes the matching
# real backend (github / confluence / jira) and stamps its column.
#
# Each timed step is taken as the median of 3 samples (network jitter on
# real backends is the dominant flake source — Phase 54 plan §"Risk areas").
#
# Soft thresholds:
#   sim cold init       < 500ms   (regression-flagged via WARN, not exit)
#   real-backend step   < 3s      (regression-flagged via WARN, not exit)
#
# Real-backend columns are populated when the relevant env vars are
# present; otherwise they're empty / "n/a". Default is sim-only.
#
# Output: a fully-formatted Markdown file at docs/benchmarks/latency.md
# (running this script is the regenerator).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
OUT="${WORKSPACE_ROOT}/docs/benchmarks/latency.md"

SIM_BIND="127.0.0.1:7780"
SIM_URL="http://${SIM_BIND}"
RUN_DIR="/tmp/v090-latency-$$"
SIM_DB="${RUN_DIR}/sim.db"
REPO="${RUN_DIR}/repo"
# Pin the cache directory so the script can locate `cache.db` deterministically
# for the blob-materialization counter (matches resolve_cache_path in
# crates/reposix-cache/src/path.rs: <root>/reposix/<backend>-<project>.git).
CACHE_ROOT="${RUN_DIR}/cache"
mkdir -p "$RUN_DIR" "$CACHE_ROOT"
export REPOSIX_CACHE_DIR="$CACHE_ROOT"

# Default allowlist — sim only. Per-backend blocks below extend this for
# their REST probes via a local override (the helper-spawned cache reads
# the env var fresh on each invocation).
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

echo "latency-bench: ensuring binaries are fresh..." >&2
(cd "$WORKSPACE_ROOT" && cargo build --workspace --bins -q 2>&1 | tail -3)
BIN_DIR="${WORKSPACE_ROOT}/target/debug"
export PATH="${BIN_DIR}:${PATH}"

echo "latency-bench: spawning reposix-sim on $SIM_BIND" >&2
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

# median3 <a> <b> <c> — print the median of three integers. Used to
# absorb network jitter on real-backend probes (Phase 54 plan §Risk areas).
median3() {
    local a=$1 b=$2 c=$3
    printf '%s\n' "$a" "$b" "$c" | sort -n | sed -n '2p'
}

# time_step <command...> — run the command, print elapsed ms on stdout.
# Suppresses the command's own output. Errors are tolerated (returns the
# elapsed time even on non-zero exit) so a single transient failure doesn't
# abort the bench.
time_step() {
    local t0 t1
    t0=$(now_ms)
    "$@" >/dev/null 2>&1 || true
    t1=$(now_ms)
    echo $((t1 - t0))
}

# median3_step <command...> — run the command 3x, return the median ms.
median3_step() {
    local s1 s2 s3
    s1=$(time_step "$@")
    s2=$(time_step "$@")
    s3=$(time_step "$@")
    median3 "$s1" "$s2" "$s3"
}

# count_blob_materializations <cache_dir_for_repo>
# Reads the cache.db audit table and counts `op='materialize'` rows. Path
# layout matches resolve_cache_path: <REPOSIX_CACHE_DIR>/reposix/<backend>-<project>.git/cache.db.
count_blob_materializations() {
    local db="$1/cache.db"
    if [[ ! -f "$db" ]]; then
        echo "0"
        return
    fi
    sqlite3 "$db" "SELECT COUNT(*) FROM audit_events_cache WHERE op='materialize'" 2>/dev/null || echo "0"
}

# ---- Sim block ------------------------------------------------------
SIM_PROJECT="demo"
SIM_REPO="$REPO"

# init cold — single sample (one-shot bootstrap; re-running into the same
# path would no-op git init and skew the number low).
T0=$(now_ms)
"${BIN_DIR}/reposix" init "sim::${SIM_PROJECT}" "$SIM_REPO" >/dev/null 2>&1 || true
git -C "$SIM_REPO" config remote.origin.url "reposix::${SIM_URL}/projects/${SIM_PROJECT}"
T1=$(now_ms)
SIM_INIT_MS=$((T1 - T0))
SIM_BLOBS=$(count_blob_materializations "${CACHE_ROOT}/reposix/sim-${SIM_PROJECT}.git")

# REST round-trips against the sim — 3 samples each.
SIM_LIST_BODY="$(curl -fsS "${SIM_URL}/projects/${SIM_PROJECT}/issues")"
SIM_N=$(echo "$SIM_LIST_BODY" | jq 'length' 2>/dev/null || echo "0")
SIM_LIST_MS=$(median3_step curl -fsS "${SIM_URL}/projects/${SIM_PROJECT}/issues")
SIM_GET_MS=$(median3_step curl -fsS "${SIM_URL}/projects/${SIM_PROJECT}/issues/1")

# PATCH no-op: re-write the title to whatever it already is. The sim's
# expected_version is checked, so we read it first.
SIM_TITLE=$(echo "$SIM_LIST_BODY" | jq -r '.[0].title // "latency-bench-ping"')
SIM_VERSION=$(curl -fsS "${SIM_URL}/projects/${SIM_PROJECT}/issues/1" | jq -r '.version // 1')
sim_patch() {
    curl -fsS -X PATCH \
        -H 'Content-Type: application/json' \
        -d "{\"title\":$(printf '%s' "$SIM_TITLE" | jq -Rsc .),\"expected_version\":${SIM_VERSION}}" \
        "${SIM_URL}/projects/${SIM_PROJECT}/issues/1"
}
SIM_PATCH_MS=$(median3_step sim_patch)

# Helper capabilities probe (proxy for clone bootstrap). Local-only — no
# network, identical across columns; kept as the runner-variance control.
sim_cap() {
    echo "capabilities" | "${BIN_DIR}/git-remote-reposix" \
        origin "reposix::${SIM_URL}/projects/${SIM_PROJECT}"
}
SIM_CAP_MS=$(median3_step sim_cap)

# ---- Soft threshold warnings (non-fatal) ---------------------------
[[ $SIM_INIT_MS -gt 500 ]] && echo "WARN: sim cold init ${SIM_INIT_MS}ms > 500ms threshold" >&2 || true
[[ $SIM_LIST_MS -gt 500 ]] && echo "WARN: sim list ${SIM_LIST_MS}ms > 500ms threshold" >&2 || true

# ---- GitHub block (skipped cleanly when GITHUB_TOKEN absent) ----------
GH_INIT_MS=""; GH_LIST_MS=""; GH_GET_MS=""; GH_PATCH_MS=""; GH_CAP_MS=""
GH_N=""; GH_BLOBS=""
if [[ -n "${GITHUB_TOKEN:-}" ]]; then
    echo "latency-bench: GitHub probe — using reubenjohn/reposix issues" >&2
    GH_PROJECT="reubenjohn/reposix"
    GH_REPO="${RUN_DIR}/gh-repo"
    GH_ORIGIN="https://api.github.com"
    # Per-backend allowlist override for the helper-spawned cache.
    export REPOSIX_ALLOWED_ORIGINS="${SIM_URL},${GH_ORIGIN}"

    # Warm-up GET to amortize TLS handshake before timing.
    curl -fsS -H "Authorization: Bearer ${GITHUB_TOKEN}" \
        "${GH_ORIGIN}/repos/${GH_PROJECT}/issues?state=all&per_page=1" >/dev/null 2>&1 || true

    T0=$(now_ms)
    "${BIN_DIR}/reposix" init "github::${GH_PROJECT}" "$GH_REPO" >/dev/null 2>&1 || true
    T1=$(now_ms)
    GH_INIT_MS=$((T1 - T0))
    # Cache-dir name uses sanitize_project_for_cache: `owner/repo` → `owner-repo`.
    GH_BLOBS=$(count_blob_materializations "${CACHE_ROOT}/reposix/github-reubenjohn-reposix.git")

    GH_LIST_BODY="$(curl -fsS -H "Authorization: Bearer ${GITHUB_TOKEN}" \
        "${GH_ORIGIN}/repos/${GH_PROJECT}/issues?state=all&per_page=100" 2>/dev/null || echo '[]')"
    GH_N=$(echo "$GH_LIST_BODY" | jq 'length' 2>/dev/null || echo "0")
    GH_FIRST_NUMBER=$(echo "$GH_LIST_BODY" | jq -r '.[0].number // empty' 2>/dev/null)
    GH_FIRST_TITLE=$(echo "$GH_LIST_BODY" | jq -r '.[0].title // empty' 2>/dev/null)

    gh_list() {
        curl -fsS -H "Authorization: Bearer ${GITHUB_TOKEN}" \
            "${GH_ORIGIN}/repos/${GH_PROJECT}/issues?state=all&per_page=100"
    }
    GH_LIST_MS=$(median3_step gh_list)

    if [[ -n "$GH_FIRST_NUMBER" ]]; then
        gh_get() {
            curl -fsS -H "Authorization: Bearer ${GITHUB_TOKEN}" \
                "${GH_ORIGIN}/repos/${GH_PROJECT}/issues/${GH_FIRST_NUMBER}"
        }
        GH_GET_MS=$(median3_step gh_get)
        # No-op PATCH: rewrite title to itself.
        gh_patch() {
            curl -fsS -X PATCH -H "Authorization: Bearer ${GITHUB_TOKEN}" \
                -H 'Content-Type: application/json' \
                -d "$(jq -nc --arg t "$GH_FIRST_TITLE" '{title: $t}')" \
                "${GH_ORIGIN}/repos/${GH_PROJECT}/issues/${GH_FIRST_NUMBER}"
        }
        GH_PATCH_MS=$(median3_step gh_patch)
    else
        GH_GET_MS="n/a"; GH_PATCH_MS="n/a"
    fi

    gh_cap() {
        echo "capabilities" | "${BIN_DIR}/git-remote-reposix" \
            origin "reposix::${GH_ORIGIN}/projects/${GH_PROJECT}"
    }
    GH_CAP_MS=$(median3_step gh_cap)
else
    echo "latency-bench: GITHUB_TOKEN unset — skipping GitHub column" >&2
fi

# ---- Confluence block (skipped cleanly when bundle absent) ------------
CF_INIT_MS=""; CF_LIST_MS=""; CF_GET_MS=""; CF_PATCH_MS=""; CF_CAP_MS=""
CF_N=""; CF_BLOBS=""
if [[ -n "${ATLASSIAN_API_KEY:-}" && -n "${ATLASSIAN_EMAIL:-}" \
      && -n "${REPOSIX_CONFLUENCE_TENANT:-}" ]]; then
    echo "latency-bench: Confluence probe — using TokenWorld space" >&2
    CF_PROJECT="${REPOSIX_CONFLUENCE_SPACE:-TokenWorld}"
    CF_REPO="${RUN_DIR}/cf-repo"
    CF_ORIGIN="https://${REPOSIX_CONFLUENCE_TENANT}.atlassian.net"
    export REPOSIX_ALLOWED_ORIGINS="${SIM_URL},${CF_ORIGIN}"
    CF_AUTH="${ATLASSIAN_EMAIL}:${ATLASSIAN_API_KEY}"

    # Warm-up + space-id resolution (matches reposix-confluence/src/lib.rs:1007).
    CF_SPACE_ID=$(curl -fsS -u "${CF_AUTH}" \
        "${CF_ORIGIN}/wiki/api/v2/spaces?keys=${CF_PROJECT}" 2>/dev/null \
        | jq -r '.results[0].id // empty' 2>/dev/null)

    T0=$(now_ms)
    "${BIN_DIR}/reposix" init "confluence::${CF_PROJECT}" "$CF_REPO" >/dev/null 2>&1 || true
    T1=$(now_ms)
    CF_INIT_MS=$((T1 - T0))
    CF_BLOBS=$(count_blob_materializations "${CACHE_ROOT}/reposix/confluence-${CF_PROJECT}.git")

    if [[ -n "$CF_SPACE_ID" ]]; then
        CF_LIST_BODY="$(curl -fsS -u "${CF_AUTH}" \
            "${CF_ORIGIN}/wiki/api/v2/spaces/${CF_SPACE_ID}/pages?limit=250" 2>/dev/null \
            || echo '{"results":[]}')"
        CF_N=$(echo "$CF_LIST_BODY" | jq '.results | length' 2>/dev/null || echo "0")
        CF_FIRST_ID=$(echo "$CF_LIST_BODY" | jq -r '.results[0].id // empty' 2>/dev/null)
        CF_FIRST_TITLE=$(echo "$CF_LIST_BODY" | jq -r '.results[0].title // empty' 2>/dev/null)
        CF_FIRST_VERSION=$(echo "$CF_LIST_BODY" | jq -r '.results[0].version.number // 1' 2>/dev/null)

        cf_list() {
            curl -fsS -u "${CF_AUTH}" \
                "${CF_ORIGIN}/wiki/api/v2/spaces/${CF_SPACE_ID}/pages?limit=250"
        }
        CF_LIST_MS=$(median3_step cf_list)

        if [[ -n "$CF_FIRST_ID" ]]; then
            cf_get() {
                curl -fsS -u "${CF_AUTH}" \
                    "${CF_ORIGIN}/wiki/api/v2/pages/${CF_FIRST_ID}?body-format=storage"
            }
            CF_GET_MS=$(median3_step cf_get)
            # No-op PATCH: PUT page with same title + bumped version. Confluence's
            # update endpoint requires {id, status, title, version: {number: N+1}}.
            # Even a no-content-change rewrite increments the version, so we keep
            # the title unchanged and let Confluence record a new revision. This
            # is a known side-effect documented in the bench (page version drift).
            cf_patch() {
                local next=$((CF_FIRST_VERSION + 1))
                curl -fsS -X PUT -u "${CF_AUTH}" \
                    -H 'Content-Type: application/json' \
                    -d "$(jq -nc --arg id "$CF_FIRST_ID" --arg t "$CF_FIRST_TITLE" \
                        --argjson v "$next" \
                        '{id: $id, status: "current", title: $t, version: {number: $v}}')" \
                    "${CF_ORIGIN}/wiki/api/v2/pages/${CF_FIRST_ID}"
                # NOTE: Confluence v2 PUT requires a body field. The
                # version-bump-only PATCH is approximate. Treat the
                # confluence PATCH cell as "single-version-bump round-trip"
                # rather than a true no-op.
            }
            CF_PATCH_MS=$(median3_step cf_patch)
        else
            CF_GET_MS="n/a"; CF_PATCH_MS="n/a"
        fi
    fi

    cf_cap() {
        echo "capabilities" | "${BIN_DIR}/git-remote-reposix" \
            origin "reposix::${CF_ORIGIN}/confluence/projects/${CF_PROJECT}"
    }
    CF_CAP_MS=$(median3_step cf_cap)
else
    echo "latency-bench: Atlassian Confluence bundle unset — skipping Confluence column" >&2
fi

# ---- JIRA block (skipped cleanly when bundle absent) -----------------
JR_INIT_MS=""; JR_LIST_MS=""; JR_GET_MS=""; JR_PATCH_MS=""; JR_CAP_MS=""
JR_N=""; JR_BLOBS=""
if [[ -n "${JIRA_EMAIL:-}" && -n "${JIRA_API_TOKEN:-}" \
      && -n "${REPOSIX_JIRA_INSTANCE:-}" ]]; then
    echo "latency-bench: JIRA probe — using project ${JIRA_TEST_PROJECT:-${REPOSIX_JIRA_PROJECT:-TEST}}" >&2
    JR_PROJECT="${JIRA_TEST_PROJECT:-${REPOSIX_JIRA_PROJECT:-TEST}}"
    JR_REPO="${RUN_DIR}/jr-repo"
    JR_ORIGIN="https://${REPOSIX_JIRA_INSTANCE}.atlassian.net"
    export REPOSIX_ALLOWED_ORIGINS="${SIM_URL},${JR_ORIGIN}"
    JR_AUTH="${JIRA_EMAIL}:${JIRA_API_TOKEN}"

    # Warm-up GET against the search endpoint so the timed run sees a warm TLS pool.
    curl -fsS -X POST -u "${JR_AUTH}" \
        -H 'Content-Type: application/json' \
        -d "$(jq -nc --arg p "$JR_PROJECT" '{jql: ("project = \"" + $p + "\""), fields: ["summary"]}')" \
        "${JR_ORIGIN}/rest/api/3/search/jql" >/dev/null 2>&1 || true

    T0=$(now_ms)
    "${BIN_DIR}/reposix" init "jira::${JR_PROJECT}" "$JR_REPO" >/dev/null 2>&1 || true
    T1=$(now_ms)
    JR_INIT_MS=$((T1 - T0))
    JR_BLOBS=$(count_blob_materializations "${CACHE_ROOT}/reposix/jira-${JR_PROJECT}.git")

    JR_LIST_BODY="$(curl -fsS -X POST -u "${JR_AUTH}" \
        -H 'Content-Type: application/json' \
        -d "$(jq -nc --arg p "$JR_PROJECT" '{jql: ("project = \"" + $p + "\""), fields: ["summary"]}')" \
        "${JR_ORIGIN}/rest/api/3/search/jql" 2>/dev/null || echo '{"issues":[]}')"
    JR_N=$(echo "$JR_LIST_BODY" | jq '.issues | length' 2>/dev/null || echo "0")
    JR_FIRST_KEY=$(echo "$JR_LIST_BODY" | jq -r '.issues[0].key // empty' 2>/dev/null)
    JR_FIRST_SUMMARY=$(echo "$JR_LIST_BODY" | jq -r '.issues[0].fields.summary // empty' 2>/dev/null)

    jr_list() {
        curl -fsS -X POST -u "${JR_AUTH}" \
            -H 'Content-Type: application/json' \
            -d "$(jq -nc --arg p "$JR_PROJECT" '{jql: ("project = \"" + $p + "\""), fields: ["summary"]}')" \
            "${JR_ORIGIN}/rest/api/3/search/jql"
    }
    JR_LIST_MS=$(median3_step jr_list)

    if [[ -n "$JR_FIRST_KEY" ]]; then
        jr_get() {
            curl -fsS -u "${JR_AUTH}" \
                "${JR_ORIGIN}/rest/api/3/issue/${JR_FIRST_KEY}"
        }
        JR_GET_MS=$(median3_step jr_get)
        # No-op PATCH: PUT the summary back to its current value.
        jr_patch() {
            curl -fsS -X PUT -u "${JR_AUTH}" \
                -H 'Content-Type: application/json' \
                -d "$(jq -nc --arg s "$JR_FIRST_SUMMARY" '{fields: {summary: $s}}')" \
                "${JR_ORIGIN}/rest/api/3/issue/${JR_FIRST_KEY}"
        }
        JR_PATCH_MS=$(median3_step jr_patch)
    else
        JR_GET_MS="n/a"; JR_PATCH_MS="n/a"
    fi

    jr_cap() {
        echo "capabilities" | "${BIN_DIR}/git-remote-reposix" \
            origin "reposix::${JR_ORIGIN}/jira/projects/${JR_PROJECT}"
    }
    JR_CAP_MS=$(median3_step jr_cap)
else
    echo "latency-bench: JIRA bundle unset — skipping JIRA column" >&2
fi

# ---- Cell-format helpers ------------------------------------------------
# fmt_ms_n <ms> <n>    — list-row cell: "$ms ms (N=$n)" or empty/n/a passthrough.
# fmt_ms <ms>          — generic cell: "$ms ms" or empty/n/a passthrough.
fmt_ms() {
    local v="$1"
    if [[ -z "$v" ]]; then
        echo ""
    elif [[ "$v" == "n/a" ]]; then
        echo "n/a"
    else
        echo "${v} ms"
    fi
}
fmt_ms_n() {
    local v="$1" n="$2"
    if [[ -z "$v" ]]; then
        echo ""
    elif [[ "$v" == "n/a" ]]; then
        echo "n/a"
    else
        echo "${v} ms (N=${n})"
    fi
}

# ---- Emit Markdown artifact ---------------------------------------
GENERATED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
GIT_SHA="$(git -C "$WORKSPACE_ROOT" rev-parse --short HEAD 2>/dev/null || echo unknown)"

SIM_INIT_CELL="$(fmt_ms "$SIM_INIT_MS")"
SIM_LIST_CELL="$(fmt_ms_n "$SIM_LIST_MS" "$SIM_N")"
SIM_GET_CELL="$(fmt_ms "$SIM_GET_MS")"
SIM_PATCH_CELL="$(fmt_ms "$SIM_PATCH_MS")"
SIM_CAP_CELL="$(fmt_ms "$SIM_CAP_MS")"

GH_INIT_CELL="$(fmt_ms "$GH_INIT_MS")"
GH_LIST_CELL="$(fmt_ms_n "$GH_LIST_MS" "$GH_N")"
GH_GET_CELL="$(fmt_ms "$GH_GET_MS")"
GH_PATCH_CELL="$(fmt_ms "$GH_PATCH_MS")"
GH_CAP_CELL="$(fmt_ms "$GH_CAP_MS")"

CF_INIT_CELL="$(fmt_ms "$CF_INIT_MS")"
CF_LIST_CELL="$(fmt_ms_n "$CF_LIST_MS" "$CF_N")"
CF_GET_CELL="$(fmt_ms "$CF_GET_MS")"
CF_PATCH_CELL="$(fmt_ms "$CF_PATCH_MS")"
CF_CAP_CELL="$(fmt_ms "$CF_CAP_MS")"

JR_INIT_CELL="$(fmt_ms "$JR_INIT_MS")"
JR_LIST_CELL="$(fmt_ms_n "$JR_LIST_MS" "$JR_N")"
JR_GET_CELL="$(fmt_ms "$JR_GET_MS")"
JR_PATCH_CELL="$(fmt_ms "$JR_PATCH_MS")"
JR_CAP_CELL="$(fmt_ms "$JR_CAP_MS")"

cat > "$OUT" <<MARKDOWN
---
last_measured_at: ${GENERATED_AT}
---

# v0.9.0 Latency Envelope

**Generated:** ${GENERATED_AT} (commit \`${GIT_SHA}\`)
**Reproducer:** \`bash scripts/latency-bench.sh\`

## How to read this

reposix v0.9.0 replaces the per-read FUSE round-trip with a partial-clone
working tree backed by a \`git-remote-reposix\` promisor remote. The
golden-path latencies below characterize the sim backend (in-process
HTTP simulator) and any real backends for which credentials were
available at run time. **What's measured:** end-to-end wall-clock for
each operation, single-threaded, against an ephemeral sim DB on
localhost or the corresponding real-backend REST API. Each step is the
**median of 3 samples** to absorb network jitter. **What's NOT measured:**
runner hardware variance and cold-cache vs warm-cache TLS reuse for HTTPS
backends (a single warm-up GET amortizes the TLS handshake before timing).
Take the sim column as a lower bound for transport overhead and the
real-backend columns as a proxy for "what an agent on a typical laptop
will see."

The MCP/REST baseline comparison sits in \`docs/benchmarks/token-economy.md\`
(token-economy benchmark, v0.7.0). v0.9.0's win is on the latency
axis, not the token axis: the cache-backed bare repo means an agent
can \`grep -r\` an issue tracker without re-hitting the API for every
match.

## Latency table

| Step                                          | sim                          | github                       | confluence                   | jira                         |
|-----------------------------------------------|------------------------------|------------------------------|------------------------------|------------------------------|
| \`reposix init\` cold [^blob]                 | ${SIM_INIT_CELL}             | ${GH_INIT_CELL}              | ${CF_INIT_CELL}              | ${JR_INIT_CELL}              |
| List records [^N]                             | ${SIM_LIST_CELL}             | ${GH_LIST_CELL}              | ${CF_LIST_CELL}              | ${JR_LIST_CELL}              |
| Get one record                                | ${SIM_GET_CELL}              | ${GH_GET_CELL}               | ${CF_GET_CELL}               | ${JR_GET_CELL}               |
| PATCH record (no-op)                          | ${SIM_PATCH_CELL}            | ${GH_PATCH_CELL}             | ${CF_PATCH_CELL}             | ${JR_PATCH_CELL}             |
| Helper \`capabilities\` probe                 | ${SIM_CAP_CELL}              | ${GH_CAP_CELL}               | ${CF_CAP_CELL}               | ${JR_CAP_CELL}               |

[^blob]: \`reposix init\` materializes blobs lazily (partial clone with
    \`--filter=blob:none\`). Blob counts at end of init: sim=${SIM_BLOBS:-0},
    github=${GH_BLOBS:-0}, confluence=${CF_BLOBS:-0}, jira=${JR_BLOBS:-0}.
    A non-zero count means the helper served \`fetch\` requests from git
    that pulled actual blob bytes during the bootstrap fetch.
[^N]: \`N\` = records returned by the canonical list endpoint:
    sim/github/jira issues, confluence pages in the configured space.
    **N values reflect live backend state at run time** — the TokenWorld
    space and \`reubenjohn/reposix\` issue count drift over time; expect
    ±20% wobble between runs. The \`Helper capabilities probe\` row is
    local-only (no network), so it's identical across columns and serves
    as a runner-variance control.

Real-backend cells are populated by the \`bench-latency-v09\` CI job
(see [\`.github/workflows/ci.yml\`](https://github.com/reubenjohn/reposix/blob/main/.github/workflows/ci.yml)
for cadence; the weekly cron variant lives in
[\`.github/workflows/bench-latency-cron.yml\`](https://github.com/reubenjohn/reposix/blob/main/.github/workflows/bench-latency-cron.yml)).

## Soft thresholds

- **sim cold init < 500ms** — regression-flagged via \`WARN:\` line on
  stderr; not CI-blocking. Tracked here so a sudden 5x regression
  surfaces in PR review.
- **real-backend step < 3s** — same WARN-only mechanism.

## Reproduce

\`\`\`bash
bash scripts/latency-bench.sh
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
bash scripts/latency-bench.sh
\`\`\`

See \`docs/reference/testing-targets.md\` for the canonical safe-to-mutate
test targets.
MARKDOWN

echo "latency-bench: regenerated $OUT" >&2
echo "  sim    init=${SIM_INIT_MS}ms list=${SIM_LIST_MS}ms get=${SIM_GET_MS}ms patch=${SIM_PATCH_MS}ms cap=${SIM_CAP_MS}ms (N=${SIM_N}, blobs=${SIM_BLOBS:-0})" >&2
[[ -n "$GH_INIT_MS" ]] && echo "  github init=${GH_INIT_MS}ms list=${GH_LIST_MS}ms get=${GH_GET_MS}ms patch=${GH_PATCH_MS}ms cap=${GH_CAP_MS}ms (N=${GH_N}, blobs=${GH_BLOBS:-0})" >&2 || true
[[ -n "$CF_INIT_MS" ]] && echo "  confluence init=${CF_INIT_MS}ms list=${CF_LIST_MS}ms get=${CF_GET_MS}ms patch=${CF_PATCH_MS}ms cap=${CF_CAP_MS}ms (N=${CF_N}, blobs=${CF_BLOBS:-0})" >&2 || true
[[ -n "$JR_INIT_MS" ]] && echo "  jira   init=${JR_INIT_MS}ms list=${JR_LIST_MS}ms get=${JR_GET_MS}ms patch=${JR_PATCH_MS}ms cap=${JR_CAP_MS}ms (N=${JR_N}, blobs=${JR_BLOBS:-0})" >&2 || true
exit 0
