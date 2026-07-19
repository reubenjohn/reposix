#!/usr/bin/env bash
# quality/gates/structure/hermetic-test-network-isolation.sh
#
# Backs catalog row structure/hermetic-test-network-isolation. Regression
# lock for the Cycle-2 task (d) fix (2026-07-19): test_freshness_synth.py
# used to invoke `run.py --cadence weekly`, which fanned out into 20+ live
# HTTP calls (crates.io API, GitHub API) via unrelated weekly-cadence
# catalog rows -- a flaky/offline network turned a P1 row FAIL and
# nondeterministically broke the test's exit-code assertion (the
# "stale-P2 flake", PR#77 family). See quality/CLAUDE.md "Hermetic test
# convention" for the network-mock rule this gate enforces.
#
# CI-portable network denial: a poisoned HTTP(S) proxy pointed at an
# unreachable local port. Any unmocked/un-neutralized urllib.request call
# routes through the proxy and fails fast with ECONNREFUSED, instead of
# reaching the real network -- no CAP_SYS_ADMIN / unprivileged user
# namespace required (unlike `unshare -rn`, which this gate's author also
# verified manually but which some CI sandboxes restrict). See the
# fallback mechanism documented in quality/CLAUDE.md.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "$REPO_ROOT"

env http_proxy=http://127.0.0.1:1 https_proxy=http://127.0.0.1:1 no_proxy= \
  python3 -m pytest quality/runners/test_freshness_synth.py -v

echo "PASS: test_freshness_synth.py is hermetic -- passed with a poisoned HTTP(S) proxy denying all live network calls"
