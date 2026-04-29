---
phase: 77-good-to-haves-polish
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - docs/index.md
  - quality/gates/docs-alignment/connector-matrix-on-landing.sh
  - quality/catalogs/doc-alignment.json
  - .planning/milestones/v0.12.1-phases/GOOD-TO-HAVES.md
  - CLAUDE.md
  - .planning/phases/77-good-to-haves-polish/SUMMARY.md
  - quality/reports/verdicts/p77/walk-before.txt
  - quality/reports/verdicts/p77/walk-after.txt
autonomous: true
parallelization: false
gap_closure: false
cross_ai: false
requirements:
  - GOOD-TO-HAVES-01

must_haves:
  truths:
    - "docs/index.md:95 heading reads '## Connector capability matrix' (was '## What each backend can do')"
    - "Verifier connector-matrix-on-landing.sh regex narrowed to literal [Cc]onnector and still PASSes against the renamed heading"
    - "doc-alignment walk run after the rename keeps polish2-06-landing BOUND (no STALE_DOCS_DRIFT, no PROPOSE_RETIRE)"
    - "GOOD-TO-HAVES.md P74 entry STATUS flipped from OPEN to RESOLVED with the rename commit SHA"
    - "CLAUDE.md gains a P77 H3 subsection of 30 lines or fewer noting the closure AND the D-09 instruction that HANDOVER-v0.12.1.md deletion is a session-end (not in-phase) action"
    - "HANDOVER-v0.12.1.md remains in place at phase end (D-09 — only criterion 1 of its 3 deletion criteria is true at P77 close)"
  artifacts:
    - path: "docs/index.md"
      provides: "Renamed heading at line 95"
      contains: "## Connector capability matrix"
    - path: "quality/gates/docs-alignment/connector-matrix-on-landing.sh"
      provides: "Narrowed regex (literal connector match) — single-noun grep"
      contains: "[Cc]onnector"
    - path: "quality/reports/verdicts/p77/walk-after.txt"
      provides: "Captured stdout of doc-alignment walk after rename + regex narrow"
      min_lines: 5
    - path: ".planning/milestones/v0.12.1-phases/GOOD-TO-HAVES.md"
      provides: "P74 entry status flipped to RESOLVED with commit SHA"
      contains: "STATUS:** RESOLVED"
    - path: "CLAUDE.md"
      provides: "P77 H3 subsection under v0.12.1 in-flight"
      contains: "### P77"
    - path: ".planning/phases/77-good-to-haves-polish/SUMMARY.md"
      provides: "Phase summary"
      min_lines: 20
  key_links:
    - from: "quality/gates/docs-alignment/connector-matrix-on-landing.sh"
      to: "docs/index.md heading at line 95"
      via: "grep regex match"
      pattern: "Cc.onnector"
    - from: "quality/catalogs/doc-alignment.json polish2-06-landing row"
      to: "quality/gates/docs-alignment/connector-matrix-on-landing.sh"
      via: "verifier_path field"
      pattern: "connector-matrix-on-landing"
    - from: ".planning/milestones/v0.12.1-phases/GOOD-TO-HAVES.md"
      to: "rename commit SHA"
      via: "STATUS: RESOLVED footer"
      pattern: "RESOLVED"
---

<objective>
Drain the v0.12.1 GOOD-TO-HAVES intake: a single XS clarity item discovered by P74. Rename docs/index.md:95 heading from "What each backend can do" to "Connector capability matrix" so the catalog claim ("Connector capability matrix added to landing page") matches the live heading word-for-word, then narrow the verifier regex back to a literal [Cc]onnector match. This is the LAST phase of the v0.12.1 autonomous run.

Purpose: Close GOOD-TO-HAVES-01. Reverse the eager-widen the P74 verifier took (commit c8e4111) once the heading rename makes the original literal regex sufficient. Demonstrate the +2 polish slot working as designed (intake drained, atomic commits, ROI-aware time-box).

