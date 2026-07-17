# SESSION-HANDOVER.md — v0.15.0 Floor: P117 W1 GREEN+banked, handing W1→W2 rotation — 2026-07-17

**VERIFY LIVE BEFORE ACTING — do not trust any number below blindly, re-run the §1
verify block yourself first.**

Written by **workhorse #54** (L0 orchestrator), relieving to successor **#55**. This
file **REPLACES** (does not append to) the prior `SESSION-HANDOVER.md` (#53→#54's
handover, commit `5dc334c`, now superseded — that file's P116-close ground truth and
`.git/config` re-corruption hazard are carried forward below where still live; its
P117-not-started framing is stale, P117 W1 is now done). #54 hit **~200k own-context**
at the P117 W1→W2 boundary — past the ~150k hard line (see the gauge note in §3) — and
the manager (w1:p7) directed relief here with no further wave attempted this rotation.

**Read order:** this file → §1 ground truth (verify live FIRST) → §2 wave/cycle state →
§3 binding constraints (carry verbatim, note the gauge-undercount + `.git/config`
hazard) → §4 litmus/gate/REOPEN state → §5 mid-execution decisions + noticed-not-filed
→ §6 runbook (re-dispatching a fresh P117 C1 for W2–W4 is the primary work).

## 1. Ground truth (git) — verify live before acting

```
git rev-parse HEAD && git rev-parse origin/main && \
  git status --porcelain --untracked-files=all && \
  gh run list --branch main -L 3 --json databaseId,headSha,conclusion,workflowName \
    --jq '.[] | "\(.workflowName) \(.headSha[0:7]) \(.conclusion)"' && \
  grep -c '"last_verdict": "RETIRE_PROPOSED"' quality/catalogs/doc-alignment.json
```

**Live-verified by #54 immediately before writing this handover** (raw outputs, re-run
yourself, do not trust blindly):

- `git rev-parse HEAD` → `a00dd8f73787b0a42c8a744ca1272bc61645f7c4`
- `git rev-parse origin/main` (after `git fetch origin main`) → same,
  `a00dd8f73787b0a42c8a744ca1272bc61645f7c4` — **HEAD == origin/main**, no drift.
- `git status --porcelain --untracked-files=all` → **empty output, tree clean.**
- `gh run list --branch main -L 3 ...` →
  `Docs a00dd8f success` / `CI a00dd8f success` / `release-plz a00dd8f success`
  (a 4th recent row, `CodeQL a00dd8f success`, also confirmed) — **CI on the current
  tip is fully GREEN.**
- `grep -c '"last_verdict": "RETIRE_PROPOSED"' quality/catalogs/doc-alignment.json` →
  **`0`.**

