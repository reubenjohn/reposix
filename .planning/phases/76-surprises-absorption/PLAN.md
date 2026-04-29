---
phase: 76-surprises-absorption
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - .planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md
  - .planning/milestones/v0.12.1-phases/GOOD-TO-HAVES.md
  - quality/catalogs/doc-alignment.json
  - quality/reports/verdicts/p76/triage.md
  - quality/reports/verdicts/p76/walk-after.txt
  - quality/reports/verdicts/p76/status-after.txt
  - quality/reports/verdicts/p76/honesty-spot-check.md
  - quality/reports/verdicts/p76/verifier-prompt.md
  - CLAUDE.md
  - .planning/phases/76-surprises-absorption/SUMMARY.md
autonomous: true
parallelization: false
gap_closure: false
cross_ai: false
requirements:
  - SURPRISES-ABSORB-01
must_haves:
  truths:
    - "All 3 SURPRISES-INTAKE.md entries transition from STATUS: OPEN to a terminal status (RESOLVED | DEFERRED | WONTFIX) with rationale or commit SHA appended in a STATUS footer (per CONTEXT.md D-02 / D-04)."
    - "Entry 1 (P72-discovered, two pre-existing BOUND rows that flipped STALE_DOCS_DRIFT) is RESOLVED: each row is either rebound (source_hash refreshed against current source bytes; last_verdict BOUND) or propose-retired (last_verdict RETIRE_PROPOSED), driven by whether the claim text still describes current code."
    - "Entry 2 (P74 linkedin Source::Single STALE_DOCS_DRIFT) is annotated RESOLVED with commit SHA 9e07028 in its STATUS footer; no fresh code change is required (P75 already healed it; P76 only updates the audit trail)."
    - "Entry 3 (P74 connector-matrix synonym widening) is WONTFIX with rationale that the regex widening is the complete fix; the heading rename is filed as a P77 GOOD-TO-HAVE in GOOD-TO-HAVES.md (one new entry, size XS)."
    - "After the entry-1 resolutions land, a live `target/release/reposix-quality doc-alignment walk` shows zero net new STALE_DOCS_DRIFT rows beyond the post-action set (i.e., the two pre-existing STALE rows are either healed-to-BOUND or carried as RETIRE_PROPOSED, no longer counted as STALE)."
    - "The honesty spot-check (D-05) reads at least 2 of {P72, P73, P74, P75} PLAN.md + their VERDICT.md files and records, in quality/reports/verdicts/p76/honesty-spot-check.md (and SUMMARY.md), evidence that those phases honestly looked for out-of-scope items (Eager-resolution decisions present in plans; intake entries where appropriate; no skipped findings smuggled past)."
    - "CLAUDE.md gains a `### P76 — Surprises absorption` H3 subsection (<=30 lines) under `## v0.12.1 — in flight` that lists each of the 3 entries inline as `<title> -> RESOLVED|WONTFIX | <commit-sha-or-rationale>` (per D-08)."
    - "Phase SUMMARY.md is committed with the verifier-verdict pointer at quality/reports/verdicts/p76/VERDICT.md, the triage table reproduced inline, the honesty spot-check finding, and the catalog-row deltas."
  artifacts:
    - path: ".planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md"
      provides: "Updated intake file: 3 STATUS footers transitioned OPEN -> RESOLVED|WONTFIX. File STAYS — it is the audit trail per CLAUDE.md OP-3."
      contains: "STATUS: RESOLVED"
    - path: ".planning/milestones/v0.12.1-phases/GOOD-TO-HAVES.md"
      provides: "One new XS entry filed by P76 from intake entry 3 resolution: rename docs/index.md heading What each backend can do -> Connector capability matrix (P77 absorbs)."
      contains: "Connector capability matrix"
    - path: "quality/catalogs/doc-alignment.json"
      provides: "Up to 2 row state changes from entry 1: polish-03-mermaid-render and cli-subcommand-surface either rebound (BOUND) or propose-retired (RETIRE_PROPOSED). Possibly 0 changes if both retire."
      contains: "polish-03-mermaid-render"
    - path: "quality/reports/verdicts/p76/triage.md"
      provides: "P76 triage table: severity-grouped intake entries with proposed disposition (drives D-01)."
    - path: "quality/reports/verdicts/p76/walk-after.txt"
      provides: "Live `doc-alignment walk` stdout+stderr captured AFTER entry-1 resolutions land. Evidence that no new STALE_DOCS_DRIFT was introduced by P76's actions."
    - path: "quality/reports/verdicts/p76/status-after.txt"
      provides: "Live `doc-alignment status --top 10` capture AFTER P76's row mutations: alignment_ratio, claims_bound, claims_missing_test, claims_retire_proposed, claims_retired."
    - path: "quality/reports/verdicts/p76/honesty-spot-check.md"
      provides: "D-05 honesty-check evidence: which 2 of P72-P75 plans + verdicts were sampled; what Eager-resolution decisions / intake entries were found; verdict GREEN|RED."
    - path: "CLAUDE.md"
      provides: "P76 H3 subsection (<=30 lines) under v0.12.1 — in flight."
      contains: "P76 — Surprises absorption"
    - path: ".planning/phases/76-surprises-absorption/SUMMARY.md"
      provides: "Phase SUMMARY with intake disposition table, honesty spot-check finding, catalog deltas, verifier-verdict pointer."
      contains: "P76"
  key_links:
    - from: ".planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md (3 entries)"
      to: "STATUS footers (RESOLVED | WONTFIX)"
      via: "edit-in-place append per D-02 (do NOT delete entries; the file is audit-trail)"
      pattern: "STATUS: (RESOLVED|WONTFIX)"
    - from: "Entry 1 disposition decision"
      to: "quality/catalogs/doc-alignment.json row mutations"
      via: "target/release/reposix-quality doc-alignment {bind|propose-retire} per row"
      pattern: "doc-alignment (bind|propose-retire)"
    - from: "Entry 3 WONTFIX"
      to: ".planning/milestones/v0.12.1-phases/GOOD-TO-HAVES.md (new XS entry)"
      via: "append-only intake (new entry under ## Entries; status OPEN)"
      pattern: "Connector capability matrix"
    - from: "honesty spot-check (D-05)"
      to: ".planning/phases/{72,73,74,75}-*/PLAN.md + quality/reports/verdicts/p{72,73,74,75}/VERDICT.md"
      via: "read 2 of those 4 plan/verdict pairs; record finding inline in SUMMARY.md + honesty-spot-check.md"
      pattern: "Eager-resolution|out-of-scope|SURPRISES-INTAKE"
    - from: "CLAUDE.md ## v0.12.1 — in flight"
      to: "### P76 — Surprises absorption"
      via: "append H3 after the existing P75 H3 (CLAUDE.md:390-406)"
      pattern: "P76 — Surprises absorption"
