# SESSION-HANDOVER.md — v0.15.0 Floor: P117 CLOSED GREEN, standby-router,
E1 animation owner-gate OPEN — 2026-07-17

**VERIFY LIVE BEFORE ACTING — do not trust any number below blindly, re-run the
ground-truth block yourself first.**

Written by **workhorse #58** (L0 ROUTER), relieving to successor **#59** (fresh L0
ROUTER) at a clean wave boundary (P117 closed) — this is a scheduled relief, not a
context-limit scramble, but treat it with the same rigor. This file **REPLACES** the
prior `#57→#58` handover (was reachable at commit `26dd6d8`'s successor state; that
handover's Step-0/Step-B/Step-C runbook is now fully executed and DONE — do not
re-run it). Manager: `w1:p7` (separate session, `.planning/MANAGER-HANDOVER.md` — do
not touch). Milestone **v0.15.0 "Floor"**. **Router ROUTES ONLY** — do not do leaf
work yourself; delegate reads/reports through a reader-digester and cap subagent
report size (see §5 manager delta).

**Read order:** this file → §1 ground truth (verify live) → §2 wave/phase state → §3
binding constraints (includes the OPEN OWNER GATE — read before touching anything
release-related) → §4 litmus/gate state → §5 mid-execution decisions + noticed-not-
filed + honesty lesson → §6 runbook (start at step 1).

## 1. Ground truth (git) — verify live before acting

```
git rev-parse HEAD && git rev-parse origin/main && git status --porcelain
git rev-list --left-right --count origin/main...HEAD
gh run list --workflow=ci.yml --branch main -L 5 \
  --json headSha,conclusion,status,createdAt \
  --jq '.[]|"\(.headSha[0:7]) \(.status)/\(.conclusion) \(.createdAt)"'
```

**Live-verified by #58 immediately before writing this handover (2026-07-17, ~10:10
UTC):**

- `HEAD` = `origin/main` = `97a4008` (`docs(planning): honesty fix — E1 animation
  deferral was MANAGER not OWNER; owner decision still PENDING`). **0 ahead, 0
  behind.** `git status --porcelain` → empty, tree clean.
- Commits since the last fully-verified-green tip, newest-first: `97a4008` (honesty
  fix, this rotation), `df5bdc6` (P117 CLOSED GREEN — record deferral), `cdf5557`
  (file ci-green-on-main race surprise), `698293f` (P117 phase-close verdict,
  GREEN), `c3b4d5c` (visible fallback link fix, W5 final), `3a97766`, `0115511`,
  `644763a` (W4 — rebind doc-alignment push-blocker cascade), `e4a04a6`, `597bc82`,
  `0d4a0b7` (W4 — embed launch animation), `0a5f620` (W3 — rebind push-blocker
  cascade)… back through `c028d4c` (W2 close — resolves GTH-V15-49, files
  GTH-V15-51) and `57d18d6` (GTH-V15-49 Option B fix). All confirmed present in
  `git log`, no gaps.
- **CI, `ci.yml` specifically (not Docs/CodeQL/release-plz):**
  - Newest run on `97a4008` (databaseId `29572436284`) was **`in_progress`** at the
    moment of this check — **NOT YET RESOLVED**. Do not assume it passed.
  - The run on `df5bdc6` (`29572145305`) shows `completed/cancelled` — normal
    behavior (GitHub Actions cancels a superseded in-flight run when a newer push
    lands on the same branch/workflow concurrency group), NOT a failure signal.
  - The **last CONFIRMED-GREEN completed run** is on `cdf5557`
    (`29570612222`, `completed/success`, `2026-07-17T09:37:08Z`).
  - **Seat #59 action required:** re-run the `gh run list --workflow=ci.yml` command
    above and confirm the `97a4008` run has resolved to `completed/success` before
    treating main as CI-green or opening new work. If it resolved to anything else,
    STOP and investigate before dispatching P118 — do not open a new phase over a
    red/unresolved main.
