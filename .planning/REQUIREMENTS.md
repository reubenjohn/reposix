# Requirements — Active milestone: v0.15.0 Floor

**Status:** PLANNING (Step 9 of `/gsd-new-milestone` re-anchor — requirements defined;
phase decomposition is Step 10, not yet executed as of this commit). Arc D RATIFIED at
`6aa734a` (ADDENDUM: `.planning/milestones/audits/2026-07-12-reality-check.md`).

**Milestone goal:** Establish the launch-readiness floor — fix the t4 Confluence
oid-drift product defect, purge doc-truth launch-blockers, harden every user-facing error
to Rust-compiler-grade, re-measure live benchmarks, produce the ADR-010 mirror-fanout
decision packet, and simplify planning/docs — closing the known correctness and honesty
gaps before the journey-slice milestones (v0.23+).

**Context anchors:** `.planning/PROJECT.md` § Current Milestone: v0.15.0 Floor (the 6
lanes); `.planning/milestones/v0.15.0-phases/{SURPRISES-INTAKE,GOOD-TO-HAVES,ROADMAP}.md`
(row-level detail this file routes); `.planning/milestones/audits/2026-07-12-reality-check.md`
(Arc D authority — LAUNCH-BLOCKER inventory, disputed-metric findings, Q5/Q7 simplification
mandate).

**Shipped milestones — full REQ-ID detail in per-milestone files:**
- v0.14.0 Wave-2 hardening — `.planning/milestones/v0.14.0-phases/REQUIREMENTS.md` (SHIPPED, Phases P102–P113).
- v0.13.1 Front door hotfix — `.planning/milestones/v0.13.1-phases/REQUIREMENTS.md` (SHIPPED 2026-07-08, Phases P98–P101).
- v0.13.0 DVCS over REST — `.planning/milestones/v0.13.0-phases/REQUIREMENTS.md` (SHIPPED 2026-07-07, Phases P78–P97).
- v0.12.x Quality Gates + Carry-forwards — `.planning/milestones/v0.12.0-phases/REQUIREMENTS.md` (v0.12.0 SHIPPED 2026-04-28, Phases 56–65) + `.planning/milestones/v0.12.1-phases/REQUIREMENTS.md` (v0.12.1 SHIPPED 2026-04-30, Phases 72–77).
- v0.11.x Polish & Reproducibility — `.planning/milestones/v0.11.0-phases/REQUIREMENTS.md` (v0.11.0 SHIPPED 2026-04-25; v0.11.1/v0.11.2 polish SHIPPED 2026-04-26/27).
- v0.10.0 Docs & Narrative Shine — `.planning/milestones/v0.10.0-phases/REQUIREMENTS.md` (SHIPPED 2026-04-25, Phases 40–45).
- v0.9.0 Architecture Pivot — `.planning/milestones/v0.9.0-phases/REQUIREMENTS.md` (SHIPPED 2026-04-24, Phases 31–36; ARCH-01..19).
- v0.8.0 and earlier — see `.planning/milestones/v0.X.0-phases/ARCHIVE.md`.
- v0.13.2 Cross-link fidelity — QUEUED, not shipped (scaffolded P98–P107 placeholder numbering, zero phases executed as of 2026-07-14; resequenced behind the launch-readiness arc).

> **Convention.** Per `.planning/CLAUDE.md` § Milestones layout, each milestone's
> REQUIREMENTS.md lives inside its `*-phases/` directory once shipped. The top-level file
> holds ONLY the active milestone + this shipped-milestone index.

## v0.15.0 Requirements

### Lane 1 — FIX: t4 oid-drift fix-first (correctness)

