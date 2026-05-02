← [back to index](./index.md)

# Task 83-02-T01 — Catalog-first: mint 4 catalog rows + author 4 verifier shells

<read_first>
- `quality/catalogs/agent-ux.json` (full file — 12 P82 rows + 4
  P83-01 rows pre-existing; the 4 new P83-02 rows join the same
  `rows` array; row shape mirrors P83-01 T01's
  `agent-ux/bus-write-sot-first-success` row verbatim).
- `quality/gates/agent-ux/bus-write-sot-first-success.sh` (P83-01
  TINY-shape precedent; same shape: `cargo test -p reposix-remote
  --test <name>`, exit 0).
- `quality/catalogs/README.md` § "Unified schema" — required
  fields per row.
- `quality/PROTOCOL.md` § "Principle A" — confirms the gap
  (Principle A docs-alignment dim only; agent-ux dim hand-edited).
- `.planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md` §
  GOOD-TO-HAVES-01 — the documented-gap framing.
</read_first>

<action>
This is the **catalog-first commit** for P83-02: rows defining the
fault-injection + audit-completeness GREEN contract land BEFORE
T02–T04 implementation. FOUR verifier shell scripts ship in this
same atomic commit.

**Important — agent-ux rows are NOT a Principle A application.**
Hand-edits per documented gap (NOT Principle A) — same as P79/P80/P81/P82
+ P83-01 T01 precedent. Annotated in commit message + each row's
`_provenance_note`.

Steps:

1. **Author the four verifier shells.** Each is TINY (~30-50
   lines) mirroring `quality/gates/agent-ux/bus-write-sot-first-success.sh`.

   `quality/gates/agent-ux/bus-write-fault-injection-mirror-fail.sh`:

   ```bash
   #!/usr/bin/env bash
   # CATALOG ROW: agent-ux/bus-write-fault-injection-mirror-fail
   # CADENCE:     pre-pr (~10s wall time)
   # INVARIANT:   Fault scenario (a) — mirror push fails between
   #              confluence-write and ack. Helper exits zero with
   #              `ok refs/heads/main` (Q3.6 SoT contract); audit
   #              op helper_push_partial_fail_mirror_lag written;
   #              refs/mirrors/<sot>-head advances; synced-at frozen;
   #              mirror baseline preserved (failing-update-hook
   #              rejects the push).
   #
   # Status until P83-02 T04: FAIL.
   set -euo pipefail
   SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
   REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
   cd "${REPO_ROOT}"

   cargo test -p reposix-remote --test bus_write_mirror_fail \
       bus_write_mirror_fail_returns_ok_with_lag_audit_row \
       --quiet -- --nocapture 2>&1 | tail -20

   echo "PASS: fault-injection (a) mirror-fail produces correct end-state"
   exit 0
   ```

   `quality/gates/agent-ux/bus-write-fault-injection-sot-mid-stream.sh`:

   ```bash
   #!/usr/bin/env bash
   # CATALOG ROW: agent-ux/bus-write-fault-injection-sot-mid-stream
   # CADENCE:     pre-pr (~10s wall time)
   # INVARIANT:   Fault scenario (b) — confluence write fails
   #              mid-stream (5xx on second PATCH). Helper exits
   #              non-zero with `error refs/heads/main some-actions-failed`;
   #              NO mirror push attempted; mirror baseline preserved;
   #              NO helper_push_accepted row; NO
   #              helper_push_partial_fail_mirror_lag row;
   #              wiremock saw 2 PATCH requests (id=1 succeeded,
   #              id=2 returned 500).
   #
   # Status until P83-02 T04: FAIL.
   set -euo pipefail
   SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
   REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
   cd "${REPO_ROOT}"

   cargo test -p reposix-remote --test bus_write_sot_fail \
       bus_write_sot_mid_stream_fail_no_mirror_push_no_lag_audit \
       --quiet -- --nocapture 2>&1 | tail -20

   echo "PASS: fault-injection (b) SoT-mid-stream produces correct end-state"
   exit 0
   ```

   `quality/gates/agent-ux/bus-write-fault-injection-post-precheck-409.sh`:

   ```bash
   #!/usr/bin/env bash
   # CATALOG ROW: agent-ux/bus-write-fault-injection-post-precheck-409
   # CADENCE:     pre-pr (~10s wall time)
   # INVARIANT:   Fault scenario (c) — confluence 409 after
   #              PRECHECK B passed. Helper exits non-zero;
   #              NO mirror push; error names the failing record
   #              id (D-09 / Pitfall 3 documented behavior); NO
   #              helper_push_accepted row; mirror baseline
   #              preserved.
   #
   # Status until P83-02 T04: FAIL.
   set -euo pipefail
   SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
   REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
   cd "${REPO_ROOT}"

   cargo test -p reposix-remote --test bus_write_post_precheck_409 \
       bus_write_post_precheck_conflict_409_no_mirror_push \
       --quiet -- --nocapture 2>&1 | tail -20

   echo "PASS: fault-injection (c) post-precheck-409 produces correct end-state"
   exit 0
   ```

   `quality/gates/agent-ux/bus-write-audit-completeness.sh`:

   ```bash
   #!/usr/bin/env bash
   # CATALOG ROW: agent-ux/bus-write-audit-completeness
   # CADENCE:     pre-pr (~10s wall time)
   # INVARIANT:   Happy-path bus push writes expected rows to BOTH
   #              audit tables per OP-3:
   #              audit_events_cache: helper_push_started +
   #                helper_push_accepted + mirror_sync_written +
   #                helper_backend_instantiated;
   #              audit_events (sim DB): one row per executed REST
   #                mutation (create_record / update_record /
   #                delete_or_close).
   #
   # Status until P83-02 T04: FAIL.
   set -euo pipefail
   SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
   REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
   cd "${REPO_ROOT}"

   cargo test -p reposix-remote --test bus_write_audit_completeness \
       bus_write_audit_completeness_happy_path_writes_both_tables \
       --quiet -- --nocapture 2>&1 | tail -20

   echo "PASS: audit-completeness happy-path writes both tables"
   exit 0
   ```

