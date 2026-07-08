# v0.13.0 GOOD-TO-HAVES

> **CARRY-FORWARD BANNER — 2026-07-06 pre-v0.13.0-tag sweep.** v0.13.0 is **CLOSED-GREEN, tag imminent.** The P97 drain ledger below is historical; every entry/row still **OPEN / DEFERRED-\*** is a **live carry-forward** to the post-tag **v0.14.0 / v0.13.2 scoping session** for re-triage. STATUS/disposition refs to a now-closed **P9x** phase are historical, not live targets. The 4 **RESOLVING-P97** rows (completed at `302e8ec`) were DELETED this sweep — git is the archive (bound-to-live-state). Do NOT spin up a `v0.14.0-phases/` dir to hold these; they stay here until that scoping session ingests them.

> **Purpose.** OP-8 +2 reservation slot 2 — improvements (clarity, perf, consistency, grounding) the planned phases observed but didn't fold in. Sized XS / S / M; XS items always close; M items default-defer to next milestone. **Drained by P97 (good-to-haves polish + milestone close)** — was P88 in the original P78–P88 plan; renumbered when the milestone extended to P78–P97. (Per-entry "Default disposition for P88" lines below are historical filing-time notes; the actual drain phase is P97.)

---

## P97 OP-8 Slot-2 DRAIN LEDGER (2026-07-05, Wave A)

