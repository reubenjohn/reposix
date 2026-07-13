# SESSION-HANDOVER.md — v0.14.0 item-5 CONFIRMED GREEN; tag work = execute MANAGER RULING #3 (OPTION A) — 2026-07-13 (→ successor #12)

v0.14.0 item-5 CONFIRMED GREEN; tag work = execute MANAGER RULING #3 (OPTION A) — fix 2
harness gaps → cadence exit-0 → STEP 3.3 → STOP at READY-TO-TAG. For the incoming
top-level **ROUTING** coordinator — routes via GSD + subagents, **never leaf-works**.
Reports to the outer-loop MANAGER (herdr pane w1:p7), which watches this pane and
relays owner decisions. Map, not territory — detail lives in git + linked files. HEAD =
live state only; this REPLACES (does not append to) the prior handover — resume an
agent via SendMessage, never fork (ORCHESTRATION §11).

## 1. Ground truth (git) — VERIFY LIVE, do not trust this file's staleness

Re-run and confirm before doing anything: `git rev-parse --short HEAD && git status
--porcelain && git rev-list --left-right --count origin/main...HEAD`. Confirm CI GREEN
on HEAD **by headSha match** (NOT newest-run — the `ci-green-on-main.sh` gate race is
real, see §7; cross-check `headSha` manually every time via `gh run list --branch main
--workflow ci.yml --limit 5 --json headSha,status,conclusion`).

- **Verified this session:** `3d6f509` (prior handover) → CI `29289278354` **success**.
  `24f9e97` → CI `29288966754` **success**. `d660e6e` → CI `29287664269` **success**.
  `c5ad522` (OPEN E3 escalation recorded) — CI still needs headSha re-poll by
  successor #12 before opening further work.
- **Commits since the last handover (`3d6f509`), oldest→newest — all planning/catalog
  docs only, no product code touched:**
  - `82498cc` — verify(9th-probe): reconcile `quality/catalogs/agent-ux.json` from a
    creds-loaded `pre-release-real-backend` run. Flips the two item-5 rows stale-FAIL →
    **PASS** off genuine real-backend transcripts.
  - `c5ad522` — docs(consult): record the (now-RULED) E3 escalation in
    `.planning/CONSULT-DECISIONS.md`.
  - `7cf08a9` — docs(handover): first cut of this file for successor #12 (OPEN-blocker
    framing — since SUPERSEDED by the edit below, same file).
  - *(this commit)* — docs(handover+consult): fold Manager Ruling #3 (RULED OPTION A)
    into this file + the ledger, in one commit, per MANAGER instruction.

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
re-minting is STEP 3.3.2, gated on the cadence going exit-0 per Ruling #3 below.

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

## 3. THE BLOCKER — RULED (Ruling #3, manager, standing authority) — execute OPTION A

Manager Ruling #3 (`.planning/CONSULT-DECISIONS.md`, 2026-07-13) **RESOLVED** the prior
OPEN E3 escalation. **OPTION A is AUTHORIZED. OPTION B (re-scope) is REJECTED.** The full
`pre-release-real-backend` cadence was exit-1 SOLELY from two rows that are **harness
gaps, NOT product regressions**:

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

**Ruling #3 decision: OPTION A** — fix both harness gaps, re-run the cadence to an
honest exit-0. Item-5 litmus+p93 GREEN is acknowledged (`82498cc`). **OPTION B
REJECTED** — no P0 re-scope at milestone close; OD-2 is honored via a real exit-0, not
scope surgery.

**Binding guardrails (manager, standing authority) — carry verbatim:**
1. **t4**: guard accepts the sanctioned space KEY `REPOSIX`; KEEP the fail-closed
   PRE-mutation placement; ADD tenant + protected-id validation so `7766017`/`7798785`
   are structurally UNTOUCHABLE. The destructive rebase-ancestry run against TokenWorld
   is PRE-AUTHORIZED under the owner's standing real-backend mutation authority, WITH
   guardrails: protected pair untouched; post-run end-state verified via `python3
   scripts/confluence_tokenworld.py list` (2 protected + `2818063` current); any test
   residue cleaned or filed.
2. **github-front-door**: put `git-remote-reposix` (the built binary) on the harness
   PATH; do NOT weaken the test itself.
3. **Fix-it-twice**: correct BOTH rows' STALE owner_hint prose ("verifier script does
   not exist yet" is false) in the SAME commit; row STATUS stays runner-minted, never
   hand-set.
4. **Re-run decision rule**: if t4 then fails for a PRODUCT reason → HALT + escalate to
   manager w1:p7 with the transcript (no waive, no rushed fix); if it fails on ANOTHER
   harness gap → bounded harness fix + re-run autonomously.

Then STEP 3.3 as chartered (§4). Reversibility: no product code touched; reconcile
`82498cc` is a pure catalog-truth correction.

## 4. RESUME RUNBOOK — execute Ruling #3 (OPTION A), then STEP 3.3

**Step 0:** verify §1 ground truth + CI-green-by-headSha before doing anything else.

