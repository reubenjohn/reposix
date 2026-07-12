# v0.13.0 Surprises Intake (P96 source-of-truth) — Part 5 of 7

> Split from `SURPRISES-INTAKE.md` for the file-size gate (OP-8 drain). Index: `../SURPRISES-INTAKE.md`. Entries preserved verbatim.


**2026-07-07 wrap-up (release-gate closure on PR #68's green run, no new diagnostic evidence):**

**Context:** all required checks on PR #68 (head `14bb5e43d7ff9552245dae6f3b47caeaece4ea1f`) went
GREEN, including `quality gates (pre-pr)`
(https://github.com/reubenjohn/reposix/actions/runs/28838198234/job/85526336500 — the same run
documented in the follow-up #2 entry above: 70 PASS / 1 unrelated pre-existing FAIL
(`docs-build/p94-badges-real-vs-transient`) / 1 WAIVED cadence, exit=0). Re-verified via
`gh pr checks 68` at wrap-up time — full 22-check matrix PASS, no regressions.

**What this confirms and does NOT confirm:** the release is unblocked ON THIS RUN'S EVIDENCE —
`crlf_blob_body_round_trips_byte_for_byte` did not fail, so there is no required-check red gating
PR #68. This is NOT a root-cause fix and NOT new diagnostic evidence: hypothesis A vs. B (CRLF
assert at `protocol.rs:216-219` vs. the `stdout.contains("ok refs/heads/main")` check at
`protocol.rs:203-206`) remains exactly as unresolved as the prior follow-up #2 left it, and local
reproduction is still 0/7+ attempts across two sessions. The underlying intermittent
CI-environment-specific flake is unresolved and could recur on a future CI run for this or any
other PR that exercises the same `quality gates (pre-pr)` job.

**Recommendation:** keep **STATUS: OPEN**, severity unchanged at **HIGH** (no rationale to
downgrade the severity itself — the failure mode, when it does recur, is still an unexplained
required-check red). Reframe the urgency posture only: this entry is **monitor — not
release-blocking on a green run; revisit if it recurs.** Do not close this entry on the strength of
a single non-reproducing green run; closing requires either (a) a live recurrence captured with the
`fbe5bee` full-log diagnostics and a confirmed root cause, or (b) enough consecutive green runs
across independent PRs to justify a deliberate re-classification as environment-noise, which has not
been attempted yet.

**STATUS:** OPEN — HIGH — monitor (not release-blocking on a green run; revisit if it recurs). Root
cause still unconfirmed; hypotheses A/B both still open; next action unchanged from follow-up #2
(catch a live recurrence with `fbe5bee`'s uncapped log before any further truncation regression can
hide it again).

---

**2026-07-07 deep-dive (opus RBF debugging session — Hyp A CONFIRMED from the real CI backtrace; A/B disambiguated):**

