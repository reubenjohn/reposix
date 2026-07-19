---
phase: 126-docs-alignment-tooling-polish
plan: 01
subsystem: quality-tooling
tags: [doc-alignment, catalog-integrity, agent-ux, minted_at, coverage, docs, grader, roadmap]

# Dependency graph
requires:
  - phase: 123-quality-runner-catalog-integrity-hardening
    provides: "the _audit_field.validate_row minted_at reject path + the --persist GRADE/PERSIST split (D-P96-01) that W1's read-only write-boundary guard hardens"
  - phase: 117-doc-truth-launch-blocker-purge
    provides: "the P117-W3 STALE_DOCS_DRIFT lesson (delete-and-rebind in ONE commit) that W5's fix-in-place-not-delete pivot honors"
provides:
  - "agent-ux/real-git-push-e2e now carries a write-once minted_at anchor — the git>=2.34 load-crash landmine is defused for every cadence loading agent-ux.json (incl. the P128 milestone-close 9th probe)"
  - "run.py::save_catalog write-boundary guard: a persist=False write raises RuntimeError (validate/read cadences are structurally read-only, not by convention)"
  - "docs-alignment walk BLOCK LEADS with a per-row-STATE summary (which STATE(s) block + row ids + teach/alt/recovery), not just a ratio (DRAIN-17)"
  - "doc-alignment status prints a waived_active counter recomputed vs now — waived MISSING_TEST rows are surfaced, not hidden (DRAIN-20)"
  - "coverage::eligible_files counts the doc-of-record + narrow-source surfaces BOUND rows cite — out-of-eligible warnings 17->2 (DRAIN-21)"
  - "grader.md binds a row ONLY when the cited test FAILS on drift + after grep'ing src/ unit tests (DRAIN-18)"
affects: [126-close, milestone-close-9th-probe, doc-alignment-operators, quality-runner-persist-path]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Write-boundary integrity: a required persist= keyword on the catalog write fn raises on a non-persist write — enforces read-only cadences structurally instead of trusting the call site"
    - "Legacy-row minted_at proactive stamp: any pre-P90 row whose verifier can start re-executing (env/git-version precondition now met) must carry minted_at or it crashes load"
    - "Fix-in-place over delete when deletion would strand a human-gated confirm-retire (RETIRE_PROPOSED) — relocate source anchors to honest lines, never force a false BOUND"

key-files:
  created: []
  modified:
    - quality/runners/test_audit_field.py
    - quality/catalogs/agent-ux.json
    - quality/runners/_audit_field.py
    - quality/runners/run.py
    - quality/runners/test_run.py
    - crates/reposix-quality/src/commands/doc_alignment.rs
    - .claude/skills/reposix-quality-doc-alignment/prompts/grader.md
    - crates/reposix-quality/src/coverage.rs
    - README.md
    - .claude/skills/reposix-quality-doc-alignment/refresh.md
    - CLAUDE.md
    - docs/development/roadmap.md
    - quality/catalogs/doc-alignment.json
    - quality/gates/structure/active-milestone-matches-workspace-version.sh

key-decisions:
  - "DP-2 (prove-before-fix): the HIGH landmine got a COMMITTED pytest RED repro (44783ebe) BEFORE the field/guard fix (65e8c497), and a FRESH code-review (5d097937) confirmed the fix addresses the mechanism (validate_row reject path + non-persist write guard), not just the one symptomatic row"
  - "DP-3 (no inversion): the landmine fix adds minted_at to an existing row + hardens the existing load/write path — no new artifact/dependency/concept, already the simplest mechanism"
  - "RAISE-3 fix-in-place, NOT delete: deleting docs/development/roadmap.md would strand the human-gated confirm-retire (RETIRE_PROPOSED is $CLAUDE_AGENT_CONTEXT-guarded); instead fixed the active-milestone lie in-place + re-cited all 5 rows in the SAME commit (no STALE_DOCS_DRIFT window)"
  - "RAISE-3 false-BOUND avoidance: the active-milestone row's test falsifies CHANGELOG<->Cargo drift, NOT a 'vX is active' claim — so a 'v0.15.0 is active' rebind would be a false BOUND; kept the claim test-accurate, relocated only the source anchor to an honest line"
  - "DRAIN-21 disposition = allowlist-include (not re-cite): eligible_files() extended to the doc-of-record surfaces + 2 narrow explicit source files (never a crates/**/*.rs glob, which would flip the metric to code-coverage and tank the ratio); the 2 residual warnings are RETIRE_CONFIRMED redirect rows for already-deleted docs (decided floor, no action)"

