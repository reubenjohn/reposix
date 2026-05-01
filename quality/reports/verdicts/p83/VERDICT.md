# P83 verdict — bus remote: write fan-out (SoT-first, mirror-best-effort, fault injection)

**Verifier:** unbiased subagent · zero session context
**Verdict:** **GREEN**
**Date:** 2026-05-01
**Milestone:** v0.13.0 — DVCS over REST
**Precondition:** P82 verdict GREEN at `quality/reports/verdicts/p82/VERDICT.md` (confirmed; commit 240ef45)
**Phase shape:** split into 83-01 (write fan-out core, 6 commits) + 83-02 (fault injection + close, 5 commits + 1 CI fix-forward)

---

## Catalog row grading

All 8 P83 catalog rows were re-graded by invoking `python3 quality/runners/run.py --cadence pre-pr` from a fresh shell (zero session context) AND by directly invoking each verifier shell.

| Catalog row                                                  | Verifier exit | Asserts | Status   | Source files |
| ------------------------------------------------------------ | ------------- | ------- | -------- | ------------ |
| `agent-ux/bus-write-sot-first-success`                       | 0 (0.52s)     | 4/4     | **PASS** | `crates/reposix-remote/src/bus_handler.rs:247-294` ; `tests/bus_write_happy.rs:316-340` |
| `agent-ux/bus-write-mirror-fail-returns-ok`                  | 0 (0.43s)     | 5/5     | **PASS** | `crates/reposix-remote/src/bus_handler.rs:295-313` ; `tests/bus_write_mirror_fail.rs:241-328` |
| `agent-ux/bus-write-no-helper-retry`                         | 0 (0.01s)     | 4/4     | **PASS** | `crates/reposix-remote/src/bus_handler.rs:471-496` (push_mirror — no retry constructs); verifier greps for `for _ in 0..` / `loop {` / `tokio::time::sleep` / `--force-with-lease`/`--force` AND requires `fn push_mirror` to exist |
| `agent-ux/bus-write-no-mirror-remote-still-fails`            | 0 (0.25s)     | 3/3     | **PASS** | `crates/reposix-remote/src/bus_handler.rs:142-151` (Q3.5 hint preserved through P83 write fan-out); `tests/bus_write_no_mirror_remote.rs` |
| `agent-ux/bus-write-fault-injection-mirror-fail`             | 0 (0.43s)     | 11/11   | **PASS** | `tests/bus_write_mirror_fail.rs::bus_write_mirror_fail_returns_ok_with_lag_audit_row` (`#[cfg(unix)]`; failing-update-hook fixture) |
| `agent-ux/bus-write-fault-injection-sot-mid-stream`          | 0 (0.43s)     | 8/8     | **PASS** | `tests/bus_write_sot_fail.rs::bus_write_sot_mid_stream_fail_no_mirror_push` (PATCH /2 → 500; mirror baseline ref unchanged) |
| `agent-ux/bus-write-fault-injection-post-precheck-409`       | 0 (0.41s)     | 4/4     | **PASS** | `tests/bus_write_post_precheck_409.rs::bus_write_post_precheck_conflict_409_no_mirror_push` |
| `agent-ux/bus-write-audit-completeness`                      | 0 (0.46s)     | 5/5     | **PASS** | `tests/bus_write_audit_completeness.rs` — dual-table assertion via SQLite count_audit_cache_rows (cache) + wiremock request log (sim wire) |

**Pre-pr cadence summary:** `22 PASS, 0 FAIL, 0 PARTIAL, 3 WAIVED, 0 NOT-VERIFIED -> exit=0`.
**Pre-push cadence summary:** `26 PASS, 0 FAIL, 0 PARTIAL, 0 WAIVED, 0 NOT-VERIFIED -> exit=0`.

Catalog `status: PASS` at `quality/catalogs/agent-ux.json` for all 8 rows is consistent with the runner output.

---

## Per-requirement evidence (DVCS-BUS-WRITE-01..06)

