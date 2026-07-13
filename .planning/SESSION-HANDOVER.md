# SESSION-HANDOVER.md — Ruling #3 Option A EXECUTED at cb8ad11 (both fixes proven); tag BLOCKED on a NEW OPEN escalation — git 2.25.1 < 2.34 floors t4; awaiting manager/owner ruling A vs B — 2026-07-13 (→ successor #13)

Ruling #3 Option A EXECUTED at `cb8ad11` — both harness fixes (t4 space-KEY,
github-front-door PATH) landed and PROVEN via a fresh creds-loaded re-run. Tag work is
now BLOCKED on a NEW OPEN escalation: VM git 2.25.1 < 2.34 floors t4 to NOT-VERIFIED;
awaiting manager/owner ruling A (git upgrade) vs B (gate-policy) → successor #13. For
the incoming top-level **ROUTING** coordinator — routes via GSD + subagents, **never
leaf-works**. Reports to the outer-loop MANAGER (herdr pane w1:p7), which watches this
pane and relays owner decisions. Map, not territory — detail lives in git + linked
files. HEAD = live state only; this REPLACES (does not append to) the prior handover —
resume an agent via SendMessage, never fork (ORCHESTRATION §11).

## 1. Ground truth (git) — VERIFY LIVE, do not trust this file's staleness

Re-run and confirm before doing anything: `git rev-parse --short HEAD && git status
--porcelain && git rev-list --left-right --count origin/main...HEAD`. Confirm CI GREEN
on HEAD **by headSha match** (NOT newest-run — the `ci-green-on-main.sh` gate race is
real, see §7; cross-check `headSha` manually every time via `gh run list --branch main
--workflow ci.yml --limit 5 --json headSha,status,conclusion`).