**Expected after THIS handover's commit lands:** `HEAD == origin/main == <this
commit>`, tree CLEAN, `RETIRE_PROPOSED` count still `0`, CI green on the new tip.

- **Tip `a00dd8f`** is a **concurrent manager-tier commit** ("refresh manager handover
  — rotation #11→#12"), touching ONLY `.planning/MANAGER-HANDOVER.md` (39 ins / 25
  del), landed on top of #54's last P117 W1 commit (`56a222b`) — confirms the
  shared-tree concurrent-writer pattern flagged in the dispatch brief. #54 did not
  touch that file (guardrail honored).
- #54's own commits this rotation, oldest→newest, all confirmed present in `git log`:
  `10e2d20`, `feb2c0a` (delegation-depth directive), `4349946`, `ce50609`, `44d3476`
  (P117 research + plan + plan-patch), `52092ad`, `4af2ece`, `56a222b` (P117 Wave 1
  implementation + GTH-V15-43 filing + test-name-honesty marker).

## 2. Wave/cycle state

| Phase / item | State | Commits / evidence |
|---|---|---|
| P114 | CLOSED (t4 Confluence oid-drift fix-first) | `dc26302` et al. |
| P115 | CLOSED GREEN — human confirm-retire gate CLOSED (11 rows retired, `RETIRE_PROPOSED`→0) | `ce4d3b7`, `4bb0596` |
| P116 | CLOSED GREEN — gsd-verifier 12/12 must-haves PASS, 0 gaps, 0 blockers | `116-VERIFICATION.md`; `a1cc2d4`/`7412833`, `1ea51b3`, `5ee5e25`, `6825d13` |
| Delegation-depth directive | DONE — encoded into `ORCHESTRATION.md` (L0 = router; ~1h+ legs; `phase-coordinator` C1 per phase/wave, opus complex / sonnet default / haiku mechanical; >100-line reads via `reader-digester`; work executes 2 levels below the seat) | `10e2d20` + `feb2c0a` (§3 polish) |
| P117 planning | COMPLETE under router mode — opus `phase-coordinator` C1 dispatched, planned GREEN (all GSD gates on) | research `4349946`, plan `ce50609`, plan-checker MEDIUM patch `44d3476`; artifacts in `.planning/phases/117-doc-truth-launch-blocker-purge/` |
| P117 SC4 | **RATIFIED = Option B** (decide-and-disclose at L0): reword `attach.rs`'s dangling `detach` error ref; do NOT build a `reposix detach` subcommand. Option A filed to GOOD-TO-HAVES as `GTH-V15-43`. | `4af2ece` |
| **P117 Wave 1 (117-01)** | **COMPLETE + GREEN + BANKED.** SC3 (`reposix list`/`refresh` connection-refused teach-the-fix, matching `init.rs` exemplar) + SC4 (`attach.rs` reword). | `52092ad` + `4af2ece` + `56a222b` (`// test-name-honesty: ok` marker, verified genuine — asserts recovery-teaching, not a hollow cover-up); CI run `29550609095` success on `a00dd8f` |
| P117 Waves 2–4+ | **NOT STARTED.** Docs-editing waves — see §5/§6 for scope + the HIGH raise. | — |

**3/15 v0.15.0 "Floor" phases complete** (P114, P115, P116); P117 is 1-of-N-waves in;
11 phases remain after it (P118–P128).

