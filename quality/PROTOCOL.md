# quality/PROTOCOL.md — Autonomous Execution Protocol (v0.12.0)

> **Audience.** The autonomous agent (or the orchestrator running `/gsd-autonomous`) executing v0.12.0 phases. This is the runtime contract. Read at the start of EVERY phase.

## The one principle that everything else follows from

**The executing agent's word is not the verdict.** Every claim of done gets graded by an unbiased subagent that has zero session context — only the catalog row + the artifact. If the verifier subagent reads the catalog and sees no artifact dated this session, the row is RED. The executing agent does NOT get to talk the verifier out of it.

This is the §0.8 verifier dispatch pattern from v0.11.2 generalized to every gate, every phase, every milestone close. Drift, half-done, and silent-downgrade failure modes all collapse to: "the catalog says NOT VERIFIED, the verifier is unbiased, the row is RED."

## Reading order for an agent picking up a phase

When you start a phase, read in this order:

1. `.planning/STATE.md` — the cursor; current position + last activity.
2. `.planning/ROADMAP.md` — your phase entry.
3. `.planning/REQUIREMENTS.md` — the requirement IDs assigned to your phase.
4. **THIS FILE** (`quality/PROTOCOL.md`) — the runtime contract.
5. `quality/SURPRISES.md` — pivots earlier waves hit; do not repeat.
6. `quality/reports/verdicts/p<N-1>/<latest>.md` — previous phase's verdict; precondition gate state.

Skipping any of these reintroduces the failure modes documented below.

## Two project-wide principles

These were named in the v0.12.0 design session and apply across every gate, dimension, and tool — not just docs-alignment. They generalize the runtime contract above into operational guidance for every verifier in `quality/gates/<dim>/`.

### Principle A — Subagents propose with citations; tools validate and mint

LLM agents (extractor, grader, verifier subagents) must NEVER emit machine-checkable state directly. They produce proposals with file:line citations and invoke deterministic tools that:

1. Validate that the cited file exists, the cited lines are valid, the cited symbol resolves.
2. Compute the canonical hash, verdict, or row-state from the cited primary source.
3. Refuse to mint state if validation fails.

This eliminates the hallucination surface. A subagent that hallucinates a hash, verdict, or test binding cannot persist that hallucination — the tool catches it at the validation step.

Cross-tool examples (already in the codebase or shipping in P64):

- Test verdicts come from `cargo test` exit codes, not LLM judgment (already true).
- Subjective rubric grades come from the rubric's score-to-verdict mapping, not LLM phrasing (P61 pattern).
- Catalog row bindings come from `reposix-quality doc-alignment bind`, which validates citations and computes hashes (P64).
- Hash refresh certificates come from `bind` after a fresh GREEN grader pass; the walker NEVER refreshes hashes (P64).

### Principle B — Tools fail loud, structured, agent-resolvable

Deterministic tools assert preconditions and emit machine-readable failure when preconditions don't hold. They never silently pick a default, never auto-resolve ambiguity, never log-warn-and-continue. Result interpretation and ambiguity resolution belong to the agent that called them — because that agent is the only actor in the loop with the context to decide.

Cross-tool examples:

- `merge-shards` writes `CONFLICTS.md` on ambiguity, exits non-zero, never partially writes the catalog. Agent reads `CONFLICTS.md`, edits shard JSON files, re-runs.
- `bind` refuses to write if the cited test fn doesn't resolve.
- `confirm-retire` env-guards: refuses to run if `$CLAUDE_AGENT_CONTEXT` is set OR if stdin is not a TTY. Only human shells can confirm.
- The hash walker reports `STALE_*` states with a stderr message that names the relevant slash command; never tries to refresh hashes itself.
- The runner forwards each gate's stderr verbatim so the slash-command hint reaches the user (`docs-alignment/walk` exits non-zero with `/reposix-quality-refresh <doc>` on drift).

## Failure modes the protocol protects against

