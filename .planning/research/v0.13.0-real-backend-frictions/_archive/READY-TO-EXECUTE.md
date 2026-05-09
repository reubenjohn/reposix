# READY-TO-EXECUTE — v0.13.1 real-backend frictions

**Audience:** the next session's cold-start agent (the 3rd session in the v0.13.0 → corrected-tag arc).
**Purpose:** ≤30 minutes of reading gets you to "I can start work."

## 1. Status snapshot

- **v0.13.0 graded GREEN on 2026-05-01; tag NOT pushed.** The milestone is in a "ready-to-tag" state that the owner has frozen pending corrective phases.
- **Owner ratified Path A (Option B) on 2026-05-08:** hold the tag, extend v0.13.0 with corrective phases P89–P96 — DO NOT ship as a separate v0.13.1 milestone. Reasoning: CHANGELOG fidelity > 2–4 week tag delay. See `03-synthesis/STRATEGIC-REFRAME.md` § Q1 + Q2 + Q5; ratified summary in `SESSION-2026-05-08-HANDOFF.md` § "Decisions the owner made this session".
- **Phase shape settled (P89–P96):** 6 work + 2 reservation. Detailed REQ-IDs / success criteria / dependencies in `03-synthesis/REMEDIATION-PLAN.md` § "Proposed v0.13.1 phase shape". Note: P95/P96 are a SECOND +2 reservation pair on top of P87/P88's already-drained pair — the extension grows the milestone's reservation count from 2 to 4. The intake files (`SURPRISES-INTAKE.md` / `GOOD-TO-HAVES.md`) carry P89–P94-era entries only; pre-P87 entries already drained at v0.13.0's first close.
- **`.planning/milestones/v0.13.0-phases/ROADMAP.md` currently stops at P88.** P89–P96 are NOT yet roadmapped — that authoring is this session's primary task.
- **Sibling artifact:** a `DECISIONS-NEEDED.md` is being produced in parallel; review before starting P89-01.

## 2. Reading order for cold agents (≤30 min)

1. `README.md` (this dir) — orient on the bundle. ~3 min.
2. `SESSION-2026-05-08-HANDOFF.md` — owner decisions + 7 session-emergent ideas not in synthesis. ~5 min.
3. `03-synthesis/STRATEGIC-REFRAME.md` § Q1 (recommendation only) + § Q2 (recommendation only) + § Q5 (CTO brief). **SKIP Q3, Q4, Q6** — Q3/Q4 are settled-by-implication; Q6 is meta-context not load-bearing for execution. ~8 min.
4. `03-synthesis/REMEDIATION-PLAN.md` § "Proposed v0.13.1 phase shape" (P89–P96 detailed) + § "Dependency graph". **SKIP §§ 1, 2, 6, 7** unless a specific REQ-ID needs context. ~10 min.
5. `03-synthesis/COMPLETENESS-CHECK.md` § S1 + § S2 + § S3 (the three load-bearing gaps the synthesis missed). **SKIP M/W gaps** unless surfaced by `DECISIONS-NEEDED.md`. ~5 min.
6. `03-synthesis/PATTERNS.md` § "What ties them all together" (the 1-paragraph meta-pattern). ~2 min.

You do **not** need to read the 12 phase audits, the 4 dark-factory transcripts, or PATTERNS C1–C9 unless drilling into a specific finding. The synthesis docs exist precisely to spare you that.

## 3. First concrete task

### Pre-P89 housekeeping (do this BEFORE invoking `/gsd-phase`)

Live state still says "ready-to-tag" across STATE/CHANGELOG/release-script. A cold agent who skips this step will think the next move is to push the tag, silently undoing the strategic decision. Three explicit edits required (rationale: `COMPLETENESS-CHECK-2.md` § S-3):

1. Flip `.planning/STATE.md:6` `status:` from `ready-to-tag` to a value indicating extension-in-progress (e.g. `extending-via-corrective-phases-p89-p96`).
2. Update `CHANGELOG.md:9` line from "Release status: PENDING owner tag-cut" to "Release status: PENDING P89–P96 GREEN before tag-cut".
3. Add a guard to `.planning/milestones/v0.13.0-phases/tag-v0.13.0.sh` that fails if the post-extension P96 milestone-close verdict at `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md` is missing — OR rename the script to `.disabled` until P96 GREEN.

### Then: roadmap extension

