# PROGRESS — v0.15.0 "Floor"

_A live progress briefing. Refresh at every task/wave/capture boundary in the SAME push; every relief handover refreshes it. A stale progress file is worse than none._

## SHIPPED

- 2026-07-15 — Confluence oid-drift fix (list-vs-get render parity) shipped live on the real backend + reconcile audit — `dc26302` ✅
- 2026-07-15 — Benchmark "session" definition ratified — `3278abc` ✅
- 2026-07-15 — Latency numbers re-measured and corrected to the CI-canonical figures — `9384ca6` / `3845b13` ✅
- 2026-07-15 — Latency doc re-aligned to the corrected numbers — `92c3ab5` ✅
- 2026-07-15 — Benchmark session-spend ledger established (≤50 ceiling) — `4351d48` ✅
- 2026-07-15 — Public roadmap diagram shipped — `fa58ad6` ✅
- 2026-07-15 — JSONL-usage token-economy methodology adopted — `9be5439` ✅
- 2026-07-15 — Real MCP tool surface captured; the planned Jira/atlassian-rovo benchmark path found infeasible (no write tool + token denied + empty project) — recorded honestly, no fabricated numbers — `ece072f` ✅
- 2026-07-16 — Live token-economy benchmark captured on the GitHub backend — 6 real sessions (median-of-3 × 2 arms) running read-3-issues / edit-1 / push against reubenjohn/reposix; the reposix (git-native) arm is cheaper on every axis vs the GitHub MCP arm (≈75% cheaper per session, ≈94% fewer output tokens, ≈56% less total input-context / ≈66% less newly-cached context). Real per-session captures + GitHub MCP catalog (44 tools) + live git-native transcript committed; CAPTURE_OK green. (Findings for follow-up: reposix's GitHub write-back is read-only in this build cut so the reposix push doesn't persist — comparison unaffected; and the GitHub MCP's issue-read is lossy for raw markdown while reposix round-trips bytes faithfully.) — `4db6b64` ✅
- 2026-07-16 — `docs/benchmarks/token-economy.md` regenerated from the live GitHub captures — the synthetic count_tokens-on-fixtures baseline (retired 89.1% / 85.5% figures) is replaced by a deterministic, offline, no-API-key headline computed from the committed `benchmarks/captures/*.json` session-usage records: **~94% fewer output tokens, ~75% cheaper per session** (four axes: output ~94.3% / cache-create ~66.0% / total input-context ~55.6% / cost ~74.9%). Provenance + methodology rewritten (kills the false `scripts/demo.sh` / "modeled on Forge" claims), read-only-write-back + MCP-lossy-reads honesty caveats added, stale sidecar deleted (GTH-V15-26 resolved). — `1cdb381` (wave closed `2103d0c`, CI green, post-push P0 PASS re-minted at conclusion) ✅
- 2026-07-16 — **T6 item 1 (headline reframe) — SHIPPED** — hero surfaces re-anchored from the retired synthetic **89.1%** to the live GitHub-capture headline **~94% fewer output tokens / ~75% cheaper per session** (output ~94.3% / cache-create ~66.0% / input-context ~55.6% / cost ~74.9%), matching `token-economy.md`'s provenance framing so heroes + benchmark page tell one story. Touched: `README.md` "Three measured numbers", `docs/index.md` (token card + loop-diagram Notes + caption + token-economy card + honest-scope footer), `docs/concepts/reposix-vs-mcp-and-sdks.md` "About the MCP comparison". Both live findings folded in (GitHub write-back read-only this cut → comparison unaffected; MCP `issue_read` lossy vs reposix byte-fidelity). banned-words + mkdocs-strict + mermaid-renders green. Landed `d2fd85c`; its own push attempt BLOCKED as designed on 4 no-waiver doc-alignment rows freshly `STALE_DOCS_DRIFT` — cleared via the top-level `/reposix-quality-refresh` retire+rebind (`9a2b6f1`, 6 rows re-graded) + a time-boxed `waive` on the still-blocking 5 (`c9c2aee`, until 2026-08-15, tracked to `115-UNWAIVE-PATH.md`). Push landed, pre-push 61 PASS / 0 FAIL, CI green (`29491742214`). — `d2fd85c` (refresh `9a2b6f1`, waive `c9c2aee`) ✅
- 2026-07-16 — **T6 item 3 (retire+rebind token-economy.md rows) — SHIPPED agent-side** — the 6 synthetic `count_tokens`-over-fixture `token-economy.md` doc-alignment rows (76.4% / 85.5% / jira-real-adapter / 4,883 / 531 / 89.1%) propose-retired (`RETIRE_PROPOSED`, human-only confirm-retire pending — env-guard untouched, not worked around). Replacement rows for the LIVE four-axis figures bound/GREEN with fresh hand-verified citations: `output-reduction-94-percent`, `cost-reduction-75-percent`, `live-github-capture-methodology` — each AND-drift-bound to `bench_token_economy.py` + its offline test suite. Pytest 9 passed offline; catalog delta +3 rows / 0 removed. Pre-push walk `rc=0`. Evidence: `115-T6-CLOSEOUT.md` § Wave 1 — item 3. — `d7da383` ✅
- 2026-07-16 — **T6 item 5 (regen-clobber guard) — SHIPPED** — `emit-markdown.sh` now refuses to clobber `docs/benchmarks/latency.md`'s CI-canonical sections. New `quality/gates/perf/latency-bench/regen-guard.sh` gates the write on an end-of-file marker, refuses with a teaching error unless `REPOSIX_LATENCY_BENCH_ALLOW_CANONICAL_OVERWRITE=1`. New `regen-guard.selftest.sh` (12 assertions) passes; docs-alignment walk / banned-words / mkdocs-strict / mermaid-renders all green. Fixed a lying doc claim (Reproduce prose asserted a protection that didn't exist in code) and filed `GTH-V15-28` (line-anchored citations are a sharp edge for future doc edits). Evidence: `115-T6-CLOSEOUT.md` § Wave 2 — item 5. — `2eb5836` ✅
- 2026-07-16 — **T6 item 2 (`115-UNWAIVE-PATH.md` inventory) — SHIPPED** — wrote the full waived-row inventory in the P115 phase dir: at the time, 19 waived doc-alignment rows (8 pre-existing hero + 6 token-economy.md + 5 newly time-boxed) + 2 perf rows (`perf/token-economy-bench` / `perf/headline-numbers-cross-check`), later refined to the final 21-row/11-remaining-waived count as item 6b landed. Corrected a stale "8 uniform hero rows" framing (the batch is heterogeneous: `WAIVED-MISSING_TEST` + `RETIRE_PROPOSED` + perf rows, not one class). Filed a third corroborating `SURPRISES-INTAKE.md` pre-push wall-time-creep entry (141s at `d7da383`). Evidence: `115-T6-CLOSEOUT.md` § Wave 2 — item 2. — `c2af48b` (+ `567dce8`) ✅
- 2026-07-16 — **T6 item 7 (delete FIVE `[SELF]` CONSULT-DECISIONS entries) — SHIPPED** — all five `[SELF]` decision entries deleted from `.planning/CONSULT-DECISIONS.md`: A1 (line 71), T6-headline (96), T2-latency-canonical (114), T5-JSONL-methodology (123), T4-GitHub-pivot (153); companion note at line 159 also deleted. Post-grep confirms only the format-definition line + the unrelated live `RBF-LR-03` owner-decision entry remain (verified live: file is now 70 lines). Evidence: `115-T6-CLOSEOUT.md` § Wave 2 — item 7. — `e7a1fd2` ✅
- 2026-07-16 — **T6 item 6a (headline-numbers-cross-check gate + 8ms→6/7ms reconcile) — SHIPPED** — wrote the missing `quality/gates/perf/headline-numbers-cross-check.py` verifier + 12-test suite; reconciled the "8 ms" hero prose to canonical "6 ms get / 7 ms list" across all 3 hero surfaces (6 edits); repaired + un-waived the EXISTING P90-era `perf/headline-numbers-cross-check` catalog row (dangling-verifier fixed, no duplicate row created) — minted PASS via `run.py --cadence weekly --persist`; rebound `docs/index/latency-8ms-read` + `latency-cached-read-8ms`. Gate GREEN (RED pre-edit → PASS post-edit); walk rc=0. **CI hotfix `3eacb53`** (concurrent lane) fixed a RED main (`bench-latency-v09` regression vs the item-5 regen-clobber guard) that rode out on this push. Evidence: `115-T6-CLOSEOUT.md` § Wave 2 — item 6a. — `63fdd8d` (+ `cd125eb` closeout evidence, `3eacb53` CI hotfix) ✅
- 2026-07-16 — **T6 item 6b (cold-init 27ms→278ms reconcile + un-waive loop/perf rows) — SHIPPED — T6 (all 7 items) COMPLETE** — cold-init hero **27 ms → canonical 278 ms** (same operation, superseded dev-machine figure fixed to canonical); extended `headline-numbers-cross-check.py` with a cold-init axis + 2 absolute loop-figure checks (18 hero headlines, all match). Bound+unwaived the 3 cold-init rows + the 2 loop-token rows (`~21k` MCP / `~1.2k` reposix) + `README-md/latency-8ms`; propose-retired + re-attributed 3 more superseded 89.1%-era rows (a true duplicate pair folded, no distinct claim lost); un-waived + minted `perf/token-economy-bench` PASS (`main()` now asserts ~94.3% ±1.0pp); persisted two benign validate-only status flips (stale FAIL/NOT-VERIFIED → PASS, surgical, code/shell-coverage + security/cargo-audit). Non-hero 8ms fixed on mental-model:69 / filesystem-layer:42 / concepts-vs-mcp:15. Walk rc=0, gate exit 0, perf pytest 26/26, docs-build all green. Filed `GTH-V15-29..33` (bind --test fn-resolution unenforced; row-ID↔claim cosmetic drift; webhook-latency deliberate-exception clarity; gate script near its char ceiling; mental-model-page L21/L69 inconsistency). CI green (`29501752893`), post-push P0 PASS. **Human relay: the confirm-retire batch is now ELEVEN rows** (8 prior + 3 new) — see NOW. — `776ca85` ✅
- 2026-07-16 — **Pre-close owner-directive lane (strip retirement-history narrative) — SHIPPED** — owner ruling 2026-07-16: user-facing docs carry current truth only, correction history lives in git history + planning artifacts. Removed the old-figure retirement-story sections from `docs/benchmarks/token-economy.md` (89.1%/85.5%), `docs/concepts/reposix-vs-mcp-and-sdks.md` (4,883/531 origin sentence), `docs/index.md` (retired-figure clause), and `docs/benchmarks/latency.md` ("Superseded figures" paragraph) — current live numbers and all current-measurement caveats (read-only write-back scope, MCP-lossy caveat, live-capture provenance) kept intact. Re-bound 2 latency catalog rows for line shift; mkdocs-strict + mermaid + banned-words + docs-alignment walk all green. Zero new rows propose-retired (verified: the batch is unchanged at 11 rows). Ledger entry in `CONSULT-DECISIONS.md`; consolidated 11-row confirm-retire batch + copy-paste commands landed in `115-UNWAIVE-PATH.md` FINAL section; 3 intake filings + 1 GOOD-TO-HAVE routed. — `5a5dd29` (planning artifacts in this commit) ✅
- 2026-07-16 — **Quick task 260716-f6o (fix-it-twice: strip retirement-history narrative from the perf-gate GENERATOR) — SHIPPED** — owner ruling `5a5dd29` deliberately removed the "## What retired the old 89.1% / 85.5% figures" section from `docs/benchmarks/token-economy.md`, but the GENERATOR (`bench_token_economy_captures.py::render_token_economy_markdown`) still templated it; the P115 phase-close gate-run regen re-added it in place, leaving a dirty `+12`-line working tree. **Manager-established provenance: accidental regression vector, NOT a deliberate override of the owner ruling.** Stripped the section from the template; offline regen (`bench_token_economy.py --offline`) now reproduces the committed doc byte-for-byte (sha256 `5620699b...364fcf` match, empty `git diff`). Discarded the stray working-tree re-add (belt-and-suspenders `git checkout --`). Verified no doc-alignment catalog rebind needed — committed doc bytes unchanged, BOUND rows are the live four-axis claims, catalog untouched. — `19f9ae2` (+ `ac9e717` STATE.md record) ✅
- 2026-07-16 — **Quick task 260716-fmt (`GTH-V15-35` docs/index.md install-IA fix, both addenda) — SHIPPED** — nested "Build from source (advanced)" under the "30-second install" tabs (install-leads-with-pkg-mgr gate stays GREEN); surfaced the `reposix sim` / `reposix init` bootstrap lines in visible prose; split + destaled the two-claim `docs/index.md:93` line (stale "Phase 36" claim replaced with the real GitHub 320 ms / Confluence 202 ms figures from `docs/benchmarks/latency.md:42`); all 11 shifted doc-alignment rows mechanically rebound (walk exit 0, zero `STALE_DOCS_DRIFT`); filed one MEDIUM `SURPRISES-INTAKE.md` row (the token-economy regen test's missing byte-compare-against-committed-doc coverage — the exact gap class behind the `260716-f6o` regression). `GTH-V15-35` STATUS → DONE. — `97fad0d` (+ `2398b34` STATE.md record) ✅

- 2026-07-16 — **L0 #47 boundary — uncatalogued-hero-number intake filed (manager finding)** — three hero surfaces (`docs/index.md:17`, `README.md:27`, `docs/concepts/reposix-vs-mcp-and-sdks.md:29-31`) present the live ~94.3%/~74.9% token-economy figures with NO doc-alignment binding; once the pending 11-row confirm-retire batch lands they become entirely uncatalogued. Filed to `SURPRISES-INTAKE.md` (MEDIUM, HIGH-visibility) with the manager's remedy: `/reposix-quality-refresh` per doc binding to `bench_token_economy.py` / `headline-numbers-cross-check.py` (same as `output-reduction-94-percent` rows), execute before P117. CI green on tip (CI/Docs/release-plz/Push-on-main all success). — `029bde7` ✅
- 2026-07-16 — **L0 #48 boundary — manager-priority hero-number refresh lanes SHIPPED** — minted 3 doc-alignment rows via the surgical `bind` verb for the previously-uncatalogued hero surfaces (`docs/index/hero-token-economy-94-75`, `README/hero-token-economy-94-75`, `docs/concepts/reposix-vs-mcp-and-sdks/token-economy-output-cost`); manager intake marked RESOLVED. Full detail in the #48→#49 handover archived at `b325caf`. CI green. — `c35f993`/`7553c36`/`aa75e96`/`e185e6e` ✅
- 2026-07-16 — **L0 #49 boundary — P116 research + validation skeleton SHIPPED (one subagent-run sub-step, per #46's split-across-reliefs doctrine)** — dispatched `gsd-phase-researcher` (sonnet) → `116-RESEARCH.md` (52,340 bytes, HIGH confidence); hand-authored the `116-VALIDATION.md` step-5.5 skeleton (frontmatter filled, body explicitly planner-owned) so re-entry sails past the step-7.5 Nyquist gate. Research corrected `116-CONTEXT.md`'s mechanical framing (rulings unchanged): the false "`sync --reconcile` heals the external mirror" claim lives only in an ARCHIVED v0.14.0 intake row, NOT in any live doc-alignment catalog row (jq-verified: zero rows bind any `CLAUDE.md`); the row to actually retire is the LIVE `SURPRISES-INTAKE.md:108-116` litmus-non-idempotency entry. Pushed, CI green (`Docs`/`CI`/`Push on main` all `success`). — `05085fe` ✅
- 2026-07-16 — **L0 #50 boundary — P116 planning tail advanced by one sub-step (pattern map), plus one filed noticing + one eager fix** — filed `GTH-V15-39` (catalog row-id-prefix inconsistency, manager-routed noticing carried from #49) and eager-fixed the adjacent `GTH-V15-37`/`GTH-V15-38` copy-paste bleed in the same commit, pushed, CI green (`6d21cae`); then dispatched `gsd-pattern-mapper` (sonnet, standalone — not via `/gsd-plan-phase`) → `116-PATTERNS.md` (22,259 bytes, 8 edit-target analogs), committed, pending push. Research + validation + patterns are now ALL on disk for P116 planning; only `gsd-planner`/`gsd-plan-checker`/coverage-gates remain. — `6d21cae`, `08e94a4`
- 2026-07-16 — **L0 #51 boundary — P116 planning tail COMPLETE (planner PASS + checker PASS)** — dispatched `gsd-planner` (opus, standalone) → 3 wave-1 parallel plans (`116-01/02/03-PLAN.md`, ZERO file-overlap) + populated `116-VALIDATION.md` body + ROADMAP P116 annotation (`011096b`); filed `GTH-V15-40` (array-typed `.source` jq-crash gotcha, tag P126) after live-verifying it against the catalog, discharging #50's carried noticing (`74fb907`); dispatched `gsd-plan-checker` (sonnet, standalone) → **VERDICT PASS** (every load-bearing claim byte-checked vs disk; both req IDs land; LIVE-row-not-twin verified; FIX-03 design-only gated; zero file-overlap confirmed), eager-fixing two non-blocking WARNINGs in place: a vacuous `grep -A5` TAG-line check in `116-03-PLAN.md` T2 widened to `-A34`; `116-RESEARCH.md` Open Questions marked RESOLVED (`9dbb860`). **P116 planning is now COMPLETE — P116 EXECUTION (top-level, 3 parallel plans, zero overlap) is the next work.** — `011096b`, `74fb907`, `9dbb860`

## NOW

**P116 PLANNING IS COMPLETE — planner and plan-checker both landed PASS this rotation.**
`116-01-PLAN.md` / `116-02-PLAN.md` / `116-03-PLAN.md` (all wave 1, parallel, ZERO
file-overlap) + a populated `116-VALIDATION.md` body + the ROADMAP P116 annotation landed
in `011096b`; `gsd-plan-checker` returned **VERDICT PASS** in `9dbb860` after two
non-blocking WARNINGs were eager-fixed in place (a vacuous grep-gate widened in
`116-03-PLAN.md` T2; `116-RESEARCH.md` Open Questions marked RESOLVED). **These 3
commits (`011096b`, `74fb907`, `9dbb860`) are local-only as of this NOW refresh — the L0
orchestrator pushes them together with this PROGRESS/handover commit as the rotation's
final act; #52's first action must confirm that push landed and CI concluded green on
the new tip.**

> **POST-PUSH CORRECTION (#51, 2026-07-16):** the push LANDED — HEAD == origin/main ==
> `79200b7`, tree clean, `0  0`. CI on `79200b7` is **UNKNOWN, not red** — GitHub's Actions
> API was 503ing globally at relief (~22:15 UTC); `git push` (git protocol) was unaffected,
> so the commits ARE on `origin/main`. **#52 must certify the `79200b7` CI run green as its
> first act once the API recovers** (full detail: SESSION-HANDOVER §1 correction block).

**Next work = P116 EXECUTION** (ROADMAP marks `Execution mode: top-level` — the
top-level coordinator IS the executor, never `/gsd-execute-phase`). The 3 plans, all
wave 1 / parallel / zero file-overlap:
- **116-01** (ADR-01): bless webhook+cron mirror-convergence as `"authoritative"` in
  `docs/concepts/dvcs-topology.md` + root `CLAUDE.md`; mint a catalog-first doc-alignment
  regression guard (`quality/gates/docs-alignment/mirror-convergence-blessed.sh` + row).
  Verify keys on `"authoritative"` (0 occurrences today), NOT `"webhook"` (already
  present — would be tautological).
- **116-02** (ADR-01 + FIX-03): `docs/decisions/010-l2-l3-cache-coherence.md` — §2
  amendment (RBF-LR-04 lever CLOSED), §3 amendment (Option B sanctioned target design,
  **design-only**), References cross-link (not a file move — closes ROADMAP criterion-1).
- **116-03** (ADR-01 + FIX-03): retire the **LIVE**
  `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md:108-116` litmus row (terminal
  RESOLVED, never delete, NOT the archived v0.14.0 twin — this is the ledger's FIRST
  terminal row); update `GOOD-TO-HAVES-09` to sanctioned-target-design + boundary-relative
  TAG.

Also filed `GTH-V15-40` (array-typed `.source` jq-crash gotcha, tag P126) this rotation,
discharging the #50→#51 carried noticing.

**P115 human confirm-retire gate STILL OPEN at 11 rows** — re-verify live at every
boundary: `grep -c '"last_verdict": "RETIRE_PROPOSED"' quality/catalogs/doc-alignment.json`
→ **11**, unchanged this rotation too. The verifier's GREEN-CHECKPOINT verdict
(`115-VERIFICATION.md`, `ce4d3b7`) stands; sole remaining action is the human-only 11-row
confirm-retire batch (`115-UNWAIVE-PATH.md` §"FINAL consolidated confirm-retire batch").
`STATE.md`'s cursor is deliberately held at P114-closed (P115 CHECKPOINTED, P116
mid-plan) until the batch lands — re-check at every boundary; when it drops below 11,
advance the cursor past P115 and close the checkpoint.

**L0 relieved #51→#52 at this boundary.** #51's rotation ran the P116 planning tail to
completion via standalone-subagent dispatches (planner + checker both kept out of L0's
own context) plus one filed noticing — see `.planning/SESSION-HANDOVER.md` (#51→#52) for
full detail, including a sharpened context-budget lesson: dispatch a fixer subagent for
eager-fixes to large plan/research files rather than reading them into the coordinator's
own context (that's what cost #51 the budget this rotation, not the dispatches
themselves). **Next actor: #52 — P116 EXECUTION is the primary work; dispatch the 3
wave-1 plans (parallel, zero overlap) per their `Execution mode: top-level` +
tier rules, then run the phase-close push cadence + verifier dispatch.**

**1/15 v0.15.0 "Floor" phases complete (P115 CHECKPOINTED pending the human gate; P116
planning complete, execution next; P117–P128 not started).**

## NEXT

1. **P117/P119 shaping input (owner mandate, 2026-07-16):** the docs site should read as
   a FURNISHED PRODUCT with streamlined documentation — owner verbatim: *"Its good, but we
   can do so much better!"* Covers information architecture, progressive disclosure,
   visual polish, and a cold-reader rubric pass over every landing surface; both P117 and
   P119 planners must fold this in as an explicit acceptance-bar input. Also owner-approved
   same session: embed the owner's 80s launch animation on the mkdocs home page as a P117
   scope addition (productionization checklist filed). Full text: `GOOD-TO-HAVES.md`
   GTH-V15-36 (quality bar) / GTH-V15-37 (animation embed); annotated inline on
   `.planning/ROADMAP.md` Phase 117 + Phase 119.
2. **P116 — planning COMPLETE, EXECUTION is now the primary work item.** ADR-010 decision
   packet was RULED 2026-07-16 (manager, decide-and-disclose, owner veto window open):
   Decision 1 (ADR-01 mirror fan-out) — Option B with A folded in; Decision 2 (FIX-03
   slug→id) — Option A this milestone (design-only), Option B recorded as sanctioned
   target design, NO v0.15 build. Verbatim rulings: the two 2026-07-16 `[MANAGER]`
   entries in `CONSULT-DECISIONS.md`; packet at
   `.planning/phases/115-live-mcp-benchmark-re-measurement/P116-ADR-010-DECISION-PACKET.md`.
   3 wave-1 parallel plans now on disk (`116-01/02/03-PLAN.md`) — execute per ROADMAP
   `Execution mode: top-level`, checkpoint housekeeping (`GTH-V15-35`) already complete,
   sequenced only behind nothing (the P115 human gate does not block it).
3. Then the remaining milestone phases:
   - P117 — Doc-truth launch-blocker purge — not started
   - P118 — Post-bench honesty corrections — not started
   - P119 — Docs/planning simplification (the "P112 RAISE") — not started
   - P120 — CLI + helper error hardening to Rust-compiler-grade — not started
   - P121 — RPX error-code namespace + `reposix explain` — not started
   - P122 — `reposix-remote` + `init` hardening (HIGH carry-forwards) — not started
   - P123 — Quality-runner & catalog integrity hardening — not started
   - P124 — Container-rehearse harness hardening — not started
   - P125 — Real-backend cadence & mirror-drift resilience — not started
   - P126 — Docs-alignment tooling polish — not started
   - P127 — Surprises absorption (OP-8 Slot 1) — not started
   - P128 — Good-to-haves polish + milestone close (OP-9 Slot 2) — not started
