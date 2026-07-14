# ORCHESTRATION-REFERENCE.md — deep-dive companion to ORCHESTRATION.md

Episodic / reference detail extracted out of `.planning/ORCHESTRATION.md` to keep that
file inside the progressive-disclosure ceiling. **`ORCHESTRATION.md` is the canonical
doctrine a coordinator reads before its first dispatch; THIS file holds the material a
coordinator consults only when a specific situation arises** (a leaf-isolation BLOCK to
diagnose, a large-framework proposal to scope, a doctrine edit to sweep). Each section
below is pointed at from the matching `ORCHESTRATION.md` section.

## Enforcement map (full table)

Companion to `ORCHESTRATION.md` § "Enforcement map" — which doctrine is hook-/permission-
enforced vs. a judgment call, and the exact enforcing artifact. Everything NOT listed here
(or listed as prose/skill) is a judgment call: the ORCHESTRATION.md prose is the standard.

| Doctrine | Enforced by | Layer |
|---|---|---|
| One cargo machine-wide | `.claude/hooks/cargo-mutex.sh` (exit 2) | blocking hook |
| Leaf isolation (reposix/sim/git test setup) | `.claude/hooks/leaf-isolation-guard.sh` (exit 2) + `.githooks/pre-commit` | blocking hook |
| Uncommitted = didn't happen | `.claude/hooks/stop-uncommitted.sh` (exit 0 + systemMessage) | advisory hook (owner decision Q2) |
| Persist before compaction | `.claude/hooks/precompact-persist.sh` | state hook |
| Session brief on startup | `.claude/hooks/session-start-brief.sh` | state hook |
| Tier check / charter / lane size at dispatch | `.claude/hooks/dispatch-doctrine.sh` | JIT injection |
| Check tree before re-running a lane | `.claude/hooks/post-dispatch-relay.sh` | JIT injection |
| Coordinator role + 5 rules + tier table | `.claude/agents/phase-coordinator.md` | agent-def |
| External mutation approval | `.claude/settings.json` + `.claude/agents/steward.md` | permission + agent-def |
| Context %/relief nudge | user-level `gsd-context-monitor.js` (≤35%/≤25%) | JIT injection |
| Intention-over-plan; tangent scoping; cadence | ORCHESTRATION.md + `coordinator-dispatch` skill | prose + skill |
| Judgment-call procedures (DP-1..5) + escalation valve (E1-E4) + consult template | `.claude/skills/decision-procedures/SKILL.md` | on-demand skill (loaded when a judgment call appears, not every session) |

## Leaf-isolation enforcement mechanism (v0.14.0 P102)

Companion to `ORCHESTRATION.md` § "Leaf isolation for reposix/sim/git test setup (HARD
STOP)" — the prose hard-stop there is the human-readable contract; this is how it is now
backstopped MECHANICALLY, fail-closed, and proven by a verifier that REGENERATES its
triggered-hook transcript at grade-time. The prose rule is retained (not deleted) as the
readable contract.

- `.claude/hooks/leaf-isolation-guard.sh` — one combined PreToolUse Bash hook, three
  fail-closed guards (exit 2 = BLOCK, first-match-blocks): **A** fixture-identity (`t <t@t>`)
  reject, **B** leaf-setup-verb location, **C** shared-`.git/config` write
  (`core.bare`/`user.email`/`user.name`). Each block teaches the rule + why + the exact
  `/tmp`-clone recovery; the mechanism never invokes `git worktree remove --force` (itself a
  corruption vector). **Hardened against evasion (P102 adversarial fix lane):**
  - Guard B catches the **canonical dev spellings**, not just literal `reposix init`:
    path-suffixed (`/usr/bin/reposix init`, `./target/debug/reposix init`) and cargo
    (`cargo run -p reposix-cli -- init|attach|sync`), plus bare `attach`/`sync` and sim-seed.
  - The `/tmp`-is-safe decision resolves the **effective** target (last `cd`, or a git
    path-flag like `-C`/`-f`/`--file`/`--git-dir`, else the payload cwd) and
    **realpath-canonicalizes** it — so a `cd /tmp/x && cd <shared>` cd-back, a
    `/tmp/../<shared>` traversal, or a `/tmp` symlink that resolves back to the shared tree
    all BLOCK; a genuine `/tmp` target still passes (no over-block).
  - Guard A tolerates **quotes** around the fixture email (`'t@t'`/`"t@t"`) while keeping a
    delimiter boundary so a real address (`scott@things.io`) does not false-positive.
  - A non-empty but **unparseable** tool payload **fails closed** (exit 2), never silently
    falling through to allow; an empty payload (nothing to inspect) still passes.
