# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

Per CLAUDE.md OP-9 ("Milestone-close ritual"), this file holds *cross-milestone
distillation* — failure modes, patterns, process gaps. Per-milestone narrative
(What Was Built, Cost Observations, verbose Carry-forward) lives in
`.planning/milestones/<v>-phases/RETROSPECTIVE-FULL.md` next to each milestone's
SURPRISES-INTAKE / GOOD-TO-HAVES / CARRY-FORWARD files. Recurring patterns
that span 2+ milestones are listed once in "Cross-Milestone Trends" rather
than repeated under each milestone.

---

## Milestone: v0.13.0 — DVCS over REST

**Shipped:** 2026-05-01 (autonomous; owner-driven tag pending) | **Phases:** 11 (P78–P88) | **Plans:** 14
**Full narrative:** `.planning/milestones/v0.13.0-phases/RETROSPECTIVE-FULL.md`

The thesis-shifting milestone: from "VCS over REST" (one developer, one backend) to "DVCS over REST" — confluence (or any one issues backend) remains source-of-truth, but a plain-git mirror on GitHub becomes the universal-read surface. Devs `git clone` with vanilla git, edit, commit. Install reposix only to write back; `reposix attach` reconciles the existing checkout against the SoT, then `git push` via a bus remote fans out atomically to confluence (SoT-first) and the GH mirror.

Headline shipped surfaces: `reposix attach` (P79), mirror-lag refs (P80), L1 perf migration with `list_changed_since` precheck (P81), bus remote URL parser + cheap prechecks (P82), bus remote write fan-out with SoT-first algorithm (P83), webhook-driven mirror sync (P84), DVCS docs (P85), dark-factory third-arm regression (P86), pre-DVCS hygiene incl. gix bump + walker schema migration (P78), +2 reservation slots P87/P88.

### Milestone-specific lessons

(Cross-cutting lessons — per-phase push, layered coverage, catalog-first, +2 phase practice — live in **Cross-Milestone Trends** below; these are v0.13.0-only signals.)

1. **Cargo-test-as-verifier shape pivots away from end-to-end shell harnesses for env-propagation-sensitive surfaces.** P80 documented `reposix init`+`git fetch`+`git push` in shell scope hitting `fatal: could not read ref refs/reposix/main` in three trial runs before pivoting. The wire path goes in cargo (`assert_cmd` precise control); the agent-UX surface (teaching strings, exit codes, copy-pastable fix commands) stays in shell. Future planners need explicit signage in CLAUDE.md "Subagent delegation rules" before they rediscover the gotcha.
2. **Substrate gaps are valid deferrals; don't bundle them into the discovering phase.** P84's binstall+yanked-gix gap was a release-pipeline concern affecting any downstream consumer of the webhook template. Filing as DEFERRED to v0.13.x with severity HIGH + an owner-runnable measurement script (`scripts/webhook-latency-measure.sh --synthetic`) preserves the diagnosis without doubling P84's scope. Deferral does not soften the diagnosis, only the timing.
3. **Rust+test-bearing polish items default-defer; pure-docs polish items always close.** P88's GOOD-TO-HAVES-01 (Size S, Rust+tests+schema) DEFERRED to v0.14.0; v0.12.1's GOOD-TO-HAVES-01 (Size XS, single heading rename + verifier regex) closed in P77. XS always closes; S close-or-defer (judgment call); M default-defer (cost dominates absorption budget). Rule survived its v0.13.0 test.
4. **Narrow-deps signature for refactor lifts.** P83 lifted `apply_writes` (mutating-REST-call sequence) into a dedicated function with explicit dependency list (no implicit `&self` access). Made the fault-injection test surface trivial — pass mocks, no test-only `cfg` branches.
5. **Eager-resolution carve-out as relief valve.** P81's `refresh_for_mirror_head` no-op-skip and P83-02's `make_failing_mirror_fixture core.hooksPath` override both fit the < 1hr / no new dependency criteria — fixed inline rather than blocking the discovering phase. Both filed RESOLVED-on-discovery in SURPRISES intake for the +2 honesty trail.

### v0.13.0-specific patterns

- **Bus remote URL form `reposix::<sot>?mirror=<mirror-url>`** — query-param form sanctioned over plus-delimited (P82 `bus-url-rejects-plus-delimited.sh`). Capability branching: bus URLs omit `stateless-connect`.
- **Audit op naming convention `<surface>_<verb>_<state>`.** P83 `helper_push_partial_fail_mirror_lag` follows precedent set by `helper_backend_instantiated` (v0.11.0) + `attach_walk` (P79). Audit-table CHECK constraint is the discoverable enum.
- **Owner-runnable scripts for substrate-blocked measurements.** P84 `scripts/webhook-latency-measure.sh` ships ready-to-run; catalog row passes vacuously at v0.13.0 close (substrate gap); owner re-runs once v0.13.x lands. Diagnostic instrument is in tree.
- **Verifier `provenance_note` flag for hand-edited catalog rows.** Until GOOD-TO-HAVES-01 verb extension lands, every non-`docs-alignment` row mint carries `_provenance_note: "Hand-edit per documented gap (NOT Principle A)"`. Flag makes the gap audit-discoverable.

