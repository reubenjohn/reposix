#!/usr/bin/env bash
# quality/gates/agent-ux/mirror-refs-cited-in-reject-hint.sh — agent-ux
# verifier for catalog row `agent-ux/mirror-refs-cited-in-reject-hint`.
#
# CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/mirror-refs-cited-in-reject-hint
# CADENCE:     pre-pr (~30s wall time)
# INVARIANT:   After a successful push (refs populated), a SECOND push
#              with a stale prior triggers the conflict-reject path;
#              the helper's stderr cites refs/mirrors/<sot>-synced-at
#              with a parseable RFC3339 timestamp + `(N minutes ago)`.
#              First-push None case (no refs yet) omits the synced-at
#              hint cleanly per RESEARCH.md pitfall 7.
#
# Implementation: delegates to the integration tests
# `crates/reposix-remote/tests/mirror_refs.rs::reject_hint_cites_synced_at_with_age`
# (populated path) AND `::reject_hint_first_push_omits_synced_at_line`
# (None path). Both drive the helper directly via stdin against a
# wiremock backend; the populated-path test seeds refs via a first
# successful push, then drives a second push with a stale prior to fire
# the conflict-reject branch. The None-path test mounts the backend at
# a higher version while the inbound blob declares prior_version=3,
# producing a first-ever conflict with no refs yet.
#
# Usage: bash quality/gates/agent-ux/mirror-refs-cited-in-reject-hint.sh
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

cargo test -p reposix-remote --test mirror_refs reject_hint --quiet -- --nocapture 2>&1 | tail -20

echo "PASS: conflict-reject hint cites refs/mirrors/<sot>-synced-at with RFC3339 + (N minutes ago); first-push None case omits cleanly"
exit 0
