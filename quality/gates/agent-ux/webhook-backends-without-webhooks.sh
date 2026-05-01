#!/usr/bin/env bash
# CATALOG ROW: agent-ux/webhook-backends-without-webhooks
# CADENCE: pre-pr
# INVARIANT: Q4.2 backends-without-webhooks fallback — removing the
#            `repository_dispatch:` block from the workflow YAML
#            produces still-valid YAML that runs on cron + manual
#            dispatch only.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

TEMPLATE="docs/guides/dvcs-mirror-setup-template.yml"
test -f "$TEMPLATE" || { echo "FAIL: $TEMPLATE missing"; exit 1; }

# Confirm the repository_dispatch block exists (so trimming makes sense).
grep -q "repository_dispatch:" "$TEMPLATE" \
  || { echo "FAIL: repository_dispatch block missing — nothing to trim"; exit 1; }

# Simulate the trim: produce a copy with the `repository_dispatch:` block
# removed; assert the result still parses as YAML and still has at least
# one trigger (schedule + workflow_dispatch).
TRIMMED=$(mktemp)
trap 'rm -f "$TRIMMED"' EXIT
python3 - "$TEMPLATE" "$TRIMMED" <<'PYEOF'
import sys, yaml
src, dst = sys.argv[1], sys.argv[2]
doc = yaml.safe_load(open(src))
# YAML's `on:` key is parsed by PyYAML as the boolean True (YAML 1.1
# legacy). Find the actual key (could be 'on' or True).
on_key = None
for k in list(doc.keys()):
    if k == 'on' or k is True:
        on_key = k
        break
assert on_key is not None, f"workflow YAML has no 'on:' block; keys={list(doc.keys())}"
on_block = doc[on_key]
if isinstance(on_block, dict) and 'repository_dispatch' in on_block:
    del on_block['repository_dispatch']
# Sanity: at least one trigger remains.
assert isinstance(on_block, dict) and (on_block.get('schedule') or 'workflow_dispatch' in on_block), \
    "after trim, no triggers remain — workflow would never run"
yaml.safe_dump(doc, open(dst, 'w'))
PYEOF

python3 -c "import yaml,sys; yaml.safe_load(open(sys.argv[1]))" "$TRIMMED" \
  || { echo "FAIL: trimmed YAML does not parse"; exit 1; }

echo "PASS: cron-only mode preserved when repository_dispatch removed"
exit 0
