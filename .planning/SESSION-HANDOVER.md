# SESSION-HANDOVER.md — v0.14.0 tag: items 4a/4b/6 SHIPPED GREEN; item 5 escalated [OWNER]-pending — 2026-07-13 (→ successor #7)

For the incoming top-level workhorse (L0) — a top-level ROUTING coordinator: routes via GSD + subagents, never leaf-works. Map, not territory — detail lives in git + linked files. HEAD = live state only; delete closed/superseded entries rather than appending. The outer-loop MANAGER (herdr pane w1:p7) watches this pane and relays owner decisions; `.planning/MANAGER-HANDOVER.md` is the live owner-directive channel. Resume an agent via SendMessage, never fork (a `fork` is never a safe no-op/discard — ORCHESTRATION.md §11).

## 0. State (verify: `git rev-parse --short HEAD`, `git status --porcelain`, `gh run list --branch main --limit 8 --json headSha,status,conclusion,workflowName`)
- HEAD = origin/main = `22a7777`, tree clean. CI GREEN on `22a7777`: CI + Docs + release-plz + CodeQL all `completed/success` (ci.yml run `29273119858`; headSha verified == HEAD — no `ci-green-on-main` race). Main is genuinely green.
- **Items 4a / 4b / 6 SHIPPED GREEN** (attach-lineage fix + adf fail-closed + docs-truth), commits in order:
  `0747179` DP-2 failing repro (committed BEFORE the fix) · `eb824f3` 4a (seed `refs/reposix/origin/main` at mirror merge-base — NOT HEAD — + init-style refspec) · `d1cc811` 4b (fail-closed ADF, no empty-body PATCH escapes) · `9492718` 4b intake-close · `11ae402` item-6 docs-truth · `52be70b` 3 code-review nits filed · `22a7777` §8 real-backend arm evidence. §8 round-trip ran BOTH arms (sim §5.2 green + real-backend TokenWorld read-only forced fetch/rebase); evidence `.planning/milestones/v0.14.0-phases/evidence/item4a-real-backend-arm-2026-07-13.txt`. Tests: `crates/reposix-cli/tests/attach.rs` + `crates/reposix-remote/tests/attach_pattern_c_roundtrip_recovers.rs`.
- TokenWorld known-good = **EXACTLY 2 durable pages** (`7766017` parent + `7798785` child, child.parentId=`7766017`); byte-identical post-run (the real arm never pushed). Verify: `python3 scripts/confluence_tokenworld.py list`.
- Reality-check audit durably archived at `8e36e62` (`.planning/milestones/audits/2026-07-12-reality-check.md`, verbatim, owner-directed — done).

## 1. ACTIVE CHARTER — items 5, 7, 8 (4a/4b/6 are DONE)

**Item 5 is the tag critical path and is BLOCKED on an [OWNER]/MANAGER decision — resolve that FIRST.**

