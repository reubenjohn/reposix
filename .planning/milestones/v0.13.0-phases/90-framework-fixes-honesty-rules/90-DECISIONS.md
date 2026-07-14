# P90 — Coordinator Decisions (owner absent; autonomous per OD-3/OD-4)

**Date:** 2026-07-04 · **Coordinator:** P90 phase coordinator (fable) ·
**Authority:** OD-3 full-autonomy mandate + OD-4 tiering
(`.planning/phases/89-framework-fixes-cadence-shell-kind/89-OWNER-DECISIONS.md`).
Gray areas resolved here per OD-3 judgment; owner is notified via the phase
close report, not blocked on.

## D90-01 — QL-001 (push-planner path-shape BLOCKER) routes to P91, not P90

The QL-001 intake entry (SURPRISES-INTAKE 2026-07-04 06:20, verified repro at
75db262) is C-class product-code work: canonical path-shape decision across 4
sites (`builder.rs` / `refresh.rs` / `fast_import.rs` / `diff.rs`), stream-parser
LF fix, non-issue path filter, and a REAL `git push` round-trip regression.
P90 is F-class (framework) and its charter explicitly produces the RAISE LIST
that seeds code phases — it does not absorb M-sized cross-crate product fixes.

**Routing decision:** P91. Rationale:
1. P91's mid-stream litmus REOPEN gate (dark-factory T2 vs TokenWorld,
   REMEDIATION-PLAN P91 SC-6) structurally cannot pass while the push planner
   misclassifies every real-tree push — QL-001 is a hard dependency of P91's
   own close gate, so it belongs inside P91's plan, not after it.
2. Waiver runway: `agent-ux/real-git-push-e2e` waiver expires 2026-07-31 and
   then hard-fails pre-push. P90 closes ~2026-07-05; P91 (≤5d) lands well
   inside the runway. Routing to P92 would thin the margin behind two REOPEN
   gates; rejected.
3. Fixing in P90 would roughly double P90's scope with cargo-heavy work while
   the framework waves are already M-sized (rejected per the +2 practice's
   anti-scope-creep arm).

**Actions taken in P90:** ROADMAP.md P91 entry amended to name QL-001 with its
six sharpened acceptance criteria; SURPRISES-INTAKE QL-001 entry annotated
ROUTED-P91; waiver left at 2026-07-31 (NOT renewed — it must die when P91
lands the fix, and its expiry is the backstop if P91 slips).

## D90-02 — Waiver cliff (12× 2026-07-26 + 5 expired docs-repro) gets a
## conscious per-waiver disposition in P90

The 2026-07-03 HIGH intake entry assigns the cliff to "the phase running when
the cliff hits (likely P90 or P91)". P90 is the quality-framework phase; the
triage is done HERE, per-waiver, from R2 inventory evidence
(90-RESEARCH-inventory.md § D). Dispositions land in the RAISE LIST
(`quality/reports/raise-list-p90.md` § Waivers) and as catalog edits where the
disposition is renew/moot. No waiver silently expires into FAIL. Landing the
underlying carry-forwards (cross-platform runners, perf re-measurement,
security e2e) stays out of P90 scope — those are P95/P97/launch-readiness
work; conscious renewal with honest `tracked_in` is the P90 deliverable.

**Refinements from R2 evidence (90-RESEARCH-inventory.md § D/H):**
(a) `release/cargo-binstall-resolves` is landable in P90 (~10 LOC) — land it,
don't renew. (b) The 3 subjective waivers are NOT mooted by dispatch wiring
(they cover runner-sweep artifact clobbering, not wiring) — renew with honest
`tracked_in`. (c) The 12 cliff waivers track a dead "v0.12.1" label — renewals
must repoint `tracked_in` at live phase homes (P95/P97/launch-readiness).
(d) The 2 security rows have DANGLING verifier scripts that collide with the
2026-07-26 expiry under the new FW-07a missing-verifier rule — the waiver
renewal must also fix or honestly re-point those script paths.

