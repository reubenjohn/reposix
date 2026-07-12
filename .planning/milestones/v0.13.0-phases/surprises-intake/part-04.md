# v0.13.0 Surprises Intake (P96 source-of-truth) — Part 4 of 7

> Split from `SURPRISES-INTAKE.md` for the file-size gate (OP-8 drain). Index: `../SURPRISES-INTAKE.md`. Entries preserved verbatim.

## S-260706-rbf-02 — CONSULT-DECISIONS T1 entry has an empty Commit field (LOW / cosmetic)

**Found during:** quick 260706-rbf.
**Severity:** low / cosmetic.
**Issue:** `.planning/CONSULT-DECISIONS.md` T1 tag-timing entry (~line 75) has `**Commit:** (this
entry; handover encodes the sequencing)` — no real SHA for a load-bearing sequencing decision.
**Sketch:** backfill the SHA of the commit that recorded the T1 decision. Ledger hygiene.

## S-260707-rbf-01 — `crlf_blob_body_round_trips_byte_for_byte` intermittent red on PR #61's `quality-pre-pr` job (HIGH / unresolved, non-timeout assertion failure — upgraded from MEDIUM 2026-07-07)

**Found during:** release-gating investigation for PR #61 (v0.13.0 release-plz branch), CI run
28819166220 / rerun job 85521914911.

