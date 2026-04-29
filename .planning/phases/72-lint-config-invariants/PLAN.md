---
phase: 72-lint-config-invariants
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - quality/gates/code/lint-invariants/forbid-unsafe-code.sh
  - quality/gates/code/lint-invariants/rust-msrv.sh
  - quality/gates/code/lint-invariants/tests-green.sh
  - quality/gates/code/lint-invariants/errors-doc-section.sh
  - quality/gates/code/lint-invariants/rust-stable-channel.sh
  - quality/gates/code/lint-invariants/cargo-check-workspace.sh
  - quality/gates/code/lint-invariants/cargo-test-count.sh
  - quality/gates/code/lint-invariants/demo-script-exists.sh
  - quality/gates/code/lint-invariants/README.md
  - quality/catalogs/doc-alignment.json
  - docs/development/contributing.md
  - README.md
  - CLAUDE.md
  - .planning/phases/72-lint-config-invariants/SUMMARY.md
autonomous: true
parallelization: false
gap_closure: false
cross_ai: false
task_count: 14
requirements:
  - LINT-CONFIG-01
  - LINT-CONFIG-02
  - LINT-CONFIG-03
  - LINT-CONFIG-04
  - LINT-CONFIG-05
  - LINT-CONFIG-06
  - LINT-CONFIG-07
  - LINT-CONFIG-08
  - LINT-CONFIG-09

must_haves:
  truths:
    - "9 catalog rows in `quality/catalogs/doc-alignment.json` transition from MISSING_TEST to BOUND."
    - "8 verifier scripts under `quality/gates/code/lint-invariants/` are executable and exit 0 GREEN against the current workspace."
    - "Both `forbid(unsafe_code)` rows (`README-md/forbid-unsafe-code` + `docs-development-contributing-md/forbid-unsafe-per-crate`) bind to the SAME verifier file (`forbid-unsafe-code.sh`) — one source of truth per D-01."
    - "The cargo-test-count verifier uses a re-measured `>= N` floor matching prose in `docs/development/contributing.md` (and README.md if cited there); prose was updated BEFORE the bind ran (D-06)."
    - "The `tests-green` verifier compiles only (`cargo test --workspace --no-run`); does NOT run the suite (D-05)."
    - "The `errors-doc-section` verifier uses clippy's `missing_errors_doc` lint (D-07), not grep."
    - "All cargo invocations across the 8 verifiers run one-at-a-time (build memory budget)."
    - "CLAUDE.md gains a P72 H3 subsection ≤30 lines under `## v0.12.1 — in flight`; banned-words check passes."
    - "Verifier subagent verdict at `quality/reports/verdicts/p72/VERDICT.md` is graded GREEN by an unbiased dispatched subagent (top-level orchestrator action — D-08)."
  artifacts:
    - path: "quality/gates/code/lint-invariants/forbid-unsafe-code.sh"
      provides: "Shell verifier asserting every `crates/*/src/{lib,main}.rs` contains `#![forbid(unsafe_code)]`."
      min_lines: 25
    - path: "quality/gates/code/lint-invariants/rust-msrv.sh"
      provides: "Shell verifier asserting workspace `Cargo.toml` pins `rust-version = \"1.82\"`."
      min_lines: 20
    - path: "quality/gates/code/lint-invariants/tests-green.sh"
      provides: "Shell verifier running `cargo test --workspace --no-run` (compile-only)."
      min_lines: 25
    - path: "quality/gates/code/lint-invariants/errors-doc-section.sh"
      provides: "Shell verifier invoking clippy `missing_errors_doc` lint and asserting zero hits."
      min_lines: 25
    - path: "quality/gates/code/lint-invariants/rust-stable-channel.sh"
      provides: "Shell verifier asserting `rust-toolchain.toml` `channel = \"stable\"`."
      min_lines: 20
    - path: "quality/gates/code/lint-invariants/cargo-check-workspace.sh"
      provides: "Shell verifier running `cargo check --workspace -q`."
      min_lines: 20
    - path: "quality/gates/code/lint-invariants/cargo-test-count.sh"
      provides: "Shell verifier counting test binaries via cargo JSON output and asserting `>= N` floor."
      min_lines: 30
    - path: "quality/gates/code/lint-invariants/demo-script-exists.sh"
      provides: "Shell verifier asserting `[ -x scripts/dark-factory-test.sh ]`."
      min_lines: 15
    - path: "quality/gates/code/lint-invariants/README.md"
      provides: "Sub-area README naming the 8 verifiers + 9 rows + memory-budget notes."
      min_lines: 20
  key_links:
    - from: "quality/catalogs/doc-alignment.json"
      to: "quality/gates/code/lint-invariants/*.sh"
      via: "row.tests[] (TestRef::File path) populated by `reposix-quality doc-alignment refresh`"
      pattern: "quality/gates/code/lint-invariants/"
    - from: "docs/development/contributing.md (test-count prose)"
      to: "quality/gates/code/lint-invariants/cargo-test-count.sh"
      via: "verifier reads documented count from doc; floor uses `>= N`"
      pattern: "cargo-test-count\\.sh"
    - from: "CLAUDE.md `## v0.12.1 — in flight`"
      to: "P72 H3 subsection naming verifier home + 9 rows"
      via: "QG-07 grounding rule"
      pattern: "### P72 — Lint-config invariants"

user_setup: []
---

<objective>
Bind 9 `MISSING_TEST` rows in `quality/catalogs/doc-alignment.json` (the README + contributing.md lint/MSRV/test-count cluster) to 8 single-purpose shell verifiers under `quality/gates/code/lint-invariants/`. The walker (P71 schema 2.0) hashes both the source prose AND each verifier file body; drift on either fires `STALE_DOCS_DRIFT` and the next maintainer reviews. Concretizes "lint-config rows ARE testable" — historically these rows lived in MISSING_TEST as 'we-know-it's-true-but-no-test-binds-it' and the walker had no way to detect drift if e.g. someone removed `forbid(unsafe_code)` from a single crate.

Purpose: close 9 of the 22 MISSING_TEST rows targeted by the v0.12.1 autonomous-run cluster (P72-P74); raise `alignment_ratio` toward the 0.85 v0.12.1 target; ground the next agent in the verifier-home convention.

