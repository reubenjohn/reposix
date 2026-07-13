# SESSION-HANDOVER.md — v0.14.0 item-5 CONFIRMED GREEN (real backend); tag BLOCKED on an OPEN E3 ruling — 2026-07-13 (→ successor #12)

v0.14.0 item-5 CONFIRMED GREEN (real backend); v0.14.0 tag BLOCKED on an OPEN E3 ruling
from 2 harness-gap rows (t4 P0 + github-front-door P1). For the incoming top-level
**ROUTING** coordinator — routes via GSD + subagents, **never leaf-works**. Reports to
the outer-loop MANAGER (herdr pane w1:p7), which watches this pane and relays owner
decisions. Map, not territory — detail lives in git + linked files. HEAD = live state
only; this REPLACES (does not append to) the prior handover — resume an agent via
SendMessage, never fork (ORCHESTRATION §11).

## 1. Ground truth (git) — VERIFY LIVE, do not trust this file's staleness

Before this handover commit, HEAD was `c5ad522`. Re-run and confirm before doing
anything: `git rev-parse --short HEAD && git status --porcelain && git rev-list
--left-right --count origin/main...HEAD`. Confirm CI GREEN on HEAD **by headSha
match** (NOT newest-run — the `ci-green-on-main.sh` gate race is real, see §7; cross-check
`headSha` manually every time via `gh run list --branch main --workflow ci.yml --limit 5
--json headSha,status,conclusion`).

