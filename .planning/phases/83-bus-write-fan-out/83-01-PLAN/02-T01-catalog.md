← [back to index](./index.md) · phase 83 plan 01

## Task 83-01-T01 — Catalog-first: mint 4 catalog rows + author 4 verifier shells

<read_first>
- `quality/catalogs/agent-ux.json` (full file — 12 existing P82
  rows + the prior P79/P80/P81 rows; the 4 new P83-01 rows join the
  same `rows` array; row shape mirrors P82's
  `agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first` row
  verbatim).
- `quality/gates/agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first.sh`
  (P82 TINY-shape precedent; same shape: `cargo test -p
  reposix-remote --test <name>`, exit 0).
- `quality/gates/agent-ux/bus-fetch-not-advertised.sh` (P82 TINY
  precedent — sibling pattern).
- `quality/catalogs/README.md` § "Unified schema" — required
  fields per row.
- `quality/PROTOCOL.md` § "Principle A — Subagents propose; tools
  validate and mint" — confirms the gap (Principle A applies to
  docs-alignment dim only; agent-ux dim has no `bind` verb yet).
- `.planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md` §
  GOOD-TO-HAVES-01 — the documented-gap framing the agent-ux row
  annotations cite.
</read_first>

<action>
This is the **catalog-first commit** for P83-01: the rows defining
the GREEN contract land BEFORE T02–T06 implementation. FOUR
verifier shell scripts ship in this same atomic commit (one per
row).

**Important — agent-ux rows are NOT a Principle A application.**
The `reposix-quality bind` verb supports the `docs-alignment`
dimension only. The `agent-ux` dim is hand-edited per the existing
P79/P80/P81/P82 precedent. The orchestrator filed
GOOD-TO-HAVES-01 during P79 to track the bind-verb extension work
for a future polish slot. The catalog edits in this task are
therefore **hand-edits per documented gap**, not Principle A
applications. This is annotated in the commit message AND in each
row's `_provenance_note` field (mirroring P82's row annotations).

Steps:

