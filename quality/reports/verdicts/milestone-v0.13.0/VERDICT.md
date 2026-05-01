# Milestone v0.13.0 — DVCS over REST — Verifier Verdict

**Verdict:** RED — tag-readiness blocked
**Verifier:** unbiased subagent (zero session context)
**Date:** 2026-05-01

## Blocker (refusal-to-grade-GREEN)

**3 v0.13.0 REQ-IDs remain `[ ]` (unchecked) in `.planning/REQUIREMENTS.md`** despite their phase shipping GREEN. Per the verifier contract: "Refuse GREEN on the milestone-close verdict if ANY v0.13.0 REQ remains `[ ]` (unchecked) without an explicit deferred-to-v0.14.0 annotation."

| REQ-ID | REQUIREMENTS.md line | Phase | P78 verdict | Trace table | Block status |
|---|---|---|---|---|---|
| HYGIENE-01 | line 49 `- [ ]` | P78 (gix bump) | GREEN | "planning" | UNCHECKED — must flip `[x]` |
| HYGIENE-02 | line 50 `- [ ]` | P78 (3 WAIVED→PASS verifiers) | GREEN | "planning" | UNCHECKED — must flip `[x]` |
| MULTI-SOURCE-WATCH-01 | line 113 `- [ ]` | P78-03 (walker schema migration) | GREEN | absent (carry-forward) | UNCHECKED — must flip `[x]` |

Secondary trace-table inconsistencies (not refusal blockers, but cosmetic-debt): 6 additional REQ-IDs (DVCS-ATTACH-01..04, DVCS-SURPRISES-01, DVCS-GOOD-TO-HAVES-01) carry `[x]` checkboxes but the traceability table at line 137+ still shows them as `planning` instead of `shipped`. Not a refusal trigger because the checkboxes themselves are flipped.

**Root cause hypothesis.** P78 close (commit 791f7b9 / b9b0c61 / dd3c801 sequence) flipped doc-alignment catalog row to PASS but did not flip the REQUIREMENTS.md checkboxes. P88's commit `770b570` ("phase-close(P88): v0.13.0 milestone-close cursor + REQ checkboxes + SUMMARY") title implies REQ checkbox sweep, but the sweep missed the 3 P78 REQs (the ones discovered as unchecked were P88-era REQs).

**Resolution.** Owner edits `.planning/REQUIREMENTS.md`:
- line 49: `- [ ]` → `- [x]` for HYGIENE-01 (cite P78 verdict at `quality/reports/verdicts/p78/VERDICT.md`).
- line 50: `- [ ]` → `- [x]` for HYGIENE-02 (3 catalog rows PASS confirmed by P78 verdict).
- line 113: `- [ ]` → `- [x]` for MULTI-SOURCE-WATCH-01 (P78-03 walk_multi_source_non_first_drift_fires_stale test passing).

After flip + re-verify, milestone verdict can flip to GREEN.

## Per-phase verdict cross-check

| Phase | Verdict | Notes |
|---|---|---|
| P78 | GREEN | gix bump + 3 WAIVED verifiers + MULTI-SOURCE-WATCH-01 schema migration |
| P79 | GREEN (1 advisory) | `reposix attach` core + POC; 4 DVCS-ATTACH REQs |
| P80 | GREEN (2 advisory) | mirror-lag refs `refs/mirrors/<sot-host>-{head,synced-at}` |
| P81 | GREEN (3 advisory) | L1 perf migration; `reposix sync --reconcile` |
| P82 | GREEN (1 advisory) | bus URL parser + 2 prechecks; capability branching |
| P83 | GREEN | bus write fan-out (SoT-first + mirror-best-effort + fault injection) |
| P84 | GREEN | webhook-driven mirror sync workflow + race + first-run + latency artifact |
| P85 | GREEN | DVCS docs (topology + setup guide + troubleshooting matrix) |
| P86 | GREEN | dark-factory third-arm scenario (17 asserts) |
| P87 | GREEN | +2 slot 1 (5 SURPRISES entries terminal; honesty spot-check sampled 5 phases) |
| P88 | GREEN | +2 slot 2 (1 GOOD-TO-HAVES entry terminal DEFERRED; 4 milestone-close rows PASS) |

