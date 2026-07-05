# v0.13.0 GOOD-TO-HAVES

> **Purpose.** OP-8 +2 reservation slot 2 — improvements (clarity, perf, consistency, grounding) the planned phases observed but didn't fold in. Sized XS / S / M; XS items always close; M items default-defer to next milestone. Drained by P88 (good-to-haves polish + milestone close).

## GOOD-TO-HAVES-01 — extend `reposix-quality bind` to support all catalog dimensions

**Discovered during:** P79-02 (2026-05-01)

**Size:** S (~30-50 lines Rust)

**Source:** `quality/PROTOCOL.md` § "Principle A — Subagents propose; tools validate and mint" requires catalog rows to be MINTED via the `reposix-quality bind` verb (NOT hand-written into JSON). Today, `bind` only supports the `docs-alignment` dimension. P79-02 needed to mint a row in `quality/catalogs/agent-ux.json` (`agent-ux/reposix-attach-against-vanilla-clone`), but had to hand-edit JSON because `bind --dimension agent-ux` is not implemented. The hand-edit was annotated as "Hand-edit per documented gap (NOT Principle A)" in commit `1812647`'s message and in the row's `_provenance_note` field. Without this extension, every future agent-ux / release / code / structure / etc. catalog row will continue bypassing Principle A.

**Acceptance:**

- `reposix-quality bind --dimension <agent-ux|release|code|structure|docs-build|docs-repro|perf|security>` validates citations against the live filesystem (where applicable) and mints a row into the dimension's catalog at `quality/catalogs/<dimension>.json`.
- Refuses on invalid dimension or invalid citation.
- Test fixture in `crates/reposix-quality/tests/` covers at least one non-`docs-alignment` dimension end-to-end.
- The P79-02 hand-edit of `agent-ux/reposix-attach-against-vanilla-clone` is rebound via the new code path (provenance flag retained or auto-cleared).

**Why deferred from P79:** the gap is not load-bearing for `reposix attach` correctness — the row exists, the verifier runs, the catalog row reads PASS at phase close. Extending `bind` would have doubled P79-02's scope. Tracked here so the next milestone can close the Principle A gap cleanly.

**Default disposition for P88:** Size S; close in P88 if budget permits, else default-defer to v0.14.0 per OP-8 (M items default-defer; S items can either go either way).

**STATUS:** PARTIAL — Path A shipped post-milestone; Path B (remaining 7 dimensions: release/code/structure/docs-build/docs-repro/perf/security) DEFERRED to v0.14.0

**Rationale.** P88's scope is docs + catalog + shell only — no Rust code changes per the milestone-close charter (CLAUDE.md OP-9 + ROADMAP P88). Extending `reposix-quality bind` to all 8 catalog dimensions requires (a) ~30-50 lines of Rust spanning the `bind` verb's dispatch + per-dimension validation paths, (b) cross-dimension schema design (each catalog has its own row shape — agent-ux uses `command/expected.asserts/verifier.script` while doc-alignment uses `source/test/source_hash`), and (c) a new test fixture in `crates/reposix-quality/tests/` covering at least one non-`docs-alignment` dimension end-to-end. The work is well-scoped at S size but doesn't fit P88's pure-docs envelope; doing it here would double the phase's scope per the OP-8 "scope-creep-to-fit-the-finding" anti-pattern. v0.14.0 (`observability-and-multi-repo`) carries the gap forward with the existing GOOD-TO-HAVES-01 acceptance criteria intact. Until then, hand-edits to `agent-ux/release/code/structure/docs-build/docs-repro/perf/security` catalog JSON files continue to carry the `_provenance_note: "Hand-edit per documented gap (NOT Principle A)"` field documented at v0.13.0 P79 + reused throughout P80–P88.

If v0.14.0 budget tightens, can move to v0.14.x polish slot — the gap is operationally tolerable (every row that needs hand-editing carries the provenance flag, audit trail intact) and only blocks the cleaner provenance story, not any substantive Principle A invariant.

---

> Add new entries below this line.

## GOOD-TO-HAVES-02 — `DEFERRAL-REGEX-INTERVENING-WORDS` — deferral-pointer linter misses PNNs separated from the verb by intervening words

**Discovered during:** P89 89-05 (deferral-pointer linter, RBF-FW-05)

**Size:** XS (~5 lines shell — one widened regex alternative + a synthetic-scenario test line)

