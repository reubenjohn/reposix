# v0.15.0 Good-to-haves / carried-forward hardening

> **Purpose:** concrete landing spots for the DEFERRED / DEFERRED-TO-v0.15.0 entries the
> v0.14.0 surprises-intake promised would land here. Owner ask (2026-07-12): *labels alone
> don't count* — each carried-forward entry needs a real row with **severity + a concrete
> fix-sketch**, verbatim-faithful to the intake. Source of truth for the originals (archived,
> not deleted): `.planning/milestones/v0.14.0-phases/surprises-intake/part-01.md` + `part-02.md`.
> Landed by gsd-quick `260712-oke`. OP-8 drains this file in v0.15.0's last two phases.
>
> **Roadmap-gap reconciliation:** the intake cited "v0.15.0 framework-/helper-hardening phases"
> that the v0.15.0 `ROADMAP.md` did not list (it had only the two UX `Phase TBD` stubs). The
> two **HIGH** entries below (GTH-V15-04 modern-git verification, GTH-V15-06 subprocess-bypass
> self-safety refusal) now have `### Phase (candidate)` stubs under `ROADMAP.md` §
> "Hardening candidates"; the five MEDIUM entries live here as drain rows.

## Carried-forward from the v0.14.0 surprises-intake (7 entries)

### GTH-V15-01 — concurrent `--persist` runners race-corrupt catalog JSON
- **Source:** part-01 (discovered-by P104, 2026-07-12 07:13) · **Severity: MEDIUM** · STATUS in intake: DEFERRED-TO-v0.15.0 (framework-hardening phase).
- **What:** Two concurrent `reposix-quality` runners (or herdr `--persist` modes) were observed minting the shared catalog file (`quality/catalogs/agent-ux.json`) mid-verification during P104 grading (PID 351077 held the lock while the verifier ran). Two writers on one catalog file is a live race hazard — interleaved writes can corrupt the JSON or lose rows. Latent, not active: catalog writes are currently serialized by orchestration discipline (one persist lane at a time), and the P104 grading where it was observed did not corrupt the JSON.
- **Fix-sketch:** advisory `flock` around the catalog-JSON persist in `quality/runners/run.py`, OR serialize all catalog persist operations through a single lane with a lock file, so two concurrent `--persist` writers cannot interleave. Alternative: single-persist-lane discipline where only the primary orchestration lane writes catalogs and herdr on-demand runners read-but-do-not-persist. Belongs to the same v0.15.0 framework-hardening phase as GTH-V15-03.

### GTH-V15-02 — shell-coverage 12.54% < 13% floor (110 scripts @0%)  *(cross-reference, NOT duplicated here)*
- **Source:** part-01 (discovered-by v0.14.0 health-triage lane, 2026-07-12 07:35) · **Severity: MEDIUM** · STATUS in intake: DEFERRED to the coverage-climb work.
- **What:** `code/shell-coverage` is a live FAIL on `main` — aggregate 12.54% (564/4497 lines) below the committed 13.00% floor in `quality/shell-coverage-floor.txt`. Root cause is corpus growth (149 scripts, 110 at 0%: mostly `quality/gates/agent-ux/*` + `.claude/hooks/*`), not a coverage drop. `blast_radius: P2`, non-blocking on pre-push (`compute_exit_code` exits 1 only for a non-PASS P0/P1); the separate CI `shell-coverage` job DOES hard-fail on kcov and can surface via the P0 `code/ci-green-on-main` post-push probe.
- **Landing = CROSS-REFERENCE ONLY.** Its existing home is phases **`999.5 docs-crates-md-zero-coverage`** / **`999.6 docs-alignment-coverage-climb`** — recorded here so the trail is complete; **NOT** re-filed onto v0.15.0. Fix path (per intake, do not silently patch the floor): (a) author `quality/gates/code/shell-coverage-tests/` cases for the highest-line-count 0%-covered scripts until aggregate clears 13% (**preferred** — raise the floor over time, never force-pass by lowering it), OR (b) if some scripts are ruled structurally untestable outside real backends, lower the floor to the measured 12.54% with a documented rationale in `quality/CLAUDE.md` + a GOOD-TO-HAVES tracking item for the deferred scripts.

### GTH-V15-03 — no gate checks a row's `verifier.script` exists + is executable
- **Source:** part-01 (discovered-by P104, 2026-07-12 07:13) · **Severity: MEDIUM** · STATUS in intake: DEFERRED-TO-v0.15.0 (framework-hardening phase).
- **What:** A catalog row can be minted `status: PASS` with a `verifier.script` path that does not exist on disk (P104 caught one instance only in manual review). The pre-commit hook validates JSON but does NOT structurally verify that a row's declared `verifier.script` path exists or is executable — a window for a false-positive contract breach (a PASS row backed by a missing/non-executable verifier).
- **Fix-sketch:** add a structure-dimension gate `quality/gates/structure/verifier-script-exists.sh` that scans all catalog rows at load time and asserts, for each row with a non-null `verifier.script`, the file exists on disk AND is executable (`chmod +x`); fail at pre-commit/pre-push if any row references a missing verifier, preventing unbacked PASS rows from landing. Complement to GOOD-TO-HAVES-01 (bind-verb extension). Pairs with GTH-V15-01 in the same v0.15.0 framework-hardening phase.

### GTH-V15-04 — RBF-LR-03 fix unverified on git ≥ 2.34 stateless-connect  *(HIGH — also a ROADMAP stub)*
- **Source:** part-02 (discovered-by P105 Lane 2, 2026-07-12 08:35) · **Severity: HIGH** (verification-only residual; the parent bug is RESOLVED-in-P105, commit `bd5b9cb`, gate GREEN on git 2.25.1) · STATUS in intake: RESIDUAL — DEFERRED-TO-v0.15.0 (verification-only, NOT a live bug).
- **What:** The RBF-LR-03 fetch-ref-lock fix (`bd5b9cb`, disjoint import namespace `refs/reposix-import/*`) is confirmed real in committed source and the gate `agent-ux/rebase-recovery-reconciles` grades exit 0 / 13-of-13 asserts — but ONLY on git **2.25.1** via the `import` path. PLAN §5 remains open: whether `stateless-connect` on git **≥ 2.34** exhibits the same or a different fetch-ref-lock behavior is NOT yet exercised on a modern-git CI runner. This is a coverage extension, not an unfixed push-correctness defect.
- **Fix-sketch:** run `quality/gates/agent-ux/rebase-recovery-reconciles.sh` on a modern-git (≥ 2.34) CI runner and resolve PLAN §5 (import vs stateless-connect divergence) before closing. Roadmap home: `ROADMAP.md` § Hardening candidates.