| Failure mode | What it looks like | Mitigation |
|---|---|---|
| **Drift** | "agent reinterprets the goal mid-run" | Catalog-first rule — end-state is in git BEFORE code. Verifier reads catalog, not narrative. |
| **Half-done** | "agent declares done with broken edge cases" | Mandatory verifier-subagent grading per phase close. Mandatory plan-check before execute. |
| **Silent downgrade** | "agent shrinks scope to ship" | Waiver protocol — shrinkage requires a written waiver with TTL. No silent removals. |
| **Lost context** | "agent forgets why it pivoted" | `quality/SURPRISES.md` append-only journal. Required reading for next phase agent. |
| **Bloat** | "agent grows existing file instead of refactoring" | Anti-bloat comments at top of `end-state.py` and PROTOCOL.md. Phase plans cap LOC additions and require justification for new files. |
| **Cascading break** | "phase 3 breaks because phase 2's migration was sloppy" | Phase preconditions are gate states. P3 won't start until P2's catalog rows show GREEN. |
| **Ungrounded next agent** | "next phase agent doesn't know about this phase's new gates" | **Mandatory CLAUDE.md update per phase** (QG-07). Each phase that introduces a new file/convention/gate updates CLAUDE.md in the SAME PR. |

## Per-phase protocol (the agent reads this at start of EVERY phase)

### Step 1 — Read context

See "Reading order" above. Six files, every time:

```
1. .planning/STATE.md (current cursor)
2. .planning/ROADMAP.md ## v0.12.0 → find this phase's entry
3. .planning/REQUIREMENTS.md ## v0.12.0 → find requirements mapped to this phase
4. quality/PROTOCOL.md (this file)
5. quality/SURPRISES.md — what dead ends has earlier work hit?
6. quality/reports/verdicts/p<N-1>/<latest>.md — previous phase's verdict
```

### Step 2 — Verify precondition

If the previous phase's verdict is NOT GREEN: STOP. Do NOT begin this phase. Either:
- Fix the previous phase's RED rows (re-run its verifier; if RED, fix and re-verify), OR
- Escalate via SURPRISES.md + STATE.md update if the RED requires owner input

Phase preconditions are gate states, not just predecessor merged.

### Step 3 — Catalog-first commit

