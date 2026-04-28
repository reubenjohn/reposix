# v0.12.1 Roadmap (PLANNING)

_Carry-forward milestone closing v0.12.0 stubs + v0.11.x debts. Detail lands via `/gsd-plan-phase` per phase._

## Scope

v0.12.1 closes the v0.12.0 stubs: perf-dimension full implementation, security-dimension stubs -> real verifiers, cross-platform rehearsals, MSRV / binstall / latest-pointer / release-PAT carry-forwards, and the v0.11.1 `Error::Other` migration completion. Subjective-dimension runner invariants from P61 Wave G also land here.

## Depends on

- v0.12.0 GREEN verdict at `quality/reports/verdicts/p63/VERDICT.md`.
- `quality/PROTOCOL.md` v0.12.0 runtime contract intact.

## Phases

### Phase 64: Perf-dimension full implementation

**Goal:** Wire the 3 perf-targets stub rows (`perf/latency-bench`, `perf/token-economy-bench`, `perf/headline-numbers-cross-check`) into real verifiers that cross-check bench output against doc headline numbers; flip waivers to active enforcement.

**Requirements:** PERF-01, PERF-02, PERF-03.

**Success criteria:** All 3 perf-targets rows PASS at runner time; waivers removed; cron-driven verification posts evidence under `quality/reports/verifications/perf/`.

### Phase 65: Security-dimension stubs -> real

**Goal:** Wire `security/allowlist-enforcement` + `security/audit-immutability` to real Rust integration tests + bash verifier wrappers. Lift waivers.

**Requirements:** SEC-01, SEC-02.

**Success criteria:** Both rows PASS; outbound-HTTP allowlist enforced at runtime; audit-table immutability tested at SQLite schema layer.

### Phase 66: Cross-platform rehearsals

**Goal:** Add windows-2022 + macos-14 GH Actions runners to `release.yml` matrix; rehearse curl-installer + cargo-binstall on each.

**Requirements:** CROSS-01, CROSS-02.

**Success criteria:** Both `cross-platform` catalog rows PASS at pre-release cadence; release artifacts proven cross-platform.

### Phase 67: MSRV / binstall / release-pipeline + Error::Other completion

**Goal:** Close the v0.12.0 P56 SURPRISES carry-forward set + the v0.11.1 POLISH2-09 partial migration.

**Requirements:** MSRV-01, BINSTALL-01, LATEST-PTR-01, RELEASE-PAT-01, ERR-OTHER-01.

**Success criteria:** `cargo install reposix-cli` from crates.io succeeds; `cargo binstall reposix-cli` PASS; `releases/latest/download/...` pointer pinned to cli release; release-plz tag triggers `release.yml`; zero `Error::Other(String)` sites remain.

### Phase 68: Subjective-dimension runner invariants (P61 Wave G carry-forward)

**Goal:** Implement the 3 subjective-runner invariants from P61 Wave G.

**Requirements:** SUBJ-RUNNER-01, SUBJ-AUTODISPATCH-01, SUBJ-HARDGATE-01.

**Success criteria:** Subjective-graded rows preserve dispatched verdicts; CI auto-dispatch works on Anthropic API auth'd GH Actions runners; `release.yml` waits on `quality-pre-release.yml` verdict.
