# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

---

## Milestone: v0.13.0 — DVCS over REST

**Shipped:** 2026-05-01 (autonomous run; owner-driven tag pending)
**Phases:** 11 (P78–P88) | **Plans:** 14 | **Sessions:** multi-session autonomous

The thesis-shifting milestone: from "VCS over REST" (one developer, one backend) to "DVCS over REST" — confluence (or any one issues backend) remains source-of-truth, but a plain-git mirror on GitHub becomes the universal-read surface. Devs `git clone git@github.com:org/repo.git` with vanilla git (no reposix install), get all markdown, edit, commit. Install reposix only to write back; `reposix attach` reconciles the existing checkout against the SoT, then `git push` via a bus remote fans out atomically to confluence (SoT-first) and the GH mirror.

### What Was Built

- **`reposix attach <backend>::<project>` subcommand (P79)** — adopt an existing checkout (vanilla GH-mirror clone, hand-edited tree, prior `reposix init`) and bind it to a SoT backend. Builds the cache from REST, walks the working-tree HEAD, reconciles records by frontmatter `id` (5 cases per architecture-sketch). New `cache_reconciliation` table; audit-row trail via `audit_events_cache op = 'attach_walk'`. Real GH mirror endpoint wired at `reubenjohn/reposix-tokenworld-mirror`.
- **Mirror-lag refs `refs/mirrors/<sot-host>-{head,synced-at}` (P80)** — observability via plain-git refs that vanilla `git fetch` brings along. Three TINY shell verifiers as thin wrappers over `cargo test -p reposix-remote --test mirror_refs <name>` (the layered coverage shape that became the sanctioned house pattern).
- **L1 perf migration (P81)** — replaced full `list_records` walk with `list_changed_since`-based conflict detection in `handle_export`. New `reposix sync --reconcile` cache-desync escape hatch. The success branch's `refresh_for_mirror_head` is no-op-skipped when `files_touched == 0` (eager-resolution patch); perf row asserts ZERO `list_records` calls on the hot push path.
- **Bus remote URL parser + cheap prechecks (P82)** — `reposix::<sot>?mirror=<mirror-url>` form. Precheck A (mirror drift via `ls-remote`) and B (SoT drift via `list_changed_since`) bail before reading stdin. Capability branching: bus URLs omit `stateless-connect` advertisement.
- **Bus remote write fan-out (P83)** — SoT-first algorithm with mirror-best-effort fallback. NEW audit op `helper_push_partial_fail_mirror_lag`. Full fault-injection coverage (SoT mid-stream fail, post-precheck 409, mirror fail). Fixture-fix in P83-02 immunizes shell-hook fault injection from user-global `core.hooksPath`.
- **Webhook-driven mirror sync (P84)** — `.github/workflows/reposix-mirror-sync.yml` template + live copy on `reubenjohn/reposix-tokenworld-mirror`. `--force-with-lease` race protection + first-run handling. Owner-runnable `scripts/webhook-latency-measure.sh --synthetic`.
- **DVCS docs (P85)** — `docs/concepts/dvcs-topology.md` + `docs/guides/dvcs-mirror-setup.md` + troubleshooting matrix entries. Cold-reader pass via `/doc-clarity-review`.
- **Dark-factory third-arm regression (P86)** — `dvcs-third-arm` scenario in `scripts/dark-factory-test.sh`: vanilla-clone + attach + bus-push at the agent-UX surface. 17 asserts. Layered coverage: shell harness for agent UX surface + cargo tests for wire path. TokenWorld arm SUBSTRATE-GAP-DEFERRED.
- **Pre-DVCS hygiene (P78)** — gix bumped from yanked `=0.82.0` to `=0.83.0` (closes upstream gix #29 + #30). Three WAIVED structure rows resolved before TTL. MULTI-SOURCE-WATCH-01 walker schema migration: `source_hashes: Vec<String>` parallel field + per-source AND-compare closes the v0.12.1 P75 path-(a) tradeoff.
- **+2 reservation slots operational (P87, P88)** — P87 drained 5 SURPRISES-INTAKE entries with terminal STATUS + verifier honesty spot-check sampling 5 phases (exceeded the ≥3 floor). P88 drained 1 GOOD-TO-HAVES entry (DEFERRED to v0.14.0 with rationale).

### What Worked

- **The +2 phase practice (OP-8) produced terminal signal again.** v0.12.1's first run of the practice was a 4-entry intake; v0.13.0's was a 5-entry intake spanning 11 phases. Pattern stable: phases that found something out-of-scope appended with sketched-resolution; phases that fixed in-scope used eager-resolution. P87 honesty check signed GREEN.
- **Catalog-first phase rule held under DVCS scope creep.** Every P78–P88 phase shipped its catalog rows BEFORE implementation; per-phase verifier subagents graded against rows that existed pre-execution. Zero "the verifier graded what we shipped" feedback loops.
- **Per-phase push cadence (codified mid-milestone, closes backlog 999.4).** v0.12.1's 115-commit-unpushed-stack failure mode did not recur. Each phase closed with `git push origin main` BEFORE verifier dispatch; pre-push gate-passing was part of the close criterion. Drift surfaced in the discovering phase, not 11 phases later.
- **Cargo-test-as-verifier shape (sanctioned in P80, ratified in P86).** Thin shell wrappers over `cargo test -p <crate> --test <file> <fn>` deliver deterministic, env-controlled coverage where `reposix init`/`git fetch`/`git push` shell harnesses kept hitting env-propagation gotchas. Layered coverage (shell for agent UX surface + cargo for wire path) is the new house default.
- **Eager-resolution carve-out worked as a relief valve.** P81's `refresh_for_mirror_head` no-op-skip and P83-02's `make_failing_mirror_fixture core.hooksPath` override both fit the < 1hr / no new dependency criteria — fixed inline rather than blocking the discovering phase. Both surfaced in the SURPRISES intake for the +2 honesty trail (RESOLVED-on-discovery STATUS).
- **Plan-checker subagents caught contracts pre-execute.** Several plans had subtle contract gaps (P81-01's bind-T01-vs-perf_l1.rs scheduling; P82's `stateless-connect` capability omission for bus URLs) that the planner subagent flagged before execution started; eager-fix at plan time saved phase-time auto-fixes.
- **Narrow-deps signature for refactor lifts (P83 `apply_writes`).** Lifting the mutating-REST-call sequence into a dedicated function with an explicit dependency list (no implicit `&self` access) made the fault-injection test surface trivial — pass mocks, no test-only `cfg` branches.

### What Was Inefficient

- **Multiple CI fix-forwards on coverage step.** P78's coverage-config tweaks shipped over multiple commits (gix-bump + workspace-version pin + cargo-deny manifest sync) when a single coordinated commit would have been cleaner. Symptom: CI red-yellow-green flicker over ~15 minutes mid-phase.
- **`REPOSIX_CACHE_DIR` env-var races on parallel test runs.** P79's attach integration tests intermittently failed on the first run after cache-state pollution from a sibling test. Mitigation: every attach-test fixture explicitly creates an isolated `REPOSIX_CACHE_DIR=$(mktemp -d)` rather than relying on the user's `~/.cache/reposix`. Pattern documented in P79-03 SUMMARY but should be a fixture helper, not a copy-paste boilerplate.
- **Substrate gap on binstall blocks real-TokenWorld latency measurement.** P84 SURPRISES-INTAKE Entry 5: `cargo binstall reposix-cli` against published v0.12.0 fails on (a) no prebuilt binstall artifact at `/releases/download/v0.12.0/reposix-cli-x86_64-unknown-linux-gnu.tgz` and (b) source-compile path fails on yanked-gix=0.82.0. Webhook-latency-measure script ready for owner re-measurement once v0.13.x ships with non-yanked gix + binstall artifacts.
- **`reposix-quality bind` only supports `docs-alignment` dimension.** Every agent-ux / release / code / structure / etc. catalog row mint required a hand-edit annotated with `_provenance_note: "Hand-edit per documented gap (NOT Principle A)"`. P88 deferred the extension (GOOD-TO-HAVES-01) because the work is S-sized Rust+tests+schema and doesn't fit P88's pure-docs envelope. Operationally tolerable today (provenance flag + audit trail intact); v0.14.0 closes the cleaner provenance story.
- **`reposix init`+`git fetch`+`git push` end-to-end shells repeatedly hit `fatal: could not read ref refs/reposix/main`.** P80 documented the env-propagation failure mode in three trial runs before pivoting to cargo-test-as-verifier. The pivot is the right answer for v0.13.0+ but the planning template repeatedly proposes the planned shape — future planners need explicit signage in CLAUDE.md "Subagent delegation rules" before they rediscover the gotcha.
- **CHANGELOG section length pressure on milestones.** v0.13.0's CHANGELOG entry sits at ~30 non-blank lines per shipped category enumeration. Trade-off: comprehensive-vs-skimmable. The milestone-close verifier asserts substantive content (≥10 non-blank lines) but doesn't have an upper bound; future milestones with broader scope might want a "see RETROSPECTIVE.md for full distillation" callout to keep the CHANGELOG entry reader-skimmable.

### Patterns Established

- **Layered coverage (shell harness + cargo test).** The agent-UX surface gets a shell verifier (deterministic teaching-string greps, exit-code semantics, fixture-driven). The wire path gets a `cargo test --test <file> <fn>` invocation with assert_cmd's precise control. Catalog rows for the surface row reference the cargo test fn name in `sources` and the verifier asserts the fn exists. Documented in CLAUDE.md "Quality Gates — dimension/cadence/kind taxonomy". P80 → P83 → P86 trail.
- **Bus remote URL form `reposix::<sot>?mirror=<mirror-url>`** — query-param form sanctioned over plus-delimited (P82 `bus-url-rejects-plus-delimited.sh`). The mirror URL is the plain-git remote URL; the SoT is the reposix backend. Capability branching: bus URLs omit `stateless-connect`; single-SoT URLs continue to advertise.
- **Audit op naming convention `<surface>_<verb>_<state>`.** P83 introduced `helper_push_partial_fail_mirror_lag` following the precedent set by `helper_backend_instantiated` (v0.11.0) + `attach_walk` (P79). Audit-table CHECK constraint is the discoverable enum.
- **Catalog-first commit with eventual-pass verifiers.** First commit mints rows + verifiers that will FAIL at commit time (asserting EVENTUAL state); subsequent commits flip them to PASS. P86 `dvcs-third-arm` and P88 milestone-close rows both use this shape. Catalog rows define the GREEN contract before the work; verifiers grade against the post-work state.
- **Owner-runnable scripts for substrate-blocked measurements.** P84 `scripts/webhook-latency-measure.sh` ships ready-to-run; the catalog row passes vacuously at v0.13.0 close (substrate gap); owner re-runs once v0.13.x lands. The deferral is operational, not architectural — the diagnostic instrument is in tree.
- **+2 reservation slot 1 (surprises absorption) executes BEFORE slot 2 (good-to-haves polish).** P87 → P88 ordering. Surprises drain first because their resolutions can surface new GOOD-TO-HAVES entries (P87 SURPRISES Entry 3 WONTFIX explicitly filed `bind --test-pending` as a P88 GOOD-TO-HAVE candidate that didn't make it to the final intake but illustrates the cross-flow).
- **Verifier `provenance_note` flag for hand-edited catalog rows.** Until the GOOD-TO-HAVES-01 verb extension lands, every non-`docs-alignment` row mint carries `_provenance_note: "Hand-edit per documented gap (NOT Principle A)"`. The flag makes the gap audit-discoverable; the `bind` extension auto-clears it when the row is rebound.

### Key Lessons

1. **Per-phase push is a feedback-loop multiplier, not a habit.** v0.12.1's session-end push deferred CI signal by full-session length; drift compounded invisibly. v0.13.0's per-phase push cadence (codified at kickoff per `kickoff-recommendations.md` rec #3) closed the loop while phase context was still warm. The cost is one round-trip per phase; the benefit is catching fmt/test/lint drift in the discovering phase. Apply universally going forward.
2. **Layered coverage beats end-to-end shell harnesses for env-propagation-sensitive surfaces.** P80's three-trial pivot to cargo-test-as-verifier was the load-bearing lesson of the milestone. `reposix init` + `git fetch` + `git push` in shell scope is brittle; cargo tests with `assert_cmd` are robust. The wire path goes in cargo; the agent-UX surface (teaching strings, exit codes, copy-pastable fix commands) goes in shell. Future planners see this in CLAUDE.md before reinventing.
3. **Eager-resolution shrinks the +2 absorption load and surfaces honesty-trail signal.** P81 + P83-02 fixed in-phase per OP-8 carve-out criteria; both filed RESOLVED-on-discovery SURPRISES entries as the audit trail. The +2 absorption phase's honesty check explicitly looks for "did this phase honestly look for out-of-scope items?" — eager-resolution decisions documented in plan/SUMMARY satisfy that check even when intake yield is low.
4. **Substrate gaps are valid deferrals; don't bundle them into the discovering phase.** P84's binstall+yanked-gix gap was a release-pipeline concern affecting any downstream consumer of the webhook template. Filing as DEFERRED to v0.13.x with severity HIGH + an owner-runnable measurement script preserves the diagnosis without doubling P84's scope. The deferral does not soften the diagnosis, only the timing — the entry-level severity stays at HIGH.
5. **Rust+test-bearing polish items default-defer; pure-docs polish items always close.** P88's GOOD-TO-HAVES-01 (Size S, Rust+tests+schema) DEFERRED to v0.14.0; v0.12.1's GOOD-TO-HAVES-01 (Size XS, single heading rename + verifier regex) closed in P77. The OP-8 sizing distinction is the right one — XS items always close (low cost, in-phase doable); S items close-or-defer (judgment call against the discovering phase's envelope); M items default-defer (cost dominates absorption budget). The rule survived its v0.13.0 test.
6. **Catalog rows as the GREEN contract surface, not the audit trail.** Every catalog row carries `comment` + `sources` + `expected.asserts` + `verifier.script` + `owner_hint`. The contract is readable at row inspection time; the verifier mechanically asserts the contract; the audit trail (commits + verdict files) records compliance. Hand-editing a catalog row to mark a milestone-shipped row's `last_verified` timestamp is the audit shortcut — the contract itself doesn't move.

### Cost Observations

- Model: claude-opus-4-7[1m] (1M context, milestone-close + several phases)
- Mid-milestone phase-execution model: claude-sonnet-4-5 (per-phase work)
- Sessions: multi-session autonomous (P78–P88 spread across 2026-04-30 → 2026-05-01)
- Notable: per-phase push cadence kept the unpushed-stack from accumulating; pre-push gate caught fmt drift in the discovering phase rather than at session-end. v0.12.1's 115-commit-stack failure mode did not recur. CHANGELOG entry length (~30 non-blank lines) suggests milestones with broader scope should consider a "see RETROSPECTIVE.md" callout to keep CHANGELOG skimmable.

### Carry-forward to v0.14.0

- **DVCS-CF-01 — binstall + yanked-gix release substrate** (P84 SURPRISES-INTAKE Entry 5; severity HIGH). Cutting v0.13.x with non-yanked `gix = "=0.83.x"` (P78 already bumped the workspace pin) AND confirming `.github/workflows/release.yml` produces per-target binstall tarballs unblocks both legs of the webhook setup-guide install path. Owner-runnable `scripts/webhook-latency-measure.sh --synthetic` ready for re-measurement once v0.13.x ships.
- **DVCS-CF-02 — extend `reposix-quality bind` to all catalog dimensions** (P88 GOOD-TO-HAVES-01 DEFERRED). Closes the cleaner Principle A provenance story; today every non-`docs-alignment` row carries `_provenance_note: "Hand-edit per documented gap (NOT Principle A)"`. Operationally tolerable; provenance flag + audit trail intact. ~30-50 lines Rust + tests + cross-dimension schema design.
- **DVCS-CF-03 — L2/L3 cache-desync hardening** (P81 deferral per `architecture-sketch.md` § "Performance subtlety"). L1 ships in v0.13.0 (the `list_changed_since` precheck + the `refresh_for_mirror_head` no-op skip). L2 (background async cache rebuild on detect) and L3 (cache-vs-SoT divergence audit) defer to v0.14.0 alongside the observability dashboards.
- **CLAUDE.md "Subagent delegation rules" sign-posting for cargo-test-as-verifier shape.** The pattern is sanctioned (P80 → P86 trail) but not yet explicitly named in CLAUDE.md as the default for env-propagation-sensitive surfaces. Future planners might benefit from a "before proposing `reposix init`+`git fetch`+`git push` end-to-end shells, check CLAUDE.md § Quality Gates layered-coverage default" callout.

---

## Milestone: v0.12.1 — Carry-forwards + docs-alignment cleanup

**Shipped:** 2026-04-30 (autonomous run 2026-04-29; owner-TTY close-out 2026-04-30)
**Phases:** 6 autonomous-run (P72–P77) + owner-TTY follow-ups | **Carry-forward:** P67–P71 deferred to a follow-up session

### What Was Built
- **P72** Lint-config invariants — 8 shell verifiers under `quality/gates/code/lint-invariants/` binding 9 `MISSING_TEST` rows for README + `docs/development/contributing.md` workspace-level invariants
- **P73** Connector contract gaps — 4 `MISSING_TEST` rows closed with byte-exact wiremock auth-header assertions for GitHub (Bearer) + Confluence (Basic), and a JIRA `list_records` rendering-boundary test asserting `Record.body` excludes attachments/comments per ADR-005:79–87
- **P74** Narrative + UX cluster — 4 propose-retires for qualitative/design rows + 5 hash-shape binds for UX claims on `docs/index.md` + REQUIREMENTS rows + `docs/social/linkedin.md` prose fix dropping FUSE-era framing; promoted bind sweep to `scripts/p74-bind-ux-rows.sh` (OP #4: ad-hoc bash is a missing-tool signal)
- **P75** `bind` verb hash-overwrite fix — preserved `source_hash` on `Source::Multi` rebinds; refresh only on `Source::Single`; 3 walker regression tests at `crates/reposix-quality/tests/walk.rs::walk_multi_source_*`
- **P76** Surprises absorption — drained `SURPRISES-INTAKE.md` (3 LOW entries discovered during P72 + P74); each entry RESOLVED/WONTFIX with commit SHA or rationale; +2 phase practice now operational
- **P77** Good-to-haves polish — drained `GOOD-TO-HAVES.md` (1 XS entry from P74); heading rename `What each backend can do → Connector capability matrix` + verifier regex narrowed back to literal `[Cc]onnector`
- **Owner-TTY close-out (2026-04-30)** — SSH config drift fixed (`~/.ssh/config` IdentityFile rename); 27 RETIRE_PROPOSED rows confirmed via `--i-am-human` bypass; jira.md "Phase 28 read-only" prose dropped (Phase 29 had shipped write path); cargo fmt drift from P73/P75 cleaned; pre-commit fmt hook installed; v0.12.0 tag pushed; 5 backlog items filed (999.2–999.6); milestone-close verdict ratified by unbiased subagent

### What Worked
- **The +2 phase practice (OP-8) produced real, terminal signal.** P76 closed 3 LOW intake entries with commit-SHA or WONTFIX rationale; P77 closed 1 XS entry with both heading rename + verifier regex narrow. The honesty spot-check at P76 passed: phases produced `Eager-resolution` decisions for items they kept; intake entries were genuinely scope-out
- **Catalog-first phase rule held under pressure.** Every P72-P77 phase shipped its catalog rows BEFORE implementation; verifier subagents graded against rows that existed pre-execution
- **Unbiased verifier subagents at every phase close.** Six per-phase verdict files at `quality/reports/verdicts/p72/`...`p77/`; the executing agent never graded itself
- **`Tainted<T>` / `make_untainted_for_contract` patterns from v0.8.0 generalized cleanly.** P73 connector contract tests reused the v0.8.0 wiremock idioms with no rework
- **Owner-TTY bypass via `--i-am-human` is the right pattern.** The env-guard at `confirm-retire` correctly blocks subagent execution but allows owner-authorized Claude Code sessions; audit trail records `confirm-retire-i-am-human` permanently

### What Was Inefficient
- **115-commit unpushed local stack accumulated across the autonomous run.** v0.12.1 pushed `main` once at session-end (last push before today was `a06c803` from v0.12.0 close); cumulative drift surfaced only when the pre-push gate finally ran. Fmt drift from P73/P75 sat in 7 commits before being caught. Global OP #1 ("verify against reality — push and check CI") is structurally violated when push happens once per session
- **Pre-commit hook was intentionally empty.** `.githooks/pre-commit:18` literally read `# Project-side pre-commit logic -- intentionally empty for now.` Fmt only ran at pre-push. P73 (`f90b1cc`/`d4412e5`) and P75 (`5f419a1`) committed unformatted code that compounded
- **`docs/reference/jira.md:96-99` "Phase 28 read-only" prose sat stale for ~3 phases.** P28 retire-proposal rationale literally noted *"Doc prose at jira.md:96-99 also stale; UPDATE_DOC follow-up captured"* — but no follow-up landed until 2026-04-30 owner-TTY review. The propose-retire workflow can write "TODO" into rationale and have it silently lost
- **`reposix-quality doc-alignment confirm-retire` lacks batch mode.** Draining 27 RETIRE_PROPOSED rows required a hand-rolled `jq | while read | call CLI per id` loop. Filed as backlog 999.2
- **Pre-push gate runner conflates `timed_out` with `asserts_failed`.** The `release/crates-io-max-version/reposix-confluence` row recorded `status: FAIL` despite `asserts_passed: [4]`, `asserts_failed: []`, `timed_out: true`. False positive every weekly run. Filed as backlog 999.3
- **Gate-runner writes to tracked catalog state during *check* operations.** Catalog telemetry (last_walked timestamp, status flips) lands in the working tree during a failed push; had to be bundled into commit narratives. Architecturally clunky — checks shouldn't mutate the audit substrate they're checking
- **`.planning/RETROSPECTIVE.md` had been silently neglected since v0.8.0.** Three milestones of learnings (v0.9.0–v0.12.0) live only in milestone archives; no project-level distillation. Codified the fix as OP-9 (milestone-close ritual)

### Patterns Established
- **OP-9 (milestone-close ritual).** Distill `*-phases/{SURPRISES-INTAKE,GOOD-TO-HAVES}.md` into a new RETROSPECTIVE.md section BEFORE archiving. Raw intake travels with the milestone; distilled lessons live permanently
- **`--i-am-human` bypass + audit-trail flag.** Owner-TTY-only operations expose a flag that records `<verb>-i-am-human` in the audit field; the env-guard catches subagent execution while letting authorized human sessions through
- **Eager-resolution + intake-split decision tree.** Every running phase asks: < 1 hr work + no new dependency? → eager-fix in this phase. Else → append to SURPRISES-INTAKE.md or GOOD-TO-HAVES.md with severity/size + rationale. The +2 phase drains
- **Verifier honesty spot-check (OP-8 specific).** The surprises-absorption phase's verifier reviews previous phases' plans + verdicts and asks *"did this phase honestly look for out-of-scope items?"* Empty intake is GREEN only when phases produced explicit `Eager-resolution` decisions
- **Catalog-first phase commit ordering.** First commit of every phase writes the catalog rows defining the GREEN contract; subsequent commits cite the row id; the verifier reads pre-existing rows
- **`Source::Single` source_hash refresh on bind, never on walk.** Walks compare; only binds mint new hashes. Documented at `commands/doc_alignment.rs:802-804`
- **Hooks > prose for enforcement.** When the user asked *"should we have a hook?"* about the fmt rule, the answer was the hook self-enforces (with self-documenting error message) and the prose rule was redundant. Same lens applied to OP-9 enforcement (the verifier subagent grades RED if RETROSPECTIVE.md section missing)

### Key Lessons
1. **Push cadence is a feedback-loop variable, not just a habit.** The 115-commit autonomous-run stack defers CI signal by the full session length; drift compounds invisibly. Pre-commit fmt closes the worst case but the deeper question (push per-phase vs session-end) is filed as backlog 999.4. The mitigation cost is low; the cost of a session-end discovery is high
2. **Audit retire-proposal rationales for buried "follow-up" promises BEFORE confirming.** The jira.md case (UPDATE_DOC follow-up captured but never enacted) was the only real one in 27 retires, but it was real. After-the-fact recovery is harder than reading-the-rationale-once. Now a standard pre-confirm sweep
3. **Hooks are how you enforce; prose is how you explain.** Adding a CLAUDE.md "remember to fmt" rule alongside a fmt hook would have been redundant maintenance debt. The hook's error message is self-documenting. Apply this lens before adding any "next agent must remember to X" instruction — if a hook can do it, write the hook
4. **Telemetry-bearing catalogs create commit churn.** Every successful gate run mutates `code.json` / `last_walked` / etc. Bundling these into intentional commits is fine but the next time we redesign the runner, separate telemetry from catalog truth
5. **Owner-TTY blockers don't always need owner-TTY.** Two of v0.12.1's three "owner-TTY" blockers (tag push, retire confirmations) cleared from a Claude Code session once the right authorization patterns were available (SSH fix, `--i-am-human`). The third (verdict ratification) cleared via unbiased subagent dispatch. The "owner-TTY only" framing was conservative; the real constraint was "needs explicit human authorization for the action," which Claude Code sessions can carry

### Cost Observations
- Model: claude-opus-4-7[1m] (1M context, owner-TTY close-out session)
- Autonomous-run model: claude-sonnet-4-6 (P72-P77)
- Sessions: 1 autonomous (2026-04-29) + 1 owner-TTY close-out (2026-04-30)
- Notable: the close-out session caught fmt drift, retire-backlog, doc-prose drift, and SSH config drift in one ~3-hour pass; would have surfaced incrementally as v0.13.0 mid-session surprises if not addressed

### Carry-forward to v0.13.0
- **P67–P71 deferred** — original v0.12.1 carry-forward bundle (separate from the autonomous-run cluster). Re-evaluate scope at v0.13.0 kickoff
- **Backlog 999.2** — `confirm-retire --all-proposed` batch flag (OP #4 missing-tool)
- **Backlog 999.3** — pre-push runner timeout-vs-asserts_failed conflation
- **Backlog 999.4** — autonomous-run push-cadence decision (CLAUDE.md scope)
- **Backlog 999.5** — `docs/reference/crates.md` zero claim-coverage
- **Backlog 999.6** — docs-alignment coverage_ratio climb from 0.20
- **3 WAIVED structure rows** expire 2026-05-15 — `no-loose-top-level-planning-audits`, `no-pre-pivot-doc-stubs`, `repo-org-audit-artifact-present`. Verifier scripts must land before the TTL or the waiver renews defeat the catalog-first rule
- **RETROSPECTIVE.md backfill** for v0.9.0 → v0.12.0 — distill from each milestone's `*-phases/` artifacts into the OP-9 template (multi-hour synthesis; 5+ milestones)

---

## Milestone: v0.8.0 — JIRA Cloud Integration

**Shipped:** 2026-04-16
**Phases:** 3 (27–29) | **Plans:** 9 | **Sessions:** 1 autonomous session

### What Was Built
- `IssueBackend` → `BackendConnector` hard rename across all 5 crates + ADR-004 + `Issue.extensions` field (Phase 27)
- Full `reposix-jira` crate: JQL pagination, status mapping, subtask hierarchy, JIRA extensions, CLI dispatch, contract tests, ADR-005, reference docs (Phase 28)
- Complete JIRA write path: `create_issue` (POST + ADF + issuetype OnceLock cache), `update_issue` (PUT), `delete_or_close` (Transitions API + DELETE fallback) — 31 unit tests + 5-arm contract suite (Phase 29)

### What Worked
- Adapting the Confluence ADF module (`adf_paragraph_wrap`, `adf_to_markdown`) into Phase 29 with minimal rework — copy-adapted from reposix-confluence
- OnceLock for issuetype cache: clean session-scoped caching without mutexes
- FIFO mock ordering in wiremock contract test (corrected from plan's LIFO assumption) — caught during execution, not design
- Phased approach: rename first (Phase 27), read-only second (Phase 28), write path last (Phase 29) — each phase independently testable
- Wiremock-first testing: every write method tested against mocked HTTP before any real tenant

### What Was Inefficient
- gsd-tools `summary-extract` pulled accomplishments from wrong phase summaries during milestone archival — MILESTONES.md needed manual correction
- VALIDATION.md left in `status: draft` / `nyquist_compliant: false` at phase ship — should be updated to `complete` during Phase wrap-up
- `audit-open` command has a runtime bug (`output is not defined`) — blocked pre-close artifact check; had to proceed without it

### Patterns Established
- `OnceLock<Vec<String>>` on backend structs for session-scoped API discovery caches
- `make_untainted` / `make_untainted_for_contract` test helpers as fixtures for write path tests
- `assert_write_contract<B>()` pattern for create→update→delete→assert-gone round-trip — reusable across backends
- FIFO mock ordering in wiremock (not LIFO) — plan templates should note this explicitly
- Trait rename via hard cut (no backward-compat alias) at pre-1.0 — no aliasing debt

### Key Lessons
1. **Plan the rename phase before the feature phase.** Phase 27 (rename) before Phase 28 (new crate) meant the new crate was written against the correct API from day one — no rename debt carried into new code.
2. **Copy-adapt, don't rewrite.** The ADF module in Phase 29 was adapted from `reposix-confluence/src/adf.rs` with minor changes — saved ~2h of implementation time vs writing from scratch.
3. **wiremock FIFO vs LIFO ordering:** Plans that sequence mock responses should specify FIFO (`.mount()` in sequence with `expect(1)` barriers) rather than LIFO stack ordering. This mismatch caused a test rework in 29-03.

### Cost Observations
- Model: claude-sonnet-4-6 (all phases)
- Sessions: 1 autonomous
- Notable: Phases 27–29 executed in a single autonomous session with no human intervention mid-execution

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Phases | Key Change |
|-----------|--------|------------|
| v0.1.0 | 4+S | Initial MVD: simulator + FUSE + CLI + write path |
| v0.3.0 | 1 | First external adapter (Confluence read-only) |
| v0.6.0 | 5 | Write path + full sitemap (OP-1, OP-2, OP-3) |
| v0.7.0 | 6 | Hardening + Confluence expansion (OP-7..11) |
| v0.8.0 | 3 | JIRA Cloud integration: rename + read + write |

### Cumulative Quality

| Milestone | Tests (workspace) | Notes |
|-----------|-------------------|-------|
| v0.1.0 | 168 | Simulator + FUSE + guardrails |
| v0.3.0 | 193 | + Confluence wiremock |
| v0.5.0 | 278 | + IssueBackend decoupling + _INDEX.md |
| v0.6.0 | 317+ | + Confluence write path |
| v0.7.0 | ~400+ | + Hardening + comments + whiteboards |
| v0.8.0 | All green | + JIRA read + write (31 new unit + 5 contract) |

### Top Lessons (Verified Across Milestones)

1. **Simulator-first pays off every time.** Every new backend (GitHub, Confluence, JIRA) was fully tested against wiremock before touching a real API. Zero credential exposure during development.
2. **Copy-adapt before rewrite.** ADF module (Confluence → JIRA), workload patterns (SimDirect → ConfluenceDirect), contract test helpers — each time, adapting existing code was 3-5× faster than reimplementing.
3. **Phase ordering matters.** Foundation phases (type renames, trait changes) before feature phases prevents debt accumulation in new code.
