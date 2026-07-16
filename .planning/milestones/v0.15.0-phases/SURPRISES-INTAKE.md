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

**2026-07-16 scope-broadening update:** The original entry's scope (two oversized `GOOD-TO-HAVES.md` files) is incomplete. **`SURPRISES-INTAKE.md` itself is now also over the 20,000-char ceiling — 47,291 bytes** — and is NOT covered by this entry's original scope decision. Filing SURPRISES entries into `GOOD-TO-HAVES.md` as a workaround is contradicted by this new discovery. Additionally, the v0.15.0-phases `GOOD-TO-HAVES.md` has grown substantially since this entry was filed (from **23,584 bytes** to **46,795 bytes**), compounding the exposure to the 2026-08-08 waiver lapse. The drain-phase resolution must broaden scope to include `SURPRISES-INTAKE.md` proactively — split it into a live ledger + a `SURPRISES-INTAKE-REFERENCE.md` (or `-history.md`) alongside the two `GOOD-TO-HAVES.md` splits, all before the waiver expires, or pushes will start blocking with no single intake row naming the file.

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

## 2026-07-15 06:30 | discovered-by: L0 rotation #26 intake-filing leaf (carried forward across workhorse #24→#25→#26 handovers, 2 rotations un-filed) | severity: MEDIUM

**What:** The commit-message argument to `gsd-sdk query commit` is POSITIONAL, not a `--message` flag. Passing `--message "..."` silently commits a garbage/empty message instead of erroring — a real footgun for any agent copying the pattern.

**Why out-of-scope for the discovering session:** Discovered incidentally during unrelated work; correcting the documented example other agents copy from touches a user-global skill (`coordinator-dispatch`, outside this repo) and/or `.planning/ORCHESTRATION.md`, which needs a deliberate review pass rather than an eager patch mid another charter. Carried un-filed across two prior rotation handovers before this leaf captured it.

**Sketched resolution:** Fix-twice obligation: (i) this intake row captures the footgun; (ii) correct the documented example other agents copy from — the `coordinator-dispatch` skill and/or `.planning/ORCHESTRATION.md` commit example — to the positional form `gsd-sdk query commit "<msg>" --files <path>`, so no future example teaches the `--message` flag form. Also consider whether `gsd-sdk query commit` itself should hard-error on an unrecognized `--message` flag rather than silently committing garbage, closing the footgun at the source rather than only in docs.

**STATUS:** OPEN

## 2026-07-15 06:35 | discovered-by: manager amendment 4 to L0 rotation #26 (measured on this rotation's push; corroborates workhorse #25's 101s WARN) | severity: LOW-MEDIUM

**What:** Pre-push hook wall-clock measured **~1:31.68 (91.7s)** this rotation and **~101s** on workhorse #25's final push, vs the **~55–60s** budget documented in `quality/CLAUDE.md` § Cadences. Likely driver: `code/shell-coverage` (kcov shell coverage) measured **56.2s this run** vs the **~29s** figure documented in § Cadences — roughly 2×. Pre-push cost is a fixed whole-repo cost (NOT diff-size-scaled), so this is a genuine creep, not a big-diff artifact.

**Why out-of-scope for the discovering session:** Surfaced from a timing measurement taken during an intake-filing leaf's own push, not a planned perf-investigation phase; diagnosing whether kcov crept (corpus growth, toolchain/version change, VM contention) and deciding baseline-vs-regression is its own bounded investigation, not an eager patch mid intake-filing. Pre-push is WARN (not FAIL) so no gate blocked — no urgency to fix inline.

