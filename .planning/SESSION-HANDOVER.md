# SESSION-HANDOVER.md — v0.15.0 Floor: P114 PLANNED + pushed, execution opens next — 2026-07-15

Written by the **relief-handover-writer** on behalf of **workhorse #25** (L0
orchestrator, pane w1:p5, herded by the manager in w1:p7), relieving to **successor
#26**. This file **REPLACES** (does not append to) the prior `SESSION-HANDOVER.md`
(#24→#25's handover, committed at `d310a99`).

**Read order:** this file → §1 (verify live, do not trust timestamps) → §6 (runbook) →
dip into §2/§4/§5 as needed. **Guardrails unchanged:** do NOT touch
`.planning/MANAGER-HANDOVER.md` (separate document, separate owner — the manager, pane
w1:p7). No tag push by any coordinator — the manager cuts tags, never L0. Do NOT do git
surgery (reset/rebase/reorder/amend) on `main`. Shared tree with the manager — TARGETED
staging only, never `git add -A`/`.`.

## 1. Ground truth (git) — VERIFIED LIVE this handover, do not trust staleness

Re-run before doing anything else:
```
git rev-parse --short HEAD && git status --porcelain && \
  git rev-list --left-right --count HEAD...origin/main && \
  gh run list --branch main --workflow CI --limit 3 --json headSha,status,conclusion
```
**Verified independently this handover (2026-07-15, just now):**
- Local `main` HEAD = `768bb40`, tree **clean** (`git status --porcelain` empty),
  **EVEN** with `origin/main` (`0  0`) — all of #25's commits pushed.
- **CI CAVEAT — #26 MUST RESOLVE FIRST.** Live poll at write-time shows the newest
  `ci.yml` run on `main` (headSha `768bb40ef946aa2a2c6dc6d036e5cf0dbe973c95`) is
  `status: queued` (`conclusion: ""` — not yet concluded; it had briefly shown
  `in_progress` moments earlier). The two runs below it (`ff4401e`, `d310a99`) are both
  `completed`/`success`. #25's three commits on top of those are pure `.planning/` docs
  (no code, no `docs/**`) and the pre-push gate passed 61/61 (exit 0) before pushing, so
  CI is expected to conclude success — but this is NOT yet confirmed. **#26: re-run the
  block above FIRST. If `768bb40` (or newer) is `queued`/`in_progress`, re-poll until it
  concludes; if `success`, proceed; if `failure`, STOP and diagnose — never open
  `/gsd-execute-phase 114` over a pending or red main.**
- **Commit lineage this rotation** (all pushed, all pre-push-gate-clean):
  `768bb40` (create P114 phase plan — FIX-01 adapter render-parity, FIX-02
  reconcile-audit) ← `e11593d` (P114 Nyquist validation strategy) ← `f3e92d2` (P114
  research: oid-drift root cause + reconcile-scope audit) ← `ff4401e` (MANAGER's
  handover refresh #7→#8 — not #25's, interleaved in the shared tree) ← `d310a99`
  (#24→#25 handover).

## 2. Wave/cycle state

**Milestone v0.15.0 "Floor" re-anchor + requirements + roadmap remain DONE (inherited
from #24, unchanged this rotation). P114 planning is now DONE and pushed; execution has
not yet started.**

| Wave | Artifact | State | Commit(s) |
|---|---|---|---|
| Milestone re-anchor + REQUIREMENTS + ROADMAP | `.planning/PROJECT.md`, `STATE.md`, `REQUIREMENTS.md`, `ROADMAP.md` | **DONE (inherited from #24, unchanged).** 41 REQ-IDs, 15 phases P114–P128. | (carried, see prior handover) |
| P114 research | `.planning/phases/114-t4-confluence-oid-drift-fix-first-reconcile-audit/114-RESEARCH.md` | **DONE.** Confirmed root cause: Confluence `list_issues_impl` fetches pages with NO `body-format` param → empty body in the LIST render; `get_record` uses `?body-format=atlas_doc_format` → real body. `build_from` (builder.rs ~107–122) hashes the empty-body list-render while `read_blob` (builder.rs ~573–634, drift check ~610–618) hashes the real-body get-render at checkout → deterministic `Error::OidDrift`. | `f3e92d2` |
| P114 Nyquist validation | `114-VALIDATION.md` | **DONE.** `nyquist_compliant: true`. | `e11593d` |
| P114 phase plan | `114-01-PLAN.md` (Wave 1, `depends_on: []`, requirements: [FIX-01]) + `114-02-PLAN.md` (Wave 2, `depends_on: ["01"]`, requirements: [FIX-02]) | **DONE — plan-checker PASS on iteration 1 of 3.** 114-01: add `&body-format=atlas_doc_format` to `list_issues_impl`'s LIST url (adapter render-parity, reuses same `translate()`, zero added round-trips, preserves lazy-blob architecture) + RED→GREEN render-parity contract test in `reposix-confluence` + stale-doc fixes. 114-02: reproduction-backed `oid_drift_reconcile` mock test (drift repro + reconcile-non-recovery + aligned-resolves) + scoped doc corrections in `error.rs`/`sync.rs`/`main.rs` (they overclaim `--reconcile` heals oid-drift generally); `cache.rs` confirmed accurate, left untouched. Both FIX-01 + FIX-02 present in `requirements`; all 4 roadmap Success Criteria map to must_haves — SC1 (live TokenWorld `git checkout -B main` incl. page `7766017`, zero oid-drift abort) + SC2 (P0 gate `agent-ux/t4-conflict-rebase-ancestry-real-backend` GREEN) are REAL-BACKEND acceptance runs, NOT autonomous unit tests. | `768bb40` |
| Push batch | — | All of `f3e92d2..768bb40` pushed; pre-push gate green (61 PASS / 0 FAIL) on the final push. Post-push CI: see §1 CAVEAT — **not yet confirmed concluded at relief.** | — |

**No named-incident / diagnostic pending.** No litmus reopened this rotation.
**P114 is planned, not yet executed** — #26 opens with `/gsd-execute-phase 114` (after
resolving the §1 CI caveat), not `/gsd-plan-phase 114` again.

## 3. Binding constraints (unchanged — carried forward)

- **ONE cargo invocation machine-wide** (prefer `-p <crate>`). Leaf isolation: `/tmp`
  clones, `cd` in the SAME Bash invocation, never the shared tree.
- **Uncommitted = didn't happen.** Push per phase → confirm `code/ci-green-on-main` (P0)
  green → **never open next work over a red or pending main.**
- You **route, don't work**: delegate opus (complex/security), sonnet (default), haiku
  (mechanical); never fable at a leaf. Report to the manager (w1:p7) at each boundary or
  when blocked. Relieve past ~100k own-context tokens (hard stop ~150k) at a clean wave
  boundary — write+commit a handover first (token-absolute, not %-of-window).
- **No `--no-verify`. No tag push by any coordinator** — the MANAGER cuts tags. No git
  surgery on `main`.
- **Shared repo with the manager (w1:p7)** — both commit to the SAME working tree. Use
  TARGETED staging (`git add <explicit path>`, NEVER `git add -A`/`.`) so you never sweep
  the manager's uncommitted `MANAGER-HANDOVER.md` edits. **Do NOT touch
  `.planning/MANAGER-HANDOVER.md`** (separate owner).
- **Owner-only stays owner-only:** interactive sudo, new creds/scopes/spend beyond the
  50-session benchmark ceiling, outward publishing.
- **Arc D is RATIFIED** (`6aa734a`, under owner delegation) — normal GSD gates apply, no
  pipeline pause in effect.

## 4. Litmus / gate / REOPEN state (carried forward from #24, with updates)

- **t4 row (`agent-ux/t4-conflict-rebase-ancestry-real-backend`, P0) = the P114 fix
  target.** Do NOT re-run it "to retire a caveat" — it's being FIXED in P114 execution;
  SC2 acceptance for P114 IS the intentional re-run once FIX-01/FIX-02 land.
- `milestone-close-vision-litmus` FAIL under mirror non-idempotency is KNOWN — run
  `scripts/refresh-tokenworld-mirror.sh` FIRST before any real-backend cadence, else
  false-negative on mirror lag. Any real-backend cadence re-run needs `.env` sourced in
  the SAME invocation.
- Freshness historical-H2 regex gap is FIXED (robust version-tuple compare, `baa3583`,
  inherited from #24) — no longer a manual-strip liability. No change this rotation.
- **Open waiver clocks (unchanged):**
  - 8 hero-number doc-alignment rows expire **2026-08-15** (HARD deadline; addressed by
    P115 BENCH-01 — schedule early, ≤50 benchmark sessions on subscription, escalate to
    manager past 50).
  - `structure/file-size-limits` waiver expires **2026-08-08** (SURPRISES-INTAKE /
    GOOD-TO-HAVES progressive-disclosure split is v0.17 scope — do NOT split early).
  - `perf-targets` self-WAIVED until **2026-07-26**.
- **New this rotation — pre-push timing WARN:** #25's final push took **101s** vs the
  ~60s budget noted in `quality/CLAUDE.md` § Cadences (WARN, not FAIL). The gate note
  suggests checking for a new whole-repo gate before assuming diff size is the cause.
  Worth a glance during the next quality-upkeep pass; not blocking.

## 5. Mid-execution decisions + noticings (KEY)

**P114 planning noticings (from the phase-coordinator this rotation, L0-triaged):**

a. **Cosmetic, PASS-compatible, NOT eager-fixed.** `114-RESEARCH.md`'s
   `## Open Questions` heading lacks a `(RESOLVED)` marker; OQ1/OQ2 carry no inline
   RESOLVED note, though both are operationally resolved in the plan text (OQ2 →
   114-01 Task 2's defensive `next_url` body-format re-append; OQ1 → 114-01's
   phase-close TokenWorld-walk honesty block). #25's call: not worth the L0 context to
   open a 588-line research doc for a 2-line marker fix. **#26: fold this into P114
   execution** (the executor already touches these doc areas as part of Wave 1's
   stale-doc fixes) — or explicitly dismiss it if it doesn't naturally land in scope.

**Carry-forward noticings STILL NOT FILED (inherited from #24; #25 did not reach them —
DO NOT drop, this is now 2 rotations old):**

a. **`gsd-sdk` `commit --message` footgun.** The commit message argument is
   **POSITIONAL**, not a `--message` flag; passing `--message "..."` silently commits a
   garbage/empty message instead of erroring. Fix-twice obligation, still open: (i) file
   a SURPRISES/infra intake row, (ii) update the coordinator-dispatch skill /
   `ORCHESTRATION.md` commit example to the correct form —
   `gsd-sdk query commit "<msg>" --files <path>`. This handover was again written using
   the correct positional form; the fix needed is to the DOCUMENTED example other agents
   copy from.
b. **Stale catalog example text.** `quality/catalogs/freshness-invariants.json`
   (~L227–229), the `structure/top-level-requirements-roadmap-scope` row's
   `expected.asserts` text still hardcodes a stale `"v0.12.0"` example. Doc-only,
   non-blocking, cosmetic — fits naturally inside P119 (a DOCS-lane phase).
c. **`PROJECT.md` "Context" section is FUSE-era.** Still carries the old
   `/mnt/jira/PROJ-123.md` FUSE-mount example and a disputed "~150k→~2k (98.7%)" token
   headline figure. **Deliberately left alone** — P115 (BENCH-01 re-measure) and
   P117/P118 (DOCS truth-purge lanes) exist to fix this with real measurement. Do not
   touch ahead of those phases.

**P116 (ADR-010 packet, top-level decision-only phase, unchanged from #24's
handover) — produce options+tradeoffs for BOTH:**
- **ADR-01** — the mirror-fanout decision packet the manager is waiting to rule on.
- **FIX-03** — the GTH-09 slug→id durable-create hazard: the ADR-010 convergence
  contract is FALSE for CREATEs on id-assigning real backends (an interrupted create can
  duplicate on retry).

Route the packet to the **MANAGER (w1:p7) for ruling** — do NOT implement any chosen
ADR-010 option ahead of that ruling.

**Efficiency lesson for #26 (context economy — why only P114 PLANNING fit this
rotation):** onboarding was heavy — reading the ~32k-token
`~/.claude/get-shit-done/workflows/plan-phase.md` inline consumed most of #25's budget.
**#26: do NOT re-read that workflow file.** For P114 EXECUTION, `/gsd-execute-phase`
runs at top-level; delegate the executor/review/verify waves to a phase-coordinator from
the start (as #25 did for the planning tail) to keep L0 context lean.

## 6. Precise next steps (successor #26 runbook)

1. **Re-verify §1 ground truth live first, especially the CI caveat.** Confirm the
   newest `ci.yml` run on `main`'s current HEAD sha concluded `success` before opening
   any new work. If `queued`/`in_progress`, re-poll; if `failure`, stop and diagnose —
   never open `/gsd-execute-phase 114` over a pending or red main.
2. **Opening move: `/gsd-execute-phase 114`** (t4 Confluence oid-drift fix-first, REQs
   FIX-01 + FIX-02). Execute the 2 planned waves: Wave 1 (114-01) = adapter render-parity
   fix + RED→GREEN contract test; Wave 2 (114-02) = reconcile-audit mock test + scoped
   doc corrections. Fold in the §5(a) `(RESOLVED)`-marker fix if it lands naturally in
   scope. Per-phase: push BEFORE the verifier subagent → run
   `quality/runners/run.py --cadence post-push --persist` → confirm
   `code/ci-green-on-main` (P0) green; verifier grades catalog rows; RED loops back.
   **SC1/SC2 acceptance needs the REAL-BACKEND run:** `.env` sourced in the SAME
   invocation + `scripts/refresh-tokenworld-mirror.sh` pre-step, then the P0 gate
   `quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh`.
3. **Schedule P115 (BENCH-01) early.** Hero-number waiver HARD DEADLINE is
   **2026-08-15**. Spend ceiling ≤50 benchmark sessions on the existing subscription —
   escalate to the manager only past 50, never exceed without a manager GO.
4. **P116 (ADR-010 packet)** — produce the options+tradeoffs packet for BOTH ADR-01
   (mirror-fanout) and FIX-03 (slug→id durable create); route to the MANAGER for ruling;
   **no pre-ruling implementation** of either concern.
5. **Directive 2 (GSD-quick scale, low urgency, still NOT started, 2 rotations
   pending):** scratch-repo `reposix-scope-test-DELETEME` KEEP-policy doc into
   `docs/reference/testing-targets.md` (reset via force-push, never delete; currently
   archived, unarchive via API on first reuse).
6. **File the carry-forward noticings from §5** (the `gsd-sdk` positional-arg footgun,
   the stale catalog example text — now 2 rotations old, do not let this become a 3rd)
   so they aren't lost to another rotation.
7. **Report to the manager (w1:p7) at each phase boundary; relieve past ~100k
   own-context tokens at the next clean wave boundary** — write+commit a fresh
   `.planning/SESSION-HANDOVER.md` (REPLACE, not append), naming successor **#27**,
   following this same §3-of-`ORCHESTRATION.md` template.

**Ratchet-first sequence for reference** (canonical = Arc D ADDENDUM, digest only, do
not re-fetch): **v0.15 floor** (current milestone, P114 planned+pushed, execution now
open) → **v0.17 meta-milestone** (5 gate shapes: pivot-vocabulary lint, nav-budget,
hero-redundancy, framing-claim rows, persona whole-journey rubric; + subjective-runner
Task-dispatch fix unfreezing 3 WAIVED meaning-gates; + waiver-escalation rule; +
transcript retention; + bloat remediation incl. the SURPRISES-INTAKE/GOOD-TO-HAVES
progressive-disclosure split) → **v0.19** truth purge + IA rebuild → **v0.21** benchmark
honesty (re-fixture live baseline, CI job, headline-cross-check verifier) → **v0.23**
journey slices → **v0.25** launch kit → Show-HN. **Q3 launch gate:** Show-HN gated on a
walkable REAL-BACKEND journey (GitHub minimum), not sim-first. **Deep-survey
calibration:** ~10% latent work per pass, ~10 passes to converge, recurring deep surveys
are STANDING practice. **Q9 ceiling:** keep v0.15→v0.25 ≈ 6-milestone scale.
