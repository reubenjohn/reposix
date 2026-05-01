# Phase 83 — Plan Check

**Date:** 2026-05-01
**Reviewer:** plan-checker subagent (goal-backward verification)
**Plans audited:** `83-PLAN-OVERVIEW.md` (~899L), `83-01-PLAN.md` (~2,295L), `83-02-PLAN.md` (~1,160L)

## Verdict: **YELLOW**

The plans deliver the architecture-sketch §3 steps 4-9 contract end-to-end. D-01..D-10 ratify all six Q-A..Q-F open questions plus 4 derived decisions. The 83-01 (write fan-out core, 6 tasks) + 83-02 (fault injection, 4 tasks) split honors the ROADMAP carve-out and CLAUDE.md sequential-cargo budget. Catalog-first invariant is held. Dual-table audit contract (OP-3) is enforced end-to-end via 83-02 T04. Threat model is comprehensive (T-83-01..T-83-05).

**However, one compile-time defect (B1) will fail T02 cargo check on first attempt** — `apply_writes(... parsed: ParsedExport)` consumed-by-value plus `parsed.clone()` at the call site requires `ParsedExport` to derive `Clone`, which it currently does NOT (`#[derive(Debug, Default)]` only at `fast_import.rs:71`). The plan acknowledges the choice as "confirm during T02 read_first" — that's a soft punt on a hard precondition.

## Severity-classified issues

### BLOCKER

**B1 — `ParsedExport` lacks `Clone` derive at `fast_import.rs:71-79`.**

Plan calls `parsed.clone()` at 83-01 line 1206 + line 1688. Plan punts to "confirm during T02 read_first" (lines 1245-1248). T02 will fail cargo check on first attempt.

**Fix:** Change signature to `parsed: &ParsedExport` (matches existing `precheck` + `plan` borrow shape — preferred) OR add `Clone` to derive list as T02 prelude.

### HIGH

**H1 — Malformed `<verify><automated>` at 83-01 line 1304.** Command sequence + opening `<automated>` tag duplicated within a single block (copy-paste defect). Bash will execute concatenated, but XML structure is broken. **Fix:** Dedupe the duplicate block.

**H2 — Dimension 8 / Nyquist Check 8e: no `83-VALIDATION.md` file** (config has `nyquist_validation: true`). RESEARCH.md DOES contain `## Validation Architecture` section (line 858) with Test Framework / Phase Requirements → Test Map / Sampling Rate / Wave 0 Gaps. **Fix:** Promote that RESEARCH.md section into a separate `83-VALIDATION.md` file (5-min job).

**H3 — `helper_push_started` count divergence.** 0 in 83-01 T05 §5c (no-mirror-remote test) vs 1 in 83-02 T03 §3a (mid-stream test). Both correct per RESEARCH.md § "Audit Completeness Contract" but easy to confuse during implementation. **Fix:** Add 1-line comment in each sketch explaining the count divergence.

**H4 — 83-01 T05 §5c does not assert `helper_backend_instantiated` count == 1** even though the contract table line 647 names it. **Fix:** Add the assertion.

**H5 — `parsed.blobs` referenced post-`apply_writes` in bus_handler success+fail arms** (lines 1716-1718, 1736-1738). Works only because of the `.clone()` workaround in B1. If B1 fix removes the clone but keeps consume-by-value signature, post-call accesses become use-after-move. **B1 borrow fix resolves H5 automatically.**

### MEDIUM

**M1 — Row 2 sits at FAIL through 83-01 close.** Pre-pr cadence runner between phases would surface a phantom failure. **Fix:** Add `comment` field to row 2 catalog entry annotating the design.

**M2 — `cache_schema.sql:85-93` legacy stub `mirror_lag_partial_failure`** must be removed AND replaced with actual op `helper_push_partial_fail_mirror_lag`. **Fix:** Add `! grep -q 'mirror_lag_partial_failure' ...` to T03 verify block.

**M3 — Cross-plan `make_failing_mirror_fixture` rename risk** only doc-cited, not type-pinned. T05's grep is probably sufficient.

**M4 — `chars_out` adds `stderr_tail.len()` in failure arm** (line 1739-1741) but not success — token-cost semantics inconsistent. **Fix:** Clarify in docstring whether `chars_out` is stdout-only or stdout+stderr.

### LOW

**L1 — OVERVIEW (899L) duplicates ~30% of plan body content.** Optional trim.

**L2 — `#[cfg(unix)] mod common;` at 02-T02 §2a brittle.** Use `mod common;` + `#[cfg(unix)] use common::make_failing_mirror_fixture;`.

**L3 — RESEARCH.md `## Open Questions` (lines 816, 959) NOT marked `(RESOLVED)`.** PLAN-OVERVIEW IS the resolution venue. Optional: edit RESEARCH heading to `(RESOLVED — see 83-PLAN-OVERVIEW.md § D-01..D-10)`.

## Per-question findings (1–15) — summary

1. End-to-end goal trace — GREEN with B1 caveat.
2. 8 ROADMAP success criteria — GREEN.
3. Catalog-first invariant — GREEN.
4. `apply_writes` signature shape — GREEN.
5. Schema delta no-migration — GREEN.
6. NO `--force-with-lease` — GREEN.
7. Mirror push subprocess cwd — GREEN.
8. Both refs updated correctly — GREEN.
9. D-01..D-10 ratified — GREEN.
10. `execute_action` widening — GREEN.
11. Cargo discipline — GREEN.
12. Threat model integrity — GREEN.
13. Plan size — WARNING (LOW).
14. Cross-plan contracts — GREEN with M3.
15. `#[cfg(unix)]` gate — GREEN with L2.

## Recommendation

**YELLOW with bounded fixes.** Before T02 starts: fix B1 (signature → `&ParsedExport`), H1 (dedupe verify block), H2 (write 83-VALIDATION.md), H3+H4+H5 (test-sketch clarifications), M1-M4 (annotations + verify additions). L1-L3 optional. If B1 + H1-H5 addressed, plan goes GREEN.