| Req                  | Plan/Task              | Catalog row                                       | Test artifact                                                                  |
| -------------------- | ---------------------- | ------------------------------------------------- | ------------------------------------------------------------------------------ |
| DVCS-BUS-WRITE-01    | 83-01 T04              | `bus-write-sot-first-success`                     | `tests/bus_write_happy.rs::happy_path_writes_both_refs_and_acks_ok`             |
| DVCS-BUS-WRITE-02    | 83-01 T03 + 83-02 T02  | `bus-write-mirror-fail-returns-ok`                | `tests/bus_write_mirror_fail.rs` (P83-02 T02 lands the `#[cfg(unix)]` test)    |
| DVCS-BUS-WRITE-03    | 83-01 T04              | `bus-write-sot-first-success`                     | `tests/bus_write_happy.rs` ASSERTION 5: `refs/mirrors/sim-synced-at` written  |
| DVCS-BUS-WRITE-04    | 83-01 T04              | `bus-write-no-helper-retry`                       | grep on `crates/reposix-remote/src/bus_handler.rs` rejects retry constructs    |
| DVCS-BUS-WRITE-05    | 83-01 T04              | `bus-write-no-mirror-remote-still-fails`          | `tests/bus_write_no_mirror_remote.rs::bus_write_no_mirror_remote_emits_q35_hint` |
| DVCS-BUS-WRITE-06    | 83-02 T02 + T03 + T04  | 4 fault-injection rows + audit-completeness row   | `bus_write_mirror_fail.rs` (a) + `bus_write_sot_fail.rs` (b) + `bus_write_post_precheck_409.rs` (c) + `bus_write_audit_completeness.rs` (dual-table) |

Per-requirement spot-check on the load-bearing contract bits:

- **SoT-first.** `bus_handler::handle_bus_export` (line 247) calls `crate::write_loop::apply_writes` BEFORE `push_mirror`. On non-`SotOk` outcome (line 257-262), `state.push_failed = true` and `return Ok(())` — mirror push NEVER attempted. Confirmed by `bus_write_sot_fail.rs:316-329` which asserts mirror baseline ref unchanged after SoT mid-stream fail.
- **Plain push (D-08).** `bus_handler::push_mirror` line 478: `["push", mirror_remote_name, "main"]`. NO `--force-with-lease`, NO `--force`. Verifier `bus-write-no-helper-retry.sh` greps for both flags as red-flags — confirms zero hits.
- **No helper-side retry (Q3.6).** `push_mirror` is a single subprocess invocation; on non-zero exit it returns `MirrorResult::Failed { exit_code, stderr_tail }`, NOT a retry loop. Verifier greps for `for _ in 0..`, `loop {`, `tokio::time::sleep` — zero hits.
- **Mirror-fail dual-state.** Lines 295-313 of `bus_handler.rs`: on `MirrorResult::Failed`, the helper writes `helper_push_partial_fail_mirror_lag` audit row + `log_token_cost` row + emits stderr WARN + `proto.send_line("ok refs/heads/main")`. **`refs/mirrors/<sot>-synced-at` is NOT written** (frozen). `refs/mirrors/<sot>-head` was already advanced inside `apply_writes`. Confirmed by `bus_write_mirror_fail.rs` ASSERTIONS 5-9: partial-fail audit row count = 1, mirror-sync-written count = 0, sim-head exists, sim-synced-at absent, mirror's main ref absent (rejected by failing update hook).
- **OP-3 dual-table audit.** `bus_write_audit_completeness.rs` queries both `audit_events_cache` directly (lines 254-291: started=1, accepted=1, mirror_sync_written=1, partial_fail=0, backend_instantiated≥1) AND uses wiremock request log as the byte-equivalent for `audit_events` on the SimBackend wire path (lines 305-337: exactly 1 PATCH; no POST/DELETE; the sim's `audit.rs` middleware writes 1 row per HTTP request). Both layers asserted; per-row counts honor the RESEARCH.md "Audit Completeness Contract" Row 1.

---

## CLAUDE.md update confirmation (QG-07)

`CLAUDE.md` line 35 contains the bus-write-fan-out paragraph (P83-01 T06 substrate; P83-02 T04 extension):

- Names `write_loop::apply_writes` as the shared SoT-write loop.
- Documents the plain push (NO `--force-with-lease`, D-08 RATIFIED).
- Documents `helper_push_partial_fail_mirror_lag` as the new audit op.
- Documents the `synced-at` FROZEN-on-partial-fail semantic.
- Documents Q3.6 RATIFIED no-retry contract.
- **Fault-injection addendum (last sentence)** names all four shipped tests + the OP-3 dual-table contract enforcement.

CLAUDE.md line 185 also reflects the v0.13.0 P83-01 ship in the helper-arch summary block. Bus URL form sentence (line 33) was already updated by P82.

---

## SURPRISES-INTAKE review

`.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` contains 1 P83-related entry:

- **`make_failing_mirror_fixture` core.hooksPath bug** (discovered-by P83-02-T02, severity LOW):
  - **What:** the P83-01 T05 fixture wrote a per-repo `hooks/update` shell hook but did NOT override `core.hooksPath`. On dev machines with `~/.gitconfig` setting `core.hooksPath = ~/.git-hooks`, the user-global hooks dir wins and the failing hook NEVER fires — the failing-mirror fixture silently becomes a passing-mirror fixture.
  - **Resolution:** P83-02 T02 fixture-fix added `git config core.hooksPath <bare>/hooks` to the bare repo's local config (local config wins over global). Code-confirmed at `crates/reposix-remote/tests/common.rs:161-182`.
  - **STATUS:** RESOLVED inline (Rule 1 — fixture bug from P83-01 T05; auto-fixed inside P83-02 T02; OP-8 eager-resolution applied because <5 minutes / no new dep).

The honesty trail is clean: a real fixture bug was caught at first run of T02, fixed inline, and documented for the verifier subagent's spot-check (this verdict).

No additional P83 SURPRISES entries; no GOOD-TO-HAVE entries from P83 (the in-phase eager-resolutions appear to have absorbed the polish-class items rather than deferring).

---

## Pre-push gate snapshot

```
$ python3 quality/runners/run.py --cadence pre-push
summary: 26 PASS, 0 FAIL, 0 PARTIAL, 0 WAIVED, 0 NOT-VERIFIED -> exit=0
```

All 26 pre-push catalog rows GREEN. No regressions from earlier milestones.

---

## B1 / H1-H5 / M1-M4 plan-checker fix honesty spot-check

| Plan-checker finding                                      | Severity | Verified status                                                                                                                                                                                                                                  |
| --------------------------------------------------------- | -------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **B1 — `apply_writes` consumed-by-value + `parsed.clone()`** | BLOCKER  | **FIXED.** Signature `pub(crate) fn apply_writes(... parsed: &ParsedExport)` at `crates/reposix-remote/src/write_loop.rs:148`. `bus_handler::handle_bus_export:254` passes `&parsed`. `grep -rn "parsed\.clone()" crates/reposix-remote/src/` returns ZERO hits. |
| **H1 — duplicated `<automated>` opening tag at 83-01 line 1313** | HIGH     | **FIXED.** Line 1313 of `83-01-PLAN.md` contains exactly ONE opening `<automated>` tag.                                                                                                                                                          |
| **H2 — missing `83-VALIDATION.md`**                       | HIGH     | **FIXED.** `.planning/phases/83-bus-write-fan-out/83-VALIDATION.md` exists (59 lines); promoted from RESEARCH.md § Validation Architecture per the plan-checker recommendation.                                                                  |
| **H3 — `helper_push_started` count divergence**           | HIGH     | **VERIFIED in test bodies.** `bus_write_no_mirror_remote.rs` asserts started=0 (Q3.5 hint exit BEFORE log_helper_push_started); `bus_write_sot_fail.rs:307-314` asserts started=1 (mid-stream fail happens AFTER started row). Counts honor the RESEARCH.md contract divergence. |
| **H4 — `helper_backend_instantiated` count assertion**    | HIGH     | **FIXED.** `bus_write_audit_completeness.rs:259-263` asserts `backend_inst >= 1`.                                                                                                                                                                |
| **H5 — `parsed.blobs` post-`apply_writes` use-after-move risk** | HIGH     | **RESOLVED via B1 borrow.** `bus_handler.rs:272-276` accesses `parsed.blobs.values()` AFTER `apply_writes` returns. With `&parsed` (borrow), this is sound (parsed lives until end of fn).                                                       |
| **M1 — Row-2 `comment` field annotation**                 | MEDIUM   | **FIXED.** `quality/catalogs/agent-ux.json` row `agent-ux/bus-write-mirror-fail-returns-ok` has explicit `comment` field documenting the FAIL-through-83-01-close design.                                                                        |
| **M2 — legacy `mirror_lag_partial_failure` stub removal** | MEDIUM   | **FIXED.** `grep -c 'mirror_lag_partial_failure' crates/reposix-cache/fixtures/cache_schema.sql` returns `0`. CHECK list line 51 has only the canonical `helper_push_partial_fail_mirror_lag`.                                                  |
| **M3 — cross-plan fixture rename risk**                   | MEDIUM   | **VERIFIED.** `make_failing_mirror_fixture` exists at `crates/reposix-remote/tests/common.rs:147` and is imported by `bus_write_mirror_fail.rs:46` — name pinned; T05's grep + cargo check would catch any rename drift.                          |
| **M4 — `chars_out` semantics consistency**                | MEDIUM   | **FIXED in code.** `bus_handler.rs:272-277` documents `chars_out` as stdout-only (the "ok refs/heads/main\n" line) in BOTH success and failure arms; stderr WARN is explicitly NOT counted. Identical token-cost semantics across both arms.    |

All 9 plan-checker findings (1 BLOCKER + 5 HIGH + 4 MEDIUM) verified FIXED in shipped code.

---

## D-01..D-10 ratification cross-check

D-01..D-10 from `83-PLAN-OVERVIEW.md` were the formal ratification of Q-A..Q-F open questions plus 4 derived decisions. Each is observable in the shipped code/tests:

| Decision   | Topic                                    | Verified location                                                                          |
| ---------- | ---------------------------------------- | ------------------------------------------------------------------------------------------ |
| **D-01**   | `apply_writes` lifts existing handle_export body    | `crates/reposix-remote/src/write_loop.rs:141-310` (verbatim lift with S1 mechanical replacements; caller-vs-callee responsibility table at lines 14-46)             |
| **D-02**   | Same audit-row contract on both arms (success vs partial-fail) | `bus_handler.rs:281-313` — token_cost row written in both arms; only the head/synced-at refs + audit-op-name diverge |
| **D-03**   | Schema delta no-migration (CHECK widens; existing rows untouched) | `cache_schema.sql:31-52` extended CHECK list; `audit.rs:319-322` documents the stale-cache.db best-effort fall-through |
| **D-04**   | `#[cfg(unix)]` gating for failing-update-hook fixture | `tests/common.rs:145` + `tests/bus_write_mirror_fail.rs:26,29,338,347` (with `#[cfg(not(unix))]` stub at L347) |
| **D-05**   | RESEARCH-pattern adoption (helper-driver scaffolding from `bus_write_happy.rs` reused across 4 P83-02 tests) | All 4 fault-injection tests reuse `make_synced_mirror_fixture`/`make_failing_mirror_fixture` + `count_audit_cache_rows` + sample_issues + seed_mock + sim_backend |
| **D-06**   | `git ls-remote` shell-out (PRECHECK A; T-82-01) | `bus_handler.rs:386-433` (with `--` separator + `-`-prefix reject); inherited from P82 |
| **D-07**   | Q3.5 hint verbatim wording                | `bus_handler.rs:149` (`configure the mirror remote first: \`git remote add <name> {mirror_url}\``) |
| **D-08**   | NO `--force-with-lease`, NO `--force`     | `push_mirror` line 478: `["push", mirror_remote_name, "main"]` — plain only. `bus-write-no-helper-retry.sh` verifier greps both flags as red-flags. |
| **D-09**   | Confluence non-atomicity (Pitfall 3 / D-09) | Documented at `bus_handler.rs:66-73`; tested via `bus_write_sot_fail.rs` SoT mid-stream fail (helper_push_accepted=0 because L262 returns BEFORE write_loop's success branch fires the accepted row) |
| **D-10**   | T-83-02 stderr-tail trim to ≤3 lines      | `bus_handler.rs:484-490` (last 3 stderr lines, joined by `" / "`); audit row `reason` includes the trim. Verified at `bus_write_mirror_fail.rs:264` (`stderr.contains("exit=")`) |

All 10 decisions ratified and observable in code.

---

## Commit verification

All 11 P83 commits + 1 fix-forward present on `origin/main`:

| SHA       | Description                                                                | Phase   | Origin |
| --------- | -------------------------------------------------------------------------- | ------- | ------ |
| 3857f9a   | mint 4 bus-write-core catalog rows + 4 TINY verifiers                      | 83-01 T01 | YES |
| 76cf527   | lift handle_export write loop into write_loop::apply_writes                | 83-01 T02 | YES |
| 836dc6f   | add helper_push_partial_fail_mirror_lag audit op                           | 83-01 T03 | YES |
| 6978369   | bus_handler write fan-out replacing deferred-shipped stub                  | 83-01 T04 | YES |
| b2d18cc   | bus write happy-path + no-mirror-remote regression integration tests       | 83-01 T05 | YES |
| 4b7be9d   | flip 3 P83-01 rows FAIL→PASS + CLAUDE.md update + dark-factory broaden     | 83-01 T06 | YES |
| c5175cd   | mint 4 P83-02 fault-injection rows + 4 TINY verifiers                      | 83-02 T01 | YES |
| 4bbd38c   | mirror-fail integration test with #[cfg(unix)] failing-update-hook fixture | 83-02 T02 | YES |
| 6f3742e   | SoT-fail tests — mid-stream 5xx + post-precheck 409                        | 83-02 T03 | YES |
| d8699b6   | bus_write_audit_completeness.rs dual-table audit assertion                 | 83-02 T04 | YES |
| bd903bb   | flip 5 P83 rows FAIL→PASS + CLAUDE.md addendum                             | 83-02 close | YES |
| fc46415   | broaden dark_factory_conflict_teaching_string_present to write_loop.rs     | CI fix-forward | YES |

The fc46415 fix-forward is justified: P83-01 T02's `apply_writes` lift moved the `git pull --rebase` teaching strings out of `main.rs` and into `write_loop.rs`. The pre-existing dark-factory test (`crates/reposix-cli/tests/agent_flow.rs::dark_factory_conflict_teaching_string_present`) still file-scoped grep'd `main.rs` and would have FAIL'd CI on a fresh build. The fix-forward broadens the file scope to scan `main.rs / write_loop.rs / bus_handler.rs`. Same dark-factory contract intent (the agent must encounter the recovery hint in stderr); broader file scope only.

---

## Honesty spot-checks (5 hard stress-tests)

1. **`apply_writes` borrows `&ParsedExport` (B1)?** — `crates/reposix-remote/src/write_loop.rs:148` reads `parsed: &ParsedExport`. `grep -rn "parsed\.clone()"` returns zero hits. **PASS.**
2. **`bus_handler::handle_bus_export` plain push (no `--force-with-lease`)?** — `bus_handler.rs:478` reads `["push", mirror_remote_name, "main"]`. **PASS.**
3. **mirror-fail test uses real `#[cfg(unix)]` failing-update-hook fixture (D-04)?** — `bus_write_mirror_fail.rs:26-29` `#[cfg(unix)] mod common;` + `#[cfg(unix)] mod test_impl`. `make_failing_mirror_wtree` (L140) calls `make_failing_mirror_fixture` from common.rs (L147, gated `#[cfg(unix)]`). The fixture writes a per-repo `hooks/update` shell that exits 1 (L184-194) AND overrides `core.hooksPath` (L161-182) so the failing hook actually fires. **PASS.**
4. **Audit-completeness test queries both audit tables (or proxy)?** — `bus_write_audit_completeness.rs:254-291` queries `audit_events_cache` directly via `count_audit_cache_rows` (SQLite COUNT(*)). Lines 305-337 use wiremock's `received_requests()` as the byte-equivalent of `audit_events` on the SimBackend wire path (the sim's `middleware/audit.rs` writes 1 row per HTTP request — documented at L293-300 of the test). **PASS — both layers asserted.**
5. **Legacy stub `mirror_lag_partial_failure` removed from `cache_schema.sql` per M2?** — `grep -c 'mirror_lag_partial_failure' crates/reposix-cache/fixtures/cache_schema.sql` returns `0`. Only `helper_push_partial_fail_mirror_lag` (line 51) remains in the CHECK list. **PASS.**

All 5 honesty spot-checks PASS.

---

## Advisory items (NON-blocking — phase-close ritual)

These items are visible from artifact inspection but are NOT verifier-blocking. They are the standard post-verdict close ritual that happens AFTER the verifier subagent grades GREEN:

1. **REQUIREMENTS.md DVCS-BUS-WRITE-01..06 checkboxes still `[ ]`.** P82's analogous close commit (240ef45 — `phase-close(P82): GREEN verdict + 82-01 SUMMARY + REQ checkboxes + STATE cursor`) updated REQUIREMENTS.md `[x]` checkboxes AFTER the verifier verdict landed. P83's close commit (bd903bb) only flipped catalog rows + updated CLAUDE.md; the REQ checkbox flip + SUMMARY + STATE cursor + RETROSPECTIVE entry are expected to land in the post-verdict close commit (orchestrator-level action).
2. **No `83-01-SUMMARY.md` / `83-02-SUMMARY.md` artifacts.** Same close-ritual cadence as item 1.
3. **STATE.md cursor still names "P82 SHIPPED ... next P83".** Same close-ritual cadence.

These are NOT plan-checker findings or quality-gate failures. They are deferred to the orchestrator's post-verdict close commit — the same pattern used at P78 → P82 close.

---

## Verdict

**GREEN.** Phase 83 (the riskiest in v0.13.0) is functionally and structurally complete:

- All 8 catalog rows PASS independently (`run.py --cadence pre-pr` and direct verifier-shell invocation).
- All 6 DVCS-BUS-WRITE-01..06 requirements have observable test coverage (1937 lines of integration tests across 6 files).
- B1 borrow shape applied; M2 legacy stub removed; H1+H2+H4 fixed; all H/M plan-checker items resolved.
- D-01..D-10 ratifications observable in code.
- CLAUDE.md updated with both the write-fan-out paragraph and the fault-injection addendum (QG-07 satisfied).
- SURPRISES-INTAKE entry recorded for the `core.hooksPath` fixture bug (honest discovery + inline fix).
- Pre-push gate 26 PASS / 0 FAIL.
- All 12 commits (11 P83 + 1 fix-forward) on `origin/main`.

Phase 83 closes. The orchestrator may now run the standard post-verdict close commit (REQUIREMENTS.md checkbox flip + 83-01-SUMMARY.md + 83-02-SUMMARY.md + STATE.md cursor advance + RETROSPECTIVE.md entry) and proceed to P84.

---

_Verifier: Claude (gsd-verifier subagent) · Verdict committed: 2026-05-01_
