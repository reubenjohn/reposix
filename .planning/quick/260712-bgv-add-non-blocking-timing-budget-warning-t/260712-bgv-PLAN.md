---
phase: quick-260712-bgv
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - .githooks/pre-commit
  - .githooks/pre-push
autonomous: true
requirements: [quick-260712-bgv]

must_haves:
  truths:
    - "A pre-commit run slower than 3s emits one WARN line to stderr and still exits 0"
    - "A pre-push run slower than 90s emits one WARN line to stderr and still exits 0"
    - "A fast run (under threshold) emits NO warning"
    - "A runner FAIL still exits nonzero and does so BEFORE any warning prints"
    - "No catalog row is added — hook-script inline comments are not doc-alignment claims"
  artifacts:
    - path: ".githooks/pre-commit"
      provides: "SECONDS-based timing wrapper + 3s WARN around the pre-commit runner"
      contains: "pre-commit: WARN"
    - path: ".githooks/pre-push"
      provides: "SECONDS-based timing wrapper + 90s WARN around the pre-push runner"
      contains: "pre-push: WARN"
  key_links:
    - from: ".githooks/pre-commit"
      to: "python3 quality/runners/run.py --cadence pre-commit"
      via: "SECONDS=0 before invocation, warn AFTER RUNNER_EXIT check, BEFORE exec global"
      pattern: "SECONDS=0"
    - from: ".githooks/pre-push"
      to: "python3 quality/runners/run.py --cadence pre-push"
      via: "SECONDS=0 before invocation, warn AFTER RUNNER_EXIT check, BEFORE global delegation"
      pattern: "SECONDS=0"
---

<objective>
Add a non-blocking timing-budget warning to `.githooks/pre-commit` and
`.githooks/pre-push`, wrapping the existing `python3 quality/runners/run.py
--cadence <cadence>` invocation. When wall-clock exceeds the documented cadence
budget plus slack (pre-commit >3s, budget ~2s; pre-push >90s, budget ~60s), the
hook prints ONE stderr line pointing the reader at a likely-new whole-repo gate —
without changing exit codes or any other hook behavior.

Purpose: quality/CLAUDE.md § Cadences documents pre-commit <2s / pre-push <60s as
FIXED whole-repo costs that do NOT scale with diff size. A silent creep past budget
(e.g. someone adds another kcov-style full-corpus gate) is exactly the failure the
doc warns about. A cheap runtime tripwire surfaces it at the moment it happens.

Output: two edited hook files, a few lines each, no new script.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@./CLAUDE.md
@quality/CLAUDE.md

<doc_alignment_conclusion>
CONFIRMED during planning — NO catalog row is required for the new inline comment.

`quality/catalogs/doc-alignment.json` binds 393 claims sourced ONLY from:
  docs/ (280), .planning/ (88), README.md (17), crates/ (11 — Rust doc-comment prose).
ZERO rows bind `.githooks/` files or any `.sh`/hook script. The existing pre-commit
and pre-push files already carry extensive descriptive comments (fixture-identity
backstop, composition rules, recursion guards) with no catalog rows tracking them.
A new inline comment describing the timing warning gets the same treatment: it is a
hook-script implementation comment, not a tracked doc claim. Do NOT add a row.

The cadence/scaling claims this warning implements already live in CLAUDE.md and
quality/CLAUDE.md § Cadences ("Runtime does NOT scale with diff size" block, updated
2026-07-12) — those ARE the doc-alignment surface; no further doc change is needed.
</doc_alignment_conclusion>