---

<objective>
Drain `.planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md` — 3 LOW-severity entries discovered during P72 + P74 — to terminal status. This is the operational arm of CLAUDE.md OP-8 (the +2 phase practice): when a planned phase finds something out-of-scope it appends to intake instead of expanding scope or skipping silently, and P76 closes the loop.

Purpose: prove the +2 practice is not a no-op by closing every intake entry with a dispositioned commit, AND grade P72-P75 for honesty (D-05 spot-check). The intake is the project's "found-it-but-couldn't-fold-it-in" backlog; if P76 leaves entries OPEN or hand-waves dispositions, the practice degrades into a cosmetic ritual.

Output:
- 3 SURPRISES-INTAKE.md STATUS footers transitioned (entry 1 RESOLVED via row rebinds/retires; entry 2 RESOLVED via SHA annotation; entry 3 WONTFIX with P77-pointer).
- 1 new GOOD-TO-HAVES.md XS entry from entry 3.
- Up to 2 row state changes in quality/catalogs/doc-alignment.json (entry 1).
- Live walker + status evidence under quality/reports/verdicts/p76/.
- D-05 honesty spot-check finding under quality/reports/verdicts/p76/honesty-spot-check.md.
- P76 H3 in CLAUDE.md (<=30 lines, lists each disposition inline per D-08).
- Phase SUMMARY.md with the honesty-check finding inline.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/STATE.md
@CLAUDE.md
@.planning/phases/76-surprises-absorption/CONTEXT.md
@.planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md
@.planning/milestones/v0.12.1-phases/GOOD-TO-HAVES.md
@.planning/milestones/v0.12.1-phases/REQUIREMENTS.md
@.planning/milestones/v0.12.1-phases/ROADMAP.md
@.planning/phases/72-lint-config-invariants/SUMMARY.md
@.planning/phases/74-narrative-ux-prose-cleanup/SUMMARY.md
@.planning/phases/75-bind-verb-hash-fix/SUMMARY.md
@quality/catalogs/doc-alignment.json
@quality/PROTOCOL.md

<interfaces>
<!-- Catalog row shape and verb surface the executor will use. Extracted from -->
<!-- target/release/reposix-quality doc-alignment --help and the live catalog. -->
<!-- Executor uses these directly; no codebase exploration needed. -->

Catalog row schema (excerpt — quality/catalogs/doc-alignment.json):
```json
{
  "id": "<doc-slug>/<claim-slug>",
  "claim": "<one-line natural-language claim>",
  "source": { "file": "<path>", "line_start": <int>, "line_end": <int> },
  "source_hash": "<sha256 of source byte range>",
  "tests": ["<path-or-rust-test-id>"],
  "test_body_hashes": ["<sha256>"],
  "rationale": "<why this binding works>",
  "last_verdict": "BOUND | MISSING_TEST | RETIRE_PROPOSED | RETIRED | STALE_DOCS_DRIFT",
  "next_action": "BIND_GREEN | RETIRE | EXTRACT_TEST",
  "last_run": "<ISO-8601>",
  "last_extracted": "<ISO-8601>",
  "last_extracted_by": "<verb-name>"
}
```

Verb surface (`target/release/reposix-quality doc-alignment <verb>`):
- `bind --row-id <id> [--source-file <path> --line-start <n> --line-end <n>] [--test <path>]` — recompute source_hash from CURRENT bytes of cited range; persist last_verdict=BOUND. Use to refresh hash when the claim still describes current code.
- `propose-retire --row-id <id> --rationale "<one paragraph>"` — persist last_verdict=RETIRE_PROPOSED. Use when the claim has genuinely diverged.
- `walk` — hash-drift detector; updates last_verdict only (no rebind). Walks NEVER refresh stored hashes.
- `status [--json] [--top <n>]` — summary block + per-file coverage.

Important verb contract (from P75 SUMMARY): walks NEVER refresh stored hashes; only `bind` does. So entry 1's two STALE rows will not heal on a walk alone — they need either a fresh `bind` (with the same or updated source range) or a `propose-retire`.

Two rows targeted by entry 1 (current state from live catalog):
- `planning-milestones-v0-11-0-phases-REQUIREMENTS-md/polish-03-mermaid-render`
  - source: `.planning/milestones/v0.11.0-phases/REQUIREMENTS.md:85`
  - claim: "All mermaid diagrams render without console errors on the live site"
  - test: `quality/gates/docs-build/mermaid-renders.sh`
  - last_verdict: STALE_DOCS_DRIFT
  - Likely disposition: REBIND (line 85 still describes mermaid-render hygiene; prose drift is shallow).
