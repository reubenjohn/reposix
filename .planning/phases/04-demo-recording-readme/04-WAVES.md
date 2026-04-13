---
phase: 04-demo-recording-readme
type: waves
---

# Phase 4 — Wave / Execution Order

**Execution: SERIAL.** Two plans, no parallelism.

| Wave | Plan | Rationale |
|------|------|-----------|
| 1 | `04-01-demo-script-and-recording.md` | Produces `scripts/demo.sh`, seed extension, `docs/demo.typescript` (the `script(1)` recording), `docs/demo.transcript.txt`, `docs/demo.md`. The recording depends on the script existing. |
| 2 | `04-02-readme-polish-and-final-verify.md` | Updates `README.md` to link to `docs/demo.md` + typescript, runs the final verification sweep (`cargo test`, `cargo clippy`, `gh run list`), and pushes the commit that closes SG-08 / FC-09 / FC-08. Cannot start until Wave 1 has produced the artifacts README must reference. |

## Why serial

- Plan 2's README additions reference concrete filenames (`docs/demo.typescript`, `docs/demo.transcript.txt`, `docs/demo.md`) that only exist after Plan 1.
- Plan 2's final `gh run list` verification must run against a commit that already includes the demo artifacts; otherwise green CI doesn't prove the demo works.
- The two plans touch disjoint files (`scripts/` + `docs/` vs `README.md`), so the sequencing is not about write conflicts — it is about the causal dependency of *reviewing* what Plan 1 produced before Plan 2 writes prose about it.

## Wall-clock budget

| Plan | Target | Hard cap |
|------|--------|----------|
| 04-01 | 55 min | 65 min |
| 04-02 | 25 min | 30 min |
| **Total** | **80 min** | **95 min** (phase budget 90 min — 5 min slack consumed only if 04-01 overruns) |

## Dependencies external to this phase

None. Phases 1–3 + S are shipped and green on `main` (`S-DONE.md` confirms). Phase 4 is pure demo + documentation polish; no production-code changes.