## D90-03 — `minted_at` becomes the audit-cutoff anchor (cross-AI H2 drain)

Adopt the intake sketch: write-once `minted_at` (RFC3339) on every catalog row
minted from P90 onward; `_audit_field.validate_row` anchors the
`claim_vs_assertion_audit` requirement on `minted_at` when present and falls
back to the existing `last_verified` heuristic for legacy rows (docs-alignment
dimension exemption unchanged). Validator rejects a post-P90 row that lacks
`minted_at`, closing the backdated-`last_verified` dodge for all new rows.
Legacy exemption class retires at P95 RBF-D-06 as designed.

## D90-04 — RBF-FW-07 semantics: missing-verifier demotes; env-skip
## fail-closed WITH preserved history (AMENDED post plan-check)

Two deliberately distinct behaviors, both replacing today's single
preserve-everything branch:
- **Missing verifier script** = framework-integrity failure → row flips
  NOT-VERIFIED (never preserves PASS), artifact carries a distinct
  `error: verifier-not-found` marker so a deploy glitch is distinguishable
  from a regression at a glance. (Drains cross-AI H4.)
- **Env-gated skip** (pre-release-real-backend without creds): AMENDED
  2026-07-04 after plan-check mandatory fix #2 — the original blanket
  skip-preserve would have let a future post-P91 real PASS on the P0 litmus
  row survive a cred-less milestone-close: exactly the skip-as-pass channel
  OD-2 forbids. Final semantics: on skip, current `status` flips (and
  persists) NOT-VERIFIED — fail-closed, OD-2-safe — while the prior REAL
  grade is preserved in write-history fields (`last_real_grade` +
  `last_real_verified`) and the artifact/report carries an explicit
  `skip_reason: env-missing` marker. This drains M8's actual complaint
  (silent ground-truth LOSS with no record of why) without reopening the
  OD-2 hole: honesty fail-closed, history preserved, churn explained.
  Regression tests MUST include "prior PASS + scrubbed env → NOT-VERIFIED
  with last_real_grade=PASS", not only the prior-NOT-VERIFIED case.

## D90-05 — coverage_kind enforcement split: hard for new rows, RAISE for legacy

Per F-K4a, transport/perf-claim rows need `coverage_kind: real-backend` or
explicit `WAIVED + until_date` (no PASS-with-comment). Enforcement:
- Rows minted ≥ P90 (i.e. carrying `minted_at`): hard validation at
  catalog-load time.
- Legacy P78–P88 rows: flagged into the RAISE LIST only; the migration is
  P95 RBF-D-06's chartered work. Hard-blocking legacy rows today would turn
  pre-push RED on ~every catalog before the migration phase exists — that is
  the deferral-loop the framework fixes are supposed to prevent, not create.

## D90-06 — sanctioned-target residual (cross-AI H1 residual) defers to P91

The env-gate stays a skip heuristic. The proof obligation (litmus verifier
asserts the resolved target is one of the sanctioned three and fails loud)
belongs in the litmus body P91 writes — exactly as the intake sketch proposes.
P90 records this as a named P91 acceptance criterion in the ROADMAP amendment
(same commit as D90-01's) rather than adding a second, weaker allowlist check
in `_realbackend` that would duplicate the real assertion. Intake entry
annotated ROUTED-P91.

## D90-07 — In-scope extras confirmed

Per the dispatch charter's "also in scope if planning agrees":
1. The 5 closable MISSING_TEST docs-alignment rows (cli.md ×4 +
   exit-codes-locked) get REAL tests in P90's cargo wave (they are the
   framework's own honesty debt; leaving them waived while shipping honesty
   rules would be self-exempting). The 6th (`git-checkout-branch-command`)
   stays waived — QL-001-blocked, unfixable until P91 by definition.
