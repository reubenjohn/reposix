# C2-MILESTONE-HANDOVER.md — v0.15.0 "Floor" C2 continuation, P122/P123 boundary, 2026-07-18

Written by the outgoing C2 milestone coordinator-of-coordinators at a **coordinated clean
relief**: both the current C2 and L0 rotate here simultaneously. P122 is CLOSED GREEN on
green main; P123 has NOT been dispatched yet. This is a phase boundary, not a mid-phase
pause — the successor C2 opens by dispatching a fresh C1 for P123.

**Successor's required first reads, in order:** `.planning/ORCHESTRATION.md` (full — you
are a C2, §3 governs your own relief and your C1-rotation-absorption duty), `.planning/
PROJECT.md` (Current Milestone: v0.15.0 Floor), `CLAUDE.md` (root — non-negotiables), then
this file in full before dispatching anything.

---

## 1. Ground truth (git)

- **HEAD == origin/main == `a9e1f4c4`.** `git status`: clean, nothing to commit, up to
  date with origin/main. (Re-verify yourself — this is a snapshot at handover-write time.)
- **Last 5 commits** (all P122 close, newest first):
  - `a9e1f4c4` — docs(122-close): advance STATE 8→9 (P122 CLOSED GREEN); kept the two new
    on-demand catalog rows on-demand cadence, matching P120 sibling precedent
  - `00ab1579` — docs(122-verdict): phase-close verdict GREEN — SC1/SC2/SC3 verified
    against reality (**verdict file: `quality/reports/verdicts/p122/VERDICT.md`**)
  - `985e7dc2` — docs(122-review): commit gsd-code-reviewer REVIEW.md (SHIP-WITH-NITS)
  - `cb7b511b` — fix(code): manual_let_else clippy lint in resolve_import_parent
    (pre-push RED fix)
  - `f5974ebe` — fix(122-close): correct RPX-0406 latch-1 corruption narrative
- **CI, verified live via `gh run list --branch main`:** newest runs on `a9e1f4c4` are
  ALL `success` — `CI` (run `29638768189`, 5m28s), `release-plz` (run `29638768191`),
  `CodeQL` (run `29638768144`), `Docs` (run `29638923434`). Main is GREEN. The verdict
  file additionally cites CI run `29637816791` @ `985e7dc2` (the pre-STATE-advance commit)
  = success, and the P0 `code/ci-green-on-main` probe PASS (0.78s) at close.
- **No deviations to know about** — this was a clean deterministic close (see §5 for the
  liveness lesson that made it need to be deterministic rather than a passive relay).

## 2. Wave/cycle state

| Phase | Plans | State | Verdict / commits |
|---|---|---|---|
| P114 t4 oid-drift fix-first + reconcile audit | 2/2 | DONE | `114-VERIFICATION.md`, real-backend PASS 2026-07-15T17:56Z |
| P115 Live MCP benchmark re-measurement | 1/1 | DONE | top-level execution mode |
| P116 ADR-010 mirror-fanout packet + slug→id design | 3/3 | DONE | top-level execution mode |
| P117 Doc-truth launch-blocker purge | 6/7 | DONE (E1 sub-task owner-PENDING, not blocking) | see §5 |
| P118 Post-bench honesty corrections | 1/1 | DONE | `p118/VERDICT.md` |
| P119 Docs/planning simplification (P112 RAISE) | 4/4 | DONE (DP-4 pivot) | `p119/VERDICT.md` |
| P120 CLI + helper error hardening | 1/1 | DONE | `p120/VERDICT.md` |
| P121 RPX error-code namespace + `reposix explain` | 1/1 | DONE | `p121/VERDICT.md` (`80a37cea`) |
| **P122 `reposix-remote` + `init` hardening** | **4/4** | **DONE — CLOSED GREEN** | `p122/VERDICT.md` (`00ab1579`); close `a9e1f4c4` |
| **P123 Quality-runner & catalog integrity hardening** | **0/TBD** | **NOT STARTED — next** | — |
| P124 Container-rehearse harness hardening | 0/TBD | NOT STARTED | — |
| P125 Real-backend cadence & mirror-drift resilience | 0/TBD | NOT STARTED | — |
| P126 Docs-alignment tooling polish | 0/TBD | NOT STARTED | — |
| P127 Surprises absorption (OP-8 Slot 1) | 0/TBD | NOT STARTED | — |
| P128 Good-to-haves polish + milestone close (OP-9 Slot 2) | 0/TBD | NOT STARTED | — |

