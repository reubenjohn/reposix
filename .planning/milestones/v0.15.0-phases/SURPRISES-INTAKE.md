# v0.15.0 Surprises Intake

> **Append-only intake for surprises discovered during v0.15.0-era execution
> (and items routed forward from prior milestones).**
> Each entry is something the discovering session chose NOT to fix eagerly because it was
> out-of-scope. A v0.15.0 drain phase (OP-8 Slot 1) closes this file.
>
> **Eager-resolution preference:** if a surprise can be closed inside its discovering
> phase without doubling scope (rough heuristic: < 1 hour incremental work, no new
> dependency, no new file outside the phase's planned set), do it there. This file is for
> items that genuinely don't fit.
>
> **Distinction from `GOOD-TO-HAVES.md`:** entries here fix something that's BROKEN,
> RISKY, or BLOCKING. Improvements/polish go in `GOOD-TO-HAVES.md`.

## Entry format

```markdown
## YYYY-MM-DD HH:MM | discovered-by: <source> | severity: BLOCKER|HIGH|MEDIUM|LOW

**What:** One-paragraph description of what was found.

**Why out-of-scope for the discovering session:** Why eager-resolution wasn't possible.

**Sketched resolution:** One paragraph proposing how the drain phase should resolve.

**STATUS:** OPEN  (← drain phase updates to RESOLVED|DEFERRED|WONTFIX with rationale or commit SHA)
```

---

## Entries

## 2026-07-13 14:00 | discovered-by: quick 260713-q0e (RED-main honest-rework, Manager Ruling #5 Option A) | severity: MEDIUM

**What:** The RED-main fix that unblocked `quality-post-release` (`quality/gates/docs-repro/container-rehearse.sh` now emits each catalog row's `expected.asserts` verbatim as `asserts_passed` on container exit 0) makes the F-K4b per-expected-assert congruence gate (`quality/runners/_audit_field.py::asserts_congruent`) a **TAUTOLOGY for every `kind: container` docs-repro row**: because the emitted `asserts_passed` strings are byte-identical to the `expected.asserts`, the token-map is self-congruent by construction, so a no-op `exit 0` script would pass F-K4b just as a real one does. Honesty currently rests entirely on two out-of-band properties: (a) each example `run.sh` being FAIL-LOUD (`set -euo pipefail` + one real end-to-end assert per catalog assert, so exit 0 genuinely ⟺ the asserts hold — verified true today for examples 01/02/04), and (b) each catalog assert describing only what exit 0 load-bearingly establishes. **Sub-item surfaced during the same rework:** example-05's asserts #2/#3 violated (b) — they claimed the agent "observes the blob-limit error from helper stderr and recovers via git sparse-checkout" and that "siblings stay sparse," but `run.sh` provably never reads helper stderr (the fast-import fetch path bypasses the per-RPC `command=fetch` blob-limit check; `run.sh:28` greps a source CONSTANT; `expected-output.md` documents zero `blob_limit_exceeded` rows) and `run.sh:51`'s bare `ls issues/*.md` only needs ≥1 file. Those two asserts were reworded to the truth in quick 260713-q0e (pre-emptive sparse-checkout pattern + source-constant presence only); the tautology itself remains.

**Why out-of-scope for the discovering session:** Manager Ruling #5 (Option A) scoped the RED-main resolution to a bounded, reversible honest fix (keep the fail-loud emission for 01/02/04, reword example-05 to the truth) and explicitly deferred the structural redesign to v0.15.0. Redesigning F-K4b's container-class grading or building a fail-loud meta-check touches the load-bearing quality-honesty contract (E2 valve) and is a larger, ADR-adjacent design task; doing it inside a RED-main hotfix would be scope surgery under time pressure. B (restore waivers) was rejected (reverses shipped P106); C (full redesign now) was deferred here.

**Sketched resolution (TWO sub-items, one row):**
  (a) **F-K4b container-class tautology redesign.** Make container-row congruence EARNED, not emitted. Two candidates: (1) per-step-earned emission like `quality/gates/docs-repro/tutorial-replay.sh` — each example script prints a machine-parseable `ASSERT-PASS: <text>` line only after the step that establishes that specific assert actually succeeds, and container-rehearse harvests those instead of copying `expected.asserts` verbatim; OR (2) a fail-loud meta-check that asserts each container script aborts (non-zero) when any single one of its asserts is made false (a mutation-style smoke test), so exit 0 carries real congruence weight. Prefer (1) — it mirrors the already-honest `tutorial-replay.sh` and gives per-assert evidence.
  (b) **example-05 real-runtime-error deeper fix.** Today example-05 exercises only the PRE-EMPTIVE sparse-checkout pattern + the source-constant presence; the REAL runtime blob-limit error (`BLOB_LIMIT_EXCEEDED_FMT` firing on a `command=fetch` RPC that exceeds `REPOSIX_BLOB_LIMIT`) is exercised only by `quality/gates/agent-ux/dark-factory.sh`. Make example-05 drive the real runtime error + recovery cycle (needs the helper to take the per-RPC blob-limit path on the example's fetch — cf. the v0.10 stateless-connect-only read path noted in `examples/05-blob-limit-recovery/expected-output.md`), so the example teaches the genuine observe-error → `git sparse-checkout set` → retry loop rather than a pre-emptive stand-in.

**STATUS:** OPEN

## 2026-07-13 20:30 | discovered-by: b773c04 RED-main arc (SESSION-HANDOVER successor #16 noticing, routed by item-0 cursor refresh) | severity: MEDIUM

**What:** `quality/gates/docs-repro/container-rehearse.sh` backgrounds the ephemeral sim (`&`) and relies on a bash `EXIT` trap to tear it down. When the runner's `subprocess.run(timeout=...)` SIGKILLs the harness — exactly what happened in the original b773c04 CI failure ("Terminate orphan process pid 15322") — the EXIT trap NEVER fires, so the sim orphans on host port 7878. A later container row can then bind-fail on 7878, or silently `curl` a stale sim from a prior run, producing a false pass or a confusing flake. This is the robustness gap underneath the sim-readiness race already noted on back-to-back local runs.

**Why out-of-scope for the discovering session:** The b773c04 fix-first charter was scoped to greening the RED-main gate via the timeout-budget edit (drop unused apt packages + bump `timeout_s` 300→600); hardening the harness process lifecycle is a separate quality-gate-script change (touches `quality/gates/docs-repro/container-rehearse.sh` internals, which the reality-check arc is NOT owner-ratified to mutate for defect lanes) and belongs to a v0.15.0 drain phase, not a RED-main hotfix.

**Sketched resolution:** Make the sim teardown SIGKILL-proof rather than trap-dependent — wrap the `docker run` in an internal `timeout` strictly shorter than the row's catalog `timeout_s` so the harness reaps its own children before the outer SIGKILL fires, AND/OR start the sim in its own process group (`setsid` / `set -m`) and kill the whole group on teardown so an orphaned sim cannot survive a hard kill. Pair with a pre-`docker run` port-7878-free wait so a stale sim is detected (and fail-loud) rather than silently reused.

**STATUS:** OPEN

## 2026-07-13 20:30 | discovered-by: b773c04 RED-main arc (SESSION-HANDOVER successor #16 noticing, routed by item-0 cursor refresh) | severity: MEDIUM (verify — provenance unconfirmed, not a proven defect)

**What:** `quality-post-release.yml` has no obvious `cargo build -p reposix-cli` step, yet `container-rehearse.sh` needs the pre-built `target/debug/reposix` binary host-mounted (`-v target:...:rw`, on PATH via `--network host`) — the examples run that binary, not an in-container build. Run 29302973970 SUCCEEDED, so the binary WAS present, but WHERE it came from (a cache restore? a prior-job artifact download? an unnamed/implicit build step?) is UNCONFIRMED. If provenance is an incidental cache hit rather than an explicit build, a cold runner (cache miss, no prior job) could silently degrade every `kind:container` docs-repro row to NOT-VERIFIED — the global-CLAUDE "never let a metric you don't watch decay" failure mode.

**Why out-of-scope for the discovering session:** Confirming the binary provenance requires reading the `quality-post-release.yml` job graph + a live run's logs (~10 min) and possibly a workflow edit — orthogonal to the timeout-budget RED-main fix, and workflow-file mutation is outside the b773c04 charter. This is a "verify, don't assume" item: the row passed, but the guarantee is unproven, so it is filed rather than eager-fixed.

**Sketched resolution:** Trace how `target/debug/reposix` reaches the `quality-post-release` runner (read the workflow + a run's step logs). If it is an implicit cache hit, add an explicit `cargo build -p reposix-cli` (or an `actions/download-artifact` from the release build) as a hard dependency of the container-rehearse step, so the container rows are provenance-guaranteed on a cold runner rather than silently NOT-VERIFIED. If an explicit step already exists, document it inline so the next reader does not re-open this question.

**STATUS:** OPEN
