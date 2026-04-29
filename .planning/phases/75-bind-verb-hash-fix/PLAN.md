---
phase: 75-bind-verb-hash-fix
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/reposix-quality/src/commands/doc_alignment.rs
  - crates/reposix-quality/tests/walk.rs
  - quality/catalogs/doc-alignment.json
  - CLAUDE.md
  - quality/reports/verdicts/p75/walk-after-fix.txt
  - .planning/phases/75-bind-verb-hash-fix/SUMMARY.md
  - .planning/milestones/v0.13.0-phases/CARRY-FORWARD.md
autonomous: true
requirements:
  - BIND-VERB-FIX-01
must_haves:
  truths:
    - "On Source::Single -> Source::Multi conversion, source_hash is preserved as the FIRST source's hash (not overwritten with the newly cited source's hash)."
    - "On Source::Multi append, source_hash is preserved unchanged (it remains the first source's hash)."
    - "On Source::Single -> Source::Single rebind (same source citation, prose may have changed), source_hash IS updated to the freshly-computed hash of the (single) cited source's current bytes."
    - "Walker integration test asserts a Multi([A,B]) row with both files byte-stable stays BOUND (no false STALE_DOCS_DRIFT)."
    - "Walker integration test asserts a Multi([A,B]) row whose A drifts fires STALE_DOCS_DRIFT (correct detection, path-(a) limitation: walker only watches first source)."
    - "Walker integration test asserts a Single(A) row that drifted to STALE_DOCS_DRIFT then re-bound (with the same source citation) heals to BOUND on the next walk."
    - "Live 388-row catalog walked post-fix produces zero net new STALE_DOCS_DRIFT transitions; the linkedin Single-row STALE state from P74 heals to BOUND after a fresh bind."
    - "CLAUDE.md gains a P75 H3 subsection (<=20 lines) under v0.12.1 in-flight that names the bind-verb hash semantics + the path-(a) tradeoff (walker watches first source only)."
    - "v0.13.0 carry-forward MULTI-SOURCE-WATCH-01 is filed as the deferred follow-up for path-(b)."
  artifacts:
    - path: "crates/reposix-quality/src/commands/doc_alignment.rs"
      provides: "Refined verbs::bind logic preserving source_hash on Multi paths"
      contains: "source_hash"
    - path: "crates/reposix-quality/tests/walk.rs"
      provides: "Three new regression tests covering Multi-stable, Multi-first-drift, Single-rebind-heal"
      contains: "fn walk_multi_source_stable_no_false_drift"
    - path: "quality/reports/verdicts/p75/walk-after-fix.txt"
      provides: "Live-catalog smoke evidence (stdout + stderr from `target/release/reposix-quality doc-alignment walk`)"
    - path: "CLAUDE.md"
      provides: "P75 H3 subsection (<=20 lines) under v0.12.1 in-flight"
      contains: "P75"
    - path: ".planning/milestones/v0.13.0-phases/CARRY-FORWARD.md"
      provides: "MULTI-SOURCE-WATCH-01 entry filed for v0.13.0"
      contains: "MULTI-SOURCE-WATCH-01"
    - path: ".planning/phases/75-bind-verb-hash-fix/SUMMARY.md"
      provides: "Phase 75 SUMMARY with verifier verdict link, commits, evidence"
  key_links:
    - from: "crates/reposix-quality/src/commands/doc_alignment.rs::verbs::bind"
      to: "Row::source_hash"
      via: "conditional assignment based on result kind (Single vs Multi-from-Single vs Multi-append)"
      pattern: "row\\.source_hash = "
    - from: "crates/reposix-quality/tests/walk.rs"
      to: "verbs::bind + verbs::walk"
      via: "assert_cmd CLI invocations + JSON post-condition asserts on source_hash and last_verdict"
      pattern: "Source::Multi|source_hash"
    - from: "quality/catalogs/doc-alignment.json"
      to: "live walker run via target/release/reposix-quality"
      via: "post-fix `doc-alignment walk` invocation; linkedin row STALE_DOCS_DRIFT -> BOUND after a single fresh bind"
      pattern: "STALE_DOCS_DRIFT|BOUND"
---

<objective>
Fix the `verbs::bind` hash-overwrite bug surfaced repeatedly during the v0.12.1 cluster sweeps (HANDOVER §4) and broadened by the P74 finding logged in `SURPRISES-INTAKE.md` (`docs/social/linkedin.md` Source::Single row stuck in STALE_DOCS_DRIFT).

