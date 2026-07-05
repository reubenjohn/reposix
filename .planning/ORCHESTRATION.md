# ORCHESTRATION.md — reposix orchestration doctrine

Canonical, committed home for how autonomous reposix sessions run: delegation
shape, coordinator discipline, relief/handover, operating cadence, and the
durable-state rules. Distilled from session 7e2a4cf2 (2026-07-03/04). Replaces the
session-local accumulator that previously held these rules in `/tmp`.

> **Pointer contract (D-CONV-7 shape):** root `CLAUDE.md` keeps a ~6-line summary +
> a pointer here; THIS file is authoritative long-form. Update both when doctrine
> changes. Provenance + evidence: `.planning/research/doctrine-institutionalization/`.

## Enforcement map (read first)

| Doctrine | Enforced by | Layer |
|---|---|---|
| One cargo machine-wide | `.claude/hooks/cargo-mutex.sh` (exit 2) | blocking hook |
| Uncommitted = didn't happen | `.claude/hooks/stop-uncommitted.sh` (exit 0 + systemMessage) | advisory hook (owner decision Q2) |
| Persist before compaction | `.claude/hooks/precompact-persist.sh` | state hook |
| Session brief on startup | `.claude/hooks/session-start-brief.sh` | state hook |
| Tier check / charter / lane size at dispatch | `.claude/hooks/dispatch-doctrine.sh` | JIT injection |
| Check tree before re-running a lane | `.claude/hooks/post-dispatch-relay.sh` | JIT injection |
| Coordinator role + 5 rules + tier table | `.claude/agents/phase-coordinator.md` | agent-def |
| External mutation approval | `.claude/settings.json` + `.claude/agents/steward.md` | permission + agent-def |
| Context %/relief nudge | user-level `gsd-context-monitor.js` (≤35%/≤25%) | JIT injection |
| Intention-over-plan; tangent scoping; cadence | this file + `coordinator-dispatch` skill | prose + skill |

Everything below that is NOT hook- or permission-enforced is a judgment call — the
prose here is the standard you are held to.

## 1. Three-tier model delegation

The top-level orchestrator delegates **only** to `fable` coordinators (spawned via the
`phase-coordinator` agent, `model: inherit` so it stays fable). Each coordinator tiers
its own sub-delegation:
- **opus** — genuinely complex or security-judgment lanes.
- **sonnet** — default implementation.
- **haiku** — mechanical/leaf work (reads that return a digest, single-file edits).

**Never `fable` at a leaf** — it is overkill and defeats the tiering. (OD-4 item 1.)

> **No-fable mode (P92→v1.0):** when the top-level session itself runs on opus,
> `.planning/RUNBOOK-TO-V1/index.md` Amendments 1–3 govern: five-tier recursion
> (L0's whole drive reaches end state at ~10% of its own context, total → L1
> portion coordinators → L2 phase coordinators → L3 work lanes → L4 helpers), the
> 10x scoping rule, honest corrective iteration,
> and a single-shot fable consult valve replacing standing fable coordinators.

## 2. Coordinators route, they do not work

A coordinator's own tool calls are limited to: Agent dispatches, one-line git/gh
ground-truth checks (`log --oneline`, `status`, `run list`), and reading SHORT
reports/handovers. The 5 rules (verbatim, owner directive 2026-07-04
"your subagents are coordinators, not workers"):

1. **ROUTE, DON'T WORK.** Never read source files or long plans yourself (dispatch a
   `reader-digester` that returns a ≤300-word digest), run test suites/litmus/builds
   yourself (dispatch a runner), write or edit repo files yourself (dispatch an
   executor — including planning docs and handovers), or review diffs yourself
   (dispatch a reviewer).
2. **SLICE LANES SMALL.** No executor lane should need >100 tool calls; if a plan
   implies more, split into sub-lanes before dispatching. Executors may sub-delegate
   mechanical parts to haiku.
