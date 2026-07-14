← [back to index](./index.md) · phase 82 plan 01

## Task 82-01-T01 — Catalog-first: mint 6 catalog rows + author 6 verifier shells

<read_first>
- `quality/catalogs/agent-ux.json` (full file — 6 existing rows post-P81;
  the 6 new rows join the same `rows` array; row shape mirrors the
  existing `agent-ux/sync-reconcile-subcommand` row, lines TBD —
  re-confirm exact line range during T01 read_first via `grep -n
  "sync-reconcile-subcommand" quality/catalogs/agent-ux.json`).
- `quality/gates/agent-ux/sync-reconcile-subcommand.sh` (P81 TINY-shape
  precedent; same shape: `cargo test -p reposix-remote --test <name>`,
  exit 0).
- `quality/gates/agent-ux/mirror-refs-write-on-success.sh` (P80 TINY
  precedent — sibling pattern).
- `quality/catalogs/README.md` § "Unified schema" — required fields
  per row.
- `quality/PROTOCOL.md` § "Principle A — Subagents propose; tools
  validate and mint" — confirms the gap (Principle A applies to
  docs-alignment dim only; agent-ux dim has no `bind` verb yet).
- `.planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md` § GOOD-TO-HAVES-01
  — the documented-gap framing the agent-ux row annotations cite.
</read_first>

<action>
This is the **catalog-first commit** for P82: the rows defining the
GREEN contract land BEFORE T02–T06 implementation. SIX verifier shell
scripts ship in this same atomic commit (one per row).

**Important — agent-ux rows are NOT a Principle A application.** The
`reposix-quality bind` verb supports the `docs-alignment` dimension
only. The `agent-ux` dim is hand-edited per the existing P80/P81
precedent. The orchestrator filed GOOD-TO-HAVES-01 during P79 to
track the bind-verb extension work for a future polish slot. The
catalog edits in this task are therefore **hand-edits per documented
gap**, not Principle A applications. This is annotated in the commit
message AND in each row's `_provenance_note` field (mirroring P81's
row annotations).

Steps:

1. **Author the six verifier shells.** Each is TINY (~30-50 lines)
   mirroring `quality/gates/agent-ux/sync-reconcile-subcommand.sh`.
   The verifiers exercise the relevant scenario via cargo-test
   delegation; they will FAIL until T02–T05 implement and T06 flips
   the catalog status.

   `quality/gates/agent-ux/bus-url-parses-query-param-form.sh`:

   ```bash
   #!/usr/bin/env bash
   # quality/gates/agent-ux/bus-url-parses-query-param-form.sh — agent-ux
   # verifier for catalog row `agent-ux/bus-url-parses-query-param-form`.
   #
   # CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/bus-url-parses-query-param-form
   # CADENCE:     pre-pr (~10s wall time)
   # INVARIANT:   bus_url::parse("reposix::sim::demo?mirror=file:///tmp/m.git")
   #              returns Route::Bus { sot: <expected>, mirror_url: <expected> }.
   #              Single-backend "reposix::sim::demo" (no ?) returns Route::Single(...).
   #
   # Status until P82-01 T06: FAIL.
   set -euo pipefail
   SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
   REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
   cd "${REPO_ROOT}"

   cargo test -p reposix-remote --test bus_url \
       parses_query_param_form_round_trip route_single_for_bare_reposix_url \
       --quiet -- --nocapture 2>&1 | tail -20

   echo "PASS: bus_url::parse handles ?mirror= form + bare reposix:: form"
   exit 0
   ```

   `quality/gates/agent-ux/bus-url-rejects-plus-delimited.sh`:

   ```bash
   #!/usr/bin/env bash
   # CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/bus-url-rejects-plus-delimited
   # CADENCE:     pre-pr
   # INVARIANT:   bus_url::parse rejects + form (Q3.3) AND unknown query keys (Q-C);
   #              error message names ?mirror= as the canonical form.
   #
   # Status until P82-01 T06: FAIL.
   set -euo pipefail
   SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
   REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
   cd "${REPO_ROOT}"

   cargo test -p reposix-remote --test bus_url \
       rejects_plus_delimited_bus_url rejects_unknown_query_param \
       --quiet -- --nocapture 2>&1 | tail -20

   echo "PASS: bus_url::parse rejects + form AND unknown query keys"
   exit 0
   ```

   `quality/gates/agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first.sh`:

   ```bash
   #!/usr/bin/env bash
   # CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first
   # CADENCE:     pre-pr (~5s wall time)
   # INVARIANT:   PRECHECK A (mirror drift via git ls-remote) emits
   #              `error refs/heads/main fetch first` on stdout
   #              + hint on stderr; helper exits BEFORE PRECHECK B
   #              + BEFORE stdin read.
   #
   # Status until P82-01 T06: FAIL.
   set -euo pipefail
   SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
   REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
   cd "${REPO_ROOT}"

   cargo test -p reposix-remote --test bus_precheck_a \
       --quiet -- --nocapture 2>&1 | tail -20

   echo "PASS: PRECHECK A emits fetch first on mirror drift"
   exit 0
   ```

   `quality/gates/agent-ux/bus-precheck-b-sot-drift-emits-fetch-first.sh`:

   ```bash
   #!/usr/bin/env bash
   # CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/bus-precheck-b-sot-drift-emits-fetch-first
   # CADENCE:     pre-pr (~10s wall time)
   # INVARIANT:   PRECHECK B (SoT drift via list_changed_since) emits
   #              `error refs/heads/main fetch first` on stdout
   #              + hint citing refs/mirrors/<sot>-synced-at on stderr;
   #              helper exits BEFORE stdin read; ZERO PATCH/PUT calls
   #              hit wiremock.
   #
   # Status until P82-01 T06: FAIL.
   set -euo pipefail
   SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
   REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
   cd "${REPO_ROOT}"

   cargo test -p reposix-remote --test bus_precheck_b \
       --quiet -- --nocapture 2>&1 | tail -20

   echo "PASS: PRECHECK B emits fetch first on SoT drift"
   exit 0
   ```

   `quality/gates/agent-ux/bus-fetch-not-advertised.sh`:

   ```bash
   #!/usr/bin/env bash
   # CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/bus-fetch-not-advertised
   # CADENCE:     pre-pr (~5s wall time)
   # INVARIANT:   capability list emitted for bus URL contains
   #              import / export / refspec / object-format=sha1
   #              but NOT stateless-connect (DVCS-BUS-FETCH-01 / Q3.4).
   #              Capability list for bare reposix:: URL DOES contain
   #              stateless-connect (regression check on single-backend).
   #
   # Status until P82-01 T06: FAIL.
   set -euo pipefail
   SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
   REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
   cd "${REPO_ROOT}"

   cargo test -p reposix-remote --test bus_capabilities \
       --quiet -- --nocapture 2>&1 | tail -20

   echo "PASS: bus URL omits stateless-connect; single-backend retains it"
   exit 0
   ```

   `quality/gates/agent-ux/bus-no-remote-configured-error.sh`:

   ```bash
   #!/usr/bin/env bash
   # CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/bus-no-remote-configured-error
   # CADENCE:     pre-pr (~5s wall time)
   # INVARIANT:   bus URL with no local `git remote` for the mirror
   #              fails with verbatim Q3.5 hint
   #              "configure the mirror remote first: git remote add <name> <mirror-url>";
   #              NO auto-mutation of git config.
   #
   # Status until P82-01 T06: FAIL.
   set -euo pipefail
   SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
   REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
   cd "${REPO_ROOT}"

   cargo test -p reposix-remote --test bus_precheck_a \
       bus_no_remote_configured_emits_q35_hint \
       --quiet -- --nocapture 2>&1 | tail -20

   echo "PASS: bus URL with no local git remote emits Q3.5 hint"
   exit 0
   ```

   Authored AT this task; FAILS at this task; PASSes at T06.

