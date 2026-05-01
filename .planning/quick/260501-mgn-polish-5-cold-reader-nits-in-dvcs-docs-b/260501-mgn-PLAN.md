---
phase: 260501-mgn-polish-5-cold-reader-nits-in-dvcs-docs-b
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - docs/concepts/dvcs-topology.md
  - docs/guides/dvcs-mirror-setup.md
autonomous: true
requirements:
  - dvcs-cold-reader-finding-1-binstall-crate-name
  - dvcs-cold-reader-finding-2-secret-set-interactive-note
  - dvcs-cold-reader-finding-3-sed-readability
  - dvcs-cold-reader-finding-4-slug-hedge
  - dvcs-cold-reader-finding-5-template-anchor

must_haves:
  truths:
    - "docs/concepts/dvcs-topology.md line 134 says `cargo binstall reposix-cli` (NOT `cargo binstall reposix`)"
    - "docs/guides/dvcs-mirror-setup.md secret-set block (lines 61-63 region) has a one-line note flagging interactive prompt + non-TTY pointer"
    - "docs/guides/dvcs-mirror-setup.md cron-update sed example (line 147 region) is more readable than the doubly-escaped form"
    - "docs/concepts/dvcs-topology.md line 61 region states the slug is `confluence` definitively (no 'or your tenant alias' hedge)"
    - "docs/guides/dvcs-mirror-setup.md has a one-line anchor near top (after intro, before Step 1) naming docs/guides/dvcs-mirror-setup-template.yml as the workflow template location"
    - "docs/concepts/dvcs-topology.md still passes quality/gates/docs-alignment/dvcs-topology-three-roles.sh (three role tokens + Q2.2 phrase fragments still present)"
    - "docs/guides/dvcs-mirror-setup.md still passes quality/gates/docs-alignment/dvcs-mirror-setup-walkthrough.sh (Steps 1-5, Cleanup, Backends without webhooks, gh secret set, template reference all present)"
    - "scripts/banned-words-lint.sh exits 0 (no FUSE/kernel/partial-clone/promisor/stateless-connect/fast-import/protocol-v2 leaks introduced)"
    - "scripts/check-docs-site.sh exits 0 (mkdocs strict build still green)"
  artifacts:
    - path: "docs/concepts/dvcs-topology.md"
      provides: "Updated topology doc with binstall crate-name fix + slug hedge removal"
      contains: "cargo binstall reposix-cli"
    - path: "docs/guides/dvcs-mirror-setup.md"
      provides: "Updated setup guide with template anchor + secret-set interactive note + readable sed example"
      contains: "dvcs-mirror-setup-template.yml"
  key_links:
    - from: "docs/concepts/dvcs-topology.md (Pattern C, line ~134)"
      to: "the binstall crate name `reposix-cli` (matches dvcs-mirror-setup.md:89)"
      via: "literal string"
      pattern: "cargo binstall reposix-cli"
    - from: "docs/guides/dvcs-mirror-setup.md (intro region)"
      to: "docs/guides/dvcs-mirror-setup-template.yml"
      via: "anchor sentence near top of file"
      pattern: "dvcs-mirror-setup-template\\.yml"
---

<objective>
Polish 5 non-critical findings surfaced by the dvcs-cold-reader rubric verdict
(score 8 CLEAR, zero critical-friction). All five are docs-only edits across
two files (docs/concepts/dvcs-topology.md + docs/guides/dvcs-mirror-setup.md).
The verdict artifact at quality/reports/verifications/subjective/dvcs-cold-reader.json
stays AS-IS — owner re-grades on the next /reposix-quality-review run.

Purpose: tighten the cold-reader experience for the DVCS docs cluster. Three
findings fix a correctness gap a copy-paster would hit (binstall crate name,
secret-set non-TTY behavior, slug hedge ambiguity). Two improve grounding
(template anchor near top of setup guide, readable sed example).

Output: one atomic commit `docs(dvcs): polish 5 cold-reader nits (binstall crate name + secret-set note + sed readability + slug hedge + template anchor)`.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@CLAUDE.md
@.planning/STATE.md
@docs/concepts/dvcs-topology.md
@docs/guides/dvcs-mirror-setup.md
@quality/reports/verifications/subjective/dvcs-cold-reader.json

