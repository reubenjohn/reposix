# v0.13.0 GOOD-TO-HAVES — Part 1 of 8

> Split from `GOOD-TO-HAVES.md` for the file-size gate (OP-8 drain). Index: `../GOOD-TO-HAVES.md`. Entries preserved verbatim.

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

