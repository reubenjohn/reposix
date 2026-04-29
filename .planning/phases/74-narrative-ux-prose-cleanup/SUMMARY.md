---
phase: 74-narrative-ux-prose-cleanup
plan: 01
subsystem: docs-alignment / narrative + UX
tags: [narrative-retire, ux-bind, prose-fix, docs-alignment, quality-gates, v0.12.1]
provides:
  - quality/gates/docs-alignment/{install-snippet-shape,audit-trail-git-log,three-backends-tested,connector-matrix-on-landing,cli-spaces-smoke}.sh (5 hash-shape verifiers, FLAT placement)
  - scripts/p74-bind-ux-rows.sh (bind sweep promoted from heredoc per CLAUDE.md ad-hoc-bash rule)
  - 5 catalog rows transitioned MISSING_TEST -> BOUND (UX-BIND-01..05)
  - 4 catalog rows transitioned MISSING_TEST -> RETIRE_PROPOSED (NARRATIVE-RETIRE-01..04, identical-format rationale per D-09)
  - quality/reports/verdicts/p74/{status-before,status-after,summary-before,summary-after}.{txt,json}
affects:
  - docs/social/linkedin.md:21 (PROSE-FIX-01: dropped FUSE filesystem framing -> git-native partial clone)
  - quality/catalogs/doc-alignment.json (9 row state transitions + 1 STALE_DOCS_DRIFT side-effect on linkedin row)
  - CLAUDE.md (P74 H3 under v0.12.1 — in flight, D-11)
  - .planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md (2 LOW entries)
metrics:
  duration: ~45min wall-clock
  completed_date: 2026-04-29
  commits: 12
  catalog_rows_transitioned: 9 (+ 1 unintended STALE_DOCS_DRIFT)
  alignment_ratio_delta: "+0.0112 (0.9050 -> 0.9162)"
  claims_missing_test_delta: "-9 (9 -> 0)"
  claims_retire_proposed_delta: "+4 (23 -> 27)"
---

# Phase 74 Plan 01: Narrative cleanup + UX bindings + linkedin prose

