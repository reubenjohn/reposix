# P117 Phase-Close Verdict (NON-BLOCKED portion) — v0.15.0 Floor
## "Doc-truth launch-blocker purge"

**Overall: GREEN — for the NON-BLOCKED scope this pass was chartered to grade.**

- **Graded HEAD:** `c3b4d5c` (`c3b4d5cb840de602b4ca5a0a000b251a5de72e4f`) — local HEAD ==
  `origin/main` (push-before-verifier cadence satisfied). Working tree carried only the
  expected 1-line `code/ci-green-on-main.last_verified` audit-bump diff at grading entry
  (from the required post-push `--persist` re-run); folded into this verdict's commit,
  targeted-staged, no `-A`.
- **Verifier:** unbiased dispatch, zero prior session context. Every claim below was
  re-derived from the committed catalog, a live `gh run list` / `gh release view` call,
  re-running the actual gate scripts, and reading the diffs of the cited commits — SUMMARY/
  PROGRESS prose was treated as a hypothesis, not evidence, per OD-3.
- **Scope note:** this is a **partial** phase-close grading pass. Phase 117 is NOT fully
  closed — Task 1 (owner-approved `gh release upload` of the launch-animation mp4) is
  genuinely HELD pending owner action (confirmed: `gh release view docs-assets` →
  "release not found" — the asset truly does not exist yet, this is not a masked/lied
  claim). This verdict grades everything reachable without that owner action.

---

## Item 1 — `code/ci-green-on-main` (P0, post-push cadence) — **PASS**

- `gh run list --workflow=ci.yml --branch main --limit 5` → newest run `29568842392`,
  `headSha c3b4d5cb...`, `conclusion: success`, `createdAt 2026-07-17T09:07:30Z`. Matches
  local `HEAD`/`origin/main` exactly (no ahead/behind drift).
- `python3 quality/runners/run.py --cadence post-push --persist` → `[PASS] code/ci-green-on-main (P0, 1.00s)` → `summary: 1 PASS, 0 FAIL ... exit=0`.
- This is the push-cadence floor (CLAUDE.md "Push cadence" / PROTOCOL.md D-CONV-4) and it holds.

## Item 2 — P117-minted/touched catalog rows

| Row | Verifier re-run | Result |
|---|---|---|
| `docs-repro/snippet-coverage` (GTH-V15-49 Option B pivot) | `python3 quality/gates/docs-repro/snippet-extract.py --check` | **exit 0**. Catalog `expected.asserts[2]` (pivots on UNCOVERED count, not raw block count) matches the actual pivot logic; regression pin `test_pivot_counts_uncovered_not_raw_blocks` + its companion both **PASS** (2/2, `pytest -k pivot`). This is the exact gate that BLOCKED the phase's very first push (commit history: `f4c9a02` "[FABLE] consult verdict GTH-V15-49", `5c14b2f` "push BLOCKED on L0 (STALE_DOCS_DRIFT + GTH-V15-49)") — confirmed cleared, not merely reworded. |
| `structure/social-freshness` (minted W3, `1180-1216` in `freshness-invariants.json`) | `bash quality/gates/structure/social-freshness.sh` (exit 0) + `bash quality/gates/structure/social-freshness.selftest.sh` (7/7 cases: clean, FUSE-plant, `/mnt/`-plant, bare-mount-plant, `amount`/`paramount` word-boundary, allowlist-marker escape) | **PASS**. Row's `expected.asserts` (3 strings) verified against the gate's real behavior, not just prose. |
| `docs-build/animation-renders` (P2) | Row read directly: `status: NOT-VERIFIED`, `last_verified: null`, verifier script `quality/gates/docs-build/animation-renders.sh` confirmed **absent on disk** (`ls`: No such file or directory). `run.py --cadence pre-push` confirms `NOT-VERIFIED (P2)` does **not** flip `compute_exit_code` to non-zero (full pre-push run: `62 PASS, 0 FAIL, 0 PARTIAL, 1 WAIVED, 1 NOT-VERIFIED -> exit=0`). | **CORRECTLY NOT-VERIFIED-by-design.** `owner_hint` explicitly documents the artifact + verifier are intentionally absent until W5 uploads the mp4; `blast_radius: P2` so it never blocks. This is the expected owner-E1-deferred state — **not graded as a gap.** |
| `doc-alignment.json` rows touched by 117-04/117-05/117-07 (index.md IA churn ×2, ~29 shifted rows, 16-row post-embed rebind) | `bash quality/gates/docs-alignment/walk.sh` | **exit 0.** Zero unwaived `STALE_DOCS_DRIFT`. Remaining output is (a) informational `coverage:` notes for rows citing out-of-eligible source files (pre-existing, unrelated to P117), and (b) three explicit, dated, reasoned `WAIVED-*` rows (`docs/index/git-checkout-branch-command` until 2026-07-31, `docs/social/twitter/token-reduction-92pct` until 2026-10-10, `benchmarks/README-md/session-provenance` until 2026-10-10) — all pre-existing/tracked, none newly introduced to mask this phase's churn. The 3-cycle line-shift rebind this phase went through (commits `644763a`, `c3b4d5c`) landed stable: re-running the walk today shows 0 drift. |