### Inefficiencies (v0.13.0-specific)

- **Multiple CI fix-forwards on coverage step.** P78's tweaks shipped over multiple commits (gix-bump + workspace-version pin + cargo-deny manifest sync) when one coordinated commit would have been cleaner. Symptom: CI red-yellow-green flicker over ~15 minutes mid-phase.
- **`REPOSIX_CACHE_DIR` env-var races on parallel test runs.** P79's attach integration tests intermittently failed on first run after cache-state pollution. Mitigation: every fixture explicitly creates `REPOSIX_CACHE_DIR=$(mktemp -d)`. Should be a fixture helper, not copy-paste boilerplate.
- **`reposix-quality bind` only supports `docs-alignment` dimension.** Every agent-ux / release / code / structure / etc. catalog row mint required hand-edit + `_provenance_note`. P88 deferred extension (GOOD-TO-HAVES-01) — work is S-sized Rust+tests+schema. Operationally tolerable; v0.14.0 closes the cleaner provenance story.
- **CHANGELOG entry length pressure.** v0.13.0's CHANGELOG entry sits at ~30 non-blank lines per shipped category enumeration. Future broader-scope milestones might want a "see RETROSPECTIVE.md for full distillation" callout to keep CHANGELOG skimmable.

### Carry-forward to v0.14.0

Canonical list: `.planning/milestones/v0.13.0-phases/CARRY-FORWARD.md`. Headlines: DVCS-CF-01 binstall + yanked-gix release substrate (severity HIGH); DVCS-CF-02 extend `reposix-quality bind` to all catalog dimensions; DVCS-CF-03 L2/L3 cache-desync hardening; CLAUDE.md sign-posting for cargo-test-as-verifier shape.

---

## Milestone: v0.12.1 — Carry-forwards + docs-alignment cleanup

**Shipped:** 2026-04-30 (autonomous run 2026-04-29; owner-TTY close-out 2026-04-30)
**Phases:** 6 autonomous-run (P72–P77) + owner-TTY follow-ups
**Full narrative:** `.planning/milestones/v0.12.1-phases/RETROSPECTIVE-FULL.md`

Headlines: P72 lint-config invariants (8 shell verifiers), P73 connector contract gaps (byte-exact wiremock auth-header assertions), P74 narrative+UX cluster + linkedin FUSE-era prose fix, P75 `bind` verb hash-overwrite fix, P76 surprises absorption (3 LOW), P77 good-to-haves polish (1 XS), owner-TTY close-out (SSH config drift, 27 RETIRE_PROPOSED rows confirmed via `--i-am-human`, fmt drift cleanup, pre-commit fmt hook, v0.12.0 tag).

### Milestone-specific lessons

1. **Push cadence is a feedback-loop variable, not just a habit.** The 115-commit autonomous-run stack defers CI signal by the full session length; drift compounds invisibly. Pre-commit fmt closes the worst case but the deeper question (push per-phase vs session-end) was filed as backlog 999.4 and resolved by v0.13.0's per-phase-push codification.
2. **Audit retire-proposal rationales for buried "follow-up" promises BEFORE confirming.** The jira.md case (UPDATE_DOC follow-up captured but never enacted) was the only real one in 27 retires — but it was real. After-the-fact recovery is harder than reading-the-rationale-once. Now a standard pre-confirm sweep.
3. **Hooks > prose for enforcement.** When the user asked *"should we have a hook?"* about the fmt rule, the answer was the hook self-enforces (with self-documenting error message) and the prose rule was redundant. Apply this lens before adding any "next agent must remember to X" instruction — if a hook can do it, write the hook.
4. **Telemetry-bearing catalogs create commit churn.** Every successful gate run mutates `code.json` / `last_walked` / etc. Bundling these into intentional commits is fine but the next time we redesign the runner, separate telemetry from catalog truth.
5. **Owner-TTY blockers don't always need owner-TTY.** Two of v0.12.1's three "owner-TTY" blockers (tag push, retire confirmations) cleared from a Claude Code session once the right authorization patterns were available (SSH fix, `--i-am-human`). The third (verdict ratification) cleared via unbiased subagent dispatch. The "owner-TTY only" framing was conservative; the real constraint was "needs explicit human authorization for the action," which Claude Code sessions can carry.

