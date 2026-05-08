# v0.13.2 — Cross-link fidelity

> **Audience.** A cold agent (or developer) opening this folder for the first time. Read this index first; it routes you to chapters by depth of need.
> **Status.** Pre-roadmap research. Owner-approved direction; specific algorithms below are starting points, not commitments.
> **Outcome.** This folder produces enough decided design that a planning agent can write `PLAN.md` for v0.13.2. Open questions in [`08-open-questions.md`](./08-open-questions.md) must be resolved first.

## Glossary (terms that recur in this folder)

- **Fidelity assertion.** A markdown link `[text](path)` is treated as A asserting "B is what I'm framing it as." This gate grades whether that assertion is still accurate.
- **Dark-factory.** This project's working ethos: agents operate autonomously against a system that fails closed under uncertainty. Drift in fidelity is a security-of-grounding concern, not a polish concern. (Origin: `docs/research/agentic-engineering-reference.md`.)
- **Brownfield.** A repository with existing docs accumulated over time, with no historical L3 grading. The opposite of greenfield (a new repo). Brownfield support is the load-bearing requirement for this gate's adoption story.
- **BLOCKS-PLAN.** An open question that must be resolved before phase planning can produce a `PLAN.md`. Marked in [`08-open-questions.md`](./08-open-questions.md).
- **Drift-triggered L3.** L3 (LLM-graded) judgement runs only when L2 (hash-drift) detects a target change since last grade. Most pushes drift zero edges → no L3 cost.
- **GSD.** This project's planning workflow (get-shit-done). `gsd-planner`, `gsd-executor`, etc. are GSD subagents. See `CLAUDE.md` § "GSD workflow."

## What this is

