---
phase: 30
plan: 09
type: execute
wave: 4
depends_on: [30-03, 30-04, 30-05, 30-06, 30-07, 30-08]
files_modified:
  - .planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/screenshots/
  - .planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-SUMMARY.md
  - CHANGELOG.md
autonomous: false
requirements: [DOCS-01, DOCS-02, DOCS-03, DOCS-04, DOCS-05, DOCS-06, DOCS-07, DOCS-08, DOCS-09]
must_haves:
  truths:
    - "`mkdocs build --strict` is green end-to-end on the final docs tree."
    - "`vale --config=.vale.ini docs/` returns 0 (no P1/P2 violations anywhere in docs/)."
    - "`scripts/test_phase_30_tutorial.sh` runs green against `target/release/reposix-sim` + the tutorial — proves the 5-minute path is accurate."
    - "`python3 scripts/check_phase_30_structure.py` exits 0 (all structural invariants pass)."
    - "Playwright screenshots at 1280x800 + 375x667 exist for: home, mental-model, vs-mcp-sdks, tutorial, how-it-works/filesystem, how-it-works/git, how-it-works/trust-model (7 pages × 2 viewports = 14 PNGs)."
    - "doc-clarity-review against rendered `docs/index.md` returns verdict LANDED (one sentence value prop stated by a cold-reader subagent)."
    - "CHANGELOG.md has a new `[Unreleased]` → `v0.9.0` entry enumerating Phase 30 changes."
    - "30-SUMMARY.md consolidates all per-plan SUMMARYs into a single milestone-grade review including the Validation-Sign-Off table from 30-VALIDATION.md."
  artifacts:
    - path: ".planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/screenshots/"
      provides: "14 playwright screenshots (7 pages × 2 viewports)"
    - path: ".planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-SUMMARY.md"
      provides: "Milestone-grade phase summary"
      min_lines: 60
    - path: "CHANGELOG.md"
      provides: "v0.9.0 Docs & Narrative entry"
  key_links:
    - from: "30-SUMMARY.md"
      to: ".planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/screenshots/"
      via: "embedded image links"
      pattern: "screenshots/.*\\.png"
    - from: "CHANGELOG.md"
      to: ".planning/phases/30-.../30-SUMMARY.md"
      via: "phase reference"
      pattern: "Phase 30"
---

# Phase 30-09 — Final Validation, Screenshots, Cold-Reader Review, SUMMARY + CHANGELOG

<objective>
Run the phase's feedback-loop verification suite in full, commit screenshots, write the phase SUMMARY, and stage the CHANGELOG. After this plan lands, Phase 30 is shippable.

Concretely, after this plan:
- `mkdocs build --strict` + `vale --config=.vale.ini docs/` + `scripts/test_phase_30_tutorial.sh` + `python3 scripts/check_phase_30_structure.py` all return 0.
- 14 Playwright screenshots are committed to `.planning/phases/30-.../screenshots/` (7 pages × desktop+mobile).
- `doc-clarity-review docs/index.md --prompt "<custom 10-second value-prop prompt>"` returns verdict LANDED.
- Cross-AI review is captured: orchestrator invokes `gsd-code-reviewer` subagent on the diff; findings attached to 30-SUMMARY.md.
- `CHANGELOG.md` has a new `v0.9.0` entry ready for the user to tag.
- `30-SUMMARY.md` consolidates all per-plan SUMMARYs + screenshots + validation sign-off into one phase-grade document.

This is a HUMAN checkpoint plan. The last two tasks (doc-clarity-review verdict + SUMMARY composition) involve human review and may gate the phase behind a revision cycle if the verdict comes back PARTIAL or MISSED.

Purpose: close the feedback loop per CLAUDE.md OP #1. Every gate in 30-VALIDATION.md §"Validation Sign-Off" must tick. No hand-wave.

Output: 14 PNGs committed, 1 CHANGELOG entry, 1 phase SUMMARY, possibly 1 revision checkpoint.

**Locked decisions honored:**
- Screenshots target `http://127.0.0.1:*` only — localhost allowlist per `scripts/screenshot-docs.sh` (plan 30-01).
- Screenshot naming convention: `{page-slug}-{desktop|mobile}.png` under `.planning/phases/30-.../screenshots/` (not `docs/screenshots/phase-30/` — that path is for screenshots the published site references; phase evidence stays in `.planning/`).
- doc-clarity-review uses a CUSTOM prompt (RESEARCH.md §Example 4), not the default skill prompt.
- LANDED / PARTIAL / MISSED verdict — only LANDED passes the phase.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@.planning/ROADMAP.md
@.planning/REQUIREMENTS.md
@.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-VALIDATION.md
@.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-RESEARCH.md

@CHANGELOG.md

@docs/index.md
@docs/mental-model.md
@docs/vs-mcp-sdks.md
@docs/tutorial.md
@docs/how-it-works/filesystem.md
@docs/how-it-works/git.md
@docs/how-it-works/trust-model.md

@scripts/screenshot-docs.sh
@scripts/test_phase_30_tutorial.sh
@scripts/check_phase_30_structure.py

<interfaces>
Validation gates from 30-VALIDATION.md §"Validation Sign-Off":

- [ ] All tasks have `<automated>` verify OR manual verification
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify — mkdocs build is the cheap repeated probe
- [ ] Wave 0 installs Vale, screenshot runner, mermaid renderer BEFORE tasks depending on them (plan 30-01 covers)
- [ ] No watch-mode flags — CI-compatible one-shot runs only
- [ ] Feedback latency < 90s (Vale ~3s, mkdocs ~10s, screenshots ~30s, tutorial test ~20s)

Playwright MCP tool interface (invoked by the orchestrator):
- `mcp__playwright__browser_navigate` with a URL like `http://127.0.0.1:8000/mental-model/`
- `mcp__playwright__browser_resize` with `width=1280, height=800` or `width=375, height=667`
- `mcp__playwright__browser_take_screenshot` with `filename=...` saving to a target path

doc-clarity-review skill: `~/.claude/skills/doc-clarity-review/SKILL.md` defines the subprocess-isolation pattern. Invoke via `claude -p "<prompt>" <file>` per RESEARCH.md §Example 4.
</interfaces>
</context>

## Chapters

- **[Task 1: Run the full validation suite](./task-1-validation-suite.md)** — Run all four automated gates (mkdocs, Vale, structural linter, tutorial e2e); capture timing. All must return 0 before proceeding.
- **[Task 2: Capture Playwright screenshots](./task-2-screenshots.md)** — Start mkdocs serve, generate screenshot manifest, invoke Playwright MCP for 14 PNGs (7 pages × 2 viewports), visual review, git-add.
- **[Task 3: Cold-reader review](./task-3-cold-reader.md)** — Human-gated checkpoint: run doc-clarity-review with custom prompt, parse LANDED/PARTIAL/MISSED verdict, user confirms, copy evidence to phase dir.
- **[Task 4: Compose 30-SUMMARY.md + CHANGELOG.md](./task-4-summary-changelog.md)** — Consolidate 8 per-plan SUMMARYs, write milestone-grade SUMMARY with gates table + screenshots + verdict; append v0.9.0 CHANGELOG entry; commit.
- **[Verification, Success Criteria, Output](./verification-success-output.md)** — Final checklist, success criteria, and the PHASE 30 SHIPPED output block.