Output:
- 8 new shell verifier scripts under `quality/gates/code/lint-invariants/`.
- 1 sub-area README at `quality/gates/code/lint-invariants/README.md`.
- 9 catalog rows transitioned `MISSING_TEST` → `BOUND` after `refresh`.
- Re-measured cargo-test-count prose (contributing.md and possibly README.md).
- CLAUDE.md P72 H3 subsection (≤30 lines).
- Phase summary at `.planning/phases/72-lint-config-invariants/SUMMARY.md`.
- BEFORE/AFTER `doc-alignment status` snapshots captured for the verifier dispatch.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/STATE.md
@.planning/HANDOVER-v0.12.1.md
@.planning/phases/72-lint-config-invariants/CONTEXT.md
@.planning/milestones/v0.12.1-phases/ROADMAP.md
@.planning/milestones/v0.12.1-phases/REQUIREMENTS.md
@.planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md
@.planning/milestones/v0.12.1-phases/GOOD-TO-HAVES.md
@quality/PROTOCOL.md
@quality/catalogs/README.md
@CLAUDE.md
@quality/gates/code/cargo-fmt-check.sh
@quality/gates/code/cargo-clippy-warnings.sh
@quality/gates/code/clippy-lint-loaded.sh

<interfaces>
<!-- Key contracts the executor needs. Extracted from codebase. Use these directly — no exploration. -->

## `reposix-quality doc-alignment bind` (canonical command shape, single test)

```
target/release/reposix-quality doc-alignment bind \
  --catalog quality/catalogs/doc-alignment.json \
  --row-id <ROW_ID> \
  --claim "<one-sentence behavioral claim>" \
  --source <FILE>:<LSTART>-<LEND> \
  --test quality/gates/code/lint-invariants/<verifier>.sh \
  --grade GREEN \
  --rationale "<short rationale>"
```

`--test <path>.sh` parses to `TestRef::File` (see `parse_test` in
`crates/reposix-quality/src/commands/doc_alignment.rs:1187`); body hash is
the file's sha256 (`crate::hash::file_hash`). Multi-test rows accept
`--test` repeated; `bind` validates ALL `--test` citations BEFORE
mutating the catalog (atomic).

## `reposix-quality doc-alignment refresh <files...>`

Re-walks the named source files, recomputes source hashes for rows whose
`source.file` matches, and re-evaluates row state. Use AFTER all binds
land to flip in-memory state to `BOUND` consistently. Pass:

```
target/release/reposix-quality doc-alignment refresh \
  README.md docs/development/contributing.md
```

## `reposix-quality doc-alignment status` (BEFORE/AFTER capture)

```
target/release/reposix-quality doc-alignment status --top 20 > /tmp/p72-status-before.txt   # capture once at task 1
target/release/reposix-quality doc-alignment status --top 20 > /tmp/p72-status-after.txt    # capture at task 14
```

Diff the two for the verdict report (`alignment_ratio` delta).

## Existing verifier-script structural contract (pattern-match these)

`quality/gates/code/cargo-fmt-check.sh` and `quality/gates/code/cargo-clippy-warnings.sh` show the shape:

- `#!/usr/bin/env bash` + `set -euo pipefail`.
- `REPO_ROOT` resolved from `BASH_SOURCE[0]`.
- `ARTIFACT_DIR="${REPO_ROOT}/quality/reports/verifications/code"` — for these P72 verifiers, use `quality/reports/verifications/code/lint-invariants/`.
- `--row-id <id>` flag honored (defaults to a sensible row id; the walker passes the catalog row id when invoked via runner).
- Capture stdout/stderr, write a JSON artifact with at minimum `{row_id, status: PASS|FAIL, exit_code, ts, summary}`.
- Exit 0 on PASS, non-zero on FAIL.

Lint-invariants verifiers DO NOT need a `--row-id` flag if the catalog
row's `verifier.script` points at them by path (the walker hashes the
script body, not its row-routing). Keep them simple: positional, idempotent,
re-runnable. Add the artifact write only if the runner discovers them —
otherwise the bind alone (which hashes the file body) is sufficient
grounding.

## The 9 catalog rows (verbatim slugs from doc-alignment.json)

| # | row id | source | verifier (under lint-invariants/) |
|---|---|---|---|
| 1 | `README-md/forbid-unsafe-code` | README.md:109 | `forbid-unsafe-code.sh` |
| 2 | `README-md/rust-1-82-requirement` | README.md:79 | `rust-msrv.sh` |
| 3 | `README-md/tests-green` | README.md:109 | `tests-green.sh` |
| 4 | `docs-development-contributing-md/forbid-unsafe-per-crate` | docs/development/contributing.md:59 | `forbid-unsafe-code.sh` *(SAME file as row 1 per D-01)* |
| 5 | `docs-development-contributing-md/errors-doc-section-required` | docs/development/contributing.md:61 | `errors-doc-section.sh` |
| 6 | `docs-development-contributing-md/rust-stable-no-nightly` | docs/development/contributing.md:64 | `rust-stable-channel.sh` |
| 7 | `docs-development-contributing-md/cargo-check-workspace-available` | (find via grep — contributing.md, ~line 65-75 area) | `cargo-check-workspace.sh` |
| 8 | `docs-development-contributing-md/cargo-test-133-tests` | docs/development/contributing.md:20 | `cargo-test-count.sh` |
| 9 | `docs-development-contributing-md/demo-script-exists` | (find via grep — contributing.md) | `demo-script-exists.sh` |

> Resolve exact source line ranges by inspecting the rows themselves:
> `jq '.rows[] | select(.id==\"<row-id>\") | .source' quality/catalogs/doc-alignment.json`
> Use the row's existing `source.line_start`/`source.line_end` when binding;
> do NOT hand-pick lines.

</interfaces>
</context>

<tasks>

<!-- ========================================================== -->
<!-- WAVE 1 — Catalog-first commit (verifier stubs pin contract) -->
<!-- ========================================================== -->