- `.githooks/pre-commit` — git-native defense-in-depth: rejects a commit whose resolved
  `git var GIT_AUTHOR_IDENT`/`GIT_COMMITTER_IDENT` is the fixture identity. Fires only in
  the shared repo (via `core.hooksPath`); a `/tmp` fixture clone does not inherit it.
- Catalog rows (`agent-ux/fleet-safety-{tat-identity-reject,leaf-isolation-enforce,
  shared-config-write-guard}`, kind `shell-subprocess`) grade the guards by re-running the
  verifier at grade-time (which writes a fresh, gitignored transcript) — the durable
  committed proof is the verifier script + the hook, NOT a frozen transcript.
- **Coverage boundary (honest):** the PreToolUse hook fires only on the Claude Code Bash
  *tool* — a git/reposix write spawned by a subprocess or script bypasses it. The pre-commit
  backstop catches fixture *commits* on that bypass path, but NOT `reposix init` / `git
  config` writes. A binary-side / git-alias non-tool backstop for guards B/C is filed
  (`.planning/milestones/v0.14.0-phases/GOOD-TO-HAVES.md`). Until then the prose hard-stop
  remains the enforcement of record for the uncovered surface — coordinators still bake the
  `cd /tmp` shape into every dispatched setup lane.

## Large-framework proposals are expected work, not scope-creep

Companion to `ORCHESTRATION.md` §5 (Tangent scoping). When a lane spots leverage that pays
off across THIS project and others — a reusable framework, a cross-cutting guardrail, an
infra investment that deviates substantially from the current charter — surfacing and
scoping that proposal (a ~10-line scope memo: what, why now, cross-project value, cost,
cost-of-delay) is a first-class deliverable. The quality-infra milestone is the canonical
precedent: an unplanned tangent that became the backbone. The owner still gates scope/spend
(valve E3) — but the valve gates **approval, never the surfacing**. Withholding a
high-leverage proposal because "it's not on the plan" is the failure mode, not the
discipline.

## Budget over-fan anti-pattern (§11 detail)

Companion to `ORCHESTRATION.md` §11 (Budgets). **Consolidate recon into ONE lane per
dispatch decision** — absorb a single synthesized conclusion, never N report-bearing agents
fanned into your own window. Splitting preflight by source (git/CI/steward/charter) and
reading each report yourself is the over-fan anti-pattern: it makes you an L1 gathering
evidence, not an L0 receiving a verdict. Past ~5% of your context before the first
coordinator is dispatched → you're over-fanning; stop and consolidate. What does NOT
multiply: one cargo machine-wide, one-tree-writer-at-a-time, ≤400-word reports (§2 and the
enforcement map already cover these — more agents means more discipline on shared resources,
not less).

## Honest corrective iteration (HCI) — full text (§12)

Companion to `ORCHESTRATION.md` §12. Every deliverable, at every tier, passes an independent
corrective review before it counts done, and iterating to convergence is expected, never a
failure: **the author never self-grades** — a fresh reviewer or verifier grades from
committed artifacts only (this project's standing verifier-subagent mandate,
`quality/PROTOCOL.md`, is the phase-level instance of this rule; audits get a fresh-eyes
re-audit capped at 3 iterations; a milestone gets the 9-probe verdict plus an independent
honesty spot-check, author ≠ orchestrator). A review that finds HIGH/RED gets a targeted
fix, then a re-review by a DIFFERENT fresh reviewer (never the one being checked, never the
author) — two failed iterations at the same gate escalates to valve E4. An empty findings
section from code-touching review is itself a red flag; "PASS because the author said so" is
not a grade.

## Provenance, permanence audit, and the promotion-sweep standing rule

Companion to `ORCHESTRATION.md` § Provenance.

Distilled from session 7e2a4cf2. Full 14-theme inventory with verbatim owner quotes,
per-theme encoding-status, and the coverage check:
`.planning/research/doctrine-institutionalization/` (index + theme chapters + REPORT.md).
Prior committed homes: `89-OWNER-DECISIONS.md` (OD-1..4), `.planning/PRACTICES.md`
(OP-8/OP-9), `quality/SURPRISES.md` (D-CONV-1..8). ORCHESTRATION.md supersedes the
session-local accumulator (`~/.claude/jobs/7e2a4cf2/tmp/PENDING-INTAKE-AND-CHORES.md`).

