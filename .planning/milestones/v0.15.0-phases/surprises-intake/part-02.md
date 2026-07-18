# v0.15.0 Surprises Intake — Part 2 of 7

> Split from `SURPRISES-INTAKE.md` for the file-size gate (OP-8 drain). Index: `../SURPRISES-INTAKE.md`. Entries preserved verbatim.

## 2026-07-14 20:42 | discovered-by: L0 rotation #22 (t4 real-backend re-run, pre-release-real-backend cadence) | severity: MEDIUM

**What:** `pre-release-real-backend` cadence needs a documented mirror-refresh pre-step. The vision-litmus non-idempotency (documented in `.planning/milestones/v0.14.0-phases/surprises-intake/part-03.md` § litmus non-idempotency, Manager Ruling #2): the litmus's own successful push re-stales the GitHub mirror it reads, so a SECOND-run vision-litmus is RED unless `scripts/refresh-tokenworld-mirror.sh` runs FIRST. This is not wired into the cadence invocation. This caused a P0 vision-litmus FAIL in the 2026-07-15 t4 run that was a false-negative — the committed catalog PASS from 2026-07-13 remains legitimate on a freshly-refreshed mirror.

**Why out-of-scope for the discovering session:** Wiring a mandatory pre-step into the milestone-close cadence invocation (or making the litmus self-reconcile) is a harness/procedure change needing its own review, not an eager patch mid cadence-run; it also overlaps `GTH-V15-09` (self-reconcile path already filed in `GOOD-TO-HAVES.md`) and needs the same owner-gate discipline as that entry.

**Sketched resolution:** Milestone-close should either run `scripts/refresh-tokenworld-mirror.sh` as a documented pre-step of `pre-release-real-backend`, or the litmus should self-reconcile per the fix-sketch already in `GTH-V15-09` (fetch backend-current through the reposix bus remote before the marker push). This entry cross-refs `GTH-V15-09` and documents the concrete cadence-wiring gap plus the specific 2026-07-15 false-negative it caused, so the next reader does not re-diagnose from scratch.

**STATUS:** RESOLVED — 2026-07-16 [MANAGER] ruling (`.planning/CONSULT-DECISIONS.md`,
commit `8212373`): webhook + 30-min cron blessed as the AUTHORITATIVE external-mirror
convergence mechanism; the `scripts/refresh-tokenworld-mirror.sh` litmus pre-step remains
documented manual op-recovery, NOT a product fix; `GTH-V15-38` tracks the elective Option
C (post-write snapshot fan-out) upgrade if the pre-step becomes a real recurring incident.

## 2026-07-14 20:43 | discovered-by: L0 rotation #22 (t4 real-backend re-run — preflight vs runner env-loading gap) | severity: HIGH

**What:** `quality/runners/run.py` does not source `./.env`, while `scripts/preflight-real-backends.sh` DOES — causing a false-green-preflight / silent-skip gap: the real-backend cadence skips all rows when run without `.env` pre-sourced into the shell, but preflight independently reports the backends reachable, so the two together give a false impression of full coverage. CONFIRMED this rotation: sourcing `.env` in the same invocation (`set -a; . ./.env; set +a`) fixed it — all 6 real-backend rows executed for real once the env vars were present in `run.py`'s process.

**Why out-of-scope for the discovering session:** The immediate re-run was unblocked by manually sourcing `.env` before invoking `run.py`; fixing `run.py` itself (or its documented invocation) is a code/docs change to the shared quality-runner infra, not an eager patch inside a single cadence re-run, and the "fix it twice" doctrine (root CLAUDE.md meta-rule) requires updating the doc references in the same change.

**Sketched resolution:** Make `run.py` source `.env` itself (e.g. load it via Python's own env-loading at startup), OR bake `set -a; . ./.env; set +a` into the documented invocation everywhere `pre-release-real-backend` is referenced. Fix-it-twice: update the `pre-release-real-backend` doc references in `.planning/CLAUDE.md` and `docs/reference/testing-targets.md` to show the corrected invocation (or note that `run.py` now self-sources), so the next agent does not silently skip-and-declare-green again.

**STATUS:** RESOLVED — Phase 123 SC1 (`a99d87cb`; `quality/runners/_env_load.py`, `run.py`
self-sources `./.env` when present).

## 2026-07-14 20:44 | discovered-by: L0 rotation #22 (t4 real-backend re-run — `--persist` write review) | severity: HIGH

**What:** `--persist` silently rewrites genuinely-GREEN catalog rows to a worse status on a skip/false-negative. This rotation's run downgraded `vision-litmus` PASS→FAIL in the persist write — caught and `git restore`d before commit only because the diff was reviewed before staging. The prior rotation downgraded 2 P0 rows the same way on an env-skip (see the preceding env-loading-gap and mirror-staleness entries above — either false-negative feeds directly into this persist behavior, compounding the risk).

**Why out-of-scope for the discovering session:** Changing `--persist`'s write semantics (adding a confirm gate) is a change to the shared quality-runner framework's core write path — needs its own review given how many cadences depend on `--persist`, not an eager patch mid a real-backend re-run where the immediate risk was mitigated by manual `git restore` + review-before-commit.

**Sketched resolution:** Gate skip/fail-driven `status` rewrites behind an opt-in flag (e.g. `--allow-downgrade`), or refuse by default to rewrite a committed GREEN `status` to a worse value without an explicit confirm flag — surfacing a loud warning instead ("row X was PASS, new result is FAIL/SKIP — pass --allow-downgrade to persist this change") so a false-negative run cannot silently corrupt catalog state. Pairs with the preceding env-loading-gap and mirror-staleness entries as root causes that feed spurious downgrades into this behavior.

**STATUS:** RESOLVED — Phase 123 SC2 (`584b6691`; `--allow-downgrade` opt-in, refuse-by-default
guard in `quality/runners/_persist_guard.py`).

## 2026-07-15 06:30 | discovered-by: L0 rotation #26 intake-filing leaf (carried forward across workhorse #24→#25→#26 handovers, 2 rotations un-filed) | severity: MEDIUM

**What:** The commit-message argument to `gsd-sdk query commit` is POSITIONAL, not a `--message` flag. Passing `--message "..."` silently commits a garbage/empty message instead of erroring — a real footgun for any agent copying the pattern.

**Why out-of-scope for the discovering session:** Discovered incidentally during unrelated work; correcting the documented example other agents copy from touches a user-global skill (`coordinator-dispatch`, outside this repo) and/or `.planning/ORCHESTRATION.md`, which needs a deliberate review pass rather than an eager patch mid another charter. Carried un-filed across two prior rotation handovers before this leaf captured it.

**Sketched resolution:** Fix-twice obligation: (i) this intake row captures the footgun; (ii) correct the documented example other agents copy from — the `coordinator-dispatch` skill and/or `.planning/ORCHESTRATION.md` commit example — to the positional form `gsd-sdk query commit "<msg>" --files <path>`, so no future example teaches the `--message` flag form. Also consider whether `gsd-sdk query commit` itself should hard-error on an unrecognized `--message` flag rather than silently committing garbage, closing the footgun at the source rather than only in docs.

**STATUS:** OPEN

## 2026-07-15 06:35 | discovered-by: manager amendment 4 to L0 rotation #26 (measured on this rotation's push; corroborates workhorse #25's 101s WARN) | severity: LOW-MEDIUM

**What:** Pre-push hook wall-clock measured **~1:31.68 (91.7s)** this rotation and **~101s** on workhorse #25's final push, vs the **~55–60s** budget documented in `quality/CLAUDE.md` § Cadences. Likely driver: `code/shell-coverage` (kcov shell coverage) measured **56.2s this run** vs the **~29s** figure documented in § Cadences — roughly 2×. Pre-push cost is a fixed whole-repo cost (NOT diff-size-scaled), so this is a genuine creep, not a big-diff artifact.

**Why out-of-scope for the discovering session:** Surfaced from a timing measurement taken during an intake-filing leaf's own push, not a planned perf-investigation phase; diagnosing whether kcov crept (corpus growth, toolchain/version change, VM contention) and deciding baseline-vs-regression is its own bounded investigation, not an eager patch mid intake-filing. Pre-push is WARN (not FAIL) so no gate blocked — no urgency to fix inline.

**Sketched resolution:** Fix-twice obligation: (i) investigate whether kcov shell-coverage crept — more `.sh` files under coverage (corpus growth), a kcov/toolchain version change, or transient VM contention; re-measure `code/shell-coverage` in isolation on a quiet VM to separate contention from real cost; (ii) if the higher figure is a legitimate new baseline, update the § Cadences documented number (~29s → measured, and the ≈55s pre-push aggregate → measured) to match reality; if it's a regression, find and fix the cause (e.g. trim the covered corpus, cache kcov output, or parallelize where the cargo-mutex allows). Corroborated across two rotations (#25 ~101s, #26 ~91.7s), so it is a stable creep rather than a one-off flake.

**STATUS:** OPEN

## 2026-07-15 21:45 | discovered-by: P115-T2 (BENCH-01 live latency re-measurement) | severity: MEDIUM

**What:** latency-bench PATCH probe sends unsupported `expected_version` → times an error path (sim patch figure invalid). `bash quality/gates/perf/latency-bench.sh` emits, 3x per run, `ERROR reposix_sim::error: json error error=unknown field 'expected_version', expected one of 'title','body','status','assignee','labels'`; reproduced across 3 consecutive local runs 2026-07-15 and present in CI run 29452237641. The `patch=Nms` row therefore times a 400 rejection, not a successful patch. Mechanism: the bench sim PATCH probe constructs a no-op PATCH body containing `expected_version`; the reposix-sim issue-update handler schema only accepts title/body/status/assignee/labels → 400.

**Why out-of-scope for the discovering session:** Deciding the intended contract first (accept `expected_version` for optimistic concurrency, or drop it from the bench body) touches reposix-sim request validation and/or the bench probe — a >1h scoped change, not an eager fix inside a re-measurement task.

**Sketched resolution:** Either (a) add `expected_version` to the sim's accepted update fields if optimistic-concurrency is intended, or (b) drop `expected_version` from the bench PATCH body. Decide the intended contract first, then fix the losing side (sim schema or bench probe) to match.

**STATUS:** OPEN

## 2026-07-15 22:00 | discovered-by: P115 roadmap gsd-quick noticing (OD-3) | severity: LOW

**What:** `docs/development/roadmap.md` is a STALE internal snapshot that lies about the
active milestone. Its "Active milestone" section (L20-22) still reads "**v0.11.0 Polish &
Reproducibility** — PLANNING (Phases 50–55 scaffolded)", and its shipped-milestones table
(L18) stops at v0.10.0 — reality is v0.15.0 (Floor). The file is `not_in_nav` (not linked
from mkdocs nav), so it does not surface to readers via the docs site, but it remains a
committed artifact an agent or contributor could stumble on and trust. Now that a public
`docs/roadmap.md` exists as the canonical current-state surface, this internal snapshot's
staleness is more conspicuous — it duplicates a job the public doc already does, but worse.

**Why out-of-scope for the discovering session:** Surfaced incidentally by a P115 roadmap
gsd-quick lane (OD-3 noticing obligation), not a planned docs-freshness pass; deciding
whether to refresh the snapshot to current state or replace it with a redirect/pointer to
`docs/roadmap.md` is a small but distinct doc-hygiene decision, not an eager one-line patch
inside the P115 benchmark-remeasurement charter.

**Sketched resolution:** Either (a) refresh `docs/development/roadmap.md`'s "Active
milestone" section + shipped table to the current v0.15.0 (Floor) state, or (b) replace its
content with a short redirect/pointer to the now-canonical public `docs/roadmap.md`, so the
internal file cannot drift out of sync again. Prefer (b) if the internal file's only purpose
was to duplicate what `docs/roadmap.md` now covers — one source of truth beats two.

**STATUS:** OPEN

## 2026-07-15 17:18 | discovered-by: L0 rotation #36 (read-only pre-push-spike diagnosis, charter item 2) | severity: LOW

> **Root-cause deep-dive for the existing `2026-07-15 06:35` pre-push-timing item above — enriches, does NOT duplicate.** The drain phase should resolve both together.

**What:** Root-caused the pre-push over-budget WARN (rotation #35 saw 109s; #25/#26 saw ~101s/~91.7s; budget doc says ≈55–60s). It is **mostly environment variance layered on a modest kcov-corpus-growth creep** — NOT a new gate and NOT a stable new floor. Evidence: (1) a fresh `python3 quality/runners/run.py --cadence pre-push` on the identical commit state measured **64s total**, proving large run-to-run variance on unchanged code; (2) the dominant row `code/shell-coverage` (kcov) genuinely grew from the documented **29s** (measured 2026-07-12 08:21, commit `fc8264d`) to **~37s now**, because two MORE kcov-traced shell harnesses landed hours *after* the budget doc was written that same day — `fbb7782` (08:42, `60-code-ci.sh`, 7 stub-binary invocations) and `fe8febb` (18:52, `real-backend-env-gate.sh`, 2 scrubbed-env invocations) — neither reflected in the "≈55s" budget text. Timed breakdown of the 64s run: `code/shell-coverage` 37.16s · `agent-ux/rebase-recovery-reconciles` 9.14s · `hook-throttle` 2.02s · `mkdocs-strict` 1.98s · `badges-resolve` 1.96s · `no-orphan-docs` 1.89s · `fleet-safety-leaf-isolation-enforce` 1.32s · ~45 other rows sub-1s (most 0.03s). So the "≈55–60s" budget is **stale** (never re-baselined after the two post-08:21 harness additions), AND #35's specific 109s is the high-variance tail of a distribution now centered nearer ~65–75s. (Adjacent, noticed not filed separately: `60-code-ci.sh` rebuilds a stub `gh` binary + fresh PATH in a `mktemp` dir on each of its 7 invocations — more IO-syscall-heavy per call than peer harnesses, a plausible amplifier of VM I/O-contention variance. `docs-alignment/link-resolution` double-counts `docs/index.md` — cosmetic, ALREADY noted in the handover, do NOT re-file.)

**Why out-of-scope for the discovering session:** Rotation #36's charter item 2 was explicitly a **read-only investigation — "file findings, change nothing."** Re-baselining the budget doc + raising the WARN threshold (both mutating edits) were out of charter, so filed rather than applied.

**Sketched resolution:** Re-baseline `quality/CLAUDE.md` § Cadences pre-push budget from ≈55–60s to **~75s** (median of several post-corpus-growth runs) and raise the WARN threshold from 90s to **~100s**, so normal kcov ptrace/IO jitter stops flagging as noise. Optionally, reduce `60-code-ci.sh`'s per-invocation stub-`gh` rebuild churn (build once, reuse) to shave the variance amplifier. Full evidence: this rotation's read-only diagnosis (no code touched); base commit `1b20c15`.

**STATUS:** OPEN

## 2026-07-16 05:00 | discovered-by: P115 Task-4 capture executor (L0 #38) | severity: BLOCKER

**What:** The P115 Task-4 live-MCP-vs-reposix token benchmark (6 capture sessions = 1 backend × median-of-3 × 2 arms) **cannot run as specified**. Three independent findings, each verified this rotation: **(1)** the ratified `atlassian-rovo` MCP server (`https://mcp.atlassian.com/v1/mcp`, `atlassian-mcp-server` v1.0.0) advertises **only 3 Teamwork Graph tools** (`getTeamworkGraphContext`, `getTeamworkGraphObject`, `addTeamworkGraphContext`) — there is NO `editJiraIssue`/`createJiraIssue`/`updateJiraIssue`/JQL-search tool, so the benchmark task's "edit 1 issue" step has no tool (`addTeamworkGraphContext` mutates the relationship graph only, not issue fields). The synthetic fixture assumed a full-CRUD server (`sooperset/mcp-atlassian`), a DIFFERENT server. **(2)** A real `tools/call` (`getTeamworkGraphContext` on `JiraSpace KAN`) with the ratified Bearer API token is **permission-denied**: `"You don't have permission to connect via API token. Please ask your organization admin for access."` — tools LOAD (handshake 200) but do NOT function with this credential. This closes the explicit open caveat (1) in `115-ROVO-AUTH-CHECK.md` (tool-level authz was never verified) with a negative result. **(3)** Jira project **KAN has 0 issues** — `reposix init jira::KAN` synced to an empty tree (`sync(jira:KAN): 0 issues`, `git ls-tree HEAD` empty), so neither arm can "read 3 issues"; this blocks the reposix arm independent of the MCP findings. Only 1 of 50 sessions was spent (the smoke test); no numbers were fabricated.

**Why out-of-scope for the discovering session:** Every unblock path is a charter/owner decision that also changes what the benchmark measures and/or spends the capped 50-session budget: (A) grant the token graph access + redefine the mcp-arm task to a read+link workflow (issue-field edits are impossible on this server); (B) swap the mcp-arm to `sooperset/mcp-atlassian` (full CRUD, needs setup + egress-allowlist review); (C) seed KAN with ≥3 issues (unblocks only the reposix arm). The executor delivered the honest, no-decision-needed artifacts (real tool catalog, grounded server-choice note, smoke-session capture, ledger row) and escalated rather than unilaterally redefine the ratified task, swap servers, or seed the backend.

**Sketched resolution:** Owner picks A / B / C (or a combination). Reposix arm needs C regardless. MCP arm needs A or B. If A, update `115-MCP-SERVER-CHOICE.md` and the benchmark task definition to the read+link workflow and note the arms are no longer read+field-edit-comparable. If B, register `sooperset/mcp-atlassian`, extend `REPOSIX_ALLOWED_ORIGINS` review, re-run the smoke test, then the 6 captures. Evidence: `benchmarks/fixtures/mcp_jira_catalog.json` (real 3-tool surface), `benchmarks/captures/mcp-kan-smoke.json`, `.planning/phases/115-live-mcp-benchmark-re-measurement/115-MCP-SERVER-CHOICE.md` (§ Blockers + § Recommendation).

**STATUS:** **RESOLVED (2026-07-16, L0 #39)** — path (D, not A/B/C): **[SELF] pivot to the GitHub backend** instead of Jira. All 6 captures landed on `github-probe` + `reubenjohn/reposix` (ledger rows 2–7, `running_total` 7/50); the Jira/`atlassian-rovo` findings above are retained as the evidence trail. Jira remains addable later if org-admin API-token + a CRUD Jira MCP are provisioned. See `115-MCP-SERVER-CHOICE.md` § LIVE-CAPTURE CHOICE.