<interfaces>
Existing pre-commit runner block (`.githooks/pre-commit`, `set -uo pipefail`, lines 71-85):
```bash
# (2) Quality Gates runner -- propagates exit code.
python3 "$REPO_ROOT/quality/runners/run.py" --cadence pre-commit
RUNNER_EXIT=$?
if [[ $RUNNER_EXIT -ne 0 ]]; then
  exit "$RUNNER_EXIT"
fi

# (3) Optional personal add-on chain.
GLOBAL="$HOME/.git-hooks/pre-commit"
if [[ -x "$GLOBAL" ]] && ! grep -q "delegate to repo-local" "$GLOBAL" 2>/dev/null; then
  exec "$GLOBAL" "$@"
fi
exit 0
```
NOTE: step (3) uses `exec` — it REPLACES the process. The warning MUST print BEFORE
step (3), i.e. between the RUNNER_EXIT check and the `GLOBAL=` line.

Existing pre-push runner block (`.githooks/pre-push`, `set -euo pipefail`, lines 57-76):
```bash
# (2) Quality Gates runner -- propagates exit code.
python3 "$REPO_ROOT/quality/runners/run.py" --cadence pre-push
RUNNER_EXIT=$?
if [[ $RUNNER_EXIT -ne 0 ]]; then
  exit "$RUNNER_EXIT"
fi

# (3) Optional personal add-on chain. ...
GLOBAL="$HOME/.git-hooks/pre-push"
if [[ -n "$(git config core.hooksPath 2>/dev/null)" ]] && [[ -x "$GLOBAL" ]]; then
  printf '%s\n' "$stdin_buf" | "$GLOBAL" "$@"
fi
```
NOTE: pre-push runs under `set -e`, so a runner FAIL aborts the script (nonzero exit)
BEFORE `RUNNER_EXIT=$?` — the warning is only reached on success. Place the warning
between the RUNNER_EXIT check and the `GLOBAL=` line (step 3).
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Wrap pre-commit and pre-push runners with a non-blocking SECONDS timing warning</name>
  <files>.githooks/pre-commit, .githooks/pre-push</files>
  <action>
Add `SECONDS=0` on the line immediately BEFORE each `python3 "$REPO_ROOT/quality/runners/run.py" --cadence <cadence>` invocation (do not disturb the existing bare invocation + `RUNNER_EXIT=$?` pattern — that pattern is deliberate; see the header comments). Capture elapsed by reading the bash `SECONDS` builtin.

In `.githooks/pre-commit`: insert the warning block AFTER the `if [[ $RUNNER_EXIT -ne 0 ]]; then exit "$RUNNER_EXIT"; fi` block and BEFORE the `# (3)` / `GLOBAL="$HOME/.git-hooks/pre-commit"` line (step 3 uses `exec`, which would swallow a later warning). Threshold 3s (budget ~2s). Use exactly:
```bash
# (2b) Non-blocking timing-budget tripwire. quality/CLAUDE.md § Cadences documents
# pre-commit <2s as a FIXED whole-repo cost (does NOT scale with diff size). A creep
# past budget usually means a new whole-repo gate, not a bigger diff. Warn only; never
# touch the exit code (a real FAIL already exited above). Same class of hook-internal
# comment as the blocks above — not a doc-alignment claim, no catalog row.
elapsed=$SECONDS
if [[ $elapsed -gt 3 ]]; then
  echo "pre-commit: WARN — took ${elapsed}s (budget ~2s, quality/CLAUDE.md § Cadences) — check for a new whole-repo gate before assuming diff size is the cause." >&2
fi
```

In `.githooks/pre-push`: insert the analogous block AFTER the `if [[ $RUNNER_EXIT -ne 0 ]]; then exit "$RUNNER_EXIT"; fi` block and BEFORE the `# (3)` / `GLOBAL="$HOME/.git-hooks/pre-push"` line. Threshold 90s (budget ~60s). Use exactly:
```bash
# (2b) Non-blocking timing-budget tripwire. quality/CLAUDE.md § Cadences documents
# pre-push <60s as a FIXED whole-repo cost (does NOT scale with diff size — a one-line
# commit and a 500-file commit pay the same tax). A creep past budget usually means a
# new whole-repo gate (another kcov-style full-corpus walk), not a bigger diff. Warn
# only; never touch the exit code (a real FAIL already exited above / aborted under
# set -e). Not a doc-alignment claim, no catalog row.
elapsed=$SECONDS
if [[ $elapsed -gt 90 ]]; then
  echo "pre-push: WARN — took ${elapsed}s (budget ~60s, quality/CLAUDE.md § Cadences) — check for a new whole-repo gate before assuming diff size is the cause." >&2
fi
```

