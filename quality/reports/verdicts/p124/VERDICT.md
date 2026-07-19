---
phase: 124-container-rehearse-harness-hardening
milestone: v0.15.0
verifier: independent phase-close verifier (zero session context beyond the charter; OP-7 remediation — no verifier VERDICT.md existed for P124)
verified_at: 2026-07-19T00:54:18Z
head: d3d8052fb2bc883427547fb769732a35cc4618a4
overall: GREEN
score: 4/4 success criteria PASS (example-05 post-release LEG NOT-VERIFIED on disk by-design; re-verified PASS in-container here)
post_push_cadence: MIXED — P124 phase-close tip 790aa73c ci.yml + release-plz.yml BOTH success (CI-green bar met at close); current HEAD d3d8052f (post-close docs commits) ci.yml CANCELLED via a transient "quality gates (pre-pr)" 15m-cap timeout (all other jobs green; identical gate set passed on 790aa73c). code/ci-green-on-main = NOT-VERIFIED (never FAIL). Coordinator holds the push until main ci.yml re-runs green.
requirements: DRAIN-13, DRAIN-14, DRAIN-22, DRAIN-23, DRAIN-24 — all SATISFIED
aggregate_p2_fail_disposition: 8/8 pre-existing-baseline — none P124-introduced
overrides_applied: 0
---

# Phase 124 — Container-rehearse harness hardening: independent phase-close verdict

**Goal:** `quality/gates/docs-repro/container-rehearse.sh`'s `kind: container` docs-repro
rows are provenance-guaranteed and immune to SIGKILL orphaning, exit-code disagreement, and
tautological assertion-congruence.

**Context (OP-7 remediation):** P124 was closed GREEN and STATE advanced 10→11 (`b01afabc`)
on the runner output + the executing lane's word, with **no independent verifier VERDICT.md**
— while p120/p121/p122/p123 each carry one. This artifact grades P124 against reality and
supplies the missing verdict. **Independent finding: the GREEN close was substantively
correct** — every SC is genuinely delivered, and all 8 aggregate FAILs are pre-existing
baseline; the OP-7 defect is the missing artifact, not a wrong grade.

**Ground truth:** `HEAD == origin/main == d3d8052f`, working tree clean (bar gitignored
`quality/reports/` gate artifacts from this run). P124 impl commits `ac632c5d..790aa73c`
present; close commits `3b1a61ce..d3d8052f` present. Substrate on this host: docker 28.1.1,
lsof, ss, kcov, `target/debug/reposix` (Jul 18), git 2.50.1 — every gate ran for real.

**Method:** re-ran every P124 gate/selftest, ran the example-05 container harness in
`ubuntu:24.04` (git 2.43), re-ran all 8 aggregate-FAIL gates, ran the pre-push cadence, and
hit the live GitHub API for CI status. Did not trust SUMMARY/runner claims.

---

## Overall: GREEN — 4/4 SC delivered · 8/8 aggregate FAILs pre-existing · no P0/P1 FAIL

The runner verdict (`2026-07-18T20-05-49Z.md`) colors **RED** solely because it counts 8
**P2** agent-ux hygiene FAILs at the aggregate. **Every one is pre-existing baseline** (proven
below); none is P124-introduced; no P0/P1 gate FAILs. Applying the charter verdict logic —
GREEN iff (a) 4 SCs delivered, (b) all P0/P1 green, (c) all 8 P2 FAILs pre-existing — the
phase is **GREEN**. One post-close CI caveat is surfaced under NOTICED #1 (coordinator-gated).

| SC | Verdict | One-line basis |
|----|---------|----------------|
| SC1 | **PASS** | Congruence EARNED — selftest T4 proves a no-op `exit 0` fixture does NOT earn congruence; gate rc=0; harness deletes the verbatim-copy path + harvests `^ASSERT-PASS:` with a non-empty guard. example-05 drove the **real** `[RPX-0503]` refusal in-container (harvested 3 lines, exit 0). |
| SC2 | **PASS** | `container-rehearse-sigkill-safe.sh` rc=0; after a mid-run SIGKILL, port 7878 is **free** and **no orphan `reposix sim`** survives. Process-group teardown + internal `timeout` + fail-loud stale-orphan gate all present. |
| SC3 | **PASS** | `container-rehearse-binary-provenance.sh` rc=0 — `quality-post-release.yml` builds `target/debug/reposix` explicitly (inline provenance comment) before the post-release gates. |
| SC4 | **PASS** | `container-rehearse-exit-from-artifact.sh` rc=0 — forced artifact `exit_code=1` with docker rc=0 → harness exits 1; `.sim-*.log` git-ignored (`git check-ignore` rc=0). |

