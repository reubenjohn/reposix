# SESSION-HANDOVER.md — v0.15.0 Floor: P115 BENCH-01 Wave 1 CLOSED (T2 latency corrected), Waves 2–5 next — 2026-07-15

Written by the **relief-handover-writer** on behalf of **workhorse #32** (L0
orchestrator, herded by the manager in w1:p7), relieving to **successor #33**. This
file **REPLACES** (does not append to) the prior `SESSION-HANDOVER.md` (#31→#32's
handover, superseded here).

**Read order:** this file → §1 (verify live) → §6 runbook (Wave 2 / T3 ledger scaffold
is the opening move) → §3/§4/§5 as needed. **Guardrails unchanged:** do NOT touch
`.planning/MANAGER-HANDOVER.md` (separate document, separate owner — the manager, pane
w1:p7). No tag push by any coordinator — the manager cuts tags, never L0. Do NOT do git
surgery (reset/rebase/reorder/amend) on `main`. Shared tree with the manager — TARGETED
staging only, never `git add -A`/`.`.

## 1. Ground truth (git) — VERIFIED LIVE this handover, do not trust staleness

Re-run before doing anything else:
```
git rev-parse HEAD && git status --porcelain && \
  git rev-list --left-right --count HEAD...origin/main && \
  gh run list --branch main --workflow CI --limit 3
```
**Verified independently this handover (2026-07-15, just now):**
- `HEAD` = `3845b136aa084711a58252dd1130bdc1e2b7602b` ("docs(115): correct sim latency
  to CI canonical figures, ledger + defect filing"). Tree **CLEAN**.
- Local `main` is **2 commits AHEAD of origin/main, 0 behind — UNPUSHED,
  INTENTIONAL** (P115 mid-phase; push is deferred to PHASE CLOSE per the documented
  cadence, not to a wave/relief boundary). The two unpushed commits:
  - `9384ca6` — "docs(benchmarks): re-measure v0.9.0 latency envelope (P115 BENCH-01
    T2)" — initial re-measure, later found to carry a noise-outlier figure.
  - `3845b13` (HEAD) — "docs(115): correct sim latency to CI canonical figures,
    ledger + defect filing" — supersedes `9384ca6`'s sim cold-init figure with the
    CI-canonical value, adds a `[SELF]` methodology ruling to
    `.planning/CONSULT-DECISIONS.md`, and files a defect to
    `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md`.
  **#33: do NOT treat `ahead > 0` as an anomaly.** After this handover's own commit
  lands, ahead will be **3**. Re-verify with `git rev-list --left-right --count
  HEAD...origin/main` and expect the count to keep climbing wave-over-wave until T6
  closes the phase and pushes.
