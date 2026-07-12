# v0.13.0 GOOD-TO-HAVES

> **CARRY-FORWARD BANNER — 2026-07-06 pre-v0.13.0-tag sweep.** v0.13.0 is **CLOSED-GREEN, tag imminent.** The P97 drain ledger below is historical; every entry/row still **OPEN / DEFERRED-\*** is a **live carry-forward** to the post-tag **v0.14.0 / v0.13.2 scoping session** for re-triage. STATUS/disposition refs to a now-closed **P9x** phase are historical, not live targets. The 4 **RESOLVING-P97** rows (completed at `302e8ec`) were DELETED this sweep — git is the archive (bound-to-live-state). Do NOT spin up a `v0.14.0-phases/` dir to hold these; they stay here until that scoping session ingests them.

> **Purpose.** OP-8 +2 reservation slot 2 — improvements (clarity, perf, consistency, grounding) the planned phases observed but didn't fold in. Sized XS / S / M; XS items always close; M items default-defer to next milestone. **Drained by P97 (good-to-haves polish + milestone close)** — was P88 in the original P78–P88 plan; renumbered when the milestone extended to P78–P97. (Per-entry "Default disposition for P88" lines below are historical filing-time notes; the actual drain phase is P97.)

---

## Split index (OP-8 file-size drain)

This ledger exceeded the *.md 20k budget and was split into 8 per-part child files under `good-to-haves/`. Every entry is preserved verbatim; append new entries to the last part (or a new part) and add the title here.

- [`good-to-haves/part-01.md`](good-to-haves/part-01.md) — 4 entries:
  - P97 OP-8 Slot-2 DRAIN LEDGER (2026-07-05, Wave A)
  - GOOD-TO-HAVES-01 — extend `reposix-quality bind` to support all catalog dimensions
  - GOOD-TO-HAVES-03 — `run.py` runner has no per-row / per-dimension scope flag (only `--cadence`)
  - GOOD-TO-HAVES-03 — `bind` cannot retarget/remove a cross-file cite; mental-model 27ms mirror + sibling-row drift
- [`good-to-haves/part-02.md`](good-to-haves/part-02.md) — 9 entries:
  - GOOD-TO-HAVES-04 — mechanically verify the two permanently-yellow headline-number rows (8ms-cached-read, 89.1%-token-reduction)
  - GOOD-TO-HAVES-05 — deferral-pointer-linter misses word-form "Phase NN" pointers
  - GOOD-TO-HAVES-06 — structure `wc -l` gate on run.py / verdict.py so the ≤350/≤400 caps are checked, not aspirational
  - GOOD-TO-HAVES-07 — move `parse_rfc3339` from `run.py` into `_freshness.py`
  - GOOD-TO-HAVES-10 — `docs/reference/exit-codes.md` TL;DR table omits clap's own usage-error exit-2 layer
  - GOOD-TO-HAVES-11 — extend `subcommand_help_renders` (cli.rs) beyond 3/15 spot-checked subcommands
  - GOOD-TO-HAVES-12 — annotate `docs/reference/cli.md` exit-codes table: helper-only vs CLI-only examples
  - GOOD-TO-HAVES-14 — helper `list for-push` reports `?` (unknown remote SHA), forcing a redundant export on every push
  - GOOD-TO-HAVES-15 — consolidated file-size overages under the `structure/file-size-limits` waiver (expires 2026-08-08)
- [`good-to-haves/part-03.md`](good-to-haves/part-03.md) — 9 entries:
  - GOOD-TO-HAVES-16 — `quality/runners/run.py` mutates the catalog in place with no `--dry-run` escape hatch
  - 2026-07-05 | `reposix init` should honour `REPOSIX_SIM_ORIGIN` (test-port hardcode) | discovered-by: P91 CI-red fix executor
  - 2026-07-05 | Owner may want to set the `JIRA_TEST_PROJECT` repo secret (KAN) | discovered-by: P91 CI-red fix executor
  - 2026-07-05 | Coverage-as-asset: propose a `code/coverage-ratchet` catalog row | discovered-by: doctrine-coverage audit (owner request)
  - 2026-07-05 | Cache delta-sync under-reports changed records, blocking `git rebase`'s lazy blob fetch | discovered-by: P92 T4 prove-before-fix executor
  - 2026-07-05 | Preamble-anchored marker scan (retire the fixed 6-line lookback in `test-name-vs-asserts.sh`) | discovered-by: P95 marker-footgun pass | severity: LOW
  - 2026-07-05 | Tighten `audit-immutability.sh` WAL grep to a single-line match | discovered-by: P92 security-waiver-flip executor | severity: LOW
  - 2026-07-05 | Drain or consciously renew `structure/file-size-limits` before its 2026-08-08 waiver expiry | discovered-by: P92 verifier + security-waiver-flip executor | severity: LOW
  - 2026-07-05 | Malformed `last_fetched_at` cursor bricks the fetch leg but only warns the push leg (inconsistent degradation) | discovered-by: P93 Exec1 (noticed while building the D-P92-03 repro) | severity: LOW
