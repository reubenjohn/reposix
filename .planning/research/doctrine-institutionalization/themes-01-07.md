# Themes 1–7 — doctrine directive inventory (chapter 1 of 3)

Sharded verbatim from the session-7e2a4cf2 directive inventory (`DIRECTIVES.md`,
now split for the 20,000-byte generic-md cap). Back to `index.md`. Companion
chapters: `themes-08-14.md` (themes 8–14), `coverage-check.md` (full coverage table).

---

## Theme 1 — Three-tier model delegation: fable coordinates, opus/sonnet/haiku execute

**Normative statement:** The top-level orchestrator delegates ONLY to
`fable`-tier subagents, which act as coordinators and themselves further
delegate to `opus` (complex/security-judgment lanes), `sonnet` (default
implementation), or `haiku` (mechanical/leaf work). `fable` must never be
used at the leaf/implementation level — it is overkill there and the tiering
rule exists specifically to prevent that waste.

**Verbatim quotes:**

> "You're at 26% context. Not a lot of budget left to reach 50%. Delegate as
> usual. Only additional guidance I have is to not use fable subagents at the
> lowest level for actual implementation work since its overkill - hope that
> is intuitive - use sonnet or in complex cases opus." — directive #5

> "As the top level coordinator, I think its a good idea for you to only
> delegate to fable subagents which may then in turn delegate to
> opus/sonnet/haiku subagents depending on the need. I will make sure to give
> Fable the credit." — directive #8

> "Its worth considering delegating more to a fable subagent with larger
> scopes that needs to further delegate." — directive #2 (first mention,
> before the rule was made explicit)

**Session evidence:** Directive #2 is the first, softer nudge toward
fable-as-coordinator; #5 hardens it into an explicit tiering rule; #8
escalates it into a **stated hierarchy rule** ("top-level ONLY delegates to
fable; fable delegates to opus/sonnet/haiku") immediately before the 15-lane
"Repo-wide quality audit fleet" dispatch (the 170-row QUALITY-LEDGER). The
rule was later **committed verbatim** as OD-4 item 1 in
`.planning/phases/89-framework-fixes-cadence-shell-kind/89-OWNER-DECISIONS.md`
(2026-07-04): *"The top-level orchestrator delegates ONLY to fable
coordinators, which tier sub-delegation (opus for genuinely complex/security
lanes, sonnet default, haiku mechanical). Never fable at the leaf level."*
The rule was violated in practice even after being stated — see Theme 2,
directive #15 ("coordinators not workers"), which diagnoses fable-tier
coordinators doing leaf-level work inline instead of delegating further.

**Current encoding status:** **COMMITTED** at
`.planning/phases/89-framework-fixes-cadence-shell-kind/89-OWNER-DECISIONS.md`
§ OD-4 item 1, and echoed in the accumulator's "MODEL TIERING" block. **NOT
in root `CLAUDE.md`** — grepping `CLAUDE.md` for "fable" returns zero
matches. This is a real gap: the rule governs every session's delegation
shape but currently lives only in a single phase's decisions file, not in
the document every agent reads first.

---

## Theme 2 — Coordinators are routers, not workers

**Normative statement:** A coordinator's own tool calls are limited to Agent
dispatches, one-line git/gh ground-truth checks, and reading short
reports/handovers. It must never itself read source files or long plans,
run test suites/builds, write/edit repo files, or review diffs — those are
always a sub-dispatch. Every lane should stay under ~100 tool calls and
sub-agent reports must stay ≤400 words with full evidence in committed
artifacts, never chat. A coordinator must never idle-wait or poll.

**Verbatim quotes:**

> "I'm noticing that the coordinators you spawn are not able to handle the
> large scope and having their context rot because they aren't delegating to
> subagents effectively. Your subagents are coordinators not workers." —
> directive #15

> "Btw great progress and work so far! But again you need to leverage
> subagents more, the codebase needs to exhaustively be maintained for
> perfect quality - almost delegating to coordinators that further delegate
> given the scope of the project and how many things can be of sub par
> quality - every file, doc, src, CI, etc. Leave no stone unturned, don't
> race to complete features while letting technical debt and chores pile
> up." — directive #7

**Session evidence:** This is the **sharpest finding of the session**: a
second-order delegation failure diagnosed *after* the P91 coordinator needed
relief (see Theme 3/10) — mid-tier coordinators across P89, P90, and P91
generations were themselves accumulating context by doing work inline
instead of pushing it one level further down. The fix is recursive: not just
the top-level orchestrator, but *every* level of the hierarchy, must
actually delegate. The accumulator records the concrete operational rules
this produced under "COORDINATOR CONTEXT DISCIPLINE" (5 numbered rules:
route-don't-work; slice lanes small; reports small/evidence committed; never
wait inline; proactive relief — the last is Theme 3).

**Current encoding status:** **HIDDEN STATE** — the 5-rule "COORDINATOR
CONTEXT DISCIPLINE" block exists only in
`~/.claude/jobs/7e2a4cf2/tmp/PENDING-INTAKE-AND-CHORES.md`, a session-local
accumulator file living in a `/tmp`-adjacent jobs directory (`~/.claude/jobs/`),
**not in the repo at all**. Grepping the repo for "coordinators not workers"
or "coordinators, not workers" returns zero matches. This is the single
highest-value gap: the sharpest, most load-bearing lesson of the session is
currently unrecoverable by any future session that doesn't happen to read
this specific `/tmp`-adjacent file.

---

## Theme 3 — Context-budget targets and proactive relief at wave boundaries

**Normative statement:** Sessions track context percentage continuously
from turn one, not just at milestones, and target reaching the full
end-state by ~50% context (not 100%). At every wave boundary a coordinator
past ~50% of its own context must dispatch an executor to write+commit a
handover artifact, then signal for relief — relief is cheap, context rot is
not.

**Verbatim quotes:**

> "Also your context is at 18% and I'm hoping for you to reach the end state
> by 50%." — directive #2

> "You've already used up 12% of your context." — directive #17 (earliest,
> pre-target, pure awareness)

