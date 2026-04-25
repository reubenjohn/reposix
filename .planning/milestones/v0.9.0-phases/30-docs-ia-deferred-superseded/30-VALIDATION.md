---
phase: 30
slug: docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-17
---

# Phase 30 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | mkdocs (build --strict) + Vale (banned-word linter) + doc-clarity-review (value-prop review) + Playwright (screenshots) |
| **Config file** | `mkdocs.yml`, `.vale.ini` (Wave 0 installs), `scripts/screenshot-docs.{sh,py}` (Wave 0 installs) |
| **Quick run command** | `mkdocs build --strict` |
| **Full suite command** | `mkdocs build --strict && vale docs/ && scripts/screenshot-docs && gh run view` |
| **Estimated runtime** | ~60–90 seconds (mkdocs ~10s, vale ~3s, screenshots ~30s, CI view ~5s) |

---

## Sampling Rate

- **After every task commit:** Run `mkdocs build --strict`
- **After every plan wave:** Run `mkdocs build --strict && vale docs/`
- **Before `/gsd-verify-work`:** Full suite must be green (mkdocs + vale + screenshots + `doc-clarity-review docs/index.md` returns `LANDED`)
- **Max feedback latency:** 90 seconds

---

## Per-Task Verification Map

> Filled by the planner as PLAN.md tasks are written. Below is the structural skeleton the planner must populate.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 30-NN-NN | NN | 0/1/2/3/4 | DOCS-0X | — | N/A (docs phase — no threat surface beyond banned-word linter) | build / lint / visual / content | `mkdocs build --strict` / `vale docs/` / `scripts/screenshot-docs` / `doc-clarity-review docs/index.md` | ✅ / ❌ W0 | ⬜ pending |

---

## Wave 0 Requirements

- [ ] `.vale.ini` + `styles/reposix/BannedLayerTerms.yml` — Vale config enforcing P2 banned terms above `docs/how-it-works/**`
- [ ] `scripts/screenshot-docs.{sh,py}` — Playwright wrapper that screenshots landing, how-it-works/*, tutorial at 1280px + 375px widths and commits to `.planning/phases/30-.../screenshots/`
- [ ] `scripts/render-mermaid.{sh,py}` — mcp-mermaid → SVG/PNG renderer for `docs/assets/diagrams/`
- [ ] `vale` binary installed (one-off Go binary, ~15MB) — checked in Wave 0 precheck
- [ ] `pre-commit` hook or git hook wiring Vale to docs commits (or CI-only if pre-commit deferred)

*Nyquist requires: Wave 0 installs MUST precede any task that depends on vale/screenshots/mermaid render.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Value prop lands in 10 seconds (human judgment) | DOCS-01 | Ultimate arbiter is a cold reader, not a linter | `doc-clarity-review docs/index.md --prompt "state reposix's value proposition in one sentence. Return LANDED/PARTIAL/MISSED + the sentence."` — `LANDED` is the pass; `PARTIAL`/`MISSED` requires copy revision. Calibrate on first run. |
| Mental model readable in 60 seconds | DOCS-03 | Same — requires comprehension judgment | `doc-clarity-review docs/mental-model.md` with a bespoke prompt checking the three conceptual keys surface |
| Diagrams are elegant, not comprehensive (aesthetics) | DOCS-02 | Subjective | Human review of rendered SVGs in phase SUMMARY |
| Theme "signals careful deliberate product" | DOCS-08 | Subjective | Human review of Playwright screenshots |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify OR manual verification in the table above
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify (mkdocs build is the cheap repeated probe)
- [ ] Wave 0 installs Vale, screenshot runner, mermaid renderer BEFORE tasks depending on them
- [ ] No watch-mode flags — CI-compatible one-shot runs only
- [ ] Feedback latency < 90s
- [ ] `nyquist_compliant: true` set in frontmatter after planner populates per-task table

**Approval:** pending