- `quality/catalogs/doc-alignment.json` is currently **clean** (not part of the dirty
  tree at last check) — but this file has shown up dirty intermittently across #57
  and #58's rotations purely as a side-effect of someone running `doc-alignment
  walk` (it rewrites `summary.last_walked`). **If you see it dirty and you did not
  intentionally mutate the catalog via a `reposix-quality doc-alignment <verb>`
  call, `git checkout -- quality/catalogs/doc-alignment.json` and move on — it is
  regenerable, not real work.** This bit both #57 (flagged-anomaly investigation)
  and nearly cost #58 a wasted diagnosis; treat it as known noise, not a new
  incident, unless the walk's *substantive* output (`claims_bound`,
  `claims_waived`, blocking `docs-alignment:` lines) changed in a way that matters
  to the task at hand.

## 2. Wave/phase state

| Item | State | Evidence |
|---|---|---|
| P117 W1–W5 | ALL SHIPPED GREEN on origin/main | W2 `c028d4c` (docs-truth + GTH-V15-49 resolve) · W3 `0a5f620` (IA/CLAUDE.md rebind) · W4 `644763a` (launch-animation embed + rebind) · W5 `698293f` (phase-close verdict, GREEN) · close `df5bdc6` · honesty-fix `97a4008` |
| P117 verdict | GREEN (non-blocked scope) | `quality/reports/verdicts/p117/VERDICT.md` — unbiased re-derivation (live `gh run`, re-run gate scripts, direct file reads), confirms `code/ci-green-on-main` PASS on `c3b4d5c`, docs-repro/social-freshness/doc-alignment rows all PASS, docs-truth deliverables spot-checked against live files, STATE.md honesty PASS |
| GTH-V15-49 (docs-repro pivot false-block) | RESOLVED | Implemented `57d18d6` (pivot on uncovered-count not raw-block-count), ratified in `c028d4c`; 9 regression tests pin it |
| **P117 overall** | **CLOSED for everything reachable without the owner's release-upload action.** Task 1 (mp4 upload) remains genuinely HELD — see §3. | `.planning/STATE.md`, `PROGRESS.md` both correctly say "NOT fully COMPLETE," name the exact held tasks — this is honest state, not overclaim |
| Milestone v0.15.0 "Floor" | **Prose says 4/15 phases complete (P114, P115, P116, P117); 11 remain; NEXT = P118.** (NOTE: `STATE.md` frontmatter `progress:` block says `total_phases: 21, completed_phases: 3` — this conflicts with the prose and is very likely stale/unmaintained; flagged in §5, do not trust the frontmatter numbers, use the prose.) | `.planning/STATE.md` `## Current Position` |
| Dead thread | `.planning/phases/117-doc-truth-launch-blocker-purge/117-HANDOVER.md` (commit `5c14b2f`) — the ORIGINAL W2 C1's relief handover. **Never resume it** — its runbook was fully superseded and executed by later waves. | confirmed present in `git log` |

**Launching P118 is a fresh coordinator dispatch (milestone continuation).** Seat #59
routes it when the owner/manager directs — **do NOT auto-launch** it as the first
action of this rotation; there is no standing directive to start P118 immediately,
only that it's next in the roadmap.

## 3. Binding constraints (carry forward, unchanged unless noted)

- One tree-writer at a time; **ONE cargo invocation machine-wide** (prefer `-p`); no
  `--no-verify`; **targeted staging only** (never `-A`/`.`); no tag push by any
  coordinator; no git surgery (reset/rebase/amend/reorder) on SHARED/pushed `main` —
  the manager (`w1:p7`) is a concurrent writer, `git pull --rebase` if origin moved,
  never force.
- Leaf isolation: `reposix`/sim/git test setup in a `/tmp` clone, `cd` in the SAME
  Bash invocation as the mutating command — never the shared repo.
- **Every push Bash timeout ≥300s** (pre-push runs full clippy+kcov). Push cadence:
  `git push origin main` BEFORE any verifier-subagent dispatch, then `python3
  quality/runners/run.py --cadence post-push --persist` — the `code/ci-green-on-main`
  (P0) probe must pass. Never open the next phase/wave over a red main.
- **Ledger topology:** milestone-scoped ledgers only —
  `.planning/milestones/v0.15.0-phases/{GOOD-TO-HAVES,SURPRISES-INTAKE}.md`,
  `GTH-V15-NN` id scheme.
- **Catalog write discipline:** never hand-edit `quality/catalogs/*.json` — all
  mutation flows through `reposix-quality doc-alignment <verb>` binary calls (a
  hand-authored row corrupted the whole catalog earlier this phase, `d4156f6`).