**Cursor: 9/15 phases complete (P114–P122), 60%.** Next = P123.

**Named-incident post-mortem to read before dispatching any C1 that will push with CI
in-flight:** the P122 close-liveness incident — full detail in §5 below. Read it; it is
the single most important operational lesson of this rotation.

### Phase-list ground truth: P123–P128 scope is ALREADY PINNED, not a blank slate

**Correction to a premise you may have inherited:** the milestone-scoped
`.planning/milestones/v0.15.0-phases/ROADMAP.md` IS a stale "PLANNING / Phase TBD" stub
(it only contains the two old UX phase stubs + two "Phase (candidate)" entries — all four
of which are ALREADY IMPLEMENTED as P120/P121/P122). **Do not plan phases from that file.**
This staleness is itself tracked: **GTH-V15-27** (LOW, OPEN) — "Milestone-scoped
`v0.15.0-phases/ROADMAP.md` is a stale stub superseded by the live `.planning/ROADMAP.md`."

**The AUTHORITATIVE live roadmap is the top-level `.planning/ROADMAP.md`** (§ "v0.15.0
Floor (PLANNING)"). It already has concrete Goal + Depends-on + Requirements +
Success-Criteria for EVERY phase P123–P128 — what's NOT yet done is the `/gsd-plan-phase`
wave/plan breakdown (`**Plans**: TBD` on all six). So the successor C2's job per phase is
"run `/gsd-plan-phase 12X`" (which pins the concrete plan/wave shape against the
already-scoped Goal+SC), not "invent scope from scratch." Verified by reading the live
file directly (not trusting a paraphrase) — ground truth as of this handover:

- **P123 — Quality-runner & catalog integrity hardening.** Reqs DRAIN-01/03/04/05/06/10.
  SC1: `run.py` self-sources `.env` (closes the false-green-preflight gap). SC2:
  `--persist` refuses to downgrade a committed-GREEN row without `--allow-downgrade`.
  SC3: concurrent `--persist` runners can't race-corrupt the catalog JSON (flock/single
  lane). SC4: a `structure/verifier-script-exists.sh` gate catches a missing/non-exec
  `verifier.script`. SC5: `code/ci-green-on-main` watches a required-workflow LIST (not
  hardcoded `ci.yml` only), and the t4 gate surfaces the real oid-drift stderr instead of
  a misleading git-version fallback. **This directly absorbs the two open HIGH
  SURPRISES-INTAKE rows below (env-loading gap, `--persist` downgrade) — they are this
  phase's reason for being, not separate follow-up work.**
- **P124 — Container-rehearse harness hardening.** Reqs DRAIN-13/14/22/23/24. Earned
  per-step `ASSERT-PASS:` congruence (not verbatim-emitted); SIGKILL-proof sim teardown;
  `target/debug/reposix` provenance on the CI runner; exit code derived strictly from
  persisted `exit_code`; `.sim-*.log` gitignored.
- **P125 — Real-backend cadence & mirror-drift resilience.** Reqs DRAIN-02/12. Mandatory
  mirror-refresh pre-step (or self-reconciling litmus) for `pre-release-real-backend`;
  milestone-close vision-litmus self-heals for BOTH backend AND GitHub-mirror drift.
- **P126 — Docs-alignment tooling polish.** Reqs DRAIN-15..21. `doc-clarity-review`
  hard-fails on a canary probe; README expands "MCP" on first use; grader binds only when
  a cited-number drift would fail the test; `plan-refresh` warns when invoked cold;
  `status` surfaces `waived_active`; the 16 stale "cites out-of-eligible-file" warnings
  resolved.
