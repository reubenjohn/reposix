← [index](./index.md) | → [T01b](./T01-catalog-rows.md)

## Task 84-01-T01 — Catalog-first (part 1: read_first + shells 1-3)

<read_first>
- `quality/catalogs/agent-ux.json` (full file — existing rows from
  P79–P83; the 6 new rows join the same `rows` array; row shape
  mirrors P81's `agent-ux/sync-reconcile-subcommand` and P82's
  `agent-ux/bus-fetch-not-advertised`. Re-confirm exact schema
  during T01 read_first via `python3 -c 'import json;
  json.load(open("quality/catalogs/agent-ux.json"))'`.)
- `quality/gates/agent-ux/sync-reconcile-subcommand.sh` (P81 TINY
  verifier precedent — 30-line shape).
- `quality/gates/agent-ux/bus-fetch-not-advertised.sh` (P82 TINY
  verifier precedent — grep-shape).
- `quality/gates/agent-ux/dark-factory.sh` (P59+ shell-harness
  precedent — file:// bare-repo fixtures with mktemp + trap).
- `quality/catalogs/README.md` § "Unified schema" — required fields
  per row.
- `quality/PROTOCOL.md` § "Principle A — Subagents propose; tools
  validate and mint" — confirms the gap (Principle A applies to
  docs-alignment dim only; agent-ux dim has no `bind` verb yet).
- `.planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md` §
  GOOD-TO-HAVES-01 — the documented-gap framing the agent-ux row
  annotations cite.
- `.planning/phases/84-webhook-mirror-sync/84-PLAN-OVERVIEW.md`
  § "Decisions ratified at plan time" D-04 (`agent-ux.json` is the
  catalog home, NOT a new file).
</read_first>

<action>
This is the **catalog-first commit** for P84: the rows defining the
GREEN contract land BEFORE T02–T06 implementation. SIX verifier
shell scripts ship in this same atomic commit (one per row).

**Important — agent-ux rows are NOT a Principle A application.**
The `reposix-quality bind` verb supports the `docs-alignment`
dimension only. The `agent-ux` dim is hand-edited per the existing
P79/P80/P81/P82/P83 precedent. The orchestrator filed
GOOD-TO-HAVES-01 during P79 to track the bind-verb extension work
for a future polish slot. The catalog edits in this task are
therefore **hand-edits per documented gap**, not Principle A
applications. This is annotated in the commit message AND in each
row's `_provenance_note` field (mirroring P79–P83 row annotations).

Steps:

1. **Author the six verifier shells.** Each is TINY (~30-80 lines)
   mirroring `quality/gates/agent-ux/bus-fetch-not-advertised.sh`
   for the grep-shape rows and `quality/gates/agent-ux/dark-factory.sh`
   for the shell-harness rows. The verifiers exercise the relevant
   scenario; they will FAIL until T02–T05 implement and T06 flips
   the catalog status.

   `quality/gates/agent-ux/webhook-trigger-dispatch.sh` (~70 lines —
   asserts BOTH copies exist + structural invariants):

   ```bash
   #!/usr/bin/env bash
   # quality/gates/agent-ux/webhook-trigger-dispatch.sh — agent-ux
   # verifier for catalog row `agent-ux/webhook-trigger-dispatch`.
   #
   # CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/webhook-trigger-dispatch
   # CADENCE:     pre-pr (~15s wall time)
   # INVARIANT:   Workflow YAML exists at BOTH:
   #              (a) docs/guides/dvcs-mirror-setup-template.yml (canonical repo, this checkout)
   #              (b) reubenjohn/reposix-tokenworld-mirror:.github/workflows/reposix-mirror-sync.yml (live; via gh api)
   #              The two are byte-equal modulo whitespace (diff -w returns zero).
   #              YAML structure: repository_dispatch types=[reposix-mirror-sync] AND
   #                              cargo binstall reposix-cli (NOT bare 'reposix') AND
   #                              NO github.event.client_payload references.
   #
   # Status until P84-01 T06: FAIL.
   set -euo pipefail
   SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
   REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
   cd "${REPO_ROOT}"

   TEMPLATE="docs/guides/dvcs-mirror-setup-template.yml"
   MIRROR_REPO="reubenjohn/reposix-tokenworld-mirror"
   LIVE_PATH=".github/workflows/reposix-mirror-sync.yml"

   # 1. Template copy must exist + parse as YAML.
   test -f "$TEMPLATE" || { echo "FAIL: $TEMPLATE missing"; exit 1; }
   python3 -c "import yaml,sys; yaml.safe_load(open(sys.argv[1]))" "$TEMPLATE" \
     || { echo "FAIL: $TEMPLATE does not parse as YAML"; exit 1; }

   # 2. Live copy must exist via gh api.
   command -v gh >/dev/null \
     || { echo "FAIL: gh not on PATH"; exit 1; }
   LIVE_CONTENT=$(mktemp); trap "rm -f $LIVE_CONTENT" EXIT
   gh api "repos/${MIRROR_REPO}/contents/${LIVE_PATH}" -q .content 2>/dev/null \
     | base64 -d > "$LIVE_CONTENT" \
     || { echo "FAIL: live copy ${MIRROR_REPO}:${LIVE_PATH} unreachable"; exit 1; }
   test -s "$LIVE_CONTENT" \
     || { echo "FAIL: live copy is empty"; exit 1; }

   # 3. Two copies are byte-equal modulo whitespace.
   diff -w "$TEMPLATE" "$LIVE_CONTENT" >/dev/null \
     || { echo "FAIL: template and live copies differ (modulo whitespace)"; \
          diff -w "$TEMPLATE" "$LIVE_CONTENT" | head -20; exit 1; }

   # 4. Structural greps on the YAML.
   grep -q "repository_dispatch:" "$TEMPLATE" \
     || { echo "FAIL: missing repository_dispatch trigger"; exit 1; }
   grep -qE "types:\s*\[\s*reposix-mirror-sync\s*\]" "$TEMPLATE" \
     || { echo "FAIL: missing types: [reposix-mirror-sync]"; exit 1; }
   grep -q "cargo binstall reposix-cli" "$TEMPLATE" \
     || { echo "FAIL: missing 'cargo binstall reposix-cli' (D-05; must NOT be bare 'reposix')"; exit 1; }
   ! grep -qE "cargo binstall reposix(\s|$)(?!-cli)" "$TEMPLATE" 2>/dev/null \
     || { echo "FAIL: bare 'cargo binstall reposix' detected (use reposix-cli per D-05)"; exit 1; }
   ! grep -q "github.event.client_payload" "$TEMPLATE" \
     || { echo "FAIL: github.event.client_payload reference detected (S2/T-84-01 violation)"; exit 1; }
   ! grep -q "set -x" "$TEMPLATE" \
     || { echo "FAIL: 'set -x' detected (T-84-02 secret-leak risk)"; exit 1; }

   echo "PASS: workflow YAML present in both copies, byte-equal mod whitespace, structural invariants hold"
   exit 0
   ```

   `quality/gates/agent-ux/webhook-cron-fallback.sh` (~30 lines —
   grep-shape):

   ```bash
   #!/usr/bin/env bash
   # CATALOG ROW: agent-ux/webhook-cron-fallback
   # CADENCE: pre-pr
   # INVARIANT: Workflow YAML has literal cron '*/30 * * * *' (D-06; NEVER vars.*),
   #            actions/checkout@v6 with fetch-depth: 0 (D-04 / Pitfall 4),
   #            concurrency: { group: reposix-mirror-sync, cancel-in-progress: false } (D-01).
   set -euo pipefail
   SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
   REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
   cd "${REPO_ROOT}"

   TEMPLATE="docs/guides/dvcs-mirror-setup-template.yml"
   test -f "$TEMPLATE" || { echo "FAIL: $TEMPLATE missing"; exit 1; }

   grep -qF "'*/30 * * * *'" "$TEMPLATE" \
     || { echo "FAIL: missing literal cron '*/30 * * * *' (D-06)"; exit 1; }
   ! grep -qE 'cron:\s*.*\$\{\{' "$TEMPLATE" \
     || { echo "FAIL: cron field uses \${{ ... }} interpolation (Pitfall 3)"; exit 1; }
   grep -q "actions/checkout@v6" "$TEMPLATE" \
     || { echo "FAIL: missing actions/checkout@v6"; exit 1; }
   grep -qE "fetch-depth:\s*0" "$TEMPLATE" \
     || { echo "FAIL: missing fetch-depth: 0 (D-04 / Pitfall 4)"; exit 1; }
   grep -q "group: reposix-mirror-sync" "$TEMPLATE" \
     || { echo "FAIL: missing concurrency group (D-01)"; exit 1; }
   grep -qE "cancel-in-progress:\s*false" "$TEMPLATE" \
     || { echo "FAIL: missing cancel-in-progress: false (D-01)"; exit 1; }

   echo "PASS: cron literal + fetch-depth + concurrency invariants hold"
   exit 0
   ```

   `quality/gates/agent-ux/webhook-force-with-lease-race.sh` (the
   T04 harness — content authored at T01 as the verifier shell;
   T04 deepens or stays as-is — see T04 for the verbatim body).
   At T01, ship the SHELL of the harness with a leading
   `echo "TODO: T04 implements"; exit 1` so the verifier FAILS
   cleanly until T04 completes:

   ```bash
   #!/usr/bin/env bash
   # CATALOG ROW: agent-ux/webhook-force-with-lease-race
   # CADENCE: pre-pr (~1s wall time)
   # INVARIANT: --force-with-lease=refs/heads/main:<SHA-A> rejects
   #            when the remote main has advanced to <SHA-B> via a
   #            concurrent push. Mirror state is untouched (still
   #            <SHA-B>) after the failed push.
   #
   # Status until P84-01 T04: FAIL (stub). T04 replaces with the full
   # ~50-line file:// bare-repo race walk-through.
   set -euo pipefail
   echo "FAIL: T04 not yet shipped (race walk-through harness)"
   exit 1
   ```

   Shells 4-6 + catalog JSON + validate + commit continued in [T01-catalog-rows.md](./T01-catalog-rows.md).
</action>
