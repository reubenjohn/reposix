---
quick_id: 260716-fmt
slug: gth-v15-35-docs-index-install-ia
title: "GTH-V15-35 — docs/index.md install-IA: nest 'Build from source' under 30-second install + surface bootstrap prose + destale L93"
status: ready
created: 2026-07-16
type: quick
autonomous: true    # runs through commit+push; Task 3 carries a CONDITIONAL STOP-BEFORE-PUSH (cargo / subagent-fan-out rebind)
files_modified:
  - docs/index.md
  - quality/catalogs/doc-alignment.json   # written ONLY via `reposix-quality doc-alignment bind`, never hand-edited
  - .planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md
  - .planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md
must_haves:
  truths:
    - "The 'Build from source (advanced)' <details> block sits immediately after the 30-second install tab set (after L67, before '## After — one commit'), collapsed and subordinate to the package-manager path — never promoted above or beside it."
    - "structure/install-leads-with-pkg-mgr-docs-index stays GREEN — the first pkg-mgr command (curl|sh / brew / cargo binstall / powershell irm) appears BEFORE the first git-clone/cargo-build source-compile snippet by byte offset in docs/index.md."
    - "The `reposix sim &` + `reposix init sim::demo /tmp/reposix-demo` bootstrap commands appear in VISIBLE prose (independent of the collapsed block) so the 'After — one commit' demo (~L69-76) has a visible step creating /tmp/reposix-demo."
    - "docs/index.md L93's stale 'Real-backend cells fill in once CI secret packs are wired (Phase 36)' claim is replaced with the real GitHub 320 ms / Confluence 202 ms figures (per docs/benchmarks/latency.md:42); the two-claim line is split so the soft-threshold clause becomes standalone."
    - "Every doc-alignment row whose line anchor shifted or whose content changed is re-anchored via `reposix-quality doc-alignment bind` (claim→test bindings preserved); `bash quality/gates/docs-alignment/walk.sh` reports zero STALE_DOCS_DRIFT."
    - "mkdocs-strict, mermaid-renders, banned-words, install-leads, and the 5-line-install claim all pass on the edited docs/index.md."
    - "One SURPRISES-INTAKE row is filed (MEDIUM) for the token-economy regen test gap."
    - "GTH-V15-35 STATUS = DONE; a targeted-staged commit is pushed to origin/main with a green pre-push."
  artifacts:
    - path: "docs/index.md"
      provides: "block relocated under install section + visible bootstrap prose + L93 split/destaled"
    - path: "quality/catalogs/doc-alignment.json"
      provides: "shifted/changed rows re-anchored to new line/hash via `bind` (no hand-edit)"
    - path: ".planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md"
      provides: "new MEDIUM row for the token-economy regen test gap"
    - path: ".planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md"
      provides: "GTH-V15-35 STATUS → DONE"
  key_links:
    - from: "docs/index.md 30-second install tab set (L46-67)"
      to: "relocated <details> Build-from-source block"
      via: "block inserted AFTER the four pkg-mgr tabs so git-clone/cargo-build stay below the pkg-mgr offset (install-leads GREEN)"
    - from: "docs/index.md soft-threshold clause (new standalone line)"
      to: "docs/index/soft-threshold-24ms"
      via: "reposix-quality doc-alignment bind recomputes source_hash at the new line"
    - from: "each shifted docs/index.md row"
      to: "its new line number in docs/index.md"
      via: "reposix-quality doc-alignment bind --row-id <id> --source docs/index.md:<new-lines> (BIND-RELOCATION-FIX, in-place same-file re-anchor)"
---

# Quick Task 260716-fmt — GTH-V15-35 docs/index.md install-IA fix (both addenda)

Owner-directed docs-IA fix on `docs/index.md`. Authoritative row:
`.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md` **L237-261** (GTH-V15-35, its
Addendum L245-252 and Addendum 2 L253-261). Execute BOTH addenda in the SAME wave as the
block move — no STALE_DOCS_DRIFT across a wave boundary (GTH-V15-28 convention).

## Ground truth (verified read-only during planning — authoritative)

- **The block to move** is `docs/index.md` **L120-136**: `<details markdown><summary><strong>Build
  from source (advanced)</strong></summary> … </details>`. It contains `git clone
  https://github.com/reubenjohn/reposix` (L126) and `cargo build --release --workspace --bins`
  (L127) — the two strings the install-leads gate keys on — plus the bootstrap lines
  `reposix sim &` (L129), `reposix init sim::demo /tmp/reposix-demo` (L130),
  `git checkout -B main refs/reposix/origin/main` (L131).