- **P127 — Surprises absorption, OP-8 Slot 1.** Every SURPRISES-INTAKE row added DURING
  P114–P126's own execution gets a terminal STATUS. Explicitly NOT the already-routed
  DRAIN-01..25 work (that's P123–P126's job) — only NEW intake surfaced while those run.
- **P128 — Good-to-haves polish + milestone close, OP-9 Slot 2.** GOOD-TO-HAVES drained
  to terminal STATUS; `.planning/RETROSPECTIVE.md` v0.15.0 section distilled BEFORE
  archive (verifier grades RED if missing); `CHANGELOG.md` `[v0.15.0]` finalized;
  milestone-close verdict dispatched GREEN including the non-skippable 9th probe;
  `tag-v0.15.0.sh` authored (owner gate — owner runs it, you do not).

This confirms the task premise that **P127 = OP-8 Slot 1** and **P128 = OP-9 Slot 2** —
verified directly against the live ROADMAP, not assumed.

## 3. Binding constraints (unchanged — embed in EVERY C1 charter verbatim)

- **ONE cargo invocation machine-wide.** Cargo is **FOREGROUND-only** — never
  `run_in_background`/detached (orphans the build, evades `cargo-mutex.sh`, OOM risk —
  this VM has OOM-crashed three times from parallel cargo builds). Prefer `-p <crate>`,
  `jobs=2`.
- **Leaf test setup (`reposix init` / sim-seed / `git config` / `git commit` for test
  fixtures) runs in a `/tmp` clone, `cd`-ing into it in the SAME Bash invocation — NEVER
  the shared repo.** Mechanically enforced (PreToolUse `leaf-isolation-guard.sh`, exit 2)
  + pre-commit backstop, but the Bash-tool-only coverage boundary means a subprocess/
  script write can still bypass it (see the P116-close incident, §5 SURPRISES-INTAKE
  line ~545, still OPEN as a hardening gap — HIGH).
- **Uncommitted = didn't happen.** Commit before ending any turn. Mid-phase commits stay
  local until the phase-close push.
- **No `--no-verify`, ever.**
- **One tree-writer at a time.** Tree-mutating work is serial; read-only inspection may
  fan out in parallel.
- **Push cadence:** `git push origin main` BEFORE the verifier subagent dispatch, THEN
  `python3 quality/runners/run.py --cadence post-push --persist` — the P0
  `code/ci-green-on-main` probe must show main's NEWEST `ci.yml` run = success. Never
  open the next phase over a red main.
- **Tainted-by-default / `REPOSIX_ALLOWED_ORIGINS` egress allowlist.** Sim is the default
  backend for every demo/unit-test/autonomous loop.