Closed the remaining 9 `MISSING_TEST` rows: 5 UX claims bound to tiny hash-shape shell verifiers under `quality/gates/docs-alignment/` (FLAT placement, mirroring P73's `jira-adapter-shipped.sh` ground truth), 4 narrative rows transitioned to `RETIRE_PROPOSED` with identical-format rationale (owner-TTY confirms in HANDOVER step 1). Also dropped the v0.4-era "FUSE filesystem" framing from `docs/social/linkedin.md:21`, re-anchoring the prose to the v0.9.0 git-native partial clone architecture.

After P72 + P73 + P74, `claims_missing_test` is **0** within the docs-alignment dimension — the autonomous-run cluster's primary closure target hit.

## Completed tasks (12 atomic commits)

| #  | Task                                                          | Commit    | Requirement          |
| -- | ------------------------------------------------------------- | --------- | -------------------- |
| 1  | Capture BEFORE snapshot (catalog-first, status pre-mutation)  | a973063   | (audit)              |
| 2  | Scaffold 5 verifier stubs (FLAT placement)                    | 21030a0   | UX-BIND-01..05       |
| 3  | Implement install-snippet-shape.sh                            | b090049   | UX-BIND-01           |
| 4  | Implement audit-trail-git-log.sh (SIGPIPE eager-fix)          | dd89abd   | UX-BIND-02           |
| 5  | Implement three-backends-tested.sh                            | 7a2161d   | UX-BIND-03           |
| 6  | Implement connector-matrix-on-landing.sh (synonym widen)      | c8e4111   | UX-BIND-04           |
| 7  | Implement cli-spaces-smoke.sh                                 | 11311a7   | UX-BIND-05           |
| 8  | propose-retire 4 narrative rows (identical-format rationale)  | 97f071a   | NARRATIVE-RETIRE-01..04 |
| 9  | Drop FUSE framing from linkedin.md:21                         | 54f6bad   | PROSE-FIX-01         |
| 10 | Bind 5 UX rows to hash-shape verifiers                        | efc75ab   | UX-BIND-01..05       |
| 11 | walk + AFTER snapshot + SURPRISES-INTAKE entries              | 17cdc76   | UX-BIND-01..05       |
| 12 | CLAUDE.md H3 subsection (≤30 lines per D-11)                  | 6cae44d   | (D-11)               |

## Catalog deltas

| Metric                  | BEFORE | AFTER  | Delta   |
| ----------------------- | ------ | ------ | ------- |
| `alignment_ratio`       | 0.9050 | 0.9162 | +0.0112 |
| `claims_missing_test`   | 9      | 0      | -9      |
| `claims_bound`          | 324    | 328    | +4 (5 binds, -1 from linkedin STALE_DOCS_DRIFT) |
| `claims_retire_proposed`| 23     | 27     | +4      |
| `claims_retired`        | 30     | 30     | 0       |

## Row transitions (10 actions)

**5 MISSING_TEST -> BOUND:**
1. `docs/index/5-line-install` -> install-snippet-shape.sh
2. `docs/index/audit-trail-git-log` -> audit-trail-git-log.sh
3. `docs/index/tested-three-backends` -> three-backends-tested.sh
4. `planning-milestones-v0-11-0-phases-REQUIREMENTS-md/polish2-06-landing` -> connector-matrix-on-landing.sh
5. `planning-milestones-v0-8-0-phases-REQUIREMENTS-md/spaces-01` -> cli-spaces-smoke.sh

**4 MISSING_TEST -> RETIRE_PROPOSED** (D-09 identical rationale; owner-TTY confirms via HANDOVER step 1):
6. `use-case-20-percent-rest-mcp`
7. `use-case-80-percent-routine-ops`
8. `mcp-fixture-synthesized-not-live`
9. `mcp-schema-discovery-100k-tokens`

**1 BOUND -> STALE_DOCS_DRIFT** (unintended side-effect — confirms P75 walker bug for `Source::Single`):
- `docs/social/linkedin/token-reduction-92pct` (linkedin.md:21) — prose edit drove source_hash drift; walker did not auto-rebind. Logged to SURPRISES-INTAKE; P75 fixes the walker bug, then a single `walk` will heal this row.

## Eager-fix decisions (in-phase, D-09 / OP-8)

1. **connector-matrix regex widened** — the live heading reads "What each backend can do," a synonym for "connector capability matrix." Verifier regex broadened to `[Cc]onnector|[Bb]ackend`. < 5 min, single file. Heading rename to "Connector capability matrix" filed as future GOOD-TO-HAVE (see P77 backlog).
2. **audit-trail-git-log SIGPIPE fix** — initial impl `git log | head -1` exited 141 under `set -o pipefail`. Switched to `git log --oneline -n 1` (no pipe). < 2 min, single file.
3. **bind sweep promoted to scripts/p74-bind-ux-rows.sh** — the heredoc was 1287 chars and the `deny-ad-hoc-bash` hook blocked it. Promoted to a committed script per CLAUDE.md OP-4 ("Ad-hoc bash is a missing-tool signal"). The script is one-shot, idempotent, and documents the bind invocations.

## SURPRISES-INTAKE entries (2 LOW)

1. **2026-04-29 20:55 / LOW** — Linkedin row didn't auto-heal STALE_DOCS_DRIFT after the prose edit + 2 walks. Confirms the walker hash-overwrite bug also affects `Source::Single` rows (not just `Source::Multi` as previously suspected). P75's regression test must cover both cases.
2. **2026-04-29 20:56 / LOW** — connector-matrix-on-landing regex synonym widening leaves the `[Bb]ackend` heading in place; renaming the section to "Connector capability matrix" is a P77 GOOD-TO-HAVE, not a P76 surprise.

## GOOD-TO-HAVES entries

None added in P74. The connector-matrix heading rename (above) is filed under SURPRISES-INTAKE entry #2 with a P77-pointer note rather than as a separate GOOD-TO-HAVES entry — the executor's judgment was that the synonym widening fully closes the verifier contract; the heading rename is purely cosmetic.

## Plan-vs-source deviations

1. CONTEXT.md D-01 says verifier home is `quality/gates/docs-alignment/verifiers/`; the actual P73-established convention is FLAT (`quality/gates/docs-alignment/jira-adapter-shipped.sh`). Plan + executor adopted FLAT to match ground truth.
2. CONTEXT.md "Specifics" mentions `refresh docs/index.md`; the actual verb is `walk` (no per-doc filter). Plan + executor used `walk` per P73 SUMMARY's deviation table.
3. (Pre-commit guard side-effect) The `deny-ad-hoc-bash` hook caught the bind sweep heredoc; resolution was to promote to `scripts/p74-bind-ux-rows.sh` rather than `--no-verify`. This is the CLAUDE.md OP-4 contract working as designed.

## Pre-commit status

All 12 commits passed pre-commit hooks (no `--no-verify`). `bash scripts/banned-words-lint.sh` passes against the CLAUDE.md P74 H3 (28 body lines, ≤30 per D-11).

## Top-level orchestrator handoff

1. SUMMARY.md (this file) landed by orchestrator (executor's Write was blocked by a global guard).
2. **Dispatch `gsd-verifier` (Path A via `Task` tool)** with the QG-06 prompt template, N=74. Verifier writes `quality/reports/verdicts/p74/VERDICT.md`. Phase does NOT close until graded GREEN.
3. **DO NOT run `confirm-retire`** — owner-TTY only (HANDOVER step 1 covers the bulk-confirm, including the 4 NEW propose-retires landed here).

## Next phase

P75 (bind-verb hash-overwrite fix) is the natural follow-up — its regression test must cover the `Source::Single` case the linkedin row exposed during P74's walk.