Purpose: stop false `STALE_DOCS_DRIFT` from firing on multi-source rows after a sweep adds a citation, AND clarify the rebind-heal path for Single rows whose prose drifted. Path (a) per CONTEXT.md D-01 (preserve first-source semantics in `bind`); the walker is NOT touched beyond its existing first-source compare. Path (b) (full multi-source hash array + walker iteration) is filed as v0.13.0 carry-forward `MULTI-SOURCE-WATCH-01`.

Output:
- One Rust-source change in `crates/reposix-quality/src/commands/doc_alignment.rs::verbs::bind` (+ inline doc comment naming the invariant).
- Three new walker integration tests in `crates/reposix-quality/tests/walk.rs` (Multi-stable, Multi-first-drift, Single-rebind-heal).
- Live-catalog smoke evidence under `quality/reports/verdicts/p75/walk-after-fix.txt`.
- A `<=20`-line P75 H3 subsection in CLAUDE.md and a v0.13.0 carry-forward entry.
- Phase SUMMARY.md.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/STATE.md
@.planning/HANDOVER-v0.12.1.md
@.planning/phases/75-bind-verb-hash-fix/CONTEXT.md
@.planning/milestones/v0.12.1-phases/ROADMAP.md
@.planning/milestones/v0.12.1-phases/REQUIREMENTS.md
@.planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md
@CLAUDE.md
@quality/PROTOCOL.md
@crates/reposix-quality/src/commands/doc_alignment.rs
@crates/reposix-quality/src/catalog.rs
@crates/reposix-quality/tests/walk.rs
@crates/reposix-quality/tests/bind_validation.rs

<interfaces>
<!-- Extracted from crates/reposix-quality/src/catalog.rs and src/commands/doc_alignment.rs. -->
<!-- Executor uses these directly; no codebase scavenger hunt required. -->

From `crates/reposix-quality/src/catalog.rs`:
```rust
pub enum Source {
    Single(SourceCite),
    Multi(Vec<SourceCite>),
}

impl Source {
    /// Returns the source citation(s) as a Vec; Multi preserves order, first element first.
    pub fn as_slice(&self) -> Vec<SourceCite> { /* ... */ }
}

pub struct SourceCite {
    pub file: String,
    pub line_start: usize,
    pub line_end: usize,
}

pub struct Row {
    pub id: String,
    pub claim: String,
    pub source: Source,
    pub source_hash: Option<String>,        // <-- the bug site: walker reads this against source.as_slice()[0]
    pub tests: Vec<String>,
    pub test_body_hashes: Vec<String>,
    pub rationale: Option<String>,
    pub last_verdict: RowState,             // BOUND | STALE_DOCS_DRIFT | STALE_TEST_DRIFT | STALE_TEST_GONE | MISSING_TEST | TEST_MISALIGNED | RETIRE_PROPOSED | RETIRE_CONFIRMED
    pub next_action: NextAction,
    /* timestamps elided */
}
```

From `crates/reposix-quality/src/commands/doc_alignment.rs::verbs::bind` (current buggy form, line ~285-303):
```rust
if let Some(row) = cat.row_mut(row_id) {
    let mut sources = row.source.as_slice();
    let already_present = sources.iter().any(|c| {
        c.file == new_source.file && c.line_start == new_source.line_start && c.line_end == new_source.line_end
    });
    if !already_present {
        sources.push(new_source.clone());
    }
    row.source = if sources.len() == 1 {
        Source::Single(sources.into_iter().next().expect("len==1"))
    } else {
        Source::Multi(sources)
    };
    row.claim = claim.to_string();
    row.source_hash = Some(src_hash);   // <-- BUG: overwrites unconditionally; src_hash is hash of the INVOKED source,
                                        //     which on Single->Multi conversion is NOT the first element's hash.
    /* ... tests, rationale, verdict, timestamps ... */
}
```

