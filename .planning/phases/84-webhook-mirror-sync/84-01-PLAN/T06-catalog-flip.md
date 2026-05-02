← [back to index](./index.md) · phase 84 plan 01

## Task 84-01-T06 — Catalog flip + CLAUDE.md + per-phase push

<read_first>
- All 6 catalog rows in `quality/catalogs/agent-ux.json` (T01 minted
  as FAIL; ready to flip to PASS now that T02–T05 substrates exist).
- All 6 verifier shells under `quality/gates/agent-ux/webhook-*.sh`
  (T01–T05 authored / replaced).
- `CLAUDE.md` § Architecture (current state — find the section
  re-read line range via `grep -n "## Architecture" CLAUDE.md`) +
  § Commands (similar; `grep -n "## Commands" CLAUDE.md` — note:
  the section may be named "Commands you'll actually use" per
  current CLAUDE.md state).
- `.planning/phases/84-webhook-mirror-sync/84-PLAN-OVERVIEW.md`
  § "CLAUDE.md (T06; QG-07; ...)" — the verbatim paragraph + bullet
  shape.
- Pre-push hook is at `.git/hooks/pre-push`; runs `scripts/end-state.py`
  + dimension verifiers; the 6 P84 rows' verifiers fire here.
</read_first>

<action>
Three concerns: re-grade catalog rows → CLAUDE.md update → per-phase
push.

### 6a. Flip catalog rows FAIL → PASS via the runner

```bash
# Re-grade all P84 rows. The pre-pr cadence catches 5 of the 6;
# pre-release catches the latency floor.
python3 quality/runners/run.py --cadence pre-pr --tag webhook 2>&1 | tail -30
python3 quality/runners/run.py --cadence pre-release --tag webhook 2>&1 | tail -10

# Sanity-check: each row's status is PASS in the catalog after the
# runner mutates the JSON (the runner writes status fields after
# verifier exits 0).
python3 -c '
import json
d = json.load(open("quality/catalogs/agent-ux.json"))
p84 = [r for r in d["rows"] if r["id"].startswith("agent-ux/webhook-")]
for r in p84:
    print(f"{r[\"id\"]}: status={r[\"status\"]}")
    assert r["status"] == "PASS", f"{r[\"id\"]} not PASS: {r[\"status\"]}"
'
```

If any row remains FAIL after the runner, INVESTIGATE BEFORE
COMMITTING:
- `webhook-trigger-dispatch`: live copy not reachable (T02
  mirror-repo push failed silently? `gh api` returns 404? template/
  live diff non-zero modulo whitespace?).
- `webhook-cron-fallback`: structural grep failed (`fetch-depth: 0`
  missing? cron interpolation regression?).
- `webhook-force-with-lease-race`: T04 harness regressed.
- `webhook-first-run-empty-mirror`: T03 harness regressed.
- `webhook-latency-floor`: T05 artifact missing or p95 > 120.
- `webhook-backends-without-webhooks`: trim simulation produces
  invalid YAML.

### 6b. CLAUDE.md update (one paragraph + one bullet per QG-07)

Find the right insertion sites:

```bash
grep -n "^## Architecture" CLAUDE.md
grep -n "^## Commands you'll actually use" CLAUDE.md
```

In § Architecture, append a new paragraph (4-6 sentences) AFTER the
existing v0.13.0 architecture content:

```markdown
**Webhook-driven mirror sync (v0.13.0 P84+).** A reference GitHub
Action workflow lives in the mirror repo's
`.github/workflows/reposix-mirror-sync.yml` (NOT in
`reubenjohn/reposix`; the canonical repo carries the template at
`docs/guides/dvcs-mirror-setup-template.yml`). The two copies are
byte-equal modulo whitespace; the catalog row
`agent-ux/webhook-trigger-dispatch` enforces the invariant. Triggers:
`repository_dispatch` (event_type=`reposix-mirror-sync`) for the
webhook path + cron `*/30 * * * *` (literal — GH Actions parses
schedule blocks BEFORE evaluating `${{ vars.* }}`, so cadence
overrides require editing the YAML directly per Q4.1) for the safety
net. Secrets convention: `gh secret set ATLASSIAN_API_KEY`,
`ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT` on the **mirror repo**
(per `ci.yml:114-120` precedent). The workflow uses `cargo binstall
reposix-cli` (NOT bare `reposix` — that's the workspace name; the
binstall metadata lives in `crates/reposix-cli/Cargo.toml`). First-
run handling (Q4.3) branches on `git show-ref --verify --quiet
refs/remotes/mirror/main`: present → `--force-with-lease=...` push;
absent → plain push. Race protection: a concurrent bus push (P82+P83)
landing between the workflow's fetch and its push triggers a clean
lease rejection — the mirror is already in sync. Full owner walk-
through: `docs/guides/dvcs-mirror-setup.md` (P85).
```