**P117 remaining scope (for the successor C1's briefing, not yet executed):** SC1
(`index.md`: Confluence = wiki, `reposix init` not `git clone`), SC2
(`filesystem-layer.md` `cat`-doesn't-secretly-network + propagate the fix to
`glossary.md`/`cli.md`/`git-remote.md` — `index.md`/`git-layer.md`/`time-travel.md`/
`trust-model.md` already CLEAN, do not re-touch), DOCS-05 (the real live lie is
`benchmarks/README.md:34`'s nonexistent `scripts/demo.sh` — `token-economy.md` and
`reposix_session.txt` were ALREADY fixed in P115, do NOT relabel them as broken),
furnished-product/IA polish (`GTH-V15-36` — owner "furnished product" bar, verbatim
"Its good, but we can do so much better!"), 80s launch-animation embed (`GTH-V15-37`),
and 117-06 (fix-twice CLAUDE.md sweep + `docs/social/**` freshness gate + dead-code
delete — **its sweep is scoped to root+scoped `CLAUDE.md` only, NOT `docs/**`**, see
the §6 HIGH raise).

## 3. Binding constraints (carry verbatim)

- **ROUTER MODE** (now encoded in `ORCHESTRATION.md` §2/§3/§11): L0 dispatches a
  `phase-coordinator` C1 per phase/wave; opus complex / sonnet default / haiku
  mechanical, never fable at a leaf; >100-line reads via `reader-digester`; target ~1h+
  substantive work per handover.
- **GAUGE NOTE (load-bearing).** The relief line is ~100k soft / **150k hard ABSOLUTE
  own-context**. The Claude Code token-usage HOOK UNDERCOUNTS — #54 read it as ~123k
  while the real gauge (manager-verified, ~20% of a 1M window) was ~200k. **Trust the
  actual gauge %, not the hook's token number**; relieve on the gauge.
- One tree-writer at a time; ONE cargo invocation machine-wide (prefer `-p`); no
  `--no-verify`; targeted staging (never `-A`/`.`); do NOT touch
  `.planning/MANAGER-HANDOVER.md` (manager's file, separate owner); no tag push by any
  coordinator; no git surgery (reset/rebase/amend/reorder) on `main` — the manager
  (w1:p7) is a concurrent writer on main, `git pull --rebase` if origin moved, never
  force; leaf isolation in `/tmp` same-Bash-invocation; **every push Bash timeout
  ≥300s**; refresh `PROGRESS.md`'s `## NOW` at every boundary push; never open the next
  phase over a red main.
- **Human gate is DONE** (P115's 11 rows retired; P115 + P116 both closed) — do NOT
  re-check or re-open it.
- **`.git/config` re-corruption hazard (carried from #53, still live risk).** A sibling
  worktree lane can re-corrupt the shared `.git/config` (`core.bare = true` +
  fixture-identity `[user] email = t@t` injected). If ANY work-tree git op suddenly
  fails ("this operation must be run in a work tree", or a `.githooks/pre-commit`
  fixture-identity reject): FIRST run `cat .git/config`, check for those two symptoms,
  repair via a direct edit (`core.bare = false`, remove the injected `[user]` block —
  race-safe, either lands cleanly or errors if a concurrent write raced it). Filed
  `SURPRISES-INTAKE.md` HIGH by #53; not re-encountered this rotation.

## 4. Litmus / gate / REOPEN state

- **P117 Wave 1 (117-01): GREEN.** SC3 + SC4 delivered, test-name-honesty marker
  independently verified genuine (asserts the sim connection-refused error actually
  teaches the recovery command, not just present). CI on the wave's tip = run
  `29550609095`, `success`, re-confirmed live moments before writing this handover
  (current tip `a00dd8f`, one manager commit ahead, is also green).
- **SC4 decision: RATIFIED, not provisional.** Option B (reword) shipped; Option A
  (real `detach` subcommand) is deliberately deferred, tracked as `GTH-V15-43` — do not
  re-litigate this in W2+ unless the owner reopens it.
- **doc-alignment invariants: HOLDING.** `RETIRE_PROPOSED` = 0 (live-verified above).
  P117 W2+ will edit `docs/**` and is EXPECTED to trip `STALE_DOCS_DRIFT` pre-push
  BLOCK — this is normal, not a regression signal (see §6 item 4 for the recovery
  move).
- **No open litmus/waiver clocks tracked by #54 this rotation** beyond the carried
  `.git/config` hazard (§3) — #54 did not re-verify the `GTH-V15-21` file-size waiver
  (2026-08-08 expiry) live; #55 should re-check it if touching any file near the 20k
  soft ceiling in W2+ doc edits.

## 5. Mid-execution decisions + noticed-not-filed

1. **SC4 = Option B, ratified at L0 (decide-and-disclose).** Recorded above in §2/§4;
   this is a closed decision, not an open thread — do not re-raise unless the owner
   does.
2. **HIGH, open, NOT yet folded into any wave — `docs/guides/troubleshooting.md:329`
   still names the phantom `reposix detach`** (twin of the `attach.rs` reference W1
   just purged): *"To switch SoT, run `reposix detach` first (or remove the
   `extensions.partialClone` config + cache directory by hand)."* Live-confirmed by
   #54 via direct read. 117-06's fix-twice sweep is scoped to root+scoped `CLAUDE.md`
   only (confirmed by reading `117-06-PLAN.md`'s Task 2 action text) — it does **not**
   cover `docs/**`, so this phantom-command lie survives every currently-planned wave
   unless W2+ absorbs it explicitly. See §6 item 3 for the required action.
3. **Intake candidates noticed during W1, not yet filed** — triage-and-file is #55's
   job, not yet done by #54:
   - (a) `attach.rs` (26,021 bytes) / `list.rs` (21,015 bytes) now both exceed the 20k
     soft ceiling (currently masked by the `GTH-V15-21` `--warn-only` waiver, expires
     2026-08-08) — split-candidate: extract a shared `backend_errors` module (sibling
     of the existing `GTH-V15-08`/`GTH-V15-42` split candidates).
   - (b) The github/confluence/jira connection-refused error arms still lack a
     copy-paste recovery line — only the sim arm was brought up to the `init.rs`
     teach-the-fix bar this wave. Candidate for a `GOOD-TO-HAVES` row (north-star UX
     mandate, root `CLAUDE.md` § Ownership charter item 5).
   - (c) `cargo nextest` is not installed in this environment even though plans and
     `crates/CLAUDE.md` recommend it — a dev-image/doc-drift candidate for
     `SURPRISES-INTAKE.md`.
   - Already filed, confirmed present, do not re-file: leaf-isolation grep
     false-positive → tracked in `117-06-PLAN.md` Task 2 item (c); SC4 Option A →
     `GTH-V15-43` (`4af2ece`).
4. **HELD external mutation (owner gate, do not auto-execute).** The animation lane's
   (`GTH-V15-37`) GitHub-Release asset upload is an external mutation requiring
   owner-named-target approval per `ORCHESTRATION.md` §9 — RAISE to the
   owner/manager when that step is reached; do not self-authorize. The rest of
   animation productionization (JSX precompile, localStorage-neutralization,
   autoplay-off, editor-disable) proceeds in-plan without a gate.
5. **Sibling-lane / manager-tier awareness.** The manager (w1:p7) landed one commit
   (`a00dd8f`) mid-rotation touching only `MANAGER-HANDOVER.md` — no conflict, no
   action needed, noted for continuity.

## 6. Precise next steps (successor #55 runbook)

1. **Standard first-act verify block (§1).** Run it yourself; confirm `HEAD ==
   origin/main == a00dd8f` (or this handover's own commit if it has landed by the time
   you read this), tree clean, `RETIRE_PROPOSED` = 0, CI green on the tip.
2. **The prior P117 C1 is DEAD after this pane rotation.** Re-dispatch a **fresh opus
   `phase-coordinator` C1** for P117 Waves 2–4+ from the committed plan state (charter
   template: `coordinator-dispatch` skill). Charter it to execute the remaining waves
   → phase-close (gsd-verifier goal-backward + gsd-code-reviewer folding in the W1 diff
   + nyquist, per all-gates-on mode) → advance `STATE.md`/`ROADMAP.md`/`PROGRESS.md`.
3. **HIGH — must be folded into the W2+ charter or an explicit added wave:**
   `docs/guides/troubleshooting.md:329` still names the phantom `reposix detach` (§5
   item 2). Either absorb it into a docs wave (W2 or later) or broaden 117-06's sweep
   to include `docs/**`. If neither happens, P117 ships shipping the exact
   phantom-command lie the phase exists to purge — treat this as a phase-close
   blocker, not an optional nicety.
4. **EXPECT doc-alignment drift during W2+.** Editing `docs/**` will trip the
   `STALE_DOCS_DRIFT` pre-push BLOCK. The dispatched C1 should CHECKPOINT and RAISE to
   #55 (L0) rather than attempting the fix itself; **#55 runs
   `/reposix-quality-refresh <doc>` at TOP LEVEL** — depth-2 fan-out is unreachable
   from inside the C1's own subagent tree (`.planning/CLAUDE.md` § Subjective-rubric
   dispatch). Budget context for this — it is part of why #54 relieved at the W1→W2
   boundary rather than absorbing it directly.
5. **Push cadence unchanged:** `git push origin main` BEFORE any verifier-subagent
   dispatch, then `python3 quality/runners/run.py --cadence post-push --persist`
   (`code/ci-green-on-main` is P0). Never open the next phase/wave over a red main.
   Every push Bash timeout ≥300s.
6. **Triage-and-file the §5 item 3 intake candidates** ((a) `attach.rs`/`list.rs`
   split-candidate, (b) non-sim backend error-teaching gap, (c) `cargo nextest`
   dev-image drift) to `GOOD-TO-HAVES.md` / `SURPRISES-INTAKE.md` with severity +
   sketch — do not silently drop them, do not scope-creep them into W2+ execution
   without an explicit decision.
7. **Refresh `PROGRESS.md`'s `## NOW`** at every boundary push.
8. **REPLACE this handover** (not append) at your own relief, following this same
   `.planning/ORCHESTRATION.md` §3 template, re-verifying every claim live before
   carrying it forward.
