# RUNBOOK-TO-V1 — operating runbook for the no-fable drive from P92 to v1.0

Authored 2026-07-05 by the outgoing session's meta-plan author (fable) as the last
fable-tier artifact of the 7e2a4cf2 engagement. Audience: the NEXT top-level
orchestrator session, running on **opus and below** (no fable at any standing tier).
Mission: execute P92 → v0.13.0 tag → launch-readiness → v0.13.2 → v0.14.x → v1.0.0
without losing the judgment quality the fable session supplied. An opus coordinator
following this runbook should never need to *invent* process — only apply judgment
inside the frames written here.

## Authority and precedence

1. Owner decisions (OD-1..4 in
   `.planning/phases/89-framework-fixes-cadence-shell-kind/89-OWNER-DECISIONS.md`)
   and any newer owner message outrank everything here.
2. Root `CLAUDE.md` + `quality/PROTOCOL.md` govern gates, push cadence, build
   memory budget, threat model. This runbook never relaxes them.
3. `.planning/ORCHESTRATION.md` is canonical orchestration doctrine, including
   §11–12 (five-tier tiering, the 10%/10x rules, DP-1..5 + escalation valve
   pointer, honest corrective iteration) and the `decision-procedures` skill they
   point to. This runbook no longer amends that doctrine (it was promoted here
   2026-07-05, since it's general, not arc-specific) — it only APPLIES it to the
   P92→v1.0 drive's specific portions and phase frames (below).
4. This runbook. Fix-it-twice applies to it: when the owner catches a miss, patch
   the code AND the relevant chapter here (and ORCHESTRATION.md if doctrine-level).

## Amendments 1–3 — general doctrine now lives in ORCHESTRATION.md §11–12

The five-tier delegation model, the 10%-of-own-context end-state rule, the 10x-optimism
rule, and honest corrective iteration (HCI) are **general orchestration doctrine, not
specific to this drive** — promoted to `.planning/ORCHESTRATION.md` §11–12 (2026-07-05
permanence audit) so they survive this runbook's archival at v1.0. Read those sections
for the full tiering table, budget rules, and HCI mechanics; the DP-1..5 procedures +
escalation valve they reference live in `.claude/skills/decision-procedures/SKILL.md`.
This runbook amends nothing further here — it only APPLIES that doctrine to the
P92→v1.0 drive's specific shape:

- **L0 portion arithmetic for this drive:** the whole drive to v1.0 is ~5 portions
  (ch.03 §A); the ~10% budget applies to the DRIVE, not each portion — children (L1
  portion coordinators) absorb the 10x blowup inside THEIR contexts, never L0's. A
  drive tracking above ~10% of L0 context with portions still outstanding is a scoping
  failure: split further or rotate L0, don't push through.
- **Splits pre-authorized for this drive:** the a/b split (P92a/P92b-style) is written
  into each phase frame (ch.03 §B) so no child needs to ask permission to split.
- **HCI instances specific to this drive:** the P96-style honesty spot-check (author ≠
  orchestrator) and the 9-probe milestone verdict (ch.03 §E) are this drive's concrete
  applications of the general HCI rule.

Owner directives that motivated the original amendments (2026-07-05, for provenance):
*"the only way to scope out the work well is to go up to 4 levels deep in agent
recursion"*; *"the top level agent needs to scope out very large portions of work that
are expected to only consume 10% of the context to reach the end state"*; *"assume that
work is 10x more complex than originally planned"*; *"Iterations are honest corrective
review is crucial."*

## Chapter map

| Chapter | Contents | Read when |
|---|---|---|
| `01-decision-procedures.md` | This drive's class exemplars for DP-1..5 + the valve — the general procedures themselves live in `.claude/skills/decision-procedures/SKILL.md` | Whenever a judgment call appears |
| `02-loops-and-context.md` | The 5 loops (per-phase, litmus-REOPEN, audit-fleet+drain, re-audit-to-convergence, timers/steward), per-tier context budgets, rot-signal checklist, rotation + relay protocols, periodic-reminder table | Before first dispatch; re-read §A at every phase close |
| `03-road-to-v1.md` | The 5-portion map, pre-framed decision points P93/P95/P96/P97, launch-readiness scoping rubric, v0.13.2, v0.14→v1.0 formalization, ADR-009 activation, the gradeable v1.0 end-state checklist | At each portion boundary; §H before declaring v1.0 |

## Decision ledger

All valve-adjacent decisions land in **`.planning/CONSULT-DECISIONS.md`** — format +
routing rules: `.claude/skills/decision-procedures/SKILL.md` § "Decision ledger". An
empty ledger after a multi-phase run is itself a red flag.

## Relationship to existing surfaces

- `.planning/SESSION-HANDOVER.md` — ground truth for the *current* rotation
  (HEAD, holds, in-flight state). This runbook is the standing process; the
  handover is the perishable state. Read both.
- `.planning/ORCHESTRATION.md` — doctrine long-form (coordinator 5 rules, handover
  template, cadence A/B, tangent scoping, relay, external-mutation approval,
  §11–12 tiering/budgets/HCI). This runbook cites rather than restates it.
- `.claude/skills/decision-procedures/SKILL.md` — DP-1..5 + escalation valve + fable
  consult template + ledger format, in full. Load it when a judgment call appears.
- `.claude/skills/coordinator-dispatch/SKILL.md` — the paste-ready dispatch
  template. Use it for every L1/L2 dispatch.
- Hooks (`.claude/hooks/`) inject JIT reminders automatically; ch.02 §E lists what
  they cover vs what YOU must remember procedurally.

## Successor-session bootstrap prompt

Paste this (or its close paraphrase) as the opening frame of the next top-level
session:

> You are the top-level orchestrator (L0, opus) for reposix at
> `/home/reuben/workspace/reposix`, branch `main`. Operating tier: no-fable at
> any level you control, EXCEPT single-shot fable consults when the escalation
> valve prescribes them; report-only diet — you route, decide, and integrate;
> you never read source files, run builds, or edit the tree yourself. Scope
> delegations so large that the entire drive reaches its end state by the time
> you have spent ~10% of your own context — children absorb the 10x blowup in
> their contexts, not yours; your remaining 90% is correction margin, never
> planned workload; delegate every read >100 lines; consume ≤400-word reports.
> — Read order: (1) STATE.md (2) SESSION-HANDOVER.md (3) RUNBOOK-TO-V1/index.md
> + ch.01–03 (4) ORCHESTRATION.md (5) v0.13.0 ROADMAP next-phase section only.
> JIT layer: hooks inject dispatch checks — heed them; the `coordinator-dispatch`
> skill is your dispatch template; the `decision-procedures` skill is the
> authority for DP-1..5 + valve E1–E4 + the CONSULT-DECISIONS ledger — load it
> whenever a hook or situation names it. — Standing rules: five-tier recursion;
> assume every charter is 10x more complex than planned — recon first, split
> until children fit budgets at 10x; honest corrective iteration at every tier
> (author never self-grades); ONE cargo invocation machine-wide; push origin
> main before verifier dispatch; no dispatch over undrained BLOCKERs; escalate
> only per the valve, otherwise decide-and-record. — First three actions: 1.
> Ground-truth check (delegate): git/CI/holds (PR #61, tag script `.disabled`)
> and RE-DERIVE all counts/dates from the repo — numbers in this prompt may
> have drifted. 2. Steward window: waiver clocks, orphan processes, stale
> doc-alignment rows, scheduled-workflow health. 3. Dispatch Portion-1's L1
> coordinator (`model: opus`, charter = ch.03 §B: v0.13.0 close-out P92–P97):
> P92 first, litmus T1+T4, REOPEN on ≥1 HIGH; pre-authorized P92a/P92b split;
> relief pre-planned at ~50%.
