# P118 Verdict — Post-bench honesty corrections

**Overall: GREEN**

**Phase goal:** the disputed token-count figure (DOCS-07/SC1) and the stale tag-cut premise
(DOCS-09/SC2) are corrected with accurate, current information — prose-only, no scope re-open
(SC3). Graded goal-backward against the codebase at HEAD `a2b0186`; the executing agents' word
did not close this phase, this grading did.

**Verifier:** Claude (gsd-verifier), unbiased grader
**Verified:** 2026-07-17
**Shipped commits:** `e510d4f` (PLAN) · `59f932a` (SC1) · `246a2d1` (SC2) · `a2b0186` (readability split)

---

## STEP 1 — CI-green litmus (P0)

`python3 quality/runners/run.py --cadence post-push --persist` →
`[PASS] code/ci-green-on-main (P0, 0.93s)` — summary `1 PASS, 0 FAIL … exit=0`.
The P0 probe asserts main's NEWEST `ci.yml` run concluded success; ground truth confirms
push-event run `29575437132` for sha `a2b0186` on `main` = completed/success, origin/main
HEAD = `a2b0186`. Unrelated `release-plz-*` PR-branch `action_required` runs are out of P0 scope.
**Result: GREEN — not a RED-and-stop.**

---

## STEP 2 — SC grades

### SC1 (DOCS-07) — PROJECT.md token figure re-cite — **GREEN**

`.planning/PROJECT.md` L49 "Why this exists":

- **(a) FUSE path gone.** `grep -nE 'mnt/jira|PROJ-123' .planning/PROJECT.md` → exit 1 (absent).
  Replaced by git-native `cat issues/<id>.md` inside the checkout. ✓
- **(b) Fabricated ratio gone.** `grep -nE '~2k|~150k|75x|75×'` → exit 1 (absent). ✓
- **(c) Figures trace to and MATCH `docs/benchmarks/token-economy.md`** (diffed against the
  source table, L37/39/40):
  | Metric | token-economy.md (source) | PROJECT.md (cite) | Verdict |
  |---|---|---|---|
  | Output tokens | 1,213 vs 21,171 — **~94.3% fewer** | 1,213 vs 21,171 — **~94% fewer** | ✓ honest round (down) |
  | Total input-context | 244,556 vs 550,219 — **~55.6% smaller** | 244,556 vs 550,219 — **~56% smaller** | ✓ honest round |
  | Cost per session | $0.2076 vs $0.8278 — **~74.9% cheaper** | $0.21 vs $0.83 — **~75% lower** | ✓ honest round |
  Raw counts are byte-exact; the three percentages round honestly to nearest whole. No mismatch. ✓
- **(d) Source cited.** Line ends `… offline-reproducible (docs/benchmarks/token-economy.md)`. ✓
- **(e) Synthetic 89.1% NOT resurrected.** `grep -Fc '89.1' .planning/PROJECT.md` → **0**. ✓
  (R1 already adjudicated: using P115's live figures is the correct honest call; the retired
  synthetic 89.1% baseline is intentionally not carried into PROJECT.md.)

### SC2 (DOCS-09) — stale tag-cut premise corrected — **GREEN**

`.planning/milestones/audits/2026-07-12-reality-check.md`:

- **(a) Premise IS stale.** `git tag -l` confirms **v0.13.0, v0.13.1, v0.14.0 all exist**
  (both project-level `vX` and per-crate `reposix-*-vX` tag families present). ✓
- **(b) Five stale homes + §5-Q1 companion annotated.** Dated master banner added at doc top,
  plus inline `[SUPERSEDED 2026-07-17 — P118/DOCS-09 …]` markers at §2 bloat-map (~L352),
  §3 Exhibit E (~L429), §4 punch-list (~L483), §4 Option ii (~L502), §5 Q2 tag-cuts (~L567),
  and the §5-Q1 89.1% companion (~L566). ✓
- **(c) Annotation-only.** `git show 246a2d1 --numstat` → **22 insertions, 0 deletions**;
  every original 2026-07-12 audit sentence preserved verbatim adjacent to its marker. History
  annotated, not rewritten/deleted. ✓

### SC3 — prose-only, no scope re-open — **GREEN**

`git diff --name-only e510d4f^..a2b0186` lists ONLY:
`.planning/PROJECT.md`, `.planning/milestones/audits/2026-07-12-reality-check.md`,
`.planning/phases/118-post-bench-honesty-corrections/118-01-PLAN.md`.
ZERO code / cargo / `docs/**` / `mkdocs.yml` / catalog changes. Diff stat: 198 insertions,
1 deletion, all under `.planning/`. ✓

---

## Findings

None blocking. No RED loop-back items.

## Intake disposition

Nothing filed. No SURPRISES-INTAKE / GOOD-TO-HAVES items surfaced by this verification — the
change set is tightly scoped and internally consistent.

## NOTICED (ownership deliverable)

- **Honest-rounding direction is safe.** All three PROJECT.md percentages round to nearest
  whole; the headline 94.3%→94% rounds *down* (conservative). None inflate the claim. Good.
- **89.1% still lives in reality-check.md — correctly.** The audit file retains "89.1%" inside
  its §5-Q1 text, but that is the historical record and its SUPERSEDED companion explicitly
  routes the reader to the live P115 figure. This is annotate-not-rewrite done right, not a
  leaked synthetic figure. SC1(e)'s grep is correctly scoped to PROJECT.md only.
- **Input-context re-mapping is defensible.** The old "~2k vs ~150k tokens of context" claim is
  re-anchored to "total input-context 244,556 vs 550,219" — an honest analog swap (the sentence
  says as much), not a silent metric substitution.
- **Reproducibility claim is load-bearing.** PROJECT.md asserts the benchmark is
  "offline-reproducible"; token-economy.md is the cited home. Not re-run here (out of prose-phase
  scope), but the citation chain is intact and P0 CI is green.

---

_Verdict: **GREEN** — all three success criteria satisfied, P0 CI green, scope clean._
_Verifier: Claude (gsd-verifier) · 2026-07-17_
