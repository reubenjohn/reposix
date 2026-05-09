# v0.13.1 Synthesis Verification — read-only audit

**Auditor:** unbiased verification subagent (zero session context)
**Date:** 2026-05-08
**Inputs sampled:** `03-synthesis/{PATTERNS,REMEDIATION-PLAN,STRATEGIC-REFRAME,COMPLETENESS-CHECK}.md`; `02-phase-audits-may08/phase-audit-p7{8,9},p8{0..8}.md` + `vision-audit.md`; `01-dark-factory-may02/SUMMARY.md`; live tree at `/home/reuben/workspace/reposix/`; `git tag`, `gh release list`, `git log`.

## One-paragraph verdict

**The four synthesis docs are trustworthy as input to v0.13.1 roadmapping.** Every load-bearing factual claim I sampled — the "v0.13.0 tag has not been pushed" claim, the production phase-ID-leak literal at `attach.rs:147-166`, the `sync.rs:79-92` parallel literal, the `with_audit` omission across all three real-backend `instantiate_*` paths, the absence of `quality-pre-pr.yml` workflow, the `dark_factory_real_confluence` test stopping at URL-shape, the `webhook-latency.json` `n=1, method: synthetic-dispatch` stub, the `subjective/dvcs-cold-reader` row's `NOT_VERIFIED` status, the `bus_precheck_b.rs:262-273` test asserting the OPPOSITE of what the verdict cited, the `fd2e247` commit's existence as a post-milestone GOOD-TO-HAVES-01 closure, and the `quality-pre-pr.yml` not existing — all CONFIRMED against live artifacts. Three minor citation drifts surfaced (the `architecture-sketch/index.md` lookup table is actually in `REQUIREMENTS.md:130-133`; the "23 minutes after milestone close" arithmetic is closer to 17 minutes; "51 HIGH" is the exact total but only 44 H-row entries appear in the inventory tables — H-K1/K2/K3 are dependency pointers not findings). One nuance critique against COMPLETENESS S2: REMEDIATION-PLAN P91/P92/P93 SCs already cite per-phase dark-factory T-N runs; the S2 claim that "P96 is the ONLY firing" is overstated, though the load-bearing point (no formal "REOPEN this phase if T-N fails" gate) holds. None of the drifts undermine the substantive findings or the v0.13.1 phase shape.

---

## Findings table

