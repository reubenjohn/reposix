# 127-HANDOVER.md — P127 (Slot-1 SURPRISES-INTAKE drain, OP-8) coordinator relief handover, 2026-07-19

Predecessor: C2 phase-coordinator for P127, milestone v0.15.0 "Floor". Relieved
proactively at ~140k own-context tokens (§3 rule 5: absolute-token ceiling, hard stop
150k) at a clean Wave-2-done boundary, to give the successor fresh runway for the
remaining DP-2 (T1) lane + the sharpness-critical phase close.

**Successor: a FRESH C2 phase-coordinator, dispatched on `model: opus`.** Read this
handover in full before touching anything. Required reading order after this file:
`.planning/phases/127-surprises-intake-drain/127-01-PLAN.md` →
`.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md` index rows for #1 (T1) and
part-08.md → `.planning/CONSULT-DECISIONS.md` tail (the `[SELF gsd-executor P127
Slot-1 drain]` entry, DP-3 + T4 resolution) → `.planning/ORCHESTRATION.md` §3
(liveness doctrine, SendMessage tier limitation — both apply to you as a C2).

**Do not touch:** T2, T3, T4, T5, T6 dispositions (all recorded, committed, verified
below) — do not re-open or re-litigate them. **Do not** attempt a re-split of
`ORCHESTRATION.md` in this phase (see §5 below — file it, don't fix it). **Do not**
self-watch CI in the background — see §3 binding constraints.

## 1. Ground truth (git)

- `HEAD` = `ca597510ef8f79ec68f7a1d8f20f3a69328916fe`. `git status`: tree CLEAN
  (verified at handover time — re-verify yourself before your first mutating move).
- Local `main` is **5 commits AHEAD of `origin/main`, 0 behind**. `origin/main` =
  `a5511dd31ae59930aaaa128bc6bcf8475765fcc9` (pre-P127). **NOTHING has been pushed
  yet.** `origin/main`'s newest `ci.yml` run (`29691392637`) concluded `success` —
  main is GREEN as of the last push. **Re-run `git fetch origin main && git
  rev-list --left-right --count origin/main...HEAD` yourself immediately before you
  push** — concurrent sessions/agents can move `origin/main` between this handover
  being written and you acting on it.
- The 5 unpushed commits, oldest first:
  1. `393f9765` — `docs(127-01): plan Slot-1 SURPRISES-INTAKE drain triage` (the PLAN,
     no code change).
  2. `2bf6094a` — `fix(127-01): remove dead PROTECTED_IDS var in
     refresh-tokenworld-mirror.sh (T5)`.
  3. `974c41c0` — `fix(127-01): drop WARN-only assert #2 from shell-coverage row to
     defuse F-K4b demotion (T3)`.
  4. `f8b8d9ed` — `fix(127-01): re-mint hermetic-test-network-isolation WAIVED —
     CI-sandbox portability filed (T2)`.
  5. `ca597510` — `fix(127-01): reconcile traceability DRAIN rows — flip
     unentangled, hold 22/23/24 pending open intake (T4)`.
- No deviations from the plan's stated file list were found in the diffs — each
  commit touches exactly the files the PLAN's `files_modified` front-matter names for
  that item (T5 → `scripts/refresh-tokenworld-mirror.sh`; T3 →
  `quality/catalogs/code.json` + a new locking test in
  `quality/runners/test_audit_field.py`; T2 →
  `quality/catalogs/freshness-invariants.json` + `surprises-intake/part-08.md`; T4 →
  `.planning/REQUIREMENTS.md` + `SURPRISES-INTAKE.md` + `surprises-intake/part-08.md`
  + `CONSULT-DECISIONS.md`).

## 2. Wave/cycle state

| Wave | Item | State | Commit(s) |
|---|---|---|---|
| 1 | Plan authored | DONE | `393f9765` |
| 2 | T5 dead-var eager-fix | DONE | `2bf6094a` |
| 2 | T3 shell-coverage F-K4b defusal (DP-3) | DONE | `974c41c0` |
| 2 | T2 hermetic-test-isolation re-mint WAIVED | DONE | `f8b8d9ed` |
| 2 | T4 traceability reconcile (conservative) | DONE | `ca597510` |
| 2 | T6 file-size waiver confirmation | DONE (no-op, recorded) | — (confirmation only, no diff) |
| 3 | **T1 SIGKILL DP-2 prove-before-fix** | **NOT STARTED** | — |
| 4 | File NEW ORCHESTRATION.md size regression | **NOT STARTED** | — |
| 5 | Phase close (push → post-push cadence → verifier → close bookkeeping) | **NOT STARTED** | — |

No named incident occurred during Waves 1–2 (all 5 gated dispositions cleared their
discipline gates cleanly — see `CONSULT-DECISIONS.md` tail for the DP-3/T4 reasoning
trail). No post-mortem reading required before dispatching the T1 executor beyond the
PLAN's own T1 section and the part-08.md intake entry.

## 3. Binding constraints (unchanged)

- **One tree-writer at a time.** This is a shared repo; serialize all mutating
  dispatches.
- **ONE cargo invocation machine-wide, FOREGROUND-only, never `run_in_background`.**
  T1's repro/fix work is cargo/sim-dependent (P1 gate) — hold this hard.
- **Leaf isolation** — any `reposix init` / sim-seed / `git commit`/`config` for T1's
  repro work happens in a `/tmp` clone, `cd` in the SAME Bash invocation, never the
  shared repo. Mechanically enforced by `.claude/hooks/leaf-isolation-guard.sh`.
- **No `--no-verify`, ever.**
- **Push cadence:** `git push origin main` BEFORE dispatching the verifier. Then
  `python3 quality/runners/run.py --cadence post-push --persist` — the
  `code/ci-green-on-main` (P0) probe must confirm main's NEWEST `ci.yml` run
  succeeded, not merely that some past run was green. Never open/advance past this
  phase over a red main.
- **Commit-trailer format:** `Co-Authored-By: Claude Opus 4.8 (1M context)
  <noreply@anthropic.com>` (or your actual model identity) on relief/handover
  commits; `Claude-Session: <handle>` trailer also used in this repo's convention —
  include both on your own relief commit if/when you rotate.
- **Model tiering:** you (this C2) are opus per explicit dispatch instruction. Any T1
  executor/verifier/reviewer you dispatch should tier per complexity (T1's repro +
  fix touches a security/reliability-relevant process-group kill path — lean opus or
  at minimum sonnet, not haiku).
- **Liveness doctrine (load-bearing for a C2):** everything before the push (plan →
  execute → code-review) runs straight through. At the push→CI-in-flight boundary you
  **STOP and RETURN** to your dispatching parent (L0) with pushed SHA + in-flight
  `ci.yml` run id — you do NOT self-watch CI in the background. L0 owns the durable
  CI watch.
- **SendMessage tier limitation (STANDING):** SendMessage is NOT granted at the C2
  tier or below. You cannot background-and-resume your own children, and a child
  cannot resume-by-id back to you. Drive every phase-close step via **fresh
  verifier→executor LEAVES**, dispatched serially — never fork-to-resume a
  coordinator, never background-and-resume a child at your tier. Embed this caveat
  verbatim in every leaf charter you write.

## 4. Litmus / gate / REOPEN state

- **Main CI:** newest run on `origin/main` (`a5511dd3`) = `success` (run id
  `29691392637`, also a passing `Docs` and `CodeQL` run at that SHA). No REOPEN in
  flight. Your 5 unpushed commits have NOT yet been through CI — that's the whole
  remaining-work item #3 below.
- **`code/shell-coverage` (T3):** honesty gate 96/96 PASS locally at commit time (per
  the executor's report folded into `CONSULT-DECISIONS.md`); new regression test
  `TestShellCoverageWarnOnlyAssertDefused` added in `974c41c0` locks the F-K4b
  defusal. `apply_pass_gates` proved FAIL-before/PASS-after against the real row +
  artifact.
- **`structure/hermetic-test-network-isolation` (T2):** re-minted `WAIVED`, waiver
  `until: 2026-09-15T00:00:00Z`, `blast_radius: P2` (never gates a phase exit),
  `dimension_owner: "P127 / intake part-08"`, tracked back to
  `surprises-intake/part-08.md`. Grades WAIVED via `run_row`'s `waiver_active`
  short-circuit — verifier is NOT run while waived. Deeper CI-portability CODE fix
  stays filed, NOT this phase's job.
- **`structure/file-size-limits` (T6):** waiver `until: 2026-08-08T00:00:00Z`
  unexpired, confirmed still covers `STATE.md` (35617B), `good-to-haves/part-07.md`
  (13417B), `run.py` (32059B) — plus two NEW residuals that grew warn-only under the
  same umbrella during this phase's own work: `test_audit_field.py` now **36866B**
  (grew from T3's locking test) and `REQUIREMENTS.md` now **26964B** (grew from T4's
  reconcile annotations). Both stay inside the documented waiver — do not churn to
  shrink them in P127.
- **Open waiver expiry clocks to carry forward:** hermetic-test-network-isolation
  2026-09-15; file-size-limits 2026-08-08. Neither expires before this phase should
  reasonably close; if your close slips past 2026-08-08 re-check the file-size
  waiver's status before declaring T6 still a non-issue.
- **No REOPEN currently active on any P127-touched row.**

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

- **DP-3 (intake-design inversion) cleared T3 and T2** — documented in
  `CONSULT-DECISIONS.md` (`2026-07-19 [SELF gsd-executor P127 Slot-1 drain]`). Treat
  as settled; do not re-run DP-3 on these two.
- **T4's RD-2 (naive "phase closed GREEN → mark all its DRAIN rows Complete") vs the
  PLAN's anti-tautology guard was a genuine live conflict**, resolved conservatively
  in favor of the guard. This is the single most important judgment call in the phase
  so far — do not let a later lane bulk-flip DRAIN-22/23/24 without a fresh verdict
  that specifically closes T1 (for 23) and T3's class (for 22) and provenance (for
  24). DRAIN-11 and DRAIN-13 are ALSO held (11 because of the ORCHESTRATION.md
  re-growth caught below; 13 because its readiness-gate half shares the port-7878
  orphan surface with the still-open T1 SIGKILL item).
- **NOTICED, NOT YET FILED — ORCHESTRATION.md size regression.** Phase 119 split
  `.planning/ORCHESTRATION.md` under the 20000B ceiling at close (20480B was already
  over per the T4 commit message, worth double-checking the exact historical number
  if it matters, but the CURRENT size is unambiguous and freshly measured this
  session: **24119 bytes, verified by `wc -c` at handover time — 4119B over the
  20000B ceiling**). This is a live regression under the file-size waiver umbrella
  (warn-only, main stays green) but the split from Phase 119 did NOT stick — the file
  has re-grown via P123–126 doctrine additions. **This has NOT been filed as its own
  SURPRISES-INTAKE entry yet** — T4's commit message references it as the reason
  DRAIN-11 stays held, but no standalone intake row exists. This is remaining-work
  item #2 below; do not skip it.
- **Noticing from T3's commit (OD-3):** the `code/shell-coverage` row's `owner_hint`
  and `claim_vs_assertion_audit` had been describing the WARN-only counter assert as
  a graded pass/fail claim when it never was — a lying-audit smell, corrected in
  `974c41c0` as part of the same commit (not filed separately; already fixed).
- **Nothing else noticed-not-filed from Waves 1–2** beyond what's captured above and
  in the PLAN's own "Noticing (OD-3, at plan-authoring time)" section (dir-naming
  convention mismatch caught and corrected at plan time; already resolved, no action
  needed).