The phase's FIRST commit MUST write the catalog rows that define this phase's end-state contract. The rows go into `quality/catalogs/<dimension>.json` with `status: NOT-VERIFIED` and a `verifier:` block pointing at where the verifier WILL be (even if the file doesn't exist yet).

This commit answers: "what does GREEN look like for this phase, before I write any code?"

The verifier subagent at phase close will read these catalog rows and grade against them. If the catalog row's `expected.asserts` are vague or missing, the verifier can't grade — so the agent is forced to be specific UP FRONT.

### Step 4 — Implementation commits

Standard GSD execution. Use subagents per OP-2 (aggressive delegation). Each implementation commit:
- Cites the catalog row(s) it implements (e.g., `Implements catalog row: release/installer-curl-sh`)
- Updates the row's `verifier.script` if the path moves
- Does NOT mark `status: PASS` directly — only the runner does that after a successful verify

### Step 5 — CLAUDE.md update (MANDATORY)

Every phase that introduces a new file, convention, gate, or operational rule MUST update CLAUDE.md in the SAME PR. The verifier subagent grades this as a phase-close requirement (QG-07).

Anti-bloat rules:
- Append a paragraph + code reference; do NOT rewrite existing sections
- Delete sections that this phase supersedes
- Cross-reference `quality/PROTOCOL.md` for runtime details rather than duplicating

This rule exists because the v0.12.0 plan ORIGINALLY had CLAUDE.md updated only in P63 — owner caught that and flagged: "I think we need to update the CLAUDE.md in each phase not just at the end?" Yes. Per-phase ungrounded next agents = recurrence of the same misses.

### Step 6 — Run the verifier

Before claiming done:

```
quality/runners/run.py --cadence pre-commit  # for any pre-commit gates this phase added
quality/runners/run.py --cadence pre-push    # for any pre-push gates this phase added
quality/runners/run.py --cadence weekly      # for weekly gates
quality/runners/verdict.py --phase <N>       # rolls up to quality/reports/verdicts/p<N>/<ts>.md
```

Rows carry `cadences: list[str]`; a single gate may fire at multiple
cadences (a fast mechanical check tagged `["pre-commit", "pre-push", "pre-pr"]`
will be picked up by all three runner invocations).

Milestone-close ritual MUST also invoke
`python3 quality/runners/run.py --cadence pre-release-real-backend` and
require exit 0; absent ⇒ verdict graded RED (per RBF-FW-03 9th probe).

**Honest callout (added P91 91-06):** every `run.py --cadence ...` invocation
above mutates the catalog JSON in place as a side effect of running — there
is currently no `--dry-run` flag to preview a would-be verdict diff without
writing it. If you want to see what a cadence run would flip before
committing to the mutation, there is no supported way to do that today;
tracked as GOOD-TO-HAVES-16 (`.planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md`).

**OD-2 hard-RED skip-semantics (89-OWNER-DECISIONS.md, binding):** If the
`pre-release-real-backend` cadence cannot EXECUTE against the sanctioned
target at milestone-close, the milestone-close verdict is **RED**. Milestone
does NOT close. No owner-waiver. No `until_date`. No PASS-with-comment. No
skip-counts-as-pass. The 9th-probe entry in the milestone-close verdict
template carries NO waiver column (skip ⇒ RED). Mechanically enforced at
catalog-load time: any row tagged `pre-release-real-backend` that carries a
`waiver` block refuses to load (`quality/runners/_audit_field.py`,
SystemExit before any verifier runs). Two states must never be
blurred: `NOT-VERIFIED` remains the honest status for the P89–P95 SLOT
state (env set, substrate not yet built); creds/targets-missing-at-
milestone-close is a DIFFERENT state and is RED, full stop.

#### Milestone-close 9th probe (added P89 RBF-FW-03 89-06)

The milestone-close ritual runs against the TEMPLATE at
`quality/dispatch/milestone-close-verdict.md` (9-row probe table). Probe 9
is catalog row `agent-ux/milestone-close-vision-litmus-real-backend`
(`blast_radius: P0`, verifier `quality/gates/agent-ux/milestone-close-vision-litmus.sh`)
— it runs the vision litmus test against the sanctioned real backend
(TokenWorld for v0.13.0; see `docs/reference/testing-targets.md`). Until
P91–P95 ship their substrate, this row legitimately reads NOT-VERIFIED via
the two paths already documented above (env-gate short-circuit; exit-75
mapping) — see "Verifier exit-code conventions" below and the OD-2 block
immediately above for the full semantics; this subsection does not repeat
them. **P97's `tag-v0.13.0.sh` MUST wire this guard** (the script currently
ships `.disabled` per Path A/Option B; P97 RBF-G-04 re-enables it and adds
the probe-9 invocation before any tag is cut).

#### Absorption honesty spot-check dispatch (P90 RBF-FW-10 / F-K5, D90-08)

Every milestone's Slot-1 absorption phase (OP-8's "+2 phase practice",
`.planning/PRACTICES.md` § OP-8) MUST run the honesty spot-check per the
F-K5 meta-rule template at `quality/dispatch/absorption-honesty-spot-check.md`
— four binding clauses, verbatim in that file: (a) sample **every** phase
that closed without filing intake, (b) spot-check author **≠** the
milestone orchestrator, (c) rubric is "walk one critical example
end-to-end mentally — does it work?" (not "did the phase follow
procedure?"), (d) the verifier **content-hash-binds** the report, not
mere file existence. Row `agent-ux/absorption-honesty-template-present`
(verifier `quality/gates/agent-ux/absorption-honesty-template-present.sh`)
asserts the template itself exists, its sha256 matches the script's
`EXPECTED_SHA` constant, and all four clauses are grep-present — this
guards the template, not any given milestone's report. Editing the
template requires updating `EXPECTED_SHA` in the same commit (the
hash-binding contract).

