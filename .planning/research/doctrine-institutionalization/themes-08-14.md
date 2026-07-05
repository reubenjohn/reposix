# Themes 8–14 — doctrine directive inventory (chapter 2 of 3)

Sharded verbatim from the session-7e2a4cf2 directive inventory (`DIRECTIVES.md`,
now split for the 20,000-byte generic-md cap). Back to `index.md`. Companion
chapters: `themes-01-07.md` (themes 1–7), `coverage-check.md` (full coverage table).

---

## Theme 8 — Build memory budget: one cargo process machine-wide

**Normative statement:** Never run more than one cargo invocation
(check/build/test/clippy) at a time across the whole machine, because
parallel cargo workspace builds have crashed the VM from RAM pressure.
Prefer per-crate over workspace-wide checks; use `--jobs 2`; avoid
rust-analyzer + cargo contention.

**Verbatim quotes:**

> "The system crashed for some reason likely because we ran out of memory."
> — directive #18

> "Be careful about parallel agents issuing large rust related commands." —
> directive #19

**Session evidence:** This is the cleanest one-line-caution → codified-rule
transformation in the session. #18 confirms a *real, observed* OOM crash at
03:38 (not hypothetical); #19, one minute later, is the verbatim owner
instruction. It was later expanded into `CLAUDE.md`'s full "Build memory
budget" section (hard rules 1–2, soft rules on `--jobs 2`, `cargo nextest`,
rust-analyzer contention) — and the crash recurred (directive #10 confirms
"the OS had shutdown... probably because of inactivity" as a separate,
later, non-OOM crash during P90; the accumulator separately logs "OOM
context: VM's 3rd crash 2026-07-03").

**Current encoding status:** **COMMITTED**, fully, at `CLAUDE.md` § "Build
memory budget (load-bearing — read before parallelizing)". This is the
single most completely institutionalized directive in the inventory — full
hard/soft rule breakdown, in the file every agent reads first.

---

## Theme 9 — Durable state over chat; crash-recovery doctrine

**Normative statement:** Environment/session crashes are recoverable, not
catastrophic — recovery means resuming orphaned background agents from
their on-disk transcripts, not restarting from scratch. But anything that
lived only in an ephemeral `/tmp` file and was never committed is
permanently and honestly lost — reconstruct from git history with
documented provenance, never fabricate what can't be recovered.

**Verbatim quotes:**

> "The OS had shutown (probably because of inactivity). I will fix it.
> Please continue your work." ... "Resume any previous agents by sending
> them messages." — directive #10 (repeated three times: 13:37, 13:43,
> 13:48)

**Session evidence:** This crash killed a background coordinator
("Quality convergence: blockers, unify, drain") mid-flight. The concrete,
costly consequence: the 170-row `/tmp` `QUALITY-LEDGER.md` was **lost**
across the crash because no committed copy existed, while `D-CONV-1..6`
work survived because it had already been committed. It was reconstructed
post-hoc as `.planning/audits/QUALITY-LEDGER.md` "with documented provenance
rather than fabricated" — i.e., the recovery was honest about what was lost
rather than papering over the gap. `STATE.md`'s `last_activity` field
independently confirms: *"ledger reconstruction at
.planning/audits/QUALITY-LEDGER.md"*.

