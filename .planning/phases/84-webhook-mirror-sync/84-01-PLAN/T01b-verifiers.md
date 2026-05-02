← [back to index](./index.md) · phase 84 plan 01

## Task 84-01-T01 — Verifier Shell Scripts (Part B)

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

   `quality/gates/agent-ux/webhook-first-run-empty-mirror.sh`
   (similar T01 stub):

   ```bash
   #!/usr/bin/env bash
   # CATALOG ROW: agent-ux/webhook-first-run-empty-mirror
   # CADENCE: pre-pr (~2s wall time)
   # INVARIANT: First-run handling per Q4.3:
   #   (4.3.a) fresh-but-readme mirror (gh repo create --add-readme):
   #     workflow's "if git show-ref --verify --quiet
   #     refs/remotes/mirror/main; then lease-push" branch fires;
   #     lease push succeeds; mirror's main advances.
   #   (4.3.b) truly-empty mirror (gh repo create, no --add-readme):
   #     plain-push branch fires; main is created on mirror.
   #   Both fixtures are file:// bare repos.
   #
   # Status until P84-01 T03: FAIL (stub). T03 replaces with the full
   # ~80-line two-sub-fixture harness.
   set -euo pipefail
   echo "FAIL: T03 not yet shipped (first-run handling harness)"
   exit 1
   ```

   `quality/gates/agent-ux/webhook-backends-without-webhooks.sh`
   (~40 lines — grep-shape + trim simulation):

   ```bash
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

   # Simulate the trim: produce a copy with the `repository_dispatch:` line
   # AND the next line (`types: [reposix-mirror-sync]`) removed; assert the
   # result still parses as YAML and still has at least one trigger
   # (schedule + workflow_dispatch).
   TRIMMED=$(mktemp); trap "rm -f $TRIMMED" EXIT
   python3 - <<'PYEOF' "$TEMPLATE" "$TRIMMED"
   import sys, yaml
   src, dst = sys.argv[1], sys.argv[2]
   doc = yaml.safe_load(open(src))
   if 'on' in doc and 'repository_dispatch' in doc['on']:
       del doc['on']['repository_dispatch']
   # Sanity: at least one trigger remains.
   assert doc['on'].get('schedule') or doc['on'].get('workflow_dispatch'), \
       "after trim, no triggers remain — workflow would never run"
   yaml.safe_dump(doc, open(dst, 'w'))
   PYEOF

   python3 -c "import yaml,sys; yaml.safe_load(open(sys.argv[1]))" "$TRIMMED" \
     || { echo "FAIL: trimmed YAML does not parse"; exit 1; }

   echo "PASS: cron-only mode preserved when repository_dispatch removed"
   exit 0
   ```

   `quality/gates/agent-ux/webhook-latency-floor.sh` (~25 lines —
   asset-exists + JSON p95 check; T01 mints, T05 closes by landing
   the artifact):

   ```bash
   #!/usr/bin/env bash
   # CATALOG ROW: agent-ux/webhook-latency-floor
   # CADENCE: pre-release
   # INVARIANT: quality/reports/verifications/perf/webhook-latency.json
   #            exists, parses, has p95_seconds <= 120 (falsifiable
   #            threshold per ROADMAP P84 SC4).
   #
   # Status until P84-01 T05: FAIL (artifact does not exist yet).
   set -euo pipefail
   SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
   REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
   cd "${REPO_ROOT}"

   ARTIFACT="quality/reports/verifications/perf/webhook-latency.json"
   test -f "$ARTIFACT" \
     || { echo "FAIL: $ARTIFACT does not exist"; exit 1; }
   P95=$(python3 -c "import json,sys; print(json.load(open(sys.argv[1]))['p95_seconds'])" "$ARTIFACT" 2>/dev/null) \
     || { echo "FAIL: $ARTIFACT does not parse or lacks p95_seconds field"; exit 1; }
   [ "$P95" -le 120 ] \
     || { echo "FAIL: p95_seconds=$P95 > 120s threshold (ROADMAP P84 SC4)"; exit 1; }

   echo "PASS: $ARTIFACT p95=${P95}s within 120s threshold"
   exit 0
   ```

   Authored AT this task; FAILs at this task; PASSes at T06 once
   T02–T05 land their substrates.