Output: One renamed heading, one narrowed verifier regex, one walk verdict capture, GOOD-TO-HAVES.md entry RESOLVED, P77 H3 in CLAUDE.md, SUMMARY.md.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@.planning/phases/77-good-to-haves-polish/CONTEXT.md
@.planning/milestones/v0.12.1-phases/GOOD-TO-HAVES.md
@.planning/HANDOVER-v0.12.1.md
@docs/index.md
@quality/gates/docs-alignment/connector-matrix-on-landing.sh

<interfaces>
The verifier this phase modifies. Current widened shape (lines 12-18 of connector-matrix-on-landing.sh):

  if ! grep -qE '^## .*([Cc]onnector|[Bb]ackend)' "$DOC"; then
    echo "FAIL: docs/index.md has no ... connector ... or ... backend ... heading ..."
    exit 1
  fi
  if ! grep -qE '^\| .* \| .* \|' "$DOC"; then
    echo "FAIL: docs/index.md has no markdown table rows ..."
    exit 1
  fi

The first regex is what P74 widened (commit c8e4111). After P77's rename, this narrows back to '^## .*[Cc]onnector' (single-noun match). The second regex (table-row check) is unrelated and stays as-is.

docs/index.md:95 (current) reads: ## What each backend can do
After P77 rename it reads: ## Connector capability matrix

The surrounding paragraph at lines 97-100 also says "The four built-in backends differ in capabilities" — leave it alone (the prose can mention "backends" even when the heading is "Connector capability matrix"; the verifier only checks the heading line).
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Catalog-first — record GREEN baseline before changes</name>
  <files>quality/reports/verdicts/p77/walk-before.txt</files>
  <action>
    Confirm the GREEN starting point. Read the current state of `polish2-06-landing` row in `quality/catalogs/doc-alignment.json` via jq and confirm `last_verdict` is BOUND. Build the binary if needed (`cargo build -p reposix-quality --release` — single cargo invocation per memory budget). Run `target/release/reposix-quality doc-alignment walk` and capture stdout to `quality/reports/verdicts/p77/walk-before.txt`. Record the alignment_ratio and the polish2-06-landing row verdict for the SUMMARY. Do NOT modify the catalog yet. Per D-02, commit as `chore(p77): record GREEN baseline before heading rename` (just the walk-before.txt + any catalog churn the walk produced as a byproduct).
  </action>
  <verify>
    <automated>jq -e '.rows[] | select(.id | endswith("polish2-06-landing")) | .last_verdict == "BOUND"' quality/catalogs/doc-alignment.json &amp;&amp; test -s quality/reports/verdicts/p77/walk-before.txt</automated>
  </verify>
  <done>polish2-06-landing confirmed BOUND in catalog; walk-before.txt captured non-empty stdout; baseline alignment_ratio recorded for the SUMMARY; one commit landed.</done>
</task>

<task type="auto">
  <name>Task 2: Rename heading at docs/index.md:95</name>
  <files>docs/index.md</files>
  <action>
    Edit `docs/index.md` line 95 to change the heading from `## What each backend can do` to `## Connector capability matrix`. ONLY the heading line changes — leave the surrounding paragraph (lines 97-100) and the table (lines 102-107) untouched. The paragraph still references "backends" in prose; that is fine because the verifier only matches the heading. Commit per D-02 with message starting `polish(p77): rename docs/index.md heading "What each backend can do" -> "Connector capability matrix" (was: discovered-by P74)`. Commit body quotes the GOOD-TO-HAVES.md P74 entry verbatim (entire block from `## discovered-by: P74` through `**STATUS:** OPEN`).
  </action>
  <verify>
    <automated>grep -qE '^## Connector capability matrix$' docs/index.md &amp;&amp; ! grep -qE '^## What each backend can do$' docs/index.md</automated>
  </verify>
  <done>Heading reads "## Connector capability matrix"; old "## What each backend can do" removed; one atomic commit with the verbatim GOOD-TO-HAVES.md entry quoted in the body.</done>
</task>