### GTH-V15-05 — `resolve_import_parent()` silently degrades on ANY git error
- **Source:** part-02 (discovered-by P105 docs fix-twice lane, 2026-07-12 09:40) · **Severity: MEDIUM** · STATUS in intake: DEFERRED-TO-v0.15.0 (helper-hardening phase).
- **What:** `resolve_import_parent()` (`crates/reposix-remote/src/main.rs:400-419`) degrades to the parentless path (`None` → no `from`, no `deleteall`) on **any** git error, not just ref-absence. Two conflations: (1) the `rev_parse` closure returns `None` via `.ok()?` (`main.rs:407`) when the `git` spawn itself fails (binary missing / I-O error), swallowing a real environmental fault as "no parent"; (2) `!out.status.success()` (`main.rs:408`) treats every non-zero rev-parse exit as ref-absent. A future regression making rev-parse fail for a non-absence reason would silently re-open the RBF-LR-03 non-descendant "does not contain" abort with no operator-facing error. NOT addressed by the `bd5b9cb` disjoint-namespace fix (different failure mode).
- **Fix-sketch:** distinguish ref-absent (the legitimate parentless case: `rev-parse --verify --quiet <ref>` exits 1 with empty stdout AND the git spawn succeeded) from spawn / other rev-parse failures; on the latter, error the fetch loudly (`fatal:` + recovery hint) instead of degrading to a parentless overlay; keep the empty-stdout → `None` path for the genuine first-fetch case. Add a unit test injecting a non-absence git failure and asserting the fetch errors rather than emitting a parentless overlay. Small (<1h) but needs a cargo window. Belongs to the same `crates/reposix-remote` helper-hardening phase as GTH-V15-04 (residual verification) and GTH-V15-06.

### GTH-V15-06 — subprocess-bypass corruption residual: no binary-side self-safety refusal in `reposix init`  *(HIGH — also a ROADMAP stub)*
- **Source:** part-02 (discovered-by D2 re-seal Wave 1, 2026-07-12) · **Severity: HIGH** · STATUS in intake: DEFERRED-TO-v0.15.0. **The ACTIVE corruption vector is already CLOSED.**
- **What:** A live recurrence of the S-260707-pr-08 shared-tree corruption occurred AFTER P102 shipped: a leaf subagent created a git worktree INSIDE the shared repo and ran `reposix init` / sim-seed via a path that does NOT go through the Claude Code Bash tool, so the Bash-tool-only PreToolUse leaf-isolation hook never fired (`core.bare=true`, `origin` repointed to the sim, `HEAD` thrashed, `refs/reposix/*` polluted). The PRIMARY cut (hook Cases 9-11: config-read false-positive, git-init-bare, cargo-sim-seed spelling) shipped in P102 `2ad2bf5`; shared tree repaired at `9d78d62`; a partial binary-side check landed at `3206a2b`. What REMAINS is defense-in-depth: only a BINARY-SIDE refusal can stop a non-Bash-tool subprocess bypass.
- **Fix-sketch:** `reposix init` (NOT `attach` — attach legitimately adopts an existing checkout) refuses when its effective target would nest inside the reposix SOURCE checkout / shared dev tree, WITHOUT breaking the sanctioned `/tmp` dark-factory flow. Pair with a self-safety check that refuses to operate when the effective `.git` is the shared repo's object store (worktree-shared config detected). Full defense-in-depth cut + cross-flow testing = a dedicated v0.15.0 hardening phase (new binary code >1h). Deferring does NOT re-open the active vector — it hardens the already-closed one. Roadmap home: `ROADMAP.md` § Hardening candidates.