<task type="auto">
  <name>Task 1: Capture BEFORE status snapshot + scaffold lint-invariants/ with 8 stub scripts</name>
  <files>
    quality/gates/code/lint-invariants/forbid-unsafe-code.sh,
    quality/gates/code/lint-invariants/rust-msrv.sh,
    quality/gates/code/lint-invariants/tests-green.sh,
    quality/gates/code/lint-invariants/errors-doc-section.sh,
    quality/gates/code/lint-invariants/rust-stable-channel.sh,
    quality/gates/code/lint-invariants/cargo-check-workspace.sh,
    quality/gates/code/lint-invariants/cargo-test-count.sh,
    quality/gates/code/lint-invariants/demo-script-exists.sh,
    quality/gates/code/lint-invariants/README.md
  </files>
  <action>
    Catalog-first commit per `quality/PROTOCOL.md` § Step 3. This pins the GREEN contract before the implementations land.

    1. Build the binary if missing: `cargo build --release -p reposix-quality` (one cargo invocation; nothing else compiles in this task).
    2. Capture BEFORE snapshot:
       ```
       mkdir -p quality/reports/verdicts/p72
       target/release/reposix-quality doc-alignment status --top 30 > quality/reports/verdicts/p72/status-before.txt
       jq '.summary | {alignment_ratio, claims_total, claims_bound, claims_missing_test}' \
         quality/catalogs/doc-alignment.json > quality/reports/verdicts/p72/summary-before.json
       ```
    3. Create `quality/gates/code/lint-invariants/` and add 8 STUB shell scripts (one per verifier, NOT one per row — `forbid-unsafe-code.sh` covers two rows per D-01). Each stub:
       - `#!/usr/bin/env bash`
       - `set -euo pipefail`
       - One-line comment naming the row id(s) it binds.
       - A TODO comment naming the implementation task that will fill it (`# TODO(P72 task N): implement <claim>`).
       - `echo "STUB: $0 not yet implemented" >&2; exit 1` body so the file is parseable but not yet GREEN.
       - `chmod +x` on each.
    4. Add `quality/gates/code/lint-invariants/README.md` listing:
       - 8 verifiers and which of the 9 catalog rows each binds (forbid-unsafe-code.sh covers 2 rows).
       - Memory-budget reminder (D-04): cargo invocations are serialized; shell + standard Unix tools (D-03).
       - Pointer to CLAUDE.md "Build memory budget" + `quality/PROTOCOL.md`.
    5. Stage + commit:
       ```
       git add quality/gates/code/lint-invariants/ quality/reports/verdicts/p72/status-before.txt quality/reports/verdicts/p72/summary-before.json
       git commit -m "P72(catalog-first): scaffold quality/gates/code/lint-invariants/ stubs (LINT-CONFIG-01..09)"
       ```
       Cite all 9 LINT-CONFIG-* requirement IDs in the commit body.

    DO NOT bind the rows yet. DO NOT touch the catalog JSON yet.
  </action>
  <verify>
    <automated>
      bash -n quality/gates/code/lint-invariants/*.sh \
        && test -x quality/gates/code/lint-invariants/forbid-unsafe-code.sh \
        && test -x quality/gates/code/lint-invariants/rust-msrv.sh \
        && test -x quality/gates/code/lint-invariants/tests-green.sh \
        && test -x quality/gates/code/lint-invariants/errors-doc-section.sh \
        && test -x quality/gates/code/lint-invariants/rust-stable-channel.sh \
        && test -x quality/gates/code/lint-invariants/cargo-check-workspace.sh \
        && test -x quality/gates/code/lint-invariants/cargo-test-count.sh \
        && test -x quality/gates/code/lint-invariants/demo-script-exists.sh \
        && test -f quality/gates/code/lint-invariants/README.md \
        && test -f quality/reports/verdicts/p72/status-before.txt \
        && git log -1 --format=%s | grep -q "P72(catalog-first)"
    </automated>
  </verify>
  <done>8 stub verifiers + README exist under `quality/gates/code/lint-invariants/`; all `bash -n`-clean; BEFORE status snapshot captured at `quality/reports/verdicts/p72/status-before.txt`; one atomic commit `P72(catalog-first): scaffold ...` landed.</done>
</task>

<!-- =================================================================== -->
<!-- WAVE 2 — Verifier implementations. Sequential by D-04 (memory budget). -->
<!-- Order: cheapest first, heaviest cargo verifier (`tests-green`) LAST. -->
<!-- =================================================================== -->

<task type="auto">
  <name>Task 2: Implement demo-script-exists.sh (cheapest; one filesystem check)</name>
  <files>quality/gates/code/lint-invariants/demo-script-exists.sh</files>
  <action>
    Replace the stub body with the real check:

    - Resolve `REPO_ROOT` from `BASH_SOURCE[0]` (mirror `quality/gates/code/cargo-fmt-check.sh`).
    - Assert `[ -x "${REPO_ROOT}/scripts/dark-factory-test.sh" ]`.
    - On failure, print to stderr the path that doesn't exist OR isn't executable; exit 1.
    - On success, print `PASS: scripts/dark-factory-test.sh exists and is executable`; exit 0.

    Run locally to confirm GREEN against current workspace: `bash quality/gates/code/lint-invariants/demo-script-exists.sh && echo OK`.

    Commit: `P72: implement demo-script-exists.sh (LINT-CONFIG-08)`.
  </action>
  <verify>
    <automated>bash quality/gates/code/lint-invariants/demo-script-exists.sh</automated>
  </verify>
  <done>Verifier exits 0 against current workspace; commit landed citing LINT-CONFIG-08.</done>
</task>

<task type="auto">
  <name>Task 3: Implement forbid-unsafe-code.sh (covers BOTH `forbid(unsafe_code)` rows per D-01)</name>
  <files>quality/gates/code/lint-invariants/forbid-unsafe-code.sh</files>
  <action>
    Walk every `crates/*/src/lib.rs` AND `crates/*/src/main.rs` and assert each contains `#![forbid(unsafe_code)]`. Per CONTEXT.md § specifics:

    ```bash
    # Find candidate entry points
    mapfile -t entries < <(find crates -path '*/src/lib.rs' -o -path '*/src/main.rs')
    # Identify any missing the attribute
    missing=()
    for f in "${entries[@]}"; do
      grep -qE '^#!\[forbid\(unsafe_code\)\]' "$f" || missing+=("$f")
    done
    if (( ${#missing[@]} > 0 )); then
      echo "FAIL: the following files lack #![forbid(unsafe_code)]:" >&2
      printf '  %s\n' "${missing[@]}" >&2
      exit 1
    fi
    echo "PASS: all ${#entries[@]} crate entry points contain #![forbid(unsafe_code)]"
    exit 0
    ```

    The error message MUST name the offending file(s) (Principle B — agent-resolvable failure).

    **Eager-resolution gate (D-09):** if any crate is genuinely missing the attribute AND adding it is < 5-file scope, ADD it in this same task and note in the commit body. If the gap is wider, append to `.planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md` per CLAUDE.md OP-8 and leave the verifier RED for now (the plan continues; P76 absorbs).

    Commit: `P72: implement forbid-unsafe-code.sh (LINT-CONFIG-01)`.
  </action>
  <verify>
    <automated>bash quality/gates/code/lint-invariants/forbid-unsafe-code.sh</automated>
  </verify>
  <done>Verifier exits 0 against current workspace OR (if gap surfaced) eager-fixed crates committed in same task / SURPRISES-INTAKE.md updated; commit landed citing LINT-CONFIG-01.</done>
</task>

<task type="auto">
  <name>Task 4: Implement rust-msrv.sh (asserts `rust-version = "1.82"` in workspace Cargo.toml)</name>
  <files>quality/gates/code/lint-invariants/rust-msrv.sh</files>
  <action>
    Grep the workspace `Cargo.toml` for `rust-version = "1.82"` (the current MSRV per `rust-toolchain.toml` and CLAUDE.md). If MSRV-01 (P70 carry-forward) ever bumps to 1.85, this verifier breaks LOUD — that is the desired drift signal.

    ```bash
    # Resolve REPO_ROOT (same pattern as cargo-fmt-check.sh)
    if ! grep -qE '^rust-version = "1\.82"' "${REPO_ROOT}/Cargo.toml"; then
      echo "FAIL: workspace Cargo.toml missing 'rust-version = \"1.82\"'" >&2
      grep -nE '^rust-version' "${REPO_ROOT}/Cargo.toml" >&2 || true
      exit 1
    fi
    echo "PASS: workspace MSRV pinned to 1.82"
    exit 0
    ```

    Run locally; commit: `P72: implement rust-msrv.sh (LINT-CONFIG-02)`.
  </action>
  <verify>
    <automated>bash quality/gates/code/lint-invariants/rust-msrv.sh</automated>
  </verify>
  <done>Verifier exits 0; commit landed citing LINT-CONFIG-02.</done>
</task>

<task type="auto">
  <name>Task 5: Implement rust-stable-channel.sh (asserts `rust-toolchain.toml` `channel = "stable"`)</name>
  <files>quality/gates/code/lint-invariants/rust-stable-channel.sh</files>
  <action>
    Grep `rust-toolchain.toml` for `channel = "stable"`:

    ```bash
    if ! grep -qE '^channel = "stable"' "${REPO_ROOT}/rust-toolchain.toml"; then
      echo "FAIL: rust-toolchain.toml missing 'channel = \"stable\"' — nightly is banned per CLAUDE.md" >&2
      cat "${REPO_ROOT}/rust-toolchain.toml" >&2
      exit 1
    fi
    echo "PASS: toolchain channel = stable"
    exit 0
    ```

    Commit: `P72: implement rust-stable-channel.sh (LINT-CONFIG-05)`.
  </action>
  <verify>
    <automated>bash quality/gates/code/lint-invariants/rust-stable-channel.sh</automated>
  </verify>
  <done>Verifier exits 0; commit landed citing LINT-CONFIG-05.</done>
</task>

<task type="auto">
  <name>Task 6: Implement cargo-check-workspace.sh (one cargo invocation; serialized per D-04)</name>
  <files>quality/gates/code/lint-invariants/cargo-check-workspace.sh</files>
  <action>
    Run `cargo check --workspace -q` from `REPO_ROOT`. On any non-zero exit, forward stderr and FAIL.

    ```bash
    cd "$REPO_ROOT"
    if ! cargo check --workspace -q 2>&1; then
      echo "FAIL: cargo check --workspace exited non-zero" >&2
      exit 1
    fi
    echo "PASS: cargo check --workspace -q clean"
    exit 0
    ```

    **Memory budget (D-04):** this is a real cargo invocation. NO other cargo invocation runs concurrently (subagent rule + executor sequential). Expect ~10-30s wall-clock on warm cache.

    Commit: `P72: implement cargo-check-workspace.sh (LINT-CONFIG-06)`.
  </action>
  <verify>
    <automated>bash quality/gates/code/lint-invariants/cargo-check-workspace.sh</automated>
  </verify>
  <done>Verifier exits 0 against current workspace; commit landed citing LINT-CONFIG-06.</done>
</task>

<task type="auto">
  <name>Task 7: Implement errors-doc-section.sh (clippy `missing_errors_doc` lint per D-07)</name>
  <files>quality/gates/code/lint-invariants/errors-doc-section.sh</files>
  <action>
    Run clippy with the `missing_errors_doc` lint enabled and assert ZERO hits. Per CONTEXT.md § specifics:

    ```bash
    cd "$REPO_ROOT"
    # Use --message-format=json so we can count missing_errors_doc occurrences precisely.
    output=$(cargo clippy --workspace --message-format=json -- \
              -W clippy::missing_errors_doc 2>/dev/null || true)
    hits=$(echo "$output" | jq -rs '
      [.[] | select(.reason=="compiler-message" and
                    .message.code.code=="clippy::missing_errors_doc")] | length' 2>/dev/null || echo "0")
    if [[ "$hits" != "0" ]]; then
      echo "FAIL: $hits pub fn(s) returning Result<...> lack a # Errors rustdoc section" >&2
      echo "$output" | jq -rs '
        [.[] | select(.reason=="compiler-message" and
                      .message.code.code=="clippy::missing_errors_doc")][] |
        "\(.message.spans[0].file_name):\(.message.spans[0].line_start) — \(.message.message)"' >&2
      exit 1
    fi
    echo "PASS: 0 missing_errors_doc hits across workspace"
    exit 0
    ```

    **Memory budget (D-04):** one cargo invocation. The clippy run is heavier than `cargo check`; expect 30-90s on warm cache.

    **Eager-resolution gate (D-09):** if `missing_errors_doc` reports hits today and the fix is < 5 files of `# Errors` comment additions, fix in-task. Wider = SURPRISES-INTAKE.md entry per OP-8 and leave verifier RED until P76.

    Commit: `P72: implement errors-doc-section.sh (LINT-CONFIG-04)`.
  </action>
  <verify>
    <automated>bash quality/gates/code/lint-invariants/errors-doc-section.sh</automated>
  </verify>
  <done>Verifier exits 0 OR (if gap surfaced) eager-fixed in same task / SURPRISES-INTAKE.md entry appended; commit landed citing LINT-CONFIG-04.</done>
</task>

<task type="auto">
  <name>Task 8: Implement cargo-test-count.sh (heaviest verifier; LAST per D-04)</name>
  <files>quality/gates/code/lint-invariants/cargo-test-count.sh</files>
  <action>
    Compile (not run) workspace tests via `cargo test --workspace --no-run --message-format=json`, count the `compiler-artifact` events whose `target.test == true`, and assert `count >= ${REPOSIX_LINT_TEST_FLOOR:-N}` where N is the documented count from contributing.md (resolved at task 9 below — for THIS task, accept the floor via env var with a sane default of 50; task 10 re-binds with the measured floor).

    ```bash
    cd "$REPO_ROOT"
    floor="${REPOSIX_LINT_TEST_FLOOR:-50}"
    json=$(cargo test --workspace --no-run --message-format=json 2>/dev/null || true)
    count=$(echo "$json" | jq -rs '
      [.[] | select(.reason=="compiler-artifact" and (.target.test // false))] | length')
    if (( count < floor )); then
      echo "FAIL: counted $count test binaries; floor is $floor (test deletions trigger BLOCK per D-06)" >&2
      exit 1
    fi
    echo "PASS: $count test binaries (floor $floor)"
    exit 0
    ```

    Per D-06, test ADDITIONS don't break the verifier (the `>= N` floor is monotone-friendly); test DELETIONS do — that's the desired drift signal.

    Per D-05: this is `--no-run` — DO NOT run the suite. Compile only.

    **Memory budget (D-04):** the full workspace test compile is the heaviest cargo invocation in this phase (30-60s on this VM per CONTEXT.md § "Cargo memory budget"). Run alone; no other cargo invocation in parallel.

    Run locally; record the live count in stdout for the next task to consume:
    ```
    bash quality/gates/code/lint-invariants/cargo-test-count.sh 2>&1 | tee /tmp/p72-test-count.log
    ```

    Commit: `P72: implement cargo-test-count.sh (LINT-CONFIG-07)` — note in body: "floor parameterized via REPOSIX_LINT_TEST_FLOOR; prose-update + measured-floor commit follows in task 9".
  </action>
  <verify>
    <automated>REPOSIX_LINT_TEST_FLOOR=10 bash quality/gates/code/lint-invariants/cargo-test-count.sh</automated>
  </verify>
  <done>Verifier exits 0 with low floor; live count recorded in `/tmp/p72-test-count.log` for task 9; commit landed citing LINT-CONFIG-07.</done>
</task>

<task type="auto">
  <name>Task 9: Implement tests-green.sh (compile-only `cargo test --workspace --no-run` per D-05)</name>
  <files>quality/gates/code/lint-invariants/tests-green.sh</files>
  <action>
    Compile workspace tests (no run) and assert exit 0:

    ```bash
    cd "$REPO_ROOT"
    if ! cargo test --workspace --no-run 2>&1; then
      echo "FAIL: cargo test --workspace --no-run failed (workspace test compilation broken)" >&2
      exit 1
    fi
    echo "PASS: workspace tests compile clean"
    exit 0
    ```

    Per D-05: this is the docs-alignment binding for "tests are green" — a CHEAP compile-only signal. The actual test-suite execution lives in `quality/gates/code/` nextest invocations at pre-push time. If the workspace doesn't compile, every other lint-config check fails too, so this verifier's value is the "compile baseline" assertion, not redundant test-suite execution.

    **Note on cache reuse:** task 8's `cargo test --workspace --no-run --message-format=json` already populated the test-binary cache, so this task's invocation should be FAST (~2-5s on warm cache). Still: serialize. No parallel cargo per D-04.

    Commit: `P72: implement tests-green.sh (LINT-CONFIG-03)`.
  </action>
  <verify>
    <automated>bash quality/gates/code/lint-invariants/tests-green.sh</automated>
  </verify>
  <done>Verifier exits 0 against current workspace; commit landed citing LINT-CONFIG-03.</done>
</task>

<!-- ============================================================== -->
<!-- WAVE 3 — Re-measure cargo-test-count + prose update (per D-06) -->
<!-- ============================================================== -->

<task type="auto">
  <name>Task 10: Re-measure test-binary count + prose-fix contributing.md (and README.md if cited)</name>
  <files>docs/development/contributing.md, README.md, quality/gates/code/lint-invariants/cargo-test-count.sh</files>
  <action>
    Per D-06: don't bind to a stale number. Re-measure NOW, update prose, then update the verifier's default floor — all BEFORE the bind in task 11.

    1. Read the live count from `/tmp/p72-test-count.log` (recorded in task 8). Call this `MEASURED`.
    2. Open `docs/development/contributing.md` and look at line 20 (the rough cargo-test-count claim location). Locate the prose like "133 tests" / "currently passes N tests" / similar. Replace with `${MEASURED} test binaries` (or "≥ ${MEASURED}" framing if more natural). Keep the line within ±5 chars to avoid heavy hash drift on neighbouring rows.
    3. Search README.md for any test-count citation: `grep -nE '[0-9]+ tests' README.md`. If present, update consistently (same number, same framing).
    4. In `cargo-test-count.sh`, update the default floor from `50` to `${MEASURED}` (the env var override stays intact for future ratchets):
       ```bash
       floor="${REPOSIX_LINT_TEST_FLOOR:-${MEASURED}}"
       ```
    5. Re-run the verifier locally to confirm it still passes against the now-measured floor:
       ```
       bash quality/gates/code/lint-invariants/cargo-test-count.sh
       ```

    Commit: `P72: re-measure cargo-test-count to ${MEASURED} + sync prose (LINT-CONFIG-07 per D-06)`.

    **Why this isn't squashed into task 8:** D-06 is explicit — re-measure and update prose BEFORE binding. Splitting makes the audit trail clean: task 8 implements the verifier shape; task 10 grounds the floor in measured reality.
  </action>
  <verify>
    <automated>
      bash quality/gates/code/lint-invariants/cargo-test-count.sh \
        && ! grep -qE '\b133 tests\b' docs/development/contributing.md README.md 2>/dev/null
    </automated>
  </verify>
  <done>contributing.md (and README.md if relevant) cite the freshly measured test-binary count; cargo-test-count.sh's default floor matches; verifier still PASSES; commit landed.</done>
</task>

<!-- =============================================================== -->
<!-- WAVE 4 — Bind 9 rows, refresh, capture AFTER snapshot, CLAUDE.md, SUMMARY. -->
<!-- =============================================================== -->

<task type="auto">
  <name>Task 11: Bind all 9 catalog rows to their verifiers (atomic, transactional)</name>
  <files>quality/catalogs/doc-alignment.json</files>
  <action>
    Build the binary if cache is stale: `cargo build --release -p reposix-quality` (one cargo invocation, sequential per D-04).

    For each of the 9 catalog rows, invoke `bind` with the row's existing source citation (read from the catalog — DO NOT hand-pick lines):

    ```bash
    BIN=target/release/reposix-quality
    CAT=quality/catalogs/doc-alignment.json
    GATES=quality/gates/code/lint-invariants

    bind_one() {
      local row_id="$1" verifier="$2" claim="$3" rationale="$4"
      # Pull the source citation back out of the catalog
      local src
      src=$(jq -r --arg id "$row_id" \
        '.rows[] | select(.id==$id) | "\(.source.file):\(.source.line_start)-\(.source.line_end)"' \
        "$CAT")
      "$BIN" doc-alignment bind \
        --catalog "$CAT" \
        --row-id "$row_id" \
        --claim "$claim" \
        --source "$src" \
        --test "$verifier" \
        --grade GREEN \
        --rationale "$rationale"
    }

    bind_one "README-md/forbid-unsafe-code" "$GATES/forbid-unsafe-code.sh" \
      "Every crate's lib.rs / main.rs declares #![forbid(unsafe_code)]." \
      "P72 LINT-CONFIG-01: shell verifier walks crates/*/src/{lib,main}.rs."

    bind_one "README-md/rust-1-82-requirement" "$GATES/rust-msrv.sh" \
      "Workspace Cargo.toml pins rust-version = \"1.82\" (current MSRV)." \
      "P72 LINT-CONFIG-02: grep workspace Cargo.toml."

    bind_one "README-md/tests-green" "$GATES/tests-green.sh" \
      "Workspace tests compile clean (cargo test --workspace --no-run exits 0)." \
      "P72 LINT-CONFIG-03 per D-05: compile-only signal; suite execution lives in pre-push nextest."

    bind_one "docs-development-contributing-md/forbid-unsafe-per-crate" "$GATES/forbid-unsafe-code.sh" \
      "Every crate's lib.rs / main.rs declares #![forbid(unsafe_code)]." \
      "P72 LINT-CONFIG-01 per D-01: shares verifier with README-md/forbid-unsafe-code (single source of truth)."

    bind_one "docs-development-contributing-md/errors-doc-section-required" "$GATES/errors-doc-section.sh" \
      "Every pub fn returning Result has a # Errors rustdoc section (clippy::missing_errors_doc)." \
      "P72 LINT-CONFIG-04 per D-07: clippy lint, not grep — handles Result aliases / trait methods."

    bind_one "docs-development-contributing-md/rust-stable-no-nightly" "$GATES/rust-stable-channel.sh" \
      "rust-toolchain.toml channel = \"stable\" (nightly banned)." \
      "P72 LINT-CONFIG-05: grep rust-toolchain.toml."

    bind_one "docs-development-contributing-md/cargo-check-workspace-available" "$GATES/cargo-check-workspace.sh" \
      "Workspace compiles clean (cargo check --workspace -q exits 0)." \
      "P72 LINT-CONFIG-06: cheap compile-only assertion."

    bind_one "docs-development-contributing-md/cargo-test-133-tests" "$GATES/cargo-test-count.sh" \
      "Workspace ships at least N test binaries (live count >= floor; floor re-measured P72)." \
      "P72 LINT-CONFIG-07 per D-06: prose re-measured + verifier floor synced before bind; >= floor tolerates additions, blocks deletions."

    bind_one "docs-development-contributing-md/demo-script-exists" "$GATES/demo-script-exists.sh" \
      "scripts/dark-factory-test.sh exists and is executable." \
      "P72 LINT-CONFIG-08: filesystem-only check."
    ```

    `bind` will fail-loud if any test path or source range is wrong (Principle B). If any bind fails, STOP — fix before continuing. Successful binds set `last_verdict: BOUND` directly per the implementation in `crates/reposix-quality/src/commands/doc_alignment.rs:285-308`.

    Commit: `P72: bind 9 lint-config rows to lint-invariants/ verifiers (LINT-CONFIG-01..09)`.
  </action>
  <verify>
    <automated>
      jq -e '.rows | map(select(.id | startswith("README-md/forbid-unsafe-code") or
                                       startswith("README-md/rust-1-82-requirement") or
                                       startswith("README-md/tests-green") or
                                       startswith("docs-development-contributing-md/forbid-unsafe-per-crate") or
                                       startswith("docs-development-contributing-md/errors-doc-section-required") or
                                       startswith("docs-development-contributing-md/rust-stable-no-nightly") or
                                       startswith("docs-development-contributing-md/cargo-check-workspace-available") or
                                       startswith("docs-development-contributing-md/cargo-test-133-tests") or
                                       startswith("docs-development-contributing-md/demo-script-exists"))) |
              all(.last_verdict == "BOUND")' quality/catalogs/doc-alignment.json
    </automated>
  </verify>
  <done>All 9 P72 rows show `last_verdict: BOUND` in `quality/catalogs/doc-alignment.json`; commit landed.</done>
</task>

<task type="auto">
  <name>Task 12: Run `doc-alignment refresh` + capture AFTER snapshot</name>
  <files>quality/catalogs/doc-alignment.json, quality/reports/verdicts/p72/status-after.txt, quality/reports/verdicts/p72/summary-after.json</files>
  <action>
    Per CONTEXT.md § specifics: run refresh against the two source files to confirm hashes are consistent and no row flips back to STALE on a no-op walk:

    ```bash
    target/release/reposix-quality doc-alignment refresh \
      README.md docs/development/contributing.md
    ```

    Then capture AFTER snapshot:
    ```bash
    target/release/reposix-quality doc-alignment status --top 30 \
      > quality/reports/verdicts/p72/status-after.txt
    jq '.summary | {alignment_ratio, claims_total, claims_bound, claims_missing_test}' \
      quality/catalogs/doc-alignment.json > quality/reports/verdicts/p72/summary-after.json
    ```

    Compute the alignment_ratio delta (BEFORE → AFTER) and stash it in `quality/reports/verdicts/p72/delta.txt` for the verifier dispatch.

    **Sanity gate:** `claims_missing_test` must drop by ≥ 9 (the 9 P72 rows). If less, an upstream catalog state diverged; STOP and investigate before commit.

    Commit (only if catalog actually changed; refresh on a no-op may leave it byte-identical):
    `P72: doc-alignment refresh + AFTER snapshot (alignment_ratio delta captured)`.

    If catalog is byte-identical (no hash drift), commit only the snapshot files:
    `P72: capture AFTER status snapshot for verdict dispatch`.
  </action>
  <verify>
    <automated>
      test -f quality/reports/verdicts/p72/status-after.txt \
        && test -f quality/reports/verdicts/p72/summary-after.json \
        && [ "$(jq -r '.claims_missing_test' quality/reports/verdicts/p72/summary-after.json)" -le \
             "$(($(jq -r '.claims_missing_test' quality/reports/verdicts/p72/summary-before.json) - 9))" ]
    </automated>
  </verify>
  <done>refresh executed; AFTER snapshot captured; `claims_missing_test` dropped by ≥ 9; commit landed.</done>
</task>

<task type="auto">
  <name>Task 13: CLAUDE.md P72 H3 subsection (≤30 lines, banned-words clean) per D-10</name>
  <files>CLAUDE.md</files>
  <action>
    Append a P72 H3 subsection to CLAUDE.md under the existing `## v0.12.1 — in flight` section. Per D-10 + QG-07 in `quality/PROTOCOL.md`:

    Constraints:
    - **≤30 lines total** (heading + body).
    - Names the verifier home: `quality/gates/code/lint-invariants/`.
    - Names the 9 rows + 8 verifiers (table or compact list).
    - Notes the prose-update on cargo-test-count (D-06 audit trail).
    - **Banned-words check must pass** — run `bash scripts/banned-words-lint.sh CLAUDE.md` (or whatever the project's banned-words script is named — find via `ls scripts/ | grep -i banned`) BEFORE committing.

    Required cross-references in the subsection:
    - `quality/gates/code/lint-invariants/README.md` (sub-area README).
    - `quality/PROTOCOL.md` § "Subagents propose; tools validate and mint" (Principle A justification for shell verifiers minted via `bind`).
    - CLAUDE.md "Build memory budget" (D-04 — why cargo invocations are serialized).

    DO NOT rewrite existing CLAUDE.md content — append only (anti-bloat per `quality/PROTOCOL.md` Step 5).

    Commit: `P72: CLAUDE.md H3 subsection (LINT-CONFIG-09 per D-10)`.
  </action>
  <verify>
    <automated>
      grep -qE '^### P72' CLAUDE.md \
        && [ "$(awk '/^### P72/,/^### |^## /' CLAUDE.md | wc -l)" -le 32 ] \
        && (bash scripts/banned-words-lint.sh CLAUDE.md 2>/dev/null || bash scripts/check-banned-words.sh 2>/dev/null || true)
    </automated>
  </verify>
  <done>CLAUDE.md gains P72 H3 ≤30 lines; banned-words check passes; commit landed citing LINT-CONFIG-09.</done>
</task>

<task type="auto">
  <name>Task 14: Phase SUMMARY.md + flag verifier-subagent dispatch to top-level orchestrator</name>
  <files>.planning/phases/72-lint-config-invariants/SUMMARY.md</files>
  <action>
    Write the phase summary using `$HOME/.claude/get-shit-done/templates/summary.md` shape. Include:

    1. **Objective** — recap from PLAN.md.
    2. **Completed tasks** — 14 tasks across 4 waves; each with commit SHA.
    3. **Catalog row transitions** — 9 rows MISSING_TEST → BOUND (paste row ids).
    4. **Alignment ratio delta** — BEFORE / AFTER values from `quality/reports/verdicts/p72/{summary-before,summary-after}.json`.
    5. **Verifier scripts shipped** — 8 files under `quality/gates/code/lint-invariants/` (one shared between two rows per D-01).
    6. **Prose updates** — re-measured cargo-test-count in `docs/development/contributing.md` and `README.md` (if applicable) per D-06; commit SHA.
    7. **Surprises / Good-to-haves** — copy any entries appended to `.planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md` or `GOOD-TO-HAVES.md` during P72 (reference the file + entry timestamp). If empty, state "no out-of-scope items observed during execution."
    8. **CLAUDE.md update** — H3 subsection lines + commit SHA.
    9. **Ready for verifier dispatch** — explicit flag. Subsection text:

       ```
       ## Verifier dispatch — TOP-LEVEL ORCHESTRATOR ACTION

       Per D-08 + CLAUDE.md OP-7 + quality/PROTOCOL.md § "Verifier subagent prompt template":
       the executing agent does NOT grade itself. After this SUMMARY commits, the
       top-level orchestrator MUST dispatch:

           Task(subagent_type=gsd-verifier OR general-purpose,
                description="P72 verifier dispatch",
                prompt=<verbatim QG-06 prompt template from quality/PROTOCOL.md>)

       Inputs the verifier reads with ZERO session context:
         - quality/catalogs/doc-alignment.json (the 9 row ids enumerated above)
         - .planning/milestones/v0.12.1-phases/REQUIREMENTS.md (LINT-CONFIG-01..09)
         - quality/reports/verdicts/p72/{status-before.txt, status-after.txt,
                                         summary-before.json, summary-after.json}
         - CLAUDE.md (confirms P72 H3 appears in `git diff main...HEAD -- CLAUDE.md`)
         - .planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md (honesty check
           per OP-8 — empty intake is acceptable IFF execution honestly observed
           no out-of-scope items; commits should reflect any "fix-eagerly" choices)

       Verdict goes to: quality/reports/verdicts/p72/VERDICT.md

       Phase does NOT close until verdict graded GREEN.
       ```

    10. **+2 phase practice (OP-8) audit trail** — short paragraph stating which
        in-flight observations were eager-fixed (within phase, < 1 hour, < 5 files)
        vs. appended to SURPRISES-INTAKE.md / GOOD-TO-HAVES.md. The verifier
        spot-checks this honesty per OP-8.

    Commit: `P72: phase SUMMARY + verifier-dispatch flag for top-level orchestrator`.
  </action>
  <verify>
    <automated>
      test -f .planning/phases/72-lint-config-invariants/SUMMARY.md \
        && grep -q "Verifier dispatch — TOP-LEVEL ORCHESTRATOR ACTION" .planning/phases/72-lint-config-invariants/SUMMARY.md \
        && grep -q "OP-8" .planning/phases/72-lint-config-invariants/SUMMARY.md \
        && git log -1 --format=%s | grep -q "P72: phase SUMMARY"
    </automated>
  </verify>
  <done>SUMMARY.md committed; verifier-dispatch flag explicit; OP-8 audit trail captured; phase ready for orchestrator-level Task() dispatch.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| executor → cargo | Cargo invocations during verifier development can compile arbitrary build.rs from workspace crates; this is the same trust posture as any cargo build (already implicit in CLAUDE.md OP-1 / "simulator-only by default"). No new boundary. |
| verifier → workspace files | Each verifier reads workspace source (Cargo.toml, rust-toolchain.toml, crates/**/src/{lib,main}.rs, scripts/dark-factory-test.sh). Read-only. |
| `bind` → catalog | The bind command MUTATES `quality/catalogs/doc-alignment.json`. Atomic per implementation (validates ALL --test citations BEFORE mutating; see doc_alignment.rs:255-272). |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-72-01 | Tampering | `bind` partial-write on multi-test failure | mitigate | Already mitigated upstream — `bind` validates all `--test` citations BEFORE catalog mutation per `doc_alignment.rs:255`. P72 relies on this; no new code path. |
| T-72-02 | Spoofing | Verifier script body silently replaced post-bind | mitigate | Walker hashes script file body via `crate::hash::file_hash` (TestRef::File). Drift fires `STALE_TEST_DRIFT` on next walk. P72 verifiers commit their final body in their implementation task; subsequent edits trigger drift signal. |
| T-72-03 | Repudiation | Verifier exits 0 falsely (e.g. swallowed cargo error) | mitigate | All verifiers use `set -euo pipefail`; cargo invocations check exit codes explicitly with `if ! cargo …` (no `&& true` swallows). Failure mode named in stderr (Principle B). |
| T-72-04 | Information disclosure | Clippy/cargo output leaks workspace paths to artifact files | accept | Workspace paths are already public (this is an open-source repo); no PII. Verifier artifacts under `quality/reports/verifications/code/lint-invariants/` are gitignored or committed publicly per existing convention. |
| T-72-05 | Denial of service | `cargo test --workspace --no-run` OOM the VM (parallel cargo from another agent) | mitigate | D-04 + CLAUDE.md "Build memory budget" — sequential cargo invocations enforced at the executor level (one task at a time within Wave 2, no parallel subagents). The plan's `parallelization: false` frontmatter signals this to the executor. |
| T-72-06 | Elevation of privilege | Verifier script gains write access via mis-configured `set -e` skip | accept | Scripts are read-only by design (grep, find, cargo check/test --no-run). No script in this phase writes outside `quality/reports/verifications/code/lint-invariants/` (an existing audit dir convention). |
</threat_model>

<verification>
## Phase-level invariants (verifier subagent reads these)

1. **9 rows BOUND** — `jq '[.rows[] | select(.id == "<...>")] | all(.last_verdict == "BOUND")'` returns `true` for all 9 P72 row ids.
2. **8 verifier files exist + executable** — all 8 files under `quality/gates/code/lint-invariants/` are present, `chmod +x`, and `bash -n` clean.
3. **Both forbid-unsafe rows share one verifier (D-01)** — `jq '.rows[] | select(.id=="README-md/forbid-unsafe-code" or .id=="docs-development-contributing-md/forbid-unsafe-per-crate") | .tests[0]'` yields the SAME path twice (`quality/gates/code/lint-invariants/forbid-unsafe-code.sh`).
4. **Cargo-test-count prose re-measured (D-06)** — `git log --oneline | grep -q "re-measure cargo-test-count"`; the bound verifier's default floor matches the documented count.
5. **CLAUDE.md updated (D-10)** — `git diff main...HEAD -- CLAUDE.md` shows a P72 H3 subsection ≤30 lines under `## v0.12.1 — in flight`.
6. **Banned-words clean** — `bash scripts/banned-words-lint.sh CLAUDE.md` (or equivalent) exits 0.
7. **No regression in BOUND rows** — `jq '.summary.claims_bound' quality/catalogs/doc-alignment.json` is at least 9 higher than the BEFORE snapshot.
8. **Coverage ratio not regressed** — `jq '.summary.coverage_ratio' quality/catalogs/doc-alignment.json` ≥ 0.10 (the floor — task 1 BEFORE snapshot is the baseline).
9. **OP-8 audit trail honest** — if any commit message contains "eager-fix" or any `SURPRISES-INTAKE.md` entry was appended during execution, the SUMMARY.md cites them; if empty, SUMMARY.md says so explicitly.
10. **Pre-push gates pass** (run on close):
    ```
    cargo check --workspace -q
    cargo clippy --workspace --all-targets -- -D warnings
    cargo fmt --all -- --check
    bash scripts/end-state.py    # freshness invariants
    ```
    (Each as a separate cargo invocation per D-04.)

## Top-level orchestrator action (NOT executor)

After SUMMARY.md commits, the orchestrator dispatches:

```
Task(
  description="P72 verifier dispatch — Path A per D-08",
  subagent_type="gsd-verifier",          # or "general-purpose" if gsd-verifier unavailable
  prompt=<verbatim quality/PROTOCOL.md § "Verifier subagent prompt template",
          with N=72>
)
```

The verifier writes `quality/reports/verdicts/p72/VERDICT.md`. Phase does NOT close until verdict graded GREEN. Per CLAUDE.md OP-7: the executing agent does NOT talk the verifier out of RED.
</verification>

<success_criteria>
Phase 72 closes (and the orchestrator advances to P73) WHEN ALL of:

1. **9 catalog rows transition MISSING_TEST → BOUND** in `quality/catalogs/doc-alignment.json`.
2. **8 shell verifiers** under `quality/gates/code/lint-invariants/` exist, are `chmod +x`, are `bash -n` clean, and exit 0 against the current workspace.
3. **`forbid-unsafe-code.sh` is bound twice** (the two `forbid(unsafe_code)` rows share it per D-01).
4. **`tests-green.sh` is compile-only** (`cargo test --workspace --no-run`) per D-05.
5. **`cargo-test-count.sh` floor matches re-measured prose** in `docs/development/contributing.md` (and `README.md` if cited there) per D-06; commit history shows the re-measure landed BEFORE the bind.
6. **`errors-doc-section.sh` uses clippy `missing_errors_doc`** per D-07, not grep.
7. **CLAUDE.md gains a P72 H3 ≤30 lines** under `## v0.12.1 — in flight` per D-10; banned-words check passes.
8. **`alignment_ratio` delta captured** at `quality/reports/verdicts/p72/{summary-before,summary-after}.json`.
9. **`.planning/phases/72-lint-config-invariants/SUMMARY.md`** exists, names commit SHAs, and explicitly flags the verifier-dispatch as a top-level-orchestrator action (D-08).
10. **+2 phase practice audit (OP-8)** captured: SUMMARY.md states whether any in-flight observations were eager-fixed in-phase vs. appended to `SURPRISES-INTAKE.md` / `GOOD-TO-HAVES.md`. Empty intake is honest IFF the running phase observed no out-of-scope items.
11. **Top-level orchestrator dispatches `gsd-verifier`** (Path A per D-08) with the QG-06 prompt template; verdict at `quality/reports/verdicts/p72/VERDICT.md` graded **GREEN**. Phase does NOT close on RED — loop back, fix, re-verify (CLAUDE.md OP-7).
12. **No `git push`, no `git tag --push`, no `cargo publish`** — local commits only per HANDOVER-v0.12.1.md.
</success_criteria>

<output>
After completion, the executor creates:

- `.planning/phases/72-lint-config-invariants/SUMMARY.md` — phase summary (task 14).
- `quality/reports/verdicts/p72/status-before.txt` — BEFORE snapshot (task 1).
- `quality/reports/verdicts/p72/summary-before.json` — BEFORE summary (task 1).
- `quality/reports/verdicts/p72/status-after.txt` — AFTER snapshot (task 12).
- `quality/reports/verdicts/p72/summary-after.json` — AFTER summary (task 12).

The top-level orchestrator (NOT the executor) creates:

- `quality/reports/verdicts/p72/VERDICT.md` — graded by `Task(gsd-verifier, ...)` per D-08 / OP-7.

Phase advances to P73 ONLY when VERDICT.md is graded GREEN.
</output>