**What was found:** `crates/reposix-remote/tests/protocol.rs::crlf_blob_body_round_trips_byte_for_byte`
failed inside the `quality gates (pre-pr)` job (via `quality/gates/agent-ux/p94-git243-fallback-sentinel.sh`
Arm 2's `CARGO_BUILD_JOBS=2 cargo test -p reposix-remote --test stateless_connect_e2e --test protocol`),
while the separate `test` job (`cargo test --workspace --locked`, full/unthrottled) passed on the same
commit. The test is a pure in-memory byte-assertion (ephemeral `wiremock::MockServer`, no real git
checkout/autocrlf path) — provably immune to the checkout-level causes CRLF bugs normally come from.

**Reproduction attempted (6 runs, all GREEN — mechanism NOT confirmed):**
1. `cargo test -p reposix-remote --test protocol` alone x3 — all green.
2. The gate's exact invocation (`CARGO_BUILD_JOBS=2 cargo test -p reposix-remote --test
   stateless_connect_e2e --test protocol`, warm `cargo build --workspace --bins --quiet` first) x3 —
   all green.
3. Same exact invocation with the full CI env matched (`CARGO_TERM_COLOR=always RUST_BACKTRACE=1
   CARGO_INCREMENTAL=0 RUSTFLAGS="-D warnings"`) x3 — all green.
4. Manual source review of `protocol.rs`: the CRLF test and its neighbors share no static/global
   state, no fixed port (each uses `MockServer::start()` — ephemeral loopback port), no shared temp
   dir. No plausible in-file race identified.
5. `--locked` divergence ruled out: the gate's unlocked, throttled, warm-build invocation reproduces
   cleanly every time locally; no dependency-resolution drift observed.

**Real root cause NOT confirmed** — could not reproduce the failure in this sandbox under any of the
above conditions. Leading (unconfirmed) hypothesis: CI-runner resource contention. The failing test is
the only one in the file that combines a real ephemeral TCP `wiremock` server + `tokio::task::spawn_blocking`
+ a real subprocess spawn, all under an `assert_cmd` `Command::timeout` (was 15s) — on a shared 2-vCPU
GH Actions runner running many tests concurrently (default thread-parallel `cargo test` across 9+3
tests), an occasional CPU-starved subprocess could plausibly blow a 15s wall-clock budget without any
actual byte-level regression.

**Why out-of-scope for eager full resolution:** the actual CI panic/assertion message for the failing
run was itself lost — `p94-git243-fallback-sentinel.sh` piped the full cargo-test log through
`tail -25` before printing it, and the tail window landed entirely inside the backtrace footer,
never reaching the panic line or assertion diff. Confirming the true mechanism requires catching a
live recurrence with full diagnostics, which this session could not force to reproduce.

**Eager-fixed in place (commit — see below), no new dependency:**
1. `quality/gates/agent-ux/p94-git243-fallback-sentinel.sh` — the cargo-test failure branch now
   archives the *full* log to `quality/reports/verifications/agent-ux/p94-git243-fallback-sentinel-cargo-test-failure.log`
   (gitignored) before truncating, and greps for `panicked at` / `failures:` / `assertion...failed`
   context plus a 60-line tail (previously 25, and *only* the tail) — the next occurrence will carry
   an actual diagnosable message instead of a bare backtrace footer.
2. `crates/reposix-remote/tests/protocol.rs` — bumped the 4 `wiremock`-backed `Command::timeout`
   calls (the CRLF test + the two H-03 500-response tests + the H-02 non-UTF-8 test) from 15s to 30s
   as a defensive headroom increase against shared-runner contention. This does NOT touch any
   assertion or expected value — only the wall-clock budget for a real subprocess+network round trip.

**Sketched resolution (if it recurs):** with the new full-log archive + panic-line grep in place, the
next CI recurrence will surface the actual panic/assertion text. If it turns out to be a genuine
timeout (subprocess killed mid-test), the 30s bump already applied may fully resolve it; if the fuller
log instead reveals a genuine logic bug, re-open with that evidence. Consider adding
`--test-threads=1` to the gate's cargo invocation if the doubled timeout still flakes, to remove
thread-parallelism as a contention vector entirely (narrower fix pending confirmed evidence, not
applied here since no shared in-file resource was identified to justify it up front).

**STATUS:** OPEN — mitigated (log capture + timeout headroom) but root cause unconfirmed; needs a
live recurrence with the new diagnostics to close definitively.

---

**2026-07-07 follow-up (RBF investigation, commit `aa2b33d`'s timeout-bump fix retested):**

**(a) Timeout-bump fix proven ineffective.** PR #61 (now head `deee8fd`, run `28837407948`, job
`85523987205`) hit the SAME failure again on `quality gates (pre-pr)` AFTER the 15s→30s timeout
bump from `aa2b33d` landed. `test result: FAILED. 8 passed; 1 failed; ... finished in 0.14s` —
the whole `protocol.rs` test binary (all 9 tests) finished in 140ms, nowhere near either the old
15s or new 30s timeout. **The timeout theory is dead.**

**(b) Confirmed a real, non-timeout assertion failure, not a race.** `timed_out: False` in the
verifier's own JSON, and the failing test's backtrace (frame 24/25) points straight at
`tests/protocol.rs:219:6` inside `crlf_blob_body_round_trips_byte_for_byte` (called from its
`{{closure}}` at line 154, the `tokio::test` body) — i.e. one of the two `assert!` calls after the
push succeeds (`stdout.contains("ok refs/heads/main")` at :203-206, or the CRLF-preservation check
at :216-219) is false. 0.14s wall-clock rules out any subprocess-hang / contention-timeout
explanation for either the original 15s or the bumped 30s budget.

**(c) New diagnostic finding: the actual panic/assertion message is STILL invisible in CI, and
now we know exactly why.** `quality/runners/dump_verifications.py` had `_TAIL_LINES = 40` —
it prints only the LAST 40 lines of each verifier's captured `stderr`. But
`p94-git243-fallback-sentinel.sh`'s failure branch (the "fix" from `aa2b33d`) prints
`grep -n -B2 -A15 'panicked at|failures:|assertion.*failed'` (containing the real panic text)
FOLLOWED BY a separate `tail -60` of the raw log. A 40-line tail-window falls entirely inside
that trailing 60-line block, so the grep context — the only place the actual panic message and
`body=...` diff lives — is discarded before it ever reaches the CI log. Confirmed by grepping the
full raw job log for `panicked`/`thread '`/`CRLF`/`body=` — zero matches anywhere in the printed
output, even though the gate script's own grep command (if its match had survived truncation)
would have caught it. **Fixed in commit `fbe5bee`** (pushed to `main`): bumped `_TAIL_LINES` to
200 with a comment explaining the truncation-window bug. This does not fix the underlying test
failure, but the NEXT CI recurrence (on any branch rebased past `fbe5bee`) will finally surface
the real assertion text.

**(d) Local reproduction still elusive.** Repeated the CI job's exact sequence again in this
session (`cargo build --workspace --bins --quiet` then `CARGO_BUILD_JOBS=2 cargo test -p
reposix-remote --test stateless_connect_e2e --test protocol`, 1x combined + 5x isolated single-test
re-runs) — 6/6 GREEN locally, consistent with the prior investigation's 6/6 GREEN. The failure
appears to require the actual GitHub Actions runner environment (2-vCPU `ubuntu-latest`,
`Swatinem/rust-cache`-restored `target/`) to manifest; it has never reproduced in this sandbox
despite matching every documented aspect of the CI sequence, including env vars.

**Updated hypothesis:** given (b) rules out contention/timeout and the failure is fast and
deterministic-looking within a given CI run, the leading candidate shifts to either (i) a genuine
environment-dependent behavior difference (e.g. JSON string-escaping of `\r`/`\n` differing by
`serde_json` version, or a `wiremock`/`http`-stack difference) between whatever dependency
versions the CI runner's `Swatinem/rust-cache`-restored lockfile-driven build resolves vs. this
sandbox's already-built `target/`, or (ii) an assertion race specific to `stdout.contains("ok
refs/heads/main")` (line 203-206) rather than the CRLF assertion at line 219 itself — the
backtrace only proves the panic unwound through the `#[tokio::test]` body, not which specific
`assert!` fired first, and dump_verifications' truncation swallowed the disambiguating line.
**This distinction is now resolvable** on the next CI recurrence once `fbe5bee` is on the branch
under test.

**Severity raised to HIGH:** this blocks PR #61 (v0.13.0 release, `quality gates (pre-pr)` is a
required check) and is release-blocking per the settled GO/NO-GO cadence until the real assertion
text is captured and either fixed or the test is proven backend/environment-specific.

