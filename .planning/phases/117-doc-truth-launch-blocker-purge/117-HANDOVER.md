# 117-HANDOVER.md — P117 Wave-2 C1 relief, push-BLOCKED boundary, 2026-07-16

Written by the outgoing P117 C1 phase-coordinator, relieving at a clean **W2 (117-02 ∥
117-03, docs-truth) code-complete / push-BLOCKED** boundary — NOT a phase close, NOT a
mid-wave relief. Wave 2 is fully implemented and code-review-clean (reviewer YELLOW→GREEN
after the `c76b2c2` mermaid/heading fix that closed GTH-V15-46); the ONLY thing standing
between this tree and a shipped W2 is two pre-push gate blockers that a C1 cannot clear
itself (one needs an L0-level `/reposix-quality-refresh`, the other needs an owner/L0
gate-design ruling). Successor is a **fresh C1** (or L0 acting directly) — this is a
single-phase C1 rotation with no C2 in play for P117.

**Read order:** this file in full → `.planning/phases/117-doc-truth-launch-blocker-purge/PROGRESS.md`
(§ NOW — refresh its BLOCKED framing once #1/banned-words is understood to be already
FIXED by `02a72e3`, see §5 below) → `.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md`
entry `GTH-V15-49` (full pivot analysis, verbatim technical detail not repeated here) →
`.planning/ORCHESTRATION.md` §3 if the template itself is in doubt.

**Do NOT touch:** `docs/guides/troubleshooting.md`, `docs/how-it-works/filesystem-layer.md`,
`docs/index.md`, `docs/reference/cli.md`, `docs/reference/git-remote.md` — their content
is FINAL for W2; further edits before the L0 doc-alignment refresh (§6) would re-drift
the BOUND rows a second time. Do NOT run `walk.sh` / `bind` / `/reposix-quality-refresh`
from inside a C1 (checkpoint protocol, `quality/CLAUDE.md`) — that is an L0-only move.
Do NOT touch `quality/gates/docs-repro/snippet-extract.py:171` without an owner/L0 ruling
on GTH-V15-49 (a) vs (b) first — it is a GREEN-contract change requiring catalog-first +
verifier re-grade.

## 1. Ground truth (git)

- `HEAD = 02a72e3b0ffcb8bf3510109aa4586da6a5647310`. `origin/main = 752895af1b49c9694307305ba1e1829b708c3abd`
  (STALE — last relief handover, #55→#56). Local `main` is **11 commits ahead of
  origin/main, 0 behind** (`git status --branch` verified live this session). All 11
  are UNPUSHED and BLOCKED — see §4. This is expected, not corruption: the pre-push
  hook is correctly refusing to land W2 until the two blockers below clear.
- `git status`: tracked tree **CLEAN**, nothing staged, nothing modified (verified live).
- Commits since `origin/main` (oldest → newest, all local-only):
  1. `f42455a` — docs(117): seed phase PROGRESS.md — W2 in progress
  2. `b82be63` — docs(117-02): fix SC1 Confluence-as-wiki + SC2 cat-networks lie
  3. `214f45e` — docs(117-02): reword troubleshooting.md phantom `reposix detach`
  4. `098e2f5` — docs(117-02): file GTH-V15-46 — SC2 mermaid/heading residual
  5. `0dc0e20` — docs(117-03): SC5a — de-fabricate benchmarks/README.md provenance + catalog-first row
  6. `daa9755` — docs(117-03): SC5b — de-FUSE docs/social/twitter.md
  7. `2a70388` — docs(planning): file GTH-V15-47 — dead render_results_markdown (zero callers)
  8. `874bf11` — docs(117-03): twitter.md truth cleanup — drop unshipped Google Keep connector + refresh char counts
  9. `c76b2c2` — docs(117-02): fix filesystem-layer cat→REST diagram/heading — align visual with SC2 prose (closes GTH-V15-46)
  10. `58adabb` — docs(117): PROGRESS — W2 push BLOCKED by pre-push (banned-words + snippet-coverage + docs-alignment walk), wave RED
  11. `02a72e3` — docs(117-02): clear banned-word regression; escalate docs-repro block-count pivot (GTH-V15-49)
- **This handover's own commit will make it 12 ahead** (relief-handover-writer dispatch,
  same rotation) — do not be alarmed if the successor sees 12 instead of 11.
- Numbered deviations the successor MUST know:
  1. `58adabb`'s PROGRESS.md text is now **partially stale**: it lists the
     `structure/banned-words` P1 FAIL as still-open, but `02a72e3` (the very next
     commit) already fixed it (`troubleshooting.md:334` "promisor" → "lazy-fetch").
     Only #2 (docs-repro/snippet-coverage) and #3 (docs-alignment/walk) remain live
     blockers. PROGRESS.md's "NOW" section needs a refresh pass once the successor
     re-verifies this live — deliberately NOT done in this handover commit (task
     scope was targeted to the handover file + GOOD-TO-HAVES.md only, see commit body).
  2. `GTH-V15-46` is RESOLVED-in-phase (closed by `c76b2c2`) but its GOOD-TO-HAVES.md
     row is left as a resolved-not-archived record per the file's existing convention
     (other RESOLVED rows in that file are left in place, not deleted).