- [ ] **FIX-01**: Fix the Confluence `list_records`-vs-`get_record` oid-drift product
  defect. Root cause: `crates/reposix-cache/src/builder.rs:610-618` (`read_blob`) computes
  the tree oid from the `list_records` body at build time but materializes the blob from
  the `get_record` body at checkout time; for pages where the two API representations
  render to different bytes (proven deterministic on page 7766017 — NOT the
  eventual-consistency race the code comment assumes) the drift-check aborts `git checkout
  -B main` before any push. Fix either (1) the Confluence adapter so `list_records` and
  `get_record` render identical bytes per page, or (2) `build_from`'s oid computation to
  use the get-representation it will actually materialize from. Acceptance: the P0 gate
  `agent-ux/t4-conflict-rebase-ancestry-real-backend` passes against live Confluence
  TokenWorld with zero oid-drift abort.
- [ ] **FIX-02** *(depends on FIX-01 landing)*: Audit `reposix sync --reconcile`'s
  oid-drift recovery claim (GTH-V15-19). `builder.rs`/`cache.rs` doc comments claim
  `sync --reconcile` recovers oid-drift; confirm whether a fresh `list_records` rebuild
  actually clears the systematic list-vs-get drift class FIX-01 targets, or merely
  reproduces the same stale list-oid. If it does not recover, correct the doc comments to
  scope the claim to the eventual-consistency race it was originally written for.

### Lane 2 — DOCS: doc-truth launch-blocker purge + planning simplification

- [ ] **DOCS-01**: Fix `docs/index.md:13` category+scope error — "REST-based issue
  trackers (Jira, GitHub Issues, Confluence)" mischaracterizes Confluence as an issue
  tracker; rewrite to "systems of record — issue trackers (Jira, GitHub Issues) and wikis
  (Confluence)". Same sentence's "can `git clone`" claim is also wrong — the bootstrap verb
  is `reposix init`.
