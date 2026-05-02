#!/usr/bin/env bash
# quality/gates/perf/latency-bench/github.sh -- github real-backend probe block.
#
# Sourced by ../latency-bench.sh after lib.sh + sim block. Skipped cleanly
# when GITHUB_TOKEN is absent. Reads: BIN_DIR, RUN_DIR, SIM_URL, CACHE_ROOT.
# Writes (exports): GH_INIT_MS, GH_LIST_MS, GH_GET_MS, GH_PATCH_MS, GH_CAP_MS,
# GH_N, GH_BLOBS.

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
