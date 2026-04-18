---
doc_type: gsd-feedback-draft
project: reposix
captured_during: Phase 30 planning session, 2026-04-18
status: awaiting-user-review
target_upstream: https://github.com/gsd-build/get-shit-done
---

# GSD Feedback Draft — Phase 30 planning session

Issues and enhancements observed while planning Phase 30 (reposix v0.9.0 Docs & Narrative milestone). Each item is formatted as a near-ready issue body; the **Advise on** line flags what the user should review before filing.

Install context: GSD installed via `npx get-shit-done-cc@latest`, lives at `~/.claude/get-shit-done/` (not a git repo, overwritten on `/gsd-update`). Upstream: `gsd-build/get-shit-done`. All line numbers below refer to the installed copy at the time of capture.

---

## 1. `gsd-tools state planned-phase` rebuilds STATE frontmatter from stale body text and uncleaned phase dirs

**Type:** bug
**Severity:** medium

### What happened

Running `gsd-tools state planned-phase --phase 30 --plans 9` at the end of `/gsd-plan-phase 30` corrupted `.planning/STATE.md`:

- `stopped_at` was overwritten with the stale v0.8.0-era message `"Phase 29 complete — milestone v0.8.0 complete, all phases done, UAT 9/9 passed"`.
- Progress counters were wrong: `total_phases: 5, completed_phases: 3, total_plans: 18, completed_plans: 10`. The correct shape for v0.9.0 at that moment was `1 / 0 / 9 / 0`.

Root cause (from reading `~/.claude/get-shit-done/bin/lib/state.cjs`):

1. `cmdStatePlannedPhase` (line 1183) writes the narrowed body-field updates, then calls `writeStateMd` → `syncStateFrontmatter` → `buildStateFrontmatter`.
2. `buildStateFrontmatter` (line 714) pulls `stoppedAt` via `stateExtractField(bodyContent, 'Stopped At')` / `'Stopped at'` (line 723). In this project's STATE.md, the `## Session Continuity` section contains a historical line `Stopped at: Phase 29 complete — ...` (line 211 of the STATE body). `stateExtractField`'s regex `^Stopped at:\s*(.*)` is anchored to line-start and matches the first occurrence in the document — which is the archival narrative line, not a fresh field. **So `stopped_at` in the frontmatter gets clobbered with ancient narrative text every time the frontmatter is rebuilt.**
3. `buildStateFrontmatter` also rescans `.planning/phases/` on disk (line 744), filtered by `getMilestonePhaseFilter` in `core.cjs` (line 1452). That filter parses ROADMAP.md's current-milestone section via `extractCurrentMilestone` (core.cjs line 1145), but that function only cuts at the next `## `-level milestone heading matching `v\d+\.\d+` / ✅ / 📋 / 🚧. **The `<details>` blocks wrapping shipped milestones (v0.1.0–v0.7.0, v0.8.0) still contain `### Phase 27:`, `### Phase 28:`, `### Phase 29:` headings that are inside the v0.9.0 slice, so the filter treats those phases as current-milestone.** Combined with the pre-cleanup phase dirs still sitting at `.planning/phases/27-...`, `28-...`, `29-...`, `30-...`, plus Phase 999.1 from `## Backlog` (also inside the slice because `## Backlog` doesn't match the cutter pattern), the scan produced `5 phases / 3 completed / 18 plans / ~10 summaries` — exactly matching the corruption.

Two compounding bugs: (a) frontmatter field extraction from an un-delimited body picks up archival prose; (b) the milestone-scoped phase filter treats `<details>`-wrapped shipped phases as part of the in-progress milestone.

### Evidence

- Project repo commit `09815e5` ("fix(state): restore correct v0.9.0 values after gsd-tools state planned-phase regression").
- STATE.md line 211 is `Stopped at: Phase 29 complete — milestone v0.8.0 complete, all phases done, UAT 9/9 passed` inside `## Session Continuity`.
- ROADMAP.md contains `<details>` blocks at lines 48–72 and 74–105 with shipped-milestone phase headings.

### Expected

`state planned-phase` should update *only* the specific fields it advertises (Status, Last Activity, plan counts) without silently overwriting unrelated frontmatter fields from noisy body text, and the milestone phase filter should ignore phases archived inside `<details>` blocks.

### Actual

Frontmatter `stopped_at`, `progress.total_phases`, `progress.completed_phases`, `progress.total_plans`, `progress.completed_plans`, `progress.percent` were all rewritten from the wrong sources.

### Suggested fix

Two independent cuts:

1. **Strip `<details>…</details>` blocks in `extractCurrentMilestone` before returning the current-milestone slice** (core.cjs line 1205 area). The function already strips them from the preamble (line 1209) — extend that to the currentSection. This matches the semantic intent: `<details>` in ROADMAP means "collapsed because shipped."
2. **Make `stateExtractField` honor a delimited "frontmatter-candidate" region** rather than matching anywhere in the body. Options: (a) explicitly scan only the `## Current Position` or a new `## State Fields` subsection, not the whole body; (b) require a fenced-block / exact-heading context; (c) track which fields the tool "owns" and stop re-deriving them from body prose — if `cmdStatePlannedPhase` didn't touch `Stopped at`, don't rewrite `stopped_at` in the frontmatter.

Option 2(c) is probably the minimal-blast-radius change: frontmatter fields should only be derived from body content for fields the current command is explicitly syncing, and preserved-as-is otherwise.

### Advise on

Whether to file (1) and (2) as separate issues or one combined bug. The filter bug (1) is arguably the more general hazard — any tool using `getMilestonePhaseFilter` is exposed (uat.cjs already uses it). The field-extraction bug (2) only bites projects that have old "Stopped at: ..." prose in their STATE.md body, which is a common pattern but not universal.

---

## 2. MILESTONES.md silently drifts from archived ROADMAP snapshots across milestone closes

**Type:** bug
**Severity:** medium

### What happened

In reposix, MILESTONES.md was missing v0.6.0 and v0.7.0 entries. Both milestones were completed and their phases archived (`.planning/milestones/v0.6.0-ROADMAP.md`, `.planning/milestones/v0.7.0-ROADMAP.md` exist), but MILESTONES.md just wasn't appended to during those closes. The canonical data was sitting in the archived ROADMAP snapshots the whole time.

### Evidence

- Project repo commit `9b02a42` ("docs: backfill MILESTONES.md with v0.6.0 and v0.7.0 entries") added 35 lines by reading from the archived ROADMAP snapshots.

### Expected

`gsd-complete-milestone` either always appends to MILESTONES.md (the current workflow documentation at `complete-milestone.md` line 221 claims this is automatic via `gsd-tools milestone complete`), or running it on a repo where prior milestones drifted surfaces the gap and offers to backfill.

### Actual

Whatever caused the drift (older GSD version not auto-appending, or a bypass during the v0.6.0 / v0.7.0 closes), the current tooling offers no detection — `gsd-health` and similar commands don't flag "archive exists but no MILESTONES.md entry."

### Suggested fix

Add a `gsd-health` check (or `gsd-tools state validate` extension): for each `.planning/milestones/v[X.Y]-ROADMAP.md` snapshot, confirm a corresponding `## v[X.Y]` heading exists in MILESTONES.md. If missing, print a diagnostic and offer `--backfill` to synthesize entries from the archive. This is pure derivation from committed artifacts, no judgment calls.

### Advise on

Framing: is this a "bug" (the invariant should never hold) or a "missing check" (acknowledge drift can happen, provide a healer)? I've leaned on the latter in the suggested fix. Also, user may want to clarify whether earlier GSD versions actually emitted MILESTONES.md entries or not — the current complete-milestone.md says it's automatic, so if reposix was on older GSD during v0.6.0 / v0.7.0 closes, the framing is "historical migrator" not "live bug."

---

## 3. `/gsd-complete-milestone` should default to archiving phase dirs, not prompt for it

**Type:** enhancement
**Severity:** low

### What happened

After v0.8.0 shipped, the Phase 30 planning session opened with 25+ stale phase directories (`01-...` through `29-...`) still at the top level of `.planning/phases/`. The user had to run `/gsd-cleanup` as a separate step. This directly contributed to bug #1 (the stale phase dirs are what let the phase-filter scan produce an inflated plan count).

Inspection of `complete-milestone.md` line 430-444 shows the prompt `"Archive phase directories to milestones/?"` with two options: "Yes" (move them) or "Skip" (leave in place, run `/gsd-cleanup` later). The existence of a separate `/gsd-cleanup` command makes "Skip" feel like a reasonable default to a fatigued human operator at the end of a milestone close — but it leaves the workspace in a state that actively poisons later commands.

### Evidence

- Project repo commit `96b3a2b` ("chore: archive phase directories from completed milestones v0.1.0-v0.8.0") — archived 25 phase dirs after they'd sat at top level through Phases 28, 29, and the v0.9.0 kickoff.
- Bug #1 in this doc — the stale dirs directly caused the STATE corruption.

### Expected