- `docs/decisions/009-stability-commitment/cli-subcommand-surface`
  - source: `crates/reposix-cli/src/main.rs:37-299`
  - claim: "CLI subcommand surface (init|sim|list|refresh|spaces|log|history|tokens|cost|gc|doctor|version) is locked under semver"
  - test: `crates/reposix-cli/tests/agent_flow_real.rs::dark_factory_real_github`
  - last_verdict: STALE_DOCS_DRIFT
  - Likely disposition: REBIND (main.rs is 420 lines; `enum Cmd` still at line 37; range may need narrowing).
- Executor MUST verify with `sed -n '<range>p'` before deciding. If a claim genuinely no longer matches current code, propose-retire instead.

GOOD-TO-HAVES entry-format (from GOOD-TO-HAVES.md header):
```markdown
## discovered-by: P<N> | size: XS|S|M | impact: clarity|perf|consistency|grounding

**What:** One-paragraph description.

**Proposed fix:** One line.

**STATUS:** OPEN
```
The new entry P76 files (from intake entry 3 resolution): discovered-by: P74 (the original discovering phase), size: XS, impact: clarity, what = heading rename to literal-match the catalog row claim text.
</interfaces>

</context>

<tasks>

<task type="auto">
  <name>Task 1: Triage + verify (Wave 1) — read intake, inspect entry-1 source ranges, write triage table</name>
  <files>quality/reports/verdicts/p76/triage.md</files>
  <action>
    Per CONTEXT.md D-01 (triage-by-severity-then-execute) and D-07 (executor mode for &lt;=5 LOW entries):

    1. Read `.planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md` end-to-end. Confirm: 3 entries, all severity LOW.
    2. For entry 1 (P72-discovered, two pre-existing STALE rows), inspect each cited source range to decide REBIND vs PROPOSE-RETIRE:
       - Row `polish-03-mermaid-render` cites `.planning/milestones/v0.11.0-phases/REQUIREMENTS.md:85`. Run `sed -n '85p' .planning/milestones/v0.11.0-phases/REQUIREMENTS.md` and confirm it still describes mermaid-render hygiene. Decision: REBIND if claim text still describes the line; PROPOSE-RETIRE if drifted.
       - Row `cli-subcommand-surface` cites `crates/reposix-cli/src/main.rs:37-299`. Run `sed -n '37,40p; 295,299p' crates/reposix-cli/src/main.rs` and confirm `enum Cmd` is still at line 37 with the documented subcommand surface. If end-of-range moved, narrow to a current range and rebind. If subcommand surface changed substantively, PROPOSE-RETIRE.
    3. For entry 2, confirm `docs/social/linkedin/token-reduction-92pct` is `last_verdict == BOUND` in the live catalog (`jq` lookup) and that commit `9e07028` is in `git log` with the linkedin-heal narrative. Disposition is RESOLVED via SHA annotation only (no row mutation, no code change).
    4. For entry 3, re-read entry text + P74 SUMMARY § "GOOD-TO-HAVES entries". Confirm the regex widening landed in commit `c8e4111` and the connector-matrix verifier is passing. Disposition is WONTFIX with P77-pointer; ALSO file the heading-rename as a NEW GOOD-TO-HAVES.md entry (XS, clarity).
    5. Write `quality/reports/verdicts/p76/triage.md` containing a markdown table:
       ```
       | Entry | Severity | Disposition | Rationale | Action commit (Wave 2) |
       |-------|----------|-------------|-----------|------------------------|
       | 1a polish-03 | LOW | REBIND or RETIRE | <evidence from sed> | row-state commit |
       | 1b cli-subcommand-surface | LOW | REBIND or RETIRE | <evidence from sed> | row-state commit |
       | 2 linkedin Source::Single | LOW | RESOLVED (annotate w/ 9e07028) | P75 already healed; pure audit-trail update | annotation-only commit |
       | 3 connector-matrix synonym | LOW | WONTFIX + new GOOD-TO-HAVE | regex widening is complete fix; heading rename is P77 polish | annotation + GOOD-TO-HAVES.md commit |
       ```
       Below the table, paste the `sed` outputs verbatim (so the triage file is self-grounding).
    6. **No commit yet** — this task produces evidence; commits land in Wave 2.

    Note: per D-09, no NEW SURPRISES discovered during P76 itself; if anything new surfaces, append to GOOD-TO-HAVES.md (P77) instead.
  </action>
  <verify>
    <automated>test -f quality/reports/verdicts/p76/triage.md &amp;&amp; grep -q "polish-03" quality/reports/verdicts/p76/triage.md &amp;&amp; grep -q "cli-subcommand-surface" quality/reports/verdicts/p76/triage.md &amp;&amp; grep -q "linkedin" quality/reports/verdicts/p76/triage.md &amp;&amp; grep -q "connector-matrix" quality/reports/verdicts/p76/triage.md</automated>
  </verify>
  <done>quality/reports/verdicts/p76/triage.md exists with a 4-row disposition table and inline `sed` evidence for entry 1's two rows. No commits yet.</done>
</task>