#### Milestone-close adversarial pass (P90 RBF-FW-12 / D90-09)

A milestone cannot close GREEN on catalog-row status alone. A **fresh
subagent, distinct from the milestone orchestrator**, reads catalog row
**descriptions only** (`comment` / `expected.asserts` / `owner_hint` /
`claim_vs_assertion_audit` — never verifier source or implementation) per
`quality/dispatch/milestone-adversarial.md`, and grades whether each row's
assertion would actually falsify its description if the description were
false. It writes
`quality/reports/verifications/adversarial/<version>.json` (the D90-09
path — deliberately **not** the `milestone-adversarial/` path sketched in
`90-RESEARCH-runner.md` § RBF-FW-12; the catalog row comment for
`agent-ux/milestone-adversarial-pass` names the D90-09 path as
authoritative).

`quality/runners/verdict.py --milestone <version>` reads this artifact via
`milestone_adversarial_gate(repo_root, version)` (`verdict.py:159-173`):
blocks (forces `color = "red"`) if the artifact is absent, or if it
reports ≥1 entry in `rows_failed`. The gate is wired in `main()`
(`verdict.py:386-394`) strictly as a **darken-only** step — it composes
with `compute_color`/`compute_exit_code` by forcing red after they run; it
never lightens an already-red verdict back to green (D-CONV-2's 3-state
contract is unmodified). Row `agent-ux/milestone-adversarial-pass`
(verifier `quality/gates/agent-ux/milestone-adversarial-pass.sh`, a thin
wrapper over `python3 -m unittest quality.runners.test_verdict`) backs the
regression cases in `TestMilestoneAdversarialGate`
(`quality/runners/test_verdict.py`): artifact-absent blocks even when every
row is PASS, empty `rows_failed` does not block, ≥1 failed row blocks, and
darken-only never flips an already-red verdict to green.

#### New runner/validator semantics (P90 90-01/90-02, RBF-FW-06/07/08/12 + F-K4b)

Progressive disclosure: this is the reference index; full design rationale
lives in `.planning/phases/90-framework-fixes-honesty-rules/90-RESEARCH-runner.md`
(cited `R1` below) and the four **deviations from R1's original proposal**
ratified during implementation are recorded in
`.planning/phases/90-framework-fixes-honesty-rules/90-PAUSE-HANDOFF.md`
§ 1 (cited `PAUSE-HANDOFF` below) — read that file before trusting R1's
prose over the code for the skip-path and congruence-gating semantics.

- **Missing-verifier demote + `error` marker (RBF-FW-07a).** A catalog row
  whose `verifier.script` does not exist on disk flips **unconditionally**
  to `NOT-VERIFIED` (`run.py:252-270`, transient `_verifier_missing` flag
  stripped before persist) — no longer preserving a stale PASS (R1 § 1a).
  The artifact's `error: "verifier not found at ..."` marker is now
  **rendered** by `verdict.py`'s NOT-VERIFIED section (`verdict.py:258-265`)
  so a missing-script row is visually distinguishable from a merely-stale
  one, instead of looking identical (R1 noticing 2, closed in this task).
- **Env-gated skip is fail-closed, not preserve-status (RBF-FW-07b,
  AMENDED D90-04 — deviates from R1's original "preserve prior status"
  design; see PAUSE-HANDOFF item (1)).** A cred-less
  `pre-release-real-backend` run ALWAYS flips (and persists) the row to
  `NOT-VERIFIED` (`run.py:187-207`), including the P0 vision-litmus row —
  this closes the skip-as-pass channel OD-2 forbids. The prior real grade
  is not lost: when the prior status was PASS/FAIL/PARTIAL it is recorded
  once in the write-history fields **`last_real_grade`** /
  **`last_real_verified`** (`run.py:203-205`; schema:
  `quality/catalogs/README.md:41`), never overwritten by a subsequent
  skip. The skip artifact carries the machine marker **`skip_reason:
  "env-missing"`** plus human-readable **`skip_detail`**
  (`run.py:194-195`) — NOT a single combined field; both names are
  load-bearing for any code that reads the artifact (PAUSE-HANDOFF item
  (2)).