**Step 1 — fix the two harness gaps (single execution path, no branching — Option B is
rejected):**
1. Fix the t4 guard per guardrail 1 (§3) — accept the sanctioned space KEY `REPOSIX`,
   KEEP the fail-closed PRE-mutation placement, ADD tenant + protected-id validation.
   **NOTE:** t4 runs a DESTRUCTIVE rebase-ancestry test against live TokenWorld —
   protected ids `7766017`/`7798785` must stay protected throughout (this run is
   PRE-AUTHORIZED per guardrail 1, with the post-run end-state check named there).
2. Fix the github-front-door harness per guardrail 2 — put `target/debug` (or
   equivalent) on PATH; do NOT weaken the test itself.
3. Fix-it-twice: correct BOTH rows' stale `owner_hint`/comment prose in the SAME
   commit(s) as their respective fixes (guardrail 3); row STATUS stays runner-minted,
   never hand-set.
4. Re-run the CREDS-LOADED cadence (fresh unbiased `gsd-verifier`, §2 command) and
   apply the re-run decision rule (guardrail 4): a PRODUCT-reason failure → HALT +
   escalate to manager w1:p7 with the transcript (no waive, no rushed fix); ANOTHER
   harness gap → bounded fix + re-run autonomously. Target: confirm **exit-0**.

**Step 2 — STEP 3.3 mechanicals (in order; STOP at READY-TO-TAG, NEVER push the tag):**
1. **OP-9 retro distillation FIRST** (the ratifier grades RED without it) — finalize
   `.planning/RETROSPECTIVE.md`'s v0.14.0 section per the gap list: distill the 8
   `part-03.md` intakes, the item-5 string-encoded-ADF regression + root cause + fix
   (`49666eb`), the DP-2 mechanism review + hollow-real-twin finding, the item-5
   test-fix lane 5a–5d, the litmus non-idempotency finding (RULED-DEFER→v0.15.0, Ruling
   #2 — quote verbatim, see §5), AND item-7's CREATE-recovery WAIVED flag (quote
   verbatim, see §5). ALSO distill this session's discoveries: the `run.py` no-dotenv
   mechanism gap (§2) and the Ruling #3 harness-gap fix arc (§3).
2. Re-mint `quality/reports/verdicts/milestone-v0.14.0/VERDICT.md` GREEN off the FRESH
   creds-loaded exit-0 probe artifacts (update `head_at_grade` to the current HEAD). The
   stale on-disk VERDICT is dated 2026-07-12 / `head_at_grade 9890a67` — do NOT trust it.
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
- **Ruling #3 (E3 valve, this session, RULED OPTION-A):** the full binding-guardrails
  text is quoted verbatim in §3 above — pull it directly from there (and from
  `.planning/CONSULT-DECISIONS.md`) into the retro/READY-TO-TAG report; do not
  paraphrase the four guardrails or the re-run decision rule.

**NEW this session — file to `SURPRISES-INTAKE.md` / `GOOD-TO-HAVES.md`, do not drop:**

- (a) `run.py` has no `.env` autoload — structurally breaks the mandated unbiased-
  subagent 9th-probe dispatch pattern (a naive dispatch silently env-gate-skips every
  real-backend row instead of failing loud). Fix sketch: add `--env-file`/dotenv
  autoload to `run.py`, OR codify the in-shell-source dispatch pattern (§2) as the
  ONLY sanctioned invocation in `quality/PROTOCOL.md`.
- (b) `--persist` creds-absent clobber footgun — should refuse to overwrite when every
  in-scope row env-skips, or refuse to clobber `last_real_grade` with a value staler
  than an existing PASS transcript.
- (c) t4's stale `owner_hint`/comment in `agent-ux.json` — being fixed in Step 1.3 of
  §4, but file it anyway so the pattern (owner_hint prose going stale after a verifier
  ships) gets a durable record.
- (d) github-front-door's PATH gap + the sibling-verifier PATH inconsistency (litmus
  bootstraps `target/debug` onto PATH; github-front-door doesn't) — being fixed in Step
  1.2 of §4, file the pattern anyway.

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

- `.planning/CONSULT-DECISIONS.md` — prune CLOSED entries at a clean moment (the
  DROP/HALT item-5 chain, RESOLVED B1, Ruling #2 and Ruling #3 once fully acted on) —
  git is the archive, nothing is lost by pruning. Was ~42.7k chars before this session's
  Ruling #3 edit (over the 20k soft-warn) — re-check size after this commit.
- `.planning/milestones/v0.14.0-phases/surprises-intake/part-03.md` is **~27.3k chars**
  (over the 20k soft-warn) — OP-8 drain candidate.
- **KNOWN GATE RACE (HIGH, filed, not fixed):** `ci-green-on-main.sh` grades PASS off
  the newest `gh run` WITHOUT asserting `headSha` == pushed HEAD — cross-check `headSha`
  manually every time (§1).

---
History lives in git — `git log` / `git show`, not restated here.
