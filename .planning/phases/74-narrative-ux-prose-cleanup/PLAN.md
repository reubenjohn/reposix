---
phase: 74-narrative-ux-prose-cleanup
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - quality/gates/docs-alignment/install-snippet-shape.sh
  - quality/gates/docs-alignment/audit-trail-git-log.sh
  - quality/gates/docs-alignment/three-backends-tested.sh
  - quality/gates/docs-alignment/connector-matrix-on-landing.sh
  - quality/gates/docs-alignment/cli-spaces-smoke.sh
  - quality/catalogs/doc-alignment.json
  - docs/social/linkedin.md
  - CLAUDE.md
  - .planning/phases/74-narrative-ux-prose-cleanup/SUMMARY.md
  - quality/reports/verdicts/p74/status-before.txt
  - quality/reports/verdicts/p74/status-before.json
  - quality/reports/verdicts/p74/status-after.txt
  - quality/reports/verdicts/p74/status-after.json
autonomous: true
parallelization: false
gap_closure: false
cross_ai: false
task_count: 14
requirements:
  - NARRATIVE-RETIRE-01
  - NARRATIVE-RETIRE-02
  - NARRATIVE-RETIRE-03
  - NARRATIVE-RETIRE-04
  - UX-BIND-01
  - UX-BIND-02
  - UX-BIND-03
  - UX-BIND-04
  - UX-BIND-05
  - PROSE-FIX-01

must_haves:
  truths:
    - "5 catalog rows transition `MISSING_TEST` → `BOUND` against new shell verifiers under `quality/gates/docs-alignment/` (UX-BIND-01..05)."
    - "4 catalog rows transition `MISSING_TEST` → `RETIRE_PROPOSED` with the identical-format D-09 rationale (NARRATIVE-RETIRE-01..04)."
    - "5 verifier shell scripts exist under `quality/gates/docs-alignment/` (FLAT, NOT under a `verifiers/` subdir, matching the P73 `jira-adapter-shipped.sh` placement convention per D-01 + P73 SUMMARY decision row 2)."
    - "Each new verifier is 10–30 lines (D-02 TINY-shape rule), uses `set -euo pipefail`, prints a `PASS:` / `FAIL:` line on stderr, and exits 0 GREEN against the current workspace at the moment it is committed."
    - "`docs/social/linkedin.md:21` no longer contains `FUSE filesystem`; replaced one-line per D-08 with `git-native partial clone + git-remote-helper for REST issue trackers`. Existing BOUND row `docs/social/linkedin/token-reduction-92pct` re-hashes via walker after the post-edit `walk` and remains BOUND."
    - "ZERO new test files created under `crates/` (D-10). Phase is shell + prose + catalog only."
    - "CLAUDE.md gains a P74 H3 subsection ≤30 lines under `## v0.12.1 — in flight` (D-11); banned-words lint passes."
    - "OP-8 honesty audit: any out-of-scope discovery during execution (e.g. `reposix spaces --help` actually broken, connector matrix actually missing from `docs/index.md`) is either eager-fixed (< 1 hour, no new dep, no new file outside the planned set per D-12) OR appended to `.planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md` with severity + sketched-resolution per CLAUDE.md OP-8."
    - "BEFORE/AFTER snapshots captured at `quality/reports/verdicts/p74/status-{before,after}.{txt,json}` so the verifier subagent can grade ratio deltas with zero session context."
    - "Verifier subagent verdict at `quality/reports/verdicts/p74/VERDICT.md` is graded GREEN by an unbiased dispatched subagent (top-level orchestrator action; phase does NOT close until GREEN)."
  artifacts:
    - path: "quality/gates/docs-alignment/install-snippet-shape.sh"
      provides: "Shape-check verifier asserting `docs/index.md:19` matches the install-channel pattern (`curl|brew|cargo binstall|irm`); D-03."
      min_lines: 12
    - path: "quality/gates/docs-alignment/audit-trail-git-log.sh"
      provides: "Premise verifier asserting `git log --oneline | head -1` returns ≥1 line in the repo (D-04)."
      min_lines: 10
    - path: "quality/gates/docs-alignment/three-backends-tested.sh"
      provides: "Test-fn-existence verifier asserting `grep -c 'fn dark_factory_real_' crates/reposix-cli/tests/agent_flow_real.rs` is ≥3 (D-05)."
      min_lines: 12
    - path: "quality/gates/docs-alignment/connector-matrix-on-landing.sh"
      provides: "Landing-grep verifier asserting `docs/index.md` contains BOTH a `^## .*[Cc]onnector` heading AND ≥1 markdown table row (D-06)."
      min_lines: 15
    - path: "quality/gates/docs-alignment/cli-spaces-smoke.sh"
      provides: "CLI smoke verifier asserting `target/release/reposix spaces --help` exits 0 AND stdout contains `List all readable Confluence spaces` (D-07)."
      min_lines: 18
    - path: "quality/catalogs/doc-alignment.json"
      provides: "5 BOUND transitions + 4 RETIRE_PROPOSED transitions + 1 auto-rebind (linkedin row at line 21 via source_hash drift+heal)."
      contains: '"id": "docs/index/5-line-install"'
    - path: "docs/social/linkedin.md"
      provides: "Line-21 prose with `FUSE filesystem` removed and `git-native partial clone + git-remote-helper` introduced (D-08)."
      contains: "git-native partial clone"
    - path: "CLAUDE.md"
      provides: "P74 H3 subsection ≤30 lines under `## v0.12.1 — in flight`."
      contains: "### P74 — Narrative cleanup"
    - path: ".planning/phases/74-narrative-ux-prose-cleanup/SUMMARY.md"
      provides: "Phase summary with frontmatter (catalog deltas, decisions, verifier-dispatch instruction)."
      min_lines: 80
    - path: "quality/reports/verdicts/p74/status-before.txt"
      provides: "BEFORE snapshot — `target/release/reposix-quality doc-alignment status` text output captured pre-mutation."
      min_lines: 5
    - path: "quality/reports/verdicts/p74/status-after.txt"
      provides: "AFTER snapshot — `target/release/reposix-quality doc-alignment status` text output captured post-walk."
      min_lines: 5
  key_links:
    - from: "quality/catalogs/doc-alignment.json (UX-BIND-01..05 rows)"
      to: "quality/gates/docs-alignment/{install-snippet-shape,audit-trail-git-log,three-backends-tested,connector-matrix-on-landing,cli-spaces-smoke}.sh"
      via: "row.tests[] (TestRef::File path) populated by `target/release/reposix-quality doc-alignment bind --test <path>`"
      pattern: "quality/gates/docs-alignment/.*\\.sh"
    - from: "quality/catalogs/doc-alignment.json (NARRATIVE-RETIRE-01..04 rows)"
      to: "RowState::RetireProposed via propose-retire verb"
      via: "`target/release/reposix-quality doc-alignment propose-retire --row-id <id> --rationale <D-09 one-liner>`"
      pattern: '"last_verdict":\\s*"RETIRE_PROPOSED"'
    - from: "docs/social/linkedin.md:21"
      to: "Existing BOUND row `docs/social/linkedin/token-reduction-92pct`"
      via: "walker source_hash recompute after prose edit (D-08); expected: STALE_DOCS_DRIFT cleared on first post-edit walk"
      pattern: "git-native partial clone"
    - from: "CLAUDE.md `## v0.12.1 — in flight`"
      to: "P74 H3 subsection naming the 5 verifiers + 4 retires + linkedin edit"
      via: "QG-07 grounding rule"
      pattern: "### P74 — Narrative cleanup"
---

<objective>
Close the remaining 9 docs-alignment `MISSING_TEST` rows for the v0.12.1 narrative+UX cluster (4 propose-retires + 5 hash-shape binds) plus 1 linkedin prose drift fix. Catalog action count: 10. ZERO new Rust tests. ZERO new files under `crates/`. Five tiny shell verifiers + one one-line prose edit + nine catalog mutations + a `walk` to finalize.