requirements-completed: [DRAIN-15, DRAIN-16, DRAIN-17, DRAIN-18, DRAIN-19, DRAIN-20, DRAIN-21]

# Metrics
duration: ~1 day (6 strictly-serial waves)
completed: 2026-07-19
---

# Phase 126 Plan 01: Docs-alignment tooling polish (DRAIN-15..21) + landmine + RAISE-3 Summary

**DRAIN-15..21 make the doc-alignment skill/tooling surface more reliable and less confusing; one EARLY HIGH landmine (`agent-ux/real-git-push-e2e` load-crash) is defused repro-first with a read-only write-boundary guard; and RAISE-3 fixes the stale `docs/development/roadmap.md` active-milestone lie in-place (re-citing all 5 doc-alignment rows in one commit) rather than deleting a file whose retirement is human-gated.**

## Performance

- **Duration:** ~1 day across 6 strictly-serial executor waves (W1..W6)
- **Completed:** 2026-07-19
- **Tasks:** 6 waves (W1 HIGH landmine · W2 Lane B · W3 Lane C · W4 micro-batch · W5 RAISE-3 · W6 close)
- **Files modified:** 14 code/doc/catalog + 5 planning/bookkeeping

## Accomplishments (per-lane, with SHAs)

- **W1 landmine (HIGH, DP-2 repro→fix→hardening):** `44783ebe` (RED repro: aged `last_verified` w/o `minted_at` crashes catalog load) → `65e8c497` (add `minted_at` to `agent-ux/real-git-push-e2e` + `run.py::save_catalog` read-only write-boundary guard: `persist=False` write raises `RuntimeError`) → `d0753ef6` (fix-twice: validate cadences read-only + legacy-minted_at doctrine in `quality/CLAUDE.md`). DP-2 mechanism-vs-symptom review **PASS** `5d097937`. The row's stale `git 2.25.1` comment was refreshed to the box's real git (2.50.1). Whole-corpus regression: `test_run.py::TestNoArmedMintedAtLandmine`, `TestSaveCatalogPersistGuard`, `TestValidateOnlyMultiCatalogByteIdentical`.
- **W2 Lane B:** `d093bc7f` (DRAIN-17 — walk BLOCK now LEADS with `docs-alignment BLOCK: N row(s) blocking across M state(s):` then a per-STATE line naming count + exact row ids + teach/alt/recovery; regression `walk-block-summary.selftest.sh`) · `e693deeb` (DRAIN-18 — `grader.md` binds only when the cited test FAILS if the number/claim drifts AND after grep'ing `src/` unit tests, not just re-reading the cited test).
- **W3 Lane C:** `0270f91c` (DRAIN-20 — `status` prints a `waived_active` counter recomputed vs `now` via `waiver_status(now) == Active`, in the `== global ==` block + json payload; plus a RETIRE_PROPOSED walk-line `row=<id>` UX fix) · `e8823049` (DRAIN-21 — out-of-eligible warnings **17→2**; `eligible_files()` extended to count `benchmarks/README.md` + 3 archived `.planning/` prose docs + the archived-REQUIREMENTS cutoff v0.11.0→v0.15.0 + 2 NARROW explicit source files `backend.rs`/`main.rs` that 11 BOUND rows verify against; `coverage_ratio` 0.165→0.180, floor 0.10 PASS) · `1df18239` (clippy pedantic doc_markdown backtick).
- **W4 micro-batch:** `1ef508bf` (DRAIN-16 README MCP→`Model Context Protocol (MCP)` first-use + DRAIN-19 `refresh.md` cold-`plan-refresh`-under-reports note + DRAIN-15 in-repo doc-clarity-review subscription/canary caveat) · `639ff67f` (3 stale git-version comment refreshes in sibling agent-ux rows).
- **W5 RAISE-3:** `588c1546` (fixed the stale `docs/development/roadmap.md` active-milestone lie **in-place** + re-cited all 5 doc-alignment rows in-commit + refreshed the stale `active-milestone-matches-workspace-version.sh` header; NOT deleted — deletion would strand the human-gated confirm-retire).

## DRAIN-21 dispositions (16→17 audit-confirmed, recorded inline per plan)

The audit found **17** (not the ROADMAP-def's estimated 16) `coverage: row <id> cites out-of-eligible file <path>` warnings. Disposition was **allowlist-include** (extend `eligible_files()`), not per-row re-cite, because the cited files are all legitimate doc-of-record or narrow-source surfaces the metric's denominator should count:

1. **`benchmarks/README.md`** — legitimate doc surface, added to the eligible set.
2. **3 archived `.planning/` prose docs** — doc-of-record prose the BOUND rows cite; added.
3. **archived-REQUIREMENTS cutoff v0.11.0→v0.15.0** — extends the archived milestone-REQUIREMENTS glob forward so the newer archived milestones' REQUIREMENTS are eligible.
4. **2 NARROW explicit source files (`backend.rs`, `main.rs`)** — the exact source files 11 BOUND rows verify against; added as explicit paths, NEVER a `crates/**/*.rs` glob (which would convert the metric to code-coverage and drop the ratio below floor).
5. **Residual floor = 2 (decided, no action):** the two remaining warnings are `RETIRE_CONFIRMED` redirect rows for already-deleted `docs/architecture.md` + `docs/demo.md`. Filed forward as a good-to-have (skip RETIRE_CONFIRMED rows in the emitter to reach 0).

`collect_backfill_inputs` (the prose-miner input) mirrors only the shared base + the REQUIREMENTS cutoff bump — the 2 source files stay OUT (the miner extracts prose, never Rust source); this one intentional divergence is documented in both fn doc comments. **NOTICED:** the two lists are hand-maintained parallel copies (filed forward — DRY behind one shared base).

## DRAIN-15 out-of-repo surfacing (to L0)

The confusing behavior lives OUTSIDE this repo — `~/.claude/skills/doc-clarity-review/SKILL.md`'s nested `claude -p` returns a confusing non-error when it can't see file content. The in-repo committable deliverable (a subscription/canary caveat on root `CLAUDE.md` § Cold-reader pass) landed in `1ef508bf`. **Surfaced to L0 — out of repo commit boundary:** the recommended fix is a one-line **canary-probe HARD-FAIL** in the user-global `SKILL.md` — the nested `claude -p` should exit non-zero (fail the review) when it cannot see the target file's content, so a cold-reader dispatch cannot silently under-report. Filed as a GOOD-TO-HAVE (OWNER-ACTION) since L0 must apply it out-of-repo.

## Deviations from Plan

### 1. [Rule 3-adjacent — doctrine-over-literal] W5 fixed the roadmap in-place instead of deleting it

- **Found during:** W5 (RAISE-3).
- **Issue:** The plan's W5 action said DELETE `docs/development/roadmap.md` + redirect + retire the `v0-11-0-active-milestone` row via `confirm-retire`. But `confirm-retire` is `$CLAUDE_AGENT_CONTEXT`-guarded (human-only); an autonomous delete would strand a RETIRE_PROPOSED row.
- **Fix:** Took the plan's own documented escape-hatch intention — fixed the active-milestone lie in-place, re-cited all 5 rows to honest lines in the SAME commit (no STALE_DOCS_DRIFT window), kept each claim test-accurate (avoided a false BOUND on the active-milestone claim).
- **Committed in:** `588c1546`.

### 2. [CLAUDE.md hard-constraint over coordinator-literal] docs/roadmap.md keeps P126 "In flight now", not "Landed recently"

- **Found during:** W6 close (this lane).
- **Issue:** The coordinator dispatch asked to move P126 into "Landed recently" dated 2026-07-19. But `.planning/CLAUDE.md` phase-close doctrine (and the PLAN.md W6 action) require a phase to stay "In flight now" until it is **verifier-graded GREEN**, not merely pushed — and this lane explicitly does NOT dispatch the verifier (CI in-flight, handed to the coordinator/L0).
- **Fix:** Placed P126 in "In flight now" (pushed 2026-07-19, CI in-flight, verifier pending); kept P125 in "Landed recently" (genuinely verifier-graded GREEN); advanced "Up next, in order" to P127/P128 + future arcs. STATE.md's phase counter optimistically advances to 13/15 — the doctrine explicitly sanctions this counter-vs-strip split. A follow-up leaf moves P126 to "Landed recently" once the verifier grades GREEN.
- **Files modified:** docs/roadmap.md (binding-free strip; walk.sh exit 0 confirmed).

## Known Stubs

None. (The 2 residual DRAIN-21 warnings are RETIRE_CONFIRMED redirect rows for already-deleted docs — a decided floor, filed forward, not a stub.)

## Self-Check: PASSED

- FOUND commits: 44783ebe, 65e8c497, d0753ef6, 5d097937, d093bc7f, e693deeb, 0270f91c, e8823049, 1df18239, 1ef508bf, 639ff67f, 588c1546
- FOUND: docs/development/roadmap.md (fixed in-place, not deleted)

---
*Phase: 126-docs-alignment-tooling-polish*
*Completed: 2026-07-19*
