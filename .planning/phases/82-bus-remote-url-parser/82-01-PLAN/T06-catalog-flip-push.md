← [back to index](./index.md) · phase 82 plan 01

## Task 82-01-T06 — Catalog flip + CLAUDE.md update + per-phase push

<read_first>
- `quality/runners/run.py` (find the `--cadence pre-pr` invocation
  shape that re-grades the catalog rows; same shape P81's T04 used).
- `CLAUDE.md` § Architecture (find the right insertion point — likely
  after the "L1 conflict detection" paragraph P81 added).
- `CLAUDE.md` § Commands → "Local dev loop" block (find the bullet
  list to extend; sibling of `reposix sync --reconcile` from P81).
- `.planning/STATE.md` (find the "Current Position" cursor — note
  that the cursor advance is an orchestrator-level action, NOT a
  plan task).
- `quality/catalogs/agent-ux.json` (post-T01; confirm the 6 P82 rows
  are present at status FAIL).
</read_first>

<action>
Three concerns: catalog flip → CLAUDE.md update → per-phase push.
The push is the terminal action.

### 6a. Catalog flip — flip 6 rows FAIL → PASS

Run the catalog runner to re-grade after T01–T05 land:

```bash
python3 quality/runners/run.py --cadence pre-pr 2>&1 | tee /tmp/p82-runner.log
```

The runner reads each row's `verifier.script`, runs it, and updates
`status` based on exit code. After T02–T05 ship, all 6 verifiers
exit 0 and the rows flip to PASS.

Confirm via:

```bash
python3 -c '
import json
data = json.load(open("quality/catalogs/agent-ux.json"))
rows = {r["id"]: r["status"] for r in data["rows"]}
required = [
    "agent-ux/bus-url-parses-query-param-form",
    "agent-ux/bus-url-rejects-plus-delimited",
    "agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first",
    "agent-ux/bus-precheck-b-sot-drift-emits-fetch-first",
    "agent-ux/bus-fetch-not-advertised",
    "agent-ux/bus-no-remote-configured-error",
]
not_pass = [(r, rows[r]) for r in required if rows.get(r) != "PASS"]
assert not not_pass, f"rows not PASS: {not_pass}"
print("all 6 P82 rows flipped to PASS")
'
```

### 6b. CLAUDE.md update — one paragraph + one bullet (QG-07)

Edit `CLAUDE.md`. Two insertions:

1. **§ Architecture.** Add a paragraph describing the bus URL form
   and Q3.4 PUSH-only contract. Place AFTER the existing "L1
   conflict detection (P81+)" paragraph (or wherever the architecture
   flow names mirror integration). New paragraph verbatim:

   ```
   **Bus URL form (P82+).** `reposix::<sot-spec>?mirror=<mirror-url>`
   per Q3.3 — the SoT side dispatches via the existing
   `BackendConnector` pipeline (sim / confluence / github / jira);
   the mirror is a plain-git URL consumed as a shell-out argument
   to `git ls-remote` / `git push`. Bus is PUSH-only (Q3.4) — fetch
   on a bus URL falls through to the single-backend code path, so
   the helper does NOT advertise `stateless-connect` for bus URLs.
   The `+`-delimited form is dropped; unknown query keys (anything
   other than `mirror=`) are rejected. Mirror URLs containing `?`
   must be percent-encoded (the first unescaped `?` in the bus URL
   is the bus query-string boundary). See
   `.planning/research/v0.13.0-dvcs/architecture-sketch.md § 3` and
   `decisions.md § Q3.3-Q3.6` for the algorithm + open-question
   resolutions.
   ```

2. **§ Commands → "Local dev loop" block.** Add a bullet for the
   bus push form. Place AFTER the existing `reposix sync --reconcile`
   bullet (P81):

   ```
   git push reposix main                                     # bus push (URL: reposix::<sot>?mirror=<url>; SoT-first writes land in P83)
   ```

### 6c. Per-phase push (terminal action)

