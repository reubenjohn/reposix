# 79-03 Plan Summary — `reposix attach` tests + idempotency + close

> Reconstructed post-hoc 2026-05-01 by orchestrator after the OS crash that interrupted the original 79-03 executor's SUMMARY write. Source of truth: the three commits below + the verifier's GREEN verdict at `quality/reports/verdicts/p79/VERDICT.md`.

## Tasks shipped (3/3)

**T01 — Reconciliation integration tests (5 cases):** `crates/reposix-cli/tests/attach.rs` exercises every row in `architecture-sketch.md` § "Reconciliation cases":
- match
- backend-record-deleted (warn + skip + `--orphan-policy={delete-local,fork-as-new,abort}`)
- no-id-frontmatter (warn + skip)
- duplicate-id (hard error)
- mirror-lag (cache marks for next fetch)

**T02 — Idempotency + reject + Tainted-materialization + audit-row tests:**
- `re_attach_same_sot_is_idempotent` (Q1.3) — `crates/reposix-cli/tests/attach.rs:604`
- `re_attach_different_sot_is_rejected` (Q1.2) — `crates/reposix-cli/tests/attach.rs:678`
- `attach_then_read_blob_returns_tainted` (DVCS-ATTACH-04 reframed, runtime path) — `crates/reposix-cli/tests/attach.rs:749`
- `attach_audit_log_records_walk_event` (OP-3 audit hook test) — `crates/reposix-cli/tests/attach.rs:802`
- Type-pin assertion (`_is_tainted(_: Tainted<Vec<u8>>)`) — `crates/reposix-cli/tests/attach.rs:789-791`

**T03 — CLAUDE.md + push + catalog flip:**
- CLAUDE.md § Commands documents `reposix attach`
- CLAUDE.md describes `cache_reconciliation` table
- Catalog row `agent-ux/reposix-attach-against-vanilla-clone` flipped FAIL → PASS in `quality/catalogs/agent-ux.json`
- Pushed to origin/main

## Commits

| SHA | Subject |
|---|---|
| `a558d4a` | fix(cache,cli): idempotent `Cache::open` + `build_from` + `REPOSIX_SIM_ORIGIN` env override (P79-03 fix-forward) |
| `791f7b9` | test(cli): integration tests for `reposix attach` (DVCS-ATTACH-01..04 + OP-3) |
| `dd3c801` | docs(claude.md,quality): document `reposix attach` + flip catalog row to PASS (P79-03 close) |

## Acceptance

- All 5 REQ-IDs (POC-01, DVCS-ATTACH-01..04) shipped and verified GREEN by unbiased subagent. See `quality/reports/verdicts/p79/VERDICT.md`.
- Catalog row at PASS; `python3 quality/runners/run.py --cadence pre-push` shows 26 PASS / 0 FAIL / 0 WAIVED at phase close.
- CLAUDE.md updated in-phase (per QG-07 carry-forward).

## Deviations / SURPRISES

- One fix-forward commit (`a558d4a`) was needed to make `Cache::open` and `Cache::build_from` idempotent — required for Q1.3 idempotent re-attach. Plus `REPOSIX_SIM_ORIGIN` env override added to let integration tests point the attach path at a custom sim origin.
- No SURPRISES-INTAKE entries written — all observed items were eager-resolved within the plan (per OP-8) or absorbed from POC-FINDINGS at task time.

## Phase-close artifacts

- Verifier verdict: `quality/reports/verdicts/p79/VERDICT.md` (GREEN with 3 non-blocking advisories — this SUMMARY closes advisory #1)
- POC artifacts: `research/v0.13.0-dvcs/poc/` (kept; throwaway code per POC contract — owner may delete in P88 milestone close if desired)

## Cross-references

- Phase plan: `.planning/phases/79-poc-reposix-attach-core/79-03-PLAN.md`
- POC findings: `research/v0.13.0-dvcs/poc/POC-FINDINGS.md` (5 INFO + 2 REVISE absorbed at 79-02 task time per orchestrator decision)
- Architecture sketch: `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § "Reconciliation cases" + Q1.1/Q1.2/Q1.3
- Reframed DVCS-ATTACH-04 (`Cache::read_blob` lazy seam): `.planning/REQUIREMENTS.md` § "`reposix attach` core" — see 2026-05-01 reframe note