- **Insertion point** is immediately after L67 ("Full step-by-step in first-run.") and
  before L69 ("## After — one commit"). The four pkg-mgr install tabs live at L46-65 and
  are NOT edited.
- **install-leads gate is a byte-offset check** (`quality/gates/structure/freshness/install_paths.py`):
  the FIRST match of `brew install | cargo binstall | curl…| sh | powershell…irm` must
  appear BEFORE the FIRST match of `git clone https?:// | cargo build --release`. Moving the
  block to sit AFTER the pkg-mgr tabs (L46-65) keeps the git-clone/cargo-build offset BELOW
  the pkg-mgr offset → gate stays GREEN. **Never** add `git clone https://` or `cargo build
  --release` in the new visible prose (would create an earlier source-compile offset).
- **L93 is a two-claim line.** Current text: `Latency for each backend is captured in […].
  Sim cold init is 278 ms (soft threshold 500 ms); list-issues 7 ms; capabilities probe
  5 ms. Real-backend cells fill in once CI secret packs are wired (Phase 36).` The FIRST
  clause ("Sim cold init is 278 ms…") is hash-bound by catalog row
  `docs/index/soft-threshold-24ms` (source L93-93, tests: `headline-numbers-cross-check.py`,
  `test_headline_numbers_cross_check.py`, `latency-bench.sh`). The SECOND clause ("Real-backend
  cells fill in … Phase 36") is STALE — `docs/benchmarks/latency.md:42` already carries real
  **GitHub 320 ms / Confluence 202 ms** "Get one record" figures (verified 2026-07-16).
- **Mechanical rebind path CONFIRMED.** `reposix-quality doc-alignment bind --row-id <id>
  --claim <same> --source docs/index.md:<new-lines> --test <same>… --grade GREEN --rationale
  <text>` re-validates the citation, recomputes `source_hash` at the new lines, and REPLACES
  the row's same-file cite in place (BIND-RELOCATION-FIX P89, `commands/doc_alignment.rs:327-363`).
  Because the claim + tests are passed unchanged, the claim→test binding is PRESERVED — a pure
  re-anchor, **no subagent fan-out**. Fan-out (`/reposix-quality-refresh`) is only for RE-DERIVING
  claim→test bindings, which this task does not need.
- **Binary is pre-built → NO cargo needed.** `target/release/reposix-quality` (2026-07-15) and
  `target/debug/reposix-quality` exist; `walk.sh` auto-detects release-then-debug. The `bind`
  verb runs against the pre-built binary. If — contrary to this — the binary is absent or
  rejects (schema drift → would need `cargo build -p reposix-quality`), that is the STOP-AND-REPORT
  trigger below.
- **28 catalog rows are bound to docs/index.md.** Rows at L7-67 (badges, hero, install tabs)
  are ABOVE the edits → unshifted. Rows from L73 downward shift; the shift/rebind set is the
  11 rows enumerated in Task 3.

## Constraints (encode + honor — non-negotiable)

- **No cargo.** Do NOT run any `cargo` invocation. The `bind`/`walk` steps use the PRE-BUILT
  `reposix-quality` binary. If a cargo build becomes necessary (binary missing/incompatible),
  **STOP and report** — do not build. (The push's own pre-push hook is the one sanctioned
  cargo-adjacent run; never spawn a parallel cargo alongside it.)
- **Catalog is written ONLY via `reposix-quality doc-alignment bind`** — never hand-edit
  `quality/catalogs/doc-alignment.json`, never edit it to force-pass. The binary computes hashes.
- **Diagnostic walks use the wrapper** `bash quality/gates/docs-alignment/walk.sh` (grades a
  /tmp copy, no committed-catalog mutation) — NEVER the raw `reposix-quality doc-alignment walk`.
- **STOP-BEFORE-PUSH** if rebind genuinely needs subagent fan-out (`/reposix-quality-refresh`
  playbook) or a cargo build: checkpoint and hand to the top-level orchestrator. NEVER push
  with STALE_DOCS_DRIFT rows; NEVER skip the rebind. The pre-push `docs-alignment/walk.sh` gate
  BLOCKS on stale rows — that is the enforcement backstop, not something to route around.
- **Targeted staging ONLY.** `git add <specific paths>` — NEVER `git add -A` / `git add .`.
- **Do NOT touch `.planning/MANAGER-HANDOVER.md`.**
- **Never `--no-verify`** on commit or push. Push with Bash timeout ≥ 300000 ms (pre-push ~2min).
- **Banned-words Layer 1** applies to visible `docs/index.md` prose: no FUSE-era or git-native
  plumbing words (`partial-clone`, `promisor`, `stateless-connect`, `fast-import`, `protocol-v2`)
  and no `replace`. Describe the bootstrap as creating a "working tree" / "demo checkout", not a
  "partial-clone".
- **Uncommitted = didn't happen** — commit + push before stopping.

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
</execution_context>

<context>
@docs/index.md
@README.md
@docs/benchmarks/latency.md
@.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md
@.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md
@quality/gates/structure/freshness/install_paths.py
@.claude/skills/reposix-banned-words/SKILL.md
</context>

<tasks>

<task type="auto">
  <name>Task 1: PRE-EDIT doc-alignment baseline capture (attribution anchor)</name>
  <files>quality/catalogs/doc-alignment.json (read-only), docs/index.md (read-only)</files>
  <action>
    Capture the GREEN baseline BEFORE editing so all post-edit drift is fully attributable.

    1. Confirm the pre-built binary is usable WITHOUT cargo:
         target/release/reposix-quality doc-alignment status --json | head
       Must exit 0 and print coverage. If the binary is ABSENT or errors in a way that would
       require `cargo build -p reposix-quality`, STOP and report (no-cargo constraint) — do NOT build.
    2. Capture the pre-edit GREEN walk (grade-only /tmp copy — never the raw subcommand):
         bash quality/gates/docs-alignment/walk.sh ; echo "walk exit=$?"
       Expect exit 0 (baseline clean). Record the coverage/alignment summary.
    3. Enumerate the 28 rows bound to docs/index.md with their current line anchors + source_hash
       + tests + last_verdict (read-only jq/python over the catalog). Save to the scratch dir so
       the post-edit rebind (Task 3) can diff old→new anchors. The 11 rows expected to shift are
       listed verbatim in Task 3; record them plus their current line numbers now.
  </action>
  <verify>
    <automated>bash quality/gates/docs-alignment/walk.sh &amp;&amp; target/release/reposix-quality doc-alignment status --json >/dev/null &amp;&amp; echo "baseline GREEN + binary usable (no cargo)"</automated>
  </verify>
  <done>Pre-built `reposix-quality` confirmed usable without cargo; pre-edit `walk.sh` exits 0 (GREEN baseline); the 28 index.md rows' current anchors/hashes/tests are recorded for attribution.</done>
</task>

<task type="auto">
  <name>Task 2: Edit docs/index.md — relocate block + surface bootstrap prose + split/destale L93</name>
  <files>docs/index.md</files>
  <action>
    All three edits land in this one file, one wave (their interdependence sets the final line
    numbers Task 3 rebinds against).

    (a) RELOCATE THE BLOCK. Move the entire `<details markdown><summary><strong>Build from source
        (advanced)</strong></summary> … </details>` block (currently L120-136, plus its trailing
        blank) to sit immediately after L67 ("Full step-by-step in [first-run]…") and before the
        `## After — one commit` heading (L69). Keep the block byte-identical (collapsed `<details>`,
        same prose/commands) so its content hash is preserved for the moved rows.
        HARD CONSTRAINT (install-leads GREEN): the block must land AFTER the four pkg-mgr install
        tabs (curl/PowerShell/Homebrew/cargo-binstall, L46-65). The block's `git clone https://`
        + `cargo build --release` must remain BELOW the first pkg-mgr command by byte offset.
        Never promote from-source above/beside the primary package-manager path — it stays
        subordinate + collapsed.

    (b) ADDENDUM 1 — VISIBLE BOOTSTRAP PROSE. Independent of the collapsed block, add VISIBLE prose
        so the `## After — one commit` demo (~L69-76, which `cd`s into /tmp/reposix-demo) has a
        visible step that CREATES /tmp/reposix-demo. Surface the two bootstrap commands that every
        install method needs — modeled on README.md Quick Start L56-77:
          reposix sim &                              # start the simulator on :7878
          reposix init sim::demo /tmp/reposix-demo   # create the demo working tree
          cd /tmp/reposix-demo && git checkout -B main refs/reposix/origin/main
        Put these in a short visible fenced block just before/at the top of the "After — one commit"
        section so the demo reads top-to-bottom (create tree → edit → commit → push). BANNED-WORDS:
        describe as a "working tree" / "demo checkout" — do NOT write "partial-clone" (Layer 1),
        and do NOT add `git clone https://` or `cargo build --release` here (install-leads offset).

    (c) ADDENDUM 2 (MANDATORY) — SPLIT + DESTALE L93. Split the current two-claim L93 into TWO lines:
          Line A (keep, standalone): "Latency for each backend is captured in
            [`docs/benchmarks/latency.md`](benchmarks/latency.md). Sim cold init is `278 ms`
            (soft threshold `500 ms`); list-issues `7 ms`; capabilities probe `5 ms`."
            — this is the clause bound by `docs/index/soft-threshold-24ms`; keep those numbers verbatim.
          Line B (rewrite the STALE clause): replace "Real-backend cells fill in once CI secret packs
            are wired (Phase 36)." with the TRUTH, citing latency.md: e.g. "Real-backend numbers are
            already captured: get-one-record is `320 ms` against GitHub and `202 ms` against
            Confluence ([latency](benchmarks/latency.md))." (use the figures from
            docs/benchmarks/latency.md:42 — GitHub 320 ms / Confluence 202 ms). No "Phase 36",
            no "fill in once".
        Splitting decouples future single-claim edits from the hash-bound soft-threshold clause.

    Leave the install tabs (L46-65) and L19 ("5-line install") UNCHANGED — the 5-line count is
    verified in Task 4, not edited here.
  </action>
  <verify>
    <automated>python3 - <<'PY'