- **`minted_at` write-once audit-cutoff anchor (D90-03 / cross-AI H2).**
  `_audit_field.py:38-42,229-247`: when a row carries `minted_at` it is the
  SOLE anchor for the `claim_vs_assertion_audit` cutoff (a hand-edited
  `last_verified` can no longer move it); rows whose `last_verified` is
  on/after `P90_MINT_CUTOFF` (`_audit_field.py:42`,
  `2026-07-05T00:00:00Z`) but lack `minted_at` are rejected at load.
  Legacy rows (no `minted_at`) fall back to the pre-P90 `last_verified`
  heuristic — the docs-alignment dimension is exempted from this whole
  validation loop unchanged (`run.py:98-100`).
- **`coverage_kind` hard-new / RAISE-legacy split + `transport_claim`
  tri-state (RBF-FW-06 / D90-05).** `_audit_field.py:81-99,260-278`: rows
  minted ≥ P90 that are transport/perf-claim (via
  `is_transport_or_perf_row` — explicit `transport_claim: true/false`
  overrides the description-regex, which is gated to the
  `perf`/`agent-ux`/`security` dimensions) MUST declare
  `coverage_kind: real-backend` (or a compliant waiver) — hard `SystemExit`
  at load if absent. Legacy (pre-`minted_at`) transport/perf rows are
  RAISE-only (`quality/reports/raise-list-p90.md`), never blocked — the
  migration is P95 RBF-D-06's chartered work, not enforced retroactively.
- **`kind: shell-subprocess` transcript enforcement at grade time
  (RBF-FW-08 / M6).** Two halves, co-located in `_audit_field.py` per R1
  noticing: `_has_transcript_contract` (load-time: a transcript CONTRACT
  must be declared) and `transcript_evidence_ok` (`_audit_field.py:100-124`,
  runtime: the transcript file must actually exist and carry an `argv:`
  line). `apply_pass_gates` (`_audit_field.py:182-220`, called from
  `run.py:350`) demotes a would-be PASS to FAIL when the transcript
  evidence is missing — a shell-subprocess row can no longer PASS on exit
  0 alone with no transcript.
- **F-K4b per-expected-assert congruence, gated on `minted_at`
  (ROADMAP SC2 / D90-12 item 1 — deviates from a naive "both lists
  non-empty" design; see PAUSE-HANDOFF item (1)).** `asserts_congruent`
  (`_audit_field.py:151-181`) requires each `expected.asserts` entry to
  normalized-token-match at least one `asserts_passed` entry; an unmatched
  expected assert blocks the PASS flip. Called from `apply_pass_gates`
  **only when `row.get("minted_at")` is truthy** (`_audit_field.py:209`) —
  new-regime rows only. This is deliberate, not an oversight: legacy prose
  asserts (e.g. `docs-repro/example-03`) would false-RED under any
  token-threshold check, so congruence is **armed-but-dormant** on legacy
  rows until they're re-minted. The p86 F6 fixture (9-item `expected` vs
  17-item `asserts_passed`, two uncovered) is the required regression case
  proving the gate actually fires when it should.

#### Latency budgets

Each cadence carries a hard time budget; rows whose verifier exceeds the
budget either move down-cadence or get split. Over-budget at the gate
that contributors hit interactively (pre-commit, pre-push) is
self-defeating: people learn to bypass.

