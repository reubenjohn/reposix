# 126-HANDOVER.md — P126 "Docs-alignment tooling polish" C2 relief handover, 2026-07-19

Written by the outgoing **P126 wave-4 C2 phase-coordinator** (seat #68), relieving at
~130k own context. Successor is also a **P126-tier C2 phase-coordinator** (wave-5),
reporting to **L0** — SendMessage does NOT work C2→child/child→C2 at this tier (ratified
standing doctrine, `.planning/ORCHESTRATION.md` §3/§11); every close moves through FRESH
verifier→executor LEAVES, never fork-to-resume, never self-watch CI.

**Required reading order:** (1) this file in full, (2) `git log --oneline -20` +
`gh run view 29688725032` to reconfirm §1 is still current (state below was gathered live
this rotation, not carried from a stale snapshot — but a further push by anyone else
between this handover and your first action would move ground truth), (3)
`quality/CLAUDE.md` § "Backgrounded-process fd convention" + § "`minted_at` also arms the
GRADE-TIME F-K4b congruence gate" (both directly relevant to §1/§4 below), (4)
`.planning/ORCHESTRATION.md` §3 "Liveness doctrine" (you will hit the push→CI boundary if
you push a further fix).

**Do-not-touch guardrails:** do NOT push (local `main` already **equals** `origin/main`
at `ba13553f` — there is nothing to push yet; a push only happens after you land a fix for
§1's NEW finding below); do NOT re-diagnose the P0 F-K4b regression as still open — it is
**CONFIRMED FIXED** (see §1); do NOT treat CI run `29688725032` as "in-flight" — it has
**CONCLUDED FAILURE**, see §1 for why this deviates from what this coordinator was
originally briefed to expect; do NOT touch `docs-build/badges-resolve` /
`docs-build/p94-badges-real-vs-transient` / `docs-build/animation-renders` — standing
known-transient/owner-gated P2s, verify-by-reobservation only, never code-fixed.

---

## 1. Ground truth (git)

Verified live this rotation via `git log`, `git status`, `gh run view` — not carried from
a stale snapshot.

- **HEAD = `ba13553f3a7bceaff8ceac73250ef083ddacecd4`.** `main`, tracking `origin/main`,
  **0 ahead / 0 behind** (already pushed — confirmed via `git branch -vv`). `git status
  --porcelain` → clean.
- **Full P126 commit trail**, newest first (all landed + pushed):
  - `ba13553f` — fix(126-01): defuse F-K4b grade-time demote on `real-git-push-e2e` (P0 CI
    regression from `65e8c497`'s `minted_at` add). Removed the mutually-exclusive
    git<2.34→NOT-VERIFIED entry from `expected.asserts` in `quality/catalogs/agent-ux.json`
    (F-K4b is PASS-path-only; that branch bypasses it entirely via exit 75, never belonged
    in the PASS claim set). Added `test_audit_field.py::TestFK4bMutuallyExclusiveBranch`
    (RED repro + GREEN lock + live-catalog guard), wired to pre-push via
    `structure/asserts-congruence-grade-time`. Documented in `quality/CLAUDE.md` § Honesty
    rules (fix-twice).
  - `dc60cc21` — docs(126-01): file GTH-V15-94..98 + verdict.py PROTOCOL fix-twice note.
    **This is the commit whose CI run (`29687639465`) first went RED**, surfacing the P0
    regression `ba13553f` fixes.
  - `7f70b0de` — docs(126-01): close P126 — SUMMARY + STATE/ROADMAP/REQUIREMENTS advance +
    `docs/roadmap.md` strip refresh + landmine intake CLOSED bookkeeping.
  - `588c1546` — W5 RAISE-3: fixed stale `docs/development/roadmap.md` active-milestone
    claim IN-PLACE + re-cited 5 doc-alignment rows (NOT deleted — deletion strands the
    human-gated confirm-retire step).
  - `639ff67f`, `1ef508bf` — W4 micro-batch (DRAIN-16/19/15-in-repo + 3 git-version comment
    refreshes).
  - `1df18239`, `e8823049`, `0270f91c` — W3 Lane C (DRAIN-20 `waived_active` counter +
    `RETIRE_PROPOSED` walk `row=<id>`; DRAIN-21 out-of-eligible warnings 17→2, coverage
    0.165→0.180 floor PASS; clippy).
  - `e693deeb`, `d093bc7f` — W2 Lane B (DRAIN-18 grader binds only on drift-fails-test;
    DRAIN-17 walk BLOCK names blocking row-STATE(s)).
  - `5d097937`, `d0753ef6`, `65e8c497`, `44783ebe` — W1 EARLY-LANE landmine: RED repro →
    `minted_at` fix + `save_catalog(persist=)` read-only guard → fix-twice doc → DP-2
    review PASS (mechanism-vs-symptom).
  - `d7a07b0f` — plan(126-01).
- **CI on `ba13553f` (run `29688725032`) has CONCLUDED — NOT in-flight, and NOT a clean
  green either.** This coordinator was originally briefed that this run was in-flight and
  would settle the open questions; it finished mid-rotation and the actual result is a
  **third, distinct failure signature** requiring fresh diagnosis. Ground truth, verified
  directly (`gh run view 29688725032`, `--log-failed`, `--json conclusion,status,jobs`):
  - **Overall conclusion: `failure`.** 14 of 15 jobs SUCCEEDED (clippy, gitleaks, rustfmt,
    shell-coverage, test, runner-unit-tests-hermetic, 4× integration-contract, dark-factory
    regression, coverage, bench-latency-v09). **Only `quality gates (pre-pr)` failed**
    (started 13:21:41Z, completed 13:27:37Z, 5m56s).
  - **GOOD NEWS, confirmed: the P0 F-K4b fix HOLDS on CI.**
    `agent-ux/real-git-push-e2e` ran and **PASSED in 4.28s** (log line: `-> START
    agent-ux/real-git-push-e2e (P0)` at 13:23:22Z, `[PASS ] agent-ux/real-git-push-e2e
    (P0, 4.28s)` at 13:23:26Z). `ba13553f` is a real, working fix — do not re-open it.
  - **NEW finding: the whole `run.py --cadence pre-pr` process was SIGKILLed mid-run.**
    Step log's last lines: `docs-build/badges-resolve` FAIL (30.03s, known-transient),
    `docs-build/p94-badges-real-vs-transient` FAIL (67.58s, known-transient),
    `docs-build/animation-renders` NOT-VERIFIED (known owner-gated stub) — then:
    ```
    .../ef65e2be-9f84-487c-b2fe-1ea2acb67f35.sh: line 1: 3075 Killed  timeout -k 30 1200 python3 quality/runners/run.py --cadence pre-pr
    ##[error]Process completed with exit code 137.
    ```
    The process died at 13:27:34Z — **only ~6 minutes** into the job, nowhere near the
    `timeout`'s own 1200s (20min) deadline, so this is **NOT** the fd-inheritance 28-minute
    hang class `cef3a2ea` fixed (that one times out at ~20.5min with a different shape).
    Exit 137 = SIGKILL delivered from OUTSIDE the `timeout` wrapper — most consistent with
    an OOM-kill by the runner's kernel, but this coordinator has no `gh`-CLI-accessible
    dmesg/memory-graph to confirm; **root cause is UNDIAGNOSED, this is the top open item**.
  - The catalog run died **before reaching** `docs-reproducible.json`,
    `freshness-invariants.json` (which contains `structure/hermetic-test-network-isolation`,
    the row this coordinator's brief expected to be the deciding question), or
    `security-gates.json`. **`structure/hermetic-test-network-isolation`'s CI status is
    therefore UNKNOWN — neither confirmed PASS nor FAIL on `ba13553f`**, not "still open" in
    the sense of a known FAIL. Locally on this dev box it runs clean: `bash
    quality/gates/structure/hermetic-test-network-isolation.sh` → PASS, 0.07s, exit 0 (this
    coordinator ran it directly to check; `ba13553f`'s diff does not touch that gate, its
    test, or `run.py`, so a local PASS is decent but NOT CI-conclusive evidence).
  - **Survivor process at forensic-dump time:** `ps auxf` (the `cef3a2ea`-added "Dump
    survivor process tree on failure" step) shows one orphaned `runner 11014 ...
    /home/runner/work/reposix/reposix/target/debug/reposix sim --bind 127.0.0.1:7878
    --ephemeral` (RSS ~18MB — too small to be the OOM cause by itself, but it is a genuine
    leaked backgrounded sim that some earlier row didn't clean up; worth a look but
    secondary to the SIGKILL root cause).
  - **Contrast with the PRIOR failing run `29687639465` (on `dc60cc21`):** that run ran to
    completion (clean `exit=1`, not killed) with an honest 5-FAIL summary — the P0 F-K4b
    regression (0.68s) + the 2 known-transient badge P2s + `structure/hermetic-test-network-
    isolation` itself FAILing fast at **0.02s** (sub-pytest-startup speed, i.e. a
    setup/import-time error, not the poisoned-proxy pytest actually running and failing).
    That is a DIFFERENT, now-superseded signature from this rotation's SIGKILL — do not
    conflate the two; both point at `quality gates (pre-pr)` but likely have different
    causes.

---

## 2. Wave/cycle state

| Wave | Scope | State | Commits |
|---|---|---|---|
| Plan | — | DONE | `d7a07b0f` |
| W1 (EARLY-LANE landmine) | minted_at load-crash defuse + read-only guard | DONE, DP-2 PASS | `44783ebe`,`65e8c497`,`d0753ef6`,`5d097937` |
| W2 (Lane B) | DRAIN-17/18 | DONE | `d093bc7f`,`e693deeb` |
| W3 (Lane C) | DRAIN-20/21 | DONE | `0270f91c`,`e8823049`,`1df18239` |
| W4 (micro-batch) | DRAIN-16/19/15-in-repo + git-version refresh | DONE | `1ef508bf`,`639ff67f` |
| W5 (RAISE-3) | stale roadmap fix-in-place | DONE | `588c1546` |
| W6 (close) | SUMMARY/STATE/ROADMAP/REQUIREMENTS + GTH filing | DONE | `7f70b0de`,`dc60cc21` |
| **Post-close P0 CI regression fix** | F-K4b grade-time demote defuse | **DONE, CONFIRMED on CI** | `ba13553f` |
| **Post-close SIGKILL diagnosis** | `quality gates (pre-pr)` exit 137 mid-run | **NOT STARTED — top priority** | — |
| Verifier close | fresh `gsd-verifier` catalog-row grade | **BLOCKED on the above** | — |

**Named-incident summary for the successor:** this is the THIRD distinct CI blocker this
phase has produced (F-K4b regression → FIXED; a 0.02s hermetic fast-fail on the prior run →
UNKNOWN if still live, unreached by the newest run; a mid-run SIGKILL on the newest run →
UNDIAGNOSED). Do not assume fixing one clears the others — verify each independently once
a clean full run exists.

---

## 3. Binding constraints (unchanged — embed verbatim in every dispatch)

- **ONE cargo invocation machine-wide, FOREGROUND-only** — never `run_in_background`
  (orphans the build, evades `cargo-mutex.sh`, OOM risk on this VM too). Prefer `-p
  <crate>`.
- **Leaf test setup runs in a `/tmp` clone, `cd`-ing in the SAME Bash invocation** — never
  the shared repo (`leaf-isolation-guard.sh` + pre-commit backstop + `reposix init`
  binary-side refusal).
- **Uncommitted = didn't happen.** Commit before ending any turn. No `--no-verify`, ever.
  One tree-writer at a time.
- **Push cadence:** `git fetch origin && git rebase origin/main` (re-check ahead/behind —
  other sessions push to main concurrently), then `git push origin main` **BEFORE** the
  verifier-subagent dispatch, then `python3 quality/runners/run.py --cadence post-push
  --persist` — P0 `code/ci-green-on-main` must show main's newest `ci.yml` run = success.
  **Never open the next phase over a red main.**
- **SendMessage NOT granted at C2 tier.** L0→C2 works; C2→child and child→C2 FAIL —
  serialize strictly, close via FRESH verifier→executor LEAVES, never fork-to-resume,
  never self-watch CI. At the push→CI-in-flight boundary: STOP, return to L0 the pushed SHA
  + run id(s) + "awaiting CI green." L0 holds the durable watch.
- **Model tiering:** opus for security/complex, sonnet default, haiku mechanical. Never
  fable at a leaf.
- **Commit-trailer format:** `Co-Authored-By: Claude Opus 4.8 (1M context)
  <noreply@anthropic.com>` on every non-trivial commit.
- **Catalog-first / mint discipline:** any legacy row whose `last_verified` crosses the P90
  cutoff needs `minted_at` in the SAME commit (`quality/CLAUDE.md` § Catalog-first rule) —
  AND remember `minted_at` also arms F-K4b grade-time congruence (§1's regression class);
  keep `expected.asserts` PASS-path-only on any such row.

---

## 4. Litmus / gate / REOPEN state

All verified directly against CI transcripts + a local run this rotation, not taken on
faith from prior summaries:

| Row | Last confirmed CI state | Where | Notes |
|---|---|---|---|
| `agent-ux/real-git-push-e2e` (P0) | **PASS, 4.28s** | run `29688725032` | F-K4b fix (`ba13553f`) confirmed working on CI. Do not re-open. |
| `structure/hermetic-test-network-isolation` (P2) | **UNKNOWN on `ba13553f`** (run died before reaching it); **FAIL, 0.02s** on the superseded prior run `29687639465` (dc60cc21); **local PASS, 0.07s** on this dev box | quality/gates/structure/hermetic-test-network-isolation.sh | 0.02s CI-FAIL smells like setup/import error, not the poisoned-proxy pytest itself running and failing (pytest startup alone takes longer). Needs a CLEAN run past this row to know its real CI status. |
| `docs-build/badges-resolve` (P2) | FAIL, ~30s, both recent runs | — | **Standing known-transient live-network P2. Do NOT code-fix.** Verify-by-reobservation only. |
| `docs-build/p94-badges-real-vs-transient` (P2) | FAIL, ~67s, both recent runs | — | Same as above — standing, transient. |
| `docs-build/animation-renders` (P2) | NOT-VERIFIED (verifier stub absent) | — | Owner-gated (E1 launch-animation publish, GTH-V15-37), deliberate. |
| `quality gates (pre-pr)` job (whole) | **SIGKILL / exit 137** on `ba13553f`; **clean exit 1 (5-FAIL)** on `dc60cc21` | CI job, not a catalog row | **The blocking item.** Root cause undiagnosed — see §1 + §6 step 1. |

**No formal REOPEN cycle yet** — nothing here has looped RED→fix→RED-again through a
verifier; the SIGKILL is a fresh finding from this rotation's own direct observation, not
a bounced verdict.

**Pre-push suite:** not re-run full this rotation (only the single hermetic row was
spot-checked locally). Run the full `pre-push` cadence before your next push regardless.

---

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

### Decisions made live

1. This coordinator chose to **directly observe CI ground truth via `gh run view` rather
   than trust the "in-flight" framing in its own dispatch brief** — the run concluded
   mid-rotation and the actual result materially changed the picture (a new SIGKILL
   blocker, not the expected F-K4b-vs-hermetic binary outcome). Successor: always
   re-verify CI state directly before acting on any inherited "in-flight" claim; it can go
   stale within minutes.
2. Did **not** attempt to fix or further diagnose the SIGKILL this rotation (out of scope
   for a relief-handover — no code-touching, and root-causing an OOM/kill needs a full
   pre-pr-cadence local run + possibly a resource-constrained reproduction, which is
   leaf-executor work, not handover-writer work).
3. Did **not** update `.planning/STATE.md`'s Current-Position prose (still reads the
   pre-`ba13553f` narrative, last_updated `13:00:00Z`, predating the P0 fix and this
   SIGKILL finding) — per the P123 precedent (STATE.md advances POST-VERDICT, not
   mid-close) and this task's explicit "ONE handover file" scope. **Successor: STATE.md is
   stale and needs a real update once the phase actually closes GREEN** (§6 step 5).

### Noticed, not yet filed (successor should file per OP-8 or eager-fix if <1h)

- **NEW (this rotation, HIGH priority to diagnose, not yet filed as SURPRISES-INTAKE):**
  the `quality gates (pre-pr)` job SIGKILL (exit 137) on `ba13553f`/run `29688725032`,
  ~6min in, right after the `docs-build` catalog and before `docs-reproducible.json` — see
  §1/§4. File a SURPRISES-INTAKE HIGH entry if not resolved same-session; this is now the
  single blocker on P126's close.
- A leaked `reposix sim --bind 127.0.0.1:7878 --ephemeral` survivor process (PID 11014 at
  forensic-dump time) — some gate backgrounds a sim and doesn't reap it before the parent
  dies. Minor on its own (small RSS) but worth folding into whatever fixes the SIGKILL
  investigation, per `quality/CLAUDE.md` § "Backgrounded-process fd convention."
- **Doc drift, LOW:** `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md`'s top-level
  index line for the W1 landmine (`surprises-intake/part-08.md`) still summarizes it as
  `severity: HIGH | OPEN` even though the underlying `part-08.md` entry itself correctly
  reads `**STATUS: CLOSED (P126 W1)**`. One-line index-summary staleness, not a real open
  item — fix in the same commit that next touches that file (fix-twice discipline).
- **Carried from P126 close (already filed, not new):** GTH-V15-94 (split
  `doc_alignment.rs`, 84,994 chars), GTH-V15-95 (RETIRE_CONFIRMED warning-emitter skip),
  GTH-V15-96 (DRY eligible/backfill file lists), GTH-V15-97 (split `run.py`/`test_run.py`
  over the `.py` ceiling), GTH-V15-98 (owner-action: canary-probe HARD-FAIL in the
  user-global `doc-clarity-review` SKILL.md — out-of-repo-boundary, cannot be fixed here).
- **Carried, architectural:** the `minted_at`/F-K4b coupling footgun (adding `minted_at`
  silently arms grade-time congruence checking) — decouple via an explicit opt-in flag
  rather than implicit activation. Filed conceptually in `quality/CLAUDE.md`'s Honesty
  rules section; a GOOD-TO-HAVES row for the actual code change is still owed.
- **Carried:** the 2 live-network `docs-build` P2 badge rows add ~127s serial cost to every
  `pre-pr` run; worth reconsidering their cadence membership (do they need to run on every
  PR, or would `weekly` + `on-demand` suffice?).

---

## 6. Precise next steps (successor runbook)

1. **Diagnose the SIGKILL first — this is the sole remaining blocker.** Run the full local
   `pre-pr` cadence end-to-end and watch memory/process behavior:
   `python3 quality/runners/run.py --cadence pre-pr` (validate-only, no `--persist`; needs
   git ≥ 2.34 + prebuilt bins — this dev box qualifies). If it completes locally, the
   SIGKILL is CI-runner-resource-specific (likely OOM on a smaller/shared runner) — look
   for a row spawning something memory-heavy right after `docs-build/animation-renders`
   (chronologically next: `docs-reproducible.json`'s `container-congruence-earned` /
   `snippet-coverage` / `container-rehearse-sigkill-safe` / `container-rehearse-exit-from-
   artifact`, then `freshness-invariants.json`). Cross-check whether a recent, unrelated
   PR changed anything about container/docker usage in those gates. If it reproduces
   locally too, you have a real repro — bisect which row triggers it.
2. **Once you have a theory, fix it, then re-run the FULL `pre-pr` cadence locally to
   confirm `structure/hermetic-test-network-isolation` (unreached, still genuinely unknown
   on CI) grades PASS** — do not assume the local dev-box PASS carries over; run it as
   part of the same full-cadence pass that also proves the SIGKILL is gone.
3. **Commit the fix**, targeted staging only, trailer format from §3.
4. **Fetch-rebase-push**: `git fetch origin && git rebase origin/main && git push origin
   main` (re-check ahead/behind immediately before — do not trust this handover's §1
   snapshot after any elapsed time).
5. **STOP at the liveness boundary.** Do not self-watch CI. Return to L0: pushed SHA, the
   new in-flight `ci.yml` run id, and "awaiting CI green to run post-push cadence + close."
6. **On L0 relay of CI green:** run `python3 quality/runners/run.py --cadence post-push
   --persist`; confirm `code/ci-green-on-main` PASS for both `ci.yml` and `release-plz.yml`.
   Then dispatch a **fresh** `gsd-verifier` leaf (zero session context) to grade P126's
   catalog rows from committed artifacts, writing `quality/reports/verdicts/p126/
   VERDICT.md`. RED loops back to fix; GREEN → proceed.
7. **On verifier GREEN, do the deferred STATE.md/roadmap close bookkeeping** (§5 item 3):
   update `.planning/STATE.md` Current-Position/Current-Focus to reflect the REAL P126
   close (cite `ba13553f` + whatever fixed the SIGKILL, not just the pre-`ba13553f`
   narrative it currently carries); flip `docs/roadmap.md`'s P126 entry from "In flight
   now" to "Landed recently" with today's date (binding-free — never let a
   `doc-alignment.json` row cite those moving lines); confirm `docs/roadmap.md`'s
   "Up next, in order" still correctly leads with P127.
8. **File the SIGKILL incident to `SURPRISES-INTAKE.md`** (resolved-same-session note if
   you fixed it in this rotation) and fold in the small noticings from §5 (leaked sim
   process, the index-summary doc drift) per OP-8 — eager-fix the doc drift now if handy
   (<1h, no dependency).
9. **Report done to L0** with the verdict path, the SIGKILL root-cause summary, and
   confirmation `structure/hermetic-test-network-isolation` graded for real on CI (not just
   locally).

**Escalate-only (report to L0, wait, do not act unilaterally):** any git tag `v*`/crates.io
publish; the E1 launch-animation publish (GTH-V15-37, owner-PENDING); any real-backend
MUTATION beyond the three sanctioned targets; any user-visible breaking change.
