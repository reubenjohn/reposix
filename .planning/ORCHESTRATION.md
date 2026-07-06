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
| Judgment-call procedures (DP-1..5) + escalation valve (E1-E4) + consult template | `.claude/skills/decision-procedures/SKILL.md` | on-demand skill (loaded when a judgment call appears, not every session) |

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

> **No-fable mode:** when the top-level session itself runs on opus (or below), §11
> governs — five-tier recursion, the 10%/10x budget rules, named judgment-call
> procedures, and a single-shot fable consult valve replacing standing fable
> coordinators. §12 holds honest-corrective-iteration doctrine. Any arc-scoped
> runbook for a specific drive (e.g. a `.planning/RUNBOOK-TO-V1`-shaped doc, or its
> successor for whatever's currently active) APPLIES this doctrine to that drive's
> portions and pre-framed decisions — doctrine lives here and outlives any one
> runbook; the runbook is disposable, this file is not.

## 2. Coordinators route, they do not work

A coordinator's own tool calls are limited to: Agent dispatches, one-line git/gh
ground-truth checks (`log --oneline`, `status`, `run list`), and reading SHORT
reports/handovers. The 5 rules (verbatim, owner directive 2026-07-04
"your subagents are coordinators, not workers"):

1. **ROUTE, DON'T WORK.** Never read source files or long plans yourself (dispatch a
   `reader-digester` that returns a ≤300-word digest), run test suites/litmus/builds
   yourself (dispatch a runner), write or edit repo files yourself (dispatch an
   executor — including planning docs and handovers), or review diffs yourself
   (dispatch a reviewer). Role words — `subagent_type`s in
   `coordinator-dispatch` §2 (executor/runner→`gsd-executor`, reviewer→
   `gsd-code-reviewer`).
2. **SLICE LANES SMALL.** No executor lane should need >100 tool calls; if a plan
   implies more, split into sub-lanes before dispatching. Executors may sub-delegate
   mechanical parts to haiku.
3. **REPORTS ARE SMALL, EVIDENCE IS COMMITTED.** Instruct every sub-agent: ≤400-word
   structured report; full evidence goes into committed artifacts (SUMMARY files,
   verdicts, transcripts), never into chat.
4. **NEVER WAIT INLINE.** No polling loops, watchers, or sleeps. Background children
   notify on completion; if a reply mis-routes, the orchestrator relays it. If you
   find yourself idle-waiting, end your dispatch turn.
5. **PROACTIVE RELIEF (absolute tokens, NOT %).** At every wave boundary ask: am I past
   ~100k tokens of my own context? If yes: dispatch `relief-handover-writer` to
   write+commit the handover, then tell the orchestrator to relieve you. **Hard stop
   ~150k** — relieve immediately, start no new work. Measure ABSOLUTE tokens, never a % of
   the window: quality degrades past ~150k regardless of a 1M window, so "50% of 1M" =
   500k is far past useful (a coordinator that waited for 50% would already be rotting).
   Relief is cheap; rot is not. See §3 for the C1/C2 two-tier structure that absorbs the
   resulting (more frequent) rotation below L0.

**Bottom-up triage (the receiving side of "noticing is a deliverable").** Every
sub-agent report carries a NOTICED section + RAISE LIST + intake disposition (rule 3 +
ownership charter). The parent does not just collect these — it **triages each item and
routes it, never drops it**: (a) low-deviation from the current charter and it fits the
10x capacity → **absorb** into this wave (replan the wave; DP-5 charter test); (b) real
work outside this charter → **re-delegate** as a new lane, or hand to the owning
downstream phase as a RAISE-LIST item; (c) larger-than-intake → **file / escalate**
(OP-8; valve). A leaf's reported friction that surfaces in NO commit, NO intake row, and
NO re-dispatch is a dropped deliverable — the same red flag as an empty noticing section.
Leaves report friction UP because they cannot see the whole charter; making the
absorb-vs-redelegate call is the parent's job precisely because it can.

## 3. Context budget + relief/handover protocol

Track your ABSOLUTE token spend from turn one (context budget: §2 rule 5 — ~100k soft,
~150k hard, absolute not %). The user-level `gsd-context-monitor.js` hook fires advisory
%-remaining warnings; treat them as a backstop, not the trigger — relieve proactively at
wave boundaries on the absolute ~100k line. This section covers the C1/C2 tier structure
that absorbs the resulting rotation and the handover protocol itself.

