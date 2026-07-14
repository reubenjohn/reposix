# Phase 84: Webhook-driven mirror sync — GH Action workflow + setup guide — Research

**Researched:** 2026-05-01
**Domain:** GitHub Actions workflow authoring; `repository_dispatch` triggers; `--force-with-lease` race protection; webhook-vs-cron dual-trigger orchestration; real-backend latency measurement against TokenWorld + reposix-tokenworld-mirror.
**Confidence:** HIGH (all upstream substrates shipped — P80 mirror refs, P82 URL parser, P83 bus write fan-out; the mirror repo itself exists per CARRY-FORWARD § DVCS-MIRROR-REPO-01; workflow shape is ratified verbatim in `architecture-sketch.md` § "Webhook-driven mirror sync"; only first-run + latency-measurement details are DEFERRED-to-implementation in `decisions.md` Q4.3/Q4.1).

## Summary

P84 is a **mostly-YAML + integration-test phase**. The reference workflow shape is ratified verbatim in `architecture-sketch.md`; the four ratified Q4 decisions (Q4.1 cron `*/30` configurable, Q4.2 webhook-less backends documented but not implemented, Q4.3 first-run handled gracefully) leave only two implementation details to resolve: (a) the EXACT first-run `--force-with-lease` invariant against an empty mirror with no `mirror/main` ref, and (b) the latency measurement methodology (recommended: real TokenWorld + a synthetic harness for CI repeatability).

Phase scope is narrow because all the heavy lifting already shipped: P80 mints `refs/mirrors/<sot>-head` + `refs/mirrors/<sot>-synced-at` ref helpers (`crates/reposix-cache/src/mirror_refs.rs`); P83 wires bus pushes to update both refs (the workflow becomes a **no-op refresh** when the bus already touched them, per Q2.3); `cargo binstall` metadata is present in `crates/reposix-cli/Cargo.toml:19` (binary distribution path is ready); the real GH mirror at `github.com/reubenjohn/reposix-tokenworld-mirror` is provisioned with `gh` auth scopes confirmed (CARRY-FORWARD § DVCS-MIRROR-REPO-01).

**Primary recommendation:** Single-plan with **6 tasks** — catalog rows first, then YAML, then three integration tests (first-run, race, latency), then a CLAUDE.md update + close. No phase split needed (P84 is much narrower than P83's risk surface). Place the workflow file at `.github/workflows/reposix-mirror-sync.yml` IN THE MIRROR REPO (`reubenjohn/reposix-tokenworld-mirror`), NOT in `reubenjohn/reposix` — per CARRY-FORWARD § DVCS-MIRROR-REPO-01 P84 bullet, *"the GH Action workflow at `.github/workflows/reposix-mirror-sync.yml` lands in THIS repo, not in `reubenjohn/reposix`."* This is load-bearing and easy to miss; surface it in the plan's first task.

## Chapters

| Chapter | Sections |
|---------|----------|
| [ch1-design.md](./ch1-design.md) | Architectural Responsibility Map · User Constraints · Phase Requirements · Standard Stack |
| [ch2-yaml-and-semantics.md](./ch2-yaml-and-semantics.md) | Workflow YAML Shape · `--force-with-lease` Semantics · First-run Handling (Q4.3) |
| [ch3-measurement-and-ops.md](./ch3-measurement-and-ops.md) | Latency Measurement Strategy · Secrets Convention · Backends Without Webhooks (Q4.2) |
| [ch4-quality-and-security.md](./ch4-quality-and-security.md) | Catalog Row Design · Test Infrastructure · Plan Splitting Recommendation |
| [ch5-pitfalls-and-code.md](./ch5-pitfalls-and-code.md) | Common Pitfalls (7) · Code Examples |
| [ch6-runtime-and-validation.md](./ch6-runtime-and-validation.md) | Runtime State Inventory · Environment Availability · Validation Architecture · Security Domain |
| [ch7-context-and-sources.md](./ch7-context-and-sources.md) | State of the Art · Assumptions Log · Open Questions · Project Constraints · Sources · Metadata |