- `origin/main` tip `3278abc` ("docs(planning): A1 ruled [SELF] — benchmark session =
  one agentic conversation; manager handover → #32 on P115 execution") — CI (`ci.yml`)
  run `29452237641` = **completed/success (GREEN)**, 5m28s, concluded
  `2026-07-15T21:31:34Z`. The immediately-prior run `29451926899` (on the #31→#32
  relief-handover commit `804f5b0`) shows **cancelled**, not failure — superseded by
  the next push landing before it finished; not a signal.
- `.planning/STATE.md`: `completed_phases: 1` (of `total_phases: 21`); prose still
  reads "1/15 phases complete" (P114–P128 numbering window) — **P115 is NOT yet
  closed**, counter has not moved. `status: executing`, `last_activity: "2026-07-15 --
  Phase 115 planning complete"` (stale prose — execution IS in progress now, T1/T2
  done; STATE.md update is deferred to T6/phase-close, not a Wave-1 task).
- Phase dir `.planning/phases/115-live-mcp-benchmark-re-measurement/` contains
  `115-RESEARCH.md`, `115-PLAN.md` (6 tasks, 5 waves), `115-VALIDATION.md` — confirmed
  present on disk this handover, unchanged since planning.

## 2. Wave/cycle state

| Wave/Task | Plan | State | Commits |
|---|---|---|---|
| P114 (all waves + verification + close) | — | **DONE + CI GREEN** | tip carried from prior rotations |
| P115 planning (research → plan → plan-check → revision) | `115-PLAN.md` | **DONE, pushed, CI GREEN** | `cd0c951`, `8e1e970` |
| **P115 Wave 1 — T1 (A1 gate)** | `115-PLAN.md` | **DONE — RULED [SELF]** | `3278abc` |
| **P115 Wave 1 — T2 (latency re-measure)** | `115-PLAN.md` | **DONE — corrected to CI-canonical figures** | `9384ca6` (initial), `3845b13` (correction) |
| **P115 Wave 2 — T3 (ledger scaffold)** | `115-PLAN.md` | **NOT STARTED — THE OPENING MOVE FOR #33** | — |
| P115 Wave 3 — T4 (live-MCP token capture, ≤50 sessions) | `115-PLAN.md` | NOT STARTED | — |
| P115 Wave 4 — T5 (`token-economy.md` regen) | `115-PLAN.md` | NOT STARTED | — |
| P115 Wave 5 — T6 (un-waive path + consolidation) | `115-PLAN.md` | NOT STARTED | — |
| P116 ADR-010 packet (ADR-01 mirror-fanout + FIX-03 GTH-09 slug→id) | not yet written | NOT STARTED, after P115 execution, routes to MANAGER for ruling | — |
| roadmap-diagram gsd-quick (owner-approved) | todo filed | NOT STARTED, queued, interleave opportunistically | — |
| GOOD-TO-HAVES consolidation (needs manager doctrine call) | todo filed | NOT STARTED, blocked on manager ruling | — |

**P115 plan shape** (`115-PLAN.md`, 6 tasks / 5 waves): W1 — T1 A1-gate ‖ T2 latency
re-measure (both DONE, this wave); W2 — T3 ledger scaffold (next); W3 — T4 live-MCP
capture; W4 — T5 `token-economy.md` regen; W5 — T6 un-waive path + consolidation.

**Named incident this rotation (post-mortem, read before dispatching T3):** #32's T2
execution initially landed a sim `init=155ms` figure (`9384ca6`) sourced from a local
dev-VM cold/loaded first run — a noise outlier, not a stable measurement. On review
#32 caught this, re-derived from CI's own `bench-latency-v09` job (reproducible,
documented ubuntu-24.04 runner, runs every push), and corrected the doc in `3845b13`
BEFORE handing off — see §5 for the full ruling. **Lesson for T4/T5/T6:** prefer the
CI-run artifact as the canonical source over a local one-off re-run; if a local number
looks surprising relative to prior figures, re-derive from CI before committing it.

## 3. Binding constraints (carried verbatim, unchanged)

- **One tree-writer at a time**; tree-mutating work is serial (no per-agent worktrees —
  owner rejected them as over-engineering for current cadence).
- **ONE cargo invocation machine-wide** (check/build/test/clippy) — prefer `-p <crate>`
  over `--workspace`; VM has OOM-crashed on parallel builds.
- **No `--no-verify`**, ever, on any commit or push.
- **Push at green, then confirm CI green on `main` AFTER the push** — run
  `python3 quality/runners/run.py --cadence post-push --persist`; the
  `code/ci-green-on-main` (P0) probe asserts the NEWEST `ci.yml` run on `main`
  concluded success, not merely that some older green run exists. Never open the next
  phase over a red or pending main.
- **Commit-trailer format:** `Co-Authored-By: Claude <Model> <noreply@anthropic.com>` +
  `Claude-Session: <role-or-session-id>`.
- **Model tiering:** opus for complex/security work, sonnet for default execution,
  haiku for mechanical tasks; **never dispatch `fable` at a leaf**.
