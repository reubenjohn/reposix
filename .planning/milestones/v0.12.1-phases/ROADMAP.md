# v0.12.1 Roadmap (PLANNING)

_Carry-forward milestone closing v0.12.0 stubs + v0.11.x debts + the docs-alignment punch list surfaced by P65. Detail lands via `/gsd-plan-phase` per phase._

> **Phase numbering update (2026-04-28).** v0.12.0 grew P64 (docs-alignment infrastructure) and P65 (docs-alignment backfill) before tagging. v0.12.1 grew P66 (coverage_ratio metric — the second axis alongside `alignment_ratio`) at the front of the milestone. Existing v0.12.1 phases originally scoped at P64–P68 were renumbered to P66–P70 in the v0.12.0/P64+P65 insertion; THIS commit renumbers them again to P67–P71 to make room for the new P66 (coverage). Doc-alignment gap-closure phases append at P72+ — concrete cluster scopes come from `quality/reports/doc-alignment/backfill-<ts>/PUNCH-LIST.md` (P65 deliverable). The stub entries at P72+ list the cluster *shapes* we expect; the human refines them post-tag.

## Scope

v0.12.1 closes:

1. **v0.12.0 carry-forwards** — perf-dimension full implementation, security-dimension stubs → real verifiers, cross-platform rehearsals, MSRV / binstall / latest-pointer / release-PAT carry-forwards, the v0.11.1 `Error::Other` migration completion, subjective-dimension runner invariants from P61 Wave G.
2. **Docs-alignment gap closure** — the v0.9.0 pivot silently dropped behaviors prior milestones promised; P65's backfill surfaces them as `MISSING_TEST` rows clustered by user-facing surface (Confluence backend parity, JIRA shape, ease-of-setup happy path, outbound HTTP allowlist behavior, and others discovered by the backfill). Each cluster gets its own phase. Each phase's success criterion is "every catalog row in the cluster transitions BOUND or RETIRE_CONFIRMED." The catalog becomes the milestone-completion contract.

## Depends on

- v0.12.0 GREEN verdict at `quality/reports/verdicts/milestone-v0.12.0/VERDICT.md` (re-graded after P64 + P65 ship).
- `quality/catalogs/doc-alignment.json` populated with extracted claims (P65 deliverable).
- `quality/reports/doc-alignment/backfill-<ts>/PUNCH-LIST.md` reviewed by the human; cluster scopes confirmed.
- `quality/PROTOCOL.md` v0.12.0 runtime contract intact (the two project-wide principles from P64 inform every gap-closure phase).

## Phases

### Phase 66: coverage_ratio metric — docs-alignment second axis

**Goal:** Add the `coverage_ratio` metric to the docs-alignment dimension as the second axis alongside `alignment_ratio`. Where `alignment_ratio` answers "of the claims we extracted, how many bind to passing tests?", `coverage_ratio` answers "of the prose we said we'd mine, what fraction of lines did we actually cover with at least one row?". The two together make the 2x2 (high/low alignment vs high/low coverage) the agent's mental model for gap-closure work. The `status` verb gains a per-file table sorted worst-coverage-first so an agent looking to widen coverage reads `status --top 10` and knows which files to mine next.

**Requirements:** COVERAGE-01, COVERAGE-02, COVERAGE-03, COVERAGE-04, COVERAGE-05.

**Depends on:** v0.12.0 P64 (catalog + walker live) + P65 (388 rows populated). No dependency on the other v0.12.1 phases.

**Execution mode:** `executor`.

**Success criteria:**
- `Summary` struct grows `coverage_ratio` + `lines_covered` + `total_eligible_lines` + `coverage_floor` (all `#[serde(default)]` for back-compat).
- `coverage::compute_per_file` + `compute_global` correctly union overlapping/adjacent ranges; multi-source rows attribute to each cited file independently; out-of-eligible-set rows warn + skip.
- `walk` populates the global summary fields each run AND BLOCKs when `coverage_ratio < coverage_floor` (default 0.10) with structured stderr naming the slash-command recovery path.
- `status` displays a global summary block + a per-file (worst-first) table; `--top N`, `--all`, `--json` flags supported.
- `quality/catalogs/README.md` documents the 2x2 alignment-vs-coverage matrix + `coverage_floor` ratchet semantics.
- Verifier subagent verdict GREEN at `quality/reports/verdicts/p66/VERDICT.md`.

**Critical invariant:** `coverage_floor` stays at 0.10 in the shipping commit. A future deliberate human commit ratchets it up as gap-closure phases (P72+) widen coverage.

### Phase 67: Perf-dimension full implementation

**Goal:** Wire the 3 perf-targets stub rows (`perf/latency-bench`, `perf/token-economy-bench`, `perf/headline-numbers-cross-check`) into real verifiers that cross-check bench output against doc headline numbers; flip waivers to active enforcement.

