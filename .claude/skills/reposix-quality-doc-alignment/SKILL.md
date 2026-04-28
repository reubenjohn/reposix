---
name: reposix-quality-doc-alignment
description: "Run the docs-alignment dimension — refresh stale rows for a single doc OR run the full backfill audit across docs/ + archived REQUIREMENTS.md. Subagents propose claim→test bindings with file:line citations; the `reposix-quality` binary validates and mints catalog state. Reads quality/catalogs/doc-alignment.json. Top-level only (depth-2 subagent fan-out unreachable from inside /gsd-execute-phase; subscription users cannot fall back to claude -p)."
argument-hint: "[refresh <doc-file>] | [backfill]"
allowed-tools:
  - Bash
  - Read
  - Edit
  - Write
  - Task
  - Grep
  - Glob
---

<objective>
Operate the docs-alignment quality dimension. Two playbooks:

- **refresh** — one stale doc has drifted; re-extract claims + re-bind tests for that doc only.
- **backfill** — full audit across `docs/**/*.md`, `README.md`, and archived REQUIREMENTS.md from milestones v0.6.0–v0.11.0. Run once at v0.12.0 close to surface the punch list v0.12.1 closes.

Both playbooks are **top-level only**. Reasons:
- `gsd-executor` does not have the `Task` tool, and depth-2 subagent spawning is forbidden by the harness. Subagent fan-out cannot live inside an executor session.
- Subscription users cannot fall back to `claude -p` subprocess invocation (no API key on a subscription).

The user-facing slash commands are `/reposix-quality-refresh <doc-file>` and `/reposix-quality-backfill`; both delegate here.

Cross-references:
- Catalog: `quality/catalogs/doc-alignment.json`
- Dimension home: `quality/gates/docs-alignment/README.md`
- Refresh playbook: `.claude/skills/reposix-quality-doc-alignment/refresh.md`
- Backfill playbook: `.claude/skills/reposix-quality-doc-alignment/backfill.md`
- Subagent prompts: `.claude/skills/reposix-quality-doc-alignment/prompts/{extractor,grader}.md`
- Architecture: `.planning/research/v0.12.0-docs-alignment-design/02-architecture.md`
- Execution-mode rationale: `.planning/research/v0.12.0-docs-alignment-design/03-execution-modes.md`
</objective>

<process>

<step name="parse_args">
Parse `$ARGUMENTS`:

- `refresh <doc-file>` → load `.claude/skills/reposix-quality-doc-alignment/refresh.md` and execute.
- `backfill` (no further args) → load `.claude/skills/reposix-quality-doc-alignment/backfill.md` and execute.
- (no args) → print usage + summary of both playbooks; exit 0.

Reject any other shape with a clear error naming the two playbook entry points.
</step>

<step name="precondition_checks">
Before either playbook runs, assert:

1. `cargo build -p reposix-quality --release` succeeds. The binary at `target/release/reposix-quality` is the source of truth for catalog mutation; subagents never write JSON directly.
2. `quality/catalogs/doc-alignment.json` exists and parses. If missing, fail loud and tell the user to run `/gsd-execute-phase 64` first (P64 ships the catalog seed).
3. The orchestrator has `Task` (depth 0). If invoked from inside `gsd-executor` (depth 1, no `Task`), fail loud with the rationale and tell the user to re-invoke from a top-level session.
</step>

<step name="run_playbook">
Hand off to the playbook file. Each playbook is normative — follow it verbatim.

- `refresh.md` — single-doc refresh; one Opus grader Task per stale row.
- `backfill.md` — `plan-backfill`, fan-out ~25–35 Haiku extractor Tasks in waves of 8, `merge-shards`, `PUNCH-LIST.md`, verifier dispatch.
</step>

<step name="commit_results">
Per playbook:

- **refresh** — commit catalog row updates + any verification artifacts; one atomic commit.
- **backfill** — commit cadence per `06-p65-backfill-brief.md` § "Commit cadence" (MANIFEST → run-dir → catalog → PUNCH-LIST → CLAUDE.md → SURPRISES → VERDICT). Phase-close commit follows the standard atomic-commit pattern.
</step>

</process>

<success_criteria>
- [ ] `refresh <doc-file>` re-extracts claims for one doc only, dispatches grader subagents, and exits 0 when all rows for that doc are GREEN or appropriately MISSING_TEST/RETIRE_PROPOSED.
- [ ] `backfill` produces a deterministic `MANIFEST.json`, dispatches extractor subagents in waves of 8, runs `merge-shards`, generates `PUNCH-LIST.md`, dispatches the verifier (Path A), and exits 0.
- [ ] Subagents NEVER write catalog JSON directly. All state mutation flows through `reposix-quality doc-alignment <subcmd>` calls — the binary validates citations and computes hashes.
- [ ] Conflicts from `merge-shards` are surfaced via `CONFLICTS.md`; the orchestrator resolves by editing shard JSONs and re-running. No partial catalog writes.
- [ ] `confirm-retire` is environment-guarded — refuses to run if `$CLAUDE_AGENT_CONTEXT` is set. Only humans can confirm retirement.
- [ ] Both playbooks fail loud (non-zero exit + structured stderr) when invoked from a depth-1 context (no `Task` tool).
- [ ] Banned-words clean (no forbidden synonym for `migrate`).
</success_criteria>

<cross_references>
- `.planning/research/v0.12.0-docs-alignment-design/02-architecture.md` — catalog schema, hash semantics, binary surface, skill layout.
- `.planning/research/v0.12.0-docs-alignment-design/03-execution-modes.md` — top-level vs executor; why this skill is top-level only.
- `.planning/research/v0.12.0-docs-alignment-design/05-p64-infra-brief.md` — implementation spec for the framework.
- `.planning/research/v0.12.0-docs-alignment-design/06-p65-backfill-brief.md` — normative backfill protocol (consumed by `backfill.md`).
- `quality/PROTOCOL.md` — runtime contract; "subagents propose with citations; tools validate and mint" + "tools fail loud, structured, agent-resolvable".
- `.claude/skills/reposix-quality-review/SKILL.md` — P61 precedent for skill shape.
</cross_references>
