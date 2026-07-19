# RELIEF-HANDOVER-C2-wave-2.md — v0.15.0 "Floor" C2 relief @ P125-closed / P126-not-started boundary, 2026-07-19

Written by the outgoing v0.15.0 C2 milestone coordinator-of-coordinators, relieving at a
**clean phase boundary** — P125 is CLOSED GREEN (verifier-confirmed), nothing is mid-flight
in a tree-writer's hands, and P126 has not yet been opened. Follows
`RELIEF-HANDOVER-C2-wave-N.md` convention (predecessor: `RELIEF-HANDOVER-C2-wave-1.md`,
which covered the P125 close-drive itself). That file's content is superseded by this one
for everything from the P125 close onward; read it only for the P125-close narrative
(SendMessage-disabled discovery, the roadmap "Landed recently" open-question resolution)
if you need the history — this file is self-contained for resuming work.

**Successor's required first reads, in order:** this file in full →
`.planning/ORCHESTRATION.md` §3 (relief/liveness doctrine) → `.planning/STATE.md`
(authoritative cursor, read the Current-Position prose, not just frontmatter) →
`quality/reports/verdicts/p125/VERDICT.md` (what GREEN actually covered) →
`RELIEF-HANDOVER-C2-wave-1.md` (P125 close-drive history, optional/background) →
`C2-MILESTONE-HANDOVER.md` (background/doctrine only, stale since `c267f0e8` — NOT current
state).

**Do-not-touch guardrails:** do NOT self-watch the in-flight CI run named in §1 below; do
NOT dispatch a new tree-writer until you have confirmed a clean, no-live-leaf tree; do NOT
archive the milestone without BOTH gates named in §4; do NOT self-authorize anything in
§6's HELD/ESCALATE list.

---

## 1. Ground truth (git)

Verified live this rotation via `git log`, `git status`, `gh run list` — not carried from
a stale snapshot:

- **Local HEAD (before this handover commit) = `4cc9836a`.** `git status --porcelain` —
  clean. `git rev-list --left-right --count origin/main...HEAD` = **`0 0`** — local and
  `origin/main` are identical; `4cc9836a` is pushed and synced. This handover's own commit
  will advance HEAD one further and needs its own push (§6 step 1).
- **CI on `4cc9836a` is IN-FLIGHT, not yet confirmed green** — verified via `gh run list
  --branch main`: the `CI` workflow run `29674209349` and `release-plz` run `29674209355`
  both show `status: in_progress` (started `2026-07-19T05:01:37Z`); a `Push on main` run
  (`29674209068`) already completed `success`. **This is a live push→CI-in-flight
  boundary — L0's job to watch, not this handover's.** Do not treat `4cc9836a` as
  CI-confirmed-green until L0 relays that.
- **CI on the prior commit `21b89fe3` (P125 close-bookkeeping) IS confirmed green** — `CI`
  run `29673847760` = `success` (completed `2026-07-19T04:47:22Z`), `Docs` workflow run
  `29673992856` = `success`. This is solid ground.
