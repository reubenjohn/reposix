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

## NOW

**P115 phase-close cadence — the ONLY remaining item.** T6 (all 7 items) is COMPLETE,
agent-side, on `main`; the pre-close owner-directive doc-narrative strip has also SHIPPED
(`5a5dd29`, zero new rows propose-retired). The **sole remaining action is a HUMAN-ONLY
confirm-retire** on an **11-row batch** of `WAIVED-RETIRE_PROPOSED` doc-alignment rows —
authoritative list + copy-paste commands now consolidated in
`.planning/phases/115-live-mcp-benchmark-re-measurement/115-UNWAIVE-PATH.md` § "FINAL
consolidated confirm-retire batch" (supersedes the scattered "11 rows" mentions elsewhere).
Verb: `reposix-quality doc-alignment confirm-retire --row-id <id>` (real TTY only, refuses
`$CLAUDE_AGENT_CONTEXT`). Manager (w1:p7) pinged twice (initial 8-row ask, then the
corrected 11-row ask) and relays to the owner; standing instruction is NOT to idle-wait —
checkpoint at close-readiness if the pass hasn't landed. Once it lands (or the phase
checkpoints without it per that instruction):

1. Re-push + re-run `python3 quality/runners/run.py --cadence post-push --persist` if new
   commits land.
2. Dispatch a `gsd-verifier` subagent for catalog-row PASS grading (RED loops back;
   expect the 11 waived rows to read as the documented human gate, not a silent failure).
3. Advance `.planning/STATE.md` cursor past P115.
4. Refresh `PROGRESS.md` in the close push.
5. **Also owed before declaring the T6 hero-surface edits shipped:** a cold-reader pass
   (`/doc-clarity-review` on `docs/index.md` + `README.md`) — T6's lanes made minimal
   mechanical number swaps; the surrounding framing/prose was not re-reviewed.

## NEXT

1. **Post-P115-close eager-fix (owner-directed, tracked `/gsd-quick`):** nest
   `docs/index.md`'s "Build from source (advanced)" material under the "30-second
   install" section. Two execution cautions (verbatim, MUST hold): (a) the
   `structure/install-leads-with-pkg-mgr-docs-index` freshness gate must stay green
   (install path leads with package manager); (b) doc-alignment catalog rows bound to
   `docs/index.md` must be checked BEFORE the edit and refreshed in the same wave. Full
   text: `GOOD-TO-HAVES.md` GTH-V15-35.
2. **P117/P119 shaping input (owner mandate, 2026-07-16):** the docs site should read as
   a FURNISHED PRODUCT with streamlined documentation — owner verbatim: *"Its good, but we
   can do so much better!"* Covers information architecture, progressive disclosure,
   visual polish, and a cold-reader rubric pass over every landing surface; both P117 and
   P119 planners must fold this in as an explicit acceptance-bar input. Also owner-approved
   same session: embed the owner's 80s launch animation on the mkdocs home page as a P117
   scope addition (productionization checklist filed). Full text: `GOOD-TO-HAVES.md`
   GTH-V15-36 (quality bar) / GTH-V15-37 (animation embed); annotated inline on
   `.planning/ROADMAP.md` Phase 117 + Phase 119.
3. **P116 — ADR-010 decision packet** (ADR-01 mirror-fanout + FIX-03 GTH-09 slug→id
   durable-create; options + tradeoffs, NO implementation) — **COMMITTED** at
   `.planning/phases/115-live-mcp-benchmark-re-measurement/P116-ADR-010-DECISION-PACKET.md`
   (rescued from `/tmp` by #44, 2026-07-16; recommends B for ADR-01, A-now/B-as-target
   for FIX-03) → route to **MANAGER (w1:p7) for ruling** — end turn and await the
   ruling before any implementation. Owner's mp4 export path also filed onto
   GTH-V15-37 (charter addendum).
4. Then the remaining milestone phases:
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
