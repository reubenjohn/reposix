#!/usr/bin/env bash
# Sweep all 9 published reposix crates through quality/gates/release/crates-io-max-version.py.
# Promoted to scripts/ per CLAUDE.md §4 (Self-improving infrastructure) so the next agent
# can re-run the sweep with one named command instead of reconstructing the loop.
#
# Usage: bash scripts/check_crates_io_max_version_sweep.sh
#
# Exits 0 iff every crate verifies PASS; 1 otherwise. Per-crate stdout is the
# verifier's own output; the artifact JSONs land at
# quality/reports/verifications/release/crates-io-max-version-<crate>.json.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${REPO_ROOT}"

CRATES=(
  reposix-cli
  reposix-remote
  reposix-core
  reposix-cache
  reposix-sim
  reposix-github
  reposix-confluence
  reposix-jira
  reposix-swarm
)

fail=0
for crate in "${CRATES[@]}"; do
  if python3 quality/gates/release/crates-io-max-version.py --crate "${crate}"; then
    echo "  PASS: ${crate}"
  else
    echo "  FAIL: ${crate}"
    fail=1
  fi
done

if [[ "${fail}" -eq 0 ]]; then
  echo "all 9 crates PASS"
else
  echo "at least one crate FAILED — see per-crate artifacts"
fi
exit "${fail}"
