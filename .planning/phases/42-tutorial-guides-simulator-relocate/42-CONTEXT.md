---
phase: 42
name: Tutorial + three guides + simulator relocate (DOCS-04, DOCS-05, DOCS-06)
milestone: v0.10.0
status: in-progress
date: 2026-04-24
skip_discuss: true
requirements: [DOCS-04, DOCS-05, DOCS-06]
---

# Phase 42 CONTEXT — Tutorial + guides + simulator relocate

## Phase boundary (from ROADMAP.md §Phase 42)

Phase 42 ships five new docs and (re)homes the simulator page:

- `docs/tutorials/first-run.md` — five-minute first-run tutorial against the simulator (DOCS-06).
- `docs/guides/write-your-own-connector.md` — `BackendConnector` walkthrough (DOCS-04 part 1).
- `docs/guides/integrate-with-your-agent.md` — Claude Code / Cursor / SDK pointer page (DOCS-04 part 2).
- `docs/guides/troubleshooting.md` — push rejections, audit-log queries, blob-limit recovery (DOCS-04 part 3).
- `docs/reference/simulator.md` — reference page for the in-process axum simulator (DOCS-05; no prior `docs/simulator*` exists, so this is a fresh write rather than a `git mv`).

Pure docs work. No `cargo` invocations, no `mkdocs build --strict` — Phase 43 owns the nav restructure and Phase 45 owns the strict build + screenshots.

## Inputs (already-committed artifacts)

- `docs/concepts/mental-model-in-60-seconds.md` (Phase 40) — three keys; the tutorial elaborates them with a real run.
- `docs/how-it-works/{filesystem-layer,git-layer,trust-model}.md` (Phase 41) — the trio the guides cross-link back to.
- `docs/reference/testing-targets.md` — Confluence TokenWorld, GitHub `reubenjohn/reposix`, JIRA `TEST`.
- `docs/benchmarks/v0.9.0-latency.md` — `8 ms` get-issue, `24 ms` cold init, `9 ms` list, `5 ms` capabilities probe.
- `crates/reposix-core/src/backend.rs` — `BackendConnector` trait + `BackendFeature` + `DeleteReason`.
- `crates/reposix-jira/`, `crates/reposix-github/`, `crates/reposix-confluence/` — three reference connector implementations.
- `crates/reposix-sim/src/{lib,main}.rs` — sim binds `127.0.0.1:7878` by default; uses `crates/reposix-sim/fixtures/seed.json` as seed.
- `crates/reposix-cache/src/path.rs` — cache lives at `<XDG_CACHE_HOME>/reposix/<backend>-<project>.git/cache.db` (or `REPOSIX_CACHE_DIR` override).
- `crates/reposix-cli/src/init.rs` — translates `sim::demo` → `reposix::http://127.0.0.1:7878/projects/demo`.
- `.claude/skills/reposix-agent-flow/SKILL.md` — referenced from the integrate-with-your-agent guide.

## Claude's discretion (skip-discuss decisions)

- **Inline runner, no planner hop.** Mirrors Phase 41 — five small docs files, atomic commits per file. Wave sizing is one commit per file.
- **No `scripts/tutorial-runner.sh` this phase.** ROADMAP §42 success criterion 2 requires it but the runner script is descoped from this Phase 42 runner per the user prompt ("OPTIONAL; deferred to Phase 45 if scope creeps"). Phase 43/45 reconciliation.
- **No `mkdocs build --strict`.** Phase 45 owns it. We keep relative paths consistent so the strict build is one-shot later.
- **`docs/reference/simulator.md` is a fresh write.** No prior `docs/simulator*.md` exists (verified `find docs -iname "*sim*" -o -iname "*simulator*"` returned nothing). Phase 43 finalizes nav placement.
- **Old `docs/connectors/guide.md` is NOT redirected/deleted here.** Phase 43 nav restructure handles redirects/stubs (per ROADMAP §43 success criterion 7). This phase only authors the new guide.
- **P1 (no "replace") + P2 (no FUSE/fusermount/kernel/syscall in user-facing pages) enforced manually.** Phase 43 codifies in the linter.

## Wave sizing — single-pass, no planner hop

Five atomic-commit waves, one file per commit:

- **42-01:** `docs/reference/simulator.md` (DOCS-05).
- **42-02:** `docs/tutorials/first-run.md` (DOCS-06).
- **42-03:** `docs/guides/write-your-own-connector.md` (DOCS-04 part 1).
- **42-04:** `docs/guides/integrate-with-your-agent.md` (DOCS-04 part 2).
- **42-05:** `docs/guides/troubleshooting.md` (DOCS-04 part 3).

Verification (`42-VERIFICATION.md`) follows the runner's goal-backward checks.

## Success criteria (assertable subset)

1. Five new files exist at the expected paths.
2. Tutorial includes the seven steps from the runner brief (install · start sim · `reposix init` · inspect · edit · commit/push · verify audit).
3. Each guide has the section structure called out in the runner brief.
4. Cross-links present (tutorial → mental model + benchmarks; troubleshooting → git-layer; guides → trust-model + skill).
5. The word `replace` appears 0 times in any of the five new files (`grep -ic replace <file>` returns 0).
6. P2 banned terms (`FUSE`, `fusermount`, `kernel`, `syscall`, `daemon`) appear 0 times in tutorial + guides (Layer 1–2). Simulator reference (Layer 4) may briefly mention if relevant — try not to.

Out of scope (deferred):

- mkdocs.yml nav restructure → Phase 43.
- `docs/connectors/guide.md` redirect/deletion → Phase 43.
- Banned-words linter wiring → Phase 43.
- `scripts/tutorial-runner.sh` end-to-end runner → Phase 45 (per runner brief).
- doc-clarity-review release gate → Phase 44.
- Playwright screenshots + final `mkdocs build --strict` → Phase 45.