3. **REPORTS ARE SMALL, EVIDENCE IS COMMITTED.** Instruct every sub-agent: ≤400-word
   structured report; full evidence goes into committed artifacts (SUMMARY files,
   verdicts, transcripts), never into chat.
4. **NEVER WAIT INLINE.** No polling loops, watchers, or sleeps. Background children
   notify on completion; if a reply mis-routes, the orchestrator relays it. If you
   find yourself idle-waiting, end your dispatch turn.
5. **PROACTIVE RELIEF.** At every wave boundary ask: am I past ~50% of my context? If
   yes: dispatch `relief-handover-writer` to write+commit the handover, then tell the
   orchestrator to relieve you. Relief is cheap; rot is not.

## 3. Context budget + relief/handover protocol

Track context percentage from turn one, not just at milestones. Target reaching the
full end-state by **~50% context** (not 100%) — the second half is relief headroom.
The user-level `gsd-context-monitor.js` hook fires advisory warnings at ≤35% remaining
(WARNING) and ≤25% (CRITICAL); do not wait for it — relieve proactively at wave
boundaries.

A relieved coordinator must be **told in advance** it is being relieved, so it writes a
deliberate, complete handover (persistence is solicited, not automatic). Dispatch
`relief-handover-writer` with the ORCHESTRATION handover template:

**Handover file template** (distilled from `90-PAUSE-HANDOFF.md` +
`91-HANDOVER.md`; the `relief-handover-writer` agent writes+commits this):

```
# <N>-HANDOVER.md — <phase> <relief|pause> handover, <date>

Who wrote this, why relieved/paused, successor's required reading order, and any
"do not touch X / do not start Y" guardrails.

## 1. Ground truth (git)
HEAD sha, `git status` tree state, ahead/behind count, per-commit one-liners since
the last known-clean sha, and numbered deviations the successor MUST know.

## 2. Wave/cycle state
Markdown table: Wave | Plan | State (DONE/IN-PROGRESS/NOT STARTED) | Commits.
Plus any named-incident post-mortem to read before dispatching an executor.

## 3. Binding constraints (unchanged)
One tree-writer at a time; ONE cargo machine-wide; no `--no-verify`; push only at
green; commit-trailer format; model tiering.

## 4. Litmus / gate / REOPEN state
Gate run history (run #, exit code, transcript path), open-waiver expiry clocks.

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"
De-facto decisions made live; loose ends not yet routed to intake.

## 6. Precise next steps (successor runbook)
Numbered, ordered, imperative: spot-check → re-run gate → dispatch next wave →
close ritual (verdict path, CI green, RAISE LIST, intake disposition, STATE.md
cursor advance, final-report contents).
```

The writer confirms the commit SHA in its report.

Top-level (whole-session) rotations use the same template but write
`.planning/SESSION-HANDOVER.md` (not a phase-numbered filename) — see that file for
the current live example. Successor top-level sessions running without fable
follow `.planning/RUNBOOK-TO-V1/index.md` — the committed runbook for the
P92→v1.0 drive (decision procedures, escalation valve, loops, context budgets,
portion map, bootstrap prompt).

## 4. Operating cadence A/B + debt-drain

Two standing operations, interleaved:
- **A — Phase chain.** One fable coordinator per phase, owns the tree, reports a verdict.
- **B — Quality upkeep.** Read-only audit fleets (`audit-fleet-lane`) run DURING phases
  (parallel-safe). A **debt-drain window** runs BETWEEN every phase close and the next
  dispatch: drain `eager-window` ledger rows (sonnet fixers, one tree-writer at a time),
  route `intake-P<N>` rows into intake files, mint `catalog-row` candidates.

**No dispatch over undrained BLOCKERs.** A phase does not dispatch while BLOCKER-class
quality-ledger rows sit undrained. Cross-cutting unify/simplify judgment (collapsing
near-duplicate tooling, accepting trivial capability loss for major complexity
reduction) is itself fable-tier delegation-worthy work. (OD-4 item 2; D-CONV-1..8.)

