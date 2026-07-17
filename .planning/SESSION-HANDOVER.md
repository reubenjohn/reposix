# SESSION-HANDOVER.md — v0.15.0 Floor: P117 W1 intake corrected+banked, handing W2–W5 C1 dispatch — 2026-07-16

**VERIFY LIVE BEFORE ACTING — do not trust any number below blindly, re-run the §1
verify block yourself first.**

Written by **workhorse #55** (L0 orchestrator), relieving to successor **#56**. This
file **REPLACES** (does not append to) the prior `SESSION-HANDOVER.md` (#54→#55's
handover, commit `29eeec3`, now superseded — that file's P117-W1-just-banked ground
truth is carried forward below; its "intake not yet filed" framing is stale, #55 filed
it this rotation). #55 ran the §1 verify block, digested the remaining P117 plans, then
spent its context budget catching and correcting a botched intake filing rather than
dispatching the P117 C1 — see §5 for the full incident and the lesson for #56.

**Read order:** this file → §1 ground truth (verify live FIRST) → §2 wave/cycle state →
§3 binding constraints (carry verbatim) → §4 litmus/gate/REOPEN state → §5 mid-execution
decisions + noticed-not-filed (read the intake-filing incident before touching any
ledger) → §6 runbook (dispatching a fresh P117 C1 for W2–W5 is the primary work,
pre-drafted charter included).

## 1. Ground truth (git) — verify live before acting

```
git rev-parse HEAD && git rev-parse origin/main && \
  git status --porcelain --untracked-files=all && \
  gh run list --branch main -L 4 --json databaseId,headSha,conclusion,workflowName,status \
    --jq '.[] | "\(.workflowName) \(.headSha[0:7]) status=\(.status) conclusion=\(.conclusion)"' && \
  grep -c '"last_verdict": "RETIRE_PROPOSED"' quality/catalogs/doc-alignment.json
```

**Live-verified by #55 immediately before writing this handover** (raw outputs, re-run
yourself, do not trust blindly):

- `git rev-parse HEAD` → `e8f0fa22173864a3282cfebd8def0867cce884b8`
- `git rev-parse origin/main` → same, `e8f0fa2...` — **HEAD == origin/main**, no drift.
- `git status --porcelain --untracked-files=all` → **empty output, tree clean.**
- `gh run list --branch main -L 4 ...` (widened to -L 12 for full history) →
  **prior tip `29eeec3` is FULLY GREEN**: `CI 29eeec3 success`, `CodeQL 29eeec3
  success`, `Docs 29eeec3 success`, `release-plz 29eeec3 success`. **Current tip
  `e8f0fa2`**: `CodeQL e8f0fa2 success`; **`CI e8f0fa2` and `release-plz e8f0fa2` were
  still `status=in_progress` (conclusion empty) at verify time** — #56 MUST re-poll
  these to green before dispatching the C1 (do not open W2 over an unresolved CI run).
  `Docs` did not fire on `e8f0fa2` (commit touched only `.planning/milestones/**`, no
  `docs/**` diff — expected, not a gap).
- `grep -c '"last_verdict": "RETIRE_PROPOSED"' quality/catalogs/doc-alignment.json` →
  **`0`.**