<task type="auto">
  <name>Task 3: Narrow verifier regex back to literal [Cc]onnector</name>
  <files>quality/gates/docs-alignment/connector-matrix-on-landing.sh</files>
  <action>
    Edit the verifier so the heading-check regex narrows from `^## .*([Cc]onnector|[Bb]ackend)` to `^## .*[Cc]onnector`. Update the comment block (lines 1-7) to reflect the rename: replace the "the live heading is '## What each backend can do' — same intent, different word" sentence with "the live heading is '## Connector capability matrix' — literal claim+heading match (P77 narrow following P74 widen)". Update the FAIL message (line 13) from `has no '## ...connector...' or '## ...backend...' heading` to `has no '## ...connector...' heading`. The table-row check (line 16) stays untouched. Run the verifier locally to confirm it still passes: `bash quality/gates/docs-alignment/connector-matrix-on-landing.sh`. Commit per D-02: `polish(p77): narrow connector-matrix-on-landing regex to literal [Cc]onnector (was: discovered-by P74, follow-up to rename)`. Body quotes the same GOOD-TO-HAVES.md entry.
  </action>
  <verify>
    <automated>bash quality/gates/docs-alignment/connector-matrix-on-landing.sh &amp;&amp; grep -q 'Cc.onnector' quality/gates/docs-alignment/connector-matrix-on-landing.sh &amp;&amp; ! grep -q 'Bb.ackend' quality/gates/docs-alignment/connector-matrix-on-landing.sh</automated>
  </verify>
  <done>Verifier passes against the renamed heading; regex contains only [Cc]onnector (no [Bb]ackend alternation); FAIL message updated; comment block updated; one atomic commit.</done>
</task>

<task type="auto">
  <name>Task 4: Run walk-after, confirm polish2-06-landing stays BOUND</name>
  <files>
    quality/reports/verdicts/p77/walk-after.txt
    quality/catalogs/doc-alignment.json
  </files>
  <action>
    Run `target/release/reposix-quality doc-alignment walk` (single cargo-side invocation; binary already built in Task 1 unless the walker re-hashed sources). Tee stdout to `quality/reports/verdicts/p77/walk-after.txt`. Diff against walk-before.txt to confirm: (a) polish2-06-landing row remains BOUND, (b) no NEW STALE_DOCS_DRIFT rows introduced, (c) alignment_ratio unchanged or improved. The walk MAY refresh `source_hash` on the polish2-06-landing row because the heading line changed — that is expected; commit the catalog churn. If alignment_ratio drops or any new STALE rows appear, STOP and surface to the verifier. Commit: `chore(p77): record walk-after verdict + refreshed source_hash for polish2-06-landing`.
  </action>
  <verify>
    <automated>jq -e '.rows[] | select(.id | endswith("polish2-06-landing")) | .last_verdict == "BOUND"' quality/catalogs/doc-alignment.json &amp;&amp; test -s quality/reports/verdicts/p77/walk-after.txt &amp;&amp; ! grep -q STALE_DOCS_DRIFT quality/reports/verdicts/p77/walk-after.txt</automated>
  </verify>
  <done>walk-after.txt captured; polish2-06-landing remains BOUND post-rename; no new STALE rows; alignment_ratio unchanged or improved; catalog churn (if any) committed.</done>
</task>

<task type="auto">
  <name>Task 5: Flip GOOD-TO-HAVES.md entry to RESOLVED</name>
  <files>.planning/milestones/v0.12.1-phases/GOOD-TO-HAVES.md</files>
  <action>
    Update the single P74 entry in `.planning/milestones/v0.12.1-phases/GOOD-TO-HAVES.md`. Change the last line from `**STATUS:** OPEN` to `**STATUS:** RESOLVED — &lt;rename-commit-sha&gt; (heading), &lt;regex-narrow-commit-sha&gt; (verifier). Walk-after verdict captured at quality/reports/verdicts/p77/walk-after.txt; polish2-06-landing remains BOUND.` Use the actual SHAs from Task 2 and Task 3 (`git log --format=%H -n 1 -- docs/index.md` and `git log --format=%H -n 1 -- quality/gates/docs-alignment/connector-matrix-on-landing.sh`). Commit per D-02: `polish(p77): mark GOOD-TO-HAVES P74 entry RESOLVED`.
  </action>
  <verify>
    <automated>grep -q 'STATUS:\*\* RESOLVED' .planning/milestones/v0.12.1-phases/GOOD-TO-HAVES.md &amp;&amp; ! grep -q 'STATUS:\*\* OPEN' .planning/milestones/v0.12.1-phases/GOOD-TO-HAVES.md</automated>
  </verify>
  <done>The single P74 entry shows STATUS RESOLVED with the two relevant commit SHAs; no OPEN entries remain in the file; one atomic commit.</done>