### v0.12.1-specific patterns

- **OP-9 (milestone-close ritual).** Distill `*-phases/{SURPRISES-INTAKE,GOOD-TO-HAVES}.md` into a new RETROSPECTIVE.md section BEFORE archiving. Raw intake travels with the milestone; distilled lessons live permanently.
- **`--i-am-human` bypass + audit-trail flag.** Owner-TTY-only operations expose a flag that records `<verb>-i-am-human` in the audit field; env-guard catches subagent execution while letting authorized human sessions through.
- **Eager-resolution + intake-split decision tree.** Every running phase asks: < 1 hr work + no new dependency? → eager-fix. Else → append to SURPRISES-INTAKE.md or GOOD-TO-HAVES.md with severity/size + rationale. The +2 phase drains.
- **Verifier honesty spot-check (OP-8).** Surprises-absorption phase's verifier reviews previous phases' plans + verdicts and asks *"did this phase honestly look for out-of-scope items?"* Empty intake is GREEN only when phases produced explicit `Eager-resolution` decisions.
- **`Source::Single` source_hash refresh on bind, never on walk.** Walks compare; only binds mint new hashes. Documented at `commands/doc_alignment.rs:802-804`.

### Inefficiencies (v0.12.1-specific)

- **115-commit unpushed local stack across the autonomous run.** Pushed `main` once at session-end; cumulative drift surfaced only when pre-push gate finally ran. Fmt drift from P73/P75 sat in 7 commits before being caught. (Fixed in v0.13.0 via per-phase push.)
- **Pre-commit hook was intentionally empty.** `.githooks/pre-commit:18` literally read `# Project-side pre-commit logic -- intentionally empty for now.` Fmt only ran at pre-push. P73 + P75 committed unformatted code that compounded.
- **`docs/reference/jira.md:96-99` "Phase 28 read-only" prose sat stale for ~3 phases.** P28 retire rationale literally noted *"Doc prose at jira.md:96-99 also stale; UPDATE_DOC follow-up captured"* — but no follow-up landed until 2026-04-30 owner-TTY review. Propose-retire workflow can write "TODO" into rationale and have it silently lost.
- **`reposix-quality doc-alignment confirm-retire` lacks batch mode.** Draining 27 RETIRE_PROPOSED rows required hand-rolled `jq | while read | call CLI per id`. Backlog 999.2.
- **Pre-push gate runner conflates `timed_out` with `asserts_failed`.** False positive every weekly run. Backlog 999.3.
- **`.planning/RETROSPECTIVE.md` had been silently neglected since v0.8.0.** Three milestones of learnings (v0.9.0–v0.12.0) live only in milestone archives. Codified the fix as OP-9.

---

## Milestone: v0.8.0 — JIRA Cloud Integration (summary)

**Shipped:** 2026-04-16 | **Phases:** 27–29 | **Full narrative:** `.planning/milestones/v0.8.0-phases/RETROSPECTIVE-FULL.md`

Most-cited patterns established here: copy-adapt of Confluence ADF module into JIRA, `OnceLock<Vec<String>>` session-scoped API discovery cache, `assert_write_contract<B>()` create→update→delete→assert-gone round-trip (reusable across backends), wiremock FIFO mock ordering (NOT LIFO).

Key cross-milestone lesson: **plan the rename phase before the feature phase** — Phase 27 (`IssueBackend` → `BackendConnector` rename) before Phase 28 (new JIRA crate) meant the new crate was written against the correct API from day one, no rename debt carried into new code.

---

## Cross-Milestone Trends

### Recurring patterns

- **Catalog-first phase rule.** Every phase's first commit mints catalog rows defining the GREEN contract; the verifier subagent grades against rows that existed pre-execution. v0.12.1 ratified, v0.13.0 held under DVCS scope creep.
- **Unbiased verifier subagent at every phase close.** Per-phase verdict files at `quality/reports/verdicts/p<n>/`; executing agent never grades itself.
- **+2 reservation slots (OP-8).** Surprises absorption (slot 1) before good-to-haves polish (slot 2), because surprises resolution can surface new polish entries. Slot 1 has a verifier honesty spot-check.
- **Layered coverage** (shell harness for agent UX surface + cargo test for wire path). Sanctioned in v0.13.0 P80 → P86; the new house default for env-propagation-sensitive surfaces.
- **Hooks > prose for enforcement.** v0.12.1 fmt-hook lesson; hook error message is self-documenting; CLAUDE.md "next agent must remember to X" rules are redundant if a hook can enforce.
- **Per-phase push cadence.** Codified mid-v0.13.0 (closes backlog 999.4); kept the v0.12.1 115-commit unpushed-stack failure mode from recurring.
- **Plan-checker subagents catch contracts pre-execute.** Eager-fix at plan time saves phase-time auto-fixes (v0.13.0 P81-01 + P82 examples).
- **Catalog-first commit with eventual-pass verifiers.** First commit mints rows + verifiers that FAIL at commit time (asserting EVENTUAL state); subsequent commits flip them to PASS.