**Source:** `crates/reposix-cli/src/attach.rs:163` — the stderr message says "github/confluence/jira land alongside the integration tests in P79-03". The F-K6-verbatim pattern `lands? (alongside|in) P[0-9]+` requires the PNN to immediately follow the verb phrase, so "land alongside the integration tests in P79-03" is INVISIBLE to `quality/gates/structure/deferral-pointer-linter.sh` — its P79-03 pointer is never cross-referenced. Zero impact today (the same line's "not yet wired in P79-02" fragment matches pattern 1 and P79 resolves), but a future deferral written only in the intervening-words phrasing would silently escape the linter entirely — neither the orphan-PNN BLOCK nor the no-PNN BLOCK fires when no pattern matches at all.

**Acceptance:**

- Pattern 2 widened to tolerate a bounded run of intervening words (e.g. `lands? (alongside|in) ([a-zA-Z-]+ ){0,5}P[0-9]+` or equivalent), OR a documented decision that F-K6-verbatim stays and the phrasing convention is enforced editorially.
- Synthetic-scenario coverage: a line like `// this lands alongside the follow-up work in P999` BLOCKs.
- PNN extraction stays phrase-scoped (the widened fragment must not swallow adjacent allowlist-marker PNNs like the P91 in attach.rs:163's trailing comment).

**Why deferred from 89-05:** the three patterns are mandated F-K6 VERBATIM by 89-CONTEXT.md D-05a/b and the 89-05 plan; widening them is a design decision outside the task's envelope, and content cross-reference/pattern polish is already earmarked for P90/P95 (CONTEXT D-05c).

**Default disposition:** Size XS; XS items always close per CLAUDE.md OP-8 — fold into the P90/P95 polish slot that already owns deferral-linter content cross-reference.

**STATUS:** OPEN

## GOOD-TO-HAVES-03 — `run.py` runner has no per-row / per-dimension scope flag (only `--cadence`)

**Discovered during:** P89 89-04 (kind: shell-subprocess worked example, RBF-FW-02)

**Size:** S (~15-25 lines Python — add mutually-exclusive `--row <id>` / `--dimension <dim>` argparse options that further filter `in_scope` after the cadence filter in `main()`)

**Source:** `quality/runners/run.py:325` — the runner accepts ONLY `--cadence` (required, `choices=VALID_CADENCES`). There is no way to grade a single row or a single dimension. Consequence observed in 89-04: verifying that the runner copies `transcript_path` into the top-level artifact required exercising the real `run_row` -> artifact-synthesis path for exactly one row (`agent-ux/kind-shell-subprocess-worked-example`), but the only sanctioned CLI path — `python3 quality/runners/run.py --cadence pre-push` — fans out across EVERY pre-push-tagged row, several of which shell out to `cargo build --workspace` / `cargo clippy` / `cargo test` (dark-factory, reposix-attach, code-dim gates). Under the task's binding NO-cargo constraint + CLAUDE.md "Build memory budget" (VM crashed twice on cargo RAM pressure), the full cadence run was unsafe, so the end-to-end runner proof had to be driven via an isolated Python harness importing `run.run_row` directly (evidence in the 89-04 session scratchpad). The plan's step 9 even invoked a fictional `--row agent-ux/kind-shell-subprocess-worked-example` flag that does not exist.

**Acceptance:**

- `python3 quality/runners/run.py --cadence pre-push --row <id>` grades and persists exactly one row (still stdlib-only, still cadence-gated so `--row` must also match the cadence).
- Optionally `--dimension <dim>` scopes to one catalog file (mutually exclusive with `--row`).
- A single-row invocation of a cheap mechanical/shell row (e.g. the shell-subprocess worked example) completes without triggering any sibling cargo verifier — makes the "prove the runner preserves transcript_path" check a one-liner instead of a bespoke harness.
- Existing full-cadence behavior unchanged when neither flag is passed.

**Why deferred from 89-04:** adding a runner CLI flag is outside RBF-FW-02's envelope (kind + transcript convention), touches `main()` row-selection + argparse, and warrants its own `test_run_scope.py` unit coverage — an S-sized change that would expand 89-04's scope. The transcript_path preservation was proven via the isolated harness in-session, so nothing is blocked; this only removes future friction (and closes the gap the plan's fictional `--row` implied should exist).

**Default disposition:** Size S; default-defer to a P90/P95 runner-polish slot or v0.14.0. Cheap enough (~20 lines + one test) that a runner-touching phase should fold it in eagerly per OP-8 eager-resolution.

**STATUS:** OPEN

## GOOD-TO-HAVES-03 — `bind` cannot retarget/remove a cross-file cite; mental-model 27ms mirror + sibling-row drift

**Discovered during:** doc-alignment unblock lane (2026-07-04)

**Size:** S (three loosely-related grounding items surfaced together)

**(a) Tooling gap — no way to retarget/remove one cross-file cite.** BIND-RELOCATION-FIX (P89) makes `bind` replace SAME-file cites and append DIFFERENT-file ones — deliberately, since multi-source rows are legitimate. But that leaves no verb to RETARGET or REMOVE a cross-file cite that has drifted to vanished content. `docs/why/token-economy-89-1-percent` cited `docs/concepts/mental-model-in-60-seconds.md:17` for the 89.1% claim, but that file no longer carries 89.1% anywhere (the cite had drifted to a tree-diagram line). The only fix was a surgical hand-edit of the catalog JSON to point the second cite at the canonical `docs/benchmarks/token-economy.md:17`, then re-bind. A `bind --drop-source <file>` (or `unbind-source`) verb would keep such repairs inside Principle A instead of hand-editing.

**(b) mental-model still says `24 ms` (doc-lie vs the reconciled 27 ms).** QL-027's reconciliation was scoped to `docs/index.md` + `README.md`; `docs/concepts/mental-model-in-60-seconds.md` lines 21 and 69 still say `24 ms` cold init, now inconsistent with the canonical 27 ms. Deferred (not fixed in-lane) because editing those lines flips three currently-non-blocking `STALE_TEST_DRIFT` rows (`bootstrap-timing-24ms-vs-27ms`, `docs/why/cold-init-24ms-sim`, `docs/why/cached-read-8ms`) into blocking `STALE_DOCS_DRIFT` requiring rebinds — scope creep beyond the unblock lane. Also noticed: `docs/why/cached-read-8ms` cites mental-model:21 (a bootstrap-latency line, no `8 ms`) — a mis-cite worth fixing in the same pass.

**(c) Sibling checkout row not flipped.** `docs/tutorials/first-run/checkout-origin` (claim "`git checkout -B main refs/reposix/origin/main` succeeds after reposix init") is still bound to the config-only `dark_factory_sim_happy_path` (which never runs the checkout) yet the grader did NOT flip it, while it DID flip the near-identical `docs/index/git-checkout-branch-command`. Same latent weakness; should be flipped/rebound together when the P90/P91 round-trip lands.

**Default disposition:** Size S; (b) is the most user-visible (a headline-number doc-lie) — fold into the next docs-alignment refresh or the P90/P91 round-trip phase (which will already be rebinding the checkout rows). (a) and (c) ride the same P90/P91 quality-framework slot.

**STATUS:** OPEN

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

**STATUS:** OPEN

## GOOD-TO-HAVES-07 — move `parse_rfc3339` from `run.py` into `_freshness.py`

**Discovered during:** P90 90-04 (2026-07-04)

**Size:** XS-S (~10 lines Python — move one helper function + update the one import site)

**Source:** `quality/runners/verdict.py` needs `parse_rfc3339` (used for `minted_at`/`last_verified` comparisons) but the canonical implementation lives in `run.py`, forcing `verdict.py` to do a lazy `from run import parse_rfc3339` inside a function body rather than a clean top-level import — a minor layering smell (verdict.py importing from the runner it's meant to summarize, not a shared helper module). `_freshness.py` already exists as the shared-helper module for exactly this kind of cross-file utility.

**Acceptance:** `parse_rfc3339` relocated to `_freshness.py`; `run.py` and `verdict.py` both import it from there; the lazy in-function import in `verdict.py` removed; existing tests (`test_freshness_synth.py` and friends) still pass unchanged.

**Why deferred:** 90-04's task envelope was the honesty-rules PROTOCOL.md/schema docs work, not a `run.py`/`verdict.py` refactor; moving the function is a clean, low-risk change but touches both files' import graphs and deserved its own small change rather than a rider.

**Default disposition:** XS — always closes; fold into the next runner-touching phase (P92/P95 quality-framework window).

**STATUS:** OPEN

## GOOD-TO-HAVES-08 — trim/split `quality/reports/raise-list-p90.md` below the pre-commit WARN threshold

**Discovered during:** P90 90-05 (2026-07-04)

**Size:** XS

**Source:** `quality/reports/raise-list-p90.md` is 24,679 chars, above the 20k-char pre-commit WARN threshold (warns, does not block). The file is P90's SC5 deliverable (5-section RAISE LIST seeding P91/P92/P95 work) and is genuinely dense evidence, not padding, so trimming risks losing decision-ready detail; a split by section (waivers / dishonest-test baseline / magic-fixture schism / subagent-graded migration record) would keep each file under threshold without losing content.

**Acceptance:** Either (a) split `raise-list-p90.md` into per-section files under `quality/reports/raise-list-p90/` with an index, each under 20k chars, or (b) trim prose duplication while preserving every cited fact/table, bringing the single file under 20k chars. Pre-commit WARN clears.

**Why deferred:** the WARN is non-blocking and the file is fresh SC5 evidence other phases (P91/P92/P95) are about to consume verbatim — restructuring it mid-consumption risked breaking those phases' citations; better done as a deliberate follow-up once the RAISE LIST's consumers have read it once.

**Default disposition:** XS — always closes; fold into whichever of P91/P92/P95 finishes draining the RAISE LIST first (natural moment to restructure what's left).

**STATUS:** OPEN

## GOOD-TO-HAVES-09 — doc-note the WAL asymmetry between `reposix-core::open_audit_db` and `reposix-cache::open_cache_db`

**Discovered during:** P90 90-05 (2026-07-04)

**Size:** XS

**Source:** `reposix-cache`'s `open_cache_db` sets `PRAGMA journal_mode=WAL`; `reposix-core`'s `open_audit_db` does not. Investigated during 90-05's security-waiver renewal (audit-immutability verifier reads both DBs) and confirmed NOT a bug: the audit DB is single-writer-per-process and append-only, so WAL's concurrent-reader benefit doesn't apply the same way; the asymmetry is a deliberate-by-outcome, undocumented state.

**Acceptance:** A short code comment on `open_audit_db` (or a line in `docs/how-it-works/trust-model.md`) stating the asymmetry is intentional and why, so a future reader doesn't file it as a bug again.

**Why deferred:** zero functional risk, pure documentation debt; 90-05's task envelope was waiver disposition, not code-comment polish.

**Default disposition:** XS — always closes; fold into the next `reposix-core`/audit-touching phase.

**STATUS:** OPEN

## GOOD-TO-HAVES-10 — `docs/reference/exit-codes.md` TL;DR table omits clap's own usage-error exit-2 layer

**Discovered during:** P90 90-06 (2026-07-04)

**Size:** XS

**Source:** Empirically confirmed during 90-06's real-test work: clap's own argument-parsing usage errors (e.g. missing required arg, unknown flag) exit 2 BEFORE reposix's own `anyhow`-based error handler ever runs — a distinct pre-dispatch exit-2 layer from the one `docs/reference/exit-codes.md`'s TL;DR table documents (which describes reposix's own handler's exit-2 semantics). The corresponding catalog claim text was corrected in 90-06 to reflect this distinction; the doc prose itself was not updated.

**Acceptance:** Add a sentence/footnote to the TL;DR table in `docs/reference/exit-codes.md` distinguishing "clap usage-error exit 2 (pre-dispatch)" from "reposix handler exit 2 (post-dispatch)".

**Why deferred:** doc-prose polish, not a test/catalog correctness issue (the catalog claim is already accurate); out of 90-06's real-test-writing envelope.

**Default disposition:** XS — always closes; fold into the next docs-touching phase or a `/reposix-quality-refresh docs/reference/exit-codes.md` pass.

**STATUS:** OPEN

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

**STATUS:** OPEN

## GOOD-TO-HAVES-13 — doc-note: sandbox `rg` binary breaks under process substitution

**Discovered during:** P90 90-03 (2026-07-04)

**Size:** XS

**Source:** The `rg` (ripgrep) binary available in this agent sandbox is an emulation layer that breaks under process substitution (`<(...)`) constructs, unlike real ripgrep. Quality gates in this repo use `grep` by convention rather than `rg`, which sidesteps the issue, but the convention itself isn't documented anywhere an agent would find it before hitting the same breakage.

**Acceptance:** A short note (CLAUDE.md "What to do when context fills" area, or a `quality/PROTOCOL.md` aside) stating: prefer `grep` over `rg` for process-substitution-heavy shell in this sandbox; `rg`'s emulation here doesn't support `<(...)`.

**Why deferred:** pure agent-session grounding note, not a code or catalog change; noticed as an aside during 90-03's gate-authoring work, not itself in scope for the gate being authored.

**Default disposition:** XS — always closes; fold into the next CLAUDE.md-touching commit.

**STATUS:** OPEN

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

## GOOD-TO-HAVES-16 — `quality/runners/run.py` mutates the catalog in place with no `--dry-run` escape hatch

**Discovered during:** P91 91-06 deferred-items.md reconciliation (2026-07-04)

**Size:** S

**Source:** `run.py` writes verdicts back into the catalog JSON as a side effect of running (catalog-first state mutation), with no flag to preview what a run would change without committing the mutation. An agent (or a human) who wants to know "what would this cadence flip before I run it for real" has no way to ask without accepting the write.

**Acceptance:** Add a `--dry-run` flag to `run.py` that executes the full verifier sweep and prints the would-be verdict diff (row id, old status → new status) without writing the catalog file. Document the flag in `quality/PROTOCOL.md` alongside the existing runner-behavior description (the XS honest callout added in 91-06 names this gap; this entry is the actual flag implementation).

**Why deferred:** implementing a true dry-run mode means threading a write-suppression flag through every catalog-mutation call site in `run.py` and its shared runner helpers — real (if small) plumbing, not a one-line change, and orthogonal to 91-06's docs-only charter.

**Default disposition:** S — default-defer; natural fit for the next `quality/runners/` framework-touching phase (P95/P96 territory, alongside GOOD-TO-HAVES-06's `run.py`/`verdict.py` line-count gate).

**STATUS:** OPEN

---

## 2026-07-05 | `reposix init` should honour `REPOSIX_SIM_ORIGIN` (test-port hardcode) | discovered-by: P91 CI-red fix executor

**What:** `crates/reposix-cli/src/init.rs:55` (`translate_spec_to_url`) hardcodes `DEFAULT_SIM_ORIGIN` (`127.0.0.1:7878`) for `sim::<slug>` and — unlike `crates/reposix-cli/src/sync.rs:84` — does NOT honour the `REPOSIX_SIM_ORIGIN` env override. This forced the `agent-ux/real-git-push-e2e` gate to bind its sim on the exact default port 7878 (commit 5eae1c9): binding any other port makes init bake a 7878 URL into `remote.origin.url` while the sim listens elsewhere, so the fetch targets a dead port AND trips the egress allowlist. The task's original NOTICED item ("SIM_BIND=7781 hardcode port-collision risk") is a symptom of this asymmetry.

**Acceptance:** teach `translate_spec_to_url` (or its caller) to honour `REPOSIX_SIM_ORIGIN` when the backend is `sim`, mirroring `sync.rs:84-90` (`std::env::var("REPOSIX_SIM_ORIGIN").ok().filter(|s| !s.is_empty())`). Then `real-git-push-e2e.sh` can use a dedicated collision-proof port again by exporting `REPOSIX_SIM_ORIGIN`, and the init/sync inconsistency is closed. Add a unit test for the override (needs a non-env-mutating shape — e.g. pass the resolved origin as a param, or a serialized-env test lock as `history.rs` uses).

**Why deferred:** production-code change touching init URL generation + a non-trivial (env-mutation-safe) test; the CI-red hotfix pinned the port to 7878 instead, which is sequential-run-safe. Low value until someone needs distinct sim ports across concurrent gates.

**Default disposition:** S — default-defer; next `reposix-cli` init/sync-touching phase.

**STATUS:** OPEN

---

## 2026-07-05 | Owner may want to set the `JIRA_TEST_PROJECT` repo secret (KAN) | discovered-by: P91 CI-red fix executor

**What:** `.github/workflows/ci.yml` forwards `JIRA_TEST_PROJECT: ${{ secrets.JIRA_TEST_PROJECT }}` in the jira job env, but the repo has no such secret, so it arrives as the empty string. The code was hardened this session to treat empty-set as unset (falls back to `TEST`, commit 963f8bc), so CI is now robust either way. HOWEVER, per `docs/reference/testing-targets.md` + the ci.yml comment (D91-09), the owner's live JIRA project key is **KAN**, not `TEST` — the intent is for the real-backend smoke to target the project the tenant actually owns. Right now, absent the secret, the jira init-smoke targets `jira::TEST` (which passes because it's a config-string smoke that doesn't require the project to exist).

**Acceptance:** owner runs `gh secret set JIRA_TEST_PROJECT` (value `KAN`) so any future jira gate that lists/mutates real records targets the owned project. Purely an owner action; the code is already robust to it being present-or-absent.

**Default disposition:** XS owner-action — no code change. File for owner awareness only.

**STATUS:** OPEN (owner decision)

---

## 2026-07-05 | Coverage-as-asset: propose a `code/coverage-ratchet` catalog row | discovered-by: doctrine-coverage audit (owner request)

**What:** `cargo-llvm-cov` runs in CI (`.github/workflows/ci.yml` `coverage` job → Codecov, lines 366-386) but NO doctrine or gate watches, raises, or ratchets code coverage. Coverage is generated and then ignored — it can silently regress phase-over-phase with zero signal. The steward-window checklist (RUNBOOK ch.02 §A-L5) now carries a watch-only reminder, but watch-only is honest-but-weak: nothing prevents a slow decay.

**Acceptance:** mint one `code/coverage-ratchet` row in `quality/catalogs/` (dimension `code`, kind `mechanical`) + one verifier in `quality/gates/code/` that reads the lcov/Codecov result and flags when workspace line coverage drops below a stored ratchet floor (floor only moves UP — a green run bumps the floor; a drop fails/alerts). This is the framework's own one-row-one-verifier extension contract (the runner discovers by tag — no new top-level script, no new pre-push wiring). Start as `weekly` / `pre-pr` **alerting** to establish a trustworthy baseline, promote to blocking (`pre-release`) once the floor is stable.

**Why deferred / proposal-only:** inventing a blocking enforcement gate silently is exactly the anti-pattern the framework forbids. A coverage gate carries real design decisions — floor value, per-crate vs workspace, alert-vs-block, flaky-coverage tolerance — that deserve a scoped phase, not a bolt-on. Filed as a proposal (not a stealth gate) so the owner / next `quality/gates/code/`-touching phase scopes it deliberately.

**Default disposition:** M — default-defer; natural fit for the launch-readiness milestone (its headline-numbers work already mechanizes CI-verified metrics) or the next `quality/gates/code/` phase.

**STATUS:** OPEN

---

## 2026-07-05 | Cache delta-sync under-reports changed records, blocking `git rebase`'s lazy blob fetch | discovered-by: P92 T4 prove-before-fix executor

STATUS 2026-07-05 (D-P92-03): **REPRODUCED — CONFIRMED REAL, deterministic** (P93 Exec1, prove-before-fix lane). `not our ref <oid>` reproduced 4/4 in the same-wall-clock-second window (1 deterministic cursor-pin + 3 natural same-second runs, git 2.54 container); CLEAN in the 2s-gap negative control (`1 changed` → ordinary `CONFLICT (content)`). Exec2's non-repro was a timing straddle across a second boundary, not evidence of falseness. **Root cause + full transcripts + FAILING RED regression:** `.planning/phases/93-cache-coherence/93-DP2-REPRO-NOTES.md`. Trigger = sim `list_changed_since` seconds-truncation + strict `>` (`crates/reposix-sim/src/routes/issues.rs:138-139,180-183`); latent amplifier = `Cache::sync` Step 4 sources the tree from `list_records` (full current) while blob-materialization + `oid_map` cover only the `list_changed_since` delta (`crates/reposix-cache/src/builder.rs:293-328`), so an under-report leaves a dangling tree entry → `read_blob` `UnknownOid` → `not our ref`. NOTE: the earlier "even 2+ seconds after" phrasing was imprecise — the window is the SAME truncated second, not an unconditional lag. No production fix applied (coordinator-gated). D-P92-03 `[SELF]` close deferred to the coordinator.

**What:** During the T4 two-writer conflict repro (`.planning/phases/92-push-flow-correctness/92-T4-REPRO-NOTES.md`), after B's stale push is correctly rejected and B runs `git pull --rebase origin main`, the FETCH leg succeeds and advances `refs/reposix/origin/main` (ancestry preserved, HIGH-1 stays fixed), but `git rebase`'s own 3-way merge then fails: `fatal: git upload-pack: not our ref <oid>` / `could not fetch <oid> from promisor remote`. Root cause candidate: the cache's delta-sync (`list_changed_since` cursor query, `crates/reposix-remote/src/precheck.rs`) reports `0 changed (of 6)` even 2+ seconds after the conflicting writer's edit landed on the backend, so the specific blob the rebase's merge needs was never lazily materialized into B's cache object store — `git upload-pack` then genuinely doesn't have the object (`allowAnySHA1InWant=true` is already set per `crates/reposix-cache/src/cache.rs:713-726`, so this isn't a config gap; the object plain isn't there).

**Acceptance:** root-cause why `list_changed_since`/the cache's "since" cursor comparison misses a record that unambiguously changed (check the SIM's `updated_at` timestamp precision vs the cursor comparison operator — `>` vs `>=` — and/or whether the cache's delta-sync path actually writes new blob objects for records it DOES detect as changed vs just advancing a marker commit). Add a regression test exercising `git pull --rebase` (not just `git fetch`) to completion for the T4 two-writer scenario once root-caused.

**Why deferred:** different root cause than `cb630e5` (that fix was `Cache::open`'s git-config shell-out env pollution; this is the delta-sync cursor/materialization path) — Rule 4 territory (architectural, needs its own investigation), not a hand-wave-able quick fix. Also affects whether SC1's literal "completes step 6 + step 7" acceptance criterion can be met as worded — a later P92 wave or P93+ should decide whether to loosen that wording or fix the underlying gap.

**Default disposition:** M — default-defer; likely intersects P93's L2/L3 cache-coherence redesign (`refresh_for_mirror_head` / SotPartialFail work) since both touch the cache's post-write refresh semantics.

**STATUS:** OPEN

---

## 2026-07-05 | Quality-gate footgun: test-name-honesty marker silently ignored outside the 6-line lookback window | discovered-by: P92 Exec2 (SC5 test verification) | severity: MEDIUM

**What:** The `test-name-vs-asserts` agent-ux gate (`quality/gates/agent-ux/test-name-vs-asserts.sh`) only scans a 6-line window immediately after each `#[test]` or `fn test_` function signature for the honesty marker (`// HONEST: {reason}`). A correctly-present marker placed farther than 6 lines from the test declaration is silently ignored by the gate's regex scan, so the test reads as passing the gate despite carrying an honest marker — the gate's own contract is invisible once the marker drifts beyond the window. A P92 executor hit this: SC5 test read as passing until the marker was moved into range, surfacing the gate's silent distance requirement.

**Why deferred from P92:** SC5's task envelope is writing the test and embedding the marker; understanding the gate's placement-distance requirement belongs in the gate's own documentation + CLAUDE.md notes, not SC5's scope. The gate is correctly tuned (6 lines = function signature + 1-2 lines of setup is the typical case; enforcing tight placement is reasonable), but the distance constraint is undocumented.

**Sketched resolution:** (1) Document the 6-line lookback window in `quality/gates/agent-ux/test-name-vs-asserts.sh`'s header comment (state the window size, give an example of a correctly-placed marker and a mis-placed one outside the window). (2) Add a similar note to `quality/CLAUDE.md` under the test-name-honesty section (or wherever it documents the marker format) — name the placement rule so a future agent planning a honesty marker knows where to put it BEFORE writing the test. This is fix-it-twice per CLAUDE.md OD-3 meta-rule (notice is a deliverable; update the instructions to prevent recurrence).

**Default disposition:** MEDIUM — default-defer to the P95 quality-framework honesty/documentation pass; the gate itself is correct and documented at `quality/PROTOCOL.md` § "Honesty rules: test-name-asserts / marker format," but the placement-distance requirement is missing from both the gate script and CLAUDE.md.

**STATUS:** OPEN

---

## 2026-07-05 | Tighten `audit-immutability.sh` WAL grep to a single-line match | discovered-by: P92 security-waiver-flip executor | severity: LOW

**What:** `quality/gates/security/audit-immutability.sh` validates that `crates/reposix-cache/src/db.rs` sets `PRAGMA journal_mode=WAL` via two independent grep calls: `grep -q 'journal_mode' <db.rs> && grep -q '"WAL"' <db.rs>`. This checks both substrings exist *anywhere* in the file, not on the same statement. A `journal_mode` mention in a comment plus `"WAL"` in an unrelated log string would pass the check vacuously today.

**Acceptance:** Tighten the gate's WAL validation to a single-line or statement-scoped pattern (e.g. grep for `PRAGMA journal_mode.*WAL` or equivalent within a single line's context) so the gate confirms the pragma is actually set, not just that both words appear scattered in the file.

**Why deferred:** Low risk today (`db.rs` is stable and the current check passes correctly); the asymmetry matters only if `db.rs` becomes a high-churn area where a casual comment or log edit could trigger a false-pass.

**Default disposition:** LOW — fold into the next `quality/gates/security/` framework-touching phase (P95/P96) or when `db.rs` development density increases. No blocker today.

**STATUS:** OPEN | 2026-07-05 debt-drain triage: DEFERRED (confirmed, not actioned). Editing the gate's grep logic requires RE-RUNNING the gate (cargo) to confirm it still passes post-edit, which cannot be verified under this window's no-cargo firewall. Kept OPEN, routed to the P95/P96 security-gates window as already noted above.

---

## 2026-07-05 | Refresh stale header caveats in the two security gate scripts | discovered-by: P92 security-waiver-flip executor | severity: LOW

**What:** `quality/gates/security/allowlist-enforcement.sh` and `quality/gates/security/audit-immutability.sh` both carry header comments stating the gate "has NOT been executed via a real cargo run" — now false. Both gates ran green (real cargo, full pre-commit/pre-push sweep) on 2026-07-05; their catalog rows are PASS (commit 99b57b4). The header comments are lying to the next reader.

**Acceptance:** Update both scripts' header comments to reflect that they have been executed and passed (remove or update the "NOT been executed" caveat). Optionally note the execution date or the commit at which they first passed.

**Why deferred:** pure documentation update; the gates themselves are correct and running. This is fix-it-twice grounding (notice + update instructions so the next reader isn't misled).

**Default disposition:** LOW — fold into the next CLAUDE.md or `quality/gates/security/` documentation pass, or into whichever phase next touches these scripts.

**STATUS:** RESOLVED — 2026-07-05 debt-drain triage, commit `ae93cfb` (`docs(security-gates): refresh stale "not executed" caveats in gate script headers (P92 GTH)`). Both scripts' header comments now state the real 2026-07-05 execution (12/13 + 8/8 + 1/1 tests passing) and the P92 CI run 28735908764 confirmation, matching `quality/catalogs/security-gates.json`'s owner_hint fields for both rows. Comment-only change; no shell logic touched.

---

## 2026-07-05 | Drain or consciously renew `structure/file-size-limits` before its 2026-08-08 waiver expiry | discovered-by: P92 verifier + security-waiver-flip executor | severity: LOW

**What:** The `structure/file-size-limits` catalog row is WAIVED (warn-now/block-later) until 2026-08-08. On that date, the waiver expires and the gate will start BLOCKING all pushes where files exceed their per-extension budgets (`crates/reposix-cache/src/db.rs` already listed in `GOOD-TO-HAVES-15`). The milestone-close checklist (P97) must address this waiver before the date flips: either drain the overage files back under their limits OR make a conscious decision to extend the waiver (a tracked, intentional renewal, not a silent auto-renewal).

**Acceptance:** As part of the P97 milestone-close process, either (a) complete or schedule the work in `GOOD-TO-HAVES-15` to bring overaged files under their budget before 2026-08-08, OR (b) update the waiver row in `quality/catalogs/structure.json` with a documented reason and a new expiry date (ensuring the reason and date appear in the phase that renewed it). Do not let the waiver silently start blocking pushes mid-milestone.

**Why deferred:** the actual file-size splits are scoped to `GOOD-TO-HAVES-15` (M-sized real work); this entry is a hygiene reminder to ensure the waiver is consciously renewed or the work is scheduled before the date.

**Default disposition:** LOW (waiver-management) — fold into the P97 milestone-close checklist and governance review. The actual file splits remain M-sized and default-defer per `GOOD-TO-HAVES-15`.

**STATUS:** OPEN | 2026-07-05 debt-drain triage: LEFT AS-IS. This is a P97 milestone-close governance action (renew waiver or complete GTH-15 splits); no debt-drain window action taken. Flagged that it remains a hard, non-skippable P97 gate item ahead of the 2026-08-08 expiry.

## 2026-07-05 | Malformed `last_fetched_at` cursor bricks the fetch leg but only warns the push leg (inconsistent degradation) | discovered-by: P93 Exec1 (noticed while building the D-P92-03 repro) | severity: LOW

**What:** A corrupted/malformed `meta.last_fetched_at` value degrades two ways depending on which path reads it. `Cache::read_last_fetched_at` (`crates/reposix-cache/src/cache.rs:521-540`, used by the PUSH precheck) TOLERATES it — logs a `WARN` (`cache.last_fetched_at malformed: …; falling back to first-push semantics`) and returns `None`, so the push still works. But `Cache::sync` (`crates/reposix-cache/src/builder.rs:235-238`, the FETCH leg) parses the same value with `.map_err(Error::Sqlite)` and HARD-ERRORS, aborting the whole `git pull`/`git fetch` with `git-remote-reposix: cache.sync before upload-pack tunnel: sqlite: bad last_fetched_at …: premature end of input`. Observed live while pinning the cursor during the D-P92-03 repro (a `.999999999+00:00` value with no date parsed fine as a `chrono` fixed-offset but tripped the seed-vs-delta parse asymmetry).

**Why it matters:** the cache is committed-artifact state (OP-4 "no hidden state"), but nothing guarantees the cursor is never corrupted (partial write, manual poke, future migration). A corrupted cursor should degrade the SAME way in both paths — fall back to the seed / first-push full sync — not leave `push` working while `fetch`/`pull` is bricked with a raw sqlite error the agent can't act on. It's also a teaching-free error message (violates OD-3 ownership item 2).

**Sketched resolution:** make `sync()`'s Step-1 cursor parse mirror `read_last_fetched_at`'s tolerance — on parse failure, `WARN` + treat as absent (fall through to the `build_from` seed path) instead of `?`-propagating `Error::Sqlite`. Add a unit test in `delta_sync.rs` planting a garbage cursor and asserting `sync()` recovers via the seed path rather than erroring. Small, self-contained; NOT part of the coordinator-gated delta-sync coherence fix.

**Default disposition:** LOW — real but narrow (requires an already-corrupted cursor); good first-issue-sized robustness + error-message-teaching fix. Candidate for a P93 wave or the OP-8 absorption slots.

**STATUS:** OPEN

---

## 2026-07-05 | Consider a dedicated read-only "runner" subagent type distinct from `gsd-executor` | discovered-by: grounding-bug fix (coordinator dispatched `subagent_type: "executor"`, got "Agent type not found") | severity: LOW

**What:** ORCHESTRATION.md's coordinator rule 1 names four dispatch roles — reader, runner (build/test/litmus), executor (file write/edit), reviewer (diffs) — but only three of those roles have a dedicated registered `subagent_type` (`reader-digester`, `gsd-executor`, `gsd-code-reviewer`). "runner" has no dedicated type; today it's covered by overloading `gsd-executor` for routine build/test runs during implementation, and `gsd-verifier` for phase-close gate/litmus grading (`quality/PROTOCOL.md` Step 6/7). This was resolved as a DOCS fix (canonical role→subagent_type table added to `.claude/skills/coordinator-dispatch/SKILL.md` §2) rather than a new agent def, per the "don't invent, file it" rule.

**Why it might be worth a real type:** `gsd-executor` has `Edit`+`Write` tools. A coordinator that wants an arms-length "did the tests actually pass" signal — distinct from "the same agent that could edit the code to make it look like it passed" — currently has no `Bash`+`Read`-only, no-`Edit` runner to dispatch for that purpose. `gsd-verifier` fills this need for phase-close grading (it's genuinely a fresh, unbiased dispatch per `quality/PROTOCOL.md`), but mid-execution "run the test suite and report" (e.g. a `cargo nextest run -p <crate>` sanity check between plan waves) has no read-only equivalent — a coordinator either does it itself (violates ROUTE, DON'T WORK) or dispatches a full `gsd-executor` (which can silently patch a failing assertion instead of just reporting it failed).

**Sketched resolution:** a new `.claude/agents/gsd-runner.md`-style def (or reuse `reader-digester`'s tool set: `Read, Grep, Glob, Bash`, no `Edit`/`Write`) scoped to "run a named build/test/litmus command, report pass/fail + tail of output, never touch files." Small, mechanical, haiku-tier.

**Default disposition:** LOW — current `gsd-executor`/`gsd-verifier` split already covers correctness (verifier is the ungameable grading step); this is a defense-in-depth / mid-execution-hygiene nice-to-have, not a gap that blocks anything today. Owner-gated spend — do not create without explicit approval.

**STATUS:** OPEN

---

## 2026-07-05 debt-drain triage

The docs-only, <1h-sized GTH orphans (GOOD-TO-HAVES-02, -05, -06, -07, -09, -10, -12, -13; and the 2026-07-05 test-name-honesty marker entry above) were reviewed this window and LEFT to their already-routed phases (P90/P91/P95/refresh) — each either touches a linter/runner needing a real test run (cargo-adjacent) or a `docs/**` file under the mkdocs + doc-alignment regime routed to P95, so none were safe to eager-fix in a no-cargo, no-docs-alignment debt-drain window. See the companion `SURPRISES-INTAKE.md` for this same window's disposition of the surprises backlog and the branch-hygiene/PR-triage housekeeping entry.

---

## 2026-07-05 | `.git/hooks/pre-push` is a dead symlink to a nonexistent target | discovered-by: 2026-07-05 debt-drain branch-hygiene triage | severity: LOW

**What:** `.git/hooks/pre-push` is a symlink to `../../scripts/hooks/pre-push`, which does not exist (`ls` on the target errors ENOENT). It is currently INERT because `core.hooksPath` is set to `.githooks`, so the real active pre-push hook is `.githooks/pre-push` — the dead symlink never fires. Cosmetic debt only: no functional impact today, but a confusing artifact for anyone inspecting `.git/hooks/` directly (e.g. `git config --unset core.hooksPath` would silently resurrect a hook pointing at nothing).

**Acceptance:** Delete the dead `.git/hooks/pre-push` symlink (or replace it with a thin delegator to `.githooks/pre-push` if defense-in-depth against a future `core.hooksPath` unset is wanted).

**Why deferred:** `.git/` is not tracked by git itself (deleting a file there isn't a normal commit), so this needs either a `scripts/install-hooks.sh` update (if that script created the dangling symlink) or a one-off local cleanup step, not a tree-writer commit. Out of scope for a docs-only debt-drain window.

**Default disposition:** LOW — tidiness only, zero functional impact while `core.hooksPath=.githooks` remains set. Fold into the next `scripts/install-hooks.sh` touch or P95/P97 housekeeping pass.

**STATUS:** OPEN

---

## 2026-07-05 | Strategy 2 (defense-in-depth): reclassify delete-time `NotFound` as idempotent success | discovered-by: P93 DP-2 FIX lane (D-P93-02) | severity: LOW

**Deliberate deferral, NOT an oversight.** This is the SECOND candidate fix for the D-P93-01 ghost-`oid_map`-row HIGH. The coordinator chose **Strategy 1 (prune `oid_map` on sync)** as the shipped root-cause fix (commit `272882c`, ledger `D-P93-02`); Strategy 2 is filed here as a considered, independent defense-in-depth layer that was intentionally NOT taken this lane.

**What:** In `git-remote-reposix`'s write path, `execute_action`'s `PlannedAction::Delete` arm currently treats an `Error::NotFound` from `BackendConnector::delete_or_close` as a FAILURE (feeding `write_loop`'s `failed_ids` → `SotPartialFail`). Strategy 2 would reclassify it as idempotent success: a `Delete` whose target is already absent has reached its desired end state, so it should count as a no-op success, not a partial failure.

**Why it was NOT chosen as the primary fix (see D-P93-02 rationale):** Strategy 2 masks the symptom rather than fixing the root cause — the ghost `oid_map` row would still survive, `Cache::list_record_ids()` would still resurrect the dead id, and the planner would still emit + dispatch a phantom `DELETE` to the SoT on every push (wasted request + audit noise; a latent hazard against a real backend with soft-delete/restore semantics). It also broadens `SotPartialFail` semantics generally: ANY future NotFound-on-delete would silently reclassify to success, masking genuine "the record I meant to delete isn't there" bugs. Strategy 1 leaves the cache coherent and emits zero phantom Deletes.

**Why it's still worth having (defense-in-depth):** even with Strategy 1 shipped, a genuine two-writer delete race exists — agent A pushes a `Delete` for issue N while agent B's push (or an upstream actor) already removed N between A's precheck and A's write. That is NOT a ghost row; it's a real concurrent-delete collision that Strategy 1 does not address, and today it would surface as a `SotPartialFail` for a delete that actually achieved its goal. Strategy 2 would make that race degrade gracefully.

**Sketched resolution (~10-20 lines + a test):** in `execute_action`'s `Delete` arm, map `Err(Error::NotFound)` (from `delete_or_close`) to the same success outcome as a 2xx delete, with a `WARN`-level audit note distinguishing "deleted" from "already-absent". Add an integration test (mirror the `partial_failure_recovery.rs` wiremock harness) asserting a `Delete` against an already-404 id yields `ok refs/heads/main`, NOT `SotPartialFail`. Confirm the reclassification is scoped to the `Delete` arm only (a `NotFound` on Create/Update remains a real failure).

**Default disposition:** LOW — real but narrow (requires a genuine concurrent-delete race now that the ghost-row root cause is fixed); a good defense-in-depth follow-up. Default-defer to a v0.14.0 push-flow-robustness or the OP-8 absorption slots. Reversible (one match arm).

**STATUS:** OPEN (deliberate deferral — Strategy 1 shipped as the primary fix)

---

## 2026-07-05 | `badges-resolve` FAILs on pre-push (docs-build + structure dimensions) | discovered-by: P93 Wave 1 de-risk executor | severity: MEDIUM

**What:** The `badges-resolve` check (spanning the `docs-build` and `structure` quality
dimensions — README/docs badge URLs must resolve, e.g. shields.io, Codecov, CI status)
is FAILing on pre-push. Most likely a shields.io or Codecov transient network flake
rather than a genuine broken badge, but this has not yet been confirmed either way.

**Acceptance:** During the P94–P97 debt-drain window, re-run `badges-resolve` in
isolation (ideally on more than one occasion, spaced apart) to distinguish a real broken
badge URL from a transient upstream flake. If transient: consider whether the gate needs
a retry/backoff before failing, or a documented waiver note. If real: fix the underlying
badge URL/config.

**Why deferred:** confirming real-vs-transient requires multiple isolated re-runs over
time, which doesn't fit Wave 1's single-pass de-risk window; the fix (if any) is also
docs-build/structure-gate territory, orthogonal to this wave's durable-record + push-risk
scope.

**Default disposition:** MEDIUM — confirm real-vs-transient in the P94–P97 debt window,
then fix (if real) or waive with a documented reason (if transient/flaky-upstream).

**STATUS:** OPEN

---

## 2026-07-05 | Intake files don't name meta-infra (orchestration/agents/skills/hooks/runner-infra/coordinator-discipline) as in-scope | discovered-by: P93 Wave 1 de-risk executor | severity: LOW (deferred tangent)

**What:** `SURPRISES-INTAKE.md` and `GOOD-TO-HAVES.md`'s own framing describes what
phases discover in code/docs/catalogs, but neither file's scope language explicitly
names the ORCHESTRATION/agent-definition/skill/hook/runner-infra/coordinator-discipline
layer as fair game for an intake entry — a finding about, say, a hook footgun or a
coordinator-discipline gap could plausibly be read as "out of these files' scope" by a
literal reader, even though such findings are exactly the kind of thing this project's
dark-factory doctrine wants surfaced (cf. CLAUDE.md OP-4's "self-improving infrastructure"
and the OD-3 meta-rule "fix it twice"). A 4-edit proposal to close this gap has already
been drafted: (i) a scope note in `PRACTICES.md`, (ii) an addition to `ORCHESTRATION.md`
§5, (iii) a new `decision-procedures` entry DP-5, and (iv) a fix-it-twice cross-reference
in root `CLAUDE.md`.

**Acceptance:** Land the 4-edit proposal (PRACTICES.md scope note; ORCHESTRATION.md §5;
decision-procedures DP-5; root CLAUDE.md fix-it-twice pointer) as a tracked `/gsd-quick`,
scheduled AFTER the v0.13.0 tag lands (P97 GREEN) — NOT during the active P92→P97 close-out
drive, to avoid touching orchestration doctrine mid-milestone-close.

**Why deferred:** this is itself a meta-infra/tangent proposal (per root CLAUDE.md
Operating Principle 4, "high-leverage tangents are first-class to propose... the owner
gates the spend, never the surfacing") — surfacing it now, landing it later, post-tag,
keeps the active close-out drive's blast radius small.

**Default disposition:** LOW/deferred-tangent — land as a `/gsd-quick` AFTER the
v0.13.0 tag, not during this drive.

**STATUS:** OPEN

---

## 2026-07-05 | `.claude/hooks/dispatch-doctrine.sh` re-fires its full text on EVERY Agent dispatch with no session-scoped guard | discovered-by: P93 Wave 1 de-risk executor | severity: LOW (cheap fix)

**What:** `.claude/hooks/dispatch-doctrine.sh` fires its full doctrine text on every
single `Agent`/subagent dispatch within a session, with no "already applied this
session" marker to suppress repeats. This is the root cause of the dispatch-doctrine
reminder text re-appearing on every dispatch observed across recent sessions — it's
working as coded, just coded to repeat unconditionally.

**Acceptance:** Add a session-scoped marker file (e.g. under the session's scratch/temp
dir, or keyed off a session-id env var already available to hooks) that the hook checks
before firing; once set, subsequent dispatches within the same session skip the full
text (a one-line "doctrine already applied this session" ack is fine, or full silence).
Verify: dispatch two Agents in the same session, confirm the doctrine text fires once
and is suppressed (or abbreviated) on the second.

**Why deferred:** small (~5-15 lines shell), but touches `.claude/hooks/` — a
tooling/infra surface outside this wave's `.planning/` ledger + push-derisk scope, and
warrants its own tiny verification pass (confirm session-marker semantics don't leak
across genuinely separate sessions) rather than a rushed inline edit here.

**Default disposition:** cheap fix — pick up in the P94–P97 debt window or the next
`.claude/hooks/`-touching phase.

**STATUS:** OPEN

---

## 2026-07-05 | Confluence connector's `Record::labels` is not wired to real Confluence labels | discovered-by: P93 Wave 2a executor | severity: P2

**What:** The Confluence `BackendConnector` implementation does not populate
`Record::labels` from the real Confluence page label API — `Record::labels` reads back
empty/default regardless of what labels the page actually carries in Confluence Cloud.
Feature gap, not a correctness regression (nothing currently reads `Record::labels` for
Confluence records downstream), but the field silently lies about page state today.

**Acceptance:** Map the page's label API response (Confluence Cloud REST `GET
/wiki/rest/api/content/{id}/label` or equivalent) into `Record::labels` in the confluence
adapter (`crates/reposix-confluence/src/lib.rs`), for both the read path
(`get_record`/`list_records`) and any write-path round-trip. Add a contract test
(mirroring the existing `contract.rs` pattern) asserting a page with a real label
round-trips through `Record::labels`.

**Why deferred:** feature-gap noticed while filing Wave 2a's tracked items — wiring the
label API is real connector work (new REST call + response mapping + a contract test
against the live tenant) deserving its own scoped task, not a rider on the push-unblock
lane.

**Default disposition:** P2 — fold into the next `reposix-confluence`-touching phase or
a v0.14.0 connector-completeness window.

**STATUS:** OPEN

---

## 2026-07-05 | Sim same-second `list_changed_since` under-report (defense-in-depth precision fix) | discovered-by: P93 Wave 2b executor | severity: P3

**What:** The D-P92-03 *trigger* — the sim (and every second-resolution backend) drops
same-wall-clock-second writes from `list_changed_since` — is still live. The *amplifier*
(the load-bearing cache-coherence invariant break) is CLOSED and verified GREEN this wave:
`Cache::sync` Step 5 now upserts `oid_map` for the full `list_records` set (`e2a7297`), so
a dropped delta no longer produces an unservable OID. ADR-010's sub-decision ratified the
sim-precision fix as **optional defense-in-depth, explicitly NOT load-bearing for
coherence** ("a precision fix alone would NOT fix the amplifier and MUST NOT ship as the
sole remedy"). It is filed here rather than shipped in the Wave-2b minimal-fix window
because (a) it is optional per the ratified ADR, (b) it does NOT help the real backends —
Confluence/JIRA/GitHub `updated_at` are inherently second-resolution, so a sim-only
precision fix cannot close the trigger there (the committed invariant fix already does),
and (c) done correctly it is a multi-crate change with a wiremock-contract and
regression-test-comment ripple, past the OP-8 "<1h clean eager-fix" line.

**Benefit if done:** eliminates the *natural* same-second under-report on the sim (fewer
missed changes → more eager pre-materialization on the delta path) and de-risks OTHER
`list_changed_since` consumers, notably the push precheck's changed-set. Pure efficiency /
belt-and-suspenders; the coherence guarantee does not depend on it.

**Acceptance (the "sub-second precision on `updated_at`" option — minimal correct):**

- `crates/reposix-sim/src/routes/issues.rs:139` `now_rfc3339()` → `SecondsFormat::Nanos`
  (fixed-width, so SQLite lexicographic `>` stays monotonic).
- `crates/reposix-sim/src/routes/issues.rs:180` cutoff → `SecondsFormat::Nanos` to MATCH
  storage (mixed widths break lexicographic monotonicity — e.g. `10:08:10Z` sorts after
  `10:08:10.5…Z` because `'Z' > '.'`), and update the seconds-precision comment at
  `:175-178`.
- `crates/reposix-sim/src/seed.rs:99` seed timestamp → `SecondsFormat::Nanos` (keep ALL
  stored `updated_at`/`created_at` at one fixed width).
- `crates/reposix-core/src/backend/sim.rs:290` — the SimBackend CLIENT also truncates the
  OUTGOING cursor to `SecondsFormat::Secs`; it must send sub-second too, else the server
  cannot do better than seconds regardless of its own precision. THIS is why a
  server-only change would be a broken half-fix.
- Update the wiremock contract assertion at `crates/reposix-core/src/backend/sim.rs:1089`
  (`query_param("since", "2026-04-24T00:00:00Z")` → the Nanos rendering).
- Update the now-stale "the sim truncates the cursor to seconds" comments in
  `crates/reposix-cache/tests/delta_sync.rs` (~L497-498) and
  `crates/reposix-cache/tests/cache_coherence.rs` (~L303-305). The two coherence
  regressions SURVIVE this change unmodified in behavior: both pin the cursor to
  `upd.with_nanosecond(999_999_999)` (the max nanosecond of the write-second), so a
  full-precision compare still drops the write (`actual_ns < 999_999_999`) and
  `changed_ids.len() == 0` still holds — the invariant is then proven independent of
  timestamp resolution. Note: `>` with Nanos needs no de-dup (nanosecond wall-clock
  collisions across two HTTP writes are effectively impossible; the cursor is set to
  `Utc::now()` strictly after any observed write).
- Migration caveat: persistent sim DBs seeded by a pre-fix binary hold seconds-format
  rows; mixed widths would mis-sort. Acceptable — the sim DB is a rebuildable runtime
  artifact (`runtime/`), not a stability contract; re-seed on upgrade.

**Why deferred:** optional per ratified ADR-010; multi-crate (reposix-sim + reposix-core
client + a wiremock contract test) with no real-backend benefit; the Wave-2b charter
scoped a single-tree-writer minimal fix and the load-bearing coherence fix already shipped
and verified GREEN. Half-shipping (server-only) would be a lying "fix" that does not make
same-second writes visible.

**Default disposition:** P3 — pick up in the P94–P97 debt window or the next
`reposix-sim`-touching phase; re-dispatchable as its own scoped wave using the sketch above.

**STATUS:** OPEN

---

## 2026-07-05 | `verdict.py --phase N` is a pure rollup and does NOT scope the P0/P1 gate to phase-N rows | discovered-by: P93 RED-loop verifier (unbiased phase-close grade at `bf3bc9c`) | severity: P2

**What:** `quality/runners/verdict.py --phase N` reads the FULL catalog rollup and reports
overall RED/GREEN against the global P0/P1 gate — it does not filter or scope that gate to
rows tagged/owned by phase N. Concretely, at the P93 phase-close grading session,
`verdict.py --phase 93` reported RED with "103/112 P0/P1 green", but the 9 red rows mixed
P93's own (at-the-time) ungraded rows together with unrelated, pre-existing stale rows from
OTHER phases (`real-git-push-e2e`, `t4-conflict-rebase-ancestry`, `cargo-binstall-resolves`,
`subjective/dvcs-cold-reader`, `p92-mid-stream-litmus-t1-t4`, etc.). A verifier or executor
skimming the rollup's headline RED could easily misattribute the failure to the phase being
graded, or — in the opposite direction — rubber-stamp a genuine phase-N regression as "not
mine" and dismiss it because the rollup doesn't say which rows belong to N.

**Benefit if done:** a phase-close verifier gets a `--phase N` output that actually answers
"is phase N's own contract green," not "is the whole catalog green as of today" — removing
a class of misdiagnosis (both false-attribution and false-dismissal) at exactly the moment
(phase-close grading) where an accurate signal matters most.

**Acceptance:** `verdict.py --phase N` gains a phase-scoped sub-line (e.g. "phase-N rows:
X/Y green") computed from rows whose `id` or a row-level `phase` field matches N, printed
ALONGSIDE (not replacing) the existing global rollup line. Minimal-viable: derive the
phase-N row set from the existing `pNN-*`/`RBF-*` id-prefix convention already used across
catalogs (no new schema field required); if a durable per-row `phase` field is preferred
instead, land it as a superset covering both this and future phases' id-prefix drift.

**Why deferred:** discovered while grading, not implementing — fixing `verdict.py` itself is
a `quality/runners/`-framework change outside the P93 RED-loop mechanical re-run charter
(mint artifacts, don't touch runner code). Filed for the P94–P97 debt-drain window,
alongside the other `run.py`/`verdict.py` surface-area items already queued there
(GOOD-TO-HAVES `--dry-run` flag, `--row`/`--dimension` scope flags, the recurring
self-mutation bug in SURPRISES-INTAKE).

**Default disposition:** P2 — fold into the same P94–P97 `quality/runners/`-touching debt
window as the sibling `run.py` scope-flag work.

**STATUS:** OPEN

---

## 2026-07-05 | `dark-factory.sh sim` T1-T3 emits a confusing `blocked origin` WARN on git < 2.34 that reads like a failure at first glance | discovered-by: P93 RED-loop verifier (unbiased phase-close grade at `bf3bc9c`) | severity: P3

**What:** On a box whose on-path `git` is below the script's documented `>=2.34` floor
(e.g. 2.25.1), `dark-factory.sh sim`'s T1-T3 leg emits a `WARN git fetch --filter=blob:none
failed with status exit status: 128` plus a raw `error: cannot list issues for import:
blocked origin: http://127.0.0.1:7878/...` / `fatal: Unsupported command` stderr block
during the on-box init fetch attempt — yet the script still exits 0. This is by design: the
sim arm validates config wiring + the recovery-hint message text, not a full end-to-end
fetch (the real fetch is exercised separately, e.g. in a git-2.54 container for T4). But the
raw WARN + `fatal:`/`blocked origin` stderr, un-annotated, reads exactly like a fetch
failure on first glance — a future reader (human or agent) skimming the transcript could
reasonably conclude the gate is broken or that sim connectivity failed, when in fact exit 0
is correct and expected.

**Benefit if done:** a one-line annotation ("expected on git < 2.34 — validating config +
recovery-hint text only, not a live fetch; see T4 container arm for the real fetch") next to
the WARN removes a recurring "is this actually broken?" double-take for anyone reading a
dark-factory-sim transcript on an old-git box, without changing the gate's pass/fail logic.

**Acceptance:** `quality/gates/agent-ux/dark-factory.sh` (sim arm) emits an explanatory note
alongside the WARN when the on-box git fetch fails due to sub-2.34 version detection (it
already detects the version to decide script behavior elsewhere), OR the note is added to
the row's `owner_hint` / a comment near the WARN's emission site so a transcript reader has
the context inline instead of needing to cross-reference this file.

**Why deferred:** cosmetic / documentation-of-intent only — does not change the gate's
correctness or its exit code; discovered while grading (verifier read the transcript), not
implementing. Filed rather than eager-fixed because the RED-loop charter is a mechanical
artifact-minting re-run, not a `quality/gates/` script edit.

**Default disposition:** P3 — pick up alongside other `dark-factory.sh` polish items in the
P94–P97 debt window.

**STATUS:** OPEN

---

## 2026-07-05 | Arm the F-K4b congruence gate for the 5 P93 agent-ux verification artifacts — all carry `asserts_passed: []` | discovered-by: P93 phase-close verifier (unbiased re-verify) | severity: P3

**What:** All five P93 runner-minted verification artifacts (RBF-LR-01/02/04/05 +
D-P92-03) carry `asserts_passed: []`. This is legitimate today — `asserts_congruent` is a
documented no-op when either the expected or actual asserts list is empty
(`_audit_field.py:169-170`) — so these agent-ux mechanical gates inherit the fleet-wide
"exit-0-IS-the-assertion" posture, and the P93 verdict confirmed the gate honesty
line-by-line. But it means the F-K4b per-expected-assert congruence protection is
**dormant** on these five P0 rows: a gate that emitted structured `asserts_passed` entries
would arm F-K4b's real per-assertion congruence check instead of relying on bare exit-code
honesty alone.

**Benefit if done:** teaching the five agent-ux gate wrappers
(`p93-l2-l3-coherence-adr.sh`, `p93-cache-coherence.sh`, `p93-delta-sync-coherence.sh`,
`p93-l1-promise-reconciled.sh`, and the mid-stream litmus T1-T4 wrapper) to emit a
structured `asserts_passed` list (one entry per catalog-row `expected.asserts` item) arms
F-K4b's per-assertion congruence protection on these P0 rows, closing the same class of
"test name lies" gap that `agent-ux/test-name-vs-asserts` already polices for Rust tests.

**Why deferred:** discovered while grading (unbiased phase-close verify), not
implementing — editing the five gate scripts' output-parsing/emission logic is
`quality/gates/agent-ux/`-touching work outside the RED-loop/verify charter, and each
wrapper's assert-list needs a deliberate per-row pass rather than a blind mechanical edit.

**Default disposition:** P3 — fold into the next `quality/gates/agent-ux/`-touching phase
or the P94–P97 debt-drain window, alongside the sibling `verdict.py --phase` scoping item
already filed above.

**STATUS:** OPEN