- **Verified this session:** `3d6f509` (prior handover) → CI `29289278354` **success**.
  `24f9e97` → CI `29288966754` **success**. `d660e6e` → CI `29287664269` **success**.
  `c5ad522` (this session's newest code commit before this handover) → CI `29290936475`
  was still `in_progress` at verification time — **successor #12 must re-poll and confirm
  `success` before opening any further work.**
- **Commits since the last handover (`3d6f509`), oldest→newest — all planning/catalog
  docs only, no product code touched:**
  - `82498cc` — verify(9th-probe): reconcile `quality/catalogs/agent-ux.json` from a
    creds-loaded `pre-release-real-backend` run. Flips the two item-5 rows stale-FAIL →
    **PASS** off genuine real-backend transcripts.
  - `c5ad522` — docs(consult): record the OPEN E3 escalation (2 harness-gap rows block a
    clean cadence exit) in `.planning/CONSULT-DECISIONS.md`.
  - *(this handover commit lands on top)*

**item-5 is GENUINELY GREEN — the core v0.14.0 deliverable is DONE.** A fresh unbiased
CREDS-LOADED 9th-probe run graded `agent-ux/milestone-close-vision-litmus-real-backend`
(P0) + `agent-ux/p93-partial-failure-recovery-real-confluence` (P0) **BOTH PASS** against
live TokenWorld:
- litmus: real helper push round-trip to page `2818063` → REST confirm → dual-table
  audit → mirror-ref advance, exit 0.
- p93: real UPDATE-recovery `cargo test` against Confluence, exit 0.

Catalog reconciled FAIL→PASS and committed as `82498cc`. TokenWorld end-state verified:
exactly 3 pages, protected ids `7766017`+`7798785` intact, sacrificial `2818063`
present, **no deletions**.

`quality/reports/verdicts/milestone-v0.14.0/VERDICT.md` is **STILL RED on disk** (dated
2026-07-12, `head_at_grade: 9890a67`) — this predates everything above. Do NOT trust it;
re-minting is STEP 3.3.2, gated on the E3 ruling (§3) landing and the cadence going
exit-0.

## 2. CORRECTED MECHANISM KNOWLEDGE (the prior handover was FALSE here — overwrite it)

`quality/runners/run.py` does **NOT** source `.env`; it reads `os.environ` directly. A
fresh zero-context subagent shell inherits NO creds, so all 6 real-backend rows
env-gate-skip to NOT-VERIFIED and the cadence exits 1. **The prior handover's claim
("env vars only sourced from `.env` by the runner") was FALSE** and led a prior verifier
to grade the cadence as env-blocked when it was actually running creds-less.

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

## 3. THE OPEN BLOCKER (E3, pending manager/owner ruling — `.planning/CONSULT-DECISIONS.md` `c5ad522`)

The full `pre-release-real-backend` cadence is **exit-1**, SOLELY from two rows that are
**harness gaps, NOT product regressions**:

- **`agent-ux/t4-conflict-rebase-ancestry-real-backend` (P0):** guard literal accepts
  only space DISPLAY-NAME `"TokenWorld"`, but the sanctioned space KEY is `REPOSIX`
  (litmus/attach-sync/p93 all use `REPOSIX` successfully) → structurally can NEVER pass.
  Guard fires PRE-mutation (mass-delete-safe). Bug location:
  `quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh:161-164`. Its
  catalog `owner_hint`/comment (`agent-ux.json` ~:1744/:1768) are STALE/FALSE ("verifier
  script does not exist yet" — it exists, ran Jul 12, exit 1).
- **`agent-ux/github-front-door-real-backend` (P1):** `git-remote-reposix` not on PATH
  in the cadence harness → `reposix init github::...` dies with "unable to find remote
  helper for 'reposix'" before any GitHub REST call. (The litmus verifier bootstraps
  `target/debug` onto PATH; this one doesn't — sibling-verifier inconsistency.)

**Why blocked:** the prior handover (#11's, superseded) instructed "disregard the 2 rows
per their owner_hints" — that rests on a FALSIFIED premise (shipped-but-broken-harness,
not never-shipped/verifier-not-found), and **OD-2 forbids waiving a P0 row or a
non-exit-0 cadence at milestone close.** The VERDICT cannot be re-minted GREEN as-is.

**The two options put to the manager (`CONSULT-DECISIONS.md`, status OPEN):**
- **(A, recommended)** authorize the 2 small NON-PRODUCT harness fixes + a cadence
  re-run to exit-0.
- **(B)** re-scope these rows out of the v0.14.0 gate / defer to v0.15.0 via a mechanism
  that does NOT waive a P0.

## 4. RESUME RUNBOOK (branch on the manager's ruling)

**Step 0:** verify §1 ground truth + CI-green-by-headSha. Read the manager's ruling in
`.planning/CONSULT-DECISIONS.md` (search for the E3 OPEN entry — it will be marked
RULED once the manager responds) / the herdr pane.

**If ruling = A (fix harness):** dispatch executor lane(s):
1. Fix the t4 guard to recognize the sanctioned space by KEY `REPOSIX` (or validate
   tenant + protected-ids instead of the display-name literal) — PRESERVE the
   mass-delete-safety intent + protected-id protection; refresh the stale
   `owner_hint`/comment. **NOTE:** t4 runs a DESTRUCTIVE rebase-ancestry test against
   live TokenWorld — protected ids `7766017`/`7798785` must stay protected throughout.
2. Fix the github-front-door harness to put `target/debug` on PATH (mirror the litmus
   verifier's bootstrap).
3. Commit each atomically.
4. Re-run the CREDS-LOADED cadence (fresh unbiased `gsd-verifier`, §2 command) → confirm
   **exit-0**.
5. → STEP 3.3 below.

**If ruling = B (re-scope/defer):** apply the manager's named mechanism, re-run the
cadence → exit-0, → STEP 3.3 below.

**STEP 3.3 mechanicals (unchanged shape from prior handovers; in order; STOP at
READY-TO-TAG, NEVER push the tag):**

1. **OP-9 retro distillation FIRST** (the ratifier grades RED without it) — finalize
   `.planning/RETROSPECTIVE.md`'s v0.14.0 section per the gap list: distill the 8
   `part-03.md` intakes, the item-5 string-encoded-ADF regression + root cause + fix
   (`49666eb`), the DP-2 mechanism review + hollow-real-twin finding, the item-5
   test-fix lane 5a–5d, the litmus non-idempotency finding (RULED-DEFER→v0.15.0, Ruling
   #2 — quote verbatim, see §5), AND item-7's CREATE-recovery WAIVED flag (quote
   verbatim, see §5). ALSO distill THIS session's discovery: the `run.py` no-dotenv
   mechanism gap + the 2 harness-gap rows (§2, §3).
2. Re-mint `quality/reports/verdicts/milestone-v0.14.0/VERDICT.md` GREEN off the FRESH
   creds-loaded probe artifacts (update `head_at_grade` to the current HEAD). The stale
   on-disk VERDICT is dated 2026-07-12 / `head_at_grade 9890a67` — do NOT trust it.
3. Dispatch a FRESH unbiased ratification subagent per
   `quality/dispatch/milestone-close-verdict.md` (author ≠ minter, zero prior context).
4. Author `.planning/milestones/v0.14.0-phases/tag-v0.14.0.sh`, patterned on
   `.planning/milestones/v0.13.0-phases/tag-v0.13.0.sh.disabled` (that v0.13 script is
   DISABLED — also see `.planning/milestones/v0.12.0-phases/tag-v0.12.0.sh` as an
   active-pattern reference). Clean-tree + signed-tag guards.
5. **STOP at READY-TO-TAG.** Compact report to MANAGER w1:p7: SHAs, artifact paths,
   cadence exit-0 + per-row grades, TokenWorld end state. **Manager pushes the tag** —
   never you.

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

**NEW this session — file to `SURPRISES-INTAKE.md` / `GOOD-TO-HAVES.md`, do not drop:**

- (a) `run.py` has no `.env` autoload — structurally breaks the mandated unbiased-
  subagent 9th-probe dispatch pattern (a naive dispatch silently env-gate-skips every
  real-backend row instead of failing loud). Fix sketch: add `--env-file`/dotenv
  autoload to `run.py`, OR codify the in-shell-source dispatch pattern (§2) as the
  ONLY sanctioned invocation in `quality/PROTOCOL.md`.
- (b) `--persist` creds-absent clobber footgun — should refuse to overwrite when every
  in-scope row env-skips, or refuse to clobber `last_real_grade` with a value staler
  than an existing PASS transcript.
- (c) t4's stale `owner_hint`/comment in `agent-ux.json` (says "verifier not
  implemented"; it is implemented and has run).
- (d) github-front-door's PATH gap + the sibling-verifier PATH inconsistency (litmus
  bootstraps `target/debug` onto PATH; github-front-door doesn't).

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

- `.planning/CONSULT-DECISIONS.md` is **~42.7k chars** (over the 20k soft-warn, grew
  again this session) — prune CLOSED entries (the DROP/HALT item-5 chain, RESOLVED B1,
  and Ruling #2 once fully acted on; DELETE this session's OPEN E3 entry once the
  manager rules + it's acted on) — git is the archive, nothing is lost by pruning.
- `.planning/milestones/v0.14.0-phases/surprises-intake/part-03.md` is **~27.3k chars**
  (over the 20k soft-warn) — OP-8 drain candidate.
- **KNOWN GATE RACE (HIGH, filed, not fixed):** `ci-green-on-main.sh` grades PASS off
  the newest `gh run` WITHOUT asserting `headSha` == pushed HEAD — cross-check `headSha`
  manually every time (§1).

---
History lives in git — `git log` / `git show`, not restated here.