**Expected after THIS handover's commit lands:** `HEAD == origin/main == <this
commit>`, tree CLEAN, `RETIRE_PROPOSED` count still `0`, `CI`/`release-plz` concluded
`success` on `e8f0fa2` (or the new tip if a later push landed first).

**Local pre-push gate on `e8f0fa2`** (run by #55 as part of the push that landed it):
**FULLY GREEN — 61 PASS / 0 FAIL / 1 WAIVED / 0 NOT-VERIFIED, exit 0; secret-scan
clean.** Pre-push wall-clock ≈100s (WARN, over the ~60s budget) — this is the KNOWN
kcov-corpus-creep item, ALREADY filed in `SURPRISES-INTAKE.md` (2026-07-15 06:35 +
17:18 entries) — **do not re-file, do not treat as a regression.**

- #55's own commits this rotation, oldest→newest: none of substance beyond the single
  intake-correction commit `e8f0fa2` (a prior local-only commit `c7548e0` was reset
  before ever being pushed — see §5 item 1 for why).

## 2. Wave/cycle state

| Phase / item | State | Commits / evidence |
|---|---|---|
| P114 | CLOSED (t4 Confluence oid-drift fix-first) | `dc26302` et al. |
| P115 | CLOSED GREEN — human confirm-retire gate CLOSED (11 rows retired, `RETIRE_PROPOSED`→0) | `ce4d3b7`, `4bb0596` |
| P116 | CLOSED GREEN — gsd-verifier 12/12 must-haves PASS, 0 gaps, 0 blockers | `116-VERIFICATION.md`; `a1cc2d4`/`7412833`, `1ea51b3`, `5ee5e25`, `6825d13` |
| P117 planning | COMPLETE — opus `phase-coordinator` C1 dispatched, planned GREEN (all GSD gates on) | research `4349946`, plan `ce50609`, plan-checker MEDIUM patch `44d3476`; artifacts in `.planning/phases/117-doc-truth-launch-blocker-purge/` |
| P117 SC4 | **RATIFIED = Option B** (decide-and-disclose at L0): reword `attach.rs`'s dangling `detach` error ref; do NOT build a `reposix detach` subcommand. Option A filed to GOOD-TO-HAVES as `GTH-V15-43`. Do not re-litigate. | `4af2ece` |
| **P117 Wave 1 (117-01)** | **COMPLETE + GREEN + BANKED.** SC3 (connection-refused teach-the-fix) + SC4 (`attach.rs` reword). | `52092ad` + `4af2ece` + `56a222b` (test-name-honesty marker, verified genuine); CI run `29550609095` success on `a00dd8f` |
| **P117 W1 intake** | **DONE this rotation** — GTH-V15-44 (attach.rs/list.rs split candidate), GTH-V15-45 (non-sim backend error-teaching gap), + MEDIUM SURPRISES row (cargo nextest not installed). Filed into the CORRECT canonical milestone-scoped ledgers after a botched first attempt was caught and reset (see §5 item 1). | `e8f0fa2` |
| P117 Waves 2–5 (117-02..117-07) | **NOT STARTED.** No fresh C1 dispatched this rotation — #55 ran out of context budget at the W1→W2 boundary doing the intake correction instead. **Primary job for #56.** | — |

**3/15 v0.15.0 "Floor" phases complete** (P114, P115, P116); P117 W1 (of 7 plan files,
117-01..117-07) is done; 11 phases remain after P117 (P118–P128).

**P117 remaining wave DAG (verified against the plan files — NOT the plans' internal
wave labels, which predate #54 spending "Wave 1" on 117-01):**

- W2: **117-02** (SC1+SC2 doc-truth — `docs/index.md`, `docs/how-it-works/
  filesystem-layer.md`, `docs/reference/{glossary,cli,git-remote,confluence}.md`) ∥
  **117-03** (SC5 benchmarks provenance + social — `benchmarks/README.md`,
  `docs/social/twitter.md`, `quality/catalogs/doc-alignment.json`) — parallel, no deps.
- W3: **117-04** (furnished-product IA/cold-reader polish — `docs/index.md` +
  `how-it-works/{filesystem-layer,git-layer,time-travel,trust-model}.md`; dep 117-02) ∥
  **117-06** (fix-twice gate build + dead-code sweep — `quality/gates/structure/
  social-freshness.sh`, `quality/catalogs/freshness-invariants.json`,
  `quality/gates/perf/bench_token_economy_io.py`; **its CLAUDE.md sweep scope is
  root+scoped `CLAUDE.md` ONLY, NOT `docs/**`**; dep 117-03) — parallel.
- W4: **117-05** (launch animation embed — poster webp + click-to-play mp4,
  `docs/index.md` + `mkdocs.yml` + `quality/catalogs/docs-build.json`; dep 117-04).
- W5: **117-07** (coordinator close — doc-alignment refresh + mp4 upload + cold-reader
  sign-off + push; dep all).

Plan artifacts: `.planning/phases/117-doc-truth-launch-blocker-purge/
{117-02..117-07}-PLAN.md`, `PATTERNS.md`, `RESEARCH.md`.

## 3. Binding constraints (carry verbatim)

- One tree-writer at a time; ONE cargo invocation machine-wide (prefer `-p`); no
  `--no-verify`; targeted staging (never `-A`/`.`); do NOT touch
  `.planning/MANAGER-HANDOVER.md` (manager's file, separate owner); no tag push by any
  coordinator; no git surgery (reset/rebase/amend/reorder) on SHARED/pushed `main` — the
  manager (w1:p7) is a concurrent writer, `git pull --rebase` if origin moved, never
  force. **CLARIFICATION (established this rotation):** rewinding your OWN unpushed
  local commit before it is ever pushed to origin is NOT the shared-history surgery this
  forbids — #55 did exactly that (`git reset` on local-only `c7548e0`, origin never saw
  it) to fix a botched intake filing; this is safe and was the correct move.
- Leaf isolation in `/tmp` same-Bash-invocation; **every push Bash timeout ≥300s**;
  refresh `PROGRESS.md`'s `## NOW` at every boundary push; never open the next
  phase/wave over a red main.
- **GAUGE NOTE (load-bearing, carried from #54).** The relief line is ~100k soft /
  **150k hard ABSOLUTE own-context**. The Claude Code token-usage HOOK UNDERCOUNTS —
  **trust the actual gauge %, not the hook's token number**; relieve on the gauge.
- **Human gate is DONE** (P115's 11 rows retired; P115 + P116 both closed) — do NOT
  re-check or re-open it.
- **`.git/config` re-corruption hazard (carried from #53, NOT re-encountered by #54 or
  #55).** A sibling worktree lane can re-corrupt the shared `.git/config`
  (`core.bare = true` + fixture-identity `[user] email = t@t` injected). If ANY
  work-tree git op suddenly fails ("this operation must be run in a work tree", or a
  `.githooks/pre-commit` fixture-identity reject): FIRST run `cat .git/config`, check
  for those two symptoms, repair via a direct edit (`core.bare = false`, remove the
  injected `[user]` block).
- **Ledger topology (established this rotation — read before filing anything):** the
  CANONICAL v0.15 intake ledgers are the MILESTONE-SCOPED
  `.planning/milestones/v0.15.0-phases/{GOOD-TO-HAVES,SURPRISES-INTAKE}.md`
  (`GTH-V15-NN` id scheme). The root `.planning/GOOD-TO-HAVES.md` (255 lines,
  `GOOD-TO-HAVES-NN` id scheme, last real content from v0.14) is a SEPARATE STALE
  ledger — **do NOT file new v0.15 intake there**, and there is no root
  `SURPRISES-INTAKE.md` at all (do not create one). Both milestone-scoped ledgers are
  now large (`GOOD-TO-HAVES.md` 350 lines, `SURPRISES-INTAKE.md` 582 lines, ~65k chars
  combined) — over the 20k soft ceiling, WARN-only under the `GTH-V15-21` waiver
  (expires 2026-08-08); the drain-phase split is tracked in the SURPRISES 2026-07-14
  21:00 entry, scope-broadened 2026-07-16.

## 4. Litmus / gate / REOPEN state

- **P117 Wave 1 (117-01): GREEN + banked.** SC3 + SC4 delivered, test-name-honesty
  marker independently verified genuine.
- **SC4 decision: RATIFIED, not provisional.** Option B (reword) shipped; Option A
  (real `detach` subcommand) deliberately deferred, tracked as `GTH-V15-43` — do not
  re-litigate in W2+ unless the owner reopens it.
- **doc-alignment invariants: HOLDING.** `RETIRE_PROPOSED` = 0 (live-verified above).
  P117 W2+ will edit `docs/**` and is EXPECTED to trip `STALE_DOCS_DRIFT` pre-push
  BLOCK — normal, not a regression signal (see §6 for the recovery move).
- **CI on current tip `e8f0fa2`: PARTIALLY VERIFIED at handover time** — `CodeQL`
  green, `CI`/`release-plz` still `in_progress`. #56's FIRST ACT must re-poll these to
  a concluded `success` before dispatching the C1 (see §1). Prior tip `29eeec3` is
  fully green across all 4 workflows — confirmed.
- **Waiver clock:** `GTH-V15-21` file-size `--warn-only` expires **2026-08-08**
  (`attach.rs`/`list.rs` + both intake ledgers ride on it).
- **Intake state: DONE this rotation.** GTH-V15-44 (split candidate), GTH-V15-45
  (non-sim error-teaching gap), + MEDIUM SURPRISES row (nextest not installed) — filed
  correctly in `e8f0fa2`. Already-filed, do NOT re-file: SC4 Option A → `GTH-V15-43`;
  leaf-isolation grep false-positive → `117-06-PLAN.md` Task 2(c).

## 5. Mid-execution decisions + noticed-not-filed

1. **INCIDENT: a prior haiku executor botched the W1 intake filing; #55 caught and
   corrected it.** Commit `c7548e0` (local, never pushed to origin) filed the three
   W1-noticed intake candidates from #54's handover into the WRONG ledgers: it appended
   to the STALE root `.planning/GOOD-TO-HAVES.md` (wrong id scheme, wrong milestone
   scope) and CREATED a new, fragmenting root `.planning/SURPRISES-INTAKE.md` (no such
   file should exist — see §3 ledger topology). #55 reset that commit (safe, per the §3
   clarification — origin never saw it) and re-filed correctly into the milestone-scoped
   ledgers next to the existing `GTH-V15-43` precedent. Landed as `e8f0fa2`, pushed
   green. **Runbook item "triage-and-file W1 intake" from the #54 handover = DONE.**
2. **LESSON for #56 (act on this, don't just note it): delegate intake corrections to a
   subagent.** #55 read the full 227-line `SURPRISES-INTAKE.md` directly into L0
   context to diagnose the botched filing — that single read cost ~40k tokens and
   consumed the budget that should have gone to dispatching the P117 C1. Next time a
   ledger needs forensic inspection or correction, dispatch a `reader-digester` or a
   scoped haiku/sonnet subagent to do the read-and-diagnose, and only pull the summary
   into L0.
3. **#55 did NOT dispatch the P117 C1 this rotation.** The prior C1 (dispatched by #54)
   is dead (pane rotated). #56's primary job is the fresh dispatch — see §6 and the
   pre-drafted charter.
4. **HIGH, open, NOT yet folded into any wave — `docs/guides/troubleshooting.md:329`
   still names the phantom `reposix detach`** (twin of the `attach.rs` reference W1
   already purged). Live-confirmed by #55 via direct read, unchanged text: *"Re-running
   `reposix attach` against the same SoT is **idempotent** ... Re-running against a
   **different** SoT is **rejected** with `working tree already attached to
   <existing-sot>; multi-SoT not supported in v0.13.0`. To switch SoT, run `reposix
   detach` first (or remove the `extensions.partialClone` config + cache directory by
   hand)."* FIX: reword to DROP the phantom `reposix detach` (SC4 = Option B ratified,
   no such subcommand; Option A deferred as `GTH-V15-43`) and keep the real manual
   recovery. Suggested after-text: *"To switch SoT, remove the `extensions.partialClone`
   config + cache directory by hand, then re-attach — there is no `reposix detach`
   subcommand in this cut (a real one is tracked as `GTH-V15-43`)."* Mirror the
   `attach.rs` W1 reword. `117-06`'s fix-twice sweep is scoped to root+scoped
   `CLAUDE.md` only (confirmed by reading `117-06-PLAN.md`'s Task 2 action text) — it
   does NOT cover `docs/**`, so this phantom-command lie survives every currently-planned
   wave unless the C1's W2 absorbs it explicitly. **If this is not folded in, P117
   ships the exact phantom-command lie the phase exists to purge — treat as a
   phase-close blocker.**
5. **HELD external mutation (owner gate, do not auto-execute).** `117-07` Task 1's
   animation lane (`GTH-V15-37`) GitHub-Release asset upload (`gh release upload`) is an
   external mutation requiring owner-named-target approval per `ORCHESTRATION.md` §9.
   The C1 CHECKPOINTS + RAISES when it reaches that step; #56 RAISES to owner/manager —
   do NOT self-authorize. Rest of animation productionization (embed/poster/mkdocs
   wiring in `117-05`) proceeds in-plan without a gate. mp4 source:
   `/home/reuben/workspace/reposix-animation-pitch/Reposix Launch Animation.mp4`
   (~7MB, release-asset-only).
6. **Already-carried, still live, unchanged this rotation:** SC4 = Option B ratified
   (§2/§4, do not re-raise); the `.git/config` re-corruption hazard (§3, not
   re-encountered); sibling-lane / manager-tier awareness (the manager, w1:p7, is a
   concurrent writer on `.planning/MANAGER-HANDOVER.md` — no conflict this rotation, no
   action needed).

## 6. Precise next steps (successor #56 runbook)

1. **Standard first-act verify block (§1).** Run it yourself; confirm `HEAD ==
   origin/main == e8f0fa2` (or this handover's own commit if it has landed by the time
   you read this), tree clean, `RETIRE_PROPOSED` = 0. **Additionally poll `gh run list`
   until `CI` and `release-plz` on the tip both show `status=completed
   conclusion=success`** — they were still `in_progress` when #55 verified; do not
   dispatch the C1 over an unresolved or red run.
2. **Dispatch a FRESH opus `phase-coordinator` C1** for P117 Waves 2–5 from the
   committed plan state (charter template: `coordinator-dispatch` skill). The prior C1
   is dead (pane rotated) — do not try to resume it. Use the pre-drafted charter below,
   paste-ready:

   ```
   You are the opus phase-coordinator C1 owning P117 Waves 2–5 (docs-truth + launch-blocker
   purge) for reposix, end-to-end BY DELEGATING (dispatch gsd-executor/gsd-code-reviewer/
   gsd-verifier/reader-digester leaves — never do leaf work yourself). Committed plan state:
   `.planning/phases/117-doc-truth-launch-blocker-purge/{117-02..117-07}-PLAN.md` (+
   PATTERNS.md, RESEARCH.md). 117-01 is DONE+GREEN — do not redo. Execute by this dependency
   DAG: W2 = 117-02 ∥ 117-03; W3 = 117-04 ∥ 117-06; W4 = 117-05; W5 = 117-07 (close). FOLD
   INTO W2 (117-02 lane): reword `docs/guides/troubleshooting.md:329` to drop the phantom
   `reposix detach` and keep the manual recovery (SC4=Option B, no such subcommand;
   GTH-V15-43 defers it) — mirror the attach.rs W1 reword; this is a phase-close blocker if
   missed. CHECKPOINT + RAISE to L0 (do NOT attempt yourself): (a) any STALE_DOCS_DRIFT
   pre-push BLOCK — report the drifted-doc list, L0 runs /reposix-quality-refresh at top
   level; (b) the 117-07 animation mp4 `gh release upload` — external mutation, owner-gated.
   117-06's CLAUDE.md sweep is root+scoped CLAUDE.md ONLY, not docs/**. Constraints: one
   tree-writer at a time; ONE cargo machine-wide (-p); no --no-verify; targeted staging; no
   tag push; leaf isolation in /tmp same-Bash-invocation; push Bash timeout ≥300s; refresh
   PROGRESS.md ## NOW at boundaries; never open a wave over red main. Push cadence: git push
   BEFORE any verifier dispatch, then post-push cadence (code/ci-green-on-main P0). Relieve
   past ~100k own-context (trust the gauge %, not the hook). OWNERSHIP CHARTER: (1)
   acceptance criteria are the floor not the ceiling; (2) noticing is a deliverable — report
   lying doc claims, hollow tests, teaching-free errors, dead code; (3) eager-fix (<1h, no
   new dep) or file to SURPRISES-INTAKE/GOOD-TO-HAVES with severity+sketch, never silently
   skip; (4) verify against reality — render the docs/hit the gate; (5) north star:
   Rust-compiler-grade UX, would a skeptical first-time dev come away impressed? Report
   ≤400 words: verdict, commit SHAs, RAISE LIST for L0, intake disposition, NOTICED section
   — evidence to committed artifacts, never chat.
   ```

3. **Be ready to run `/reposix-quality-refresh <doc>` at TOP LEVEL** when the C1
   RAISES a `STALE_DOCS_DRIFT` list — depth-2 fan-out is unreachable from inside the
   C1's own subagent tree (`.planning/CLAUDE.md` § Subjective-rubric dispatch). Budget
   context for this — it is context-heavy, which is why #54 AND #55 both relieved at
   this boundary; consider relieving again after a refresh cycle or two rather than
   pushing through in one sitting. Decide batch-vs-per-wave at the first drift block
   (per-wave keeps push cadence clean given the concurrent manager-writer; the plan's
   `117-07` "consolidated refresh" assumes batching).
4. **Push cadence unchanged:** `git push origin main` BEFORE any verifier-subagent
   dispatch, then `python3 quality/runners/run.py --cadence post-push --persist`
   (`code/ci-green-on-main` is P0). Never open the next wave over a red main. Every
   push Bash timeout ≥300s.
5. **Refresh `PROGRESS.md`'s `## NOW`** at every boundary push.
6. **REPLACE this handover** (not append) at your own relief, following this same
   `.planning/ORCHESTRATION.md` §3 template, re-verifying every claim live before
   carrying it forward.