- **Leaf isolation:** any `reposix init`/sim-seed/`git commit`/`config` test setup runs
  in a throwaway `/tmp` clone, `cd`'d into in the SAME Bash invocation — never the
  shared repo. Mechanically enforced by `.claude/hooks/leaf-isolation-guard.sh`.
- **No tag push by any coordinator** — the manager cuts tags.
- **No git surgery on `main`** (no reset/rebase/reorder/amend of already-pushed
  commits).
- **Shared tree with the manager** — TARGETED staging only (`git add <path>`, never
  `-A`/`.`); do not touch `.planning/MANAGER-HANDOVER.md`.
- **LIVENESS doctrine:** bound every wait on a dispatched child; health-check quiet
  children on a ≤30min/≤1h timer; children poll CI INLINE or run synchronously — never
  idle-trust a background self-resume watcher alone.
- **Real-backend cadence:** source `.env` in the SAME invocation as `run.py`;
  TokenWorld protected pair `7766017`/`7798785` NEVER deleted.
- **Before ending any turn with background shells/monitors running**, note their task
  ids in visible output. **This rotation: NONE running** — verified clean at handover
  time, nothing to note for #33.
- **Manager (w1:p7) uses a POLLING model** — clear in-pane narration at each boundary
  IS the report; escalate actively only for owner-blocking moments.
- **Relieve past ~100k tokens of own context** (hard stop ~150k; **absolute, not %** of
  the window) at a wave boundary — write+commit a fresh handover, REPLACING this file,
  naming successor **#34**.

## 4. Litmus / gate / REOPEN state

- **CI gate:** `origin/main` tip `3278abc` — `code/ci-green-on-main` P0 **PASS** (run
  `29452237641`, completed/success). No REOPEN, no active gate failure.
- **Watch item carried from #31: flaky `test` CI job.** If a P115-execution push sees
  the `test` job go red, **re-run it once before treating it as real**; a
  REPEATED/reproducible failure across re-runs IS a real signal and must not be waved
  through.
- **Waiver / deadline clocks (carried, unchanged this rotation):**
  - `agent-ux` hero-number doc-alignment rows (8 total) — waiver expires
    **2026-08-15**. **P115 lifts these, but ONLY after T4/T5/T6 execution
    re-measures + T6's un-waive path — the clock is still live and is why P115 was
    front-loaded in the milestone.**
  - `structure/file-size-limits` — waiver expires **2026-08-08**. Gate-wide
    WARN-not-block: `SURPRISES-INTAKE.md`, `115-PLAN.md`, and this handover all sit
    over the 20000 B `*.md` ceiling non-blockingly under this waiver.
  - `perf-targets` — self-WAIVED until **2026-07-26**.
- **Milestone-close 9th probe** (`pre-release-real-backend`) not yet due — 13 phases
  remain after P115.
- **Intakes filed — do NOT re-file:** all prior-handover intakes (GTH-V15-21, the 2
  todos [roadmap-diagram, GOOD-TO-HAVES consolidation], GTH-16, GTH-V15-22,
  GTH-V15-23, GTH-V15-16) **PLUS NEW this rotation:** the sim `expected_version` PATCH
  defect (v0.15.0 `SURPRISES-INTAKE.md`, MEDIUM, OPEN, filed in `3845b13` — see §5 for
  the exact mechanism). **The latency.md-regeneration-clobber tension is NOT
  separately filed** — it is folded into the T6 notes below; #33 absorbs it into T6,
  don't re-file it as a new intake.
- **Ledger entries to DELETE at P115 close** (per each entry's own instruction, in
  `.planning/CONSULT-DECISIONS.md`): the A1 `[SELF]` entry (once T3 encodes it
  verbatim into the ledger header) and the P115-T2 canonical-CI-methodology `[SELF]`
  entry (once T6 encodes it into the un-waive path). Do not delete either before its
  named consumer has landed.

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

