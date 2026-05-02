← [back to index](./index.md) · phase 80 plan 01

## Task 80-01-T01 — Catalog-first: mint 3 agent-ux rows + author 3 verifier shells

<read_first>
- `quality/catalogs/agent-ux.json` (full file — 2 existing rows; new 3 rows
  join the same `rows` array; row shape mirrors
  `agent-ux/reposix-attach-against-vanilla-clone` from P79).
- `quality/gates/agent-ux/reposix-attach.sh` (full file — TINY-shape
  precedent for the new verifier shells; same `cargo build → start sim
  → mktemp → run scenario → assert via git for-each-ref / git log /
  stderr grep` shape).
- `quality/catalogs/README.md` § "Unified schema" — required fields per row.
- `quality/PROTOCOL.md` § "Principle A — Subagents propose; tools
  validate and mint" — confirms the gap (Principle A applies to docs-
  alignment dim only; agent-ux dim has no `bind` verb yet).
- `.planning/phases/79-poc-reposix-attach-core/79-PLAN-OVERVIEW.md`
  § "New GOOD-TO-HAVES entry" — the documented-gap framing this task
  annotates verbatim.
- `.planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md` (if exists —
  GOOD-TO-HAVES-01 entry filed during P79; new annotations cite it).
</read_first>

<action>
This is the **catalog-first commit** for P80: the rows defining the
GREEN contract land BEFORE T02-T04 implementation. The rows reference
verifier shell scripts that do NOT exist yet — they will be authored
in this same task and ship in this same atomic commit.