Milestone close leaves the workspace clean. Default to "archive," require a flag or explicit "Skip" answer to retain.

### Actual

The prompt is symmetric ("Yes" / "Skip" with no indicated default) and the consequence of "Skip" (future commands scanning a polluted `phases/` dir) is not surfaced.

### Suggested fix

Three options, most-to-least invasive:

1. Swap the default: phrase the prompt as "Archive phase directories? [Y/n]" and archive when the user hits enter. "Skip" becomes the explicit choice.
2. Keep the prompt symmetric but add a warning to the "Skip" branch: *"Note: stale phase dirs can confuse `gsd-tools state` rescans in the next milestone. Run `/gsd-cleanup` before starting the next milestone's phases."*
3. Auto-chain: run the archive step by default, skip the prompt entirely unless `--no-archive` is passed.

### Advise on

Whether to push for (3) auto-chain (strong opinion: the current design is a gotcha) or (1) default-yes (milder). User's call on how opinionated the tool should be.

---

## 4. `/gsd-plan-phase` should emit `NN-PLAN-OVERVIEW.md` as a standard reviewer artifact

**Type:** enhancement
**Severity:** medium

### What happened

After planning Phase 30 (9 plans, ~5000 lines of PLAN.md text across `30-01-PLAN.md` through `30-09-PLAN.md`, plus `CONTEXT.md`, `30-RESEARCH.md`, `30-VALIDATION.md`, `30-PATTERNS.md`), there was no single document a human or code-review subagent could read to sanity-check the shape of the phase. Every reviewer has to either:

(a) Read 9 individual PLAN.md files (cost: context-window blow-out, time sink, reviewer becomes a bottleneck).
(b) Skim PLAN.md frontmatter and guess at the wave topology.

Neither is acceptable for a 9-plan phase. The orchestrator ended up hand-authoring `30-PLAN-OVERVIEW.md` to unblock review — 80 lines containing: scope one-liner, ASCII wave diagram showing the 5-wave dependency chain, per-plan summary table with objective / file count / requirements, cross-cutting constraints (P1/P2 framing, simulator-first, wave-4-is-a-real-gate), explicit deferred/out-of-scope list.

Inspection of `~/.claude/get-shit-done/workflows/plan-phase.md` (1288 lines) confirms: no step produces such an overview. The workflow produces `*-PLAN.md` files, optional `CONTEXT.md` / `RESEARCH.md` / `PATTERNS.md`, but nothing that aggregates.

### Evidence

- Project repo commit `919d3a3` ("docs(30): add PLAN-OVERVIEW for reviewer sanity-check") — 80-line hand-authored overview.
- `grep -i overview /home/reuben/.claude/get-shit-done/workflows/plan-phase.md` → zero matches.

### Expected

For a phase with >3 plans, `/gsd-plan-phase` produces a `NN-PLAN-OVERVIEW.md` alongside individual PLAN.md files. This is a reviewer-facing artifact, not an execution artifact — executors still read their specific PLAN.md.

### Actual

No overview artifact is emitted; reviewer must either read all plans or reconstruct the topology from PLAN frontmatter.

### Suggested fix

Extend `plan-phase.md` with a terminal step (after `gsd-plan-checker`) that emits `NN-PLAN-OVERVIEW.md` containing:

- Scope one-liner (pulled from RESEARCH or CONTEXT).
- Wave diagram (ASCII or Mermaid) derived from `depends_on` in PLAN frontmatter.
- Per-plan summary table: ID, wave, objective (one line from PLAN goal), file-count, requirements_addressed.
- Cross-cutting constraints — explicitly invite the planner to surface them rather than leaving them implicit in individual PLAN.md `must_haves` blocks.
- Deferred / out-of-scope bullet list.

The dependency graph is cheap to extract programmatically from PLAN frontmatter; the cross-cutting constraints and deferred list require one more LLM pass. A `plan-overview-composer` subagent would be the clean delegation.

Gate this on `plan_count >= 3` to avoid overhead for small phases.

### Advise on

Gating threshold (3? 5?) and whether this should be unconditional for all phases. Also whether the overview composer should be a new subagent role or a responsibility of `gsd-planner` / `gsd-plan-checker`. Could frame as "add a reviewer-sanity-check artifact to plan-phase" to make the value obvious.

---

## 5. No canonical-artifact boundary surfaced — agents misread historical one-offs as GSD conventions

**Type:** docs / dx
**Severity:** low-medium

### What happened

