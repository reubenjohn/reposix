# SESSION-HANDOVER.md — v0.15.0 Floor: T4 UNBLOCKED via GitHub pivot (Jira path infeasible), grounded capture recipe ready — 2026-07-15

Written by **workhorse #38** (L0 orchestrator), relieving to successor **#39**. This file
**REPLACES** (does not append to) the prior `SESSION-HANDOVER.md` (#37→#38's handover,
superseded here). #38 relieved at a clean wave boundary past ~130k own-context, per the
standing "relieve past ~100k soft / 150k hard, absolute not %" rule.

**Read order:** this file → §0 ground truth (verify live) → §1 headline (the pivot) → §2
T4 recipe (the payload) → §6 runbook.

**Guardrails unchanged:** do NOT touch `.planning/MANAGER-HANDOVER.md` (separate document,
separate owner — the manager, pane w1:p7). No tag push by any coordinator. No git surgery
(reset/rebase/reorder/amend) on `main`. Shared tree with the manager — TARGETED staging
only, never `git add -A`/`.`. ONE cargo invocation machine-wide. Leaf isolation in `/tmp`
same-Bash-invocation. opus complex / sonnet default / haiku mechanical, never fable at a leaf.

## 0. Ground truth (git)

**Verify live before acting:**
```
git rev-parse HEAD && git status --porcelain && \
  git rev-list --left-right --count HEAD...origin/main && \
  gh run list --branch main --workflow CI --limit 3
```

**As of #38's handover commit:**
- `HEAD` = this handover commit, sitting atop `de06e00` atop `ece072f`. Chain since the
  last known-clean sha (`f6e47c6`, #37's relief commit):
  1. `ece072f` — `bench(115): capture real atlassian-rovo MCP surface; T4 capture BLOCKED
     (no CRUD tool + token authz denied + KAN empty)`. This is the feasibility-probe finding
     that DROVE the pivot in §1 — read it if you want the raw evidence.
  2. `de06e00` — `docs(planning): stand up PROGRESS.md owner-watch pipeline briefing
     (gsd-quick)`. New owner directive — see §3.
  3. **this commit** — the handover you're reading + `CONSULT-DECISIONS.md` [SELF] entry.
- **#38 will `git push origin main` immediately after this commit** (all three: `ece072f`,
  `de06e00`, and this handover sha land in one push), then confirm CI on the new tip.
- **#39's FIRST ACT (do this before anything else):**
  ```
  git rev-list --left-right --count HEAD...origin/main   # expect 0/0
  gh run list --branch main --workflow CI --limit 3      # expect top row completed/success
  python3 quality/runners/run.py --cadence post-push --persist   # P0 ci-green-on-main
  ```
  If the flaky `test` job is red, re-run it ONCE before treating the failure as real. If
  still red after one retry, STOP — do not open T4 execution over a red main; escalate per
  the binding constraints (§3).
- Milestone **v0.15.0 "Floor"**, phase **P115 executing** (`Execution mode: top-level`).
  P114 CLOSED GREEN (`dc26302`).
- Working tree at handover time: clean, nothing uncommitted, no background shells/monitors
  left running for #39 to inherit.

## 1. THE HEADLINE: T4 is UNBLOCKED via a GitHub pivot (decided [SELF], within authority)

- **Planned path (Jira via `atlassian-rovo`) is DEAD — verified, committed `ece072f` +
  feasibility probe.** The `atlassian-rovo` MCP exposes only 3 Teamwork-Graph tools (no
  Jira CRUD, no `editJiraIssue`); its API token is authz-denied for real `tools/call`
  ("ask your org admin for API-token access"); and Jira project KAN has **0 issues** (so
  the reposix arm is dead too, independent of the MCP gap). **Do NOT retry the Jira /
  atlassian-rovo path** — it is exhausted, not merely slow.