- [`good-to-haves/part-04.md`](good-to-haves/part-04.md) — 8 entries:
  - 2026-07-05 | Consider a dedicated read-only "runner" subagent type distinct from `gsd-executor` | discovered-by: grounding-bug fix (coordinator dispatched `subagent_type: "executor"`, got "Agent type not found") | severity: LOW
  - 2026-07-05 debt-drain triage
  - 2026-07-05 | `.git/hooks/pre-push` is a dead symlink to a nonexistent target | discovered-by: 2026-07-05 debt-drain branch-hygiene triage | severity: LOW
  - 2026-07-05 | Strategy 2 (defense-in-depth): reclassify delete-time `NotFound` as idempotent success | discovered-by: P93 DP-2 FIX lane (D-P93-02) | severity: LOW
  - 2026-07-05 | Intake files don't name meta-infra (orchestration/agents/skills/hooks/runner-infra/coordinator-discipline) as in-scope | discovered-by: P93 Wave 1 de-risk executor | severity: LOW (deferred tangent)
  - 2026-07-05 | `.claude/hooks/dispatch-doctrine.sh` re-fires its full text on EVERY Agent dispatch with no session-scoped guard | discovered-by: P93 Wave 1 de-risk executor | severity: LOW (cheap fix)
  - 2026-07-05 | Confluence connector's `Record::labels` is not wired to real Confluence labels | discovered-by: P93 Wave 2a executor | severity: P2
  - 2026-07-05 | Sim same-second `list_changed_since` under-report (defense-in-depth precision fix) | discovered-by: P93 Wave 2b executor | severity: P3
