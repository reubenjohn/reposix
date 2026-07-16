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
- 2026-07-16 — **T6 headline reframe (item 1) — LANDED + PUSHED** — hero surfaces re-anchored from the retired synthetic **89.1%** to the live GitHub-capture headline **~94% fewer output tokens / ~75% cheaper per session** (output ~94.3% / cache-create ~66.0% / input-context ~55.6% / cost ~74.9%), matching `token-economy.md`'s provenance framing so heroes + benchmark page tell one story. Touched: `README.md` "Three measured numbers", `docs/index.md` (token card + loop-diagram Notes + caption + token-economy card + honest-scope footer), `docs/concepts/reposix-vs-mcp-and-sdks.md` "About the MCP comparison". Both live findings folded in (GitHub write-back read-only this cut → comparison unaffected; MCP `issue_read` lossy vs reposix byte-fidelity). banned-words + mkdocs-strict + mermaid-renders green. Committed `d2fd85c`; its own push attempt BLOCKED as designed on 4 no-waiver doc-alignment rows freshly `STALE_DOCS_DRIFT` — cleared via the top-level `/reposix-quality-refresh` retire+rebind (`9a2b6f1`, 6 rows re-graded) + a time-boxed `waive` on the still-blocking 5 (`c9c2aee`, until 2026-08-15, tracked to `115-UNWAIVE-PATH.md`). Push landed, pre-push 61 PASS / 0 FAIL. `code/ci-green-on-main` P0 pending confirmation of the post-push CI run (`29491742214`, in flight at handover). — `d2fd85c` (refresh `9a2b6f1`, waive `c9c2aee`) ✅
- 2026-07-16 — **T6 item 3 — AGENT-SIDE DONE** — the 6 synthetic `count_tokens`-over-fixture `token-economy.md` doc-alignment rows (76.4% / 85.5% / jira-real-adapter / 4,883 / 531 / 89.1%) are `RETIRE_PROPOSED` (agent-side propose-retire only; HUMAN-ONLY confirm-retire NOT run, env-guard untouched). Replacement rows for the LIVE four-axis figures are `BOUND`/GREEN with fresh hand-verified citations: `output-reduction-94-percent` (`token-economy.md:37`), `cost-reduction-75-percent` (`:40`), `live-github-capture-methodology` (`:8-13`) — each bound to `bench_token_economy.py` + `test_bench_token_economy.py` (AND-drift watch). Verified against reality: pytest 9 passed offline; doc regenerates byte-for-byte from committed captures; catalog delta +3 rows / 0 removed (`claims_bound` 263→266). Pre-push walk `rc=0`, zero blocking (6 rows `WAIVED-RETIRE_PROPOSED`, waiver reason refreshed to accurate post-rebind guidance, same `until=2026-08-15` / `tracked_in=115-UNWAIVE-PATH.md`). Evidence: `.planning/phases/115-live-mcp-benchmark-re-measurement/115-T6-CLOSEOUT.md` § Wave 1 — item 3 agent-side. **Pending human relay:** batch `confirm-retire` for these 6 + the 2 concepts-page `RETIRE_PROPOSED` rows (`token-baseline-mcp-4883` / `token-baseline-reposix-531`) into one manager ask. ✅
- 2026-07-16 — **T6 item 5 — DONE** — `emit-markdown.sh` now refuses to clobber `docs/benchmarks/latency.md`'s CI-canonical sections. New `quality/gates/perf/latency-bench/regen-guard.sh` gates the write on a `reposix:regen-guard:protected-begin` marker (placed at end-of-file so it never shifts the 14 doc-alignment citations line-anchored above it — an earlier top-of-file placement tripped `STALE_DOCS_DRIFT` on all 14, caught by walk.sh before commit, fixed by relocating the marker); refuses with a teaching error (what/why/copy-paste recovery) unless `REPOSIX_LATENCY_BENCH_ALLOW_CANONICAL_OVERWRITE=1`. Verified against reality against `/tmp` destinations only: fresh regen still works, a `/tmp` copy of the real doc is refused byte-identical, override proceeds; the real committed `latency.md` itself trips the guard. New `regen-guard.selftest.sh` (12 assertions, follows the `file-size-limits.selftest.sh` convention) passes; docs-alignment walk / banned-words / mkdocs-strict / mermaid-renders all green. Also fixed a lying doc claim (Reproduce prose asserted a protection that didn't exist in code). Filed `GTH-V15-28` (line-anchored doc-alignment citations are a general sharp edge for future doc edits). Evidence: `115-T6-CLOSEOUT.md` § Wave 2 — item 5. ✅