A new quality gate dimension: **cross-link-fidelity**. Grades whether every markdown link `[A](B)` in the project remains a faithful *fidelity assertion* (does A's framing of B match what B currently teaches?), using a four-level scrutiny ladder from mechanical link-resolves up to LLM-graded subjective fidelity.

Catches the **unknown-unknowns** failure mode: a reader following progressive disclosure trusts the parent doc to forecast its children, and the parent doc silently lies because nobody re-graded it after the children drifted.

## Why now

Three converging forces (full argument in [`01-vision-and-problem.md`](./01-vision-and-problem.md)):

1. **Dark-factory ethos.** Drift in fidelity means an autonomous agent gets a misleading map of the codebase. Security-of-grounding concern.
2. **Subagent grading is now cheap.** Existing `subjective-rubrics.json` infrastructure already dispatches Sonnet judges; we apply the same mechanism to a new edge type.
3. **Edge count is tractable.** ~400 edges in this project; ~$50/month at full L3 cadence post-bootstrap.

## The design in 90 seconds

### Five primitives

| Primitive | What it is |
|---|---|
| **Edge** | A `(source, target, anchors)` tuple from a markdown link |
| **Edge state** | One of `UNGRADED \| GRADED \| STALE \| BROKEN` |
| **Scope** | Glob pattern set + level + tags + grading-defaults |
| **Catalog** | Machine-managed JSON; ~4 runner-readable rows at `quality/catalogs/cross-link-fidelity.json` |
| **Tracker** | Machine-managed JSON; per-edge state at `quality/state/cross-link-fidelity-tracker.json` |
| **Config** | Human-authored TOML of scopes + policy |

### Four-level scrutiny ladder

| Level | Check | Cost |
|---|---|---|
| L0 | Link target file exists | ~0 |
| L1 | `#anchor` exists in target | ~0 |
| L2 | Target's content hash unchanged since last grade | ~0 |
| L3 | LLM judge: "does the source still adequately forecast the target?" | ~$0.05 + 30s |

### Five enforcement modes (per scope, owner-controlled progression)

`warn` → `block-broken` → `block-stale` → `block-floor` → `block-newedge`

### Brownfield-friendly: ratcheting coverage

Coverage @ L3 floor is monotonic per scope. Pushes that decrease it BLOCK. Floor only goes up. UNGRADED edges are legitimate baseline — adopters climb from 0% over time.

## Decisions already made

(Excerpted; full set ADR-1 through ADR-18 in [`06-decisions-log.md`](./06-decisions-log.md). The selection below is the load-bearing subset; the full ADR list captures every decision including ones inherited unchanged from prior chats.)

- **ADR-1.** Edges, not nodes, are the grading unit.
- **ADR-3.** Scope-only configuration with a `default` scope (owner direction).
- **ADR-5.** Tracker (machine state) and config (human policy) are separate files.
- **ADR-6.** Three-flavor grading context (target ⊕ edge ⊕ source); namespaced under `cross_link_fidelity:` in frontmatter.
- **ADR-8.** Project default `max_level: L3` (dark-factory ethos).
- **ADR-11.** Edge state taxonomy is brownfield-friendly; UNGRADED is legitimate.
- **ADR-12.** Ratcheting coverage floor (monotonic, per-scope) is the enforcement mechanism.
- **ADR-15.** Bootstrap is CI-only; pre-push runs only L0+L1+L2+drift-triggered-L3.

## What still needs owner decision

The 3 BLOCKS-PLAN questions are RATIFIED (see [`08-open-questions.md`](./08-open-questions.md) § "Owner ratification"): Q2 = Rust sub-command of `reposix-quality`; Q6 = cred-hygiene regex pre-commit only; Q14 = 10-phase decomposition.

What's still open: **whether to formalize this as its own milestone** (e.g., v0.13.2) or absorb into another, and 6 deferrable design questions in [`08-open-questions.md`](./08-open-questions.md) (schema versioning discipline, edge-count scale, auto-classifier heuristics, prompt registry override, floor reset semantics, auto-fix sketch — none block phase planning).

## Routing — what to read by depth of need

### "I'm trying to understand the concept"
1. [`01-vision-and-problem.md`](./01-vision-and-problem.md) — why this gate exists.
2. [`02-architecture.md`](./02-architecture.md) — five primitives + ladder + scope model.

### "I'm formalizing this as a milestone / planning the first phase"
1. [`PROPOSED-ROADMAP.md`](./PROPOSED-ROADMAP.md) — one proposed phase shape (10 phases). NOT yet formalized; owner decides whether this becomes its own milestone or absorbs into another.
2. [`08-open-questions.md`](./08-open-questions.md) § "Owner ratification" — the 3 BLOCKS-PLAN answers.
3. [`06-decisions-log.md`](./06-decisions-log.md) — what's already decided (ADRs 1–28).
4. [`02-architecture.md`](./02-architecture.md) — conceptual model.

### "I'm writing the implementation"
1. [`03-schemas.md`](./03-schemas.md) — config + tracker + frontmatter schemas.
2. [`04-cli-and-workflow.md`](./04-cli-and-workflow.md) — CLI verb set + integration points.
3. [`examples/`](./examples/) — concrete files (config, tracker row, frontmatter, ladder walkthrough).

### "I'm onboarding a brownfield repo"
1. [`09-brownfield-and-onboarding.md`](./09-brownfield-and-onboarding.md) — full journey from day-1 to steady-state.
2. [`04-cli-and-workflow.md`](./04-cli-and-workflow.md) § "Pattern: brownfield onboarding."

### "I'm worried about edge cases"
1. [`05-edge-cases.md`](./05-edge-cases.md) — 14 named failure modes with recovery paths.

### "I'm extracting this as a standalone tool"
1. [`07-extraction-plan.md`](./07-extraction-plan.md) — boundaries to preserve, migration path, vision.
2. [`prior-art.md`](./prior-art.md) — OSS landscape; confirms L3 is greenfield.

### "I'm reviewing this design (security / soundness)"
1. [`01-vision-and-problem.md`](./01-vision-and-problem.md) — threat framing (dark-factory ethos, security-of-grounding).
2. [`05-edge-cases.md`](./05-edge-cases.md) § 12 (secret leakage) and § 11 (API outage handling).
3. [`08-open-questions.md`](./08-open-questions.md) Q6 (sanitization tier) and Q1 (schema versioning).
4. [`06-decisions-log.md`](./06-decisions-log.md) ADR-9 (fail-closed cap), ADR-15 (CI/local cost split), ADR-17 (fail-closed on outage).

## Folder layout

```
.planning/research/v0.13.2-cross-link-fidelity/
├── index.md                              # this file — entry point
├── 01-vision-and-problem.md              # why
├── 02-architecture.md                    # what (concepts)
├── 03-schemas.md                         # what (data shapes)
├── 04-cli-and-workflow.md                # how (verbs, CI integration)
├── 05-edge-cases.md                      # named failure modes
├── 06-decisions-log.md                   # ADRs 1–28
├── 07-extraction-plan.md                 # standalone-tool roadmap
├── 08-open-questions.md                  # owner ratifications + deferrable opens
├── 09-brownfield-and-onboarding.md       # adoption journey
├── prior-art.md                          # OSS landscape (research-agent output)
├── PROPOSED-ROADMAP.md                   # one proposed phase shape (10 phases) — NOT yet formalized as a milestone
└── examples/
    ├── default-config.toml               # working .cross-link-fidelity for this project
    ├── tracker-row.json                  # one tracker entry, fully populated
    ├── frontmatter.md                    # target-side grading context block
    └── ladder-walkthrough.md             # one edge traced through L0→L3
```

## What this folder does not contain

- A `PLAN.md`. That's `gsd-planner`'s output, post-discuss-phase, after the work is formalized as a milestone.
- Implementation code. That belongs in `crates/cross-link-fidelity/` (per ADR + extraction plan).
- A formalized milestone roadmap. [`PROPOSED-ROADMAP.md`](./PROPOSED-ROADMAP.md) sketches one shape (10 phases) but the owner has not yet decided whether this work becomes its own milestone, absorbs into another, or renumbers.
- Owner decisions still open. The 3 BLOCKS-PLAN questions ARE ratified ([`08-open-questions.md`](./08-open-questions.md) § "Owner ratification"); 6 deferrable questions remain.

## Cross-references back

- `CLAUDE.md` § "Quality Gates": will list `cross-link-fidelity` as a 10th dimension once the gate ships.
- `quality/PROTOCOL.md`: will document the gate's runtime contract once the gate ships.
- `quality/catalogs/cross-link-fidelity.json`: the runner-readable catalog (~4 rows; exists post-bootstrap).
- `quality/state/cross-link-fidelity-tracker.json`: the per-edge tracker (~400 rows; exists post-bootstrap; gate-internal, not runner-discovered).
- `.cross-link-fidelity`: the config file (exists post-onboarding).