2. **Hand-edit `quality/catalogs/agent-ux.json`.** Add 6 new rows
   to the existing `rows` array. Each row shape mirrors P82's
   `agent-ux/bus-fetch-not-advertised` (or P81's
   `agent-ux/sync-reconcile-subcommand`) verbatim. Example for
   row 1; the other 5 follow the same template (id / sources /
   command / expected / verifier / artifact paths swapped):

   ```json
   {
     "id": "agent-ux/webhook-trigger-dispatch",
     "dimension": "agent-ux",
     "cadence": "pre-pr",
     "kind": "mechanical",
     "_provenance_note": "Hand-edit per documented gap (NOT Principle A): reposix-quality bind only supports the docs-alignment dimension at v0.13.0; agent-ux dim mints stay hand-edited until GOOD-TO-HAVES-01 ships the verb extension. See .planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md § GOOD-TO-HAVES-01.",
     "sources": [
       "docs/guides/dvcs-mirror-setup-template.yml",
       "reubenjohn/reposix-tokenworld-mirror:.github/workflows/reposix-mirror-sync.yml (live)",
       ".planning/research/v0.13.0-dvcs/architecture-sketch.md (§ Webhook-driven mirror sync — verbatim YAML skeleton)",
       ".planning/research/v0.13.0-dvcs/decisions.md (Q4.1/Q4.2/Q4.3)",
       ".planning/REQUIREMENTS.md (DVCS-WEBHOOK-01)"
     ],
     "command": "bash quality/gates/agent-ux/webhook-trigger-dispatch.sh",
     "expected": {
       "asserts": [
         "bash quality/gates/agent-ux/webhook-trigger-dispatch.sh exits 0",
         "docs/guides/dvcs-mirror-setup-template.yml exists and parses as YAML",
         "gh api repos/reubenjohn/reposix-tokenworld-mirror/contents/.github/workflows/reposix-mirror-sync.yml returns 200 (live copy reachable)",
         "diff -w of the two copies returns zero (byte-equal modulo whitespace)",
         "YAML contains: repository_dispatch types=[reposix-mirror-sync], cargo binstall reposix-cli (NOT bare 'reposix'), NO github.event.client_payload references, NO set -x"
       ]
     },
     "verifier": {
       "script": "quality/gates/agent-ux/webhook-trigger-dispatch.sh",
       "args": [],
       "timeout_s": 60,
       "container": null
     },
     "artifact": "quality/reports/verifications/agent-ux/webhook-trigger-dispatch.json",
     "status": "FAIL",
     "last_verified": null,
     "freshness_ttl": null,
     "blast_radius": "P1",
     "owner_hint": "if RED: workflow YAML drifted between template and live copy; or one copy missing; or YAML structure regressed (binstall name, client_payload reference, missing fetch-depth); inspect verifier output for which structural grep failed",
     "waiver": null
   }
   ```

   The other 5 rows follow this template; specific deltas:

   - **Row 2** `agent-ux/webhook-cron-fallback` — sources cite the
     template YAML + ROADMAP P84 SC1 + Q4.1 (cron literal). asserts
     cite literal cron, `fetch-depth: 0`, `cancel-in-progress: false`.
     command: `bash quality/gates/agent-ux/webhook-cron-fallback.sh`.
     artifact:
     `quality/reports/verifications/agent-ux/webhook-cron-fallback.json`.
   - **Row 3** `agent-ux/webhook-force-with-lease-race` — sources
     cite the template YAML's push step + RESEARCH.md § "`--force-with-lease`
     Semantics" + DVCS-WEBHOOK-02. asserts: race walk-through fixture
     produces lease-rejection; mirror state untouched after rejection.
     command: `bash quality/gates/agent-ux/webhook-force-with-lease-race.sh`.
   - **Row 4** `agent-ux/webhook-first-run-empty-mirror` — sources
     cite the template YAML's first-run branch + Q4.3 + DVCS-WEBHOOK-03.
     asserts: BOTH 4.3.a fresh-but-readme AND 4.3.b truly-empty
     sub-cases pass; lease push fires for 4.3.a, plain push for
     4.3.b.
   - **Row 5** `agent-ux/webhook-backends-without-webhooks` — sources
     cite the template YAML + Q4.2 + ROADMAP P84 SC5. asserts: trim
     of `repository_dispatch:` block produces still-valid YAML with
     at least one trigger remaining (schedule or workflow_dispatch).
   - **Row 6** `agent-ux/webhook-latency-floor` — `cadence:
     "pre-release"` (D-02; NOT pre-pr); `kind: "asset-exists"`.
     sources cite the latency JSON + RESEARCH.md § "Latency
     Measurement Strategy" + ROADMAP P84 SC4 + DVCS-WEBHOOK-04.
     asserts: `quality/reports/verifications/perf/webhook-latency.json`
     exists, parses, `p95_seconds ≤ 120`.

3. **Validate JSON parses + the rows are addressable:**

   ```bash
   python3 -c '
   import json
   data = json.load(open("quality/catalogs/agent-ux.json"))
   row_ids = {r["id"] for r in data["rows"]}
   for required in [
       "agent-ux/webhook-trigger-dispatch",
       "agent-ux/webhook-cron-fallback",
       "agent-ux/webhook-force-with-lease-race",
       "agent-ux/webhook-first-run-empty-mirror",
       "agent-ux/webhook-backends-without-webhooks",
       "agent-ux/webhook-latency-floor",
   ]:
       assert required in row_ids, f"missing row: {required}"
   print("all 6 P84 rows present in agent-ux.json")
   '
   ```