## 5. Tangent scoping (charter = budget)

Tooling, guardrails, and hooks are deliverables, not distractions (the entire
`quality/` framework was an unplanned tangent). Graduated response:
- **<1h + no new dependency** → fix inline in the discovering lane.
- **larger** → file to `SURPRISES-INTAKE.md` / `GOOD-TO-HAVES.md` with severity + sketch.
- **balloons past intake-sized** → surface as an explicit scope decision to the owner.
  **Never silently absorb; never scope-creep to fit.** (OP-8; directive #12.)

## 6. Pause/resume at logical boundary

Pause is a first-class operation the owner invokes mid-flight (e.g. to restart with new
permissions). The correct response is NOT to stop immediately — **finish the current
atomic unit, then produce a durable, complete handover** (template §3) before standing
down. A resumed session opens with ground-truth git, then the handover's reading list.
(Directives #4, #11, #14.)

## 7. Durable state over chat

- **Everything the session owns lives in a committed-or-fixture artifact from row one.**
  Cache state, sim state, planning artifacts — no "works in my session."
- **Uncommitted = didn't happen.** `/tmp` does not survive an OS crash. This session's
  170-row `/tmp` `QUALITY-LEDGER.md` was permanently lost across the 2026-07-03 crash
  because no committed copy existed; `D-CONV-1..6` survived because they were committed.
- **Recovery is honest reconstruction, never fabrication.** The ledger was rebuilt as
  `.planning/audits/QUALITY-LEDGER.md` "with documented provenance rather than
  fabricated." Resume orphaned background agents from their on-disk transcripts (via
  SendMessage) rather than restarting from scratch. (Directives #10, #18.)

## 8. Orchestrator relay for mis-routed replies

When a subagent's reply cannot be delivered (cross-session addressing failure), the
orchestrator relays the full report inline rather than silently dropping it. Before
re-running any lane, **check the agent tree / git log first** — the work may already be
committed. (Episode; `post-dispatch-relay.sh` injects this reminder.)

## 9. External / security-sensitive mutations need owner-named-target approval

Actions that mutate state outside the local repo (remote branch/PR ops, bypassing a
secret scanner) or touch security posture require an explicit owner approval **naming
the specific target**. A permission classifier refusing to let an agent self-authorize
such an action is **correct behavior, design feedback, not a bug to route around** —
e.g. the Google-API-key false positive was unblocked only by the owner's named
"proceed," not by the agent self-authorizing a `gitleaks:allow`. Sanctioned targets:
`docs/reference/testing-targets.md`. (Directive #9; `steward` agent embeds this.)

## 10. Mission over plan (executive judgment)

The coordinator's job is to understand the project's underlying intention and make
executive, autonomous pivots toward it — not to faithfully execute a stale plan. This
authorizes resequencing the roadmap itself. Exemplar: OD-4 item 3 pulled a
launch-readiness milestone (hero demo, CI-verified headline numbers, install-path
excellence, Show-HN kit) AHEAD of v0.13.2 cross-link, re-derived from the owner's stated
intention ("puts this project on the global map"), not from the existing phase order.

## Provenance

Distilled from session 7e2a4cf2. Full 14-theme inventory with verbatim owner quotes,
per-theme encoding-status, and the coverage check:
`.planning/research/doctrine-institutionalization/` (index + theme chapters + REPORT.md).
Prior committed homes: `89-OWNER-DECISIONS.md` (OD-1..4), `.planning/PRACTICES.md`
(OP-8/OP-9), `quality/SURPRISES.md` (D-CONV-1..8). This file supersedes the
session-local accumulator (`~/.claude/jobs/7e2a4cf2/tmp/PENDING-INTAKE-AND-CHORES.md`).