## Item 3 — Phase deliverables (goal-backward spot-checks)

**(a) Docs-truth restored — spot-checked, all three named claims verified against the live file, not the commit message:**

- **RBF-LR-03 overclaim softened** — `docs/index.md` "What it looks like underneath" section now reads: *"...git pull --rebase && git push, whether the base moved ... (v0.14.0 RBF-LR-03 fix — proven GREEN on git 2.25.1 via the `import` path; verification on the modern-git read path (git ≥2.34) is still open, see backlog item DRAIN-07)..."* — confirmed via `git show d2cc7c6` (added the caveat, dropped bare "reliably") and `git show ab0324b` (reworded away the literal `stateless-connect` token for banned-words compliance while preserving the caveat's substance).
- **No `stateless-connect` in the Layer-1 hero** — `grep -n stateless-connect docs/` returns 13 hits across `docs/reference/*`, `docs/how-it-works/*`, `docs/development/*`, `docs/decisions/*` — **zero** in `docs/index.md`. Confirmed by direct full-file read of `docs/index.md` and the `ab0324b` commit diff (explicit reword rationale: *"docs/index.md:167 named the literal Layer-1-banned plumbing term `stateless-connect`... Reworded... per docs/.banned-words.toml Layer 1 policy"*).
- **P95 stale citation gone** — `git show d2cc7c6 -- docs/index.md`: removed *"(tracked as a P95 candidate)"* from the Confluence-comments-gap line (P95 is an unrelated, already-shipped v0.13.0 phase); `grep -n P95 docs/index.md` → no hits.

**(b) Original launch blocker purged** — the docs-repro FAIL that blocked the phase's first push attempt (GTH-V15-49, raw-block-count pivot false-positive) is cleared; see Item 2 row 1 above. Independently re-run, not just re-read from catalog.

**(c) Launch-animation embed shipped with a visible fallback link** — `docs/index.md` L29-37 (read directly): a `<video controls preload="none" poster="assets/animation/reposix-launch.thumbnail.webp">` with a `<source>` pointing at the pinned `docs-assets` release-tag URL, an in-`<video>` fallback anchor, **plus** (added in the graded HEAD commit `c3b4d5c`) a standalone `<p><em>Video not playing? <a href="...">Download the launch animation (mp4)</a>.</em></p>` immediately after `</video>` — visible regardless of *why* playback fails (missing codec support vs. a 404 on the not-yet-uploaded asset), per the commit's own stated rationale. Confirmed via `git show c3b4d5c -- docs/index.md` (the 2-line diff) and a full read of the rendered markdown.

## Item 4 — State honesty — **PASS, and more precise than this dispatch's own framing (see NOTICED)**

- `.planning/STATE.md` `## Current Position`: *"Phase: **P117 ... — IN PROGRESS, close-pending-owner-E1.**"* followed by an explicit, itemized *"phase 117 is NOT fully COMPLETE"* list naming **Task 1** (owner mp4 upload, HELD-E1), **Task 2** (post-upload playwright artifact + verifier authoring, gated on Task 1), **Task 3** (cold-reader `/doc-clarity-review` + `/reposix-quality-review --rubric` + badge resolution — top-level slash commands), **Task 4** (final pre-push/push/post-push-CI/verifier dispatch).
- `.planning/phases/117-doc-truth-launch-blocker-purge/PROGRESS.md` frontmatter: `status: executing`, `last_activity` names W5/117-07 as "IN PROGRESS, close-pending-owner-E1"; body correctly enumerates Task 1 as HELD-E1 and Tasks 2-4 as still pending, does not claim the phase is done.
- Neither file overclaims completeness. **PASS.**

---

## NOTICED

1. **The dispatch framing ("the ONLY outstanding item is the owner-gated E1 upload") is narrower than what STATE.md/PROGRESS.md themselves document.** Task 3 (cold-reader `/doc-clarity-review` + `/reposix-quality-review --rubric` + badge resolution over `docs/index.md`/`README.md`/the how-it-works quartet) is a `must_haves.truths` entry in `117-07-PLAN.md` itself ("the cold-reader rubric ... sign off on index.md + README.md + rewritten landing surfaces") and is **not** gated on the owner's mp4 upload — it could run today, independent of E1. It has not run this phase: `quality/catalogs/subjective-rubrics.json` shows `subjective/cold-reader-hero-clarity` still at its pre-P117 grade (`last_verified 2026-07-04`, `WAIVED` until 2026-09-15 for an unrelated runner-limitation reason, score 8 from *before* this phase's substantial `docs/index.md` rewrites), and `subjective/dvcs-cold-reader` has never run at all (filed as `GTH-V15-58` this same session, separate concern). STATE.md/PROGRESS.md are accurate on this (they list Task 3 as pending); the verification-dispatch prompt's one-line summary is the only place that undersells it. Recommend the coordinator run Task 3 in the same session as Task 1/2/4 rather than treating E1 as the sole remaining gate.
2. **Badge resolution (part of Task 3) already passes** — spot-checked all three `docs/index.md` badges (`ci.yml` badge, `quality-weekly.yml` badge, shields.io endpoint) via `curl -I`/`-o /dev/null -w %{http_code}` → 200/200/200. Not a gap, but worth recording since Task 3 hasn't formally run it yet.
3. **`gh release view docs-assets` confirms "release not found"** — independently proves Task 1 is genuinely un-done (not a silently-skipped-then-claimed-done step); good-faith honesty signal for the phase.
4. **GTH-V15-58/59 (filed in this phase's final commit)** are legitimate self-noticed framework gaps (cold-reader rubric never run; `run.py`'s verifier-not-found branch produces untracked stub cruft for intentionally-deferred rows) — correctly filed to GOOD-TO-HAVES rather than silently absorbed or ignored.

---

## Verdict

**GREEN** for the chartered NON-BLOCKED scope: main is green on the newest CI run, all P117-minted/touched catalog rows grade as expected (including the intentionally-NOT-VERIFIED P2 animation row), the three named docs-truth defects are independently confirmed fixed in the live file (not just claimed), the original launch-blocking docs-repro FAIL is cleared and regression-pinned, the animation embed ships with a visible fallback link, and STATE.md/PROGRESS.md honestly reflect an incomplete phase. The **only owner-gated blocker** is the E1 mp4 upload — but Task 3 (cold-reader/badge sign-off) is a second, non-owner-gated, still-pending item the coordinator should not conflate with "waiting on the owner" (see NOTICED #1).

---

_Verified: 2026-07-17T09:22:00Z_
_Verifier: Claude (unbiased phase-close dispatch, zero prior session context)_