| Cadence       | Budget   | Notes                                                                      |
|---------------|----------|----------------------------------------------------------------------------|
| pre-commit    | <2s      | every commit; over budget means contributors `--no-verify`                 |
| pre-push      | <60s     | per-phase per CLAUDE.md "Push cadence"; runner gate is part of phase-close |
| pre-pr        | <10min   | CI tier-1 (PR check) — wired as the `quality-pre-pr` job in ci.yml (D-CONV-1, 2026-07-04). Rows whose substance a dedicated ci.yml job already covers (fmt, clippy, cargo test --workspace, dark-factory sim arm) are intentionally NOT tagged pre-pr — see quality/SURPRISES.md "Quality Convergence" D-CONV-1. |
| pre-release   | <15min   | tag-time release gate                                                      |
| weekly        | n/a      | alerting cadence; not blocking                                             |
| post-release  | n/a      | alerting cadence; not blocking                                             |
| on-demand     | n/a      | manual / subagent invocation                                               |
| pre-release-real-backend | n/a (env-gated) | mandatory at milestone-close; default-skip in CI; first-run cache-warming heavy |

First-run-heavy: the initial `--cadence pre-release-real-backend` invocation
against TokenWorld / `reubenjohn/reposix` / JIRA `TEST` pays a cache-warming
cost (`list_records` walk + blob materialization). Run it in a dedicated
session window with no concurrent cargo work (CLAUDE.md "Build memory budget").

#### Verifier exit-code conventions

The runner maps verifier exit codes via
`quality/runners/_realbackend.py:map_exit_code_to_status`: `0` → PASS, `2` →
PARTIAL, `75` → NOT-VERIFIED, anything else → FAIL. Verifiers MAY exit 75
(sysexits.h `EX_TEMPFAIL` repurposed) to signal NOT-VERIFIED — the runner
preserves the honest status rather than mapping the non-zero exit to FAIL.
Used by the milestone-close-vision-litmus SLOT verifier (P89 RBF-FW-03) when
env IS set but the P91–P95 substrate has not landed. Exit-75 is NOT a
soft-skip channel: per OD-2 the creds-missing-at-milestone-close state must
never reach exit-75 — that state is hard RED at the verdict layer (see the
OD-2 block above).

Cadence-specific runner semantics (P61 SUBJ-03):
- **pre-release** is the cadence where freshness-TTL enforcement materially gates a release. STALE subagent-graded rows (kind=subagent-graded with expired freshness_ttl) flip to NOT-VERIFIED; `compute_exit_code` treats P0+P1 NOT-VERIFIED as RED. The pre-release workflow at `.github/workflows/quality-pre-release.yml` fails the release with a hint pointing the maintainer at the dispatcher (`bash .claude/skills/reposix-quality-review/dispatch.sh --all-stale --force`). Auto-dispatch from CI (would require Anthropic API auth on GH Actions runners) is a v0.12.1 carry-forward via MIGRATE-03.
- **weekly** STALE rows raise visibility but P2 rows do not block exit, per `compute_exit_code`'s P0+P1-only gating.

If any P0+P1 row is RED: do NOT claim done. Either fix or file a waiver (next step).

### Step 7 — Dispatch unbiased verifier subagent

```
Agent({
  description: "Phase N verifier dispatch",
  subagent_type: "general-purpose",
  prompt: <see "Verifier subagent prompt template" below — copy verbatim>
})
```

The subagent's verdict goes to `quality/reports/verdicts/p<N>/<ts>.md`. Phase does NOT close on RED.

### Step 8 — Update STATE.md + commit

Update STATE.md `## Current Position` to reflect phase complete. Append to `## Accumulated Context` ROADMAP-EVOLUTION section. Commit + push.

### Step 9 — If anything pivoted, append to SURPRISES.md

If this phase encountered an unexpected obstacle (whether resolved or worked-around), append one line to `quality/SURPRISES.md`:

```
2026-04-28 P57: discovered that quality/runners/run.py needs a tag-cache
because re-reading every catalog file per gate is O(N^2) — added
quality/runners/_cache.py to cache parsed catalogs per run.
```

The next phase's agent reads this. Dead ends documented = dead ends not repeated.

## Waiver protocol

Catalog rows support a `waiver` field:

```jsonc
"waiver": {
  "until":            "2026-05-15T00:00:00Z",   // RFC3339 expiration
  "reason":           "GH Actions windows runners are real money; deferred to v0.12.1",
  "dimension_owner":  "v0.12.0 P59 — see SURPRISES.md 2026-04-29",
  "tracked_in":       "v0.12.0 MIGRATE-03"      // optional ref to where this gets resurfaced
}
```

