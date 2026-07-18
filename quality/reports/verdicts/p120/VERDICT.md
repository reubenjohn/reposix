# P120 VERDICT: GREEN

Phase 120 — v0.15.0 "Floor" UX-01: CLI/helper error-message hardening + credential-hygiene fixes.
Graded goal-backward against SC1–SC3 plus the non-leakage-assertion crux (WR-01/02/03). Every
claim below was verified against reality (tests RUN, `git log` re-derived, `teach_scan.py` executed,
CI conclusion read from `gh`), not from SUMMARY/HANDOVER prose.

Verifier: Claude (gsd-verifier), unbiased phase-close. Verified: 2026-07-17.
Working-tree tip at grade time: `efdb38e6` (impl landed `859ba0e3`; WR fixes at `9bd88d27`).

---

## Overall verdict: GREEN

All three success criteria PASS. All three credential-leak fixes (WR-01/02/03) carry a genuine
non-leakage regression test that constructs a secret-bearing string and asserts the secret is
ABSENT — confirmed by running each test. Catalog-first ordering held. Main CI is green.

---

## Per-SC verdicts

### SC1 — every enumerated `reposix-cli` subcommand error routes through the shared teaching builder/helper (or a documented exception) and emits Fix:/Recovery: — PASS

- `python3 quality/gates/agent-ux/teach_scan.py --scope cli` → `clean (14 files, no un-dispositioned bail!/anyhow! block).` exit 0 (RE-RUN this session).
- Shared builder present: `crates/reposix-core/src/errmsg.rs` (`pub fn teach`, `pub struct Teach`, exported via `lib.rs`); shape helpers in `crates/reposix-cli/src/errors.rs` (`spec_parse_error`, `missing_env_var_error`, `cache_build_error`, `missing_cache_db_error`).
- `doctor.rs` is the documented structured-report exception (`crates/CLAUDE.md` § "doctor.rs is exempt from the teach-scan by scope").
- Catalog row `agent-ux/cli-errors-teach-recovery` = `status: PASS`, `last_verified: 2026-07-17T17:15:19Z`.
- CLI regression tests run green (WR-02 `sync_parse_error_redacts_mirror_credentials`, WR-03 `wr03_cache_path_error_redacts…` both `... ok`).

### SC2 — every enumerated `reposix-remote` helper error dispositioned (RETROFIT or EXEMPT-marked) and meets the bar where retrofit — PASS

- `python3 quality/gates/agent-ux/teach_scan.py --scope helper` → `clean (7 files, no un-dispositioned bail!/anyhow! block).` exit 0 (RE-RUN this session).
- Helper redaction/teaching tests run green: `redact_userinfo_{strips_creds_in_bus_mirror_component, passes_through_credential_free_urls_byte_exact, strips_creds_in_reposix_prefixed_origin, scrubs_token_in_git_authentication_failed_prose}` (4 unit, all `ok`); `malformed_credentialed_bus_url_does_not_leak_userinfo` (integration, `ok`).
- Catalog row `agent-ux/helper-errors-teach-recovery` = `status: PASS`, `last_verified: 2026-07-17T17:15:19Z`.
- The `helper-errors-teach-recovery.sh` gate leg (a) was widened from `--test errors_teach_recovery` to bare `cargo test -p reposix-remote` (commit `9fc73726`) so the bin-target WR-01 test is actually inside the gate's coverage — the bin-vs-integration seam the HANDOVER §4 flagged is closed.

### SC3 — the 2 agent-ux rows + 2 gate scripts + teach_scan.py + 2 test scaffolds are the phase's FIRST commit (git-log ordering) — PASS

Re-derived independently via `git log` (NOT trusted from HANDOVER):
- Plan commit: `02376a08 docs(planning): P120 plan`.
- **W0 `142e1278`** — `feat(P120-W0): catalog-first …` — touches ONLY:
  `quality/catalogs/agent-ux.json`, `quality/gates/agent-ux/cli-errors-teach-recovery.sh`,
  `quality/gates/agent-ux/helper-errors-teach-recovery.sh`, `quality/gates/agent-ux/teach_scan.py`,
  `crates/reposix-cli/tests/errors_teach_recovery.rs`, `crates/reposix-remote/tests/errors_teach_recovery.rs`
  (6 files, **713 insertions, 0 deletions — zero `src/*.rs` impl lines**).
- First impl commit: **W1 `15e971bc`** — first to touch `crates/**/src/*.rs` (`errmsg.rs`, `init.rs`, `attach.rs`, `errors.rs`, `lib.rs`).
- `142e1278` strictly precedes `15e971bc`. Catalog SHA predates every impl SHA. SC3 holds.

