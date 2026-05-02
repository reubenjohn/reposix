← [back to index](./index.md) · phase 84 plan 01

## Task 84-01-T01 — Catalog-first: mint 6 catalog rows + author 6 verifier shells (Part A)

## Task 84-01-T01 — Catalog-first: mint 6 catalog rows + author 6 verifier shells

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