**Sketched resolution:** Fix-twice obligation: (i) investigate whether kcov shell-coverage crept — more `.sh` files under coverage (corpus growth), a kcov/toolchain version change, or transient VM contention; re-measure `code/shell-coverage` in isolation on a quiet VM to separate contention from real cost; (ii) if the higher figure is a legitimate new baseline, update the § Cadences documented number (~29s → measured, and the ≈55s pre-push aggregate → measured) to match reality; if it's a regression, find and fix the cause (e.g. trim the covered corpus, cache kcov output, or parallelize where the cargo-mutex allows). Corroborated across two rotations (#25 ~101s, #26 ~91.7s), so it is a stable creep rather than a one-off flake.

**STATUS:** OPEN

## 2026-07-15 21:45 | discovered-by: P115-T2 (BENCH-01 live latency re-measurement) | severity: MEDIUM

**What:** latency-bench PATCH probe sends unsupported `expected_version` → times an error path (sim patch figure invalid). `bash quality/gates/perf/latency-bench.sh` emits, 3x per run, `ERROR reposix_sim::error: json error error=unknown field 'expected_version', expected one of 'title','body','status','assignee','labels'`; reproduced across 3 consecutive local runs 2026-07-15 and present in CI run 29452237641. The `patch=Nms` row therefore times a 400 rejection, not a successful patch. Mechanism: the bench sim PATCH probe constructs a no-op PATCH body containing `expected_version`; the reposix-sim issue-update handler schema only accepts title/body/status/assignee/labels → 400.

**Why out-of-scope for the discovering session:** Deciding the intended contract first (accept `expected_version` for optimistic concurrency, or drop it from the bench body) touches reposix-sim request validation and/or the bench probe — a >1h scoped change, not an eager fix inside a re-measurement task.

**Sketched resolution:** Either (a) add `expected_version` to the sim's accepted update fields if optimistic-concurrency is intended, or (b) drop `expected_version` from the bench PATCH body. Decide the intended contract first, then fix the losing side (sim schema or bench probe) to match.

**STATUS:** OPEN

## 2026-07-15 22:00 | discovered-by: P115 roadmap gsd-quick noticing (OD-3) | severity: LOW

**What:** `docs/development/roadmap.md` is a STALE internal snapshot that lies about the
active milestone. Its "Active milestone" section (L20-22) still reads "**v0.11.0 Polish &
Reproducibility** — PLANNING (Phases 50–55 scaffolded)", and its shipped-milestones table
(L18) stops at v0.10.0 — reality is v0.15.0 (Floor). The file is `not_in_nav` (not linked
from mkdocs nav), so it does not surface to readers via the docs site, but it remains a
committed artifact an agent or contributor could stumble on and trust. Now that a public
`docs/roadmap.md` exists as the canonical current-state surface, this internal snapshot's
staleness is more conspicuous — it duplicates a job the public doc already does, but worse.

**Why out-of-scope for the discovering session:** Surfaced incidentally by a P115 roadmap
gsd-quick lane (OD-3 noticing obligation), not a planned docs-freshness pass; deciding
whether to refresh the snapshot to current state or replace it with a redirect/pointer to
`docs/roadmap.md` is a small but distinct doc-hygiene decision, not an eager one-line patch
inside the P115 benchmark-remeasurement charter.

**Sketched resolution:** Either (a) refresh `docs/development/roadmap.md`'s "Active
milestone" section + shipped table to the current v0.15.0 (Floor) state, or (b) replace its
content with a short redirect/pointer to the now-canonical public `docs/roadmap.md`, so the
internal file cannot drift out of sync again. Prefer (b) if the internal file's only purpose
was to duplicate what `docs/roadmap.md` now covers — one source of truth beats two.

**STATUS:** OPEN

## 2026-07-15 17:18 | discovered-by: L0 rotation #36 (read-only pre-push-spike diagnosis, charter item 2) | severity: LOW

> **Root-cause deep-dive for the existing `2026-07-15 06:35` pre-push-timing item above — enriches, does NOT duplicate.** The drain phase should resolve both together.

