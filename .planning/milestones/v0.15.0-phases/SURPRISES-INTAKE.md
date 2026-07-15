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

## 2026-07-14 21:00 | discovered-by: quick 260714-rcv (L0 rotation #21 post-tag cursor refresh — carried noticing from rotation #20) | severity: MEDIUM

**What:** Two milestone-scoped `GOOD-TO-HAVES.md` ledgers are over the `structure/file-size-limits` 20000-char `*.md` ceiling: `.planning/milestones/v0.14.0-phases/GOOD-TO-HAVES.md` = **27629 chars** (the ~27.6k figure carried from #20) and `.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md` = **23584 chars**. (Scope correction vs the #20 hand-off, which named `.planning/GOOD-TO-HAVES.md`: that root file is only **4525 chars** — NOT oversized; the oversized files are the two milestone-scoped ones.) The 20k breach is currently MASKED repo-wide by the `structure/file-size-limits` OVER-BUDGET-tier `--warn-only` waiver expiring **2026-08-08T00:00:00Z** (`quality/catalogs/freshness-invariants.json` L666); when it lapses the over-budget tier reactivates `exit 1` and BLOCKS the push. SEPARATELY, the v0.14.0 file (27629) also breaches the distinct `agent-ux/p111-milestone-hygiene` **24000-byte** no-ballooning ceiling (`quality/gates/agent-ux/p111-milestone-hygiene.sh:98`, `check_size "${GTH}" 24000`); that gate genuinely `exit 1`s today (catalog `status: FAIL`, `waiver: null`, verified by running it) but is **on-demand / milestone-bounded**, NOT wired into pre-push/CI, so it does NOT gate main (main is green at 6aa734a) — it is a latent RED that a milestone-close re-run would surface.

**Why out-of-scope for the discovering session:** #21 is a planning-artifact-only cursor refresh + intake-filing quick (no code, no gate rework); the fix is a progressive-disclosure file split across two ledgers with row-ID cross-ref preservation — larger than the cursor task's bounded scope. Filing it into `GOOD-TO-HAVES.md` would be self-defeating (this row flags that very file class as oversized), so it is filed here in SURPRISES-INTAKE.

**Sketched resolution:** Apply the same progressive-disclosure split already landed for `ORCHESTRATION.md → ORCHESTRATION-REFERENCE.md`, `ROADMAP.md → ARCHIVE.md`, and `STATE.md → STATE-history.md`: split each oversized `GOOD-TO-HAVES.md` into a lean live ledger + a `GOOD-TO-HAVES-REFERENCE.md` (or `-history.md`) companion, moving closed/terminal/DEFERRED entries out while PRESERVING the `GTH-*` / `GOOD-TO-HAVES-NN` row IDs and inbound cross-refs (catalogs + ROADMAPs cite these IDs). Natural home: the Arc D **v0.17 meta-milestone "bloat remediation"** bucket. Waiver clock: must land before **2026-08-08** or the `structure/file-size-limits` push-block reactivates.

**STATUS:** OPEN

## 2026-07-14 21:05 | discovered-by: quick 260714-rcv (L0 rotation #21 post-tag cursor refresh — carried noticing from rotation #20; scope corrected against reality) | severity: LOW

**What:** `.planning/milestones/v0.13.0-phases/ROADMAP.md` has **six broken `**Plan:**` markdown links** — P79 (L28), P80 (L39), P81 (L45), P82 (L51), P83 (L57), P84 (L63) — each pointing at a `NN-PLAN-OVERVIEW.md` FILE that never existed; the real artifact is a `NN-PLAN-OVERVIEW/` DIRECTORY (`index.md` + `chapter-1-architecture.md` + `chapter-2-execution.md`). Scope correction vs the carried #20 note ("P80–82 and P84–88"): P79 and P83 are ALSO broken (six total, not five); P78 (L22) is FINE (`78-PLAN-OVERVIEW.md` is a real file, not a dir); and P85–P88 have NO link at all — they read `**Plan:** TBD (Pxx plan-overview not yet authored)`, which is a SEPARATE staleness because those plans DO exist as `NN-01-PLAN.md` + `NN-01-SUMMARY.md`.

**Why out-of-scope for the discovering session:** Pre-existing cosmetic doc-link drift, unrelated to the #21 cursor-refresh charter; filed rather than eager-fixed to keep the cursor commit atomic (six ROADMAP link edits plus the optional P85–88 TBD rewrites is a distinct doc-hygiene pass).

**Sketched resolution:** Repoint the six P79–P84 links to the directory form — `NN-PLAN-OVERVIEW/` (whose `index.md` is the entry point), e.g. `[80-PLAN-OVERVIEW](80-mirror-lag-refs/80-PLAN-OVERVIEW/index.md)`. Secondary (same pass): replace the four P85–P88 `**Plan:** TBD (Pxx plan-overview not yet authored)` stubs with links to the real `NN-01-PLAN.md` plans so the ROADMAP stops claiming authored plans are unwritten.

**STATUS:** OPEN

## 2026-07-14 20:40 | discovered-by: L0 rotation #22 (t4 real-backend re-run, agent-ux/t4-conflict-rebase-ancestry-real-backend cadence) | severity: HIGH

**What:** Confluence `list_records`-vs-`get_record` oid drift breaks partial-clone checkout against live Confluence. `reposix init confluence::...` then `git checkout` aborts: `oid drift: requested 288fbcf7938289e822c36d758cace9efa98e5ab2, backend returned 959a0393bfff07a3d8faeb69293db0fd9d13ff54 for issue 7766017` — fails at `git checkout -B main`, BEFORE any push (read-only). Root cause traced to `crates/reposix-cache/src/builder.rs:610-618` (`read_blob`): the tree oid is computed from the `list_records` body at `reposix init`/`build_from` time; the blob is materialized from the `get_record` body at checkout time. For Confluence page 7766017 these two API representations render to DIFFERENT bytes, so the safety drift-check (working as designed) refuses to serve the blob and the partial-clone fetch aborts. Deterministic: re-ran validate-only, byte-identical oids both runs. Page 7766017 is an UNMUTATED protected fixture — so this is NOT the eventual-consistency race the code comment assumes; it is a systematic list-body vs get-body representation difference.

**Why out-of-scope for the discovering session:** Discovered mid a real-backend verification cadence run (t4), not a planned fix-first phase; the fix requires either aligning Confluence adapter rendering (`crates/reposix-core` backend connector) or reworking `build_from`'s oid computation in `reposix-cache` — both are new-scope code changes needing their own dedicated phase and test coverage, not an eager patch inside a cadence re-run.

**Sketched resolution:** Fix candidates: (1) align the Confluence adapter so `list_records` and `get_record` render the same body per page, OR (2) have `build_from` compute tree oids from the get-representation it will actually materialize from. Impact: this is THE reason the `agent-ux/t4-conflict-rebase-ancestry-real-backend` (P0) caveat does NOT retire; partial-clone checkout against live Confluence is broken for at least page 7766017, likely broader. Reliable reproducer via the t4 gate. Evidence: t4 cadence run 2026-07-15; log `/tmp/t4-realbackend-run.log`; artifact `quality/reports/verifications/agent-ux/t4-conflict-rebase-ancestry-real-backend.json`.

**STATUS:** OPEN

## 2026-07-14 20:41 | discovered-by: L0 rotation #22 (t4 real-backend re-run, same session as the oid-drift defect above) | severity: MEDIUM

**What:** Misleading t4 error message misattributes oid-drift to git version. The drift aborts with `requires git >= 2.34 stateless-connect fetch`, but git in the executing environment is 2.50.1 — the message misattributes an oid-drift/coherence failure to a git-version problem and teaches the wrong fix. Violates the root CLAUDE.md Ownership-charter north star ("every user-facing error must teach the fix").

**Why out-of-scope for the discovering session:** Fixing the message requires editing `quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh`'s error-classification logic — a harness-script change separate from the product-defect fix above and outside the scope of a real-backend cadence re-run.

**Sketched resolution:** `quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh` should surface the real stderr (`oid drift … for issue 7766017`) instead of falling back to the git-version message whenever the actual failure is an oid-drift abort rather than a stateless-connect/git-version gate failure.

**STATUS:** OPEN

## 2026-07-14 20:42 | discovered-by: L0 rotation #22 (t4 real-backend re-run, pre-release-real-backend cadence) | severity: MEDIUM

**What:** `pre-release-real-backend` cadence needs a documented mirror-refresh pre-step. The vision-litmus non-idempotency (documented in `.planning/milestones/v0.14.0-phases/surprises-intake/part-03.md` § litmus non-idempotency, Manager Ruling #2): the litmus's own successful push re-stales the GitHub mirror it reads, so a SECOND-run vision-litmus is RED unless `scripts/refresh-tokenworld-mirror.sh` runs FIRST. This is not wired into the cadence invocation. This caused a P0 vision-litmus FAIL in the 2026-07-15 t4 run that was a false-negative — the committed catalog PASS from 2026-07-13 remains legitimate on a freshly-refreshed mirror.

**Why out-of-scope for the discovering session:** Wiring a mandatory pre-step into the milestone-close cadence invocation (or making the litmus self-reconcile) is a harness/procedure change needing its own review, not an eager patch mid cadence-run; it also overlaps `GTH-V15-09` (self-reconcile path already filed in `GOOD-TO-HAVES.md`) and needs the same owner-gate discipline as that entry.

**Sketched resolution:** Milestone-close should either run `scripts/refresh-tokenworld-mirror.sh` as a documented pre-step of `pre-release-real-backend`, or the litmus should self-reconcile per the fix-sketch already in `GTH-V15-09` (fetch backend-current through the reposix bus remote before the marker push). This entry cross-refs `GTH-V15-09` and documents the concrete cadence-wiring gap plus the specific 2026-07-15 false-negative it caused, so the next reader does not re-diagnose from scratch.

**STATUS:** OPEN

## 2026-07-14 20:43 | discovered-by: L0 rotation #22 (t4 real-backend re-run — preflight vs runner env-loading gap) | severity: HIGH

**What:** `quality/runners/run.py` does not source `./.env`, while `scripts/preflight-real-backends.sh` DOES — causing a false-green-preflight / silent-skip gap: the real-backend cadence skips all rows when run without `.env` pre-sourced into the shell, but preflight independently reports the backends reachable, so the two together give a false impression of full coverage. CONFIRMED this rotation: sourcing `.env` in the same invocation (`set -a; . ./.env; set +a`) fixed it — all 6 real-backend rows executed for real once the env vars were present in `run.py`'s process.

**Why out-of-scope for the discovering session:** The immediate re-run was unblocked by manually sourcing `.env` before invoking `run.py`; fixing `run.py` itself (or its documented invocation) is a code/docs change to the shared quality-runner infra, not an eager patch inside a single cadence re-run, and the "fix it twice" doctrine (root CLAUDE.md meta-rule) requires updating the doc references in the same change.

**Sketched resolution:** Make `run.py` source `.env` itself (e.g. load it via Python's own env-loading at startup), OR bake `set -a; . ./.env; set +a` into the documented invocation everywhere `pre-release-real-backend` is referenced. Fix-it-twice: update the `pre-release-real-backend` doc references in `.planning/CLAUDE.md` and `docs/reference/testing-targets.md` to show the corrected invocation (or note that `run.py` now self-sources), so the next agent does not silently skip-and-declare-green again.

**STATUS:** OPEN

## 2026-07-14 20:44 | discovered-by: L0 rotation #22 (t4 real-backend re-run — `--persist` write review) | severity: HIGH

**What:** `--persist` silently rewrites genuinely-GREEN catalog rows to a worse status on a skip/false-negative. This rotation's run downgraded `vision-litmus` PASS→FAIL in the persist write — caught and `git restore`d before commit only because the diff was reviewed before staging. The prior rotation downgraded 2 P0 rows the same way on an env-skip (see the preceding env-loading-gap and mirror-staleness entries above — either false-negative feeds directly into this persist behavior, compounding the risk).

**Why out-of-scope for the discovering session:** Changing `--persist`'s write semantics (adding a confirm gate) is a change to the shared quality-runner framework's core write path — needs its own review given how many cadences depend on `--persist`, not an eager patch mid a real-backend re-run where the immediate risk was mitigated by manual `git restore` + review-before-commit.

**Sketched resolution:** Gate skip/fail-driven `status` rewrites behind an opt-in flag (e.g. `--allow-downgrade`), or refuse by default to rewrite a committed GREEN `status` to a worse value without an explicit confirm flag — surfacing a loud warning instead ("row X was PASS, new result is FAIL/SKIP — pass --allow-downgrade to persist this change") so a false-negative run cannot silently corrupt catalog state. Pairs with the preceding env-loading-gap and mirror-staleness entries as root causes that feed spurious downgrades into this behavior.

**STATUS:** OPEN