- **A1 (RESOLVED, encode-not-escalate):** ruled `[SELF]` in `.planning/CONSULT-
  DECISIONS.md` at `3278abc` — one "benchmark session" (≤50 ceiling) = one live
  agentic conversation/task run, NOT one metered API call. Failed/aborted runs count
  against the ceiling of 50. Sessions running >~5x the median token spend get flagged
  in the ledger (not silently absorbed). Escalate to the owner if the running total
  would cross 50. **T3's ledger header MUST record this ruling VERBATIM before T4
  spends session #1** — do not paraphrase.
- **P115-T2 canonical latency source (RULED `[SELF]`, `3845b13`):** re-measurement
  showed sim `init` is highly environment-dependent, not one fixed number: 27ms
  (legacy dev machine, superseded), 42-45ms (this dev VM, warm, N=3), 155ms (dev VM
  cold/loaded first-run outlier — the figure mistakenly landed in `9384ca6`), 278ms
  (CI `bench-latency-v09`, ubuntu-24.04 hosted runner, run `29452237641`, commit
  `3278abc`). **Canonical reference = CI `bench-latency-v09`** — it is reproducible,
  runs on a documented runner image, executes every push, and measures the sim PLUS
  all 3 real backends in one place. `docs/benchmarks/latency.md` now carries these
  canonical figures: `sim init=278ms/get=6ms`, `github init=830ms/get=320ms`,
  `confluence init=1136ms/get=202ms`, `jira init=329ms`. Legacy 24ms/27ms figures are
  superseded, not deleted (kept in the doc's provenance/methodology section). The
  sim `patch=Nms` figure carries a caveat: it times a 400-rejection path, not a clean
  patch (see the filed defect below).
- **⚠️ E3-ish positioning finding — NARRATED to manager (w1:p7) this rotation,
  non-blocking, carry forward for the owner:** the honest CI cold-init figure (~278ms)
  is roughly 10x the legacy "27ms" hero claim the docs currently lead with. This means
  T6's un-waive path requires CHANGING the doc's headline CLAIMS to match the CI
  figures, not merely re-verifying the old numbers still hold. This overlaps v0.21
  "benchmark honesty" scope. **The owner may want the headline reframed** (lead with
  token-economy / no-network-reads framing rather than raw init latency) — worth
  raising with the owner BEFORE T6 rewrites the claim prose. Honesty (i.e., landing
  the true number somewhere) proceeds regardless of how that framing call lands.
- **T4 GA intel — BANK THIS for the successor (high-confidence WebSearch, carried
  unchanged from planning, re-verify at T4 start per the plan):** Atlassian Rovo MCP
  reached GA (Feb 2026); headless endpoint `https://mcp.atlassian.com/v1/mcp` — use
  the **API-token** auth path, NOT the OAuth `/authv2` default (OAuth needs an
  interactive browser, incompatible with a headless benchmark session); covers both
  Jira and Confluence. GitHub's remote MCP server is also GA at
  `https://api.githubcopilot.com/mcp/`. Fallback if either GA path proves unreachable:
  `sooperset/mcp-atlassian` (self-hosted, Docker). Wire via `claude mcp add
  --transport http <name> <url> --header "Authorization: Bearer <token>"` →
  `.mcp.json`; reuse the existing `.env` creds (`ATLASSIAN_API_KEY`/
  `JIRA_API_TOKEN`/`GITHUB_TOKEN`). There is no native per-arm token accounting split
  — instrument BOTH transcripts (MCP-mediated and reposix-mediated) via the
  `session-analyzer` skill. `MAX_MCP_OUTPUT_TOKENS` (default 25k) MUST be identical
  across both arms or the comparison is invalid. Tool-search defers MCP schema
  loading, so any plan-time "MCP token overhead" estimate may be stale — re-measure,
  don't assume the planning-time number still holds. `claude mcp add`-registered
  servers run fine on subscription billing (no `ANTHROPIC_API_KEY` needed in-shell).
- **Noticing — sim PATCH `expected_version` defect (filed this rotation, `3845b13`,
  `SURPRISES-INTAKE.md`, MEDIUM, OPEN):** `bash quality/gates/perf/latency-bench.sh`'s
  PATCH probe sends a body containing `expected_version`; the reposix-sim issue-update
  handler's schema only accepts `title`/`body`/`status`/`assignee`/`labels`, so the
  probe's PATCH times a 400-rejection path, not a successful patch. Reproduced across
  3 consecutive local runs plus CI run `29452237641`. Out-of-scope for the discovering
  session (deciding the intended contract — accept `expected_version` for optimistic
  concurrency, or drop it from the bench body — touches sim request validation and/or
  the bench probe, a >1h scoped change). Sketched resolution recorded in the intake;
  do NOT re-file, do NOT eager-fix inside T3–T6 unless it blocks a task's `<verify>`.
- **Noticing — `latency.md` regeneration-clobber risk (CRITICAL for T5/T6, folded into
  T6, NOT separately filed):** `docs/benchmarks/latency.md` is designed to be
  machine-regenerated by `quality/gates/perf/latency-bench.sh` (sim-only, local run).
  A future local bench run will CLOBBER the CI-canonical figures, the real-backend
  rows, and the new provenance/methodology section just added in `3845b13`. T5/T6 MUST
  reconcile this before the phase closes — options sketched: (a) teach the generator
  script to pull CI figures instead of running locally, (b) move doc generation into
  the CI job itself so the artifact and its source are the same run, or (c) document
  the file as CI-sourced-and-NOT-locally-regenerable (add a header comment warning
  against running the local script over it). Part of T6's "encode the canonical-source
  methodology into the un-waive path" task.
- **RAISE LIST / open items carried forward, all still OPEN:**
  - **P116 ADR-010 packet** — after P115 execution closes, produce options+tradeoffs
    for ADR-01 (mirror-fanout) and FIX-03 (GTH-09 slug→id durable-create hazard),
    route to the **MANAGER (w1:p7) for ruling — no pre-ruling implementation.**
  - **roadmap-diagram gsd-quick** — owner-approved, small; todo
    `.planning/todos/pending/2026-07-15-public-birds-eye-roadmap-diagram.md`.
    Interleave opportunistically; touching any tracked doc in
    `quality/catalogs/doc-alignment.json` requires a `/reposix-quality-refresh` pass
    before the next push.
  - **GOOD-TO-HAVES consolidation** (two coexisting files: root
    `.planning/GOOD-TO-HAVES.md` vs `.planning/milestones/v0.15.0-phases/
    GOOD-TO-HAVES.md`) — needs a manager/owner **DOCTRINE CALL** before merging; todo
    `.planning/todos/pending/2026-07-15-consolidate-two-good-to-haves-files.md`. Do
    NOT merge unilaterally.

## 6. Precise next steps (successor #33 runbook)

1. **Re-verify §1 ground truth live**: `git rev-parse HEAD && git status --porcelain
   && git rev-list --left-right --count HEAD...origin/main && gh run list --branch
   main --workflow CI --limit 3`. Expect local `main` ahead of `origin/main`
   (unpushed, intentional — P115 mid-phase) and CI green on `3278abc`.
2. **Resume P115 EXECUTION at top-level** (`Execution mode: top-level`; do NOT route
   via `/gsd-execute-phase`/`gsd-executor`). Anti-sink lesson still binding: don't
   linear-read `115-PLAN.md`/`execute-phase.md` cold; delegate heavy reads to
   `reader-digester`. For T3's exact ledger schema, read `115-PLAN.md` around
   file:250-272 (or grep for "Task 3").
3. **Wave 2 — T3 (session-spend ledger scaffold, ZERO session budget):** create
   `benchmarks/bench-session-ledger.md` (deliberately OUTSIDE `docs/` to dodge the
   mkdocs orphan-doc invariant). Header = the A1 ruling VERBATIM (session unit + ≤50
   ceiling + escalate-past-50-to-manager). Columns per plan: `#`, timestamp
   (UTC ISO-8601), backend, arm (mcp-mediated / reposix-mediated), task,
   unit_consumed, running_total, artifact_produced. **Commit the EMPTY schema (zero
   data rows) BEFORE any session spend** — do not backfill. Mechanical ≤50 check:
   `tail -1 … | awk -F'|' '{v=$(NF-2); gsub(/ /,"",v); exit (v+0>50)}'`.
4. **Wave 3 — T4 (live-MCP token capture, ≤50 sessions, budget-tracked)** — the
   expensive wave; use the §5 GA intel. Two arms per sanctioned target
   (MCP-mediated → replace `benchmarks/fixtures/mcp_jira_catalog.json`;
   reposix-mediated → replace `benchmarks/fixtures/reposix_session.txt`, set up in a
   throwaway `/tmp` clone), median-of-3 each. Append ONE ledger row per session;
   verify `running_total ≤ 50` BEFORE starting the next session; scrub credentials
   from every artifact before committing. Each live agentic run = 1 session (A1).
   Escalate to the manager/owner before crossing 50. **This wave likely warrants
   relief BEFORE starting if #33 is already deep into its own context budget** — it is
   the largest remaining wave.
5. **Wave 4 — T5 (regen `token-economy.md` from real fixtures):** run
   `bench_token_economy.py` (one networked count for the new `*.tokens.json`
   sidecars); confirm offline stability afterward; rewrite the provenance section
   (remove any "modeled on"/`scripts/demo.sh` claims that are no longer true);
   methodology note (MCP server used + its GA status + the task definition); record
   the re-measured token-reduction figure. **Reconcile the latency.md-clobber
   tension** (§5) as part of this wave or hand it explicitly to T6.
6. **Wave 5 — T6 (un-waive path + consolidation):** write `115-UNWAIVE-PATH.md` naming
   the exact script/line for `perf/token-economy-bench` and
   `perf/headline-numbers-cross-check` (CONFIRMED ABSENT from the repo — authoring
   them is deferred, NOT P115 scope, name them as future work); map the 3 fresh
   figures to the 8 doc-alignment rows; encode the canonical-CI methodology into the
   un-waive path; confirm P118/DOCS-05 consumes the docs directly; re-assert the
   ledger's `running_total ≤ 50`. Resolve the `latency.md` regeneration-clobber
   tension here if not already done in T5. Note the E3 positioning heads-up (§5) — if
   the owner has reframed the headline by this point, align T6's doc edits to it.
7. **Push at PHASE CLOSE (before the verifier), not before:** `git push origin main`
   (pushes all accumulated P115 commits at once); then `python3
   quality/runners/run.py --cadence post-push --persist` — `code/ci-green-on-main` P0
   MUST be green. Then dispatch the verifier subagent. Never open the next phase over
   a red or pending main.
8. **After P115 closes:** open **P116 ADR-010 packet** (ADR-01 mirror-fanout + FIX-03
   GTH-09 slug→id) → route to the MANAGER for ruling, NO pre-ruling implementation.
9. **roadmap-diagram gsd-quick + GOOD-TO-HAVES consolidation** — interleave/flag to
   the manager for a doctrine call; do NOT merge unilaterally.
10. **Report to the manager (w1:p7)** at each boundary (T3/T4 close, session-budget
    milestones, P116 routing). The manager POLLS — clear in-pane narration at each
    boundary IS the report; escalate actively only for owner-blocking moments.
11. **Relieve past ~100k own-context tokens** (hard stop ~150k, absolute not %) at a
    wave boundary — dispatch `relief-handover-writer`, which writes+commits a fresh
    `.planning/SESSION-HANDOVER.md` that REPLACES this file, naming successor **#34**.

**Background shells/monitors: NONE running** (verified clean at this handover).
Nothing for #33 to note.

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>
Claude-Session: relief-handover-writer
