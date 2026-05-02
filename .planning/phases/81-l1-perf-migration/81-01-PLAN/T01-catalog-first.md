← [back to index](./index.md) · phase 81 plan 01

## Task 81-01-T01 — Catalog-first: mint 3 catalog rows + author 2 verifier shells

<read_first>
- `quality/catalogs/perf-targets.json` (full file — 3 existing WAIVED
  rows; new 1 row joins the same `rows` array; row shape mirrors the
  existing `perf/latency-bench` row).
- `quality/catalogs/agent-ux.json` (full file — 5 existing rows; new
  1 row joins; shape mirrors P80's
  `agent-ux/mirror-refs-write-on-success` row).
- `quality/catalogs/doc-alignment.json` (large file — read selectively
  via `grep` for an existing `BOUND` row's shape; the new row mints
  via the bind verb, NOT a hand-edit).
- `quality/gates/agent-ux/mirror-refs-write-on-success.sh` (TINY-shape
  precedent; same shape: `cargo test -p <crate> --test <name>`,
  exit 0).
- `quality/catalogs/README.md` § "Unified schema" — required fields per row.
- `quality/PROTOCOL.md` § "Principle A — Subagents propose; tools
  validate and mint" — confirms the gap (Principle A applies to docs-
  alignment dim only; perf + agent-ux dim have no `bind` verb yet).
- `.planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md` § GOOD-TO-HAVES-01
  — the documented-gap framing the perf + agent-ux row annotations cite.
