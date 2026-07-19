# SESSION-HANDOVER.md — v0.15.0 Floor: #68→#69 relief — P126 one verifier away from
close, main GREEN, SIGKILL was a FLAKE (confirmed by clean rerun) — 2026-07-19

**VERIFY LIVE BEFORE ACTING — every number below was live-verified by workhorse seat #68
(this writer) immediately before this write (`git fetch`, `git rev-parse`, `git status`,
`gh run list`/`gh run view` incl. per-attempt job breakdowns). Concurrent pushes drift
state — re-run the §1 block yourself before doing anything else.**

Written by **workhorse seat #68** (L0 ROUTER), relieving at ~200k own context — past the
18%/180k soft gauge, at a NEAR-CLEAN boundary: **P126 is one verifier away from close,
main is GREEN.** This file **REPLACES** the prior `#67→#68` handover in place. Milestone
**v0.15.0 "Floor"**. Router ROUTES ONLY — delegate reads through a reader-digester, dispatch
a fresh closer coordinator, own the CI-watch loop yourself, cap subagent reports ≤400 words.

**Read order:** this file → §1 ground truth (verify live) → §2 milestone/phase state → §3
THE HEADLINE (P126 lane work + P0 fix DONE; SIGKILL flake exonerated) → §4 live agents
(the stopped, unreachable successor) → §5 Runbook (start at step 1) → §6 RAISE-LIST + HOLDS.

## 1. Ground truth (git/CI) — verified live, re-verify before acting

**Re-verify block (run this yourself first):**
```
git fetch origin main
git rev-parse HEAD origin/main && git status --porcelain
gh run list --branch main --workflow ci.yml --limit 3 --json databaseId,status,conclusion,headSha
gh run view <newest-id> --json conclusion   # TRUE conclusion — gh run watch exits 0 even on cancelled
```

**Live-verified by #68 immediately pre-write:**

- **Local HEAD = `9ad40505`** (`9ad405055b4b01815378b05e996b0f65d668a240`) — a **docs-only,
  UNPUSHED** commit that adds `.planning/phases/126-docs-alignment-tooling-polish/
  126-HANDOVER.md` (~20KB, a C2-tier relief handover, NOT this file). `origin/main` =
  **`ba13553f`**. Local is **1 commit ahead** of origin. Tree clean (`git status
  --porcelain` empty). THIS handover will land as a second commit on top of `9ad40505`,
  pushed together (see Runbook step 2).
- **Main is GREEN on `ba13553f`.** `gh run list --branch main --workflow ci.yml --limit 3`
  shows the newest run is **`29688725032`, headSha `ba13553f`, conclusion `success`**. This
  run has TWO attempts: **attempt 1 = `failure`** (SIGKILL flake, see §3), **attempt 2 =
  `success`** (all 15 jobs, confirmed via `gh run view 29688725032 --json jobs` — clippy,
  gitleaks, rustfmt, shell-coverage, test, runner-unit-tests-hermetic, 4× integration-
  contract, dark-factory regression, coverage, bench-latency-v09, and **`quality gates
  (pre-pr)` = success**). `code/ci-green-on-main` P0 would PASS.
- One older run for context: `29687639465` (headSha `dc60cc21`) = `failure` — this is the
  **P0 F-K4b regression run**, 14/15 jobs success, only `quality gates (pre-pr)` failed;
  superseded by the `ba13553f` fix (§3).

## 2. Milestone/phase state

- `STATE.md` frontmatter: `completed_phases: 13`, `total_phases: 15`, `percent: 87` —
  **optimistically advanced** by P126's close-bookkeeping commit (doctrine-sanctioned: the
  counter moves at close-bookkeeping, `docs/roadmap.md` keeps the phase "In flight now"
  until the fresh verifier grades GREEN). **True closed count is 12/15 (P114–P125); P126
  is IN-FLIGHT, not yet verifier-graded.** `docs/roadmap.md`'s three-block strip correctly
  still lists P126 under "In flight now" (verified live, line 38-42).
- Remaining arc: **P126 (finish close — verifier + roadmap-move) → P127 (Slot 1,
  SURPRISES-INTAKE drain) → P128 (Slot 2, GOOD-TO-HAVES drain + OP-9 retrospective + the
  non-skippable 9th `pre-release-real-backend` probe) → milestone-close.**

## 3. THE HEADLINE — P126 close: all lane work + P0 fix DONE, SIGKILL was a FLAKE, only the verifier + roadmap-move remain

Narrate so #69 does NOT re-investigate:

- **All P126 DRAIN-15..21 lane work is DONE.** Full trail (newest first, all committed):
  `588c1546` (W5 RAISE-3 — fixed the stale `docs/development/roadmap.md` active-milestone
  lie IN-PLACE, re-cited 5 doc-alignment rows in the SAME commit), `7f70b0de` (W6 close
  bookkeeping — SUMMARY/STATE/ROADMAP/REQUIREMENTS advance + roadmap-strip refresh),
  `dc60cc21` (GTH-V15-94..98 filed + `verdict.py` PROTOCOL fix-twice note), `ba13553f`
  (the P0 hotfix, below). The `minted_at` landmine (front-loaded EARLY-LANE W1, `44783ebe`
  → `65e8c497` → `d0753ef6` → DP-2 review PASS `5d097937`) is CLOSED.
- **P0 `agent-ux/real-git-push-e2e` F-K4b regression** (surfaced on CI run `29687639465`,
  FAIL 0.68s) was **root-caused + fixed at `ba13553f`, CONFIRMED PASSING on CI (4.28s, run
  `29688725032`).** Root cause (now documented in `quality/CLAUDE.md` § Honesty rules,
  fix-twice done): adding `minted_at` to a legacy mechanical row that emits
  `asserts_passed` ALSO arms the grade-time F-K4b `asserts_congruent` check, which demotes
  PASS→FAIL when an `expected.asserts` entry describes a mutually-exclusive branch (the
  git<2.34 skip path) that the PASS run never executes. Fix removed that mutually-exclusive
  entry from `expected.asserts` (F-K4b is PASS-path-only). Regression-locked at pre-push
  via `test_audit_field.py::TestFK4bMutuallyExclusiveBranch`. **DO NOT re-open.**
- **The pre-pr CI RED that followed (run `29688725032` attempt 1) was a SIGKILL/exit-137
  FLAKE, NOT the badges — CONFIRMED by direct log inspection this rotation.** The step log's
  last line before the process died: `-> START docs-repro/container-rehearse-sigkill-safe
  (P1)` at `13:27:34.69Z`, immediately followed by `Killed timeout -k 30 1200 python3
  quality/runners/run.py --cadence pre-pr` / `Process completed with exit code 137` at
  `13:27:34.80Z` — i.e. the kill landed within ~0.1s of that gate's START line. The 2 badge
  P2 FAILs (`docs-build/badges-resolve` 30.03s, `docs-build/p94-badges-real-vs-transient`
  67.58s) printed EARLIER, mid-run, and are unrelated to the kill — an earlier read (both
  the P126 C2's own dispatch brief AND #68's first pass) mis-attributed the RED to those
  badges; the actual killer is the `container-rehearse-sigkill-safe` gate's own kill logic
  (ironic given its name). **A clean rerun of the SAME run id (attempt 2) passed green in
  full**, confirming FLAKE — the gate ran clean on the 3 prior CI runs too, so this is not a
  reliable reproduction, just an intermittent kill.
- **NEW LATENT INFRA BUG to file (strong noticing — carry forward, do not lose):**
  `container-rehearse-sigkill-safe`'s flake SIGKILLed the ENTIRE `run.py` process (all ~83
  gates across every catalog), not just its own subprocess — its kill logic is evidently
  process-group/parent-scoped (**same class of bug as the fd-inheritance deadlock fixed at
  `cef3a2ea`** — a gate leaking control over process lifetime beyond its own child). It must
  target only its own child PID/subtree. This is a real eager-fix-or-GTH candidate for
  P126's close-fold (or P127/P128 if not eager-fixable in one lane) — file it, don't lose it.
  A leaked orphan `reposix sim --bind 127.0.0.1:7878 --ephemeral` (PID 11014, RSS ~18MB) was
  also observed in the forensic `ps auxf` dump at kill time — likely the same
  under-reaping family, fold together.
- **Badges (`docs-build/badges-resolve`, `p94-badges-real-vs-transient`,
  `structure/badges-resolve`):** a CONFIRMED external shields.io DYNAMIC-endpoint outage —
  #68 ran the gate locally this rotation: 6 PASS / 2 FAIL, only the 2 dynamic shields.io
  URLs timed out, all github/codecov/static-shields returned 200. It self-resolved on the
  green rerun (both badge rows are standing known-transient P2s per the P126 C2's own
  do-not-touch guardrail — verify-by-reobservation only, never code-fixed). **No waiver
  needed / moot.** If it recurs and actually blocks a JOB (confirm the P2 truly blocks
  pre-pr first, don't assume), a dated honest waiver on the pair per PROTOCOL is
  manager-authorized — name the outage + `until_date`, mint properly (never hand-edit
  `last_verified`/`minted_at`), never weaken the gate.
- `structure/hermetic-test-network-isolation` (P2) was **UNREACHED** on the SIGKILLed
  attempt 1 (the process died before `freshness-invariants.json` loaded) but ran and PASSED
  on the clean attempt 2 (part of the "all 15 jobs success" confirmation above) — no longer
  an open question.

## 4. Live agents — the stopped, unreachable successor

- **Successor P126 close-C2 `af191017691adc8bc`** (opus phase-coordinator, spawned in #68's
  session, picked up the wave-4 C2's `126-HANDOVER.md`): re-verified CI ground truth live,
  determined the SIGKILL was a flake (not a code bug), fired a rerun of the SAME run id
  (`29688725032`) to attempt 2, confirmed it green, and **STOPPED at the rerun→CI boundary**
  awaiting an L0 GREEN relay (no new commit/push — the C2 only reran the existing CI run,
  which is why `9ad40505` remains the only new local commit and is still unpushed).
  **It is cross-session — #69 almost certainly CANNOT SendMessage it** (SendMessage is not
  granted at the C2 tier or below; see the verbatim caveat below). Its committed work
  (`126-HANDOVER.md` at `9ad40505`) is durable; its close-plan is captured in §5.
  **#69: do NOT depend on reaching af19 — dispatch a FRESH closer C2 (opus
  phase-coordinator) to finish the close.**

