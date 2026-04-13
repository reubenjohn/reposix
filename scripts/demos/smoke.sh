#!/usr/bin/env bash
# scripts/demos/smoke.sh — run every Tier 1 demo via assert.sh.
#
# Invoked by CI (.github/workflows/ci.yml `demos-smoke` job) and by
# developers before pushing. Exits 0 iff all Tier 1 demos pass their
# own ASSERTS markers AND return rc=0.
#
# Prereq: release binaries (reposix-sim, reposix-fuse, git-remote-reposix)
# must be on PATH. In CI the job does:
#   cargo build --release --workspace --bins
#   PATH="$PWD/target/release:$PATH" bash scripts/demos/smoke.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

DEMOS=(
    "${SCRIPT_DIR}/01-edit-and-push.sh"
    "${SCRIPT_DIR}/02-guardrails.sh"
    "${SCRIPT_DIR}/03-conflict-resolution.sh"
    "${SCRIPT_DIR}/04-token-economy.sh"
)

PASS=0
FAIL=0
FAILED_LIST=()

echo "================================================================"
echo "  reposix demos — smoke suite (${#DEMOS[@]} demos)"
echo "================================================================"

for demo in "${DEMOS[@]}"; do
    if [[ ! -f "$demo" ]]; then
        echo ">>> MISSING $(basename "$demo")" >&2
        FAIL=$((FAIL + 1))
        FAILED_LIST+=("$(basename "$demo"): not found")
        continue
    fi
    name="$(basename "$demo")"
    echo
    echo "----------------------------------------------------------------"
    echo ">>> $name"
    echo "----------------------------------------------------------------"
    set +e
    bash "${SCRIPT_DIR}/assert.sh" "$demo"
    rc=$?
    set -e
    if [[ $rc -eq 0 ]]; then
        PASS=$((PASS + 1))
    else
        FAIL=$((FAIL + 1))
        FAILED_LIST+=("$name: rc=$rc")
        # Fail-fast: a broken demo means the whole suite is untrustworthy,
        # and running later demos on the same host risks cascading cleanup
        # issues (stale FUSE mounts, lingering sim processes).
        break
    fi
done

echo
echo "================================================================"
echo "  smoke suite: ${PASS} passed, ${FAIL} failed (of ${#DEMOS[@]})"
echo "================================================================"

if [[ $FAIL -gt 0 ]]; then
    echo "failed:"
    for f in "${FAILED_LIST[@]}"; do
        echo "  - $f"
    done
    exit 1
fi

exit 0