- `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § "Performance
  subtlety: today's `list_records` walk on every push" — the prose
  paragraph the doc-alignment row binds (specifically: "L1 trades one
  safety property: today's `list_records` would catch a record that
  exists on backend but missing from cache" — line 164).
</read_first>

<action>
This is the **catalog-first commit** for P81: the rows defining the
GREEN contract land BEFORE T02–T04 implementation. Two verifier shell
scripts ship in this same atomic commit (referenced by the perf +
agent-ux rows). The doc-alignment row is BOUND via the `reposix-quality
doc-alignment bind` verb — Principle A applies.

**Important — perf + agent-ux rows are NOT a Principle A application.**
The `reposix-quality bind` verb supports the `docs-alignment` dimension
only. The `perf` and `agent-ux` dims are hand-edited per the existing
P80 precedent. The orchestrator filed GOOD-TO-HAVES-01 during P79 to
track the bind-verb extension work for a future polish slot. The
catalog edits in this task for perf + agent-ux are therefore
**hand-edits per documented gap**, not Principle A applications. This
is annotated in the commit message AND in each row's `_provenance_note`
field (mirroring the P80 row's annotation).

Steps:

1. **Author the two verifier shells.** Each is TINY (~30-50 lines)
   mirroring `quality/gates/agent-ux/mirror-refs-write-on-success.sh`.
   The verifiers exercise the relevant scenario via cargo-test
   delegation; they will FAIL until T02 + T03 + T04 implement and test
   the wiring.

   `quality/gates/perf/list-call-count.sh`:

   ```bash
   #!/usr/bin/env bash
   # quality/gates/perf/list-call-count.sh — perf verifier for catalog
   # row `perf/handle-export-list-call-count`.
   #
   # CATALOG ROW: quality/catalogs/perf-targets.json -> perf/handle-export-list-call-count
   # CADENCE:     pre-pr (~30s wall time)
   # INVARIANT:   With N=200 records seeded in the wiremock harness and
   #              a one-record edit pushed via the export verb, the
   #              precheck makes >=1 list_changed_since REST call AND
   #              ZERO list_records REST calls. The positive-control
   #              sibling test confirms wiremock fails RED if the matcher
   #              were reverted (closes RESEARCH.md MEDIUM risk).
   #
   # Implementation: delegates to the integration test
   # `crates/reposix-remote/tests/perf_l1.rs::l1_precheck_uses_list_changed_since_not_list_records`
   # which drives `git-remote-reposix` directly via stdin against a
   # wiremock backend, and counts REST calls via wiremock matchers.
   #
   # Status until P81-01 T04: FAIL — wiring is scaffold-only in T01-T03;
   # the integration test + behavior coverage land in T04.
   #
   # Usage: bash quality/gates/perf/list-call-count.sh
   set -euo pipefail
   SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
   REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
   cd "${REPO_ROOT}"

   cargo test -p reposix-remote --test perf_l1 \
       l1_precheck_uses_list_changed_since_not_list_records \
       --quiet -- --nocapture 2>&1 | tail -20

   echo "PASS: L1 precheck makes >=1 list_changed_since calls AND zero list_records calls (N=200 wiremock harness)"
   exit 0
   ```

   `quality/gates/agent-ux/sync-reconcile-subcommand.sh`:

   ```bash
   #!/usr/bin/env bash
   # quality/gates/agent-ux/sync-reconcile-subcommand.sh — agent-ux
   # verifier for catalog row `agent-ux/sync-reconcile-subcommand`.
   #
   # CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/sync-reconcile-subcommand
   # CADENCE:     pre-pr (~30s wall time)
   # INVARIANT:   `reposix sync --reconcile --help` exits 0 AND the
   #              integration smoke test `reposix-cli/tests/sync.rs::sync_reconcile_advances_cursor`
   #              passes (cache last_fetched_at advances after running
   #              the subcommand against a sim).
   #
   # Status until P81-01 T04: FAIL.
   set -euo pipefail
   SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
   REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
   cd "${REPO_ROOT}"

   cargo build -p reposix-cli --quiet
   CLI_BIN="${REPO_ROOT}/target/debug/reposix"
   "${CLI_BIN}" sync --reconcile --help > /dev/null \
       || { echo "FAIL: reposix sync --reconcile --help nonzero exit" >&2; exit 1; }

   cargo test -p reposix-cli --test sync sync_reconcile_advances_cursor \
       --quiet -- --nocapture 2>&1 | tail -20

   echo "PASS: reposix sync --reconcile help+subcommand smoke green"
   exit 0
   ```

   Authored AT this task; FAILS at this task; PASSes at T04.

2. **Hand-edit `quality/catalogs/perf-targets.json`.** Add 1 new row to
   the existing `rows` array (currently 3 WAIVED rows). Row shape
   mirrors the existing `perf/latency-bench` shape verified at planning
   time:

   ```json
   {
     "id": "perf/handle-export-list-call-count",
     "dimension": "perf",
     "cadence": "pre-pr",
     "kind": "mechanical",
     "_provenance_note": "Hand-edit per documented gap (NOT Principle A): reposix-quality bind only supports the docs-alignment dimension at v0.13.0; perf dim mints stay hand-edited until GOOD-TO-HAVES-01 ships the verb extension. See .planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md § GOOD-TO-HAVES-01.",
     "sources": [
       "crates/reposix-remote/src/main.rs::handle_export",
       "crates/reposix-remote/src/precheck.rs::precheck_export_against_changed_set",
       "crates/reposix-remote/tests/perf_l1.rs::l1_precheck_uses_list_changed_since_not_list_records",
       ".planning/research/v0.13.0-dvcs/architecture-sketch.md (Performance subtlety section)",
       ".planning/REQUIREMENTS.md (DVCS-PERF-L1-01, DVCS-PERF-L1-03)"
     ],
     "command": "bash quality/gates/perf/list-call-count.sh",
     "expected": {
       "asserts": [
         "bash quality/gates/perf/list-call-count.sh exits 0 against the local cargo workspace",
         "with N=200 records seeded in wiremock and a one-record edit pushed, the helper makes >=1 list_changed_since REST call AND zero list_records REST calls (cursor-present hot path)",
         "the positive-control sibling test confirms wiremock fails RED if expect(0) on list_records is reverted"
       ]
     },
     "verifier": {
       "script": "quality/gates/perf/list-call-count.sh",
       "args": [],
       "timeout_s": 180,
       "container": null
     },
     "artifact": "quality/reports/verifications/perf/list-call-count.json",
     "status": "FAIL",
     "last_verified": null,
     "freshness_ttl": null,
     "blast_radius": "P1",
     "owner_hint": "if RED: crates/reposix-remote/src/precheck.rs regressed OR handle_export reverted to unconditional list_records walk; check stderr for whether wiremock saw a list_records call",
     "waiver": null
   }
   ```

3. **Hand-edit `quality/catalogs/agent-ux.json`.** Add 1 new row, shape
   verbatim from P80's `agent-ux/mirror-refs-write-on-success` row:

   ```json
   {
     "id": "agent-ux/sync-reconcile-subcommand",
     "dimension": "agent-ux",
     "cadence": "pre-pr",
     "kind": "mechanical",
     "_provenance_note": "Hand-edit per documented gap (NOT Principle A): reposix-quality bind only supports the docs-alignment dimension at v0.13.0; agent-ux dim mints stay hand-edited until GOOD-TO-HAVES-01 ships the verb extension. See .planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md § GOOD-TO-HAVES-01.",
     "sources": [
       "crates/reposix-cli/src/main.rs (Sync subcommand)",
       "crates/reposix-cli/src/sync.rs",
       "crates/reposix-cli/tests/sync.rs::sync_reconcile_advances_cursor",
       ".planning/REQUIREMENTS.md (DVCS-PERF-L1-02)"
     ],
     "command": "bash quality/gates/agent-ux/sync-reconcile-subcommand.sh",
     "expected": {
       "asserts": [
         "bash quality/gates/agent-ux/sync-reconcile-subcommand.sh exits 0 against the local cargo workspace",
         "reposix sync --reconcile --help exits 0 (clap-derive surface present)",
         "the smoke test sync_reconcile_advances_cursor passes (last_fetched_at advances after sync --reconcile)"
       ]
     },
     "verifier": {
       "script": "quality/gates/agent-ux/sync-reconcile-subcommand.sh",
       "args": [],
       "timeout_s": 180,
       "container": null
     },
     "artifact": "quality/reports/verifications/agent-ux/sync-reconcile-subcommand.json",
     "status": "FAIL",
     "last_verified": null,
     "freshness_ttl": null,
     "blast_radius": "P1",
     "owner_hint": "if RED: crates/reposix-cli/src/sync.rs regressed OR clap subcommand surface broke; check stderr for whether the --help path or the smoke test failed",
     "waiver": null
   }
   ```

4. **Bind the docs-alignment row via the `reposix-quality` verb.**
   The architecture-sketch's Performance-subtlety paragraph is the
   prose source; the new test is the verifier. Use the bind verb:

   ```bash
   reposix-quality doc-alignment bind \
       --doc .planning/research/v0.13.0-dvcs/architecture-sketch.md \
       --line 164 \
       --claim "L1 trades one safety property: today's list_records would catch a record that exists on backend but missing from cache" \
       --verifier "crates/reposix-remote/tests/perf_l1.rs::l1_precheck_uses_list_changed_since_not_list_records" \
       --row-id "docs-alignment/perf-subtlety-prose-bound"
   ```

   The exact CLI flag names MUST be confirmed during T01 read_first via
   `reposix-quality doc-alignment bind --help`; the bind verb's
   contract is "subagent proposes; tool mints" per Principle A. If the
   bind verb signature differs (e.g., requires `--source-line` instead
   of `--line`), use the actual flag names and document the divergence
   in the commit message body.

   **NOTE.** The architecture-sketch prose paragraph is at line 164 of
   `.planning/research/v0.13.0-dvcs/architecture-sketch.md` per
   RESEARCH.md § Catalog Row Design. T01 confirms this line still
   contains the cited claim verbatim (P80 / earlier work may have
   shifted line numbers; if so, re-locate the paragraph and pass the
   actual line number to `bind`).

5. **Validate JSON parses + the rows are addressable + the bound row
   is `BOUND` (not `MISSING_TEST`):**

   ```bash
   python3 -c '
   import json
   for f, ids in [
       ("quality/catalogs/perf-targets.json", ["perf/handle-export-list-call-count"]),
       ("quality/catalogs/agent-ux.json", ["agent-ux/sync-reconcile-subcommand"]),
   ]:
       data = json.load(open(f))
       rows = data["rows"]
       row_ids = {r["id"] for r in rows}
       for required in ids:
           assert required in row_ids, f"missing row in {f}: {required}"
   print("perf + agent-ux rows present")
   '
   reposix-quality doc-alignment status --row-id docs-alignment/perf-subtlety-prose-bound
   # expected output contains: status=BOUND
   ```

6. **chmod + atomic catalog-first commit:**

   ```bash
   chmod +x quality/gates/perf/list-call-count.sh \
            quality/gates/agent-ux/sync-reconcile-subcommand.sh
   git add quality/gates/perf/list-call-count.sh \
           quality/gates/agent-ux/sync-reconcile-subcommand.sh \
           quality/catalogs/perf-targets.json \
           quality/catalogs/agent-ux.json \
           quality/catalogs/doc-alignment.json
   git commit -m "$(cat <<'EOF'
   quality(perf,agent-ux,docs-alignment): mint L1-perf catalog rows + 2 TINY verifiers (DVCS-PERF-L1-01..03 catalog-first)

   - quality/gates/perf/list-call-count.sh — TINY 30-line verifier delegating to cargo test
   - quality/gates/agent-ux/sync-reconcile-subcommand.sh — TINY 30-line verifier delegating to cargo test + --help check
   - quality/catalogs/perf-targets.json — 1 row added: perf/handle-export-list-call-count (FAIL initial)
   - quality/catalogs/agent-ux.json — 1 row added: agent-ux/sync-reconcile-subcommand (FAIL initial)
   - quality/catalogs/doc-alignment.json — 1 row BOUND via reposix-quality doc-alignment bind: docs-alignment/perf-subtlety-prose-bound

   Initial status: FAIL for perf + agent-ux (verifiers exist; implementation lands in T02-T04). docs-alignment row is BOUND immediately (the bind verb mints with the verifier-test pair already coupled).
   Runner re-grades to PASS at T04 BEFORE per-phase push.

   Hand-edit per documented gap (NOT Principle A): reposix-quality bind supports docs-alignment dim only. perf + agent-ux dim mints are hand-edited until GOOD-TO-HAVES-01 ships the verb extension. See .planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md § GOOD-TO-HAVES-01.

   Phase 81 / Plan 01 / Task 01 / DVCS-PERF-L1-01..03 (catalog-first).
   EOF
   )"
   ```

   NO push yet — the per-phase push is the terminal task of this plan
   (T04), not of T01.
</action>

<verify>
  <automated>python3 -c 'import json; assert "perf/handle-export-list-call-count" in {r["id"] for r in json.load(open("quality/catalogs/perf-targets.json"))["rows"]}, "perf row missing"; assert "agent-ux/sync-reconcile-subcommand" in {r["id"] for r in json.load(open("quality/catalogs/agent-ux.json"))["rows"]}, "agent-ux row missing"' && test -x quality/gates/perf/list-call-count.sh && test -x quality/gates/agent-ux/sync-reconcile-subcommand.sh</automated>
</verify>

<done>
- 2 verifier shells exist under `quality/gates/perf/` and
  `quality/gates/agent-ux/`, each executable, ~30-50 lines, mirroring
  P80's TINY shape (delegate to `cargo test`, exit 0).
- Running each verifier in isolation (without T02-T04 yet) fails
  cleanly: `bash <verifier>; rc=$?; [[ $rc != 0 ]]` succeeds — the
  rows' initial FAIL status reflects reality.
- `quality/catalogs/perf-targets.json` has 1 new row; `agent-ux.json`
  has 1 new row; each row's `status` is `FAIL`; `verifier.script` ends
  in `.sh`; required fields per `quality/catalogs/README.md` schema
  are present.
- `quality/catalogs/doc-alignment.json` has 1 new row with `status:
  BOUND` minted via `reposix-quality doc-alignment bind`.
- `python3 -c 'import json; json.load(open("quality/catalogs/perf-targets.json")); json.load(open("quality/catalogs/agent-ux.json")); json.load(open("quality/catalogs/doc-alignment.json"))'`
  exits 0 (JSON parses).
- Each hand-edited row's `_provenance_note` annotates "Hand-edit per
  documented gap (NOT Principle A)" and references GOOD-TO-HAVES-01.
- Commit message annotates the same.
- `git log -1 --oneline` shows the catalog-first commit.
- `git diff --stat HEAD~1` shows ≤ 5 files: 2 new .sh + 3 catalog edits.
</done>

---

