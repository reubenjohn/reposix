# v0.15.0 Good-to-haves / carried-forward hardening — Part 2 of 9

> Split from `GOOD-TO-HAVES.md` for the file-size gate (OP-8 drain). Index: `../GOOD-TO-HAVES.md`. Entries preserved verbatim.

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