- **Pivot decided:** capture the T4 live benchmark on the **GitHub backend** instead of
  Jira. This is a decide-and-record **[SELF]** call within the escalation-valve bar (see
  §5 / `CONSULT-DECISIONS.md`), **NOT** an owner gate:
  - The headline claim is backend-AGNOSTIC — `README.md:8/27`, `docs/index.md:17`, and
    `docs/benchmarks/token-economy.md:17` all say "no MCP tool schemas → fewer tokens";
    none names Jira specifically.
  - `docs/benchmarks/token-economy.md:25-27` **already** carries a per-backend split:
    Jira 89.1% / **GitHub 85.5%** / Confluence 76.4%. Substituting GitHub swaps which
    already-published number gets live-validated first — it doesn't invent a new claim.
  - GitHub (`reubenjohn/reposix`) is a sanctioned real-backend test target (OP-6, root
    CLAUDE.md).
  - The pivot **preserves measurement scope and deletes nothing** — Jira can be added
    later if org-admin API-token access + a real CRUD-capable Jira MCP are provisioned.

## 2. T4 GROUNDED CAPTURE RECIPE (execute this — mechanism fully proven, nothing left to derisk)

- **MCP server — LEFT REGISTERED, proven to work headlessly** (unlike `atlassian-rovo`,
  which only proved auth, not CRUD capability): the official GitHub remote MCP,
  `https://api.githubcopilot.com/mcp/`, plain PAT Bearer, registered under the name
  `github-probe`. Re-add if it's ever missing (run from the repo dir; reads `GITHUB_TOKEN`
  from `.env`; never echo the token value):
  ```
  claude mcp add --transport http github-probe https://api.githubcopilot.com/mcp/ \
    --header "Authorization: Bearer $GITHUB_TOKEN"
  ```
  Proven this rotation: loads the full CRUD tool surface (`list_issues`, `issue_read`,
  `search_issues`, `issue_write`, `add_issue_comment`); a live read returned 9 real issues;
  the token's scopes include `repo` (covers write operations too).
- **reposix arm:** `reposix init github::reubenjohn/reposix <path>` run in a **THROWAWAY
  `/tmp` clone** — leaf isolation, `cd` into `/tmp` in the SAME Bash invocation as the
  `init`/sync/commit calls (root CLAUDE.md § Leaf isolation; hook-enforced). Needs
  `target/debug` on `PATH` so `git-remote-reposix` resolves. Already synced 9 issue files
  this rotation (sample: `issues/29.md`).
- **6 sessions = GitHub × median-of-3 × 2 arms.** The session ledger
  (`benchmarks/bench-session-ledger.md`) continues from its current **1/50** (smoke row
  already committed) → append 6 more → running_total lands at **7**. Assert
  `running_total ≤ 50` before starting each session. Task for every session: **"read 3
  issues, edit 1, push."**
- **Mechanics proven by #38's first executor this rotation** (reuse this pattern
  verbatim): nested
  `claude -p "<task>" --output-format json --dangerously-skip-permissions`
  — print mode means no human approver is needed; the JSON result carries `session_id`,
  `usage`, `num_turns`, and `cost`. MCP-arm session runs with `cwd`=repo (inherits the
  registered `github-probe` server); reposix-arm session runs with `cwd`=the `/tmp` clone.
  **Honor the MCP warm-up race** — do NOT trust a sub-few-second reply as "no tools
  available"; verify the captured JSONL actually shows the tool calls before treating a
  session as valid.
