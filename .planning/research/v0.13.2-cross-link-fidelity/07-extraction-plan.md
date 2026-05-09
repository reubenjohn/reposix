# 07 — Extraction plan: standalone tool roadmap

> **Read first.** [`02-architecture.md`](./02-architecture.md) and [`09-brownfield-and-onboarding.md`](./09-brownfield-and-onboarding.md).
> **Companion.** [`prior-art.md`](./prior-art.md) — confirms L3 is greenfield in OSS.

## TL;DR

The L3 layer (LLM-graded edge fidelity) has no OSS analog. lychee/markdownlint do L0–L1; nothing does L2 (drift-since-grade) on doc graphs; nothing does L3 (subjective fidelity grading). After this gate ships in-tree and proves itself for one milestone, extracting it as `cross-link-fidelity` (or similar) is a real OSS contribution, not a nice-to-have.

This chapter is the discipline that keeps that extraction cheap. Build it scoped now; preserve the abstractions that matter; resist the "build the framework first" trap.

## Phasing — in-tree first, extract later

| Phase | Where | Goal |
|---|---|---|
| **v0.13.2** | `crates/reposix-quality` + `quality/gates/cross-link-fidelity/` | Ship the gate in-tree. Use this project's edge graph as the dogfood. |
| **v0.14.x or v0.15.x** | Same | Stabilize: schema versions frozen, CLI surface stable, two milestones of in-tree adoption. |
| **Extraction trigger** | Owner decision | When the in-tree implementation has stopped changing for 2 milestones AND a second project (internal or external) wants to adopt it. |
| **v1.0 standalone** | `github.com/reubenjohn/cross-link-fidelity` (or similar) | Lift-and-shift the Rust crate; `reposix-quality` depends on it as a library. |

**The "build the framework first" trap.** It's tempting to design the extraction-ready API before shipping anything. Don't. Ship the gate in-tree, learn which abstractions are load-bearing, then extract. The abstractions that survive contact with one real project will survive abstracting; the abstractions invented in advance won't.

## What stays portable (and what doesn't)

The abstractions below were chosen for portability from day 1. None of them couple to reposix specifically.

| Component | Portable? | Notes |
|---|---|---|
| Edge primitive | Yes | `(source, target, anchors)` is universal. No reposix in the schema. |
| Scrutiny ladder (L0–L3) | Yes | Levels are about doc-graph grading, not project-specific. |
| Scope-only config | Yes | Glob patterns + scope semantics are language-agnostic. |
| Edge state taxonomy | Yes | UNGRADED/GRADED/STALE/BROKEN is universal. |
| Ratcheting coverage | Yes | Pure metric mechanic. |
| Phased enforcement modes | Yes | Pure policy mechanic. |
| Tracker JSON schema | Yes | No reposix-specific fields. Schema-versioned. |
| Frontmatter `cross_link_fidelity:` block | Yes | Generic; namespace is the tool's name. |
| L3 judge prompt template | Mostly | Templated; users provide their own grading_context. The template skeleton ships with the tool. |
| `reposix-quality` CLI sub-command structure | NO | This is the in-tree harness. Standalone tool ships its own binary. |
| `quality/catalogs/`, `quality/gates/<dim>/` directory layout | NO | reposix-specific Quality Gates framework. Standalone tool's outputs live wherever the user configures. |
| `cred-hygiene` integration on grading_context | Yes via interface | Standalone tool ships a default secret-scanner; users can plug their own. |
| Anthropic SDK direct calls | NO | Standalone tool needs LLM-provider abstraction (Anthropic / OpenAI / local). v1 ships Anthropic only; abstraction added at v1.5 if requested. |

## Boundaries to preserve in-tree

These boundaries are the load-bearing decisions that make extraction cheap. Ignoring them means a hard rewrite later.

### Boundary 1: edge data model has no reposix terms

The tracker JSON, the config TOML, the frontmatter schema — none of them use words like "reposix," "DVCS," "cache," etc. The data model is language-agnostic and tool-agnostic.

✅ Good: `cross_link_fidelity.grading_context: "this doc teaches the architecture of X"`
❌ Bad: `cross_link_fidelity.reposix_phase: "v0.13.2"`

### Boundary 2: the Rust crate has no `reposix-` dependency in its public API

The Rust crate (call it `cross-link-fidelity` for now) lives at `crates/cross-link-fidelity/` and depends on:

- `gix` (or just `git2`) — for tracker discovery and PR-aware operations.
- `serde_yaml`, `serde_json`, `toml` — for schemas.
- `anthropic` SDK — for L3 dispatch.
- `pulldown-cmark` or `markdown` — for AST walking.

What it MUST NOT depend on:

- `reposix-core`, `reposix-cache`, `reposix-cli`, etc. The data model is its own.
- `reposix-quality` — that's the consumer, not a dependency.

The in-tree harness (`reposix-quality`) wraps `cross-link-fidelity` with reposix-specific glue: catalog row registration, badge path conventions, integration with `quality/gates/<dim>/walk.sh`. The wrapper is thin.

### Boundary 3: tests live in the crate, not in `crates/reposix-cli/tests/`

Cross-link-fidelity tests use synthetic doc graphs as fixtures, not the reposix doc tree. This makes the crate testable in isolation; reposix uses it via integration tests in `reposix-quality`.

### Boundary 4: schema versioning is from day 1

Three schemas (config, tracker, frontmatter), each with a version field. Migration verbs exist from v1. Even if v1 only has version 1, the discipline of "introduce changes via migration" is set.

This matters because the tracker is git-tracked: a schema bump affects every project that's ever adopted the gate. Migrations need to be lossless and human-reviewable.

### Boundary 5: LLM dispatch reuses the existing rubric infrastructure

Per ADR-26, L3 dispatch reuses `.claude/skills/reposix-quality-review/lib/persist_artifact.py` (verdict persistence) + the Path A/B pattern from `dispatch_inline_subagent.sh` (in-session via Claude Code Task tool, with subprocess-stub fallback). v1 does NOT introduce a new `JudgeProvider` trait or a parallel SDK wrapper.

Multi-vendor abstraction (OpenAI, local Ollama, etc.) is a property of the rubric infrastructure, not cross-link. If the rubric dispatcher gains multi-vendor support post-extraction, cross-link inherits it for free. This boundary is what keeps the gate small enough to ship as a clean crate.

## Migration path: in-tree → standalone

When extraction trigger fires, the migration is mostly mechanical:

1. **Move the crate.** `git filter-branch` or `git subtree split` `crates/cross-link-fidelity/` to a new repo. Preserve history.
2. **Publish to crates.io.** `cargo publish` with v1.0. Pre-1.0 versions stay yanked or as v0.x in reposix's git history.
3. **Reposix depends on it.** `crates/reposix-quality/Cargo.toml` adds `cross-link-fidelity = "1.0"`. Drop the in-tree path dependency.
4. **Standalone repo gets its own README, docs site, examples.** Mostly lift the chapters in this folder, repurposed for an external audience.
5. **Schema endpoints move.** Tracker JSON `$schema` URL points at `schemas.cross-link-fidelity.dev/...` (or wherever the standalone tool hosts its schema).

What does NOT migrate:

- The `quality/gates/cross-link-fidelity/walk.sh` shell wrapper — reposix's internal harness.
- The `[scopes.default]` config that's project-specific (anchor-readme glob patterns, etc).
- The catalog row registration in `quality/catalogs/cross-link-fidelity.json` (the runner-readable catalog with ~4 rows; the per-edge tracker at `quality/state/cross-link-fidelity-tracker.json` is gate-internal — see [03-schemas.md](./03-schemas.md) § "Four schemas, four owners" or ADR-25).

## What this gate could become — vision

Beyond reposix:

- **Adopters who care about doc-graph fidelity:** OSS projects with progressive-disclosure docs (mkdocs, docusaurus, jekyll), corporate wikis with parent-page conventions, agent-tooling projects whose docs are LLM-context.
- **Composability:** plug into pre-commit-hook frameworks, GitHub Actions marketplace, Codecov-style PR comments.
- **L3 as a service:** offer hosted L3 grading for adopters who don't want to manage their own Anthropic/OpenAI billing. Out of scope at v1.
- **Multi-format:** beyond markdown — RST, AsciiDoc, Notion exports, etc. Out of scope at v1.

## What this gate is NOT trying to be

- **A general LLM-eval framework.** DeepEval / G-Eval do that. We're a doc-graph-shaped slice.
- **A markdown linter.** markdownlint / vale do that. We're orthogonal.
- **A documentation generator.** mkdocs / docusaurus / sphinx. We grade fidelity, we don't generate.
- **A monorepo / cross-repo system.** Each repo owns its own graph. Cross-repo edges are L0 only.

Keeping the scope narrow at v1 is the path to a useful v1. The "knowledge graph fidelity" framing is intoxicating; resist building the bigger thing before the smaller thing ships.

## Naming the standalone tool

Owner decision; sketched options:

- `cross-link-fidelity` — descriptive, exact.
- `linkgrade` — short, brandable.
- `mdtruth` — playful, abstract.
- `forecast-check` — domain-evocative.

Recommend deferring naming until the in-tree implementation has stabilized; bikeshedding now is premature.