---

## Non-leakage crux — the 3 credential-leak fixes (WR-01/02/03)

Each fix was located, its guarding test named, and the test RUN to confirm it constructs a
secret-bearing string then asserts the secret is ABSENT.

### WR-01 — `bus_handler.rs` (MirrorResult::Failed → OP-3 audit row + operator diag)

- **Fix:** `record_mirror_partial_fail` (`bus_handler.rs:600-620`) redacts ONCE via
  `crate::backend_dispatch::redact_userinfo(stderr_tail)` (`:608`) and feeds the REDACTED tail to
  BOTH sinks — the append-only `helper_push_partial_fail_mirror_lag` audit row (`:610`) and the
  operator diag (`:613-617`).
- **Test:** `bus_handler::tests::wr01_mirror_partial_fail_scrubs_token_from_both_audit_row_and_diag` (`bus_handler.rs:701`).
- **Asserts non-leakage: YES.** Drives the REAL sink helper against a real `Cache` with a
  token-in-username git-auth-prose tail (`SECRET = "mirror-pushtoken-REDACTME-abc123def456"`), then:
  operator diag — `assert!(!diag.contains(SECRET))` (`:742`), `assert!(!diag.contains("x-access-token"))` (`:746`);
  **persisted audit row** read back from `cache.db` — `assert!(!reason.contains(SECRET))` (`:776`),
  `assert!(!reason.contains("x-access-token"))` (`:780`); plus positive `<redacted>@github.com` survival (`:750`, `:784`).
  The audit-row leg is the load-bearing half (a durable OP-3 forensic table).
- **Run result:** `... ok`.
- Sibling `precheck_mirror_drift_redacts_credentials_and_teaches_on_ls_remote_failure` (`:656`) — drives real `git ls-remote` against a credentialed unreachable loopback, `assert!(!msg.contains("SECRETTOKEN123"))` (`:662`) — `... ok`.

### WR-02 — `sync.rs` (`reposix sync` parse-error echo of raw remote URL)

- **Fix:** `sync.rs:121` `let safe_url = backend_dispatch::redact_userinfo(&url);` interpolated into the `teach(…)` headline (`:127`) instead of the raw `url`.
- **Test:** `sync_parse_error_redacts_mirror_credentials` (`crates/reposix-cli/tests/sync.rs:254`).
- **Asserts non-leakage: YES.** Drives the REAL `reposix sync --reconcile` binary against a tree whose
  `remote.origin.url` is `reposix::…/projects/..?mirror=https://x-access-token:SECRETSYNC77@mirror.example.com/m.git`, then
  `assert!(!combined.contains(SECRET))` (`:281`), `assert!(!combined.contains("x-access-token"))` (`:285`),
  plus `<redacted>@mirror.example.com` survival (`:289`).
- **Run result:** `... ok`.

### WR-03 — `worktree_helpers.rs` (`cache_path_from_worktree` parse-error echo — feeds history/tokens/cost/gc)

- **Fix:** `worktree_helpers.rs:220` `let safe_url = redact_userinfo(&url);` used in the `.with_context(…)` (`:222`) instead of raw `url`.
- **Test:** `worktree_helpers::tests::wr03_cache_path_error_redacts_mirror_credentials_in_malformed_bus_url` (`worktree_helpers.rs:410`).
- **Asserts non-leakage: YES.** Sets up a real tree with a credentialed `?mirror=` bus remote and a
  path-traversal SoT slug that forces the parse failure, drives the REAL `cache_path_from_worktree`, then
  `assert!(!msg.contains(SECRET))` (`:434`, `SECRET = "SECRETWT99"`), `assert!(!msg.contains("x-access-token"))` (`:438`),
  plus `<redacted>@mirror.example.com` survival (`:442`). Doc comment also pins the `strip_url_userinfo`-would-leak distinction.
- **Run result:** `... ok`.

**All three WR tests assert non-leakage — YES / YES / YES.** No test merely exercises the path without
asserting the secret's absence. `redact_userinfo` (the shared scrubber, `backend_dispatch.rs:468`) is
additionally unit-tested for the `reposix::`-prefixed and git-auth-prose shapes.

### Catalog-first for NEW quality-gate rows: YES

The only NEW gate rows this phase added are the two `agent-ux/*-teach-recovery` rows, minted in the W0
catalog-first commit `142e1278`, before all implementation. The `cred-hygiene.sh` P0 gate that the
WR-01 fixture token interacted with is PRE-EXISTING (`quality/gates/structure/cred-hygiene.sh`,
introduced p60 `c18a1247`) — not a P120 gate, so not subject to this phase's catalog-first ordering.