- [ ] **DOCS-02**: Rewrite `docs/how-it-works/filesystem-layer.md` off its false FUSE-era
  premise (L7-10/19/42/63 claim `cat` "secretly triggers a network call" / "becomes a REST
  call"; self-contradicts at L57: "reposix never touches the tree except git fetch/push").
  Rename the page — the title itself is architecture rot (L17 admits "there is no virtual
  filesystem") — and fix propagated cross-links at `index.md:144`, `git-layer.md:120`,
  `time-travel.md:90/92`, `trust-model.md:103`.
- [ ] **DOCS-03**: Un-strand `reposix list` (`list.rs:80`) and `reposix refresh`
  (`refresh.rs:207`) connection-refused errors — both currently surface an opaque reqwest
  error with zero teaching when the sim (the DEFAULT backend) isn't running. Reuse the
  3-part teach-fix/suggest-alternative/copy-paste-recovery text already proven at
  `init.rs:365-370`.
- [ ] **DOCS-04**: Delete-or-implement the phantom `reposix detach` subcommand.
  `crates/reposix-cli/src/attach.rs:135-138`'s multi-SoT-conflict error tells the user to
  "Run `reposix detach` first" but no `detach` subcommand exists anywhere in
  `crates/reposix-cli/src/`. Implement it, or rewrite the error to a real recovery path.
- [ ] **DOCS-05**: Relabel the token-fixture provenance lie. `docs/benchmarks/token-economy.md:51-52`
  + `benchmarks/README` claim `reposix_session.txt` (531 tokens) is "the literal output of
  `scripts/demo.sh`" — that script does not exist, and the fixture depicts the deprecated
  FUSE architecture (`/mnt/reposix/...` paths, internally inconsistent IDs). Relabel
  honestly (hand-authored fixture, not a captured demo run) even before BENCH-01
  re-measures.
- [ ] **DOCS-06**: Fix `docs/social/twitter.md:16`'s FUSE framing — "a FUSE filesystem +
  git-remote-helper for issue trackers" describes the pre-v0.9.0 deleted architecture;
  rewrite to the current git-native partial-clone description.
- [ ] **DOCS-07**: Retract/relabel the disputed "~150k→~2k (98.7%)" token-count figure and
  the FUSE-era `/mnt/jira/PROJ-123.md` example path in `.planning/PROJECT.md`'s Context
  section (owner-retraction-flagged by the 2026-07-12 reality-check). Replace with the
  CI-verified 89.1% figure (or BENCH-01's re-measured value) and a git-native example path.
- [ ] **DOCS-08**: Docs/planning simplification (the "P112 RAISE"). Delete stale/superseded
  legacy planning and doc content OUTRIGHT rather than keep-with-banners — git history IS
  the archive (ratified Q5/Q7, Arc D ADDENDUM). Scope: the loose phase-dir archival cascade
  now unblocked by both v0.13.0/v0.14.0 tags landing, the stale top-level catalog JSONs
  (`v0.11.1-catalog.json`, `docs_reproducible_catalog.json`), loose MANAGER/SESSION-HANDOVER
  transients — full inventory at phase-plan time. **Excludes** the
  SURPRISES-INTAKE/GOOD-TO-HAVES progressive-disclosure bloat split (see Out of Scope).

### Lane 3 — UX: user-facing error hardening to Rust-compiler-grade

*(folds in the existing v0.15 ROADMAP's two UX `Phase TBD` stubs as one lane)*

- [ ] **UX-01**: Bring every user-facing `reposix` CLI subcommand error
  (`crates/reposix-cli/src/`: init/attach/list/sync/doctor/spaces/refresh/etc.) and every
  `reposix-remote` git helper error (`crates/reposix-remote/src/main.rs`,
  `stateless_connect.rs`) to the Rust-compiler-grade three-part standard: (1) teach the
  fix, (2) suggest the alternative, (3) give a copy-paste recovery command — measured
  against the `init.rs::refuse_existing_repo_root` exemplar. Catalog-first: first commit
  writes the `agent-ux` contract rows before any implementation commit.
- [ ] **UX-02**: Ship a structured, stable error-code namespace (`RPX-xxxx`) across the
  same CLI + helper surface, and a `reposix explain <code>` subcommand printing a
  non-empty cause + fix + copy-paste recovery for every code emitted anywhere in the CLI
  or helper — mirroring `rustc --explain E0308`.

### Lane 4 — BENCH: live MCP benchmark re-measurement

- [ ] **BENCH-01**: Re-measure the live token-economy + latency benchmark figures backing
  the 8 hero-number doc-alignment-waived rows, before the **2026-08-15** hard
  waiver-expiry deadline — schedule EARLY in the milestone. Spend ceiling: **≤50**
  benchmark sessions on the existing subscription (owner-confirmed); escalate to the owner
  only past 50. Output unblocks DOCS-05/DOCS-07's honest relabeling and the waived
  `perf/token-economy-bench` + `perf/headline-numbers-cross-check` catalog rows.

### Lane 5 — ADR: ADR-010 mirror-fanout decision packet

- [ ] **ADR-01**: Produce the ADR-010 mirror-fanout decision packet — enumerate options +
  tradeoffs (alongside `docs/decisions/010-l2-l3-cache-coherence.md`) for the owner/manager
  to RULE on. Does **NOT** require implementing the chosen option pre-ruling; the
  deliverable is the packet + a recorded owner ruling (`.planning/CONSULT-DECISIONS.md` or
  equivalent).

### Lane 6 — DRAIN: intake / good-to-have drain (OP-8)

**SURPRISES-INTAKE (4 rows):**

- [ ] **DRAIN-01** *(MED)*: Fix the t4 gate's misleading error — it misattributes
  oid-drift aborts to a git-version problem. `quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh`
  should surface the real stderr (`oid drift … for issue 7766017`) instead of the
  git-version fallback whenever the actual failure is an oid-drift abort.
- [ ] **DRAIN-02** *(MED)*: Document a mandatory mirror-refresh pre-step for the
  `pre-release-real-backend` cadence — run `scripts/refresh-tokenworld-mirror.sh` first (or
  make the litmus self-reconcile, see DRAIN-12/GTH-V15-09), so a second-run vision-litmus
  doesn't false-negative on its own prior push re-staling the GitHub mirror.
- [ ] **DRAIN-03** *(HIGH)*: `quality/runners/run.py` doesn't source `./.env` (unlike
  `scripts/preflight-real-backends.sh`) — a false-green-preflight / silent-skip gap. Make
  `run.py` self-source `.env`, or bake `set -a; . ./.env; set +a` into every documented
  `pre-release-real-backend` invocation; fix-it-twice the doc references in
  `.planning/CLAUDE.md` + `docs/reference/testing-targets.md`.
- [ ] **DRAIN-04** *(HIGH)*: `--persist` silently downgrades a committed-GREEN catalog row
  to a worse status on a skip/false-negative run with no confirm gate — add an
  `--allow-downgrade` opt-in (default refuse) so a false-negative run cannot silently
  corrupt catalog state.

**GOOD-TO-HAVES (17 of 18 rows; GTH-V15-19 is FIX-02 above):**

- [ ] **DRAIN-05** *(MED, GTH-V15-01)*: Concurrent `--persist` runners can race-corrupt the
  shared catalog JSON — advisory `flock` around catalog persist in `run.py`, or serialize
  all persist ops through a single locked lane.
- [ ] **DRAIN-06** *(MED, GTH-V15-03)*: No gate checks a catalog row's `verifier.script`
  exists + is executable — add `quality/gates/structure/verifier-script-exists.sh`.
- [ ] **DRAIN-07** *(HIGH, GTH-V15-04)*: RBF-LR-03's rebase-recovery fix (`bd5b9cb`) is
  proven GREEN only on git 2.25.1 via the `import` path — exercise
  `rebase-recovery-reconciles.sh` on a modern-git (≥2.34) CI runner and resolve whether
  `stateless-connect` exhibits the same fetch-ref-lock behavior.
- [ ] **DRAIN-08** *(MED, GTH-V15-05)*: `resolve_import_parent()`
  (`crates/reposix-remote/src/main.rs:400-419`) degrades to the parentless path on ANY git
  error, not just ref-absence — distinguish ref-absent from spawn/other rev-parse failure,
  error loudly on the latter, add a regression test.
- [ ] **DRAIN-09** *(HIGH, GTH-V15-06)*: Add a binary-side self-safety refusal in
  `reposix init` (not `attach`) so a non-Bash-tool subprocess/worktree bypass targeting the
  shared source tree is refused with a Rust-compiler-grade error, without breaking the
  sanctioned `/tmp` dark-factory flow.
- [ ] **DRAIN-10** *(MED, GTH-V15-07)*: `code/ci-green-on-main`'s phase-close probe
  hardcodes `WORKFLOW=ci.yml` and never watches release-plz — parameterize into a
  required-workflow list (or add a sibling `code/release-green-on-main` row) after
  resolving whether release-plz runs on every push and how a no-op run's conclusion is
  classified.
- [ ] **DRAIN-11** *(MED/hygiene, GTH-V15-08)*: `.planning/ORCHESTRATION.md` is 26968B vs
  its 20000B `structure/file-size-limits` ceiling (waived until 2026-08-08) — split
  reference detail to a sibling doc before the waiver lapses.
- [ ] **DRAIN-12** *(MED→HIGH, GTH-V15-09)*: Make the milestone-close vision-litmus
  fixture self-healing for BOTH backend drift (trashed protected pages) and GitHub mirror
  drift (`reposix sync --reconcile` does NOT push to the mirror) — reconcile the mirror to
  backend-current through the reposix bus remote before the marker push; fix the helper's
  misleading `git pull --rebase` teaching string for the mirror-drift case.
- [ ] **DRAIN-13** *(MED, GTH-V15-10)*: `container-rehearse.sh`'s harness return code and
  its persisted artifact `exit_code` can disagree (rc=0 masking artifact exit_code=1); a
  sim-readiness race between rapid sequential runs produces transient "sim not reachable"
  flakes — derive the harness exit strictly from the persisted `exit_code`; add a
  pre-`docker run` port-7878-free + sim-reachability readiness gate.
- [ ] **DRAIN-14** *(LOW, GTH-V15-11)*: Add a `.sim-*.log` pattern to `.gitignore` scoped
  to `quality/reports/verifications/docs-repro/`.
- [ ] **DRAIN-15** *(LOW-MED, GTH-V15-12)*: The user-global `doc-clarity-review` skill's
  nested `claude -p` returns a confusing non-error (not a hard fail) when it can't see file
  content — add a canary-probe hard-fail or the subscription-caveat note the doc-alignment
  skill already carries (`~/.claude/skills/doc-clarity-review/SKILL.md`, outside this repo).
- [ ] **DRAIN-16** *(LOW, GTH-V15-13)*: Expand "MCP" to "Model Context Protocol (MCP)" on
  first use in `README.md`.
- [ ] **DRAIN-17** *(LOW-MED, GTH-V15-14)*: Pre-push docs-alignment block message names the
  ratio, not the real blocking row-STATE(s) — fix `walk.sh` to name the blocking state(s).
- [ ] **DRAIN-18** *(MED, GTH-V15-15)*: Doc-alignment grader compute-vs-assert reliability
  gap — harden `.claude/skills/reposix-quality-doc-alignment/prompts/grader.md` to only
  bind a row if the test fails when the number drifts, and to grep `src/` unit tests
  rather than just the currently-cited test.
- [ ] **DRAIN-19** *(LOW, GTH-V15-16)*: `plan-refresh <doc>` under-reports drift when
  invoked cold (before a `walk`) — add a one-line note to the playbook/prompt.
- [ ] **DRAIN-20** *(LOW, GTH-V15-17)*: doc-alignment `status` hides that MISSING_TEST rows
  are waived — add a `waived_active` counter to the `status` block.
- [ ] **DRAIN-21** *(LOW, GTH-V15-18)*: Audit the 16 pre-existing "cites
  out-of-eligible-file" coverage warnings — decide whether the eligible-file allowlist
  should include them or the rows should re-cite eligible files.

## Future Requirements (deferred beyond v0.15.0)

Arc D shape (`.planning/PROJECT.md` § Current Milestone): **v0.17** meta-milestone (the
five gate shapes that would have caught the reality-check findings; ALSO the
SURPRISES-INTAKE/GOOD-TO-HAVES progressive-disclosure bloat split, see Out of Scope) →
**v0.19** truth purge + IA rebuild → **v0.21** benchmark honesty → **v0.23** journey
slices → **v0.25** launch kit + Show-HN, with stub milestones (v0.16, v0.18, …)
interleaved draining surprises/good-to-haves as they surface.

## Out of Scope

- **GTH-V15-02** (shell-coverage 12.54% < 13% floor). Its own text states it is v0.14.0
  scope (existing home: phases `999.5 docs-crates-md-zero-coverage` /
  `999.6 docs-alignment-coverage-climb`) — cross-referenced, not re-filed onto v0.15.0.
- **SURPRISES-INTAKE / GOOD-TO-HAVES progressive-disclosure bloat split** (the 2026-07-14
  21:00 SURPRISES entry: `.planning/milestones/v0.14.0-phases/GOOD-TO-HAVES.md` = 27629B,
  `.../v0.15.0-phases/GOOD-TO-HAVES.md` = 23584B, both over the 20000B ceiling). Deferred
  to the **v0.17 meta-milestone bloat-remediation bucket** per Arc D's ratified shape — do
  NOT split early; splitting now would compete with DOCS-08's simplification work using a
  different mechanic (split vs. delete) for the same class of file.
- **Owner-only spend beyond BENCH-01's 50-session ceiling.** Escalation past 50 sessions
  is an owner decision, not in-scope work product of this milestone.

## Traceability

*(Empty — phase decomposition is Step 10 of this `/gsd-new-milestone` run, not yet
executed. The roadmapper fills REQ-ID → Phase → Status below.)*

| REQ-ID | Phase | Status |
|--------|-------|--------|
| *(pending roadmapper)* | | |

**REQ-ID count:** 35 total — FIX (2), DOCS (8), UX (2), BENCH (1), ADR (1), DRAIN (21).