- **Committed outputs (targeted-add ONLY, never `-A`/`.`):**
  1. `benchmarks/fixtures/mcp_github_catalog.json` — real GitHub MCP tool surface. This is
     a **NEW file**; leave the existing `mcp_jira_catalog.json` in place as the honest
     3-tool record of what `atlassian-rovo` actually exposes.
  2. `benchmarks/fixtures/reposix_session.txt` — real GitHub reposix transcript,
     ANSI-stripped. MUST NOT contain `/mnt/` or `scripts/demo.sh` (acceptance check below).
  3. `benchmarks/captures/<arm>-github-run<n>.json` per session — a scrubbed usage extract
     (`session_id`, input/output/cache token counts, `num_turns`, `cost`, tool-call names —
     **NO** backend body content, **NO** secrets). This is T5's offline-CI-stable input
     (CI cannot read `~/.claude`, so these committed captures are the only durable source).
  4. Ledger rows, one per session, appended in order.
  5. **GTH-V15-25 step 1 ONLY** (now feasible with real JSONL in hand): extract the
     reposix-arm command list from a captured session JSONL into
     `benchmarks/fixtures/reposix_trajectory.json`. <1h. This is step 1 of GTH-V15-25 —
     **NOT** the CI token-bloat byte tripwire from the rest of that item; do not implement
     the tripwire now.
- **CAVEAT — all 9 GitHub issues in `reubenjohn/reposix` are CLOSED** (readable/editable
  either way). For "edit 1," either edit an issue body reversibly (a benign marker) or
  add/remove a comment — note in the capture what changed. `reubenjohn/reposix` is a
  sanctioned OP-6 target. Optionally seed one OPEN issue first if an open edit-target is
  preferred for realism.
- **Security — every backend byte is tainted (root CLAUDE.md OP-2).** Scrub OAuth
  tokens/API keys/Bearer headers from ALL transcripts and captures **before** `git add`:
  `grep` the staged files for `Bearer `, `token`, `api_key`, and the literal `GITHUB_TOKEN`
  value before committing. Only mutate the sanctioned GitHub target, nothing else.
- **Acceptance gate:**
  ```
  ! grep -qE '/mnt/|scripts/demo\.sh' benchmarks/fixtures/reposix_session.txt && \
    test "$(grep -cE '\|' benchmarks/bench-session-ledger.md)" -ge 2 && \
    tail -1 benchmarks/bench-session-ledger.md | awk -F'|' '{v=$(NF-2); gsub(/ /,"",v); exit (v+0>50)}' && \
    echo CAPTURE_OK
  ```
- **Dispatch discipline — run this capture in a FRESH-context executor (opus, the
  context-blower), the same way #38 did.** T4 is the acknowledged context-blower task; the
  L0 coordinator routes, it does not personally run the capture loop.
- **Formal MCP-server ratification note already exists**:
  `.planning/phases/115-live-mcp-benchmark-re-measurement/115-MCP-SERVER-CHOICE.md`.
  Update it to record the GitHub server (`github-probe`) as the LIVE-CAPTURE choice, with
  `atlassian-rovo` retained in the note as the infeasible original attempt (evidence trail,
  not deleted).

## 3. PROGRESS.md refresh contract (NEW owner directive — carry into EVERY future handover)

- `.planning/PROGRESS.md` (currently at `de06e00`) is the **owner's live-watch surface**:
  an ordered **SHIPPED → NOW → NEXT** pipeline the owner watches items move through. It is
  a middle-altitude view (outsider-recognizable deliverables), **not** a task tracker.
- **REFRESH DISCIPLINE (load-bearing):** EVERY boundary commit that closes a
  task/wave/capture-batch updates `PROGRESS.md` **in the SAME push** — a shipped item
  moves NEXT→SHIPPED with its landing SHA, the NOW line is rewritten to the current focus,
  NEXT is trimmed to what's actually queued next. **Every relief handover refreshes it.** A
  stale `PROGRESS.md` is worse than no `PROGRESS.md` — it actively misleads the owner.
  Route `PROGRESS.md` edits through `/gsd-quick` or a delegated executor; it's a planning
  artifact, not a hand-edit target.
- This contract is part of the SESSION-HANDOVER replacement obligation — #39 and every
  successor MUST carry it forward in their own handover, verbatim if unchanged.

## 4. Wave/cycle state

