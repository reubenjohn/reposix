---
phase: 30
plan: 07
type: execute
wave: 2
depends_on: [30-01, 30-02, 30-04]
files_modified:
  - docs/guides/write-your-own-connector.md
  - docs/connectors/guide.md
  - docs/guides/integrate-with-your-agent.md
  - docs/guides/troubleshooting.md
  - docs/reference/simulator.md
autonomous: true
requirements: [DOCS-04, DOCS-05]
must_haves:
  truths:
    - "docs/guides/write-your-own-connector.md contains the full 465-line content from docs/connectors/guide.md, preserved verbatim via `git mv` (original path is source of truth for file history)."
    - "docs/connectors/guide.md no longer exists after the move — no dangling links because mkdocs.yml's `not_in_nav` from plan 30-04 lists `connectors/*`."
    - "docs/guides/integrate-with-your-agent.md is authored greenfield with 4 sections (Claude Code / Cursor / Custom SDK / Gotchas) each containing a real prompt/config example and a pointer to why.md#token-economy-benchmark."
    - "docs/guides/troubleshooting.md has 3 symptom/cause/fix triads filled with runnable diagnostic commands."
    - "docs/reference/simulator.md is filled as a Reference-tier page: CLI flag table lifted from docs/reference/cli.md, endpoint summary lifted from docs/reference/http-api.md, and a NEW 'Seeding + fixtures' section describing crates/reposix-sim/fixtures/seed.json."
  artifacts:
    - path: "docs/guides/write-your-own-connector.md"
      provides: "465-line connector-authoring guide, moved from connectors/"
      min_lines: 400
    - path: "docs/guides/integrate-with-your-agent.md"
      provides: "4-section agent-integration guide with concrete prompt/config examples"
      min_lines: 100
    - path: "docs/guides/troubleshooting.md"
      provides: "3 symptom/cause/fix triads for common failure modes"
      min_lines: 50
    - path: "docs/reference/simulator.md"
      provides: "Dev-tooling reference with CLI flags, endpoints, and seeding"
      min_lines: 60
  key_links:
    - from: "docs/guides/integrate-with-your-agent.md"
      to: "docs/why.md"
      via: "token-economy-benchmark pointer"
      pattern: "why.md#token-economy-benchmark"
    - from: "docs/reference/simulator.md"
      to: "crates/reposix-sim/fixtures/seed.json"
      via: "seeding section reference"
      pattern: "reposix-sim/fixtures/seed.json"
---

# Phase 30-07 — Complete guides/ tree and reference/simulator.md

<objective>
Complete the guides/ tree and reference/simulator.md. After this plan lands:

- `docs/guides/write-your-own-connector.md` exists with full 465-line content preserved from `docs/connectors/guide.md`. The move is done via `git mv` so git history follows. `docs/connectors/guide.md` no longer exists.
- `docs/guides/integrate-with-your-agent.md` is a 4-section guide: system-prompt pattern for Claude Code, `.cursorrules` pattern for Cursor, ~20-line Python/TypeScript SDK example, and a "Gotchas" section covering the taint boundary + `REPOSIX_ALLOWED_ORIGINS` setup.
- `docs/guides/troubleshooting.md` has 3 real symptom/cause/fix triads (empty-folder, bulk-delete rejection, audit-log query) with runnable diagnostic commands.
- `docs/reference/simulator.md` is a complete Reference page: CLI flags, endpoints, and a new "Seeding + fixtures" section describing the seed.json shape.

Purpose: DOCS-04 ships three guides (connector, agent, troubleshooting) + DOCS-05 relocates the simulator page. This plan handles all four in one scope — they're independent Layer-2/3 authoring tasks with no content-dependencies among each other.

Output: 1 file move (`connectors/guide.md` → `guides/write-your-own-connector.md`), 3 filled stubs.

**Locked decisions honored:**
- `git mv` preserves git history per PATTERNS.md §"docs/guides/write-your-own-connector.md" (exact file move classification).
- DOCS-05 simulator under `reference/`, not `how-it-works/` — plan 30-04 already placed it in nav. This plan fills content.
- integrate-with-your-agent examples reference token-savings numbers from `why.md` by link, NOT by duplicating methodology.
- All guide stubs must stay Vale-clean (Layer-2 — ProgressiveDisclosure rule active).
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-RESEARCH.md
@.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-PATTERNS.md