- **Commit chain since the wave-1 handover boundary (`cb4c2b3d`), newest first:**
  - `4cc9836a` — `/gsd-quick` eager-fix of P125 verifier RAISE #1 (a cold-reader tension in
    `docs/roadmap.md`'s "How to read this" line vs. the dated/numbered three-block strip
    above it); scoped the "no phase numbers/dates" claim to the mermaid capability map
    only. Also verified (not re-executed — already present, no fabrication) that the
    follow-on milestone arc (v0.17→v0.19→v0.21→v0.23→v0.25 + stub-milestone note) was
    already correctly landed in `cb4c2b3d`'s "Up next, in order" block. Gates clean;
    `docs/roadmap.md` is binding-free (only `docs/development/roadmap.md` carries live
    `doc-alignment.json` rows — see §5).
  - `21b89fe3` — P125 **close bookkeeping**: STATE.md → 12/15 (80%), now
    **verifier-confirmed** (not optimistic — the wave-1 handover's caveat is resolved);
    `ROADMAP.md` P125 row → Complete 3/3; `docs/roadmap.md` strip P125 → "Landed recently";
    3 new SURPRISES-INTAKE rows from verifier NOTICED items (§5).
  - `55d66378` — **independent gsd-verifier VERDICT for P125: GREEN**, all 3 SCs, committed
    at `quality/reports/verdicts/p125/VERDICT.md`. Fresh verifier, zero P125 authoring
    context, graded against committed artifacts (not executor's word) — the correct HCI
    pattern, closing the OP-7 gap that bit P124.
  - `043447c3` — an L0 relief handover (`#64→#65`), not this C2's file; noted for
    completeness, no action owed.
  - `6472f020` — the wave-1 C2 relief handover itself (superseded by this file).
  - `cb4c2b3d` — (already covered in wave-1's own ground truth) the three-block roadmap
    reshape quick, pushed this rotation's predecessor.
- **P125 CLOSED GREEN, verifier-confirmed.** `quality/reports/verdicts/p125/VERDICT.md`
  (12551 bytes, committed `55d66378`): **SC1** (mirror-refresh pre-step
  `scripts/refresh-tokenworld-mirror.sh` documented + wired to the `pre-release-real-backend`
  cadence, DRAIN-02) GREEN; **SC2** (milestone-close vision-litmus self-heals BOTH backend
  drift and GitHub-mirror drift, reconciling through the reposix bus remote before the
  marker push, DRAIN-12) GREEN; **SC3** (helper `git pull --rebase` mirror-lag teaching
  string fixed + v0.14.0 attach-tree recovery blockquote made remote-explicit, DRAIN-12)
  GREEN.
- **STATE.md milestone counter: 12/15 (80%), verifier-confirmed.** `next = P126`
  (Docs-alignment tooling polish, DRAIN-15..21) per both STATE.md's Current-Position prose
  and `.planning/ROADMAP.md:79`. **Do not advance the counter again for the P125 close** —
  12 already counts P125; only a RED P125 verdict would have reverted it to 11 (this trap
  was caught and avoided this rotation).
- **Milestone: 12/15 phases verdict-closed GREEN (P114–P125, all with committed
  `quality/reports/verdicts/p12X/VERDICT.md`); P126–P128 not started.**

## 2. Wave/cycle state

| Phase | Plans/SCs | State | Commits / verdict |
|---|---|---|---|
| P114–P121 | — | DONE, CLOSED GREEN | unchanged, see `C2-MILESTONE-HANDOVER.md` history |
| P122 `reposix-remote` + `init` hardening | 4/4 | DONE, CLOSED GREEN | `p122/VERDICT.md` (`00ab1579`) |
| P123 Quality-runner & catalog integrity | 5/5 SC | DONE, CLOSED GREEN | `p123/VERDICT.md` (`2f6d62ff`) |
| P124 Container-rehearse harness hardening | 4/4 SC | DONE, CLOSED GREEN | `p124/VERDICT.md` (`c267f0e8`, OP-7 remediation) |
| **P125 Real-backend cadence & mirror-drift resilience** | **3/3 SC** | **DONE, CLOSED GREEN** | verdict `55d66378`; close-bookkeeping `21b89fe3`; RAISE#1 eager-fix `4cc9836a` |
| P126 Docs-alignment tooling polish (DRAIN-15..21) | 0/TBD | **NOT STARTED — next** | — |
| P127 Surprises absorption (OP-8 Slot 1) | 0/TBD | NOT STARTED | — |
| P128 Good-to-haves polish + milestone close (OP-9 Slot 2) | 0/TBD | NOT STARTED | — |

No named incident this rotation (P125's close was clean — verifier dispatch → GREEN →
close-bookkeeping → one eager-fix quick, no reopen, no confabulation, no orphaned build).

## 3. Binding constraints (unchanged — embed verbatim in every dispatch)

- **ONE cargo invocation machine-wide, FOREGROUND-only** (never `run_in_background`;
  orphans the build, evades `cargo-mutex.sh`, OOM risk). Prefer `-p <crate>`.
- **Leaf test setup runs in a `/tmp` clone, `cd`-ing in the SAME Bash invocation — NEVER
  the shared repo.** Mechanically enforced (`leaf-isolation-guard.sh` + pre-commit
  backstop + binary-side `reposix init` refusal RPX-0406).
- **Uncommitted = didn't happen.** Commit before ending any turn.
- **No `--no-verify`, ever.**
- **One tree-writer at a time.**
- **SendMessage is DISABLED at the C2 tier and below** (carried finding from the wave-1
  handover, re-confirmed operative this rotation — no new contradicting evidence, but also
  still NOT formalized into `ORCHESTRATION.md` doctrine as a tier caveat, see §5). Practical
  consequence: **drive every phase close via FRESH verifier→executor LEAVES** (a new
  dispatch each time — the P122-blessed deterministic pattern, `ORCHESTRATION.md` §11),
  **never fork-to-resume a coordinator** (P123 confabulated a zero-tool no-op close this
  way) and **never background-and-resume a child** — you cannot recall or resume it once
  dispatched. **L0→C2 SendMessage DOES appear to work** (untested by this handover directly,
  but consistent with the Liveness doctrine's own wording and unrefuted across two
  rotations now).
- **NEVER self-watch CI.** STOP and RETURN to L0 at every push→CI-in-flight boundary with
  the pushed SHA + in-flight `ci.yml`/workflow run id(s); L0 holds the durable watch and
  SendMessages the coordinator to resume on green.
- **Push cadence:** `git fetch origin && git rebase origin/main`, then `git push origin
  main` **BEFORE** the verifier-subagent dispatch, THEN `python3 quality/runners/run.py
  --cadence post-push --persist` — the P0 `code/ci-green-on-main` probe must show main's
  NEWEST `ci.yml` run = success. Never open the next phase over a red main.
- **Tainted-by-default / `REPOSIX_ALLOWED_ORIGINS` egress allowlist.** Sim is the default
  backend everywhere.
- **Model tiering:** every C1 gets an EXPLICIT `model` override — opus
  security/genuinely-complex, sonnet default, haiku mechanical. Never fable at a leaf.
- **Commit-trailer format:** `Co-Authored-By: Claude Opus 4.8 (1M context)
  <noreply@anthropic.com>` on every non-trivial commit.
- **OD-3 ownership charter (5 points) embedded in every dispatch:** acceptance criteria
  are the floor not the ceiling; noticing is a deliverable; eager-fix (<1h, no new dep) or
  file to `SURPRISES-INTAKE`/`GOOD-TO-HAVES` (OP-8), never silently skip; verify against
  reality; Rust-compiler-grade UX as the standing north star.
- **Mission over stale-plan literalism** (ORCHESTRATION.md §10) — verify a plan's premise
  before executing it. The P125 RAISE#1 eager-fix this rotation is a live example: the
  "extend Up-next into the follow-on arc" task was verified ALREADY DONE (landed in
  `cb4c2b3d`) before any further edit, avoiding duplicate/conflicting work.

## 4. Litmus / gate / REOPEN state

- **CI run `29674209349` (`CI` workflow) + `29674209355` (`release-plz`) on `4cc9836a`** —
  IN-FLIGHT at last read, no conclusion yet. **L0 holds this watch.** No REOPEN state —
  nothing has failed; this is pending-verdict, not red. This handover's own commit will
  push a further commit and trigger a fresh CI run on top of this one (§6 step 1) — expect
  the successor's very first CI check to be against a SHA newer than `4cc9836a`.
- **P125 verdict: GREEN, committed, final.** `quality/reports/verdicts/p125/VERDICT.md`
  (`55d66378`) — no reopen pending.
- **P124 verdict:** GREEN, committed, unchanged — `quality/reports/verdicts/p124/
  VERDICT.md` (`c267f0e8`).
- **Milestone archive is GATED on BOTH of the following** (do not archive v0.15.0 without
  both — carried from the original C2 charter, unchanged):
  1. **OP-9 retrospective distillation** — a new `.planning/RETROSPECTIVE.md` section
     written FROM intakes + run-findings BEFORE archive (the ratification subagent grades
     RED if missing).
  2. **The non-skippable 9th `pre-release-real-backend` probe**
     (`python3 quality/runners/run.py --cadence pre-release-real-backend`, exit 0), per
     `.planning/CLAUDE.md` § Milestone-close 9th probe (RBF-FW-03) — the vision litmus
     against the sanctioned real backend (TokenWorld), catalog row
     `agent-ux/milestone-close-vision-litmus-real-backend` (`blast_radius: P0`, never
     waived).
- **Open-waiver expiry clocks (unchanged this rotation, re-carry):**
  - `structure/file-size-limits` OVER-BUDGET `--warn-only` waiver — **expires
    2026-08-08T00:00:00Z**. `RELIEF-HANDOVER-C2-wave-1.md` (25010 bytes) is already over
    the 20k soft ceiling and NOT yet confirmed in this waiver's covered-file count — check
    before it grows further. This file (`wave-2`) starts fresh rather than re-editing
    wave-1 past its size, per the same discipline. Cross-ref **GTH-V15-90** (two
    `structure/file-size-limits` residual files grew further over ceiling in 125-01) — a
    P127-slotted drain item.
  - Hero-number doc-alignment waivers (8 rows) — expire 2026-08-15, not re-verified fresh
    this rotation.
- **Standing `code/shell-coverage` P2 counter drift (34-vs-27, non-blocking, exit 0)** —
  re-confirmed present, unresolved, TRACKED at
  `.planning/milestones/v0.15.0-phases/surprises-intake/part-07.md:44` (+corroborating
  rows through line 177). Routed to P127 (§ below); if it recurs in **CI** (not just local
  kcov) it needs a `scripts/shell_coverage.py` retune, not just an owner accept-decision.

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

- **Task (d) from this rotation's charter — DONE, verified not fabricated.** "Extend
  Up-next into the follow-on milestone arc" was ALREADY present in `docs/roadmap.md`
  (landed `cb4c2b3d`, matches `.planning/PROJECT.md` Arc D: v0.17 gate-shapes → v0.19
  truth-purge+IA → v0.21 benchmark-honesty → v0.23 journey-slices → v0.25 launch-kit, +
  interleaved stub-milestone note). Only the RAISE#1 scoping fix (line ~105) needed doing;
  done at `4cc9836a`, binding-free, gates clean.
- **STILL NOT FORMALIZED, carried a second rotation — the SendMessage-disabled-at-C2-tier
  finding.** Wave-1 recommended raising this to L0 explicitly so `ORCHESTRATION.md` §3/§11
  gets a tier caveat (or the finding is confirmed session-specific). No evidence this
  rotation that it was raised or resolved — `grep -n SendMessage
  .planning/CONSULT-DECISIONS.md .planning/STATE.md` returns nothing. **Successor: raise
  this to L0 at the next natural report-up point** (do not let it silently carry a third
  rotation without at least an explicit ping).
- **RAISE-LIST for the successor (each with a destination — do not re-diagnose):**
  1. **`verdict.py --phase` bare-session false-RED trap** → **P126** fix-twice (a note is
     owed to `quality/PROTOCOL.md` per the wave-1 handover's original routing; carry
     forward, not yet actioned).
  2. **`docs.yml` deploy-gap + stale `docs/development/roadmap.md` duplicate** →
     **P126 or a `/gsd-quick`**. Verified this rotation: `docs/development/roadmap.md`
     EXISTS (3992 bytes, version-pinned v0.1.0…v0.11.0 content, last touched `2026-07-07`)
     and carries **5 live `quality/catalogs/doc-alignment.json` rows** (confirmed via
     `grep -c '"file": "docs/development/roadmap.md"'` = 5) while `docs/roadmap.md` is the
     current, actively-maintained capability page. This is a genuine stale-duplicate risk
     — a binding-guarded file nobody is updating. **Caveat: this handover could NOT find a
     committed artifact for the "docs.yml deploy-gap BUG" half of this claim** (no hit in
     `quality/reports/verdicts/p125/VERDICT.md`, `.planning/STATE.md`, or recent commit
     messages for "docs.yml" + "deploy-gap"/"gap") — it is carried forward verbatim from
     the outgoing coordinator's own session findings per instruction, but the successor
     should independently verify the `docs.yml` half before acting on it, not just the
     `docs/development/roadmap.md` duplicate half (which IS independently confirmed).
  3. **Cold-reader residual: `docs/roadmap.md:3` header** ("grouped by capability, not by
     release number or date") reads in minor tension with the now dated/numbered
     three-block strip below it — same shape as the RAISE#1 fix just landed at `4cc9836a`;
     mirror that scoping if reworded. Route through the mandated `/doc-clarity-review`
     cold-reader pass before shipping the next user-facing roadmap edit. → **P126**.
  4. **`code/shell-coverage` 34-vs-27 counter-drift** (P2, standing FAIL, exit=0
     non-blocking) → **P127 (CLOSE-INTEGRITY drain slot)**. TRACKED at
     `surprises-intake/part-07.md:44` + corroborating rows through :177. If it recurs in
     CI (not just local kcov), retune `scripts/shell_coverage.py`; else this is an
     owner accept/waive decision.
  5. **File-size residuals** — `.planning/STATE.md` (31846 bytes, ~1.6× the 20k soft
     ceiling) + `surprises-intake/part-07.md` (30153 bytes, ~1.5×) → **P127**. Split
     part-07 per the existing OP-8 split-index convention; GTH-V15-90 tracks this, waiver
     clock 2026-08-08.
  6. **Dead `PROTECTED_IDS` var at `scripts/refresh-tokenworld-mirror.sh:66`** (already
     flagged in the v0.14.0 intake, still unresolved) → **P127**.
  7. **P124's `REQUIREMENTS.md` DRAIN-13/14/22/23/24 still unmarked** — re-verified this
     rotation via direct `grep`: lines 179, 191, 201, 247, 252 still show `[ ]` /
     traceability rows at 333-334, 342-344 still show "Pending", despite P124 being CLOSED
     GREEN with its own VERDICT attesting delivery. → **P128**: cross-check against
     `p124/VERDICT.md`'s SC1-SC4 before flipping the checkboxes + traceability rows.
  - **Filed this milestone (P125 verifier NOTICED, already committed to
    `surprises-intake/part-07.md`), for future drain — do not re-file:**
    - **ROW A** (`part-07.md:101`, LOW-MEDIUM) — weak OR-assert in
      `crates/reposix-remote/tests/push_conflict.rs:352-354` (`|| "reposix attach"` — the
      contract's OR is satisfied on either branch, per VERDICT.md NOTICED §1 discussion).
    - **ROW B** — env-var naming divergence: `scripts/refresh-tokenworld-mirror.sh:65`
      reads `REPOSIX_CONFLUENCE_SPACE_OVERRIDE` (default `REPOSIX`) while
      `quality/gates/agent-ux/milestone-close-vision-litmus.sh:53` reads
      `REPOSIX_CONFLUENCE_SPACE` (default falls back to `REPOSIX_CONFLUENCE_SPACE` when the
      override is unset) — currently consistent by construction, but an operator
      overriding one and not the other silently retargets only half the real-backend
      cadence. Verbatim location confirmed at `part-07.md:391,406`.
    - **GOOD-TO-HAVES GTH-V15-91** (`good-to-haves/part-10.md:43`) — SC2's litmus
      self-heal (backend-drift preflight + mirror reconcile-before-marker-push ordering)
      has ZERO self-contained tests; a reconcile-ordering regression would escape CI
      entirely today. → P127 drain candidate.
  - **Standing:** **GTH-V15-87** (`good-to-haves/part-10.md:13`) — zsh
    `${PIPESTATUS[0]}`/`pipestatus` is 1-indexed, a footgun that already bit a P124 leaf;
    re-confirmed still open this wave, no new occurrence.

## 6. Precise next steps (successor runbook)

1. **Push this handover commit and hand the SHA + this rotation's TWO in-flight CI runs to
   L0.** `git push origin main`. Then re-run `gh run list --branch main --limit 5` to get
   the NEW run id(s) triggered by this push, and report BOTH that id and the still-possibly-
   unresolved `4cc9836a` runs (`29674209349` CI, `29674209355` release-plz) to L0. **Do not
   self-watch either.**
2. **AWAIT L0 green** on the newest run before dispatching any tree-writer. If L0 has not
   yet relayed a verdict, STOP and wait (SendMessage-to-resume per the Liveness doctrine,
   noting the still-open §5 tier caveat — if a resume doesn't arrive in a reasonable window,
   escalate to L0 directly rather than self-watching).
3. **Open P126 (Docs-alignment tooling polish, DRAIN-15..21)** — dispatch a fresh C1
   `phase-coordinator` (model tier per complexity; this phase reads as sonnet-default
   unless research surfaces security-judgment work) with the FULL GSD arc: research → plan
   → plan-check → execute → code-review → phase-close push → post-push cadence →
   independent fresh `gsd-verifier` → committed `quality/reports/verdicts/p126/VERDICT.md`
   in the SAME close (the OP-7 lesson — never defer the verdict). Hand it §5's RAISE-LIST
   items 1–3 (verdict.py bare-session trap, docs.yml/docs-development-roadmap duplicate,
   cold-reader header residual) so it opens already primed, plus this file's §3 binding
   constraints verbatim.
4. **Continue P127 (OP-8 Slot 1) → P128 (OP-9 Slot 2 + milestone close)** in the same
   pattern, one fresh C1 per phase. Hand P127 §5 items 4–6 + GTH-V15-91 + the standing
   `code/shell-coverage` drift; hand P128 §5 item 7 (REQUIREMENTS.md DRAIN-13/14/22/23/24)
   plus the milestone-archive dual-gate in §4 (OP-9 RETROSPECTIVE + the 9th
   `pre-release-real-backend` probe) — P128 does NOT close the milestone until both pass.
5. **HELD / ESCALATE-FIRST — never self-authorize, carry forward unchanged:** any
   release/tag (`v*`/crates.io); any real-backend mutation beyond the 3 sanctioned targets
   (Confluence TokenWorld / GitHub `reubenjohn/reposix` issues / JIRA `TEST`); the
   `structure/file-size-limits` waiver umbrella (expires 2026-08-08, owner decision among
   fix-drift/accept/shard still owed); hero-number doc-alignment waivers (expire
   2026-08-15); the milestone ARCHIVE itself (gated per §4, plus report-to-L0-before-
   archive).
6. **Relieve yourself (the C2) past ~100k tokens of your OWN context** (hard stop ~150k,
   absolute not %) at the next phase boundary: dispatch `relief-handover-writer` to write
   `RELIEF-HANDOVER-C2-wave-3.md` (fresh file, same convention — do not re-edit this one
   past ~20-25KB). Report to L0 ONLY at: your own relief, an owner-decision escalation,
   milestone-close-ready, a 2–3-phase checkpoint, and each push→CI-in-flight handoff.

---

**No pointer update needed to `C2-MILESTONE-HANDOVER.md` this rotation** — it already
carries a pointer (added at the wave-1 handover) directing readers past `c267f0e8` to the
`RELIEF-HANDOVER-C2-wave-N.md` chain; this file continues that chain, no separate edit
required.