## 6. Precise next steps (successor runbook)

Execute in this order. Do not skip or reorder.

1. **T1 — SIGKILL process-group-kill (HIGH, DP-2 prove-before-fix), still OPEN.**
   Dispatch a fresh `gsd-executor` leaf (tier: opus or sonnet, not haiku — this
   touches a process-management security-relevant path) with a charter to:
   a. Build a COMMITTED minimal repro FIRST, proving whether
      `container-rehearse-sigkill-safe.sh` (or a sibling that `setsid`/group-kills)
      propagates its kill signal beyond its own child subtree — i.e. does it kill the
      parent `run.py` process group / sibling gates, and does it leak an orphan
      `reposix sim` PID / port-7878 binding. Cross-reference the fd-inheritance
      sibling fix `cef3a2ea` for the reaping+isolation pattern already used elsewhere
      in this repo.
   b. **If the repro FAILS (demonstrates the escape):** CONFIRMED. The repro becomes
      the locking regression test. THEN dispatch a fix (target ONLY the script's own
      child PID/subtree, not the process group) with the repro's path named in its
      charter. The fix merges ONLY when (i) the repro flips GREEN under the fix AND
      (ii) a FRESH `gsd-code-reviewer` leaf (not the same agent that wrote the fix)
      confirms it addresses the mechanism, not just the symptom.
   c. **If the repro is un-buildable inside ONE lane budget:** DOWNGRADE the intake
      entry to *suspicion*, keep it filed with the repro attempt recorded, do NOT
      ship a speculative kill-scoping change. Update the `SURPRISES-INTAKE.md` index
      row (line 107, `#1`) and `part-08.md` entry accordingly either way — CONFIRMED
      w/ fix SHA, or DOWNGRADED w/ repro-attempt trace. DRAIN-23 stays Pending
      regardless of this outcome (already held per §5) unless the fix lands AND is
      code-reviewer-confirmed, in which case flip it in the SAME close-bookkeeping
      pass as step 3.
   d. Binding constraints from §3 apply in full — one cargo invocation
      machine-wide, FOREGROUND only; leaf isolation in `/tmp` for any sim/reposix
      setup.

