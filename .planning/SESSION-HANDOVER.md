# SESSION-HANDOVER.md — hygiene lane CLOSED; new owner directives relayed — 2026-07-13

For the incoming top-level workhorse (L0) OR this session's continuation. Map, not
territory — detail lives in git + linked files. HEAD = live state only; delete
closed/superseded entries rather than appending.

## 0. Owner calibration — READ FIRST (over-ask LESS)

Decide-and-record, not gating. Pick the path the owner's model implies, log to
`.planning/CONSULT-DECISIONS.md`, proceed — owner vetoes if you misread. Reserve STOPs
for the genuinely-owner class: irreversible/destructive, external-backend mutations,
credential/spend. The outer-loop MANAGER (herdr pane **w1:p7**) is watching this pane,
relays owner decisions, and will formally send the queued charter expansion (§3) at your
next idle. `.planning/MANAGER-HANDOVER.md` is the live owner-directive channel — read it.

## 1. Current state (confirm with `git rev-parse origin/main`)

- `origin/main == 6b25e11` at hygiene-close (this handover commit sits on top). CI was
  **in_progress** at push of the split commits (`.planning`-only docs → green expected;
  the final commit here re-runs it — verify with `gh run list --branch main -L 3`).
- **No `v0.14.0` / `v0.13.0` tag exists.** Both are now MANAGER-delegated (§2).

## 2. HYGIENE LANE — COMPLETE (my charter, all pushed + gate-green)

Three progressive-disclosure splits (RAISE item 2 + the SURPRISES ceiling), full pre-push
suite **60 PASS / 0 FAIL** at push:
1. **SURPRISES-INTAKE** `v0.14.0-phases/` 43988→**3797 B** (`dc3c21a`) — split verbatim to
   `surprises-intake/part-01.md`+`part-02.md`; **0 OPEN** (all 17 entries already terminal
   from P110/P111 — the owner's "substantial drain" was already done; this was correct
   byte-relief). p111 milestone-hygiene gate PASS.
2. **STATE.md** 21137→**10846 B** (`3491d24`) — closed narrative → `.planning/STATE-history.md`;
   frontmatter + live owner-gate STOP retained (only *resolved* Blockers/Concerns moved).
3. **v0.14.0 ROADMAP.md** 37764→**13015 B** (`068bcfc`) — completed-phase bodies → sibling
   `ARCHIVE.md` (27821 B); 12 phase headings + **P112 DO-NOT-START stub kept whole**. p111 PASS.
4. GSD tracking dirs for all three landed (`6b25e11`).

## 3. NEW OWNER DIRECTIVES (from MANAGER-HANDOVER.md — newer than the prior handover)

- **BOTH v0.14.0 AND v0.13.0 tags DELEGATED TO THE MANAGER** (w1:p7). The manager runs the
  tag sequence end-to-end and routes the sub-work (9th probe → mint+ratify aggregate verdict
  → tag script → push) THROUGH the workhorse, verifying each artifact. **The workhorse stays
  HARD-BARRED from tag-push AND foreign-tree work.** So: still do NOT self-cut/push a tag.
- **`scripts/preflight-real-backends.sh` = PASS 3/3** (manager verified firsthand 2026-07-12).
  The old "no real-backend creds → 9th probe NOT-VERIFIED" claim is **STALE** — the probe can
  run for real when the manager routes it.
- **Owner mandated session SERIALIZATION** — no parallel sessions writing the shared tree
  (contention RESOLVED by decision; no worktree infra). This session confirmed the pain:
  multiple grandchildren wrote the tree concurrently before I stopped them.

### Queued charter-expansion (manager sends formally at idle — do NOT front-run past budget)
1. **Intake drain — RECONCILED, no action needed** except: several `DEFERRED-TO-v0.15.0`
   entries now live in `surprises-intake/part-01/02.md`; migrate them to the v0.15.0 planning
   surface at `/gsd-new-milestone` so they aren't orphaned in a closed milestone's archive.
2. **75% file-size early-warning gate (owner ask).** Extend `structure/file-size-limits` (or
   add a sibling row) to emit a non-blocking WARN at ≥75% of any tracked file's ceiling;
   ≥100% keeps the existing block. Mind the waiver-until-2026-08-08 (warn-now/block-later)
   interplay; fix-twice into `quality/CLAUDE.md`. Route via `/gsd-quick`. NOTE: an untracked
   `.planning/quick/260712-bgv-add-non-blocking-timing-budget-warning-*` dir already exists
   (not mine) — inspect before starting; may be a prior start.
3. **Record owner decisions** — `[OWNER]` disposition rows in `CONSULT-DECISIONS.md`
   (serialization; both-tag delegation) + fix-twice the serialization decision into
   `ORCHESTRATION.md` doctrine.
4. **Serialization cleanup of the foreign tree** — routed via workhorse *when the manager
   says so*; NOT yet (still hard-barred). See §4.

## 4. FOREIGN UNCOMMITTED WORK — DO NOT TOUCH (still, until manager routes it)

`M quality/catalogs/code.json` (+ unpersisted status flips — a pre-push validate-only run
flagged code.json + security-gates.json flips NOT persisted; leave them), untracked
`.planning/phases/21-*`, `22-*`, `scripts/demos/`, `scripts/dev/`,
`quality/reports/verifications/docs-repro/`, and a foreign `git stash`. NEVER
`git add .`/`-A`/`git clean`/`git stash *`. Explicit-path commits only.

## 5. SESSION FINDINGS / RAISEs (none dropped)

- **Subagent file-tools (Read/Write/Edit) were NON-FUNCTIONAL this session** across multiple
  `/gsd-quick` grandchildren (both planners reported it; executors fell back to sed/perl/awk
  and still produced correct commits). **L0's own tools worked fine.** Implication: prefer
  L0-inline for small file edits this session, or health-check a subagent's Write before
  delegating file-heavy work. If it persists, escalate — it undercuts the delegation doctrine.
- **`ARCHIVE.md` (27821 B) + `mhb-PLAN.md` (22408 B) exceed the 20000-char soft limit**
  (non-blocking; `structure/file-size-limits` WAIVED until 2026-08-08). Cold archives —
  acceptable; and exactly the case the queued 75% gate (§3.2) is meant to surface earlier.
- Shared-tree contention is real: I had to `TaskStop` two live grandchildren to regain
  single-writer control and finish/commit Split 3 inline.

## 6. STILL-STANDING STOPs for the workhorse

Do NOT: run `pre-release-real-backend` probe, mint the aggregate milestone-v0.14.0 verdict,
cut/push ANY tag, or touch foreign tree work — all remain the MANAGER's (§3) to route/own.

## 7. Doctrine

Full delegation/relief/cadence: `.planning/ORCHESTRATION.md` §3 + §11. Relief at ~100k own
context (hard 150k). ONE cargo invocation machine-wide. Leaf isolation: setup in a `/tmp`
clone, `cd` in the SAME bash call. Phase/quick-close needs CI-green-on-main AFTER push.
To resume a specific agent, **SendMessage it — never `fork`**.

---
History lives in git — `git log` / `git show`, not restated here.
