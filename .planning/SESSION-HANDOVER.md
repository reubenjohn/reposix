# SESSION-HANDOVER.md — v0.14.0 item-5 test-fix + guardrail-5 doc-truth DONE; item-8 mechanicals (retro distill → verdict remint → ratify → tag-script author → STOP) queued — 2026-07-13 (→ successor #11)

For the incoming top-level workhorse (L0) — a top-level **ROUTING** coordinator: routes
via GSD + subagents, never leaf-works. Reports to the outer-loop MANAGER (herdr pane
w1:p7), which watches this pane and relays owner decisions. Map, not territory — detail
lives in git + linked files. HEAD = live state only; this replaces (does not append to)
the prior handover — resume an agent via SendMessage, never fork (ORCHESTRATION §11).

## 1. Ground truth (git) — VERIFY LIVE, do not trust this file's staleness

Verified this session via `git rev-parse --short HEAD`, `git status --porcelain`,
`git rev-list --left-right --count origin/main...HEAD`, `gh run list --branch main
--workflow ci.yml --limit 5 --json headSha,status,conclusion`, `gh run view <id> --json
jobs`:

- **HEAD = `24f9e97`**, tree **clean** (`git status --porcelain` empty), **0 ahead / 0
  behind `origin/main`** — already pushed. Re-verify these three before doing anything.
- **CI:** newest `ci.yml` run (`29288966754`) is **`in_progress`** on `24f9e97` — a
  planning+docs+catalog-only push, expected PASS. Prior run (`29287664269`) on `d660e6e`
  (STEP-2 code) **concluded SUCCESS**, and I confirmed via `gh run view --json jobs` that
  the `integration (contract, real confluence v09)` job (plus the `real github`/`real
  jira`/non-v09-real-confluence jobs) all show `conclusion: success` — the item-5 fix
  landed clean against CI including the real-Confluence contract lane.
  **Successor #11's first action: re-poll `gh run list` for `24f9e97`'s headSha and
  confirm `conclusion: success` (not just `status: completed`) before opening STEP
  3.2/3.3 — never open work over a red main.**
- **Manager Ruling #2 (2026-07-13, `a905bd0` + `04f985d`):** litmus non-idempotency
  **DEFER → v0.15.0; the v0.14.0 tag PROCEEDS.** Recorded verbatim in
  `.planning/CONSULT-DECISIONS.md` (`## 2026-07-13 [MANAGER] Ruling #2 …`); the
  `surprises-intake/part-03.md` non-idempotency row is flipped to `RULED-DEFER→v0.15.0`.
  This ruling is **closed** — do not re-litigate it, only carry the caveat forward at
  tag time (§4 below).
- **Commits since the last handover (`a905bd0`), oldest→newest:**
  - `04f985d` — record Ruling #2 in CONSULT-DECISIONS + flip part-03 row (STEP 0, done).
  - `c2eb2ad` — fix(confluence): default `ConfBodyAdf` inner `Raw.value` to `Null`,
    averts a list-wide DoS (item 5c).
  - `c7ae07a` — test(cli): make the real-TokenWorld ADF-decode twin non-vacuous via
    `get_record` (item 5a) — **executed against live TokenWorld, PASSED**.
  - `e504121` — test(swarm): string-encode ADF value in `mini_e2e.rs` fixture to match
    the real wire shape (item 5b).
  - `ad62dd3` — fix(mirror-script): `set -e`, non-circular verify, deletion-aware
    overlay in `refresh-tokenworld-mirror.sh` (item 5d).
  - `d660e6e` — style: trim mirror-script comments to the 160-line readability cap
    (item 5d, CI GREEN incl. real-confluence contract — verified above).
  - `b7e79ab` — docs(testing-targets): document sacrificial editable page `2818063`,
    kill the "exactly 2 pages" myth (guardrail 5, doc-truth edit).
  - `24f9e97` — refresh(doc-alignment): re-bind 13 drifted rows in
    `testing-targets.md` that the doc-truth edit shifted to STALE_DOCS_DRIFT (guardrail
    5, mint pass by an unbiased Opus grader — pre-push 61 PASS / 0 FAIL / exit 0).
