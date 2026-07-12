# v0.13.0 GOOD-TO-HAVES — Part 2 of 8

> Split from `GOOD-TO-HAVES.md` for the file-size gate (OP-8 drain). Index: `../GOOD-TO-HAVES.md`. Entries preserved verbatim.

## GOOD-TO-HAVES-04 — mechanically verify the two permanently-yellow headline-number rows (8ms-cached-read, 89.1%-token-reduction)

**Discovered during:** D-CONV-2 (2026-07-04, quality/SURPRISES.md "Quality Convergence" — verdict 3-state honest contract)

**Size:** M (needs new verifier scripts + wiring, not just a catalog edit)

**Source:** `quality/catalogs/docs-reproducible.json` rows `benchmark-claim/8ms-cached-read` and `benchmark-claim/89.1-percent-token-reduction` are `kind: manual` with `verifier.script: null` — structurally unable to PASS by any runner invocation; they are the sole reason the `weekly` cadence verdict has never been brightgreen (both are P2, so they render as a yellow badge, not a blocking red one, once D-CONV-2's `--fail-on red` tolerance lands on `quality-weekly.yml`). `perf/headline-numbers-cross-check` (`quality/gates/perf/headline-numbers-cross-check.py`) already exists and is tagged `weekly` — it's the closest existing automation, but these two `benchmark-claim/*` rows themselves remain manual-only and were deliberately NOT downgraded/waived by D-CONV-2 (owner directive: "verifying them mechanically is exactly 'CI-verified headline numbers' from the launch-readiness milestone (OD-4 §3)").

**Acceptance:**

- `benchmark-claim/8ms-cached-read` and `benchmark-claim/89.1-percent-token-reduction` each get a real `verifier.script` (likely thin wrappers around/reusing `perf/headline-numbers-cross-check.py`'s existing extraction logic) that can mechanically PASS/FAIL against `docs/benchmarks/latency.md` and `docs/benchmarks/token-economy.md`.
- The `weekly` cadence badge reaches brightgreen without any `--fail-on red` tolerance once both rows PASS for real.
- Routed to the launch-readiness milestone per OD-4 §3 (not a v0.13.0 P90/P91 fix — mechanizing these two rows is scoped work, not a quality-convergence cleanup).

**Why deferred:** building+wiring two new mechanical verifiers (not just a catalog-row edit) is real engineering work, not the trivial-capability-loss simplification D-CONV-2 is scoped to do; it belongs in the milestone that owns "CI-verified headline numbers" as a deliverable.

**Default disposition:** Size M; default-defer to the launch-readiness milestone per OP-8 (M items default-defer) and OD-4 §3. Owner quote (2026-07-04): "makes weekly badge honestly green."

**STATUS:** OPEN

## GOOD-TO-HAVES-05 — deferral-pointer-linter misses word-form "Phase NN" pointers

**Discovered during:** Quality Convergence re-audit Round 1 (2026-07-04)

**Size:** S

**Source:** `quality/gates/structure/deferral-pointer-linter.sh` regexes (`not yet wired in P\d+`, `lands? (alongside|in) P\d+`, `substrate-gap-deferred`) miss the word form: `crates/reposix-jira/src/client.rs:381` carried `#[allow(dead_code)] // ... production retry wired in Phase 29` — a stale pointer to an already-shipped phase that the linter never saw. Proven non-hypothetical (that pointer sat stale for a milestone; the underlying dead-code finding is being fixed in the convergence fix wave).

**Acceptance:** linter catches `(wired|ships?|lands?) in Phase \d+` (word form) in `crates/`, with the same phrase-scoped PNN extraction + PLAN-artifact cross-reference as the existing patterns; existing allowlist-marker semantics unchanged; a fixture-style negative test or in-script self-check documents the new shape.

**Why deferred:** regex-scope expansion on a blocking pre-push gate deserves its own small change with false-positive review across the existing crates/ corpus, not a rider on an unrelated fix wave.

**Default disposition:** S — close in P90 (quality-framework honesty phase) or next debt-drain window.

**STATUS:** OPEN

## GOOD-TO-HAVES-06 — structure `wc -l` gate on run.py / verdict.py so the ≤350/≤400 caps are checked, not aspirational

**Discovered during:** P90 90-02 (2026-07-04)

**Size:** XS (one catalog row + a `wc -l` verifier)

**Source:** `quality/runners/run.py` carries a documented ≤350-line anti-bloat cap (header `:6-7`, `_freshness.py:4-7`, 90-RESEARCH-runner.md § 1), yet it sits at **459 lines** after 90-02 (was 429 pre-P90; 90-02 added ~+30 for the FW-07a/07b branch edits + the FW-08/F-K4b PASS-gate call-site, with the actual decision logic pushed into `_audit_field.apply_pass_gates` / `asserts_congruent` / `transcript_evidence_ok` per the helper-first rule). `verdict.py` is 367/≤400. The caps are prose in docstrings that NO gate enforces — so run.py silently breached its cap for two milestones and the only pressure toward helper extraction is agents reading the docstring. A mechanical `wc -l` structure gate (row + verifier asserting `run.py ≤ 350` and `verdict.py ≤ 400`, RAISE-only or waived-with-tracked_in for the current run.py overage until a dedicated run.py-decomposition phase) would make the cap real.

**Acceptance:** a `structure`-dimension catalog row + `quality/gates/structure/file-size-limits.sh`-style verifier (that gate already exists for other files — extend it or add a sibling) asserting the line-count caps on `run.py`/`verdict.py`; the current run.py overage is either waived with an honest `tracked_in` pointing at a run.py-decomposition phase, or the cap is RAISE-only until then. Do NOT hard-block pre-push on the pre-existing overage (that would turn every push RED before the decomposition phase exists — the deferral-loop the framework fixes prevent).

**Why deferred:** decomposing run.py to actually MEET the cap is real refactoring (extract the main()-loop persistence machinery into a helper), not a 90-02-scoped edit; and adding a hard-blocking gate before that refactor lands would RED every push. Filing the gate + the honest overage disposition is the XS down-payment; the refactor is the M follow-up.

**Default disposition:** XS — the gate+disposition close in a near-term structure/debt window; the run.py decomposition that makes the cap green is M (default-defer). Filed by 90-02 (sole Wave-B writer of this file per D90-12 item 4).

**Appended P96 Wave 3a — concrete run.py decomposition target + a free dead-condition cleanup:** `run.py` has since grown to **510 lines** (verified on HEAD `889c922`; was 459 at 90-02) — the cap drifts further every phase. The natural M-refactor extraction unit is the **persist-gate / pending-mint machinery** the P96 grade/persist split added (`run.py:483-500`: the `catalog_dirty → if args.persist: save_catalog else: pending_mint.append` block plus the validate-only `note:` printer), lifted into a sibling module (e.g. `quality/runners/_persist.py`) mirroring how `_audit_field` / `_freshness` already host the decision logic. **Zero-risk down-payment available now:** the guard at ~`run.py:491` `if pending_mint and not args.persist:` carries a redundant `not args.persist` — `pending_mint` is ONLY appended in the `else` branch of `if args.persist:` (`run.py:487-488`), so a non-empty `pending_mint` already implies `not args.persist`. Dropping the dead condition (`if pending_mint:`) is a harmless one-token cleanup with no behavior change — safe to fold into the next `run.py`-touching phase.

**Appended P96 close — `run_row` stale-artifact freshness (LOW; pairs with the persist-gate extraction above):** for verifiers that do NOT self-write their own JSON artifact, `run.py`'s `run_row` merges the PRIOR artifact via `setdefault`, which carries that artifact's STALE `ts` / `last_verified` forward even though the row's *grade* is freshly recomputed this run. Grade correctness is fail-safe (never stale — the status IS re-derived), but a TTL'd row's artifact would report an **understated freshness**: the row is really graded now, yet its timestamp claims the previous mint. LOW because no grading hazard, purely a reported-freshness lag. Fix: stamp a fresh `ts`/`last_verified` on EVERY `run_row` pass (not only when the verifier self-writes), so artifact freshness tracks the actual grading moment. Fold into the same `run.py`-touching phase as the persist-gate extraction above.

**STATUS:** OPEN

## GOOD-TO-HAVES-07 — move `parse_rfc3339` from `run.py` into `_freshness.py`

**Discovered during:** P90 90-04 (2026-07-04)

**Size:** XS-S (~10 lines Python — move one helper function + update the one import site)

**Source:** `quality/runners/verdict.py` needs `parse_rfc3339` (used for `minted_at`/`last_verified` comparisons) but the canonical implementation lives in `run.py`, forcing `verdict.py` to do a lazy `from run import parse_rfc3339` inside a function body rather than a clean top-level import — a minor layering smell (verdict.py importing from the runner it's meant to summarize, not a shared helper module). `_freshness.py` already exists as the shared-helper module for exactly this kind of cross-file utility.

**Acceptance:** `parse_rfc3339` relocated to `_freshness.py`; `run.py` and `verdict.py` both import it from there; the lazy in-function import in `verdict.py` removed; existing tests (`test_freshness_synth.py` and friends) still pass unchanged.

**Why deferred:** 90-04's task envelope was the honesty-rules PROTOCOL.md/schema docs work, not a `run.py`/`verdict.py` refactor; moving the function is a clean, low-risk change but touches both files' import graphs and deserved its own small change rather than a rider.

**Default disposition:** XS — always closes; fold into the next runner-touching phase (P92/P95 quality-framework window).

**STATUS:** OPEN

## GOOD-TO-HAVES-10 — `docs/reference/exit-codes.md` TL;DR table omits clap's own usage-error exit-2 layer

**Discovered during:** P90 90-06 (2026-07-04)

**Size:** XS

**Source:** Empirically confirmed during 90-06's real-test work: clap's own argument-parsing usage errors (e.g. missing required arg, unknown flag) exit 2 BEFORE reposix's own `anyhow`-based error handler ever runs — a distinct pre-dispatch exit-2 layer from the one `docs/reference/exit-codes.md`'s TL;DR table documents (which describes reposix's own handler's exit-2 semantics). The corresponding catalog claim text was corrected in 90-06 to reflect this distinction; the doc prose itself was not updated.

**Acceptance:** Add a sentence/footnote to the TL;DR table in `docs/reference/exit-codes.md` distinguishing "clap usage-error exit 2 (pre-dispatch)" from "reposix handler exit 2 (post-dispatch)".

**Why deferred:** doc-prose polish, not a test/catalog correctness issue (the catalog claim is already accurate); out of 90-06's real-test-writing envelope.

**Default disposition:** XS — always closes; fold into the next docs-touching phase or a `/reposix-quality-refresh docs/reference/exit-codes.md` pass.

**STATUS:** DEFERRED-v0.14.0 (docs-alignment-coupled) — P97 Wave A drafted the "Two exit-`2` layers" footnote, but the edit DRIFTS the bound **P0** docs-alignment row `docs/decisions/009-stability-commitment/exit-codes-locked` (`walk: STALE_DOCS_DRIFT sources_drifted=[0] on docs/reference/exit-codes.md`). Recovery is a `/reposix-quality-refresh docs/reference/exit-codes.md` rebind = a `doc-alignment.json` catalog mint + the `reposix-quality` binary — both **out of Wave A's no-catalog / no-cargo scope** (Wave B owns catalogs). Edit **REVERTED** to keep the P0 pre-push walk green; the footnote + its rebind must land together in a docs-alignment-touching v0.14.0 window (or a Wave-B catalog mint). No content lost — the footnote is re-appliable verbatim. **Lesson:** "safe doc-only XS" is NOT safe when the doc carries a bound docs-alignment claim.

## GOOD-TO-HAVES-11 — extend `subcommand_help_renders` (cli.rs) beyond 3/15 spot-checked subcommands

**Discovered during:** P90 90-06 (2026-07-04)

**Size:** XS-S

**Source:** The existing `subcommand_help_renders`-style test in `cli.rs` spot-checks only 3 of the CLI's 15 subcommands' `--help` output rendering; the other 12 (including the newer `attach`/`sync`) are untested for help-render sanity.

**Acceptance:** Parameterize the test over the full current subcommand list (or add the missing 12 as additional cases) so a broken `--help` render on any subcommand fails CI, not just the 3 currently covered.

**Why deferred:** 90-06's task was the 5 MISSING_TEST docs-alignment rows, not a general test-coverage expansion; widening this test is adjacent but distinct scope.

**Default disposition:** XS-S — close in the next CLI-touching phase (P91 adds `attach`/`sync` real-backend coverage and is a natural place to extend this test to include them).

**STATUS:** OPEN

## GOOD-TO-HAVES-12 — annotate `docs/reference/cli.md` exit-codes table: helper-only vs CLI-only examples

**Discovered during:** P90 90-06 (2026-07-04)

**Size:** XS

**Source:** Some of the exit-code examples in `docs/reference/cli.md`'s table are helper-only (`git-remote-reposix`) behaviors and others are CLI-only (`reposix` binary) behaviors, but the table doesn't currently label which is which — a reader could reasonably try an CLI-only exit code against the helper (or vice versa) and be confused when it doesn't reproduce.

**Acceptance:** Add a column or inline annotation to the exit-codes table in `docs/reference/cli.md` marking each row helper-only / CLI-only / shared.

**Why deferred:** doc-clarity polish noticed while writing the 90-06 exit-code tests; not itself a test-correctness gap, out of the MISSING_TEST-closure envelope.

**Default disposition:** XS — always closes; fold into the next docs-touching phase or a `/reposix-quality-refresh docs/reference/cli.md` pass.

**STATUS:** DEFERRED-v0.14.0 (docs-alignment-coupled) — same class as GTH-10: the Scope-column edit to `docs/reference/cli.md`'s exit-codes table DRIFTS the bound **P0** docs-alignment row `docs/reference/cli.md/exit_codes` (`walk: STALE_DOCS_DRIFT`). Recovery = `/reposix-quality-refresh docs/reference/cli.md` rebind (`doc-alignment.json` mint + binary = Wave B). Edit **REVERTED** to keep the P0 walk green. **Carry-forward finding (for the v0.14.0 fix):** the cli.md code-`2` row stays imprecise — it lists backend-unreachable/IO as exit `2`, but those are exit `1` for the `reposix` CLI per canonical `exit-codes.md` (exit `2` is clap pre-dispatch or the helper crash). The Scope-column annotation + the rebind should land together.

## GOOD-TO-HAVES-14 — helper `list for-push` reports `?` (unknown remote SHA), forcing a redundant export on every push

**Discovered during:** P91 litmus-REOPEN second-push mass-delete root-cause (2026-07-04)

**Size:** M

**Source:** `git-remote-reposix`'s `list`/`list for-push` arm hardcodes `? refs/heads/main` (remote value UNKNOWN). Because git can never conclude the ref is up-to-date, it re-runs the `export` helper on EVERY `git push` — even when the local ref already equals what git tracks in `refs/reposix/*`. That is exactly what produced the no-commit `feature done` / `reset` / `from 000…000` / `done` stream on a second push (the mass-delete trigger, now neutralized by the `saw_commit` guard in `5612fa6`). Reporting the real remote head SHA would let git short-circuit no-op pushes entirely (no helper spawn, no REST round-trip, no cache open).

**Acceptance:** `list for-push` reports the actual remote head SHA (derived from the cache's `refs/reposix/origin/main` or a cheap `list_changed_since`/head lookup) instead of `?`, so a genuinely-current push is skipped by git before the helper does any work. Add a test asserting a second `git push` with no new commit does NOT re-invoke `apply_writes` (e.g. zero new audit rows / zero REST calls). The `saw_commit` guard remains the correctness backstop; this is the efficiency + belt-and-suspenders layer.

**Why deferred:** computing an accurate remote SHA in `list for-push` touches the cache head-derivation + protocol arm (`main.rs`), is >1h, and the `saw_commit` fix already removes the data-loss danger. Efficiency/robustness improvement, not a correctness gap.

**Default disposition:** M — default-defer; fold into a v0.14.0 helper-protocol or perf window.

**STATUS:** OPEN

## GOOD-TO-HAVES-15 — consolidated file-size overages under the `structure/file-size-limits` waiver (expires 2026-08-08)

**Discovered during:** P91 91-02/91-04/91-05 (deferred-items.md), P91 T2 code-review pass, and P91 91-06 docs edits (2026-07-04)

**Size:** M (real split work across ~9 files, two languages)

**Source:** The `structure/file-size-limits` catalog row is WAIVED until 2026-08-08, and the list of files over their per-extension budget (`.rs`/`.md` 20000 chars, `.py` 15000 chars) has grown across the P91 window rather than shrunk:
- `crates/reposix-cli/src/doctor.rs` — 64780 chars (noticed by the P91 T2 code-review pass; single largest overage in the workspace).
- `crates/reposix-cli/tests/attach.rs` — 44330 chars (same T2 pass).
- `crates/reposix-confluence/tests/contract.rs` — 37844 chars (was 32583 pre-91-04; D91-08's hybrid-rewrite added ~5.3k; tracked in `deferred-items.md` § 91-04).
- `quality/runners/test_audit_field.py` — 18861/15000, `quality/runners/test_realbackend.py` — 16889/15000, `quality/runners/verdict.py` — 16498/15000 (all pre-existing, tracked in `deferred-items.md` § Wave-5 91-05).
- `.planning/milestones/v0.13.0-phases/REQUIREMENTS.md` — 20954/20000 (newly crossed the budget in 91-06's honest DVCS-ATTACH flip; the file had only 18 chars of headroom before that edit — any real correction would have crossed it).
- `docs/guides/troubleshooting.md` — 22339/20000 and `docs/reference/cli.md` — 22158/20000 (both pre-existing overages — 22020 and 21764 respectively before 91-06 — nudged slightly further by the LOW8/MED5/Pattern-C-sweep edits in this phase).

**Acceptance:** Split each file along its natural seams (`doctor.rs` by diagnostic-check group; `attach.rs` tests by reconciliation-case family, mirroring the pattern `reposix-remote`'s test suite already uses; `contract.rs` by connector-mode arm, e.g. hoist the `_live`/`_live_hierarchy` arms into a sibling `tests/contract_live.rs` per the 91-04 sketch; the three `quality/runners/*.py` files by function-group into sibling modules; the three docs files via progressive disclosure — child pages or linked docs — per project CLAUDE.md OP-4) until each is back under its budget, then confirm `structure/file-size-limits` passes un-waived for these paths.

**Why deferred:** each split is real design work (natural seam identification + import/export wiring, or in the docs case a nav restructure), not a mechanical trim; doing all nine properly is well over the 1-hour eager-fix budget, and the waiver already covers the group until 2026-08-08 so nothing is silently RED today.

**Default disposition:** M — default-defer to the pre-2026-08-08 waiver-renewal window (or a dedicated P95/P96 structure-hygiene pass); do NOT let the waiver silently auto-renew past its TTL without this list being re-triaged (per HYGIENE-02's precedent for waiver expiry discipline).

**STATUS:** OPEN