**What:** Root-caused the pre-push over-budget WARN (rotation #35 saw 109s; #25/#26 saw ~101s/~91.7s; budget doc says ≈55–60s). It is **mostly environment variance layered on a modest kcov-corpus-growth creep** — NOT a new gate and NOT a stable new floor. Evidence: (1) a fresh `python3 quality/runners/run.py --cadence pre-push` on the identical commit state measured **64s total**, proving large run-to-run variance on unchanged code; (2) the dominant row `code/shell-coverage` (kcov) genuinely grew from the documented **29s** (measured 2026-07-12 08:21, commit `fc8264d`) to **~37s now**, because two MORE kcov-traced shell harnesses landed hours *after* the budget doc was written that same day — `fbb7782` (08:42, `60-code-ci.sh`, 7 stub-binary invocations) and `fe8febb` (18:52, `real-backend-env-gate.sh`, 2 scrubbed-env invocations) — neither reflected in the "≈55s" budget text. Timed breakdown of the 64s run: `code/shell-coverage` 37.16s · `agent-ux/rebase-recovery-reconciles` 9.14s · `hook-throttle` 2.02s · `mkdocs-strict` 1.98s · `badges-resolve` 1.96s · `no-orphan-docs` 1.89s · `fleet-safety-leaf-isolation-enforce` 1.32s · ~45 other rows sub-1s (most 0.03s). So the "≈55–60s" budget is **stale** (never re-baselined after the two post-08:21 harness additions), AND #35's specific 109s is the high-variance tail of a distribution now centered nearer ~65–75s. (Adjacent, noticed not filed separately: `60-code-ci.sh` rebuilds a stub `gh` binary + fresh PATH in a `mktemp` dir on each of its 7 invocations — more IO-syscall-heavy per call than peer harnesses, a plausible amplifier of VM I/O-contention variance. `docs-alignment/link-resolution` double-counts `docs/index.md` — cosmetic, ALREADY noted in the handover, do NOT re-file.)

**Why out-of-scope for the discovering session:** Rotation #36's charter item 2 was explicitly a **read-only investigation — "file findings, change nothing."** Re-baselining the budget doc + raising the WARN threshold (both mutating edits) were out of charter, so filed rather than applied.

**Sketched resolution:** Re-baseline `quality/CLAUDE.md` § Cadences pre-push budget from ≈55–60s to **~75s** (median of several post-corpus-growth runs) and raise the WARN threshold from 90s to **~100s**, so normal kcov ptrace/IO jitter stops flagging as noise. Optionally, reduce `60-code-ci.sh`'s per-invocation stub-`gh` rebuild churn (build once, reuse) to shave the variance amplifier. Full evidence: this rotation's read-only diagnosis (no code touched); base commit `1b20c15`.

**STATUS:** OPEN

## 2026-07-16 05:00 | discovered-by: P115 Task-4 capture executor (L0 #38) | severity: BLOCKER

**What:** The P115 Task-4 live-MCP-vs-reposix token benchmark (6 capture sessions = 1 backend × median-of-3 × 2 arms) **cannot run as specified**. Three independent findings, each verified this rotation: **(1)** the ratified `atlassian-rovo` MCP server (`https://mcp.atlassian.com/v1/mcp`, `atlassian-mcp-server` v1.0.0) advertises **only 3 Teamwork Graph tools** (`getTeamworkGraphContext`, `getTeamworkGraphObject`, `addTeamworkGraphContext`) — there is NO `editJiraIssue`/`createJiraIssue`/`updateJiraIssue`/JQL-search tool, so the benchmark task's "edit 1 issue" step has no tool (`addTeamworkGraphContext` mutates the relationship graph only, not issue fields). The synthetic fixture assumed a full-CRUD server (`sooperset/mcp-atlassian`), a DIFFERENT server. **(2)** A real `tools/call` (`getTeamworkGraphContext` on `JiraSpace KAN`) with the ratified Bearer API token is **permission-denied**: `"You don't have permission to connect via API token. Please ask your organization admin for access."` — tools LOAD (handshake 200) but do NOT function with this credential. This closes the explicit open caveat (1) in `115-ROVO-AUTH-CHECK.md` (tool-level authz was never verified) with a negative result. **(3)** Jira project **KAN has 0 issues** — `reposix init jira::KAN` synced to an empty tree (`sync(jira:KAN): 0 issues`, `git ls-tree HEAD` empty), so neither arm can "read 3 issues"; this blocks the reposix arm independent of the MCP findings. Only 1 of 50 sessions was spent (the smoke test); no numbers were fabricated.

**Why out-of-scope for the discovering session:** Every unblock path is a charter/owner decision that also changes what the benchmark measures and/or spends the capped 50-session budget: (A) grant the token graph access + redefine the mcp-arm task to a read+link workflow (issue-field edits are impossible on this server); (B) swap the mcp-arm to `sooperset/mcp-atlassian` (full CRUD, needs setup + egress-allowlist review); (C) seed KAN with ≥3 issues (unblocks only the reposix arm). The executor delivered the honest, no-decision-needed artifacts (real tool catalog, grounded server-choice note, smoke-session capture, ledger row) and escalated rather than unilaterally redefine the ratified task, swap servers, or seed the backend.

**Sketched resolution:** Owner picks A / B / C (or a combination). Reposix arm needs C regardless. MCP arm needs A or B. If A, update `115-MCP-SERVER-CHOICE.md` and the benchmark task definition to the read+link workflow and note the arms are no longer read+field-edit-comparable. If B, register `sooperset/mcp-atlassian`, extend `REPOSIX_ALLOWED_ORIGINS` review, re-run the smoke test, then the 6 captures. Evidence: `benchmarks/fixtures/mcp_jira_catalog.json` (real 3-tool surface), `benchmarks/captures/mcp-kan-smoke.json`, `.planning/phases/115-live-mcp-benchmark-re-measurement/115-MCP-SERVER-CHOICE.md` (§ Blockers + § Recommendation).

**STATUS:** **RESOLVED (2026-07-16, L0 #39)** — path (D, not A/B/C): **[SELF] pivot to the GitHub backend** instead of Jira. All 6 captures landed on `github-probe` + `reubenjohn/reposix` (ledger rows 2–7, `running_total` 7/50); the Jira/`atlassian-rovo` findings above are retained as the evidence trail. Jira remains addable later if org-admin API-token + a CRUD Jira MCP are provisioned. See `115-MCP-SERVER-CHOICE.md` § LIVE-CAPTURE CHOICE.

## 2026-07-16 06:05 | discovered-by: P115 Task-4 capture executor (L0 #39) | severity: MEDIUM

**What:** During the T4 GitHub capture, the **reposix-arm `git push` was rejected**: the `git-remote-reposix` helper returns `patch issue 60: not supported: update_record — reposix-github is read-only in this cut`. This is **intentional and documented** (`crates/reposix-github/src/lib.rs` `create_record`/`update_record`/`delete_or_close` all return not-supported; `crates/reposix-cli/src/doctor.rs:1467` states "github: read=yes, create/update/delete=— (read-only in this cut)"). So the reposix arm can read+edit+local-commit GitHub issues but **cannot write them back**. Two consequences: (1) the T4 SESSION-HANDOVER §2 recipe's assertion "the push writes back to GitHub via the helper" is **wrong for this cut** — only the mcp-arm `issue_write` persisted; the reposix-arm push does not. The token-economy comparison is UNAFFECTED (it measures agent context size, not write capability, and the failed-push tokens are negligible). (2) Any user-facing "push writes back" claim that implies GitHub is covered would be inaccurate for this cut (Confluence TokenWorld is the write-capable real backend; sim always writes).

**Why surfaced not fixed:** enabling GitHub issue writes is a real feature (REST PATCH mapping + conflict-detection + audit rows + egress/taint review), far more than <1h; and it's a deliberate cut, not a bug. The capture proceeded honestly (reposix arm = read+edit+commit+push-attempt with the documented read-only error; transcript at `benchmarks/fixtures/reposix_session.txt` shows it verbatim).

**Sketched resolution (for L0 to route):** either (a) implement `reposix-github` write-back (a scoped feature lane — pairs with the P122 `reposix-remote`/`init` hardening or a dedicated backend-write phase), or (b) if writes stay cut, audit docs/README/how-it-works so no "push writes back" claim reads as covering GitHub without the read-only caveat. Evidence: `crates/reposix-github/src/lib.rs:654/666/677`, `doctor.rs:1467`, `benchmarks/fixtures/reposix_session.txt`.

**STATUS:** OPEN

## 2026-07-16 07:47 | discovered-by: P115-T5 close-out executor (SURPRISES-INTAKE filing pass) | severity: MEDIUM

**What:** During T5's push, the pre-push gate summary printed "60 PASS, 1 FAIL" yet the hook exited 0. Traced the mechanism: `quality/runners/run.py::compute_exit_code` (L403-409) exits 0 "iff every P0+P1 row is PASS or WAIVED" — a FAIL row with `blast_radius: P2` (or lower) is counted in the printed summary but never flips the exit code. Separately, `run.py` L324-327/355-356 confirm a verifier timeout maps unconditionally to `row["status"] = "FAIL"` (there is no distinct TIMEOUT/WARN status) — so a timeout-class failure is visually indistinguishable from a real assertion failure in the summary line. Per the T5 executor's contemporaneous account, the specific FAILing row was an identification-class check hitting its ~5-minute timeout on the pre-pr-adjacent suite, pre-existing and unrelated to T5's changes (the `docs-repro benchmark-claim` rows T5 actually touched did NOT regress). Could not independently reproduce which exact row FAILed within this filing pass's ~10-minute budget — the exit-code MECHANISM above is verified fact (file:line, read directly from `run.py`); the specific-row attribution is hearsay from the T5 executor, recorded as such rather than re-stated as independently confirmed.

**Why out-of-scope for the discovering session:** Deciding whether a P2-blast-radius timeout-class FAIL is an intentional by-design WARN-equivalent (vs an oversight that should either block the push or get its underlying timeout fixed) requires reading the P0/P1/P2 blast-radius design rationale across the framework docs and possibly the prior RBF decisions that shaped `compute_exit_code` — a scoped semantics investigation, not a same-session mechanical fix. This close-out pass was scoped to filing + one small doc line, not framework-semantics changes.

**Sketched resolution:** (1) Decide by design: is a P2-blast-radius FAIL (timeout or otherwise) meant to be silently non-blocking? If yes, make the printed summary and/or per-row label distinguish it (e.g. `FAIL(non-blocking P2)`) or introduce a real `TIMEOUT` status distinct from `FAIL`, so a human scanning "60 PASS, 1 FAIL" does not read it as a live regression demanding triage. If no (P2 FAILs are meant to eventually get fixed, just not gate the push), the exit-0 behavior is already correct — but the summary line should still not present as ambiguous between "ignorable" and "needs attention." (2) Separately, consider whether verifier timeouts specifically deserve their own status distinct from an assertion-failure FAIL, since "timed out" and "assertions failed" are different failure classes with different triage paths and root causes.

**STATUS:** OPEN

## 2026-07-16 12:00 | discovered-by: P115-T6 Wave 2 item 2 executor (115-UNWAIVE-PATH.md inventory pass) | severity: MEDIUM

> **Corroborates, does NOT duplicate, the existing `2026-07-15 06:35` pre-push-timing
> item and its `2026-07-15 17:18` root-cause deep-dive above.** Third data point in the
> same creep; the drain phase should resolve all three together.

**What:** Pre-push hook wall-clock measured **~141s** on the push landing `d7da383`
(T6 Wave 1 item 3 agent-side retire+bind), following **~128s** on the immediately
preceding push. Both are well above the **~55–60s** budget documented in root
`CLAUDE.md` § GSD workflow ("pre-push ≈55s, dominated by kcov shell-coverage +
full-workspace clippy/mkdocs, not by what changed") and above even the **~75s**
re-baseline the `2026-07-15 17:18` entry already proposed. Pre-push cost is a fixed
whole-repo cost (NOT diff-size-scaled per the same CLAUDE.md line), so two consecutive
~128-141s runs read as the creep continuing past the point the prior entry's
re-baseline anticipated, not a one-off variance tail.

**Why out-of-scope for the discovering session:** This session's charter was a bounded
Wave-2 item-2 lane (write `115-UNWAIVE-PATH.md`, file this intake row, commit+push) —
profiling the pre-push gate stages or re-baselining `quality/CLAUDE.md` § Cadences is a
distinct investigation already owned by the two entries above; re-doing that work here
would duplicate scope rather than close it.

**Sketched resolution:** Same sketch as the two entries above, now with a third
corroborating measurement: (1) profile the pre-push gate stage-by-stage (the
`2026-07-15 17:18` entry's timed breakdown — `code/shell-coverage` 37.16s dominant — is
the starting point, re-run it against current `main` to see if the dominant row grew
further); (2) decide whether to cache kcov/clippy output across runs (biggest lever —
kcov + full-workspace clippy are both re-executed from scratch every push regardless of
diff size) or scope `mkdocs-strict` to only changed docs pages; (3) once a stable new
baseline is established, update root `CLAUDE.md` § GSD workflow's "≈55s" figure to match
reality (currently three data points — 91.7s, 128s, 141s — all exceed even the ~75s
re-baseline already proposed, suggesting the creep is ongoing past that estimate, not
just newly-discovered).

**STATUS:** OPEN

## 2026-07-16 07:50 | discovered-by: P115-T5 close-out executor (SURPRISES-INTAKE filing pass, relaying a T5 executor mid-task noticing) | severity: MEDIUM

**What:** Orchestration doctrine (`.planning/ORCHESTRATION.md` + the coordinator-dispatch skill) assumes a coordinator can `SendMessage` a running lane to deliver a mid-task scope correction, but the phase-coordinator harness exposes only `Agent`/`Bash`/`Read` as tools — no `SendMessage`. During T5, an L0 mid-task scope correction could NOT be delivered to the running executor; it was saved only because the executor's charter carried a "plan wins" clause that happened to keep it on the correct path anyway. This is a silent single-point-of-failure: if a future mid-flight correction is safety-critical (not just a redirection nicety) and the "plan wins" fallback doesn't happen to cover it, there is no delivery mechanism at all — a coordinator can only wait for the lane to finish or return, never interrupt it.

**Why out-of-scope for the discovering session:** Exposing `SendMessage` in the phase-coordinator agent definition's tool list (or formally documenting the gap + mitigation pattern in `ORCHESTRATION.md` §11) is an agent-def / orchestration-doctrine change — outside a scoped close-out pass whose charter explicitly forbids editing `ORCHESTRATION.md` or any agent def directly ("file only").

**Sketched resolution:** Either (a) add `SendMessage` to the phase-coordinator agent definition's tool list so a coordinator can interrupt/redirect a running lane mid-task, or (b) if `SendMessage` is intentionally withheld from coordinators (e.g. a deliberate isolation boundary), document the gap explicitly in `.planning/ORCHESTRATION.md` §11 alongside the "plan wins" clause as the documented mitigation pattern, so a future coordinator does not assume SendMessage works and silently rely on an undocumented fallback. Owner should decide (a) vs (b) — this is a capability/tooling decision, not a mechanical fix.

**Cross-reference (2026-07-16, filed by the P115 owner-directive lane):** per
`SESSION-HANDOVER.md` §6 finding 1, the T6 coordinator forked a fable-tier leaf to
deliver a mid-task scope correction (`SendMessage` unavailable in the subagent harness),
creating a momentary second tree-writer and a fable-at-leaf tiering violation; it executed
cleanly, no corruption. Judged the SAME underlying gap as this entry — cross-referenced
here rather than filed as a new row.

**STATUS:** OPEN

## 2026-07-16 | discovered-by: P115 owner-directive lane doc sweep | severity: MEDIUM

**What:** `docs/social/twitter.md:18` and `docs/social/linkedin.md:21` (plus
`docs/social/assets/_build_benchmark.py`, `_build_combined.py`, `benchmark.svg`) still
present the OLD 89.1% token-reduction figure as current, with no retirement language —
verified live: both files read "89.1% fewer tokens" today. Catalog rows
`docs/social/twitter/token-reduction-92pct` and `docs/social/linkedin/token-reduction-92pct`
sit at `STALE_TEST_DRIFT` with `next_action: BIND_GREEN`, actively tracking the old
number (confirmed by direct catalog read).

**Why out-of-scope for the discovering session:** The owner-directive lane's charter was
narrowly scoped to removing retirement-HISTORY narrative from user-facing docs (the
89.1%/85.5% retirement-story sections, not stale numbers still presented as current); the
social drafts are a distinct staleness class — an old live number, not a narrative — and
deciding whether to refresh/freeze/retire them is an owner call, not a mechanical doc edit
inside this lane.

**Sketched resolution:** Owner decision needed: (a) refresh the drafts + assets to the
current live ~94.3%/~75% four-axis figures, (b) freeze them as intentionally-dated
snapshots (add a "as of <date>" caveat so they read honestly), or (c) retire the two
catalog rows if the social posts themselves are considered historical artifacts not meant
to track current numbers. Whichever is chosen, update the two catalog rows to match
(re-bind if refreshed, retire if frozen/historical).

**STATUS:** OPEN

## 2026-07-16 | discovered-by: P115 owner-directive Wave-1 executor | severity: MEDIUM

**What:** 34 catalog rows (the `docs/benchmarks/latency.md` L38-44 block,
`bench-latency-cron.yml`, `docs/index.md` hero rows) carry stale `test_body_hashes` for
`quality/gates/perf/latency-bench.sh` even though the script content is identical to
HEAD — pre-existing `STALE_TEST_DRIFT` bit-rot that `walk.sh` does NOT block on (verified:
its documented blocking list is `STALE_DOCS_DRIFT` / `MISSING_TEST` / `STALE_TEST_GONE` /
`TEST_MISALIGNED` / `RETIRE_PROPOSED` — `STALE_TEST_DRIFT` is silent, `walk.sh:71-72`).

**Why out-of-scope for the discovering session:** Wave 1's charter was the doc-narrative
strip (removing retirement-history prose from four docs); re-binding 34 unrelated
`test_body_hashes` on a script that hasn't actually changed is a distinct catalog-hygiene
sweep, not a doc-narrative edit, and touching 34 rows is far larger than the Wave 1 diff.

**Sketched resolution:** Dedicated re-bind pass on the 34 affected rows (re-hash
`quality/gates/perf/latency-bench.sh` and refresh each row's `test_body_hashes` to match,
since the content itself is unchanged — this is a hash-refresh, not a content fix).
Separately, consider surfacing `STALE_TEST_DRIFT` at pre-push or in post-push reporting
(even non-blocking) so this class of drift can't decay silently across future script
edits.

**STATUS:** OPEN

## 2026-07-16 | discovered-by: P115 phase-close cold-reader pass (L0 #44) | severity: MEDIUM

**What:** `README.md:109-110` "Project status" is 3 months / 4+ versions stale on the
repo's most visible surface: it claims v0.9.0 shipped + v0.10.0 "landing 2026-04-25" as
latest, while the repo is at v0.14.0 (Cargo.toml + git tags) with v0.15.0 executing
(2026-07-16). Same lying-hero-claim class the P115 destaling lanes just purged from the
benchmark numbers — but in a narrative section no number-focused gate watches.

**Why out-of-scope for the discovering session:** the cold-reader lane was report-only;
a status-section rewrite is a framing decision (what to promise about v0.15.0, whether
README should carry a version ticker at all) — planner/owner input, not a figure swap,
and the section's shape invites recurring staleness (would rot again by v0.16.0).

**Sketched resolution:** (a) version-agnostic status line linking GitHub releases /
CHANGELOG as the single version-truth source, or (b) keep a version line but bind a
doc-alignment row asserting README version == latest git tag so drift blocks at
pre-push. (b) is the fix-twice option — P117 (doc-truth purge) is the natural home.

**STATUS:** OPEN

## 2026-07-16 | discovered-by: P115 cold-reader dispatch (L0 #44) | severity: MEDIUM (tooling, silent-failure class)

**What:** the user-global `doc-clarity-review` skill
(`/home/reuben/.claude/skills/doc-clarity-review/`) is silently broken: its documented
`claude -p "<prompt>" <file1> <file2>` invocation does NOT attach trailing file paths in
the current CLI — the subprocess receives no files and "reviews" ambient cwd context
(`~/.claude/CLAUDE.md` + project CLAUDE.md) instead, returning a plausible-looking review
of the WRONG target. Confirmed via diagnostic call (`claude -p "What files were you
given?" f1 f2` → "no files"). Dangerous class: a silent wrong-target review reads as a
pass on the right target. (The P115 pass itself completed via the leaf's manual
fallback: isolated read of only the two target files.)

**Why out-of-scope for the discovering session:** the skill lives in the user's global
`~/.claude/skills/`, not this repo; fixing it is cross-project tooling work.

**Sketched resolution:** inline file CONTENT into the `-p` prompt (cat the copied `/tmp`
files / heredoc) instead of passing paths as args; add a loud self-check ("state the
first heading of each file you were given") so a no-files run fails visibly. Distinct
from the unreproduced "intermittent Read/Edit harness failures" noticing in #43's
handover §5.4 — that stays unfiled pending a live repro.

**STATUS:** OPEN

## 2026-07-16 | discovered-by: quick 260716-fmt (GTH-V15-35) | severity: MEDIUM

**What:** `test_main_offline_regenerates_doc_from_captures` in
`quality/gates/perf/test_bench_token_economy.py` (L212-244) asserts idempotency ONLY
against a synthetic `tmp_path` fixture — it monkeypatches `bench.RESULTS` /
`bench.CAPTURES` / `bench.BENCH_DIR` to a temp dir and checks that a second
`--offline` run reproduces the same bytes as the first — and NEVER diffs the
regenerated doc against the real committed `docs/benchmarks/token-economy.md`. That is
the exact gap that let the 260716-f6o generator regression (the retired-narrative
section re-added by the template) reach a P115 phase-close gate run undetected: the
test's own idempotency check passed because it only compares the synthetic doc to
itself, not to the committed source of truth.

**Why out-of-scope for the discovering session:** 260716-fmt's charter is a docs/index.md
install-IA fix (block relocation + bootstrap prose + L93 destale); adding a new
byte-compare regression test to a different quality gate (`perf/test_bench_token_economy.py`)
is an unrelated test-authoring change outside that scope, surfaced only because the
260716-f6o regression this gap enabled was fixed in the immediately preceding commit.

**Sketched resolution:** Add a regression test that runs the offline regenerator against
the REAL committed captures (not the synthetic `tmp_path` fixture) and byte-compares
(sha256) the output against the committed `docs/benchmarks/token-economy.md` — so any
future generator/doc divergence fails a test instead of silently dirtying the working
tree at a gate run, the way the 260716-f6o regression did.

**STATUS:** OPEN
