---
phase: 90
slug: framework-fixes-honesty-rules
type: execute
execution_mode: top-level
tdd_mode: false
mvp_mode: false
nyquist_compliant: true
catalog_first: true
requirements: [RBF-FW-06, RBF-FW-07, RBF-FW-08, RBF-FW-09, RBF-FW-10, RBF-FW-12]
in_scope_extras: [F-K4b-asserts-congruence-SC2, MISSING_TEST-5-rows, coverage_kind-legacy-RAISE, waiver-cliff-disposition, subagent-graded-migration, intake-drain]
depends_on: [89]
created: 2026-07-04
---

# Phase 90 — Plan Overview

> **EXECUTION MODE: TOP-LEVEL.** Run from a top-level Claude session. **NOT**
> invocable via `/gsd-execute-phase` (gsd-executor lacks `Task`; depth-2 fan-out
> forbidden). The orchestrator routes / decides / integrates; one sub-subagent
> per task does the read/write and returns a ≤300-word TLDR while writing full
> detail to disk. **Two-channel rule** applies to every task.

## Related docs (authorities this OVERVIEW binds)

- `90-DECISIONS.md` — **BINDING** coordinator decisions D90-01..D90-12
  (D90-04 AMENDED + D90-12 added post plan-check, GO-WITH-FIXES). Plans
  implement; do not relitigate.
- `90-RESEARCH-runner.md` (R1) — implementation-ready runner/validator designs
  with `file:line` insertion points, LOC budgets, per-behavior unit-test plans.
- `90-RESEARCH-inventory.md` (R2) — catalog/test census, RAISE raw material,
  waiver triage, MISSING_TEST test designs, gate-authoring precedent.
- ROADMAP.md § Phase 90 (`.planning/milestones/v0.13.0-phases/ROADMAP.md:153-169`)
  — the 8 success criteria (SC1–SC8) these plans map to 1:1.
- `quality/PROTOCOL.md` (catalog-first, verifier exit codes, waiver protocol),
  `quality/catalogs/README.md` (row schema), `quality/SURPRISES.md` D-CONV-1..8
  (the framework was JUST unified — do not re-fragment).

## Phase goal

Close the verifier-shape exemptions that let P78–P88 grade GREEN on sim-only
coverage. Land **catalog-row honesty rules** (transport/perf rows carry
`coverage_kind`; PASS-with-comment banned per F-K4a), **runner honesty
semantics** (missing-verifier demotes; env-skip fail-closes to NOT-VERIFIED
with the prior real grade preserved in history fields; `minted_at` becomes the
immutable audit-cutoff anchor; shell-subprocess PASS requires a real transcript
at grade time; per-expected-assert congruence blocks vocabulary-mismatch PASS),
the **`test-name-vs-asserts.sh`**
triage gate (F-K8), the **absorption-phase honesty-check meta-rule** template
(F-K5, consumed by P96), and the **milestone-close adversarial-pass GREEN-block**
(RBF-FW-12). Walking the new gates + validators over the live catalog produces
the committed **RAISE LIST** (`quality/reports/raise-list-p90.md`) that seeds
P92/P94/P95. The phase leaves `python3 quality/runners/run.py --cadence
pre-push` **honestly green** at close.

Fix-class: **F** (framework). Per D90-01 this phase does NOT absorb the QL-001
product fix (routed P91) and does NOT land the legacy `coverage_kind` migration
(P95 RBF-D-06) — it RAISE-lists both.

## Requirement map (D90-remapped numbering)

The REMEDIATION-PLAN's original P90 REQ-IDs were **remapped by the coordinator**
(D90-04) so the RBF-FW-07/08 slots now carry the cross-AI intake fixes. The
authoritative P90 requirement set:

