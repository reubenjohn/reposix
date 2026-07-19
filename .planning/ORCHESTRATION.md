# ORCHESTRATION.md — reposix orchestration doctrine

Canonical, committed home for how autonomous reposix sessions run: delegation
shape, coordinator discipline, relief/handover, operating cadence, durable state.
Distilled from session 7e2a4cf2 (2026-07-03/04).

> **Pointer contract (D-CONV-7 shape):** root `CLAUDE.md` keeps a ~6-line summary +
> a pointer here; THIS file is authoritative long-form — update both when doctrine
> changes. Episodic deep-dive detail (enforcement-map table, leaf-isolation mechanism,
> large-framework proposals, §11 budget elaboration, full HCI text, provenance) lives in
> the companion `.planning/ORCHESTRATION-REFERENCE.md`.

## Enforcement map (read first)

Some doctrine is mechanically enforced (blocking hooks — `cargo-mutex.sh`,
`leaf-isolation-guard.sh` + `.githooks/pre-commit`; advisory/state hooks; JIT injections;
permission + agent-defs; on-demand skills). Everything NOT hook- or permission-enforced is a
**judgment call — the prose here is the standard you are held to.** Full per-doctrine
enforcing-artifact table: `.planning/ORCHESTRATION-REFERENCE.md` § "Enforcement map (full
table)".

### Leaf isolation for reposix/sim/git test setup (HARD STOP)

**Any leaf that runs `reposix init`, sim seeding, or any `git commit` / `git config`
for test setup MUST operate in a throwaway clone under `/tmp` — NEVER the coordinator's
shared repo — and every such command MUST `cd` into its `/tmp` target dir within the
SAME Bash invocation.** Two facts make this non-negotiable: (i) agent "worktrees" share the
coordinator's `.git/config` + object store — NOT isolated; and (ii) a leaf's cwd resets to
the repo root between Bash calls, so a setup command that did not `cd` into `/tmp` in the same
invocation runs against the real shared repo. Evidentiary anchor: a sim-seed leaf that skipped
this ran git ops under the `t <t@t>` fixture identity and flipped `core.bare=true`, corrupting
the shared repo (repaired twice). Coordinators: bake the `cd /tmp/<target> && <cmd>` shape into
every dispatched setup lane; a leaf that touches shared-repo git state is a REJECT.

**Now backstopped mechanically (v0.14.0 P102):** `.claude/hooks/leaf-isolation-guard.sh`
(PreToolUse Bash, exit 2, three fail-closed guards — fixture-identity reject / leaf-setup
location / shared-config write) blocks the bad move before it runs, plus the git-native
`.githooks/pre-commit` fixture-identity check. The prose hard-stop is RETAINED as the
readable contract (not deleted). Full mechanism — guard A/B/C detail, evasion hardening,
and the honest coverage boundary (hook fires only on the Bash *tool*; subprocess writes
bypass it): `.planning/ORCHESTRATION-REFERENCE.md` § "Leaf-isolation enforcement mechanism".

## 1. Three-tier model delegation

The top-level orchestrator delegates **only** to `fable` coordinators (spawned via the
`phase-coordinator` agent, `model: inherit` so it stays fable). Each coordinator tiers its
own sub-delegation: **opus** — complex or security-judgment lanes; **sonnet** — default
implementation; **haiku** — mechanical/leaf work (digest-returning reads, single-file edits).

**Never `fable` at a leaf** — it is overkill and defeats the tiering. (OD-4 item 1.)

> **No-fable mode:** when the top-level session itself runs on opus (or below), §11
> governs (five-tier recursion, 10%/10x budgets, named judgment-call procedures, a
> single-shot fable consult valve replacing standing fable coordinators) and §12 holds
> HCI doctrine. Any arc-scoped runbook for a specific drive (e.g. a
> `.planning/RUNBOOK-TO-V1`-shaped doc) APPLIES this doctrine to that drive's portions —
> doctrine lives here and outlives any one runbook; the runbook is disposable, this file
> is not.

## 2. Coordinators route, they do not work

> **L0 is a ROUTER, not a worker (owner directive 2026-07-17).** The top-level seat's own
> window is reserved for routing, gate checks, and verification — it dispatches a
> `phase-coordinator` C1 per phase/wave (explicit model tier: opus complex / sonnet default /
> haiku mechanical) under which gsd leaves run, so MOST substantive work executes TWO levels
> below the seat. Target **~1h+ of substantive work per workhorse handover** — this corrects a
> measured drift to depth-0/1 work with ~25–45 min legs (seats #46–#52). Rationale (owner
> verbatim intent): reduce low-level detail at BOTH the workhorse and manager tiers so
> meta-level judgment capacity is preserved.

