#!/usr/bin/env bash
# quality/gates/agent-ux/mirror-refs-write-on-success.sh — agent-ux
# verifier for catalog row `agent-ux/mirror-refs-write-on-success`.
#
# CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/mirror-refs-write-on-success
# CADENCE:     pre-pr (~30s wall time)
# INVARIANT:   After a single-backend push via the existing handle_export
#              path, the cache's bare repo has BOTH refs/mirrors/<sot>-head
#              and refs/mirrors/<sot>-synced-at; the synced-at tag's
#              message body's first line matches `mirror synced at <RFC3339>`;
#              audit_events_cache contains a row with op = 'mirror_sync_written'.
#
# Implementation: delegates to the integration test
# `crates/reposix-remote/tests/mirror_refs.rs::write_on_success_updates_both_refs`
# which drives `git-remote-reposix` directly via stdin against a wiremock
# backend, asserts refs in the cache's bare repo via `git for-each-ref` /
# `git cat-file -p`, and queries cache.db for the audit row. The test
# exercises the same handle_export success branch the verifier-shell
# scenario would, with a more deterministic harness (no port races, no
# `git fetch` plumbing through a working-tree clone).
#
# Usage: bash quality/gates/agent-ux/mirror-refs-write-on-success.sh
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

cargo test -p reposix-remote --test mirror_refs write_on_success_updates_both_refs --quiet -- --nocapture 2>&1 | tail -20

echo "PASS: mirror-refs written on push success; both refs resolvable; tag message body well-formed; audit row present"
exit 0
