# Docs-alignment backfill playbook

> **Normative.** Followed verbatim by the umbrella `reposix-quality-doc-alignment` skill in `backfill` mode and by the P65 top-level orchestrator (per `.planning/research/v0.12.0-docs-alignment-design/06-p65-backfill-brief.md`). This playbook IS the implementation of that brief; do not re-derive from `02-architecture.md` or invent a parallel protocol.

## When this runs

- **First time:** P65 of v0.12.0 — populate the empty catalog seeded by P64.
- **Recovery:** anytime the catalog is corrupted or wholesale invalidated (manual edit, git history rewrite, etc.). Re-running is idempotent: existing rows that match the extractor's output are preserved with their certificates intact; new rows are added.

The user types `/reposix-quality-backfill` from a fresh top-level Claude session. (Cannot run from inside `gsd-executor` — depth-2 unreachable, no `Task`.)

## Inputs (the corpus)

The chunker walks these globs deterministically:

```
docs/**/*.md
README.md
.planning/milestones/v0.6.0-phases/REQUIREMENTS.md
.planning/milestones/v0.7.0-phases/REQUIREMENTS.md
.planning/milestones/v0.8.0-phases/REQUIREMENTS.md
.planning/milestones/v0.9.0-phases/REQUIREMENTS.md
.planning/milestones/v0.10.0-phases/REQUIREMENTS.md
.planning/milestones/v0.11.0-phases/REQUIREMENTS.md
```

## Protocol the orchestrator follows

```
1. Run: reposix-quality doc-alignment plan-backfill
   → MANIFEST.json at quality/reports/doc-alignment/backfill-<ts>/MANIFEST.json
   → expect ~25–35 shards (directory-affinity, ≤3 files per shard, alphabetical fallback)
   → COMMIT MANIFEST.json (catalog-first analog: the chunker output is the contract).

2. For each wave of 8 shards:
     For each shard in the wave (parallel via Task):
       prompt = read .claude/skills/reposix-quality-doc-alignment/prompts/extractor.md
                + manifest_line (paths + row-id namespace)
       spawn Task(subagent_type="general-purpose", model="haiku", prompt=prompt)
       subagent must:
         - read each cited file
         - identify behavioral claims with file:line citations
         - search tests/ for binding tests via grep
         - emit `reposix-quality doc-alignment {bind, mark-missing-test, propose-retire} ...` calls
         - never write JSON by hand — the binary computes hashes + persists state
     Wait for all 8 to complete (Task blocks until done).
     Orchestrator only sees per-shard summary stdout strings.

3. After all waves complete:
   Run: reposix-quality doc-alignment merge-shards quality/reports/doc-alignment/backfill-<ts>/
   If exit 0:
     The catalog is written. Proceed to step 4.
   If exit non-zero:
     Read CONFLICTS.md. For each conflict:
       - read each shard's cited prose
       - read the cited test
       - decide: same row with multi-source citation, or genuinely different rows?
       - edit the relevant shards/NNN.json directly
     Re-run merge-shards.
     If conflicts persist after 2 rounds: halt + checkpoint to .planning/STATE.md.
     Do NOT partial-commit the catalog.

4. Run: reposix-quality doc-alignment status
   Capture: claims_total, claims_bound, claims_missing_test, claims_retire_proposed, alignment_ratio.
   If alignment_ratio < 0.50:
     This is expected for the first backfill. Write a floor_waiver block to the catalog summary:
       until=2026-07-31
       reason="initial backfill; gap closure phased in v0.12.1"
       dimension_owner="reuben"
   The walker honors floor_waiver when present and unexpired.

5. Generate quality/reports/doc-alignment/backfill-<ts>/PUNCH-LIST.md:
   Cluster MISSING_TEST rows by user-facing surface
     ("Confluence backend parity", "JIRA epic/story shape", "ease-of-setup happy path",
      "outbound HTTP allowlist behavior", etc.).
   For each cluster: row IDs + claim text + source citation.
   List RETIRE_PROPOSED rows separately for human review.

6. Commit cadence (atomic, one per logical step):
   - MANIFEST.json
   - The full backfill-<ts>/ run dir (all shard JSONs + MERGE.md or CONFLICTS.md resolutions)
   - The populated quality/catalogs/doc-alignment.json (written by merge-shards)
   - quality/reports/doc-alignment/backfill-<ts>/PUNCH-LIST.md
   - CLAUDE.md update (P65 H3 subsection ≤40 lines)
   - quality/SURPRISES.md update (if any obstacles arose)
   - Phase-close VERDICT (after verifier subagent dispatch)

7. Dispatch P65 verifier subagent (Path A — top-level orchestrator HAS Task).
   Verdict at quality/reports/verdicts/p65/VERDICT.md.

8. After P65 GREEN:
   Dispatch milestone-close verifier for v0.12.0 (Path A).
   Verdict at quality/reports/verdicts/milestone-v0.12.0/VERDICT.md.
   If GREEN: STOP. Do not push the tag. Update STATE.md to "v0.12.0 ready-to-tag (re-verified after P64+P65); owner pushes tag."
```

## Wave dispatch details

- **Wave size:** 8 concurrent subagents. Tier limits comfortable; halve to 4 on 429 (run dir state allows resume).
- **Model tier:** Haiku for extractors. Narrow scope; prompt overhead would dominate Opus cost.
- **Inline grader role:** Each extractor verifies test alignment before calling `bind`. False BOUND is the worst failure mode — when in doubt, the extractor calls `mark-missing-test` instead. Bulk-backfill BOUNDs get re-graded automatically at the next phase close anyway.
- **Sharding:** ≤3 files per shard, directory-affinity first, alphabetical within. Re-running plan-backfill is byte-deterministic.

## Expected output envelope

After P65 ships, the catalog should look approximately:

```jsonc
{
  "summary": {
    "claims_total": 100–200,
    "claims_bound": 30–80,
    "claims_missing_test": 50–120,
    "claims_retire_proposed": 5–20,
    "claims_retired": 0,
    "alignment_ratio": 0.30–0.50,
    "floor": 0.50,
    "floor_waiver": { "until": "2026-07-31", "reason": "initial backfill; v0.12.1 gap closure", "dimension_owner": "reuben" }
  },
  "rows": [ ... ]
}
```

If actual numbers diverge wildly (e.g. 800 claims, or 5 claims) — pause and investigate before commit. Either extraction is too aggressive (granularizing every sentence) or too conservative.

## Cross-references

- Umbrella skill: `.claude/skills/reposix-quality-doc-alignment/SKILL.md`
- Refresh playbook: `.claude/skills/reposix-quality-doc-alignment/refresh.md`
- Extractor prompt: `.claude/skills/reposix-quality-doc-alignment/prompts/extractor.md`
- P65 brief (source of truth): `.planning/research/v0.12.0-docs-alignment-design/06-p65-backfill-brief.md`
- Architecture: `.planning/research/v0.12.0-docs-alignment-design/02-architecture.md`
- Execution-mode rationale: `.planning/research/v0.12.0-docs-alignment-design/03-execution-modes.md`