<catalog_safety_pre_audit>
Pre-audit completed during planning — embedded here so the executor does not
re-derive it:

1. **Subjective rubric (dvcs-cold-reader)** lives in
   `quality/catalogs/subjective-rubrics.json` (dimension `subjective`). The
   subjective dimension does NOT track `source_hash` per row — verdicts are
   subagent-graded JSON artifacts with TTL-based freshness. Editing
   docs/concepts/ + docs/guides/ files does NOT tip a subjective row.

2. **docs-alignment dimension** tracks `source_hash` of cited line ranges. Two
   rows cite the target files:
   - `docs-alignment/dvcs-topology-three-roles-bound` — hashes
     `docs/concepts/dvcs-topology.md` **lines 5-11** (intro paragraph). Edit
     points (line 61, line 134) lie OUTSIDE this range. No hash drift.
   - `docs-alignment/dvcs-mirror-setup-walkthrough-bound` — hashes
     `docs/guides/dvcs-mirror-setup.md` **lines 5-11** (intro paragraph + "What
     you need before you start" boundary). Edit points (lines 61-63, line 147,
     line 89-region) lie OUTSIDE this range. The new template-anchor sentence
     (Finding 5) MUST be inserted at line 12 or later (i.e. AFTER the hashed
     intro range, BEFORE Step 1 at line 21) to avoid shifting line 5-11 content.
     The verdict artifact's reading of "right after the intro paragraph" is
     compatible with line 12+.
   - `docs-alignment/dvcs-troubleshooting-matrix-bound` — hashes
     `docs/guides/troubleshooting.md` line 227 only. Troubleshooting.md is NOT
     edited; row is unaffected.

3. **Verifier-script invariants** (`quality/gates/docs-alignment/dvcs-*.sh`):
   - `dvcs-topology-three-roles.sh` requires three role tokens (`SoT-holder`,
     `mirror-only consumer`, `round-tripper`) + Q2.2 phrase fragments
     (`mirror last caught up`, `NOT a`, `current SoT state`). None of the five
     edits touch these tokens.
   - `dvcs-mirror-setup-walkthrough.sh` requires `Step 1`..`Step 5`, `Cleanup`,
     `Backends without webhooks`, `gh repo create`, `gh secret set`,
     `gh workflow disable`, and a `dvcs-mirror-setup-template.yml` reference.
     Finding 5 ADDS a template reference (anchor); the existing reference at
     line 35 stays. No invariant broken.

Conclusion: doc-alignment walker will NOT fire `STALE_DOCS_DRIFT` on these
edits provided the line-12+ anchor placement is honored. No
`reposix-quality doc-alignment plan-refresh` invocation needed.
</catalog_safety_pre_audit>

<interfaces>
<!-- Exact line targets and verbatim before/after strings the executor uses. -->
<!-- Extracted from current file contents during planning. No exploration needed. -->

# Finding 1 — docs/concepts/dvcs-topology.md:134 (binstall crate name)

Current line 134 reads:
```
cargo binstall reposix
```

Replace with:
```
cargo binstall reposix-cli
```

Rationale (verbatim from dvcs-cold-reader.json findings + CLAUDE.md
"Webhook-driven mirror sync (v0.13.0 P84+)"): published crate name is
`reposix-cli`; bare `reposix` is the workspace name and would 404 on binstall.
docs/guides/dvcs-mirror-setup.md:89 already says `reposix-cli`; line 134 is
inconsistent.

# Finding 4 — docs/concepts/dvcs-topology.md:61 (slug hedge)

Current line 61 reads:
```
For a Confluence SoT at `reuben-john.atlassian.net`, the host slug renders as `confluence` (or your tenant alias). The bus push and the webhook sync both write these refs — the bus push because the bus push **is** a sync from a developer's perspective; the webhook because the webhook is the only writer between bus pushes.
```

Replace with (drop the "or your tenant alias" hedge; reword for definitiveness):
```
For a Confluence SoT at `reuben-john.atlassian.net`, the host slug is `confluence`. (The slug always names the backend kind, not your tenant. The four canonical slugs are `sim`, `github`, `confluence`, `jira`.) The bus push and the webhook sync both write these refs — the bus push because the bus push **is** a sync from a developer's perspective; the webhook because the webhook is the only writer between bus pushes.
```

