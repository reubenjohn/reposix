# 08 — Open questions for owner attention

> **Status.** These are the design questions still open. Each blocks some part of v0.13.2 phase planning. Marked by impact.
> **Read after.** All other chapters. Resolving these is the precondition to writing PLAN.md.

## How to read this chapter

Each question has:
- **Question** — what's open.
- **Why it matters** — what gets blocked or wrong if unresolved.
- **Options** — sketched alternatives with rationale.
- **Recommendation** — my best guess (overrideable).
- **Impact** — `BLOCKS-PLAN` (must resolve before phase planning), `BLOCKS-EXTRACTION` (resolve before standalone tool ships), `DEFERRABLE` (can resolve mid-execution).

---

## Q1 — Schema versioning discipline (BLOCKS-EXTRACTION)

**Question.** Do we adopt strict semver on tracker `schema_version` from v1, or relax to "format is forward-compatible until v2" with looser version mechanics?

**Why it matters.** The tracker is git-tracked. A schema bump touches every project that's ever adopted the gate. If we're sloppy at v1, extraction migration is painful.

**Options.**
- (a) Strict semver from day 1. Migration verbs (`migrate-tracker 1.0.0-to-2.0.0`) ship with v1.
- (b) "v1 stable; v2 will define migration." Cheaper now, more painful later.

**Recommendation.** (a). Cost is low; benefit accrues over years.

**Impact.** BLOCKS-EXTRACTION. (a) is the path the chapters assume.

---

## Q2 — Rust binary vs Python script for v1 (BLOCKS-PLAN)

**Question.** Does v1 ship as a sub-command of `reposix-quality` (Rust binary), or as `scripts/cross-link.py` (Python)?

**Why it matters.** Affects extraction cost. Rust → cleaner extraction (lift the crate). Python → faster v1 iteration, but Python-tool extraction has different dynamics.

**Options.**
- (a) Rust sub-command of `reposix-quality`. Mirrors `doc-alignment`. Higher iteration cost; lower extraction cost.
- (b) Python script `scripts/cross-link.py`. Faster to iterate. Extraction means rewriting in Rust later.
- (c) Hybrid: walker + tracker in Rust (durability matters); L3 dispatch in Python (iteration matters). Adds a serialization boundary between them.

**Recommendation.** (a) — Rust. Even if iteration is slower, the cost asymmetry is worth it: doc-alignment proved Rust-from-day-1 works for this shape, and extraction-as-a-crate is much cleaner.

**Impact.** BLOCKS-PLAN.

---

## Q3 — When does the regrade-update commit land? (DEFERRABLE)

**Question.** When `cross-link plan-refresh` updates the tracker after a successful grade, does it commit (a) in the same commit as the source edit that triggered the refresh, (b) in a follow-up "regrade" commit, or (c) batched into a daily "tracker update" commit?

**Why it matters.** Affects PR review workflow + git history readability.

**Options.**
- (a) Same commit as source edit. Atomic but couples doc edits with tracker churn.
- (b) Follow-up commit per push. Clean but doubles commit count.
- (c) Daily batch. Minimal noise; tracker drift visible in batches.

**Recommendation.** (b) for pre-push refreshes (immediate, atomic, visible in PR diff). (c) for cron-driven background bootstrap (batched commit at end of cron run).

**Impact.** DEFERRABLE — implementation detail; can be revisited.

---

## Q4 — Edge-count scale baseline (DEFERRABLE)

**Question.** Have we measured the realistic max edge count for v1? What's the upper bound on a typical adopter's repo?

**Why it matters.** Sizing the walker performance + cap defaults.

**Measurements (this project, 2026-05-08):** 49 markdown files, 233 md→md edges, 25 READMEs, 134 relative-parent edges, 7 intra-doc anchors. Total addressable: ~400 edges.

