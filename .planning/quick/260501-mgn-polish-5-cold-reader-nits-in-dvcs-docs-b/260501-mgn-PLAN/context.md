# Context — catalog safety pre-audit, interfaces, verification steps

← [back to index](./index.md)

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