### Recurring failure modes

- **Push-deferral compounds drift invisibly.** Session-end push defers CI signal by full session length. v0.12.1 surfaced fmt drift only when pre-push finally ran. Mitigation: per-phase push (v0.13.0+).
- **End-to-end shell harnesses are env-propagation-brittle.** `reposix init`+`git fetch`+`git push` in shell scope kept hitting `fatal: could not read ref refs/reposix/main`. Mitigation: cargo-test-as-verifier with `assert_cmd` (v0.13.0 P80 pivot).
- **Substrate gaps masquerade as feature gaps.** v0.13.0 P84 binstall+yanked-gix surfaced as a webhook-template feature shortfall; was actually a release-pipeline issue. Mitigation: file as DEFERRED with severity HIGH + owner-runnable measurement script in tree.
- **Telemetry-bearing catalogs create commit churn.** Every successful gate run mutates state. Architectural cleanup deferred; today bundled into intentional commits.
- **Buried "follow-up" promises in retire-rationales get lost.** v0.12.1 jira.md case sat stale ~3 phases. Mitigation: pre-confirm sweep audits rationales for TODO promises.
- **Tooling gaps surface as ad-hoc bash + provenance flags.** Both v0.12.1 (`confirm-retire` no batch mode → hand-rolled `jq | while`) and v0.13.0 (`bind` no non-docs-alignment → hand-edit + `_provenance_note`). OP #4 (ad-hoc bash is a missing-tool signal) catches the first; the second is filed as DVCS-CF-02.

### Process gaps that resolved

- **OP-9 (milestone-close ritual)** codified in v0.12.1 after recognizing RETROSPECTIVE.md had been silently neglected since v0.8.0.
- **Per-phase push cadence** codified in v0.13.0 after v0.12.1's 115-commit autonomous-run stack.
- **`--i-am-human` bypass** added in v0.12.1 to let owner-authorized Claude Code sessions clear "owner-TTY only" gates with permanent audit-trail flag.
- **Verifier honesty spot-check** added to OP-8 in v0.12.1 P76; sampled 5 phases in v0.13.0 P87 (exceeded ≥3 floor).

### Process Evolution

| Milestone | Phases | Key Change |
|-----------|--------|------------|
| v0.1.0 | 4+S | Initial MVD: simulator + FUSE + CLI + write path |
| v0.3.0 | 1 | First external adapter (Confluence read-only) |
| v0.6.0 | 5 | Write path + full sitemap (OP-1, OP-2, OP-3) |
| v0.7.0 | 6 | Hardening + Confluence expansion (OP-7..11) |
| v0.8.0 | 3 | JIRA Cloud integration: rename + read + write |
| v0.12.1 | 6+TTY | Catalog-first + +2 phase practice ratified; OP-9 codified |
| v0.13.0 | 11 | DVCS over REST; per-phase push + layered coverage default |

### Cumulative Quality

| Milestone | Tests (workspace) | Notes |
|-----------|-------------------|-------|
| v0.1.0 | 168 | Simulator + FUSE + guardrails |
| v0.3.0 | 193 | + Confluence wiremock |
| v0.5.0 | 278 | + IssueBackend decoupling + _INDEX.md |
| v0.6.0 | 317+ | + Confluence write path |
| v0.7.0 | ~400+ | + Hardening + comments + whiteboards |
| v0.8.0 | All green | + JIRA read + write (31 new unit + 5 contract) |
| v0.12.1 | >= 368 binaries | Re-measured P72 per D-06 |
| v0.13.0 | All green | + DVCS attach/bus/mirror + fault-injection coverage |

### Top Lessons (Verified Across Milestones)

1. **Simulator-first pays off every time.** Every new backend (GitHub, Confluence, JIRA) was fully tested against wiremock before touching a real API. Zero credential exposure during development.
2. **Copy-adapt before rewrite.** ADF module (Confluence → JIRA), workload patterns (SimDirect → ConfluenceDirect), contract test helpers, fault-injection fixtures — adapting existing code is 3-5× faster than reimplementing.
3. **Phase ordering matters.** Foundation phases (type renames, trait changes) before feature phases prevents debt accumulation in new code.
4. **Catalog-first + unbiased verifier closes the "graded what we shipped" feedback loop.** Rows defining the GREEN contract exist before the work; verifier subagent grades artifacts with zero session context.
5. **Per-phase push + layered coverage are the v0.13.0 ratifications that change autonomous-run shape going forward.**
