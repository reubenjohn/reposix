---
name: reposix-quality-review
description: "Run subjective rubric checks (cold-reader hero clarity, install positioning, headline-numbers sanity) by dispatching one unbiased subagent per stale catalog row in parallel. Reads quality/catalogs/subjective-rubrics.json, persists JSON artifacts to quality/reports/verifications/subjective/<rubric-id>.json. The next runner sweep re-grades the catalog rows. Pre-approved for v0.12.0 P61 per .planning/research/v0.12.0/open-questions-and-deferrals.md line 124."
argument-hint: "[--rubric <rubric-id>] [--all-stale] [--force]"
allowed-tools:
  - Bash
  - Read
  - Task
---

<objective>
Run subjective rubric checks against catalog rows that are stale or unverified. Each rubric is implemented as a prompt + source-file selection; an unbiased subagent grades the rubric and emits a numeric score + verdict + rationale + evidence files. The skill persists the verdict as JSON to `quality/reports/verifications/subjective/<rubric-id>.json`. The next quality runner sweep reads the artifact and updates the catalog row's status (PASS/FAIL/PARTIAL based on score-vs-threshold).

Default invocation modes:
- `--rubric <id>` -- dispatch one rubric (used by `quality/runners/run.py` via the catalog row's `verifier.script` field).
- `--all-stale` -- dispatch every rubric whose row `is_stale` OR `last_verified=null` (parallel per OP-2).
- `--force` -- dispatch every rubric regardless of freshness (manual on-demand mode).
- (no args) -- print usage + the 3 seed rubrics with their stale status.

Cross-references:
- Catalog: `quality/catalogs/subjective-rubrics.json`
- Dimension home: `quality/gates/subjective/README.md`
- Rubric prompts: `.claude/skills/reposix-quality-review/rubrics/<id>.md`
- Cold-reader rubric integrates `$HOME/.claude/skills/doc-clarity-review/SKILL.md` (Wave D).
</objective>

<process>

<step name="parse_args">
Parse `$ARGUMENTS`:
1. If first token is `--rubric`, consume the next token as the rubric ID (e.g. `subjective/cold-reader-hero-clarity`). Set `MODE=single`.
2. If first token is `--all-stale`, set `MODE=all-stale`.
3. If first token is `--force`, set `MODE=force`.
4. Otherwise: print usage + the 3 seed rubrics; exit 0.

Reject unknown rubric IDs with a clear error naming the 3 valid IDs.
</step>

<step name="load_rubric">
Read `quality/catalogs/subjective-rubrics.json` via `lib/catalog.py:load_subjective_catalog`.

For `--rubric` mode: find the row by id; load `.claude/skills/reposix-quality-review/rubrics/<slug>.md` as the rubric prompt body; expand the row's `sources` field into a concrete file list.

For `--all-stale`: filter the catalog to rows where `is_stale(row, now)` returns True OR `last_verified` is null. Build a list of `(rubric-id, prompt-path, sources)` tuples.

For `--force`: include ALL rows regardless of freshness.
</step>

<step name="dispatch_subagent">
For each rubric in scope:

1. Construct the subagent prompt: "You have ZERO session context. You are an unbiased reviewer. Read these files: <list>. Apply this rubric: <prompt body from rubrics/<slug>.md>. Output JSON with shape `{score: int 1-10, verdict: 'CLEAR'|'NEEDS-WORK'|'CONFUSING', rationale: str, evidence_files: [str]}`."

2. **Path A (preferred)** -- if the Task tool is available in the calling Claude session: dispatch via `Task({subagent_type: 'general-purpose', prompt: <prompt>})`. For `--all-stale` + `--force` modes, dispatch all rubrics IN PARALLEL (one Task per rubric in a single tool-call block; the harness handles concurrency). This is the OP-2 case: rubrics are independent; subagents share no state.

3. **Path B (fallback)** -- if Task is not available (the runner subprocess invocation typically lacks Task): emit a stub artifact (`score=0, verdict=NOT-IMPLEMENTED, dispatched_via=Path-B-runner-subprocess`). The runner re-grade will mark the row FAIL; Wave G's full-skill dispatch (with Task tool) re-runs and produces the real artifact.

4. **Special case (cold-reader rubric)** -- the cold-reader rubric's implementation is the existing `doc-clarity-review` global skill. Instead of constructing a custom subagent prompt, invoke `/doc-clarity-review --prompt <cold-reader-prompt> README.md docs/index.md`. Parse the `_feedback.md` output for the verdict + score (CLEAR=10, NEEDS-WORK=5, CONFUSING=2).

Capture each subagent's JSON output for the persist step.
</step>

<step name="persist_artifact">
For each rubric: write `quality/reports/verifications/subjective/<slug>.json` via `lib/persist_artifact.py:persist_artifact`. Canonical shape:

```json
{
  "ts": "<RFC3339-UTC>",
  "rubric_id": "subjective/<slug>",
  "score": 7,
  "verdict": "CLEAR",
  "rationale": "<one paragraph>",
  "evidence_files": ["README.md:1-50", "docs/index.md:1-50"],
  "dispatched_via": "Path A subagent" | "Path B in-session" | "doc-clarity-review",
  "asserts_passed": ["..."],
  "asserts_failed": [],
  "stale": false
}
```

The skill computes `asserts_passed` + `asserts_failed` from the rubric's `expected.asserts` field (catalog row).

The skill does NOT write the catalog row's `status` directly. The next quality runner sweep reads the artifact and updates `status` per `compute_exit_code` semantics. Single-writer invariant matches `quality/runners/run.py:run_row`.
</step>

<step name="exit_code">
For `--rubric` mode: exit 0 if `score >= 7` (PASS), exit 2 if `4 <= score <= 6` (PARTIAL), exit 1 if `score < 4` (FAIL). The runner's `run_row` maps these to PASS/PARTIAL/FAIL via the existing exit-code mapping (see `quality/runners/run.py:295-302`).

For `--all-stale` + `--force`: print a summary table (rubric | score | verdict). Exit 0 iff every rubric exited 0; exit 1 otherwise.
</step>

</process>

<success_criteria>
- [ ] `--rubric` mode dispatches one rubric and persists one artifact at `quality/reports/verifications/subjective/<slug>.json`.
- [ ] `--all-stale` mode dispatches every stale rubric in parallel and persists artifacts.
- [ ] `--force` mode dispatches all rubrics regardless of freshness.
- [ ] Each artifact has the canonical JSON shape (`ts, rubric_id, score, verdict, rationale, evidence_files, dispatched_via, asserts_passed, asserts_failed, stale`).
- [ ] Path A vs Path B fallback works (Task tool present vs absent).
- [ ] `doc-clarity-review` integration works for the cold-reader rubric (Wave D fills in).
- [ ] Exit codes match the catalog row's status mapping (`>=7` PASS / `4-6` PARTIAL / `<4` FAIL).
- [ ] Banned-words clean (no forbidden synonym for `migrate`).
- [ ] No skill outside `.claude/skills/reposix-quality-review/*` is modified (skill-scope guard).
</success_criteria>

<cross_references>
- `quality/catalogs/subjective-rubrics.json` -- 3-row catalog (P61 Wave A locked the contract).
- `quality/gates/subjective/README.md` -- dimension home; rubric table + conventions.
- `quality/runners/run.py` -- runner contract (`is_stale`, `run_row`, `compute_exit_code`).
- `quality/PROTOCOL.md` -- runtime contract.
- `$HOME/.claude/skills/doc-clarity-review/SKILL.md` -- the existing skill that implements the cold-reader rubric.
- `.planning/research/v0.12.0/open-questions-and-deferrals.md` line 124 -- pre-approval scope for this skill.
</cross_references>