</task>

<task type="auto">
  <name>Task 6: CLAUDE.md P77 H3 subsection (load-bearing D-09 note)</name>
  <files>CLAUDE.md</files>
  <action>
    Add a new H3 subsection `### P77 — good-to-haves polish (closed)` under the "v0.12.1 — in flight" section of CLAUDE.md (locate the section by searching for the most recent P-numbered H3 — P76 if it exists, otherwise the last P7x H3). Body must be 30 lines or fewer. Content covers, in this order:

    1. One sentence stating P77 closed GOOD-TO-HAVES-01 by draining the v0.12.1 intake (1 XS item).
    2. The closure: heading rename + verifier regex narrow; cite the two commit SHAs.
    3. Walk-after verdict location: `quality/reports/verdicts/p77/walk-after.txt`; polish2-06-landing remains BOUND.
    4. **D-09 load-bearing note (verbatim shape):** "P77 is the LAST phase of the v0.12.1 autonomous run. HANDOVER-v0.12.1.md is intentionally LEFT IN PLACE at P77 close. Its self-deletion criteria (HANDOVER §'Cleanup criterion') require all 6 phases verifier-GREEN AND owner pushed v0.12.0 tag AND owner confirmed retires AND v0.12.1 milestone-close verdict GREEN — only criterion 1 is true at P77 close. The session-end commit that removes HANDOVER-v0.12.1.md is an orchestrator-level action OUTSIDE the phase, written by the top-level coordinator after the verifier subagent grades P77 GREEN."
    5. Pointer: see `quality/reports/verdicts/p77/VERDICT.md` for unbiased grading.

    Commit per D-02: `docs(p77): CLAUDE.md H3 — good-to-haves polish closure + HANDOVER-deletion ownership note`.
  </action>
  <verify>
    <automated>grep -q '### P77' CLAUDE.md &amp;&amp; grep -q 'HANDOVER-v0.12.1.md is intentionally LEFT IN PLACE' CLAUDE.md &amp;&amp; awk '/^### P77/,/^### |^## /' CLAUDE.md | wc -l | awk '$1 &lt;= 32 { exit 0 } { exit 1 }'</automated>
  </verify>
  <done>CLAUDE.md gains a P77 H3 of 30 lines or fewer covering closure + D-09 HANDOVER-deletion ownership note; one atomic commit.</done>
</task>

<task type="auto">
  <name>Task 7: SUMMARY.md</name>
  <files>.planning/phases/77-good-to-haves-polish/SUMMARY.md</files>
  <action>
    Write the phase SUMMARY following `$HOME/.claude/get-shit-done/templates/summary.md`. Required content:
    - **Phase goal recap:** Close GOOD-TO-HAVES-01 (drain GOOD-TO-HAVES.md, XS items first).
    - **What shipped:** Heading rename at docs/index.md:95; verifier regex narrow in connector-matrix-on-landing.sh; walk-after capture; intake entry flipped to RESOLVED; CLAUDE.md P77 H3.
    - **Catalog impact:** polish2-06-landing source_hash refreshed (or unchanged — record actual). Alignment_ratio before/after. No new STALE rows.
    - **Commits:** numbered list with one-line summaries + SHAs (Task 1 baseline, Task 2 rename, Task 3 regex, Task 4 walk-after, Task 5 RESOLVED, Task 6 CLAUDE.md, Task 7 this SUMMARY).
    - **D-09 explicit note:** "HANDOVER-v0.12.1.md left in place at phase close. Its deletion is a session-end orchestrator action, not an in-phase action."
    - **Tech_stack / decisions / patterns:** standard digest fields per template.
    - **Verifier dispatch:** Path A `gsd-verifier` from top-level orchestrator; verdict at `quality/reports/verdicts/p77/VERDICT.md`.
    Commit: `docs(p77): phase SUMMARY`.
  </action>
  <verify>
    <automated>test -f .planning/phases/77-good-to-haves-polish/SUMMARY.md &amp;&amp; [ "$(wc -l &lt; .planning/phases/77-good-to-haves-polish/SUMMARY.md)" -ge 20 ] &amp;&amp; grep -q 'HANDOVER-v0.12.1.md left in place' .planning/phases/77-good-to-haves-polish/SUMMARY.md</automated>
  </verify>
  <done>SUMMARY.md exists with 20+ lines, covers all required sections, includes the verbatim D-09 HANDOVER-deletion ownership note; one atomic commit; phase ready for verifier dispatch.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| docs/index.md (renderable user-facing) | Heading rename only — no script execution surface; mkdocs renders markdown; renaming a heading cannot inject script. |