2. The magic-fixture hazard sweep ships as a RAISE LIST section (evidence
   from R2 § E); fixes route to the owning phases.

## D90-08 — F-K5 absorption template lives at quality/dispatch/

`quality/dispatch/absorption-honesty-spot-check.md` (sibling of
`milestone-close-verdict.md`), carrying the F-K5 meta-rule verbatim: sample
EVERY no-intake phase; spot-check author ≠ milestone orchestrator (fresh
independent subagent); rubric = "walk one critical example end-to-end
mentally — does it work?"; verifier hash-binds spot-check content.
Referenced from PRACTICES.md OP-8 and PROTOCOL.md so P96 cannot miss it.

## D90-09 — Adversarial pass artifact + verdict hook (RBF-FW-12)

Rubric at `quality/dispatch/milestone-adversarial.md` (fresh subagent reads
catalog row descriptions ONLY and grades whether the assertion would falsify
the description). Artifact at
`quality/reports/verifications/adversarial/<milestone>.json`. Milestone-close
verdict blocks GREEN if the artifact is absent for the closing milestone or
reports ≥1 failed row audit. Implementation is the minimal hook in the
existing verdict path — no new subsystem (D-CONV discipline: do not
re-fragment the just-unified framework).

## D90-10 — subagent-graded migration: dvcs-third-arm → mechanical;
## dvcs-cold-reader → wire for real

R2 evidence (90-RESEARCH-inventory.md § B) resolves both:
- `agent-ux/dvcs-third-arm`: pure shell asserts, zero subagent calls anywhere
  → **flip to `kind: mechanical`** (intent was never subjective grading).
- `subjective/dvcs-cold-reader`: nominally dispatch-wired but falls through to
  a Path-B stub — the actually-decorative row → **wire the dispatch for real
  grading** (intent WAS subjective grading; F-K4c's "wires dispatch.sh if
  intent was real grading" branch).

## D90-11 — Already-fixed intake entries close RESOLVED, not re-fixed

The `NOT_VERIFIED` underscore-typo intake entry (2026-07-03 21:35 LOW) was
already fixed by the convergence window (commit c0d5459) — P90 marks it
RESOLVED with that SHA rather than duplicating work. Any other intake entry
found already-landed gets the same treatment, with the SHA cited.

## D90-12 — Plan-check (GO-WITH-FIXES) dispositions

Plan-check verdict 2026-07-04: GO-WITH-FIXES, three mandatory items. Rulings:
1. **SC2/F-K4b ships properly in 90-02f — per-expected-assert congruence, not
   zero-global-overlap.** Each `expected.asserts` entry must map to at least
   one `asserts_passed` entry (normalized per-pair token matching); any
   unmatched expected assert blocks the PASS flip. The p86 F6 shape (9-item
   expected vs 17-item passed, heavy shared vocabulary, two asserts
   uncovered) is a REQUIRED regression fixture. Deferring SC2 to P95 was
   considered and rejected — it is a hard ROADMAP criterion and the strawman
   design would have graded PASS dishonestly, the exact failure class P90
   exists to kill. Backward-compat: check fires only when both lists are
   non-empty; conservative matching tuned against the existing mechanical
   rows (zero false RED on the current catalog is an acceptance criterion).
2. **Skip-preserve redesigned** per amended D90-04 (fail-closed NOT-VERIFIED
   + `last_real_grade`/`last_real_verified` history + `skip_reason` marker;
   applies to ALL pre-release-real-backend rows, litmus P0 included).
3. **Dispatcher paths corrected** in 90-03: the real dispatcher is
   `.claude/skills/reposix-quality-review/dispatch.sh` (+ its `lib/`);
   `quality/gates/agent-ux/dispatch.sh` does not exist and all wiring +
   verification greps must point at the skill tree.
4. (LOW) GOOD-TO-HAVES.md single-writer: 90-04 routes its candidate entries
   through 90-07's drain task; 90-02 remains the only Wave-B writer of that
   file.