## 2. Wave/cycle state

| Wave | Plans | Concern | State | Commits |
|---|---|---|---|---|
| W1 | 117-01 | SC3 connection-refused errors + SC4 `attach.rs` reword | DONE, GREEN + banked (pushed, CI green pre-#56) | `52092ad` / `4af2ece` / `56a222b` |
| W2 | 117-02 ∥ 117-03 | docs-truth (SC1/SC2/SC5 + propagation) | **CODE-COMPLETE, code-review GREEN, push BLOCKED** | `f42455a`..`02a72e3` (11 commits, §1) |
| W3 | 117-04 ∥ 117-06 | per ROADMAP DAG | NOT STARTED | — |
| W4 | 117-05 | per ROADMAP DAG | NOT STARTED | — |
| W5 | 117-07 | phase close | NOT STARTED | — |

No named-incident post-mortem this rotation — the two blockers below are gate-design/
governance friction, not an incident.

## 3. Binding constraints (unchanged)

One tree-writer at a time; ONE cargo invocation machine-wide; never `--no-verify`; push
only at green, then confirm CI green on `main` AFTER the push via the `code/ci-green-on-main`
P0 post-push probe — never open the next phase over a red main; commit-trailer format
(`Co-Authored-By` +, where applicable, `Claude-Session`); model tiering (fable → opus
complex/security, sonnet default, haiku mechanical — never fable at a leaf). Leaf isolation
hard-stop applies to any test/fixture setup a successor lane runs (`/tmp` clone, `cd` in
the SAME Bash invocation) — not directly exercised this rotation (docs-only work) but
still binding for W3/W4/W5 lanes that touch code.

## 4. Litmus / gate / REOPEN state

**Pre-push gate run this rotation (informal — via the actual `git push` attempt, not a
standalone `--persist` run): 3 FAILs at first attempt, now 1 confirmed-fixed + 1
escalated + 1 deferred-by-design:**

1. **`structure/banned-words` (P1)** — FAILED at first push attempt (`troubleshooting.md:334`
   "promisor"). **FIXED** by `02a72e3` (reworded to "lazy-fetch"; verified the gate exits 0
   on this line per the commit body — successor should still re-run the actual
   `bash quality/gates/structure/banned-words.sh` or attempt the push to get a fresh
   authoritative transcript before trusting this is fully clear, since no standalone
   gate-run artifact was persisted this rotation).
2. **`docs-repro/snippet-coverage` (P1) — STILL BLOCKING.** `block_count = 51 > 50`
   threshold; `uncovered_count = 0` (coverage is PERFECT). Root cause: the pivot at
   `quality/gates/docs-repro/snippet-extract.py:171` fires on RAW `len(blocks)`, not on
   the allow-list-filtered/uncovered count — no allow-list addition can reduce it. Filed
   as **GTH-V15-49** (MEDIUM, BLOCKING) with full analysis + two named fix options (a =
   full allow-list-mode cutover per the docs-repro README's own pivot rules, requiring a
   `quality/SURPRISES.md` + `quality/PROTOCOL.md` governance cutover; b = narrow fix,
   change the pivot to count only uncovered blocks). **Needs an owner/L0 ruling before
   any code change** — this alters a gate's GREEN contract, which routes through
   catalog-first + verifier + code-review, not a C1-scoped fix.
3. **`docs-alignment/walk` (P0) — STILL BLOCKING, deferred by design.** BOUND-row
   `STALE_DOCS_DRIFT` from the SC1/SC2 content edits: `docs/index.md` (2 rows),
   `docs/reference/cli.md`, `docs/reference/git-remote.md`,
   `docs/how-it-works/filesystem-layer.md` (the `blob-lazy-first-cat` row needs a full
   REWRITE, not a re-bind, per the `c76b2c2` mermaid re-root). This was intentionally
   NOT chased in W2 — the checkpoint protocol (`quality/CLAUDE.md`) reserves `walk`/`bind`/
   `/reposix-quality-refresh` for an L0-level pass so a C1 doesn't mutate committed
   catalog counters mid-wave. **Resolution: L0 runs `/reposix-quality-refresh` at top
   level against the FINAL committed content** — do not edit these five docs further
   before that refresh runs, or it will re-drift a second time.

**Open waiver expiry clocks (carried, unchanged this rotation):**
- GOOD-TO-HAVES.md / SURPRISES-INTAKE.md OP-8 oversize waiver: expires **2026-08-08**
  (see §5 — GOOD-TO-HAVES.md is now 75,621 bytes = 378% of the 20k soft ceiling, worse
  than when that waiver was set; flagged by 4+ lanes filing into it this milestone).

No REOPEN state active — nothing that previously passed has regressed this rotation.

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

**De-facto decisions made live this rotation:**
- Chose "lazy-fetch" over the gate's own suggested "partial-clone" reword for the
  banned-word fix, because "partial-clone" is ALSO a banned Layer-2 term — a real gap in
  the gate's own error-message suggestion that nobody has filed. **NOTICED, NOT YET
  FILED** — worth a small GOOD-TO-HAVES entry (the `structure/banned-words` error text
  should not suggest a replacement that itself trips the same gate) if a future lane
  hits it again; left un-filed this rotation because it's a one-line self-correcting
  friction, not a recurring blocker, and this handover's commit scope was held to
  exactly two files per the dispatching agent's instruction.
- Filed GTH-V15-49 as MEDIUM/BLOCKING rather than half-building either fix option
  in-lane — deliberate: both options are GREEN-contract changes needing catalog-first +
  verifier routing, outside a docs-truth C1's charter.

**Noticed-not-yet-filed, now triaged this rotation (filed as GTH-V15-50 in the SAME
commit as this handover):**
- `scripts/banned-words-lint.sh:24`'s comment claims "matched term printed to stderr,"
  but `scan_files` actually prints the whole offending LINE, not the isolated matched
  token — the author has to eyeball which word among possibly several tripped the gate.
  This is a Rust-compiler-grade-UX gap per the root CLAUDE.md ownership charter (teach
  the fix precisely). Filed **GTH-V15-50**, LOW severity, sketch: isolate + print the
  matched token in `scan_files`.

**Not yet routed to intake (successor should judge on next touch, not silently drop):**
- The docs-repro pivot design flaw (GTH-V15-49) is a footgun beyond this one block-count
  crossing: it converts PERFECT coverage into a hard push-block at an arbitrary raw
  count, and its only sanctioned escape is undocumented-cost governance work. This
  erodes trust in the quality system generally — worth L0 attention as a standing
  design note, not just a one-off unblock. Raised to L0 in this rotation's report; not
  independently filed as a second ledger row (GTH-V15-49 already captures the concrete
  instance + both fix options).

## 6. Precise next steps (successor runbook)

1. **Spot-check ground truth first.** Re-run `git log --oneline -12`, `git status
   --branch`, and re-attempt `git push origin main` yourself before trusting §1/§4 —
   confirm independently whether blocker #1 (banned-words) is truly clear and get a
   fresh transcript for #2/#3.
2. **RAISE, do not self-attempt, the two live blockers:**
   - (a) L0 runs `/reposix-quality-refresh` at top level for the `docs-alignment/walk`
     `STALE_DOCS_DRIFT` (the five doc rows named in §4 item 3). Content is final —
     do not edit those docs first.
   - (b) L0/owner rules on GTH-V15-49 option (a) vs (b) (§4 item 2, full detail in
     `GOOD-TO-HAVES.md`). Do not bump `PIVOT_THRESHOLD` and do not delete/merge a real
     recovery block to dodge the count — both game the gate rather than fix it.
3. **Once BOTH clear, re-push:** `git push origin main` (Bash timeout ≥300000ms — the
   pre-push hook runs full clippy + kcov, the one sanctioned cargo invocation) → wait
   for the new `ci.yml` run on `main` to conclude `success` → run
   `python3 quality/runners/run.py --cadence post-push --persist` and confirm the
   `code/ci-green-on-main` P0 probe PASSes → update
   `.planning/phases/117-doc-truth-launch-blocker-purge/PROGRESS.md` `## NOW` from
   "W2 push BLOCKED" to W3, and correct the stale banned-words-still-open framing
   noted in §1 deviation 1. Grade W2 RED if the push does not land or main's newest CI
   is not green afterward.
4. **Then dispatch W3 = 117-04 ∥ 117-06.** Note: 117-06 is a **root+scoped CLAUDE.md
   sweep ONLY** — it must NOT touch `docs/**` (that's 117-04's + already-done W2's
   lane); keep the two plans' file sets disjoint even though both run in the same wave,
   and still serialize tree-writer access (one writer at a time even across disjoint
   files, per binding constraints).
5. **Then W4 = 117-05.**
6. **Then W5 = 117-07 (phase close).** Contains one **owner-gated external mutation**:
   `gh release upload` of
   `/home/reuben/workspace/reposix-animation-pitch/Reposix Launch Animation.mp4` — RAISE
   to L0/owner, do NOT self-authorize (OP-9 / ORCHESTRATION §9 named-target rule).
7. **Intake housekeeping (any point after step 3):** GOOD-TO-HAVES.md is now 75,621
   bytes / 378% of the 20k progressive-disclosure ceiling (warn-only/waived to
   2026-08-08, 4+ lanes have filed into it this milestone alone including this
   rotation's GTH-V15-50) — flag for the milestone-close OP-9 archive-split
   conversation; do not let it silently keep growing past close.
8. **Final-report contents when this phase eventually closes:** the RAISE LIST above
   (doc-alignment refresh, GTH-V15-49 ruling, GOOD-TO-HAVES.md size, W5 animation
   upload owner-gate), intake disposition (GTH-V15-44 through GTH-V15-50, all still
   OPEN except GTH-V15-46 RESOLVED), and the STATE.md cursor advance once W5 lands.
