# SESSION-HANDOVER.md — v0.15.0 Floor: P117 W2 fully triaged, 3 push-blockers
documented + fix path — 2026-07-16

**VERIFY LIVE BEFORE ACTING — do not trust any number below blindly, re-run §1
yourself first.**

Written by **workhorse #56** (L0 ROUTER), relieving to successor **#57** (fresh
router). This file **REPLACES** the prior `#55→#56` handover (commit `752895a`, now
superseded). Manager: `w1:p7` (separate owner, `.planning/MANAGER-HANDOVER.md` —
do not touch). Milestone **v0.15.0 "Floor"**, phase **P117** (docs-truth +
launch-blocker purge). **Router ROUTES ONLY** (ORCHESTRATION §"L0 is a ROUTER") — do
not do leaf work yourself; all >100-line reads go through a reader-digester. Relieve
at ~100k soft / ~150k hard **own-context** — trust the status-line gauge %, not the
hook's token count.

**Read order:** this file → §1 ground truth (verify live) → §2 wave/cycle state →
§3 binding constraints → §4 litmus/gate/REOPEN state (the 3 push-blockers live here)
→ §5 mid-execution decisions + noticed-not-filed → §6 runbook.

## 1. Ground truth (git) — verify live before acting

```
git rev-parse HEAD && git rev-parse origin/main && git status --porcelain
git rev-list --left-right --count origin/main...HEAD
gh run list --branch main -L 4 --json workflowName,headSha,conclusion,status \
  --jq '.[]|"\(.workflowName) \(.headSha[0:7]) \(.status)/\(.conclusion)"'
```

**Live-verified by #56 immediately before writing this handover:**

- `HEAD` = `f4c9a020` — local main tip.
- `origin/main` = `752895af` — **13 commits ahead, 0 behind** (`git rev-list
  --left-right --count origin/main...HEAD` → `0<TAB>13`).
