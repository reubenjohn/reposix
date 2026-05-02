#!/usr/bin/env bash
# quality/gates/code/lint-invariants/git-version-requirement-documented.sh
#
# Asserts the contributing.md invariant: "git >= 2.34 required for
# partial-clone + stateless-connect."
#
# This is a documentation-existence check: the runtime requirement is
# documented in BOTH docs/development/contributing.md AND CLAUDE.md (so
# both human contributors and agents are warned of the prerequisite).
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

failed=0

# The literal "2.34" must appear in both docs (each line that mentions
# 2.34 in this project also mentions git, so the bare-version check
# is sufficient and avoids regex portability concerns).
for doc in docs/development/contributing.md CLAUDE.md; do
  if ! grep -F -q '2.34' "$doc"; then
    echo "FAIL: $doc does not document the '2.34' git version requirement" >&2
    failed=1
  fi
done

if [[ "$failed" -ne 0 ]]; then
  exit 1
fi

echo "PASS: 'git >= 2.34' requirement documented in contributing.md + CLAUDE.md"