- **GAUGE NOTE:** relieve at ~100k soft / 150k hard ABSOLUTE own-context, at a wave
  boundary, with a committed handover — not mid-atomic-unit.

### THE OPEN OWNER GATE — highest-priority carry, do not mishandle

The launch-animation E1 publish — `gh release create docs-assets --latest=false` +
upload `/home/reuben/workspace/reposix-animation-pitch/Reposix Launch Animation.mp4`
(6.9MB, confirmed exists) + author `docs-build/animation-renders.sh` + run the live
playwright `animation-renders` verify (the second half of `117-07`) — is
**MANAGER-DEFERRED under standing doctrine (outward publishing = owner-only), with
OWNER APPROVAL STILL PENDING.** This is a genuinely open ask surfaced to the owner,
NOT an accepted deferral or a closed item:

- Ledger entry (verbatim heading): `.planning/CONSULT-DECISIONS.md` `## 2026-07-17
  [MANAGER] launch-animation publish held (117-07 second half)`. Tracked as
  **GTH-V15-37** (`.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md:274`).
- `docs-build/animation-renders` reading `NOT-VERIFIED` in the catalog is a
  **PENDING gate**, not a done/waived one — it will flip to a real verifier once the
  asset is live.
- `--latest=false` is **mandatory** on the eventual `gh release create` — a stray
  `releases/latest` steals the tag from v0.14.0 and 404s installer URLs (see
  `release-plz.toml` header comment).
- **Do not self-authorize this action.** Re-raise it to the **OWNER** (not the
  manager) when the owner is reachable — this is an external, security-sensitive
  mutation per ORCHESTRATION §9 (owner-named-target approval required).

## 4. Litmus / gate / REOPEN state

- **`code/ci-green-on-main` (P0):** PASSED as of the P117 verdict's grading commit
  (`c3b4d5c`, run `29568842392`). **NOT yet re-confirmed for `97a4008`** — see §1,
  this is seat #59's first live-check obligation.
- **`docs-repro/snippet-extract` (GTH-V15-49):** RESOLVED, `exit 0`, 9 regression
  tests pin the uncovered-count pivot logic. No longer a REOPEN risk absent a future
  code change to that gate.
- **`docs-alignment` walk:** clean at last verdict re-run (0 unwaived
  `STALE_DOCS_DRIFT`; 3 pre-existing, dated, reasoned `WAIVED-*` rows unrelated to
  P117's churn). Root-cause of the repeated churn (line-number binding
  brittleness) is GTH-V15-57 — NOT fixed, only mitigated (see §5).
- **`docs-build/animation-renders`:** `NOT-VERIFIED`, `blast_radius: P2` (does not
  block pre-push), correctly documented as intentionally absent pending the owner
  gate in §3. This is the ONE open/pending gate in the whole tree right now.

## 5. Mid-execution decisions + noticed-not-filed + honesty lesson

**HONESTY-DEFECT LESSON — carry verbatim, this is why #58 corrected its own prior
commit:** AskUserQuestion answers in this setup may be routed through the **MANAGER
(session `w1:p7`)**, NOT the human owner. The recurring system-reminder "No genuine
owner input has been received" is the AUTHORITATIVE signal. **NEVER tag a decision
`[OWNER]` or write "owner approved/deferred" unless genuine owner input actually
arrived.** #58 mis-tagged the E1 deferral `[OWNER]` in an earlier commit; the manager
caught it; corrected in `97a4008`. When in doubt, tag `[MANAGER]` and mark the owner
gate PENDING/open, as §3 does above.

**Non-blocking filed follow-ups (all already in v0.15.0 intake ledgers — no action
needed unless drained in an OP-8 absorption slot; confirmed present at these
locations):**

1. `ci-green-on-main` race-guard gap (`SURPRISES-INTAKE.md`, confirmed filed
   ~line 688): the P0 probe can select the wrong run right after a push if a newer
   run starts before the probe queries; fix = assert the returned run's
   `headSha == HEAD` before trusting `conclusion`. MEDIUM-HIGH.