@docs/connectors/guide.md
@docs/guides/write-your-own-connector.md
@docs/guides/integrate-with-your-agent.md
@docs/guides/troubleshooting.md
@docs/reference/simulator.md

@docs/reference/cli.md
@docs/reference/http-api.md
@docs/why.md
@docs/demo.md

<interfaces>
Existing content sources (lifted with minimal rephrasing):

- `docs/connectors/guide.md` — 465 lines; full content moves verbatim to guides/.
- `docs/reference/cli.md` §`reposix sim` (lines 47-58) — flag table.
- `docs/reference/http-api.md` (lines 7-59) — endpoint pairs.
- `crates/reposix-sim/fixtures/seed.json` — shape referenced by reference/simulator.md.
- `docs/why.md` §token-economy-benchmark — cited by integrate-with-your-agent.md.
- `docs/demo.md` §"Limitations / honest scope" — voice for troubleshooting triads.

git mv contract: `git mv <src> <dst>` — the dst file already exists as a stub (from plan 30-02); we must remove the stub first, then `git mv` in its place. Alternatively: `git rm docs/guides/write-your-own-connector.md && git mv docs/connectors/guide.md docs/guides/write-your-own-connector.md`. The executor picks whichever sequence preserves history on the source.
</interfaces>
</context>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| integrate-with-your-agent examples -> reader copy-paste | Example prompts and SDK code must not include real tokens, real tenants, or real webhook URLs. |
| troubleshooting diagnostic commands -> reader execution | Commands like `sqlite3 ... 'SELECT ...'` run locally on the reader's machine; must not exfiltrate to a remote. |
| reference/simulator seed.json content | Describing seed.json reveals fixture structure but no secrets (fixture is public in the repo). |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-30-07-01 | Information Disclosure | Example prompts include real endpoints or API tokens | mitigate | Examples use `/tmp/tutorial-mnt` (synthetic path) and `acme.atlassian.net` (placeholder). Reviewer greps diff for `reuben-john.atlassian.net` and other real hostnames. |
| T-30-07-02 | Tampering | git mv loses file content (edge case in some git versions) | mitigate | Verify content after move via `wc -l docs/guides/write-your-own-connector.md` matches `wc -l` of the original (~465 lines). |
| T-30-07-03 | Repudiation | Integrate-guide claims token-savings figure different from why.md | mitigate | Do NOT duplicate the number. Link back: `See [Why reposix](../why.md#token-economy-benchmark)`. |
</threat_model>

## Chapters

- **[T1 — git mv connectors/guide.md to guides/write-your-own-connector.md](./T01-git-mv-connector-guide.md)**
  Move the 465-line connector-authoring guide from `docs/connectors/guide.md` to `docs/guides/write-your-own-connector.md` via `git mv`, preserving history; verify content and run `mkdocs build --strict`.

- **[T2 — Fill integrate-with-your-agent.md + troubleshooting.md](./T02-fill-agent-and-troubleshooting.md)**
  Author the 4-section agent-integration guide (Claude Code / Cursor / Custom SDK / Gotchas) and 3 symptom/cause/fix troubleshooting triads; Vale-clean verification.

- **[T3 — Fill docs/reference/simulator.md](./T03-fill-simulator-reference.md)**
  Complete the simulator reference page with CLI flag table, HTTP endpoint table, seeding + fixtures section, and audit log info; `mkdocs build --strict` green.

## Verification

1. `mkdocs build --strict` exits 0.
2. `~/.local/bin/vale --config=.vale.ini docs/guides/ docs/reference/simulator.md` exits 0.
3. `python3 scripts/check_phase_30_structure.py` — guide-existence checks pass; deleted-paths still show docs/architecture.md + docs/security.md + docs/demo.md (expected until Wave 3).
4. `git log --follow docs/guides/write-your-own-connector.md` shows commits from before the rename (git rename detection working).

## Success Criteria

- write-your-own-connector.md moved via git mv with history preserved; old path removed.
- integrate-with-your-agent.md has 4 filled sections with concrete examples.
- troubleshooting.md has 3 symptom/cause/fix triads.
- reference/simulator.md is a complete Reference page.
- mkdocs build --strict green; Vale clean.

<output>
After completion, create `.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-07-SUMMARY.md` documenting:
- git-mv confirmation (line count preserved)
- Which sections of integrate-with-your-agent.md landed (Claude Code / Cursor / Custom SDK / Gotchas)
- 3 troubleshooting triads listed
- simulator.md flag table compared against current reposix-sim binary (any drift flagged)
</output>
