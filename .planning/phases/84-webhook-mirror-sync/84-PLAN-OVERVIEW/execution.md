# Execution: phase-close, risks, delegation, verification

← [back to index](./index.md)

## Phase-close protocol

Per CLAUDE.md OP-7 + REQUIREMENTS.md § "Recurring success criteria
across every v0.13.0 phase":

1. **All commits pushed.** Plan terminates with `git push origin
   main` against the CANONICAL repo in T06 (per CLAUDE.md "Push
   cadence — per-phase", codified 2026-04-30, closes backlog 999.4).
   Pre-push gate-passing is part of the plan's close criterion. The
   mirror-repo push (T02) was an earlier independent operation; T06
   does not re-touch the mirror repo.
2. **Pre-push gate GREEN.** If pre-push BLOCKS: treat as plan-internal
   failure (fix, NEW commit, re-push). NO `--no-verify` per CLAUDE.md
   git safety protocol.
3. **Verifier subagent dispatched.** AFTER 84-01 pushes (i.e., after
   T06 completes), the orchestrator dispatches an unbiased verifier
   subagent per `quality/PROTOCOL.md` § "Verifier subagent prompt
   template" (verbatim copy). The subagent grades the six P84
   catalog rows from artifacts with zero session context.
4. **Verdict at `quality/reports/verdicts/p84/VERDICT.md`.** Format
   per `quality/PROTOCOL.md`. Phase loops back if verdict is RED.
5. **STATE.md cursor advanced.** Update `.planning/STATE.md` Current
   Position from "P83 SHIPPED ... next P84" → "P84 SHIPPED 2026-MM-DD"
   (commit SHA cited).
6. **CLAUDE.md updated in T06.** T06's CLAUDE.md edit lands in the
   terminal commit (one § Architecture paragraph + one § Commands
   bullet per QG-07).
7. **REQUIREMENTS.md DVCS-WEBHOOK-01..04 checkboxes flipped.**
   Orchestrator (top-level) flips `[ ]` → `[x]` after verifier GREEN.
   NOT a plan task.

## Risks + mitigations

| Risk                                                                                                  | Likelihood | Mitigation                                                                                                                                                                                                                                                                                                |
|-------------------------------------------------------------------------------------------------------|------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Executor commits the workflow YAML to the WRONG repo** (RESEARCH.md Pitfall 1) — lands in `reubenjohn/reposix` instead of `reubenjohn/reposix-tokenworld-mirror`. | MEDIUM     | T02's first sub-step explicitly names the target repo as the mirror. T02's verifier asserts `gh api repos/reubenjohn/reposix-tokenworld-mirror/contents/.github/workflows/reposix-mirror-sync.yml` returns 200 (not a local file check). T01's plan body and T02's `<read_first>` both name the constraint at the top.        |
| **`cargo binstall reposix` instead of `cargo binstall reposix-cli`** (RESEARCH.md Pitfall 2)         | MEDIUM     | D-05 RATIFIED. Verifier greps the YAML for `cargo binstall reposix-cli` (and asserts `cargo binstall reposix\b` does NOT appear). Comment in YAML cites `crates/reposix-cli/Cargo.toml` `[package.metadata.binstall]`.                                                                                  |
| **`${{ vars.MIRROR_SYNC_CRON }}` in schedule field** (RESEARCH.md Pitfall 3 / Q4.1 misimpl)          | LOW-MED    | D-06 RATIFIED. Verifier greps for the literal `'*/30 * * * *'`. CLAUDE.md (T06) + P85 docs (later) document the edit-the-YAML flow for cadence overrides. Bench-cron precedent (`bench-latency-cron.yml:18` hardcodes `'0 13 * * 1'`) cited.                                                                  |
| **`actions/checkout@v6` `fetch-depth: 0` missing** (RESEARCH.md Pitfall 4) — shallow clone breaks `--force-with-lease`'s lease comparison | LOW        | T02 explicitly sets `fetch-depth: 0` in the checkout step per the verbatim YAML in RESEARCH.md. Verifier `webhook-cron-fallback.sh` greps for `fetch-depth: 0`.                                                                                                                                          |
| **Mirror's auto-generated README replaced on first run causes owner confusion** (RESEARCH.md Pitfall 5) | LOW        | Documented in P85's setup guide. Not a P84-blocker — the behavior is correct; the documentation is the mitigation.                                                                                                                                                                                          |
| **Workflow runs every 30 min during phase development, cluttering the mirror repo's Actions tab** (RESEARCH.md Pitfall 6) | MEDIUM     | T02's mirror-repo push lands LATE (after T01 catalog work in canonical repo, after authoring in canonical repo, after CI on the canonical repo's commit succeeds). Owner can disable the workflow on the mirror repo via `gh workflow disable reposix-mirror-sync --repo reubenjohn/reposix-tokenworld-mirror` between T02 and milestone close if cron noise is bothersome. Document in T02's notes. |
| **GH Actions `repository_dispatch` requires a PAT, not the runner `GITHUB_TOKEN`** (RESEARCH.md Pitfall 7) | LOW        | Documented in P85's setup guide; NOT a P84-implementation issue (the workflow itself receives the dispatch event; it doesn't trigger workflows in other repos). T05's synthetic-dispatch test uses the executor's `gh auth status`-confirmed token, which has `repo` scope.                                |
| **`cargo binstall reposix-cli` fails because the latest published version lacks the confluence init path** (RESEARCH.md Assumption A5) | LOW        | T02 verifies empirically before declaring the row green: dry-run `cargo binstall --dry-run reposix-cli` resolves a recent version (the executor sanity-checks). If a release-lag-driven gap surfaces, file as P84 SURPRISES-INTAKE entry — eager-resolve by pinning the binstall version in YAML.                  |
| **Synthetic dispatch via `gh api ... /dispatches` doesn't fire the workflow on `main` because it requires the workflow file already on `main`** | LOW        | T05's synthetic harness runs AFTER T02's mirror-repo push. The workflow is on `main` of the mirror repo by then. If it isn't, T05 fails with a clear "no workflow runs found" error — fix is sequencing, not a substrate issue.                                                                          |
| **Pre-push hook BLOCKs on a pre-existing drift unrelated to P84**                                    | LOW        | Per CLAUDE.md § "Push cadence — per-phase": treat as phase-internal failure. Diagnose, fix, NEW commit (NEVER amend), re-push. Do NOT bypass with `--no-verify`.                                                                                                                                            |
| **Real-TokenWorld latency measurement (`scripts/webhook-latency-measure.sh`) requires manual edits in TokenWorld**                          | LOW        | The script is OWNER-RUNNABLE, NOT CI-runnable. T05 ships the SCRIPT and the SYNTHETIC-method JSON; the real-TokenWorld pass is post-phase, owner-driven, and reported in a separate JSON refresh. Catalog row's `verdict: PASS` requires synthetic p95 ≤ 120 — achievable. |
| **Atlassian Cloud webhook config doesn't support the `Authorization: token <PAT>` header shape** (RESEARCH.md Assumption A6) | LOW        | Atlassian's outbound-webhook docs claim arbitrary URL + headers. If empirically broken, file as P85 SURPRISES-INTAKE entry — fallback is a relay service, but P85 is the right phase to discover this; P84 implements the receiving side correctly regardless.                                          |

## +2 reservation: out-of-scope candidates

`.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` and
`GOOD-TO-HAVES.md` exist already (created during P79). P84 surfaces
candidates only when they materialize during execution — none
pre-filed at planning time.

Anticipated candidates the plan flags (per OP-8):

- **LOW** — Recurring weekly latency measurement. D-02 ratified
  one-shot at `pre-release` cadence; if v0.14.0 owner-load surfaces
  value in recurring measurement, file as v0.14.0 GOOD-TO-HAVE.
- **LOW** — `cancel-in-progress: true` for the concurrency block
  IF empirical multi-run measurements show queue-depth saturation.
  D-01 ratified `false` (queue, don't cancel); revisit if real data
  shows the queue growing unbounded.
- **LOW-MED** — Workflow file at the mirror repo's `main` branch
  fires its cron on every commit to the mirror repo. If the mirror's
  `main` advances frequently (e.g., 100+ pushes/day from the bus
  remote in production), the cron could overlap heavily with the
  push activity. D-01's `concurrency: cancel-in-progress: false`
  handles this, but at high volume the queue could grow. Eager-resolve
  IF empirical measurement shows it; file otherwise.
- **MED** — `actionlint` (Go binary) static-analysis of the workflow
  YAML in the verifier shells. RESEARCH.md § "Test Infrastructure"
  recommends but defers to "install on demand." If the verifier
  catches a YAML bug in production that `actionlint` would have
  caught at write-time, file as v0.14.0 GOOD-TO-HAVE to add it to
  pre-push as a structure-dim row.

Items NOT in scope for P84 (deferred per the v0.13.0 ROADMAP):

- DVCS docs (P85). T06 only updates CLAUDE.md; the
  `dvcs-mirror-setup.md` walkthrough lives in P85.
- Confluence webhook configuration automation. Owner-side manual
  setup per RESEARCH.md § "Deferred Ideas"; P85 documents the manual
  flow.
- Bidirectional sync. Mirror is read-only from confluence's
  perspective. Out of scope per architecture-sketch.
- Multi-mirror fan-out. Workflow targets a single mirror. v0.14.0.
- Real-TokenWorld latency headline number (the n=10 manual-edit
  pass). Owner runs `scripts/webhook-latency-measure.sh` post-phase;
  the JSON refresh is a separate commit (likely milestone-close P88
  or a v0.14.0 P0 task). T05 ships the synthetic number.
- `actionlint` integration. Filed as a candidate above.

## Subagent delegation

Per CLAUDE.md "Subagent delegation rules" + the gsd-planner spec
"aggressive subagent delegation":

| Plan / Task                                                      | Delegation                                                                                                                                                                                                                  |
|------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 84-01 T01 (6 catalog rows + 6 verifier shells)                   | `gsd-executor` — catalog-first commit; **hand-edits agent-ux.json per documented gap (NOT Principle A)**.                                                                                                                  |
| 84-01 T02 (workflow YAML — template in canonical + live in mirror) | Same 84-01 executor. Two git-push operations (canonical repo via working tree; mirror repo via temp clone). NO cargo invocations.                                                                                                |
| 84-01 T03 (first-run shell harness)                              | Same 84-01 executor. Pure shell + `git init --bare` fixtures. No cargo.                                                                                                                                                       |
| 84-01 T04 (force-with-lease race shell harness)                  | Same 84-01 executor. Pure shell + `git init --bare` fixtures. No cargo.                                                                                                                                                       |
| 84-01 T05 (latency artifact + measurement script)                | Same 84-01 executor. Synthetic `gh api`-based dispatch + JSON write. No cargo.                                                                                                                                                |
| 84-01 T06 (catalog flip + CLAUDE.md + push)                      | Same 84-01 executor (terminal task).                                                                                                                                                                                          |
| Phase verifier (P84 close)                                       | Unbiased subagent dispatched by orchestrator AFTER 84-01 T06 pushes per `quality/PROTOCOL.md` § "Verifier subagent prompt template" (verbatim). Zero session context; grades the six catalog rows from artifacts.        |

Phase verifier subagent's verdict criteria (extracted for P84):

- **DVCS-WEBHOOK-01:** template copy at `docs/guides/dvcs-mirror-setup-template.yml`
  exists and parses as valid YAML; live copy reachable via `gh api
  repos/reubenjohn/reposix-tokenworld-mirror/contents/.github/workflows/reposix-mirror-sync.yml`
  (200 OK); `diff -w` of the two copies returns zero (byte-equal
  modulo whitespace); workflow YAML contains `repository_dispatch`
  with `types: [reposix-mirror-sync]` AND `schedule:` with cron
  `'*/30 * * * *'` AND `cargo binstall reposix-cli` (NOT `reposix`)
  AND `actions/checkout@v6` with `fetch-depth: 0` AND
  `concurrency: { group: reposix-mirror-sync, cancel-in-progress:
  false }` (D-01).
- **DVCS-WEBHOOK-02:** workflow YAML's "Push to mirror" step uses
  `git push --force-with-lease=refs/heads/main:${LEASE_SHA}` shape
  (grep-able); shell harness `webhook-force-with-lease-race.sh`
  exits 0 (asserting the rejection-on-race property). Mirror's
  `main` ref untouched after the simulated race.
- **DVCS-WEBHOOK-03:** workflow YAML branches on `git show-ref
  --verify --quiet refs/remotes/mirror/main`; shell harness
  `webhook-first-run-empty-mirror.sh` exits 0 (asserting both 4.3.a
  and 4.3.b sub-cases handled correctly).
- **DVCS-WEBHOOK-04:** `quality/reports/verifications/perf/webhook-latency.json`
  exists with required fields (`measured_at`, `method`, `n`,
  `p50_seconds`, `p95_seconds`, `max_seconds`, `target_seconds`,
  `verdict`); `webhook-latency-floor.sh` asserts `p95_seconds ≤ 120`.
- **Q4.2 backends-without-webhooks (5th catalog row):** workflow's
  `repository_dispatch` block is deletable without breaking the
  cron-only path; the verifier `webhook-backends-without-webhooks.sh`
  greps the YAML for the documented trim path AND validates that
  removing the `repository_dispatch:` block produces still-valid
  YAML.
- **`client_payload` ignored (S2):** verifier greps the YAML for
  `github.event.client_payload` and asserts ZERO matches.
- New catalog rows in `quality/catalogs/agent-ux.json` (6); each
  verifier exits 0; status PASS after T06.
- Recurring (per phase): catalog-first ordering preserved (T01 commits
  catalog rows BEFORE T02–T06 implementation); per-phase push
  completed; verdict file at `quality/reports/verdicts/p84/VERDICT.md`;
  CLAUDE.md updated in T06.

## Verification approach (developer-facing)

After T06 pushes and the orchestrator dispatches the verifier subagent:

```bash
# Verifier-equivalent invocations (informational; the verifier subagent runs from artifacts):
bash quality/gates/agent-ux/webhook-trigger-dispatch.sh
bash quality/gates/agent-ux/webhook-cron-fallback.sh
bash quality/gates/agent-ux/webhook-force-with-lease-race.sh
bash quality/gates/agent-ux/webhook-first-run-empty-mirror.sh
bash quality/gates/agent-ux/webhook-backends-without-webhooks.sh
bash quality/gates/agent-ux/webhook-latency-floor.sh
python3 quality/runners/run.py --cadence pre-pr --tag webhook         # re-grade pre-pr rows
python3 quality/runners/run.py --cadence pre-release --tag webhook    # re-grade webhook-latency-floor

# Synthetic dispatch + ref-arrival check (manual; not part of the catalog):
gh api repos/reubenjohn/reposix-tokenworld-mirror/dispatches \
  -f event_type=reposix-mirror-sync \
  -f client_payload='{"trigger":"manual-test"}'
gh run watch --repo reubenjohn/reposix-tokenworld-mirror \
  $(gh run list --repo reubenjohn/reposix-tokenworld-mirror \
     --workflow=reposix-mirror-sync --limit 1 --json databaseId -q '.[0].databaseId')
gh api repos/reubenjohn/reposix-tokenworld-mirror/git/refs/mirrors/confluence-synced-at \
  -q .object.sha
```

The fixtures for T03 + T04 use **two local bare repos**
(`mktemp -d` + `git init --bare` + `file://` URL) per RESEARCH.md
§ "Test Fixture Strategy". Same approach as
`scripts/dark-factory-test.sh`.

This is a **subtle point worth flagging**: success criterion 2
(`--force-with-lease` race) is satisfied by two simultaneous
contracts — (a) the workflow YAML's push step uses the lease syntax
(grep-able from the YAML), AND (b) the shell harness asserts that an
actual lease check rejects on simulated drift. The integration tests
assert BOTH.