- `quality/reports/verdicts/milestone-v0.14.0/VERDICT.md` is **STILL RED** on disk
  (dated 2026-07-12, `head_at_grade: 9890a67` — predates the item-5 fix entirely). This
  is expected staleness, not a regression: it has not been re-minted since the fix
  landed. Re-minting it is part of STEP 3.3 below, gated on a fresh 9th-probe PASS.

## 2. Wave/cycle state

| Wave / Step | State | Commits |
|---|---|---|
| STEP 0 — record Manager Ruling #2 | DONE | `04f985d` |
| STEP 2a — non-vacuous real-TokenWorld ADF twin | DONE (executed+PASSED live) | `c7ae07a` |
| STEP 2b — `mini_e2e.rs` fixture string-encode fix | DONE | `e504121` |
| STEP 2c — `ConfBodyAdf` DoS fix (`#[serde(default)]`) | DONE | `c2eb2ad` |
| STEP 2d — mirror-script `set -e` + non-circular verify | DONE | `ad62dd3`, `d660e6e` |
| STEP 3.1 — guardrail-5 doc-truth edit + docs-alignment re-bind | DONE (pre-push 61/61 GREEN) | `b7e79ab`, `24f9e97` |
| **STEP 3.2 — 9th-probe unbiased verifier (litmus + p93 grade)** | **NOT STARTED** | — |
| **STEP 3.3 — item-8 mechanicals (retro → verdict → ratify → tag-script → STOP)** | **NOT STARTED** | — |

No named incident this session — the wave ran clean. The only open question resolved
mid-flight was Ruling #2 (§1), now closed.

## 3. Binding constraints (unchanged, carry verbatim)