**Important — this is NOT a Principle A application.** The
`reposix-quality bind` verb supports the `docs-alignment` dimension
only. The agent-ux dim is hand-edited per the existing two-row
precedent in `quality/catalogs/agent-ux.json`. The orchestrator filed
GOOD-TO-HAVES-01 during P79 to track the bind-verb extension work for
a future polish slot. The catalog edit in this task is therefore a
**hand-edit per documented gap**, not a Principle A application. This
is annotated in the commit message AND in each row's
`_provenance_note` field (mirroring the P79 row's annotation).

Steps:

1. **Author the three verifier shells.** Each is TINY (~30-60 lines)
   mirroring `quality/gates/agent-ux/reposix-attach.sh`. The verifiers
   exercise the relevant scenario end-to-end against a sim subprocess;
   they will FAIL until T02 + T03 + T04 implement and test the wiring.

   Authored AT this task; FAILS at this task; PASSes at T04.

   The full verbatim source of all three verifier shell scripts —
   `mirror-refs-write-on-success.sh`,
   `mirror-refs-readable-by-vanilla-fetch.sh`, and
   `mirror-refs-cited-in-reject-hint.sh` — lives in the sibling
   chapter [03b-T01-verifier-shells.md](./03b-T01-verifier-shells.md).
   Split out for chapter-budget hygiene; content preserved verbatim.

2. **Hand-edit `quality/catalogs/agent-ux.json`.** Add 3 new rows to
   the existing `rows` array (currently 2 rows: `agent-ux/dark-factory-sim`
   + `agent-ux/reposix-attach-against-vanilla-clone`). Each row uses
   the exact P79 row shape verified at planning time.

   ```json
   {
     "id": "agent-ux/mirror-refs-write-on-success",
     "dimension": "agent-ux",
     "cadence": "pre-pr",
     "kind": "mechanical",
     "_provenance_note": "Hand-edit per documented gap (NOT Principle A): reposix-quality bind only supports the docs-alignment dimension at v0.13.0; agent-ux dim mints stay hand-edited until GOOD-TO-HAVES-01 ships the verb extension. See .planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md § GOOD-TO-HAVES-01.",
     "sources": [
       "crates/reposix-cache/src/mirror_refs.rs",
       "crates/reposix-remote/src/main.rs::handle_export (success branch)",
       "crates/reposix-remote/tests/mirror_refs.rs::write_on_success_updates_both_refs",
       ".planning/REQUIREMENTS.md (DVCS-MIRROR-REFS-01, DVCS-MIRROR-REFS-02)"
     ],
     "command": "bash quality/gates/agent-ux/mirror-refs-write-on-success.sh",
     "expected": {
       "asserts": [
         "bash quality/gates/agent-ux/mirror-refs-write-on-success.sh exits 0 against the local cargo workspace",
         "after a single-backend push, the cache's bare repo has refs/mirrors/sim-head AND refs/mirrors/sim-synced-at resolvable via git for-each-ref",
         "git log refs/mirrors/sim-synced-at -1 --format=%B first line matches `mirror synced at <RFC3339>`",
         "audit_events_cache table contains a row with op = 'mirror_sync_written' for the push (verified by integration test)"
       ]
     },
     "verifier": {
       "script": "quality/gates/agent-ux/mirror-refs-write-on-success.sh",
       "args": [],
       "timeout_s": 180,
       "container": null
     },
     "artifact": "quality/reports/verifications/agent-ux/mirror-refs-write-on-success.json",
     "status": "FAIL",
     "last_verified": null,
     "freshness_ttl": null,
     "blast_radius": "P1",
     "owner_hint": "if RED: crates/reposix-cache/src/mirror_refs.rs writers regressed OR crates/reposix-remote/src/main.rs::handle_export success-branch wiring regressed; check stderr for whether refs were absent, tag message body malformed, or audit row missing",
     "waiver": null
   }
   ```

   Repeat the row shape for the other two:

   - `agent-ux/mirror-refs-readable-by-vanilla-fetch` — sources reference
     `mirror_refs.rs` + `handle_export` success branch + the integration
     test `vanilla_fetch_brings_mirror_refs` + REQUIREMENTS DVCS-MIRROR-REFS-02;
     command points at `mirror-refs-readable-by-vanilla-fetch.sh`.
   - `agent-ux/mirror-refs-cited-in-reject-hint` — sources reference
     `crates/reposix-remote/src/main.rs::handle_export (conflict reject branch)`
     + `crates/reposix-cache/src/mirror_refs.rs::Cache::read_mirror_synced_at`
     + the integration test `reject_hint_cites_synced_at_with_age` +
     REQUIREMENTS DVCS-MIRROR-REFS-03; command points at
     `mirror-refs-cited-in-reject-hint.sh`.

   Each row's `_provenance_note` is verbatim the same hand-edit-per-
   documented-gap framing.

3. **Validate JSON parses + the rows are addressable:**

   ```bash
   python3 -c '
   import json
   data = json.load(open("quality/catalogs/agent-ux.json"))
   rows = data["rows"]
   ids = {r["id"] for r in rows}
   for required in (
       "agent-ux/mirror-refs-write-on-success",
       "agent-ux/mirror-refs-readable-by-vanilla-fetch",
       "agent-ux/mirror-refs-cited-in-reject-hint",
   ):
       assert required in ids, f"missing row: {required}"
   '
   ```

4. **chmod + atomic catalog-first commit:**

   ```bash
   chmod +x quality/gates/agent-ux/mirror-refs-write-on-success.sh \
            quality/gates/agent-ux/mirror-refs-readable-by-vanilla-fetch.sh \
            quality/gates/agent-ux/mirror-refs-cited-in-reject-hint.sh
   git add quality/gates/agent-ux/mirror-refs-*.sh \
           quality/catalogs/agent-ux.json
   git commit -m "$(cat <<'EOF'
   quality(agent-ux): mint mirror-refs catalog rows + 3 TINY verifiers (DVCS-MIRROR-REFS-01..03 catalog-first)

   - quality/gates/agent-ux/mirror-refs-write-on-success.sh — TINY 60-line verifier (mirrors reposix-attach.sh)
   - quality/gates/agent-ux/mirror-refs-readable-by-vanilla-fetch.sh — TINY 50-line verifier
   - quality/gates/agent-ux/mirror-refs-cited-in-reject-hint.sh — TINY 70-line verifier
   - quality/catalogs/agent-ux.json — 3 rows added:
     - agent-ux/mirror-refs-write-on-success (FAIL initial)
     - agent-ux/mirror-refs-readable-by-vanilla-fetch (FAIL initial)
     - agent-ux/mirror-refs-cited-in-reject-hint (FAIL initial)
   - Initial status: FAIL (verifiers exist; implementation lands in T02-T04)
   - Runner re-grades to PASS at T04 BEFORE per-phase push.

   Hand-edit per documented gap (NOT Principle A): reposix-quality bind
   supports docs-alignment dim only. agent-ux dim mints are hand-edited
   until GOOD-TO-HAVES-01 ships the verb extension. See
   .planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md § GOOD-TO-HAVES-01.

   Phase 80 / Plan 01 / Task 01 / DVCS-MIRROR-REFS-01..03 (catalog-first).
   EOF
   )"
   ```

   NO push yet — the per-phase push is the terminal task of this plan
   (T04), not of T01.
</action>

<verify>
  <automated>python3 -c 'import json; rows = json.load(open("quality/catalogs/agent-ux.json"))["rows"]; ids = {r["id"] for r in rows}; assert {"agent-ux/mirror-refs-write-on-success","agent-ux/mirror-refs-readable-by-vanilla-fetch","agent-ux/mirror-refs-cited-in-reject-hint"}.issubset(ids), "rows missing"' && test -x quality/gates/agent-ux/mirror-refs-write-on-success.sh && test -x quality/gates/agent-ux/mirror-refs-readable-by-vanilla-fetch.sh && test -x quality/gates/agent-ux/mirror-refs-cited-in-reject-hint.sh</automated>
</verify>

<done>
- 3 verifier shells exist under `quality/gates/agent-ux/`, each
  executable, ~30-70 lines.
- Running each verifier in isolation (without T02-T04 yet) fails
  cleanly: `bash <verifier>; rc=$?; [[ $rc != 0 ]]` succeeds — the
  rows' initial FAIL status reflects reality.
- `quality/catalogs/agent-ux.json` has 3 new rows; each row's `status`
  is `FAIL`; `verifier.script` ends in `.sh`; required fields per
  `quality/catalogs/README.md` schema are present.
- `python3 -c 'import json; json.load(open("quality/catalogs/agent-ux.json"))'`
  exits 0 (JSON parses).
- Each row's `_provenance_note` annotates "Hand-edit per documented
  gap (NOT Principle A)" and references GOOD-TO-HAVES-01.
- Commit message annotates the same.
- `git log -1 --oneline` shows the catalog-first commit.
- `git diff --stat HEAD~1` shows ≤ 4 files: 3 new .sh + 1 catalog edit.
</done>

---