- **Model tiering:** every C1 gets an EXPLICIT `model` override — opus for
  security/genuinely-complex (P123's catalog-integrity + concurrency-safety work is a
  strong opus candidate; P124's SIGKILL-proofing likewise), sonnet default, haiku
  mechanical. Never `fable` at a leaf.

## 4. Litmus / gate / REOPEN state

- **P122 verdict:** GREEN, `quality/reports/verdicts/p122/VERDICT.md`, verdict commit
  `00ab1579`. 3/3 SC PASS. `persist_downgrade: NONE` (569 catalog rows byte-identical
  before/after `--persist`) — confirmed no silent corruption on this close (contrast with
  the still-open `--persist`-downgrade SURPRISES row below, which is about a DIFFERENT,
  earlier, real incident).
- **Open-waiver expiry clocks (all still ticking, none newly created this rotation):**
  - `structure/file-size-limits` OVER-BUDGET-tier `--warn-only` waiver on
    `GOOD-TO-HAVES.md`/`SURPRISES-INTAKE.md`/etc — **expires 2026-08-08T00:00:00Z**
    (`quality/catalogs/freshness-invariants.json` L666). See §5 OWED lane below — the
    GOOD-TO-HAVES.md split is the concrete action item against this clock.
  - Hero-number doc-alignment waivers (8 rows, BENCH-01-fed) — **expire 2026-08-15**
    (already re-measured by P115; the waivers themselves still need to be lifted using
    P115's figures — check `115-UNWAIVE-PATH.md` if this hasn't happened yet; not
    verified fresh by this handover, flag as a re-ground item for the successor).
  - GTH-V15-78 `rebase-recovery-reconciles.sh` ~42k-char over-budget tier — same
    2026-08-08 waiver umbrella.
- **No open REOPEN state.** P122 is CLOSED GREEN with no outstanding gate failures.

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

### CRITICAL LIVENESS LESSON — read before dispatching P123's C1

The P122 C1 (opus) ran the full arc — plan → execute → code-review (SHIP-WITH-NITS,
nits fixed in `f5974ebe`) → **pushed the implementation** (`985e7dc2`) — then went
**DORMANT** after backgrounding a CI watch that never re-woke it. The C2 (this session)
had to take **deterministic control of the close**: dispatch gsd-verifier directly to
grade the verdict, then gsd-executor directly to advance STATE + push, rather than
waiting on the C1's self-resume.

**The lesson, stated as an operating rule for you (the successor C2):** a coordinator
with in-flight CI **ACTIVELY OWNS the close** — it does NOT end its own turn hoping a
grandchild's report bubbles up through a passive relay. A C1's backgrounded-CI-watch
self-resume, and multi-level report bubbling in general, is **FRAGILE** — it caused a
~4.5h relay gap earlier this milestone (a separate, worse incident). **When ANY C1 you
dispatch pushes and CI is in-flight, ensure a LIVE watcher exists that will re-wake IT**
— either (a) a background `gh run watch <run-id> --exit-status --interval 20` that YOU
(the C2) own and are prepared to act on, or (b) explicit confirmation the C1's own
self-resume mechanism is armed and will actually fire. **Never relieve yourself, and
never let a C1 end its turn, on a passive upward-relay assumption.** If in doubt, take
the close deterministically yourself the way this rotation did — it is cheap and it
worked.

### OWED / TRACKED lanes (carry forward — not yet actioned)

- **ROADMAP progress-table fix owed.** `.planning/ROADMAP.md`'s § Progress table (near
  the bottom, "## Progress") shows **Phase 121 as "0/1 Not started"** and **Phase 122 as
  stale** despite both being CLOSED GREEN (verified — I read the live table above at
  §2's ground-truth pass: it still reads 121 = "0/1 Not started" and does not yet
  reflect 122's 4/4-complete state either, even though the phase-index checkboxes above
  the table DO show both `[x]`). **Fold `roadmap.update-plan-progress 121` +
  `roadmap.update-plan-progress 122` into P123's C1's first grounding step** — its
  planner touches the roadmap anyway when it runs `/gsd-plan-phase 123`.
- **GOOD-TO-HAVES.md cleanup lane — timing call is YOURS to make.** The file is
  **139,235 bytes** (~7× the 20k `.md` ceiling), no index, waived under GTH-V15-02
  corpus-growth **until 2026-08-08**. It MUST be archived/split (resolved-entry archive
  + index) BEFORE that waiver lapses, or pushes start blocking repo-wide. Every C1 writes
  to this file at its own phase close (fix-twice / noticing-disposition), so the split
  MUST run at a **no-C1-intake-writing quiet point** — a between-phase window YOU
  control, e.g. right after a phase closes and before the next C1 is dispatched.
  **Recommendation: run it at an EARLY between-phase boundary (e.g. after P123 or P124
  closes) rather than gambling it lands naturally by P128** — the milestone may not
  close before 2026-08-08, and P128 (OP-9 Slot 2, milestone close) is the wrong place to
  discover this waiver has already lapsed and is blocking the close push itself.
- **P117 anomaly (live, not a P122/P123 blocker).** P117's W5 coordinator close is
  incomplete: the launch-animation E1 mp4 asset publish (`gh release upload` to the
  `docs-assets` release) + the post-upload `animation-renders` playwright verify are
  **owner-PENDING** (manager-deferred 2026-07-17 under standing "outward publishing =
  owner-only" doctrine — confirmed via `.planning/CONSULT-DECISIONS.md` 2026-07-17
  entry: "OWNER DECISION STILL PENDING"). Tracked as **GTH-V15-37**. This is E1-class —
  see §"ESCALATE-to-L0" below. Do not self-authorize; do not let it silently rot either
  — carry it forward in every handover until an owner ruling lands.

