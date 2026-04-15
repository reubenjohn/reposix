---
phase: 21
verified: 2026-04-15T00:00:00Z
status: passed
score: 8/8
overrides_applied: 0
---

# Phase 21: OP-7 Hardening Bundle — Verification Report

**Phase Goal:** Harden the v0.3-v0.5 read path by completing the OP-7 bundle: credential pre-push hook + SSRF tests (HARD-00), contention swarm proving If-Match 409 determinism (HARD-01), Confluence 500-page truncation probe with WARN + `list_issues_strict` + `--no-truncate` CLI flag (HARD-02), kill-9 chaos test proving WAL atomicity (HARD-03), macOS CI hooks step shipped + FUSE matrix deferred to self-hosted runner (HARD-04 partial), tenant URL redaction in list_issues error messages (HARD-05).

**Verified:** 2026-04-15T00:00:00Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| #  | Truth                                                                             | Status     | Evidence                                                                                                |
|----|-----------------------------------------------------------------------------------|------------|---------------------------------------------------------------------------------------------------------|
| 1  | `ContentionWorkload` exists in `crates/reposix-swarm/src/contention.rs`          | VERIFIED   | `pub struct ContentionWorkload` at line 35; `impl ContentionWorkload` at line 44; `impl Workload` at 70 |
| 2  | `Mode::Contention` is wired in `crates/reposix-swarm/src/main.rs`                | VERIFIED   | `Mode::Contention =>` dispatch arm found at line 152                                                    |
| 3  | `list_issues_strict` exists in `crates/reposix-confluence/src/lib.rs`            | VERIFIED   | `pub async fn list_issues_strict` at line 534                                                           |
| 4  | `redact_url` exists in `crates/reposix-confluence/src/lib.rs`                    | VERIFIED   | `fn redact_url(raw: &str) -> String` at line 427                                                        |
| 5  | `no_truncate` is wired in `crates/reposix-cli/src/list.rs`                       | VERIFIED   | `no_truncate: bool` field at line 67; used in Confluence branch at line 91                              |
| 6  | `chaos_kill9_no_torn_rows` exists in `crates/reposix-swarm/tests/chaos_audit.rs` | VERIFIED   | `async fn chaos_kill9_no_torn_rows` at line 153                                                         |
| 7  | `.github/workflows/ci.yml` contains `test-pre-push.sh` hooks CI step             | VERIFIED   | `run: bash scripts/hooks/test-pre-push.sh` at line 55 of ci.yml                                         |
| 8  | `cargo test --workspace` exits 0 (all tests pass)                                | VERIFIED   | All 28 test suites: 0 failed; total ~355 passing tests; 10 ignored (chaos/FUSE guarded by `#[ignore]`)  |

**Score:** 8/8 truths verified

### Required Artifacts

| Artifact                                                  | Expected                                    | Status     | Details                                          |
|-----------------------------------------------------------|---------------------------------------------|------------|--------------------------------------------------|
| `crates/reposix-swarm/src/contention.rs`                  | ContentionWorkload struct + Workload impl   | VERIFIED   | 125 lines; full GET-PATCH-If-Match cycle         |
| `crates/reposix-swarm/src/main.rs`                        | Mode::Contention dispatch arm               | VERIFIED   | dispatch arm + --target-issue arg present        |
| `crates/reposix-confluence/src/lib.rs`                    | list_issues_strict + redact_url             | VERIFIED   | both symbols present; redact applied to all error paths |
| `crates/reposix-cli/src/list.rs`                          | --no-truncate flag wired to list_issues_strict | VERIFIED | no_truncate: bool at line 67; Confluence branch at line 91 |
| `crates/reposix-swarm/tests/chaos_audit.rs`               | chaos_kill9_no_torn_rows test               | VERIFIED   | 285 lines; two-cycle SIGKILL + WAL replay        |
| `.github/workflows/ci.yml`                                | test-pre-push.sh step in test job           | VERIFIED   | bash step at line 55                             |

### Key Link Verification

| From                        | To                                     | Via                                 | Status   | Details                                                        |
|-----------------------------|----------------------------------------|-------------------------------------|----------|----------------------------------------------------------------|
| `Mode::Contention` (main.rs) | `ContentionWorkload::new` (contention.rs) | `use reposix_swarm::contention::ContentionWorkload` | WIRED | Import + dispatch arm verified |
| `list.rs` no_truncate=true  | `ConfluenceBackend::list_issues_strict` | direct call in Confluence branch    | WIRED    | `b.list_issues_strict(&project).await?` at list.rs line 91    |
| `redact_url`                | all error format strings in lib.rs     | called at every URL interpolation   | WIRED    | SUMMARY 21-C documents 9 sites; grep confirms no raw URL leaks |

