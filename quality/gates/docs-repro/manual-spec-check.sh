#!/usr/bin/env bash
# quality/gates/docs-repro/manual-spec-check.sh -- markdown-spec checker for
# kind: manual rows. Usage: manual-spec-check.sh <catalog-row-id>. <=50 lines.
# For docs-repro/example-03-claude-code-skill: asserts the 2 markdown files
# exist + reference reposix-agent-flow. Stdlib only; no docker.
set -uo pipefail
ROW_ID="${1:?usage: $0 <catalog-row-id>}"
REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
ARTIFACT="$REPO_ROOT/quality/reports/verifications/docs-repro/${ROW_ID#docs-repro/}.json"
mkdir -p "$(dirname "$ARTIFACT")"

PASSED=()
FAILED=()
EXIT_CODE=0

case "$ROW_ID" in
  docs-repro/example-03-claude-code-skill)
    for f in examples/03-claude-code-skill/RUN.md examples/03-claude-code-skill/agent-prompt.md; do
      if [[ -s "$REPO_ROOT/$f" ]]; then PASSED+=("$f exists with content"); else FAILED+=("$f missing or empty"); fi
    done
    if grep -q '\.claude/skills/reposix-agent-flow' "$REPO_ROOT/examples/03-claude-code-skill/RUN.md" 2>/dev/null; then
      PASSED+=("RUN.md references .claude/skills/reposix-agent-flow")
    else
      FAILED+=("RUN.md does NOT reference .claude/skills/reposix-agent-flow")
    fi
    ;;
  *)
    echo "manual-spec-check: unknown row id $ROW_ID" >&2
    exit 2
    ;;
esac

[[ ${#FAILED[@]} -gt 0 ]] && EXIT_CODE=1

python3 - "$ARTIFACT" "$ROW_ID" "$EXIT_CODE" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "${PASSED[@]:-}" -- "${FAILED[@]:-}" <<'PY'
import json, sys
artifact, rid, exit_code, ts = sys.argv[1:5]
rest = sys.argv[5:]
sep = rest.index("--")
passed = [s for s in rest[:sep] if s]
failed = [s for s in rest[sep + 1:] if s]
data = {"ts": ts, "row_id": rid, "exit_code": int(exit_code),
        "asserts_passed": passed, "asserts_failed": failed}
open(artifact, "w").write(json.dumps(data, indent=2) + "\n")
PY

exit "$EXIT_CODE"
