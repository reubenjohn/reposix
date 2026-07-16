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
- 2026-07-16 — `docs/benchmarks/token-economy.md` regenerated from the live GitHub captures — the synthetic count_tokens-on-fixtures baseline (retired 89.1% / 85.5% figures) is replaced by a deterministic, offline, no-API-key headline computed from the committed `benchmarks/captures/*.json` session-usage records: **~94% fewer output tokens, ~75% cheaper per session** (four axes: output ~94.3% / cache-create ~66.0% / total input-context ~55.6% / cost ~74.9%). Provenance + methodology rewritten (kills the false `scripts/demo.sh` / "modeled on Forge" claims), read-only-write-back + MCP-lossy-reads honesty caveats added, stale sidecar deleted (GTH-V15-26 resolved). — `1cdb381` ✅

## NOW

**T6 — un-waive + headline reframe + phase close.** The live token-economy numbers now
live in `docs/benchmarks/token-economy.md`; T6 re-anchors the *hero* surfaces to them.
Decide the headline framing (re-anchor the README / `docs/index` / `docs/why` hero to the
live GitHub number, OR keep 89.1% with an explicit GitHub-regime caveat), folding in the two
T4 findings (no GitHub write-back claim; reposix byte-fidelity positioning). Then: the second
`latency.md` honest-headline refresh; un-waive the 8 hero-number rows; **retire (confirm-retire
is human-only) + rebind the 6 `token-economy.md` doc-alignment rows** T5 left `WAIVED-STALE_DOCS_DRIFT`
until 2026-08-15 (76.4% / 85.5% / 4883 / 531 / 89.1% / jira-real-adapter); un-waive the
`perf/token-economy-bench` catalog row by adding the ~94% headline assertion; write
`115-UNWAIVE-PATH.md`; phase-close cadence (push → CI green → verifier).

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
