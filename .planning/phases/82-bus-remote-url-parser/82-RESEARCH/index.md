# Phase 82: Bus remote — URL parser, prechecks, fetch dispatch — Research

**Researched:** 2026-05-01
**Domain:** git remote helper protocol; URL routing; cheap drift prechecks
**Confidence:** HIGH

## Summary

Phase 82 stands up the **read/dispatch surface** of the bus remote — URL recognition, two cheap prechecks (mirror drift via `git ls-remote`; SoT drift via `list_changed_since`), and a capability advertisement that excludes `stateless-connect`. The write fan-out (steps 4–9 of the bus algorithm) lands in P83.

The work is concentrated in `crates/reposix-remote/`: a new `bus_url.rs` module for parsing the `?mirror=<url>` query-param form, a new `bus_handler.rs` (or extension to `main.rs`'s dispatch loop) for the two prechecks + fail-fast paths, and a small refactor of the `capabilities` arm to branch on whether the remote is single-backend or bus. Single-backend `reposix::<sot-spec>` URLs continue to dispatch to `handle_export` verbatim — the bus is *additive*. PRECHECK B reuses the **same** `precheck::precheck_export_against_changed_set` already shipped in P81 (M1 narrow-deps signature pays off — cache, backend, project, runtime, parsed export are all the function takes; for P82 we call it with an empty `parsed` shape OR a coarser "any change since cursor?" wrapper). Architecture-sketch step 3 in the bus algorithm is *coarser* than P81's intersect-against-push-set semantics, because in P82 stdin has not yet been read and the push set is unknown — see Section 3 below for the resolution.

**Primary recommendation:** Ship URL parsing as a dedicated `bus_url.rs` module so the parse/format round-trip is unit-testable in isolation. Use `std::process::Command::new("git").args(["ls-remote", ...])` for PRECHECK A — gix's native `connect`-handshake API is overkill for fetching a single ref and adds a non-trivial code surface that isn't where the value is. Add a coarser `precheck_sot_drift_any` wrapper around P81's existing function that asks *"did anything change since `last_fetched_at`?"* without needing the parsed export stream; this is a 10-line wrapper, not a refactor. Reserve a single `BusRemote` struct in the dispatch path that carries `{ sot: ParsedRemote, mirror_url: String }`; the existing `parse_remote_url` continues to handle single-backend URLs unchanged.

## Chapters

| Chapter | Contents |
|---|---|
| [ch01-architecture.md](./ch01-architecture.md) | Architectural Responsibility Map · Standard Stack · Architecture Patterns · Don't Hand-Roll · Runtime State Inventory |
| [ch02-pitfalls-examples.md](./ch02-pitfalls-examples.md) | Common Pitfalls · Code Examples · State of the Art |
| [ch03-validation-security-plan.md](./ch03-validation-security-plan.md) | Assumptions Log · Open Questions · Environment Availability · Validation Architecture · Security Domain · Catalog Row Design · Test Fixture Strategy · Plan Splitting · Sources · Metadata |