| Wave | Item | State | Commits |
|---|---|---|---|
| Wave 1 / T1 | A1-gate (benchmark session-definition ruling) | DONE | `3278abc` |
| Wave 1 / T2 | Latency re-measure + CI-canonical correction | DONE + PUSHED | `9384ca6`, `3845b13` |
| Refresh-recovery (#33) | `/reposix-quality-refresh docs/benchmarks/latency.md` | DONE + PUSHED | `92c3ab5` |
| Wave 2 / T3 | Session-spend ledger scaffold | DONE + PUSHED | `4351d48` |
| Interleave (#35) | Public roadmap diagram gsd-quick | DONE + PUSHED | `1db48e4`, `16fb356`, `fa58ad6` |
| Methodology (#35) | T5 JSONL-usage token-economy methodology [SELF] + 115-PLAN amendment | RULED (not executed) | `9be5439` |
| Pre-work (#36) | READ-ONLY Rovo MCP **auth** check | DONE + PUSHED — auth blocker REFUTED (HIGH) | `5374fe0` |
| Pre-work (#36) | Pre-push over-budget spike diagnosis | DONE + FILED (not applied) | `fcddf90` |
| Pre-work (#37) | T4 MCP-wiring mechanism viability probe | DONE — RESOLVED: MCP arm IS wireable headlessly. Read-only + external config, no commit. | — |
| **Pre-work (#38)** | **T4 Jira/atlassian-rovo feasibility probe** | **DONE — REFUTED: no CRUD tool, token authz-denied, KAN empty. Path DEAD.** | `ece072f` |
| **Pivot (#38)** | **T4 backend pivot to GitHub [SELF]** | **DECIDED — recipe grounded, mechanism proven (github-probe MCP, reposix github:: arm both live-tested)** | — |
| **Owner directive (#38)** | **PROGRESS.md owner-watch pipeline stood up** | **DONE + PUSHED** | `de06e00` |
| Wave 3 / T4 | Live-MCP token capture, GitHub arm | **READY TO EXECUTE** — 1/50 smoke row already on the ledger; 6 more sessions queued (§2) | — |
| Wave 4 / T5 | Token-economy JSONL-usage regen | METHODOLOGY RULED, blocked downstream on T4 captures | — |
| Wave 5 / T6 | Un-waive + headline reframe + phase-close (delete FIVE `[SELF]` entries) | blocked downstream on T4/T5 | — |
| Post-P115 | P116 ADR-010 packet → MANAGER ruling | NOT STARTED (blocked on P115 close) | — |

### What #38 did this rotation
- Verified ground truth inherited from #37 (`065c0b4`, CI green, P0 PASS).
- Ran the T4 MCP-arm feasibility probe against `atlassian-rovo`: confirmed only 3
  Teamwork-Graph tools exposed (no Jira CRUD), confirmed the API token is authz-denied for
  real `tools/call`, confirmed Jira project KAN has 0 issues. Committed the finding
  (`ece072f`) — this is the evidence base for the pivot in §1.
- Made the [SELF] pivot decision (§1) — GitHub as the live-capture backend instead of Jira.
- Proved the GitHub pivot's mechanism end-to-end: registered `github-probe` MCP (full CRUD
  surface, live 9-issue read confirmed), ran `reposix init github::reubenjohn/reposix` in a
  throwaway `/tmp` clone (9 issue files synced, leaf-isolation-clean).
- Delegated the owner's PROGRESS.md directive to `/gsd-quick`; landed `de06e00`.
- Writing + committing this handover (this commit) + the `CONSULT-DECISIONS.md` [SELF]
  entry for the pivot, in the SAME commit, then pushing all three commits and confirming CI.

## 5. Litmus / gate / REOPEN state

- CI on the pre-handover tip (`de06e00`'s parent chain): last CONFIRMED green was on
  `065c0b4` (run `29462093167`, verified by #37). #38 has NOT yet re-confirmed CI on
  `ece072f`/`de06e00` — **that confirmation is deferred to immediately after #38 pushes**,
  and #39's §0 first-act re-verifies it independently. Do not assume green until you've
  seen the `gh run list` output yourself.
- **8 hero-number rows** (docs/index + README) remain **WAIVED-MISSING_TEST until
  2026-08-15** — T6 un-waives after T4/T5 re-measure lands (now GitHub-sourced, not Jira).
- File-size soft-ceiling WARNs (waived until 2026-08-08, class `GTH-V15-21`):
  `115-PLAN.md` ~32.6kB, `GOOD-TO-HAVES.md` ~30.6kB.
- **`SURPRISES-INTAKE.md` now ~34kB** — exceeded the 20k soft ceiling this rotation
  (grew from `ece072f`'s blocker append), WAIVED to 2026-08-08. Progressive-disclosure
  split eventually, not urgent.
- **Pre-push budget WARN** re-baseline (~55s→~75s, WARN 90s→100s) still FILED, not
  APPLIED (#36 diagnosis, `SURPRISES-INTAKE.md` 2026-07-15 17:18 entry). Apply during the
  OP-8 drain.
- No REOPEN state pending.

## 6. Mid-execution decisions not yet formalized + "noticed, not yet filed"

- **[SELF] decision this rotation** (fully recorded in `.planning/CONSULT-DECISIONS.md`,
  §1 above is the summary): pivot T4's live-capture backend from Jira/`atlassian-rovo` to
  GitHub. Reversible; DELETE the ledger entry once T4 captures land (folds into the T6
  five-entry `[SELF]`-deletion sweep, §7 below).
- **Doc-truth drift, NOT YET FILED** — `.planning/milestones/v0.15.0-phases/ROADMAP.md` is
  a stale "PLANNING / Phase TBD" stub; the LIVE P114–P128 index is `.planning/ROADMAP.md`.
  The milestone-scoped file looks authoritative by naming convention but is superseded.
  Noticed by #38 while grounding this handover; **#39 should file this to
  `GOOD-TO-HAVES.md`** (either populate it or delete it — low stakes, <1h either way).
- **MCP servers left registered, outside the git tree (not a repo change, but load-bearing
  state #39 depends on):** `github-probe` (the T4 prerequisite going forward — leave it
  registered) AND `atlassian-rovo` (proven non-functional for CRUD this rotation via
  `ece072f` — leave registered for now, safe to `claude mcp remove atlassian-rovo` after
  P115 closes if desired). `mcp-mermaid` is still DOWN — re-check before any diagram task.
- **Owner FYI, not a gate:** the owner watches `PROGRESS.md`; its NOW section surfaces the
  pivot directly. The pivot sits within #38's [SELF] authority — #39 may proceed straight
  into T4 execution without waiting on an explicit owner/manager confirm. If #39 wants an
  optional courtesy heads-up to the manager (w1:p7) before spending 6 more sessions, that's
  discretionary, not required.

## 7. Precise next steps (successor #39 runbook)

1. **FIRST ACT — confirm CI green on the tip AFTER #38 pushed this handover.**
   `git rev-list --left-right --count HEAD...origin/main` (expect **0/0**); `gh run list
   --branch main --workflow CI --limit 3` (top row `completed`/`success`); then
   `python3 quality/runners/run.py --cadence post-push --persist` (P0 `ci-green-on-main`
   asserts the NEWEST `ci.yml` run succeeded). If the flaky `test` job goes red, re-run it
   ONCE before treating it as real; if still red, STOP — do not open T4 execution over a
   red main; escalate.
2. **Dispatch T4 execution to a fresh-context executor (opus)** per the recipe in §2:
   register/verify `github-probe`, run the reposix arm in a throwaway `/tmp` clone, execute
   6 capture sessions (median-of-3 × 2 arms), append ledger rows one at a time asserting
   `running_total ≤ 50` before each, scrub secrets before every commit, fold in GTH-V15-25
   step 1 while the JSONL is fresh, update `115-MCP-SERVER-CHOICE.md`.
3. **Refresh `PROGRESS.md`** in the SAME push that lands each capture batch / at each
   boundary commit, per the §3 contract — do not defer this to phase-close.
4. **Then T5 → T6 → phase-close → P116 packet:**
   - **T5:** implement the JSONL-usage path in `quality/gates/perf/bench_token_economy.py`
     (headline via `session-analyzer` parse of the committed `benchmarks/captures/` files
     — not `count_tokens`; no `ANTHROPIC_API_KEY` needed). Now fed by the GitHub captures →
     produces the live **85.5% GitHub** number. Preserve the `bench_token_economy_io.py`
     re-export surface + the module-level `FIXTURES`/`BENCH_DIR`/`RESULTS` monkeypatch
     contract. Regen `docs/benchmarks/token-economy.md`; keep it offline-CI-stable from
     committed fixtures; match README; catalog-first if a perf-row contract changes.
     `scripts/bench_token_economy.py` is a shim (**not** a symlink).
   - **T6** (`115-UNWAIVE-PATH.md`). **Headline-framing decision #39 must make/surface:**
     the live-validated number will be GitHub 85.5%, not the currently-published Jira
     89.1% — either re-anchor the hero headline to the live GitHub number, or keep 89.1%
     with an explicit caveat that live validation covers the GitHub regime.
     `token-economy.md` already carries all three backend numbers, so doc churn is low.
     Then: a SECOND `/reposix-quality-refresh docs/benchmarks/latency.md` (the headline
     reframe re-drifts its 14 doc-alignment rows — grep
     `quality/catalogs/doc-alignment.json` before editing docs), the regen-clobber guard
     (`emit-markdown.sh` must NOT overwrite CI-canonical latency sections), un-waive the 8
     hero-number rows (5 latency + 3 token, currently waived to 2026-08-15), and **DELETE
     the [SELF] ledger entries** — now FIVE total: the original four (A1 definition, T2
     latency-canonical, T6 headline-framing, T5 JSONL-methodology) PLUS the T4
     GitHub-pivot entry (§1/§6 here) once captures land.
   - **Phase-close cadence:** `git push origin main` BEFORE verifier dispatch; then
     `python3 quality/runners/run.py --cadence post-push --persist` (P0
     `ci-green-on-main`); dispatch the verifier subagent for catalog-row PASS; advance
     `.planning/STATE.md` cursor; refresh `PROGRESS.md` in the close push; never open the
     next phase over a red main.
   - **P116** (after P115 closes): produce the ADR-010 packet (ADR-01 mirror-fanout +
     FIX-03 GTH-09 slug→id durable-create options+tradeoffs) and route it to the
     **MANAGER (w1:p7) for ruling — NO pre-ruling implementation.**
5. **If the weekly subscription cap hits mid-work:** commit+push whatever landed, REPLACE
   this session handover, end cleanly — your successor resumes from there. Reset-gating is
   RETIRED (owner ruling, `c7cea90`) — never defer or schedule work AROUND a reset; only
   react to a cap that actually hits.

## 8. Binding constraints (carry verbatim, unchanged)

One tree-writer at a time; ONE cargo invocation machine-wide (prefer `-p`); no
`--no-verify`; targeted staging (never `-A`/`.`); don't touch
`.planning/MANAGER-HANDOVER.md`; no tag push; no git surgery on main; leaf isolation in
`/tmp` same-invocation; opus complex / sonnet default / haiku mechanical, never fable at a
leaf; relieve past ~100k own-context (hard 150k, absolute not %) at a wave boundary; push
at green, then confirm `code/ci-green-on-main` P0 AFTER push; never open the next phase
over a red main; reset-gating RETIRED — never defer or schedule work for a weekly reset,
only react to a cap that actually hits (if it hits: commit+push, refresh this handover +
`PROGRESS.md`, end cleanly).