4. **chmod + atomic catalog-first commit:**

   ```bash
   chmod +x quality/gates/agent-ux/webhook-trigger-dispatch.sh \
            quality/gates/agent-ux/webhook-cron-fallback.sh \
            quality/gates/agent-ux/webhook-force-with-lease-race.sh \
            quality/gates/agent-ux/webhook-first-run-empty-mirror.sh \
            quality/gates/agent-ux/webhook-backends-without-webhooks.sh \
            quality/gates/agent-ux/webhook-latency-floor.sh
   git add quality/gates/agent-ux/webhook-trigger-dispatch.sh \
           quality/gates/agent-ux/webhook-cron-fallback.sh \
           quality/gates/agent-ux/webhook-force-with-lease-race.sh \
           quality/gates/agent-ux/webhook-first-run-empty-mirror.sh \
           quality/gates/agent-ux/webhook-backends-without-webhooks.sh \
           quality/gates/agent-ux/webhook-latency-floor.sh \
           quality/catalogs/agent-ux.json
   git commit -m "$(cat <<'EOF'
quality(agent-ux): mint 6 webhook-mirror-sync catalog rows + 6 TINY verifiers (DVCS-WEBHOOK-01..04 catalog-first)

- quality/gates/agent-ux/webhook-trigger-dispatch.sh — TINY ~70-line verifier (asserts both YAML copies + structural grep)
- quality/gates/agent-ux/webhook-cron-fallback.sh — TINY ~30-line verifier (literal cron + fetch-depth + concurrency)
- quality/gates/agent-ux/webhook-force-with-lease-race.sh — T01 stub; T04 replaces with full ~50-line race harness
- quality/gates/agent-ux/webhook-first-run-empty-mirror.sh — T01 stub; T03 replaces with full ~80-line two-sub-fixture harness
- quality/gates/agent-ux/webhook-backends-without-webhooks.sh — TINY ~40-line verifier (trim simulation + YAML re-parse)
- quality/gates/agent-ux/webhook-latency-floor.sh — TINY ~25-line asset-exists + p95 floor check (T05 lands the JSON)
- quality/catalogs/agent-ux.json — 6 rows added (status FAIL initial); flips to PASS at T06 BEFORE per-phase push

Hand-edit per documented gap (NOT Principle A): reposix-quality bind supports docs-alignment dim only. agent-ux dim mints stay hand-edited until GOOD-TO-HAVES-01 ships the verb extension.

Phase 84 / Plan 01 / Task 01 / DVCS-WEBHOOK-01 + DVCS-WEBHOOK-02 + DVCS-WEBHOOK-03 + DVCS-WEBHOOK-04 (catalog-first).
EOF
)"
   ```

   NO push yet — the per-phase push is the terminal task of this
   plan (T06), not of T01. The mirror-repo push happens in T02 as a
   separate operation.
</action>

<verify>
  <automated>python3 -c 'import json; ids = {r["id"] for r in json.load(open("quality/catalogs/agent-ux.json"))["rows"]}; required = ["agent-ux/webhook-trigger-dispatch","agent-ux/webhook-cron-fallback","agent-ux/webhook-force-with-lease-race","agent-ux/webhook-first-run-empty-mirror","agent-ux/webhook-backends-without-webhooks","agent-ux/webhook-latency-floor"]; missing = [i for i in required if i not in ids]; assert not missing, f"missing rows: {missing}"' && for f in webhook-trigger-dispatch webhook-cron-fallback webhook-force-with-lease-race webhook-first-run-empty-mirror webhook-backends-without-webhooks webhook-latency-floor; do test -x "quality/gates/agent-ux/${f}.sh" || { echo "missing executable: ${f}.sh"; exit 1; }; done</automated>
</verify>

<done>
- 6 verifier shells exist under `quality/gates/agent-ux/`, each
  executable, mirroring P81/P82 TINY shape.
- Running each verifier in isolation (without T02–T05 yet) FAILS
  cleanly: each exits non-zero with a diagnostic naming the missing
  artifact (template YAML, harness body, JSON file).
- `quality/catalogs/agent-ux.json` has 6 new rows; each row's
  `status` is `FAIL`; `verifier.script` ends in `.sh`; required
  fields per `quality/catalogs/README.md` schema are present.
- `python3 -c 'import json; json.load(open("quality/catalogs/agent-ux.json"))'`
  exits 0 (JSON parses).
- Each row's `_provenance_note` annotates "Hand-edit per documented
  gap (NOT Principle A)" and references GOOD-TO-HAVES-01.
- Commit message annotates the same.
- `git log -1 --oneline` shows the catalog-first commit.
- `git diff --stat HEAD~1` shows 7 files: 6 new .sh + 1 catalog edit.
</done>

---

