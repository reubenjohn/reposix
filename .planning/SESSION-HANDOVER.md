# SESSION-HANDOVER.md — v0.15.0 Floor: P117 W2 docs-alignment fully decided,
apply-script staged, push-attempt + Step C handed — 2026-07-16

**VERIFY LIVE BEFORE ACTING — do not trust any number below blindly, re-run §1
yourself first.**

Written by **workhorse #57** (L0 ROUTER), relieving to successor **#58** (fresh
router) at the ~150k hard-stop context boundary. This file **REPLACES** the prior
`#56→#57` handover (commit `26dd6d8`, now superseded). Manager: `w1:p7` (separate
owner, `.planning/MANAGER-HANDOVER.md` — do not touch). Milestone **v0.15.0 "Floor"**,
phase **P117** (docs-truth + launch-blocker purge). **Router ROUTES ONLY**
(ORCHESTRATION §"L0 is a ROUTER") — do not do leaf work yourself; all >100-line reads
go through a reader-digester. Relieve at ~100k soft / ~150k hard **own-context**.

**Read order:** this file → §1 ground truth (verify live, note the FLAGGED anomaly) →
§2 wave/cycle state → §3 binding constraints → §4 litmus/gate/REOPEN state → §5
mid-execution decisions + noticed-not-filed → §6 runbook (start at step 0).

## 1. Ground truth (git) — verify live before acting

```
git rev-parse HEAD && git rev-parse origin/main && git status --porcelain
git rev-list --left-right --count origin/main...HEAD
gh run list --branch main -L 4 --json workflowName,headSha,conclusion,status \
  --jq '.[]|"\(.workflowName) \(.headSha[0:7]) \(.status)/\(.conclusion)"'
```

**Live-verified by #57 immediately before writing this handover:**

- `HEAD` = `d4156f6` (`d4156f646e003d3c43a10ebd2b7789769120e1e6`).
- `origin/main` = `752895af` — **15 commits ahead, 0 behind** (`git rev-list
  --left-right --count origin/main...HEAD` → `0<TAB>15`).