| Doc | § | Claim | Result | Evidence |
|---|---|---|---|---|
| STRATEGIC | Q1 | "v0.13.0 tag has not been pushed" | CONFIRMED | `git tag -l 'v0.13*'` returns empty; `gh release view v0.13.0` → "release not found"; `gh release list` newest is `v0.12.0` (2026-04-30) and `reposix-*-v0.12.0` per-package (2026-05-01); `CHANGELOG.md:9` says "Release status: PENDING owner tag-cut" |
| PATTERNS | C3 | `attach.rs:147-166` rejects real backends with "P79-02 scaffold (sim only); github/confluence/jira land alongside the integration tests in P79-03" | CONFIRMED | `crates/reposix-cli/src/attach.rs:147-166` byte-for-byte matches |
| PATTERNS | C3 | `sync.rs:79-92` carries parallel "P82+" leak | CONFIRMED | `crates/reposix-cli/src/sync.rs:79-92` byte-for-byte matches; literal `"sync --reconcile: backend ... not yet wired in v0.13.0 (sim only); ... lands alongside the bus-remote work in P82+"` |
| PATTERNS | C3 visibility | "`architecture-sketch/index.md`'s lookup table marks DVCS-ATTACH-01..04 'shipped' without a sim-only qualifier" | DRIFTED | The "shipped" lookup table is at `.planning/milestones/v0.13.0-phases/REQUIREMENTS.md:130-133` (not `architecture-sketch/index.md`); a `grep -rn "DVCS-ATTACH" .planning/research/v0.13.0-dvcs/` returns zero hits. The substantive claim (rows marked "shipped" without qualifier) is correct; the file-path citation is wrong |
| PATTERNS | C5 | OP-3 silent on every helper push: `audit_events` never written by real-backend bus push because `instantiate_{confluence,jira,github}` skips `.with_audit(audit_conn)` | CONFIRMED | `crates/reposix-remote/src/backend_dispatch.rs:234-274` — three `instantiate_*` functions, none chain `.with_audit(...)` |
| PATTERNS | C5 | "verifier subagent (P78 F4) trusted executor SUMMARY for workspace-wide cargo gate it never re-ran" | CONFIRMED via P78 audit text; not re-verified against verdict file (sample didn't reach there) | `phase-audit-p78.md:53-62` (F4) reads exactly that |
| PATTERNS | C7 | Cross-phase self-licensing-deferral-loop (P80 → P86 → RETROSPECTIVE) | CONFIRMED structurally | `phase-audit-p80.md:48-57` (F4+F5), `phase-audit-p86.md:161` (F11), `phase-audit-p87.md:61-101` (F4+F10) all cite this loop |
| PATTERNS | C8 | `webhook-latency.json` has `n: 1, method: synthetic-dispatch, p95: 5s` | CONFIRMED | `quality/reports/verifications/perf/webhook-latency.json` lines 1-11 contain exactly those values; row in `quality/catalogs/agent-ux.json` has `freshness_ttl: null` |
| PATTERNS | C9 | `quality-pre-pr.yml` workflow does not exist | CONFIRMED | `.github/workflows/` contains 9 files, none named `quality-pre-pr.yml`; `cadences: ["pre-pr"]` is wired in catalog rows (26 in agent-ux.json alone) but no CI invokes it |
| PATTERNS | C9 | `bash quality/gates/agent-ux/p88-good-to-haves-drained.sh` against today's tree returns FAIL while catalog row reads `PASS, last_verified: 2026-05-01T22:30:00Z` | CONFIRMED | Live re-run prints "FAIL: ... 1 entry/entries but only 0 terminal STATUS lines" (script exits 0 — separate bug). Catalog row in `quality/catalogs/agent-ux.json` reads `status: PASS`, `last_verified: 2026-05-01T22:30:00Z` |
| PATTERNS | C9 | `bus_precheck_b.rs:262-273` asserts the OPPOSITE of what P82 verdict cited | CONFIRMED | `crates/reposix-remote/tests/bus_precheck_b.rs:263-280` — comment says "PRIMARY ASSERTION (post-P83-01): NO fetch-first signal" + assertion is `assert!(!stdout.contains("fetch first"))` (deferred-shipped removed by P83-01 T04) |
| PATTERNS intro | "12 audits + dark-factory rollup; total findings ~160" | UNVERIFIABLE in detail; HIGH count CONFIRMED | `grep -c "SEVERITY: HIGH\]"` across 12 audit files = 51; total finding count includes MED/LOW/SUSPECT not separately counted |
| REMEDIATION | §1 | "51 HIGH" inventory across audits | CONFIRMED for source count | 51 HIGH across `phase-audit-p7{8,9}.md`, `phase-audit-p8{0..8}.md`, `vision-audit.md` (broken down: p78=2, p79=4, p80=3, p81=2, p82=4, p83=3, p84=4, p85=4, p86=7, p87=4, p88=4, vision=10) |
| REMEDIATION | §1 | Inventory enumerates all 51 HIGH | DRIFTED | Inventory tables contain 44 `H-` rows; unique H-IDs sum to 49 (3 are H-K1/K2/K3 dependency pointers, not findings); H-E1 + H-F1 are explicitly "promoted from MED". So the actual finding-rows = ~43, with at least 8 HIGH findings not explicitly itemized in §1 (e.g. p86 F1, F3 are partial-cluster cites not own rows; p87 F8 cited only in H-H6; p82 F1 covered in H-I7 but the F-number-to-H-number mapping is many-to-many). The mapping is informal not bijective. **Material impact:** the `~51 HIGH → P89-P96` traceability is rough; an executor running the plan should expect 4-8 HIGH findings to be implicit-covered rather than explicit-mapped |
| REMEDIATION | A | H-A1 cites `vision-audit F1` for `reposix attach` sim-only | CONFIRMED | `vision-audit.md:21-31` (F1) reads exactly that |
| REMEDIATION | A | H-A2 cites `vision-audit F2` / `p79 F1` for production phase-ID leak | CONFIRMED | `vision-audit.md:33-44` (F2); `phase-audit-p79.md:28-76` (F1); both cite `attach.rs:147-166` |
| REMEDIATION | A | H-A5 — `agent_flow_real.rs` has `init` smoke but NOT `attach`; "name promises more than asserts" | CONFIRMED | `crates/reposix-cli/tests/agent_flow_real.rs` has `dark_factory_real_{github,confluence,jira}` (lines 128/146/170) all calling only `run_init_and_assert`; zero `attach_real_*` functions; live grep returns no matches |
| REMEDIATION | B | H-B3 cites `p83 F3` for `instantiate_*` skipping `with_audit` | CONFIRMED | `phase-audit-p83.md:40-49` (F3) cites exact files; `backend_dispatch.rs:234-274` has zero `with_audit` chaining |
| REMEDIATION | G | H-G2 cites `vision-audit F9` (milestone-close 8 probes none real-backend) | CONFIRMED | `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md` lists 8 probes; none invoke a real backend; probe 4 = sim-arm dark-factory only |
| REMEDIATION | I | H-I11 — `webhook-latency-floor` row passes vacuously | CONFIRMED | `webhook-latency.json` shows `n: 1, method: synthetic-dispatch`; row's `freshness_ttl` is null per `phase-audit-p84.md` F2 |
| REMEDIATION | I | H-I17 — GOOD-TO-HAVES-01 work shipped 23 minutes after milestone close in `fd2e247` | CONFIRMED with minor numerical drift | `fd2e247` exists with subject "feat(reposix-quality): bind --dimension agent-ux (closes 80% of GOOD-TO-HAVES-01)" at 2026-05-01 15:49:43 -0700; closest milestone-close commit `da72c89` at 15:32:29 → delta is 17m 14s (cited "23 minutes" is +5.5m off but commit + claim direction confirmed) |
| STRATEGIC | Q1 | CHANGELOG.md:11 contains "Devs `git clone git@github.com:org/repo.git` ... `reposix attach` reconciles" | CONFIRMED | Exact text at `CHANGELOG.md:11` |
| STRATEGIC | Q1 | "no v0.13.0 tag exists publicly" → CHANGELOG.md:7 says PENDING owner tag-cut | CONFIRMED | `git tag` confirms; `CHANGELOG.md:9` ("> Release status: PENDING owner tag-cut") confirms |
| STRATEGIC | Q3 | DVCS-DOCS-04 cold-reader rubric was NOT_VERIFIED at milestone GREEN | CONFIRMED | `quality/catalogs/subjective-rubrics.json` row `subjective/dvcs-cold-reader` has `status: NOT_VERIFIED`, `last_verified: null`; cadences `["pre-release"]` |
| COMPLETENESS | S1 | REMEDIATION P89/P90 don't name external/cross-AI arbiter; chicken-and-egg | CONFIRMED | Grep for `gsd-review|cross-ai|codex|external arbiter|cross-cli` in REMEDIATION-PLAN.md returns zero hits |
| COMPLETENESS | S2 | "dark-factory regression run is named ONLY as a P96 success criterion" | CONTRADICTED in literal terms; spirit holds | REMEDIATION P91 SC5 names "T2 in dark-factory exercise re-runs and passes 5/5 boxes" (line 215); P92 SC1 names sim+TokenWorld T4 step 6+7 (line 241); P93 SC4 names "T3 in dark-factory exercise re-runs and passes 8/8 boxes" (line 273); P94 SC1 names T1 (line 307); P96 SC4 is the full-rerun. So per-phase T-N runs ARE scheduled. The deeper S2 critique (no formal "REOPEN this phase if T-N fails" gate that prevents the next phase from starting until the T-N green) DOES still hold — the SCs are pass/fail criteria, not phase-gates. **Sharpened framing:** the per-phase T-N runs are "should pass before phase declares green," not "block downstream phase from starting." S2's recommendation (mid-stream checkpoint with REOPEN authority) remains useful. |
| COMPLETENESS | S3 | Two of 11 deferrals (L2/L3 cache-coherence; SotPartialFail recovery test) undermine v0.13.1 vision | CONFIRMED structurally | Both items appear in REMEDIATION-PLAN §6 deferral table; both are cited as PATTERNS C8 (substrate-gap-deferred) recurrences |

---

## Cross-doc contradictions

None material.

Two soft tensions worth naming for the orchestrator:
1. **PATTERNS** says "the catalog rows that would gate the invariant don't exist or don't fire on the load-bearing path" (C5). **REMEDIATION-PLAN** F-K1/F-K3/F-K4 propose new rows that gate on cadence/kind, but **STRATEGIC** Q3 argues the patch-class fixes are necessary-but-not-sufficient and the load-bearing axis is "claim-vs-assertion congruence." **COMPLETENESS** S1 says that congruence axis itself can't be self-graded by the framework that introduced it. The four docs are coherent in direction but graded for tension between depth-of-fix (REMEDIATION = patch-list) and structural-fix (STRATEGIC + COMPLETENESS = redesign-with-external-arbiter). The orchestrator is consuming a plan that itself has internal disagreement about whether F-K1..F-K8 are sufficient. Worth surfacing.
2. **STRATEGIC Q2** recommends extending v0.13.0; **REMEDIATION-PLAN** uses `v0.13.1` as the milestone identity; **COMPLETENESS** does not re-engage the question. The owner's `SESSION-2026-05-08-HANDOFF.md` decision (per the prompt: "owner has decided ... to extend v0.13.0 with corrective phases P89–P96") resolves this — but REMEDIATION-PLAN's nomenclature still talks about "v0.13.1 milestone" identity in §1/§3/§5. If the owner extends v0.13.0, the REMEDIATION-PLAN section headers (e.g. §3 "Proposed v0.13.1 phase shape") need a search-and-replace. Cosmetic; not blocking.

---

## Stale-claim section

Three stale or off-by-some claims.

1. **PATTERNS C3 visibility surface** points to `architecture-sketch/index.md`'s lookup table; the actual lookup table marking DVCS-ATTACH-01..04 "shipped" without sim-only qualifier is in `.planning/milestones/v0.13.0-phases/REQUIREMENTS.md:130-133`. Substantive claim is true; file-path is wrong.
2. **REMEDIATION-PLAN H-I17** (and **PATTERNS C6**) claim "23 minutes after milestone close in commit `fd2e247`." The actual delta from `da72c89` (P88 milestone-close-prep) at `15:32:29 -0700` to `fd2e247` at `15:49:43 -0700` is 17m 14s. Direction + commit SHA are right; arithmetic is +5m off. (The phrase "23 minutes" first appeared in `phase-audit-p88.md:39-49` F2 — minor authoring drift compounded into the synthesis.)
3. **REMEDIATION-PLAN §1 inventory** declares "51 HIGH finding inventory (51 items, classified)"; the actual table contains 44 H-rows. 49 unique H-IDs (3 are K-prefix dependency pointers, not findings) correspond to ~43 finding-IDs. Not every audit HIGH gets an explicit H-row (cluster-level rows like H-E1, H-F1 fold multiple HIGHs). The "51 items" headline is the inventory the plan PROMISES; the inventory it DELIVERS is ~43 named items + cluster cover. **This is a process-defect: a future planner reading "51 → 8 phases" expects bijective traceability that does not exist in the document.** Not load-bearing for v0.13.1 phase shape; load-bearing for the catalog-first contract the plan is supposedly modeling.

---

## Things the synthesis got right and would be foolish to second-guess

These hold up cleanly under verification. If v0.13.1 starts with these as ground truth, it starts on solid footing.

1. **PATTERNS C1 — Sim-substitution-as-coverage.** The `dark_factory_real_confluence` (and parallel `_github`/`_jira`) test bodies bottom out at URL-shape assertions; live `agent_flow_real.rs` confirms zero `git fetch`/`git push` invocations. The ~28 findings count is plausible against the catalog rows tagged `cadences: ["pre-pr"]` (26 in agent-ux alone, never invoked).
2. **PATTERNS C3 — Scaffold-only-with-phase-ID-leak.** Both literals are byte-for-byte present in `attach.rs` and `sync.rs`. F-K7 (banned production-error-tokens regex) is the right concrete fix; the work is XS-effort to ship.
3. **PATTERNS C5 — OP-3 invariant violated silently.** The three `instantiate_*` functions in `backend_dispatch.rs` confirm the audit_events table is unwritten by construction on real-backend pushes. This is a critical, codebase-grounded finding.
4. **PATTERNS C9 — Catalog-state-fossilized.** The live re-run of `p88-good-to-haves-drained.sh` printing FAIL while the catalog row reads PASS is reproducible right now. Fix-class is mechanical (re-run sweep on freshness, or content-hash binding). Also flagged: the verifier exits 0 despite printing FAIL — a separate bug worth tracking in v0.13.1's framework work.
5. **STRATEGIC Q1 — "Hold the tag" Path B option.** The premise is verifiable: no `v0.13.0` tag, no `v0.13.0` GitHub release, CHANGELOG.md:9 explicitly says PENDING. Option B (hold tag, extend) is therefore literally available — there is no public surface to retract. The owner's stated decision to extend is consistent with the verifiable state.
6. **STRATEGIC Q4 — v0.14.0 readiness gate.** v0.14.0's OTel-on-helper-push presumes the helper-push-audit path works; PATTERNS C5 confirms it doesn't. Block-v0.14.0-on-extended-v0.13.0 is the right sequencing.
7. **STRATEGIC Q6 / PATTERNS meta-pattern — vertical-slice verification with no horizontal-composition gate.** The 11 GREEN per-phase verdicts + GREEN milestone-close verdict aggregating to a vision that fails the dark-factory exercise is the textbook instance of the lesson. This is the most precise diagnosis in the bundle.
8. **COMPLETENESS S1 — framework grading itself is a tautology risk.** Grep for cross-AI/external-arbiter mention in the plan returns zero. The risk is real and unaddressed. Recommendation (pre-commit "what would convince me?" doc + non-Claude `gsd-review` audit of P89/P90 plans) is concrete and tractable.
9. **COMPLETENESS S3 — deferral list contains items that recur the C8 anti-pattern.** Specifically L2/L3 cache-coherence (audit-row stability) and SotPartialFail recovery test (round-trip recovery). Both are cited as `PATTERNS C8 — substrate-gap-deferred-but-row-passes-vacuously` instances — yet the remediation plan defers them via the same shape it diagnoses.

---

**End of verification.**