In § Commands you'll actually use, append a new bullet under the
`# Run the stack` section or under a new `# Webhook-driven mirror
sync` sub-section:

```markdown
# Webhook-driven mirror sync (v0.13.0 P84+; mirror repo only)
gh api repos/reubenjohn/reposix-tokenworld-mirror/dispatches \
  -f event_type=reposix-mirror-sync                          # manually trigger mirror sync (synthetic; cron is */30min)
bash scripts/webhook-latency-measure.sh                       # owner-runnable n=10 real-TokenWorld latency measurement
```

### 6c. Atomic terminal commit + per-phase push

```bash
git add quality/catalogs/agent-ux.json CLAUDE.md
git commit -m "$(cat <<'EOF'
quality(P84): flip 6 webhook catalog rows PASS + CLAUDE.md update + per-phase push

- quality/catalogs/agent-ux.json: 6 P84 rows status FAIL → PASS
  (agent-ux/webhook-trigger-dispatch, webhook-cron-fallback,
  webhook-force-with-lease-race, webhook-first-run-empty-mirror,
  webhook-backends-without-webhooks, webhook-latency-floor)
- CLAUDE.md § Architecture: webhook-driven mirror sync paragraph
  (workflow path in mirror repo; secrets convention; Q4.1 cron-edit
  constraint; first-run branch; race protection; forward-link to P85)
- CLAUDE.md § Commands: gh api ... /dispatches + scripts/webhook-latency-measure.sh

Phase 84 / Plan 01 / Task 06 / phase-close (push BEFORE verifier
subagent dispatch per CLAUDE.md "Push cadence — per-phase").
EOF
)"

# Per-phase push to canonical repo.
git push origin main
```

If pre-push BLOCKs:
- Diagnose the failing invariant (named in pre-push output).
- Fix in a NEW commit (NEVER amend per CLAUDE.md git safety
  protocol).
- Re-push.
- DO NOT bypass with `--no-verify`.

After the push lands, the orchestrator dispatches the verifier
subagent. NOT a plan task — that's an orchestrator-level action.
</action>

<verify>
  <automated>python3 -c 'import json; d=json.load(open("quality/catalogs/agent-ux.json")); p84=[r for r in d["rows"] if r["id"].startswith("agent-ux/webhook-")]; assert len(p84)==6, f"expected 6 P84 rows, got {len(p84)}"; [print(f"{r[\"id\"]}: {r[\"status\"]}") for r in p84]; failing=[r["id"] for r in p84 if r["status"]!="PASS"]; assert not failing, f"non-PASS rows: {failing}"' && grep -q "Webhook-driven mirror sync" CLAUDE.md && grep -q "gh api repos/reubenjohn/reposix-tokenworld-mirror/dispatches" CLAUDE.md && git log -1 --format="%s" | grep -q "P84\|webhook" && git log origin/main..HEAD --oneline | wc -l | grep -q "^0$"</automated>
</verify>

<done>
- All 6 P84 catalog rows in `quality/catalogs/agent-ux.json` have
  `status: PASS`.
- `CLAUDE.md` § Architecture has the new "Webhook-driven mirror
  sync (v0.13.0 P84+)" paragraph (4-6 sentences) per QG-07.
- `CLAUDE.md` § Commands has the new `gh api .../dispatches` bullet
  + `scripts/webhook-latency-measure.sh` bullet.
- Per-phase push to canonical repo's `origin/main` completed; pre-
  push gate GREEN; `git log origin/main..HEAD` returns no
  unpushed commits.
- The mirror-repo push (T02) was a separate operation; T06 doesn't
  re-touch it.
- Commit message annotates "P84 / Plan 01 / Task 06 / phase-close".
- The 6 P84 verifier shells all exit 0 when run individually.
- Verifier subagent dispatch is the orchestrator's next step (NOT
  this plan's; outside the plan body).
- REQUIREMENTS.md DVCS-WEBHOOK-01..04 checkbox flips are likewise
  the orchestrator's next step (NOT this plan's).
</done>