A coordinator's own tool calls are limited to: Agent dispatches, one-line git/gh
ground-truth checks (`log --oneline`, `status`, `run list`), and reading SHORT
reports/handovers. The 5 rules (verbatim, owner directive 2026-07-04
"your subagents are coordinators, not workers"):

1. **ROUTE, DON'T WORK.** Never yourself read source or any **>100-line** file/plan/report
   (dispatch a `reader-digester` → ≤300-word digest), run test suites/litmus/builds (dispatch a runner),
   write/edit repo files incl. planning docs + handovers (dispatch an executor), or review
   diffs (dispatch a reviewer). `subagent_type` mapping: `coordinator-dispatch` §2
   (executor/runner→`gsd-executor`, reviewer→`gsd-code-reviewer`).
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
   ~100k tokens of my own context? If yes: dispatch `relief-handover-writer` to write+commit
   the handover, then tell the orchestrator to relieve you. **Hard stop ~150k** — relieve
   immediately, start no new work. Measure ABSOLUTE tokens, never a % of the window: quality
   degrades past ~150k regardless of a 1M window (so "50% of 1M" = 500k is far past useful).
   Relief is cheap; rot is not. §3 has the C1/C2 structure that absorbs the resulting
   rotation below L0.

**Bottom-up triage (the receiving side of "noticing is a deliverable").** Every sub-agent
report carries a NOTICED section + RAISE LIST + intake disposition. The parent **triages each
item and routes it, never drops it** — absorb into this wave / re-delegate as a new lane /
file-or-escalate (OP-8; valve). A leaf's reported friction that surfaces in NO commit, NO
intake row, and NO re-dispatch is a dropped deliverable — the same red flag as an empty
noticing section. Full absorb-vs-redelegate routing table:
`.planning/ORCHESTRATION-REFERENCE.md` § "Bottom-up triage routing".

### Single-writer discipline (shared-tree serialization — owner ruling 2026-07-12)

Exactly ONE session/agent writes the shared working tree at a time. Parallel tree-writers race the git index and force mid-flight `TaskStop`s (observed in the v0.14.0 hygiene lane). Rules:
- A coordinator MAY fan out **read-only** inspection agents in parallel.
- **Tree-mutating** work (file edits + `git add`/`commit`) is **serial**: dispatch the next tree-writer only after the prior one's commit has landed.
- **No worktree infrastructure** — the owner rejected per-agent worktrees as over-engineering for the current single-machine autonomous cadence; serialize instead.

Full owner rationale: git history (2026-07-12 [OWNER] session-serialization ruling).

## 3. Context budget + relief/handover protocol

Track ABSOLUTE token spend from turn one (§2 rule 5 — ~100k soft, ~150k hard, absolute not
%). The `gsd-context-monitor.js` hook fires advisory %-remaining warnings; treat them as a
backstop, not the trigger — relieve proactively at wave boundaries on the absolute ~100k
line. This section covers the C1/C2 tier structure that absorbs that rotation + the handover
protocol.

**Two coordinator tiers (absorb relief churn below L0).** Because the ~100k line rotates
coordinators MORE often than the old ~50%-of-1M line did, a multi-phase milestone must NOT
run as one C1 phase-coordinator reporting every rotation straight to L0. Instead L0
dispatches a **coordinator-of-coordinators (C2)**: a milestone-scoped `phase-coordinator`
that dispatches **one C1 `phase-coordinator` per phase** (which dispatch executors/readers) —
each C1 dispatched with a model tier matched to phase complexity: opus complex / sonnet
default / haiku mechanical.
(A single-phase milestone may skip C2 and run one C1 directly.) When a C1 relieves at ~100k,
its successor is dispatched by its **parent C2**, not L0; L0 hears from the C2 only at
milestone boundaries or an E1–E4 escalation. **No new agent type is needed:**
`phase-coordinator` serves both tiers via the §11 five-tier recursion — the tier is set by
charter SCOPE (milestone → C2; single phase → C1), and each tier relieves on its OWN ~100k
line.