**Requirements:** PERF-01, PERF-02, PERF-03.

**Execution mode:** `executor`.

**Success criteria:** All 3 perf-targets rows PASS at runner time; waivers removed; cron-driven verification posts evidence under `quality/reports/verifications/perf/`.

### Phase 68: Security-dimension stubs -> real

**Goal:** Wire `security/allowlist-enforcement` + `security/audit-immutability` to real Rust integration tests + bash verifier wrappers. Lift waivers.

**Requirements:** SEC-01, SEC-02.

**Execution mode:** `executor`.

**Success criteria:** Both rows PASS; outbound-HTTP allowlist enforced at runtime; audit-table immutability tested at SQLite schema layer.

### Phase 69: Cross-platform rehearsals

**Goal:** Add windows-2022 + macos-14 GH Actions runners to `release.yml` matrix; rehearse curl-installer + cargo-binstall on each.

**Requirements:** CROSS-01, CROSS-02.

**Execution mode:** `executor`.

**Success criteria:** Both `cross-platform` catalog rows PASS at pre-release cadence; release artifacts proven cross-platform.

### Phase 70: MSRV / binstall / release-pipeline + Error::Other completion

**Goal:** Close the v0.12.0 P56 SURPRISES carry-forward set + the v0.11.1 POLISH2-09 partial migration.

**Requirements:** MSRV-01, BINSTALL-01, LATEST-PTR-01, RELEASE-PAT-01, ERR-OTHER-01.

**Execution mode:** `executor`.

**Success criteria:** `cargo install reposix-cli` from crates.io succeeds; `cargo binstall reposix-cli` PASS; `releases/latest/download/...` pointer pinned to cli release; release-plz tag triggers `release.yml`; zero `Error::Other(String)` sites remain.

### Phase 71: Subjective-dimension runner invariants (P61 Wave G carry-forward)

**Goal:** Implement the 3 subjective-runner invariants from P61 Wave G.

**Requirements:** SUBJ-RUNNER-01, SUBJ-AUTODISPATCH-01, SUBJ-HARDGATE-01.

**Execution mode:** `executor`.

**Success criteria:** Subjective-graded rows preserve dispatched verdicts; CI auto-dispatch works on Anthropic API auth'd GH Actions runners; `release.yml` waits on `quality-pre-release.yml` verdict.

### Phase 72+: Docs-alignment gap closure (cluster phases)

**Goal:** For each cluster of `MISSING_TEST` rows surfaced by P65's `PUNCH-LIST.md`, ship the missing implementation and tests until every row in the cluster transitions to `BOUND`. `RETIRE_PROPOSED` rows in the cluster are reviewed by the human; for those judged genuinely superseded, the human runs `reposix-quality doc-alignment confirm-retire` (env-guarded — agents cannot do this).

**Phase shape per cluster** (the human refines exact cluster scopes post-P65):

- **P72 — Confluence backend parity gap closure.** Re-implement the page-tree symlink behavior the v0.9.0 pivot dropped. Bind tests in `tests/agent_flow_real.rs::dark_factory_real_confluence` (and a new `tests/init_shape.rs::confluence_*` if needed). Run `reposix-quality doc-alignment refresh docs/reference/confluence.md` after binding tests; expect every Confluence MISSING_TEST row to flip BOUND.
- **P73 — JIRA backend parity gap closure.** Same shape, against the JIRA cluster.
- **P74 — Ease-of-setup happy path.** Bind tests against the install-and-init prose in `README.md` and `docs/index.md`.
- **P75 — Outbound HTTP allowlist behavior.** Bind tests against the threat-model claims in CLAUDE.md and `research/threat-model-and-critique.md`.
- **P76+** — additional clusters as P65 surfaces them.

**Per-cluster requirements:** DOC-ALIGN-GAP-* (filed by P65's PUNCH-LIST.md generation; one REQ per cluster minimum).

**Execution mode:** `executor` (the gap-closure work itself is implementation-shaped — write tests, write code, commit. Refresh runs after binding are top-level slash-command invocations the executor's plan emits as a "user, run `/reposix-quality-refresh <doc>` then resume" checkpoint).

**Success criteria (per cluster phase):** every catalog row in the cluster transitions BOUND or RETIRE_CONFIRMED; `reposix-quality doc-alignment status` confirms alignment_ratio rises by the expected delta; verifier subagent verdict GREEN at `quality/reports/verdicts/p<N>/VERDICT.md`.

**Milestone-completion contract:** v0.12.1 closes when `reposix-quality doc-alignment status` reports `claims_missing_test == 0` (or all remaining `MISSING_TEST` rows have explicit waivers with `dimension_owner` and `until` dates) AND `claims_retire_proposed == 0`. `floor_waiver` from P65 expires; the floor ratchets up to whatever `alignment_ratio` is at milestone close (rounded down to the nearest 0.05).
