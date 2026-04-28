---
name: reposix-quality-backfill
description: "Run the full docs-alignment backfill audit across docs/ + archived REQUIREMENTS.md. Top-level slash command — delegates to reposix-quality-doc-alignment backfill playbook. Dispatches ~25–35 Haiku extractor subagents in waves of 8, runs merge-shards, generates PUNCH-LIST.md."
argument-hint: ""
allowed-tools:
  - Bash
  - Read
  - Edit
  - Write
  - Task
  - Grep
  - Glob
  - Skill
---

<objective>
Thin slash-command entry that delegates to the umbrella `reposix-quality-doc-alignment` skill in **backfill** mode. Designed to be invoked from a fresh top-level Claude session at v0.12.0 close (P65) and again whenever the catalog needs to be re-established from scratch (e.g., after a manual edit corrupted state).

This skill exists as a separate slash command so the user can type `/reposix-quality-backfill` directly. The full implementation lives in `.claude/skills/reposix-quality-doc-alignment/backfill.md`.
</objective>

<process>
<step name="validate_args">
Reject any arguments. Backfill is the universe of `docs/**/*.md` + `README.md` + archived REQUIREMENTS.md by definition; subsetting belongs in `refresh` mode.
</step>

<step name="delegate">
Invoke `Skill(skill="reposix-quality-doc-alignment", args="backfill")` and let the umbrella skill handle the rest.
</step>
</process>

<cross_references>
- `.claude/skills/reposix-quality-doc-alignment/SKILL.md` — umbrella skill (parses `refresh` vs `backfill`).
- `.claude/skills/reposix-quality-doc-alignment/backfill.md` — backfill playbook (normative; consumes `06-p65-backfill-brief.md`).
- `.planning/research/v0.12.0-docs-alignment-design/06-p65-backfill-brief.md` — backfill protocol source of truth.
- `.planning/research/v0.12.0-docs-alignment-design/03-execution-modes.md` — why this is top-level only.
</cross_references>