**Recommended entry point:** invoke `/gsd-phase` (CRUD on `ROADMAP.md`) to append P89–P96 entries to `.planning/milestones/v0.13.0-phases/ROADMAP.md`, sourcing goals/REQ-IDs/success-criteria/dependencies/execution-mode verbatim from `03-synthesis/REMEDIATION-PLAN.md` § "Proposed v0.13.1 phase shape".

**Rationale:** the milestone identity is settled (extend v0.13.0, not new v0.13.1 — STRATEGIC-REFRAME Q2). `/gsd-new-milestone` is therefore the wrong tool; this is roadmap-extension, not milestone-creation. The roadmap currently stops at P88, so insertion is purely append-shaped.

**Open question this session must resolve before P89-01 begins:** S2 (mid-stream litmus checkpoints between P91–P94) and S3 (vision-coverage-delta annotation on every "deferred to v0.14.0" item) from `COMPLETENESS-CHECK.md` are owner-open. Both should be folded into the roadmap entries (as success criteria on P91/P92/P93 and as a deferral-policy clause on P94/P95) BEFORE running `/gsd-discuss-phase 89`.

## 4. Quality bar for "ready to execute P89-01"

All five must be true before transitioning from roadmapping to executing:

1. `.planning/milestones/v0.13.0-phases/ROADMAP.md` contains P89–P96 entries, each citing REMEDIATION-PLAN REQ-IDs.
2. Owner has signed off on `DECISIONS-NEEDED.md` answers (Decision 1 = S2 mid-stream checkpoints, Decision 2 = S3 vision-coverage-delta on deferrals, Decision 3 = patch-vs-redesign scope tension, Decision 4 = retroactive verdict-file treatment). (S1 cross-AI arbiter is already approved per `SESSION-2026-05-08-HANDOFF.md:8`.)
3. P89's plan (`.planning/phases/89-*/89-PLAN-OVERVIEW.md`) exists and addresses S1's chicken-and-egg risk (the framework-fix phase cannot self-grade — `gsd-review` cross-AI peer review is the leading candidate; `gsd-review` skill is installed per SESSION-2026-05-08-HANDOFF § "Decisions").
4. SURPRISES-INTAKE.md / GOOD-TO-HAVES.md scaffolding for P89–P96 work-in-flight is initialized at `.planning/milestones/v0.13.0-phases/` (existing files; confirm they accept v0.13.1-era entries).
5. CHANGELOG.md `[v0.13.0]` entry edits are queued (not landed) for the eventual tag-cut — known-issues language removed once P96 milestone-close verdict GREEN.

## 5. Open decisions still needed before P89-01

The sibling subagent is producing `DECISIONS-NEEDED.md` in this directory. Owner reviews it, answers each, then this session proceeds. Do not enumerate or pre-answer the items here — `DECISIONS-NEEDED.md` is the single source of truth for what's still open.

## 6. Hard constraints the 3rd session inherits

- **Orchestrator-only / "main agent never executes"** — see CLAUDE.md OP-2 + SESSION-2026-05-08-HANDOFF item 4. P89, P90, P95, P96 are explicitly `Execution mode: top-level` in REMEDIATION-PLAN; they MUST run from the top-level Claude session, NOT inside `/gsd-execute-phase`. Code-typing belongs in subagents.
- **Two-channel rule** — subagents write FULL detail to disk; return ≤300-word TLDR to orchestrator. See SESSION-2026-05-08-HANDOFF item 5. Promote to CLAUDE.md when next editing CLAUDE.md.
- **Build memory budget** — never run more than one cargo invocation at a time; prefer per-crate over `--workspace`. See CLAUDE.md § "Build memory budget".
- **Catalog-first per phase** — every phase's FIRST commit writes catalog rows defining its GREEN contract (CLAUDE.md § "Quality Gates"). For P89/P90 specifically, COMPLETENESS-CHECK § S1 flags chicken-and-egg risk: the framework-fix rows cannot grade themselves; surface a non-framework arbiter (gsd-review or pre-committed acceptance doc) in the P89 plan.
- **Per-phase push BEFORE verifier** — `git push origin main` is part of phase close, not an end-of-session sweep (CLAUDE.md § "Push cadence").
- **Simulator is default; real-backend tests gate milestone-close** — CLAUDE.md OP-1. The whole point of v0.13.1 is operationalizing this for the milestone-close gate that v0.13.0 silently exempted.
