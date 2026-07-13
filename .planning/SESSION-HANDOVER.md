# SESSION-HANDOVER.md — v0.14.0 tag HALTED: item 5 executed → litmus RED (real coherence bug); [OWNER/MANAGER]-pending — 2026-07-13 (→ successor #9)

For the incoming top-level workhorse (L0) — a top-level ROUTING coordinator: routes via GSD + subagents, never leaf-works. Map, not territory — detail lives in git + linked files. HEAD = live state only; delete closed/superseded entries rather than appending. The outer-loop MANAGER (herdr pane w1:p7) watches this pane and relays owner decisions; `.planning/MANAGER-HANDOVER.md` is the live owner-directive channel. Resume an agent via SendMessage, never fork (ORCHESTRATION.md §11).

## 0. State (verify: `git rev-parse --short HEAD`, `git status --porcelain`, `gh run list --branch main --limit 8 --json headSha,status,conclusion,workflowName`)
- HEAD ≈ this handover commit on origin/main, tree clean — **verify live.** Main CI not red at pause.
- **The v0.14.0 tag is BLOCKED.** Item 5 RESTORE was EXECUTED and the gating litmus came back **RED — a real coherence bug** (NOT mirror lag). This is the RED-honest halt branch of guardrail 4; fix-first applies; do NOT paper over, do NOT tag. **The next decision is the MANAGER's/OWNER's** (fix-vs-waive-vs-rescope + tag-timeline), then a fresh successor executes it.
- TokenWorld backend = **EXACTLY 3 current pages** (`2818063` current-v7 [RESTORED], `7766017`+`7798785` PROTECTED). Verify: `python3 scripts/confluence_tokenworld.py list`. PLUS one orphan test page **`9994241`** left by p93 teardown (Confluence 500 on DELETE) — cleanup owed: `python3 scripts/confluence_tokenworld.py delete 9994241`.
- Real-backend creds: env vars UNSET in the shell but **`.env` IS present** (runner sources it) — the 9th probe IS runnable. Do NOT read `.env`.
- Full RED evidence + DP/valve reasoning: `.planning/CONSULT-DECISIONS.md` § "EXECUTED 2026-07-13 (successor #8) — RESTORE done, litmus RED".

## 1. ACTIVE CHARTER — item 5 is RED-BLOCKED → the ball is with the MANAGER. Successor: execute the manager's ruling, then item 8 → READY-TO-TAG. NEVER push the tag.

### Item 5 — EXECUTED → RED (real coherence bug). TAG HALTED. [OWNER/MANAGER]-pending.
The RESTORE ran exactly per its 6 guardrails (opus, sole tree-writer). Restore succeeded (3-page end state, protected pair intact, guardrails 1/2/3 met). The 9th probe (`run.py --cadence pre-release-real-backend --persist`) came back **exit 1 / 2 PASS · 4 FAIL**:
- **litmus FAIL** — v1-vs-v7 mismatch; the ONE documented recovery (`git pull --rebase reposix main`, `litmus-flow.sh:100-111`) collapsed to a content conflict in `pages/2818063.md` (fail `:108`).
- **p93 FAIL (exit 101)** — panic `crates/reposix-cli/tests/agent_flow_real.rs:925`, recovery push `some-actions-failed`. **p93 was GREEN at 15:28 today, RED at 19:06 — trigger NOT pinned down.**
- **Prime suspects (code-reading, unconfirmed):** (a) `adf_to_markdown` fails for ALL 3 pages (`root node type must be "doc", got ""`) → fail-closed sentinel bodies → guaranteed rebase conflict (hits protected pair too = broad translate bug); (b) delta-sync cursor advanced past a concurrent write, `list_changed_since` dropped `2818063`.

**This is E3 (release-scope/timeline tradeoff) — the manager/owner decides. Successor #9 must NOT unilaterally launch a fix campaign; wait for the manager ruling.** Recommendation already relayed to the manager (verbatim in §CONSULT EXECUTED subsection): **authorize a bounded read-only DIAGNOSTIC lane FIRST** (is the trigger `2818063`'s empty-ADF body or a genuine `reposix-confluence` regression? explain green@15:28→red@19:06) → then fix → re-run item 5 → item 8. Do NOT waive — this is a real UPDATE-recovery coherence failure, distinct from the WAIVED CREATE-recovery RBF-LR-03 gap.

**Guardrails 5 & 6 still TODO** (the lane stopped at the RED branch): (5) eager-fix the fixture-shape doc-truth in `docs/reference/testing-targets.md` (+ any doc asserting "exactly 2 / 2 durable pages"; correct shape = 2 protected never-deleted + 1 sacrificial editable `2818063`) — this is safe to do NOW, independent of the manager ruling; (6) file the broken mirror-sync Action (run `25223195636`, cargo-binstall P84 gap) to SURPRISES-INTAKE.