<task type="auto">
  <name>Task 2: Resolve entry 1 (Wave 2) — rebind or propose-retire two pre-existing STALE rows; capture live walk + status</name>
  <files>quality/catalogs/doc-alignment.json, quality/reports/verdicts/p76/walk-after.txt, quality/reports/verdicts/p76/status-after.txt, .planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md</files>
  <action>
    Per CONTEXT.md D-02 (each resolution is its own atomic commit; commit body quotes the original entry verbatim and appends rationale):

    1. **Build the binary if needed:** `cargo build --release -p reposix-quality` (single cargo invocation; honor CLAUDE.md "Build memory budget" — no parallel cargo).
    2. **For row `polish-03-mermaid-render`** (per triage decision):
       - If REBIND: `target/release/reposix-quality doc-alignment bind --row-id 'planning-milestones-v0-11-0-phases-REQUIREMENTS-md/polish-03-mermaid-render' --source-file '.planning/milestones/v0.11.0-phases/REQUIREMENTS.md' --line-start 85 --line-end 85 --test 'quality/gates/docs-build/mermaid-renders.sh'`. Confirm `last_verdict` flips to BOUND in the catalog (`jq`).
       - If PROPOSE-RETIRE: `target/release/reposix-quality doc-alignment propose-retire --row-id '<id>' --rationale "<one-paragraph rationale why claim no longer maps to source>"`. Confirm `last_verdict == RETIRE_PROPOSED`.
       - Atomic commit: `git add quality/catalogs/doc-alignment.json && git commit -m "fix(p76): RESOLVED entry-1a polish-03-mermaid-render rebind (was: discovered-by P72)"` (or `propose-retire` variant). Body quotes the intake entry's first sentence verbatim and appends the disposition rationale.
    3. **For row `cli-subcommand-surface`** (per triage decision):
       - If REBIND: `target/release/reposix-quality doc-alignment bind --row-id 'docs/decisions/009-stability-commitment/cli-subcommand-surface' --source-file 'crates/reposix-cli/src/main.rs' --line-start <verified-start> --line-end <verified-end> --test 'crates/reposix-cli/tests/agent_flow_real.rs::dark_factory_real_github'`. Verify `last_verdict == BOUND`.
       - If PROPOSE-RETIRE: same pattern with appropriate rationale.
       - Atomic commit: `git add quality/catalogs/doc-alignment.json && git commit -m "fix(p76): RESOLVED entry-1b cli-subcommand-surface rebind (was: discovered-by P72)"` (or `propose-retire` variant).
    4. **Capture live evidence** (after both row mutations land):
       - `target/release/reposix-quality doc-alignment walk 2&gt;&amp;1 | tee quality/reports/verdicts/p76/walk-after.txt`. Verify net new STALE_DOCS_DRIFT introduced by P76 == 0.
       - `target/release/reposix-quality doc-alignment status --top 10 2&gt;&amp;1 | tee quality/reports/verdicts/p76/status-after.txt`.
    5. **Update SURPRISES-INTAKE.md entry 1 STATUS footer**: change `**STATUS:** OPEN` to `**STATUS:** RESOLVED | row 1a: <commit-sha-1> (<rebind|propose-retire>) | row 1b: <commit-sha-2> (<rebind|propose-retire>)`. Do NOT delete the entry — file is audit-trail per CLAUDE.md OP-3.
    6. Atomic commit: `git add quality/reports/verdicts/p76/walk-after.txt quality/reports/verdicts/p76/status-after.txt .planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md && git commit -m "fix(p76): RESOLVED entry-1 evidence + intake-footer (live walk + status, no net new STALE_DOCS_DRIFT)"`.

    Allowed combine: if both rows are clean rebinds and the executor judges the diff legible, ONE combined commit covering both rebinds is acceptable. Default behaviour: separate commits.

    Failure mode: if a `bind` invocation fails (e.g. line range doesn't exist anymore), fall back to `propose-retire` rather than fighting the binary.
  </action>
  <verify>
    <automated>jq -e '.rows[] | select(.id == "planning-milestones-v0-11-0-phases-REQUIREMENTS-md/polish-03-mermaid-render") | .last_verdict | test("^(BOUND|RETIRE_PROPOSED)$")' quality/catalogs/doc-alignment.json &amp;&amp; jq -e '.rows[] | select(.id == "docs/decisions/009-stability-commitment/cli-subcommand-surface") | .last_verdict | test("^(BOUND|RETIRE_PROPOSED)$")' quality/catalogs/doc-alignment.json &amp;&amp; test -s quality/reports/verdicts/p76/walk-after.txt &amp;&amp; test -s quality/reports/verdicts/p76/status-after.txt &amp;&amp; grep -q "STATUS:.*RESOLVED" .planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md</automated>
  </verify>
  <done>Both target rows have last_verdict in {BOUND, RETIRE_PROPOSED}. walk-after.txt + status-after.txt captured and committed. SURPRISES-INTAKE.md entry 1 STATUS footer flipped to RESOLVED with both commit SHAs. Live walk shows no net new STALE_DOCS_DRIFT introduced by P76's actions.</done>
</task>

<task type="auto">
  <name>Task 3: Resolve entry 2 (Wave 2) — annotate linkedin entry RESOLVED with P75 SHA 9e07028</name>
  <files>.planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md</files>
  <action>
    Pure annotation; no code change, no row mutation (P75 already healed the row).

    1. Confirm `9e07028` exists in `git log --oneline -50` and its message matches the linkedin-heal narrative (P75 SUMMARY: "linkedin row heal + live-walk smoke at quality/reports/verdicts/p75/walk-after-fix.txt").
    2. Confirm via `jq -r '.rows[] | select(.id == "docs/social/linkedin/token-reduction-92pct") | .last_verdict' quality/catalogs/doc-alignment.json` returns `BOUND`.
    3. Update SURPRISES-INTAKE.md entry 2 STATUS footer from `**STATUS:** OPEN` to:
       ```
       **STATUS:** RESOLVED | healed by P75 commit 9e07028 (verbs::bind hash-overwrite fix + fresh rebind landed the source_hash refresh; row last_verdict transitioned STALE_DOCS_DRIFT -> BOUND). P76 confirms via live catalog query; no new code change required.
       ```
    4. Atomic commit: `git add .planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md && git commit -m "fix(p76): RESOLVED entry-2 linkedin Source::Single (was: discovered-by P74; healed by P75 9e07028)"`. Commit body quotes the original entry's `**What:**` paragraph verbatim and appends the resolution sentence.
  </action>
  <verify>
    <automated>grep -q "STATUS:.*RESOLVED.*9e07028" .planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md &amp;&amp; jq -e '.rows[] | select(.id == "docs/social/linkedin/token-reduction-92pct") | .last_verdict == "BOUND"' quality/catalogs/doc-alignment.json</automated>
  </verify>
  <done>Entry 2 STATUS footer references commit 9e07028 and reads RESOLVED. Live catalog confirms linkedin row last_verdict is BOUND. One annotation-only commit landed.</done>
</task>

<task type="auto">
  <name>Task 4: Resolve entry 3 (Wave 2) — WONTFIX connector-matrix synonym + file new GOOD-TO-HAVES entry</name>
  <files>.planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md, .planning/milestones/v0.12.1-phases/GOOD-TO-HAVES.md</files>
  <action>
    Two-part resolution: WONTFIX the SURPRISES entry with rationale + pointer; ALSO file a new GOOD-TO-HAVES entry for P77 to absorb.

    1. Update SURPRISES-INTAKE.md entry 3 STATUS footer from `**STATUS:** OPEN` to (one paragraph max per D-04):
       ```
       **STATUS:** WONTFIX | rationale: The connector-matrix-on-landing.sh regex widening (commit c8e4111: `[Cc]onnector` -> `[Cc]onnector|[Bb]ackend`) is a complete fix — the verifier asserts the failure mode P74 cared about ("matrix accidentally deleted from landing") against the live heading "What each backend can do." Renaming the section to "## Connector capability matrix" to literal-match the catalog row claim text is purely cosmetic; filed as a P77 GOOD-TO-HAVE (size XS, impact clarity). Cost of revisiting later is low: the row stays BOUND under the widened regex, and the heading rename — when shipped — will narrow the regex back without coverage loss.
       ```
    2. Append a new entry to `.planning/milestones/v0.12.1-phases/GOOD-TO-HAVES.md` under `## Entries` (replacing the `_(none yet — populated by P72-P76 during execution)_` placeholder if it's still there):
       ```markdown
       ## discovered-by: P74 | size: XS | impact: clarity

       **What:** `docs/index.md:95` heading reads "## What each backend can do" — synonym for the catalog row claim "Connector capability matrix added to landing page" (row `polish2-06-landing`). P74 widened the verifier regex to accept either noun (`[Cc]onnector|[Bb]ackend`) per the eager-fix decision logged in P74's SUMMARY. Renaming the heading to "## Connector capability matrix" would let the verifier regex narrow back to a literal claim+heading match — a small clarity win, no behaviour change.

       **Proposed fix:** Edit `docs/index.md:95` heading to `## Connector capability matrix`. Optionally narrow `quality/gates/docs-alignment/connector-matrix-on-landing.sh` regex back to `[Cc]onnector` (single-noun match) once the rename lands.

       **STATUS:** OPEN
       ```
    3. Atomic commit: `git add .planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md .planning/milestones/v0.12.1-phases/GOOD-TO-HAVES.md && git commit -m "fix(p76): WONTFIX entry-3 connector-matrix synonym + file P77 GOOD-TO-HAVE (was: discovered-by P74)"`. Commit body quotes the original SURPRISES entry's `**What:**` paragraph verbatim and notes the GOOD-TO-HAVES.md handoff.
  </action>
  <verify>
    <automated>grep -q "STATUS:.*WONTFIX" .planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md &amp;&amp; grep -q "Connector capability matrix" .planning/milestones/v0.12.1-phases/GOOD-TO-HAVES.md &amp;&amp; grep -q "discovered-by: P74 | size: XS" .planning/milestones/v0.12.1-phases/GOOD-TO-HAVES.md</automated>
  </verify>
  <done>SURPRISES-INTAKE entry 3 marked WONTFIX with rationale paragraph. New XS GOOD-TO-HAVES entry filed (discovered-by P74, impact clarity). One combined commit landed covering both file edits.</done>
</task>

<task type="auto">
  <name>Task 5: Honesty spot-check (Wave 3, D-05) — sample 2 of P72-P75 plan/verdict pairs; record finding</name>
  <files>quality/reports/verdicts/p76/honesty-spot-check.md</files>
  <action>
    Per CONTEXT.md D-05 ("verifier subagent honesty check"): the EXECUTOR pre-grades by sampling 2 of {P72, P73, P74, P75}; the verifier subagent (Wave 7 dispatch) re-runs the same check from zero context. Two captures of the same evidence prevents a single agent's bias.

    1. **Choose 2 phases to sample.** Recommended pair: **P74** (highest intake yield: 2 entries) and **P75** (zero intake — the harder honesty case). This pair tests both the "found-and-logged" path AND the "looked-but-found-nothing-and-said-so" path. Alternative: P72 + P75, if the executor wants to spread coverage.
    2. **For each chosen phase**, read `.planning/phases/<phase>-*/PLAN.md` AND `quality/reports/verdicts/p<N>/VERDICT.md`. Look for:
       - **Eager-resolution decisions in the plan** — does the plan name an OP-8 / D-09 clause and explain how it will distinguish in-scope eager fixes from out-of-scope intake appends?
       - **Intake entries match plan's discovered findings** — every "I considered fixing X but it was out of scope" reasoning in the plan/SUMMARY should map to either an in-phase commit (eager-fix) or a SURPRISES-INTAKE entry. Conversely, every intake entry from that phase should trace back to a SUMMARY paragraph naming the discovery.
       - **Empty intake honesty** (P73, P75 case) — when a phase appended NO intake entries, the SUMMARY must state explicitly "we looked, found nothing out-of-scope" or "all discoveries were eager-fixed in-phase per OP-8." The P75 SUMMARY § "SURPRISES-INTAKE / GOOD-TO-HAVES appends" is the model: "none — the fix landed cleanly. No bind/walker bugs surfaced beyond the documented scope."
       - **No silent skips** — does any verdict file mention a finding that does NOT appear in either the SUMMARY's eager-fix list or the SURPRISES-INTAKE entries? That's a RED smell.
    3. **Write `quality/reports/verdicts/p76/honesty-spot-check.md`** with this structure:
       ```markdown
       # P76 Honesty Spot-Check (D-05)

       Sampled: <phase A>, <phase B>.

       ## <phase A>
       - PLAN.md OP-8/D-09 reference: <quote or 'absent'>
       - Eager-resolution decisions named in plan: <list>
       - Intake entries from this phase: <list of entries from SURPRISES-INTAKE.md by timestamp>
       - SUMMARY § naming each intake-discovery: <yes/no, with line refs>
       - VERDICT.md flag: <quote any honesty-related verifier note>
       - **Finding:** GREEN | YELLOW | RED  (rationale)

       ## <phase B>
       (same structure)

       ## Aggregate
       The 3 SURPRISES-INTAKE entries trace to 3 phases (P72, P74, P74); 4 phases ran (P72-P75); 2 phases produced intake (P72: 1, P74: 2); 2 produced none (P73, P75). The pattern of {found-some, found-none-and-said-so} is consistent with honest looking.

       **Aggregate finding: GREEN** (or RED with named gaps).
       ```
    4. **Note:** the prompt for this phase pre-asserts the honesty check should grade GREEN. The executor MUST still produce the document and verify the assertion holds against artifacts. If the spot-check disagrees with the prompt's prediction, the executor reports honestly and lets the verifier subagent (Wave 7) re-grade.
    5. **No commit yet** — this file lands in the SUMMARY commit (Wave 5) so SUMMARY can quote it.
  </action>
  <verify>
    <automated>test -f quality/reports/verdicts/p76/honesty-spot-check.md &amp;&amp; grep -q "Aggregate finding: GREEN" quality/reports/verdicts/p76/honesty-spot-check.md &amp;&amp; test $(grep -c "^## " quality/reports/verdicts/p76/honesty-spot-check.md) -ge 2</automated>
  </verify>
  <done>quality/reports/verdicts/p76/honesty-spot-check.md exists with 2 phase H2 sections + Aggregate H2; aggregate finding GREEN with rationale grounded in plan/verdict quotes.</done>
</task>

<task type="auto">
  <name>Task 6: CLAUDE.md P76 H3 (Wave 4) — &lt;=30-line subsection under v0.12.1 — in flight</name>
  <files>CLAUDE.md</files>
  <action>
    Per CONTEXT.md D-08 (P76 H3 lists resolutions inline) and CLAUDE.md QG-07 (each phase introducing a file/convention/gate updates CLAUDE.md):

    1. Locate the `## v0.12.1 — in flight` H2 (currently CLAUDE.md:325). The most recent H3 is `### P75 — bind-verb hash-overwrite fix` (CLAUDE.md:390-406).
    2. Append a new H3 immediately after the P75 H3 (and before the `## Quick links` H2 at line 408):
       ```markdown
       ### P76 — Surprises absorption

       Drained `.planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md` (3 LOW
       entries discovered during P72 + P74). The +2 phase practice (OP-8) is
       now operational: every intake entry has a terminal STATUS footer.

       Resolutions:
       - **Entry 1 (P72-discovered)**: 2 pre-existing STALE rows healed.
         polish-03-mermaid-render -> RESOLVED | <commit-sha-1>
         (rebind|propose-retire). cli-subcommand-surface -> RESOLVED |
         <commit-sha-2> (rebind|propose-retire).
       - **Entry 2 (P74-discovered)**: linkedin Source::Single -> RESOLVED |
         healed by P75 commit 9e07028 (audit-trail annotation only).
       - **Entry 3 (P74-discovered)**: connector-matrix synonym -> WONTFIX |
         regex widening (c8e4111) is the complete fix; heading rename filed
         as P77 GOOD-TO-HAVE (size XS, impact clarity).

       Honesty spot-check (D-05): sampled <phase A> + <phase B> plan/verdict
       pairs. Aggregate finding GREEN — intake yield (P72: 1, P74: 2, P73: 0,
       P75: 0) is consistent with phases honestly looking. Evidence at
       quality/reports/verdicts/p76/honesty-spot-check.md.

       Catalog deltas: claims_bound delta = (+2 if both rebound; -2 if both
       propose-retired; mixed otherwise). Live walker post-resolution shows
       zero net new STALE_DOCS_DRIFT.
       ```
       Replace `<commit-sha-1>`, `<commit-sha-2>`, `<phase A>`, `<phase B>` with the actual values from Tasks 2 + 5.
    3. Verify line count: `awk '/^### P76/,/^### |^## /' CLAUDE.md | wc -l` &lt;= 30 (P76 H3 body lines, exclusive of the next-section header). If over, condense the resolutions into single-line bullets.
    4. **Banned-words lint:** `bash scripts/banned-words-lint.sh CLAUDE.md` MUST pass.
    5. Atomic commit: `git add CLAUDE.md && git commit -m "docs(p76): CLAUDE.md P76 H3 — surprises absorption (3 entries dispositioned)"`.
  </action>
  <verify>
    <automated>grep -q "### P76 — Surprises absorption" CLAUDE.md &amp;&amp; awk '/^### P76/{flag=1; next} /^### |^## /{if(flag){flag=0}} flag' CLAUDE.md | wc -l | awk '{exit ($1 &gt; 30)}' &amp;&amp; bash scripts/banned-words-lint.sh CLAUDE.md</automated>
  </verify>
  <done>CLAUDE.md `### P76 — Surprises absorption` H3 exists under `## v0.12.1 — in flight` (after P75 H3), &lt;=30 body lines, banned-words lint passes. One commit landed.</done>
</task>

<task type="auto">
  <name>Task 7: SUMMARY.md (Wave 5) — phase summary with disposition table, honesty finding, deltas, verifier pointer</name>
  <files>.planning/phases/76-surprises-absorption/SUMMARY.md</files>
  <action>
    Use `~/.claude/get-shit-done/templates/summary.md` shape. SUMMARY MUST include:

    1. **Frontmatter** (matching P75's pattern):
       ```yaml
       ---
       phase: 76-surprises-absorption
       plan: 01
       status: COMPLETE
       requirement_closed: SURPRISES-ABSORB-01
       milestone: v0.12.1
       mode: --auto (sequential gsd-executor on main)
       duration_min: <wall-clock minutes>
       verifier_verdict_path: quality/reports/verdicts/p76/VERDICT.md
       ---
       ```
    2. **One-liner**: "Drained the v0.12.1 SURPRISES-INTAKE (3 LOW entries) to terminal status; +2 phase practice (OP-8) is operational; honesty spot-check on P72-P75 graded GREEN."
    3. **Disposition table** (reproduce from triage.md, but with FINAL commit SHAs):
       ```
       | Entry | Discovered-by | Severity | Disposition | Commit / Rationale |
       |-------|---------------|----------|-------------|--------------------|
       | 1a polish-03 | P72 | LOW | <REBIND or RETIRE_PROPOSED> | <sha> |
       | 1b cli-subcommand-surface | P72 | LOW | <REBIND or RETIRE_PROPOSED> | <sha> |
       | 2 linkedin Source::Single | P74 | LOW | RESOLVED (annotated) | 9e07028 (P75 heal) |
       | 3 connector-matrix synonym | P74 | LOW | WONTFIX + new GOOD-TO-HAVE | <sha> + P77-pointer |
       ```
    4. **Commits table** (newest first, like P75 SUMMARY § "Commits"):
       ```
       | SHA | Type | What |
       |-----|------|------|
       | <SUMMARY-sha> | docs | this SUMMARY |
       | <CLAUDE.md-sha> | docs | CLAUDE.md P76 H3 |
       | <entry-3-sha> | fix | WONTFIX entry-3 + GOOD-TO-HAVE filing |
       | <entry-2-sha> | fix | RESOLVED entry-2 linkedin (annotation) |
       | <entry-1-evidence-sha> | fix | RESOLVED entry-1 walk + status + intake-footer |
       | <entry-1b-sha> | fix | RESOLVED entry-1b cli-subcommand-surface |
       | <entry-1a-sha> | fix | RESOLVED entry-1a polish-03-mermaid-render |
       ```
    5. **Catalog deltas** (from status-after.txt vs P75's post-state):
       ```
       | Metric | Pre-P76 | Post-P76 | Delta |
       |--------|---------|----------|-------|
       | claims_bound | 329 | <X> | <delta> |
       | claims_retire_proposed | 27 | <Y> | <delta> |
       | claims_stale_docs_drift | <pre-stale> | <post-stale> | <delta> |
       | alignment_ratio | 0.9190 | <Z> | <delta> |
       ```
    6. **Honesty spot-check inline summary** (quote 3-5 lines from honesty-spot-check.md aggregate; full file referenced).
    7. **Verifier dispatch handoff** (mirror P75 SUMMARY § "Verifier verdict"):
       - State the executing agent does NOT grade itself (CLAUDE.md OP-7).
       - Top-level orchestrator dispatches `gsd-verifier` Path A via `Task` tool, N=76.
       - Verifier reads with zero session context: SURPRISES-INTAKE.md (3 footers terminal), GOOD-TO-HAVES.md (1 new XS entry), quality/catalogs/doc-alignment.json (entry-1 row state), quality/reports/verdicts/p76/{triage.md, walk-after.txt, status-after.txt, honesty-spot-check.md}, CLAUDE.md (P76 H3 in `git diff main...HEAD`), .planning/phases/{72,73,74,75}-*/PLAN.md + their VERDICT.md files (D-05 honesty cross-check).
       - Verdict path: `quality/reports/verdicts/p76/VERDICT.md`.
       - Phase does NOT close until graded GREEN.
    8. **Self-Check section** (file-existence + commit-existence cross-check, like P75 SUMMARY).

    Atomic commit: `git add .planning/phases/76-surprises-absorption/SUMMARY.md quality/reports/verdicts/p76/honesty-spot-check.md && git commit -m "docs(p76): SUMMARY.md + honesty spot-check (D-05) — 3 intake entries dispositioned"`.
  </action>
  <verify>
    <automated>test -f .planning/phases/76-surprises-absorption/SUMMARY.md &amp;&amp; grep -q "requirement_closed: SURPRISES-ABSORB-01" .planning/phases/76-surprises-absorption/SUMMARY.md &amp;&amp; grep -q "verifier_verdict_path: quality/reports/verdicts/p76/VERDICT.md" .planning/phases/76-surprises-absorption/SUMMARY.md &amp;&amp; grep -q "9e07028" .planning/phases/76-surprises-absorption/SUMMARY.md &amp;&amp; grep -q "Honesty" .planning/phases/76-surprises-absorption/SUMMARY.md</automated>
  </verify>
  <done>.planning/phases/76-surprises-absorption/SUMMARY.md committed with frontmatter, disposition table, commits table, catalog deltas, honesty-spot-check inline summary, verifier dispatch handoff, self-check. honesty-spot-check.md committed alongside.</done>
</task>

<task type="auto">
  <name>Task 8: Prep verifier-dispatch handoff brief (Wave 6) — write the prompt the top-level orchestrator will dispatch</name>
  <files>quality/reports/verdicts/p76/verifier-prompt.md</files>
  <action>
    Per CLAUDE.md OP-7 + CONTEXT.md D-10: the executing agent does NOT grade itself. The TOP-LEVEL ORCHESTRATOR (not gsd-executor; gsd-executor lacks the `Task` tool) dispatches `gsd-verifier` Path A. This task captures the verbatim prompt + input list so the orchestrator can dispatch without context drift.

    1. Read the QG-06 prompt template from `quality/PROTOCOL.md` § "Verifier subagent prompt template".
    2. Write `quality/reports/verdicts/p76/verifier-prompt.md` with three sections:
       - **`## Dispatch invocation`** — the literal `Task(subagent_type="gsd-verifier", description="P76 verifier dispatch", prompt=...)` shape.
       - **`## Verifier prompt (verbatim, N=76)`** — the QG-06 template body with N=76 substituted, plus an explicit "the verifier MUST independently execute the D-05 honesty check (sample ≥2 of {P72,P73,P74,P75} plan+verdict pairs); do NOT just rubber-stamp the executor's pre-grade at quality/reports/verdicts/p76/honesty-spot-check.md."
       - **`## Inputs the verifier reads with zero session context`** — bulleted list:
         - .planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md (3 STATUS footers terminal: RESOLVED|WONTFIX)
         - .planning/milestones/v0.12.1-phases/GOOD-TO-HAVES.md (1 new XS entry)
         - quality/catalogs/doc-alignment.json (entry-1 rows state)
         - quality/reports/verdicts/p76/{triage.md, walk-after.txt, status-after.txt, honesty-spot-check.md}
         - CLAUDE.md (P76 H3 confirmed via `git diff main...HEAD -- CLAUDE.md`)
         - .planning/phases/{72,73,74,75}-*/PLAN.md + quality/reports/verdicts/p{72,73,74,75}/VERDICT.md (D-05 honesty cross-check)
         - .planning/phases/76-surprises-absorption/SUMMARY.md
       - **`## Verdict path`** — `quality/reports/verdicts/p76/VERDICT.md`. Phase does NOT close until graded GREEN.
    3. Atomic commit: `git add quality/reports/verdicts/p76/verifier-prompt.md && git commit -m "docs(p76): verifier-dispatch handoff brief (top-level orchestrator action)"`.

    Note: the executor's responsibility ends here. The TOP-LEVEL session (which has the `Task` tool) reads `verifier-prompt.md`, dispatches the subagent, and writes the verdict. If the orchestrator is operating sequentially through phases, the human / orchestrator surfaces this brief at phase close and runs the dispatch before moving to P77.
  </action>
  <verify>
    <automated>test -f quality/reports/verdicts/p76/verifier-prompt.md &amp;&amp; grep -q "gsd-verifier" quality/reports/verdicts/p76/verifier-prompt.md &amp;&amp; grep -q "N=76" quality/reports/verdicts/p76/verifier-prompt.md &amp;&amp; grep -q "D-05" quality/reports/verdicts/p76/verifier-prompt.md &amp;&amp; grep -q "honesty" quality/reports/verdicts/p76/verifier-prompt.md</automated>
  </verify>
  <done>quality/reports/verdicts/p76/verifier-prompt.md committed with dispatch invocation, verbatim QG-06 prompt (N=76), input list, verdict-path. Top-level orchestrator can dispatch the verifier without re-reading the SUMMARY.</done>
</task>

</tasks>

<verification>
Phase-level checks:
1. SURPRISES-INTAKE.md has zero `**STATUS:** OPEN` lines (`! grep -q "STATUS:.*OPEN" .planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md`).
2. GOOD-TO-HAVES.md has exactly 1 entry under `## Entries` (the new XS one filed by P76).
3. Live `target/release/reposix-quality doc-alignment status --json` reports zero net new STALE_DOCS_DRIFT introduced by P76 (entry-1's two pre-existing STALEs are out of STALE state; no other rows tipped).
4. CLAUDE.md `### P76 — Surprises absorption` H3 body &lt;=30 lines, banned-words lint clean.
5. quality/reports/verdicts/p76/honesty-spot-check.md exists, samples 2 phases, aggregate GREEN.
6. quality/reports/verdicts/p76/VERDICT.md exists and reads GREEN (verifier dispatch outcome).
7. All 7 commits present in `git log --oneline` between this phase's first and last commit.
</verification>

<success_criteria>
P76 ships when:
- All 3 SURPRISES-INTAKE entries are terminal (RESOLVED | WONTFIX | DEFERRED).
- 1 new GOOD-TO-HAVES.md XS entry filed (P74 connector-matrix heading rename).
- quality/catalogs/doc-alignment.json reflects entry-1 row mutations (rebind or propose-retire).
- quality/reports/verdicts/p76/{triage.md, walk-after.txt, status-after.txt, honesty-spot-check.md} all committed.
- CLAUDE.md P76 H3 &lt;=30 lines under `## v0.12.1 — in flight`, lists each disposition inline.
- .planning/phases/76-surprises-absorption/SUMMARY.md committed with frontmatter `status: COMPLETE`, `requirement_closed: SURPRISES-ABSORB-01`, verifier verdict pointer.
- Verifier subagent verdict at `quality/reports/verdicts/p76/VERDICT.md` is GREEN, including the D-05 honesty cross-check on at least 2 of {P72, P73, P74, P75}.
- Up to 7 commits land on `main` (Tasks 2-7 produce 5-7 commits depending on entry-1 split).
- No `git push`, no tag, no `cargo publish` (per HANDOVER §"What the owner owes").
</success_criteria>

<output>
After completion, create `.planning/phases/76-surprises-absorption/SUMMARY.md` per Task 7's spec. SUMMARY commit is the second-to-last commit; the final commit is whichever of {entry-3, CLAUDE.md, SUMMARY} lands last per the executor's chosen ordering.

The verifier verdict at `quality/reports/verdicts/p76/VERDICT.md` is created by the top-level orchestrator's Task-8 dispatch, NOT by gsd-executor. The phase does NOT close until that verdict is GREEN.
</output>
