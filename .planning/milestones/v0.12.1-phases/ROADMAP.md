# v0.12.1 Roadmap (PLANNING)

_Carry-forward milestone closing v0.12.0 stubs + v0.11.x debts + the docs-alignment punch list surfaced by P65. Detail lands via `/gsd-plan-phase` per phase._

> **Phase numbering update (2026-04-28).** v0.12.0 grew P64 (docs-alignment infrastructure) and P65 (docs-alignment backfill) before tagging. Existing v0.12.1 phases originally scoped at P64–P68 are renumbered to P66–P70 to preserve the monotonic project-wide phase counter. Doc-alignment gap-closure phases append at P71+ — concrete cluster scopes come from `quality/reports/doc-alignment/backfill-<ts>/PUNCH-LIST.md` (P65 deliverable). The stub entries at P71+ list the cluster *shapes* we expect; the human refines them post-tag.

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

### Phase 66: Perf-dimension full implementation

**Goal:** Wire the 3 perf-targets stub rows (`perf/latency-bench`, `perf/token-economy-bench`, `perf/headline-numbers-cross-check`) into real verifiers that cross-check bench output against doc headline numbers; flip waivers to active enforcement.

**Requirements:** PERF-01, PERF-02, PERF-03.

**Execution mode:** `executor`.

**Success criteria:** All 3 perf-targets rows PASS at runner time; waivers removed; cron-driven verification posts evidence under `quality/reports/verifications/perf/`.

### Phase 67: Security-dimension stubs -> real

**Goal:** Wire `security/allowlist-enforcement` + `security/audit-immutability` to real Rust integration tests + bash verifier wrappers. Lift waivers.

**Requirements:** SEC-01, SEC-02.

**Execution mode:** `executor`.

**Success criteria:** Both rows PASS; outbound-HTTP allowlist enforced at runtime; audit-table immutability tested at SQLite schema layer.

### Phase 68: Cross-platform rehearsals

**Goal:** Add windows-2022 + macos-14 GH Actions runners to `release.yml` matrix; rehearse curl-installer + cargo-binstall on each.

**Requirements:** CROSS-01, CROSS-02.

**Execution mode:** `executor`.

**Success criteria:** Both `cross-platform` catalog rows PASS at pre-release cadence; release artifacts proven cross-platform.

### Phase 69: MSRV / binstall / release-pipeline + Error::Other completion

**Goal:** Close the v0.12.0 P56 SURPRISES carry-forward set + the v0.11.1 POLISH2-09 partial migration.

**Requirements:** MSRV-01, BINSTALL-01, LATEST-PTR-01, RELEASE-PAT-01, ERR-OTHER-01.

**Execution mode:** `executor`.

**Success criteria:** `cargo install reposix-cli` from crates.io succeeds; `cargo binstall reposix-cli` PASS; `releases/latest/download/...` pointer pinned to cli release; release-plz tag triggers `release.yml`; zero `Error::Other(String)` sites remain.

### Phase 70: Subjective-dimension runner invariants (P61 Wave G carry-forward)

**Goal:** Implement the 3 subjective-runner invariants from P61 Wave G.

**Requirements:** SUBJ-RUNNER-01, SUBJ-AUTODISPATCH-01, SUBJ-HARDGATE-01.

**Execution mode:** `executor`.

**Success criteria:** Subjective-graded rows preserve dispatched verdicts; CI auto-dispatch works on Anthropic API auth'd GH Actions runners; `release.yml` waits on `quality-pre-release.yml` verdict.

### Phase 71+: Docs-alignment gap closure (cluster phases)

**Goal:** For each cluster of `MISSING_TEST` rows surfaced by P65's `PUNCH-LIST.md`, ship the missing implementation and tests until every row in the cluster transitions to `BOUND`. `RETIRE_PROPOSED` rows in the cluster are reviewed by the human; for those judged genuinely superseded, the human runs `reposix-quality doc-alignment confirm-retire` (env-guarded — agents cannot do this).

**Phase shape per cluster** (the human refines exact cluster scopes post-P65):

- **P71 — Confluence backend parity gap closure.** Re-implement the page-tree symlink behavior the v0.9.0 pivot dropped. Bind tests in `tests/agent_flow_real.rs::dark_factory_real_confluence` (and a new `tests/init_shape.rs::confluence_*` if needed). Run `reposix-quality doc-alignment refresh docs/reference/confluence.md` after binding tests; expect every Confluence MISSING_TEST row to flip BOUND.
- **P72 — JIRA backend parity gap closure.** Same shape, against the JIRA cluster.
- **P73 — Ease-of-setup happy path.** Bind tests against the install-and-init prose in `README.md` and `docs/index.md`.
- **P74 — Outbound HTTP allowlist behavior.** Bind tests against the threat-model claims in CLAUDE.md and `research/threat-model-and-critique.md`.
- **P75+** — additional clusters as P65 surfaces them.

**Per-cluster requirements:** DOC-ALIGN-GAP-* (filed by P65's PUNCH-LIST.md generation; one REQ per cluster minimum).

**Execution mode:** `executor` (the gap-closure work itself is implementation-shaped — write tests, write code, commit. Refresh runs after binding are top-level slash-command invocations the executor's plan emits as a "user, run `/reposix-quality-refresh <doc>` then resume" checkpoint).

**Success criteria (per cluster phase):** every catalog row in the cluster transitions BOUND or RETIRE_CONFIRMED; `reposix-quality doc-alignment status` confirms alignment_ratio rises by the expected delta; verifier subagent verdict GREEN at `quality/reports/verdicts/p<N>/VERDICT.md`.

**Milestone-completion contract:** v0.12.1 closes when `reposix-quality doc-alignment status` reports `claims_missing_test == 0` (or all remaining `MISSING_TEST` rows have explicit waivers with `dimension_owner` and `until` dates) AND `claims_retire_proposed == 0`. `floor_waiver` from P65 expires; the floor ratchets up to whatever `alignment_ratio` is at milestone close (rounded down to the nearest 0.05).