**Two coordinator tiers (absorb relief churn below L0).** Because the ~100k line rotates
coordinators MORE often than the old ~50%-of-1M line did, a milestone must NOT run as one
C1 phase-coordinator reporting every rotation straight to L0 — that floods the top with
successor-dispatch churn. Instead the top orchestrator dispatches a
**coordinator-of-coordinators (C2)**: a milestone / multi-phase-scoped `phase-coordinator`
that itself dispatches **one C1 `phase-coordinator` per phase** (which dispatch
executors/readers). (A single-phase milestone may skip C2 and run one C1 directly;
multi-phase → C2.) When a C1 relieves at ~100k, its successor is dispatched by its
**parent C2**, not by L0. L0 hears from the C2 only at milestone boundaries or an E1–E4
escalation. **No new agent type is needed:** `phase-coordinator` serves both tiers via the
§11 five-tier recursion — the tier is set by charter SCOPE (milestone → C2, dispatches
phase-coordinators; single phase → C1, dispatches executors). Each tier relieves on its
OWN ~100k line.

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
the current live example. Successor top-level sessions running without fable read
§11–12 below for the standing doctrine (tiering, budgets, judgment procedures,
HCI), then whatever arc-scoped runbook currently governs the active drive (if any)
for the portion map and pre-framed decisions specific to that drive — check
`.planning/STATE.md` for which runbook, if any, is live.

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
`quality/` framework was an unplanned tangent — and the project's best investment).
Graduated response:
- **<1h + no new dependency** → fix inline in the discovering lane.
- **larger** → file to `SURPRISES-INTAKE.md` / `GOOD-TO-HAVES.md` with severity + sketch.
- **balloons past intake-sized** → surface as an explicit scope decision to the owner.
  **Never silently absorb; never scope-creep to fit.** (OP-8; directive #12.)

**Proposing a large generic framework is expected work, not scope-creep to suppress.**
When a lane spots leverage that pays off across THIS project and others — a reusable
framework, a cross-cutting guardrail, an infra investment that deviates substantially
from the current charter — surfacing and scoping that proposal (a ~10-line scope memo:
what, why now, cross-project value, cost, cost-of-delay) is a first-class deliverable.
The quality-infra milestone is the canonical precedent: an unplanned tangent that became
the backbone. The owner still gates scope/spend (valve E3) — but the valve gates
**approval, never the surfacing**. Withholding a high-leverage proposal because "it's not
on the plan" is the failure mode, not the discipline.

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

## 11. No-fable mode: five-tier tiering, budgets, judgment calls

Two operating configurations, chosen by whether a fable-tier session runs at the
top level. §1 above is the fable-present default (three tiers under a standing
fable coordinator). This section is the general, arc-independent doctrine for the
**no-fable configuration** — any top-level session that itself runs opus or below.
The deep procedural mechanics this section only names live in
`.claude/skills/decision-procedures/SKILL.md` — load that skill at the moment a
judgment call actually appears, not every session; this section is the map, not
the territory.

**Five-tier recursion.** L0 top-level orchestrator (opus) scopes work into
portions so the WHOLE drive reaches end state by ~10% of L0's own context, total
— report-only diet, never reads source/builds/edits. L1 portion coordinator
(opus) owns one large portion, charters L2s. L2 phase/drain-window coordinator
(opus for security-judgment work, sonnet otherwise) owns a phase/window,
charters L3 lanes. (A milestone-scoped L1 is the C2 coordinator-of-coordinators; a
phase-scoped L2 is a C1 — §3 carries the C1-relief-absorbed-by-C2 rule.) L3 work lanes
(sonnet) are where most actual work happens,
≤100 tool calls, case-by-case L4 delegation. L4 helper leaves (haiku mechanical,
sonnet for review judgment) are terminal. Fable appears in exactly one place:
the single-shot consult valve below — never a standing coordinator, never a leaf.

**Budgets.** Every tier's ENTIRE charter reaches end state by ~10% of ITS OWN
context (recurses per tier, not per sub-unit) — the remaining ~90% is correction
margin, never planned workload; tracking past it with scope outstanding is a
scoping failure, split or rotate. Assume every charter is 10x more complex than
planned: recon via an L4 lane before every dispatch, and pre-authorize the split
in the charter rather than making the child ask permission. **Consolidate
recon into ONE lane per dispatch decision** — absorb a single synthesized
conclusion, never N report-bearing agents fanned into your own window.
Splitting preflight by source (git/CI/steward/charter) and reading each
report yourself is the over-fan anti-pattern: it makes you an L1 gathering
evidence, not an L0 receiving a verdict. Past ~5% of your context before the
first coordinator is dispatched → you're over-fanning; stop and consolidate.
What does NOT
multiply: one cargo machine-wide, one-tree-writer-at-a-time, ≤400-word reports
(§2 and the enforcement map already cover these — more agents means more
discipline on shared resources, not less).

**Judgment calls.** Five named decision procedures — DP-1 coordinator-rot
diagnosis, DP-2 prove-before-fix on BLOCKERs, DP-3 intake-design inversion, DP-4
executive resequencing (§10 in procedural form), DP-5 tangent-vs-charter
classification (§5 in procedural form) — turn recurring judgment calls into
trigger → evidence → decide → escalate. Anything a DP doesn't resolve goes
through a 4-criterion escalation valve: **E1** irreversible/destructive → owner,
always; **E2** architecture-shaping (ADR-class) → fable consult first; **E3**
mission-priority tradeoff → owner for scope/spend; **E4** two failed
self-attempts at the same gate → fable consult. Below the bar: decide-and-record
in `.planning/CONSULT-DECISIONS.md` (append-only; create on first use) — never
idle, waiting for permission you don't need is itself a rot signal. Full DP
mechanics, the fable consult-dispatch template, and the ledger entry format live
in the decision-procedures skill named above.

## 12. Honest corrective iteration (HCI)

Every deliverable, at every tier, passes an independent corrective review before
it counts done, and iterating to convergence is expected, never a failure:
**the author never self-grades** — a fresh reviewer or verifier grades from
committed artifacts only (this project's standing verifier-subagent mandate,
`quality/PROTOCOL.md`, is the phase-level instance of this rule; audits get a
fresh-eyes re-audit capped at 3 iterations; a milestone gets the 9-probe verdict
plus an independent honesty spot-check, author ≠ orchestrator). A review that
finds HIGH/RED gets a targeted fix, then a re-review by a DIFFERENT fresh
reviewer (never the one being checked, never the author) — two failed
iterations at the same gate escalates to valve E4 above. An empty findings
section from code-touching review is itself a red flag; "PASS because the
author said so" is not a grade.

## Provenance

Distilled from session 7e2a4cf2. Full 14-theme inventory with verbatim owner quotes,
per-theme encoding-status, and the coverage check:
`.planning/research/doctrine-institutionalization/` (index + theme chapters + REPORT.md).
Prior committed homes: `89-OWNER-DECISIONS.md` (OD-1..4), `.planning/PRACTICES.md`
(OP-8/OP-9), `quality/SURPRISES.md` (D-CONV-1..8). This file supersedes the
session-local accumulator (`~/.claude/jobs/7e2a4cf2/tmp/PENDING-INTAKE-AND-CHORES.md`).

**2026-07-05 permanence audit.** §11–12 (five-tier tiering, the 10%/10x rules,
the DP-1..5 / escalation-valve pointer, honest corrective iteration) were
promoted here from an arc-scoped runbook's amendments (that runbook itself is
unchanged in purpose, just relieved of restating general doctrine) — they are
general doctrine, not specific to any one drive, so they belong in a permanent
home rather than only in a runbook that archives when its drive ends. The deep
procedural bodies (DP-1..5 full triggers/evidence/decide/escalate text, the E1-E4
valve table, the fable consult-dispatch template, the `.planning/CONSULT-DECISIONS.md`
ledger format) live in `.claude/skills/decision-procedures/SKILL.md` rather than
inline here, per the owner's steer that episodic procedures belong in
on-demand skills, not in doctrine every session re-reads in full — this file
stays the distilled core; the skill is the reference manual. Permanent homes
never hard-reference a transient/arc-scoped file by path (only the reverse is
safe) — this file describes any arc-scoped runbook generically rather than
naming one, so it never goes stale when a runbook archives.

**Promotion sweep (standing rule).** Any doctrine change or promotion into this
file MUST end with a stale-reference sweep across `.claude/hooks/`,
`.claude/agents/`, `.claude/skills/`, and the SessionStart brief, grepping for
superseded numbers, tier names, and rule phrasings. These injected surfaces are
trusted and never re-read by the agents they instruct — a stale hook poisons
every future session silently; the 17b1c94 promotion skipped this sweep and
left the SessionStart brief teaching the superseded ~50% budget until a
fresh-agent audit caught it (fixed fe5e8f2).
