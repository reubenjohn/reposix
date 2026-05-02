# Task 1 — Apply 5 cold-reader polish edits (atomic commit)

← [back to index](./index.md)

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