**STATUS:** OPEN — HIGH. Timeout theory dead; root cause still unconfirmed but now diagnosable
(log-truncation bug fixed in `fbe5bee`). Next action: re-run `quality gates (pre-pr)` on a branch
rebased past `fbe5bee` and read the now-uncapped verifier stderr for the actual panic/assertion
message.

---

**2026-07-07 follow-up #2 (post-`fbe5bee` re-run on PR #68, non-reproduction):**

**Context:** PR #68 (branch `release-plz-2026-07-07T02-37-20Z`, head `14bb5e43d7ff9552245dae6f3b47caeaece4ea1f`,
already includes `fbe5bee`'s 200-line tail-window fix) had its `CI`/`Security audit`/`quality gates
(pre-pr)` workflows re-triggered via a real-actor `gh pr close 68` / `gh pr reopen 68` (the
`pull_request`-trigger silently not re-firing after a release-plz branch regen is itself now filed
as a new process good-to-have, see `GOOD-TO-HAVES.md` 2026-07-07). Watched CI run `28838198234`
(the `CI` workflow) to completion; the `quality gates (pre-pr)` job runs as run `28838198234` /
job `85526336500`.

**Result: the flaky test did NOT reproduce this run.** The `quality gates (pre-pr cadence)` step's
full output shows `[PASS ] agent-ux/p94-git243-fallback-sentinel (P1, 15.49s)` — the verifier that
drives `CARGO_BUILD_JOBS=2 cargo test -p reposix-remote --test stateless_connect_e2e --test
protocol` (which includes `crlf_blob_body_round_trips_byte_for_byte`) completed GREEN in 15.49s.
Overall job summary: `70 PASS, 1 FAIL, 0 PARTIAL, 1 WAIVED, 0 NOT-VERIFIED -> exit=0` — the single
FAIL is the pre-existing, unrelated, non-blocking P2 row `docs-build/p94-badges-real-vs-transient`
(already tracked separately in `GOOD-TO-HAVES.md`'s "badges real-vs-transient flap" note), not the
CRLF test. Grepped the full job log for `panicked`, `assert.*failed`, `crlf_blob`,
`protocol.rs:2`, `body=`, `FAILED`, `failures:` — zero matches anywhere in the archived output,
because the test simply passed; there was no failure branch to trigger the `fbe5bee` full-log
archive or the panic-line grep this time.

**What this means:** the `fbe5bee` log-capture fix is still unverified in the one way that matters
(catching a live recurrence) because this CI run happened to be green. Hypothesis A vs. B
(CRLF-preservation assert at `protocol.rs:216-219` vs. the unrelated `stdout.contains("ok
refs/heads/main")` check at `protocol.rs:203-206`) remains **UNRESOLVED** — no new evidence either
way. Source-line mapping re-confirmed by direct read of `crates/reposix-remote/tests/protocol.rs`
lines 195-220 on this same commit: the `stdout.contains("ok refs/heads/main")` assert is at lines
203-206 (`"stdout missing ok: {stdout}"`), and the CRLF-preservation assert is at lines 216-219
(`body_str.contains("line-one\\r\\nline-two\\r\\n")`, `"POST body did not preserve CRLF — raw-bytes
path stripped \\r; body={body_str}"`) — both spans unchanged since the original filing, so the
2026-07-07 follow-up #1's line-number-to-source mapping (`protocol.rs:219:6` in a *previous* CI
backtrace) still points at the CRLF assert specifically, not the earlier `ok refs/heads/main` check,
IF that previous backtrace frame is accurate. This run adds no independent confirmation of that
frame; it only re-verifies the source stayed put and that the log-capture fix has not yet been
exercised against a real failure.

**STATUS:** OPEN — HIGH. Root cause still unconfirmed; `fbe5bee`'s diagnostic fix remains
unverified-in-anger (this run was green, not a recurrence). Per prove-before-fix on BLOCKERs, no
fix attempted. Next action: keep watching subsequent CI runs on this PR (or the next release-plz
regen) for a genuine recurrence, then immediately pull the job log before any further truncation
regressions can hide it again.

---