| connector-matrix-on-landing.sh (verifier) | Reads `docs/index.md` via `grep` only; no eval, no sourcing of doc content. Bytes-in / exit-code-out. |
| quality/catalogs/doc-alignment.json | Walker rewrites `source_hash` for the rebound row. Catalog is project-controlled, not attacker-influenced. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-77-01 | Tampering | docs/index.md heading | accept | Heading rename is a markdown text edit; mkdocs treats it as content; no script-execution surface introduced. |
| T-77-02 | Tampering | connector-matrix-on-landing.sh regex | mitigate | Verifier executed in CI + pre-push; narrowed regex still asserts the claim; Task 3 verifies the script PASSes against the renamed heading before commit. |
| T-77-03 | Repudiation | GOOD-TO-HAVES.md status flip | mitigate | Each commit message references the GOOD-TO-HAVES entry verbatim per D-02; commits are append-only in git history; STATUS line cites concrete SHAs. |
</threat_model>

<verification>
- All 7 tasks committed atomically (no squashing — D-02 explicit).
- `bash quality/gates/docs-alignment/connector-matrix-on-landing.sh` exits 0 against the renamed heading.
- `target/release/reposix-quality doc-alignment walk` reports polish2-06-landing BOUND post-rename.
- `quality/reports/verdicts/p77/walk-before.txt` and `walk-after.txt` both committed and non-empty.
- `.planning/milestones/v0.12.1-phases/GOOD-TO-HAVES.md` shows STATUS RESOLVED for the single P74 entry; zero OPEN entries.
- CLAUDE.md P77 H3 subsection of 30 lines or fewer present, including the verbatim D-09 HANDOVER-deletion ownership note.
- `.planning/HANDOVER-v0.12.1.md` STILL EXISTS at phase close (D-09 — only criterion 1 of its 3 deletion criteria is true at P77 close).
- SUMMARY.md committed with all required sections.
- Top-level orchestrator dispatches `gsd-verifier` Path A; verdict written to `quality/reports/verdicts/p77/VERDICT.md`.
</verification>

<success_criteria>
- GOOD-TO-HAVES-01 requirement closed (single XS entry RESOLVED with commit SHA).
- Catalog row polish2-06-landing remains BOUND across the rename + regex narrow.
- HANDOVER-v0.12.1.md preserved (its deletion is owned by the session-end orchestrator, not P77).
- Verifier subagent grades the catalog rows GREEN at `quality/reports/verdicts/p77/VERDICT.md`.
- Phase fits inside the 30-min XS budget; no scope creep into S/M items (D-04, D-06, D-10).
</success_criteria>

<output>
After completion, create `.planning/phases/77-good-to-haves-polish/SUMMARY.md` (Task 7).

Verifier dispatch is a TOP-LEVEL action (Path A `gsd-verifier` via Task tool from the orchestrator session, not from inside `gsd-executor`). Verdict at `quality/reports/verdicts/p77/VERDICT.md`.

The session-end commit that removes `.planning/HANDOVER-v0.12.1.md` is OUT OF SCOPE for this plan — handled by the top-level orchestrator after P77 verdict GREEN.
</output>
