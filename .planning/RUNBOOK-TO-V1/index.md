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
3. `.planning/ORCHESTRATION.md` is canonical orchestration doctrine. This runbook
   AMENDS it in exactly three places (below): the no-fable tier table, the
   five-tier recursion / 10x scoping rule, and honest corrective iteration.
   Where they conflict, the amendment wins for the P92→v1.0 drive.
4. This runbook. Fix-it-twice applies to it: when the owner catches a miss, patch
   the code AND the relevant chapter here (and ORCHESTRATION.md if doctrine-level).

## Amendment 1 — five-tier delegation model (owner directives, 2026-07-05)

Owner, verbatim: *"the only way to scope out the work well is to go up to 4 levels
deep in agent recursion"*; *"the top level agent needs to scope out very large
portions of work that are expected to only consume 10% of the context to reach the
end state. That means L2 agents have very large work and are themselves
coordinators"*; *"L3 is where most of the actual work happens and need to make case
by case calls on what work needs to be delegated to L4 agents e.g. explore agents,
review agents, etc."*

| Tier | Role | Model | Scope + discipline |
|---|---|---|---|
| **L0** | Top-level orchestrator | opus | Scopes the drive into **very large portions** such that the ENTIRE drive reaches its end state by the time L0 has spent **~10% of its own context, total** — not 10% per portion. Report-only diet; routes, decides, integrates. Never reads source, never builds, never edits. |
| **L1** | Portion coordinator | opus (`model: opus` on a `phase-coordinator` dispatch) | Owns one large portion (a milestone close-out, a whole new milestone). A pure coordinator: charters L2s, arbitrates, integrates verdicts. |
| **L2** | Phase / drain-window coordinator | opus for security-judgment phases, sonnet otherwise | **Still a coordinator with very large work** — owns a phase or debt-drain window end-to-end by chartering L3 lanes. Never leaf work. |
| **L3** | Work lanes (executors, runners, fixers) | sonnet default | **Where most actual work happens.** Makes **case-by-case calls** on what to push down to L4 — recon before edit, review before report, digest instead of raw read. ≤100 tool calls; if bigger, split via L2. |
| **L4** | Helper leaves | haiku mechanical, sonnet for review judgment | `Explore` recon, `reader-digester` digests, reviewer lanes, single-file mechanical edits, gate runners. Terminal — no further spawning. |

Fable exists in exactly one place: the **single-shot consult valve** (ch.01 §Valve),
dispatched by L0 with `model: fable`, one bounded question per dispatch. Never a
standing fable coordinator; never fable at a leaf (OD-4 item 1 still holds).