Walker compare site (line ~836-850):
```rust
let cite = row.source.as_slice().into_iter().next();   // <-- only the FIRST source is checked.
let source_drift: Option<bool> = match (cite, row.source_hash.as_ref()) {
    (Some(c), Some(stored)) => {
        let p = PathBuf::from(&c.file);
        if p.exists() {
            match hash::source_hash(&p, c.line_start, c.line_end) {
                Ok(now) => Some(&now != stored),
                Err(_) => Some(true),
            }
        } else { Some(true) }
    }
    _ => None,
};
```
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Inspect and document current bind/walker hash semantics (no commits)</name>
  <files>crates/reposix-quality/src/commands/doc_alignment.rs (read-only)</files>
  <action>
    Read `crates/reposix-quality/src/commands/doc_alignment.rs::verbs::bind` (lines 224-333) and `verbs::walk` (lines 802-end) and confirm in your own words (a) the buggy assignment site, (b) which arm of the walker reads `source_hash`, (c) which arm of the walker reads `test_body_hashes`. Echo the line numbers + your understanding into the executor's chat output (NOT a committed file).

    Also confirm the P74 finding's mechanic: a `Single(A)` row whose A's range bytes drifted enters `STALE_DOCS_DRIFT` after `walk` (correct). The walker's docstring (line 802-804) says it NEVER mutates stored hashes -- so the row stays STALE until a fresh `bind` runs and refreshes `source_hash`. Confirm whether re-running `bind` with the same `Source::Single(A)` (already_present == true, sources len stays 1) DOES recompute `src_hash` from the current file bytes and write it to `row.source_hash`. If yes, the heal path already works under current code -- the linkedin row's "stayed STALE on second walk" is expected (walks don't heal; binds do). If no, identify the gap.

    Do NOT commit. Output your findings as the task's verify text. This task EXISTS to prevent the executor from blindly applying the fix; the next maintainer's reading of the code matters more than the patch.
  </action>
  <verify>
    <automated>echo "Task 1 is read-only; verification = the executor's findings appear in chat output naming exact line numbers for (a) bug site, (b) walker first-source compare, (c) walker test-hash compare." </automated>
  </verify>
  <done>Executor has read both functions, named the line numbers, and articulated the three semantic cases that the fix must cover (Single result, Single->Multi promotion, Multi append).</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Add three failing regression tests in walk.rs (TDD RED)</name>
  <files>crates/reposix-quality/tests/walk.rs</files>
  <behavior>
    Three new tests covering the three cases the fix must satisfy. Two are expected to FAIL pre-fix (the bug is real); one MAY pass pre-fix (depending on whether the heal path is already correct -- Task 1 told us). All three MUST pass post-fix.

    Test A -- `walk_multi_source_stable_no_false_drift` (CASE: Single->Multi conversion, both files byte-stable):
      1. Seed an empty catalog (reuse existing `seed_catalog` helper).
      2. Write doc_a.md and doc_b.md to the temp dir, both with stable two-line content.
      3. Write a test_file.rs with a parseable `fn alpha()`.
      4. Bind row `row/multi` to `doc_a:1-2` -> row becomes `Source::Single(doc_a)`, source_hash == hash(doc_a:1-2).
      5. Re-bind row `row/multi` with `doc_b:1-2` -> row becomes `Source::Multi([doc_a, doc_b])`. ASSERT post-bind row.source_hash == the value captured after step 4 (i.e. hash(doc_a:1-2), NOT hash(doc_b:1-2)).
      6. Run `walk`; ASSERT exit success AND row last_verdict == "BOUND" AND no STALE_DOCS_DRIFT in stderr.

    Test B -- `walk_multi_source_first_drift_fires_stale` (CASE: Multi row, A drifts -> STALE):
      1. Seed catalog + bind row `row/multi-drift-a` to `doc_a:1-2`, then re-bind with `doc_b:1-2` (same shape as Test A through step 5).
      2. Edit doc_a.md (replace bytes in lines 1-2 with different content).
      3. Run `walk`; ASSERT exit failure AND stderr contains "STALE_DOCS_DRIFT" AND row last_verdict == "STALE_DOCS_DRIFT".
      4. Document inline (rust comment) that this is the path-(a) tradeoff: the walker only watches the FIRST source. Drift in B would NOT fire under path-(a). DO NOT add an assertion for the "B drifts -> stays BOUND" sub-case in this test; that's the documented limitation, asserting it would lock-in the limitation. (If a future agent moves to path-(b), this test should still pass on the A-drift path.)

    Test C -- `walk_single_source_rebind_heals_after_drift` (P74 finding):
      1. Seed catalog + bind row `row/heal` to `doc_c:1-2`. Capture `source_hash_v1`.
      2. Edit doc_c.md -- change lines 1-2 bytes.
      3. Run `walk`; ASSERT exit failure AND row last_verdict == "STALE_DOCS_DRIFT" AND row.source_hash STILL == source_hash_v1 (walker doesn't refresh).
      4. Re-bind row `row/heal` to `doc_c:1-2` (same citation, current bytes). ASSERT post-bind row.source_hash != source_hash_v1 (refreshed) AND row.last_verdict == "BOUND".
      5. Run `walk`; ASSERT exit success AND row last_verdict == "BOUND".

    Existing helper `seed_catalog` and `bind_row` may need a small extension: `bind_row` currently writes a fixed source range `:1-2` -- reuse it as-is; if a test needs a different doc, write a new local closure rather than altering the shared helper (don't fight the existing shape).

    All three tests follow the existing file's `assert_cmd::Command::cargo_bin("reposix-quality")` style. Use `serde_json::Value` to read the catalog JSON post-bind/post-walk (existing tests do this already).
  </behavior>
  <action>
    Append the three tests to `crates/reposix-quality/tests/walk.rs`. Run `cargo test -p reposix-quality --test walk` and confirm:
    - Test A FAILS pre-fix (assertion on source_hash == hash(doc_a) fails because the buggy bind overwrote with hash(doc_b)).
    - Test B's outcome depends on the bug interplay: pre-fix, doc_b's hash is what's stored; A's drift may or may not trip the compare depending on order. EXPECTED: Test B may PASS pre-fix (walker compares against stored hash regardless of which source it semantically represents) but MUST PASS post-fix.
    - Test C's outcome was identified in Task 1. Run it to see actual pre-fix behavior.

    Commit RED:
      git add crates/reposix-quality/tests/walk.rs
      git commit -m "test(p75): add bind-verb hash regression tests (RED)\n\nThree walker integration tests cover the BIND-VERB-FIX-01 invariants:\n- Single->Multi conversion preserves first-source hash (currently FAILS).\n- Multi row first-source drift fires STALE_DOCS_DRIFT.\n- Single row rebind heals after STALE_DOCS_DRIFT.\n\nP74 SURPRISES-INTAKE.md broadened the regression scope to Source::Single rows;\nTest C covers that case.\n\nRequirement: BIND-VERB-FIX-01.\nPhase: 75-bind-verb-hash-fix."
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && cargo test -p reposix-quality --test walk 2>&1 | tail -40</automated>
  </verify>
  <done>Three new tests appended; `cargo test -p reposix-quality --test walk` shows at minimum Test A FAILING with an assertion on source_hash mismatch; commit landed with the RED test SHA visible in `git log -1 --oneline`.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 3: Apply the bind-verb fix and turn tests GREEN</name>
  <files>crates/reposix-quality/src/commands/doc_alignment.rs</files>
  <behavior>
    The bind verb's `if let Some(row) = cat.row_mut(row_id)` arm (lines 285-310 of the current file) updates `row.source_hash = Some(src_hash);` unconditionally. The fix:

      - When the result is `Source::Single` (i.e. `sources.len() == 1` after the optional push):
          * The single source IS the invoked source (or the existing one when `already_present`).
          * `src_hash` is the freshly-computed hash of that single source's CURRENT bytes.
          * Behaviour: `row.source_hash = Some(src_hash);`  (heals drift on Single rebind -- D-01 refined).
      - When the result is `Source::Multi` (sources.len() > 1) AND the existing row was previously `Source::Single`:
          * The first element IS the prior single source. Its hash is what `row.source_hash` already holds. Preserve it.
          * Behaviour: leave `row.source_hash` UNCHANGED (do not assign).
      - When the result is `Source::Multi` AND the existing row was already `Source::Multi`:
          * The first element is unchanged (the new source appends OR is already_present == no-op). Preserve `source_hash`.
          * Behaviour: leave `row.source_hash` UNCHANGED.

    Implementation sketch (inside the existing `if let Some(row)` arm):

    ```rust
    // BIND-VERB-FIX-01 (P75): preserve first-source hash semantics.
    // The walker compares row.source_hash against source.as_slice()[0]'s
    // current bytes. Therefore source_hash MUST equal hash(first source).
    //   - Single result: the only source IS the freshly hashed one; refresh
    //     source_hash (this is the heal path for Single rows whose prose
    //     drifted -- P74 finding logged in SURPRISES-INTAKE.md).
    //   - Multi result: the first source is the EXISTING first source; its
    //     hash is already in row.source_hash. DO NOT overwrite.
    // Path (b) -- walker hashes every source from a Multi -- is filed as
    // v0.13.0 carry-forward MULTI-SOURCE-WATCH-01. Path (a) limitation:
    // drift in non-first Multi sources will not fire STALE_DOCS_DRIFT.
    let result_is_single = sources.len() == 1;
    row.source = if result_is_single {
        Source::Single(sources.into_iter().next().expect("len==1"))
    } else {
        Source::Multi(sources)
    };
    row.claim = claim.to_string();
    if result_is_single {
        row.source_hash = Some(src_hash);
    }
    // else: preserve existing row.source_hash (first-source invariant).
    ```

    Note: the new-row arm (line ~315 area, when `cat.row_mut(row_id)` returns None) constructs a Single, so `row.source_hash = Some(src_hash)` there is correct and stays as-is.

    After patching, run `cargo test -p reposix-quality --test walk`. All three new tests must pass; the existing walker tests must STILL pass (no regression).

    Then run the full crate test suite: `cargo test -p reposix-quality`. Per the memory budget, this is one cargo invocation only (single crate, ~15s). All tests must pass.

    If any existing test in `bind_validation.rs`, `bind_shell_verifier.rs`, `walk.rs`, `coverage.rs`, `merge_shards.rs`, `confirm_retire_envguard.rs`, `cli_smoke.rs`, or `hash_test_fn.rs` regresses, fix it ONLY if the regression is a stale assertion (e.g. a pre-existing test that asserted the buggy overwrite-on-Multi behaviour as a feature). If a regression points at an unintended semantic change in your fix, revisit the patch -- do NOT silently update the assertion.

    Commit GREEN:
      git add crates/reposix-quality/src/commands/doc_alignment.rs
      git commit -m "fix(p75): preserve first-source hash on Source::Multi promotion\n\nverbs::bind previously overwrote row.source_hash with the newly-cited\nsource's hash even when the result was Source::Multi. The walker reads\nsource_hash against source.as_slice()[0], so a Single->Multi promotion\nbroke the invariant and fired false STALE_DOCS_DRIFT.\n\nPath (a) per CONTEXT.md D-01: refresh source_hash only when the result\nis Source::Single; preserve it on Multi paths. Path (b) (walker iterates\nall sources from Multi) is filed as v0.13.0 MULTI-SOURCE-WATCH-01.\n\nP74 SURPRISES-INTAKE.md noted that Single rows whose prose drifted also\nappeared affected; the heal path (re-bind with same citation, refreshed\nbytes) works under this fix because Single results refresh source_hash\nfrom the current file bytes.\n\nRequirement: BIND-VERB-FIX-01.\nPhase: 75-bind-verb-hash-fix.\nTests: walk_multi_source_stable_no_false_drift + walk_multi_source_first_drift_fires_stale + walk_single_source_rebind_heals_after_drift."
  </behavior>
  <action>
    Apply the patch above. Run `cargo test -p reposix-quality` (single crate) and confirm all tests GREEN. Commit.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && cargo test -p reposix-quality 2>&1 | tail -25 && git log -1 --oneline</automated>
  </verify>
  <done>`cargo test -p reposix-quality` exits 0; all three new tests pass; no pre-existing test regresses; commit landed naming the requirement and tests.</done>
</task>

<task type="auto">
  <name>Task 4: Live-catalog smoke + linkedin row heal</name>
  <files>quality/catalogs/doc-alignment.json, quality/reports/verdicts/p75/walk-after-fix.txt</files>
  <action>
    Build release binary (one cargo invocation): `cargo build -p reposix-quality --release`.

    Step 1 -- Pre-fix state confirmation: read `quality/catalogs/doc-alignment.json` and find the `docs/social/linkedin/token-reduction-92pct` row (per P74 SURPRISES-INTAKE.md it is currently `STALE_DOCS_DRIFT`). Capture its `source_hash` and `last_verdict` to the verdict report.

    Step 2 -- Heal the linkedin row by running a fresh bind. Use the binary directly (NOT a custom shell pipeline -- the slash command surface owns this). The CONTEXT.md D-08 note says "expected STALE_DOCS_DRIFT then auto-rebind"; the auto-rebind is `refresh`, not `walk`. Run:

      target/release/reposix-quality doc-alignment refresh docs/social/linkedin.md

    `refresh` re-extracts claims and re-binds rows; this should pick up the new prose hash and update `source_hash`. If `refresh` doesn't heal the row, fall back to an explicit `bind` of the row with its current source citation (as-is from the row's `source` field).

    Step 3 -- Live walk: run `target/release/reposix-quality doc-alignment walk` and capture stdout + stderr to `quality/reports/verdicts/p75/walk-after-fix.txt`. Format:

      === target/release/reposix-quality doc-alignment walk ===
      exit: <code>
      ---- STDOUT ----
      <stdout>
      ---- STDERR ----
      <stderr>
      ---- BEFORE/AFTER ----
      linkedin row pre-fix: STALE_DOCS_DRIFT (source_hash=<v1>)
      linkedin row post-fix: BOUND (source_hash=<v2>)
      net new STALE_DOCS_DRIFT transitions: 0

    The pre-existing two STALE_DOCS_DRIFT rows from the P72 entry in SURPRISES-INTAKE.md (`polish-03-mermaid-render` + `cli-subcommand-surface`) MAY still be STALE -- they are out of P75 scope per the SURPRISES-INTAKE entry (P76 drains them). The verdict file must explicitly note that those two rows are NOT P75's responsibility and that the count of net-new STALE_DOCS_DRIFT transitions caused by P75 is zero.

    The walker may exit non-zero overall (because pre-existing STALE rows still block); that's fine. The verdict file documents WHICH rows are stale and confirms NONE are caused by the P75 fix.

    Commit:
      git add quality/catalogs/doc-alignment.json quality/reports/verdicts/p75/walk-after-fix.txt
      git commit -m "docs(p75): heal linkedin row + capture live-walk smoke evidence\n\nRefresh docs/social/linkedin.md re-bound docs/social/linkedin/token-reduction-92pct\nfrom STALE_DOCS_DRIFT -> BOUND with the post-P74 prose hash.\n\nLive walk evidence at quality/reports/verdicts/p75/walk-after-fix.txt.\nThe two pre-existing STALE rows logged in SURPRISES-INTAKE.md (P72) are\nout of P75 scope; P76 drains them.\n\nRequirement: BIND-VERB-FIX-01.\nPhase: 75-bind-verb-hash-fix."

    Memory budget: this task uses one cargo invocation (the release build). Subsequent invocations are the binary, not cargo.
  </action>
  <verify>
    <automated>cat /home/reuben/workspace/reposix/quality/reports/verdicts/p75/walk-after-fix.txt | head -60 && jq '.rows[] | select(.id == "docs/social/linkedin/token-reduction-92pct") | {id, last_verdict, source_hash}' /home/reuben/workspace/reposix/quality/catalogs/doc-alignment.json</automated>
  </verify>
  <done>Verdict file exists with stdout/stderr capture; linkedin row's `last_verdict == "BOUND"`; verdict file explicitly counts zero net-new STALE_DOCS_DRIFT transitions caused by P75; commit landed.</done>
</task>

<task type="auto">
  <name>Task 5: CLAUDE.md P75 H3 (<=20 lines) + v0.13.0 carry-forward</name>
  <files>CLAUDE.md, .planning/milestones/v0.13.0-phases/CARRY-FORWARD.md</files>
  <action>
    Edit CLAUDE.md: add a P75 H3 subsection under the existing "v0.12.1 -- in flight" section (locate the section by grep for `v0.12.1`; if no in-flight subsection exists, insert under the most appropriate top-level heading -- look at how P72/P73/P74 H3 subsections were added in their phases for the convention, or follow the OP-8 + Quality-Gates section style).

    Constraint: the subsection MUST be <=20 lines (smaller than P72-P74 because the fix is smaller -- per CONTEXT.md D-08). It MUST cover:
      1. The bind-verb invariant: source_hash always equals hash(first source). On Single result, refresh; on Multi result, preserve.
      2. The path-(a) tradeoff: the walker only watches the FIRST source in a Multi row. Drift in non-first sources WON'T fire STALE_DOCS_DRIFT under path-(a). The next maintainer needs to know this without re-discovering.
      3. The pointer to v0.13.0 MULTI-SOURCE-WATCH-01 for path-(b).
      4. Reference to the regression tests in `crates/reposix-quality/tests/walk.rs` for the invariant.

    Style: progressive disclosure (per OP-4). The H3 is a quick reference; the long-form rationale lives in this PLAN.md and SURPRISES-INTAKE.md. Avoid duplicating the bug history; just state the invariant + tradeoff.

    Then create or append to `.planning/milestones/v0.13.0-phases/CARRY-FORWARD.md` (create the directory if absent: `mkdir -p .planning/milestones/v0.13.0-phases`). Add an entry:

    ```markdown
    ## MULTI-SOURCE-WATCH-01 -- walker hashes every source from Source::Multi

    **Source:** v0.12.1 P75 (BIND-VERB-FIX-01) shipped path (a) -- preserve first-source hash on Multi promotion. Path (b) (walker iterates every source citation in a Multi row, hashes each, ANDs the results) was deferred.

    **Why deferred:** path (b) requires schema migration (`source_hash` -> `source_hashes: Vec<String>`) with parallel-array invariant on Multi rows, plus migration of the populated 388-row catalog. Out of scope for a single-phase fix.

    **Acceptance:** `Row::source_hashes: Vec<String>` parallel-array to `source.as_slice()`; `walk` hashes each source citation against its corresponding `source_hashes[i]`; `bind` writes/preserves all entries on the parallel array; existing single-source-hash field migrates via `serde(default)` + a one-time backfill.

    **Carries from:** v0.12.1 phase 75; logged in `.planning/phases/75-bind-verb-hash-fix/PLAN.md`.
    ```

    If the v0.13.0-phases directory already has its own CARRY-FORWARD.md or REQUIREMENTS.md scaffolded, append; otherwise create a fresh CARRY-FORWARD.md.

    Commit:
      git add CLAUDE.md .planning/milestones/v0.13.0-phases/CARRY-FORWARD.md
      git commit -m "docs(p75): document path-(a) tradeoff + file MULTI-SOURCE-WATCH-01 (v0.13.0)\n\nCLAUDE.md P75 H3 (<=20 lines) names the bind-verb invariant\n(source_hash == hash(first source)) and the walker's first-source-only\nlimitation. Path (b) (walker iterates every Source::Multi citation)\nfiled as v0.13.0 carry-forward.\n\nRequirement: BIND-VERB-FIX-01.\nPhase: 75-bind-verb-hash-fix."

    Verify line count: `wc -l` on the diff for CLAUDE.md should show the new subsection is <=20 lines (excluding the surrounding hunks).
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && grep -n "P75\|MULTI-SOURCE-WATCH-01" CLAUDE.md && cat .planning/milestones/v0.13.0-phases/CARRY-FORWARD.md | head -20 && git log --oneline -1</automated>
  </verify>
  <done>CLAUDE.md has a P75 H3 subsection <=20 lines naming the invariant + tradeoff; v0.13.0 CARRY-FORWARD.md has MULTI-SOURCE-WATCH-01 entry; commit landed.</done>
</task>

<task type="auto">
  <name>Task 6: Phase 75 SUMMARY.md</name>
  <files>.planning/phases/75-bind-verb-hash-fix/SUMMARY.md</files>
  <action>
    Write the phase SUMMARY using the GSD template (`@$HOME/.claude/get-shit-done/templates/summary.md`). Required sections:

    - Phase: 75-bind-verb-hash-fix
    - Status: COMPLETE
    - Requirement closed: BIND-VERB-FIX-01
    - Commits (newest first, with SHAs from `git log --oneline -10`):
      * Task 5 commit (CLAUDE.md + carry-forward)
      * Task 4 commit (live walk smoke)
      * Task 3 commit (the fix, GREEN)
      * Task 2 commit (regression tests, RED)
    - What was found / what was fixed: 1-paragraph summary of the bind-verb hash-overwrite bug + the P74 broadening to Source::Single rows (or confirmation that the existing rebind path handled it).
    - What was deferred: MULTI-SOURCE-WATCH-01 (v0.13.0).
    - Live evidence: pointer to `quality/reports/verdicts/p75/walk-after-fix.txt` AND the linkedin row's last_verdict transition (STALE_DOCS_DRIFT -> BOUND).
    - Verifier verdict: TO BE FILED at `quality/reports/verdicts/p75/VERDICT.md` by top-level orchestrator dispatch (Path A `gsd-verifier`).
    - Surprises file appends (if any during P75 execution): list, or "none -- the fix landed cleanly".
    - Good-to-haves file appends (if any): list, or "none".
    - CLAUDE.md update: line range of the P75 H3 (e.g. `CLAUDE.md:NNN-MMM`).

    Commit:
      git add .planning/phases/75-bind-verb-hash-fix/SUMMARY.md
      git commit -m "docs(p75): phase SUMMARY -- BIND-VERB-FIX-01 closed\n\nSee .planning/phases/75-bind-verb-hash-fix/SUMMARY.md.\n\nRequirement: BIND-VERB-FIX-01.\nPhase: 75-bind-verb-hash-fix."
  </action>
  <verify>
    <automated>cat /home/reuben/workspace/reposix/.planning/phases/75-bind-verb-hash-fix/SUMMARY.md | head -50 && git log --oneline -6</automated>
  </verify>
  <done>SUMMARY.md committed with all required sections; commit chain on the branch shows: tests RED -> fix GREEN -> live smoke -> CLAUDE.md+carry-forward -> SUMMARY (5 atomic commits, in that order).</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| filesystem -> verbs::bind | Operator-supplied `--source <file>:<lstart>-<lend>` could point at any path. Existing validation rejects non-existent files (line 244-249). The fix does not introduce a new boundary. |
| verbs::bind -> Catalog JSON | Mutation to `source_hash` and `source` fields. The fix narrows when `source_hash` is overwritten. |
| Catalog JSON -> verbs::walk | Walker reads `source_hash` and compares to `source.as_slice()[0]`. The fix preserves the existing read pattern; no new sink. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-75-01 | Tampering | `verbs::bind` -> `row.source_hash` | mitigate | The fix narrows the write path to `if result_is_single`. A regression test (`walk_multi_source_stable_no_false_drift`) asserts the invariant on the post-bind catalog state. |
| T-75-02 | Information disclosure | live-walk smoke output (`walk-after-fix.txt`) | accept | The capture is local to `quality/reports/verdicts/p75/`; no PII; file paths and verdict states only. Already committed pattern (P64-P74 verdicts). |
| T-75-03 | Denial of service | Walker false STALE_DOCS_DRIFT firing pre-push | mitigate | The bug ITSELF is a soft DoS (false-positive blocking pre-push). The fix removes the false-positive vector. The `walk_multi_source_stable_no_false_drift` test asserts no STALE_DOCS_DRIFT on a stable Multi row. |
| T-75-04 | Repudiation | Catalog mutation by `bind` | accept | Existing audit pattern (`audit_events_cache` for filesystem reads, `audit_events` for backend mutations) does not cover catalog mutations -- catalog is operator-driven, not network-touching. The git log on `quality/catalogs/doc-alignment.json` is the audit trail. No new exposure. |
| T-75-05 | Elevation of privilege | `cargo test` execution path | accept | Fix lives in test-only + `verbs::bind` source code. No new build-time hook, no new CLI flag, no new env var. |
</threat_model>

<verification>
Phase-level checks:
- `cargo test -p reposix-quality` exits 0 (single crate, ~15s, memory budget honored).
- `target/release/reposix-quality doc-alignment walk` post-fix shows linkedin row BOUND; no NEW STALE_DOCS_DRIFT transitions traceable to P75.
- `quality/reports/verdicts/p75/walk-after-fix.txt` exists with stdout + stderr + before/after annotations.
- `git log --oneline -6` shows 5 P75 commits in canonical order: RED tests -> GREEN fix -> live smoke -> CLAUDE.md+carry-forward -> SUMMARY.
- `CLAUDE.md` P75 H3 subsection is <=20 lines.
- `.planning/milestones/v0.13.0-phases/CARRY-FORWARD.md` contains MULTI-SOURCE-WATCH-01.
- `.planning/phases/75-bind-verb-hash-fix/SUMMARY.md` exists and links the verdict path.

Top-level orchestrator (NOT this PLAN's executor) dispatches `gsd-verifier` Path A after this PLAN completes; verdict at `quality/reports/verdicts/p75/VERDICT.md`. The SUMMARY.md is the input the verifier reads.
</verification>

<success_criteria>
- BIND-VERB-FIX-01 closed: regression tests + fix + live smoke evidence on disk + CLAUDE.md updated + carry-forward filed + SUMMARY written.
- Test suite GREEN on `cargo test -p reposix-quality`.
- Linkedin row (`docs/social/linkedin/token-reduction-92pct`) heals from STALE_DOCS_DRIFT to BOUND in `quality/catalogs/doc-alignment.json`.
- Zero net-new STALE_DOCS_DRIFT transitions traceable to P75.
- Walker first-source-only limitation explicitly documented in CLAUDE.md (next maintainer doesn't re-discover by surprise).
- Path (b) carries forward as MULTI-SOURCE-WATCH-01 to v0.13.0.
- Phase verifier verdict (dispatched by top-level orchestrator after this PLAN) grades GREEN.
</success_criteria>

<output>
After completion, the executor will have produced:
- `.planning/phases/75-bind-verb-hash-fix/SUMMARY.md` (the SUMMARY for verifier consumption).
- `quality/reports/verdicts/p75/walk-after-fix.txt` (live smoke evidence).
- 5 atomic commits in canonical order (RED -> GREEN -> SMOKE -> DOCS -> SUMMARY).

Top-level orchestrator dispatches `gsd-verifier` Path A AFTER this PLAN executes; the verifier writes `quality/reports/verdicts/p75/VERDICT.md`. P75 closes when the verdict is GREEN.
</output>
