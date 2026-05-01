# v0.13.0 Surprises Intake (P87 source-of-truth)

> **Append-only intake for surprises discovered during P78-P86 execution.**
> Each entry is something the discovering phase chose NOT to fix eagerly because it was massively out-of-scope. P87 drains this file.
>
> **Eager-resolution preference:** if a surprise can be closed inside its discovering phase without doubling the phase's scope (rough heuristic: < 1 hour incremental work, no new dependency introduced, no new file created outside the phase's planned set), do it there. The intake file is for items that genuinely don't fit.
>
> **Distinction from `GOOD-TO-HAVES.md`:** entries here fix something that's BROKEN, RISKY, or BLOCKING. Improvements/polish go in `GOOD-TO-HAVES.md` (P88).

## Entry format

```markdown
## YYYY-MM-DD HH:MM | discovered-by: P<N> | severity: BLOCKER|HIGH|MEDIUM|LOW

**What:** One-paragraph description of what was found.

**Why out-of-scope for P<N>:** Why eager-resolution wasn't possible (scope, time, dependency).

**Sketched resolution:** One paragraph proposing how P87 should resolve.

**STATUS:** OPEN  (← P87 updates to RESOLVED|DEFERRED|WONTFIX with rationale or commit SHA)
```

---

## Entries

## 2026-05-01 09:30 | discovered-by: P80 | severity: LOW

**What:** P80's verifier subagent (verdict GREEN) flagged that the three `agent-ux/mirror-refs-*` verifier shells underwent a shape change vs. the planned design. The plan called for `reposix init <sim-spec>` + `git fetch` + `git push` end-to-end scenarios (mirroring the P79 `reposix-attach.sh` precedent); the executor rewired all three as thin wrappers around the integration tests (`cargo test -p reposix-remote --test mirror_refs <name>`). The change is reasonable — deterministic, faster to run, no `reposix init` flakiness — but it bypasses the dark-factory `reposix init` → `git fetch` end-to-end surface that the P79-style verifier-shell pattern was specifically designed to exercise.

**Why out-of-scope for P80:** Eager-resolution at T04 time forced the shape change because the original `reposix init`-based shell hit a `fatal: could not read ref refs/reposix/main` snag (Q6 from the planner's open-questions). The executor already exceeded the verifier's TINY budget in service of getting one-shape-of-coverage shipped; reverting to the dark-factory shape would have required a deeper dive into the helper's first-push ref-advertisement contract that's already P82's territory.

**Sketched resolution:** P87 evaluates whether the `cargo test`-as-verifier shape is the new house pattern OR a one-off P80 deviation. If house: update CLAUDE.md "Subagent delegation rules" + the verifier-shell convention in `quality/PROTOCOL.md` to name `cargo test` as a sanctioned verifier kind (or document the restriction). If one-off: P86's dark-factory third-arm regression (which DOES exercise `reposix init` + `git fetch` + bus-push end-to-end against a real GH mirror) covers the same surface; P87 confirms by reading P86's verdict and either closes this entry as RESOLVED or files a P88 GOOD-TO-HAVE to reshape the P80 verifiers post-hoc.

**STATUS:** OPEN

## 2026-05-01 11:30 | discovered-by: P81-01-T04 | severity: LOW

**What:** P80's `handle_export` success branch unconditionally calls `cache.refresh_for_mirror_head()` to capture the post-write tree's synthesis-commit OID for the mirror-head ref write. `refresh_for_mirror_head` invokes `Cache::build_from()` which makes a `list_records` REST call. The P80 author left a comment ("P81 L1 migration replaces this with list_changed_since") flagging this as P81 territory, but the P81 plan body's `<must_haves>` did not include the replacement. Without addressing it, the perf regression test cannot assert "ZERO list_records calls on the hot path" — every successful push fires one.

**Why out-of-scope for the original P81 plan body:** The plan focused on the conflict-detection precheck rewrite and didn't fold in the P80 success-branch fix. Replacing `refresh_for_mirror_head` with a list_changed_since-driven equivalent would require either (a) wider Cache crate refactoring (Cache::sync also list_records during its delta tree-rebuild step at builder.rs:291) or (b) cleverness about when the post-write tree refresh is needed.

**Sketched resolution:** RESOLVED in T04 via eager-resolution per OP-8 (small, scope-bounded, no new dependency). Single edit in `crates/reposix-remote/src/main.rs` success branch: when `files_touched == 0` (no creates / updates / deletes executed), SKIP the `refresh_for_mirror_head` call. Justified because no tree change occurred, so the existing mirror-head ref still reflects the prior tree's OID. Self-healing on next non-trivial push: `refresh_for_mirror_head` fires when `files_touched > 0`. This drops the perf test's no-op-push list_records count from 1 → 0. The full L1 promise (`refresh_for_mirror_head` itself uses `list_changed_since` for the post-write tree synthesis) defers to v0.14.0 alongside the L2/L3 cache-desync hardening per architecture-sketch.md § Performance subtlety. Filed for visibility; the eager-resolution chose the smallest correct change rather than the architecturally-complete one.

**STATUS:** RESOLVED | T04 commit (eager-resolution; CLAUDE.md L1 paragraph documents the no-op skip semantics)

## 2026-05-01 11:00 | discovered-by: P81-01-T01 | severity: LOW

**What:** P81-01 plan body schedules `reposix-quality doc-alignment bind` to mint the `docs-alignment/perf-subtlety-prose-bound` row in T01 (catalog-first commit), with the test citation pointing at `crates/reposix-remote/tests/perf_l1.rs::l1_precheck_uses_list_changed_since_not_list_records`. The bind verb at `crates/reposix-quality/src/commands/doc_alignment.rs:265-270` validates that the cited test file exists on disk AND computes a `test_body_hash` against the cited fn (file + fn must both exist). Since `perf_l1.rs` is created in T04, the bind in T01 fails with `bind: --test #0 ...: test file ... does not exist`. The plan didn't account for the bind verb's filesystem validation contract.

**Why out-of-scope for P81-01-T01:** Eager-resolution: defer the docs-alignment bind from T01 to T04 (when perf_l1.rs lands). T01 still mints the perf + agent-ux rows (catalog-first integrity preserved for those two); the docs-alignment row mints in T04 alongside perf_l1.rs creation. This is a 1-line schedule change, not a scope expansion — fits OP-8 eager-resolution criteria (< 1 hour, no new dependency, no new file).

**Sketched resolution:** RESOLVED in T04 by adding the `reposix-quality doc-alignment bind` invocation to T04's action body alongside the perf_l1.rs creation. The plan body's intent is preserved (the docs-alignment row IS minted by the bind verb per Principle A); only the schedule shifts T01→T04. P88 may consider whether the bind verb should accept a `--test-pending` flag for true catalog-first contracts where the test file ships in a later commit of the same phase, but this is a tooling polish item not a P81 blocker.

**STATUS:** OPEN