- `gh run list` on `752895a` (origin's current tip): `Docs`, `CI`, `release-plz`,
  `CodeQL` all `completed/success` — origin CI is GREEN as of `752895a`, unchanged
  since #56's handover (no push has landed this rotation either).
- The 15 local/unpushed commits, oldest→newest: `f42455a` (seed PROGRESS W2),
  `b82be63` (SC1 Confluence-as-wiki fix), `214f45e` (troubleshooting.md reword),
  `098e2f5` (file GTH-V15-46), `0dc0e20` (SC5a benchmarks provenance, catalog-first
  row — the row that later corrupted, see below), `daa9755` (SC5b de-FUSE
  twitter.md), `2a70388` (file GTH-V15-47), `874bf11` (twitter.md truth cleanup),
  `c76b2c2` (filesystem-layer fix), `58adabb` (PROGRESS: W2 push BLOCKED), `02a72e3`
  (clear banned-word regression; escalate GTH-V15-49), `5c14b2f` (P117 W2 C1 relief),
  `f4c9a02` (fable consult verdict, GTH-V15-49 → Option B), `26dd6d8` (#56→#57 relief
  — 3 push-blockers documented), `d4156f6` (**this rotation's work**: fix catalog
  corruption, push-blocker #3 CLEARED — see §4).
- `target/release/reposix-quality` is **already built (release profile)**, mtime
  `2026-07-16 21:54` — reuse it, no rebuild needed.

**FLAGGED ANOMALY — investigate before running anything:** `git status --porcelain`
is **NOT clean** despite HEAD being the same commit `d4156f6` this handover names as
current:
```
 M quality/catalogs/doc-alignment.json
?? .planning/phases/117-doc-truth-launch-blocker-purge/apply-w2-refresh.sh
```
The `??` line is this rotation's Deliverable 1 (see §6) — expected, will be staged
into the handover commit. **The ` M quality/catalogs/doc-alignment.json` is NOT
mine and is NOT part of this commit** — I did not run any `reposix-quality`
mutation this rotation (verified: my own tool calls were `Read`/`Write`/`grep`/
`bash -n` only, no binary invocation). Facts gathered about it:
- File mtime `2026-07-16 22:16:32 -0700` = `2026-07-17T05:16:32Z`, which is the
  EXACT `last_walked` timestamp the diff writes into `summary.last_walked` — i.e.
  someone ran `doc-alignment walk` (or an equivalent full re-walk) at that instant.
  That's **before** my session's own activity started (my first transcript-read
  tool call was ~22:23 PDT) — almost certainly a leftover from seat #57's own
  pre-dispatch work (e.g. computing the current STALE_DOCS_DRIFT row set to hand
  the two graders), never reverted.
- The diff is **collateral, not targeted**: `claims_total` 400→401,
  `claims_bound` 278→245, `claims_missing_test` 0→1, `alignment_ratio` 0.837→0.736,
  **`claims_waived` 19→0** (all 19 previously-waived rows lost their waived status
  in this walk — this looks like a possible tool regression, not intentional), plus
  scattered `last_verdict` flips (`BOUND`→`STALE_TEST_DRIFT`,
  `STALE_TEST_DRIFT`→`STALE_DOCS_DRIFT`, etc.) on rows OUTSIDE the 19 rows this
  rotation's script touches.
- **Confirmed safe for the apply script:** all 19 target row `id`s (17 bind +
  twitter + benchmarks) are present exactly once each in the dirty file — `bind`/
  `waive`/`propose-retire` overwrite a row's state unconditionally, so this dirty
  walk snapshot does not block Step B below.
- **NOT resolved by me — do not guess, verify before Step B:** decide whether to
  (a) run `git checkout -- quality/catalogs/doc-alignment.json` to reset to the
  clean `d4156f6` state and let `apply-w2-refresh.sh` + a fresh `walk` regenerate
  everything from scratch (safer, recommended), or (b) keep the dirty snapshot and
  investigate the `claims_waived: 19→0` drop first (if real waivers were silently
  dropped, that's a separate tool bug worth filing before it bites the next push).
  **Recommendation: option (a)** — `git checkout` it before Step B, since `walk`
  output is fully regenerable and the drop-to-0 waived count is unexplained.

## 2. Wave/cycle state

| Wave | Plan | State | Commits |
|---|---|---|---|
| W1 (117-01) | SC3+SC4 | DONE + GREEN + BANKED (unchanged) | `52092ad`, `4af2ece`, `56a222b` |
| **W2 (117-02 ∥ 117-03)** | SC1/SC2 doc-truth + SC5 benchmarks/social | Code COMPLETE. Push-blocker **#3 (catalog corruption) CLEARED** this rotation (`d4156f6`). Push-blocker **#1 (STALE_DOCS_DRIFT) FULLY DECIDED** — 17 binds + propose-retire + 2 waives extracted from 3 completed grader transcripts, staged as `apply-w2-refresh.sh` (Deliverable 1, uncommitted-as-code but committed-as-script this rotation) — **NOT YET APPLIED**. Push-blocker **#2 (GTH-V15-49)** ratified Option B, **NOT YET IMPLEMENTED**. **Push has NOT been attempted this rotation.** | `f42455a`..`d4156f6`, 15 commits, LOCAL/UNPUSHED |
| W3 (117-04 ∥ 117-06) | IA/cold-reader polish ∥ fix-twice gate+dead-code sweep | NOT STARTED | — |
| W4 (117-05) | launch animation embed | NOT STARTED | — |
| W5 (117-07) | coordinator close (refresh + upload + push) | NOT STARTED (animation `gh release upload` is owner-gated, §5) | — |

**Named incidents to read before touching anything:**
- `.planning/phases/117-doc-truth-launch-blocker-purge/117-HANDOVER.md` (commit
  `5c14b2f`) — the original W2 C1's detailed relief handover; treat that C1 as DEAD,
  never resume it.
- Commit `d4156f6` message — full root-cause + fix detail for the catalog
  corruption (three stacked serde/validation errors on one hand-typed row).
- `.planning/CONSULT-DECISIONS.md` (search `GTH-V15-49`) — the ratified Option B
  text for the docs-repro pivot, Step C-i's spec.

## 3. Binding constraints (carry verbatim, unchanged)

- One tree-writer at a time; **ONE cargo invocation machine-wide** (prefer `-p`); no
  `--no-verify`; **targeted staging only** (never `-A`/`.`); no tag push by any
  coordinator; no git surgery (reset/rebase/amend/reorder) on SHARED/pushed `main` —
  the manager (`w1:p7`) is a concurrent writer, `git pull --rebase` if origin moved,
  never force.
- Leaf isolation: `reposix`/sim/git test setup in a `/tmp` clone, `cd` in the SAME
  Bash invocation as the mutating command — never the shared repo.
- **Every push Bash timeout ≥300s** (pre-push runs full clippy+kcov, ~55s
  standalone but budget generously). Push cadence: `git push origin main` BEFORE any
  verifier-subagent dispatch, then `python3 quality/runners/run.py --cadence
  post-push --persist` — the `code/ci-green-on-main` (P0) probe must pass. Never
  open the next wave over a red main.
- **Ledger topology:** milestone-scoped ledgers only —
  `.planning/milestones/v0.15.0-phases/{GOOD-TO-HAVES,SURPRISES-INTAKE}.md`,
  `GTH-V15-NN` id scheme. Do NOT use the stale root `.planning/GOOD-TO-HAVES.md` or
  create a root `SURPRISES-INTAKE.md`.
- **Catalog write discipline (load-bearing, reinforced this rotation in
  `quality/CLAUDE.md` by `d4156f6`):** subagents NEVER write
  `quality/catalogs/*.json` directly by hand — all mutation flows through
  `reposix-quality doc-alignment <verb>` binary calls. A hand-authored row is how
  push-blocker #3 happened. `apply-w2-refresh.sh` (this rotation's Deliverable 1)
  follows this rule strictly — every line is a `$BIN doc-alignment <verb>`
  invocation, no hand JSON edits.
- **`apply-w2-refresh.sh` is not idempotent** — `bind`/`waive`/`propose-retire`
  error on a row already in a terminal state matching the verb. Do not blindly
  re-run after a partial failure; re-run `walk` first, see which of the 20 commands
  still need to fire, hand-trim.
- **GAUGE NOTE:** relieve at ~100k soft / 150k hard ABSOLUTE own-context.

## 4. Litmus / gate / REOPEN state — 3 push-blockers tracked since #55/#56

**#1 STALE_DOCS_DRIFT (P0) — FULLY DECIDED, NOT YET APPLIED.** Three Opus graders
(two independent full-catalog graders "A" and "B" covering 14 drifted rows between
them, plus a targeted re-bind pass for 3 rows B could only `mark-missing-test` on
first pass) produced 17 `bind` commands + 1 `propose-retire` + confirmation of 1
`waive` (benchmarks) — all extracted verbatim into
`.planning/phases/117-doc-truth-launch-blocker-purge/apply-w2-refresh.sh`
(Deliverable 1, this rotation). **Row-id cross-check: all 17 target row-ids matched
the expected list exactly, in order, with ZERO discrepancy** (see script header
comment for the full list). A 2nd waive command (twitter, RETIRE_PROPOSED
follow-up) was supplied verbatim by the router (not present in any transcript) and
is also staged in the script. **Script has NOT been run.** Clears when Step B (§6)
runs it and `walk` confirms no `docs-alignment:`-prefixed blocking lines remain.

**#2 GTH-V15-49 (docs-repro gate false-block).** RESOLVED-BY-DECISION → **Option
B**, ratified `.planning/CONSULT-DECISIONS.md`, commit `f4c9a02`. **Implementation
still pending** — this is Step C-i (§6): change
`quality/gates/docs-repro/snippet-extract.py:171` from `len(blocks) >
PIVOT_THRESHOLD` to `len(uncovered) > PIVOT_THRESHOLD`; update
`quality/gates/docs-repro/README.md:25` + the `:173` error text; MINT/MODIFY the
`docs-reproducible.json` catalog row FIRST (GREEN-contract change), then verifier
grades. **This blocker will almost certainly still be live after Step B's push
attempt** — do not be surprised by a REJECT naming this gate.

**#3 CATALOG CORRUPTION — CLEARED this rotation (`d4156f6`).** Row
`benchmarks/README-md/session-provenance` carried three stacked hand-edit
corruptions (`next_action: "BIND"` not a real enum variant; missing required
`last_verdict`; `tests`/`test_body_hashes` parallel-array length mismatch) that
made the ENTIRE catalog unparseable by `reposix-quality`. Fixed via targeted field
corrections (not a re-mint) — verified `doc-alignment plan-refresh docs/index.md`
now parses cleanly. Fix-twice done: incident filed to `SURPRISES-INTAKE.md`
(dated entry, 2026-07-16 23:59 PDT block, no `GTH-V15-NN` numeric id assigned —
this file's convention mixes dated and numbered entries), and the mint-only rule
reinforced in `quality/CLAUDE.md` with the three concrete failure modes.

**CI:** origin main is green on `752895a` (§1) — cleared, not re-checked until the
next push lands.

## 5. Mid-execution decisions + noticed-not-filed

1. **GTH-V15-49 resolution remains a formalized decision** (`CONSULT-DECISIONS.md`,
   commit `f4c9a02`) — do not re-litigate.
2. **The FLAGGED ANOMALY in §1 (dirty `doc-alignment.json`) is noticed-but-
   UNRESOLVED.** #57 found it, characterized it, recommended `git checkout` before
   Step B, but did NOT resolve it (out of scope for this rotation's two
   deliverables). #58 must decide and act before running the apply script.
3. **NEW noticed-not-filed (from the re-bind grader's own report, carry forward to
   `GOOD-TO-HAVES.md`):** the row `filesystem-layer/extensions-partialclone-signals-
   promisor` binds primarily to `dark_factory_sim_happy_path`
   (`crates/reposix-cli/tests/agent_flow.rs`), which is `#[ignore]`d (spawns a sim
   child process; not in the default `cargo nextest run` lane). The claim is real
   and the test does assert it — but nothing in CI actually runs that assertion by
   default. Sketch: either add a non-ignored unit-level assertion of the init
   config keys (`extensions.partialClone`/`promisor`/`partialclonefilter`), or wire
   the `#[ignore]`d dark-factory scenarios into a dedicated (slower) CI job.
4. **NEW noticed-not-filed (Step A's own observation, carry forward):**
   `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md` is now ~70k chars
   (350% of the 20k structure-dimension ceiling; already WAIVED until 2026-08-08 as
   a pre-existing condition, this rotation's addition compounds it further). Worth
   a milestone-close-adjacent split (OP-9 distillation is the natural point) rather
   than letting it keep growing unaddressed.
5. **NEW noticed-not-filed (observed during ground-truth verification of the dirty
   catalog):** `doc-alignment walk` output includes several `coverage: row ... cites
   out-of-eligible file` informational lines — these are non-blocking (only
   `docs-alignment:`-prefixed lines block the pre-push gate) but worth a later audit
   of the eligibility config if they keep accumulating.
6. **Doctrine note for #58 and beyond:** the two full-catalog Opus graders (A, B)
   dispatched by seat #57 each returned exhaustive per-row rationale in their final
   text and pushed seat #57's own context from ~96k to ~144k across that single
   wave — nearly the entire soft-to-hard relief budget consumed by report size
   alone, not orchestration work. **When fanning out N Opus graders that each
   return full per-row rationale, route their reports through a reader-digester or
   explicitly cap report size in the dispatch prompt** — otherwise the coordinator's
   own context balloons faster than the work it's coordinating.
7. **Already-resolved, no longer noticed-not-filed:** push-blocker #3 (catalog
   corruption) — filed + fixed this rotation, see §4.
8. **HELD — W5 animation upload** (`gh release upload` of `/home/reuben/workspace/
   reposix-animation-pitch/Reposix Launch Animation.mp4`, for `117-07`). External
   mutation, OWNER-GATED. The dispatched Step C coordinator must **RAISE** this to
   the router — never self-authorize; router relays to the owner.
9. **Non-blocking — `GOOD-TO-HAVES.md`** also over its 20k ceiling (waived to
   2026-08-08) — same OP-9 distillation point as item 4, not this rotation's
   problem to fix.
10. Prior W2 C1 (dead, do not resume) + already-filed W1 intake (`GTH-V15-44`,
    `GTH-V15-45`, nextest-not-installed row) — unchanged from #56's handover, do
    not re-file.

## 6. Precise next steps (successor #58 runbook)

**Step 0 — verify + resolve the FLAGGED ANOMALY (§1) before anything else.**
1. Run the §1 verify block yourself; confirm `HEAD`/`origin/main`/CI facts still
   hold (or have moved — re-poll if a push landed since this was written).
2. Decide + act on the dirty `quality/catalogs/doc-alignment.json`: recommended —
   `git checkout -- quality/catalogs/doc-alignment.json` to reset to the clean
   `d4156f6` state (walk output is fully regenerable; the `claims_waived: 19→0`
   drop is unexplained and safer not to build on). If you instead choose to keep
   it, first investigate why 19 waivers vanished.

**Step B tail — finish the W2 unblock (in order):**
1. `bash .planning/phases/117-doc-truth-launch-blocker-purge/apply-w2-refresh.sh`
   — applies the 17 binds + twitter propose-retire + twitter waive + benchmarks
   waive (20 `$BIN` invocations total, serially). NOTE: the catalog write path is
   atomic-rename but NOT concurrency-safe — never parallelize; the script already
   runs everything serially, do not try to speed it up.
2. `./target/release/reposix-quality doc-alignment walk` — confirm the blocking
   rows are gone. `coverage:`-prefixed lines are informational (non-blocking); the
   blocking lines are `docs-alignment:`-prefixed (`STALE_DOCS_DRIFT` /
   `MISSING_TEST` / `TEST_MISALIGNED` / `STALE_TEST_GONE` / `RETIRE_PROPOSED`). If
   `walk` still exits 1, capture the remaining `docs-alignment:` lines — that is
   the real residual push-blocker #1 remainder, investigate before proceeding.
3. `git add quality/catalogs/doc-alignment.json && git commit` — message e.g.
   `refresh(doc-alignment): re-bind 17 P117 W2 drifted rows; retire+waive twitter,
   waive benchmarks`; end with `Co-Authored-By: Claude Opus 4.8 (1M context)
   <noreply@anthropic.com>`.
4. `git push origin main` — **Bash timeout ≥300s (pre-push runs full
   clippy+kcov). NEVER `--no-verify`.** EXPECTATION: this push clears blocker #1
   (STALE_DOCS_DRIFT) and blocker #3 is already cleared; **it will almost
   certainly still REJECT on blocker #2 = GTH-V15-49 docs-repro gate**
   (`quality/gates/docs-repro/snippet-extract.py:171`). If it REJECTS → capture
   the exact stderr; that is Step C-i's job, do NOT hand-fix it here. If it
   unexpectedly LANDS → run `python3 quality/runners/run.py --cadence post-push
   --persist` and confirm the `code/ci-green-on-main` P0 probe PASSes before
   proceeding to Step C.

**Step C — dispatch a fresh opus C1 `phase-coordinator`** (model: opus; do NOT
resume the dead W2 C1 — read `117-HANDOVER.md` first, §2). Charter, three
sub-steps in order:
   (i) **Implement GTH-V15-49 Option B** (ratified `.planning/CONSULT-DECISIONS.md`,
   commit `f4c9a02`): change `quality/gates/docs-repro/snippet-extract.py:171` from
   `len(blocks) > PIVOT_THRESHOLD` to `len(uncovered) > PIVOT_THRESHOLD`; update
   `README.md:25` + the `:173` error text; MINT a `docs-reproducible.json`
   GREEN-contract catalog row FIRST (catalog-first rule), then dispatch the
   verifier. Clears blocker #2.
   (ii) **Re-push all W2 commits** (now ~17, or however many after Step B tail) —
   push BEFORE the verifier, then `run.py --cadence post-push --persist` →
   `code/ci-green-on-main` P0. Grade W2 RED if the push doesn't land or main's
   newest CI isn't green.
   (iii) **Execute W3 → W4 → W5 per the DAG:** W3 = `117-04 ∥ 117-06` (117-06 is a
   root+scoped `CLAUDE.md`-**only** sweep, NOT `docs/**` — serialize the single
   tree-writer so it can't collide with 117-04); W4 = `117-05` (launch animation
   embed); W5 = `117-07` (coordinator close). Reuse the full W2 C1 ownership
   charter (OD-3) and §3 constraints verbatim.
   **HELD, owner-gated:** the `gh release upload` of the launch animation mp4 —
   the C1 RAISES this to the router, never self-authorizes; router relays to the
   owner (§5 item 8).

**Close-out (every relief):**
5. Refresh `PROGRESS.md`'s `## NOW` at every wave boundary/push.
6. Route §5's new noticed-not-filed items (3, 4, 5, 6) to
   `GOOD-TO-HAVES.md`/`SURPRISES-INTAKE.md` as they fit (OP-8) — they were
   deliberately left un-filed this rotation to keep the two committed deliverables
   scoped; do not let them silently drop.
7. **REPLACE this handover** (not append) at your own relief, re-verifying every
   claim live before carrying it forward.
