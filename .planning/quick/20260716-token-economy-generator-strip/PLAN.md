---
quick_id: 260716-f6o
slug: token-economy-generator-strip
title: "Fix-it-twice for 5a5dd29 — strip retirement-history narrative from the perf-gate GENERATOR"
status: ready
created: 2026-07-16
type: quick
autonomous: true
files_modified:
  - quality/gates/perf/bench_token_economy_captures.py
  - .planning/PROGRESS.md
# docs/benchmarks/token-economy.md is TOUCHED (regen writes it in place) but MUST end
# byte-identical to HEAD and MUST NOT be staged — see Task 3/Task 6.
must_haves:
  truths:
    - "The perf-gate generator no longer templates the '## What retired the old 89.1% / 85.5% figures' section."
    - "Offline regen (bench_token_economy.py --offline) produces docs/benchmarks/token-economy.md byte-for-byte identical to the committed HEAD version."
    - "The stray +12-line working-tree diff on docs/benchmarks/token-economy.md is discarded, not committed."
    - "The commit stages ONLY the generator + PROGRESS.md (+ this quick PLAN/SUMMARY); the doc is NOT staged; catalog files are NOT touched."
  artifacts:
    - path: "quality/gates/perf/bench_token_economy_captures.py"
      provides: "generator template with the retired-narrative section removed"
    - path: ".planning/PROGRESS.md"
      provides: "provenance note (generator fix + working-tree discard)"
  key_links:
    - from: "quality/gates/perf/bench_token_economy_captures.py::render_token_economy_markdown"
      to: "docs/benchmarks/token-economy.md"
      via: "bench_token_economy.py --offline main() -> RESULTS.write_text(md)"
---

# Quick Task 260716-f6o — strip retirement-history narrative from the perf-gate GENERATOR