**Options.**
- (a) Assume small (≤500 edges). Walker is unoptimized; OK for v1.
- (b) Optimize walker for ≥10,000 edges. Premature.
- (c) Measure on a larger codebase (kubernetes/website, rust-lang/rust) before extraction.

**Recommendation.** (a) for v1. Re-measure pre-extraction.

**Impact.** DEFERRABLE.

---

## Q5 — Auto-classifier heuristics for nav-only edges (DEFERRABLE)

**Question.** What heuristics does `cross-link suggest-scopes` use to identify nav-only links? Do we ship a pre-built scope, or rely on user-config?

**Why it matters.** Onboarding friction. Auto-classification reduces config-from-scratch burden.

**Options.**
- (a) No auto-classification. User writes scopes by hand.
- (b) `suggest-scopes` proposes scopes; user reviews and commits. (chapter 09 proposes this.)
- (c) Built-in `nav-only` scope inferred at walk time without config.

**Recommendation.** (b). User retains agency; assistant reduces effort.

**Impact.** DEFERRABLE — can ship v1 without `suggest-scopes`; add in v1.1.

---

## Q6 — Secret leakage risk in grading_context (BLOCKS-PLAN)

**Question.** Beyond the cred-hygiene regex check, do we add additional sanitization to `grading_context` content before shipping it to Anthropic?

**Why it matters.** L3 ships grading_context to Anthropic. Secrets in grading_context leak.

**Options.**
- (a) cred-hygiene regex pre-commit check only. Catches known patterns; misses novel formats.
- (b) Reject any grading_context containing `${...}` env-var syntax (we already proposed this).
- (c) Length limit (e.g., ≤2KB per grading_context). Forces brevity; reduces accidental dump risk.
- (d) Allow-list characters/entities (no full URLs, no base64-looking blocks, no tokens >40 chars without spaces).

**Recommendation.** (a) + (b) + (c) for v1. (d) is heavy-handed; defer.

**Impact.** BLOCKS-PLAN — security must be baked in, not bolted on.

---

## Q7 — Subagent prompt registry: in-tree vs portable (BLOCKS-EXTRACTION)

**Question.** Does the L3 judge prompt template ship with the binary (compiled-in), or live in a config file users can override?

**Why it matters.** Adopters with different doc cultures may want to tune the prompt. But user-provided prompts are a footgun (poorly-tuned prompts → noise verdicts).

**Options.**
- (a) Compiled-in only. Users can't override.
- (b) Config-overridable, with a strict schema (variables that must appear).
- (c) Compiled-in default + per-scope override block in TOML.

**Recommendation.** (c) for v1.5. (a) for v1 — keep the prompt fixed until we have data on what variations help.

**Impact.** BLOCKS-EXTRACTION at v1.5 (the tool needs this for cross-domain adoption).

---

## Q8 — Floor reset semantics (DEFERRABLE)

**Question.** What's the exact UX for `cross-link reset-floor <scope> --reason "<text>"`?

**Why it matters.** Floor reset is a "destructive" operation (loses ratcheting history). Needs friction proportional to risk.

**Options.**
- (a) `--reason` flag required; written to tracker as audit log.
- (b) Same as (a) + writes to a separate `cross-link-floor-history.json` log.
- (c) Same as (b) + requires PR approval (`--approved-by <github-user>`).

**Recommendation.** (a) for v1. (b) for v1.1. (c) is overkill.

**Impact.** DEFERRABLE.

---

## Q9 — Cross-scope coverage aggregation (DEFERRABLE)

**Question.** Should there be a project-aggregate `coverage_floor` across scopes, or only per-scope?

**Why it matters.** Adopters may want a single "fidelity score" badge.

**Options.**
- (a) Per-scope floors only. Aggregate badge is computed at-display-time.
- (b) Per-scope + project-aggregate floor with separate ratchet.

**Recommendation.** (a). Different scopes have different cost/value profiles; cross-scope ratchet conflates them.

**Impact.** DEFERRABLE.

---

## Q10 — What does `cross-link suggest-promote` do? (DEFERRABLE)