Rules:

- Waivers expire. Expired waivers flip the row to FAIL on next verify.
- `until` must be ≤ 90 days from `last_verified` (longer waivers require the dimension owner's explicit approval — for v0.12.0 that's the milestone owner).
- `reason` must be specific. "Not in scope" is not a reason. "GH Actions windows runners cost $0.08/min and we don't have budget approved" is.
- `tracked_in` should point at a v0.X.Y carry-forward requirement OR a GitHub issue.

Waivers are the principled escape hatch. Without them, an agent stuck on an inherited problem either silently descopes (bad) or blocks indefinitely (bad). With them, the descope is explicit, time-bounded, and re-surfaced when the waiver expires.

## SURPRISES.md format

Append-only. One line per surprise. Format:

```
YYYY-MM-DD P<N>: <what happened> — <one-line resolution>
```

Examples:

```
2026-04-28 P56: release.yml tag-glob fix broke the dist version derivation because
${{ github.ref }} now resolves to refs/tags/reposix-cli-v0.11.3 not refs/tags/v0.11.3 —
added a step to strip the reposix-cli- prefix.

2026-04-29 P59: ubuntu:24.04 doesn't have curl by default — added apt-get install curl
to the container preamble; also added a precondition check in the catalog row.
```

Required reading for next phase agent. The next agent does NOT repeat investigations of things already journaled here.

## Skill dispatch — when to spawn subagents

Per CLAUDE.md OP-2 (aggressive subagent delegation):

| Situation | Spawn what |
|---|---|
| Researching how to implement an approach | `gsd-phase-researcher` (foreground; wait for findings) |
| Writing the phase plan | `gsd-planner` (foreground) |
| Plan-check review of the plan | `gsd-plan-checker` (foreground; mandatory before execute) |
| Phase execution | `gsd-executor` (foreground; one at a time per CLAUDE.md "Build memory budget") |
| Code review after phase ships | `gsd-code-reviewer` (foreground) |
| Phase close verification | `general-purpose` with the prompt template below |
| Dispatching subjective rubrics | `reposix-quality-review` skill (after P61 ships) |
| Cold-reader pass on user-facing surfaces | `doc-clarity-review` skill |

**Coordinator-level safe rule:** at most ONE phase-executor subagent doing cargo work at a time (CLAUDE.md "Build memory budget" — VM has crashed twice from parallel cargo). Doc-only / planning-only subagents can still run in parallel with one cargo subagent.

## When stuck rules

If the agent is stuck (verifier RED for >1 hour with no clear path forward):