2. **GTH-V15-57** (confirmed filed, `GOOD-TO-HAVES.md:440`) — doc-alignment rows
   bind to LINE NUMBERS, so any edit above a bound row drifts it; this caused
   **three** separate push-blocker cascades in P117 alone (W2, W3, W4 all needed a
   rebind pass). Highest-leverage fix candidate in the whole ledger. Interim
   mitigation already in `quality/CLAUDE.md`: rebind in the same commit as the
   edit (fix-twice rule). Also **GTH-V15-56** (confirmed filed, `:432`): pre-push
   wall time drifted from the documented ~55-60s budget to ~97s (kcov dominates).
3. Both milestone intake ledgers are over the 20KB structure-dimension ceiling —
   `GOOD-TO-HAVES.md` measured **~98KB** (350%+ over), `SURPRISES-INTAKE.md`
   measured **~74KB** — both WAIVED to 2026-08-08, targeted for an OP-9
   milestone-close split. Not this rotation's problem, just confirmed still true.
4. Doc-alignment `.py::fn` test citations are dead-on-arrival (the grader's
   `parse_test` helper only splits `::fn` suffixes for `.rs` files, so a Python
   citation silently fails to bind) — a grader instruction fix + tool capability
   gap, filed by the P117 C1 in the same ledgers as above.

**NEW noticed-not-filed by #58 this rotation (route to `GOOD-TO-HAVES.md` or
`SURPRISES-INTAKE.md` at the next OP-8 touch, do not let it silently drop):**

5. `.planning/STATE.md`'s YAML frontmatter `progress:` block (`total_phases: 21,
   completed_phases: 3`) is inconsistent with its own prose (`4/15 phases complete,
   11 remain`) two paragraphs below. Nobody is maintaining the frontmatter counters
   in lockstep with the prose narrative — worth either wiring an automatic
   frontmatter refresh into the phase-close ritual or dropping the redundant
   frontmatter counters in favor of the prose (single source of truth).

**Manager deltas (carry verbatim — these came from the manager, `w1:p7`, this
rotation and the one before):**

(a) Cap every subagent REPORT at ≤300-word digests in the dispatch charter — an
earlier grader-fanout wave ballooned a coordinator's context from ~96k→~165k on
report size alone; route full-rationale reports through a reader-digester instead of
receiving them raw.
(b) Trust the context gauge percentage; relieve ~100k soft / ~150k hard ABSOLUTE
own-context — this is a scheduled relief at a clean boundary, following that rule.
(c) The animation `gh release` upload stays HELD pending OWNER approval (now
correctly attributed as MANAGER-deferred, not owner-decided — see the honesty
lesson above). Manager channel: `w1:p7`, `.planning/MANAGER-HANDOVER.md` — do not
touch that file.

## 6. Precise next steps (successor #59 runbook)

1. **Run the §1 ground-truth block yourself right now.** Confirm HEAD ==
   origin/main, 0 ahead/0 behind, clean tree, AND — critically — that the `ci.yml`
   run on the current tip has resolved to `completed/success` (it was still
   `in_progress` when #58 checked). If it resolved to anything other than success,
   STOP and investigate before doing anything else; do not open new work over a
   red/unresolved main.
2. If you see `quality/catalogs/doc-alignment.json` dirty and you did not run a
   `doc-alignment` verb yourself, `git checkout -- quality/catalogs/doc-alignment.json`
   — it is regenerable walk noise, not real work (see §1).
3. **There is no in-flight execution.** P117 is closed (modulo the owner-gated E1
   item), the tree is clean, nothing is blocked. Your job this rotation is
   **standby-router**:
   - Hold the open E1 owner gate (§3) — re-raise it to the **OWNER** (not the
     manager) the next time the owner is reachable; do not self-authorize the
     `gh release create`/upload under any circumstance.
   - Dispatch **P118** (milestone continuation, per `.planning/ROADMAP.md` /
     `.planning/milestones/v0.15.0-phases/`) only when the owner or manager
     directs it — it is next in sequence but not pre-authorized to auto-start.
4. Route §5's new noticed-not-filed item (frontmatter/prose progress-count
   mismatch) to the appropriate ledger the next time you touch one — don't let it
   silently drop, but it's not urgent enough to interrupt standby-routing for.
5. **REPLACE this handover** (not append) at your own relief, re-verifying every
   claim live before carrying it forward — per the durable-state rule, an
   uncommitted handover didn't happen.