**Current encoding status:** **NOT ENCODED** as a general principle.
`CLAUDE.md`'s existing OP-4 ("No hidden state") is adjacent but narrower in
scope — it's about cache/simulator/helper *runtime* state having no
"works in my session" bugs, not about orchestrator-owned planning artifacts
surviving crashes. The specific lesson — "if it's not committed, an OS crash
erases it, and the honest response is documented reconstruction, not
fabrication" — has no general home yet, though the *practice* of moving
transient tracking into committed form is itself precedent (D-CONV-7's "SURPRISES.md
revival" and "every removed block gains a named home that was already the
authoritative source" — see Theme 14).

---

## Theme 10 — Pause-at-logical-boundary protocol and handover artifacts

**Normative statement:** Pause is a first-class operation the owner can
invoke mid-flight (e.g. for permission-mode changes) — but the correct
response is not to stop immediately, it's to finish the current atomic unit
and then produce a durable, complete handoff artifact before standing down.
This recurred as a protocol, not a one-off.

**Verbatim quotes:**

> "Can you pause because I want to reopen the session with permissions in
> auto mode" — directive #4

> "Can you please pause at the earliest possible logical point because I
> want to restart the session with appropriate permissions again." —
> directive #11

**Session evidence:** #4 and #11 are near-verbatim repeats of the same
request at two different points in the session, establishing this as a
recurring protocol rather than incidental. The concrete artifacts this
produced: `.planning/phases/90-framework-fixes-honesty-rules/90-PAUSE-HANDOFF.md`
(mid-phase pause, same coordinator resumes — structured as ground-truth-git,
phase-cycle-position, binding-artifacts-reading-list, operating-constraints,
close-requirements, waiver-clock) and
`.planning/phases/91-attach-sync-real-backend-wiring/91-HANDOVER.md`
(coordinator-relief shape, committed at `edc3d52` — wave-state table,
named-incident post-mortem, litmus/REOPEN state, mid-execution decisions not
yet formalized, "noticed, not yet filed" section, precise numbered
next-steps runbook). Both share a skeleton: ground-truth-git opener → what's
done/pending → binding constraints → numbered next steps → close-ritual
checklist. This shared skeleton is itself unwritten-but-real doctrine (see
Theme 2/3 for why relief happens, and directive #14 for why the *content* of
these handoffs must be solicited in advance).

**Current encoding status:** **PARTIALLY encoded** — the two files are
**COMMITTED** as strong worked exemplars (a future coordinator can literally
copy their structure), but no general prose rule ("pause = finish atomic
unit then write a structured handoff with these 6 sections") is written down
anywhere as an explicit template. The pattern must currently be
reverse-engineered from reading both files side by side (as
`repo-artifacts-digest.md` § (b) already did, for exactly this reason).

---

## Theme 11 — Orchestrator relay for mis-routed subagent replies

**Normative statement:** When a subagent's reply cannot be delivered to its
intended recipient (e.g. a cross-session addressing failure), the
orchestrator relays the full report inline as a fallback rather than
silently dropping it. A coordinator watching for this must check the
existing agent tree before blindly re-running a lane that may have already
completed.

**Verbatim quotes:** No direct owner quote — this is an **episode**, not an
owner instruction, surfaced only in the cross-reference section of the
directives extract:

> "Mis-routed subagent reply: a 'P90 plan authoring' agent's reply 'could
> not be delivered — no agent named `general-purpose` is reachable from this
> session' (cross-session addressing failure), relayed its full report
> inline as a fallback instead of silently failing." — episode
> cross-reference, `7e2a4cf2-directives.md`

**Session evidence:** This is the dispatcher's known-directive (i)
("orchestrator relay: sub-agent replies mis-route; check tree before
re-running lanes") validated against the sources — it maps to a real,
observed episode in this session, but has **no owner-authored verbatim
quote** behind it; it is orchestrator-discovered operational behavior, not
an instruction the owner gave. Flagged explicitly so it is not conflated
with a directive.

**Current encoding status:** **NOT ENCODED.** No rule, no mention, anywhere
in `CLAUDE.md`/`PRACTICES.md`/`quality/`. This is purely episodic —
recorded once, in the accumulator's cross-reference notes and the curated
directives extract, never turned into a standing instruction.

---

## Theme 12 — External/security-sensitive mutations require owner-named-target approval

**Normative statement:** Actions that mutate state outside the local repo
(remote branch/PR operations, bypassing a secret scanner) or that touch
security posture require an explicit owner approval naming the specific
target — a permission classifier's refusal to let an agent self-authorize
such an action is correct behavior, not a bug to route around.

**Verbatim quotes:**

> "It turns out the google API key was a false positive. Please proceed." —
> directive #9

**Session evidence:** This unblocked a subagent that had halted a commit
because the permission classifier correctly refused to let it autonomously
commit a `gitleaks:allow` scanner-bypass annotation over what looked like a
leaked secret. The lesson is explicitly two-sided: the classifier's refusal
was *correct* (security-posture decisions are owner-only, even for a
demonstrated false positive), and the fix is the owner's explicit,
named unblock — not the agent guessing or self-authorizing. The same shape
recurs structurally in the accumulator's "Steward window checklist," which
lists a "Safe-now batch (blocked by permission classifier 2026-07-03, needs
owner approval of named targets)" — dependabot rebases, specific PR closures,
specific branch deletions, each named explicitly rather than batched
generically.

**Current encoding status:** **NOT ENCODED** as a general rule anywhere in
committed docs. `CLAUDE.md`'s threat-model section covers the *mechanism*
(egress allowlist, `Tainted<T>`, audit tables) but not this specific
owner-approval-for-classifier-denials social contract. The steward-checklist
instance is **HIDDEN STATE only** (accumulator file), not committed.

---

## Theme 13 — Fix-it-twice meta-rule; CLAUDE.md stays current

**Normative statement:** When the owner catches a quality miss the agent
should have caught itself, the fix is two-fold: fix the concrete issue, AND
update the governing instructions (CLAUDE.md and/or long-form practices) so
the next agent's session reads the tightened rule. Shipping the fix alone,
without updating instructions, guarantees recurrence. Every phase
introducing a new file/convention/gate updates `CLAUDE.md` in the same PR by
*revising* existing sections, not appending narrative.

**Verbatim quotes:** No single new verbatim owner quote in this session
introduces this rule fresh — it is a pre-existing, already-committed
`CLAUDE.md` principle that this session's evidence *repeatedly exercises*
(directive #19's one-liner → codified "Build memory budget" rule is itself
an instance of this meta-rule firing).

**Session evidence:** Every "HIDDEN STATE" gap identified throughout this
document (Themes 2, 3, 6, 9, 11, 12) is exactly the failure mode this
meta-rule exists to prevent — a lesson was learned live in the session, an
operational rule was written down *somewhere*, but the "fix it twice" step
(propagating it into `CLAUDE.md`) did not fully happen before the session
ended, which is precisely why directive #16 (Theme 14) commissions this
extraction task in the first place.

**Current encoding status:** **COMMITTED** — the meta-rule itself already
exists verbatim in `CLAUDE.md` § "Meta-rule: when an owner catches a quality
miss, fix it twice", and "**CLAUDE.md stays current**" is a separate,
also-committed bullet in § "Quality Gates" ("Each phase introducing a new
file/convention/gate updates CLAUDE.md in the same PR"). Ironically, this
session's own accumulation of HIDDEN STATE items shows the meta-rule was not
fully applied to *itself* — the very artifact you are reading is the
belated "fix it twice" pass this rule demands.

---

## Theme 14 — Institutionalize this session's discipline: hooks/skills over prose, ground in existing infra, persist reports, scope ruthlessly

**Normative statement:** "All future sessions should operate like this
one" — but achieving that is explicitly **not** satisfied by more prose in
top-level `CLAUDE.md` alone; for highly autonomous use cases, doctrine needs
enforcement/scaffolding: subfolder-scoped CLAUDE.md files, programmatic
Claude Code hooks (not advisory text), and skills/agents. Any report
distilling this doctrine that would lose fidelity under a hard word limit
should also be persisted as a durable artifact in the codebase. New designs
should be grounded in infrastructure that already exists (e.g. the hooks
already present at `~/.claude/hooks/`) rather than invented from scratch.
When scope threatens to balloon (e.g. mining an entire session transcript),
cut ruthlessly to the highest-value target rather than spreading thin.

**Verbatim quotes:**

> "I would like all future sessions to operate like this one. All the
> directives, guidance on fable, opus, sonnet, haiku, achieving end state by
> understanding the underlying intention, rather than faithfully executing
> plans, identifying tangential enabler/tooling/maintenance work, etc. Can
> you slot in a step where a coordinator explores this session's jsonl file
> and any other files you deem relevant to see how future sessions can
> follow that discipline? Ideally this is done in more creative ways that
> just prose in the top level CLAUDE.md file. We might want to push to lower
> level CLAUDE.md files in subfolders, as programmatic hooks where possible
> via skills/agents, etc. Can you please make sure you exhaustively capture
> all the successful strategies/guidance I provided that weren't obvious to
> you or a future fable agent?" — directive #16 (session capstone)

> "I hope its clear why CLAUDE.md is a good strategy, but alone it is
> insufficient for such highly autonomous usecases that needs better
> guardrails." — late message, 2026-07-04 (today)

> "Also, worth persisting a report in the codebse in addition if the 600
> word limit is too lossy." — late message, 2026-07-04 (today)

> "There are hooks in ~/.claude/" — mid-task correction, 2026-07-04 (today)

> "Only focus on the largest one not the others" — mid-task correction,
> 2026-07-04 (today)

**Session evidence:** Directive #16 is the direct commission for this
extraction task and every artifact in `/tmp/.../DOCTRINE/`. It explicitly
names two disciplines as the ones to propagate: (1) "understand underlying
intention... rather than faithfully executing plans" (Theme 4), and (2)
"identifying tangential enabler/tooling/maintenance work" (Theme 5). It also
explicitly rejects flat CLAUDE.md prose as sufficient and names three richer
homes: subfolder CLAUDE.md files, programmatic hooks, skills/agents. Ground
truth check on "There are hooks in `~/.claude/`" confirms this is **literally
true and non-trivial**: `~/.claude/settings.json` already wires 11 real
hooks (`SessionStart` → `gsd-check-update.js`, `gsd-session-state.sh`;
`PostToolUse` → `gsd-context-monitor.js`, `gsd-phase-boundary.sh`,
`gsd-read-injection-scanner.js`; `PreToolUse` → `gsd-prompt-guard.js`,
`gsd-read-guard.js`, `gsd-workflow-guard.js`; `Bash` PreToolUse →
`gsd-validate-commit.sh`, `deny-ad-hoc-bash.js`) plus a `statusLine` hook —
these are GSD-framework hooks living in the user-global `~/.claude/hooks/`
directory, not reposix-specific, but they prove the "programmatic hooks"
mechanism the owner asked about is already live infrastructure to build on,
not a greenfield feature request. The "600 word limit" reference is to this
dispatching agent's own final-reply budget — the owner is explicitly
pre-authorizing a companion committed-artifact report *in addition to* the
terse chat reply, because 300–600 words cannot carry an inventory this size
without lossy compression. "Only focus on the largest one" is a live example
of the owner exercising exactly the tangent-scoping discipline of Theme 5 —
cutting a broader transcript-mining ask down to the single highest-value
session rather than diffusing effort.

**Current encoding status:** **NOT ENCODED anywhere yet.** This is the
freshest, least-institutionalized theme in the inventory — it is the
directive that produced the very artifacts you are reading (this file plus
its sibling extracts), and by definition nothing downstream of it has been
built yet. No subfolder CLAUDE.md files exist in the repo
(`repo-artifacts-digest.md` confirms `crates/CLAUDE.md`, `.planning/CLAUDE.md`,
`quality/CLAUDE.md`, `docs/CLAUDE.md` are all absent), no reposix-specific
`.claude/hooks/` exist (only user-global GSD hooks at `~/.claude/hooks/`),
and no new skill/agent encodes this doctrine. This entire theme is the
**punch list for the next phase of work**, not a completed practice.

---