**Liveness doctrine (push→CI-in-flight boundary — load-bearing, L0 ruling 2026-07-17,
root-caused by the P122 close-liveness incident).** Background-task re-invocation is
reliable ONLY at L0 (top-level) — a coordinator's OWN backgrounded `gh run watch` (or
any self-owned background-bash watcher) does NOT reliably re-invoke it; it goes dormant
and stalls the phase close (the P122 C1 backgrounded its own CI watch and never woke; the
C2 had to be poked by L0 to take deterministic control). A coordinator must therefore
NEVER background its own CI watch and end its turn assuming it will wake. Everything
BEFORE the push (plan → execute → code-review) runs straight through; the
push→CI-in-flight boundary is the ONE stop-and-return point: the coordinator STOPS and
RETURNS to its dispatching parent a short status — pushed SHA + in-flight `ci.yml` run id
+ "awaiting CI green to run post-push cadence + close." Direct child-agent completion
notifications DO reliably re-invoke a parent (that path works); bare background-bash
watchers do not — the parent relays the run id up to L0 (which holds the durable CI
watch) and SendMessages the coordinator to resume the phase close on green. Never relieve
or end a turn on a passive self-watch/upward-relay assumption.

**SendMessage tier limitation (STANDING, MANAGER decide-and-disclose ruling — owner veto
open, 2026-07-18) — WHY the durable CI watch lives at L0.** SendMessage is a tool-grant
limitation of the `phase-coordinator` registry entry: it is NOT granted at the
phase-coordinator (C2) tier or below. So a C2
cannot SendMessage / halt / resume its own background children, and a child cannot
resume-by-id back to its parent C2 — the failure is specifically **C2→child and
child→C2**. **L0→C2 SendMessage DOES work** (an L0 top-level session resumes a C2 by
session id — confirmed repeatedly), which is exactly why the durable CI watch and the
resume-on-green SendMessage sit at L0, not at the coordinator. Because a C2 cannot
background-and-resume a child, coordinators at the C2 tier and below MUST **serialize
strictly** and drive every phase close via **FRESH verifier→executor LEAVES** (the
P122-blessed deterministic pattern — dispatch the leaves directly; never `fork` a
coordinator to resume/close it, never background-and-resume a child at the C2 tier).
This is **RATIFIED standing doctrine, not a temporary workaround** — embed the caveat
verbatim in every C2/C1 charter. Ledger: `.planning/CONSULT-DECISIONS.md`
(2026-07-18 [OWNER]).

A relieved coordinator must be **told in advance** it is being relieved, so it writes a
deliberate, complete handover (persistence is solicited, not automatic). Dispatch
`relief-handover-writer` with the ORCHESTRATION handover template:

The `relief-handover-writer` agent writes+commits this handover. Full template:
`.planning/ORCHESTRATION-REFERENCE.md` § "Handover file template (§3 detail)".

Top-level (whole-session) rotations use the same template but write
`.planning/SESSION-HANDOVER.md`. Successor top-level sessions running without fable read
§11–12 for standing doctrine (tiering, budgets, judgment procedures, HCI), then whatever
arc-scoped runbook governs the active drive (`.planning/STATE.md` names which, if any).

## 4. Operating cadence A/B + debt-drain

Two standing operations, interleaved: **A — Phase chain** (one fable coordinator per phase,
owns the tree, reports a verdict) and **B — Quality upkeep** (read-only audit fleets during
phases + a debt-drain window between every phase close and the next dispatch). Detail:
`.planning/ORCHESTRATION-REFERENCE.md` § "Operating cadence A/B".

**No dispatch over undrained BLOCKERs.** A phase does not dispatch while BLOCKER-class
quality-ledger rows sit undrained. Cross-cutting unify/simplify judgment (collapsing
near-duplicate tooling, accepting trivial capability loss for major complexity
reduction) is itself fable-tier delegation-worthy work. (OD-4 item 2; D-CONV-1..8.)

## 5. Tangent scoping (charter = budget)