> "Maybe its a good idea to spawn a fresh coordinator at the next logical
> point to take over since the current one seems to be malfunctioning
> probably because its context is quite full." — directive #13

> "Probably also a good idea to inform it so it correctly persists the
> handover information needed." — directive #14

**Session evidence:** #17 shows context-awareness starting before any
numeric target existed; #2 introduces the 50%-of-context end-state target as
a planning constraint; #13 is the owner diagnosing context-rot *from the
outside* in a live P91 coordinator (stalling on a Wave-5.5 writer/polling
loop) and prescribing rotation as the fix — this became the "P91 relief
handover" episode, where the coordinator committed `91-HANDOVER.md` at
`edc3d52` and explicitly stood down. #14 is the critical follow-up: a
relieved coordinator must be *told in advance* it is being relieved so it
deliberately writes a durable, complete handover — persistence is solicited,
not automatic.

**Current encoding status:** **HIDDEN STATE** for the general rule (the
accumulator's "COORDINATOR CONTEXT DISCIPLINE" rule 5, "PROACTIVE RELIEF" —
not in the repo). **COMMITTED as a worked exemplar only**: the actual
`91-HANDOVER.md` artifact at commit `edc3d52` and
`90-PAUSE-HANDOFF.md` demonstrate the pattern in practice (see Theme 10), but
no general prose rule ("relieve yourself past ~50% context at wave
boundaries") exists anywhere in committed `CLAUDE.md`/`PRACTICES.md`.

---

## Theme 4 — Understand underlying intention over faithful plan execution; executive pivots

**Normative statement:** The coordinator's job is to understand the
project's underlying intention and make executive, autonomous decisions and
tangential pivots toward that intention — not to faithfully execute a
pre-written plan when the plan no longer serves the goal. This explicitly
authorizes resequencing the roadmap itself.

**Verbatim quotes:**

> "As the top level coordinator... I want you to understand the underlying
> intention of this project and make executive autonomous decisions and
> tangential pivots that ultimately puts this project on the global map by
> the time you reach 50% context by being extremely intelligent about how you
> delegate to subagents and how you prompt them to take responsibility." —
> directive #8

> "I would like all future sessions to operate like this one. All the
> directives, guidance on fable, opus, sonnet, haiku, achieving end state by
> understanding the underlying intention, rather than faithfully executing
> plans, identifying tangential enabler/tooling/maintenance work, etc." —
> directive #16

**Session evidence:** This directly produced **OD-4 item 3** ("EXECUTIVE
RESEQUENCE"), committed in
`89-OWNER-DECISIONS.md`: *"v0.13.2 cross-link (P98-P107) moves AFTER a new
launch-readiness milestone (post-P97 tag): asciinema hero demo, CI-verified
headline numbers... install-path excellence, positioning/Show-HN kit —
pulled forward from v0.10.0-post-pivot v1.0 vision. Goal: global-map
adoption."* `STATE.md` echoes this: workstream B is explicitly
"RESEQUENCED per OD-4 item 3 (2026-07-04)". This is the clearest example in
the whole session of a coordinator *not* following the existing roadmap
literally, but re-deriving the sequence from the owner's stated intention
("puts this project on the global map").

**Current encoding status:** **COMMITTED** at `89-OWNER-DECISIONS.md` § OD-4
and `STATE.md` (both concrete resequencing instances). The *general
principle* — "understand intention over faithful execution" as a standing
instruction for all future sessions — is **NOT ENCODED** in `CLAUDE.md`
prose anywhere; only the one resulting resequencing decision is committed,
not the underlying doctrine that produced it. Directive #16 itself is the
meta-request to fix exactly this gap (see Theme 14).

---

## Theme 5 — Tangential enabler/tooling/maintenance work is first-class; tangent-scoping discipline

**Normative statement:** Investment in tooling, guardrails, and hooks is
itself a deliverable, not a distraction from feature work. When such
tangential work balloons (as the entire quality-gates framework did), the
coordinator must explicitly account for it and re-scope subagent briefs
around it — with a graduated response: <1h fixes land inline, larger items
go to intake, and anything ballooning past that must be surfaced as an
explicit scope decision to the owner, never silently absorbed.

**Verbatim quotes:**

> "The name of the game is investing in tooling, guardrails, hooks, etc.
> E.g. a hook to ensure mermaid in md files render correctly, or tests that
> assert if documentation is correct by checking hash drifts, or having
> instructions local to a file." — directive #1

> "The whole quality infra was a tangent of this project. Please account for
> unexpected tangets like that and scope work appropriately for your fable
> subagents." — directive #12

**Session evidence:** Directive #1 (the session's opening message) sets
tooling/guardrails as co-equal with features from turn one. Directive #12
retroactively *names* the entire quality-gates framework (`quality/`
directory, 9 dimensions, catalog schema) as an unplanned tangent that had
already consumed a large fraction of the session — explicitly instructing
that this *class* of emergent enabling work must be planned for going
forward, not treated as a one-off surprise. The dispatcher's known-directive
(e) adds a specific escalation phrase — "SCOPE DECISION NEEDED" — for work
that balloons past intake-sized; **this exact phrase does not appear
verbatim anywhere in the mined transcript or the accumulator** (grep across
the repo returns zero hits). It is a reasonable synthesis of directive #12 +
the accumulator's `<1h`-inline / else-intake rule (Theme 7, ownership
charter point 3) but is not itself an owner quote — flagged here so it is
not mistaken for verbatim owner language.

**Current encoding status:** **PARTIALLY encoded.** The `<1h`-inline /
else-file split is **COMMITTED** as OP-8's "eager-resolution preference" in
`.planning/PRACTICES.md` (and summarized in `CLAUDE.md` OP-8). The specific
"tangent-within-a-tangent must be accounted for in subagent scoping" lesson
from directive #12, and the "SCOPE DECISION NEEDED" escalation step for
ballooning tangents, are **NOT ENCODED** anywhere as a general rule — only
the one concrete instance (the quality-infra buildout itself) exists,
retroactively, as journal entries in `quality/SURPRISES.md`.

---

## Theme 6 — Quality upkeep as a standing parallel operation

**Normative statement:** Quality maintenance runs as a continuous, parallel
operation alongside feature phases — read-only audit fleets during phases,
a dedicated debt-drain window between every phase close and the next
dispatch, and iterative fresh-eyes re-audits until convergence (catching
progressively less-obvious issues each round, not stopping after one pass).
A phase must not dispatch while BLOCKER-class quality-ledger rows sit
undrained. Cross-cutting unification/simplification judgment calls
(collapsing near-duplicate tooling, accepting trivial capability loss for
major complexity reduction) are themselves fable-tier delegation-worthy
work.

**Verbatim quotes:**

> "Good work! The quality upkeep can be very complex! It can involve
> connecting seemingly distant functionality in CI/packages/quality
> gates/etc that can be unified, making critical judgement calls that can
> significantly reduce complexity with trivial loss in capability - worthy
> of a fable subagent and need multiple iterations to catch less and less
> obvious issues before proceeding with building out more features." —
> directive #8

> "Btw great progress and work so far! But again you need to leverage
> subagents more, the codebase needs to exhaustively be maintained for
> perfect quality... Leave no stone unturned, don't race to complete
> features while letting technical debt and chores pile up." — directive #7

**Session evidence:** This produced the accumulator's "OPERATING CADENCE"
doctrine (two standing operations, A: phase chain, B: quality upkeep,
interleaved) and, concretely, the 8 **D-CONV** unification decisions in
`quality/SURPRISES.md` (2026-07-04): D-CONV-1 (pre-pr cadence real CI
wiring), D-CONV-2 (verdict.py 3-state honest exit contract), D-CONV-3
(scripts/ collapse from ~33 to ~13 files + inverse-registry orphan-scan
gate), D-CONV-4 (cred-hygiene two-layer scanning), D-CONV-6
(test-pre-push.sh dirty-tree guard), D-CONV-7 (CLAUDE.md compaction via
progressive disclosure — the direct precedent for *this* extraction task),
D-CONV-8 (journal revival, since `quality/SURPRISES.md` had gone dead
2026-04-29 through 2026-07-04 despite `PROTOCOL.md` calling it required
reading). OD-4 item 2 committed the "features WAIT for convergence" rule
verbatim.

**Current encoding status:** **PARTIALLY encoded.** The 8 concrete D-CONV
decisions are **COMMITTED** in `quality/SURPRISES.md`, and OD-4 item 2 is
**COMMITTED** in `89-OWNER-DECISIONS.md`. The accumulator's explicit
"OPERATING CADENCE A/B" framing (phase-chain vs. debt-drain-window,
interleaved, with the "no dispatch over undrained BLOCKERs" rule stated as a
standing operational law) is **HIDDEN STATE only** — not committed anywhere
in the repo as a general standing-operation description.

---

## Theme 7 — Ownership charter: noticing as a deliverable, verify against reality, polish-for-adoption

**Normative statement:** Every subagent that touches a real surface owns it
beyond its acceptance criteria. Every report must include what was noticed
nearby (lying docs, tests that don't assert what their names promise,
teaching-free error messages, dead code, missing edge cases) — an empty
noticing section from code-touching work is itself a red flag. Claims
without a reality-check artifact (run it, render it, hit the backend) are
not done. The north star is polish for adoption: would a skeptical
first-time dev come away impressed?

**Verbatim quotes (charter, embedded verbatim in every dispatch per the
accumulator):**

> "1. You own what you touch. Acceptance criteria are the floor, not the
> ceiling. Done = you'd defend this surface in review as excellent work, not
> "plan executed."
> 2. Noticing is a deliverable. Every report includes what you noticed near
> your work: doc claims that lie, tests that don't assert what their names
> promise, error messages that don't teach recovery, dead code, stale
> comments, missing edge cases. An empty noticing section from a phase
> touching real code is itself a red flag (mirrors the verifier honesty
> check).
> 3. Eager-fix or file — never silently skip. <1h + no new dependency → fix
> in place. Else → SURPRISES-INTAKE / GOOD-TO-HAVES with severity + sketch.
> 4. Verify against reality. Run the thing, render the page, hit the
> backend. A claim without an artifact is not done.
> 5. North star: polish for adoption — would a skeptical dev hitting this
> surface for the first time come away impressed? (Owner mandate OD-3,
> 2026-07-03.)"
> — accumulator's "OWNERSHIP CHARTER" block, attributed to owner mandate
> OD-3, 2026-07-03

> "There are too many things that only someone close to the ground level
> doing actual implementation will notice. I need you to take full ownership
> of this codebase." — directive #3

> "Also the README CI badge is red - is that expected?" — directive #6
> (owner's own verify-against-reality behavior, modeling point 4)

> "Did you have a question for me? I saw a dialog" — directive #20 (owner
> watching the UI directly during autonomous runs — same reality-check
> instinct, from the owner's side)

**Session evidence:** Directive #3 is the origin story: it follows a
subagent's real-backend fixture report surfacing ground-truth details (stale
doc-comments, fragile-by-design assertions) invisible from a plan alone —
proving that "full ownership" includes implicit quality bars never
enumerated in any plan. Directives #6 and #20 show the owner *personally*
exercising the same discipline (watching CI badges, watching for stalled
permission dialogs) that the charter demands of subagents — the standard is
symmetric, not just imposed downward.

**Current encoding status:** **COMMITTED** — the 5-point charter is folded
into `CLAUDE.md` § "Subagent delegation rules" → "**Ownership charter for
dispatched subagents**" (paraphrased close to verbatim, all 5 numbered
points present, explicitly dated "Owner mandate OD-3, 2026-07-03"). This is
one of the best-institutionalized directives in the whole inventory.

---