**Fix-it-twice** for owner ruling commit **5a5dd29** ("strip retirement-history narrative
from user-facing docs"). That commit deliberately removed the `## What retired the old
89.1% / 85.5% figures` section from `docs/benchmarks/token-economy.md`, but left the
**generator** that produces the doc still templating that section. The **P115 phase-close
gate run** regenerated the doc in place and silently re-added the stripped section,
leaving a dirty working tree (`+12` lines).

**Manager-established provenance (fixed, do not re-litigate):** this is an **accidental
regression vector**, NOT a deliberate override of the owner ruling. The scope is fixed —
strip the generator template, prove regen matches the committed doc, discard the stray
working-tree diff, note it, commit + push. **Do not expand scope.**

## Ground truth (verified read-only during planning — authoritative)

- **Committed doc has NO retired section.** `git show HEAD:docs/benchmarks/token-economy.md`
  jumps straight from `Equivalently: the MCP arm costs **~4.0x** ... for the same result.`
  (one blank line) to `## What this DOES measure`.
  Committed sha256 = **`5620699b15da82b8123404b7038ae634259891a17517b1f1cba23e598f364fcf`**.
- **Working tree is dirty** with exactly the `+12`-line re-add of the retired section
  (section header + blank + 9-line paragraph + trailing blank).
- **The generator templates the section.** In
  `quality/gates/perf/bench_token_economy_captures.py`, `render_token_economy_markdown()`
  returns an f-string whose body contains the `## What retired the old 89.1% / 85.5%
  figures` header + paragraph, sitting between the `carries **~{...}x** ... for the same
  result.` line and the `## What this DOES measure` header.
- **Regen is fully offline, no cargo.** `python3 quality/gates/perf/bench_token_economy.py
  --offline` → `main()` reads committed `benchmarks/captures/*.json`, computes medians,
  renders via `render_token_economy_markdown()`, and `RESULTS.write_text(md)` writes
  `docs/benchmarks/token-economy.md` deterministically. No `ANTHROPIC_API_KEY`, no
  network, no cargo. `main()` also idempotency-guards (prints `(unchanged ...)` when the
  file already matches).
- **Bound catalog rows are unaffected.** The active BOUND rows in
  `quality/catalogs/doc-alignment.json` for this doc are the live four-axis numeric claims
  (`output-reduction-94-percent`, `cost-reduction-75-percent`,
  `live-github-capture-methodology`, etc.). The retired-narrative *prose* was never a
  bound claim, and the `89.1%`/`85.5%`/`4883`/`531` rows are already `RETIRE_PROPOSED`.
  Because the committed doc bytes will NOT change, no bound-claim hash drifts and **no
  rebind is required.**

## Constraints (encode + honor — non-negotiable)

- **No cargo needed** for this task. Do NOT run any `cargo` invocation. (If the `git push`
  pre-push hook builds something as part of its normal ~2min run, that is the single
  sanctioned invocation — never run a parallel cargo alongside it.)
- **Targeted staging ONLY.** `git add <specific files>` — NEVER `git add -A` or `git add .`.
- **Do NOT stage `docs/benchmarks/token-economy.md`** — after Task 3 it must show NO diff
  vs HEAD; committing any change to it would revert the shipped owner ruling.
- **Do NOT edit `.planning/MANAGER-HANDOVER.md`.**
- **Do NOT touch any `quality/catalogs/*.json`** — no rebind is required (see ground truth);
  only touch a catalog if Task 4 genuinely surfaces a hash mismatch, and even then flag
  first (see Task 4).
- **Uncommitted = didn't happen** — commit + push before stopping.

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
</execution_context>

<context>
@quality/gates/perf/bench_token_economy_captures.py
@quality/gates/perf/bench_token_economy.py
@.planning/PROGRESS.md
</context>

<tasks>

<task type="auto">
  <name>Task 1: Strip the retired-narrative section from the generator template</name>
  <files>quality/gates/perf/bench_token_economy_captures.py</files>
  <action>
    In `render_token_economy_markdown()`, the returned f-string currently contains the
    `## What retired the old 89.1% / 85.5% figures` header + its 9-line paragraph,
    positioned between the `carries ... for the same result.` line and the
    `## What this DOES measure` header. Remove that section cleanly.

    Use an exact-match Edit. Replace this block (verbatim from the f-string body):

        carries **~{red['input_multiple']:.2f}x** the total input-context for the same result.

        ## What retired the old 89.1% / 85.5% figures

        The previous token-economy figures (an **89.1%** headline and a per-backend
        **85.5%** GitHub number) came from a *different, synthetic* methodology: running
        Anthropic's `count_tokens` over a static, hand-constructed JSON fixture that
        stood in for an MCP tool catalog. That measured the size of a fixture, not the
        cost of a live agent run. It is **retired here** in favour of the live
        session-usage medians above -- real sessions, a real GitHub backend, and the
        GitHub MCP server's real tool surface. The synthetic fixtures remain in the repo
        only as provenance for that retired estimate; they no longer back any published
        number.

        ## What this DOES measure

    with:

        carries **~{red['input_multiple']:.2f}x** the total input-context for the same result.

        ## What this DOES measure

    Net effect: exactly ONE blank line separates the `carries ... result.` line from
    `## What this DOES measure`, matching the committed doc structure. Do NOT touch any
    other part of the f-string, the provenance constants, or the median math.
  </action>
  <verify>
    <automated>! grep -q "What retired the old" quality/gates/perf/bench_token_economy_captures.py &amp;&amp; ! grep -q "89.1% headline and a per-backend" quality/gates/perf/bench_token_economy_captures.py &amp;&amp; python3 -c "import ast,pathlib; ast.parse(pathlib.Path('quality/gates/perf/bench_token_economy_captures.py').read_text()); print('parse OK')"</automated>
  </verify>
  <done>The generator no longer templates the retired-narrative section; the file still parses as valid Python; `## What this DOES measure` remains present and follows the `Equivalently/carries` lines with a single blank separator.</done>
</task>

<task type="auto">
  <name>Task 2: Regenerate the doc offline and prove byte-for-byte identity to committed HEAD</name>
  <files>docs/benchmarks/token-economy.md</files>
  <action>
    Run the offline regenerator (deterministic, no network, no API key, no cargo):

        python3 quality/gates/perf/bench_token_economy.py --offline

    This rewrites `docs/benchmarks/token-economy.md` in place from the committed captures
    using the now-stripped template. Then PROVE the fix is correct by comparing the
    regenerated file against the COMMITTED version:

        git show HEAD:docs/benchmarks/token-economy.md | sha256sum
        sha256sum docs/benchmarks/token-economy.md
        git diff --stat docs/benchmarks/token-economy.md   # expect: empty (no diff)

    Expected committed sha256:
    `5620699b15da82b8123404b7038ae634259891a17517b1f1cba23e598f364fcf`.

    HARD GATE: the two sha256 values MUST be identical AND `git diff docs/benchmarks/token-economy.md`
    MUST be empty. If they match → the generator fix is correct and the working tree is
    already clean (the regen overwrote the stray +12-line re-add). If they do NOT match →
    STOP. Do NOT proceed to Task 3's checkout (which would mask residual template drift).
    Instead, diff the regen output against committed, identify the residual difference in
    the template, fix it in Task 1's scope (still generator-only, still within the ruling),
    and re-run this task. Report the residual in the summary if it cannot be resolved
    within scope.
  </action>
  <verify>
    <automated>test "$(git show HEAD:docs/benchmarks/token-economy.md | sha256sum | cut -d' ' -f1)" = "$(sha256sum docs/benchmarks/token-economy.md | cut -d' ' -f1)" &amp;&amp; git diff --quiet -- docs/benchmarks/token-economy.md &amp;&amp; echo "byte-identical + clean"</automated>
  </verify>
  <done>Regenerated `docs/benchmarks/token-economy.md` is byte-for-byte identical to `HEAD` (sha256 `5620699b...364fcf`); `git diff` on the doc is empty; the fix is proven — future gate-run regens will no longer re-add the stripped section.</done>
</task>

<task type="auto">
  <name>Task 3: Discard any residual stray working-tree diff on the doc (belt-and-suspenders)</name>
  <files>docs/benchmarks/token-economy.md</files>
  <action>
    After Task 2 the doc should already be clean (regen produced committed bytes). As a
    guaranteed fail-safe that the shipped owner ruling is preserved, discard any working-
    tree change on the doc:

        git checkout -- docs/benchmarks/token-economy.md

    This is a no-op if Task 2 left the tree clean; it is the explicit safety net the
    manager ruling requires so the stray section-re-add can NEVER reach the commit. The
    doc MUST end byte-identical to HEAD and MUST NOT be staged (Task 6). Do NOT run this
    to MASK a Task-2 sha mismatch — Task 2 must have passed its hard gate first.
  </action>
  <verify>
    <automated>git diff --quiet -- docs/benchmarks/token-economy.md &amp;&amp; test "$(git show HEAD:docs/benchmarks/token-economy.md | sha256sum | cut -d' ' -f1)" = "$(sha256sum docs/benchmarks/token-economy.md | cut -d' ' -f1)" &amp;&amp; echo "doc == HEAD, unstaged"</automated>
  </verify>
  <done>`docs/benchmarks/token-economy.md` working tree == HEAD (empty diff, sha256 match); the stray +12-line re-add is gone and will not be committed.</done>
</task>

<task type="auto">
  <name>Task 4: Confirm no doc-alignment catalog rebind is required (fail closed)</name>
  <files>quality/catalogs/doc-alignment.json</files>
  <action>
    The committed doc bytes are unchanged (Task 2/3), so bound-claim hashes computed over
    the doc are unchanged by construction — no rebind is expected. VERIFY rather than
    assume, WITHOUT cargo:

    1. Confirm the doc == HEAD (already proven in Task 3: empty diff + sha256 match). This
       is the primary evidence that no bound-claim content hash can have drifted.
    2. Inspect the rows bound to `docs/benchmarks/token-economy.md`:

           grep -n 'token-economy' quality/catalogs/doc-alignment.json

       Confirm no BOUND (non-retired) row anchors its claim to the stripped
       `## What retired the old 89.1% / 85.5% figures` narrative (it never did — the
       bound rows are the live four-axis numeric claims + methodology; the 89.1%/85.5%
       rows are already `RETIRE_PROPOSED`).

    FAIL CLOSED: if — contrary to expectation — any BOUND row's claim hash mismatches or a
    bound row anchors to the removed section, DO NOT silently proceed and DO NOT edit the
    catalog to force-pass. Flag the specific row id + mismatch in the summary and stop for
    manager review. (Do NOT run the raw `reposix-quality doc-alignment walk` subcommand —
    it mutates the committed catalog's counters; the pre-push `docs-alignment/walk.sh` gate
    in Task 6 grades against a /tmp copy and is the automated backstop.)
  </action>
  <verify>
    <automated>git diff --quiet -- quality/catalogs/doc-alignment.json &amp;&amp; git diff --quiet -- docs/benchmarks/token-economy.md &amp;&amp; echo "catalog untouched, doc == HEAD -> no rebind"</automated>
  </verify>
  <done>Verified doc bytes unchanged → bound-claim hashes unchanged; no BOUND row anchors to the removed section; `quality/catalogs/doc-alignment.json` is untouched (no rebind needed). Any mismatch would be flagged in the summary, not silently patched.</done>
</task>

<task type="auto">
  <name>Task 5: Record the fix + working-tree discard in PROGRESS.md</name>
  <files>.planning/PROGRESS.md</files>
  <action>
    Append a short, dated entry (Edit, do not rewrite the file) recording:
    - The generator fix: stripped the `## What retired the old 89.1% / 85.5% figures`
      section from `quality/gates/perf/bench_token_economy_captures.py`
      (`render_token_economy_markdown`), so offline regen now matches the committed doc.
    - The working-tree discard of the stray `+12`-line re-add on
      `docs/benchmarks/token-economy.md`.
    - Provenance: the section was re-added by the P115 phase-close gate-run regen.
    - Manager ruling: this was an **accidental regression vector**, NOT a deliberate
      override of owner ruling 5a5dd29; fix-it-twice (generator + working tree).
    - Reference: quick task 260716-f6o.
    Keep it to a few lines, matching the surrounding style. Do NOT touch
    `.planning/MANAGER-HANDOVER.md`.
  </action>
  <verify>
    <automated>grep -q "260716-f6o" .planning/PROGRESS.md &amp;&amp; grep -qi "accidental regression vector" .planning/PROGRESS.md &amp;&amp; git diff --quiet -- .planning/MANAGER-HANDOVER.md</automated>
  </verify>
  <done>PROGRESS.md carries a concise entry citing 260716-f6o, the generator fix, the working-tree discard, and the "accidental regression vector" provenance; MANAGER-HANDOVER.md untouched.</done>
</task>

<task type="auto">
  <name>Task 6: Commit (targeted staging) and push origin main</name>
  <files>quality/gates/perf/bench_token_economy_captures.py, .planning/PROGRESS.md, .planning/quick/20260716-token-economy-generator-strip/PLAN.md</files>
  <action>
    Stage ONLY the specific files — never `-A` / `.`. Explicitly do NOT stage
    `docs/benchmarks/token-economy.md` (must show no diff) and do NOT stage any
    `quality/catalogs/*.json`.

        git add quality/gates/perf/bench_token_economy_captures.py \
                .planning/PROGRESS.md \
                .planning/quick/20260716-token-economy-generator-strip/PLAN.md
        # (also stage 260716-f6o-SUMMARY.md if the executor writes one under the quick dir)

    Pre-commit sanity: `git status --short` must NOT list
    `docs/benchmarks/token-economy.md` as staged, and `git diff --cached --stat` must NOT
    include it or any catalog json.

    Commit with a message such as:

        fix(perf): strip retired-narrative section from token-economy generator (260716-f6o)

        Fix-it-twice for owner ruling 5a5dd29: the perf-gate generator
        (bench_token_economy_captures.py::render_token_economy_markdown) still templated
        the "## What retired the old 89.1% / 85.5% figures" section that 5a5dd29 stripped
        from docs/benchmarks/token-economy.md. The P115 phase-close gate-run regen re-added
        it in place (dirty +12 lines). Removed the section from the template; offline regen
        now reproduces the committed doc byte-for-byte. Discarded the stray working-tree
        diff. Provenance: accidental regression vector, not a deliberate override.

        Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>

    Then push (pre-push hook runs ~2min — set Bash timeout >= 300000ms):

        git push origin main

    If the pre-push `docs-alignment/walk.sh` (or any gate) BLOCKS, do NOT bypass — read the
    named invariant, and if it implicates the doc/catalog, re-verify Tasks 2-4 and report.
  </action>
  <verify>
    <automated>git diff --cached --name-only | grep -qx 'docs/benchmarks/token-economy.md' &amp;&amp; { echo 'FAIL: doc staged'; exit 1; }; git diff --cached --name-only | grep -q 'quality/catalogs/' &amp;&amp; { echo 'FAIL: catalog staged'; exit 1; }; git log origin/main -1 --oneline | grep -q '260716-f6o' &amp;&amp; echo 'pushed'</automated>
  </verify>
  <done>Commit lands on `origin/main` with ONLY the generator + PROGRESS.md (+ quick PLAN/SUMMARY) staged; `docs/benchmarks/token-economy.md` and all catalog json are NOT in the commit; push succeeded and the pre-push gates (including docs-alignment walk) passed.</done>
</task>

</tasks>

<verification>
- `grep -q "What retired the old" quality/gates/perf/bench_token_economy_captures.py` → NO match (template stripped).
- `python3 quality/gates/perf/bench_token_economy.py --offline` produces
  `docs/benchmarks/token-economy.md` == committed HEAD (sha256 `5620699b...364fcf`).
- `git diff docs/benchmarks/token-economy.md` → empty (stray diff discarded, doc unchanged).
- `git diff quality/catalogs/doc-alignment.json` → empty (no rebind).
- `.planning/PROGRESS.md` records the fix; `.planning/MANAGER-HANDOVER.md` untouched.
- Commit on `origin/main`; doc + catalog NOT staged; pre-push gates green.
</verification>

<success_criteria>
1. Generator template no longer emits the retired-narrative section.
2. Offline regen reproduces the committed doc byte-for-byte (future gate runs stay clean).
3. Working-tree doc ends byte-identical to HEAD; the stray re-add is discarded, never committed.
4. No catalog rebind (verified, fail-closed); catalog files untouched.
5. PROGRESS.md notes the fix + provenance; MANAGER-HANDOVER.md untouched.
6. Targeted-staged commit pushed to origin/main with green pre-push; no cargo run by this task.
</success_criteria>

<output>
After completion, create
`.planning/quick/20260716-token-economy-generator-strip/260716-f6o-SUMMARY.md`
recording: the exact template lines removed, the regen sha256 match, confirmation the doc
was NOT committed, the no-rebind catalog finding, and the pushed commit hash. Note in the
summary anything noticed near the work (OD-3 noticing deliverable) — e.g. other generator
sections whose prose could drift from the committed doc, or docstring line-number
references now stale.
</output>