1. **Author the four verifier shells.** Each is TINY (~30-50
   lines) mirroring `quality/gates/agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first.sh`.
   The verifiers exercise the relevant scenario via cargo-test
   delegation (3 of them) or via a static grep (`bus-write-no-helper-retry.sh`);
   they will FAIL until T02–T05 implement and T06 flips the
   catalog status. EXCEPT row 2 (`bus-write-mirror-fail-returns-ok`)
   which is exercised by P83-02's `bus_write_mirror_fail.rs` test
   — its verifier is authored here in P83-01 T01 (catalog-first
   contract land BEFORE implementation, where "implementation"
   includes BOTH P83-01 + P83-02). Status remains FAIL until P83-02
   T04's catalog flip.

   **NOTE on row 2:** P83-01 T06's catalog flip ONLY flips rows 1,
   3, 4 to PASS (the rows whose tests exist by T05 close). Row 2
   stays FAIL through P83-01 close; flips to PASS in P83-02 T04
   when `bus_write_mirror_fail.rs` lands. This is intentional and
   matches RESEARCH.md § "Catalog Row Design" — the 4 P83a-relevant
   rows are minted in P83-01's catalog-first commit; the 4 rows
   covering P83-02's contract are minted in P83-02's catalog-first
   commit. Row 2's contract straddles both plans (the
   `bus_write_mirror_fail.rs` test is a P83-02 deliverable, but
   the row binds to the audit-op + ref-update logic that P83-01
   ships).

   Authoritative resolution per orchestrator instruction: row 2
   ships FAIL in P83-01 T01; its sibling P83-02 row
   (`bus-write-fault-injection-mirror-fail`) covers the
   fault-injection assertion; row 2 itself is left for P83-02 T04
   to flip to PASS. Document this in T01's commit message.

   `quality/gates/agent-ux/bus-write-sot-first-success.sh`:

   ```bash
   #!/usr/bin/env bash
   # quality/gates/agent-ux/bus-write-sot-first-success.sh — agent-ux
   # verifier for catalog row `agent-ux/bus-write-sot-first-success`.
   #
   # CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/bus-write-sot-first-success
   # CADENCE:     pre-pr (~10s wall time)
   # INVARIANT:   bus_handler::handle_bus_export reads stdin via
   #              parse_export_stream, calls write_loop::apply_writes,
   #              writes refs/mirrors/<sot>-head AND refs/mirrors/<sot>-synced-at
   #              on SoT-success + mirror-success; audit_events_cache
   #              has helper_push_started + helper_push_accepted +
   #              mirror_sync_written rows; helper exits zero with
   #              `ok refs/heads/main` on stdout.
   #
   # Status until P83-01 T06: FAIL.
   set -euo pipefail
   SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
   REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
   cd "${REPO_ROOT}"

   cargo test -p reposix-remote --test bus_write_happy \
       happy_path_writes_both_refs_and_acks_ok \
       --quiet -- --nocapture 2>&1 | tail -20

   echo "PASS: bus write SoT-first-success path writes both refs + dual audit table"
   exit 0
   ```

   `quality/gates/agent-ux/bus-write-mirror-fail-returns-ok.sh`:

   ```bash
   #!/usr/bin/env bash
   # CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/bus-write-mirror-fail-returns-ok
   # CADENCE:     pre-pr (~10s wall time)
   # INVARIANT:   On SoT-success + mirror-fail, helper writes
   #              helper_push_partial_fail_mirror_lag audit row,
   #              advances refs/mirrors/<sot>-head, leaves
   #              refs/mirrors/<sot>-synced-at FROZEN, emits stderr
   #              warn, returns ok refs/heads/main to git.
   #
   # Status until P83-02 T04: FAIL.
   # NOTE: This row is minted in P83-01 T01 (catalog-first contract);
   # its test ships in P83-02 T02. P83-01 T06 leaves row 2 FAIL;
   # P83-02 T04 flips to PASS.
   set -euo pipefail
   SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
   REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
   cd "${REPO_ROOT}"

   cargo test -p reposix-remote --test bus_write_mirror_fail \
       bus_write_mirror_fail_returns_ok_with_lag_audit_row \
       --quiet -- --nocapture 2>&1 | tail -20

   echo "PASS: bus write SoT-success+mirror-fail path returns ok with lag audit row"
   exit 0
   ```

   `quality/gates/agent-ux/bus-write-no-helper-retry.sh`:

   ```bash
   #!/usr/bin/env bash
   # CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/bus-write-no-helper-retry
   # CADENCE:     pre-pr (~2s wall time)
   # INVARIANT:   crates/reposix-remote/src/bus_handler.rs contains
   #              NO retry constructs adjacent to push_mirror calls
   #              (no `for _ in 0..` loops, no `loop {`, no
   #              `tokio::time::sleep`, no `--force-with-lease`,
   #              no `--force` in args).
   #
   # Status until P83-01 T06: FAIL (until T04 lands push_mirror
   # AND the grep confirms no retry constructs).
   #
   # Per Q3.6 RATIFIED 2026-04-30: surface, no helper-side retry.
   # User retries the whole push.
   set -euo pipefail
   SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
   REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
   cd "${REPO_ROOT}"

   FILE="crates/reposix-remote/src/bus_handler.rs"
   if [[ ! -f "${FILE}" ]]; then
       echo "FAIL: ${FILE} does not exist" >&2
       exit 1
   fi

   # Grep for retry-shaped patterns. Any hit fails RED.
   # Filter out comments first to avoid false positives on doc text.
   FILTERED=$(grep -v '^\s*//' "${FILE}" || true)

   if echo "${FILTERED}" | grep -qE 'for[[:space:]]+_[[:space:]]+in[[:space:]]+0\.\.'; then
       echo "FAIL: retry construct found (for _ in 0..) — Q3.6 RATIFIED no-retry violated" >&2
       exit 1
   fi
   if echo "${FILTERED}" | grep -qE '^\s*loop[[:space:]]*\{'; then
       echo "FAIL: bare loop construct found — Q3.6 RATIFIED no-retry violated" >&2
       exit 1
   fi
   if echo "${FILTERED}" | grep -qE 'tokio::time::sleep'; then
       echo "FAIL: tokio::time::sleep found — retry-via-sleep is Q3.6 violated" >&2
       exit 1
   fi
   if echo "${FILTERED}" | grep -qE -- '--force-with-lease|--force[^-]'; then
       echo "FAIL: --force / --force-with-lease found — D-08 RATIFIED plain push violated" >&2
       exit 1
   fi

   # Confirm push_mirror exists (T04 must land before T06 catalog flip).
   if ! grep -q 'fn push_mirror' "${FILE}"; then
       echo "FAIL: fn push_mirror not found in ${FILE} (T04 not yet shipped)" >&2
       exit 1
   fi

   echo "PASS: bus_handler.rs contains no retry constructs adjacent to push_mirror"
   exit 0
   ```

   `quality/gates/agent-ux/bus-write-no-mirror-remote-still-fails.sh`:

   ```bash
   #!/usr/bin/env bash
   # CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/bus-write-no-mirror-remote-still-fails
   # CADENCE:     pre-pr (~5s wall time)
   # INVARIANT:   bus URL with no local `git remote` for the mirror
   #              fails with verbatim Q3.5 hint after P83 lands;
   #              regression check that P83's write fan-out doesn't
   #              accidentally bypass P82's STEP 0 check.
   #
   # Status until P83-01 T06: FAIL.
   set -euo pipefail
   SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
   REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
   cd "${REPO_ROOT}"

   cargo test -p reposix-remote --test bus_write_no_mirror_remote \
       bus_write_no_mirror_remote_emits_q35_hint \
       --quiet -- --nocapture 2>&1 | tail -20

   echo "PASS: P82 no-mirror-remote hint preserved end-to-end after P83 write fan-out"
   exit 0
   ```

   Authored AT this task; FAIL at this task; rows 1, 3, 4 PASS at
   T06; row 2 PASS at P83-02 T04.

2. **Hand-edit `quality/catalogs/agent-ux.json`.** Add 4 new rows
   to the existing `rows` array. Each row shape mirrors P82's
   `agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first` row
   verbatim. Example for row 1; the other 3 follow the same template
   (id / sources / command / expected / verifier / artifact paths
   swapped):

   ```json
   {
     "id": "agent-ux/bus-write-sot-first-success",
     "dimension": "agent-ux",
     "cadence": "pre-pr",
     "kind": "mechanical",
     "_provenance_note": "Hand-edit per documented gap (NOT Principle A): reposix-quality bind only supports the docs-alignment dimension at v0.13.0; agent-ux dim mints stay hand-edited until GOOD-TO-HAVES-01 ships the verb extension. See .planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md § GOOD-TO-HAVES-01.",
     "sources": [
       "crates/reposix-remote/src/bus_handler.rs::handle_bus_export",
       "crates/reposix-remote/src/write_loop.rs::apply_writes",
       "crates/reposix-remote/tests/bus_write_happy.rs::happy_path_writes_both_refs_and_acks_ok",
       ".planning/research/v0.13.0-dvcs/architecture-sketch.md (§ 3 — bus algorithm steps 4-9)",
       ".planning/research/v0.13.0-dvcs/decisions.md (Q2.3 RATIFIED both refs on success)",
       ".planning/REQUIREMENTS.md (DVCS-BUS-WRITE-01 + DVCS-BUS-WRITE-03)"
     ],
     "command": "bash quality/gates/agent-ux/bus-write-sot-first-success.sh",
     "expected": {
       "asserts": [
         "bash quality/gates/agent-ux/bus-write-sot-first-success.sh exits 0",
         "bus_handler::handle_bus_export calls write_loop::apply_writes",
         "On SoT-success + mirror-success: refs/mirrors/<sot>-head and refs/mirrors/<sot>-synced-at both advance",
         "audit_events_cache contains helper_push_started + helper_push_accepted + mirror_sync_written rows",
         "audit_events contains per-record mutation rows",
         "helper exits zero with `ok refs/heads/main` on stdout"
       ]
     },
     "verifier": {
       "script": "quality/gates/agent-ux/bus-write-sot-first-success.sh",
       "args": [],
       "timeout_s": 60,
       "container": null
     },
     "artifact": "quality/reports/verifications/agent-ux/bus-write-sot-first-success.json",
     "status": "FAIL",
     "last_verified": null,
     "freshness_ttl": null,
     "blast_radius": "P1",
     "owner_hint": "if RED: crates/reposix-remote/src/bus_handler.rs OR write_loop.rs regressed; check whether apply_writes was lifted correctly OR the mirror_sync_written row is missing on the success path",
     "waiver": null
   }
   ```

   The other 3 rows follow this template; specific deltas:

   - **Row 2** `agent-ux/bus-write-mirror-fail-returns-ok` —
     sources cite `bus_handler::handle_bus_export` + `push_mirror`
     + `audit::log_helper_push_partial_fail_mirror_lag` +
     `tests/bus_write_mirror_fail.rs::bus_write_mirror_fail_returns_ok_with_lag_audit_row`
     + DVCS-BUS-WRITE-02 + Q3.6. asserts: helper exits zero with
     `ok refs/heads/main` on stdout AND stderr WARN AND
     `helper_push_partial_fail_mirror_lag` row in
     `audit_events_cache` AND `refs/mirrors/<sot>-head` advanced
     AND `refs/mirrors/<sot>-synced-at` UNCHANGED from baseline.
     `owner_hint`: "if RED: bus_handler.rs partial-fail branch
     regressed OR audit op missing from cache_schema.sql CHECK
     list". Per M1 from PLAN-CHECK.md, also include a `comment`
     field on this row: *"Row stays FAIL through 83-01 close; flips
     PASS at 83-02 T04 audit-completeness landing. pre-pr cadence
     runners between phases SHOULD NOT flag this as a phantom
     failure — the FAIL is by design until 83-02's
     `bus_write_mirror_fail.rs` test ships."*
   - **Row 3** `agent-ux/bus-write-no-helper-retry` — sources cite
     `crates/reposix-remote/src/bus_handler.rs::push_mirror`
     + DVCS-BUS-WRITE-04 + Q3.6 RATIFIED. asserts: bus_handler.rs
     contains NO retry constructs adjacent to push_mirror; NO
     `--force-with-lease`; NO `--force` flag. `owner_hint`: "if
     RED: a retry construct was introduced via cargo-cult from
     P84; reject in code review".
   - **Row 4** `agent-ux/bus-write-no-mirror-remote-still-fails` —
     sources cite
     `bus_handler::resolve_mirror_remote_name` + P82 STEP 0 path
     + `tests/bus_write_no_mirror_remote.rs::bus_write_no_mirror_remote_emits_q35_hint`
     + DVCS-BUS-WRITE-05 + Q3.5. asserts: bus URL with no local
     `git remote` for the mirror still emits verbatim Q3.5 hint
     after P83 lands; NO auto-mutation of `.git/config`; NO stdin
     read (regression guarantee). `owner_hint`: "if RED: P83's
     write fan-out accidentally bypassed P82's STEP 0 check; check
     whether handle_bus_export's mirror_remote_name resolution
     order changed".

3. **Validate JSON parses + the rows are addressable:**

   ```bash
   python3 -c '
   import json
   data = json.load(open("quality/catalogs/agent-ux.json"))
   row_ids = {r["id"] for r in data["rows"]}
   for required in [
       "agent-ux/bus-write-sot-first-success",
       "agent-ux/bus-write-mirror-fail-returns-ok",
       "agent-ux/bus-write-no-helper-retry",
       "agent-ux/bus-write-no-mirror-remote-still-fails",
   ]:
       assert required in row_ids, f"missing row: {required}"
   print("all 4 P83-01 rows present in agent-ux.json")
   '
   ```

4. **chmod + atomic catalog-first commit:**

   ```bash
   chmod +x quality/gates/agent-ux/bus-write-sot-first-success.sh \
            quality/gates/agent-ux/bus-write-mirror-fail-returns-ok.sh \
            quality/gates/agent-ux/bus-write-no-helper-retry.sh \
            quality/gates/agent-ux/bus-write-no-mirror-remote-still-fails.sh
   git add quality/gates/agent-ux/bus-write-sot-first-success.sh \
           quality/gates/agent-ux/bus-write-mirror-fail-returns-ok.sh \
           quality/gates/agent-ux/bus-write-no-helper-retry.sh \
           quality/gates/agent-ux/bus-write-no-mirror-remote-still-fails.sh \
           quality/catalogs/agent-ux.json
   git commit -m "$(cat <<'EOF'
quality(agent-ux): mint 4 bus-write-core catalog rows + 4 TINY verifiers (DVCS-BUS-WRITE-01..05 catalog-first)

- quality/gates/agent-ux/bus-write-sot-first-success.sh — TINY ~30-line cargo-test verifier (rows 1)
- quality/gates/agent-ux/bus-write-mirror-fail-returns-ok.sh — TINY ~30-line cargo-test verifier (row 2; test ships in P83-02 T02; row stays FAIL through P83-01 close)
- quality/gates/agent-ux/bus-write-no-helper-retry.sh — TINY ~50-line grep-based verifier asserting Q3.6 + D-08 (no retry / no --force) (row 3)
- quality/gates/agent-ux/bus-write-no-mirror-remote-still-fails.sh — TINY ~30-line cargo-test verifier (row 4)
- quality/catalogs/agent-ux.json — 4 rows added (status FAIL initial); rows 1+3+4 flip to PASS at P83-01 T06; row 2 flips to PASS at P83-02 T04 BEFORE per-phase push

Hand-edit per documented gap (NOT Principle A): reposix-quality bind supports docs-alignment dim only. agent-ux dim mints stay hand-edited until GOOD-TO-HAVES-01 ships the verb extension. See .planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md § GOOD-TO-HAVES-01.

Phase 83 / Plan 01 / Task 01 / DVCS-BUS-WRITE-01 + DVCS-BUS-WRITE-02 + DVCS-BUS-WRITE-03 + DVCS-BUS-WRITE-04 + DVCS-BUS-WRITE-05 (catalog-first).
EOF
)"
   ```

   NO push yet — the per-phase push is the terminal task of this
   plan (T06), not of T01.
</action>

<verify>
  <automated>python3 -c 'import json; ids = {r["id"] for r in json.load(open("quality/catalogs/agent-ux.json"))["rows"]}; required = ["agent-ux/bus-write-sot-first-success","agent-ux/bus-write-mirror-fail-returns-ok","agent-ux/bus-write-no-helper-retry","agent-ux/bus-write-no-mirror-remote-still-fails"]; missing = [i for i in required if i not in ids]; assert not missing, f"missing rows: {missing}"' && for f in bus-write-sot-first-success bus-write-mirror-fail-returns-ok bus-write-no-helper-retry bus-write-no-mirror-remote-still-fails; do test -x "quality/gates/agent-ux/${f}.sh" || { echo "missing executable: ${f}.sh"; exit 1; }; done</automated>
</verify>

<done>
- 4 verifier shells exist under `quality/gates/agent-ux/`, each
  executable, ~30-50 lines, mirroring P82's TINY shape (delegate to
  `cargo test`, OR static grep for the no-retry verifier).
- Running each verifier in isolation (without T02–T05 yet) fails
  cleanly: `bash <verifier>; rc=$?; [[ $rc != 0 ]]` succeeds — the
  rows' initial FAIL status reflects reality.
- `quality/catalogs/agent-ux.json` has 4 new rows; each row's
  `status` is `FAIL`; `verifier.script` ends in `.sh`; required
  fields per `quality/catalogs/README.md` schema are present.
- `python3 -c 'import json; json.load(open("quality/catalogs/agent-ux.json"))'`
  exits 0 (JSON parses).
- Each row's `_provenance_note` annotates "Hand-edit per documented
  gap (NOT Principle A)" and references GOOD-TO-HAVES-01.
- Commit message annotates the same AND notes that row 2 stays
  FAIL through P83-01 close.
- `git log -1 --oneline` shows the catalog-first commit.
- `git diff --stat HEAD~1` shows 5 files: 4 new .sh + 1 catalog edit.
</done>

---

## Task 83-01-T02 — Lift `handle_export` write loop into `write_loop::apply_writes`