Tooling, guardrails, and hooks are deliverables, not distractions (the `quality/`
framework was an unplanned tangent — and the project's best investment). Graduated response:
- **<1h + no new dependency** → fix inline in the discovering lane.
- **larger** → file to the active milestone's
  `.planning/milestones/<active>-phases/{SURPRISES-INTAKE,GOOD-TO-HAVES}.md` with severity + sketch.
- **balloons past intake-sized** → surface as an explicit scope decision to the owner.
  **Never silently absorb; never scope-creep to fit.** (OP-8; directive #12.)

**Proposing a large generic framework is expected work, not scope-creep to suppress** — the
owner gates approval, never the surfacing. Rationale + the ~10-line scope-memo shape:
`.planning/ORCHESTRATION-REFERENCE.md` § "Large-framework proposals are expected work".

## 6. Pause/resume at logical boundary

Pause is a first-class owner operation invoked mid-flight. The correct response is NOT to
stop immediately — **finish the current atomic unit, then produce a durable, complete
handover** (template §3) before standing down; a resumed session opens with ground-truth git,
then the handover's reading list. Detail: `.planning/ORCHESTRATION-REFERENCE.md` §
"Pause/resume at logical boundary".

## 7. Durable state over chat

- **Everything the session owns lives in a committed-or-fixture artifact from row one** —
  cache, sim, planning artifacts; no "works in my session."
- **Uncommitted = didn't happen.** `/tmp` does not survive an OS crash — a 170-row `/tmp`
  `QUALITY-LEDGER.md` was permanently lost across the 2026-07-03 crash (no committed copy);
  `D-CONV-1..6` survived because they were committed.
- **Recovery is honest reconstruction, never fabrication** — rebuild with documented
  provenance (`.planning/audits/QUALITY-LEDGER.md`), not fabricated content. Resume orphaned
  background agents from their on-disk transcripts (via SendMessage) rather than restarting.
  (Directives #10, #18.)
- **Executor standing rule (P120 close, root-caused an orphan):** cargo is
  **FOREGROUND-only** — never `run_in_background`/detached (it orphans the build, evades
  the `cargo-mutex.sh` serialization, and risks the OOM this VM has already hit three
  times); and **commit before ending any turn** — an executor that spawned a detached
  `run_in_background` cargo and then ended its turn WITHOUT committing left both an
  orphaned background build AND no commit, the literal double-violation this rule closes.
  Same doctrine, one-liner cross-ref: root `CLAUDE.md` § "Build memory budget".

## 8. Orchestrator relay for mis-routed replies

On a cross-session addressing failure the orchestrator relays the full report inline rather
than silently dropping it; before re-running any lane, **check the agent tree / git log
first** — the work may already be committed (`post-dispatch-relay.sh` injects this). Detail:
`.planning/ORCHESTRATION-REFERENCE.md` § "Orchestrator relay for mis-routed replies".

## 9. External / security-sensitive mutations need owner-named-target approval

Actions that mutate state outside the local repo (remote branch/PR ops, bypassing a
secret scanner) or touch security posture require an explicit owner approval **naming
the specific target**. A permission classifier refusing to let an agent self-authorize
such an action is **correct behavior, design feedback, not a bug to route around** (e.g.
the Google-API-key false positive was unblocked only by the owner's named "proceed," not by
a self-authorized `gitleaks:allow`). Sanctioned targets: `docs/reference/testing-targets.md`.
(Directive #9; `steward` agent embeds this.)

## 10. Mission over plan (executive judgment)

The coordinator's job is to understand the project's underlying intention and make
executive, autonomous pivots toward it — not to faithfully execute a stale plan. This
authorizes resequencing the roadmap itself (DP-4 in procedural form). Resequencing exemplar:
`.planning/ORCHESTRATION-REFERENCE.md` § "Mission over plan — resequencing exemplar".

## 11. No-fable mode: five-tier tiering, budgets, judgment calls

Two operating configurations, chosen by whether a fable-tier session runs at the top level.
§1 is the fable-present default (three tiers under a standing fable coordinator); this
section is the arc-independent doctrine for the **no-fable configuration** — any top-level
session that itself runs opus or below. Deep procedural mechanics named here live in
`.claude/skills/decision-procedures/SKILL.md` (load that skill when a judgment call appears,
not every session); this section is the map, not the territory.

**Five-tier recursion.** L0 top-level orchestrator (opus) scopes work into portions so the
WHOLE drive reaches end state by ~10% of L0's own context — report-only diet, never reads
source/builds/edits. The practical shape: substantive work lands **two levels below** the top
seat (L0 routes → C1/C2 coordinate → leaves execute); L0's own window stays reserved for
routing, gate checks, and verification — including owning the durable CI watch at the
push→CI-in-flight boundary (§3 "Liveness doctrine"): a C1/C2 never self-owns a background
watch to wake itself; it stops and returns status instead. L1 portion coordinator (opus) owns one large portion, charters L2s. L2
phase/drain-window coordinator (opus for security-judgment work, sonnet otherwise) owns a
phase/window, charters L3 lanes. (A milestone-scoped L1 is the C2 coordinator-of-coordinators;
a phase-scoped L2 is a C1 — §3 carries the C1-relief-absorbed-by-C2 rule.) L3 work lanes
(sonnet) are where most actual work happens, ≤100 tool calls, case-by-case L4 delegation. L4
helper leaves (haiku mechanical, sonnet for review judgment) are terminal. Fable appears in
exactly one place: the single-shot consult valve below — never a standing coordinator, never
a leaf.

**Budgets.** Every tier's ENTIRE charter reaches end state by ~10% of ITS OWN
context (recurses per tier, not per sub-unit) — the remaining ~90% is correction
margin, never planned workload; tracking past it with scope outstanding is a
scoping failure, split or rotate. Assume every charter is 10x more complex than
planned: recon via an L4 lane before every dispatch, pre-authorizing the split in the charter
rather than making the child ask. Consolidate recon into ONE lane per dispatch decision (the
over-fan anti-pattern — splitting preflight by source and reading each report yourself makes
you an L1 gathering evidence, not an L0 receiving a verdict — is detailed in
`.planning/ORCHESTRATION-REFERENCE.md` § "Budget over-fan anti-pattern"). What does NOT
multiply: one cargo machine-wide, one-tree-writer-at-a-time, ≤400-word reports.

**Judgment calls.** Five named decision procedures (DP-1 coordinator-rot diagnosis, DP-2
prove-before-fix on BLOCKERs, DP-3 intake-design inversion, DP-4 executive resequencing =
§10 procedural, DP-5 tangent-vs-charter = §5 procedural) turn recurring calls into
trigger → evidence → decide → escalate. Unresolved → a 4-criterion escalation valve: **E1**
irreversible/destructive → owner always; **E2** architecture-shaping (ADR-class) → fable
consult first; **E3** mission-priority tradeoff → owner for scope/spend; **E4** two failed
self-attempts at the same gate → fable consult. Below the bar: decide-and-record in
`.planning/CONSULT-DECISIONS.md` (append-only; create on first use) — never idle, waiting
for permission you don't need is itself a rot signal. Full DP mechanics + the fable
consult-dispatch template + the ledger format live in the decision-procedures skill.
**Tag-blocking product bugs → fix-first,
no consult; escalate ONLY if the fix turns architectural (STOP + report to manager)**
— owner ruling 2026-07-13 (git history). **A `fork` is never a safe no-op or discard
placeholder** — it inherits full context and becomes a live PARALLEL tree-writer
(single-writer-discipline violation, §2). Never dispatch a fork to "throw away"; end the
turn instead. **Nor is `fork` a way to resume or close a warm agent** — to resume a warm
agent, SendMessage its id (its context is intact); to drive a phase close, dispatch the
verifier→executor LEAVES directly (the P122-blessed deterministic pattern), NEVER `fork` a
coordinator to "resume/close" it, since a fork clones the parent's context and can
confabulate a no-op close (P123-close incident: the fork returned ZERO tool uses while
claiming "close executing," STATE.md unchanged — caught only by verify-against-reality).

**SendMessage tier limitation (STANDING, MANAGER decide-and-disclose ruling — owner veto
open, 2026-07-18).** The reason the
phase-close pattern above dispatches FRESH verifier→executor LEAVES rather than resuming
a warm coordinator is a hard tool-grant limitation: SendMessage is NOT granted at the
phase-coordinator (C2) tier or below (it is absent from the `phase-coordinator` registry
entry's grant). A C2 therefore cannot SendMessage / halt / resume its own background
children, and a child cannot resume-by-id back to its parent C2 — the failure is
specifically **C2→child and child→C2**. **L0→C2 SendMessage DOES work** (an L0 session
resumes a C2 by session id — confirmed repeatedly). Consequence: coordinators at C2 and
below MUST **serialize strictly** and drive every phase close via fresh verifier→executor
LEAVES — never `fork` a coordinator to resume/close it, never background-and-resume a
child at the C2 tier. This is **RATIFIED standing doctrine, not a temporary workaround**;
embed the caveat verbatim in every C2/C1 charter. Ledger:
`.planning/CONSULT-DECISIONS.md` (2026-07-18 [OWNER]).

## 12. Honest corrective iteration (HCI)

Every deliverable, at every tier, passes an independent corrective review before it counts
done — **the author never self-grades**; a fresh reviewer/verifier grades from committed
artifacts only (the verifier-subagent mandate, `quality/PROTOCOL.md`, is the phase-level
instance). HIGH/RED gets a targeted fix, then a re-review by a DIFFERENT fresh reviewer; two
failed iterations at the same gate escalate to valve E4. "PASS because the author said so" is
not a grade. Full text: `.planning/ORCHESTRATION-REFERENCE.md` § "Honest corrective iteration
(HCI)".

## Provenance

Distilled from session 7e2a4cf2; the provenance section points to the full 14-theme
inventory at `.planning/research/doctrine-institutionalization/` (index + theme chapters +
REPORT.md). Prior committed homes, the 2026-07-05 permanence audit (why §11–12 are here),
and the **standing promotion-sweep rule** (any doctrine change ends with a stale-reference
sweep across `.claude/hooks/`, `.claude/agents/`, `.claude/skills/`, and the SessionStart
brief): `.planning/ORCHESTRATION-REFERENCE.md` § "Provenance, permanence audit, and the
promotion-sweep standing rule".
