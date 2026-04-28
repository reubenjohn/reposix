---
name: reposix-quality-refresh
description: "Refresh stale docs-alignment catalog rows for a single doc that drifted. Top-level slash command — delegates to reposix-quality-doc-alignment refresh playbook. Invoke after pre-push BLOCKs with STALE_DOCS_DRIFT."
argument-hint: "<doc-file>"
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
Thin slash-command entry that delegates to the umbrella `reposix-quality-doc-alignment` skill in **refresh** mode for a single drifted doc file. Designed to be invoked from a fresh top-level Claude session — typically when a `git push` was BLOCKed by `STALE_DOCS_DRIFT` on a specific doc and the user needs to re-bind that doc's claims.

This skill exists as a separate slash command so the user can type `/reposix-quality-refresh <doc-file>` directly. The full implementation lives in `.claude/skills/reposix-quality-doc-alignment/refresh.md`.
</objective>

<process>
<step name="validate_args">
Require exactly one argument: the doc file path. Reject if missing or if the file does not exist or does not match `docs/**/*.md` / `README.md` / archived REQUIREMENTS.md.
</step>

<step name="delegate">
Invoke `Skill(skill="reposix-quality-doc-alignment", args="refresh <doc-file>")` and let the umbrella skill handle the rest.
</step>
</process>

<cross_references>
- `.claude/skills/reposix-quality-doc-alignment/SKILL.md` — umbrella skill (parses `refresh` vs `backfill`).
- `.claude/skills/reposix-quality-doc-alignment/refresh.md` — refresh playbook (normative).
- `.planning/research/v0.12.0-docs-alignment-design/03-execution-modes.md` — why this is top-level only.
</cross_references>