**L0 portion arithmetic:** the whole drive to v1.0 is ~5 portions (ch.03 §A), but
the ~10% budget applies to the DRIVE, not each portion — plans are chronically
optimistic (Amendment 2's 10x rule), so children (L1 portion coordinators) absorb
the blowup inside THEIR contexts, never L0's. L0's remaining ~90% is correction
margin — for verdicts, decisions, and the unexpected — never planned workload. The
drive tracking above ~10% of L0 context with portions still outstanding is a
**scoping failure**: split further or rotate L0, don't push through.

## Amendment 2 — the 10x rule

Owner, verbatim: *"The operating principle should be to assume that work is 10x
more complex than originally planned."* Binding at every dispatch, every tier:

- **Recursion is the scoping mechanism.** Scoping happens at every level, not once
  at the top. L0 sizes portions; L1 sizes phases; L2 sizes lanes; L3 sizes its L4
  delegations. A level whose charter exceeds its budget splits and delegates down —
  it does not stretch.
- **Recon before dispatch.** Before ANY dispatch, run cheap reconnaissance (an L4
  `Explore`/`reader-digester` over the touched surface) and re-size assuming the
  work is 10x the plan's implied effort. Split until each child charter fits its
  budget EVEN IF it turns out 10x. "This fits in one lane" is a hypothesis, not a
  plan.
- **Splits are pre-authorized, not exceptional.** Write the a/b split
  (P92a/P92b-style) into the dispatch charter so the child never has to come back
  for permission to split.
- **What does NOT multiply:** the build memory budget (ONE cargo invocation
  machine-wide across all five tiers — `cargo-mutex.sh` enforces), the
  one-tree-writer-at-a-time rule, and report size (≤400 words at every boundary).
  More agents means MORE discipline on shared resources, not less.
- **Cost control at depth:** haiku for every mechanical leaf; digests not raw
  reads at every boundary; evidence committed once, never re-transmitted up the
  tree.

## Amendment 3 — honest corrective iteration (HCI)

Owner, verbatim: *"Iterations are honest corrective review is crucial."* Every
deliverable, at every tier, passes an **independent corrective review** before it
counts as done — and iteration to convergence is the expected shape, not a failure:

- **Author never self-grades.** L3 work → an L4 reviewer lane (fresh context,
  grades from artifacts); phase → unbiased verifier subagent (existing PROTOCOL.md
  template); audit → fresh-eyes re-audit (ch.02 §A-L4, cap 3); milestone → 9-probe
  verdict + the P96-style honesty spot-check (author ≠ orchestrator).
- **Corrective loop:** review finds HIGH/RED → targeted fix lane → re-review by a
  FRESH reviewer (not the one whose findings are being checked, and never the
  author). Two failed iterations at the same gate → escalation valve E4 (ch.01).
- **Honesty checks are part of review:** an empty findings section from a
  code-touching review is itself a red flag; reviews cite file:line evidence;
  "PASS because the author said so" is not a grade. Skipped-finding + empty-intake
  combinations grade RED (OP-8 honesty check).

## Chapter map

| Chapter | Contents | Read when |
|---|---|---|
| `01-decision-procedures.md` | The 5 fable-shaped decision procedures (DP-1..5), the escalation valve (E1..E4), the fable consult-dispatch template, the decision ledger | Before first dispatch; whenever a judgment call appears |
| `02-loops-and-context.md` | The 5 loops (per-phase, litmus-REOPEN, audit-fleet+drain, re-audit-to-convergence, timers/steward), per-tier context budgets, rot-signal checklist, rotation + relay protocols, periodic-reminder table | Before first dispatch; re-read §A at every phase close |
| `03-road-to-v1.md` | The 5-portion map, pre-framed decision points P93/P95/P96/P97, launch-readiness scoping rubric, v0.13.2, v0.14→v1.0 formalization, ADR-009 activation, the gradeable v1.0 end-state checklist | At each portion boundary; §H before declaring v1.0 |

## Decision ledger

All valve-adjacent decisions land in **`.planning/CONSULT-DECISIONS.md`**
(append-only; create on first use). One section per decision:

```
## <YYYY-MM-DD> [SELF|FABLE|OWNER] <one-line question>
- Context: <2-3 lines>
- Decision: <what was decided>
- Rationale: <why; what evidence>
- Reversibility: <how to undo, or IRREVERSIBLE>
- Commit: <sha of the change that implements it, when applicable>
```

`[SELF]` = decide-and-record below the valve bar. `[FABLE]` = consult verdict
(ch.01 template). `[OWNER]` = owner stop + reply. The ledger is the successor's
proof that judgment was exercised, not skipped — an empty ledger after a
multi-phase run is itself a red flag.

## Relationship to existing surfaces

- `.planning/SESSION-HANDOVER.md` — ground truth for the *current* rotation
  (HEAD, holds, in-flight state). This runbook is the standing process; the
  handover is the perishable state. Read both.
- `.planning/ORCHESTRATION.md` — doctrine long-form (coordinator 5 rules, handover
  template, cadence A/B, tangent scoping, relay, external-mutation approval).
  This runbook cites rather than restates it.
- `.claude/skills/coordinator-dispatch/SKILL.md` — the paste-ready dispatch
  template. Use it for every L1/L2 dispatch; add the model override and the
  10x/recursion/HCI clauses from Amendments 1–3.
- Hooks (`.claude/hooks/`) inject JIT reminders automatically; ch.02 §E lists what
  they cover vs what YOU must remember procedurally.

## Successor-session bootstrap prompt

Paste this (or its close paraphrase) as the opening frame of the next top-level
session:

> You are the top-level orchestrator (L0, opus) for reposix at
> `/home/reuben/workspace/reposix`, branch `main`. Operating mode: no-fable,
> report-only diet — you route, decide, and integrate; you never read source
> files, run builds, or edit the tree yourself. Scope delegations so large that
> the entire drive reaches its end state by the time you have spent ~10% of your
> own context — children absorb the 10x blowup in their contexts, not yours; your
> remaining 90% is correction margin, never planned workload. Delegate every read
> >100 lines; consume ≤400-word reports.
>
> Read order (digests, not raw reads, for 4 and 5):
> 1. `.planning/STATE.md`
> 2. `.planning/SESSION-HANDOVER.md`
> 3. `.planning/RUNBOOK-TO-V1/index.md`, then chapters 01–03 — your operating manual
> 4. `.planning/ORCHESTRATION.md` (doctrine; §3 relief template)
> 5. v0.13.0 ROADMAP — next-phase section only
>
> Standing rules: five-tier recursion (L0 portions → L1 portion coordinators →
> L2 phase coordinators → L3 work lanes → L4 helpers: explore/review/digest);
> assume every charter is 10x more complex than planned — recon first, split
> until children fit budgets at 10x; honest corrective iteration at every tier
> (author never self-grades; iterate to convergence, cap 3, then valve); ONE
> cargo invocation machine-wide; push origin main before verifier dispatch; no
> dispatch over undrained BLOCKERs; escalate only per the valve (RUNBOOK ch.01),
> otherwise decide-and-record in `.planning/CONSULT-DECISIONS.md`.
>
> First three actions:
> 1. Ground-truth check (delegate to L4): git log/status vs SESSION-HANDOVER §1;
>    `gh run list` (CI green?); QUALITY-LEDGER BLOCKER scan; holds intact
>    (PR #61 untouched, `tag-v0.13.0.sh` still `.disabled`).
> 2. Steward window (steward agent, owner-named targets only): waiver clocks
>    (file-size 2026-08-08; security 2026-08-15), orphan processes,
>    `JIRA_TEST_PROJECT=KAN` secret gap, 17 stale doc-alignment rows → schedule
>    top-level `/reposix-quality-refresh`.
> 3. Dispatch Portion-1's L1 coordinator (`model: opus`, charter = ch.03 §B:
>    v0.13.0 close-out P92–P97): P92 first, litmus T1+T4 (sim + TokenWorld),
>    REOPEN on ≥1 HIGH; pre-authorized split P92a/P92b if recon sizes it >16h;
>    relief pre-planned at ~50% context at every coordinator tier.