- [`good-to-haves/part-05.md`](good-to-haves/part-05.md) — 8 entries:
  - 2026-07-05 | `verdict.py --phase N` is a pure rollup and does NOT scope the P0/P1 gate to phase-N rows | discovered-by: P93 RED-loop verifier (unbiased phase-close grade at `bf3bc9c`) | severity: P2
  - 2026-07-05 | `dark-factory.sh sim` T1-T3 emits a confusing `blocked origin` WARN on git < 2.34 that reads like a failure at first glance | discovered-by: P93 RED-loop verifier (unbiased phase-close grade at `bf3bc9c`) | severity: P3
  - 2026-07-05 | Arm the F-K4b congruence gate for the 5 P93 agent-ux verification artifacts — all carry `asserts_passed: []` | discovered-by: P93 phase-close verifier (unbiased re-verify) | severity: P3
  - 2026-07-05 | `.planning/CONSULT-DECISIONS.md` is 25,074 chars, above the 20k soft limit | discovered-by: P94 catalog-first planning lane | severity: P3
  - 2026-07-05 | GitHub `list_records` → `list_records_complete` delegation is a self-recursion footgun | discovered-by: P94 Finish lane A | severity: P3
  - 2026-07-05 | Split the `doc_alignment.rs` 71k monolith into per-verb modules (bind/walk/status/merge) | discovered-by: P96 Wave 3a (OP-8 Slot 1 hygiene) | severity: LOW
  - 2026-07-05 | Split `cache_coherence.rs` (23.4k) when the crates-source file-size budget is enforced | discovered-by: P96 Wave 3a (OP-8 Slot 1 hygiene) | severity: LOW
  - 2026-07-05 | `catalog-immutable-on-read` gate covers only the `pre-commit` cadence in its real-tree check, not `pre-release`/`pre-push` | discovered-by: P96 phase-close (verdict NOTICED #2 review) | severity: LOW
- [`good-to-haves/part-06.md`](good-to-haves/part-06.md) — 8 entries:
  - 2026-07-06 | env-gate/`minted_at` load-fragility: an env-missing skip advances `last_verified` to run-time, making a `minted_at`-less post-P90 row unloadable once skipped | discovered-by: P97 milestone-close (OP-9 RETROSPECTIVE distillation, verify-against-reality on the 9th-probe mint) | severity: MEDIUM
  - 2026-07-06 | ORCHESTRATION.md exceeds the 20k soft char-limit (~21.7k) | discovered-by: relief-threshold/C2 doctrine review | severity: LOW
  - 2026-07-06 | C2 (coordinator-of-coordinators) recursion is doctrine-only — never exercised | discovered-by: relief-threshold/C2 doctrine review | severity: MEDIUM
  - 2026-07-06 | `docs/guides/troubleshooting.md` is 25.5k chars, over the 20k progressive-disclosure soft limit | discovered-by: v0.13.0-intake-disposition sweep (260706-crf cold-reader carryover) | severity: MEDIUM
  - 2026-07-06 | `codecov/project` posts a phantom -16% release-blocker when the Rust lcov upload silently fails | discovered-by: PR #61 codecov triage lane | severity: MEDIUM
  - 2026-07-06 | Durable Confluence TokenWorld fixture page 7798785 has been accidentally trashed 2+ times, losing parent linkage to 7766017 each time | discovered-by: PR #61 CI-red repair lane | severity: LOW-MEDIUM
  - 2026-07-07 | release-plz branch regenerations silently drop `pull_request`-triggered workflows (CI, Security audit, quality gates) | discovered-by: v0.13.0 release CI investigation | severity: MEDIUM
  - 2026-07-07 | Manually merging `origin/main` onto a live release-plz branch races the bot's own regeneration and usually loses | discovered-by: v0.13.0 release CI investigation | severity: LOW-MEDIUM (process note)
- [`good-to-haves/part-07.md`](good-to-haves/part-07.md) — 8 entries:
  - 2026-07-07 | Quality gates asserting a third-party tool's exact surface string false-negative when the vendor rewords output | discovered-by: v0.13.0 post-release CI run 28839335746 investigation | severity: LOW-process
  - 2026-07-07 | `SURPRISES-INTAKE.md` has outgrown its own pre-commit soft limit (~77k chars vs. 20k warn threshold) | discovered-by: v0.13.0 post-release verification pass | severity: LOW (process)
  - 2026-07-07 | Ship a bundled default seed inside the release binary so getting-started needs no `--seed-file` / no network fetch | discovered-by: v0.13.1 Wave E (README quick-start doc-lie fix) | severity: MEDIUM
  - 2026-07-07 | doc-alignment catalog carries a ~180-row backlog of un-rebound drift outside this milestone's edited docs | discovered-by: v0.13.1 Wave E1b (doc-alignment rebind lane) | severity: MEDIUM
  - 2026-07-07 | `doc-alignment walk` mutates the committed catalog in place with no `--persist` gate, unlike `run.py`'s GRADE/PERSIST split | discovered-by: v0.13.1 Wave E1b (doc-alignment rebind lane) | severity: MEDIUM
  - 2026-07-05 | `badges-resolve` FAILs on pre-push (docs-build + structure dimensions) | discovered-by: P93 Wave 1 de-risk executor | severity: MEDIUM
  - 2026-07-07 | `git-version-requirement-documented.sh` is a bare `grep -F '2.34'`, cannot detect a hard-vs-recommended regression | discovered-by: v0.13.1 mechanical filing lane | severity: LOW
  - 2026-07-07 | doc-alignment `walk` mutates the catalog with no `--persist` gate — dirties the tree on every validate-only run | discovered-by: v0.13.1 mechanical filing lane (cross-referencing Waves E1/E1b/F1b) | severity: MEDIUM
- [`good-to-haves/part-08.md`](good-to-haves/part-08.md) — 4 entries:
  - 2026-07-07 | ~180-row doc-alignment backlog likely harbors more Haiku-backfill false-BONDS + latent TEST_DRIFT rows — needs systematic re-grade | discovered-by: v0.13.1 mechanical filing lane (cross-referencing Wave F1b + C2-f handover) | severity: MEDIUM
  - 2026-07-07 | Pre-commit size soft-warnings: `crates/reposix-cli/src/main.rs` and `quality/gates/agent-ux/zero-shot-onboarding.sh` both over budget | discovered-by: v0.13.1 mechanical filing lane | severity: LOW
  - 2026-07-07 | Cosmetic front-door UX: stray `builtin seed loaded` INFO line prints before `reposix sim`'s clean banner | discovered-by: v0.13.1 mechanical filing lane | severity: LOW
  - 2026-07-07 | Shared push-test helper that always injects a per-test `REPOSIX_CACHE_DIR` tempdir | discovered-by: p94/protocol.rs CRLF-flake fix executor | severity: LOW