2. **FILE a NEW surprise: ORCHESTRATION.md file-size regression.** Add a
   `SURPRISES-INTAKE.md` index row + a `surprises-intake/part-08.md` (or next
   available part) entry: severity MEDIUM, describing that Phase 119 split
   `.planning/ORCHESTRATION.md` under the 20000B ceiling at close but it has
   RE-GROWN to 24119B (verified 2026-07-19) — a live regression, currently warn-only
   under the file-size waiver residual (main stays green). Candidate disposition:
   folds into the GTH-V15-89 family (a machine gate that cross-checks
   ORCHESTRATION.md size at each phase close) → route to P128 Slot 2
   GOOD-TO-HAVES.md, NOT fixed here. **Do NOT attempt a re-split in P127** — it's
   >1h of work with real risk of breaking cross-references throughout the doc (fails
   the OP-8 eager-fix bar; DRAIN-11 stays correctly Pending because of this exact
   fact).

3. **CLOSE the phase:**
   a. Re-check ahead/behind (`git fetch origin main && git rev-list --left-right
      --count origin/main...HEAD`) — do not assume the state from §1 is still
      current.
   b. `git push origin main` — BEFORE dispatching the verifier.
   c. `python3 quality/runners/run.py --cadence post-push --persist` — confirm
      `code/ci-green-on-main` (P0) is GREEN against main's NEWEST `ci.yml` run (not
      an older green run).
   d. Dispatch a FRESH `gsd-verifier` leaf (unbiased, did not write the code) to
      grade P127 from committed artifacts: the P0 post-push probe result, plus each
      of the 6 items' recorded disposition (T1's outcome from step 1, T2–T6 already
      committed per §1/§2).
   e. On GREEN verdict, do close bookkeeping in a single commit: SUMMARY.md in
      `.planning/phases/127-surprises-intake-drain/`, `STATE.md` cursor advance,
      `docs/roadmap.md` three-block strip refresh (move P127 In-flight→Landed;
      confirm first that no `quality/catalogs/doc-alignment.json` row cites the
      specific lines you're editing in the three-block section — binding-free
      constraint is HARD).
   f. **STOP at the push→CI-in-flight boundary and return to L0 (router seat #69 or
      whatever the current seat is)** with: the verifier's verdict (GREEN/RED), each
      of the 6 items' final disposition (SHA / kept-filed / downgraded), the
      close-bookkeeping commit SHA, and the in-flight `ci.yml` run id from your push
      in step 3b. **Do NOT self-watch that CI run** — L0 owns the durable watch and
      will SendMessage you (or dispatch your successor) to resume on green.

If at any point you approach your own ~100k-token line, write and commit your OWN
relief handover (same template, `.planning/phases/127-surprises-intake-drain/
127-HANDOVER-2.md` or similar numbered successor file) before ending your turn —
uncommitted state didn't happen.

---

Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>
Claude-Session: relief-handover-writer, v0.15.0 P127 C2 relief