Purpose: drives `claims_missing_test` from the post-P73 figure (9) toward 0 (4 land as `RETIRE_PROPOSED` awaiting owner-TTY confirm in HANDOVER step 1; 5 land as `BOUND`; the linkedin row stays `BOUND` after auto-rebind). After P74 + owner-TTY confirm-retire round, `alignment_ratio` rises by ≥0.013 vs BEFORE. P74 is the FASTEST of the autonomous run (per HANDOVER budget table: 1.5–2 hr wall-clock).

Output: 5 verifier shell scripts, 1 prose-edited markdown line, 1 catalog with 9 rewritten rows, 1 CLAUDE.md H3 update, 1 SUMMARY.md, BEFORE/AFTER snapshots under `quality/reports/verdicts/p74/`.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/HANDOVER-v0.12.1.md
@.planning/phases/74-narrative-ux-prose-cleanup/CONTEXT.md
@.planning/milestones/v0.12.1-phases/ROADMAP.md
@.planning/milestones/v0.12.1-phases/REQUIREMENTS.md
@.planning/phases/72-lint-config-invariants/SUMMARY.md
@.planning/phases/73-connector-contract-gaps/SUMMARY.md
@CLAUDE.md
@quality/PROTOCOL.md
@quality/catalogs/README.md
@crates/reposix-quality/src/commands/doc_alignment.rs
@quality/gates/docs-alignment/walk.sh
@quality/gates/docs-alignment/jira-adapter-shipped.sh
@docs/index.md
@docs/social/linkedin.md
@crates/reposix-cli/tests/agent_flow_real.rs
@crates/reposix-cli/src/spaces.rs

<interfaces>
<!-- Surfaces that the executor invokes verbatim. NO codebase exploration -->
<!-- needed beyond what's pasted here. -->

CLI surface (verified live in `crates/reposix-quality/src/commands/doc_alignment.rs`):

```text
target/release/reposix-quality doc-alignment bind \
    --row-id <ID> \
    --claim <STR> \
    --source "<file>:<lstart>-<lend>" \
    --test <PATH or file::fn> \
    --grade GREEN \
    --rationale <STR>

target/release/reposix-quality doc-alignment propose-retire \
    --row-id <ID> \
    --claim <STR> \
    --source "<file>:<lstart>-<lend>" \
    --rationale <STR>

target/release/reposix-quality doc-alignment walk
target/release/reposix-quality doc-alignment status [--json] [--top N] [--all]
```

Notes (load-bearing — these are the corrections P73 SUMMARY documented):

  1. The verb is `walk`, NOT `refresh`. There is no `refresh <doc>` per-file form.
     CONTEXT.md "Specifics" line referencing `refresh docs/index.md` is OUT OF DATE;
     use `walk` (it scans the whole catalog for hash drift).
  2. `--test` accepts a shell-script PATH (e.g. `quality/gates/docs-alignment/cli-spaces-smoke.sh`)
     OR a Rust `<file>::<fn>` citation. P74 uses the shell-path form.
  3. `--grade` MUST be `GREEN` (the bind verb rejects any other value).
  4. `bind` REPLACES the row's `tests` vector AND recomputes `source_hash` from
     the cited `<file>:<lstart>-<lend>`. Multi-test bindings can be done in a
     single invocation by repeating `--test`.

Target catalog row IDs (verified to exist in current `quality/catalogs/doc-alignment.json`):

  Propose-retire (4):
    - use-case-20-percent-rest-mcp           (line ~55  in catalog)
    - mcp-fixture-synthesized-not-live       (line ~93)
    - use-case-80-percent-routine-ops        (line ~212)
    - mcp-schema-discovery-100k-tokens       (line ~4731)

  Bind (5):
    - docs/index/5-line-install                                       (cat line ~131)
    - docs/index/audit-trail-git-log                                  (cat line ~824)
    - docs/index/tested-three-backends                                (cat line ~662)
    - planning-milestones-v0-11-0-phases-REQUIREMENTS-md/polish2-06-landing  (cat line ~2102)
    - planning-milestones-v0-8-0-phases-REQUIREMENTS-md/spaces-01     (cat line ~6901)

  Auto-rebind (NO direct mutation; walker handles via source_hash drift):
    - docs/social/linkedin/token-reduction-92pct                      (cat line ~6960)

