← [back to index](./index.md) · phase 30 research

## Project Constraints (from CLAUDE.md)

The project `CLAUDE.md` adds specifics that Phase 30 must honor:

| Directive | Phase 30 Implication |
|-----------|---------------------|
| OP #1: close the feedback loop | Playwright screenshots + `mkdocs build --strict` + `gh run view` — all three must be green on the phase-ship commit. |
| OP #4: self-improving infrastructure | The banned-word linter MUST be promoted to a committed script (Vale config + hook + CI step). Same for tutorial verification. No ad-hoc bash. |
| "Simulator is the default / testing backend" | The tutorial MUST run against `reposix-sim`, not a real backend. No real-backend credentials appear in tutorial examples. |
| "Tainted by default" | Trust-model page discusses taint; does NOT discuss "security by X" — must discuss what's enforced. |
| "Audit log is non-optional" | Trust-model page's audit-log claim must match the existing `docs/security.md` SG-06 row exactly. |
| "No hidden state" | All phase state is in `.planning/phases/30-.../` + `docs/`. No "here's a good idea I didn't commit." |
| "Mount point = git repo" | Tutorial must demonstrate this — `git init` in the mount is load-bearing. |
| "Always enter through a GSD command" | No work outside `/gsd-plan-phase 30` → `/gsd-execute-phase 30` → `/gsd-verify-work`. |
| Subagent delegation: "Aggressive" | 5 parallel subagents per the source-of-truth note's fanout. Orchestrator coordinates only. |
| Threat model enforcement | Every trust-model claim cross-referenced against shipped SG-* evidence. |

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| README.md as landing | mkdocs-material site with Diátaxis IA | Phases 25-26 (docs reorg) | Current docs/ is correctly organized but not narratively led; Phase 30 fixes the landing layer. |
| Client-side mermaid was brittle on mobile | mkdocs-material 9.x auto-themes + mobile-aware | 2024+ | Our current mkdocs-material 9.7.1 has it; no migration needed. |
| Docs linters were prose-focused (style) only | Layer-scoped semantic linters (Vale) | Ongoing | Vale's `extends: existence` with glob-scoped rules is the modern pattern. |
| Social cards required a paid Material insiders tier | Free tier since 9.4 | 2024+ | `material[imaging]` works on our OSS 9.7.1. |
| Custom Jinja landing page was the only option for hero design | Material grid cards + markdown handles most cases | 2024+ | Recommend markdown-native. |

**Deprecated / outdated:**
- Pre-9.x mkdocs-material had `navigation.instant` rendering quirks with mermaid — resolved in 9.x.
- Proselint hasn't had a major release since 2021 and is less actively maintained than Vale.

## Assumptions Log

All claims in this research were verified against (a) tool availability on this host (via `command -v` probes), (b) project artifacts (files read directly), or (c) cited external sources (official docs + WebSearch + WebFetch analyses). No `[ASSUMED]` entries.

**User confirmation needed — none.** The research verified the tooling, patterns, and source content. Decisions remain with the planner (linter choice, pre-render vs client-side mermaid, scope-split).

## Open Questions

1. **Should `docs/security.md` be deleted or kept as a condensed index?**
   - What we know: its content carves into `how-it-works/trust-model.md`.
   - What's unclear: external inbound links (from GitHub repo, issues, etc.) point to `security.md` — soft 404s will accumulate.
   - Recommendation: delete, accept soft 404s, add a redirect via gh-pages' `404.html` if the team wants (one-liner meta refresh). The planner decides.

2. **Should the simulator reference page live under `reference/` or under `development/`?**
   - What we know: CONTEXT.md says "Reference." Diátaxis validates "Reference."
   - What's unclear: since it's dev-tooling specifically, it sits adjacent to `development/contributing.md`.
   - Recommendation: **reference/simulator.md** — sim is end-user-facing too (contributors and plugin authors use it). CONTEXT.md is authoritative here.

3. **How much of `docs/demos/` stays?**
   - What we know: `docs/demos/index.md` is rich (13KB). `docs/demos/recordings/` is the asciicast.
   - What's unclear: does the 5-minute tutorial replace the demo tier-2 walkthrough entirely, or do they coexist?
   - Recommendation: keep `docs/demos/` intact as a deep-dive section (possibly move under `guides/` or keep standalone). Tutorial is the 5-minute path; demos are the "here are five more scenarios" depth.

4. **Does the `docs/guides/` nav include the existing tier-demo scripts (e.g. `scripts/demos/05-mount-real-github.sh`)?**
   - What we know: existing `docs/demos/index.md` documents these.
   - What's unclear: should they get proper per-backend guides ("Connect to GitHub", "Connect to Jira", "Connect to Confluence")?
   - Recommendation: create stubs — `docs/guides/connect-github.md`, `connect-jira.md`, `connect-confluence.md` — that link to existing tier demos and the reference pages. Stubs are fine; full content is a future phase.