Sim-first for code; real backends only via `REPOSIX_ALLOWED_ORIGINS`; sanctioned targets
ONLY — **TokenWorld = 2 PROTECTED (`7766017`+`7798785`, never deleted) + 1 SACRIFICIAL
EDITABLE (`2818063`)** (verify with `python3 scripts/confluence_tokenworld.py list` —
**note:** this fails in a bare shell with `KeyError: REPOSIX_CONFLUENCE_TENANT`; the env
vars are unset here by design and only sourced from `.env` by the runner — do NOT read
`.env` yourself, and do not treat this KeyError as a TokenWorld-state problem); `.env` is
present; **NO tag push ever** (manager's); never open work over a red main; ONE cargo
invocation machine-wide (prefer `-p`); /tmp leaf isolation (`cd` in the SAME bash
invocation); single-writer discipline (one tree-mutating agent at a time; read-only
agents may parallelize); relief ~100k soft / ~150k hard context (absolute, not %) →
REPLACE this file, commit+push, end turn, manager rotates you. A `fork` is never a safe
discard — end the turn instead. Resume a child via SendMessage, never fork.

## 4. Litmus / gate / REOPEN state

- **Milestone VERDICT.md is RED on disk** (`quality/reports/verdicts/milestone-v0.14.0/VERDICT.md`,
  dated 2026-07-12, `head_at_grade: 9890a67`) — this predates the item-5 fix, item-5's
  test-fix lane, and guardrail 5. It graded RED off a real 9th-probe run that predates
  `49666eb`/`c2eb2ad`/`c7ae07a`/`e504121`. It must be **re-minted**, not trusted, after a
  fresh clean 9th-probe run (STEP 3.2) — this is STEP 3.3 item 2.
- **9th-probe cadence (`pre-release-real-backend`) has NOT been re-run since the fix.**
  Successor #11's STEP 3.2 is the first live re-run against the current HEAD. Per the
  prior handover's documented interim op (part of the Ruling-#2-defer path): the cadence
  runner invokes `bash scripts/refresh-tokenworld-mirror.sh` FIRST — one clean run
  legitimately grades item-5 GREEN (litmus + p93 PASS specifically).
- **Known pre-existing NOT-VERIFIED rows (do not let these block the item-5 grade):**
  `agent-ux/t4-conflict-rebase-ancestry-real-backend` (P0) and
  `agent-ux/github-front-door-real-backend` (P1) — both `error: verifier not found`,
  never-shipped scripts (tracked as GTH-V15-03 precedent in the stale VERDICT.md). If the
  cadence exits nonzero, confirm it is these same two rows and not a regression before
  treating item-5 as blocked.
- **Litmus non-idempotency finding: RULED-DEFER→v0.15.0 (Ruling #2, closed).** Carry the
  caveat **verbatim** into the tag-readiness report: "litmus is non-idempotent against
  its own GitHub mirror fan-out (pre-write client tree, not post-write materialized
  snapshot) — a pre-existing ADR-010 RBF-LR-04 fan-out characteristic, not a v0.14.0
  regression. Product fix (mirror-sync pushing the post-write snapshot) routed to
  v0.15.0. The interim op — one clean `refresh-tokenworld-mirror.sh` run before the 9th
  probe — legitimately grades item-5 GREEN for this tag."
- **KNOWN GATE RACE (HIGH, filed, NOT fixed):** `ci-green-on-main.sh` grades PASS off the
  newest `gh run` WITHOUT asserting `headSha` == pushed HEAD. Cross-check `headSha`
  manually (done this session for both `24f9e97` and `d660e6e` — both matched).
- **Item-7 is RESOLVED (DEFER to v0.15.0) — carry this WAIVED flag verbatim into
  READY-TO-TAG:** "p93 is GREEN as an UPDATE-recovery proof against live TokenWorld
  (`1c424d7`). It does NOT cover CREATE-recovery: a partial-fail whose landed action was
  a create against an id-assigning backend genuinely does not converge. This is the
  owner-signed WAIVED known limitation of ADR-010 §3 / RBF-LR-03, hand-recoverable,
  routed to v0.15.0." (Separate from item-5's now-fixed UPDATE-recovery RED.)

## 5. Mid-execution decisions not yet formalized + noticed-not-yet-filed

Nothing new this session required an ad-hoc decision beyond executing the already-ruled
Ruling #2 and the already-scoped STEP 2/3.1 lanes — no new `CONSULT-DECISIONS.md` entry
was needed. The following are **carried forward, still unfiled/unactioned**, do not drop:

- **GOOD-TO-HAVE:** dead `PROTECTED_IDS` variable in `scripts/refresh-tokenworld-mirror.sh`
  (defined but the guard loop hardcodes `7766017 7798785` literally instead of reading
  it).
- **GOOD-TO-HAVE / v0.15.0:** `crates/reposix-cli/tests/agent_flow_real.rs` (~47k chars)
  and `translate.rs` (~26k) are split candidates (by backend) — flagged, not filed as a
  formal row yet.
- **Item-8 doc TODOs (from the prior handover, still open):** re-STATUS
  `surprises-intake/part-03.md:59-61` (stale "active p93 blocker" language, overtaken by
  `1c424d7`); file a v0.15.0 GOOD-TO-HAVE for slug→id create-reconciliation (distinct
  from GTH-09, which is already filed for the broader ADR-010 redesign — confirm overlap
  before filing a duplicate); LOW: title-sweep 2 pre-rewrite "p93 smoke A" orphan pages
  still sitting in TokenWorld.
- **Item-7 WAIVED flag** — see §4, must ride into the tag-readiness report verbatim.
- **RETROSPECTIVE.md's v0.14.0 section is STARTED but INCOMPLETE for this tag-blocker
  arc.** It currently narrates only the wave-2 hardening (P102–P113, shipped
  2026-07-12) — it has NO mention yet of: the item-5 string-encoded-ADF regression + its
  root cause + fix (`49666eb`), the DP-2 mechanism review + its hollow-real-twin finding,
  the item-5 test-fix lane (5a–5d), the litmus non-idempotency finding + Ruling #2, or
  item-7's CREATE-recovery WAIVED flag. STEP 3.3's retro-distillation sub-step must add
  this narrative (OP-9) BEFORE re-minting the verdict — the ratifier grades RED if it's
  missing.

## 6. Precise next steps (successor #11 runbook) — STOP at READY-TO-TAG, NEVER push the tag

0. Re-verify §1's ground truth live (`git rev-parse --short HEAD`, `git status
   --porcelain`, `gh run list --branch main --workflow ci.yml --limit 3 --json
   headSha,status,conclusion`). Confirm `24f9e97`'s run concluded `success` before
   proceeding. If it's still `in_progress`, wait/poll — do not open STEP 3.2/3.3 over an
   unresolved or red run.

1. **STEP 3.2 — 9th-probe unbiased VERIFIER (`gsd-verifier`, OP-7).** Dispatch a fresh
   subagent with no prior session context to run
   `python3 quality/runners/run.py --cadence pre-release-real-backend --persist`
   (this invokes `bash scripts/refresh-tokenworld-mirror.sh` first, per the documented
   interim op — one clean run legitimately grades item-5 GREEN). Grade **item-5
   specifically on litmus (`agent-ux/milestone-close-vision-litmus-real-backend`) + p93
   (`agent-ux/p93-partial-failure-recovery-real-confluence`) PASS.** Re-confirm the two
   pre-existing NOT-VERIFIED rows (`t4-conflict-rebase-ancestry-real-backend`,
   `github-front-door-real-backend`) are unchanged from the stale VERDICT.md's own record
   (§4) — do not let them fail the item-5 grade; do not silently absorb them as "fixed"
   either, they stay NOT-VERIFIED/filed debt.

2. **STEP 3.3.1 — OP-9 retro distillation FIRST.** Finalize the v0.14.0 section of
   `.planning/RETROSPECTIVE.md` per §5's gap list above: distill the 8 `part-03.md`
   intakes, the item-5 RED run + root cause + fix, the DP-2 mechanism review + hollow-twin
   finding, the item-5 test-fix lane (5a–5d), the litmus non-idempotency finding (now
   RULED-DEFER→v0.15.0), and item-7's CREATE-recovery WAIVED flag. The ratifier in step 4
   grades RED if this is missing or thin.

3. **STEP 3.3.2 — re-mint `quality/reports/verdicts/milestone-v0.14.0/VERDICT.md`
   GREEN**, sourced from the fresh STEP 3.2 probe artifacts (not the stale 2026-07-12
   run) — update `head_at_grade` to the current HEAD.

4. **STEP 3.3.3 — dispatch a FRESH unbiased ratification subagent** per
   `quality/dispatch/milestone-close-verdict.md` (author ≠ this session, zero prior
   context) to confirm the re-minted verdict is legitimate.

5. **STEP 3.3.4 — author `.planning/milestones/v0.14.0-phases/tag-v0.14.0.sh`**,
   patterned on `.planning/milestones/v0.13.0-phases/tag-v0.13.0.sh` (clean-tree +
   signed-tag guards).

6. **STOP at READY-TO-TAG.** Do not run the script, do not push a tag — that step is the
   manager's. Send a compact report to MANAGER w1:p7: commit SHAs for every step above,
   artifact paths (retro section, VERDICT.md, tag script), the 9th-probe exit code +
   per-row grade, and TokenWorld's end state (3 pages, unchanged counts, no deletions).

7. **File, do not drop, the carried noticed-not-filed items** in §5 (dead
   `PROTECTED_IDS` var, the two split-candidate test files, the `part-03.md` re-STATUS +
   v0.15.0 GTH filing, the orphan-page title sweep) to their respective intake files
   before or alongside STEP 3.3 — whichever is cheaper given the wave's shape.

### Ops lessons / hygiene debt (carry)

- **KNOWN GATE RACE (HIGH, filed, not fixed):** `ci-green-on-main.sh` grades PASS off the
  newest `gh run` WITHOUT asserting `headSha` == pushed HEAD — cross-check `headSha`
  manually every time.
- **docs-alignment grader misfire gotcha:** an Opus grader Task can return with 0 tool
  uses + a generic "please continue" passthrough (did nothing). Verify with
  `plan-refresh` + `git status`; re-dispatch FRESH with an explicit "EXECUTE NOW, use
  your tools" opening if it happens again.
- **Hygiene:** `.planning/CONSULT-DECISIONS.md` is **~40.3k chars** (WARN >20k, and has
  grown, not shrunk, since the last handover) — prune superseded entries (the
  DROP/HALT item-5 chain, RESOLVED B1 restore/reconcile, now-closed Ruling #2 discussion
  once acted on) at a clean moment; git is the archive.
  `surprises-intake/part-03.md` is **~27.3k chars** (over the 20k md soft-warn) — OP-8
  drain candidate.
- `run_helper_export_real` discards helper stderr (opaque real-backend failures) —
  already filed, not yet fixed.

---
History lives in git — `git log` / `git show`, not restated here.