- **Verified this session:** HEAD = `cb8ad11` (Ruling #3 harness fixes landed) → CI
  `29292131694` **success**, headSha-matched. Tree clean, `0 0` vs `origin/main`.
- **Commits since the last handover (`2ea4b3d`), oldest→newest:**
  - `cb8ad11` — fix(harness): t4 accepts space KEY `REPOSIX` + github-front-door
    builds/PATHs the helper (Ruling #3 Option A). Product-adjacent test-harness fix, no
    reposix-core/cli/cache/remote code touched.
  - *(this commit)* — docs(consult+handover): record the OPEN git-floor escalation (t4
    git 2.25.1<2.34) discovered by the post-fix re-run; refresh this file for
    successor #13.

**item-5 is STILL GENUINELY GREEN — the core v0.14.0 deliverable remains DONE**, proven
again on the SAME creds-loaded re-run that surfaced the new blocker (below):
`milestone-close-vision-litmus-real-backend` (P0) and
`p93-partial-failure-recovery-real-confluence` (P0) both **PASS** against live
TokenWorld. Nothing in this session's findings touches that result.

`quality/reports/verdicts/milestone-v0.14.0/VERDICT.md` is **STILL RED on disk** (dated
2026-07-12, `head_at_grade: 9890a67`) — do NOT trust it; re-minting is STEP 3.3.2,
gated on the cadence going exit-0, which is now further gated on THIS session's OPEN
escalation being ruled (§3).

## 2. CORRECTED MECHANISM KNOWLEDGE (a prior handover was FALSE here — this supersedes it)

`quality/runners/run.py` does **NOT** source `.env`; it reads `os.environ` directly. A
fresh zero-context subagent shell inherits NO creds, so all 6 real-backend rows
env-gate-skip to NOT-VERIFIED and the cadence exits 1. **An older handover's claim ("env
vars only sourced from `.env` by the runner") was FALSE** and led a prior verifier to
grade the cadence as env-blocked when it was actually running creds-less.

To run the real-backend cadence for real, the dispatched verifier MUST source `.env` in
the SAME Bash invocation as `run.py` (shell state does NOT persist across Bash calls):

```bash
cd /home/reuben/workspace/reposix && set -a && . ./.env && set +a && \
  bash scripts/refresh-tokenworld-mirror.sh && \
  python3 quality/runners/run.py --cadence pre-release-real-backend --persist
```

Pattern precedent: `scripts/refresh-tokenworld-mirror.sh:~70`. `.env` **is present**
with working creds (gitignored — never echo/export/commit a secret).

**`--persist` footgun:** running the cadence in a creds-ABSENT shell overwrites real
committed grades with env-missing NOT-VERIFIED and rewrites `last_real_grade` — a prior
verifier had to revert the catalog to avoid corrupting the audit record. **Only
`--persist` from a creds-LOADED shell.**

## 3. THE BLOCKER — Ruling #3 is DONE (executed + proven); a NEW OPEN escalation blocks the tag

**Ruling #3 (Option A) is FULLY EXECUTED.** Both harness fixes landed at `cb8ad11` and
were PROVEN by a fresh creds-loaded re-run:
`github-front-door-real-backend` FAIL→PASS (helper on PATH); t4's space-KEY guard now
accepts the sanctioned `REPOSIX` key and clears the section-2 guard. Do NOT re-open or
re-litigate Ruling #3 — it is closed business.

**But the same re-run surfaced a THIRD, distinct gap Ruling #3 never contemplated**,
now recorded as a fresh OPEN entry in `.planning/CONSULT-DECISIONS.md`
(`2026-07-13 [OWNER]`, git-floor): the cadence graded **5 PASS / 0 FAIL / 1
NOT-VERIFIED → exit 1**. The sole holdout is
`agent-ux/t4-conflict-rebase-ancestry-real-backend` (P0), which now clears the
space-KEY guard but then hits a hard environment precondition:
`asserts_failed ["git 2.25.1 < 2.34"]`, `skip_reason precondition-not-met`
(`quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh:179-192`). Only git
2.25.1 exists on this VM (no ≥2.34 anywhere), and passwordless sudo is NOT available, so
a VM-wide system git upgrade is an E1 environment mutation outside a coordinator's
unilateral authority.

**Two options are on the table (full text in the ledger entry — quote from there, do
not paraphrase in the retro/READY-TO-TAG report):**
- **A** — owner authorizes + runs an interactive-sudo git upgrade to ≥2.34, then a fresh
  creds-loaded re-run lets t4 actually execute its pre-authorized destructive scenario
  (honest PASS or a genuine product FAIL to re-escalate).
- **B** — owner ratifies t4 NOT-VERIFIED as a documented environment limitation for the
  v0.14.0 tag (the non-skippable litmus + p93 already PASSED live); this partially
  reopens Ruling #3's "no P0 re-scope" stance, so it needs fresh owner ratification, not
  a coordinator's unilateral call.

**HALT until ruled.** Do NOT unilaterally upgrade system git (E1, no passwordless
sudo). Do NOT waive the t4 P0 row without an explicit Option-B ratification landing in
the ledger first.

## 4. RESUME RUNBOOK — branch on the new ruling, then STEP 3.3

**Step 0:** verify §1 ground truth + CI-green-by-headSha before doing anything else.
Then read the OPEN `2026-07-13 [OWNER]` entry in `.planning/CONSULT-DECISIONS.md` for
the ruling outcome (A or B) before proceeding — do not assume.

**Step 1 — branch on the ruling (this is new; supersedes any prior Step-1 framing):**

- **IF RULED A:** the owner upgrades system git to ≥2.34 via interactive sudo (verify
  `git --version` reports ≥2.34 afterward). Then dispatch a FRESH creds-loaded
  `gsd-verifier` to re-run `pre-release-real-backend` via the §2 in-shell-source
  command. Apply the standing re-run decision rule (Ruling #3 guardrail 4, still
  binding): a PRODUCT-reason t4 failure → HALT + escalate to manager w1:p7 with the
  transcript (no waive, no rushed fix); ANOTHER harness/env gap → bounded fix + re-run
  autonomously; a clean exit-0 → proceed straight to Step 3.3 below.
- **IF RULED B:** proceed straight to Step 3.3 below; the VERDICT + RETROSPECTIVE must
  explicitly document t4 as an environment-limited NOT-VERIFIED (not a silent waiver),
  with litmus + p93 carrying item-5's GREEN status as the load-bearing evidence.

**Step 2 — STEP 3.3 mechanicals (unchanged from prior handovers; in order; STOP at
READY-TO-TAG, NEVER push the tag):**
1. **OP-9 retro distillation FIRST** (the ratifier grades RED without it) — finalize
   `.planning/RETROSPECTIVE.md`'s v0.14.0 section per the gap list: distill the 8
   `part-03.md` intakes, the item-5 string-encoded-ADF regression + root cause + fix
   (`49666eb`), the DP-2 mechanism review + hollow-real-twin finding, the item-5
   test-fix lane 5a–5d, the litmus non-idempotency finding (RULED-DEFER→v0.15.0, Ruling
   #2 — quote verbatim, see §5), item-7's CREATE-recovery WAIVED flag (quote verbatim,
   see §5), the Ruling #3 harness-gap fix arc (§3, now closed), AND this session's
   git-floor escalation + its ruling outcome (quote the ledger entry, do not
   paraphrase).
2. Re-mint `quality/reports/verdicts/milestone-v0.14.0/VERDICT.md` GREEN off the
   ruling-appropriate exit-0 (Ruled A) or ratified (Ruled B) probe artifacts (update
   `head_at_grade` to the current HEAD). The stale on-disk VERDICT is dated 2026-07-12 /
   `head_at_grade 9890a67` — do NOT trust it.
3. Dispatch a FRESH unbiased ratification subagent per
   `quality/dispatch/milestone-close-verdict.md` (author ≠ minter, zero prior context).
4. Author `.planning/milestones/v0.14.0-phases/tag-v0.14.0.sh`, patterned on
   `.planning/milestones/v0.13.0-phases/tag-v0.13.0.sh.disabled` (that v0.13 script is
   DISABLED — also see `.planning/milestones/v0.12.0-phases/tag-v0.12.0.sh` as an
   active-pattern reference). Clean-tree + signed-tag guards.
5. **STOP at READY-TO-TAG.** Compact report to MANAGER w1:p7: SHAs, artifact paths,
   cadence result + per-row grades, TokenWorld end state, which ruling (A/B) was
   executed. **Manager pushes the tag** — never you.

## 5. Mid-execution decisions + noticed-not-filed

**Carry verbatim into the RETROSPECTIVE / READY-TO-TAG report (do not paraphrase):**

- **Ruling #2 (litmus non-idempotency, closed, DEFER→v0.15.0):** "litmus is
  non-idempotent against its own GitHub mirror fan-out (pre-write client tree, not
  post-write materialized snapshot) — a pre-existing ADR-010 RBF-LR-04 fan-out
  characteristic, not a v0.14.0 regression. Product fix (mirror-sync pushing the
  post-write snapshot) routed to v0.15.0. The interim op — one clean
  `refresh-tokenworld-mirror.sh` run before the 9th probe — legitimately grades item-5
  GREEN for this tag."
- **Item-7 WAIVED flag (CREATE-recovery, resolved DEFER→v0.15.0):** "p93 is GREEN as an
  UPDATE-recovery proof against live TokenWorld (`1c424d7`). It does NOT cover
  CREATE-recovery: a partial-fail whose landed action was a create against an
  id-assigning backend genuinely does not converge. This is the owner-signed WAIVED
  known limitation of ADR-010 §3 / RBF-LR-03, hand-recoverable, routed to v0.15.0."
  (Separate from item-5's now-fixed UPDATE-recovery RED.)
- **Ruling #3 (E3 valve, EXECUTED + PROVEN, RULED OPTION-A, now closed):** the full
  binding-guardrails text is quoted verbatim in `.planning/CONSULT-DECISIONS.md`
  (`2026-07-13 [MANAGER] Ruling #3`) — pull it directly from there into the
  retro/READY-TO-TAG report; do not paraphrase the four guardrails or the re-run
  decision rule. Both harness gaps it targeted are FIXED and PROVEN at `cb8ad11`.

**NEW this session — file to `SURPRISES-INTAKE.md` / `GOOD-TO-HAVES.md`, do not drop:**

- (a) `run.py` has no `.env` autoload — structurally breaks the mandated unbiased-
  subagent 9th-probe dispatch pattern (a naive dispatch silently env-gate-skips every
  real-backend row instead of failing loud). Fix sketch: add `--env-file`/dotenv
  autoload to `run.py`, OR codify the in-shell-source dispatch pattern (§2) as the
  ONLY sanctioned invocation in `quality/PROTOCOL.md`.
- (b) `--persist` creds-absent clobber footgun — should refuse to overwrite when every
  in-scope row env-skips, or refuse to clobber `last_real_grade` with a value staler
  than an existing PASS transcript.
- (c) t4's stale `owner_hint`/comment in `agent-ux.json` — FIXED at `cb8ad11`, but file
  the pattern anyway (owner_hint prose going stale after a verifier ships) for a
  durable record.
- (d) github-front-door's PATH gap + the sibling-verifier PATH inconsistency (litmus
  bootstraps `target/debug` onto PATH; github-front-door didn't) — FIXED at `cb8ad11`,
  file the pattern anyway.
- (e) THIRD harness/env gap — t4 real-backend floors at git 2.25.1 < 2.34; Ruling #3's
  space-KEY fix is UNVALIDATED until git ≥2.34 (t4 never reached the destructive flow on
  this VM). File to SURPRISES-INTAKE.md: the milestone-close 9th-probe cadence cannot
  reach honest exit-0 on a sub-2.34 VM; consider a documented ≥2.34 requirement for the
  real-backend cadence env OR a CI env with git ≥2.34.

**CARRIED forward — still unfiled/unactioned from prior handovers, do not drop:**

- Dead `PROTECTED_IDS` variable in `scripts/refresh-tokenworld-mirror.sh` (defined but
  the guard loop hardcodes `7766017 7798785` literally instead of reading it).
- Split-candidate tests: `crates/reposix-cli/tests/agent_flow_real.rs` (~47k chars) and
  `translate.rs` (~26k) — flagged, not yet filed as a formal row.
- Re-STATUS `surprises-intake/part-03.md:59-61` (stale "active p93 blocker" language,
  overtaken by `1c424d7`).
- File a v0.15.0 GOOD-TO-HAVE for slug→id create-reconciliation — confirm overlap with
  the already-filed GTH-09 (broader ADR-010 redesign) before filing a duplicate.
- LOW: title-sweep 2 pre-rewrite "p93 smoke A" orphan pages still sitting in TokenWorld.
- `run_helper_export_real` discards helper stderr (opaque real-backend failures).

## 6. Binding constraints (carry verbatim)

Sim-first for code; real backends only via `REPOSIX_ALLOWED_ORIGINS`; TokenWorld = 2
PROTECTED (`7766017`+`7798785`, NEVER delete) + 1 SACRIFICIAL EDITABLE (`2818063`); to
run the real-backend cadence, SOURCE `.env` in-shell (§2); **NO tag push ever**
(manager's); never open work over a red main; ONE cargo invocation machine-wide (prefer
`-p`); /tmp leaf isolation (`cd` in the SAME bash invocation); single-writer discipline
(one tree-mutating agent at a time; read-only may parallelize); relief ~100k soft /
~150k hard context (absolute, not %) → REPLACE this file, commit+push, end turn, manager
rotates. A `fork` is never a safe discard. Resume a child via SendMessage, never fork.

## 7. Hygiene debt (carry)

- `.planning/CONSULT-DECISIONS.md` — grew again with this session's git-floor
  escalation (on top of Ruling #3). Prune CLOSED entries at a clean moment: the
  DROP/HALT item-5 chain, RESOLVED B1, Ruling #2, and Ruling #3 once fully acted on —
  git is the archive, nothing is lost by pruning.
- `.planning/milestones/v0.14.0-phases/surprises-intake/part-03.md` is **~27.3k chars**
  (over the 20k soft-warn) — OP-8 drain candidate.
- **KNOWN GATE RACE (HIGH, filed, not fixed):** `ci-green-on-main.sh` grades PASS off
  the newest `gh run` WITHOUT asserting `headSha` == pushed HEAD — cross-check `headSha`
  manually every time (§1).

---
History lives in git — `git log` / `git show`, not restated here.