`Source` reverse-lookup (each row's existing `source.file:line_start-line_end`)
MUST be recovered by the executor before invoking bind/propose-retire — the bind
verb requires `--source <file>:<lstart>-<lend>` matching the row's existing
citation (or its first cite if `Source::Multi`). Use:

```bash
jq -r '.rows[] | select(.id == "<ID>") | .source' quality/catalogs/doc-alignment.json
```

Sibling-verifier conventions (from P73 SUMMARY decision row 2 + the live tree):

  quality/gates/docs-alignment/
    walk.sh                       # wrapper; runs `reposix-quality doc-alignment walk`
    hash_test_fn                  # binary
    jira-adapter-shipped.sh       # 22 lines; sibling P73 verifier (FLAT placement)
    README.md                     # dimension home

P74 places its 5 new shell verifiers HERE — sibling files of `jira-adapter-shipped.sh`,
NOT inside any `verifiers/` subdir. Per D-01 with executor latitude AND P73 ground
truth: the convention IS flat.

Live source surfaces (verified during prep — re-confirm during Wave 1):

  docs/index.md:19 contains: `5-line install` ... `curl`, `brew`, `cargo binstall`, or `irm`
  crates/reposix-cli/tests/agent_flow_real.rs has fns:
      fn dark_factory_real_confluence  (line ~30)
      fn dark_factory_real_github      (line ~80)
      fn dark_factory_real_jira        (line ~135)
  crates/reposix-cli/src/spaces.rs       — confirmed-live `spaces` subcommand
  docs/social/linkedin.md:21              — current line begins
                                           `🚀 reposix — a working FUSE filesystem ...`
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Capture BEFORE snapshot + verify executor's prerequisites</name>
  <files>
    quality/reports/verdicts/p74/status-before.txt
    quality/reports/verdicts/p74/status-before.json
  </files>
  <action>
    Build `target/release/reposix-quality` if it does not already exist (single cargo invocation; per-crate build keeps memory budget happy):

    ```bash
    [[ -x target/release/reposix-quality ]] || cargo build -p reposix-quality --release
    ```

    Confirm the binary exposes the verbs the plan relies on:

    ```bash
    target/release/reposix-quality doc-alignment --help \
        | grep -E '^\s*(bind|propose-retire|walk|status)\b'
    ```

    Snapshot the catalog state BEFORE any mutation:

    ```bash
    mkdir -p quality/reports/verdicts/p74
    target/release/reposix-quality doc-alignment status \
        > quality/reports/verdicts/p74/status-before.txt
    target/release/reposix-quality doc-alignment status --json \
        > quality/reports/verdicts/p74/status-before.json
    ```

    Pre-resolve each target row's existing `source` citation so subsequent
    bind/propose-retire calls pass the matching `--source <file>:<lstart>-<lend>`
    string (the verb requires the new source to match-or-extend the existing one).
    Capture into a scratch file (NOT committed; use as in-task notes):

    ```bash
    for id in \
      use-case-20-percent-rest-mcp \
      use-case-80-percent-routine-ops \
      mcp-fixture-synthesized-not-live \
      mcp-schema-discovery-100k-tokens \
      docs/index/5-line-install \
      docs/index/audit-trail-git-log \
      docs/index/tested-three-backends \
      planning-milestones-v0-11-0-phases-REQUIREMENTS-md/polish2-06-landing \
      planning-milestones-v0-8-0-phases-REQUIREMENTS-md/spaces-01 \
      docs/social/linkedin/token-reduction-92pct ; do
      echo "=== $id ==="
      jq -r --arg id "$id" '.rows[] | select(.id == $id) | {id, source, claim, last_verdict}' \
        quality/catalogs/doc-alignment.json
    done
    ```

    Commit BEFORE snapshots only (catalog-first commit per quality/PROTOCOL.md
    Principle A; this commit defines the GREEN contract for P74 by recording
    where we started):

    ```bash
    git add quality/reports/verdicts/p74/status-before.{txt,json}
    git commit -m "p74: capture BEFORE snapshot (catalog-first; status pre-mutation)"
    ```
  </action>
  <verify>
    <automated>
      test -x target/release/reposix-quality && \
      test -s quality/reports/verdicts/p74/status-before.txt && \
      test -s quality/reports/verdicts/p74/status-before.json && \
      jq -e '.alignment_ratio' quality/reports/verdicts/p74/status-before.json
    </automated>
  </verify>
  <done>
    `target/release/reposix-quality` exists; both `status-before.{txt,json}`
    snapshots committed; the JSON parses and exposes `alignment_ratio`. The
    executor has confirmed (a) the 4 propose-retire row IDs and (b) the 5
    bind row IDs and (c) the linkedin auto-rebind row exist in the catalog
    with the expected `source` citations.
  </done>
</task>

<task type="auto">
  <name>Task 2: Scaffold 5 verifier shell stubs (FLAT, executable, exit 0 placeholders)</name>
  <files>
    quality/gates/docs-alignment/install-snippet-shape.sh
    quality/gates/docs-alignment/audit-trail-git-log.sh
    quality/gates/docs-alignment/three-backends-tested.sh
    quality/gates/docs-alignment/connector-matrix-on-landing.sh
    quality/gates/docs-alignment/cli-spaces-smoke.sh
  </files>
  <action>
    Create five shell scripts as STUBS — `set -euo pipefail`, the canonical
    `SCRIPT_DIR` / `REPO_ROOT` boilerplate from `jira-adapter-shipped.sh`,
    a brief comment header, and a placeholder `echo "TODO: wave-2 implementation"`
    + `exit 0`. Each stub is 10–15 lines.

    Mirror the sibling pattern verbatim. Comment header for each:

    ```bash
    #!/usr/bin/env bash
    # P74 UX-BIND-0X: <claim>
    # Bound to row `<row-id>` at `<file>:<line>` per CONTEXT.md D-XX.
    # Wave-1 stub — implementation lands in Wave 2.
    set -euo pipefail
    SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
    REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
    echo "STUB: P74 UX-BIND-0X verifier — implementation pending"
    exit 0
    ```

    Make all five executable: `chmod +x quality/gates/docs-alignment/{install-snippet-shape,audit-trail-git-log,three-backends-tested,connector-matrix-on-landing,cli-spaces-smoke}.sh`.

    Run each stub individually to confirm it exits 0 cleanly:

    ```bash
    for f in install-snippet-shape audit-trail-git-log three-backends-tested \
             connector-matrix-on-landing cli-spaces-smoke ; do
      bash "quality/gates/docs-alignment/$f.sh" || { echo "stub $f exited non-zero"; exit 1; }
    done
    ```

    Single commit per quality/PROTOCOL.md Principle A:

    ```bash
    git add quality/gates/docs-alignment/{install-snippet-shape,audit-trail-git-log,three-backends-tested,connector-matrix-on-landing,cli-spaces-smoke}.sh
    git commit -m "p74: scaffold 5 verifier stubs (FLAT placement; UX-BIND-01..05)"
    ```

    Per D-01 + P73 SUMMARY decision row 2: FLAT placement (sibling of
    `jira-adapter-shipped.sh`), NOT a `verifiers/` subdir. CONTEXT.md D-01
    mentions a `verifiers/` subdir but P73 ground truth supersedes it: the
    convention IS flat. Do NOT create a `verifiers/` subdir.
  </action>
  <verify>
    <automated>
      for f in install-snippet-shape audit-trail-git-log three-backends-tested connector-matrix-on-landing cli-spaces-smoke; do
        [[ -x "quality/gates/docs-alignment/$f.sh" ]] || { echo "missing: $f"; exit 1; }
        bash "quality/gates/docs-alignment/$f.sh" >/dev/null
      done
    </automated>
  </verify>
  <done>
    5 stub shell files exist under `quality/gates/docs-alignment/` (FLAT, NOT
    `verifiers/` subdir), all executable, all exit 0 when invoked, single
    catalog-first commit lands.
  </done>
</task>

<task type="auto">
  <name>Task 3: Implement install-snippet-shape.sh (UX-BIND-01, D-03)</name>
  <files>quality/gates/docs-alignment/install-snippet-shape.sh</files>
  <action>
    Replace the stub body with the D-03 shape check. The verifier MUST:

    1. Read the contents of `docs/index.md` line 19 exactly.
    2. Assert that line is one line (no continuation) and contains ALL FOUR
       channel-name tokens: `curl`, `brew`, `cargo binstall`, `irm`. Use
       `grep -F` for each token (literal match) so a future doc reflow that
       changes case or whitespace fires `STALE_DOCS_DRIFT` upstream.
    3. Print `PASS:` to stdout on success, `FAIL: <reason>` to stderr + `exit 1`
       on any miss.

    Reference snippet (10–25 lines target; D-02 TINY rule):

    ```bash
    #!/usr/bin/env bash
    # P74 UX-BIND-01 (D-03): docs/index.md:19 advertises the 5-line-install
    # claim. This verifier shape-checks line 19 — asserts the line lists
    # all four install channels (curl|brew|cargo binstall|irm). Body-hash
    # drift on either this verifier OR docs/index.md:19 fires STALE_DOCS_DRIFT
    # via the walker.
    set -euo pipefail
    SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
    REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
    DOC="${REPO_ROOT}/docs/index.md"
    LINE="$(sed -n '19p' "$DOC")"
    for tok in 'curl' 'brew' 'cargo binstall' 'irm'; do
      if ! printf '%s' "$LINE" | grep -qF -- "$tok"; then
        echo "FAIL: docs/index.md:19 missing install-channel token '$tok' — line was: $LINE" >&2
        exit 1
      fi
    done
    echo "PASS: docs/index.md:19 advertises curl|brew|cargo binstall|irm"
    exit 0
    ```

    Run locally to confirm GREEN against current workspace:

    ```bash
    bash quality/gates/docs-alignment/install-snippet-shape.sh
    ```

    Commit:

    ```bash
    git add quality/gates/docs-alignment/install-snippet-shape.sh
    git commit -m "p74: implement install-snippet-shape verifier (UX-BIND-01)"
    ```
  </action>
  <verify>
    <automated>bash quality/gates/docs-alignment/install-snippet-shape.sh</automated>
  </verify>
  <done>
    Verifier matches D-02 TINY shape (10–30 lines), exits 0 GREEN against
    `docs/index.md:19` as it stands today, single commit lands. Eager-fix
    candidate per D-12: if `docs/index.md:19` does NOT contain all four
    tokens at the moment of execution, the verifier WILL fail — pause,
    inspect, and either fix `docs/index.md:19` (eager) or append a
    SURPRISES-INTAKE.md entry (out-of-scope).
  </done>
</task>

<task type="auto">
  <name>Task 4: Implement audit-trail-git-log.sh (UX-BIND-02, D-04)</name>
  <files>quality/gates/docs-alignment/audit-trail-git-log.sh</files>
  <action>
    Replace the stub with the D-04 premise check: `git log --oneline | head -1`
    returns ≥1 line. The claim being verified is "the audit trail IS git log"
    (`docs/index.md:78`). The verifier asserts the premise via shell, NOT a
    workflow.

    Reference snippet (10–20 lines):

    ```bash
    #!/usr/bin/env bash
    # P74 UX-BIND-02 (D-04): docs/index.md:78 claims "the audit trail is git log".
    # Verifier asserts the claim's premise — the repo has at least one
    # commit observable via `git log --oneline`. If `git log` breaks for any
    # reason (e.g. shallow clone in CI without history, broken .git/), the
    # verifier fires and a maintainer reviews.
    set -euo pipefail
    SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
    REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
    cd "$REPO_ROOT"
    LINE_COUNT=$(git log --oneline 2>/dev/null | head -1 | wc -l)
    if (( LINE_COUNT < 1 )); then
      echo "FAIL: git log --oneline returned no commits — audit-trail premise broken" >&2
      exit 1
    fi
    echo "PASS: git log --oneline shows ≥1 commit (audit-trail premise holds)"
    exit 0
    ```

    Local smoke + commit:

    ```bash
    bash quality/gates/docs-alignment/audit-trail-git-log.sh
    git add quality/gates/docs-alignment/audit-trail-git-log.sh
    git commit -m "p74: implement audit-trail-git-log verifier (UX-BIND-02)"
    ```
  </action>
  <verify>
    <automated>bash quality/gates/docs-alignment/audit-trail-git-log.sh</automated>
  </verify>
  <done>
    Verifier exits 0 GREEN; D-02 TINY shape (10–30 lines); single commit lands.
  </done>
</task>

<task type="auto">
  <name>Task 5: Implement three-backends-tested.sh (UX-BIND-03, D-05)</name>
  <files>quality/gates/docs-alignment/three-backends-tested.sh</files>
  <action>
    Replace the stub with the D-05 fn-count check. Greps the test file for
    the three sanctioned `dark_factory_real_*` test functions. Cheap; rebinds
    on rename via the walker's test-body-hash check.

    Reference snippet (10–20 lines):

    ```bash
    #!/usr/bin/env bash
    # P74 UX-BIND-03 (D-05): docs/index.md "tested against three real backends"
    # claim asserts 3 sanctioned `dark_factory_real_*` test functions live in
    # crates/reposix-cli/tests/agent_flow_real.rs. Verifier counts test fns;
    # asserts ≥3.
    set -euo pipefail
    SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
    REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
    TEST_FILE="${REPO_ROOT}/crates/reposix-cli/tests/agent_flow_real.rs"
    if [[ ! -f "$TEST_FILE" ]]; then
      echo "FAIL: $TEST_FILE missing — sanctioned-real-backends suite gone" >&2
      exit 1
    fi
    COUNT=$(grep -c 'fn dark_factory_real_' "$TEST_FILE" || true)
    if (( COUNT < 3 )); then
      echo "FAIL: only $COUNT dark_factory_real_* fns found in $TEST_FILE (need ≥3)" >&2
      exit 1
    fi
    echo "PASS: $COUNT dark_factory_real_* fns present (sanctioned-real-backends suite intact)"
    exit 0
    ```

    Local smoke + commit:

    ```bash
    bash quality/gates/docs-alignment/three-backends-tested.sh
    git add quality/gates/docs-alignment/three-backends-tested.sh
    git commit -m "p74: implement three-backends-tested verifier (UX-BIND-03)"
    ```
  </action>
  <verify>
    <automated>bash quality/gates/docs-alignment/three-backends-tested.sh</automated>
  </verify>
  <done>
    Verifier exits 0 GREEN; reports ≥3 fns; D-02 TINY shape; single commit lands.
  </done>
</task>

<task type="auto">
  <name>Task 6: Implement connector-matrix-on-landing.sh (UX-BIND-04, D-06)</name>
  <files>quality/gates/docs-alignment/connector-matrix-on-landing.sh</files>
  <action>
    Replace the stub with the D-06 dual-grep: assert `docs/index.md` contains
    BOTH (a) a `^## .*[Cc]onnector` heading AND (b) ≥1 markdown table row
    matching `^\| .* \| .* \|`. Catches "connector matrix accidentally
    deleted from landing" — a regression mode the v0.11.0 polish2 row was
    written to detect.

    Reference snippet (15–25 lines):

    ```bash
    #!/usr/bin/env bash
    # P74 UX-BIND-04 (D-06): polish2-06-landing claim asserts the connector
    # capability matrix lives on docs/index.md. Verifier dual-greps for
    # (a) a "## ...connector..." H2 heading AND (b) at least one markdown
    # table row. Both required. Fires STALE_DOCS_DRIFT if either disappears.
    set -euo pipefail
    SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
    REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
    DOC="${REPO_ROOT}/docs/index.md"
    if ! grep -qE '^## .*[Cc]onnector' "$DOC"; then
      echo "FAIL: docs/index.md has no '## ...connector...' heading — capability matrix likely missing" >&2
      exit 1
    fi
    if ! grep -qE '^\| .* \| .* \|' "$DOC"; then
      echo "FAIL: docs/index.md has no markdown table rows — capability matrix table missing" >&2
      exit 1
    fi
    echo "PASS: docs/index.md has connector heading + table row"
    exit 0
    ```

    Local smoke + commit:

    ```bash
    bash quality/gates/docs-alignment/connector-matrix-on-landing.sh
    git add quality/gates/docs-alignment/connector-matrix-on-landing.sh
    git commit -m "p74: implement connector-matrix-on-landing verifier (UX-BIND-04)"
    ```

    D-12 eager-resolution gate: if EITHER grep fails (heading or table missing
    from landing), pause. Two paths:
      (a) Eager-fix: if the matrix is restorable in < 30 min from
          `docs/reference/testing-targets.md` (which has it) — restore it inline.
          This is in-scope per D-12.
      (b) Out-of-scope: append a SURPRISES-INTAKE.md entry "P74 verifier
          discovered connector matrix missing from landing" and proceed
          with stub (verifier exits 1; row stays MISSING_TEST). DO NOT bind
          a failing verifier.
  </action>
  <verify>
    <automated>bash quality/gates/docs-alignment/connector-matrix-on-landing.sh</automated>
  </verify>
  <done>
    Verifier exits 0 GREEN against current `docs/index.md`; D-02 TINY shape;
    single commit lands. If GREEN cannot be achieved, OP-8 intake entry
    written and that fact noted in the phase SUMMARY.
  </done>
</task>

<task type="auto">
  <name>Task 7: Implement cli-spaces-smoke.sh (UX-BIND-05, D-07)</name>
  <files>quality/gates/docs-alignment/cli-spaces-smoke.sh</files>
  <action>
    Replace the stub with the D-07 CLI smoke. Asserts the binary is built
    AND `target/release/reposix spaces --help` exits 0 AND the stdout
    contains `List all readable Confluence spaces`.

    Reference snippet (15–25 lines):

    ```bash
    #!/usr/bin/env bash
    # P74 UX-BIND-05 (D-07): spaces-01 row claims `reposix spaces` subcommand
    # exists and is reachable. Verifier asserts (a) the release binary is
    # built, (b) `reposix spaces --help` exits 0, and (c) the help text
    # mentions "List all readable Confluence spaces" (header line in
    # crates/reposix-cli/src/spaces.rs).
    set -euo pipefail
    SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
    REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
    BIN="${REPO_ROOT}/target/release/reposix"
    if [[ ! -x "$BIN" ]]; then
      echo "FAIL: ${BIN} not built — run 'cargo build -p reposix-cli --release' first" >&2
      exit 1
    fi
    OUT="$("$BIN" spaces --help 2>&1)" || {
      echo "FAIL: 'reposix spaces --help' exited non-zero. Output: $OUT" >&2
      exit 1
    }
    if ! printf '%s' "$OUT" | grep -qF 'List all readable Confluence spaces'; then
      echo "FAIL: 'reposix spaces --help' stdout missing expected header. Got: $OUT" >&2
      exit 1
    fi
    echo "PASS: reposix spaces --help exits 0 with expected header"
    exit 0
    ```

    Build the cli release if needed (single cargo invocation; per-crate per
    memory budget):

    ```bash
    [[ -x target/release/reposix ]] || cargo build -p reposix-cli --release
    bash quality/gates/docs-alignment/cli-spaces-smoke.sh
    git add quality/gates/docs-alignment/cli-spaces-smoke.sh
    git commit -m "p74: implement cli-spaces-smoke verifier (UX-BIND-05)"
    ```

    D-12 eager-resolution gate: if the binary builds but `reposix spaces
    --help` is broken (e.g. command panics, returns non-zero, missing the
    header text), append SURPRISES-INTAKE.md and STOP — do NOT bind a
    failing verifier. The user's prep notes say the subcommand was healthy
    at handover; if regressed in the autonomous run, that's a real surprise.
  </action>
  <verify>
    <automated>
      [[ -x target/release/reposix ]] || cargo build -p reposix-cli --release && \
      bash quality/gates/docs-alignment/cli-spaces-smoke.sh
    </automated>
  </verify>
  <done>
    Verifier exits 0 GREEN; `reposix` cli release binary present; D-02 TINY
    shape (≤30 lines); single commit lands.
  </done>
</task>

<task type="auto">
  <name>Task 8: Propose-retire 4 narrative rows (NARRATIVE-RETIRE-01..04, D-09)</name>
  <files>quality/catalogs/doc-alignment.json</files>
  <action>
    Use the D-09 identical-format one-liner rationale across all 4 rows.
    Rationale template:

      "{narrative_id} — qualitative design framing; no behavioral assertion possible. Decided 2026-04-29."

    Run propose-retire 4 times. Use `jq` to recover each row's existing
    `claim` and `source` (the verb requires those exact strings):

    ```bash
    for id in \
      use-case-20-percent-rest-mcp \
      use-case-80-percent-routine-ops \
      mcp-fixture-synthesized-not-live \
      mcp-schema-discovery-100k-tokens ; do
      CLAIM=$(jq -r --arg id "$id" '.rows[] | select(.id == $id) | .claim' \
              quality/catalogs/doc-alignment.json)
      SRC=$(jq -r --arg id "$id" \
            '.rows[] | select(.id == $id) | "\(.source.file):\(.source.line_start)-\(.source.line_end)"' \
            quality/catalogs/doc-alignment.json)
      RATIONALE="$id — qualitative design framing; no behavioral assertion possible. Decided 2026-04-29."
      target/release/reposix-quality doc-alignment propose-retire \
        --row-id "$id" \
        --claim  "$CLAIM" \
        --source "$SRC" \
        --rationale "$RATIONALE"
    done
    ```

    Verify all 4 rows transitioned:

    ```bash
    jq -r '.rows[] | select(.id | test("use-case-20|use-case-80|mcp-fixture-synth|mcp-schema-discovery")) | "\(.id) → \(.last_verdict)"' \
      quality/catalogs/doc-alignment.json
    # expect: 4 lines all ending "→ RETIRE_PROPOSED"
    ```

    Single commit appending all 4 retires (CONTEXT.md "Plan shape suggestion"
    leaves this to planner; one commit minimizes catalog churn and matches
    P73's batched-bind precedent):

    ```bash
    git add quality/catalogs/doc-alignment.json
    git commit -m "p74: propose-retire 4 narrative rows (NARRATIVE-RETIRE-01..04, D-09)"
    ```

    DO NOT run `confirm-retire` — that's owner-TTY-only per HANDOVER step 1
    + CLAUDE.md OP. The verb env-guards against `CLAUDE_AGENT_CONTEXT` and
    will refuse from this session anyway.
  </action>
  <verify>
    <automated>
      [[ "$(jq -r '[.rows[] | select(.id | test("use-case-20-percent-rest-mcp|use-case-80-percent-routine-ops|mcp-fixture-synthesized-not-live|mcp-schema-discovery-100k-tokens")) | select(.last_verdict == "RETIRE_PROPOSED")] | length' quality/catalogs/doc-alignment.json)" == "4" ]]
    </automated>
  </verify>
  <done>
    All 4 narrative rows have `last_verdict: RETIRE_PROPOSED` + the D-09
    identical-format rationale; single commit lands; `confirm-retire` was
    NOT invoked.
  </done>
</task>

<task type="auto">
  <name>Task 9: Apply linkedin prose fix (PROSE-FIX-01, D-08)</name>
  <files>docs/social/linkedin.md</files>
  <action>
    Replace ONE line at `docs/social/linkedin.md:21` per D-08. The current
    line begins:

      `🚀 reposix — a working FUSE filesystem + git-remote-helper for issue trackers.`

    Replace the FIRST sentence with:

      `🚀 reposix — a working git-native partial clone + git-remote-helper for REST issue trackers.`

    Preserve the rest of the line verbatim (token-reduction-92pct claim,
    metric, etc. — only the FIRST sentence changes per D-08).

    Use `sed -i` with a pattern targeting "FUSE filesystem" specifically
    (avoids accidentally touching another line). Confirm before commit:

    ```bash
    # BEFORE
    sed -n '21p' docs/social/linkedin.md
    # apply
    sed -i 's|FUSE filesystem + git-remote-helper for issue trackers|git-native partial clone + git-remote-helper for REST issue trackers|' docs/social/linkedin.md
    # AFTER
    sed -n '21p' docs/social/linkedin.md
    grep -n 'FUSE filesystem' docs/social/linkedin.md  # expect: empty
    ```

    Single commit:

    ```bash
    git add docs/social/linkedin.md
    git commit -m "p74: drop FUSE framing from linkedin.md:21 (PROSE-FIX-01)"
    ```

    The walker (run in Task 11) recomputes `source_hash` for the existing
    BOUND row `docs/social/linkedin/token-reduction-92pct` at line 21. The
    walker emits one transient `STALE_DOCS_DRIFT` then heals on the next
    `walk` invocation IF the test_body_hash didn't change (it won't —
    the bound test is unrelated to the prose at line 21's first sentence).

    EXPECTED behavior to confirm in Task 11: row remains BOUND or transitions
    BOUND→STALE_DOCS_DRIFT→BOUND across two walks. If after the second walk
    the row is STALE_DOCS_DRIFT, that's the P75 hash-overwrite bug surfacing
    here — note in SURPRISES-INTAKE.md and proceed (the bug is being fixed
    in the next phase, P75).
  </action>
  <verify>
    <automated>
      grep -n 'FUSE filesystem' docs/social/linkedin.md && exit 1 || true
      grep -q 'git-native partial clone' docs/social/linkedin.md
    </automated>
  </verify>
  <done>
    `docs/social/linkedin.md` no longer contains `FUSE filesystem`; line 21
    contains `git-native partial clone`; single commit lands.
  </done>
</task>

<task type="auto">
  <name>Task 10: Bind 5 UX rows to the Wave-2 verifiers (UX-BIND-01..05)</name>
  <files>quality/catalogs/doc-alignment.json</files>
  <action>
    Bind each row to its corresponding shell verifier path. Use the row's
    existing `claim` + `source` (recovered via `jq`); pass the verifier path
    as `--test`; pass `--grade GREEN` (the only value bind accepts) and a
    short rationale citing P74 D-XX.

    Multi-row bind sweep:

    ```bash
    declare -A BINDINGS=(
      ["docs/index/5-line-install"]="quality/gates/docs-alignment/install-snippet-shape.sh|D-03"
      ["docs/index/audit-trail-git-log"]="quality/gates/docs-alignment/audit-trail-git-log.sh|D-04"
      ["docs/index/tested-three-backends"]="quality/gates/docs-alignment/three-backends-tested.sh|D-05"
      ["planning-milestones-v0-11-0-phases-REQUIREMENTS-md/polish2-06-landing"]="quality/gates/docs-alignment/connector-matrix-on-landing.sh|D-06"
      ["planning-milestones-v0-8-0-phases-REQUIREMENTS-md/spaces-01"]="quality/gates/docs-alignment/cli-spaces-smoke.sh|D-07"
    )
    for ROW in "${!BINDINGS[@]}"; do
      IFS='|' read -r VERIFIER DECISION <<< "${BINDINGS[$ROW]}"
      CLAIM=$(jq -r --arg id "$ROW" '.rows[] | select(.id == $id) | .claim' \
              quality/catalogs/doc-alignment.json)
      SRC=$(jq -r --arg id "$ROW" \
            '.rows[] | select(.id == $id) | "\(.source.file):\(.source.line_start)-\(.source.line_end)"' \
            quality/catalogs/doc-alignment.json)
      target/release/reposix-quality doc-alignment bind \
        --row-id "$ROW" \
        --claim  "$CLAIM" \
        --source "$SRC" \
        --test   "$VERIFIER" \
        --grade  GREEN \
        --rationale "Hash-shape bind per P74 ${DECISION}; verifier under quality/gates/docs-alignment/."
    done
    ```

    Verify all 5 transitioned:

    ```bash
    jq -r '.rows[] | select(.id | test("docs/index/5-line-install|docs/index/audit-trail-git-log|docs/index/tested-three-backends|polish2-06-landing|spaces-01")) | "\(.id) → \(.last_verdict)"' \
      quality/catalogs/doc-alignment.json
    # expect: 5 lines all ending "→ BOUND"
    ```

    Single commit:

    ```bash
    git add quality/catalogs/doc-alignment.json
    git commit -m "p74: bind 5 UX rows to hash-shape verifiers (UX-BIND-01..05)"
    ```

    `Source::Multi` upgrade does NOT happen here — every target row is
    `Source::Single` and the bind verb's behavior is to keep the citation
    as-is when the new source matches the existing one (which it does,
    because we recovered the source via jq).
  </action>
  <verify>
    <automated>
      [[ "$(jq -r '[.rows[] | select(.id == "docs/index/5-line-install" or .id == "docs/index/audit-trail-git-log" or .id == "docs/index/tested-three-backends" or .id == "planning-milestones-v0-11-0-phases-REQUIREMENTS-md/polish2-06-landing" or .id == "planning-milestones-v0-8-0-phases-REQUIREMENTS-md/spaces-01") | select(.last_verdict == "BOUND")] | length' quality/catalogs/doc-alignment.json)" == "5" ]]
    </automated>
  </verify>
  <done>
    All 5 target rows have `last_verdict: BOUND`; each row's `tests[0]`
    points at the new verifier shell; single commit lands.
  </done>
</task>

<task type="auto">
  <name>Task 11: Run walk + capture AFTER snapshot</name>
  <files>
    quality/reports/verdicts/p74/status-after.txt
    quality/reports/verdicts/p74/status-after.json
    quality/catalogs/doc-alignment.json
  </files>
  <action>
    Run the walker to recompute hashes for the linkedin auto-rebind row +
    confirm the 5 newly-bound rows hold BOUND post-walk:

    ```bash
    target/release/reposix-quality doc-alignment walk || WALK_EXIT=$?
    echo "walk exit: ${WALK_EXIT:-0}"
    ```

    Walk's exit code is non-zero iff any row is in a blocking state
    (STALE_DOCS_DRIFT, MISSING_TEST, etc.). Post-P74 the catalog STILL has
    9 + 23 = 32+ blocking rows from earlier phases not yet drained
    (RETIRE_PROPOSED awaiting owner confirm, MISSING_TEST in other clusters
    not in P74's scope), so non-zero exit is EXPECTED. The verifier subagent
    grades on the row-id transitions in scope, NOT on `walk` global exit.

    Capture AFTER snapshots:

    ```bash
    target/release/reposix-quality doc-alignment status \
        > quality/reports/verdicts/p74/status-after.txt
    target/release/reposix-quality doc-alignment status --json \
        > quality/reports/verdicts/p74/status-after.json
    ```

    Confirm linkedin row's source_hash recomputed AND row remains BOUND
    (i.e. the auto-rebind path D-08 promised actually worked):

    ```bash
    jq -r '.rows[] | select(.id == "docs/social/linkedin/token-reduction-92pct") | {id, last_verdict, source_hash}' \
      quality/catalogs/doc-alignment.json
    ```

    Expected: `last_verdict: BOUND`, `source_hash` is a fresh sha256 (matches
    the post-edit prose at line 21). If `last_verdict` is `STALE_DOCS_DRIFT`,
    that's the P75 hash-overwrite bug surfacing on a `Source::Single` row
    (it shouldn't, since the bug per HANDOVER §4 is `Source::Multi`-specific
    — but document it in SURPRISES-INTAKE.md if observed).

    Capture the catalog mutations AND the snapshots in a single commit:

    ```bash
    git add quality/catalogs/doc-alignment.json \
            quality/reports/verdicts/p74/status-after.{txt,json}
    git commit -m "p74: walk + AFTER snapshot (linkedin auto-rebind confirmed)"
    ```
  </action>
  <verify>
    <automated>
      target/release/reposix-quality doc-alignment walk >/dev/null 2>&1 || true
      test -s quality/reports/verdicts/p74/status-after.txt && \
      test -s quality/reports/verdicts/p74/status-after.json && \
      [[ "$(jq -r '.rows[] | select(.id == "docs/social/linkedin/token-reduction-92pct") | .last_verdict' quality/catalogs/doc-alignment.json)" == "BOUND" ]]
    </automated>
  </verify>
  <done>
    `walk` ran; AFTER snapshots captured; linkedin row remains BOUND with a
    refreshed `source_hash`; commit lands. Any STALE_DOCS_DRIFT on the
    linkedin row was either auto-healed by a second walk or noted in
    SURPRISES-INTAKE.md per OP-8.
  </done>
</task>

<task type="auto">
  <name>Task 12: Update CLAUDE.md (P74 H3 subsection ≤30 lines, D-11)</name>
  <files>CLAUDE.md</files>
  <action>
    Append a P74 H3 subsection under `## v0.12.1 — in flight` (immediately
    after the P73 H3 added in the previous phase). MUST be ≤30 lines per
    D-11. MUST pass `bash scripts/banned-words-lint.sh`.

    Required content (bullet shape, target ≤25 lines body + heading):

    ```markdown
    ### P74 — Narrative cleanup + UX bindings + linkedin prose fix

    Closed the docs-alignment narrative+UX cluster: 4 propose-retires (qualitative
    design rows), 5 hash-shape binds (UX claims on docs/index.md + REQUIREMENTS
    rows), and a one-line linkedin.md prose fix dropping the v0.4-era FUSE
    framing. Five new shell verifiers under `quality/gates/docs-alignment/`
    (FLAT placement, sibling of `jira-adapter-shipped.sh`):

    - `install-snippet-shape.sh` — asserts `docs/index.md:19` lists curl/brew/cargo binstall/irm.
    - `audit-trail-git-log.sh` — asserts `git log --oneline | head -1` returns ≥1 line.
    - `three-backends-tested.sh` — counts `dark_factory_real_*` fns in `agent_flow_real.rs`; asserts ≥3.
    - `connector-matrix-on-landing.sh` — greps `docs/index.md` for `## ...connector` heading + table.
    - `cli-spaces-smoke.sh` — asserts `target/release/reposix spaces --help` exits 0 + expected header.

    Each verifier is 10–30 lines (D-02 TINY shape). The shape rule:
    body-hash drift on either prose OR verifier file fires `STALE_DOCS_DRIFT`
    via the walker; an agent reviews. No deep workflow logic.

    Catalog deltas: `claims_missing_test` 9 → 0 (5 BOUND + 4 RETIRE_PROPOSED
    awaiting owner-TTY confirm-retire per HANDOVER step 1); `claims_bound`
    +5; `alignment_ratio` rises by ~+0.013 in this phase. Linkedin row at
    line 21 auto-rebound via walker source_hash recompute after the prose edit.

    No new test files in `crates/` (D-10). Phase is shell + prose + catalog only.
    ```

    Apply + lint + commit:

    ```bash
    # locate the section, append the H3 subsection
    bash scripts/banned-words-lint.sh
    git add CLAUDE.md
    git commit -m "p74: CLAUDE.md H3 subsection (≤30 lines per D-11)"
    ```

    The H3 inserts AFTER the P73 H3 (so order in `## v0.12.1 — in flight`
    is: P66 → P67..P71 (deferred) → P72 → P73 → **P74**).
  </action>
  <verify>
    <automated>
      grep -q '^### P74' CLAUDE.md && \
      bash scripts/banned-words-lint.sh && \
      [[ "$(awk '/^### P74/,/^### P[78]/' CLAUDE.md | head -n -1 | wc -l)" -le 30 ]]
    </automated>
  </verify>
  <done>
    P74 H3 subsection lands ≤30 lines under `## v0.12.1 — in flight`;
    `bash scripts/banned-words-lint.sh` passes; single commit lands.
  </done>
</task>

<task type="auto">
  <name>Task 13: Write SUMMARY.md with frontmatter</name>
  <files>.planning/phases/74-narrative-ux-prose-cleanup/SUMMARY.md</files>
  <action>
    Write the phase SUMMARY following the P73 SUMMARY shape verbatim
    (frontmatter at top with `phase`, `plan`, `subsystem`, `tags`,
    `dependency-graph`, `tech-stack`, `key-files`, `decisions`, `metrics`;
    body sections include "Objective", "Completed Tasks" table with commit
    SHAs, "Catalog row transitions" table, "Alignment ratio delta" table,
    "Surprises / Good-to-haves (OP-8 audit trail)", "CLAUDE.md update",
    "Verifier dispatch — TOP-LEVEL ORCHESTRATOR ACTION", "Self-Check").

    Required tables to populate (the executor copies the template from
    `.planning/phases/73-connector-contract-gaps/SUMMARY.md` and fills in
    P74 numbers):

    - Catalog deltas: read from `quality/reports/verdicts/p74/status-{before,after}.json`.
    - Commit SHAs: `git log --oneline -20` should show 11–12 P74 commits
      (1 BEFORE snapshot + 1 stub-scaffold + 5 verifier impls + 4
      catalog mutations [retires, prose, binds, walk-AFTER batched as
      planned] + CLAUDE.md + this SUMMARY = 11 minimum).
    - Decisions section: cite D-01..D-12 from CONTEXT.md and the divergence
      from D-01's nominal `verifiers/` subdir (chose FLAT per D-01 latitude
      + P73 ground truth).

    The SUMMARY MUST include the verbatim verifier-dispatch instruction (Path A
    via `Task` from top-level orchestrator, prompt template from
    `quality/PROTOCOL.md` § "Verifier subagent prompt template" with N=74).
    Inputs the verifier reads with ZERO session context:

      - quality/catalogs/doc-alignment.json (5 BOUND + 4 RETIRE_PROPOSED row IDs)
      - .planning/milestones/v0.12.1-phases/REQUIREMENTS.md (NARRATIVE-RETIRE-01..04, UX-BIND-01..05, PROSE-FIX-01)
      - quality/reports/verdicts/p74/status-{before,after}.{txt,json}
      - CLAUDE.md (confirm P74 H3 in `git diff main...HEAD -- CLAUDE.md`)
      - .planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md (OP-8 honesty check)
      - The 5 new verifier shell scripts (each must exit 0 GREEN at verifier-runtime)
      - docs/social/linkedin.md (line 21 — confirm FUSE removed, git-native added)

    Verdict goes to `quality/reports/verdicts/p74/VERDICT.md`. Phase does
    NOT close until graded GREEN.

    Commit:

    ```bash
    git add .planning/phases/74-narrative-ux-prose-cleanup/SUMMARY.md
    git commit -m "p74: phase SUMMARY (catalog deltas + verifier dispatch instruction)"
    ```
  </action>
  <verify>
    <automated>
      test -s .planning/phases/74-narrative-ux-prose-cleanup/SUMMARY.md && \
      grep -q '^---$' .planning/phases/74-narrative-ux-prose-cleanup/SUMMARY.md && \
      grep -q 'phase: 74-narrative-ux-prose-cleanup' .planning/phases/74-narrative-ux-prose-cleanup/SUMMARY.md && \
      grep -q 'verifier subagent' .planning/phases/74-narrative-ux-prose-cleanup/SUMMARY.md
    </automated>
  </verify>
  <done>
    SUMMARY.md exists with full frontmatter, catalog-deltas table, verifier-dispatch
    instruction, OP-8 honesty audit, and self-check section. Single commit lands.
  </done>
</task>

<task type="auto">
  <name>Task 14: Phase-close handoff + verifier-dispatch instruction</name>
  <files>
    .planning/phases/74-narrative-ux-prose-cleanup/SUMMARY.md
  </files>
  <action>
    P74's executor work is now done: 13 atomic commits land catalog mutations,
    verifier scripts, prose fix, CLAUDE.md update, and SUMMARY. The phase is
    NOT closed yet — closure requires an unbiased verifier subagent grading
    `quality/reports/verdicts/p74/VERDICT.md` GREEN.

    The executing agent (this gsd-executor) does NOT grade itself per
    CLAUDE.md OP-7 + quality/PROTOCOL.md § "Verifier subagent prompt template".
    Depth-2 subagent dispatch is forbidden (gsd-executor lacks `Task`), so the
    verifier MUST be dispatched by the TOP-LEVEL orchestrator that spawned
    this executor.

    ACTIONS for the executor before exiting:

    1. Run a final sanity sweep on the artifact set:

       ```bash
       # 5 verifiers green
       for f in install-snippet-shape audit-trail-git-log three-backends-tested \
                connector-matrix-on-landing cli-spaces-smoke ; do
         bash "quality/gates/docs-alignment/$f.sh"
       done

       # catalog state
       jq -r '[.rows[] | select(
                .id == "docs/index/5-line-install"
             or .id == "docs/index/audit-trail-git-log"
             or .id == "docs/index/tested-three-backends"
             or .id == "planning-milestones-v0-11-0-phases-REQUIREMENTS-md/polish2-06-landing"
             or .id == "planning-milestones-v0-8-0-phases-REQUIREMENTS-md/spaces-01"
             or .id == "use-case-20-percent-rest-mcp"
             or .id == "use-case-80-percent-routine-ops"
             or .id == "mcp-fixture-synthesized-not-live"
             or .id == "mcp-schema-discovery-100k-tokens"
           ) | "\(.id) → \(.last_verdict)"] | .[]' \
         quality/catalogs/doc-alignment.json
       # expected: 5 BOUND + 4 RETIRE_PROPOSED
       ```

    2. Confirm the SUMMARY.md (Task 13) ends with the verbatim
       verifier-dispatch instruction block including the row IDs, the
       artifact paths the verifier reads, and the `quality/reports/verdicts/p74/VERDICT.md`
       output path.

    3. Final executor message to the top-level orchestrator MUST include:
       (a) commit count and the SHA range of P74 commits
           (`git log --oneline | grep -E "^.{7,8} p74:"`),
       (b) the explicit dispatch instruction:

           "P74 implementation complete. Top-level orchestrator MUST now
            dispatch gsd-verifier (Path A via `Task`) with the QG-06 prompt
            template from quality/PROTOCOL.md, N=74. Verifier writes
            `quality/reports/verdicts/p74/VERDICT.md`. Phase does NOT close
            until verdict is GREEN."

       (c) the BEFORE/AFTER alignment_ratio delta read from
           `quality/reports/verdicts/p74/status-{before,after}.json`.

    No commit in this task — Task 13 already committed SUMMARY.md. This task
    is purely a hand-off contract: confirm the artifact set is healthy and
    construct the final orchestrator-facing message.
  </action>
  <verify>
    <automated>
      bash quality/gates/docs-alignment/install-snippet-shape.sh && \
      bash quality/gates/docs-alignment/audit-trail-git-log.sh && \
      bash quality/gates/docs-alignment/three-backends-tested.sh && \
      bash quality/gates/docs-alignment/connector-matrix-on-landing.sh && \
      bash quality/gates/docs-alignment/cli-spaces-smoke.sh && \
      [[ "$(jq -r '[.rows[] | select(.id == "docs/index/5-line-install" or .id == "docs/index/audit-trail-git-log" or .id == "docs/index/tested-three-backends" or .id == "planning-milestones-v0-11-0-phases-REQUIREMENTS-md/polish2-06-landing" or .id == "planning-milestones-v0-8-0-phases-REQUIREMENTS-md/spaces-01") | select(.last_verdict == "BOUND")] | length' quality/catalogs/doc-alignment.json)" == "5" ]] && \
      [[ "$(jq -r '[.rows[] | select(.id == "use-case-20-percent-rest-mcp" or .id == "use-case-80-percent-routine-ops" or .id == "mcp-fixture-synthesized-not-live" or .id == "mcp-schema-discovery-100k-tokens") | select(.last_verdict == "RETIRE_PROPOSED")] | length' quality/catalogs/doc-alignment.json)" == "4" ]]
    </automated>
  </verify>
  <done>
    All 5 verifiers exit 0; 5 BOUND + 4 RETIRE_PROPOSED row transitions
    confirmed; executor's final orchestrator-facing message names the
    dispatch instruction explicitly. Phase does NOT close in this task —
    closure happens when the verifier subagent (dispatched by the
    top-level orchestrator) grades `quality/reports/verdicts/p74/VERDICT.md`
    GREEN.
  </done>
</task>

</tasks>

<threat_model>

## Trust Boundaries

| Boundary                                    | Description                                                                                                                                |
| ------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| `quality/catalogs/doc-alignment.json` ↔ verb invocations | The catalog is the milestone-completion contract. Mutations from the dispatched CLI must be append-or-overwrite-by-row-id only.            |
| Shell verifier files ↔ docs prose           | Each verifier hashes its own body + the cited source; drift on either fires `STALE_DOCS_DRIFT`. The verifier files are themselves catalog citations. |
| `docs/social/linkedin.md:21` ↔ public posts | The line is reproduced on linkedin.com. Stale architectural framing (FUSE) misleads the audience; PROSE-FIX-01 closes the drift.            |
| `target/release/reposix spaces --help`      | The CLI surface is reachable via shell from the verifier. The verifier asserts an idempotent help-text fingerprint, not a network call.    |

## STRIDE Threat Register

| Threat ID | Category | Component                                     | Disposition | Mitigation Plan                                                                                                                                                                                                |
| --------- | -------- | --------------------------------------------- | ----------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| T-74-01   | T        | catalog rows (4 NARRATIVE-RETIRE)             | mitigate    | `propose-retire` writes `last_verdict: RETIRE_PROPOSED`, NOT `RETIRED`. Final retirement requires owner-TTY `confirm-retire`; agent context is env-guarded (`CLAUDE_AGENT_CONTEXT` blocks the verb). HANDOVER §1. |
| T-74-02   | T        | shell verifier files (5 new)                  | mitigate    | Each verifier is `set -euo pipefail`, no env-var injection, no eval, no `curl` — only `grep`, `sed`, `wc`, `git log`, `target/release/reposix spaces --help`. Walker hashes the verifier body; tampering fires drift.    |
| T-74-03   | I        | `docs/social/linkedin.md`                     | accept      | Public-by-design; the FUSE→git-native edit is a clarity fix, not a confidentiality mitigation.                                                                                                                |
| T-74-04   | D        | `cli-spaces-smoke.sh` invokes a release binary | accept      | The verifier runs `reposix spaces --help`. `--help` is a documented, side-effect-free path. Pre-push budget is <60s; the help invocation is sub-second.                                                       |
| T-74-05   | E        | `confirm-retire` (NOT INVOKED in P74)         | mitigate    | Agent never runs `confirm-retire`. The 4 NARRATIVE-RETIRE rows stay `RETIRE_PROPOSED` until owner runs the TTY-only sweep per HANDOVER §1. Plan reinforces this in Task 8.                                     |
| T-74-06   | S        | catalog rationale strings (D-09)              | mitigate    | All 4 rationale strings use the identical D-09 template; no executor-supplied free-form text that could be misinterpreted as a row-id or test path. Rationale is a string field; no parsing.                  |

</threat_model>

<verification>
Phase verifications across all 14 tasks:

1. Both BEFORE snapshots and both AFTER snapshots exist + parse as JSON / non-empty text.
2. 5 shell verifiers exist under `quality/gates/docs-alignment/` (FLAT), all executable, all exit 0 against the current workspace at the moment they're committed.
3. 4 narrative rows in `quality/catalogs/doc-alignment.json` have `last_verdict: RETIRE_PROPOSED`.
4. 5 UX rows have `last_verdict: BOUND` and each row's `tests[0]` points at the correct verifier path.
5. `docs/social/linkedin.md` no longer contains `FUSE filesystem`; line 21 contains `git-native partial clone`.
6. `docs/social/linkedin/token-reduction-92pct` row remains `BOUND` after the post-edit walk.
7. CLAUDE.md gains a P74 H3 subsection ≤30 lines under `## v0.12.1 — in flight`.
8. `bash scripts/banned-words-lint.sh` exits 0.
9. SUMMARY.md exists with full frontmatter + verifier-dispatch instruction.
10. Verifier subagent verdict at `quality/reports/verdicts/p74/VERDICT.md` is GREEN.
</verification>

<success_criteria>
P74 ships when:

- 9 catalog rows have transitioned per the row-state contract (5 → BOUND, 4 → RETIRE_PROPOSED).
- The linkedin BOUND row at line 21 was auto-rebound by the walker (source_hash refreshed; row stays BOUND).
- 5 new shell verifiers live FLAT under `quality/gates/docs-alignment/`, all 10–30 lines, all exit 0 GREEN against current workspace.
- ZERO new test files under `crates/` (D-10).
- BEFORE/AFTER status snapshots committed under `quality/reports/verdicts/p74/`.
- CLAUDE.md gains a P74 H3 subsection ≤30 lines, banned-words-clean.
- `.planning/phases/74-narrative-ux-prose-cleanup/SUMMARY.md` written with full frontmatter + verifier-dispatch instruction.
- An unbiased gsd-verifier subagent dispatched from the top-level orchestrator grades `quality/reports/verdicts/p74/VERDICT.md` GREEN.
- Phase wall-clock target: 1.5–2 hours per HANDOVER budget table. Being substantially faster is a signal of skipped steps (CLAUDE.md update, verifier dispatch, SUMMARY) — re-check.
</success_criteria>

<output>
After completion, the phase artifact set is:

- `.planning/phases/74-narrative-ux-prose-cleanup/SUMMARY.md` (this phase's run record).
- `quality/reports/verdicts/p74/{status-before,status-after}.{txt,json}` (BEFORE/AFTER snapshots).
- `quality/reports/verdicts/p74/VERDICT.md` (verifier subagent verdict — written by the dispatched verifier, NOT by the executor).
- 5 verifier shell scripts under `quality/gates/docs-alignment/` (committed).
- `docs/social/linkedin.md` (one-line prose edit committed).
- `quality/catalogs/doc-alignment.json` (9 row transitions + 1 auto-rebind committed).
- `CLAUDE.md` (P74 H3 subsection ≤30 lines committed).

Commit count target: 11–12 atomic commits, each citing one or more of the
14 task IDs. Phase closes only when verdict is GREEN.
</output>