2. **Hand-edit `quality/catalogs/agent-ux.json`.** Add 6 new rows to
   the existing `rows` array. Each row shape mirrors P81's
   `agent-ux/sync-reconcile-subcommand` row verbatim. Example for
   row 1; the other 5 follow the same template (id / sources /
   command / expected / verifier / artifact paths swapped):

   ```json
   {
     "id": "agent-ux/bus-url-parses-query-param-form",
     "dimension": "agent-ux",
     "cadence": "pre-pr",
     "kind": "mechanical",
     "_provenance_note": "Hand-edit per documented gap (NOT Principle A): reposix-quality bind only supports the docs-alignment dimension at v0.13.0; agent-ux dim mints stay hand-edited until GOOD-TO-HAVES-01 ships the verb extension. See .planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md § GOOD-TO-HAVES-01.",
     "sources": [
       "crates/reposix-remote/src/bus_url.rs::parse",
       "crates/reposix-remote/tests/bus_url.rs::parses_query_param_form_round_trip",
       "crates/reposix-remote/tests/bus_url.rs::route_single_for_bare_reposix_url",
       ".planning/research/v0.13.0-dvcs/architecture-sketch.md (§ 3 — bus URL form)",
       ".planning/research/v0.13.0-dvcs/decisions.md (Q3.3 RATIFIED ?mirror= form)",
       ".planning/REQUIREMENTS.md (DVCS-BUS-URL-01)"
     ],
     "command": "bash quality/gates/agent-ux/bus-url-parses-query-param-form.sh",
     "expected": {
       "asserts": [
         "bash quality/gates/agent-ux/bus-url-parses-query-param-form.sh exits 0",
         "bus_url::parse('reposix::sim::demo?mirror=file:///tmp/m.git') returns Route::Bus { sot: ParsedRemote { kind: Sim, project: 'demo', .. }, mirror_url: 'file:///tmp/m.git' }",
         "bus_url::parse('reposix::sim::demo') (no ?) returns Route::Single(ParsedRemote { kind: Sim, project: 'demo', .. })"
       ]
     },
     "verifier": {
       "script": "quality/gates/agent-ux/bus-url-parses-query-param-form.sh",
       "args": [],
       "timeout_s": 60,
       "container": null
     },
     "artifact": "quality/reports/verifications/agent-ux/bus-url-parses-query-param-form.json",
     "status": "FAIL",
     "last_verified": null,
     "freshness_ttl": null,
     "blast_radius": "P1",
     "owner_hint": "if RED: crates/reposix-remote/src/bus_url.rs regressed OR backend_dispatch::parse_remote_url stripped query string handling; check stderr for parse error message",
     "waiver": null
   }
   ```

   The other 5 rows follow this template; specific deltas:

   - **Row 2** `agent-ux/bus-url-rejects-plus-delimited` — sources cite
     `bus_url::parse` + tests `rejects_plus_delimited_bus_url` +
     `rejects_unknown_query_param`; cite Q3.3 + Q-C in the spec
     references. asserts: helper rejects `+` form with "use `?mirror=`
     instead" hint AND rejects unknown query keys with
     "unknown query parameter `<key>`" hint.
   - **Row 3** `agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first` —
     sources cite `bus_handler::precheck_mirror_drift` +
     `tests/bus_precheck_a.rs::bus_precheck_a_emits_fetch_first_on_drift`
     + DVCS-BUS-PRECHECK-01. asserts: helper exits non-zero with
     `error refs/heads/main fetch first` on stdout AND
     `your GH mirror has new commits` on stderr; ZERO `parse_export_stream`
     invocation observable; helper-push-started cache audit row count = 1
     but helper-push-accepted = 0.
   - **Row 4** `agent-ux/bus-precheck-b-sot-drift-emits-fetch-first` —
     sources cite `precheck::precheck_sot_drift_any` +
     `bus_handler::handle_bus_export` +
     `tests/bus_precheck_b.rs::bus_precheck_b_emits_fetch_first_on_sot_drift`
     + DVCS-BUS-PRECHECK-02. asserts: helper exits non-zero with
     `error refs/heads/main fetch first` on stdout AND hint citing
     `refs/mirrors/<sot>-synced-at` on stderr; ZERO PATCH/PUT calls
     hit wiremock.
   - **Row 5** `agent-ux/bus-fetch-not-advertised` — sources cite
     `crates/reposix-remote/src/main.rs (capabilities arm)` +
     `tests/bus_capabilities.rs::bus_url_omits_stateless_connect` +
     DVCS-BUS-FETCH-01 + Q3.4. asserts: capability list for bus URL
     contains import / export / refspec / object-format=sha1 but
     NOT stateless-connect; capability list for bare reposix:: URL
     DOES contain stateless-connect.
   - **Row 6** `agent-ux/bus-no-remote-configured-error` — sources
     cite `bus_handler::resolve_mirror_remote_name` +
     `tests/bus_precheck_a.rs::bus_no_remote_configured_emits_q35_hint`
     + Q3.5 + ROADMAP P82 SC5. asserts: bus URL referencing a
     `mirror_url` not in any local `remote.<name>.url` fails with
     verbatim Q3.5 hint *"configure the mirror remote first:
     `git remote add <name> <mirror-url>`"*; NO auto-mutation
     observable in the test working tree's `.git/config`.

3. **Validate JSON parses + the rows are addressable:**

   ```bash
   python3 -c '
   import json
   data = json.load(open("quality/catalogs/agent-ux.json"))
   row_ids = {r["id"] for r in data["rows"]}
   for required in [
       "agent-ux/bus-url-parses-query-param-form",
       "agent-ux/bus-url-rejects-plus-delimited",
       "agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first",
       "agent-ux/bus-precheck-b-sot-drift-emits-fetch-first",
       "agent-ux/bus-fetch-not-advertised",
       "agent-ux/bus-no-remote-configured-error",
   ]:
       assert required in row_ids, f"missing row: {required}"
   print("all 6 P82 rows present in agent-ux.json")
   '
   ```

