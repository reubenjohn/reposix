#!/usr/bin/env bash
# quality/gates/perf/latency-bench/jira.sh -- jira real-backend probe block.
#
# Sourced by ../latency-bench.sh after lib.sh + sim block. Skipped cleanly
# when the JIRA credential bundle is absent. Reads: BIN_DIR, RUN_DIR,
# SIM_URL, CACHE_ROOT. Writes (exports): JR_INIT_MS, JR_LIST_MS, JR_GET_MS,
# JR_PATCH_MS, JR_CAP_MS, JR_N, JR_BLOBS.

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
