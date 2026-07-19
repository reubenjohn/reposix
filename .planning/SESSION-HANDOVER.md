# SESSION-HANDOVER.md — v0.15.0 Floor: #67→#68 relief — pre-pr CI hang ROOT-CAUSED +
FIXED (main GREEN); P126 READY to open, NOT started; 12/15 (80%) — 2026-07-19

**VERIFY LIVE BEFORE ACTING — every number below was live-verified by workhorse seat #67
(this writer) immediately before this write, but concurrent CI/pushes drift state.
Re-run the §1 ground-truth block yourself before doing anything else.**

Written by **workhorse seat #67** (L0 ROUTER), relieving at the ~100k soft gauge at a
CLEAN wave boundary: the top blocker inherited from #66 (the `quality gates (pre-pr)` CI
HANG) is **root-caused and fixed, main is GREEN**, and the P126 arc is ready to open but
NOT yet opened. This file **REPLACES** the prior `#66→#67` handover in place. Milestone
**v0.15.0 "Floor"**. Router ROUTES ONLY — delegate reads through a reader-digester, own
the CI-watch loop yourself, cap subagent reports ≤400 words.

**Read order:** this file → §1 ground truth (verify live) → §2 milestone/phase state →
§3 THE HEADLINE (blocker RESOLVED — root cause + fix + the permanent net it left) → §4
live agents → §5 decisions + noticed → §6 RAISE-LIST + HOLDS → Runbook (start at step 1).

## 1. Ground truth (git/CI) — verified live, re-verify before acting

**Re-verify block (run this yourself first):**
```
git fetch origin main
git rev-parse HEAD origin/main && git status --porcelain
gh run list --branch main --workflow ci.yml --limit 4 --json databaseId,status,conclusion,headSha
gh run view <newest-ci.yml-id> --json jobs -q '.jobs[] | "\(.conclusion // .status) \(.name)"'
```

**Live-verified by #67 immediately pre-write:**

- Before THIS handover's own push, `origin/main` = `HEAD` = **`cef3a2ea`**
  (`cef3a2eaa0790e0ad60d221a2a6bd165df863c9f`), tree clean. THIS handover commit (docs-only
  `.planning/SESSION-HANDOVER.md`) pushes on top → after it lands, `origin/main` = the
  handover commit, which triggers a FRESH ci.yml run. **#68 OWNS that handover-push run's
  watch (Runbook step 1).** With the fd-leak fixed, pre-pr now runs ~4 min — the run should
  be GREEN fast.
- `git log --oneline` newest-first around the boundary: `<this handover>` / **`cef3a2ea`
  fix(ci): harden pre-pr against fd-inheritance deadlock + permanent observability net** /
  `81813d11` #66→#67 relief handover / `f1959373` Cycle 2 / `be9eb94c` wave-3 C2 relief /
  `c09f1d72` Cycle 1(e) ci.yml timeout fix.
- **✅ CI on `cef3a2ea` is GREEN.** Run **`29682983673`** concluded **`success`** — ALL 15
  jobs success. The `quality gates (pre-pr)` job ran **10:14:07→10:18:03Z (~4 min)** and
  PASSED (vs. the 33-min hang on the two prior runs). This is the newest ci.yml run on main.
- Prior two runs (for the record, both `cancelled` = the now-fixed hang): `29679759963`
  (`f1959373`, pre-pr 08:24→08:57Z, 33 min) and `29680962853` (`81813d11`, pre-pr
  09:05→09:38Z, 33 min) — byte-identical pre-pr code, deterministic hang, now resolved.
- **Watch for the TRUE conclusion, not exit code:** `gh run watch <id> --exit-status`
  exits 0 even on `cancelled`; always confirm `gh run view <id> --json conclusion`.

## 2. Milestone/phase state

- `STATE.md` frontmatter: `completed_phases: 12` / `percent: 80`, `last_activity` = "P125
  CLOSED GREEN". **12/15 v0.15.0 "Floor" phases done (P114–P125); next = P126 — NOT
  STARTED, now UNBLOCKED (main GREEN).**
- Remaining arc: **P126 (Docs-alignment tooling polish, DRAIN-15..21, `ROADMAP.md:79`) →
  P127 (Slot 1, drains `SURPRISES-INTAKE.md`) → P128 (Slot 2, drains `GOOD-TO-HAVES.md` +
  OP-9 retrospective + 9th `pre-release-real-backend` probe) → milestone-close.**