---

## Per-SC evidence (verified against reality)

### SC1 — earned congruence + example-05 real runtime (DRAIN-22) → PASS

- **Tautology closed (code):** `container-rehearse.sh` comments + code confirm the verbatim
  `expected.asserts → asserts_passed` copy path is DELETED; harness greps `^ASSERT-PASS: `
  from container stdout (`:269`), guards non-empty harvest (`:314 elif not harvested:` →
  fail), and `asserts_passed` is the HARVESTED set (`:344`), never the row's asserts.
- **Meta-check selftest (ran):** `container-congruence-earned.selftest.sh` → **ALL PASS**
  (T1 gate passes real harvest harness; T2 FAILs a re-introduced verbatim copy; T3 FAILs a
  harness missing the harvest path; **T4 proves a real fixture earns congruence and a no-op
  `exit 0` does NOT**; T5 FAILs a harness dropping the empty-harvest guard). Gate rc=0.
- **example-05 real runtime (ran in-container):** `container-rehearse.sh
  docs-repro/example-05-blob-limit-recovery` in `ubuntu:24.04` (git 2.43) → harness rc=0;
  artifact `exit_code=0`, `docker_rc=0`, **`harvested_assert_pass_count=3`**, `asserts_passed`
  HARVESTED from the container's own stdout — first line confirms the **real
  `BLOB_LIMIT_EXCEEDED_FMT` / `[RPX-0503]` / `git sparse-checkout` stderr refusal fired**.
  The example is fail-loud (`run.sh:59-70` exits non-zero unless the real `[RPX-0503]` token is
  captured), so a green harness rc IS proof the real error was driven, not stubbed.
- **Catalog:** the 4 mechanical rows are minted **PASS** (each with `minted_at` + a ≥849-char
  `claim_vs_assertion_audit`); `docs-repro/example-05-blob-limit-recovery` is **NOT-VERIFIED**
  on disk by-design (`cadences: [post-release]`) — re-grades in the post-release container job
  (and re-verified PASS in-container here).

### SC2 — SIGKILL-proof teardown + fail-loud orphan (DRAIN-23) → PASS

- `container-rehearse-sigkill-safe.sh` → **rc=0**. The selftest SIGKILLs its planted sim
  mid-run (`line 44 ... Killed setsid "$SIM_BIN" sim ...`, expected). Afterward: `lsof -ti:7878`
  → **7878 free**; `ps aux | grep 'reposix sim'` → **no orphan**. Process-group (`setsid`) kill,
  internal `timeout < timeout_s`, and the pre-run port-7878-occupied fail-loud gate are present.

### SC3 — binary provenance on the post-release runner (DRAIN-24) → PASS

- `container-rehearse-binary-provenance.sh` → **rc=0**: the YAML-parse gate confirms an explicit
  `cargo build -p reposix-cli` step (inline provenance comment) precedes `run.py --cadence
  post-release` in `quality-post-release.yml`, so container docs-repro rows no longer silently
  degrade to NOT-VERIFIED on a cold runner.

### SC4 — exit-from-artifact + `.sim-*.log` gitignore (DRAIN-13 + DRAIN-14) → PASS

- `container-rehearse-exit-from-artifact.sh` → **rc=0**: a forced artifact `exit_code=1` with a
  docker rc of 0 makes the harness exit 1 (rc-masks-artifact gap closed).
- `git check-ignore quality/reports/verifications/docs-repro/.sim-example-01-shell-loop.log` →
  **rc=0** — `.sim-*.log` is git-ignored under the docs-repro verifications dir.

---

## Aggregate P2-FAIL disposition — all 8 pre-existing-baseline, none P124-introduced

