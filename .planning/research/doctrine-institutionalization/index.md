# Doctrine Institutionalization — evidence home (landed 2026-07-04)

Owner mandate: "I would like all future sessions to operate like this one" — capture
the 2026-07-03/04 session arc's orchestration discipline exhaustively and encode it
into durable, enforceable forms. This directory is the durable Stage-A evidence home
that `.planning/ORCHESTRATION.md`'s provenance section points to (Theme 14): the
14-theme directive inventory, sharded to respect the 20,000-byte generic-md cap.

## Contents

| File | Content |
|---|---|
| `index.md` | This file — provenance, transcript-mining recipe, directive-inventory summary, five non-obvious owner strategies. |
| `themes-01-07.md` | Themes 1–7 verbatim: tiering, coordinators-route-not-work, context/relief, mission-over-plan, tangent-scoping, quality-as-standing-op, ownership charter. |
| `themes-08-14.md` | Themes 8–14 verbatim: build memory budget, durable state, pause/handover, orchestrator relay, external-mutation approval, fix-it-twice, institutionalize-via-hooks. |
| `coverage-check.md` | Full directive→theme→encoding-status coverage table (34 source items, none dropped) — kept as its own chapter because themes-08-14 + this table together exceed the 20k cap. |

## Method and provenance

Mined the main coordinator session transcript only (owner scope correction: "Only
focus on the largest one"): the session-7e2a4cf2 `.jsonl` (2.5 MB); plus the
session-local accumulator `~/.claude/jobs/7e2a4cf2/tmp/PENDING-INTAKE-AND-CHORES.md`
(hidden state — `.planning/ORCHESTRATION.md` is its durable replacement); committed
artifacts (`89-OWNER-DECISIONS.md` OD-1..4, `PRACTICES.md`, `SURPRISES.md` D-CONV,
handover exemplars `90-PAUSE-HANDOFF.md`/`91-HANDOVER.md`); live Claude Code hooks
docs; and the existing user-level GSD hook suite (`~/.claude/hooks/`, 11 wired hooks).

Transcript-mining recipe (hard-won, for future miners):
- Naive `type=="user"` filtering is wrong: 34 of 60 user-role turns were
  harness-injected task-notification relays, not owner prose.
- 4 genuine owner one-liners surfaced ONLY via `queued_command`/attachment records
  (messages typed while the agent was mid-turn; the daemon queues them).
- `~/.claude/jobs/<session-prefix>/state.json` + `timeline.jsonl` are richer arc
  evidence than transcripts alone: fan-out labels, pending question blocks, respawn
  flags, resume lineage. The jobs dir is the Claude Code daemon's per-session
  supervision record; its `tmp/` survives respawns and OS crashes but is
  machine-local hidden state. Rule minted: job `tmp/` may hold scratch, never protocol.

## Directive inventory summary

34 items: 20 curated transcript directives + 12 dispatcher-known directives validated
against sources + 2 late owner messages (CLAUDE.md-alone-insufficiency; persist
reports durably). 14 themes — full text in `themes-01-07.md` / `themes-08-14.md`;
per-item disposition in `coverage-check.md`. Honesty flags preserved: "SCOPE DECISION
NEEDED" is synthesis, not owner quote; the accumulator's COORDINATOR CONTEXT
DISCIPLINE and OPERATING CADENCE blocks are orchestrator operationalizations of owner
turns, not verbatim owner sentences.

At extraction time: hidden-state-only = coordinator context discipline (5 rules),
proactive-relief/50% rule, operating cadence A/B, classifier-denial approval pattern.
Encoded nowhere = durable-state-over-chat rule, mis-routed-reply relay rule. Already
committed = ownership charter (CLAUDE.md), OD-1..4, OP-8/9, build memory budget. Stage
B (this landing) closes most of these gaps via `.planning/ORCHESTRATION.md`, the three
scoped `CLAUDE.md` files, and the root-CLAUDE.md pointer block — see each file's own
commit history for what actually shipped.

## Five most non-obvious owner strategies

1. End-state-by-50%-context as an up-front planning constraint with continuous
   owner-side telemetry (tracked from 12% on turn one). Plans must complete inside
   half the budget; the other half is margin, not workspace.
2. Outside-in context-rot diagnosis + pre-notified rotation: a stalling coordinator
   was diagnosed from behavioral signals alone, rotated at the next logical point,
   and — the subtle half — told in advance so it deliberately persisted handover
   state (P91 -> 91-HANDOVER.md @ edc3d52). Handover quality must be solicited.
3. Casual asides are rule seeds: "Be careful about parallel agents issuing large
   rust related commands" (one line, after a real OOM crash) became CLAUDE.md's
   numbered build-memory-budget hard rule. Fix-it-twice applies to owner remarks,
   not just caught defects.
4. JIT instruction injection beats static prose (owner, 2026-07-04): hooks are an
   instruction channel, not just gates — deliver the right reminder programmatically
   at the moment it is actionable (at Agent dispatch, at cargo invocation, at Stop).
   Owner: such instructions are "more likely to be complied with than the CLAUDE.md
   and also reduces the burden on CLAUDE.md." This produced the 3-role hook taxonomy
   (hard gate / state persistence / JIT doctrine injection).
5. The owner watches external reality signals — red CI badge, stuck permission
   dialogs — so verify-against-reality runs from the owner's side too; autonomous
   designs must surface (never silently sit on) blocking dialogs.

Honorable mention: tangent-within-tangent budgeting — the whole quality-gates
framework is retrospectively a tangent; coordinator briefs must budget for emergent
tangents of that magnitude.

## Where the raw Stage-A working set lives

The full unsharded `DIRECTIVES.md` (43.7k, pre-split), `ENCODING-PLAN.md` (the
per-artifact draft plan this landing executed), `LANDING-CHECKLIST.md` (the ordered
commit sequence), and `extracts/` (raw transcript-mining evidence: per-session
user-message extracts, the curated directives extract, and the repo-artifact /
hooks-API / existing-hooks digests) are **not committed to this directory**. They
live in the session job mirror at
`/tmp/claude-1000/-home-reuben-workspace-reposix/DOCTRINE/` and are deliberately kept
out of version control: `DIRECTIVES.md` and `ENCODING-PLAN.md` are both over the
generic-md size cap in their unsharded form, `extracts/` holds raw transcript
excerpts (privacy-sensitive owner text not intended for durable publication), and the
project's scratch-never-protocol rule keeps working-set scratch out of committed
history — only the distilled, sharded artifacts in this directory are durable. A
lander revisiting this decision should re-derive from the live session transcript
rather than assuming the `/tmp` mirror is still present; it is not guaranteed to
survive a crash or cleanup.

## Landing status

The 12 artifacts this research designed (`.planning/ORCHESTRATION.md`; the three
scoped `CLAUDE.md` files; five `.claude/agents/*.md`; six `.claude/hooks/*.sh` +
`settings.json` block; the `coordinator-dispatch` skill) are landed per
`LANDING-CHECKLIST.md`'s ordered, gate-aware commit sequence — see `git log` for the
actual commit SHAs and dates rather than assuming this file's original draft
sequencing. This directory is the historical evidence trail, not the live doctrine
surface: for current orchestration doctrine, read `.planning/ORCHESTRATION.md` and
its pointer in root `CLAUDE.md`.

## Stage-B probe outcomes (2026-07-04)

Empirical results from probing the artifacts this landing shipped, gathered
during the same session (full detail + session-metrics context:
`.planning/SESSION-HANDOVER.md` §3):

- **(a) Model resolution — Q1 RESOLVED.** Agent-tool model resolves as
  `claude-fable-5`; `phase-coordinator.md` shipped `model: fable` with an
  explicit `model: inherit` fallback note.
  - **CORRECTION (2026-07-05):** the def had NOT registered — but the cause
    was NOT the model value. `phase-coordinator.md`'s `description` frontmatter
    held a bare `: ` (inside a literal `` `model: opus` ``), which broke YAML
    parsing; the harness then silently DROPPED the whole def from the registry
    (no error — the type just never appeared). Missed here because probe (b)
    exercised `reader-digester`, whose frontmatter parses; this file's never
    did. Fixed by making `description` a block scalar (`>-`); `model` also
    moved `fable`→`inherit` per §11, but that was ORTHOGONAL to the parse bug.
    An initial fix attempt misdiagnosed the cause as the model value and did
    NOT fix registration — falsified only by a post-restart dispatch smoke test
    (defs are scanned at session start, so the miss survived until restart).
    Guardrail: `scripts/check-agent-frontmatter.py`.
- **(b) New `.claude/agents/` defs dispatchable mid-session, no restart** —
  a `reader-digester` dispatch succeeded immediately after the defining
  commit landed.
- **(c) CLAUDE.md injection scope — Q5 RESOLVED.** Root `CLAUDE.md` reaches
  custom agents at spawn; scoped `CLAUDE.md` (`crates/`, `.planning/`,
  `quality/`) does NOT inject at spawn but DOES inject via system-reminder
  the moment the agent `Read`s a file under that directory — independently
  re-confirmed by the handover writer.
- **(d) Stop hook — Q2 DECIDED: ADVISORY** (exit 0 + `systemMessage`) per
  owner decision; `cargo-mutex.sh` stays blocking (exit 2, machine-global
  `pgrep`).
- **Q6 / hook live-wiring — PARTIALLY verified.** `PreToolUse` cargo-mutex
  firing on real Bash calls was observed in-session. `SessionStart` brief and
  `PreCompact` persistence reach are UNVERIFIED pending a fresh session start
  (neither event fires mid-session by construction) — confirm at next boot.
- **Q7 DONE** — `orphan-scripts/claude-hooks` catalog row minted, documenting
  the hooks surface as audited rather than orphaned.