Planning Phase 30 included a pass for "what produces the final milestone verification artifact?" Claude observed `.planning/VERIFICATION-FINAL.md` at the repo root (a v0.1-era artifact produced in a session predating current GSD) and assumed it was a canonical GSD artifact that had been abandoned. This led to a proposed edit to Plan 30-09 adding a task to "re-establish the `VERIFICATION-FINAL.md` pattern for v0.9.0 milestone close" — claimed precedent was "the existing `.planning/VERIFICATION-FINAL.md`."

The user caught the mistake: `VERIFICATION-FINAL.md` is **not** a GSD convention. Templates directory (`~/.claude/get-shit-done/templates/`, 34 files) contains:

- `verification-report.md` — per-phase, produced inside execute-phase.
- `milestone-archive.md` — milestone close.
- No `verification-final.md` or anything under that name.

The rogue file was an ad-hoc artifact from an early session — Claude authored it, named it, and it stuck around until archival. The pattern was never in GSD's templates or workflows.

### Evidence

- Project repo commit `fcba1b6` ("docs(30-09): add goal-backward VERIFICATION-FINAL.md task for v0.9.0 milestone close") — the mistake.
- Project repo commit `0e6bca7` ("Revert 'docs(30-09): add goal-backward VERIFICATION-FINAL.md task for v0.9.0 milestone close'") — the rollback.
- Project repo commit `9412558` ("chore: archive v0.1-era VERIFICATION docs into v0.1.0-phases/") — moving the misleading file out of the root.
- `ls ~/.claude/get-shit-done/templates/` has no `verification-final*` entry.

### Expected

An agent opening a new `.planning/` tree can tell at a glance which `.md` files are GSD-canonical artifacts and which are historical ad-hoc session output. Canonical artifacts carry authority (their existence implies "this is where the pattern lives"); ad-hoc artifacts do not.

### Actual

No signal distinguishes them. A confident-looking file at `.planning/VERIFICATION-FINAL.md` reads as authoritative to a fresh agent. In this case the mistake was caught at review; in other cases a bad pattern could propagate.

### Suggested fix

Two complementary cuts:

1. **Ship `templates/README.md`** that enumerates canonical artifact names, which workflow produces each, and where each lives in a typical project tree. ~40 lines, one-time effort, instantly references-able.

2. **Add a lint (`gsd-health` extension) that flags `.planning/*.md` files which don't match any known canonical pattern.** Output: "`.planning/VERIFICATION-FINAL.md` does not match any GSD template (verification-report.md is per-phase; milestone-archive.md is per-milestone). If historical, move to `.planning/archive/` or a milestone-phases subdir." Non-blocking warning — the user gets to decide whether to keep, rename, or archive.

The lint is the higher-leverage cut because it works automatically on every health check, whereas a README has to be discovered and read.

### Advise on

Scope — which one to file (README is a small upstream PR, lint is a bigger feature), or whether to file a combined "agent-orientation improvements" issue. The user has opinions about how prescriptive GSD should be about filesystem conventions.

---

## Items investigated and **not** filing

- **`cmdStatePlannedPhase` writes a `Last Activity Description` field that doesn't exist in the STATE template.** The `stateReplaceField(content, 'Last Activity Description', ...)` call at state.cjs:1209 matches nothing in this project's STATE.md (the template uses `Last activity:` inside `## Current Position`, which the tool updates separately via `updateCurrentPositionFields`). No visible user-facing effect — the no-op is silent. Logged here for completeness; not worth a separate issue.

- **SESSION-5-RATIONALE.md / SESSION-7-BRIEF.md at `.planning/` root are not canonical GSD artifacts either.** Covered under issue #5 — a templates/README + lint would catch these too.

---

## Summary table for user review

| # | Title | Type | Severity | Blocks on |
|---|-------|------|----------|-----------|
| 1 | `state planned-phase` corrupts STATE.md (frontmatter rebuild + details-block filter) | bug | medium | Decide: one issue or two; framing as user-data-loss or edge-case |
| 2 | MILESTONES.md drifts silently from archived ROADMAP snapshots | bug | medium | Framing: "bug" vs "missing migrator" |
| 3 | `/gsd-complete-milestone` should default to archiving phase dirs | enhancement | low | Opinionation level: auto-chain vs default-yes |
| 4 | `/gsd-plan-phase` should emit NN-PLAN-OVERVIEW.md | enhancement | medium | Gating threshold; whether to compose via subagent |
| 5 | No canonical-artifact boundary — historical one-offs misread as conventions | docs / dx | low-medium | Scope: README, lint, or both |

Total items: 5 seeded + 0 additional filings (2 observations logged but not filing).