2. **Hand-edit `quality/catalogs/agent-ux.json`.** Add 4 new rows
   following the P83-01 T01 row template. Specific deltas:

   - **Row 5** `agent-ux/bus-write-fault-injection-mirror-fail` —
     sources cite
     `crates/reposix-remote/src/bus_handler.rs::push_mirror` +
     `crates/reposix-remote/src/bus_handler.rs::handle_bus_export`
     partial-fail branch + `tests/bus_write_mirror_fail.rs::bus_write_mirror_fail_returns_ok_with_lag_audit_row`
     + DVCS-BUS-WRITE-06 + Q3.6.
     asserts: helper exits zero with `ok refs/heads/main` AND
     `audit_events_cache::helper_push_partial_fail_mirror_lag`
     count == 1 AND `audit_events_cache::mirror_sync_written` count
     == 0 AND `refs/mirrors/<sot>-head` advanced AND
     `refs/mirrors/<sot>-synced-at` UNCHANGED AND mirror's `main`
     ref UNCHANGED (rejected by failing hook). `owner_hint`: "if
     RED: bus_handler.rs partial-fail branch regressed OR
     `make_failing_mirror_fixture` (P83-01 T05) returned a passing
     mirror".
   - **Row 6** `agent-ux/bus-write-fault-injection-sot-mid-stream` —
     sources cite `bus_handler.rs::handle_bus_export` apply_writes
     SotPartialFail branch + `tests/bus_write_sot_fail.rs::bus_write_sot_mid_stream_fail_no_mirror_push_no_lag_audit`
     + DVCS-BUS-WRITE-06 + D-09. asserts: helper exits non-zero
     with `error refs/heads/main some-actions-failed` AND
     `audit_events_cache::helper_push_accepted` count == 0 AND
     `helper_push_partial_fail_mirror_lag` count == 0 AND mirror
     baseline preserved AND wiremock saw exactly 2 PATCH requests
     (id=1 returned 200, id=2 returned 500). `owner_hint`: "if
     RED: SoT partial-fail branch regressed OR mirror push was
     attempted despite SoT fail; check execute_action loop in
     write_loop::apply_writes".
   - **Row 7** `agent-ux/bus-write-fault-injection-post-precheck-409` —
     sources cite `bus_handler.rs` + `apply_writes` execute_action
     loop + `tests/bus_write_post_precheck_409.rs::bus_write_post_precheck_conflict_409_no_mirror_push`
     + DVCS-BUS-WRITE-06 + Q3.1 (PRECHECK B Stable). asserts:
     helper exits non-zero AND error stderr contains the failing
     record id AND NO mirror push AND `helper_push_accepted` count
     == 0 AND mirror baseline preserved AND wiremock saw exactly
     1 PATCH (the one that 409'd) AND exactly 1 list_changed_since
     (PRECHECK B Stable). `owner_hint`: "if RED: post-precheck
     409 fault not surfaced cleanly; check apply_writes
     execute_action loop bails on first error".
   - **Row 8** `agent-ux/bus-write-audit-completeness` — sources
     cite `crates/reposix-cache/src/audit.rs` (the OP-3 dual-table
     contract) + `crates/reposix-core/src/audit.rs` (the
     `audit_events` table) + `tests/bus_write_audit_completeness.rs::bus_write_audit_completeness_happy_path_writes_both_tables`
     + DVCS-BUS-WRITE-06 + OP-3. asserts: happy-path bus push
     writes ALL expected rows to BOTH tables per RESEARCH.md
     § "Audit Completeness Contract"; row counts match the
     contract table. `owner_hint`: "if RED: a per-record audit
     row was dropped (check sim adapter's create_record /
     update_record / delete_or_close success path) OR a cache
     audit op was missed (check apply_writes + bus_handler ref/audit
     write blocks)".

3. **Validate JSON parses + the rows are addressable:**

   ```bash
   python3 -c '
   import json
   data = json.load(open("quality/catalogs/agent-ux.json"))
   row_ids = {r["id"] for r in data["rows"]}
   for required in [
       "agent-ux/bus-write-fault-injection-mirror-fail",
       "agent-ux/bus-write-fault-injection-sot-mid-stream",
       "agent-ux/bus-write-fault-injection-post-precheck-409",
       "agent-ux/bus-write-audit-completeness",
   ]:
       assert required in row_ids, f"missing row: {required}"
   print("all 4 P83-02 rows present in agent-ux.json")
   '
   ```

4. **chmod + atomic catalog-first commit:**

   ```bash
   chmod +x quality/gates/agent-ux/bus-write-fault-injection-mirror-fail.sh \
            quality/gates/agent-ux/bus-write-fault-injection-sot-mid-stream.sh \
            quality/gates/agent-ux/bus-write-fault-injection-post-precheck-409.sh \
            quality/gates/agent-ux/bus-write-audit-completeness.sh
   git add quality/gates/agent-ux/bus-write-fault-injection-mirror-fail.sh \
           quality/gates/agent-ux/bus-write-fault-injection-sot-mid-stream.sh \
           quality/gates/agent-ux/bus-write-fault-injection-post-precheck-409.sh \
           quality/gates/agent-ux/bus-write-audit-completeness.sh \
           quality/catalogs/agent-ux.json
   git commit -m "quality(agent-ux): mint 4 bus-write fault-injection + audit-completeness catalog rows + 4 TINY verifiers (DVCS-BUS-WRITE-06 catalog-first)

- quality/gates/agent-ux/bus-write-fault-injection-mirror-fail.sh — TINY ~30-line cargo-test verifier (row 5)
- quality/gates/agent-ux/bus-write-fault-injection-sot-mid-stream.sh — TINY ~30-line cargo-test verifier (row 6)
- quality/gates/agent-ux/bus-write-fault-injection-post-precheck-409.sh — TINY ~30-line cargo-test verifier (row 7)
- quality/gates/agent-ux/bus-write-audit-completeness.sh — TINY ~30-line cargo-test verifier (row 8)
- quality/catalogs/agent-ux.json — 4 P83-02 rows added (status FAIL initial); flip to PASS at P83-02 T04 BEFORE per-phase push

Hand-edit per documented gap (NOT Principle A): same shape as P83-01 T01 + P82 T01.

Phase 83 / Plan 02 / Task 01 / DVCS-BUS-WRITE-06 (catalog-first)."
   ```

   NO push — the per-phase push is the terminal task of this plan
   (T04), not of T01.
</action>

<verify>
  <automated>python3 -c 'import json; ids = {r["id"] for r in json.load(open("quality/catalogs/agent-ux.json"))["rows"]}; required = ["agent-ux/bus-write-fault-injection-mirror-fail","agent-ux/bus-write-fault-injection-sot-mid-stream","agent-ux/bus-write-fault-injection-post-precheck-409","agent-ux/bus-write-audit-completeness"]; missing = [i for i in required if i not in ids]; assert not missing, f"missing rows: {missing}"' && for f in bus-write-fault-injection-mirror-fail bus-write-fault-injection-sot-mid-stream bus-write-fault-injection-post-precheck-409 bus-write-audit-completeness; do test -x "quality/gates/agent-ux/${f}.sh" || { echo "missing executable: ${f}.sh"; exit 1; }; done</automated>
</verify>

<done>
- 4 verifier shells exist under `quality/gates/agent-ux/`, each
  executable, ~30-50 lines, mirroring P83-01 T01's TINY shape.
- Running each verifier in isolation (without T02–T03) fails
  cleanly: `bash <verifier>; rc=$?; [[ $rc != 0 ]]` succeeds.
- `quality/catalogs/agent-ux.json` has 4 new rows; each row's
  `status` is `FAIL`; required fields per
  `quality/catalogs/README.md` schema are present.
- `python3 -c 'import json; json.load(open("quality/catalogs/agent-ux.json"))'`
  exits 0.
- Each row's `_provenance_note` annotates "Hand-edit per
  documented gap (NOT Principle A)" and references
  GOOD-TO-HAVES-01.
- Single atomic commit; commit message annotates the same.
- NO push — T04 is terminal.
</done>