> **Terminal drain disposition for EVERY active entry below** — this is the OP-8 Slot-2
> drain-decision record (P97). Vocabulary: **RESOLVING-P97** (executed in P97 Wave A;
> the entry's own `STATUS` is flipped to RESOLVED + commit SHA), **DEFERRED-v0.14.0**
> `[-<routed-target>]`, **DEFERRED-post-tag** (land after the v0.13.0 tag, not
> mid-milestone-close), **DEFERRED-to-Wave-B-mint** (catalog-JSON-owned; the Wave-B
> milestone `--persist` mint owns it), **OWNER-ACTION** (not agent-executable). Standing
> rationale for the runner/gate/code XS deferrals (GTH-06, GTH-06-append, GTH-07): *avoid
> destabilizing the quality-runner/gate infra during the milestone gate; pairs with the
> run.py anti-bloat.* No entry silently skipped (OP-8). Wave B (milestone mint) owns
> catalogs/verdicts/RETROSPECTIVE — this ledger is the planning-side drain only.

| # | Entry (header topic) | Disposition | Rationale (one-line) |
|---|---|---|---|
| 01 | GTH-01 bind all catalog dimensions | DEFERRED-v0.14.0 (observability-and-multi-repo) | Path A shipped; Path B = 7 dims, ~30-50 lines Rust + fixture — out of the no-cargo P97 envelope |
| 03a | GTH-03 run.py `--row`/`--dimension` scope | DEFERRED-v0.14.0 | runner CLI flag + test; pairs w/ run.py anti-bloat |
| 03b | GTH-03 bind retarget cross-file cite + mental-model 24ms | DEFERRED-v0.14.0 | (b) 24ms→27ms doc-lie flip cascades 3 STALE rows into blocking rebinds → cargo/docs-alignment, unsafe in no-cargo window |
| 04 | GTH-04 mechanize 8ms / 89.1% headline rows | DEFERRED-v0.14.0-launch-readiness | new verifier scripts (OD-4 §3); M — makes weekly badge honestly green |
| 05 | GTH-05 deferral-linter word-form "Phase NN" | DEFERRED-v0.14.0 | regex on a blocking pre-push gate needs cargo re-run + false-positive corpus sweep |
| 06 | GTH-06 wc -l gate on run.py/verdict.py (+ P96 appends) | DEFERRED-v0.14.0 | runner/gate XS — standing rationale; run.py now 510+ lines, decomposition is the paired M |
| 07 | GTH-07 move `parse_rfc3339` → `_freshness.py` | DEFERRED-v0.14.0 | runner/code XS — standing rationale |
| 10 | GTH-10 exit-codes clap-2 vs handler-2 footnote | DEFERRED-v0.14.0 (docs-alignment-coupled) | footnote drifts bound P0 row `exit-codes-locked`; rebind = catalog+binary (Wave B) — **reverted**, see entry |
| 11 | GTH-11 extend `subcommand_help_renders` beyond 3/15 | DEFERRED-v0.14.0 | cargo test-coverage widening |
| 12 | GTH-12 annotate `cli.md` exit-codes helper/CLI/shared | DEFERRED-v0.14.0 (docs-alignment-coupled) | Scope-column edit drifts bound P0 row `cli.md/exit_codes`; rebind = catalog+binary (Wave B) — **reverted**, see entry |
| 14 | GTH-14 helper `list for-push` reports `?` | DEFERRED-v0.14.0 (helper-protocol/perf) | M; `saw_commit` guard already removes the data-loss danger |
| 15 | GTH-15 consolidated file-size overages (9 files) | DEFERRED-v0.14.0 (structure-hygiene, pre-2026-08-08) | M split work; waiver-renewal itself → Wave-B mint (see FLAG below) |
| 16 | GTH-16 run.py `--dry-run` flag | DEFERRED-v0.14.0 | S runner plumbing; pairs w/ run.py anti-bloat |
| 17 | REPOSIX_SIM_ORIGIN honored in init.rs | DEFERRED-v0.14.0 (reposix-cli init/sync) | prod-code + env-safe test |
| 18 | JIRA_TEST_PROJECT=KAN repo secret | **OWNER-ACTION** | `gh secret set JIRA_TEST_PROJECT KAN` — not agent-executable (surfaced in report) |
| 19 | code/coverage-ratchet proposal | DEFERRED-v0.14.0-launch-readiness | owner-gated spend; M new gate (OD-4 §3) |
| 20 | Cache delta-sync under-reports (M) | DEFERRED-v0.14.0 (L2/L3 cache-hardening) | correctness fine today (ADR-010 Step-5 upsert); efficiency residual LOW |
| 21 | Preamble-anchored marker scan (retire 6-line lookback) | DEFERRED-v0.14.0 (quality/gates/agent-ux) | 0/85 fns affected today; changes a P0-adjacent gate's scan loop, needs corpus re-run |
| 22 | Tighten `audit-immutability.sh` WAL grep | DEFERRED-v0.14.0 (security-gates) | editing the gate needs a cargo re-run to confirm still-passes — no-cargo firewall |
| 23 | Drain/renew `structure/file-size-limits` waiver | **DEFERRED-to-Wave-B-mint** | enumeration lives in `freshness-invariants.json` (catalog); see FLAG below |
| 24 | Malformed `last_fetched_at` cursor degradation | DEFERRED-v0.14.0 | cache code + test (cargo) |
| 25 | dedicated read-only "runner" subagent type | DEFERRED-post-tag | meta-infra (`.claude/agents/`), owner-gated spend — not mid-milestone-close |
| 26 | `.git/hooks/pre-push` dead symlink | DEFERRED-v0.14.0 (next `install-hooks.sh` touch) | `.git/`-local, not a tree-writer commit; inert while `core.hooksPath=.githooks` |
| 27 | Strategy 2 delete-time NotFound idempotent | DEFERRED-v0.14.0 (push-flow-robustness) | deliberate deferral — Strategy 1 shipped as primary; defense-in-depth only |
| 28 | Item A: intake files name meta-infra (4-edit /gsd-quick) | **DEFERRED-post-tag** `/gsd-quick` | per Item A's own acceptance; **DP-5 STALE** — see NOTE below |
| 29 | Item B: `dispatch-doctrine.sh` session-guard | **DEFERRED-post-tag** `/gsd-quick` | meta-infra (`.claude/hooks/`), own verification pass |
| 30 | Confluence `Record::labels` not wired | DEFERRED-v0.14.0-connector-completeness | P2 connector work (REST call + contract test) |
| 31 | Sim same-second `list_changed_since` precision | DEFERRED-v0.14.0 (reposix-sim) | P3, OPTIONAL per ratified ADR-010 (NOT load-bearing for coherence); multi-crate |
| 32 | `verdict.py --phase N` pure rollup | DEFERRED-v0.14.0 (quality/runners) | P2; pairs w/ run.py scope-flag work |
| 33 | `dark-factory.sh sim` blocked-origin WARN | DEFERRED-v0.14.0 | P3 cosmetic gate-script annotation |
| 34 | Arm F-K4b congruence on 5 P93 agent-ux artifacts | DEFERRED-v0.14.0 (quality/gates/agent-ux) | P3; per-row assert-list edit, gate-touching |
| 35 | `CONSULT-DECISIONS.md` 25k over soft limit | DEFERRED-v0.14.0 (intake-bloat split) | P3; actively-consumed log, split not in Wave A's 6-item docs scope |
| 36 | GitHub `list_records` self-recursion footgun | DEFERRED-v0.14.0-connector-trait-cleanup | P3 latent, zero current runtime bug |
| 37 | Split `doc_alignment.rs` 71k monolith | DEFERRED-v0.14.0 | LOW/M cargo refactor; pairs w/ crates-source file-size-gate exclusion removal |
| 38 | Split `cache_coherence.rs` 23.4k | DEFERRED-v0.14.0 | LOW/S cargo test split; bundle w/ crates-source budget rollout |
| 39 | `catalog-immutable-on-read` cadence coverage | DEFERRED-v0.14.0 | XS/LOW gate+runner; pairs w/ run.py persist-gate extraction |

**Tally (2026-07-06 carry-forward sweep):** DEFERRED-v0.14.0 (incl. `-target`) = 30 · DEFERRED-post-tag = 3 · DEFERRED-to-Wave-B-mint = 1 · OWNER-ACTION = 1 → **35 entry-topics across 36 STATUS lines** (GTH-01 is one PARTIAL line; all else OPEN/DEFERRED — all live carry-forward). The **4 RESOLVING-P97** rows (GTH-02 linter regex, GTH-08 raise-list split, GTH-09 trust-model WAL doc-note, GTH-13 grep-vs-rg note) landed RESOLVED at `302e8ec` and were **DELETED in the 2026-07-06 pre-tag sweep** (git is the archive, bound-to-live-state). **Still-live reversion note:** GTH-10 + GTH-12 were reclassified **DEFERRED-v0.14.0** — editing `docs/reference/exit-codes.md` / `cli.md` drifts their bound **P0** `docs-alignment/walk` rows, whose only recovery is a `doc-alignment.json` catalog rebind (Wave B + `reposix-quality` binary), out of Wave-A scope. Both edits were **reverted** to keep the P0 pre-push gate green; re-apply together with the rebind in v0.14.0.

**FLAG — file-size waiver understatement (routed to Wave B, NOT edited here):** The
`structure/file-size-limits` waiver in `quality/catalogs/freshness-invariants.json` says
*"Current violations: 10 files"* in both its `waiver.reason` and its
`claim_vs_assertion_audit` — but `bash quality/gates/structure/file-size-limits.sh
--warn-only` reports **50 violations** (41 `.md`, 6 `.py`, 3 `.sh`; verified 2026-07-05).
A waiver that understates violations 5× is a lying artifact. Because the enumeration lives
in a **catalog JSON** — editing it would tangle with the P97 milestone `--persist` mint
that Wave B owns — it is **DEFERRED-to-Wave-B-mint**: the Wave-B mint MUST correct the
`reason`/`claim_vs_assertion_audit` count to the real 50 (keep the `until:
2026-08-08T00:00:00Z` expiry) so the waiver stops lying. Do NOT let it auto-renew past TTL
carrying the stale "10".

**NOTE — Item A's "create DP-5" step is STALE:** the 4-edit proposal (entry #28) lists
*"(iii) a new `decision-procedures` entry DP-5"* — but **DP-5 already exists**
(`DP-5 — Tangent-vs-charter classification`, `.claude/skills/decision-procedures/SKILL.md:80`
+ `.planning/RUNBOOK-TO-V1/01-decision-procedures.md:29`). The post-tag `/gsd-quick` must
NOT create DP-5; it must reconcile **where the fix-it-twice meta-rule actually lands** —
root `CLAUDE.md` already carries the "when an owner catches a quality miss, fix it twice"
meta-rule under §OD-3, so Item A's step (iv) is partly satisfied and its scope needs a
re-read before execution.

**NOTE — badges real-vs-transient flap:** the transient-classifier follow-up (distinguish a
shields.io/Codecov flake from a genuinely broken badge URL) is **DEFERRED-v0.14.0** (fix the
classifier). The catalog row `docs-build/p94-badges-real-vs-transient` correctly stays
**NOT-VERIFIED** this milestone — its assertion requires ≥2 spaced re-runs to characterize
the flap, which an autonomous single-pass window cannot manufacture. Not edited (catalog =
Wave B territory); the working-file badges entry is already archived RESOLVED.

---

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

**Appended P96 Wave 3a — efficiency residual (LOW, distinct from the correctness question):** The
CREATE-path correctness half of the same-second under-report was proved a FALSE-ALARM this
window (SURPRISES `list_changed_since UNDER-materializes` entry → RESOLVED; CONSULT-DECISIONS
`2026-07-05 [SELF] list_changed_since ... false alarm`): a record missed by a truncated delta
window is still resolvable because ADR-010's Step-5 full-list upsert writes its `oid_map` row
and `read_blob` re-fetches lazily. What REMAINS is an efficiency cost, not a correctness gap:
the way the cache stays coherent across the `>`-boundary is precisely that `Cache::sync` Step 4
sources the tree from a FULL `list_records` recompute (`crates/reposix-cache/src/builder.rs:293-328`)
rather than a pure delta — so a single same-second write forces a whole-list recompute every
sync. A `>=`-with-dedup or a monotonic high-water cursor on `list_changed_since` would let the
delta path stay authoritative and drop the full-list fallback, saving one `list_records` call
per same-second sync. LOW: correctness is fine today; this is a hot-path allocation/IO saving
for the L2/L3 cache-hardening window (v0.14.0), not a bug.

**STATUS:** OPEN

---

## 2026-07-05 | Preamble-anchored marker scan (retire the fixed 6-line lookback in `test-name-vs-asserts.sh`) | discovered-by: P95 marker-footgun pass | severity: LOW

**What:** `test-name-vs-asserts.sh` detects a fn's `#[test]` attribute / `#[ignore]` gate / honesty marker via a FIXED `CONTEXT_LINES=6` lookback ending at the `fn` signature. A marker or `#[test]` attribute placed farther than 6 lines above the fn (e.g. behind a long `///` doc block) is silently ignored — documented as a footgun in the sibling RESOLVED entry above, mitigated by documentation but not eliminated.

**Acceptance:** Replace the fixed 6-line `sed` context extraction with a scan anchored to the fn's actual attribute/doc preamble — walk upward from the `fn` line through the contiguous `#[...]` / `//` / `///` / blank-line block and stop at the first non-preamble code line; treat THAT block (plus the signature line) as the context window. This removes the distance constraint entirely (the marker/attr can sit anywhere in the fn's preamble) while keeping the same "must be in the fn's own preamble, not a sibling's" scoping.

**Why deferred / proposal-only:** (a) Zero present value — the P95 probe found 0/85 current test-pattern fns affected, so it fixes nothing on today's tree. (b) It changes a repo-wide P0-adjacent pre-push gate's core scan loop; the upward-walk needs careful over-capture guards (don't bleed into a preceding item's body across blank lines) and a full-corpus re-run to prove no new RAISEs — more than a low-risk <1h change at milestone-close. (c) Design tension worth a deliberate decision: the original filer considered tight placement a FEATURE ("signature + 1-2 setup lines is the typical case"); a preamble-anchored scan loosens that. A future `quality/gates/agent-ux/` phase should weigh feature-vs-robustness and implement + corpus-verify in one scoped pass.