Rationale: per CLAUDE.md "Mirror-lag refs" — `<sot-host>` is the SoT backend
slug (`sim` | `github` | `confluence` | `jira`). Tenant aliases are
irrelevant to the slug name; the hedge invites confusion.

# Finding 2 — docs/guides/dvcs-mirror-setup.md:61-63 (secret-set interactive note)

Current lines 60-63:
```
# Secrets — encrypted, only visible to the workflow.
gh secret set ATLASSIAN_API_KEY            # paste the API token from prerequisites
gh secret set ATLASSIAN_EMAIL              # your Atlassian account email
gh secret set REPOSIX_CONFLUENCE_TENANT    # e.g. 'acme' for acme.atlassian.net
```

Insert a one-line note IMMEDIATELY after line 63 (i.e. as a new line 64,
followed by a blank line preserving fenced-code closure context). The note
goes OUTSIDE the bash code fence (after the closing ``` on line 67 of the
"Variables" block). Concretely: the new note paragraph lands after the closing
``` (currently line 67), BEFORE the existing "Verify:" section (line 69).

Insert this paragraph after the closing ``` of the Step 3 secrets-and-vars
code block:
```
> **Note:** `gh secret set <NAME>` (without `--body`) prompts interactively for the secret value. For non-TTY contexts (CI, automation), pipe the value via stdin or pass `--body`: `printf '%s' "$TOKEN" | gh secret set ATLASSIAN_API_KEY` or `gh secret set ATLASSIAN_API_KEY --body "$TOKEN"`.
```

Single sentence per the brief; uses the markdown blockquote-Note pattern
already used in this file (e.g. line 31 "Tip:") for visual consistency.
Choose anchor: place AFTER the closing ``` of the variables block (currently
line 67) and BEFORE the "Verify:" line (currently line 69). The note belongs
between the secret-set commands and the verify block.

# Finding 3 — docs/guides/dvcs-mirror-setup.md:147 (sed readability)

Current lines 145-148 (the cron-update example inside the "Updating the cron
cadence" section):
```bash
cd /tmp/<space>-mirror
sed -i "s|cron: '\\*/30 \\* \\* \\* \\*'|cron: '\\*/15 \\* \\* \\* \\*'|" \
  .github/workflows/reposix-mirror-sync.yml
git add .github/workflows/reposix-mirror-sync.yml
```

Replace the sed line (line 147 + its continuation 148) with the more readable
single-line form (no backslash-newline; no doubly-escaped backslashes — just
single backslash before `*` for the BRE literal-asterisk):
```bash
cd /tmp/<space>-mirror
sed -i "s|'\*/30 \* \* \* \*'|'\*/15 \* \* \* \*'|" .github/workflows/reposix-mirror-sync.yml
git add .github/workflows/reposix-mirror-sync.yml
```

Verification of correctness: in `sed` BRE, `*` is a metacharacter (zero-or-more
of preceding atom); to match a literal asterisk it must be escaped as `\*`.
The double-quoted shell context turns `\\*` into `\*` (the bash escape). The
simplified form writes `\*` directly inside double quotes — bash leaves
backslash-asterisk untouched (asterisk is not a shell metachar inside double
quotes; backslash is preserved when followed by a non-special char), so sed
receives `\*` exactly as the doubly-escaped form did.

Portability note (BSD vs GNU sed): both forms behave identically across
GNU sed (Linux) and BSD sed (macOS/FreeBSD); the only BSD-vs-GNU divergence
in this command is `sed -i` (BSD requires an explicit empty backup arg
`sed -i ''`), which is unchanged from the existing form. The brief said
"note it but pick the form that's most readable" — no further note needed
because the readability fix does not introduce new portability concerns
beyond what was already present.

# Finding 5 — docs/guides/dvcs-mirror-setup.md (template anchor)

Current state: `dvcs-mirror-setup-template.yml` is referenced 4× across
the docs cluster (concepts/dvcs-topology, guides/dvcs-mirror-setup at line 35,
guides/dvcs-mirror-setup at line 196, guides/troubleshooting at line 294)
but no single anchor near the top of dvcs-mirror-setup.md names where the
file lives.

Insert a one-line anchor sentence between the existing intro section
(lines 1-19, hashed by docs-alignment row) and Step 1 (line 21). The intro
ends at line 19 (the existing tip about REPOSIX_ALLOWED_ORIGINS). Line 20
is currently blank. Insert at line 20 region — but BEFORE "## Step 1 — Create
the mirror repository" on line 21 — a single anchor paragraph:

```
> **Workflow template location:** the GitHub Action workflow this guide installs lives at [`docs/guides/dvcs-mirror-setup-template.yml`](dvcs-mirror-setup-template.yml) in the reposix repo. Step 2 below curls it from `main`.
```

The anchor lands AFTER line 11 (the boundary of the hashed
docs-alignment-row range, lines 5-11) — concretely it lands after line 19
(end of "What you need before you start") and before "## Step 1 ..." on
line 21. This guarantees zero `STALE_DOCS_DRIFT` on the row hashing lines 5-11.

Format choice: blockquote `> **Label:** ...` matches the existing `> **Tip:**`
convention at line 31. One sentence; one note paragraph; pointer to Step 2.
</interfaces>

<verification_steps_summary>
After all five edits land:

1. `bash scripts/banned-words-lint.sh` exits 0 (no FUSE/kernel/partial-clone/
   promisor/stateless-connect/fast-import/protocol-v2 leaks). The new prose
   uses no banned terms — manual scan during edit confirms.

2. `bash scripts/check-docs-site.sh` exits 0 (mkdocs strict build green;
   the existing `[`docs/guides/dvcs-mirror-setup-template.yml`](dvcs-mirror-setup-template.yml)`
   link form already used at lines 35 + 196 is mkdocs-compatible).

3. `bash quality/gates/docs-alignment/dvcs-topology-three-roles.sh` exits 0
   (three role tokens + Q2.2 phrase fragments unaffected by lines 61, 134
   edits).

4. `bash quality/gates/docs-alignment/dvcs-mirror-setup-walkthrough.sh` exits 0
   (Steps 1-5, Cleanup, Backends without webhooks, gh secret set, template
   reference all preserved; the new anchor adds an extra template reference).

5. `cargo run -p reposix-quality -- doc-alignment walk` (optional spot-check;
   if invoked, must pass — but per pre-audit, the bound rows hash lines 5-11
   only and those lines are NOT edited, so STALE_DOCS_DRIFT will not fire).
   NOTE: cargo invocation is permitted at most once during this task per
   CLAUDE.md "Build memory budget"; sequential, not parallel with anything.
</verification_steps_summary>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Apply 5 cold-reader polish edits across 2 docs files in a single atomic commit</name>
  <files>docs/concepts/dvcs-topology.md, docs/guides/dvcs-mirror-setup.md</files>
  <action>
Apply the five edits described in the &lt;interfaces&gt; section verbatim. Use the
Edit tool (or sed for the three single-line substitutions; Edit tool for the
two paragraph insertions). All five edits land in ONE commit.

Edit order (matters only for line-number stability — apply in this sequence
within each file so later line references stay valid):

**File 1: docs/concepts/dvcs-topology.md** — apply both edits before moving
to file 2. Edits at line 134 and line 61 are in different regions; either
order works. Recommended: edit line 134 first (later in file), then line 61.

  - Finding 1 (line 134): change `cargo binstall reposix` → `cargo binstall reposix-cli`.
  - Finding 4 (line 61): replace the sentence "For a Confluence SoT at
    `reuben-john.atlassian.net`, the host slug renders as `confluence` (or
    your tenant alias)." with "For a Confluence SoT at
    `reuben-john.atlassian.net`, the host slug is `confluence`. (The slug
    always names the backend kind, not your tenant. The four canonical slugs
    are `sim`, `github`, `confluence`, `jira`.)" — the sentence continues
    " The bus push and the webhook sync both write these refs..." unchanged.

**File 2: docs/guides/dvcs-mirror-setup.md** — apply edits in REVERSE line
order (line 147 first, then lines 60-67 secret-set block region, then line
20-region anchor) so earlier-line insertions do not shift later-line targets.

  - Finding 3 (line 147): replace the multi-line `sed -i "s|cron: '\\*/30
    \\* \\* \\* \\*'|cron: '\\*/15 \\* \\* \\* \\*'|" \\` and continuation
    line `  .github/workflows/reposix-mirror-sync.yml` with the single-line
    form: `sed -i "s|'\*/30 \* \* \* \*'|'\*/15 \* \* \* \*'|" .github/workflows/reposix-mirror-sync.yml`.
  - Finding 2 (after line ~67, before line ~69): insert the blockquote
    `> **Note:** ...` paragraph between the closing ``` of the secret/var
    block and the "Verify:" line. See &lt;interfaces&gt; section Finding 2 for
    verbatim text.
  - Finding 5 (between line 19 and line 21, i.e. as a new paragraph after
    the intro section and before "## Step 1 ..."): insert the
    `> **Workflow template location:** ...` blockquote anchor. See
    &lt;interfaces&gt; section Finding 5 for verbatim text. CRITICAL: the
    anchor MUST land at line 12 or later (i.e. after the hashed line 5-11
    range that the docs-alignment row tracks). Inserting before line 12 will
    shift the hashed range and tip STALE_DOCS_DRIFT on the next walker run.

After all five edits land: stage both files and commit ONCE with the exact
owner-prescribed message:

```
docs(dvcs): polish 5 cold-reader nits (binstall crate name + secret-set note + sed readability + slug hedge + template anchor)
```

(No body; one-line subject only — owner specified the exact form.)

Worktree-isolation rule: the executor runs under `isolation="worktree"`. Do
NOT touch any file outside `docs/concepts/` and `docs/guides/`. The verdict
artifact at `quality/reports/verifications/subjective/dvcs-cold-reader.json`
stays AS-IS (owner's explicit instruction; next /reposix-quality-review run
re-grades).
  </action>
  <verify>
    <automated>
# Re-read each file to confirm edits landed verbatim.
grep -n "cargo binstall reposix-cli" docs/concepts/dvcs-topology.md   # line 134 region; expect 1 hit
grep -nv "^#" docs/concepts/dvcs-topology.md | grep -c "cargo binstall reposix$" \
  # expect 0 (the bare `reposix` form must be gone; -v '^#' excludes comment lines per CLAUDE.md grep-gate hygiene)
grep -nE "host slug is .confluence." docs/concepts/dvcs-topology.md  # Finding 4; expect 1 hit
grep -nv "^#" docs/concepts/dvcs-topology.md | grep -c "or your tenant alias" \
  # expect 0 (hedge dropped)
grep -n "Workflow template location" docs/guides/dvcs-mirror-setup.md   # Finding 5; expect 1 hit
grep -n "prompts interactively" docs/guides/dvcs-mirror-setup.md        # Finding 2; expect 1 hit
grep -nE "sed -i \"s\|'\\\\\*/30" docs/guides/dvcs-mirror-setup.md       # Finding 3; expect 1 hit (single-line form)

# Catalog-safety: verifier-script gates still pass.
bash quality/gates/docs-alignment/dvcs-topology-three-roles.sh
bash quality/gates/docs-alignment/dvcs-mirror-setup-walkthrough.sh

# Banned-words-lint must still pass (no jargon leaks introduced).
bash scripts/banned-words-lint.sh

# mkdocs strict build still green (links + anchors resolve).
bash scripts/check-docs-site.sh

# Confirm the atomic commit landed with the exact message.
git log -1 --format='%s' | grep -F "docs(dvcs): polish 5 cold-reader nits (binstall crate name + secret-set note + sed readability + slug hedge + template anchor)"

# Confirm only the two intended files changed in this commit.
git show --name-only HEAD | grep -v '^commit\|^Author\|^Date\|^$\|polish 5 cold-reader' | sort | diff - <(printf 'docs/concepts/dvcs-topology.md\ndocs/guides/dvcs-mirror-setup.md\n')
    </automated>
  </verify>
  <done>
- All 5 edits landed in docs/concepts/dvcs-topology.md + docs/guides/dvcs-mirror-setup.md.
- Single atomic commit recorded with the owner's exact message.
- Both verifier scripts (dvcs-topology-three-roles.sh + dvcs-mirror-setup-walkthrough.sh) PASS.
- Banned-words-lint PASS (no jargon leaks).
- mkdocs strict build PASS (no broken links/anchors).
- No files outside docs/concepts/ + docs/guides/ touched.
- Verdict artifact at quality/reports/verifications/subjective/dvcs-cold-reader.json unchanged.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| (none) | Pure docs edit. No code paths exercised; no input-handling surface; no network egress added; no secrets touched. The `gh secret set --body` example is purely illustrative documentation — no command is executed by this task. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-mgn-01 | I (Information disclosure) | docs/guides/dvcs-mirror-setup.md secret-set note | accept | The added note describes `gh secret set --body "$TOKEN"` as a non-TTY pattern. This is owner-facing setup guidance — owners already know not to commit `$TOKEN` literals. The note does not introduce a new disclosure surface; it documents an existing CLI-command-line behavior. |
| T-mgn-02 | T (Tampering) | docs-alignment row hash drift | mitigate | Pre-audit confirmed bound rows hash lines 5-11 (intro paragraphs only); all five edits land OUTSIDE those ranges. The Finding 5 anchor placement is explicitly constrained to line 12+ to preserve hash stability. Verifier-script body-hash invariants (the `tests` field per row) are also unaffected — no edits touch the `.sh` files in quality/gates/docs-alignment/. |
</threat_model>

<verification>
End-of-task gate (the `<verify>` block in the task above is the contract).
Specifically:

1. Grep gates assert each of the five edits landed verbatim (with grep-gate
   hygiene: `grep -v '^#'` filters comment lines per CLAUDE.md "Self-invalidating
   grep gate" rule).
2. Both docs-alignment verifier scripts pass — the bound rows for these two
   docs are unchanged because their hashed line ranges (lines 5-11) are not
   edited.
3. `scripts/banned-words-lint.sh` confirms no Layer 1/Layer 2 banned terms
   leaked into the new prose.
4. `scripts/check-docs-site.sh` confirms mkdocs strict still builds (anchors
   like `[`...`](dvcs-mirror-setup-template.yml)` already resolve in the
   existing file at lines 35 + 196; the new anchor reuses the same form).
5. `git log -1 --format='%s'` matches the owner-prescribed commit message
   exactly.
6. `git show --name-only HEAD` confirms only the two intended files changed.

Per CLAUDE.md "per-phase push cadence": orchestrator pushes after this task
commits. Pre-push hook runs scripts/end-state.py freshness invariants — none
of the five edits touch any file or line that scripts/end-state.py asserts
against (no version-pinned filenames, no install-path leading commands, no
benchmarks-nav changes, no loose ROADMAP/REQUIREMENTS, no orphan docs).
</verification>

<success_criteria>
- One atomic commit with subject `docs(dvcs): polish 5 cold-reader nits (binstall crate name + secret-set note + sed readability + slug hedge + template anchor)`.
- Files changed: exactly `docs/concepts/dvcs-topology.md` + `docs/guides/dvcs-mirror-setup.md` (no others).
- All five findings from `quality/reports/verifications/subjective/dvcs-cold-reader.json` rationale resolved verbatim.
- All four gate verifications PASS (banned-words, docs-site, two docs-alignment verifier scripts).
- The verdict artifact itself remains unchanged (owner instruction; next /reposix-quality-review run regrades).
- No `cargo run -p reposix-quality -- doc-alignment plan-refresh` invocation needed (pre-audit established no source_hash drift).
</success_criteria>

<output>
After completion, create `.planning/quick/260501-mgn-polish-5-cold-reader-nits-in-dvcs-docs-b/260501-mgn-SUMMARY.md`
documenting:
- Which 5 findings closed (1-line each, citing the verdict-artifact rationale).
- The atomic commit SHA.
- The four gate-verification PASS records.
- Confirmation that source_hash drift on docs-alignment rows did NOT fire (cite the lines 5-11 invariant from the pre-audit).
</output>