4. **chmod + atomic catalog-first commit:**

   ```bash
   chmod +x quality/gates/agent-ux/bus-url-parses-query-param-form.sh \
            quality/gates/agent-ux/bus-url-rejects-plus-delimited.sh \
            quality/gates/agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first.sh \
            quality/gates/agent-ux/bus-precheck-b-sot-drift-emits-fetch-first.sh \
            quality/gates/agent-ux/bus-fetch-not-advertised.sh \
            quality/gates/agent-ux/bus-no-remote-configured-error.sh
   git add quality/gates/agent-ux/bus-url-parses-query-param-form.sh \
           quality/gates/agent-ux/bus-url-rejects-plus-delimited.sh \
           quality/gates/agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first.sh \
           quality/gates/agent-ux/bus-precheck-b-sot-drift-emits-fetch-first.sh \
           quality/gates/agent-ux/bus-fetch-not-advertised.sh \
           quality/gates/agent-ux/bus-no-remote-configured-error.sh \
           quality/catalogs/agent-ux.json
   git commit -m "$(cat <<'EOF'
quality(agent-ux): mint 6 bus-remote catalog rows + 6 TINY verifiers (DVCS-BUS-URL-01..02-PRECHECK + DVCS-BUS-FETCH-01 catalog-first)

- quality/gates/agent-ux/bus-url-parses-query-param-form.sh — TINY ~30-line verifier delegating to cargo test
- quality/gates/agent-ux/bus-url-rejects-plus-delimited.sh — TINY ~30-line verifier
- quality/gates/agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first.sh — TINY ~30-line verifier
- quality/gates/agent-ux/bus-precheck-b-sot-drift-emits-fetch-first.sh — TINY ~30-line verifier
- quality/gates/agent-ux/bus-fetch-not-advertised.sh — TINY ~30-line verifier
- quality/gates/agent-ux/bus-no-remote-configured-error.sh — TINY ~30-line verifier
- quality/catalogs/agent-ux.json — 6 rows added (status FAIL initial); flips to PASS at T06 BEFORE per-phase push

Hand-edit per documented gap (NOT Principle A): reposix-quality bind supports docs-alignment dim only. agent-ux dim mints stay hand-edited until GOOD-TO-HAVES-01 ships the verb extension. See .planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md § GOOD-TO-HAVES-01.

Phase 82 / Plan 01 / Task 01 / DVCS-BUS-URL-01 + DVCS-BUS-PRECHECK-01 + DVCS-BUS-PRECHECK-02 + DVCS-BUS-FETCH-01 (catalog-first).
EOF
)"
   ```

   NO push yet — the per-phase push is the terminal task of this plan
   (T06), not of T01.
</action>

<verify>
  <automated>python3 -c 'import json; ids = {r["id"] for r in json.load(open("quality/catalogs/agent-ux.json"))["rows"]}; required = ["agent-ux/bus-url-parses-query-param-form","agent-ux/bus-url-rejects-plus-delimited","agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first","agent-ux/bus-precheck-b-sot-drift-emits-fetch-first","agent-ux/bus-fetch-not-advertised","agent-ux/bus-no-remote-configured-error"]; missing = [i for i in required if i not in ids]; assert not missing, f"missing rows: {missing}"' && for f in bus-url-parses-query-param-form bus-url-rejects-plus-delimited bus-precheck-a-mirror-drift-emits-fetch-first bus-precheck-b-sot-drift-emits-fetch-first bus-fetch-not-advertised bus-no-remote-configured-error; do test -x "quality/gates/agent-ux/${f}.sh" || { echo "missing executable: ${f}.sh"; exit 1; }; done</automated>
</verify>

<done>
- 6 verifier shells exist under `quality/gates/agent-ux/`, each
  executable, ~30-50 lines, mirroring P81's TINY shape (delegate to
  `cargo test`, exit 0).
- Running each verifier in isolation (without T02–T05 yet) fails
  cleanly: `bash <verifier>; rc=$?; [[ $rc != 0 ]]` succeeds — the
  rows' initial FAIL status reflects reality.
- `quality/catalogs/agent-ux.json` has 6 new rows; each row's `status`
  is `FAIL`; `verifier.script` ends in `.sh`; required fields per
  `quality/catalogs/README.md` schema are present.
- `python3 -c 'import json; json.load(open("quality/catalogs/agent-ux.json"))'`
  exits 0 (JSON parses).
- Each row's `_provenance_note` annotates "Hand-edit per documented gap
  (NOT Principle A)" and references GOOD-TO-HAVES-01.
- Commit message annotates the same.
- `git log -1 --oneline` shows the catalog-first commit.
- `git diff --stat HEAD~1` shows 7 files: 6 new .sh + 1 catalog edit.
</done>

---

