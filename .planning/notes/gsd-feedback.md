---
doc_type: gsd-feedback-draft
captured: 2026-04-19
target: https://github.com/gsd-build/get-shit-done/issues
status: awaiting-user-review
---

# GSD Feedback — Draft Issues

Observed while running `/gsd-plan-phase` for a large phase (9 plans, 5 waves) on a project that had completed 8 prior milestones. Four issues below, ready to file after your review. Install context: `npx get-shit-done-cc@latest`.

> **Advise needed on each item** — see the _Advise on_ callouts before filing.

---

## Issue 1: `stopped_at` frontmatter gets silently overwritten with old session notes

**Type:** bug · **Severity:** medium

**Background:** `stopped_at` in STATE.md frontmatter is a one-line resume cursor — it records what was last happening so agents and humans know where to pick up. GSD reads it to show current status.

**What happened:** Running `gsd-tools state planned-phase` after `/gsd-plan-phase` reset `stopped_at` to a stale message from a previous milestone — completely replacing the correct current value. The symptom is STATE.md reporting the wrong phase/milestone as "in progress."

**Why it happens:** When `gsd-tools state` rebuilds frontmatter, it scans the entire STATE.md body for a line matching `Stopped at:` to repopulate `stopped_at`. If your STATE body contains old session notes (e.g. "Stopped at: Phase 5 complete…" in a `## Session Continuity` section), the tool picks up the historical note instead of the real value.

<details>
<summary>Root cause detail</summary>

`buildStateFrontmatter` calls `stateExtractField(bodyContent, 'Stopped at')` using a regex anchored to line-start (`^Stopped at:\s*(.*)`). It searches the entire document, so the first match wins — which is often a historical note in a continuity/archive section rather than the current state field.

The minimal fix: scope extraction to a known region (e.g. `## Current Position`) rather than the full body. A safer fix: `gsd-tools state` should only rewrite frontmatter fields it explicitly updated in this command invocation; unrelated fields should be left as-is.

</details>

**Expected:** `gsd-tools state planned-phase` updates the fields it advertises (status, plan counts) without touching unrelated frontmatter fields like `stopped_at`.

**Workaround until fixed:** Avoid writing `Stopped at:` in session-continuity prose in STATE.md. Use a different label (e.g. `Checkpoint:`) for archival notes.

> **Advise on:** File as one issue or split into "field extraction scopes entire body" (the root cause) and "planned-phase clobbers stopped_at" (the symptom)?

---

## Issue 2: Stale phase dirs after milestone close corrupt `gsd-tools state` phase counts

**Type:** bug + enhancement · **Severity:** medium