The push is the LAST commit of the plan. T06's catalog flip + CLAUDE.md
edit land in a single commit, then the push runs.

```bash
git add quality/catalogs/agent-ux.json \
        CLAUDE.md
git commit -m "$(cat <<'EOF'
quality(agent-ux): flip 6 P82 rows FAIL→PASS + CLAUDE.md update (DVCS-BUS-URL-01..02-PRECHECK + DVCS-BUS-FETCH-01 close)

- quality/catalogs/agent-ux.json — 6 P82 rows flipped FAIL → PASS by `python3 quality/runners/run.py --cadence pre-pr`:
  - agent-ux/bus-url-parses-query-param-form
  - agent-ux/bus-url-rejects-plus-delimited
  - agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first
  - agent-ux/bus-precheck-b-sot-drift-emits-fetch-first
  - agent-ux/bus-fetch-not-advertised
  - agent-ux/bus-no-remote-configured-error
- CLAUDE.md — § Architecture gains a "Bus URL form (P82+)" paragraph naming Q3.3 ?mirror= form, Q3.4 PUSH-only / no stateless-connect, percent-encoding requirement for ? in mirror URLs; § Commands gains a `git push reposix main` bullet citing P83 deferral

Phase 82 / Plan 01 / Task 06 / DVCS-BUS-URL-01..02-PRECHECK + DVCS-BUS-FETCH-01 (close).
EOF
)"
git push origin main
```

If pre-push BLOCKS: treat as plan-internal failure. Diagnose, fix,
NEW commit (NEVER amend). Do NOT bypass with `--no-verify`. Re-run
`git push origin main` until it succeeds.

After the push lands, the orchestrator dispatches the verifier
subagent per `quality/PROTOCOL.md § "Verifier subagent prompt
template"`. The subagent grades the six catalog rows from artifacts
with zero session context. The dispatch is an orchestrator-level
action AFTER this plan completes — NOT a plan task.
</action>

<verify>
  <automated>python3 -c 'import json; rows = {r["id"]: r["status"] for r in json.load(open("quality/catalogs/agent-ux.json"))["rows"]}; required = ["agent-ux/bus-url-parses-query-param-form","agent-ux/bus-url-rejects-plus-delimited","agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first","agent-ux/bus-precheck-b-sot-drift-emits-fetch-first","agent-ux/bus-fetch-not-advertised","agent-ux/bus-no-remote-configured-error"]; not_pass = [(r, rows.get(r)) for r in required if rows.get(r) != "PASS"]; assert not not_pass, f"rows not PASS: {not_pass}"' && grep -q "Bus URL form" CLAUDE.md && grep -q "git push reposix main" CLAUDE.md</automated>
</verify>

<done>
- `quality/catalogs/agent-ux.json` rows
  `agent-ux/bus-url-parses-query-param-form`,
  `agent-ux/bus-url-rejects-plus-delimited`,
  `agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first`,
  `agent-ux/bus-precheck-b-sot-drift-emits-fetch-first`,
  `agent-ux/bus-fetch-not-advertised`,
  `agent-ux/bus-no-remote-configured-error` all have `status: PASS`.
- CLAUDE.md § Architecture includes the "Bus URL form (P82+)"
  paragraph naming Q3.3 + Q3.4 + percent-encoding requirement.
- CLAUDE.md § Commands → "Local dev loop" block includes
  `git push reposix main` bullet citing P83 deferral.
- `git push origin main` succeeded with pre-push GREEN. The phase's
  terminal commit cites all four requirements
  (DVCS-BUS-URL-01 / DVCS-BUS-PRECHECK-01 / DVCS-BUS-PRECHECK-02 /
  DVCS-BUS-FETCH-01).
- No cargo runs in T06 (catalog flip is shell-only; CLAUDE.md edit
  is doc-only). The `cargo nextest run -p reposix-remote` smoke
  ran in T05 — covered.
- Verifier subagent dispatch (orchestrator-level action) follows
  after the push.
</done>

---