**2026-07-05 permanence audit.** §11–12 (five-tier tiering, the 10%/10x rules, the DP-1..5 /
escalation-valve pointer, honest corrective iteration) were promoted into ORCHESTRATION.md
from an arc-scoped runbook's amendments (that runbook itself is unchanged in purpose, just
relieved of restating general doctrine) — they are general doctrine, not specific to any one
drive, so they belong in a permanent home rather than only in a runbook that archives when
its drive ends. The deep procedural bodies (DP-1..5 full triggers/evidence/decide/escalate
text, the E1-E4 valve table, the fable consult-dispatch template, the
`.planning/CONSULT-DECISIONS.md` ledger format) live in
`.claude/skills/decision-procedures/SKILL.md` rather than inline, per the owner's steer that
episodic procedures belong in on-demand skills, not in doctrine every session re-reads in
full — ORCHESTRATION.md stays the distilled core; the skill is the reference manual.
Permanent homes never hard-reference a transient/arc-scoped file by path (only the reverse
is safe) — ORCHESTRATION.md describes any arc-scoped runbook generically rather than naming
one, so it never goes stale when a runbook archives.

**Promotion sweep (standing rule).** Any doctrine change or promotion into `ORCHESTRATION.md`
MUST end with a stale-reference sweep across `.claude/hooks/`, `.claude/agents/`,
`.claude/skills/`, and the SessionStart brief, grepping for superseded numbers, tier names,
and rule phrasings. These injected surfaces are trusted and never re-read by the agents they
instruct — a stale hook poisons every future session silently; the 17b1c94 promotion skipped
this sweep and left the SessionStart brief teaching the superseded ~50% budget until a
fresh-agent audit caught it (fixed fe5e8f2).

## Operating cadence A/B (§4 detail)

Companion to `ORCHESTRATION.md` §4 (the "No dispatch over undrained BLOCKERs" gate stays
inline there). Two standing operations, interleaved:
- **A — Phase chain.** One fable coordinator per phase, owns the tree, reports a verdict.
- **B — Quality upkeep.** Read-only audit fleets (`audit-fleet-lane`) run DURING phases
  (parallel-safe). A **debt-drain window** runs BETWEEN every phase close and the next
  dispatch: drain `eager-window` ledger rows (sonnet fixers, one tree-writer at a time),
  route `intake-P<N>` rows into intake files, mint `catalog-row` candidates.

## Orchestrator relay for mis-routed replies (§8 detail)

Companion to `ORCHESTRATION.md` §8. When a subagent's reply cannot be delivered
(cross-session addressing failure), the orchestrator relays the full report inline rather
than silently dropping it. Before re-running any lane, **check the agent tree / git log
first** — the work may already be committed. (Episode; `post-dispatch-relay.sh` injects this
reminder.)

## Bottom-up triage routing (§2 detail)

Companion to `ORCHESTRATION.md` §2 (the principle — never drop a raised item — stays
inline). Every sub-agent report carries a NOTICED section + RAISE LIST + intake disposition
(rule 3 + ownership charter). The parent **triages each item and routes it, never drops it**:
(a) low-deviation from the current charter and it fits the 10x capacity → **absorb** into
this wave (replan the wave; DP-5 charter test); (b) real work outside this charter →
**re-delegate** as a new lane, or hand to the owning downstream phase as a RAISE-LIST item;
(c) larger-than-intake → **file / escalate** (OP-8; valve). A leaf's reported friction that
surfaces in NO commit, NO intake row, and NO re-dispatch is a dropped deliverable — the same
red flag as an empty noticing section. Leaves report friction UP because they cannot see the
whole charter; making the absorb-vs-redelegate call is the parent's job because it can.

## Pause/resume at logical boundary (§6 detail)

Companion to `ORCHESTRATION.md` §6. Pause is a first-class operation the owner invokes
mid-flight (e.g. to restart with new permissions). The correct response is NOT to stop
immediately — **finish the current atomic unit, then produce a durable, complete handover**
(template §3) before standing down. A resumed session opens with ground-truth git, then the
handover's reading list. (Directives #4, #11, #14.)

## Mission over plan — resequencing exemplar (§10 detail)

Companion to `ORCHESTRATION.md` §10. Exemplar of executive resequencing: OD-4 item 3 pulled
a launch-readiness milestone AHEAD of v0.13.2 cross-link, re-derived from the owner's stated
intention ("puts this project on the global map"), not the existing phase order.