- `git status --porcelain` → **empty, tree clean.**
- `gh run list` on `752895a` (origin's current tip): `Docs`, `CI`, `release-plz`,
  `CodeQL` all `completed/success` — **origin CI is GREEN, but only as of `752895a`**;
  this gate is CLEARED and is NOT re-checked until the next push lands a new tip.
- The 13 local/unpushed commits, oldest→newest (`f42455a`..`f4c9a02`): `f42455a`
  (seed PROGRESS W2), `b82be63` (SC1 Confluence-as-wiki + cat-networks fix), `214f45e`
  (troubleshooting.md phantom `detach` reword), `098e2f5` (file GTH-V15-46), `0dc0e20`
  (SC5a benchmarks provenance, catalog-first row — **this is the row that later turned
  out corrupted, see §4 blocker #3**), `daa9755` (SC5b de-FUSE twitter.md), `2a70388`
  (file GTH-V15-47 dead code), `874bf11` (twitter.md truth cleanup), `c76b2c2`
  (filesystem-layer fix, closes GTH-V15-46), `58adabb` (PROGRESS: W2 push BLOCKED,
  wave RED), `02a72e3` (clear banned-word regression; escalate GTH-V15-49), `5c14b2f`
  (P117 W2 C1 relief handover), `f4c9a02` (fable consult verdict, GTH-V15-49 → Option
  B).
- `target/release/reposix-quality` is **already built (release profile)** this
  session — reuse it, no rebuild needed for the fixes below.

## 2. Wave/cycle state

| Wave | Plan | State | Commits |
|---|---|---|---|
| W1 (117-01) | SC3+SC4 | DONE + GREEN + BANKED (prior rotations, unchanged) | `52092ad`, `4af2ece`, `56a222b` |
| **W2 (117-02 ∥ 117-03)** | SC1/SC2 doc-truth + SC5 benchmarks/social | **FULLY IMPLEMENTED, code-review-clean** (per dead W2 C1's own relief report) — **PUSH BLOCKED** on 3 items, §4 | `f42455a`..`f4c9a02`, 13 commits, LOCAL/UNPUSHED |
| W3 (117-04 ∥ 117-06) | IA/cold-reader polish ∥ fix-twice gate+dead-code sweep | NOT STARTED | — |
| W4 (117-05) | launch animation embed | NOT STARTED | — |
| W5 (117-07) | coordinator close (refresh + upload + push) | NOT STARTED (animation `gh release upload` is owner-gated, §5) | — |

**Named incident to read before touching anything:** the W2 C1's own detailed relief
handover — `.planning/phases/117-doc-truth-launch-blocker-purge/117-HANDOVER.md`
(commit `5c14b2f`) — has the full W2 re-push sequence and finer-grained detail than
this file carries. Read it before dispatching the fresh C1 in §6 Step C.

## 3. Binding constraints (carry verbatim, unchanged)

- One tree-writer at a time; **ONE cargo invocation machine-wide** (prefer `-p`); no
  `--no-verify`; **targeted staging only** (never `-A`/`.`); no tag push by any
  coordinator; no git surgery (reset/rebase/amend/reorder) on SHARED/pushed `main` —
  the manager (`w1:p7`) is a concurrent writer, `git pull --rebase` if origin moved,
  never force.
- Leaf isolation: `reposix`/sim/git test setup in a `/tmp` clone, `cd` in the SAME
  Bash invocation as the mutating command — never the shared repo.
- **Every push Bash timeout ≥300s.** Push cadence: `git push origin main` BEFORE any
  verifier-subagent dispatch, then `python3 quality/runners/run.py --cadence
  post-push --persist` — the `code/ci-green-on-main` (P0) probe must pass. Never open
  the next wave over a red main.
- **Ledger topology:** milestone-scoped ledgers only —
  `.planning/milestones/v0.15.0-phases/{GOOD-TO-HAVES,SURPRISES-INTAKE}.md`,
  `GTH-V15-NN` id scheme. Do NOT use the stale root `.planning/GOOD-TO-HAVES.md` or
  create a root `SURPRISES-INTAKE.md`.
- **Catalog write discipline (load-bearing for §4 blocker #3):** subagents NEVER
  write `quality/catalogs/*.json` directly by hand — all mutation flows through
  `reposix-quality doc-alignment` binary calls. A hand-authored row is how blocker #3
  happened; any fix to it must go back through the tool, not another hand-edit.
- **GAUGE NOTE:** relief line is ~100k soft / 150k hard ABSOLUTE own-context; the
  token-usage hook undercounts — trust the gauge %.

## 4. Litmus / gate / REOPEN state — 3 push-blockers, all must clear before W2 pushes

**#1 STALE_DOCS_DRIFT (P0).** Design-deferred SC1/SC2 BOUND rows drifted on:
`docs/index.md` (×2 rows), `docs/reference/cli.md`, `docs/reference/git-remote.md`,
`docs/how-it-works/filesystem-layer.md`. **Content is FINAL — do NOT edit the docs.**
Fix = `/reposix-quality-refresh` per doc (**TOP-LEVEL ONLY** — the ROUTER runs this,
a C1/executor cannot, no `Task` at depth-1). Playbook:
`.claude/skills/reposix-quality-doc-alignment/refresh.md` (plan-refresh → opus grader
per stale row → walk → commit). **BLOCKED until #3 is fixed** (catalog must parse
first).

**#2 GTH-V15-49 (docs-repro gate false-block).** RESOLVED by E2 fable consult →
**Option B**, committed `f4c9a02`, ledger `.planning/CONSULT-DECISIONS.md`.
Implementation still pending (downstream C1, CATALOG-FIRST + verifier): change
`quality/gates/docs-repro/snippet-extract.py:171` from `len(blocks) >
PIVOT_THRESHOLD` to `len(uncovered) > PIVOT_THRESHOLD`; update
`quality/gates/docs-repro/README.md:25` pivot rule + the `:173` assertion error text;
MINT/MODIFY the `docs-reproducible.json` catalog row FIRST (GREEN-contract change),
then verifier grades. Flip-back conditions in the ledger: allow-list ≥30 entries,
chronic uncovered churn, or removal of the `:177` uncovered check.

**#3 CATALOG CORRUPTION** (discovered by #56 — NOTICED W2 defect, not yet filed as a
GTH-V15-NN, see §5). `quality/catalogs/doc-alignment.json:8870`, row
`benchmarks/README-md/session-provenance` (minted by `0dc0e20`, 117-03 SC5a) has
invalid `"next_action": "BIND"`. `NextAction` enum
(`crates/reposix-quality/src/catalog.rs:401-420`) accepts ONLY {WRITE_TEST,
FIX_IMPL_THEN_BIND, UPDATE_DOC, RETIRE_FEATURE, BIND_GREEN} — bare `BIND` is not a
member. Only 1 occurrence in the file (332 rows correctly use `BIND_GREEN`). Effect:
**entire catalog unparseable by the binary** → blocks #1's refresh AND likely the
pre-push docs-alignment gate. Root cause: the row was hand-written into the JSON
(violates the §3 catalog-write rule). Fix: change to the valid variant matching
author intent — rationale reads "defer hash+test binding to W6; confirm the test
FALSIFIES the provenance claim before flipping to BIND_GREEN" — strongest candidate
is `WRITE_TEST`; the executor must verify the chosen variant makes
`target/release/reposix-quality doc-alignment plan-refresh docs/index.md` parse AND
doesn't wrongly block/green the row (check `catalog.rs` walk semantics). **Fix
twice:** reinforce the hand-edit ban in `quality/CLAUDE.md` / the doc-alignment skill
note, and file this incident to
`.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md` (next `GTH-V15-NN`).

**CI:** origin main is green on `752895a` (§1) — cleared, not re-checked until the
next push.

## 5. Mid-execution decisions + noticed-not-filed

1. **GTH-V15-49 resolution is a formalized decision**, not de-facto — recorded in
   `.planning/CONSULT-DECISIONS.md`, commit `f4c9a02`. Do not re-litigate.
2. **Catalog corruption (§4 #3) is noticed-but-NOT-YET-FILED.** #56 found it this
   rotation; it has no `GTH-V15-NN` id yet. Whoever fixes it in §6 Step A must also
   file the incident (fix-twice rule, §4).
3. **HELD — W5 animation upload** (`gh release upload` of `/home/reuben/workspace/
   reposix-animation-pitch/Reposix Launch Animation.mp4`, for `117-07`). External
   mutation, OWNER-GATED, pending owner approval. Router raises to owner when W5 is
   reached — do NOT self-authorize.
4. **Non-blocking — `GOOD-TO-HAVES.md` 77.3k chars** (378% of the 20k ceiling;
   warn-only, waived to 2026-08-08). Milestone-close/OP-9 archive-split call, not
   this rotation's problem.
5. **Already-filed W1 intake — do NOT re-file:** `GTH-V15-44` (attach.rs/list.rs
   split candidate), `GTH-V15-45` (non-sim backend error-teaching gap), MEDIUM
   nextest-not-installed row.
6. Prior W2 C1 relieved cleanly at `5c14b2f` with its own detailed handover (§2) —
   treat that C1 as **DEAD**; always dispatch a fresh coordinator, never resume it.

## 6. Precise next steps (successor #57 runbook)

1. **Run the §1 verify block yourself.** Confirm `HEAD`/`origin/main` drift, tree
   clean, and re-poll `gh run list` if a push has landed since this was written.
2. **Step A — dispatch a focused executor** (sonnet or haiku) to fix push-blocker
   #3: change the invalid `BIND` row to the correct `NextAction` variant (candidate:
   `WRITE_TEST`), verify `target/release/reposix-quality doc-alignment plan-refresh
   docs/index.md` parses and the row's walk semantics are correct, commit with
   targeted staging, and file the `GTH-V15-NN` incident + reinforce the catalog-write
   rule in `quality/CLAUDE.md` (fix-twice). **Prerequisite for Step 3.**
3. **Step B — ROUTER runs `/reposix-quality-refresh`** (top-level-only) for the 4
   drifted docs named in §4 blocker #1 → clears #1. Do not edit doc content, only
   refresh the catalog binding.
4. **Step C — dispatch a FRESH opus `phase-coordinator` C1** (do NOT resume the dead
   W2 C1 — read `117-HANDOVER.md` first, §2). Charter:
   (i) implement docs-repro **Option B** (§4 blocker #2) catalog-first + verifier →
   clears #2;
   (ii) re-push W2's 13+ commits — push BEFORE any verifier, then `quality/runners/
   run.py --cadence post-push --persist` (`code/ci-green-on-main` P0 must pass);
   (iii) execute W3–W5 per the DAG: W3 = 117-04 ∥ 117-06 (117-06's CLAUDE.md sweep is
   root+scoped `CLAUDE.md` **ONLY**, NOT `docs/**`); W4 = 117-05; W5 = 117-07 close.
   Reuse the full W2 C1 ownership charter (OD-3) and §3 constraints verbatim. The C1
   must **RAISE** to the router — never self-authorize — the W5 animation upload
   (§5 item 3).
5. **Refresh `PROGRESS.md`'s `## NOW`** at every boundary push.
6. **REPLACE this handover** (not append) at your own relief, re-verifying every
   claim live before carrying it forward.