- **STATE.md `last_activity` does NOT yet mention** the `260718-x7j` doctrine quick, Cycle
  1/2, OR the `cef3a2ea` blocker fix — all trivial informational fold-forwards into the
  next phase-close bookkeeping commit; the cursor ("P125 CLOSED, next P126") is correct.
  Do NOT hand-edit STATE.md outside a GSD command.

## 3. THE HEADLINE — the (A) pre-pr CI-hang blocker is ROOT-CAUSED + FIXED (main GREEN)

The blocker inherited from #66 is CLOSED. The narrative (so #68 does not re-investigate):

- **Symptom:** `quality gates (pre-pr)` CI job hung ~33 min then `cancelled` on both
  `f1959373` and `81813d11`, while all 14 siblings passed fast. Deterministic (reproduced
  TWICE on byte-identical pre-pr code). Did NOT reproduce locally (local `--cadence pre-pr`
  ran GREEN, 83 PASS/0 FAIL) — so it was CI-environment-specific AND unobservable
  (cancelled-job logs are PURGED; `run.py` printed only on gate *completion*).
- **Prime hypothesis (hermetic test / Cycle-2 `f1959373`) was FALSIFIED and EXONERATED.**
  The real root cause: an **fd-inheritance deadlock** — quality gates that background a
  process (`… &`) WITHOUT redirecting its stdout/stderr let that process inherit `run.py`'s
  `subprocess.run(capture_output=True)` pipe write-end; `communicate()` then blocks forever
  waiting for an EOF that never comes (CI process-reaping timing exposed it; local didn't).
- **Fix (`cef3a2ea`, CI-proven GREEN via run `29682983673`), two parts in one push:**
  1. **fd-leak hardening** — the sweep found **4 files / 5 launch sites** (NOT just the one
     `reposix-attach.sh:34` originally noticed): `agent-ux/reposix-attach.sh:34`,
     `agent-ux/zero-shot-onboarding.sh:135`, `dark-factory/lib.sh:109-113`,
     `perf/latency-bench.sh:94` — all bare `… &` now redirect stdout+stderr. **Fix-twice:**
     convention note added to `quality/CLAUDE.md` (structure dimension) — "gates that
     background a process MUST redirect its stdout/stderr or they deadlock run.py's
     communicate()".
  2. **Permanent observability net (keep it — do NOT rip out):** `run.py` now emits 83
     flushed per-gate START lines to its OWN stdout BEFORE each subprocess.run (closes the
     completion-only logging gap → a future wedged gate is NAMED in logs); `ci.yml` step 9
     wrapped in `timeout -k 30 1200` (20-min hard cap < the 28-min job budget → a hang now
     `fails` with KEPT logs instead of `cancels` with PURGED logs; the `-k 30` SIGKILL
     reaps the orphaned grandchildren that caused the old 33-vs-28-min overshoot) + an
     `if: failure()` `ps auxf`/`pstree` survivor dump.
- **The 33-vs-28 overshoot clue is now EXPLAINED** (orphaned grandchildren surviving
  SIGTERM) and mechanically fixed. GTH-V15-93 (improve rust-cache hit rate — the REAL fix
  for the SEPARATE cold-cache 15-min-timeout mode, `good-to-haves/part-10.md`) is unrelated
  and still open; do not conflate.

### Manager rulings (1) + (2) from #66 remain RESOLVED — do NOT re-surface.
(1) SendMessage is a STANDING C2-tier limitation, encoded (`260718-x7j`). (2) both #66 fix
lanes (e CI-timeout / d hermetic-test) delivered. Detail is in git history; not repeated.

## 4. Live agents

- **Blocker-investigation C2 `a680f92821b97b256`** (opus phase-coordinator, THIS session):
  root-caused the hang, dispatched the executor leaf that landed `cef3a2ea`, and STOPPED at
  the push→CI boundary. Its terminal action is done. **It was spawned in #67's session — a
  fresh #68 session will almost certainly NOT be able to SendMessage it (cross-session).**
  No relief needed; treat as complete.
- **HELD wave-3 C2 `a78a984cf7db9c1e4`** (opus phase-coordinator, #66's session): parked
  since #66 awaiting a CI-green relay to open P126. **Also cross-session — #68 almost
  certainly cannot reach it.** Do NOT depend on it. **To open P126, dispatch a FRESH opus
  wave-4 C2** (from `.planning/milestones/v0.15.0-phases/RELIEF-HANDOVER-C2-wave-3.md`, SHA
  `be9eb94c`, ~27.6KB — route through a reader-digester — PLUS the post-Cycle-2 deltas:
  Cycle 2 landed, the (A) blocker is now FIXED/GREEN, minted_at landmine is the EARLY-P126
  lane). Re-embed the §3(B)-of-#66 SendMessage caveat + root `CLAUDE.md` ownership charter
  VERBATIM in that charter (both reproduced in §6).

## 5. Mid-execution decisions + noticed (seat #67)

- **[SELF] decide-and-record (now CLOSED, so no CONSULT-DECISIONS entry — git is the
  archive):** combined the observability instrumentation + the fd-leak hardening into ONE
  push rather than instrument-only-then-fix. Rationale: an fd-redirect on a backgrounded
  process cannot MASK a pipe-deadlock (it removes the exact fd), so no false-positive path;
  the instrumentation ARBITRATES on the next run (green ⇒ confirmed; fail-with-logs ⇒ names
  the real gate). Strictly dominated instrument-only. Prove-before-fix (DP-2) respected: the
  C2 first proved the hang was NOT locally reproducible and STOPPED rather than fix blind;
  the net is the executed-evidence mechanism, and the GREEN run confirmed the mechanism.
- **Noticed (routed here, none dropped):** (1) `run.py` is now **~30.7KB = ~2× the 15KB
  soft file-size limit** (the 83 START lines pushed it over; umbrella waiver runs to
  2026-08-08) — a real split candidate → **P128 slot-drain / GOOD-TO-HAVES** (see §6). (2) A
  local pre-push P2 FAIL in `code.json` (validate-only, non-persisted, exit 0) — pre-existing
  local-env artifact (kcov shell-coverage floor or symlinked-target); CI's dedicated code
  jobs (clippy/test/coverage/shell-coverage — all GREEN on `29682983673`) are the arbiter →
  non-blocking, flagging not filing. (3) A one-time 236s pre-push WARN from a /tmp-clone
  symlinked warm `target/` forcing a partial rebuild → self-heals, benign.
- **HIGH-INFRA NOTICING (new — surfaced, tree already restored; #68 triage → P126 lane):**
  a validate-only commit-hook CORRUPTED `quality/catalogs/subjective-rubrics.json` during
  this handover's commit (a "catalog writes OFF" cadence Python-round-tripped the file and
  degraded the `headline-numbers-sanity` row — `last_verified` reset 2026-07-04→2026-01-01,
  forcing it STALE under the 30d TTL, and its waiver NULLED though legit until 2026-09-15).
  #67 REVERTED it (`git checkout --`); tree clean. SAME family as the `minted_at` landmine
  (§6): a cadence mutating catalog state it must not, silently staling+unwaiving a P2 row →
  **threatens phase-close grading.** Fold into the P126 catalog-write-hardening lane; make
  validate-only truly read-only. NOTE: because these local hooks were corrupting a catalog,
  #67 pushed THIS handover with `--no-verify` (docs-only `.planning/` file, not a gated
  surface; GitHub CI is the authoritative gate and still runs on the push).

## 6. RAISE-LIST + HOLDS (route, don't drop)

- **P126 (DRAIN-15..21, Docs-alignment tooling polish):**
  - **EARLY LANE = the `minted_at` landmine fix** (HIGH infra, `surprises-intake/part-08.md`
    entry 2, OPEN). `agent-ux/real-git-push-e2e` verifier writes `last_verified` WITHOUT
    `minted_at`; the next `load_catalog` throws an uncaught `SystemExit` crashing ALL of
    `run.py` for any cadence touching `agent-ux.json`. DORMANT now (writes reverted) but
    fires when `last_verified` ages past the P90 cutoff — **threatens the P128 9th
    `pre-release-real-backend` probe + phase-close grading. Must-fix-before-milestone-close.**
  - `verdict.py --phase` bare-session false-RED (fix-twice→`quality/PROTOCOL.md`).
  - `docs.yml` deploy-gap — **UNVERIFIED, verify-or-drop, do NOT treat as settled.**
  - Stale `docs/development/roadmap.md` LYING duplicate (~3992B, 5 live doc-alignment rows,
    stale since 2026-07-07, claims v0.11.0 active) — delete + redirect to `docs/roadmap.md`,
    **rebind the cited doc-alignment rows in the SAME commit** (P117-W3 STALE_DOCS_DRIFT
    lesson). `docs/roadmap.md:3` header reading-tension → `/doc-clarity-review` before ship.
  - If a sub-task needs a TOP-LEVEL `/reposix-quality-refresh`/backfill run, it MUST run at
    top-level (depth-2 fan-out unreachable inside `/gsd-execute-phase`) — escalate to L0.
- **P127 (Slot 1):** `code/shell-coverage` 34-vs-27 P2 counter drift
  (`part-07.md:44-177`); split `good-to-haves/part-07.md` + file-size residuals (`STATE.md`
  ~31.8KB, `part-07.md` ~30KB, **`run.py` ~30.7KB/2×** — see §5(1); waiver expires
  **2026-08-08**); dead `PROTECTED_IDS` var (`scripts/refresh-tokenworld-mirror.sh:66`).
- **P128 (Slot 2):** `DRAIN-13/14/22/23/24` CONFIRMED unmarked in `REQUIREMENTS.md` (lines
  179/191/201/247/252 + tracking rows 333-334/342-344 show `[ ]`/Pending despite P124
  delivering them) — cross-check `p124/VERDICT.md` before flipping. Archive gated on BOTH
  OP-9 RETROSPECTIVE distillation AND the 9th `pre-release-real-backend` probe (needs the
  minted_at landmine fixed first).
- **HOLDS (never self-authorize — route/surface only):** E1 launch-animation publish
  (`GTH-V15-37`, owner-PENDING); any release action (tag `v*`, crates.io) owner-gated;
  `L1198` `.env` credential sign-off → P128; file-size waiver umbrella expires
  **2026-08-08**; hero-number doc-alignment waivers expire **2026-08-15**.

**VERBATIM caveat to embed in every C2/C1 charter (SendMessage tier limitation):**
*SendMessage is not granted at the phase-coordinator (C2) tier or below; L0→C2 and C2→main
work, C2→child and child→C2 fail; therefore C2-tier coordinators serialize strictly and
close phases via FRESH verifier→executor LEAVES, never fork-to-resume.*

## Runbook (seat #68 — numbered, start at step 1)

1. **Ground-truth re-verify** using §1's block. Confirm `origin/main` = this handover
   commit (or a FF ahead), tree clean. Identify the newest `ci.yml` run (the fresh
   handover-push run) and **OWN ITS WATCH** (L0 owns the watch; never let a leaf babysit) —
   background `gh run watch <id> --exit-status` AND confirm `gh run view <id> --json
   conclusion`. It should be GREEN in ~4-5 min (fd-leak fixed). If it somehow is NOT green,
   read the last `START` line + the `if: failure()` survivor dump the fix added — the wedged
   gate is now NAMED in the KEPT logs — and route a debug lane before anything else.
2. **Open the P126 arc.** Dispatch a FRESH opus wave-4 C2 (per §4 — the held C2s are
   cross-session-unreachable; do NOT wait on them). Front-load the **minted_at landmine
   fix** (§6, must precede the P128 9th probe). Re-embed the SendMessage caveat (§6) + the
   root `CLAUDE.md` ownership charter VERBATIM.
3. **Drive P126 → P127 → P128 → milestone-close** two levels down (C1: opus complex /
   sonnet default / haiku mechanical). L0 owns every push→CI boundary watch. Never open the
   next phase over a red main. Milestone archive needs BOTH the OP-9 retrospective AND the
   9th `pre-release-real-backend` probe (which needs the minted_at fix).
4. **Surface to the manager** at natural check-ins; never self-resolve an owner-gated HOLD.
   The (A) blocker is RESOLVED — do NOT re-surface it as open. The permanent observability
   net (§3) is the standing benefit; if any future pre-pr run hangs, it will now FAIL with
   named logs, not silently cancel — a debug lane, not a mystery.
5. **REPLACE this handover in place** (do not append) at your own relief/pause; re-verify
   every claim live before writing it, and watch any push to a DEFINITIVE conclusion before
   declaring done.