## NOW

**T6 remaining — item 1 (reframe) LANDED + PUSHED, CI-green confirmation pending; item 3
agent-side DONE (HUMAN-ONLY confirm-retire pending); items 2/5/7 DONE; item 6 next.** Numbering matches the T6 charter in
`.planning/SESSION-HANDOVER.md` §5 (item 4's second `latency.md` refresh is DROPPED — not
needed, `latency.md` never re-drifted):
1. **(item 2) — DONE.** `115-UNWAIVE-PATH.md` written in the P115 phase dir — live-grepped
   both catalogs and confirmed exactly 19 waived doc-alignment rows (8 pre-existing hero +
   6 token-economy.md + 5 newly time-boxed at `c9c2aee`) + 2 perf rows
   (`perf/token-economy-bench` / `perf/headline-numbers-cross-check`), matching the
   expected count exactly. Discrepancies found (state descriptions, stale `tracked_in`
   tags, a likely row-ID dup) documented in the doc + `115-T6-CLOSEOUT.md` § Wave 2 —
   item 2. Pre-push wall-time creep (141s at `d7da383`) filed as a third corroborating
   `SURPRISES-INTAKE.md` entry.
2. **(item 3) — AGENT-SIDE DONE, HUMAN CONFIRM-RETIRE PENDING.** The 6 `token-economy.md`
   rows (76.4% / 85.5% / 4883 / 531 / 89.1% / jira-real-adapter) are now `RETIRE_PROPOSED`
   and the live four-axis replacements are `BOUND`/GREEN (`output-reduction-94-percent` /
   `cost-reduction-75-percent` / `live-github-capture-methodology`); walk `rc=0`. Evidence:
   `115-T6-CLOSEOUT.md` § Wave 1 — item 3. **Remaining = HUMAN-ONLY confirm-retire** on
   those 6 + the 2 concepts-page rows `9a2b6f1` marked `RETIRE_PROPOSED`
   (`token-baseline-mcp-4883` / `token-baseline-reposix-531`) — batch all 8 into one
   manager w1:p7 ask.
3. **(item 5) — DONE.** Regen-clobber guard shipped: `emit-markdown.sh` refuses to
   overwrite `latency.md`'s CI-canonical sections (marker + teaching error +
   `regen-guard.selftest.sh`). Evidence: `115-T6-CLOSEOUT.md` § Wave 2 — item 5.
4. **(item 7) — DONE.** All FIVE `[SELF]` decision entries deleted from `.planning/CONSULT-DECISIONS.md`: A1 (line 71), T6 (line 96), T2 (line 114), T5 (line 123), T4 (line 153). Companion note at line 159 deleted. Post-grep confirms only the definition at line 6 remains; file structure intact (70 lines, clean EOF). Evidence: `115-T6-CLOSEOUT.md` § Wave 2 — item 7.
5. **(item 6) — NEXT.** Un-waive the 8 hero-number rows + the 3 newly-waived
   MISSING_TEST rows from `c9c2aee`; un-waive `perf/token-economy-bench` by adding the
   ~94% headline assertion; write the missing
   `quality/gates/perf/headline-numbers-cross-check.py` verifier and un-waive the
   EXISTING `perf/headline-numbers-cross-check` catalog row (correction: this row
   already exists, P90-era — it is NOT absent, only its verifier script is). Must also
   reconcile the "8 ms" hero figure against the canonical `latency.md` (now "6 ms get /
   7 ms list").
6. **(item 8)** Phase-close cadence: push → `code/ci-green-on-main` P0 → gsd-verifier →
   `STATE.md` cursor → `PROGRESS.md` refresh in the close push.

## NEXT

1. P115 closed, CI green on main
2. P116 ADR-010 decision packet (mirror-fanout + slug→id durable-create) → manager ruling
3. then the remaining milestone phases:
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