| REQ-ID | Substance (per D90) | Delivered by |
|---|---|---|
| RBF-FW-06 | `coverage_kind` on transport/perf rows; hard for new, RAISE for legacy (D90-05, F-K4a) | 90-01 (rows) + 90-02 (validator) + 90-05 (legacy RAISE) |
| RBF-FW-07 | 07a missing-verifier ⇒ NOT-VERIFIED + `error` marker; 07b env-skip **fail-closed NOT-VERIFIED + `last_real_grade`/`last_real_verified` history + `skip_reason: env-missing`, ALL pre-release-real-backend rows incl. P0 litmus** (D90-04 AMENDED, D90-12 item 2) | 90-02 |
| RBF-FW-08 | shell-subprocess PASS requires real transcript at grade time (R1 § FW-08, M6) | 90-02 |
| RBF-FW-09 | `test-name-vs-asserts.sh` triage gate (F-K8) | 90-01 (row) + 90-03 |
| RBF-FW-10 | absorption honesty-check meta-rule template (F-K5) | 90-01 (row) + 90-04 |
| RBF-FW-12 | milestone-close adversarial-pass GREEN-block (D90-09) | 90-01 (row) + 90-04 |
| **`minted_at`** (cross-AI H2, D90-03) | write-once immutable audit-cutoff anchor | 90-02 |
| **F-K4b / SC2** (confirmed shipping, D90-12 item 1) | per-expected-assert congruence: every `expected.asserts` entry must map to ≥1 `asserts_passed` entry; any unmatched expected assert blocks PASS (closes p86 F6) | 90-02 (task 90-02f) |

**In-scope extras (D90-07):** the 5 closable MISSING_TEST docs-alignment rows
get REAL tests (90-06); the magic-fixture hazard sweep ships as a RAISE section
(90-05); waiver-cliff disposition (90-05); subagent-graded migration (90-03,
D90-10); intake drain + ROADMAP + CLAUDE.md (90-07).

## Wave DAG

```
                 ┌─────────────────────────────────────────────┐
  Wave A (solo)  │ 90-01 CATALOG-FIRST  (mint ~9 rows + skeleton) │  ← MUST be phase's 1st commit
                 └───────────────────────┬─────────────────────┘
                                         │ (all cite a 90-01 row id)
        ┌────────────────────────────────┼────────────────────────────────┐
 Wave B │ 90-02 RUNNER (python)   90-03 GATE+MIGRATION (bash+catalog)   90-04 TEMPLATES+PROTOCOL (docs+verdict.py) │
 (∥)    │  owns quality/runners/*  owns test-name-vs-asserts.sh +        owns quality/dispatch/* + PROTOCOL.md +   │
        │  NO cargo, NO verdict.py  catalog kind-flips + migration script  PRACTICES.md + verdict.py               │
        └────────────────────────────────┼────────────────────────────────┘
                                         │
  Wave C (solo)  ┌──────────────────────┴──────────────────────┐
                 │ 90-05 RAISE LIST + WAIVERS                    │  reads 90-02 detector + 90-03 gate baseline
                 └──────────────────────┬──────────────────────┘
                                         │
  Wave D (solo)  ┌──────────────────────┴──────────────────────┐
                 │ 90-06 CARGO WAVE (the ONLY cargo; 5 tests +   │  un-waives 5 MISSING_TEST rows
                 │        un-waive; sequential -p scoped)        │
                 └──────────────────────┬──────────────────────┘
                                         │
  Wave E (solo)  ┌──────────────────────┴──────────────────────┐
                 │ 90-07 INTAKE DRAIN + ROADMAP + CLAUDE.md +    │  push + verifier dispatch
                 │        STATE cursor note + phase close        │
                 └─────────────────────────────────────────────┘
```

**Parallelism rule (CLAUDE.md build-memory budget):** Wave B's three tasks are
file-disjoint and **cargo-free** (90-02 = python unittest; 90-03 = bash + JSON;
90-04 = markdown + a ~18-line verdict.py add). They may run in parallel. **90-06
is the ONLY cargo work in the entire phase** and runs solo — no other cargo
invocation machine-wide while it executes. verdict.py is edited ONLY in 90-04 so
Wave B has zero file contention.

## Success-criteria traceability (ROADMAP SC1–SC8 → wave/task)

| SC | ROADMAP text (abbrev) | Home | Status |
|---|---|---|---|
| SC1 | pre-push runs `test-name-vs-asserts.sh`; flagged rows → `raise-list-p90.md` | 90-01 (row) + 90-03 (gate+wiring) + 90-05 (RAISE) | planned |
| **SC2** | **runner refuses PASS on `expected.asserts`↔`asserts_passed` mismatch (F-K4b, p86 F6)** | **90-02 task 90-02f** | **confirmed shipping per D90-12 item 1** (per-expected-assert design; p86 F6 fixture required; zero-false-RED acceptance) |
| SC3 | migration flips subagent-graded rows lacking dispatch wiring (or wires) | 90-03 (D90-10: dvcs-third-arm→mechanical, dvcs-cold-reader→wire) | planned |
| SC4 | absorption template (P96) carries F-K5 meta-rule verbatim + hash binding | 90-01 (row) + 90-04 (template) | planned |
| SC5 | walking gates produces committed RAISE LIST seeding P92/P94/P95 | 90-05 | planned |
| SC6 | adversarial pass documented in PROTOCOL; rubric at `quality/dispatch/milestone-adversarial.md`; runner blocks GREEN on ≥1 fail | 90-01 (row) + 90-04 (rubric+PROTOCOL+verdict.py hook) | planned |
| SC7 | catalog rows land first (NOT-VERIFIED) BEFORE impl; CLAUDE.md same PR | 90-01 (catalog-first) + 90-07 (CLAUDE.md) | planned |
| SC8 | phase close: push; verifier grades GREEN; verdict at `quality/reports/verdicts/p90/VERDICT.md` | 90-07 | planned |