### Noticed-not-yet-filed (from this handover-writer's own grounding pass)

- **Correction, not a new defect:** the SURPRISES-INTAKE HIGH row filed 2026-07-14
  20:40 ("Confluence `list_records`-vs-`get_record` oid drift breaks partial-clone
  checkout... page 7766017") still reads `STATUS: OPEN` in the committed file, but **P114
  (completed 2026-07-15) already fixed and LIVE-VERIFIED this exact defect** —
  `114-VERIFICATION.md` records a real-backend run against live Confluence TokenWorld
  (including page 7766017) on 2026-07-15T17:56Z: checkout exit 0, all 3 pages
  materialized, zero `OidDrift` abort, the P0 gate `agent-ux/t4-conflict-rebase-ancestry-
  real-backend` PASS. **This SURPRISES row is stale bookkeeping, not a live product
  defect** — it was simply never marked RESOLVED with a cross-ref to the P114 fix. There
  IS a genuine narrower residual: the fix closes drift for **ADF-native pages only** (the
  LIST path now requests `body-format=atlas_doc_format` with no storage fallback, while
  `get_record` DOES fall back to `body-format=storage`) — a pre-ADF (storage-only) page
  would still trip `OidDrift`; P114's verifier flagged this as OQ1 residual risk and
  confirmed it did NOT manifest on the current TokenWorld substrate (all 3 known pages
  are ADF-native). **Action for the successor:** mark this SURPRISES row RESOLVED
  (cross-ref `114-VERIFICATION.md`) during the P123 or P127 intake sweep, and separately
  file (if not already covered) a LOW/MEDIUM GOOD-TO-HAVE for the pre-ADF list-path
  storage-fallback gap as a documented residual, not treat it as an open "fix-first"
  blocker. **Do not re-litigate or re-fix the already-fixed render-parity defect** — the
  MEDIUM sibling row (misleading git-version error message on oid-drift abort) is the one
  genuinely-open piece, and it is ALREADY P123 SC5's scope (t4 gate real-stderr surfacing)
  — no new phase needed for either.
- The milestone-scoped ROADMAP staleness (GTH-V15-27, filed, LOW, OPEN) is a good <1h
  eager-fix candidate for whichever C1 next touches `.planning/milestones/v0.15.0-phases/`
  — either populate it with a pointer to the live top-level ROADMAP.md, or delete the
  stub. Not urgent; flagging so it doesn't rot further.

## 6. Precise next steps (successor runbook)

1. **Re-verify ground truth yourself** before dispatching anything: `git log --oneline
   -10`, `git status`, `gh run list --branch main --limit 5` (confirm still green — do
   not trust this handover's snapshot past your own check).
2. **Read `.planning/ROADMAP.md` § "Phase 123" directly** (not this handover's summary)
   for the authoritative Goal/Depends-on/Requirements/Success-Criteria before charting
   the C1.
3. **Dispatch a C1 `phase-coordinator` for P123** ("Quality-runner & catalog integrity
   hardening"). Recommend **opus** tier — this phase touches the shared quality-runner
   write path (`--persist` semantics, catalog-JSON concurrency) that many other cadences
   depend on; a mistake here is high-blast-radius. Charter it with:
   - Full GSD arc: `/gsd-plan-phase 123` (gsd-planner + gsd-plan-checker) → execute
     (gsd-executor waves, catalog-first per SC4's own new gate) → gsd-code-reviewer →
     phase-close push (fetch-rebase → `git push origin main` → `run.py --cadence
     post-push --persist`, confirm `code/ci-green-on-main` P0 shows main's NEWEST run =
     success) → gsd-verifier → verdict at `quality/reports/verdicts/p123/VERDICT.md` →
     on GREEN, STATE-advance (completed_phases 9→10, percent, cursor next=P124) → commit
     + push + re-confirm CI green.
   - Fold in the ROADMAP progress-table fix (§5, phases 121/122 stale rows) as this C1's
     first grounding/planning-touch step.
   - Embed verbatim: §3 binding constraints, the OD-3 ownership charter (below), and the
     recurring process lessons (below).
   - Explicitly flag P123 SC1/SC2 as directly absorbing the two open HIGH
     SURPRISES-INTAKE rows (`.env` self-sourcing false-green gap; `--persist` silent
     downgrade) — the C1 should mark those two rows RESOLVED with a commit cross-ref at
     its own phase close, not leave them dangling for P127.
   - Instruct the C1 to mark the stale oid-drift SURPRISES row RESOLVED (cross-ref
     `114-VERIFICATION.md`) as part of its intake-hygiene pass, since P123 is a natural
     touch point for that catalog/runner-adjacent bookkeeping (or defer explicitly to
     P127 if it doesn't fit — either is fine, just don't drop it).
4. **When the C1 pushes with CI in-flight, ensure a LIVE watcher** per §5's liveness
   lesson — do not end your own turn on a passive relay assumption.
5. **Absorb the C1's own relief rotation(s) yourself** (do not bubble to L0) if P123 is
   large enough to need one — dispatch its successor C1 pointed at its handover.
6. **After P123 CLOSED GREEN on green main**, decide the GOOD-TO-HAVES.md split timing
   (§5 OWED lane) — strongly consider running it now, at this early quiet point, rather
   than deferring.
7. **Continue P124 → P125 → P126 → P127 (OP-8) → P128 (OP-9 + milestone close)** in
   order, each via the same one-C1-per-phase pattern, each gated CLOSED GREEN on green
   main before the next dispatch.
8. **At P128 (or whenever milestone-close readiness is reached):** do NOT self-authorize
   any tag/release. Report milestone-close-ready to L0 and WAIT. Confirm BEFORE archive:
   the OP-9 `.planning/RETROSPECTIVE.md` v0.15.0 section is distilled (verifier grades
   RED if missing), and the non-skippable 9th probe
   `python3 quality/runners/run.py --cadence pre-release-real-backend` exits 0 with
   catalog row `agent-ux/milestone-close-vision-litmus-real-backend` PASS (P0, never
   waived).
9. **Relieve yourself (the C2) past ~100k tokens of your OWN context** (hard stop
   ~150k, absolute not %) at a phase boundary — dispatch `relief-handover-writer`,
   report the SHA to L0, stop. Report to L0 ONLY for: your own relief, an owner-decision
   escalation, milestone-close-ready, or a 2–3-phase checkpoint. Otherwise route and
   integrate; absorb C1 rotations below yourself.

---

## ESCALATE-to-L0 list (report and WAIT — never self-authorize)

- **E1 launch-animation mp4/playwright publish** (GTH-V15-37, owner-PENDING per
  `.planning/CONSULT-DECISIONS.md` 2026-07-17) — never self-authorize, never tag
  `[OWNER]` without genuine owner input.
- **Any outward release** — a git tag matching `v*` or a crates.io publish triggers the
  release pipeline. Do NOT self-cut a release/tag at milestone close; report
  milestone-close-ready to L0 and WAIT for owner routing.
- **Milestone ARCHIVE** — before `/gsd-complete-milestone` archives v0.15.0, the OP-9
  distillation into `.planning/RETROSPECTIVE.md` MUST land, AND the non-skippable 9th
  `pre-release-real-backend` probe must pass. Report milestone-close-ready to L0 BEFORE
  final archive.
- **Any user-visible breaking change**; any real-backend MUTATION beyond the sanctioned
  targets (Confluence TokenWorld / GitHub `reubenjohn/reposix` issues / JIRA `TEST`); a
  roadmap item that no longer seems right given new info.

## NON-NEGOTIABLES (embed in every C1 charter, verbatim)

ONE cargo invocation machine-wide; cargo FOREGROUND-only (never `run_in_background` —
orphans the build + evades `cargo-mutex.sh` + OOM risk); prefer `-p <crate>`, jobs=2 (VM
OOM history). Leaf test setup (`reposix init` / sim-seed / `git config` / `git commit`)
runs in a `/tmp` clone with `cd /tmp/...` in the SAME Bash invocation — NEVER the shared
repo (PreToolUse `leaf-isolation-guard.sh` enforces exit 2; pre-commit backstops).
Uncommitted = didn't happen; commit before ending any turn; mid-phase commits stay local
until phase-close push. Tainted-by-default / `REPOSIX_ALLOWED_ORIGINS` egress allowlist;
sim is the default backend. Catalog-first for any new gate (GREEN-contract row committed
BEFORE the implementation the verifier reads). No `--no-verify`.

## RECURRING PROCESS LESSONS (reinforce in every C1 charter)

- Never hand-pick gates before a push — run `python3 quality/runners/run.py --cadence
  pre-push` (FULL cadence). P117 and P121 both shipped avoidable 2-push cascades from
  hand-picked gate sweeps.
- Doc-alignment rebind-in-same-commit: any edit to a line carrying a `doc-alignment`
  binding (`quality/catalogs/doc-alignment.json`) MUST rebind cited rows in the SAME
  commit, or the next push BLOCKs with `STALE_DOCS_DRIFT`.
- Docs edits pass BOTH `docs-build/*` AND `structure/banned-words` locally before a push
  (fix-twice, P117).
- Fix-twice meta-rule: any new file/convention/gate revises the relevant CLAUDE.md (root
  or scoped) / ORCHESTRATION.md in the SAME PR, tagged with the quality dimension.

## OD-3 OWNERSHIP CHARTER (embed in every C1 + subagent charter, verbatim)

1. Acceptance criteria are the floor, not the ceiling: "done" = "I'd defend this in
   review as excellent," not "plan executed."
2. Noticing is a deliverable — every code-touching report names what it noticed nearby
   (lying doc claims, tests that don't assert their namesake, teaching-free error
   messages, dead code, stale comments, missing edge cases). An empty noticing section
   from code-touching work is a red flag.
3. Eager-fix or file — never silently skip: <1h + no new dependency → fix in place;
   else → `.planning/milestones/v0.15.0-phases/{SURPRISES-INTAKE,GOOD-TO-HAVES}.md` with
   severity + sketch.
4. Verify against reality — run the binary, hit the path, render the page; a claim
   without a committed artifact isn't done.
5. North star — Rust-compiler-grade UX: every user-facing error teaches the fix,
   suggests the alternative, gives a copy-paste recovery command (exemplar
   `crates/reposix-cli/src/init.rs::refuse_existing_repo_root`).

## C2 OPERATING DOCTRINE (your own, verbatim)

- Dispatch ONE C1 `phase-coordinator` per phase with an EXPLICIT `model` override (opus
  for security/genuinely-complex phases, sonnet default, haiku mechanical; NEVER fable
  at a leaf). Charge each C1 with the full GSD arc: plan (gsd-planner + gsd-plan-checker)
  → execute (gsd-executor waves) → code-review (gsd-code-reviewer) → phase-close push
  (fetch-rebase → `git push origin main` → `run.py --cadence post-push --persist`,
  `code/ci-green-on-main` P0 must show main's NEWEST ci.yml run = success) → verify+close
  (gsd-verifier → `quality/reports/verdicts/p12X/VERDICT.md`) → on GREEN advance STATE
  (completed_phases+1, bump percent, cursor next=P12Y) → commit+push+re-confirm CI. Only
  advance to the next phase after the current is CLOSED GREEN on green main.
- Absorb C1 rotations below the top: when a C1 relieves (writes a handover under
  `.planning/phases/12X-*/`), the C2 re-dispatches a FRESH successor C1 pointed at that
  handover — do NOT bubble C1 rotations to L0.
- Relieve YOURSELF past ~100k tokens of OWN context (hard stop ~150k; absolute, not %)
  at a PHASE boundary (never mid-phase): dispatch relief-handover-writer → report SHA to
  L0 → stop.
- Report to L0 only: (a) own relief, (b) owner-decision escalation, (c)
  milestone-close-ready, (d) a 2–3-phase checkpoint. Otherwise route and integrate.