import re
t=open('docs/index.md').read()
# block relocated: git-clone offset must be AFTER the first pkg-mgr offset
pm=re.search(r'brew install|cargo binstall|curl[^\n]*\| ?sh|powershell[^\n]*irm', t, re.I)
src=re.search(r'git clone https?://|cargo build --release', t, re.I)
assert pm and src and pm.start() < src.start(), "install-leads offset broken"
# details block sits before 'After — one commit'
assert t.index('Build from source (advanced)') < t.index('## After — one commit'), "block not above After-section"
# visible bootstrap prose exists near the demo
assert 'reposix init sim::demo /tmp/reposix-demo' in t
# L93 destaled
assert 'Phase 36' not in t and 'fill in once' not in t, "stale Phase-36 clause still present"
assert '320' in t and '202' in t, "real backend figures missing"
print('index.md edits OK')
PY</automated>
  </verify>
  <done>Block sits under the install section (collapsed, after the pkg-mgr tabs, install-leads offset preserved); the `reposix sim &` / `reposix init sim::demo /tmp/reposix-demo` bootstrap is visible before the demo; L93 is split into two single-claim lines with the stale Phase-36 clause replaced by the real 320 ms / 202 ms figures; no plumbing/banned words added.</done>
</task>

<task type="auto">
  <name>Task 3: Mechanical doc-alignment rebind of shifted/changed rows (bind; no fan-out; STOP-BEFORE-PUSH escape hatch)</name>
  <files>quality/catalogs/doc-alignment.json (written ONLY via `reposix-quality doc-alignment bind`)</files>
  <action>
    Same wave as Task 2 — re-anchor every row whose line moved or whose content changed, so no row
    is left STALE_DOCS_DRIFT.

    1. Enumerate the now-stale rows:
         bash quality/gates/docs-alignment/walk.sh   # lists STALE_DOCS_DRIFT rows + their ids
       Expected shift/rebind set (11 rows; confirm against walk output — rebind exactly what walk flags):
         docs/index/demo-workflow-sed-git-commit   claim="sed -i edit to issues/0001.md followed by git commit -am and git push works"
             --test crates/reposix-remote/tests/push_conflict.rs::clean_push_emits_ok_and_mutates_backend
         docs/index/audit-trail-git-log            claim="Audit trail is git log (no SDK vendoring required)"
             --test quality/gates/docs-alignment/audit-trail-git-log.sh
         docs/index/tested-three-backends          claim="Architecture tested end-to-end against Confluence TokenWorld, GitHub reubenjohn/reposix, JIRA TEST project"
             --test quality/gates/docs-alignment/three-backends-tested.sh
         docs/index/soft-threshold-24ms            claim="Sim cold init is 278 ms (soft threshold 500 ms)"   [CONTENT-CHANGED: cite the new standalone Line A]
             --test quality/gates/perf/headline-numbers-cross-check.py
             --test quality/gates/perf/test_headline_numbers_cross_check.py
             --test quality/gates/perf/latency-bench.sh
         docs/index/backend-capabilities-struct    claim="BackendCapabilities struct exists in crates/reposix-core/src/backend.rs"
             --test crates/reposix-cli/src/doctor.rs::check_backend_capabilities_reports_sim_supports_everything
         docs/index/reposix-sim-starts-7878        claim="reposix sim starts on port :7878 by default"
             --test crates/reposix-sim/src/main.rs::default_bind_addr_is_7878
         docs/index/reposix-init-demo-command      claim="reposix init sim::demo /tmp/reposix-demo creates partial-clone working tree"
             --test crates/reposix-cli/tests/agent_flow.rs::dark_factory_sim_happy_path
         docs/index/git-checkout-branch-command    claim="git checkout -B main refs/reposix/origin/main switches to main branch"
             --test quality/gates/docs-repro/tutorial-replay.sh
         docs/index/bootstrap-latency-24ms         claim="Bootstrap takes <= 278 ms against the simulator (CI-canonical)"
             --test quality/gates/perf/latency-bench.sh
         docs/index/blob-limit-teaching            claim="Fetch size limit caps git fetch and emits stderr with git sparse-checkout recovery hint"
             --test crates/reposix-cli/tests/agent_flow.rs::dark_factory_blob_limit_teaching_string_present
         docs/index/push-conflict-detection        claim="Push-time conflict detection rejects stale-base pushes with standard git fetch first error"
             --test crates/reposix-cli/tests/agent_flow.rs::dark_factory_conflict_teaching_string_present

    2. For EACH flagged row, find the NEW line number where its exact content now lives in
       docs/index.md, then re-anchor (claim + tests passed VERBATIM from the list above → binding
       preserved). Use the pre-built binary:
         target/release/reposix-quality doc-alignment bind \
           --row-id <id> --claim "<claim verbatim>" \
           --source "docs/index.md:<new_start>-<new_end>" \
           --test "<test1>" [--test "<test2>" …] \
           --grade GREEN \
           --rationale "line-shift rebind after GTH-V15-35 block relocation (content byte-identical; soft-threshold clause split out of two-claim L93)"
       Notes:
         - For the four moved-block rows (reposix-sim-starts-7878, reposix-init-demo-command,
           git-checkout-branch-command, bootstrap-latency-24ms) cite the line INSIDE the relocated
           block (content byte-identical → hash unchanged, only line number changes). The commands
           that ALSO appear in the new visible prose are a second copy — cite the block copy to keep
           the stored hash valid.
         - For soft-threshold-24ms cite the NEW standalone Line A (the split changes that line's
           content, so `bind` recomputes the hash; the claim + 3 tests are unchanged).
    3. Re-verify: `bash quality/gates/docs-alignment/walk.sh` → expect exit 0, zero STALE_DOCS_DRIFT.

    STOP-BEFORE-PUSH triggers (checkpoint + hand to top-level orchestrator, do NOT push, do NOT
    hand-edit the catalog):
      - the pre-built binary rejects/needs a cargo rebuild; OR
      - any row genuinely needs a RE-DERIVED claim→test binding (not a pure line/hash re-anchor),
        i.e. the `/reposix-quality-refresh docs/index.md` subagent-fan-out playbook is required.
  </action>
  <verify>
    <automated>bash quality/gates/docs-alignment/walk.sh &amp;&amp; git diff --name-only | grep -qx 'quality/catalogs/doc-alignment.json' &amp;&amp; echo "walk GREEN, catalog rebound"</automated>
  </verify>
  <done>Every row `walk.sh` flags is re-anchored via `bind` to its new line/hash with claim→test bindings preserved; `walk.sh` exits 0 with zero STALE_DOCS_DRIFT; the catalog was mutated only through the binary (no hand-edit). If fan-out/cargo was required, the executor STOPPED before push and reported.</done>
