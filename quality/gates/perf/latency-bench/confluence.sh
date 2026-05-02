#!/usr/bin/env bash
# quality/gates/perf/latency-bench/confluence.sh -- confluence real-backend probe block.
#
# Sourced by ../latency-bench.sh after lib.sh + sim block. Skipped cleanly
# when the Atlassian credential bundle is absent. Reads: BIN_DIR, RUN_DIR,
# SIM_URL, CACHE_ROOT. Writes (exports): CF_INIT_MS, CF_LIST_MS, CF_GET_MS,
# CF_PATCH_MS, CF_CAP_MS, CF_N, CF_BLOBS, CF_PROJECT.

CF_INIT_MS=""; CF_LIST_MS=""; CF_GET_MS=""; CF_PATCH_MS=""; CF_CAP_MS=""
CF_N=""; CF_BLOBS=""
if [[ -n "${ATLASSIAN_API_KEY:-}" && -n "${ATLASSIAN_EMAIL:-}" \
      && -n "${REPOSIX_CONFLUENCE_TENANT:-}" ]]; then
    CF_PROJECT="${REPOSIX_CONFLUENCE_SPACE:-TokenWorld}"
    echo "latency-bench: Confluence probe — using ${CF_PROJECT} space" >&2
    CF_REPO="${RUN_DIR}/cf-repo"
    CF_ORIGIN="https://${REPOSIX_CONFLUENCE_TENANT}.atlassian.net"
    export REPOSIX_ALLOWED_ORIGINS="${SIM_URL},${CF_ORIGIN}"
    CF_AUTH="${ATLASSIAN_EMAIL}:${ATLASSIAN_API_KEY}"

    # Warm-up + space-id resolution (matches reposix-confluence/src/lib.rs:1007).
    # Tolerate HTTP errors here — a 404/401 from the v2 spaces endpoint must NOT
    # crash the entire bench under `set -euo pipefail`. Empty CF_SPACE_ID falls
    # through to the `if [[ -n ]]` guard below which leaves CF_LIST/GET/PATCH
    # unset; fmt_ms renders those as empty cells.
    CF_SPACE_ID=$(curl -fsS -u "${CF_AUTH}" \
        "${CF_ORIGIN}/wiki/api/v2/spaces?keys=${CF_PROJECT}" 2>/dev/null \
        | jq -r '.results[0].id // empty' 2>/dev/null || echo "")

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
            # Even a no-content-change rewrite increments the version; treat the
            # confluence PATCH cell as "single-version-bump round-trip" rather
            # than a true no-op.
            cf_patch() {
                local next=$((CF_FIRST_VERSION + 1))
                curl -fsS -X PUT -u "${CF_AUTH}" \
                    -H 'Content-Type: application/json' \
                    -d "$(jq -nc --arg id "$CF_FIRST_ID" --arg t "$CF_FIRST_TITLE" \
                        --argjson v "$next" \
                        '{id: $id, status: "current", title: $t, version: {number: $v}}')" \
                    "${CF_ORIGIN}/wiki/api/v2/pages/${CF_FIRST_ID}"
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