**Default disposition:** LOW — fold into the next `quality/gates/agent-ux/` framework-touching phase. No blocker; documentation already closes the invisibility harm.

**STATUS:** OPEN

---

## 2026-07-05 | Tighten `audit-immutability.sh` WAL grep to a single-line match | discovered-by: P92 security-waiver-flip executor | severity: LOW

**What:** `quality/gates/security/audit-immutability.sh` validates that `crates/reposix-cache/src/db.rs` sets `PRAGMA journal_mode=WAL` via two independent grep calls: `grep -q 'journal_mode' <db.rs> && grep -q '"WAL"' <db.rs>`. This checks both substrings exist *anywhere* in the file, not on the same statement. A `journal_mode` mention in a comment plus `"WAL"` in an unrelated log string would pass the check vacuously today.

**Acceptance:** Tighten the gate's WAL validation to a single-line or statement-scoped pattern (e.g. grep for `PRAGMA journal_mode.*WAL` or equivalent within a single line's context) so the gate confirms the pragma is actually set, not just that both words appear scattered in the file.

**Why deferred:** Low risk today (`db.rs` is stable and the current check passes correctly); the asymmetry matters only if `db.rs` becomes a high-churn area where a casual comment or log edit could trigger a false-pass.

**Default disposition:** LOW — fold into the next `quality/gates/security/` framework-touching phase (P95/P96) or when `db.rs` development density increases. No blocker today.

**STATUS:** OPEN | 2026-07-05 debt-drain triage: DEFERRED (confirmed, not actioned). Editing the gate's grep logic requires RE-RUNNING the gate (cargo) to confirm it still passes post-edit, which cannot be verified under this window's no-cargo firewall. Kept OPEN, routed to the P95/P96 security-gates window as already noted above.

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

---

## 2026-07-05 | `.planning/CONSULT-DECISIONS.md` is 25,074 chars, above the 20k soft limit | discovered-by: P94 catalog-first planning lane | severity: P3

**Source:** `.planning/CONSULT-DECISIONS.md` measures 25,074 chars — above the 20k-char
pre-commit WARN threshold (warns, does not block), same window as the already-filed
`raise-list-p90.md` file-bloat item above (24,679 chars). The file is an append-only log
of fable/E2 consult decisions (each entry is decision-ready evidence: chosen fork +
rationale + rejected alternatives + spot-checks), so trimming risks losing the audit trail
a future planner keys off. The most recent entry (the ratified pagination-truncation
prune-safety fork) is what P94 D1 executes against.

**Acceptance:** Fold into the intake-file-bloat split during the P96/P97 milestone-close
window, alongside the sibling `raise-list-p90.md` entry above. Either (a) split
`CONSULT-DECISIONS.md` into per-quarter or per-milestone shards under
`.planning/consult-decisions/` with an index (each under 20k chars), or (b) archive the
already-superseded/closed entries to a `.planning/archive/` companion, keeping only LIVE
decisions in the root file under 20k. Preserve every chosen-fork + rationale verbatim; do
not summarize away the rejected-alternatives detail. Pre-commit WARN clears.

**Default disposition:** P3 — maintainability, no runtime impact; fold into the P96/P97
intake-file-bloat split (sibling of `raise-list-p90.md`).

**STATUS:** OPEN

## 2026-07-05 | GitHub `list_records` → `list_records_complete` delegation is a self-recursion footgun | discovered-by: P94 Finish lane A | severity: P3

**What:** `crates/reposix-github/src/lib.rs::list_records` delegates UP to the
completeness-aware form and drops the flag: `Ok(self.list_records_complete(project).await?.records)`.
GitHub is safe ONLY because it ALSO provides a CONCRETE `list_records_complete` override (the
real pagination loop) right below. But the `BackendConnector` trait's DEFAULT
`list_records_complete` (`crates/reposix-core/src/backend.rs:280`) delegates the OTHER way —
it calls `self.list_records()`. So the two defaults form a delegation cycle broken only by
GitHub's concrete override: if a future edit ever REMOVES GitHub's `list_records_complete`
override (e.g. "the trait default is fine now"), `list_records` → default `list_records_complete`
→ `list_records` → … infinite-loops / stack-overflows at runtime, with no compile-time guard.
The inline comment ("Concrete override below, so no recursion through the trait default")
documents the hazard but does not enforce it.