### GTH-V15-07 — release-plz (and other required workflows) unwatched by the phase-close CI probe
- **Source:** part-02 (discovered-by GSD-quick release-plz RED fix, 2026-07-12) · **Severity: MEDIUM** · STATUS in intake: DEFERRED (owner gate required).
- **What:** The phase-close `code/ci-green-on-main` (P0) probe hardcodes `WORKFLOW=ci.yml` and watches ONLY `ci.yml`, so a persistently-RED release-plz on main rots UNNOTICED (global CLAUDE.md: never let a metric you don't watch decay). Sibling of the RESOLVED `release.yml`-CI-ungated entry — that one GATES the tag-publish on green; this one is about WATCHING release-plz's outcome at phase-close.
- **BLOCKER NOW CLEARED:** the intake's original disposition named `quality/catalogs/code.json` as FOREIGN-LOCKED (a concurrent lane held uncommitted changes; the P110 drain forbade touching it). As of this landing the shared tree is **clean** (`git status` empty, code.json unmodified) — the catalog edit is no longer blocked. Only the owner-gate on the two open semantic questions remains.
- **Fix-sketch:** parameterize `quality/gates/code/ci-green-on-main.sh`'s hardcoded `WORKFLOW=ci.yml` into a required-workflow LIST, OR add a sibling `code/release-green-on-main` row at post-push cadence reusing the same latest-run-conclusion logic (catalog-first: write the GREEN-contract row before impl). **Resolve TWO open questions FIRST (owner gate — a false-RED would block UNRELATED phases):** (1) Does release-plz run on EVERY push to main? (2) Is a 'no release needed' run concluded `success` / `skipped` / other, so the probe treats non-failure correctly?

### GTH-V15-09 — Make the milestone-close vision-litmus fixture self-healing (backend AND mirror)
- **Source:** v0.14.0 tag-remediation lane, B1 (2026-07-13; mirror-drift dimension added after B1's reconcile probe) · **Severity: MEDIUM→HIGH (a stale mirror hard-BLOCKS the tag; see B1 evidence)**.
- **What (two distinct failure surfaces, both proven this session):**
  1. **Backend state drift** — an out-of-band edit to TokenWorld (e.g. trashing the space Home page 2818063) silently breaks the P0 9th-probe vision-litmus row, costing a full diagnosis + manual API restore. The fixture backend state is assumed, not enforced.
  2. **GitHub mirror drift (the actual B1 blocker)** — even with the backend correctly restored (`{2818063,7766017,7798785}`, 2818063 `status: current, version: 7`), the litmus STILL fails at `git push`. It clones the GitHub mirror repo `reposix-tokenworld-mirror.git` fresh each run and edits the first non-protected page (2818063); the mirror's `pages/2818063.md` is stale (`version: 1`) vs backend v7 with divergent content, so the push-time lost-update guard correctly rejects (`fetch first`). `reposix sync --reconcile` does NOT heal this — it rebuilds the LOCAL cache only, never pushes to the GitHub mirror repo (proven: mirror HEAD `3be8390` byte-identical before→after reconcile). Evidence: `.planning/milestones/v0.14.0-phases/evidence/B1-mirror-reconcile-FINDINGS-2026-07-13.md`.
- **Fix-sketch:** the litmus setup (`quality/gates/agent-ux/lib/litmus-flow.sh` / `milestone-close-vision-litmus.sh`) should self-heal BOTH surfaces before asserting: (a) detect trashed protected pages and restore them via the v2 `updatePage` `status→current` path (or fail with a copy-paste restore command); AND (b) **reconcile the GitHub mirror to backend-current before the marker push** — after clone+attach, fetch backend-current through the reposix bus remote (NOT the stale mirror, which `git pull` reads by default), rebase/rewrite the edited record onto the backend-current base, so the push carries a non-stale base version and its bus fan-out refreshes the mirror. So neither an out-of-band space edit NOR mirror drift can silently red the 9th probe again. NB: the helper's `Run: git pull --rebase` teaching string is misleading for the mirror-drift case (pull reads the stale mirror) — fixing that hint is part of this GTH.

## Hygiene (file-size early-warning)

### GTH-V15-08 — `.planning/ORCHESTRATION.md` over its progressive-disclosure ceiling
- **Source:** v0.14.0 file-size gate (`structure/file-size-limits`) · **Severity: MEDIUM (hygiene)**.
- **What:** `.planning/ORCHESTRATION.md` is **26968 B vs its 20000 B ceiling** (≈135%, >100% over-budget), currently **WAIVED until 2026-08-08**. When the waiver lapses the `structure/file-size-limits` gate reactivates and will BLOCK the push. It is already past the 75% early-warning band and over the hard ceiling.
- **Fix-sketch:** split the closed-doctrine / reference detail to a sibling (e.g. `ORCHESTRATION-detail.md` or `-history.md`) — the same progressive-disclosure move already landed for `ROADMAP.md → ARCHIVE.md` (v0.14.0 P-split) and `STATE.md → STATE-history.md` — keeping only the always-relevant dispatch/relief/cadence rules in `ORCHESTRATION.md`. Do it before the 2026-08-08 waiver lapses.

## From the b773c04 RED-main arc (2026-07-13, SESSION-HANDOVER successor #16 noticings)

### GTH-V15-10 — reconcile harness rc(0) vs artifact exit_code(1) mismatch + sim-readiness race
- **Source:** b773c04 RED-main arc (executor noticing during quick `260713-rug` prove-before-fix; routed by item-0 cursor refresh) · **Severity: MEDIUM** · STATUS: OPEN.
- **What:** During back-to-back local `container-rehearse.sh` runs, example-02 flaked ONCE — the harness returned rc=0 while a fresh artifact reported `exit_code: 1` / `FAIL: sim not reachable at 127.0.0.1:7878`; an isolated re-run (port 7878 confirmed free) came back rc=0 / exit_code 0 / asserts_failed []. Two coupled defects: (a) the harness return code and the persisted artifact `exit_code` are two success signals that can DISAGREE (the `exit "$EXIT_CODE"` vs EXIT-trap interaction lets rc=0 mask an artifact exit_code=1), so a grader trusting rc alone could pass a row the artifact failed; (b) a sim-readiness race between rapid sequential harness invocations — the prior row's ephemeral sim is still tearing down on host port 7878 when the next row binds/curls, yielding a transient `sim not reachable`. Sibling of the SIGKILL sim-leak surprise filed the same day; this one is the observable flake, that one is the orphan-process root cause.
- **Fix-sketch:** (a) make the harness return code and the artifact `exit_code` a single source of truth — have `container-rehearse.sh` derive its process exit strictly from the persisted `exit_code` (or assert they are equal and fail-loud on divergence), so a grader cannot read a green rc over a red artifact; (b) add a pre-`docker run` readiness gate — wait for host port 7878 to be FREE (previous sim fully reaped) before starting the next row's sim, and wait for the sim to be reachable before the container curls it (bind-retry / health-poll), so rapid sequential runs cannot race. Pairs with the SIGKILL sim-leak surprise (process-group teardown) in the same v0.15.0 docs-repro-harness-hardening phase.

### GTH-V15-11 — `.sim-*.log` under `quality/reports/verifications/docs-repro/` not gitignored
- **Source:** b773c04 RED-main arc (executor noticing during quick `260713-rug`; routed by item-0 cursor refresh) · **Severity: LOW (hygiene)** · STATUS: OPEN.
- **What:** The ephemeral `.sim-*.log` files that `container-rehearse.sh` drops under `quality/reports/verifications/docs-repro/` are NOT covered by `.gitignore` — the sibling `*.json` and `*.cobertura.xml` artifacts under that tree ARE ignored, but there is no `.sim-*.log` pattern, so the logs surface as untracked `??` after every docs-repro run and risk being accidentally `git add -A`'d into a commit.
- **Fix-sketch:** one-line addition to `.gitignore` — a `.sim-*.log` pattern scoped to `quality/reports/verifications/docs-repro/` (mirroring how the `*.json` / `*.cobertura.xml` artifacts in that tree are already ignored). Trivial; bundle into any v0.15.0 docs-repro-touching phase or a hygiene sweep.

### GTH-V15-12 — `doc-clarity-review` skill's nested `claude -p` returns a confusing non-error, not a hard fail, when it can't see file content
- **Source:** quick `260714-qhq` hero-qualifiers (executor noticing, 2026-07-14) · **Severity: LOW-MEDIUM** · STATUS: OPEN.
- **What:** Ran `~/.claude/skills/doc-clarity-review`'s prescribed `claude -p "$(cat _prompt.md)" file1 file2` invocation exactly as documented (copied README.md/docs/index.md to an isolated tmp dir first). The subprocess did NOT receive the file content — it replied that "no file content was included in this request," seeing only ambient session context (CLAUDE.md, tool listings). This matches the doc-alignment skill's own known caveat ("subscription users cannot fall back to `claude -p`") but the doc-clarity-review skill has no such warning and its instructions assume the invocation just works. A less careful agent skimming the output for a CLEAR/NEEDS WORK/CONFUSING verdict (rather than reading the full reply) could mistake the "I can't see the files" reply for an actual review outcome.
- **Fix-sketch:** either (a) have the skill probe once (e.g. a 1-line canary file) and hard-fail with a clear "nested claude -p unsupported in this environment, use Path A/Task-tool dispatch instead" message before burning the full review prompt, or (b) add the same subscription-caveat note the doc-alignment skill already carries, pointing callers at the Task-tool fallback. Small (<1h), lives in `~/.claude/skills/doc-clarity-review/SKILL.md` (outside this repo — user-global skill, not `.planning/`).

### GTH-V15-13 — README uses "MCP" without ever expanding the acronym on the page
- **Source:** quick `260714-qhq` hero-qualifiers cold-reader REVISE round (2026-07-14) · **Severity: LOW** · STATUS: OPEN.
- **What:** `README.md` uses "MCP" in at least two places ("no MCP tool schemas", "synthesized MCP-tool-catalog baseline") but never expands it to "Model Context Protocol" anywhere on the page — a cold reader unfamiliar with the acronym has no in-page anchor. `docs/index.md` does spell it out ("Model Context Protocol (MCP)"). Pre-existing; NOT introduced by the hero-qualifier edit.
- **Fix-sketch:** one-line first-use expansion in README — change the first "MCP" occurrence (the "no MCP tool schemas" line in the elevator pitch) to "Model Context Protocol (MCP)", leaving later uses as the bare acronym. Trivial (<5 min); bundle into any README-touching phase or a docs-hygiene sweep.

## From the L0 relief handover #19→#20 queue (2026-07-14, doc-alignment refresh session)

### GTH-V15-14 — pre-push docs-alignment block message cites the ratio, not the real blocking cause
- **Source:** successor #18 (2026-07-14) · **Severity: LOW-MEDIUM** · STATUS: OPEN.
- **What:** `walk.sh` stderr printed `alignment_ratio 0.4407 below floor 0.5000`, but the committed ratio was ~0.6994–0.7589 (above floor) — the real block was the hard-block-on-any-unwaived-blocking-STATE rule, not the ratio. Misleading diagnostic.
- **Fix-sketch:** make the block message name the blocking row-STATE(s), not (or in addition to) the ratio.

### GTH-V15-15 — doc-alignment grader compute-vs-assert reliability gap
- **Source:** successor #19, reinforced this session (2026-07-14) · **Severity: MEDIUM** · STATUS: OPEN.
- **What:** A grader false-bound `bench_token_economy.py` to a hero-number row by conflating "gate COMPUTES/PRINTS X" with "gate ASSERTS X" (`return 0` unconditionally); separately a prior grader left `git-2-34-requirement` MISSING_TEST by only inspecting the cited test, never grepping `src/` for the existing `git_version_2_*` doctor unit tests that DO bind it.
- **Fix-sketch:** harden `.claude/skills/reposix-quality-doc-alignment/prompts/grader.md` — (a) only BIND a row if the test fails when the number drifts, (b) grep `src/` unit tests, not just the currently-cited test.

### GTH-V15-16 — `plan-refresh` under-reports drift when invoked cold (before a `walk`)
- **Source:** successor #19 (2026-07-14) · **Severity: LOW** · STATUS: OPEN.
- **What:** `plan-refresh <doc>` only returns rows a PRIOR `walk` already persisted as stale — invoked cold it under-reported (3 rows vs. 21 from a subsequent `walk`).
- **Fix-sketch:** one-line note in the refresh playbook/prompt — "run `walk` first if invoked outside a pre-push block."

### GTH-V15-17 — doc-alignment `status` hides that MISSING_TEST rows are waived
- **Source:** this session (2026-07-14) · **Severity: LOW** · STATUS: OPEN.
- **What:** `status` prints `claims_missing_test 8` with no signal all 8 carry ACTIVE waivers (non-blocking) — the loud `WAIVED` lines only surface in `walk`, not `status`.
- **Fix-sketch:** add a `waived_active` counter to the `status` block.

### GTH-V15-18 — 16 pre-existing "cites out-of-eligible-file" coverage warnings
- **Source:** this session (2026-07-14) · **Severity: LOW** · STATUS: OPEN.
- **What:** doc-alignment rows citing e.g. `crates/reposix-core/src/backend.rs`, the `docs/architecture.md`/`docs/demo.md` redirect stubs, and `.planning/` archives are silently dropped from coverage accounting. Not caused by any recent change; flag for the coverage-dimension owner.
- **Fix-sketch:** audit whether the eligible-file allowlist should include these, or whether the rows should re-cite eligible files.

## From L0 rotation #22 (t4 real-backend re-run, 2026-07-14)

### GTH-V15-19 — `reposix sync --reconcile` oid-drift recovery claim is dubious for the systematic list-vs-get case
- **Source:** L0 rotation #22, t4 real-backend re-run (same session as the SURPRISES-INTAKE 2026-07-14 20:40 HIGH oid-drift defect entry) · **Severity: LOW (audit)** · STATUS: OPEN.
- **What:** `builder.rs`/`cache.rs` doc comments claim `sync --reconcile` recovers oid-drift, but a fresh `list_records` rebuild reproduces the same list-oid that still won't match the get-oid for the systematic Confluence list-vs-get representation-drift class (see the SURPRISES-INTAKE `list_records`-vs-`get_record` oid-drift entry on page 7766017, filed the same session) — so the recovery claim likely does NOT hold for that class of drift. Possible doc-lie; not yet proven, hence audit rather than fix-first.
- **Fix-sketch:** Audit `sync --reconcile`'s recovery claim once the SURPRISES-INTAKE oid-drift defect is fixed: re-run `reposix sync --reconcile` against a Confluence page exhibiting list-vs-get drift and confirm whether the reconcile actually clears the drift or merely reproduces the same stale list-oid. If it does not recover, correct the doc comments in `crates/reposix-cache/src/builder.rs` / `cache.rs` to stop claiming general oid-drift recovery, scoping the claim to the eventual-consistency race it was originally written for.

## From L0 rotation #26 (carry-forward intake filing, 2026-07-15)

### GTH-V15-20 — Stale `v0.12.0` example text in freshness-invariants catalog
- **Source:** carried forward across workhorse #24→#25→#26 handovers (2 rotations un-filed) · **Severity: LOW (cosmetic)** · STATUS: OPEN.
- **What:** `quality/catalogs/freshness-invariants.json` (~L227–229), the `structure/top-level-requirements-roadmap-scope` row's `expected.asserts` text hardcodes a stale `"v0.12.0"` example string. Doc-only, non-blocking, cosmetic.
- **Fix-sketch:** Update the example string to a current/representative milestone reference (or a placeholder pattern that doesn't go stale, e.g. `vX.Y.Z`). Fits naturally inside P119 (a DOCS-lane phase) — FILE only, do not fix now.

## From L0 rotation #27 manager queue (2026-07-15)

### GTH-V15-21 — Archived-milestone handover files will start BLOCKING pushes when the `structure/file-size-limits` waiver expires
- **Source:** manager (w1:p7) mid-task capture, 2026-07-15 · **Severity: MEDIUM** · STATUS: OPEN.
- **What:** Two ARCHIVED files exceed the file-size gate and are only kept passing by the active waiver: `.planning/milestones/v0.13.0-phases/97-HANDOVER.md` (31,271 chars) and `.planning/milestones/v0.14.0-phases/RELIEF-HANDOVER-C2-wave-2b.md` (20,132 chars). When the waiver expires **2026-08-08** the `structure/file-size-limits` gate will BLOCK any push.
- **Fix-sketch:** Decision-owner call before 2026-08-08 (hard deadline = waiver expiry): EITHER exempt archived milestone dirs (`.planning/milestones/v*.0-phases/` and/or `.planning/archive/`) from the file-size gate — likely correct, archives are immutable history — OR split the two files. No new dependencies; resolution fits easily into any v0.15.0 phase.

## From L0 rotation #30 push-unblock docs-alignment refresh (2026-07-15)

### GTH-V15-22 — `prior_rationale` line-refs in `doc-alignment.json` rot silently
- **Source:** Opus grader, `/reposix-quality-refresh docs/reference/testing-targets.md` (workhorse #30 push-unblock, 2026-07-15) · **Severity: LOW** · STATUS: OPEN.
- **What:** doc-alignment catalog rows store `prior_rationale` with hardcoded line refs, and nothing validates them against the live source — so they drift silently even when the underlying binding is sound (fns resolve by symbol, not by the stale line number). Observed instance: all JIRA rows in `quality/catalogs/doc-alignment.json` cited `agent_flow_real.rs:296`, but the real fn `dark_factory_real_jira` sits at `crates/reposix-cli/tests/agent_flow_real.rs:298`, its `skip_if_no_env!` at `:299`, and the URL-suffix assertion at `:308-311`.
- **Fix-sketch:** add a lint/periodic sweep that re-derives `prior_rationale` line refs from the current source and flags drift, OR drop line numbers from rationales in favor of symbol-only refs (fn/const names), which don't rot on unrelated edits above them in the same file. Small, no new dependency; fits a docs-alignment framework-hardening phase.

### GTH-V15-23 — `github-url-prefix` claim lives in an ADR blockquote, not the GitHub testing section
- **Source:** Opus grader, `/reposix-quality-refresh docs/reference/testing-targets.md` (workhorse #30 push-unblock, 2026-07-15) · **Severity: LOW** · STATUS: OPEN.
- **What:** Row `docs/reference/testing-targets/github-url-prefix` (claim: `remote.origin.url` starts with `reposix::https://api.github.com/`) is bound to prose at `docs/reference/testing-targets.md:245-251`, which is the ADR-008 dispatch note, not a "GitHub env vars" section. The binding itself is sound (the cited test asserts exactly that prefix) but a reader scanning the GitHub testing section for the URL contract won't find it stated there.
- **Fix-sketch:** also state the literal remote-URL prefix contract in the GitHub testing section proper (near the other GitHub env-var / setup claims), leaving the ADR-008 blockquote as-is for the dispatch-note context. Trivial (<15 min doc edit); bundle into any `docs/reference/testing-targets.md`-touching change (mind the refresh-tail caveat — this edit will itself drift catalog rows and need a `/reposix-quality-refresh` pass).

## From gsd-quick lane 260715-mk5 public roadmap diagram (2026-07-15)

### GTH-V15-24 — Structure gate asserting the roadmap↔PROJECT `<!-- SYNC:` marker pair exists on BOTH sides
- **Source:** gsd-quick lane 260715-mk5 (owner-approved w1:p7), optional noticing-grade extra · **Severity: LOW** · STATUS: OPEN.
- **What:** The public roadmap (`docs/roadmap.md`) and the planning ledger (`.planning/PROJECT.md`) now carry a bi-directional keep-in-check link, each with an adjacent `<!-- SYNC: ... -->` comment. Nothing mechanically asserts the pair stays symmetric — if one side drops its `<!-- SYNC:` comment or its link during an edit, the drift is silent until a human notices. link-resolution.py now checks the LINKS resolve (both directions), but not that the SYNC *comments* both still exist. Deferred from this lane because a real structure gate is a multi-file add, not a trivial inline one.
- **Fix-sketch:** add a `verify_sync_marker_pair` fn to the `DISPATCH` dict in `quality/gates/structure/freshness-invariants.py` asserting `grep -c '<!-- SYNC:'` is ≥1 in BOTH `docs/roadmap.md` and `.planning/PROJECT.md` (and, stretch, that each SYNC line sits next to its cross-link); register a catalog-first row `structure/roadmap-project-sync-pair` in `quality/catalogs/freshness-invariants.json` (cadence pre-push, blast_radius P2); add a `.selftest.sh` building a throwaway `/tmp` repo. Small but genuinely multi-file — fits a structure-dimension or DOCS-lane phase.

## Back-pointer note (bidirectional trail — INTENTIONALLY SKIPPED)

Task step 5 offered to append a `→ landed: v0.15.0-phases/GOOD-TO-HAVES.md` back-pointer to each
migrated entry in `part-01.md` / `part-02.md`. **Skipped by design:** both part files are ALREADY
over the 20000-char `.md` ceiling (part-01 = 21516 B, part-02 = 21574 B) — appending any text pushes
them further over budget, contradicting the OP-8 file-size drain the split was performed for. The
forward trail (this file → intake, cited per-row above) is the required deliverable and is complete;
the reverse pointer is deferred to whenever those part files are themselves progressive-disclosure-split.

## From owner↔ex-manager-#9 review session (2026-07-15)

### GTH-V15-25 — Token-bloat CI tripwire: replayed-trajectory byte guard calibrated by the JSONL benchmark
- **Source:** OWNER-PROPOSED 2026-07-15 (side session with ex-manager #9); manager assessed feasible and worth the effort · **Severity: MEDIUM** · STATUS: OPEN.
- **What:** The P115 token-economy numbers come from live JSONL-captured sessions — trustworthy, but infrequent feedback. Between re-benchmarks, a reposix change that bloats agent-visible output (fatter record frontmatter, noisier CLI/helper/error messages, bigger git output) stays invisible until someone manually re-runs the benchmark. Latency has a per-push guard (`bench-latency-v09` + `warn_if_over_3s()`); token economy has none.
- **Fix-sketch (owner sketch + manager refinements; DP-3 applies — this is a problem statement, not a spec):**
  1. **Trajectory fixture (near-free during T4):** extract the agent's command list from each captured benchmark session's JSONL → commit as a canonical-trajectory fixture per benchmark row. This extraction is a <1h byproduct of T4 capture work — eager-absorb it there; the guard itself is a separate small lane.
  2. **CI byte guard:** a `quality/gates/perf/` verifier (sibling of `latency-bench/`) replays the command list against an ephemeral sim in `/tmp` (dark-factory.sh pattern), captures all agent-visible bytes (stdout + stderr + file contents read), and asserts against a RATCHETED threshold — baseline +10% WARN, +25% FAIL; re-baselining is an explicit reviewed commit, never silent decay.
  3. **Per-backend unit checks:** render one canonical record through each adapter (sim/GitHub/JIRA/Confluence fixtures) with a per-adapter size budget — localizes the bloat source when the end-to-end guard fires.
  4. **Honesty rule:** the CI metric is BYTES, labeled as bytes, never published as tokens — docs keep only JSONL-measured token numbers (consistent with the T6 headline-framing ruling). Each real JSONL re-benchmark records the bytes→tokens ratio for the trajectory (provenance-labeled: harness, model, date) purely as calibration for interpreting the guard.
  5. **Known gap (accepted):** fixed-command replay catches content bloat, not workflow bloat (the agent needing MORE steps because UX regressed). Complement later by measuring the dark-factory live-agent transcript size at release cadence — no new API dependency.
- **Effort:** small lane (catalog-first: one catalog row + one verifier, runner discovers by tag) + the T4-byproduct extraction. Depends on P115 T4 fixtures landing first; fits OP-8 Slot-2 drain or any perf/quality-hardening phase after that.

### GTH-V15-26 — Regenerate the `reposix_session.txt.tokens.json` sidecar (stale after T4 replaced the fixture) + capture the MCP-fidelity talking point
- **Source:** P115 Task-4 capture executor (L0 #39), 2026-07-16 · **Severity: LOW** · STATUS: **RESOLVED by P115 T5 (2026-07-16)**.
- **Resolution (T5):** Took the "cleaner, aligned with T5" option flagged below — retired the count_tokens-on-fixtures headline in favour of the `benchmarks/captures/*.json` JSONL-usage methodology, which makes the sidecar obsolete. The stale `benchmarks/fixtures/reposix_session.txt.tokens.json` was **deleted** (nothing consumes it under the captures path; `bench_token_economy.py --offline` no longer counts fixtures for the headline). The MCP-fidelity talking point landed as a **factual** honesty caveat in `docs/benchmarks/token-economy.md` § "What this does NOT measure" (no marketing framing — that framing stays T6's call). `benchmarks/fixtures/README.md` provenance updated to match. Marketing-grade positioning of the fidelity point remains a T6 headline-reframe discretionary.
- **What (sidecar):** T4 replaced the synthetic FUSE-era `benchmarks/fixtures/reposix_session.txt` (had `/mnt/` paths) with a **real git-native GitHub transcript** (8041 bytes). This staled its content-hash sidecar `reposix_session.txt.tokens.json` (still records the old sha256 + 531 tokens). The offline token-economy bench (`quality/gates/perf/bench_token_economy.py --offline`) would now `SystemExit` on that fixture's cache miss — but this is **non-blocking**: the gate is NOT in `ci.yml`, and its catalog row `perf/token-economy-bench` is **WAIVED until 2026-09-15**. No `ANTHROPIC_API_KEY` was available at capture time to regenerate honestly (the README forbids hand-editing sidecars). **Fix:** with an API key, `ANTHROPIC_API_KEY=… python3 quality/gates/perf/bench_token_economy.py` to refresh the sidecar; and update the `benchmarks/fixtures/README.md` inventory row (byte count 1372→8041) + note this fixture is now real-captured, not synthetic. OR — cleaner, aligned with T5 — retire the count_tokens-on-fixtures bench in favour of the `benchmarks/captures/*.json` JSONL-usage methodology (T5 owns that regen), which makes the sidecar obsolete.
- **What (fidelity talking point, evidence already committed):** the T4 runs surfaced that `github-probe`'s `issue_read` **HTML-escapes body content** (`>=` → `&gt;=`) and **drops literal angle-bracket content**, so an MCP read-modify-write round-trip corrupts raw markdown (it even lost the HTML-comment benchmark marker), while the reposix arm round-trips raw bytes faithfully. This is strong, real evidence for reposix's bytes-in-bytes-out fidelity value prop — worth a line in the token-economy/positioning docs (recorded in `mcp_github_catalog.json` `_note` and `115-MCP-SERVER-CHOICE.md`). No new dependency; a doc/positioning nicety.
- **Effort:** tiny (one command + a README row) for the sidecar; the talking-point is a doc lane during T6 headline reframe.

## From L0 relief handover #39→#40 (carry-forward noticing filed, 2026-07-16)

### GTH-V15-27 — Milestone-scoped `v0.15.0-phases/ROADMAP.md` is a stale "PLANNING / Phase TBD" stub superseded by the live `.planning/ROADMAP.md`
- **Source:** noticed by L0 #38 while grounding its handover; carried forward and filed by L0 #39 (closing the #38→#39 "noticed, not yet filed" item) · **Severity: LOW** · STATUS: OPEN.
- **What:** `.planning/milestones/v0.15.0-phases/ROADMAP.md` is a stale "v0.15.0 … (PLANNING)" stub — scheduled 2026-07-12 by `[SELF]` (D6 in `CONSULT-DECISIONS.md`) with "Phase TBD" placeholders and phase numbers left UNASSIGNED. The LIVE index is the top-level `.planning/ROADMAP.md`, scoped 2026-07-15 via `/gsd-new-milestone` to **Phases P114–P128**. By the `.planning/CLAUDE.md` § Milestones-layout convention, per-milestone ROADMAPs live inside `*-phases/`, so the stub *looks* authoritative by naming — but it is superseded: a reader trusting it gets no real phase list, only the pre-scoping scaffold.
- **Fix-sketch:** either (a) populate `v0.15.0-phases/ROADMAP.md` with the live P114–P128 index (or a short pointer/redirect to `.planning/ROADMAP.md`), OR (b) delete the stub if the top-level `.planning/ROADMAP.md` is the single source of truth for this milestone. Low stakes, <1h either way; fits any planning-hygiene or DOCS-lane phase.

## From P115 T6 Wave 2 item 5 (regen-clobber guard, 2026-07-16)

### GTH-V15-28 — Line-anchored docs-alignment citations are a silent sharp edge for any future edit to a heavily-bound doc
- **Source:** noticed by the P115 T6 Wave-2 item-5 executor while adding the `emit-markdown.sh` regen-clobber guard · **Severity: LOW** · STATUS: OPEN.
- **What:** `docs/benchmarks/latency.md` has 14 doc-alignment rows bound into its table/soft-threshold sections. Inserting a 9-line marker comment right after the frontmatter (before any of those sections) tripped `STALE_DOCS_DRIFT` on **all 14** rows at once, even though none of the actual cited *values* changed — confirming the citations are line-anchored, not content-hash-anchored to a stable span. This is a general sharp edge, not unique to this task: any editor of a heavily-bound doc who inserts lines above a citation (a new intro paragraph, a banner, a TOC) will trip drift on everything below, and the only signal is a pre-push BLOCK with no hint that "line-anchored, insert-above-shifts-everything" is the mechanism.
- **Fix-sketch:** (a) documentation-only, cheapest: add a one-line callout to `quality/CLAUDE.md` § docs-alignment dimension warning editors that citations are line-anchored and inserts above a bound section will drift everything below it — prefer appending new content, or editing in-place, over prepending. (b) tooling fix, bigger: teach the binder/citation extractor to anchor on a stable content hash of the cited span itself (or a small context window) rather than an absolute line number, so edits elsewhere in the file don't cascade. (a) is a <15min doc fix; (b) is a real docs-alignment tooling lane.
- **Effort:** (a) trivial, fits any doc-touching lane; (b) medium, fits a docs-alignment tooling hardening phase (e.g. P126 "Docs-alignment tooling polish").

## From P115 T6 Wave 2 item 6b (cold-init reconcile + loop/perf un-waive, 2026-07-16)

### GTH-V15-29 — `doc-alignment bind --test` help advertises `<file>::<fn>` but the fn half is optional + unvalidated
- **Source:** P115 T6 Wave-2 item-6b executor · **Severity: LOW** · STATUS: OPEN.
- **What (refines the charter's original framing after first-hand observation):** `bind --test` help says the citation is `<file>::<fn>` (repeatable). Empirically it accepts BOTH forms without complaint: bare `<file>` (the perf rows bind to `quality/gates/perf/headline-numbers-cross-check.py` with no `::fn`) AND `<file>::<fn>` (`crates/reposix-cache/tests/blobs_are_lazy.rs::no_blob_objects_after_build_from` bound exit-0). The fn half is not enforced to resolve in the file at bind time — so the format string over-promises: it implies fn is required + checked, when it is an optional, unvalidated label. (The charter's phrasing "validates the whole string as a file path" would predict `::fn` to be REJECTED; observed behavior is the opposite — it is accepted, just not resolved.)
- **Fix-sketch:** either (a) enforce fn resolution (parse `::fn`, assert the symbol exists in the cited file — closes the "test name lies" surface at bind time, sibling to `test-name-vs-asserts.sh`), or (b) fix the help to document `--test` as `<file>` or `<file>::<label>` where the label is an unvalidated human hint. (b) is trivial; (a) is a small doc-alignment tooling lane.

### GTH-V15-30 — `*-8ms*` / `*-24ms*` row IDs now carry 6 ms / 278 ms claims (cosmetic ID↔claim drift)
- **Source:** P115 T6 Wave-2 item-6b executor · **Severity: LOW** · STATUS: OPEN.
- **What:** after the T6 latency reconcile, several doc-alignment row IDs embed a now-wrong number: `docs/index/latency-24ms-cold-init` / `latency-hero-24ms-mismatch` / `README-md/init-24ms` / `docs/index/soft-threshold-24ms` / `docs/index/bootstrap-latency-24ms` all now carry **278 ms** cold-init claims; `docs/index/latency-8ms-read` / `latency-cached-read-8ms` / `README-md/latency-8ms` carry **6 ms** cached-read claims. The IDs are stable primary keys cited across catalog/closeout/PROGRESS, so renaming is risky (breaks cross-refs) for a purely cosmetic mismatch — the `claim`/`source` fields are the source of truth and are correct.
- **Fix-sketch:** leave IDs as-is; if ever renamed, do it in one sweep with a cross-ref updater (catalog + all `.planning/**` mentions). Not worth a dedicated lane — fold into any future catalog-ID-hygiene pass.

### GTH-V15-31 — `webhook-latency.json` tracked-while-siblings-ignored reads like an accident (it is a deliberate P84 exception)
- **Source:** P115 T6 Wave-2 item-6b executor (verified against `.gitignore`) · **Severity: LOW** · STATUS: OPEN.
- **What:** `.gitignore:72` ignores `quality/reports/verifications/*/*.json`; `.gitignore:79-82` carve an explicit exception `!quality/reports/verifications/perf/webhook-latency.json` (P84 `asset-exists`-kind committed evidence, read by `webhook-latency-floor.sh`). It IS intentional + commented, but a reader scanning tracked files sees exactly one `verifications/**.json` tracked among ignored siblings and may mistake it for a stray commit. Not a bug — a clarity papercut.
- **Fix-sketch:** (a) trivial — no code change, just ensure any future `asset-exists` verification artifact that must be committed gets the same commented `!`-exception + a one-line note in `quality/catalogs/README.md` § kinds that `asset-exists` evidence is git-tracked by design; or (b) relocate committed evidence artifacts to a non-ignored `quality/reports/evidence/` subtree so the ignore rule needs no per-file exception.

### GTH-V15-32 — `headline-numbers-cross-check.py` sits at 99% of its 15 000-char ceiling
- **Source:** P115 T6 Wave-2 item-6b executor · **Severity: LOW** · STATUS: OPEN.
- **What:** item 6b extended the gate with cold-init + loop-figure cross-checks; after trimming the docstring/comments it is 14 922/15 000 chars (99%, EARLY-WARNING band). The NEXT claim category added will breach the ceiling and force a split under time pressure.
- **Fix-sketch:** pre-emptively extract the canonical-parsing layer (`parse_latency_canonical` / `parse_token_canonical` / `_table_rows` / `_int_cell`) into a sibling `headline_cross_check_parse.py` (mirrors the `latency-bench/` per-concern layout), leaving the claim registries + `run_cross_check` in the gate. ~15min, fits any perf-gate touch.

### GTH-V15-33 — 3 mis-cited/stale `24 ms` rows on `mental-model-in-60-seconds.md:21` need re-citation + test realignment
- **Source:** P115 T6 Wave-2 item-6b executor (left out of scope to avoid destabilizing non-blocking drift) · **Severity: LOW** · STATUS: OPEN.
- **What:** `mental-model-in-60-seconds.md:21` ("the bootstrap takes `24 ms`") is cited by 3 rows — `bootstrap-timing-24ms-vs-27ms`, `docs/why/cold-init-24ms-sim`, `docs/why/cached-read-8ms` — all `STALE_TEST_DRIFT` (bound to `latency-bench.sh`, which drifted after the item-5 regen-guard edits). One (`docs/why/cached-read-8ms`, claim "Cached read is 8ms") is additionally MIS-CITED: it points at the bootstrap line, not a cached-read line. Item 6b reconciled the hero surfaces + the same page's L69 to canonical 278 ms but left L21 untouched: changing its source would risk flipping these 3 non-blocking rows to a blocking state, and re-binding them needs a GREEN test binding that doesn't yet assert 278 ms. Result: L21 (24 ms) is now internally inconsistent with L69 (278 ms) on the same page.
- **Fix-sketch:** a small doc-alignment realignment lane — reconcile L21 `24 ms → 278 ms`, re-cite the 2 legitimate rows to the cross-check gate (extend it to cover the mental-model page, or bind to `headline-numbers-cross-check.py`), and either fix or retire the mis-cited `docs/why/cached-read-8ms` row. Sequence with GTH-V15-28 (line-anchored citations).

## From the P115 owner-directive lane (retirement-narrative strip, 2026-07-16)

### GTH-V15-34 — `reposix-quality doc-alignment confirm-retire --batch <file>` mode
- **Source:** P115 owner-directive lane, filed while writing the FINAL 11-row confirm-retire batch in `115-UNWAIVE-PATH.md` · **Severity: LOW-MEDIUM** · STATUS: OPEN.
- **What:** The human gate currently requires 11 sequential TTY invocations of `confirm-retire --row-id <ID>`, each hand-transcribed by the owner from a markdown list. This is real toil (11 copy-pastes) and a real transcription-error surface (a mistyped row-id either no-ops or, worse, retires the wrong row) for what is conceptually one decision — "retire this batch of superseded claims."
- **Fix-sketch:** add a `--batch <file>` mode to `confirm-retire` that reads one row-id per line from `<file>`, prints the full resolved list (row id + claim + current `last_verdict`) for review, asks a SINGLE interactive confirmation (not one per row), and applies all retirements atomically on `y`. Must keep the fail-closed TTY/human guard exactly as strict as the single-row path — refuses under `CLAUDE_AGENT_CONTEXT` / non-TTY stdin identically, and does not introduce a new bypass. Reduces an N-invocation manual gate to one review + one confirmation without weakening the human-only property.

## From the owner-directive lane (post-P115-close scheduling, 2026-07-16)

### GTH-V15-35 — `docs/index.md`: nest "Build from source (advanced)" under "30-second install"
- **Source:** OWNER-DIRECTED (2026-07-16, received after commit `a1f2494`) · **Severity: LOW (docs IA)** · STATUS: **SCHEDULED — immediately after P115 phase-close, as a tracked `/gsd-quick`.**
- **What:** `docs/index.md`'s `<details><summary><strong>Build from source (advanced)</strong></summary>` block (currently ~L120-136, sitting after "Connector capability matrix") reads as a disconnected install path — a cold reader following the "30-second install" section (L44-67: curl/PowerShell/Homebrew/cargo-binstall tabs) has no visual link to the advanced from-source alternative several screens later. Owner directive: move/nest the "Build from source (advanced)" material under the "30-second install" section so the two install paths (package-manager fast path + from-source advanced path) read as one cohesive section.
- **Execution cautions (MUST hold, verbatim from the owner directive):**
  1. The `structure/install-leads-with-pkg-mgr-docs-index` freshness gate must stay GREEN — the install path must continue to lead with the package-manager options (curl/Homebrew/cargo-binstall/PowerShell), with "Build from source" remaining the subordinate/advanced/collapsed alternative, never promoted above or beside the primary path.
  2. Doc-alignment catalog rows bound to `docs/index.md` must be CHECKED BEFORE the edit (`reposix-quality doc-alignment status` / `walk` scoped to that file) and REFRESHED in the SAME wave as the move (moving lines will shift line-anchored citations per GTH-V15-28 — do not leave rows STALE_DOCS_DRIFT across a wave boundary).
- **Fix-sketch:** relocate the `<details markdown>…Build from source (advanced)…</details>` block to sit immediately after (or nested as a collapsed `<details>` within) the "30-second install" `=== "curl…"` / Homebrew / cargo-binstall tab set (L44-67), before "After — one commit" (L69); re-run `structure/install-leads-with-pkg-mgr-docs-index` + the doc-alignment refresh for the shifted rows in the same commit/wave.
- **Effort:** small (<1h), single-file docs move; execute via `/gsd-quick` per the owner's scheduling.

### GTH-V15-36 — Docs site as a "furnished product": owner furnished-product quality bar for P117/P119
- **Source:** OWNER MANDATE (2026-07-16, received after commit `a1f2494`) · **Severity: MEDIUM-HIGH (broad quality bar, P117/P119 shaping input)** · STATUS: OPEN — feeds P117 (doc-truth purge) and P119 (docs simplification) planning.
- **Owner quote (verbatim):** *"Its good, but we can do so much better!"*
- **What:** Broad owner mandate that the docs site (mkdocs-built, `docs/`) should read as a **FURNISHED PRODUCT** with streamlined documentation — not merely factually correct (P117's bar) or merely destaled (P119's bar), but polished as a cohesive, professional surface. This is explicit quality-bar shaping input for BOTH P117 and P119, covering:
  1. **Information architecture** — sections in the order a first-time reader / agent actually needs them; no orphaned or oddly-sequenced material (see GTH-V15-35 for a concrete instance of this class).
  2. **Progressive disclosure** — the 30-second path stays first and uncluttered; advanced/edge-case material (build-from-source, ADR blockquotes, superseded-figure history, deep config) is pushed down/collapsed, never competing with the fast path for attention.
  3. **Visual polish of the mkdocs site** — consistent use of admonitions/tabs/details, no walls of undifferentiated prose, mermaid diagrams that render cleanly (per global CLAUDE.md Operating Principle 1), badges/links that resolve.
  4. **Cold-reader rubric pass over every landing surface** — `docs/index.md`, `README.md`, and other first-touch pages graded via `/doc-clarity-review` / `/reposix-quality-review --rubric <id>`, not just mechanically gated.
- **Fix-sketch:** P117 planner and P119 planner both take this mandate as an explicit acceptance-bar input (not just their existing success criteria) — plan a cold-reader/IA pass as a first-class task in each phase's PLAN.md, not a leftover. Concretely: (a) inventory every landing/tutorial page against the progressive-disclosure decision table (global CLAUDE.md § Self-improving infrastructure), (b) push advanced/edge material below the fold or into collapsed `<details>` blocks consistently (GTH-V15-35 is one instance; audit for siblings), (c) run `/doc-clarity-review` + the cold-reader rubric on `docs/index.md` + `README.md` + any other first-touch page BEFORE declaring either phase done, (d) treat "good, but so much better" as a directive to over-deliver polish, not just clear the stated Success Criteria checkboxes.
- **Effort:** shaping input, not a standalone task — folds into P117 + P119 planning; each phase's plan should size its own cold-reader/polish pass.

### GTH-V15-37 — Embed the owner's 80s launch animation on the mkdocs home page
- **Source:** OWNER-APPROVED P117 LANE ADDITION (2026-07-16); manager feasibility spike VERIFIED via headless playwright (`~/workspace/reposix-animation-pitch`, client-side React/JSX, 7 scenes) — renders flawlessly, scales to iframe container, zero JS errors · **Severity: MEDIUM (owner-approved scope addition)** · STATUS: OPEN — P117-shaping input.
- **What:** Embed the owner's 7-scene launch animation on the mkdocs home page (`docs/index.md`), productionized per the owner-approved checklist below (verbatim).
- **Productionization checklist (MUST hold, verbatim):**
  1. Pre-compile the JSX offline to a plain JS bundle — removes the `unpkg.com` Babel-standalone/React CDN dependency AND the ~2.8s in-browser-compile blank; self-host React or inline it.
  2. Self-host the two Google Fonts (Space Grotesk, JetBrains Mono).
  3. Embed mode: `TWEAK_DEFAULTS.motionEditor=false`, neutralize the `localStorage` `animstage:t` playhead persistence (returning visitors currently get a frozen end frame), poster + click-to-play rather than autoplay.
  4. Owner's 7MB mp4 export = video fallback + Show-HN/social asset, host as a GitHub release attachment NOT committed to the repo (file-size gates).
  5. Docs gates: assets under `docs/assets/animation/`, strip Windows `Zone.Identifier` files from uploads/, mkdocs-strict + playwright-walk coverage.
- **Addendum (2026-07-16, L0 #44):** the checklist-item-4 mp4 export now EXISTS at
  `/home/reuben/workspace/reposix-animation-pitch/Reposix Launch Animation.mp4` (7.1MB,
  verified on disk 2026-07-16) — video fallback + Show-HN/social asset; attach to a
  GitHub release, never commit to the repo.
- **Fix-sketch:** P117 planner adds a dedicated task implementing the 5-item checklist above in order (bundle precompile → font self-host → embed-mode config → video-fallback hosting → docs-gate coverage), each item independently verifiable; feasibility already de-risked by the manager's headless-playwright spike, so this is an implementation lane, not a research spike.
- **Effort:** medium — new static asset pipeline + mkdocs page wiring; fits inside P117 as an owner-approved scope addition alongside the doc-truth purge.