5. **Vision litmus BLOCKED-ENVIRONMENTAL (NOT a fix regression — 4a is correct).** `quality/gates/agent-ux/dark-factory/dvcs-third-arm.sh` hard-REDs at GUARD A because the litmus **mirror still carries a stale record `pages/2818063.md` for a TRASHED backend page** (the old B1-mirror-reconcile thread; evidence `.planning/milestones/v0.14.0-phases/evidence/B1-mirror-reconcile-FINDINGS-2026-07-13.md`). **Remediation = external mutation, needs MANAGER/OWNER approval (E1 — touches the durable-fixture contract):** `reposix refresh --backend confluence` + mirror push to **DROP** the stale record — **NOT restore** (restore adds a 3rd page, breaks the durable-pair contract). Recorded [OWNER]-pending in `.planning/CONSULT-DECISIONS.md`. **DO NOT execute the mirror push until the manager relays approval AND confirms the mirror target is owner-named-sanctioned.** Once approved → delegate a fixture-repair lane (verify TokenWorld stays EXACTLY 2 pages before+after, teardown) → re-run the litmus on the UNMODIFIED Pattern-C harness → confirm GREEN. This unblocks item 8's 9th probe.
7. **Re-assess p93 CREATE-recovery convergence gap under fix-first** (filed HIGH, `.planning/milestones/v0.14.0-phases/surprises-intake/part-03.md`, "Real-Confluence partial-failure RECOVERY does not converge"). Confident bounded fix → implement; architectural recovery-semantics question → keep the D2 honest-harness + documented v0.15.0 routing and flag PROMINENTLY in the READY-TO-TAG report. Independent of item 5 — can progress while awaiting the item-5 decision (single-writer permitting).
8. **§4 mechanicals (GATED on item 5 GREEN + the intake-honesty recount below):** honest `pre-release-real-backend` probe exit 0 → re-mint `quality/reports/verdicts/milestone-v0.14.0/VERDICT.md` GREEN → FRESH unbiased ratification subagent (template `quality/PROTOCOL.md` § Verifier subagent / `quality/dispatch/milestone-close-verdict.md`) → author `.planning/milestones/v0.14.0-phases/tag-v0.14.0.sh` (pattern `.../v0.13.0-phases/tag-v0.13.0.sh`) → **STOP at READY-TO-TAG** with a compact report (SHAs, artifact paths, key numbers). The tag push is the MANAGER's.
   - **KNOWN GATE RACE (HIGH, filed, NOT fixed):** `quality/gates/code/ci-green-on-main.sh` grades PASS off the single most-recent `gh run list` run WITHOUT asserting its `headSha` == just-pushed HEAD. Before trusting the FINAL tag decision, manually cross-check the graded run's `headSha` against `git rev-parse HEAD`. Finding: surprises-intake part-03 (2026-07-13 15:52).

### RAISE LIST from the item-4 fix coordinator (triage on receipt — do not drop)
- **Intake-honesty recount — BLOCKS ratification (do BEFORE item 8):** `SURPRISES-INTAKE.md` L37 claims "zero OPEN" but part-03 has ~9 OPEN → OP-9 ratification will red on the lie. Cross-part recount + fix the count first.
- **Data-integrity foot-gun (pre-GA):** quoted numeric frontmatter `id` → silent NO_ID → duplicate-on-push. Filed GTH; recommend before real-backend GA / OD-4.
- **Litmus hygiene (MEDIUM):** litmus has no teardown (persistent marker) + marker-commit mirror bloat.
- **Coordinator NOTICED:** the litmus's fetch is CONDITIONAL (fires only on push-reject) → a WEAKER attach-lineage guard than the forced sim/§8 arms; consider a forced-fetch assertion so recovery is guarded unconditionally.
- **Process/tooling:** pre-push ~101s over budget (kcov dominant); `gh run watch` PIPESTATUS trap; `verdict.py --phase` needs a numeric arg for tag-blocker items.
- **From successor #6:** the 43,492-byte `2026-07-12-reality-check.md` audit exceeds the 20,000-byte `*.md` ceiling (WARN-only; `structure/file-size-limits` waived until 2026-08-08) and is NOT in the waiver's enumerated 56-offender list → fold into `v0.15.0 GOOD-TO-HAVES` at the next waiver-renewal recount.

## 2. Constraints (unchanged)
Sim-first for code; real backends only via `REPOSIX_ALLOWED_ORIGINS`; sanctioned targets ONLY — **TokenWorld known-good = EXACTLY 2 durable pages (`7766017` + `7798785`); teardown every real-backend run; verify `python3 scripts/confluence_tokenworld.py list`** (a leaked/trashed fixture reds CI); **NO tag push ever** (manager's); never open work over a red main; ONE cargo invocation machine-wide (prefer `-p`); /tmp leaf isolation (`cd` in the SAME bash call). A `fork` is never a safe discard — end the turn instead. Relief ~100k soft / ~150k hard (absolute) → REPLACE this file, commit+push, end turn. Resume a child via SendMessage, never fork.

## 3. Ops lessons (carried)
- **`ci-green-on-main` headSha race** — cross-check `headSha` manually until the probe is fixed (§1 item 8).
- **Display-freeze false alarm** — a Claude Code survey-modal can freeze a pane's display while the background coordinator keeps running; health-check via GROUND-TRUTH git (file mtime, new commits, `ps`), not the pane view.
- **Rogue-fork tree-writer** — a dispatched `fork` inherited full context and became a live parallel tree-writer authoring a fictitious handover; forks are never a safe no-op placeholder (ORCHESTRATION.md §11).

---
History lives in git — `git log` / `git show`, not restated here.
