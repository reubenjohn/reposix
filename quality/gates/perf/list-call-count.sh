#!/usr/bin/env bash
# quality/gates/perf/list-call-count.sh — perf verifier for catalog
# row `perf/handle-export-list-call-count`.
#
# CATALOG ROW: quality/catalogs/perf-targets.json -> perf/handle-export-list-call-count
# CADENCE:     pre-pr (~30s wall time)
# INVARIANT:   With N=200 records seeded in the wiremock harness and a
#              one-record edit pushed via the export verb, the precheck
#              makes >=1 list_changed_since REST call AND ZERO list_records
#              REST calls. The positive-control sibling test confirms
#              wiremock fails RED if the matcher is reverted (closes
#              RESEARCH.md MEDIUM risk).
#
# Implementation: delegates to the integration test
# `crates/reposix-remote/tests/perf_l1.rs::l1_precheck_uses_list_changed_since_not_list_records`
# which drives `git-remote-reposix` directly via stdin against a
# wiremock backend, and counts REST calls via wiremock matchers.
#
# Status until P81-01 T04: FAIL — wiring is scaffold-only in T01-T03;
# the integration test + behavior coverage land in T04.
#
# Usage: bash quality/gates/perf/list-call-count.sh
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

# Run cargo test; capture stderr+stdout to a log file and propagate the
# real exit code (not tail's). Pipelines mask exit codes; using a
# tempfile keeps the verifier honest (PIPESTATUS would also work but is
# bashism-fragile across CI shells).
LOG="$(mktemp)"
trap 'rm -f "${LOG}"' EXIT
if ! cargo test -p reposix-remote --test perf_l1 \
        l1_precheck_uses_list_changed_since_not_list_records \
        --quiet -- --nocapture > "${LOG}" 2>&1; then
    echo "FAIL: l1_precheck_uses_list_changed_since_not_list_records did not pass" >&2
    tail -40 "${LOG}" >&2
    exit 1
fi

echo "PASS: L1 precheck makes >=1 list_changed_since calls AND zero list_records calls (N=200 wiremock harness)"
exit 0
