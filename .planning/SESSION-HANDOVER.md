# SESSION-HANDOVER.md — v0.14.0 READY-TO-TAG under Manager Ruling #4 / Option B; successor #13 charter COMPLETE; the tag is the MANAGER's to push — 2026-07-13

Map, not territory — detail lives in git + the linked committed artifacts, not restated
here. **HEAD = live state; verify live before trusting anything in this file** — it is a
snapshot, not a subscription. This REPLACES (does not append to) the prior
`SESSION-HANDOVER.md` (which was written pre-Ruling-#4 and is now stale in full). Resume
an agent via SendMessage, never fork (ORCHESTRATION §11).

**STATUS: v0.14.0 is READY-TO-TAG under Manager Ruling #4 / Option B (recorded caveat).
The tag is NOT pushed — pushing it is the MANAGER's action, not any coordinator's.
Successor #13's charter is COMPLETE.**

## 1. Ground truth (git) — VERIFY LIVE, do not trust this file's staleness

Re-run before doing anything else:
```
git rev-parse --short HEAD && git status --porcelain && \
  git tag -l v0.14.0 && git ls-remote --tags origin v0.14.0
```
Verified this session: HEAD = `708b3e9`, tree **clean**, `0 0` vs `origin/main` (pushed).
**No `v0.14.0` tag exists**, local or remote.

Completed SHA chain this arc, oldest → newest:
- `05788d7` — Ruling #4 recorded in `.planning/CONSULT-DECISIONS.md`; the OPEN git-floor
  escalation entry flipped → RULED-OPTION-B.
- `22e8356` — OP-9 retro distillation: `.planning/RETROSPECTIVE.md` gains a v0.14.0
  "Tag-prep arc" subsection carrying three VERBATIM caveats (Ruling #2 litmus
  non-idempotency, item-7 CREATE-recovery WAIVED, Ruling #4 t4 git-floor).
- `4645060` — runner-minted honest 5/0/1 real-backend grades persisted to
  `quality/catalogs/agent-ux.json`, from a fresh CREDS-LOADED 9th-probe re-run (t4
  FAIL→NOT-VERIFIED git-floor; github-front-door FAIL→PASS). This RE-ESTABLISHED
  evidence that a creds-absent background run had clobbered.
- `b8e309f` — `quality/reports/verdicts/milestone-v0.14.0/VERDICT.md` re-minted
  GREEN-WITH-RECORDED-CAVEATS (never claims cadence exit-0; states Ruling #4's three
  conditions verbatim).
- `5624943` — `quality/reports/verdicts/milestone-v0.14.0/RATIFICATION.md`: a FRESH
  unbiased ratifier (author ≠ minter, zero prior context) graded
  GREEN-WITH-RECORDED-CAVEATS, all 7 criteria PASS, no defect found.
- `708b3e9` (HEAD) — `.planning/milestones/v0.14.0-phases/tag-v0.14.0.sh` authored
  (the manager's to run; clean-tree/branch/version/CHANGELOG/CI/verdict/ratification
  guards + signed tag + interactive confirm; **NOT executed**) + session noticing filed
  (`surprises-intake/part-04.md`, SURPRISES-INTAKE terminal entries, GOOD-TO-HAVES-16).

## 2. Wave/cycle state

| Step | Artifact | State | Commit |
|---|---|---|---|
| Ruling #4 recorded | `CONSULT-DECISIONS.md` | DONE | `05788d7` |
| OP-9 retro distillation | `RETROSPECTIVE.md` § Tag-prep arc | DONE | `22e8356` |
| Fresh creds-loaded 9th-probe re-run, persisted | `quality/catalogs/agent-ux.json` | DONE (5/0/0/0/1, exit 1) | `4645060` |
| VERDICT re-mint | `verdicts/milestone-v0.14.0/VERDICT.md` | DONE, GREEN-WITH-RECORDED-CAVEATS | `b8e309f` |
| Independent ratification | `verdicts/milestone-v0.14.0/RATIFICATION.md` | DONE, all 7 criteria PASS | `5624943` |
| Tag script authored | `tag-v0.14.0.sh` | AUTHORED, **NOT run** | `708b3e9` |
| **Tag push** | `v0.14.0` git tag | **NOT STARTED — manager's action** | — |
| Milestone archive / STATE.md cursor | `.planning/STATE.md` | **NOT STARTED** (still reads stale "items 4-8 pending") | — |

No named incident this arc requiring a post-mortem read before dispatching further work.

## 3. Binding constraints (carry, unchanged)

NO tag push by any coordinator — ever (manager's alone); ONE cargo invocation
machine-wide (prefer `-p <crate>`); `/tmp` leaf isolation, `cd` in the SAME Bash
invocation; single tree-writer discipline (one tree-mutating agent at a time,
read-only may parallelize); sim-first for code, real backends only through
`REPOSIX_ALLOWED_ORIGINS`; relief at ~100k soft / ~150k hard context (absolute
tokens, never %) → write+commit+push a fresh handover, end turn, let the parent
rotate. A `fork` is never a safe discard — resume a child via SendMessage only.

## 4. Litmus / gate / REOPEN state

**Honest cadence result (record verbatim, NEVER claim exit-0):**
`5 PASS / 0 FAIL / 0 PARTIAL / 0 WAIVED / 1 NOT-VERIFIED -> exit=1`.

- PASS: `milestone-close-vision-litmus-real-backend` (P0),
  `p93-partial-failure-recovery-real-confluence` (P0), `github-front-door-real-backend`
  (P1), `attach-sync-real-backend` (P1), `cadence-pre-release` (P1).
- NOT-VERIFIED: `t4-conflict-rebase-ancestry-real-backend` (P0) — creds LIVE, cleared
  the env-gate + space-KEY guard, then git-floored: `git 2.25.1 < 2.34`,
  `skip_reason: precondition-not-met`, bailed PRE-mutation (never reached the
  destructive scenario).

**TokenWorld end-state:** 2 protected (`7766017` / `7798785`, never delete) + 1
sacrificial editable (`2818063`); no residue — confirmed by RATIFICATION.md criterion 7
(PASS).

**Recorded caveats this tag carries** (full text: `RETROSPECTIVE.md` § Tag-prep arc):
1. t4 env-floor (Ruling #4) — an environment gap, not a product regression; the sim
   twin is green in CI; `reposix doctor` already treats sub-2.34 git as WARN, not ERROR.
2. Litmus non-idempotency vs its own mirror fan-out (Ruling #2) — RULED-DEFER→v0.15.0.
3. p93 CREATE-recovery WAIVED (item-7) — RULED-DEFER→v0.15.0; p93 PASS is an
   UPDATE-recovery proof only.

No open REOPEN-gate clock; no waiver on any P0 row (t4 stays runner-minted
NOT-VERIFIED, never waived).

## 5. Mid-execution decisions + noticed-not-filed

- **Manager Ruling #4 (E3 valve, CLOSED, Option B executed):** tag proceeds with t4
  honestly NOT-VERIFIED under a recorded caveat rather than an owner-authorized VM git
  upgrade. Full binding text: `.planning/CONSULT-DECISIONS.md` `2026-07-13 [MANAGER]
  Ruling #4`. Do not re-litigate.
- **Option A (VM git upgrade to ≥2.34, interactive sudo) is RAISED to the OWNER, not
  attempted by any coordinator** — no passwordless sudo exists on this VM; a real
  upgrade needs the owner present interactively. This is a live, unresolved item worth
  tracking even though it no longer blocks the tag.
- **CRITICAL clobber warning for the successor:** the gitignored per-run JSONs under
  `quality/reports/verifications/agent-ux/*.json` are currently **CLOBBERED to
  `env-missing`** by a creds-absent background run that executed AFTER `4645060`
  committed. This is EXPECTED clobber-noise, NOT a regression. The COMMITTED
  `quality/catalogs/agent-ux.json` (+ `VERDICT.md`) is the SOURCE OF TRUTH — do not
  re-mint off the transient JSONs, and do not re-run the cadence creds-absent. If a
  re-run is ever genuinely needed: `set -a && . ./.env && set +a && bash
  scripts/refresh-tokenworld-mirror.sh && python3 quality/runners/run.py --cadence
  pre-release-real-backend --persist`, run in the FOREGROUND (never
  background-and-exit — an orphaned background shell caused this session's clobber).
- **Noticed, filed this session** (`surprises-intake/part-04.md`, 4 entries + a
  carried-forward one-liner cluster; also `SURPRISES-INTAKE.md` terminal-entry index and
  `GOOD-TO-HAVES-16`): the 9th-probe cadence cannot reach honest exit-0 on a sub-2.34 VM;
  the JSON-clobber footgun (twice, once via an orphaned background shell); the
  background-dispatch hazard itself; `run.py` has no `.env` autoload so a naive dispatch
  silently env-gate-skips every real-backend row; `github-front-door-real-backend`'s
  runner-persist path leaves a stale cosmetic `skip_reason: env-missing` on an otherwise
  genuine PASS row (GOOD-TO-HAVES-16).
- **Carried, still unactioned** (low priority, do not drop): dead `PROTECTED_IDS`
  variable in `scripts/refresh-tokenworld-mirror.sh`; split-candidate tests
  (`crates/reposix-cli/tests/agent_flow_real.rs` ~47k chars, `translate.rs` ~26k); stale
  `surprises-intake/part-03.md:59-61` STATUS text; `run_helper_export_real` discards
  helper stderr; oversized `.planning/CONSULT-DECISIONS.md` (see § Hygiene below).

## 6. Precise next steps (successor runbook)

1. **Verify §1 ground truth live** — HEAD, clean tree, `0 0` vs origin, no `v0.14.0` tag
   anywhere. If any of these differ from this file, STOP and reconcile before proceeding
   (someone else moved state since this was written).
2. **Do not re-run the real-backend cadence and do not re-mint the VERDICT/RATIFICATION**
   unless HEAD has actually moved past `708b3e9` with new tag-relevant work — the
   GREEN-WITH-RECORDED-CAVEATS state at `5624943`/`708b3e9` is final for this tag.
3. **Manager pushes the tag**: run `bash
   .planning/milestones/v0.14.0-phases/tag-v0.14.0.sh` from repo root (interactive
   confirm required; it performs the push itself — there is no separate manual push
   step). This triggers `.github/workflows/release.yml` (tag pattern `v*`). Verify
   afterward: `git tag -l v0.14.0 && git ls-remote --tags origin v0.14.0` both show it,
   and `gh run list --workflow release.yml --limit 1` shows the triggered run.
4. **After the tag lands**, run the milestone-close/archive flow: move
   `.planning/milestones/v0.14.0-phases/` artifacts per the standard archive convention,
   and advance `.planning/STATE.md`'s workstream-C cursor — it currently still reads the
   stale `status: fix-first-items-4-8-pending` / `next_phase: fix-first-4-8`, which no
   longer reflects reality (items 4-8 are done; the tag-prep arc through Ruling #4 is
   done too). Update `blocks_tag`, `status`, and `next_phase` to reflect
   READY-TO-TAG → TAGGED once the tag push in step 3 confirms.
5. **Separately, surface Option A to the owner** (VM git upgrade to ≥2.34) as a
   standalone follow-up — it is not tag-blocking anymore under Ruling #4, but it is the
   only way to ever get t4's real-backend destructive scenario to genuinely execute on
   this VM. Do not action it unilaterally (no passwordless sudo; owner-interactive only).
6. **Do not** re-run the pre-release-real-backend cadence creds-absent, and do not treat
   the clobbered `quality/reports/verifications/agent-ux/*.json` files as signal (see §5).
7. At the next natural boundary, consider draining hygiene debt (not blocking, not
   urgent): `.planning/CONSULT-DECISIONS.md` is ~50k chars (Rulings #2/#3 are now
   CLOSED and could be pruned — git is the archive, nothing is lost); `GOOD-TO-HAVES.md`
   is ~27.6k chars (GOOD-TO-HAVES-02 residual drain still open); the known
   `ci-green-on-main.sh` race grades PASS off the newest `gh run` without asserting
   `headSha` matches pushed HEAD — cross-check `headSha` manually
   (`gh run list --branch main --workflow ci.yml --limit 5 --json headSha,status,conclusion`)
   until that gate is hardened.
