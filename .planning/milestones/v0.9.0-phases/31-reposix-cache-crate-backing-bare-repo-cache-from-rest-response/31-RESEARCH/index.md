# Phase 31: `reposix-cache` crate — backing bare-repo cache from REST responses — Research

**Researched:** 2026-04-24
**Domain:** Rust git-object construction (gix) + SQLite append-only audit + tainted-bytes type discipline
**Confidence:** HIGH (gix/trybuild/SQLite all have direct precedent in this workspace; partial-clone consumer semantics fully verified)

## Summary

This phase creates `crates/reposix-cache/` — a pure Rust library that materializes REST API responses (via the existing `BackendConnector` trait) into a real on-disk bare git repository. The cache is the substrate every later v0.9.0 phase consumes: Phase 32's `stateless-connect` handler will tunnel protocol-v2 traffic to this bare repo, Phase 33's delta sync will mutate it incrementally, Phase 34's push handler will validate against it.

The crate has three orthogonal concerns: **git-object writing** (use `gix` 0.82 — pure Rust, no `pkg-config` / `libgit2`, all the primitives we need are stable: `init_bare`, `write_blob`, `write_object`, `edit_tree`, `commit_as`, `edit_reference`); **SQLite audit + meta** (lift the established `reposix_core::audit` schema fixture and pattern with a cache-specific `op` column extension); **trust-boundary discipline** (return `Tainted<Vec<u8>>` from blob reads, lock down by `clippy::disallowed_methods` against any `reqwest::Client` construction outside `reposix_core::http::client()`). All three are well-trodden ground in this workspace — there is precedent code to imitate or lift.

**Primary recommendation:** Use `gix` 0.82, treat `reposix_core::audit::open_audit_db` as the canonical pattern (extend with a separate `cache_events` table — do NOT reuse `audit_events` because the schema columns don't fit blob materialization), structure the crate as three modules (`builder`, `audit`, `egress`), and ship the trybuild compile-fail fixture in the same wave as the type that motivates it. A single synthesis-commit-per-sync model (per CONTEXT.md) keeps wave A's git-writing surface minimal — multi-commit history is a v0.10.0 concern.

## Chapters

| Chapter | Contents | Summary |
|---------|----------|---------|
| [constraints-and-requirements.md](./constraints-and-requirements.md) | User Constraints, Phase Requirements, Architectural Responsibility Map | Locked decisions from CONTEXT.md, phase requirement table with research support, and the component-level responsibility allocation. |
| [stack.md](./stack.md) | Standard Stack, Don't Hand-Roll, Runtime State Inventory | All library choices with rationale (core + supporting + dev deps + alternatives), a table of problems to delegate to existing crates/patterns, and on-disk state inventory for new files this phase introduces. |
| [architecture-patterns.md](./architecture-patterns.md) | Architecture Patterns | System architecture diagram (Mermaid), full data-flow walkthroughs for `build_from` and `read_blob`, recommended project structure, Pattern 1–3 (gix bare repo construction, lazy blob materialization, `extensions.partialClone`), and anti-patterns to avoid. |
| [implementation-guidance.md](./implementation-guidance.md) | Common Pitfalls, Code Examples | Six named pitfalls with root causes and mitigations, plus three verbatim code examples (audit DDL SQL, trybuild compile-fail fixture, `Cache::open`/`build_from` skeleton). |
| [validation-and-threats.md](./validation-and-threats.md) | Validation Architecture, Threat Model Delta | Test framework config, requirements-to-test mapping, sampling rate, wave-0 gaps checklist, threat-model delta table, and mandatory hardening checklist. |
| [context-and-sources.md](./context-and-sources.md) | State of the Art, Assumptions Log, Open Questions for the Planner, Environment Availability, Project Constraints, Sources, Metadata | Historical context, assumption risk table, open questions with recommendations, dependency availability, project CLAUDE.md constraints applicable to this crate, full source bibliography, and confidence/freshness metadata. |
