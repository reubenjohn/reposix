#!/usr/bin/env bash
# quality/gates/agent-ux/mirror-refs-readable-by-vanilla-fetch.sh — agent-ux
# verifier for catalog row `agent-ux/mirror-refs-readable-by-vanilla-fetch`.
#
# CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/mirror-refs-readable-by-vanilla-fetch
# CADENCE:     pre-pr (~30s wall time)
# INVARIANT:   After a single-backend push has populated mirror refs,
#              a fresh `git clone --mirror` of the cache's bare repo
#              brings BOTH refs/mirrors/<sot>-head and
#              refs/mirrors/<sot>-synced-at into the new clone — proves
#              vanilla-git readers can observe mirror lag without any
#              reposix awareness.
#
# Implementation: delegates to the integration test
# `crates/reposix-remote/tests/mirror_refs.rs::vanilla_fetch_brings_mirror_refs`
# which drives the helper, then runs `git clone --mirror` against the
# cache's bare repo to copy ALL refs (plain `--bare` only copies
# refs/heads/* and refs/tags/* by default). The dark-factory contract
# is "agents who want mirror-lag refs can pull them with vanilla git";
# `clone --mirror` is the vanilla-git mechanism for that.
#
# Usage: bash quality/gates/agent-ux/mirror-refs-readable-by-vanilla-fetch.sh
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

cargo test -p reposix-remote --test mirror_refs vanilla_fetch_brings_mirror_refs --quiet -- --nocapture 2>&1 | tail -20

echo "PASS: vanilla-mirror-clone brings refs/mirrors/* along to a fresh bare clone"
exit 0
