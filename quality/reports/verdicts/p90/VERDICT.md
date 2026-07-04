# P90 — framework-fixes-honesty-rules — VERIFIER VERDICT

**Verdict: GREEN**
**Graded:** 2026-07-04 · **Verifier:** unbiased phase verifier (zero session context)
**HEAD:** 5e8fed5 · **origin/main:** 76ef11b (one commit behind — see Deviation 3)
**Scope:** ROADMAP SC1–SC8 + 9 P90-minted catalog rows + 90-05 waivers + 90-06 tests

All grading is from committed artifacts. No `cargo` was run (per instruction); cargo-backed
claims (90-06) graded from committed test source + doc-alignment row states + `walk.sh`.

---

## Success-criteria grades

| SC | Grade | Evidence (file:line) |
|---|---|---|
| SC1 — pre-push runs `test-name-vs-asserts.sh`; flagged rows → RAISE §2 = baseline | **PASS** | `quality/gates/agent-ux/test-name-vs-asserts.sh` exit 0 (re-run); row `agent-ux/test-name-vs-asserts` cadences `[pre-push]`; `test-name-vs-asserts.baseline` (4 entries) ≡ `raise-list-p90.md:76-80` §2 table (4 rows) — identical, independently confirmed |
| SC2 — runner refuses PASS on `expected.asserts`↔`asserts_passed` mismatch (F-K4b, p86 F6) | **PASS** (HIGH-SCRUTINY) | Per-expected-assert congruence `_audit_field.py:151-179` (`asserts_congruent` loops every expected, `_pair_matches` :138-148 requires shared tokens in a *single* passed entry — rejects global-union strawman); F6 fixture `test_audit_field.py:297-353` (9-expected/17-passed, 2 uncovered, REDs; :342-353 proves uncovered pair have high *global* overlap → global-only impl would falsely PASS); minted_at-gated :209; both-non-empty backward-compat :169; 44 tests OK; wired at grade time `run.py:348-352` |
| SC3 — migration flips/wires subagent-graded rows | **PASS** | `agent-ux/dvcs-third-arm` `kind: mechanical`; `subjective/dvcs-cold-reader` wired for real — `dispatch.sh:70-74` execs `lib/dispatch_dvcs_cold_reader.sh` (file present) |
| SC4 — absorption template carries F-K5 verbatim + hash binding | **PASS** | `absorption-honesty-template-present.sh` exit 0: "exists, content-hash matches, all four F-K5 clauses present"; template at `quality/dispatch/absorption-honesty-spot-check.md` |
| SC5 — walking gates → committed RAISE LIST seeding P92/P94/P95 | **PASS** | `quality/reports/raise-list-p90.md` (220 lines), 5 sections all filled, every item disposition + routed phase (P91/P92/P94/P95/landed) |
| SC6 — adversarial pass in PROTOCOL; rubric file; verdict blocks GREEN on ≥1 fail | **PASS** | rubric `milestone-adversarial.md:25-45` (descriptions-only); `PROTOCOL.md:184-213`; `verdict.py:159-173` (`milestone_adversarial_gate`) darken-only, wired `main()` :386-394; 17 tests OK; smoke `verdict.py --milestone v0.13.0` → exit 1, red, names absent artifact |
| SC7 — catalog rows land first (NOT-VERIFIED) BEFORE impl; CLAUDE.md same PR | **PASS** | commit 603024e (phase's 1st commit) minted all 9 rows `status: NOT-VERIFIED`; CLAUDE.md revised in-phase 671095a (agent-ux cell + Honesty-rules paragraph + delegation-rules note) |
| SC8 — push; verifier grades GREEN; verdict written | **PASS** | origin/main 76ef11b advanced well past 29cc497; pre-push sweep (76ef11b) **49 PASS / 0 FAIL / 0 PARTIAL / 1 WAIVED** (`structure/file-size-limits`, pre-existing until 2026-08-08, unrelated) / 0 NOT-VERIFIED → exit 0 |

## Catalog-row grades (9 P90-minted rows — each verifier re-run independently)

| Row | Dim | P | Verifier re-run | Grade |
|---|---|---|---|---|
| structure/coverage-kind-required | structure | P1 | `runner-honesty-semantics.sh coverage-kind-required` exit 0 | **PASS** |
| structure/minted-at-write-once | structure | P0 | exit 0 | **PASS** |
| structure/verifier-missing-demotes | structure | P1 | exit 0 | **PASS** |
| structure/skip-fail-closed-with-history | structure | P0 | exit 0 | **PASS** |
| structure/shell-subprocess-transcript-runtime | structure | P1 | exit 0 | **PASS** |
| structure/asserts-congruence-grade-time | structure | P1 | exit 0 (82 tests OK) | **PASS** |
| agent-ux/test-name-vs-asserts | agent-ux | P1 | `test-name-vs-asserts.sh` exit 0 | **PASS** |
| agent-ux/absorption-honesty-template-present | agent-ux | P1 | `absorption-honesty-template-present.sh` exit 0 | **PASS** |
| agent-ux/milestone-adversarial-pass | agent-ux | P0 | `milestone-adversarial-pass.sh` exit 0 | **PASS** |

All three P0 rows PASS. All P1 rows PASS. No FAIL, no NOT-VERIFIED, no PARTIAL.

## 90-06 (5 MISSING_TEST rows → real tests) — PASS

All 5 rows now `BOUND`, un-waived, with real test citations (`cli.rs` ×4 helpers +
`reposix-remote/tests/exit_codes.rs`). Test bodies genuinely assert their bound claims
(no hollow/tautological assertions found); summary counters consistent
(`missing_test 6→1`, `waived 6→1`, `bound 270→275`). `docs/index/git-checkout-branch-command`
correctly **remains waived** (until 2026-07-31, QL-001-blocked → P91). Bonus honesty signal:
the phase caught and fixed a prior false-BOUND sibling (`exit-codes/cli-exit-0-success`).

## Waiver dispositions (90-05) — PASS

- No waiver `tracked_in` names dead "v0.12.1" (grep: ~30 hits, **zero in a `tracked_in` field**).
- `agent-ux/real-git-push-e2e` NOT renewed — `until: 2026-07-31`, `last_verified` not bumped → P91.
- `release/cargo-binstall-resolves` LANDED (waiver null, honest NOT-VERIFIED), not renewed.
- Security rows (`allowlist-enforcement`, `audit-immutability`) repointed `tracked_in: P92`;
  both dangling scripts now exist (6586 B / 6853 B); runway 2026-08-15.
- 5 MISSING_TEST doc-alignment rows BOUND + un-waived; `git-checkout-branch-command` stays waived.
- Every remaining active waiver's `tracked_in` names a live phase (P92/P95/P97) or the live
  v0.13.0 SURPRISES-INTAKE.

---

## Deviations found

| # | Deviation | Journaled? | Judgement |
|---|---|---|---|
| 1 | `milestone-adversarial-pass` row cadence re-pointed `[pre-release-real-backend]` → `[pre-push, pre-pr]` (commit e968d36) | **YES** — SURPRISES.md:131-135 + commit body | **Honest + justified.** Verifier is a creds-free mechanical unittest wrapper that could never honestly flip PASS under the env gate; the milestone-close GREEN-block is enforced by the `verdict.py --milestone` hook (proven by SC6 smoke exit 1), not the row cadence. Does not weaken SC6. |
| 2 | SC2 congruence gated on `minted_at` (new-regime rows only), not applied to legacy | **YES** — SURPRISES.md:126 + row comment | **Honest.** Matches D90-05's hard-new / RAISE-legacy split; the task brief explicitly sanctions judging against the gated reading. Legacy prose-assert rows correctly exempt. |
| 3 | HEAD 5e8fed5 (SURPRISES-journal commit) is **unpushed**; origin/main at 76ef11b | n/a | **Minor.** The phase-close push landed through the honest-green sweep (76ef11b); only the pivots-journal commit trails locally. Coordinator will push with the verdict commit. Not a GREEN-blocker (grading is off committed artifacts). |
| 4 | `structure/asserts-congruence-grade-time` `claim_vs_assertion_audit` (freshness-invariants.json:1084) references a **nonexistent** `quality.runners.test_run.TestAssertsCongruence` module and claims a **nonexistent** "second fixture sweeps every catalog row asserting zero false RED" | **NO** | **Flag (P2, non-blocking).** Mild irony in the honesty phase: the audit prose overstates its own test suite. The real fixture lives in `test_audit_field.py` and zero-false-RED is guaranteed structurally (minted_at gate + empty-list no-op) + demonstrated by the 76ef11b sweep. Implementation and real tests are sound. Worth a one-line correction (file to GOOD-TO-HAVES / next honesty-sweep). |
| 5 | Dead "v0.12.1" strings survive in a few non-routing `owner_hint`/`asserts` prose fields (e.g. cross-platform.json:31,68) | n/a | **Cosmetic, non-blocking.** None is a live routing pointer. Worth a one-line sweep in P95/P97. |

## GREEN gate check (refusal conditions)

- Every P0+P1 catalog row in scope PASS or WAIVED: **YES** (9/9 PASS; 3 P0 + 6 P1). ✓
- CLAUDE.md updated in-phase: **YES** (671095a). ✓
- SC2 is the per-expected-assert design with the p86 F6 fixture: **YES**. ✓

---

## FINAL VERDICT: **GREEN**

All 8 success criteria PASS. All 9 P90-minted catalog rows independently re-verified PASS
(3 P0, 6 P1; no FAIL/PARTIAL/NOT-VERIFIED). 90-06 tests genuine, rows BOUND + un-waived.
Waiver cliff consciously dispositioned with live `tracked_in` homes. Two coordinator
deviations (cadence re-point, minted_at gating) are both journaled and honest. Two minor
non-blocking flags (Deviation 4 self-overstating audit prose; Deviation 5 cosmetic v0.12.1
residue) are recorded for a follow-up one-line sweep but do not affect any P0/P1 assertion.

Phase 90 ships its honesty-rule contract honestly.