1. Append a SURPRISES.md entry: `<date> P<N>: stuck — <symptom>`
2. Dispatch a `gsd-phase-researcher` agent: "I'm stuck on <symptom>. Investigate and propose 2-3 paths forward."
3. If the researcher's paths all involve scope reduction: file a waiver with the milestone owner's authority (since they've granted autonomous-mode latitude).
4. If the waiver moves the work to v0.12.1: update MIGRATE-03 to reference the new carry-forward.
5. If even waivers don't resolve: pause via STATE.md cursor update + commit + STOP. Do NOT silently downgrade. The owner can resume on next session.

## Anti-bloat rules per surface

| File | Anti-bloat rule |
|---|---|
| `quality/PROTOCOL.md` | ≤500 lines (this file). New rules go here only when they apply to MORE than one dimension; dimension-specific rules go in `quality/gates/<dim>/README.md`. |
| `quality/SURPRISES.md` | ≤200 lines. When it crosses, archive oldest 50 to `quality/SURPRISES-archive-YYYY-QN.md` and start fresh. |
| `quality/runners/run.py` | ≤350 lines. Grow the dispatch table, not the runner core. |
| `quality/runners/verdict.py` | ≤400 lines. Grading rules live in catalog `expected.asserts`, not in verdict.py code paths. |
| `CLAUDE.md` | Already enforced via `scripts/banned-words-lint.sh`; v0.12.0 adds: each new section must cross-reference `quality/PROTOCOL.md` rather than duplicating runtime detail. |
| `.planning/REQUIREMENTS.md` (top-level) | Per QG-08: ONLY the active milestone + a "Previously validated" index. Historical sections live in `.planning/milestones/v0.X.0-phases/REQUIREMENTS.md`. STRUCT catalog row enforces this. |
| `.planning/ROADMAP.md` (top-level) | Same rule as REQUIREMENTS.md applies in spirit; per-milestone phase detail lives in `.planning/milestones/v0.X.0-phases/ROADMAP.md`. |

## Verifier subagent prompt template (for copy-paste in Step 7)

```
You have ZERO context from this session. You are the unbiased verifier for v0.12.0 Phase <N>.

Inputs:
1. .planning/REQUIREMENTS.md ## v0.12.0 — find requirements assigned to Phase <N>
2. quality/catalogs/*.json — find rows tagged phase=<N> or whose verifier ships in this phase
3. CLAUDE.md — confirm this phase's contributions are documented (per QG-07)
4. quality/SURPRISES.md — confirm any pivots are journaled

For each catalog row in scope:
1. Read row.verifier.script and row.expected.asserts
2. If row.kind == "container", run quality/gates/docs-repro/container-rehearse.sh <row.id>
3. Else, invoke row.verifier.script with row.verifier.args
4. Compare row.artifact contents against row.expected.asserts
5. Grade: PASS (all asserts met) | FAIL (any P0+P1 assert unmet) | PARTIAL (P2 unmet only) | NOT-VERIFIED (no artifact dated this session)

For CLAUDE.md (per QG-07):
1. git diff main...HEAD -- CLAUDE.md
2. Confirm this phase's new files/conventions/gates appear in the diff
3. If absent: phase is NOT done — block the GREEN verdict

For SURPRISES.md:
1. Read the file; confirm any pivot in the phase has a corresponding entry
2. If a pivot is mentioned in commits but absent from SURPRISES.md: ungrounded pivot — flag

Output a scored markdown table with file:line citations. Write to quality/reports/verdicts/p<N>/<ISO_TIMESTAMP>.md.
Refuse to grade GREEN unless every P0+P1 catalog row is PASS or WAIVED AND CLAUDE.md is updated.
```

**For rows with `kind: shell-subprocess`** (RBF-FW-02; see also § "Verifier exit-code conventions" above): dereference `artifact.transcript_path`, read the transcript file, and verify (a) the `argv:` line names a real binary (NOT a Python function or `assert_cmd` envelope), (b) `env_keys:` contains expected variable NAMES only (no `=value` pairs — security), (c) `exit_code:` matches the row's expected outcome, (d) `--- STDOUT ---` / `--- STDERR ---` blocks contain evidence consistent with the row's `claim_vs_assertion_audit` paragraph. Reject the row's PASS grade if the transcript is missing or shows a synthetic invocation. **Note:** the worked-example row `agent-ux/kind-shell-subprocess-worked-example` may legitimately invoke `bash --version` as a CI-portability fallback (per its `expected.asserts[0]`); transport-claim rows MUST invoke a real reposix binary or backend endpoint and the verifier subagent grades accordingly.

## Cross-references

This file (`quality/PROTOCOL.md`) is the runtime contract. The research bundle below is the design rationale — read those once at milestone planning, not every phase.

- `.planning/research/v0.12.0/vision-and-mental-model.md` — WHY this protocol exists
- `.planning/research/v0.12.0/naming-and-architecture.md` — `quality/` directory + catalog schema
- `.planning/research/v0.12.0/roadmap-and-rationale.md` — phase-specific application of this protocol
- `.planning/research/v0.12.0/decisions-log.md` — the owner conversation that produced these rules
- CLAUDE.md "Subagent delegation rules" — OP-2 detail
- CLAUDE.md "Build memory budget" — RAM guardrail (one cargo invocation at a time)