### Item 7 — RESOLVED: DEFER to v0.15.0 (unchanged). Carry the verbatim WAIVED flag into the READY-TO-TAG report:
"p93 is GREEN as an UPDATE-recovery proof against live TokenWorld (`1c424d7`). It does NOT cover CREATE-recovery: a partial-fail whose landed action was a create against an id-assigning backend genuinely does not converge. This is the owner-signed WAIVED known limitation of ADR-010 §3 / RBF-LR-03, hand-recoverable, routed to v0.15.0." — NOTE this is SEPARATE from item 5's RED (which is an UPDATE-recovery failure, not the CREATE gap). Item-8 doc TODOs still open: re-STATUS `part-03.md:59-61` (stale "active p93 blocker", overtaken by `1c424d7`); file a v0.15.0 GTH for slug→id create-reconciliation; LOW: title-sweep 2 pre-rewrite "p93 smoke A" orphans in TokenWorld.

### Item 8 — §4 mechanicals. GATED on item 5 GREEN (NOT yet — blocked on the manager ruling + a real fix). STOP at READY-TO-TAG.
Sequence once item 5 is GREEN: OP-9 triage (below) → honest `pre-release-real-backend` probe exit 0 (creds+`--persist`; runs litmus + p93) → re-mint `quality/reports/verdicts/milestone-v0.14.0/VERDICT.md` GREEN → FRESH unbiased ratification subagent (`quality/dispatch/milestone-close-verdict.md`) → author `.planning/milestones/v0.14.0-phases/tag-v0.14.0.sh` (pattern `.../v0.13.0-phases/tag-v0.13.0.sh`) → STOP at READY-TO-TAG (compact report to MANAGER w1:p7: SHAs, artifact paths, probe exit code, TokenWorld end state).
- **KNOWN GATE RACE (HIGH, filed, NOT fixed):** `ci-green-on-main.sh` grades PASS off newest `gh run list` WITHOUT asserting `headSha`==pushed HEAD. Manually cross-check before trusting the tag decision.
- **OP-9 triage — DONE this session (read-only digest, ready to apply):** intake = 3 files under `.planning/milestones/v0.14.0-phases/surprises-intake/`; **all 8 OPEN items live in `part-03.md`** (recount `466b6a6`). **All 8 route to v0.15.0 — none block the tag.** Dispositions: L82 B1 attach-recovery (HIGH → v0.15.0-defer; ground-truth intake says **NOT** resolved by item-4a `eb824f3`, still entangled — do NOT force-close); L210 ci-green race (HIGH → v0.15.0 owner-gated fix, mitigated by manual headSha cross-check); L134/149/164/186/198/241 (MED/LOW harness/process → v0.15.0-defer/GTH). `RETROSPECTIVE.md` exists with a STARTED-but-unfinished v0.14.0 section (What Was Built/Worked/Inefficient/Patterns/Key Lessons) — **must finalize the OP-9 distillation of the 8 intakes + this RED run before ratification** or the ratifier grades RED.

## 2. Constraints (unchanged)
Sim-first for code; real backends only via `REPOSIX_ALLOWED_ORIGINS`; sanctioned targets ONLY — **TokenWorld fixture shape (CORRECTED): 2 PROTECTED pages never deleted (`7766017`+`7798785`) + 1 SACRIFICIAL EDITABLE (`2818063`, current or trashed between runs)** (verify `python3 scripts/confluence_tokenworld.py list`); **NO tag push ever** (manager's); never open work over a red main; ONE cargo invocation machine-wide (prefer `-p`); /tmp leaf isolation (`cd` in the SAME bash call); single-writer (one tree-mutating agent at a time; read-only agents may parallelize). A `fork` is never a safe discard — end the turn instead. Relief ~100k soft / ~150k hard (absolute) → REPLACE this file, commit+push, end turn. Resume a child via SendMessage, never fork.

## 3. Ops lessons (carried) + hygiene debt
- **The `restore`/`delete` tool needs an explicit id** (`restore 2818063`, not bare `restore` → no-op exit 2). The manager relay's bare "restore" wording was imprecise.
- **`ci-green-on-main` headSha race** — cross-check `headSha` manually until fixed.
- **CONSULT-DECISIONS.md is ~35k chars (WARN > 20000):** prune superseded entries (git is the archive) — the DROP/HALT item-5 chain, implemented `[SELF]` D4/D5/D6, and the closed B1 restore/reconcile entries are all safe to delete at the next clean moment.
- **Catalog honesty caveat committed this session:** `agent-ux.json` shows honest RED grades incl. suspected mis-grades `t4`+`github-front-door` (owner_hints "verifier not implemented" → should read NOT-VERIFIED). NOT hand-corrected (that would paper over a runner-grading bug); the fix lane owns both.
- **Display-freeze false alarm** — health-check via GROUND-TRUTH git, not the pane view. A dispatched `fork` becomes a live parallel tree-writer — never a safe no-op.

---
History lives in git — `git log` / `git show`, not restated here.