## 5. #69 Runbook (numbered, start at step 1)

1. **Re-verify ground truth** (§1 block). Confirm main is GREEN on `ba13553f` (run
   `29688725032`, newest, conclusion `success` — check the TRUE per-run conclusion, not
   just that a rerun happened) and that `9ad40505` is still the local HEAD 1-ahead-of-origin
   (or has already been pushed by someone else — re-check before assuming).
2. **Dispatch a FRESH closer C2** (opus phase-coordinator). Charter:
   - Main is GREEN on `ba13553f`; P126 lane work + the P0 fix are all DONE and CI-confirmed
     (§3). The SIGKILL that looked like a new blocker was a confirmed FLAKE, already
     resolved by a clean rerun — do not re-diagnose it as a live blocker.
   - **First action:** push `9ad40505` (+ this handover, landed as a second commit on top,
     per the standard `git fetch origin && git rebase origin/main && git push origin main`
     cadence — re-check ahead/behind immediately before pushing, other sessions may have
     moved main).
   - **Then:** dispatch a FRESH `gsd-verifier` leaf (zero session context) to grade P126's
     SC/catalog rows from committed artifacts — `agent-ux/real-git-push-e2e` P0 (fixed
     `ba13553f`, CI-confirmed PASS), `structure/hermetic-test-network-isolation`,
     `docs-build/{badges-resolve,p94-badges-real-vs-transient,animation-renders}` P2
     (known-transient/owner-gated, do not code-fix) — plus the post-push cadence
     `code/ci-green-on-main` P0. Evidence commits: `ba13553f`/`7f70b0de`/`dc60cc21`.
   - **On verifier GREEN:** ONE close-bookkeeping commit — flip P126 in `docs/roadmap.md`
     from "In flight now" (~line 38) to "Landed recently" (~line 17, today's date,
     binding-free — confirm no `quality/catalogs/doc-alignment.json` row cites the moving
     lines), update `STATE.md` Current-Position/frontmatter to reflect the REAL close (not
     just the pre-`ba13553f` narrative it may still carry). **FOLD the intake in the SAME
     close pass:** the SIGKILL-flake resolution note, the `container-rehearse-sigkill-safe`
     leaked-process-group-kill INFRA BUG (eager-fix if <1h+no new dependency, else GTH per
     OP-8), the leaked sim PID 11014 note, GTH-V15-94..98 (already filed at `dc60cc21`,
     just confirm), the `minted_at`/F-K4b coupling footgun GTH (decouple via an explicit
     opt-in flag rather than implicit activation — conceptually noted in `quality/CLAUDE.md`,
     a GOOD-TO-HAVES row for the actual code change is still owed), and the 2 live-network
     badge P2s' ~127s serial pre-pr cost (worth reconsidering cadence membership — weekly +
     on-demand vs. every-PR).
   - **Push the close commit; L0 (#69) owns the close-push→CI watch** — the C2 stops at the
     push→CI-in-flight boundary and returns pushed SHA + run id, never self-watches.
   - Embed the SendMessage caveat + root ownership charter VERBATIM (both reproduced below).
3. **L0 owns every push→CI watch.** After the closer pushes, watch main's newest ci.yml run
   to TRUE-green (check `conclusion`, not just that `gh run watch` exited 0) before letting
   P126 grade closed. Never open P127 over a red main. NOTE: `container-rehearse-sigkill-
   safe` is a KNOWN FLAKE — if a future run SIGKILLs there again, a rerun is the documented
   first move, not a mystery to re-diagnose from scratch (though the underlying leaked-kill
   infra bug should still get fixed/filed per step 2's fold).
4. **Then drive P127 → P128 → milestone-close** two levels down (C1: opus complex / sonnet
   default / haiku mechanical). P128 needs BOTH the OP-9 RETROSPECTIVE distillation AND the
   non-skippable 9th `pre-release-real-backend` probe, and should cross-check DRAIN-13/14/
   22/23/24 REQUIREMENTS marks against `p124/VERDICT.md` before flipping.
5. **Surface owner-gated HOLDs; never self-authorize** (§6). **REPLACE this handover in
   place** (do not append) at your own relief; re-verify every claim live before writing it.

## 6. RAISE-LIST + HOLDS (carry forward, don't drop)

- P126 close-fold items (§3, §5 step 2) — SIGKILL-flake note, `container-rehearse-sigkill-
  safe` leaked-kill infra bug, leaked sim PID, `minted_at`/F-K4b footgun GTH, badge-cadence
  reconsideration.
- **P127 (Slot 1):** SURPRISES-INTAKE drain — incl. `code/shell-coverage` counter drift,
  file-size residuals (`STATE.md`, `good-to-haves/part-07.md`, `run.py` ~2× the soft limit;
  waiver umbrella expires **2026-08-08**), the container-rehearse infra bug if not
  eager-fixed in P126, dead `PROTECTED_IDS` var (`scripts/refresh-tokenworld-mirror.sh:66`).
- **P128 (Slot 2):** GOOD-TO-HAVES drain (GTH-V15-94..98 + the `minted_at`/F-K4b footgun +
  container-kill GTHs) + OP-9 retrospective + the 9th `pre-release-real-backend` probe +
  verify DRAIN-13/14/22/23/24 REQUIREMENTS marks vs. `p124/VERDICT.md`.
- **Owner escalations (filed, non-blocking):** DRAIN-15 canary-probe hard-fail belongs in
  user-global `~/.claude/skills/doc-clarity-review/SKILL.md` (OUT of repo → owner action, do
  not touch — this IS GTH-V15-98); RAISE-3 shipped fix-in-place, delete/consolidate-to-one-
  roadmap deferred (human-gated confirm-retire, do not unilaterally delete).
- **HOLDS (never self-authorize):** release/tag (`v*`, crates.io); real-backend mutation
  beyond the 3 sanctioned targets (`docs/reference/testing-targets.md`); file-size waiver
  umbrella expires **2026-08-08**; hero-number doc-alignment waivers expire **2026-08-15**;
  E1 launch-animation publish (GTH-V15-37, owner-PENDING); `.env` credential sign-off → P128.

**VERBATIM — SendMessage tier limitation (embed in every C2/C1 charter):**
*SendMessage is not granted at the phase-coordinator (C2) tier or below; L0→C2 and C2→main
work, C2→child and child→C2 fail; therefore C2-tier coordinators serialize strictly and
close phases via FRESH verifier→executor LEAVES, never fork-to-resume.*

**VERBATIM — root CLAUDE.md ownership charter (embed in every leaf charter):**
*Every subagent (executor, verifier, researcher, code-reviewer) that touches a real surface
**owns it**, not just its acceptance criteria (Owner mandate OD-3, 2026-07-03):*
1. *Acceptance criteria are the floor, not the ceiling — done means "I'd defend this in
   review as excellent," not "plan executed."*
2. *Noticing is a deliverable — every report names what it noticed near its work (lying doc
   claims, tests that don't assert what their names promise, error messages that don't teach
   recovery, dead code, stale comments, missing edge cases). An empty noticing section from
   code-touching work is itself a red flag.*
3. *Eager-fix or file, never silently skip — <1h + no new dependency → fix in place; else →
   the active milestone's `.planning/milestones/<active>-phases/{SURPRISES-INTAKE,
   GOOD-TO-HAVES}.md` with severity + sketch (OP-8).*
4. *Verify against reality — run the thing, render the page, hit the backend; a claim
   without an artifact is not done (OP-1).*
5. *North star — Rust-compiler-grade UX — end-user experience is the standing north star
   all tooling serves (docs, error messages, onboarding friction). Every user-facing error
   must (1) teach the fix, (2) suggest the alternative, (3) give a copy-paste recovery
   command — exemplar `crates/reposix-cli/src/init.rs::refuse_existing_repo_root`. UX
   polish is scheduled as first-class lanes, never leftovers. Would a skeptical dev hitting
   this surface for the first time come away impressed?*

*Meta-rule: when an owner catches a quality miss, fix it twice — fix the issue in code/docs,
AND update the relevant CLAUDE.md (root or scoped) / ORCHESTRATION.md so the next agent's
session reads the tightened rule — AND tag the dimension (routes to the right catalog +
`quality/gates/<dim>/`). Shipping the fix without updating the instructions guarantees
recurrence.*