### Data-Flow Trace (Level 4)

Not applicable — this phase produces hardening probes and CLI flags, not UI components rendering dynamic data. No Level 4 data-flow trace required.

### Behavioral Spot-Checks

| Behavior                                    | Command                                                                                         | Result               | Status |
|---------------------------------------------|-------------------------------------------------------------------------------------------------|----------------------|--------|
| ContentionWorkload symbol exists            | `grep "pub struct ContentionWorkload" crates/reposix-swarm/src/contention.rs`                  | match at line 35     | PASS   |
| Mode::Contention wired in main              | `grep "Mode::Contention" crates/reposix-swarm/src/main.rs`                                     | match at line 152    | PASS   |
| list_issues_strict symbol exists            | `grep "pub async fn list_issues_strict" crates/reposix-confluence/src/lib.rs`                  | match at line 534    | PASS   |
| redact_url symbol exists                    | `grep "fn redact_url" crates/reposix-confluence/src/lib.rs`                                    | match at line 427    | PASS   |
| no_truncate in CLI list handler             | `grep "no_truncate" crates/reposix-cli/src/list.rs`                                            | matches at lines 53,67,91 | PASS |
| chaos_kill9_no_torn_rows exists             | `grep "chaos_kill9_no_torn_rows" crates/reposix-swarm/tests/chaos_audit.rs`                    | match at line 153    | PASS   |
| Hooks CI step in ci.yml                     | `grep "test-pre-push.sh" .github/workflows/ci.yml`                                             | match at line 55     | PASS   |
| Full workspace tests pass                   | `cargo test --workspace`                                                                        | 0 failed, all ok     | PASS   |

### Requirements Coverage

| Requirement | Source Plan | Description                                          | Status    | Evidence                                          |
|-------------|------------|------------------------------------------------------|-----------|---------------------------------------------------|
| HARD-00     | 21-A, 21-E | Credential pre-push hook + SSRF tests in CI          | SATISFIED | Hook 6/6 green; SSRF 3/3 green; CI step at ci.yml:55 |
| HARD-01     | 21-B       | ContentionWorkload + Mode::Contention                | SATISFIED | contention.rs + main.rs both verified             |
| HARD-02     | 21-C       | Confluence 500-page guard + --no-truncate CLI flag   | SATISFIED | list_issues_strict + no_truncate wired            |
| HARD-03     | 21-D       | kill-9 chaos test proving WAL atomicity              | SATISFIED | chaos_audit.rs, chaos_kill9_no_torn_rows verified |
| HARD-04     | 21-E       | macOS CI parity (partial — FUSE matrix deferred)     | PARTIAL   | Hooks step ships; FUSE matrix deferred by design: macFUSE kext approval unavailable on GitHub-hosted runners |
| HARD-05     | 21-C       | Tenant URL redaction in error messages               | SATISFIED | redact_url applied to all 9+ error sites in lib.rs |

Note: HARD-04 partial delivery is an intentional, documented design decision (21-E-SUMMARY). The macOS FUSE matrix requires a self-hosted runner with macFUSE pre-approved — a platform constraint, not an implementation gap.

### Anti-Patterns Found

No blockers or warnings. Three info-level findings from code review (21-REVIEW.md) are non-blocking:

| File                                           | Line | Pattern                                        | Severity | Impact                              |
|------------------------------------------------|------|------------------------------------------------|----------|-------------------------------------|
| `crates/reposix-swarm/tests/chaos_audit.rs`    | 41   | Hardcoded port 7979 — parallel run collision   | Info     | Manual/weekly gate only, #[ignore]  |
| `crates/reposix-confluence/src/lib.rs`         | 214  | ConfLinks dead code masked by allow(dead_code) | Info     | Maintenance hazard, not a bug       |
| `crates/reposix-swarm/tests/contention_e2e.rs` | 119  | String match on rendered Markdown format       | Info     | Fragile if metrics.rs format changes |

### Human Verification Required

None. All must-haves are verifiable programmatically. The chaos test (`chaos_kill9_no_torn_rows`) is `#[ignore]` and requires `REPOSIX_CHAOS_TEST=1` to run — it was executed during development per 21-D-SUMMARY and verified 0 torn rows across two kill-9 cycles. No human visual or UX verification is needed for this hardening phase.

### Gaps Summary

No gaps. All 8 must-haves verified. The phase goal is achieved: the OP-7 hardening bundle is complete with all deliverables present, substantive, and wired. HARD-04's macOS FUSE matrix deferral is an intentional platform constraint, not a gap — the hooks CI step (the actionable part of HARD-04) shipped and is verified.

---

_Verified: 2026-04-15T00:00:00Z_
_Verifier: Claude (gsd-verifier)_