**What happened:** `/gsd-complete-milestone` prompts "Archive phase directories?" with a symmetric Yes/Skip choice. When the user picks Skip (reasonable — there's a separate `/gsd-cleanup` command), stale phase dirs remain in `.planning/phases/`. On the *next* milestone, `gsd-tools state` scans that directory to count total/completed phases for the current milestone — and counts the old dirs too, producing inflated and wrong progress numbers.

**Why it matters:** Wrong phase counts in STATE.md are confusing on their own. They also compound with Issue 1 — both bugs hit simultaneously when `state planned-phase` runs, making the corruption harder to diagnose.

<details>
<summary>Root cause detail</summary>

`buildStateFrontmatter` scans `.planning/phases/` filtered by `getMilestonePhaseFilter`. That filter parses the current-milestone section from ROADMAP.md via `extractCurrentMilestone`. If ROADMAP.md wraps shipped milestones in `<details>` blocks (a common pattern to keep the file readable), and `extractCurrentMilestone` doesn't strip those blocks before searching for phase headings, old phase headings inside `<details>` land in the "current milestone" slice. Combined with stale dirs still sitting in `phases/`, the scan over-counts.

The prompt in `complete-milestone.md` has a "Skip" branch but doesn't warn that skipping pollutes future state rescans.

</details>

**Expected:** Milestone close leaves `.planning/phases/` containing only phases for the new (upcoming) milestone. Skipping the archive step should surface a clear warning about the risk.

**Suggested fix options:**

1. Default the prompt to "archive" (Yes/n) instead of symmetric Yes/Skip.
2. Add a warning to the Skip branch: *"Stale phase dirs can cause incorrect phase counts in the next milestone. Run `/gsd-cleanup` before starting new phases."*
3. Auto-archive without prompting (opt-out via `--no-archive`).

**Workaround until fixed:** Run `/gsd-cleanup` immediately after every `/gsd-complete-milestone`, before starting the next milestone's phases.

> **Advise on:** How opinionated should the tool be? Option 3 (auto-archive) is the strongest opinion. Option 2 is the mildest change.

---

## Issue 3: MILESTONES.md can silently drift from archived milestone snapshots

**Type:** bug / missing check · **Severity:** medium

**What happened:** Two completed milestones were missing from MILESTONES.md. The milestones were fully closed (archived ROADMAP snapshots existed in `.planning/milestones/`), but MILESTONES.md was never appended to — likely because an older GSD version didn't auto-append, or the milestone-close step was bypassed. There's no detection: `gsd-health` and similar commands don't flag the mismatch.

**Expected:** For every `.planning/milestones/vX.Y-ROADMAP.md` snapshot that exists, a corresponding `## vX.Y` entry exists in MILESTONES.md — and this invariant is checked.

**Suggested fix:** Add a `gsd-health` check: scan `.planning/milestones/` for `vX.Y-ROADMAP.md` files, verify each has a heading in MILESTONES.md, and offer `--backfill` to synthesize missing entries from the archive. This is pure derivation from committed artifacts, no judgment calls.

> **Advise on:** Is this a "bug" (the invariant should never break) or a "missing migrator" (acknowledge older GSD versions may have skipped it, provide a healer)? Framing affects the issue title.

---

## Issue 4: ROADMAP.md plan list should show wave dependencies and cross-cutting constraints

**Type:** enhancement · **Severity:** low-medium

**What happened:** `/gsd-plan-phase` already writes a per-plan summary list into ROADMAP.md (wave, objective, requirements per plan). That covers the "what are the plans" question. What it doesn't show:

1. **Wave dependency chain** — which plans must complete before others can start. ROADMAP lists plans in order but doesn't make the blocking relationships explicit (e.g. "Wave 2 cannot start until Wave 1 plans are green").
2. **Cross-cutting constraints** — rules that apply to every plan in the phase (e.g. a banned-word policy, a framing principle, a security invariant). These currently live buried inside individual PLAN.md files; a reviewer skimming ROADMAP has no visibility.

Without these, a reviewer reading ROADMAP has the "what" but not the "shape" — they can't spot sequencing risks or catch a plan that violates a phase-wide constraint without reading all individual PLAN.md files.

**Expected:** The ROADMAP plan list for an in-progress phase includes:
- A wave dependency note per wave (e.g. "Wave 2 — blocked on Wave 1 completion")
- A "Cross-cutting constraints" subsection listing phase-wide rules surfaced from PLAN frontmatter `must_haves` / `constraints` fields

**Suggested implementation:** Extend the ROADMAP-writing step in `plan-phase.md` to emit these two additions after the plan checklist. Both are derivable from existing PLAN frontmatter — no extra LLM pass needed for dependencies; one short aggregation pass for constraints.

> **Advise on:** Whether wave dependency notes belong inline in the checklist or as a separate subsection. Also whether cross-cutting constraints should be pulled automatically from PLAN frontmatter or require the planner to explicitly nominate them.

---

## Issue 5: No signal distinguishes canonical GSD artifacts from historical ad-hoc session output

**Type:** docs / dx · **Severity:** low-medium

**What happened:** A planning session encountered an ad-hoc artifact at `.planning/` root that looked authoritative. The artifact had been named similarly to real GSD templates and had been sitting at the root for multiple milestones. An agent assumed it was a canonical GSD convention and proposed adding a task to reproduce it in a future phase. The mistake was caught at human review.

The core problem: canonical GSD artifacts (produced by known workflows, with defined templates) and historical one-off session outputs live in the same directory with no distinguishing signal. A confident-looking filename reads as authoritative to a fresh agent.

<details>
<summary>How this played out</summary>

The file in question was named to look like a "final verification" artifact. GSD's `templates/` directory has no such template — the real conventions are per-phase `verification-report.md` (from execute-phase) and `milestone-archive.md` (from complete-milestone). But with no index of canonical names, the agent had no way to verify.

</details>

**Suggested fix (two complementary cuts):**

1. **`templates/README.md`** — enumerate canonical artifact names, which workflow produces each, and where each lives in a typical project tree. ~40 lines, one-time effort, immediately referenceable by agents.

2. **`gsd-health` lint** — flag `.planning/*.md` files that don't match any known canonical pattern. Non-blocking warning with suggested remediation (move to archive subdir). Higher-leverage than a README because it runs automatically.

> **Advise on:** File as one issue ("agent orientation gaps") or two separate issues (README vs lint)? Also whether you want to prescribe a specific canonical-artifact registry as part of GSD's contract.

---

## Summary

| # | Title | Type | Severity |
|---|-------|------|----------|
| 1 | `stopped_at` overwritten with old session notes | bug | medium |
| 2 | Stale phase dirs corrupt state phase counts after milestone close | bug + enhancement | medium |
| 3 | MILESTONES.md silently drifts from archived snapshots | bug / missing check | medium |
| 4 | ROADMAP plan list should show wave dependencies + cross-cutting constraints | enhancement | low-medium |
| 5 | No signal distinguishes canonical artifacts from ad-hoc session output | docs / dx | low-medium |