Do NOT change any existing line, exit code, or the fixture-identity / cred-hygiene / delegation logic. Do NOT add a doc-alignment catalog row (see <doc_alignment_conclusion>).
  </action>
  <verify>
    <automated>bash -n .githooks/pre-commit && bash -n .githooks/pre-push && grep -q 'pre-commit: WARN' .githooks/pre-commit && grep -q 'pre-push: WARN' .githooks/pre-push && grep -c 'SECONDS=0' .githooks/pre-commit | grep -qx 1 && grep -c 'SECONDS=0' .githooks/pre-push | grep -qx 1 && echo VERIFY-OK</automated>
  </verify>
  <done>
Both hooks pass `bash -n`; each contains exactly one `SECONDS=0` and its WARN line; the WARN block sits after the RUNNER_EXIT check and before the global-delegation step in both files.
  </done>
</task>

<task type="auto">
  <name>Task 2: Prove threshold logic (fires when slow, silent when fast, never mutates exit code) and no-regression on a real run</name>
  <files>.githooks/pre-commit, .githooks/pre-push</files>
  <action>
Behavioral proof that does not require a genuinely slow runner. Two checks:

(a) Threshold branch — run a minimal bash snippet that mirrors the inserted logic to confirm the boundary: elapsed=91 → prints "pre-push: WARN"; elapsed=90 → prints nothing (strictly greater-than, not >=); elapsed=4 → prints "pre-commit: WARN"; elapsed=2 → nothing. Confirm the branch never alters `$?`.

(b) No-regression — run the real pre-commit hook against the working tree and confirm it still exits 0 and (because real pre-commit is ~0.7s, well under 3s) prints NO warning. Do not run the full pre-push hook here (it is a ~55s whole-repo walk); its logic is identical to pre-commit's and covered by (a) plus the `bash -n` check in Task 1.
  </action>
  <verify>
    <automated>bash -c 'w(){ local e=$1; if [[ $e -gt 90 ]]; then echo "pre-push: WARN ${e}"; fi; if [[ $e -gt 3 ]]; then echo "pre-commit: WARN ${e}"; fi; }; out=$(w 91; w 90; w 2); echo "$out" | grep -q "pre-push: WARN 91" && ! echo "$out" | grep -q "WARN 90" && ! echo "$out" | grep -q "WARN 2" && echo BRANCH-OK'</automated>
    <automated>bash .githooks/pre-commit; ec=$?; test $ec -eq 0 && echo "PRECOMMIT-EXIT-OK"</automated>
  </verify>
  <done>
Threshold branch fires strictly above budget+slack and stays silent at/below it; the live pre-commit hook exits 0 with no warning on a fast run, proving the warning is non-blocking and correctly gated.
  </done>
</task>

</tasks>

<verification>
- `bash -n` clean on both hooks (no syntax regression).
- Both hooks contain their WARN line + exactly one `SECONDS=0`.
- Threshold logic: strictly `>` budget+slack (3s / 90s); silent otherwise.
- Real pre-commit run exits 0, emits no warning (~0.7s < 3s).
- No catalog row added; no other file changed.
</verification>

<success_criteria>
- pre-commit warns once on stderr when >3s, pre-push warns once when >90s.
- Warning never changes the exit code; a runner FAIL exits nonzero first.
- Existing fixture-identity, cred-hygiene, runner, and delegation logic untouched.
- No doc-alignment catalog row (hook comments are not tracked claims — confirmed).
</success_criteria>

<output>
After completion, create `.planning/quick/260712-bgv-add-non-blocking-timing-budget-warning-t/260712-bgv-SUMMARY.md`.
</output>
