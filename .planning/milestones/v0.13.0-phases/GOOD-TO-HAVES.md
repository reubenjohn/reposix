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
