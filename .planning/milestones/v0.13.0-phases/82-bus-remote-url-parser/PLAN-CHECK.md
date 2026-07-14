# Phase 82 — Plan Check

**Date:** 2026-05-01
**Reviewer:** plan-checker subagent (goal-backward verification)
**Plans audited:** `82-PLAN-OVERVIEW.md`, `82-01-PLAN.md` (~2,852 lines, 6 tasks T01–T06)
**Reference materials read:** RESEARCH.md, ROADMAP § P82, REQUIREMENTS DVCS-BUS-*, decisions.md Q3.3–Q3.6, live source (`crates/reposix-remote/src/main.rs` lines 24–32 / 48–77 / 113 / 150–172 / 186–188 / 219 / 246; `backend_dispatch.rs:75/102/219`; `precheck.rs` 302L; `cache.rs::read_last_fetched_at:443`; `mirror_refs.rs::read_mirror_synced_at:227`; `tests/`).

## Verdict: **YELLOW**

The plan is *substantively correct* — architecture lands, six-task split is honest, security mitigations T-82-01..05 are codified, every D-01..D-06 ratification reads sound against RESEARCH and the architecture-sketch. **No architectural BLOCKERs.** One HIGH (e2e test gives false GREEN signal) + three MEDIUM defects make T05/T01 fragile as written; each has a concrete small revision. Ship after one revision pass.

## Severity-classified issues

### HIGH-1 — `tests/bus_url.rs::parses_query_param_form_round_trip` gives false GREEN signal

**Location:** T05 step 5a (~L2134–2154).

**Problem:** The test asserts `!stderr.contains("parse remote url")`. This passes if parser succeeds AND ALSO passes if helper errors at any later stage (capabilities, ensure_cache, instantiate, network) with a different message. The bus URL points at port 9 (closed) — PRECHECK B will fail with `backend-unreachable: list_changed_since`. Test effectively asserts "the helper got past parse on its way to a network failure."

**Fix:** Replace negative assertion with positive — assert stdout contains `import` and `export`, proving helper reached the capabilities arm:
```rust
let stdout = String::from_utf8_lossy(&out.stdout);
assert!(stdout.contains("import") && stdout.contains("export"),
    "expected helper to advertise capabilities; stdout={stdout} stderr={stderr}");
```
Or drop as redundant with T02 unit test.

### MED-1 — `tests/common.rs` copy is a buried conditional

**Location:** T05 step 5e (~L2594–2601).

**Problem:** Verified `crates/reposix-remote/tests/common.rs` and `tests/common/mod.rs` BOTH do not exist. P81 plan-check M3 flagged this. Current plan bundles the copy as conditional inside T05 step 5e ("only if newly added"), but T05 step 5d already imports `common::{sample_issues, seed_mock, sim_backend, CacheDirGuard}` — if the executor follows step order without reading the conditional, `cargo nextest run --test bus_precheck_b` fails to compile.

**Fix:** Lift to its own ordered sub-step (T05 step 5a-prime) BEFORE step 5a, with an unconditional `cp crates/reposix-cache/tests/common/mod.rs crates/reposix-remote/tests/common.rs` + `cargo check -p reposix-remote --tests` + atomic commit "test(remote): copy tests/common.rs from reposix-cache (P81 M3 gap)".

### MED-2 — `diag`/`ensure_cache` widening missing from `<must_haves>` checklist

**Location:** T04 HARD-BLOCKs at ~L1992–2002; `<must_haves>` block ~L155–387.

**Problem:** The widening from `fn`-private to `pub(crate)` for `diag` (L80) and `ensure_cache` (L219) is mentioned only in T04 HARD-BLOCK paragraphs + commit message — NOT in the must_haves enumerable contract. If the executor authors `bus_handler.rs` first and runs `cargo check` before editing main.rs, they hit `function 'diag' is private` errors and backtrack.

**Fix:** Add to `<must_haves>` "main.rs dispatch wiring (T04)" a bullet: "`fn diag` (L80) and `fn ensure_cache` (L219) widened from `fn`-private to `pub(crate)` so the sibling `bus_handler` module can call them. `fail_push` (L246) stays private — bus_handler defines local `bus_fail_push`. Widening is purely additive."

### MED-3 — `bus_precheck_b.rs` test bodies have placeholder ellipses

**Location:** T05 step 5d (~L2492–2578).

**Problem:** Two test functions (`bus_precheck_b_emits_fetch_first_on_sot_drift`, `bus_precheck_b_passes_when_sot_stable`) have driver shape described in prose with `...` placeholders for the wiremock fixture (~30L) and synced file:// mirror fixture (~30L). Compare T05 step 5c (`bus_precheck_a.rs`) which is fully written verbatim ~150L. Executor risks ~30–60 minutes composition + silent wiremock-matcher bugs not caught at plan-check time.

**Fix:** Spell out body verbatim referencing perf_l1.rs matchers inline (~150L addition). OR split to its own task T05b ("compose bus_precheck_b.rs body from perf_l1.rs + bus_precheck_a.rs donors") so work is enumerable.

### MED-4 — `tests/bus_url.rs::rejects_plus_delimited_bus_url` driver semantics (sub-MEDIUM)

**Location:** T05 step 5a (~L2156–2170).

**Problem:** `write_stdin("\n")` is irrelevant — URL parsed in `real_main` BEFORE dispatch loop. Stderr-substring assertions DO pass correctly. However test only asserts `!out.status.success()` — any non-zero exit passes.

**Fix:** Tighten with `assert_eq!(out.status.code(), Some(1))` if a typed exit code lands in T04. Acceptable as-written; LOW-priority.

## LOW issues

- **L1:** T04 `<verify>` chains four cargo invocations via `&&`. Acceptable.
- **L2:** `splitn(...).fold(...)` style awkward. Stylistic only.
- **L3:** `bus_precheck_a_passes_when_mirror_in_sync` doesn't positively confirm helper reached PRECHECK B. Defensive; optional.

## Per-question findings (1–13) — summary

1. End-to-end goal trace — YES.
2. 7 ROADMAP SCs — 6/7 covered; SC7 correctly out-of-band.
3. Catalog-first invariant — YES. T01 mints all 6 rows status:FAIL with TINY verifier shells BEFORE implementation.
4. Q-A/Q-B/Q-C → D-01..D-06 sound — YES.
5. `parse_remote_url` unchanged — YES.
6. PRECHECK B coarser wrapper, ~10-line addition — YES.
7. Capability-arm line numbers — YES, current.
8. PRECHECK A `--` separator + reject-`-`-prefix — YES.
9. No-remote-configured `git config --get-regexp` — YES (style nit only).
10. Cargo discipline — YES, sequential per-crate.
11. Threat model integrity — YES. T-82-01..05 STRIDE register present.
12. Plan size 2,852 lines — trim candidates identified (LOW priority).
13. Open questions — most legitimate, two MEDIUM (M2, M1).

## Recommended planner revisions (one round)

1. **HIGH-1:** rewrite e2e `parses_query_param_form_round_trip` to assert positive capability-advertise signal.
2. **MED-1:** lift `tests/common.rs` copy to its own ordered sub-step T05 step 5a-prime.
3. **MED-2:** add `diag`/`ensure_cache` widening bullet to `<must_haves>`.
4. **MED-3:** spell out `bus_precheck_b.rs` body verbatim OR split to T05b.

LOW-1/2/3/MED-4 optional refinements; not required for revision pass. After revision: YELLOW → GREEN. Architecture and decisions unchanged; ~150L plan-body addition + T05 ordering rearrangement.