All 11 phases GREEN. No phase RED. Advisory items are non-blocking notes per phase verdict authors.

## Catalog state

- `python3 quality/runners/run.py --cadence pre-push` → **26 PASS / 0 FAIL / 0 PARTIAL / 0 WAIVED / 0 NOT-VERIFIED → exit=0**.
- `quality/catalogs/freshness-invariants.json` → 18 rows; **0 WAIVED rows** (every `waiver` field literally `null` per grep audit).
- 4 P88 milestone-close rows in `agent-ux.json` all PASS (last_verified 2026-05-01T22:30:00Z).

## REQ-coverage map (36 v0.13.0 REQ-IDs)

- **31 `[x]` shipped**: 2 carry-forward (DVCS-SURPRISES-01, DVCS-GOOD-TO-HAVES-01) + 29 v0.13.0-scope shipped.
- **1 `[◐]` rubric-pending-owner**: DVCS-DOCS-04 (cold-reader rubric registered; owner runs `/reposix-quality-review --rubric dvcs-cold-reader`).
- **1 `[2]` malformed token**: minor REQUIREMENTS.md formatting noise (not a blocker; cosmetic).
- **3 `[ ]` UNCHECKED — BLOCKERS**: HYGIENE-01, HYGIENE-02, MULTI-SOURCE-WATCH-01 (see Blocker section).

## Dark-factory three-arm

`bash quality/gates/agent-ux/dark-factory.sh` → exit 0. Sim arm exercised; "DARK-FACTORY DEMO COMPLETE -- sim backend: agent UX is pure git" emitted. Catalog row `agent-ux/dvcs-third-arm` PASS (P86 verdict GREEN; 17 asserts; last_verified 2026-05-01T21:43:24Z).

## +2 reservation operational

- **Slot 1 (P87, surprises absorption)**: SURPRISES-INTAKE.md drained — 5 terminal STATUS entries (2 RESOLVED-on-discovery via OP-8 eager-resolution; 3 P87-drained: Entry 1 RESOLVED via P86 cross-reference, Entry 3 WONTFIX schedule-shift, Entry 5 DEFERRED to v0.13.x). Honesty spot-check sampled 5 phases (exceeds ≥3 floor).
- **Slot 2 (P88, good-to-haves polish)**: GOOD-TO-HAVES.md drained — entry-01 terminal STATUS DEFERRED to v0.14.0 with full rationale. Pure-docs envelope of P88 doesn't fit Rust+tests+schema scope.

Both slots executed in correct order (P87 → P88). Practice operational and producing terminal signal.

## Tag-readiness statement

**Currently NOT tag-ready.** After owner flips the 3 unchecked REQ checkboxes (HYGIENE-01, HYGIENE-02, MULTI-SOURCE-WATCH-01) and the milestone verdict re-verifies to GREEN, owner runs:

```
bash .planning/milestones/v0.13.0-phases/tag-v0.13.0.sh
```

Tag-script enforces 8 guards (clean tree, on main, version match, CHANGELOG entry, `cargo test --workspace`, pre-push runner GREEN, P88 verdict GREEN, milestone-v0.13.0 verdict GREEN). On OK:

```
git push origin v0.13.0
```

Orchestrator does NOT push the tag (ROADMAP P88 SC6 — STOP at tag boundary).

---

_Method: 11 phase-verdict file inspection, pre-push runner full execution, REQUIREMENTS.md awk-section scan, freshness-invariants waiver grep, dark-factory.sh execution, P88 verifier shell × 4 execution, tag-script guard count + executable-bit audit. Zero session context inherited._