</task>

<task type="auto">
  <name>Task 4: Docs-build + banned-words + install-leads + 5-line-install gates</name>
  <files>docs/index.md (read-only gate runs)</files>
  <action>
    Run every gate that a docs/** change must pass BEFORE commit:
      1. bash quality/gates/docs-build/mkdocs-strict.sh      # required for docs/** — expect exit 0
      2. bash quality/gates/docs-build/mermaid-renders.sh    # required for docs/** — expect exit 0
      3. scripts/banned-words-lint.sh                        # Layer 1 docs/index.md — expect exit 0
         Apply the reposix-banned-words self-check to the docs/index.md DIFF: no FUSE-era or
         git-native plumbing words (partial-clone / promisor / stateless-connect / fast-import /
         protocol-v2) and no `replace` in the new visible prose.
      4. structure/install-leads-with-pkg-mgr-docs-index — confirm GREEN (pkg-mgr offset before
         git-clone/cargo-build offset). Run via the freshness-invariants runner scoped to that row,
         or re-assert the offset directly (already checked in Task 2's verify).
      5. 5-line-install — run the bound test bash quality/gates/docs-alignment/install-snippet-shape.sh
         and confirm the "5-line install" claim (docs/index/5-line-install, L19) still holds summed
         across the four methods. Adjust docs/index.md L19 ONLY if the test fails (and if L19 content
         changes, rebind docs/index/5-line-install via `bind` in the same wave).
    Any RED here blocks the commit — fix within scope (or STOP-and-report per Task 3 escape hatch).
  </action>
  <verify>
    <automated>bash quality/gates/docs-build/mkdocs-strict.sh &amp;&amp; bash quality/gates/docs-build/mermaid-renders.sh &amp;&amp; scripts/banned-words-lint.sh &amp;&amp; bash quality/gates/docs-alignment/install-snippet-shape.sh &amp;&amp; echo "docs-build + banned-words + 5-line-install GREEN"</automated>
  </verify>
  <done>mkdocs-strict, mermaid-renders, banned-words, install-leads, and install-snippet-shape all pass on the edited docs/index.md; the "5-line install" claim still holds (adjusted + rebound only if the test demanded it).</done>
</task>

<task type="auto">
  <name>Task 5: File the SURPRISES-INTAKE row (token-economy regen test gap)</name>
  <files>.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md</files>
  <action>
    Append ONE new row (Edit, follow the file's existing `## YYYY-MM-DD HH:MM | discovered-by: … |
    severity: MEDIUM` format at L18-28) documenting:
      What: `test_main_offline_regenerates_doc_from_captures` in
        `quality/gates/perf/test_bench_token_economy.py` (L212-244) asserts idempotency ONLY against
        a synthetic `tmp_path` fixture (it monkeypatches `bench.RESULTS`/`CAPTURES`/`BENCH_DIR` to a
        temp dir and checks a second `--offline` run reproduces the same bytes) and NEVER diffs the
        regenerated doc against the real committed `docs/benchmarks/token-economy.md`. That is the
        exact gap that let the 260716-f6o generator regression (the retired-narrative section
        re-added by the template) reach a P115 phase-close gate run undetected.
      Severity: MEDIUM.
      Sketched resolution: add a regression test that runs the offline regenerator against the REAL
        committed captures and byte-compares (sha256) the output against the committed
        `docs/benchmarks/token-economy.md` — so any generator/doc divergence fails a test instead of
        silently dirtying the working tree at a gate run.
      discovered-by: quick 260716-fmt (GTH-V15-35). STATUS: OPEN.
    Do NOT touch `.planning/MANAGER-HANDOVER.md`.
  </action>
  <verify>
    <automated>grep -q "test_main_offline_regenerates_doc_from_captures" .planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md &amp;&amp; grep -q "260716-fmt" .planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md &amp;&amp; git diff --quiet -- .planning/MANAGER-HANDOVER.md</automated>
  </verify>
  <done>SURPRISES-INTAKE.md carries one well-formed MEDIUM row naming the byte-compare gap in `test_main_offline_regenerates_doc_from_captures`, citing quick 260716-fmt, with a byte-compare-against-committed-doc fix sketch; MANAGER-HANDOVER.md untouched.</done>
</task>

<task type="auto">
  <name>Task 6: GTH-V15-35 STATUS → DONE, targeted-staged commit, push origin main</name>
  <files>.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md, docs/index.md, quality/catalogs/doc-alignment.json, .planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md</files>
  <action>
    1. Update the GTH-V15-35 STATUS line (GOOD-TO-HAVES.md L238) from "SCHEDULED …" to
       "DONE — quick 260716-fmt (2026-07-16)". (The commit landing this row IS the ref; append the
       SHA in the SUMMARY rather than a second amend-commit, to keep this a single clean commit.)
    2. Targeted staging ONLY — never -A/.:
         git add docs/index.md \
                 quality/catalogs/doc-alignment.json \
                 .planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md \
                 .planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md \
                 .planning/quick/20260716-gth-v15-35-docs-index-install-ia/PLAN.md
         # (also stage 260716-fmt-SUMMARY.md if written under the quick dir)
       Confirm `git diff --cached --name-only` lists ONLY those paths and does NOT include
       `.planning/MANAGER-HANDOVER.md` or any unrelated file.
    3. Commit (NO --no-verify; pre-commit runs banned-words + fmt) with a message such as:
         docs(index): nest build-from-source under 30-second install + surface bootstrap + destale L93 (GTH-V15-35, 260716-fmt)

         Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>
    4. Push (NO --no-verify; pre-push ~2min — set Bash timeout ≥ 300000 ms):
         git push origin main
       If the pre-push `docs-alignment/walk.sh` (or any gate) BLOCKS on STALE_DOCS_DRIFT, do NOT
       bypass — return to Task 3, rebind the named row, re-commit, re-push. If a cargo build or
       subagent fan-out is what's required, STOP and report (Task 3 escape hatch).
    5. After push, confirm main is green (do not declare green without seeing green):
         gh run list --branch main --limit 1
       (and/or `python3 quality/runners/run.py --cadence post-push` — the `code/ci-green-on-main`
       probe is the authoritative check). Record the run conclusion + commit SHA in the SUMMARY.
  </action>
  <verify>
    <automated>git diff --cached --name-only | grep -q 'MANAGER-HANDOVER' &amp;&amp; { echo 'FAIL: handover staged'; exit 1; }; grep -q 'DONE' <(sed -n '238p' .planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md) &amp;&amp; git log origin/main -1 --oneline | grep -qi '260716-fmt\|GTH-V15-35' &amp;&amp; echo 'pushed + STATUS DONE'</automated>
  </verify>
  <done>GTH-V15-35 STATUS = DONE; commit lands on origin/main with ONLY the targeted paths staged (no MANAGER-HANDOVER, no unrelated files); pre-push passed; main CI confirmed green; no cargo run by this task.</done>
</task>

</tasks>

<verification>
- `docs/index.md`: Build-from-source `<details>` sits after L67 / before "After — one commit",
  collapsed; pkg-mgr offset < git-clone/cargo-build offset (install-leads GREEN).
- Visible `reposix sim &` + `reposix init sim::demo /tmp/reposix-demo` bootstrap present near the demo.
- L93 split into two lines; "Phase 36"/"fill in once" gone; real 320 ms / 202 ms figures present.
- `bash quality/gates/docs-alignment/walk.sh` → exit 0, zero STALE_DOCS_DRIFT (rows rebound via `bind`).
- mkdocs-strict + mermaid-renders + banned-words + install-snippet-shape → all exit 0.
- SURPRISES-INTAKE.md has one new MEDIUM row (token-economy regen byte-compare gap); MANAGER-HANDOVER.md untouched.
- GOOD-TO-HAVES.md GTH-V15-35 STATUS = DONE.
- Commit on origin/main with ONLY the targeted paths staged; no cargo run by this task; main CI green.
</verification>

<success_criteria>
1. "Build from source (advanced)" nested under 30-second install, collapsed + subordinate; install-leads gate GREEN.
2. Bootstrap commands surfaced in visible prose so the demo has a visible /tmp/reposix-demo creation step.
3. L93's stale Phase-36 claim replaced by real 320 ms / 202 ms figures; two-claim line split.
4. All shifted/changed doc-alignment rows re-anchored via `bind` (no fan-out, no cargo, no hand-edit); walk GREEN.
5. Docs-build + banned-words + 5-line-install gates pass.
6. One MEDIUM SURPRISES-INTAKE row filed for the token-economy regen test gap.
7. GTH-V15-35 = DONE; targeted-staged commit pushed to origin/main; pre-push + main CI green.
</success_criteria>

<output>
After completion, create
`.planning/quick/20260716-gth-v15-35-docs-index-install-ia/260716-fmt-SUMMARY.md`
recording: the block's old→new line range, the exact rebound row ids with old→new source lines,
the L93 before/after text, the SURPRISES row filed, the pushed commit hash, and the main-CI-green
confirmation. Note anything noticed near the work (OD-3 noticing deliverable) — e.g. other index.md
claims whose line-number citations are now brittle, or docstring line references gone stale.
</output>
