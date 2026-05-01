#!/usr/bin/env bash
# quality/gates/agent-ux/bus-url-parses-query-param-form.sh — agent-ux
# verifier for catalog row `agent-ux/bus-url-parses-query-param-form`.
#
# CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/bus-url-parses-query-param-form
# CADENCE:     pre-pr (~10s wall time)
# INVARIANT:   bus_url::parse("reposix::sim::demo?mirror=file:///tmp/m.git")
#              returns Route::Bus { sot: <expected>, mirror_url: <expected> }.
#              Single-backend "reposix::sim::demo" (no ?) returns Route::Single(...).
#
# Status until P82-01 T06: FAIL.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

cargo test -p reposix-remote --test bus_url \
    parses_query_param_form_round_trip route_single_for_bare_reposix_url \
    --quiet -- --nocapture 2>&1 | tail -20

echo "PASS: bus_url::parse handles ?mirror= form + bare reposix:: form"
exit 0