---

## CI / push-cadence check

`gh run list --branch main` — tip `efdb38e6`: `CI` run `29623625514` = `completed / success`;
`Docs`, `release-plz`, `CodeQL` on the same SHA all `success`. Impl SHA `859ba0e3` CI also `success`.
Main's NEWEST `ci.yml` run is green — push-cadence bar met.

---

## Gaps / Blockers

**None that block the phase goal.** SC1–SC3 all PASS; all three WR non-leakage tests assert and pass;
catalog-first ordering held; main CI green. Nothing loops back to a fresh executor.

Non-blocking items (below) are NOTICINGs and an out-of-scope symmetric leak filed to intake — none
gate P120 closure.

---

## Noticing (OD-3 deliverable)

1. **[MEDIUM — filed to SURPRISES-INTAKE this session] `doctor.rs` echoes the raw `remote.*.url`
   (mirror creds included) into three `DoctorFinding`s** (`crates/reposix-cli/src/doctor.rs:440/444/454/456`).
   Exact SAME leak class as WR-02/WR-03, closed there with `redact_userinfo`, but left raw here.
   `doctor.rs` is P120's deliberate teach-scan exception (structured report), so the scanner never saw
   it, and it is outside the WR-01/02/03 set — hence genuinely out of P120's charter, NOT an SC gap.
   Lower severity than the fixed trio: the token is the user's own, rendered in their own `reposix
   doctor` output (no git-triggered/audit-row exfil leg). Sketch + fix in the intake entry.

2. **[LOW — accepted, well-documented] WR-01 fixture token is intentionally non-`ghp_`-prefixed**
   (`mirror-pushtoken-REDACTME-abc123def456`) to avoid tripping the pre-existing `cred-hygiene.sh`
   `ghp_[A-Za-z0-9]{20,}` P0 pre-push pattern (commit `2af20e15`; in-test comment `bus_handler.rs:711-719`;
   follow-up `GTH-V15-67` filed for an inline test-fixture allow-marker so redaction tests can use
   realistic `ghp_`-shaped fixtures). `redact_userinfo` strips URL userinfo STRUCTURALLY (format-agnostic
   to the token shape), so a non-prefixed fake exercises the identical redaction path — the fix holds;
   this is a fixture-realism nit, not a coverage gap. GTH-V15-67's absence is (per the grading brief)
   a DELIBERATE filed-not-implemented item pending owner review, NOT a P120 gap.

3. **[LOW — noted in REVIEW.md, coverage nit] The W4 atlassian-arm integration regression**
   (`malformed_credentialed_bus_url_does_not_leak_userinfo`) drives the generic "unrecognised backend"
   arm (`evil.example.com`), not the atlassian-specific `Some(other)`/`None` arms it was framed around.
   Those arms redact identically via the same `redact_userinfo`, which IS unit-tested directly, so the
   fix holds — a test-targeting nit, not a leak.

4. **[INFO] IN-01 (from REVIEW.md): some recovery lines carry `<name>`/`<backend>` placeholders**
   (`bus_url.rs`, `errors.rs`) — git-help convention, not literally paste-and-run. Filed GTH-V15-63.
   Several sites already pair a placeholder with a concrete example (`errors.rs:44`). Acceptable.

5. **[POSITIVE] Test-name-honesty + teach-exempt markers across the WR diff are accurate.** Every WR
   test's `// test-name-honesty: ok` annotation truthfully names what it drives and asserts (verified
   against each body). The WR-01 test's honesty note about the persisted-audit-row leg being
   "load-bearing" is correct — stderr alone would not catch a durable-table leak. The two security
   wins (W4 atlassian-origin + WR-01 audit-row) are genuine regressions caught with same-commit
   coverage, worth naming at the v0.15.0 RETROSPECTIVE (OP-9), per HANDOVER §5.4.

---

## Filed to intake this session

- `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md` — appended (UNCOMMITTED) the MEDIUM
  `doctor.rs` raw-`remote.*.url` credential-echo entry (Noticing #1), with severity + sketched fix +
  out-of-scope rationale. STATUS: OPEN. Left uncommitted for the coordinator to bundle with this verdict.

---

_Verified: 2026-07-17_
_Verifier: Claude (gsd-verifier) — unbiased phase-close, no stake in a GREEN outcome_
_Method: goal-backward; every PASS backed by a RUN test / re-derived `git log` / executed scanner / `gh` CI conclusion_