**Every SC has a home.** The SC2 orphaning risk (planner R-1) was resolved by
the plan-check round: D90-12 item 1 confirms 90-02f ships F-K4b with the
per-expected-assert design (defer-to-P95 considered and rejected).

## Plan-level decisions (planner, within D90 authority)

- **P90-D1 — `transport_claim: false` explicit opt-out for meta rows.** R1's
  coverage_kind detector regex (`push|fetch|round-trip|latency|…`) will
  false-positive on P90's OWN meta rows (e.g. `agent-ux/test-name-vs-asserts`,
  whose `expected.asserts` literally contain `push|fetch` as the regex it
  scans). To keep the hard-block honest without a hack, the detector honors an
  explicit tri-state `transport_claim`: `true` → transport (highest precedence);
  **`false` → suppress the regex (reviewable opt-out for gate/source-grep meta
  rows)**; absent → regex decides. 90-01's meta rows carry `transport_claim:
  false`; 90-02's detector implements the tri-state. This extends R1's opt-in
  design; rationale co-located in the detector docstring + this OVERVIEW.
- **P90-D2 — `minted_at`-required reject scoped + swept, not universal-retro.**
  Per D90-03 the validator rejects a post-P90 row lacking `minted_at`, anchored
  on `last_verified >= P90_MINT_CUTOFF`. Set `P90_MINT_CUTOFF =
  "2026-07-05T00:00:00Z"` (P90 landing date; **bump if 90-01 lands later**).
  90-02 acceptance includes a full-catalog `load_catalog()` sweep proving no
  existing row trips the reject at landing. Known residual (documented, not
  fixed here): a *legacy* row whose status flips post-cutoff persists
  `last_verified >= cutoff` and would then trip the reject — operator remedy is
  "add `minted_at` at that moment" (what 89-07 did for its 5 rows); P95 RBF-D-06
  closes it permanently via snapshot-grandfathering. See risk R-2.
- **P90-D3 — verdict.py is edited ONLY in 90-04.** Both the RBF-FW-12 gate hook
  AND R1's optional `error`-marker rendering for FW-07a live in 90-04, so Wave B
  has zero verdict.py contention. 90-02 touches `run.py` / `_audit_field.py` /
  `_realbackend.py` / `test_*.py` only.
- **P90-D4 — catalog-file placement.** New runner-behavior rows → `structure`
  dimension (`quality/catalogs/freshness-invariants.json`, wrapper `"dimension":
  "structure"`, mirroring `structure/claim-vs-assertion-audit-required`). New
  agent-ux/honesty-workflow rows (test-name gate, absorption template,
  adversarial pass) → `quality/catalogs/agent-ux.json`. No new `framework.json`
  (there is no `framework` dimension — same override P89 documented). Choice is
  organizational per R2 § G (runner scopes off `cadences` + wrapper discovery).

## Anti-bloat compliance (load-bearing — R1 measured)

`run.py` is **429 lines, already 79 over its documented ≤350 cap**; `verdict.py`
is 367/400 (~33 headroom). **Every P90 addition to run.py MUST land in a sibling
helper** (`_audit_field.py` for validator concerns, `_realbackend.py` for
grade-path concerns) — never inline. 90-02's plan states final expected line
counts per file. **run.py stays over-cap after P90** (07a is a ~6-line in-place
edit to an existing branch; 07b's fail-closed+history branch adds ~14). The
honest remediation R1 recommends
(a structure-dimension `wc -l` gate on run.py/verdict.py so the cap becomes a
checked invariant, not aspirational prose) is filed to `GOOD-TO-HAVES.md` by
90-02 (XS, next milestone) — **P90 does not silently ignore the breach; it
documents it and files the enforcement.** This is exactly the dishonesty class
P90 exists to kill, so the plan names the number and the remedy explicitly.

## Risk register

| # | Risk | Severity | Mitigation |
|---|---|---|---|
| **R-1** | ~~SC2 (F-K4b asserts-congruence) orphaned~~ **RESOLVED per D90-12 item 1**: 90-02f confirmed shipping with per-expected-assert congruence (the planner's zero-global-overlap sketch was rejected as a dishonest-PASS strawman on the p86 F6 shape). Residual risk is now false-RED tuning. | ~~HIGH~~ → LOW | 90-02f carries the p86 F6 required regression fixture + the "zero false RED across all existing catalog rows against current artifacts" acceptance criterion; per-pair threshold tuned against live mechanical rows, evidence in the wave report. |
| R-2 | `minted_at` reject retro-catches a legacy row that status-flips post-cutoff | MED | P90-D2: landing-time full-catalog sweep + documented operator remedy + P95 permanent fix. Green at close. |
| R-3 | coverage_kind detector regex false-positives on meta/docs rows | MED | P90-D1 `transport_claim: false` opt-out + dimension pre-filter (`perf`/`agent-ux`/`security` only) + minted-≥-P90 gate bounds blast radius to author-controlled new rows. |
| R-4 | FW-08 transcript gate breaks the one live shell-subprocess PASS row (`agent-ux/kind-shell-subprocess-worked-example`) | MED | 90-02 MUST run `bash quality/gates/agent-ux/shell-subprocess-example.sh` for real (OP-1) and confirm its artifact carries `transcript_path`→existing file w/ `argv:` BEFORE landing the gate. |
| R-5 | Doc edits (90-07 CLAUDE.md, 90-04 PROTOCOL/PRACTICES) flip docs-alignment whole-file/line-hash rows to STALE at pre-push | MED | 90-07 anticipates: after doc edits, run `reposix-quality doc-alignment bind`/`/reposix-quality-refresh` on affected rows (top-level) so pre-push stays green; the intake `2026-07-04 05:40` + `08:30` entries are the precedent. Budget a rebind step in 90-07. |
| R-6 | new pre-push gate blows the <60s budget | LOW | R2 § G: single `rg` pass over `crates/` is <1s; no cargo/network. Comfortable. |
| R-7 | 90-06 cargo OOM (VM crashed twice) | MED | 90-06 is solo, sequential, `-p reposix-cli` / `-p reposix-remote` scoped, `--jobs 2`; no parallel cargo anywhere in the phase. |
| R-8 | 2 dangling security verifier paths (`allowlist-enforcement.sh`, `audit-immutability.sh`) collide with FW-07a demote when the 2026-07-26 waiver lapses | MED | 90-05 renews those waivers with honest `tracked_in` AND fixes/re-points the script paths (D90-02d); the collision is named as an explicit P92 line item in the RAISE LIST, not left to accidental discovery. |

## Rollback notes

- Every wave commits atomically citing a 90-01 row id; `gsd-undo` can revert a
  wave via the phase manifest. 90-01 (catalog-first) is the only irreversible-ish
  step (subsequent commits depend on the row ids) — if it needs revision, amend
  before Wave B starts.
- 90-02 runner changes are guarded by unit tests (`test_audit_field.py`,
  `test_realbackend.py`, `test_verdict.py`) — a regression reverts to the P89
  runner cleanly (helpers are additive; the two in-place `run.py` branch edits
  are the only non-additive changes and both have before/after tests).
- 90-03 catalog kind-flips + 90-05 waiver edits are pure JSON; revert = restore
  the row block. The migration (if scripted) lives under `scripts/migrations/`
  and is idempotent per D-CONV-3.
- 90-06 tests are additive; un-waive edits revert to WAIVED.

## Constraints every wave encodes

- **Catalog-first (SC7):** 90-01 is the phase's FIRST commit; every later commit
  cites a row id.
- Commit trailer `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>` +
  `Claude-Session:` line. **No `--no-verify`.** Pre-push runs the docs-alignment
  walker — doc-touching waves (90-04, 90-07) budget the STALE rebind (R-5).
- Pre-push <60s for the new gate (R-6). Phase leaves `--cadence pre-push`
  honestly green at close (SC8).
- Single-tree-writer discipline: each wave's plan names what it MUST NOT touch.
- ONE cargo invocation machine-wide (90-06 only).
