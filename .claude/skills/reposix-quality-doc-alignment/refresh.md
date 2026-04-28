# Docs-alignment refresh playbook

> **Normative.** Followed verbatim by the umbrella `reposix-quality-doc-alignment` skill in `refresh` mode. Source of truth for single-doc refresh; does not duplicate runtime detail covered in `quality/PROTOCOL.md` or the briefs at `.planning/research/v0.12.0-docs-alignment-design/`.

## When this runs

A pre-push or pre-pr cadence reported one of:

- `STALE_DOCS_DRIFT` on a specific doc — the cited line range hash changed; rows pointing at this doc need re-grading.
- `STALE_TEST_GONE` on a row whose source happens to be in the same doc — test path/symbol no longer resolves.
- `TEST_MISALIGNED` — a previous grader pass said the test no longer asserts the claim.

The user types `/reposix-quality-refresh <doc-file>` from a fresh top-level Claude session. (Cannot run from inside `gsd-executor` — depth-2 unreachable, no `Task`.)

## Protocol the orchestrator follows

```
1. Run: reposix-quality doc-alignment plan-refresh <doc-file>
   → produces stale-row manifest as JSON on stdout: list of row IDs whose source_hash drifted
     OR whose test_body_hash drifted OR whose state is STALE_TEST_GONE / TEST_MISALIGNED
   → if zero rows: exit 0 silently — pre-push misfire, nothing to do.

2. For each stale row in the manifest:
     prompt = read .claude/skills/reposix-quality-doc-alignment/prompts/grader.md
              + row context (claim text, source citation, test citation, prior verdict, prior rationale)
     spawn Task(subagent_type="general-purpose", model="opus", prompt=prompt)
     subagent must:
       - read the cited prose (file:line range)
       - read the cited test fn body
       - decide: still BOUND? RETIRE_PROPOSED? TEST_MISALIGNED? MISSING_TEST?
       - emit `reposix-quality doc-alignment <subcmd>` calls — never write JSON directly
     The orchestrator dispatches stale rows in parallel (up to ~4 at a time; refresh is small).

3. Run: reposix-quality doc-alignment walk
   → recompute hashes, grade row states, exit 0 iff no blocking states remain
   → if non-zero: surface stderr to the user, do NOT auto-resolve

4. If walk exits 0:
     git add quality/catalogs/doc-alignment.json
     git commit -m "refresh(doc-alignment): re-grade {N} stale rows in <doc-file>"
   If walk still non-zero:
     halt with the structured stderr from the walker; the user decides next steps
     (often: fix a broken test, then re-invoke the slash command).
```

## Why the grader is Opus, not Haiku

Single-doc refresh is rare and high-stakes — the grader is making a judgment call ("does this test still assert this claim?") on a row that already had a binding. False BOUND is the worst failure mode (a regression sneaks through because the catalog says "tested"). Opus pays for itself.

Backfill (P65) uses Haiku for extractors because (a) the catalog is initially empty so first-pass false BOUND is recoverable, and (b) volume.

## What this playbook explicitly does not do

- Does NOT touch any other doc's rows.
- Does NOT auto-confirm `RETIRE_PROPOSED`. Retirement requires `confirm-retire` which is `$CLAUDE_AGENT_CONTEXT`-guarded.
- Does NOT update stored hashes mechanically. Hashes update only when `bind` is called with a fresh GREEN grade.
- Does NOT modify implementation code. If the right fix is to write a missing test, the user does that as a separate task; the row stays MISSING_TEST until the test exists and a subsequent refresh binds it.

## Cross-references

- Umbrella skill: `.claude/skills/reposix-quality-doc-alignment/SKILL.md`
- Backfill playbook: `.claude/skills/reposix-quality-doc-alignment/backfill.md`
- Grader prompt: `.claude/skills/reposix-quality-doc-alignment/prompts/grader.md`
- Architecture: `.planning/research/v0.12.0-docs-alignment-design/02-architecture.md`
- Hash semantics: same doc § "Hash semantics".