**(A) Hypothesis A is CONFIRMED — it is the CRLF-preservation assert at `protocol.rs:216-219`, NOT the `stdout.contains("ok refs/heads/main")` check at :203-206.** Pulled the actual failing job log via `gh api repos/reubenjohn/reposix/actions/jobs/85523987205/logs` (run `28837407948`, the confirmed `quality gates (pre-pr)` FAILURE — note run `28819166220`'s pre-pr job actually SUCCEEDED; its failure was a rerun `85521914911`). The backtrace frame is unambiguous: `24: protocol::crlf_blob_body_round_trips_byte_for_byte at ./tests/protocol.rs:219:6` → the closing `);` of the `assert!(body_str.contains("line-one\\r\\nline-two\\r\\n"), ...)` block. The push SUCCEEDED (`ok` assert :204 passed) and a POST WAS issued (`.expect("a POST was issued")` :213 passed, else the panic would name :213). So: a Create POST fired, returned 201, but its captured body lacked the JSON-escaped CRLF substring. The actual `body=` diff is STILL not in this log — this job predates `fbe5bee`, so `dump_verifications.py`'s 40-line tail-window landed mid-backtrace and discarded the panic line. `fbe5bee` (now on main) fixes that for the NEXT recurrence.

**(B) Three prior hypotheses RULED OUT with evidence:**
1. *Timeout* — already dead (0.14s), re-confirmed.
2. *Dependency-version drift (serde_json / wiremock / http)* — DEAD. `git show deee8fd:Cargo.lock` (the failing PR #61 head) vs HEAD `Cargo.lock` are BYTE-identical for wiremock 0.6.5, hyper 1.9.0, http 1.4.0, serde_json 1.0.150, tokio 1.52.3, h2 0.4.13, want 0.3.1. Same code, same deps.
3. *Shared on-disk cache race corrupts the POST body* — DEAD by code analysis. The four wiremock export tests DO collide on one cache (`resolve_cache_path` keys only on `<backend-slug>-demo.git`; none set `REPOSIX_CACHE_DIR`), BUT the Create's body bytes come DIRECTLY from in-memory `parsed.blobs[mark]` (diff.rs:228-236 → main.rs:484-495), never re-read from the shared bare repo / cache.db. `frontmatter::parse` slices the body by byte offset (record.rs:187/213-218 — no `.replace`/`.lines()`) and `render` pushes it verbatim (record.rs:148); `parse_export_stream` captures the blob via `read_exact(len)` (fast_import.rs:183-184). Every deterministic path preserves CRLF byte-exact. Concurrent cache access CANNOT alter the POSTed body. (The shared cache IS a real test-isolation / OP-4 "no hidden state" defect that precheck reads — but precheck poisoning would fail at :199/:204/:213, never :219, so it does not explain THIS failure.)

**(C) Local reproduction: ~87 GREEN runs across three sessions, zero failures.** This session added: 30× single-core (`taskset -c 0`, `--test-threads=4`, shared cache, cleaned per iter) + 20× full gate-replica (build both `--test stateless_connect_e2e --test protocol` binaries, run in gate order sharing one cache, pinned to 2 cores `taskset -c 0-1` to mimic the 2-vCPU runner). 0/50 this session, 0/37 prior.

**(D) Leading hypothesis (now the ONLY one consistent with all evidence): intermittent INCOMPLETE POST-body capture by `wiremock`/`hyper`'s `received_requests()` under CI CPU starvation — a TEST-HARNESS artifact, not a reposix byte-handling bug.** If, under 2-vCPU contention, the recorded request body is occasionally empty/truncated, `std::str::from_utf8(&post.body)` still succeeds (empty/partial is valid UTF-8), `body_str` is empty/partial, and `.contains(CRLF)` fails at exactly :219 — matching the backtrace. This is consistent with: body is provably byte-exact in-memory, deps identical, and total local non-reproduction. The `test` job never fails because `cargo test --workspace` schedules protocol.rs among hundreds of tests (different load profile) whereas the gate runs only these two files.

**(E) DATA-INTEGRITY VERDICT for shipped v0.13.0: NO corruption risk, no urgent point-release needed.** The failure requires the parallel-test harness. In production, one `git push` = one `git-remote-reposix` process per `(backend, project)`; the body is built from in-memory parsed fast-import bytes with zero line-ending normalization anywhere in the write path. A real push does NOT mangle blob bytes. The only real cost is release-gate flakiness.

**Next experiment (single most valuable, NOT yet done — budget-gated):** push a throwaway `debug/crlf-capture` branch that loops the CRLF push ~60× per run with UNCONDITIONAL `eprintln!("DEBUG POST body = {body_str:?} len={}", post.body.len())`, open a PR to fire `quality gates (pre-pr)`, and read the captured body on the first failing iteration. If `body=` is empty/short → confirms (D), fix = poll `received_requests()` until the POST body is non-empty (wiremock timing), a test-harness fix. If `body=` shows `line-one\nline-two\n` (LF only) → a genuine reposix `\r`-stripping bug, re-open at BLOCKER. Prior sessions never ran this because they hadn't disambiguated A/B; that is now done.

**STATUS:** OPEN — HIGH — monitor. Hyp A confirmed (`:219`, CRLF assert). Timeout + dep-drift + shared-cache-body-corruption all ruled out with artifacts. Leading cause: wiremock/hyper incomplete-body capture under CI load (test-harness-only; no shipped-data risk). Definitive close still requires the `debug/crlf-capture` loop to capture the real `body=` string.

## S-260707-pr-01 — `reposix-sim` ships in NO prebuilt distribution; documented 3-crate binstall installs ZERO binaries (BLOCKER)

**Found during:** v0.13.0 post-release verification (zero-shot docs-following fleet).
**Severity:** BLOCKER.
**Issue:** The release tarball/zip contains only `reposix` + `git-remote-reposix`
(`tar tzf` confirmed against the published release asset); the Homebrew formula installs
only those two binaries (`.github/workflows/release.yml:465-466`); `cargo binstall ...
reposix-sim` fails with no prebuilt archive available (exit 94). Because `cargo binstall`
is all-or-nothing across its argument list, the documented tutorial command `cargo
binstall reposix-cli reposix-remote reposix-sim` installs **zero** binaries when any one
crate lacks a prebuilt archive. The CLI resolves `reposix-sim` as a sibling of
`current_exe()`, so the default simulator backend — the entire first-run tutorial path —
is unreachable for every non-source install band (curl installer, PowerShell installer,
Homebrew, `cargo binstall`).
**Sketch:** Either (a) add `reposix-sim` to the release build's distributed binaries
(release.yml dist-binaries list + installer scripts + Homebrew formula's `install` block),
or (b) stop advertising sim-backed onboarding to binary-install users and gate the
sim-tutorial path behind a from-source checkout, with docs updated accordingly. Option (a)
is the north-star fix — the sim is OP-1's designated first-run default.

## S-260707-pr-02 — `reposix init` hides a fatal fetch failure behind exit 0, then hands out a broken "Next:" hint (HIGH)

**Found during:** v0.13.0 post-release verification (zero-shot docs-following fleet).
**Severity:** HIGH.
**Issue:** When the sim backend is unreachable, `reposix init sim::demo <path>` prints a
`fast-import` / `backend-unreachable` crash and leaves a `.git/fast_import_crash_*` file
behind, yet the process exits **0** and still prints `Next: git checkout origin/main` — a
command that then fails with `pathspec 'origin/main' did not match any file(s) known to
git` because the fetch never populated `origin/main`. A zero-shot user (or agent) reading
only the exit code and the "Next:" line has no signal that init actually failed.
**Sketch:** Exit non-zero when the underlying fetch/import fails; suppress (or caveat) the
"Next:" hint on the failure path; clean up (or explicitly reference) the
`fast_import_crash_*` file so the user isn't left with a silent turd in `.git/`.

## S-260707-pr-03 — `reposix sim` fallback shells out to `cargo run -p reposix-sim` from a release binary (HIGH)

**Found during:** v0.13.0 post-release verification (zero-shot docs-following fleet).
**Severity:** HIGH.
**Issue:** When `reposix-sim` isn't found on `PATH`, `reposix sim` falls back to invoking
`cargo run -p reposix-sim`, which fails with `could not find Cargo.toml` because a
packaged release binary has no workspace to build from. A shipped binary must never reach
for `cargo`/`Cargo.toml` — that fallback only makes sense in a source checkout.
**Sketch:** In release builds, replace the cargo fallback with a teaching error (e.g.
`reposix-sim not found on PATH; install it via <x>`, once S-260707-pr-01 ships a prebuilt
`reposix-sim`) rather than a bare cargo-not-found failure.

## S-260707-pr-04 — reposix-swarm integration tests share one on-disk cache + sqlite path across parallel threads/binaries (MEDIUM)

**Found during:** v0.13.0 post-release verification (crlf-investigation lane, contributing
factor surfaced while chasing S-260707-rbf-01).
**Severity:** MEDIUM.
**Issue:** Integration tests share one on-disk `~/.cache/reposix/<slug>-demo.git` +
sqlite path across parallel test threads AND across the gate's two separate test
binaries — an OP-4 (no-hidden-state) violation and a latent flake source; it was a
contributing factor the crlf lane surfaced (not itself the crlf root cause).
**Sketch:** Give each test a unique `REPOSIX_CACHE_DIR` (e.g. a `tempdir` per test) so
concurrent test runs never share cache/sqlite state.

## S-260707-pr-05 — `reposix init`/`doctor` write cache state to `~/.cache/reposix/` unconditionally, with no documented override (MEDIUM)

**Found during:** v0.13.0 post-release verification (zero-shot docs-following fleet).
**Severity:** MEDIUM.
**Issue:** `reposix init`/`doctor` write cache state to `~/.cache/reposix/` unconditionally
with no CWD scoping, causing collisions between concurrent runs or testers on a shared
machine. `REPOSIX_CACHE_DIR` exists as an override but is undocumented — it doesn't appear
in `--help` output or in the docs site.
**Sketch:** Document (and confirm honoring of) a per-invocation cache-dir override; surface
`REPOSIX_CACHE_DIR` in `--help` text and in the relevant docs/reference page.

## S-260707-pr-06 — `git-remote-reposix` has no `--version`; installer.sh has no arg validation; `REPOSIX_INSTALL_DIR` undocumented (LOW)

**Found during:** v0.13.0 post-release verification (zero-shot docs-following fleet).
**Severity:** LOW.
**Issue:** (a) `git-remote-reposix` has no `--version` flag, inconsistent with
`reposix --version`. (b) `installer.sh` has no `--help`/argument parsing — it silently
runs a full install on any argument passed to it. (c) `REPOSIX_INSTALL_DIR` works as an
install-location override but is undocumented.
**Sketch:** Add `--version` to `git-remote-reposix`; add basic arg validation (and a
`--help`) to `installer.sh`; document `REPOSIX_INSTALL_DIR` alongside the install
instructions.

## S-260707-pr-07 — `docs-build/p94-badges-real-vs-transient` catalog row reported stale-NOT-VERIFIED while the underlying gate needs live re-verification (LOW)

**Found during:** v0.13.0 post-release verification, Task D (catalog-row spot-check).
**Severity:** LOW.
**Issue:** The row `p94-badges-real-vs-transient` in `quality/catalogs/docs-build.json`
was reported as stale/NOT-VERIFIED at hand-off time; the report claimed "the gate runs
green live." Actually running `bash quality/gates/docs-build/p94-badges-real-vs-transient.sh`
in this verification session gives **exit=1** (FAIL), not green:
```
PASS: badges-resolve.py re-run on >=2 spaced occasions; pass/fail pattern recorded (3 runs)
FAIL (docs-build/p94-badges-real-vs-transient): GOOD-TO-HAVES badges-resolve entry is not RESOLVED (still OPEN or missing)
```
The gate's own logic requires the `GOOD-TO-HAVES.md` `badges-resolve` entry to carry
STATUS: RESOLVED before it will pass; that entry has not been resolved, so the "runs
green live" claim does not hold as of this session. Per the task's own guard ("never
fabricate a PASS the gate didn't produce"), the catalog row was **not** flipped.
**Sketch:** Either resolve the `badges-resolve` GOOD-TO-HAVES entry (determine
real-vs-transient, record the finding, flip its STATUS to RESOLVED) so the gate's own
precondition is satisfied, or correct whoever/whatever asserted "runs green live" — the
claim was stale/wrong at verification time. Only then update the catalog row's status +
`last_verified` with the dated evidence.

**RESOLVED (2026-07-07, v0.13.1 LANE F1b):** Root cause was NOT a stale catalog row nor
a transient badge flake — the `badges-resolve` GOOD-TO-HAVES entry (which the verifier
reads as its assert-2 precondition) had been accidentally deleted by `1b37350` ("prune
v0.13.0 intakes to open-only"), so the gate's regex found "OPEN or missing". Restored the
entry in RESOLVED form (P94 D3 TRANSIENT verdict + retry/backoff fix already landed
2026-07-05). `bash quality/gates/docs-build/p94-badges-real-vs-transient.sh` now exits 0
(all 4 asserts PASS; live badges-resolve.py: 8/8 URLs HTTP 200, attempts:1). The network
was reachable — not egress-blocked.

## S-260707-pr-08 — agent "worktrees" are NOT isolated; a sim-seed leaf corrupted the shared repo (`t <t@t>` flipped `core.bare=true`) (HIGH)

**Found during:** 2026-07-07 orchestration session — a dispatched sim/seed leaf corrupted
the shared local repo `/home/reuben/workspace/reposix` (repaired twice this session).
**Severity:** HIGH.
**Issue:** Agent "worktrees" share the coordinator's `.git/config` + object store — they
are NOT isolated from the shared repo — and a leaf's cwd resets to the repo root between
Bash calls. A sim-seed leaf whose `reposix init` / seed / `git commit` / `git config`
did not `cd` into its `/tmp` target dir *within the same Bash invocation* therefore ran
against the real shared repo: it committed under the sim-fixture identity `t <t@t>` and
flipped `core.bare=true`, breaking the shared checkout for every concurrent agent. This
is systemic — any future setup leaf that assumes worktree isolation or durable cwd can
reproduce it. A prose hard-stop now lives in `.planning/ORCHESTRATION.md` § Leaf
isolation, but there is no *enforced* guardrail yet.
**Sketch:** Enforce isolation, don't just document it: (a) each setup leaf gets an
isolated `/tmp` clone + a distinct `REPOSIX_CACHE_DIR`; (b) a `.claude/hook` (pre-commit
or the existing stop-uncommitted family) that REJECTS any commit authored by `t <t@t>`
(or any known test-fixture identity) against the shared repo; and/or (c) a guard that
fails a leaf's `reposix init`/seed when `$PWD` is inside the shared repo tree. Route to
the enforcement map once shipped (`ORCHESTRATION.md` § Enforcement map).

