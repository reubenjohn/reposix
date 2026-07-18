# 123-HANDOVER.md — P123 "Quality-runner & catalog integrity hardening" C1 relief handover, 2026-07-18

Written by the outgoing **P123 C1 phase-coordinator** (opus tier — high blast radius: shared
quality-runner write path), dispatched by the milestone C2, relieved at the SC4-resolution wave
boundary (~110-125k own context, per C2's explicit relief dispatch). Successor is also a **P123 C1
phase-coordinator on opus**, reporting to **C2** (NOT L0). ROUTE, don't do leaf work yourself.

**Required reading order for the successor:** (1) this file in full, (2) `git log --oneline -30`
to confirm the chain below is unchanged, (3)
`.planning/phases/123-quality-runner-catalog-integrity-hardening/123-07-PLAN.md` (the close-wave
plan — read the deviations noted in §5 BEFORE dispatching it), (4)
`.planning/ORCHESTRATION.md` §3 "Liveness doctrine" (push→CI-in-flight stop-and-return — you WILL
hit this boundary this session).

**Do-not-touch guardrails:** do not advance `.planning/STATE.md` until AFTER a GREEN
`gsd-verifier` verdict (see §5 — this deviates from what 123-07-PLAN.md's Task 3 literally says);
do not re-open/re-fix the STALE oid-drift SURPRISES-INTAKE row (P114 already fixed it for real —
just resolve the bookkeeping); do not touch the E1 launch-animation publish or any git tag/publish
(escalate-only, see §3); do not run `gsd-sdk query state.advance-plan` against STATE.md (silently
corrupts it — raise to L0 via C2 instead, see §5).

---

## 1. Ground truth (git)

- **HEAD:** `55fc0bd4` — `fix(123-close): refine verifier-script-exists to graded-outcome scope; row PASS + pre-commit`
- **Branch:** `main`. **Tree:** clean (`git status` → "nothing to commit, working tree clean").
- **Ahead/behind:** 27 commits ahead of `origin/main`, 0 behind (as of this handover; re-check
  before pushing — other sessions push to main concurrently, PR #77 precedent).
- **Full commit chain since last-known-clean origin/main tip**, newest first (`git log --oneline
  origin/main..HEAD`):

  ```
  55fc0bd4 fix(123-close): refine verifier-script-exists to graded-outcome scope; row PASS + pre-commit
  8bf65261 docs(123-06): complete verifier-script-exists gate plan (SC4/DRAIN-06)
  e95a13bd test(123-06): selftest for verifier-script-exists + mint row honestly FAIL
  c3782526 feat(123-06): add structure/verifier-script-exists gate (SC4/DRAIN-06)
  92775ac4 docs(123-05): SUMMARY — concurrent --persist catalog-write lock (SC3/DRAIN-05)
  d2770656 docs(123-05): document the --persist concurrency lock (fix-twice)
  555d6362 feat(123-05): verifier + mint structure/persist-catalog-write-locked GREEN
  518c82d1 feat(123-05): serialize concurrent --persist with an advisory flock (SC3/DRAIN-05)
  dffd6966 docs(123-04): document the --persist committed-GREEN downgrade guard (fix-twice)
  f19ad6f5 feat(123-04): verifier + mint P0 structure/persist-refuses-downgrade GREEN
  584b6691 feat(123-04): refuse committed-GREEN catalog downgrade in --persist without --allow-downgrade
  f2e3ec9e docs(123-03): complete SC5a/SC5b plan summary + ROADMAP 3/7
  8a6dde2a docs(123-03): fix-twice — document the required-workflow-list convention
  adb51c0c fix(123-03): SC5b — t4 real-backend-flow surfaces real checkout stderr
  e5ad4241 style(123-03): trim ci-green-on-main.sh header under the 10k .sh file-size ceiling
  5ad70863 feat(123-03): SC5a — ci-green-on-main watches required-workflow list
  887adf57 docs(123-02): complete SC1/DRAIN-03 plan summary + ROADMAP 2/7
  b9f9b445 docs(123-02): fix-twice — run.py self-sources ./.env, manual pre-source no longer required
  a99d87cb feat(123-02): run.py self-sources ./.env, closing the false-green-preflight gap (SC1/DRAIN-03)
  7a002495 test(123-02): add failing TestEnvSelfSourcing for run.py .env self-sourcing
  ce8d485e docs(123-01): complete catalog-first Wave 1 plan summary + note gsd-sdk state.advance-plan side-effect
  5a6a3362 feat(123-01): rewrite SC5a/SC5b contracts for required-workflow list + real-stderr fix (DRAIN-01/10)
  3cf15cc9 feat(123-01): catalog-first GREEN-contract rows for SC1-SC4 (DRAIN-03/04/05/06)
  4179e011 docs(123-plan-check): apply gsd-plan-checker RED fixes (6/6) to P123 plans
  f3f0edbb docs(123): create phase plan — quality-runner & catalog integrity hardening (7 plans, 6 waves)
  b2eca628 docs(quick-260718-lvd): fold corrected LIVENESS DOCTRINE into orchestration doctrine
  eb4f02c0 docs(planning): #61→#62 relief — P122 CLOSED, v0.15.0 at 9/15 (60%), C2 handover @95bc7c5f; L0-owns-CI-watch doctrine
  ```

  **This entire chain rides the P123 phase-close push as ONE unit.** `eb4f02c0` (prior L0
  whole-session handover) and `b2eca628` (an orchestration-doctrine quick, unrelated to P123's
  code) are pre-existing commits from before P123 started — do NOT drop them, do NOT push them
  standalone; they ride this push because they were never pushed after landing.

### Numbered deviations from the plan the successor must know

1. **`quality/gates/structure/verifier-script-exists.sh` (SC4) was refined mid-wave**, not shipped
   as originally scoped in `123-01`/`123-06` — see §2 incident note and §5 for the full
   design-decision writeup that must be flagged to C2.
2. **`.planning/REQUIREMENTS.md`'s DRAIN-06 checkbox is ALREADY flipped to `[x]`** (both the ~L220
   checkbox list and the ~L326 coverage-table row already read `Complete`) — this happened as part
   of 123-06's own fix-twice commit, ahead of the close wave. The remaining 123-07 Task 2 work is
   only **5** checkboxes (DRAIN-01/03/04/05/10), not 6 as the plan's literal task text implies.
3. **SC2 (`_persist_guard.py` downgrade guard) and SC3 (flock) are ALREADY documented** in
   `quality/PROTOCOL.md` (downgrade guard ~L178-184, flock ~L197-207) via their own wave's
   fix-twice commits (`dffd6966`, `d2770656`). SC4 (verifier-script-exists) and SC5a
   (required-workflow list) are ALREADY documented in `quality/CLAUDE.md` (~L79-90,
   ~L187-224). **The only genuinely undocumented convention left for 123-07 Task 2 is SC1
   (`.env` self-sourcing)** — it exists ONLY in `.planning/CLAUDE.md` (§ "Milestone-close 9th
   probe") today, not in `quality/CLAUDE.md` or `quality/PROTOCOL.md`. Confirmed by grep: zero
   hits for `.env`/`self-sourc` in either quality-surface file.
4. **`.planning/phases/123-quality-runner-catalog-integrity-hardening/123-04-SUMMARY.md` is
   genuinely missing** (the 123-04 executor skipped it — confirmed by directory listing) —
   backfill it in the close wave.
5. **`.planning/ROADMAP.md`'s `123-04-PLAN.md` checkbox (~L238) is unchecked** despite 123-04
   being fully landed (commits `584b6691`/`f19ad6f5`/`dffd6966`) — the Progress table row (~L312)
   correctly reads `5/7 | In Progress` matching the 5 checked boxes (01/02/03/05/06), so 04 and 07
   are the two still needing a flip; 123-07 Task 3 item 6 already covers flipping all 7.

---

## 2. Wave/cycle state

| Wave | Plan | State | Commits |
|---|---|---|---|
| 0 (plan+plan-check) | — | DONE | `f3f0edbb`, `4179e011` |
| 1 | 123-01 (catalog-first, SC1-SC4 rows + SC5a/SC5b contract rewrite) | DONE | `3cf15cc9`, `5a6a3362`, `ce8d485e` |
| 2 | 123-02 (SC1/DRAIN-03 — `.env` self-sourcing) | DONE | `7a002495`, `a99d87cb`, `b9f9b445`, `887adf57` |
| 3 | 123-03 (SC5a/DRAIN-10 required-workflow list + SC5b/DRAIN-01 t4 real-stderr) | DONE | `5ad70863`, `e5ad4241`, `adb51c0c`, `8a6dde2a`, `f2e3ec9e` |
| 4 | 123-04 (SC2/DRAIN-04 — persist-refuses-downgrade) | DONE (SUMMARY missing — owed) | `584b6691`, `f19ad6f5`, `dffd6966` |
| 5 | 123-05 (SC3/DRAIN-05 — advisory flock) | DONE | `518c82d1`, `555d6362`, `d2770656`, `92775ac4` |
| 6 | 123-06 (SC4/DRAIN-06 — verifier-script-exists gate) | DONE | `c3782526`, `e95a13bd`, `8bf65261` |
| 6.5 (unplanned) | SC4 gate-scope fix | DONE | `55fc0bd4` |
| 7 (close wave) | 123-07 | **NOT STARTED** | — |

**Named-incident post-mortem to read before dispatching 123-07 (or any further executor):** SC4's
`structure/verifier-script-exists.sh`, as originally landed in wave 6, scanned catalog rows
UNCONDITIONALLY (any row with a non-null `verifier.script` had to resolve+be-executable). This
flagged 5 legitimately-deferred rows as violations: 2 cross-platform WAIVED rows
(`windows-2022`/`macos-14` rehearsal, WAIVED to 2026-09-15), `docs-build/animation-renders`
(NOT-VERIFIED, owner-gated to 117-07 W5), and 2 `docs-repro/benchmark-claim` rows
(`verifier.script: null`) — a genuine row FAIL(P1) that would have blocked the phase-close push.
The predecessor classified this **DP-5 in-charter** (internal gate-scope correction, making the
new gate honor deferrals the project already sanctioned elsewhere — NOT one of the four E1-E4
escalation triggers) and refined the gate (commit `55fc0bd4`) to a **graded-outcome scope**: flag a
missing/non-executable `verifier.script` ONLY for rows whose `status` ∈ {PASS, FAIL, PARTIAL}
(asserts a real run result); exempt WAIVED/NOT-VERIFIED/null-script rows (they assert no
verifier-backed result, so a missing verifier there is not a false-green). Live catalog result:
155 in-scope graded rows / 0 violations / 17 exempt; the 32 `chmod +x` fixes 123-06 already applied
to graded rows remain enforced; selftest is a 14/14 full truth table. **This refinement is fully
documented** in the row's own `comment`/`claim_vs_assertion_audit` (verified directly in
`quality/catalogs/freshness-invariants.json`) and in `quality/CLAUDE.md` § "Verifier-script
existence" — but it is a genuine **scope-narrowing of a security/integrity gate mid-phase**, so it
is flagged again here per the predecessor's instruction: **the successor MUST have gsd-code-reviewer
AND gsd-verifier both explicitly scrutinize this scope decision** (does narrowing to graded-outcome
still close GTH-V15-03's "unbacked PASS" hazard, or did it just move the goalposts?) and **must
surface it to C2 at the push boundary** for visibility before the phase is declared closed — it is
reversible (a one-line scope predicate) but not yet independently re-graded by anyone outside the
executing lineage.

---

## 3. Binding constraints (unchanged)

- **ONE cargo invocation machine-wide**, cargo FOREGROUND-only, never `run_in_background`/detached
  (VM has OOM-crashed 3x on parallel builds). Prefer `-p <crate>`. (This phase is
  Python/shell-heavy; minimal cargo surface, but the rule is whole-machine, not per-phase.)
- **Leaf test setup** (`reposix init`/sim-seed/`git config`/`git commit` for fixtures) runs in a
  `/tmp` clone, `cd`-ing in the SAME Bash invocation — never the shared repo (PreToolUse
  `leaf-isolation-guard.sh` exit 2 enforces; do not try to work around it).
- **Uncommitted = didn't happen** — commit before ending any turn; mid-phase commits stay LOCAL
  until the phase-close push (already true for all 27 commits above). No `--no-verify`. Targeted
  staging only (never `-A`/`.`). **One tree-writer at a time** — the predecessor ran every wave
  SERIAL, not parallel; keep that discipline. No tag push, no crates.io publish.
- **Never hand-pick gates before a push** — run `python3 quality/runners/run.py --cadence
  pre-push --persist` (FULL). It exceeds the 2-minute foreground tool-call budget on this VM
  (cargo/kcov + full-workspace clippy + mkdocs) — run it patiently, do not kill it early.
- Docs edits must pass **BOTH** `docs-build/*` AND `structure/banned-words` (fix-twice
  discipline; a docs-build-only sweep once let a plumbing term leak to a push attempt).
- **Model tiering:** opus for SC-security/concurrency leaf work (this phase's own tier), sonnet
  default for mechanical/doc work, haiku for reads only. Never fable at a leaf.
- **Liveness protocol (load-bearing, see preamble):** do NOT self-own a `gh run watch` and end the
  turn assuming it wakes you — only L0 has reliable background re-invocation. At the
  push→CI-in-flight moment: STOP, RETURN to C2 the pushed SHA + the in-flight `ci.yml` run id; C2
  relays to L0 (durable watch) and re-invokes you (SendMessage) to resume on CI green. Do NOT run
  the post-push `code/ci-green-on-main` P0 probe before CI concludes. Everything BEFORE the push
  (plan → execute → code-review) runs straight through with no stop.

---

## 4. Litmus / gate / REOPEN state

All SC rows below are **local only** (unpushed) — verified directly against
`quality/catalogs/freshness-invariants.json` / `code.json` / `agent-ux.json` at handover time, not
taken on faith from prior summaries:

| SC | Row id | Status (verified) | last_verified | Notes |
|---|---|---|---|---|
| SC1 | `structure/quality-runner-sources-dotenv` | **PASS**, P1 | 2026-07-18T11:25:23Z | `quality/runners/_env_load.py`; existing-env-wins per key; conditional on `./.env` existing; no secret leak (KEY names only). |
| SC2 | `structure/persist-refuses-downgrade` | **PASS**, P0 | 2026-07-18T12:09:08Z | `_persist_guard.py`; fires only committed {PASS,WAIVED}→worse; `--allow-downgrade` opt-in; git-HEAD baseline; 17/17 tests. |
| SC3 | `structure/persist-catalog-write-locked` | **PASS**, P1 | 2026-07-18T12:32:25Z | `catalog_persist_lock()` fcntl.flock; wraps full read-modify-write; validate-only lock-free; 2-subprocess contention proven; 21/21 tests. |
| SC4 | `structure/verifier-script-exists` | **PASS**, P1, cadences `[pre-commit, pre-push, pre-pr]` | 2026-07-18T13:21:11Z | Refined to graded-outcome scope (see §2 incident). 155 in-scope / 0 violations / 17 exempt; 14/14 selftest truth table. **Flag to C2 per §2.** |
| SC5a | `code/ci-green-on-main` | **PASS**, P0 | 2026-07-18T11:41:37Z | Loops `WORKFLOWS=(ci.yml release-plz.yml)`; no-run-for-SHA → honest NOT-VERIFIED, never silent pass/hang. Uses `env -u GH_TOKEN GITHUB_TOKEN` around `gh` (see GTH filing in §5). |
| SC5b | `agent-ux/t4-conflict-rebase-ancestry-real-backend` | NOT-VERIFIED (env-gated, no creds) | 2026-07-13T23:54:53Z (predates the fix — expected, real-backend row, not re-run locally) | Fix lives in `quality/gates/agent-ux/lib/t4-real-backend-flow.sh` (`_t4_checkout_or_fail`); real git stderr now surfaces instead of the old "requires git >= 2.34" fallback; hermetic selftest 3/3; sim-arm untouched. |

**No `gsd-code-reviewer` review artifact exists yet for P123** (confirmed: no `REVIEW.md` under
this phase dir, no `docs(123-review):`-style commit in `git log`) — this is the very next step, not
something already done and awaiting close.

**No REOPEN state** — nothing here has cycled RED→fix→RED again; the SC4 refinement in §2 was a
design correction inside the same wave, not a verifier bounce.

**Pre-push / post-push cadence: NOT YET RUN this phase.** `python3 quality/runners/run.py
--cadence pre-push --persist` has not been executed against the full 27-commit local diff — that
happens inside 123-07 Task 3, ahead of the push.

---

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

### Decisions made live, needing successor follow-through

1. **SC4 gate-scope refinement (§2)** — DP-5 in-charter classification by the predecessor; not
   independently re-graded. Successor must route it through gsd-code-reviewer + gsd-verifier
   scrutiny and flag it to C2 explicitly at the push boundary (not just bury it in a commit
   message).
2. **STATE.md advance timing — a genuine tension between the written plan and standing
   convention, NOT YET RESOLVED:** `123-07-PLAN.md`'s Task 3 literally instructs updating
   `.planning/STATE.md` (frontmatter + Current Position + Current Focus) as part of the same task
   that ALSO runs the pre-push suite and the push — i.e. BEFORE any verifier verdict. But the
   P122 precedent (`git log`: verdict commit `00ab1579` landed BEFORE the STATE-advance commit
   `a9e1f4c4`) and this coordinator's own dispatch instructions (explicit: "Do NOT advance
   STATE.md — that is post-verdict close work") both say STATE.md advance happens AFTER a GREEN
   `gsd-verifier` verdict, not during the close wave. **Successor must resolve this before
   dispatching 123-07**: the cleanest path is to have the 123-07 executor do everything in
   Task 3 EXCEPT the STATE.md frontmatter/Current-Position/Current-Focus edit (ROADMAP.md advance
   is fine to do now — it's phase-local bookkeeping, not the milestone cursor), push, stop at the
   liveness boundary, and hold the STATE.md edit for the post-verdict close ritual — matching
   P122's actual precedent. Do not silently follow the plan's literal text over this precedent
   without registering the decision.
3. **`.env` self-sourcing doc gap is real, not yet filed as its own item** — see §1 deviation 3.
   123-07 Task 2 must add the SC1 paragraph to `quality/CLAUDE.md` (near
   `pre-release-real-backend`) and `quality/PROTOCOL.md` ("New runner/validator semantics"); it is
   currently ONLY in `.planning/CLAUDE.md`.

### Noticed, not yet filed (surface to C2 / file per OP-8 if not absorbed in 123-07)

These are named in the 123-07 plan's own Task 3 item 3 list as owed GTH filings — repeating them
here because they are NOT YET in `GOOD-TO-HAVES.md` (confirmed: no "RESOLVED in Phase 123" markers
and no GTH-V15-80/81 exist yet; GTH-V15-79 is the current highest-numbered entry, so 80/81 are the
correct next numbers):

- **MEDIUM** — SC1's `.env`-sourced `GITHUB_TOKEN` shadows an operator's `gh` keyring auth in
  gh-based gates. `code/ci-green-on-main` was patched with `env -u GH_TOKEN GITHUB_TOKEN` around
  its `gh` calls (confirmed present in the catalog row's `sources` field); audit whether any OTHER
  gh-based gate has the same exposure.
- **LOW** — `quality/gates/agent-ux/real-git-push-e2e.sh:143` has a sibling hardcoded
  git-version fallback, same class as the SC5b fix, outside DRAIN-10's scope.
- **LOW** — names-only `.env` diagnostic prints fire on every pre-commit `run.py` invocation;
  should be scoped off pre-commit.
- **LOW** — a PASS→NOT-VERIFIED "silent staleness" transition prints no loud notice (a residual
  from the plan-checker's own earlier pass).
- **LOW** — `quality/runners/test_run.py` (~36k, over the 15k `.py` ceiling, WAIVED to
  2026-08-08) should split into per-feature modules (`test_persist_guard.py`,
  `test_persist_lock.py`, ...).
- **LOW** — the downgrade guard (SC2) blocks per-catalog-FILE, not per-row; a fine-grained variant
  could allow ungraded rows in the same file to persist normally.
- **MEDIUM (GTH-V15-80, pre-filed by name in the plan, not yet in the file)** — the pre-ADF
  Confluence list-path storage-fallback residual carried forward from P114's OQ1 (confirmed real
  by direct read of `114-VERIFICATION.md`: the LIST path requests `body-format=atlas_doc_format`
  with NO storage fallback while `get_record` DOES fall back to `body-format=storage`, so a
  pre-ADF page would still oid-drift). File under a NEW "From Phase 123 close (P114 OQ1 residual
  carry-forward)" heading per the plan's exact wording.
- **LOW (GTH-V15-81)** — `.planning/REQUIREMENTS.md`'s coverage table shows `Pending` for several
  already-closed phases beyond this phase's own 6 DRAIN rows (confirmed: e.g. `UX-02`/Phase 121
  and multiple Phase 114-120 rows still read `Pending` in the table even though those phases
  shipped) — file as a hygiene item for a future drain phase; do not expand 123-07's scope to fix
  every stale row.

---

## 6. Precise next steps (successor runbook)

1. **Dispatch `gsd-code-reviewer` (foreground)** over the full P123 diff (`origin/main..HEAD`,
   27 commits). Explicitly ask it to scrutinize the SC4 graded-outcome scope refinement (§2) as a
   security/integrity-gate scope change, not just a mechanical review. Fix findings in place if
   `<1h` + no new dependency; else file per OP-8 (never silently skip).

2. **Resolve the STATE.md-timing tension (§5 item 2) BEFORE dispatching 123-07** — decide whether
   the 123-07 executor does the ROADMAP.md advance + push now and defers the STATE.md frontmatter/
   Current-Position/Current-Focus edit to the post-verdict close ritual (matching P122 precedent),
   and record that decision (a one-line note in your own handover or `.planning/CONSULT-DECISIONS.md`
   is enough — don't silently pick one without leaving a trace).

3. **Dispatch `gsd-executor` on 123-07-PLAN.md** (sonnet default; it's doc/bookkeeping-heavy, not
   concurrency-security work) with the §5/§1 corrections folded in as executor context:
   - Task 1 (intake hygiene): 4 SURPRISES-INTAKE RESOLVED statuses with real cross-refs (cite
     actual SHAs from the chain in §1 — do not fabricate); 3 GOOD-TO-HAVES RESOLVED tags
     (GTH-V15-01/03/07); file GTH-V15-80 (pre-ADF residual) per the plan's exact heading/wording.
   - Task 2 (fix-twice + REQUIREMENTS.md): only the SC1 `.env`-self-sourcing paragraph is
     genuinely missing from `quality/CLAUDE.md`/`quality/PROTOCOL.md` (§1 deviation 3) — SC2/SC3/
     SC4/SC5a are already documented, don't duplicate. Flip only 5 REQUIREMENTS.md checkboxes
     (DRAIN-01/03/04/05/10 — DRAIN-06 is already `[x]`, §1 deviation 2). File GTH-V15-81 for the
     broader coverage-table staleness noticing (do not fix every stale row).
   - Backfill `123-04-SUMMARY.md` (missing, §1 deviation 4).
   - Task 3: ROADMAP.md — flip the Phase 123 checkbox, all 7 plan checkboxes (123-04's is still
     unchecked, §1 deviation 5), and the Progress-table row to `7/7 | Complete | <date>`. Then
     (per your §5-item-2 decision) either fold the STATE.md edit in now or explicitly hold it for
     post-verdict.

4. **Run the FULL pre-push suite**: `python3 quality/runners/run.py --cadence pre-push --persist`.
   Confirm exit 0. Do not hand-pick gates. Budget patience — this exceeds 2 minutes on this VM
   (kcov + full-workspace clippy + mkdocs).

5. **Fetch-rebase-push**: `git fetch origin && git rebase origin/main && git push origin main`
   (concurrent sessions push to main — re-check ahead/behind immediately before this step, not
   just from §1's snapshot).

6. **STOP at the liveness boundary.** Do NOT background a `gh run watch`. Return to C2: the pushed
   SHA, the in-flight `ci.yml` run id, and "awaiting CI green to run post-push cadence + close" —
   plus the SC4 scope-decision flag from §2. Do not proceed further this turn.

7. **On C2 re-invoke (CI green confirmed):** run
   `python3 quality/runners/run.py --cadence post-push --persist`; confirm PASS for both `ci.yml`
   and `release-plz.yml` via `code/ci-green-on-main`. If not already done in step 3, now edit
   `.planning/STATE.md`: `completed_phases` 9→10, `percent` 60→67, append the P123-CLOSED
   Current-Position paragraph (mirror the P122 entry's structure/detail level exactly — cite the
   5 SCs, the 4 SURPRISES-INTAKE + 3 GOOD-TO-HAVES resolutions + GTH-V15-80/81 filings, "10/15
   v0.15.0 'Floor' phases complete; next = P124"), and fix the stale `Current Focus` pointer
   (still literally reads `/gsd-plan-phase 123` today — confirmed by direct read).

8. **Dispatch `gsd-verifier`** → `quality/reports/verdicts/p123/VERDICT.md`, graded goal-backward
   against SC1-SC5b. Explicitly ask it to independently re-grade the SC4 scope-narrowing decision
   (§2), not just accept the executing lineage's own `claim_vs_assertion_audit` prose. RED loops
   back to fix (do not close); GREEN → commit VERDICT + (if not already committed) STATE.md, push
   (fetch-rebase first), re-confirm `code/ci-green-on-main` PASS.

9. **Report done to C2** with the verdict path + final STATE.md diff. Raise to C2 (for L0): the
   `gsd-sdk query state.advance-plan` STATE.md-corruption bug (reproduced twice this phase, 123-01
   and 123-06, both reverted via `git checkout -- .planning/STATE.md`) — a global `get-shit-done-cc`
   package bug, not a P123-local issue; do not use that tool on STATE.md, edit it by hand.

**Escalate-only (report to C2, WAIT, do not act unilaterally):** the E1 launch-animation publish
(GTH-V15-37, owner-PENDING); any git tag `v*`/crates.io publish; any real-backend MUTATION beyond
the three sanctioned targets; any user-visible breaking change; any ROADMAP item that no longer
seems right.
