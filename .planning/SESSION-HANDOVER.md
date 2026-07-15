# SESSION-HANDOVER.md — /gsd-new-milestone re-anchor DE-RISKED (GO, no mutation yet); full runbook for #24 — 2026-07-14

Written by the **relief-handover-writer** on behalf of **workhorse #23** (L0 orchestrator,
pane w1:p5, herded by the manager in w1:p7), relieving to **successor #24**. This file
**REPLACES** (does not append to) the prior `SESSION-HANDOVER.md` (#22→#23's handover,
committed at `7eb2d50`).

**Read order:** this file → §1 (verify live, do not trust timestamps) → §6 (runbook) →
dip into §2/§4/§5 as needed. **Guardrails unchanged:** do NOT touch
`.planning/MANAGER-HANDOVER.md` (separate document, separate owner — the manager, pane
w1:p7). No tag push by any coordinator — the manager cuts tags, never L0. Do NOT do git
surgery (reset/rebase/reorder/amend) on `main`.

## 1. Ground truth (git) — VERIFIED LIVE this handover, do not trust staleness

Re-run before doing anything else:
```
git rev-parse --short HEAD && git status --porcelain && \
  git rev-list --left-right --count HEAD...origin/main && \
  gh run list --branch main --workflow CI --limit 3 --json headSha,status,conclusion
```
**Verified independently this handover (2026-07-14, just now):**
- Local `main` HEAD = `7eb2d50` (the #22→#23 relief-handover commit), tree **clean**
  (`git status --porcelain` empty), **EVEN** with `origin/main` (`0  0`).
- Newest `ci.yml` run on `main` = headSha `7eb2d50dc3fa798658e02b1e449c072d1cd72419`,
  `status: completed`, `conclusion: success`. The two prior rows (`14c4a42`, `03f1ac1`)
  are also `completed`/`success` — three-deep green streak.
- This handover's own commit will land on top of `7eb2d50` and get pushed immediately
  after. **By the time #24 reads this, HEAD will be that NEW sha and should again read
  EVEN — re-poll the block above, do not trust this file's timestamp.** If still ahead,
  wait for the push to land; if CI is `in_progress`, re-poll; if `failure`, stop and
  diagnose — never proceed over a red or pending main.
- No mutating commits landed this rotation (#23 did recon/investigation only — see §5).
  Commit lineage since the last handover is unchanged from what #23 inherited:
  `7eb2d50` (#22→#23 handover) ← `15e816d` (6 intake rows) ← `14c4a42` (manager #7
  handover) ← `03f1ac1` ← `db60480`.

## 2. Wave/cycle state

| Item | Artifact | State | Commit(s) |
|---|---|---|---|
| Item 1 — `/gsd-new-milestone v0.15.0 Floor` PROJECT.md re-anchor | `.planning/PROJECT.md` (+ROADMAP/REQUIREMENTS/STATE) | **NOT STARTED / NO MUTATION.** This rotation did DE-RISKING RECON ONLY, ending at a clean pre-mutation boundary. Now a **de-risked GO** — every hazard #22's handover flagged has been independently verified safe or resolved (see §5). | none |
| Item 2 — v0.15 floor milestone definition + planning | `.planning/milestones/v0.15.0-phases/` | **NOT STARTED.** Inventory of what must route in is now fully catalogued (§5) — no re-discovery needed. | none |
| Directive 2 — scratch-repo `reposix-scope-test-DELETEME` KEEP-policy doc | `docs/reference/testing-targets.md` | **NOT STARTED**, low urgency, GSD-quick scale. Confirmed absent via `grep` by #22; not re-checked this rotation (no reason to expect it changed). | none |
| Manager relay — ADR-010 mirror-fanout decision packet | (manager's input box, w1:p7) | **Manager is WAITING.** Its handover shows "Rule on the ADR-010 decision packet as soon as #23 produces it." It was **NOT produced this rotation** — it is a v0.15 PLANNING LANE that #24 produces during Item 2, not a standalone artifact from #23. #23 already relayed this status to the manager; no new relay needed until the packet exists. | none |

**No named-incident / diagnostic pending.** No litmus reopened this rotation. Read §5
before running `/gsd-new-milestone` — it's the de-risking payoff, skipping it means
re-deriving the same investigation.

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
- **Arc D is RATIFIED** (`6aa734a`, under owner delegation) — normal GSD gates apply,
  no pipeline pause in effect.

## 4. Litmus / gate / REOPEN state (carried forward, still valid)

- CI green on `7eb2d50` (§1). No gate re-run this rotation.
- **t4 row (`agent-ux/t4-conflict-rebase-ancestry-real-backend`, P0) = genuine PRODUCT
  DEFECT, scheduled as v0.15 FIX-FIRST.** Confluence `list_records`-vs-`get_record` oid
  drift, root-caused to `crates/reposix-cache/src/builder.rs:610-618` (`read_blob`).
  **Do NOT re-run this row to "retire a caveat"** — it's a scheduled fix, not an open
  caveat.
- `milestone-close-vision-litmus` FAIL under mirror non-idempotency is KNOWN — needs
  `scripts/refresh-tokenworld-mirror.sh` run FIRST, before the cadence, or it
  false-negatives on mirror lag every time.
- **Open waiver clocks (unchanged):**
  - 8 hero-number doc-alignment rows expire **2026-08-15** = HARD DEADLINE for the
    funded Q1 live MCP re-measurement — schedule EARLY in v0.15.
  - `structure/file-size-limits` waiver expires **2026-08-08** — covers oversized
    `SURPRISES-INTAKE.md` + `GOOD-TO-HAVES.md`; the progressive-disclosure split is
    **v0.17 bloat remediation**, not v0.15. Do not split it early out of turn.
  - `perf-targets` self-WAIVED until **2026-07-26**.

## 5. Mid-execution decisions + noticings (THE KEY NEW CONTENT this rotation)

**Frame:** #23 spent its rotation converting "run `/gsd-new-milestone` blindly" into a
**de-risked GO** after discovering the stock GSD workflow writes to ROOT `.planning/`
paths and runs `gsd-sdk query phases.clear --confirm`, which sounded like it could
conflict with reposix's customized per-milestone `*-phases/` layout. All four verdicts
below were **independently VERIFIED by a sonnet investigator**, not assumed:

- **Q1 — `phases.clear` = SAFE no-op today.** `.planning/phases/` holds only
  `999.2`–`999.6` = empty `.gitkeep` placeholders; the tool exempts `999.x` dirs
  (`phase-lifecycle.js:1185`, regex `/^999(?:\.|$)/`) → `cleared: 0`. The mechanism IS a
  real recursive delete where it matches (not a move — a separate `phasesArchive`
  handler exists), but **nothing currently matches**, so it's a safe no-op. Their
  content already lives as bullets in root `ROADMAP.md` § Backlog. Precedent: `beed160`/
  `7bfca56` ran the same command at v0.13.0 start safely.
- **Q2 — writing root `.planning/ROADMAP.md`/`REQUIREMENTS.md` = CORRECT** per
  `.planning/CLAUDE.md` § Milestones layout (root = live active-milestone content;
  `*-phases/` = shipped-milestone archival copies). **NOT** a freshness violation.
- **Q3 — precedent EXISTS.** `beed160` "docs: start milestone v0.13.0" wrote root
  PROJECT/ROADMAP/STATE the same way. So stock `/gsd-new-milestone` runs as-is on
  reposix without needing a custom variant.
- **⚠ CRITICAL NOTICING (would silently bite #24 if not read here):** the freshness
  gate's historical-heading regex is **hardcoded** `^## v0\.(?:8|9|10|11)\.[0-9]+`
  (`quality/gates/structure/freshness/structure_misc.py:20`) — it will **NOT**
  auto-flag the stale `## v0.13.0` / `## v0.13.2` / `## v0.14.0` H2 blocks currently
  sitting in root `.planning/ROADMAP.md`. **#24 MUST MANUALLY strip/archive those stale
  H2 sections** after writing v0.15 content, or root ROADMAP ships stale with a gate
  that stays silently green. (Worth filing a v0.17 intake row to widen that regex —
  fix-it-twice; not filed yet, noting it here so it isn't lost.)
- **Tag-cut LAUNCH-BLOCKER #7 = OBSOLETE.** The Arc D ADDENDUM says "cut two stalled
  tags (v0.13.0, v0.14.0)," but **ALL THREE tags** (v0.13.0, v0.13.1, v0.14.0) exist as
  git tags AND public releases (v0.14.0 = Latest) with archived `-phases/` dirs —
  VERIFIED live this rotation. The ADDENDUM's "archival cascade blocked on un-pushed
  tags" premise is fully resolved. v0.15 carries a **PROSE-CORRECTION + verify-
  archival-cascade-ran task**, NEVER a literal tag cut.
- **Inventory** (compiled by a sonnet lane; packet saved at scratchpad
  `/tmp/claude-1000/-home-reuben-workspace-reposix/f86e00a5-6674-4ea9-9826-23da13b8d3b4/scratchpad/v0.15-input-packet.md`
  — **WARN: EPHEMERAL/session-specific, may be gone for #24; re-derive from the intake
  files directly if absent**):
  - **10 open `SURPRISES-INTAKE.md` rows, all OPEN**, including the 5 fix-first items:
    **A** (HIGH, t4 oid-drift, `builder.rs:610-618`), **B** (MED, t4 gate error
    message), **C** (MED, mirror-refresh pre-step doc), **E** (HIGH, `run.py` `.env`
    sourcing), **F** (HIGH, `--persist` skip-downgrade guard). **There is NO "Item D"** —
    do not go looking for one.
  - **18 substantive `GTH-V15-*` rows**, including **GTH-V15-19** (LOW, `sync
    --reconcile` oid-drift audit, depends on Item A landing first). **EXCLUDE
    GTH-V15-02** — its own text says it's v0.14.0 scope, not v0.15; do not route it in.
  - Existing `.planning/milestones/v0.15.0-phases/ROADMAP.md` = **4 narrow phase
    stubs**, ALL scoped to UX-error-message/helper-hardening, **PREDATE Arc D**, zero
    t4/oid-drift coverage. Its "bring every user-facing error to Rust-compiler-grade"
    goal is legit but narrow — **SUPERSEDE with the full Arc D floor scope while
    folding that UX-error goal in as ONE lane** (do not clobber and lose its detail).

**Noticed-not-filed:** the freshness-gate historical-heading regex gap above (worth a
v0.17 intake row, not yet filed — flagged here so it routes through #24's Item 2
triage rather than being lost). Nothing else surfaced this rotation beyond the
investigation findings already folded into the verdicts above.

## 6. Precise next steps (successor #24 runbook)

1. **Re-verify §1 ground truth live first.** Confirm `main` is EVEN with
   `origin/main` and the newest CI run on the current HEAD sha is `success` before
   opening any new work. If still ahead, wait for the push to land; if CI is
   `in_progress`, re-poll; if `failure`, stop and diagnose.
2. **Run `/gsd-new-milestone v0.15.0 Floor` at TOP-LEVEL** (the Skill tool is
   top-level-only; depth-2 subagent dispatch is forbidden). **It's a GO — every hazard
   is de-risked (§5).** Drive its interactive gates from the ratified Arc D ADDENDUM
   (`.planning/milestones/audits/2026-07-12-reality-check.md` — digest, don't re-fetch
   the whole thing):
   - **SKIP research at Step 8** — the ADDENDUM IS the research.
   - **At Step 4, replace the stale truth banner** (`PROJECT.md:1-3`, currently claims
     v0.13.0/v0.13.2 as active milestones) with re-anchored content: v0.13.0/v0.13.1 +
     v0.14.0 all SHIPPED publicly, Arc D RATIFIED (`6aa734a`), next = v0.15 "Floor."
     Strip the stale "Active Milestones v0.13.0/v0.13.2" narrative
     (`PROJECT.md` ~lines 67-154).
   - **`phases.clear` at Step 6 is a safe no-op** — proceed without special handling.
   - Use **TARGETED staging** for every commit the command triggers (shared tree with
     the manager — never `git add -A`/`.`).
3. **v0.15 REQUIREMENTS scope** — route ALL open intakes + good-to-haves in (OP-8):
   - The **6 launch-blockers**: `index.md:13` category error; `filesystem-layer.md`
     false-premise rewrite; un-strand `reposix list`/`refresh` errors reusing
     `init.rs:370` exemplar; delete-or-implement phantom `reposix detach` at
     `attach.rs:135-138`; token-fixture provenance lie relabel; `twitter.md` FUSE
     framing.
   - The **t4 oid-drift FIX-FIRST lane** (Item A, `builder.rs`) + intake Items B/C/E/F
     + GTH-V15-19 + the 18 `GTH-V15-*` rows (excluding GTH-V15-02, §5).
   - **Funded Q1 live MCP re-measurement, scheduled EARLY** (HARD DEADLINE
     2026-08-15; spend ceiling ≤50 benchmark sessions on the existing subscription,
     owner-confirmed — escalate only past 50, do NOT exceed without manager GO).
   - **ADR-010 mirror-fanout DECISION PACKET lane** — produce options + tradeoffs for
     the MANAGER to rule on; **do NOT implement before the ruling** — the manager is
     actively WAITING for this packet.
   - **Docs/planning SIMPLIFICATION as a first-class roadmap goal** (P112 RAISE; git
     history is the archive; delete legacy outright, no keep-with-banners per Q5/Q7).
4. **Fold the existing v0.15 ROADMAP's UX-error-hardening goal in as one lane** (don't
   lose it — §5). After the roadmapper writes root `ROADMAP.md`, **MANUALLY strip the
   stale `## v0.13.0`/`## v0.13.2`/`## v0.14.0` H2 blocks** — the freshness gate will
   NOT catch them (§5 critical noticing).
5. **Tag-cut #7 = prose correction only, NO literal tag cut** (§5 — all three tags
   already shipped publicly).
6. **Push per cadence** + `quality/runners/run.py --cadence post-push --persist` →
   confirm `code/ci-green-on-main` (P0) green; never open next work over a
   red/pending main.
7. **Directive 2** (GSD-quick scale, low urgency): scratch-repo KEEP-policy doc into
   `docs/reference/testing-targets.md` (reset via force-push, never delete; currently
   archived, unarchive via API on first reuse).
8. **Report to the manager (w1:p7)** at each boundary above; **relieve past ~100k
   own-context tokens at the next clean wave boundary** — write+commit a fresh
   `.planning/SESSION-HANDOVER.md` (REPLACE, not append), naming successor #25,
   following this same §3 (of `ORCHESTRATION.md`) template.

**Ratchet-first sequence for reference** (canonical = Arc D ADDENDUM, digest only, do
not re-fetch): **v0.15 floor** → **v0.17 meta-milestone** (5 gate shapes:
pivot-vocabulary lint, nav-budget, hero-redundancy, framing-claim rows, persona
whole-journey rubric; + subjective-runner Task-dispatch fix unfreezing 3 WAIVED
meaning-gates; + waiver-escalation rule; + transcript retention; + bloat remediation
incl. the SURPRISES-INTAKE/GOOD-TO-HAVES progressive-disclosure split) → **v0.19**
truth purge + IA rebuild → **v0.21** benchmark honesty (re-fixture live baseline, CI
job, headline-cross-check verifier) → **v0.23** journey slices → **v0.25** launch kit
→ Show-HN. **Q3 launch gate:** Show-HN gated on a walkable REAL-BACKEND journey
(GitHub minimum), not sim-first. **Deep-survey calibration:** ~10% latent work per
pass, ~10 passes to converge, recurring deep surveys are STANDING practice.
**Q9 ceiling:** keep v0.15→v0.25 ≈ 6-milestone scale.