**Question.** Should there be a verb that analyzes a scope's edge-count + churn rate + verdict-mix and suggests "this scope is stable enough for max_level=L3"?

**Why it matters.** Adopters in `warn` mode need a signal for "ready to promote."

**Options.**
- (a) Manual — owner decides.
- (b) `suggest-promote` analyzes coverage @ L3 ≥ 80% + verdict-mix dominated by PASS + churn-low → suggest promote.
- (c) Auto-promote (gate self-promotes on heuristics).

**Recommendation.** (b). (c) is too aggressive.

**Impact.** DEFERRABLE — quality-of-life feature.

---

## Q11 — Multi-target edges (DEFERRABLE)

**Question.** What about `[link](path-A.md OR path-B.md)` semantics? (Markdown doesn't have this natively, but some adopters use comma-separated lists in custom syntax.)

**Why it matters.** Edge cases.

**Recommendation.** Out of scope at v1. Markdown spec doesn't support multi-target links.

**Impact.** DEFERRABLE.

---

## Q12 — Cross-link to a section that doesn't exist yet (forward-reference) (DEFERRABLE)

**Question.** Author writes a link `[link](./planned-doc.md)` before `planned-doc.md` exists. What happens?

**Options.**
- (a) BROKEN; push BLOCKs.
- (b) Allowed if the source doc has frontmatter `cross_link_fidelity.allow_forward_refs: true`.
- (c) Allowed for any link to a path under `**/planned/**` or similar configurable.

**Recommendation.** (a). Forward references should not be encouraged; they're future-debt.

**Impact.** DEFERRABLE.

---

## Q13 — Auto-fix sketch (BLOCKS-EXTRACTION)

**Question.** At extraction time, do we add an `auto-fix` mode that proposes parent-README edits when L3 grades BLOCK?

**Why it matters.** Big quality-of-life feature for adopters.

**Options.**
- (a) Manual fix only — gate produces verdicts + rationale, human/agent fixes.
- (b) `cross-link suggest-fix <edge-id>` — emits a proposed README edit; doesn't apply.
- (c) `cross-link auto-fix <edge-id> --apply` — applies the suggestion. Risky.

**Recommendation.** (b) at v1.x — suggest, never apply. (c) deferred indefinitely.

**Impact.** BLOCKS-EXTRACTION at v1.5.

---

## Q14 — Phase decomposition for v0.13.2 (BLOCKS-PLAN)

**Question.** How many phases does v0.13.2 reasonably decompose into?

**Sketch (not committed).**

| Phase | Scope |
|---|---|
| P1 | Crate skeleton + edge model + walker + tracker JSON schema. No L3. |
| P2 | Config TOML schema + scope resolution + glob matcher. |
| P3 | L0 + L1 verifiers. Pre-commit hook integration. |
| P4 | L2 hash-drift + edge state classifier. |
| P5 | L3 judge dispatcher + Anthropic SDK integration + grading_context merge. |
| P6 | `bootstrap` + `plan-refresh` + cron CI integration. |
| P7 | `suggest-scopes` migration assistant. |
| P8 | Pre-push hook integration; phased enforcement modes; cap. |
| P9 | Reposix dogfood: bootstrap + flip default to `block-newedge`. |
| P10 | +2 reservation slots (surprises + good-to-haves) per CLAUDE.md. |

**Question for owner.** Is this the right granularity? Should some phases merge? Does v0.13.2 ship at P9 or earlier?

**Impact.** BLOCKS-PLAN.

---

## Things owner should decide before phase planning starts

Summary of BLOCKS-PLAN items:

1. **Q2** — Rust vs Python for v1? (recommend Rust)
2. **Q6** — Sanitization tier for grading_context? (recommend cred-hygiene + ${} reject + length cap)
3. **Q14** — Phase decomposition shape? (sketch above)

Plus revisit-by-extraction:

4. **Q1** — Schema versioning discipline? (recommend strict from day 1)
5. **Q7** — Prompt registry override? (recommend compiled-in for v1)
6. **Q13** — Auto-fix at extraction? (recommend suggest-only)

---

## Owner ratification

> **Status.** BLOCKS-PLAN questions resolved 2026-05-08 via `AskUserQuestion` in the post-research handoff session. Owner ratifications below are load-bearing for `/gsd-discuss-phase v0.13.2 P1`.

### Q2 ratified — **Rust sub-command of `reposix-quality`**

Owner picked the recommendation. v1 ships as a sub-command of the existing `reposix-quality` Rust binary, mirroring the `doc-alignment` pattern. Extraction-as-a-crate (per [`07-extraction-plan.md`](./07-extraction-plan.md)) is the future path; iteration cost accepted.

**Implications for phase planning.**
- New crate path: `crates/reposix-quality/` already exists; cross-link-fidelity ships as a sub-command (`reposix-quality cross-link {bind, propose-retire, ...}` or a parallel binary — to be settled in P1 PLAN.md).
- Anthropic SDK dependency lands in P5 (matches doc-alignment subjective-rubric dispatch shape).
- No Python in scope. `scripts/cross-link.py` is OUT.

### Q6 ratified — **Cred-hygiene regex only (lighter than recommendation)**

Owner picked **regex only** (declined the `${...}` reject and the 2KB cap). Pre-commit blocks credential-pattern matches; templated env-vars and accidental log dumps are NOT auto-rejected at v1.

**Implications for phase planning.**
- P5 (L3 judge + Anthropic SDK) ships with cred-hygiene regex pre-commit only — **no `${...}` reject, no 2KB length cap**.
- Two leak vectors are knowingly left open at v1: (a) authors templating secrets via `${VAR}` syntax, (b) accidental large log dumps in `grading_context`. Both rely on author discipline + post-leak audit-log forensics.
- The 2KB cap and `${...}` reject move to **DEFERRED v0.13.3 candidates** (track in `GOOD-TO-HAVES.md` once milestone is open). Re-evaluate after a real leak or a dogfood near-miss.
- Documentation for `grading_context` authors must call out the absent guards explicitly so authors don't assume tooling will catch a templated secret.

### Q14 ratified — **10-phase decomposition (full sketch)**

Owner picked the recommendation. v0.13.2 ships as 10 phases per the table:

| Phase | Scope |
|---|---|
| P1 | Crate skeleton + edge model + walker + tracker JSON schema. No L3. |
| P2 | Config TOML schema + scope resolution + glob matcher. |
| P3 | L0 + L1 verifiers. Pre-commit hook integration. |
| P4 | L2 hash-drift + edge state classifier. |
| P5 | L3 judge dispatcher + Anthropic SDK + grading_context merge. **Cred-hygiene regex pre-commit only per Q6.** |
| P6 | `bootstrap` + `plan-refresh` + cron CI integration. |
| P7 | `suggest-scopes` migration assistant. |
| P8 | Pre-push hook integration; phased enforcement modes; `max_l3_per_push` cap. |
| P9 | Reposix dogfood: bootstrap + flip default to `block-newedge`. |
| P10 | +2 reservation slots (surprises + good-to-haves) per CLAUDE.md OP-8. |

**Implications for phase planning.**
- v0.13.2 milestone is **10 phases** (P1–P10). The +2 reservation is BAKED IN as P10 splits into surprises + good-to-haves at execution time per OP-8 (or stays as a single closing phase if intake is light — to be finalized when P10 opens).
- `suggest-scopes` (P7) stays in-milestone — NOT deferred to v0.13.3.
- Phases are sequential by default; merge candidates (P3+P4 if walker work proves small) are decision-points for the planner during `/gsd-plan-phase`.
- v0.13.2 ships at P9 (dogfood + flipped default); P10 is the milestone-close ritual (drain intakes + RETROSPECTIVE + tag-script).