**Structural proof (applies to all 8):** each gate inspects a **v0.12.0 / v0.13.0 / v0.14.0**
milestone artifact or a root file (`CHANGELOG.md`, `.planning/RETROSPECTIVE.md`,
`scripts/ci-wait.sh`). **None of those inputs appears in the P124 diff** (`bc4decf3..d3d8052f`,
which touched only v0.15.0 ledgers + P124's own code/catalogs), and **every input's last commit
predates P124's plan commit `ffcf865d` (2026-07-18 09:26)**. P124 (09:26–12:10) could not have
introduced or fixed any of them. I additionally re-ran all 8 for reality.

| # | Row (P2, agent-ux) | Input inspected (last commit) | Re-run now | Disposition + evidence |
|---|---|---|---|---|
| 1 | `p87-surprises-absorption` | v0.13.0 SURPRISES-INTAKE (`dad227e5`, 2026-07-12) | rc=1 (0 terminal STATUS) | **pre-existing** — input untouched by P124 |
| 2 | `p110-surprises-absorption` | v0.14.0 SURPRISES-INTAKE (`bcdee076`, 2026-07-13) | rc=1 (16 OPEN entries) | **pre-existing** — input untouched by P124 |
| 3 | `p88-good-to-haves-drained` | v0.13.0 GOOD-TO-HAVES (`dad227e5`, 2026-07-12) | rc=1 (0 entries) | **pre-existing** — input untouched by P124 |
| 4 | `v0.13.0-tag-script-present` | v0.13.0 `tag-v0.13.0.sh` (`7b189b15`, 2026-05-08; now absent) | rc=1 (not found) | **pre-existing** — input untouched by P124 |
| 5 | `p111-ci-wait-helper` | `scripts/ci-wait.sh` (`02d24233`, 2026-07-12) | **rc=0 (PASS)** | **pre-existing** — stale FAIL persisted pre-P124; reality PASSes |
| 6 | `p111-changelog-v0.14.0-section` | `CHANGELOG.md` (`b1c4b740`, 2026-07-12) | **rc=0 (PASS)** | **pre-existing** — stale FAIL persisted pre-P124; reality PASSes |
| 7 | `p111-retrospective-v0.14.0-section` | `.planning/RETROSPECTIVE.md` (`22e83561`, 2026-07-13) | **rc=0 (PASS)** | **pre-existing** — stale FAIL persisted pre-P124; reality PASSes |
| 8 | `p111-milestone-hygiene` | v0.14.0 GOOD-TO-HAVES/ROADMAP/CONSULT (`708b3e9b`, 2026-07-13) | rc=1 (item 5: v0.14.0 GTH over ceiling) | **pre-existing** — inputs untouched by P124 |

5 of the 8 genuinely FAIL now on inputs P124 never touched; 3 (#5–#7) are **stale persisted
FAILs** that actually PASS on re-run (see NOTICED #2). Either way, **zero P124 regressions.**

---

## P0/P1 gate results (re-run against reality)

- **pre-push cadence:** `python3 quality/runners/run.py --cadence pre-push` →
  `69 PASS, 1 FAIL, 0 PARTIAL, 1 WAIVED, 1 NOT-VERIFIED -> exit=0`. The **only FAIL is
  `code/shell-coverage` (P2)** — the pre-existing `transcript.sh` kcov-vs-counter drift
  (documented in `quality/CLAUDE.md` § Two honesty layers; filed OPEN in
  `surprises-intake/part-07.md`); it flips only the P2 counter-validation assert, the aggregate
  floor still passes, **exit=0**. P124's only change to `shell-coverage.sh` is a comment block
  (zero logic change; `git diff bc4decf3..d3d8052f`). **No P0/P1 FAIL.**
- **`container-congruence-earned` (P0):** PASS (rc=0 + selftest ALL PASS).
- **`code/ci-green-on-main` (P0):** graded **NOT-VERIFIED** (verifier exit 75, never FAIL).
  Resolved live via `gh`: **P124 phase-close tip `790aa73c` — ci.yml `success` +
  release-plz.yml `success`** (the CI-green-after-the-phase-push bar met at close). Current
  **HEAD `d3d8052f`** (post-close docs commits): release-plz `success`, CodeQL `success`, Docs
  `skipped`, **ci.yml `cancelled`** — run `29667079687`: every job green EXCEPT `quality gates
  (pre-pr)`, cancelled for **exceeding the 15m execution cap** (20m). The identical gate set
  concluded `success` on `790aa73c` (the `790aa73c→d3d8052f` delta is docs-only), so this is a
  **transient loaded-runner pre-pr timeout, not a P124 regression** — see NOTICED #1.

---

## Requirements coverage

| Req | SC | Status |
|-----|----|--------|
| DRAIN-22 | SC1 | SATISFIED (earned congruence + example-05 real runtime) |
| DRAIN-23 | SC2 | SATISFIED (SIGKILL-proof teardown + fail-loud orphan) |
| DRAIN-24 | SC3 | SATISFIED (explicit binary-provenance build step) |
| DRAIN-13 | SC4 | SATISFIED (exit derived strictly from persisted artifact exit_code) |
| DRAIN-14 | SC4 | SATISFIED (`.sim-*.log` gitignored) |

**Catalog-first held:** W0 commit `ac632c5d` minted all 5 rows NOT-VERIFIED (each with
`minted_at` + ≥849-char `claim_vs_assertion_audit`) BEFORE the first impl commit `a54ba881`.
The 4 mechanical rows minted PASS at close (`8d9a269a`); example-05 stays NOT-VERIFIED
(post-release). Both catalog-first integrity P0s (`minted-at-write-once`,
`claim-vs-assertion-audit-required`) are PASS in the aggregate.

---

## NOTICED (OD-3 deliverable)

1. **WARNING — main HEAD `d3d8052f` ci.yml is CANCELLED (pre-pr quality-gates 15m timeout);
   coordinator-gated.** NOT a P124-SC defect: all other jobs on that run passed (clippy, test,
   rustfmt, shell-coverage, coverage, all real-backend contract integrations, dark-factory,
   bench, CodeQL), and P124's own tip `790aa73c` passed ci.yml with the identical gate set. But
   the `quality gates (pre-pr)` CI job now sits dangerously near the 15m ceiling — a risk
   **already filed OPEN** in `surprises-intake/part-07.md` (the pre-push/pre-pr runtime-budget
   entry, STATUS: OPEN). P124 added container gates to the `pre-pr` cadence
   (`container-rehearse-sigkill-safe` ~13s + a review de-flake that widened the SIGKILL budget),
   narrowing the margin even though `790aa73c` still passed. **Recommendation to the coordinator
   (who holds the push):** (a) re-run the cancelled ci.yml job on `d3d8052f` and confirm green
   before pushing this verdict; (b) append the `29667079687` cancelled-run data point to the
   OPEN pre-pr-budget entry. Severity: MEDIUM (main-CI-green bar; transient, not a code defect).
2. **INFO — 3 of the 8 aggregate FAILs are STALE persisted-FAILs.** `p111-ci-wait-helper`,
   `p111-changelog-v0.14.0-section`, and `p111-retrospective-v0.14.0-section` PASS on re-run;
   their inputs were fixed pre-P124 but the persisted catalog status was never refreshed to
   PASS. The runner's aggregate over-counts FAILs by 3. A `--persist` refresh sweep (outside a
   VERDICT-only charter, and gated by the SC2 downgrade-guard) would reconcile them. Not a P124
   concern; recommend a hygiene-lane refresh. Severity: LOW.
3. **INFO — OP-7 process gap is artifact-only.** The independent grade CONFIRMS the GREEN close:
   all 4 SCs delivered, all 8 FAILs pre-existing. STATE-advance (10→11) was not built on a wrong
   grade — the sole defect was the missing verifier VERDICT.md, now supplied. The
   `L0-owns-CI-watch` liveness doctrine held for the phase push (`790aa73c` green); the caveat
   in #1 is a later post-close commit.
4. **INFO — example-05 row correctly NOT-VERIFIED on disk.** `cadences: [post-release]`; it is
   NOT a stub — re-graded PASS in-container here (rc=0, 3 harvested ASSERT-PASS lines) and
   re-grades in the post-release container job.

No BLOCKER findings. One coordinator-gated WARNING (#1) — the push is intentionally held until
main ci.yml re-runs green.

---

## Eager-fix vs filed

- **Not eager-fixed (correctly held):** the cancelled main ci.yml (#1) is a coordinator-held
  push condition, not a verifier fix. The pre-pr runtime-budget risk is **already filed OPEN**
  at `.planning/milestones/v0.15.0-phases/surprises-intake/part-07.md` (pre-push/pre-pr budget
  entry) — MEDIUM.
- **Filed / recommended (LOW):** the 3 stale persisted-FAIL rows (#2) — recommend a `--persist`
  hygiene refresh; kept out of this VERDICT-only charter (staging is `VERDICT.md` only).

---

_Independent verifier — verified against reality (ran every P124 gate + selftest, ran the
example-05 container harness in ubuntu:24.04, re-ran all 8 aggregate-FAIL gates, ran the
pre-push cadence, hit the live GitHub API). Verdict artifact only; committed LOCALLY, NOT
pushed — the coordinator holds the push until main ci.yml re-runs green._