**Acceptance:** Restructure so the default cannot self-delegate into a live cycle. Options:
(a) invert the direction so `list_records_complete` is the primitive and `list_records` is
the only delegator (make the trait's default `list_records` call `list_records_complete`, and
require backends to implement `list_records_complete` — the opposite of today's default), or
(b) keep the shape but add a `#[test]` / debug-assert that a mock backend using the trait
default for BOTH methods does NOT recurse (a recursion-guard sentinel), or (c) at minimum a
doc-comment cross-link on the core default warning that overriding `list_records` to delegate
to `list_records_complete` requires a concrete `list_records_complete`. Pure hardening — no
current runtime bug.

**Default disposition:** P3 — latent footgun, zero current runtime impact (GitHub's concrete
override is present); fold into a v0.14.0 connector-trait cleanup or the OP-8 good-to-haves
drain.

**STATUS:** OPEN

---

## 2026-07-05 | Split the `doc_alignment.rs` 71k monolith into per-verb modules (bind/walk/status/merge) | discovered-by: P96 Wave 3a (OP-8 Slot 1 hygiene) | severity: LOW

**Size:** M (module carve-out + re-export shim; no behavior change)

**Source:** `crates/reposix-quality/src/commands/doc_alignment.rs` is **71,288 chars / 1,716
lines** on HEAD `889c922` — the single largest source file in the workspace, hosting every
`doc-alignment` verb (bind, propose-retire, confirm-retire, mark-missing-test, plan-refresh,
plan-backfill, merge-shards, walk, status) plus the walker's drift state machine. The prose
`≤350`-style caps that pressure `run.py`/`verdict.py` (GOOD-TO-HAVES-06) have no analogue here,
and the file-size gate does NOT catch it: `quality/gates/structure/file-size-limits.sh` excludes
`^crates/.*\.rs$` outright (deferred to a future milestone's crates-source-budget cleanup, per
the gate's own exclusion comment) — so this is not "warn-only," it is currently UNMEASURED. The
monolith makes every walker/bind change a merge-conflict magnet and buries the drift-state logic
(the `source_hashes.is_empty()` false-negative in the sibling SURPRISES entry lives here).

**Acceptance:** carve per-verb modules (`doc_alignment/{bind,walk,status,merge,...}.rs`) behind a
thin `doc_alignment/mod.rs` re-export so callers are untouched; the walker's drift state machine
becomes its own unit. No behavior change; existing `reposix-quality` tests stay green. Pairs with
the eventual removal of the `^crates/.*\.rs$` file-size-gate exclusion (then this file would fail
a real budget).

**Why deferred:** cargo-touching refactor of a load-bearing binary, orthogonal to this no-cargo
hygiene window; best done in a quality-framework phase that already has the crate built.

**Default disposition:** LOW/M — no runtime impact, pure maintainability; do it in the same
window that retires the crates-source file-size-gate exclusion so the split is enforced, not just
performed.

**STATUS:** OPEN

---

## 2026-07-05 | Split `cache_coherence.rs` (23.4k) when the crates-source file-size budget is enforced | discovered-by: P96 Wave 3a (OP-8 Slot 1 hygiene) | severity: LOW

**Size:** S (test-file split by scenario cluster)

**Source:** `crates/reposix-cache/tests/cache_coherence.rs` is **23,415 chars** on HEAD `889c922`
— over the generic `*.md`/`*.rs` 20k progressive-disclosure budget. **Accurate scope note (a
correction to the loose "20k soft-limit waiver expires" framing):** this file is a `crates/**.rs`
path, which `quality/gates/structure/file-size-limits.sh` EXCLUDES entirely (`^crates/.*\.rs$`),
so it is neither flagged nor warned today — the relevant trigger is the future milestone that
RETIRES that crates-source exclusion (the same cleanup GOOD-TO-HAVES-06 and the `doc_alignment.rs`
split above both wait on), not a warn-only waiver expiry. When that budget lands, this test file
(same-second CREATE/UPDATE/DELETE cache-coherence repros, incl.
`same_second_created_record_resolvable_after_delta_sync`) should split by scenario cluster.

**Acceptance:** split into per-scenario test modules (e.g. `cache_coherence/{create,update,delete,
delta_sync}.rs` or split files) each under the source budget, preserving every existing test fn
verbatim; `cargo nextest run -p reposix-cache` stays green.

**Why deferred:** cargo-touching test refactor, no correctness value on its own; only worth doing
alongside the crates-source-budget enforcement so it is checked, not aspirational.

**Default disposition:** LOW/S — cosmetic/maintainability; bundle with the crates-source file-size
budget rollout.

**STATUS:** OPEN

## 2026-07-05 | `catalog-immutable-on-read` gate covers only the `pre-commit` cadence in its real-tree check, not `pre-release`/`pre-push` | discovered-by: P96 phase-close (verdict NOTICED #2 review) | severity: LOW

**Size:** XS (extend one gate's real-tree assert across cadences)

**Source:** `quality/gates/structure/catalog-immutable-on-read.sh` proves the self-mutation fix with
4 asserts — 3 hermetic synthetic-flip cases + 1 real-tree breadth check. The real-tree check runs
`python3 quality/runners/run.py --cadence pre-commit` (validate-only) and asserts zero catalog bytes
change. But the bug it guards actually bit the **`pre-push`** cadence (the `docs-build.json` flip),
and the milestone mint runs the **`pre-release`** cadence — neither is exercised by the gate's
real-tree assert. The P96 verdict NOTICED #2 flagged this: `pre-commit` was chosen to avoid
recursion + double-cargo, and the verifier closed the residual gap MANUALLY with a one-off real
`--cadence pre-push` validate-only run (zero drift). So the byte-immutability invariant is only
*automatically* regression-guarded on `pre-commit`; `pre-release`/`pre-push` rest on the hermetic
synthetic-flip proof.

**Acceptance:** extend the real-tree breadth assert to loop `--cadence` over
`{pre-commit, pre-push, pre-release}` (validate-only, no `--persist`), each asserting zero catalog
byte drift — while PRESERVING the cargo-free / no-recursion property that motivated the original
`pre-commit`-only choice (skip or stub any cargo-shelling row so the gate stays fast + hermetic).
Document the cadence coverage in the gate header.

**Why deferred:** widening a blocking structure gate deserves its own change with a check that the
added cadences don't drag cargo verifiers into the gate's runtime (the reason `pre-commit` was
picked) — not a P96-close rider.

**Default disposition:** XS/LOW — fold into the next `quality/runners`- or `catalog-immutable`-
touching window; pairs with the run.py persist-gate extraction (GOOD-TO-HAVES-06).

**STATUS:** OPEN

---

## 2026-07-06 | env-gate/`minted_at` load-fragility: an env-missing skip advances `last_verified` to run-time, making a `minted_at`-less post-P90 row unloadable once skipped | discovered-by: P97 milestone-close (OP-9 RETROSPECTIVE distillation, verify-against-reality on the 9th-probe mint) | severity: MEDIUM

**Size:** S (a `run.py` write-path change + regression test — the READ/skip-side mirror of the write-path load-refusal item already filed in `SURPRISES-INTAKE.md`).

**Source / mechanism:** The honesty contract says an env-gated skip "fails closed to `NOT-VERIFIED` but preserves `last_real_grade` + `skip_reason: env-missing`" (quality/CLAUDE.md § Honesty rules). But in practice the env-missing skip path ALSO stamps `last_verified` to the run-time clock. For a pre-P90 legacy row that carries no write-once `minted_at`, `_audit_field.validate_row`'s anchor heuristic is `is_new = lv is None or parse_rfc3339(lv) >= CUTOFF` — so once the skip advances `last_verified` past the 2026-07-05 cutoff, the row flips `is_new=True`, now demands a `claim_vs_assertion_audit`, and FAILs/refuses at the next load. A silent time-bomb: it does not bite on the run that moves the clock, it bites the run after. This is the exact landmine `09e10c1` closed for `code/cargo-clippy-warnings` and that `f37f468` closed for `agent-ux/cadence-pre-release-real-backend` (the milestone 9th-probe mint backfilled its missing `minted_at` = its P89 89-01 mint time `2026-07-04T03:08:07Z`, its `last_verified` having just been advanced by the env-gated skip). **Two instances fixed one-at-a-time; the CLASS remains** for any other pre-P90 row with null-or-pre-cutoff `last_verified` + no `minted_at` that gets re-graded (or env-skipped) in a blocking cadence.

**NOTE on the scaffold citation:** the P97 close task scaffold attributed the instance fix to `30b4910` — that commit is the honest file-size-waiver enumeration (10→50 files), NOT a `minted_at` fix. The real instance fix is `f37f468` (grounded against `git show`); `09e10c1` is the prior sibling instance. Recorded so the next reader does not chase the wrong SHA.

**Relationship (dedupe):** DISTINCT from — but the read-side complement of — `SURPRISES-INTAKE.md` § "2026-07-05 | `--persist` mint path should refuse to write a row it would reject at load" (that is the WRITE path: the mint refusing to persist a `minted_at`-less row). This item is the READ/env-skip path: the skip that CREATES the un-loadable state by advancing `last_verified`. Also distinct from the `run_row` stale-artifact freshness LOW above (that under-states freshness harmlessly; this one hard-refuses load).

**Acceptance:** the env-gated skip path must NOT advance `last_verified` on a row lacking `minted_at` (either preserve the prior `last_verified` alongside `last_real_grade`, or backfill a pinned `minted_at` at skip time from the row's genuine first-verification). Regression: a pre-P90 `minted_at`-less row, env-skipped in a blocking cadence, must still load on the NEXT run (no `SystemExit`/FAIL from a clock-advanced `last_verified`). The P95-designed exemption retirement (make `minted_at` unconditional across all rows) is the class-closing endgame; until then this hardens the skip path.

**Why deferred:** a `quality/runners/run.py` write-path change with its own test obligation — orthogonal to the no-cargo P97 milestone-close window; routes to the next `run.py`-touching quality-framework window.

**Default disposition:** DEFERRED-v0.14.0 (runner-hardening) `[-quality-framework]`.

**STATUS:** OPEN

---

## 2026-07-06 | ORCHESTRATION.md exceeds the 20k soft char-limit (~21.7k) | discovered-by: relief-threshold/C2 doctrine review | severity: LOW

**Size:** S (a progressive-disclosure doc split + pointer, one section relocated).

**Source / mechanism:** `[low]` ORCHESTRATION.md exceeds the 20k soft char-limit (~21.7k).
Progressive-disclosure split needed — candidate: move §11's L0–L4 tier table or §3's
C1/C2 detail to a linked doc, leaving pointers. Deferred from the relief-threshold/C2
doctrine review.

**Why deferred:** the review's charter was the ~50%→~100k relief-trigger sweep + C1/C2
legibility fixes (kept the file net-negative, not a structural split); a proper
progressive-disclosure split is its own change with its own pointer-integrity + promotion-
sweep obligations (§ Provenance "Promotion sweep" standing rule).

**Default disposition:** DEFERRED-v0.14.0 (doctrine-hygiene) — XS/LOW.

**STATUS:** OPEN

---

## 2026-07-06 | C2 (coordinator-of-coordinators) recursion is doctrine-only — never exercised | discovered-by: relief-threshold/C2 doctrine review | severity: MEDIUM

**Size:** S (an observation/instrumentation charter on the first two-tier milestone run — no code).

**Source / mechanism:** `[medium]` The C2 (coordinator-of-coordinators) recursion is
doctrine-only — NEVER exercised (first introduced `2b2736e`). On the first real C2 run,
verify: (i) a relieving C1's relief report routes to its parent C2, NOT L0 (post-dispatch-
relay cross-session addressing is historically flaky — see ORCHESTRATION §8); (ii) C2 and
C1 each relieve on their OWN ~100k line (no double-counting).

**Sketch:** instrument/observe the first milestone run under the two-tier model.

**Why deferred:** requires an actual multi-phase milestone run to exercise the two-tier
relief path; cannot be verified statically — no C2 has run since the doctrine landed.

**Default disposition:** DEFERRED-v0.14.0 (doctrine-validation) — MEDIUM.

**STATUS:** OPEN

---

## 2026-07-06 | `docs/guides/troubleshooting.md` is 25.5k chars, over the 20k progressive-disclosure soft limit | discovered-by: v0.13.0-intake-disposition sweep (260706-crf cold-reader carryover) | severity: MEDIUM

**Size:** M (>1h — a progressive-disclosure nav restructure that moves anchors other docs cross-link to).

**Source:** `docs/guides/troubleshooting.md` measures **25,503 chars** (verified 2026-07-06) — over the 20k `*.md` progressive-disclosure soft limit (root CLAUDE.md OP-4). Pre-existing debt that grew slightly this session via the 260706-crf DVCS cold-reader fixes. Root CLAUDE.md routes DVCS push/pull troubleshooting here, so the file accretes one symptom class after another with no natural cap.

**Acceptance:** split into a child page per DVCS symptom class (e.g. `troubleshooting/{push-conflicts,blob-limit,mirror-lag,sparse-checkout,...}.md`) behind a thin parent index, each under the 20k budget; update every cross-link anchor that currently targets `troubleshooting.md#<symptom>` (root CLAUDE.md § Pointer map, `docs/concepts/dvcs-topology.md`, `docs/guides/dvcs-mirror-setup.md`, ADR-010) so no fragment link rots. Confirm `quality/gates/docs-build/mkdocs-strict.sh` + the file-size gate pass post-split.

**Why >1h / deferred:** this restructures anchors that OTHER docs cross-link to, so it needs a coordinated anchor-rewrite sweep + a mkdocs walk to prove no broken fragment — not a mechanical trim, past the OP-8 "<1h clean eager-fix" line.

**Dedupe / relationship:** sibling of `GOOD-TO-HAVES-15`'s `troubleshooting.md — 22339/20000` line-item — GTH-15 tracks the RAW file-size overage across 9 files under the `structure/file-size-limits` waiver; THIS entry is the specific progressive-disclosure child-page split for this one doc. Do the split here; GTH-15's waiver-renewal accounting clears once it lands.

**Default disposition:** MEDIUM — fold into a v0.14.0 docs-progressive-disclosure or `docs-build`-touching window; pairs with GOOD-TO-HAVES-15.

**STATUS:** OPEN

---

## 2026-07-06 | `codecov/project` posts a phantom -16% release-blocker when the Rust lcov upload silently fails | discovered-by: PR #61 codecov triage lane | severity: MEDIUM

**Size:** S–M (an upload-robustness change to `.github/workflows/ci.yml` coverage job; touches CI, needs a re-run to prove).

**Source:** On CI run `28819166220` (PR #61, head `2d1f55f`) the `codecov/project` check went RED with title `68.60% (-16.46%) compared to f686ab2`. This is NOT a code regression. Evidence: (i) `codecov/patch` = SUCCESS ("all modified and coverable lines are covered by tests"); (ii) the coverage diff shows Files 130→45 (-85) and Lines 18933→1000 (-17933) — the entire Rust workspace report vanished from HEAD, not a real deletion; (iii) codecov's own banner: "HEAD has 1 upload less than BASE" with flag table `|1|0|` (the blank/default flag = the Rust `lcov.info` upload); (iv) the failing project % (68.60%) EXACTLY equals `codecov/project/shell` (68.60%) — codecov computed the default project status from the shell-only subset because the Rust report never landed. Root cause: the `coverage` job (`ci.yml:411-431`) uploads `lcov.info` with `fail_ci_if_error: false` (line 429), so a flaky/failed codecov upload leaves the job GREEN while dropping the Rust report from the merged HEAD coverage — codecov then compares a full BASE (Rust+shell) against a partial HEAD (shell only), manufacturing a phantom -16.46% that reads as a release blocker.

**Acceptance:** make the Rust coverage upload robust so a silent upload drop cannot produce a phantom project-drop that blocks release triage. Options (pick one, do NOT lower any threshold): (a) add codecov-action retry / `fail_ci_if_error: true` on the `coverage` job so an upload failure turns the job RED (honest, actionable) instead of silently poisoning the project comparison; (b) tag the Rust upload with an explicit `flags:` (e.g. `rust`) so a missing-flag report is detectable and carryforward keeps the last-good Rust numbers; (c) add codecov `after_n_builds` so the status only computes once all expected uploads (rust + shell) have arrived. Verify by re-running CI and confirming BASE and HEAD have equal upload counts and project % returns to ~85%.

**Why deferred / not eager-fixed here:** this lane's charter forbids editing CI/codecov gate config, and the correct fix needs a CI re-run to prove the upload lands equally on BASE and HEAD — cannot be verified statically. Per git log, CI was already re-triggered on the current head (`90db62c`); if that re-run lands a clean Rust upload the check clears on its own, but the underlying silent-drop fragility remains and will recur.

**Default disposition:** MEDIUM — fold into a v0.14.0 CI-robustness window; independent of code coverage quality (patch coverage is green).

**STATUS:** OPEN

---

## 2026-07-06 | Durable Confluence TokenWorld fixture page 7798785 has been accidentally trashed 2+ times, losing parent linkage to 7766017 each time | discovered-by: PR #61 CI-red repair lane | severity: LOW-MEDIUM

**Size:** S (investigation + a protection mechanism; not a code bug fix).

**Source:** During the PR #61 CI-unblock effort, the durable Confluence TokenWorld fixture page `7798785` was found with its `parentId` link to `7766017` broken (page effectively orphaned/trashed), causing a live CI red. It was repaired and verified live (parentId now `7766017`). Version history on the page shows this is NOT the first occurrence — the same parent-linkage loss happened at least once before, on 2026-07-04, and now again just prior to this repair. This is a recurring drift pattern on a fixture that other tests/gates depend on being durably present with intact hierarchy, not a one-off fluke.

**Acceptance:** investigate the root cause of the repeated trashing — candidates include (a) manual owner action in the TokenWorld space, (b) a test/cleanup routine that trashes or moves pages as a side effect (e.g. a contract test's teardown, or a stale cleanup script targeting the wrong page ID), or (c) a reposix operation (create/update/delete_or_close path) with a bug that mis-targets this page. Once root-caused, add protection: either Confluence page restrictions preventing accidental trash/move on durable fixture pages, and/or a periodic freshness check (CI or a `docs-repro`/`agent-ux` catalog row) that asserts the fixture's parent linkage is intact before it's relied upon, so a third occurrence surfaces as a clear, attributable signal instead of a mystery CI red.

**Why deferred:** root-causing requires investigation (Confluence audit trail / version history correlation with commit and workflow-run timestamps across at least two incidents) that is out of scope for the CI-unblock lane's charter (fix the immediate red, file the pattern); is not itself a code bug, and any protection mechanism (page restrictions or a new freshness gate) is new scoped work.

**Default disposition:** LOW-MEDIUM — advisory/non-blocking; fold into a v0.14.0 real-backend-fixture-hardening window or the next Confluence-connector-touching phase. Not release-blocking for v0.13.0.

**STATUS:** OPEN

---

## 2026-07-07 | release-plz branch regenerations silently drop `pull_request`-triggered workflows (CI, Security audit, quality gates) | discovered-by: v0.13.0 release CI investigation | severity: MEDIUM

**What:** Every time `release-plz` force-pushes a regeneration of its release branch (e.g. `release-plz-2026-07-07T02-37-20Z`), the `pull_request`-triggered workflows (`CI`, `Security audit`, `quality gates (pre-pr)`) do not automatically re-run against the new head — GitHub's `pull_request` trigger has repeatedly failed to fire cleanly after these force-push regenerations across recent release sessions. The current mitigation, needed 2-3 times per release in recent sessions, is a manual real-actor `gh pr close`/`gh pr reopen`, which forces a fresh `pull_request` event and re-triggers the three workflows.

**Why out-of-scope for eager-resolution:** diagnosing GitHub Actions trigger semantics precisely enough to build a reliable automated re-trigger (vs. the manual close/reopen toll) is real workflow-engineering work — requires testing across multiple regen cycles to confirm any fix actually closes the gap, not a one-line change discoverable mid-release-investigation.

**Sketched resolution:** consider adding a `workflow_dispatch` trigger keyed off release-plz branch pushes, or automate the close/reopen via a scheduled action, so the toll isn't a human/agent doing it manually each regeneration. Home: a CI/workflow-touching phase in v0.14.0 or a dedicated release-tooling window.

**Default disposition:** MEDIUM — process friction, not a correctness bug; observed recurring cost (2-3x per release cycle) makes it worth automating.

**STATUS:** OPEN

## 2026-07-07 | Manually merging `origin/main` onto a live release-plz branch races the bot's own regeneration and usually loses | discovered-by: v0.13.0 release CI investigation | severity: LOW-MEDIUM (process note)

**What:** Merging `origin/main` onto a live release-plz branch races the bot's own periodic regeneration and usually loses — the merge gets superseded before it can land, because release-plz will itself re-base off `main` on its next regen and pull in the same commits anyway. The manual merge is redundant work that also risks a confusing intermediate state (commits that appear to land, then vanish under the next force-push).

**Why out-of-scope for eager-resolution:** this is a process/runbook observation, not a code or tooling gap — nothing to fix in the codebase; the fix is documentation of the correct workflow.

**Sketched resolution:** the release runbook should note "don't manually merge onto the bot branch; land the fix on `main` and let the next regen absorb it" rather than treating a manual merge as the first move.

**Default disposition:** LOW-MEDIUM — cheap doc fix; fold into the next release-runbook touch.

**STATUS:** OPEN

## 2026-07-07 | Quality gates asserting a third-party tool's exact surface string false-negative when the vendor rewords output | discovered-by: v0.13.0 post-release CI run 28839335746 investigation | severity: LOW-process

**What:** Two gates this release went RED not because the thing they check was broken, but because the verifier's PASS condition grepped a literal, exact surface string from a third-party tool's stdout, and that tool changed its wording between versions: (1) `release/cargo-binstall-resolves` grepped only `github.com/reubenjohn/reposix/releases/download/`, but current cargo-binstall prints `has been downloaded from github.com` on a successful resolve instead of echoing the download URL — the v0.13.0 tag run actually resolved the prebuilt binary in ~1.97s with rc=0, yet the gate FAILed; (2) the p94-badges gate (`quality/gates/docs-build/p94-badges-real-vs-transient.sh`) has the same class of brittleness — asserting an exact badge/response string shape from a third-party service (shields.io or similar) rather than the underlying invariant (badge resolves / is live vs. transient-404).

**Anti-pattern:** quality gates must assert the INVARIANT the row's `expected.asserts` describes (e.g. "resolved a prebuilt binary and exited 0", "badge is live not transiently 404ing"), not a single literal substring lifted from one observed run of a third-party tool's output. Vendor CLI/service wording drifts across versions with no changelog signal to the gate; a single-string match makes every such drift a false-negative RED that looks like a real regression until a human diffs the stdout.

**Fix applied to instance (1) THIS session:** `quality/gates/release/cargo-binstall-resolves.py` PASS logic broadened to a `PASS_SIGNALS` tuple (case-insensitive, accepts both the legacy URL-echo and the newer "has been downloaded from github.com" wording) AND requires the independent "will install the following binaries" line, so a third wording change fails toward PARTIAL/FAIL-investigate rather than silently matching nothing forever. Regression test: `quality/gates/release/test_cargo_binstall_resolves.py`. Catalog row `release/cargo-binstall-resolves` (quality/catalogs/release-assets.json) updated to describe the broadened contract.

**Sketched resolution for the class:** audit all `quality/gates/**/*.py` and `*.sh` verifiers for literal-substring asserts on third-party tool/service stdout or HTTP response bodies (grep for hardcoded URL fragments, exact vendor phrase matches, or single-string `in combined` checks without a fallback signal set); for each hit, either (a) broaden to a small accepted-wordings set with a REGRESSION NOTE like the binstall fix, or (b) switch to a more structural signal (exit code + a documented structural marker) that's less likely to drift. Start with `p94-badges-real-vs-transient.sh` since it's the second instance observed this release.

**Why deferred (this occurrence, instance 2 only):** instance (1) was fixed in-session (< 1h, no new dependency); instance (2) — the p94-badges gate — was only *noticed*, not reproduced or fixed, in this dispatch; fixing it requires reading that gate's current assert logic and the live shields.io/badge-service response shape, which is out of scope for this binstall-focused fix.

**Default disposition:** LOW-process — no correctness bug in the underlying feature either time, but a recurring gate-authoring anti-pattern worth a repo-wide sweep before it produces a third false-negative.

## 2026-07-07 | `SURPRISES-INTAKE.md` has outgrown its own pre-commit soft limit (~77k chars vs. 20k warn threshold) | discovered-by: v0.13.0 post-release verification pass | severity: LOW (process)

**What:** `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` is now ~77k chars; the
pre-commit hook already warns (non-blocking) on any commit touching it past 20k chars. It
needs a distill/split pass (progressive disclosure — move resolved/dated history into
`.planning/RETROSPECTIVE.md` or an archive file, keep only live OPEN entries in the working
file per the "bound-to-live-state" rule in SESSION-HANDOVER §0).

**Why out-of-scope for eager-resolution:** distilling ~700 lines of entries requires
per-entry triage (which are truly resolved/superseded vs. still live) — that's exactly the
milestone-close OP-9 distillation work, not a drive-by edit mid-verification-pass.

**Sketched resolution:** at the next milestone-close (OP-9 distillation into
`.planning/RETROSPECTIVE.md`), split this file: archive terminal/RESOLVED/superseded
entries out (git is the archive per the file's own carry-forward-banner convention), leave
only live OPEN/DEFERRED entries in the working file, and consider a per-quarter or
per-milestone rotation so the file doesn't recross the 20k soft limit again.

**Default disposition:** LOW — recommended at milestone-close; do not distill now (mid
verification pass is not the venue).

**STATUS:** OPEN

## 2026-07-07 | Ship a bundled default seed inside the release binary so getting-started needs no `--seed-file` / no network fetch | discovered-by: v0.13.1 Wave E (README quick-start doc-lie fix) | severity: MEDIUM

**What:** `reposix sim` with no `--seed-file` seeds 0 issues (confirmed by a real run:
`reposix-sim: listening on http://127.0.0.1:7878 (seed: none (no --seed-file), 0 issues)`).
The only fixture that seeds real data, `crates/reposix-sim/fixtures/seed.json`, is a
source-tree file NOT bundled into any release archive (Homebrew, cargo-binstall,
curl-installer, PowerShell) — so every prebuilt-binary install path is stuck at 0 issues
unless the user separately fetches that fixture. Wave D (`docs/tutorials/first-run.md`,
`docs/index.md`) and Wave E (`README.md`) both worked around this by having the
getting-started flow `curl` the fixture from `raw.githubusercontent.com` before starting
the sim. That workaround is honest and verified, but it introduces a hard network
dependency into the first 60 seconds of onboarding (an air-gapped or firewalled dev, or
anyone hitting a GitHub outage, cannot complete the tutorial) and adds one more moving
part a copy-pasting dev can get wrong (wrong URL, `curl` not installed, corporate proxy
blocking raw.githubusercontent.com).

**Why out-of-scope for eager-resolution:** the real fix is embedding the fixture into the
`reposix-sim` (or `reposix-cli`) binary at compile time (e.g. `include_str!` pointing at
`crates/reposix-sim/fixtures/seed.json`, wired behind a `--seed-file` fallback so an
explicit `--seed-file` still overrides the embedded default) and deciding the UX contract
for "seed on first run with no flag" vs. "stay explicit, but ship the embedded default
under a new flag like `--seed-default`". That's an intentional behavior change to the sim
CLI plus a release-asset verification loop (rebuild all 8 crates' release binaries,
confirm the embedded bytes survive `cargo binstall` / Homebrew / curl-installer / choco
paths) — real cross-cutting work, not a docs-only drive-by.

**Sketched resolution:** add a `const DEFAULT_SEED: &str = include_str!("../fixtures/seed.json")` (or equivalent embed macro) to `reposix-sim`; when `reposix sim` starts with
no `--seed-file`, seed from the embedded default instead of seeding nothing (or add an
explicit `--seed-default` flag if "no flag = empty" is intentionally load-bearing
elsewhere and shouldn't silently change). Once shipped, revert the Wave D/E curl-fixture
workaround in `README.md` + `docs/tutorials/first-run.md` + `docs/index.md` back to a
plain `reposix sim --bind 127.0.0.1:7878 &` with no curl step and no `--seed-file` flag —
removing the network dependency from onboarding entirely. Verify against a real prebuilt
binary (not `cargo run`) so the embed genuinely survives packaging.

**Default disposition:** MEDIUM — this is the real root-cause fix underneath two doc
workarounds (Wave D + Wave E); closing it removes a network dependency from the
zero-shot-gate onboarding path and lets both docs surfaces simplify. Target: v0.14.0.

**STATUS:** OPEN

## 2026-07-07 | doc-alignment catalog carries a ~180-row backlog of un-rebound drift outside this milestone's edited docs | discovered-by: v0.13.1 Wave E1b (doc-alignment rebind lane) | severity: MEDIUM

**What:** Running `reposix-quality doc-alignment walk` against the current committed
catalog surfaces roughly 180 rows across `quality/catalogs/doc-alignment.json` in
`STALE_TEST_DRIFT` / `STALE_DOCS_DRIFT` that are unrelated to any file this milestone
(v0.13.1) touched — benchmark pages, the glossary, connector-guide docs, and older
sessions' bindings that drifted from source/test changes in prior milestones and were
never re-bound. Wave E1b rebound exactly the 8 rows whose staleness this milestone's
`docs/**`/`README.md` edits freshly introduced (`docs/tutorials/first-run.md`,
`docs/guides/troubleshooting.md`, `docs/index.md`); the pre-existing ~180-row backlog
was deliberately left untouched (git-diff-verified: only the 8 in-scope rows changed
in the commit). This means the catalog's `alignment_ratio` (0.789) and `coverage_ratio`
(0.181) summary numbers are currently propped up by a large silently-stale substrate —
neither ratio can be trusted as "everything reachable from HEAD is actually bound"
until the backlog is drained.

**Why out-of-scope for eager-resolution:** re-binding ~180 rows requires per-row
citation work (reading each doc's current content, re-deriving accurate line ranges
and claims, confirming the bound test still asserts what the claim says) — this is
exactly the `/reposix-quality-backfill` full-extraction workflow's job, not a
drive-by merge inside a scoped docs-rebind lane. Several of the backlog rows are also
flagged `coverage: row ... cites out-of-eligible file ...` (e.g. rows citing
`crates/reposix-core/src/backend.rs`, `docs/architecture.md`, `docs/demo.md`) which is
a distinct structural cleanup (retire or re-point the citation), not a hash refresh.

**Sketched resolution:** dispatch a dedicated `/reposix-quality-refresh <doc>` pass per
stale doc (or `/reposix-quality-backfill` for a full extraction sweep) before the next
milestone trusts `alignment_ratio`/`coverage_ratio` as a release gate; triage the
`cites out-of-eligible file` rows separately (retire via `propose-retire` +
`confirm-retire`, or re-point the citation to the file that actually moved).

**Default disposition:** MEDIUM — doesn't block v0.13.1 (this milestone's own edits are
now honestly bound), but the backlog's size means the dimension's headline ratios are
not yet trustworthy signal. Target: v0.14.0 scoping session.

**STATUS:** OPEN

## 2026-07-07 | `doc-alignment walk` mutates the committed catalog in place with no `--persist` gate, unlike `run.py`'s GRADE/PERSIST split | discovered-by: v0.13.1 Wave E1b (doc-alignment rebind lane) | severity: MEDIUM

**What:** `reposix-quality doc-alignment walk` (help text: "Hash drift walker --
updates `last_verdict` only") writes its recomputed `last_verdict` (and summary block:
`claims_bound`, `alignment_ratio`, `coverage_ratio`, `last_walked`, etc.) straight back
to `quality/catalogs/doc-alignment.json` on every invocation — there is no `--persist`
flag to gate this, unlike `quality/runners/run.py`, which (per the D-P96-01 GRADE/
PERSIST split documented in `quality/PROTOCOL.md`) is validate-only by default and
requires an explicit `--persist` to mutate `quality/catalogs/`. A diagnostic-only
invocation of `walk` (e.g. "let me see what's stale before I decide what to rebind")
silently dirties the tree with a full-catalog re-verdict — confirmed directly in this
lane: running `walk` once flipped `claims_bound` 261→265 and touched ~180 rows'
`last_verdict` fields, none of which this lane intended to commit. The lane recovered
via `git checkout -- quality/catalogs/doc-alignment.json` before the real (surgical,
`bind`-based) rebind work, and Wave E1 (`4f1e0f0`) hit the identical side effect and
used the same recovery.

**Why out-of-scope for eager-resolution:** changing `walk`'s persistence contract is a
runner-semantics change to a load-bearing quality-gate tool (touches the pre-push gate
at `gates/docs-alignment/walk.sh`, the `status` verb's read path, and every doc in
`quality/catalogs/README.md` describing the docs-alignment dimension) — it needs the
same design care as the P96 `run.py` GRADE/PERSIST split, not a same-session
drive-by patch mid docs-rebind.

**Sketched resolution:** add a `--persist` flag to `doc-alignment walk` mirroring
`run.py`'s contract: default invocation computes and prints the stale-row report
without writing `quality/catalogs/doc-alignment.json`; `--persist` writes it back.
Update `gates/docs-alignment/walk.sh` (the pre-push gate) to pass `--persist`
explicitly since that gate's job IS to mint the walked state. Document the flag in
`quality/catalogs/README.md` § docs-alignment dimension and cross-reference the
D-P96-01 precedent in `quality/PROTOCOL.md`.

**Default disposition:** MEDIUM — tooling-hygiene; a silent-dirty diagnostic tool is a
recurring trap (this is the second lane in two Waves to hit it) but not correctness-
blocking since every lane so far has caught it via `git status` before committing.
Target: v0.14.0 scoping session.

**STATUS:** OPEN

---

## 2026-07-05 | `badges-resolve` FAILs on pre-push (docs-build + structure dimensions) | discovered-by: P93 Wave 1 de-risk executor | severity: MEDIUM

**What:** The `badges-resolve` check (README/docs badge URLs must resolve — shields.io,
Codecov, CI status) was FAILing on pre-push. Root cause was unconfirmed at filing time:
transient upstream flake vs. genuinely-broken badge URL.

**Resolution (P94 D3, 2026-07-05):** Determined **TRANSIENT** via ≥2 spaced isolated
re-runs (3 runs across ~8 min, all 10 badge URLs returned HTTP 200 + correct
content-type on the first attempt; no deterministic 404/wrong-type ever observed).
Full evidence + verdict: `.planning/phases/94-real-backend-frictions/94-D3-badges-determination.md`.
Fix applied (TRANSIENT branch of the catalog contract): `badges-resolve.py` `head_url()`
now retries a transient failure (network error, or HTTP 408/425/429/5xx) up to
`MAX_ATTEMPTS = 3` with `BACKOFF_S = (1.0, 2.0)` spacing; a deterministic failure
(404/403/other-4xx or wrong content-type) still fails fast on the first attempt, so the
retry cannot mask a genuinely-dead badge. Net: `python3 quality/gates/docs-build/badges-resolve.py`
exits 0 reliably instead of flaking RED on pre-push.

**Note (2026-07-07):** This entry was accidentally pruned by `1b37350` ("prune v0.13.0
intakes to open-only") and restored here — the `docs-build/p94-badges-real-vs-transient`
verifier mechanically reads this RESOLVED entry as its assert-2 precondition, so it must
persist in `GOOD-TO-HAVES.md` (do not re-prune while that gate is live). Surfaced by
S-260707-pr-07.

**STATUS:** RESOLVED

---

## 2026-07-07 | `git-version-requirement-documented.sh` is a bare `grep -F '2.34'`, cannot detect a hard-vs-recommended regression | discovered-by: v0.13.1 mechanical filing lane | severity: LOW

**What:** The structure/docs gate `quality/gates/.../git-version-requirement-documented.sh`
(exact path TBD by the next touching phase — locate via `grep -rl git-version-requirement
quality/gates/`) passes as long as the literal string "2.34" survives anywhere in the
target doc. It cannot distinguish "git >= 2.34 is a HARD requirement" from "git 2.34+ is
RECOMMENDED" — so a future regression from the now-softened recommended-not-hard framing
back to a hard floor (or vice versa) would sail through the gate silently.

**Acceptance:** tighten the verifier to assert the "recommended"/"WARN not ERROR" framing
specifically (e.g. grep for the phrase pattern that distinguishes recommended-vs-required,
not just the bare version literal), so the softened git-floor story is test-enforced, not
just prose.

**Why deferred:** gate-touching change with its own false-positive review across whatever
docs it currently scans; out of this lane's mechanical-filing envelope (no `docs/**`
edits, no gate edits).

**Default disposition:** LOW — always closes; fold into the next `quality/gates/`-touching
phase or the git-floor drift fix (SURPRISES-INTAKE.md 2026-07-07 entry) landing together.

**STATUS:** OPEN

## 2026-07-07 | doc-alignment `walk` mutates the catalog with no `--persist` gate — dirties the tree on every validate-only run | discovered-by: v0.13.1 mechanical filing lane (cross-referencing Waves E1/E1b/F1b) | severity: MEDIUM

**What:** Duplicate/companion filing to the "2026-07-07 | `doc-alignment walk` mutates
the committed catalog in place" entry already in this file (search for that header) —
confirming here for visibility since it bit every push this session. `reposix-quality
doc-alignment walk` writes its recomputed verdict straight back to
`quality/catalogs/doc-alignment.json` with no `--persist` gate, unlike `run.py`'s
GRADE/PERSIST split. Bitten Waves E1, E1b, and F1b, plus every push this v0.13.1 session
— each lane had to `git checkout -- quality/catalogs/doc-alignment.json` before
committing.

**Acceptance:** add a read-only/`--persist`-gated mode so validate-only walks don't dirty
the catalog, mirroring the `run.py` D-P96-01 GRADE/PERSIST split precedent.

**Why deferred:** runner-semantics change to a load-bearing quality-gate tool (see the
existing fuller entry in this file for the full rationale) — not a mechanical-filing
action.

**Default disposition:** MEDIUM — recurring trap (3+ lanes hit it), not yet
correctness-blocking. Target: v0.14.0 scoping session. This entry is intentionally
lightweight since the full write-up already exists in this file; kept as a second
pointer so a future dedupe pass can merge them.

**STATUS:** OPEN

## 2026-07-07 | ~180-row doc-alignment backlog likely harbors more Haiku-backfill false-BONDS + latent TEST_DRIFT rows — needs systematic re-grade | discovered-by: v0.13.1 mechanical filing lane (cross-referencing Wave F1b + C2-f handover) | severity: MEDIUM

**What:** The ~180-row doc-alignment backlog (flagged elsewhere in this file's
"backlog's size means the dimension's headline ratios are not yet trustworthy signal"
entry) likely contains more Haiku-backfill false-BONDS of the shape Wave F1b already
corrected once, plus latent `STALE_TEST_DRIFT` rows on the two permanently-yellow
benchmark claims (`token-89-percent`, `latency-8ms`) that haven't been systematically
re-checked since the backlog accumulated.

**Acceptance:** a systematic re-grade pass over the ~180-row backlog in v0.14.0 —
re-verify each row's citation still resolves to content that actually supports the bound
claim (not just that the file/line exists), and specifically check the two benchmark-claim
rows for drift against current `docs/benchmarks/*.md` values.

**Why deferred:** M-sized systematic audit work, not a mechanical filing action; needs a
dedicated docs-alignment-touching phase per this project's own routing convention.

**Default disposition:** MEDIUM — real trust-in-signal risk for the docs-alignment
dimension's headline ratios, but no immediate correctness hazard. Target: v0.14.0
scoping session.

**STATUS:** OPEN

## 2026-07-07 | Pre-commit size soft-warnings: `crates/reposix-cli/src/main.rs` and `quality/gates/agent-ux/zero-shot-onboarding.sh` both over budget | discovered-by: v0.13.1 mechanical filing lane | severity: LOW

**What:** `crates/reposix-cli/src/main.rs` is 21279 chars (> 20000 budget) and
`quality/gates/agent-ux/zero-shot-onboarding.sh` is 10572 chars (> 10000 budget) — both
surfaced as pre-commit soft-warnings (non-blocking) during the v0.13.1 session.

**Acceptance:** split each file along its natural seams once either keeps growing
further past its budget (e.g. `main.rs` by subcommand-dispatch group;
`zero-shot-onboarding.sh` by scenario-phase into a sibling script or sourced helper).

**Why deferred:** soft-warning only (non-blocking), real split work not a mechanical
filing action; joins the existing consolidated file-size-overages entry
(GOOD-TO-HAVES-15) as a candidate for the same pre-2026-08-08 waiver-renewal /
structure-hygiene pass.

**Default disposition:** LOW — soft warning only, no blocking hazard today. Target:
fold into GOOD-TO-HAVES-15's consolidated split pass or the next structure-touching
phase.

**STATUS:** OPEN

## 2026-07-07 | Cosmetic front-door UX: stray `builtin seed loaded` INFO line prints before `reposix sim`'s clean banner | discovered-by: v0.13.1 mechanical filing lane | severity: LOW

**What:** A stray `INFO ... builtin seed loaded inserted=6` tracing line prints BEFORE
the sim's clean banner on `reposix sim` — the documented happy path a first-time user
follows. It's not wrong (seeding did happen), but it clutters the first thing a new user
sees before the intended clean banner.

**Acceptance:** demote the line to debug-level tracing (so it doesn't print at the
default log level), or fold its content into the documented banner block so the
front-door output stays clean and single-purpose.

**Why deferred:** cosmetic-only, `crates/reposix-sim` source edit — out of this lane's
mechanical-filing envelope (no code edits).

**Default disposition:** LOW — cosmetic only, no functional hazard. Target: the next
`reposix-sim`-touching phase or a front-door-polish pass.

**STATUS:** OPEN
