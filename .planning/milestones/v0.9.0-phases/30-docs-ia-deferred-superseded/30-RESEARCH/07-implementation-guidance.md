← [back to index](./index.md) · phase 30 research

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Banned-word scanning with code-fence awareness | Custom regex scanner that tries to strip code blocks before matching | Vale with `IgnoredScopes = code, code_block` in `.vale.ini` | Fence parsing has edge cases (nested, indented, `~~~` vs ```` ``` ````, language tags). Vale handles all of this `[VERIFIED: vale.sh/docs/styles]`. |
| Diagram rendering | Manually drawing SVG in Inkscape and embedding | Mermaid fenced code blocks | Already wired; palette-aware; source-of-truth stays in markdown; `mmdc` CLI exists if offline render is ever needed. |
| Social card generation | Writing OpenGraph/Twitter cards as static image per page | `mkdocs-material[imaging]` social plugin | Auto-generates per page with page title + description; CairoSVG + pillow already installed. |
| 10-second value-prop validation | Writing a custom subagent prompt harness | Existing `doc-clarity-review` skill with a custom `--prompt` | The skill already copies files to `/tmp`, runs Claude subprocess with zero repo context, writes `_feedback.md`. It's designed for exactly this. |
| Cold-reader friction-point discovery | Manual review by author | Same `doc-clarity-review` skill default prompt | Author cannot un-know what they wrote. An isolated subprocess is the only way to model a first-time reader. |
| Screenshot verification | Custom puppeteer scripts | Playwright MCP (already cached at `~/.cache/ms-playwright/`) | MCP invocation is one tool call; no scripting boilerplate. |

**Key insight:** The validation mechanisms this phase needs are ALREADY in our tooling stack. We don't need to invent a "10-second landing page validator" — we point `doc-clarity-review` at the rendered `docs/index.md` with a purpose-built prompt ("state reposix's value proposition in one sentence after reading this page") and check the verdict. The skill's existing `_feedback.md` artifact serves as the phase-gate evidence.

## Runtime State Inventory

This is a docs phase, not a rename / refactor / migration. No runtime state (databases, service configs, OS registrations, secrets) is being renamed or migrated. The phase may move markdown files under `docs/`, which is:

- **Stored data:** None relevant. The only "stored state" touched is markdown files (code edits, not data migrations).
- **Live service config:** None. The `docs.yml` GitHub Actions workflow is parameterized by file paths (`docs/**` + `mkdocs.yml`), which remain stable post-phase. GitHub Pages deploys from `gh-pages` branch regardless of what's in `main/docs/`.
- **OS-registered state:** None.
- **Secrets/env vars:** None. The docs pipeline is unauth to its registry (gh-pages) and uses the default `GITHUB_TOKEN`.
- **Build artifacts:** `site/` (gitignored) is regenerated on every `mkdocs build`. No stale artifacts persist.

**One indirect concern:** old URLs. If we delete `docs/architecture.md` and `docs/security.md` and the old links exist in:
- `README.md` (root) — check and update.
- `CHANGELOG.md` — historical records should NOT be rewritten per CLAUDE.md precedent from Phase 25 ("Historical planning records ... retain old filenames").
- External-to-repo links (social posts, issue comments, blog mentions) — cannot be updated; rely on GitHub's soft 404 on gh-pages.

The planner should include a Wave that does a grep-audit of internal markdown references after the file moves land.

## Common Pitfalls

### Pitfall 1: Vale flagging legitimate technical terms in code blocks
**What goes wrong:** A bash snippet `cat FUSE.md` or a Rust snippet `let mount = ...` gets flagged by the "no FUSE above Layer 3" rule.
**Why it happens:** Fenced code blocks and inline code need to be excluded from prose-linting scope.
**How to avoid:** Set `IgnoredScopes = code, code_block` in `.vale.ini` `[*]` section. Vale's markdown parser understands both fenced (```` ``` ````) and indented code blocks. `[VERIFIED: vale.sh]`.
**Warning signs:** Linter fails on an obviously-legitimate code block; the reviewer has to "just ignore" lint output (training erosion).

### Pitfall 2: `navigation.instant` breaking custom landing page
**What goes wrong:** If we go the Jinja override path for `index.md` (`overrides/home.html`), mkdocs-material's `navigation.instant` feature (enabled in current `mkdocs.yml`) conflicts with the custom template — the client-side router doesn't know about the custom home template.
**Why it happens:** `navigation.instant` SPA-routes between pages using JavaScript, but the home page is a different layout entirely.
**How to avoid:** Either (a) don't use `overrides/home.html` (recommended, use markdown-native), or (b) disable `navigation.instant` if we do go Jinja. `[CITED: alex3305.github.io/docs/selfhosted/mkdocs_material/custom_landing_page]`.
**Warning signs:** Home page looks right on hard refresh but broken when navigated to from another page via JS router.

### Pitfall 3: Mermaid diagrams rendering unstyled in dark mode
**What goes wrong:** Certain diagram types (pie, gantt, user journey, git graph, requirement) don't get auto-theming from mkdocs-material and look terrible in dark mode.
**Why it happens:** mkdocs-material "will currently only adjust the fonts and colors for flowcharts, sequence diagrams, class diagrams, state diagrams and entity relationship diagrams" `[VERIFIED: mkdocs-material reference/diagrams]`.
**How to avoid:** Stick to flowcharts, sequence, class, state, ER. For Phase 30: filesystem layer = flowchart; git layer = sequence or flowchart; trust model = flowchart with subgraphs (same pattern as existing `architecture.md`).
**Warning signs:** A diagram looks fine in light mode but unreadable in dark mode during playwright screenshot verification.

### Pitfall 4: mkdocs `--strict` rejecting orphan pages
**What goes wrong:** New pages created but not linked in `nav:` or any other page fail `mkdocs build --strict`.
**Why it happens:** `--strict` treats unreferenced pages as warnings by default in recent versions; some configs escalate to errors.
**How to avoid:** Every new page must be in `nav:` when committed. Alternatively, use `not_in_nav: |` for intentionally unlisted pages (like the archive/ pattern in the current config).
**Warning signs:** Local `mkdocs serve` works fine but CI `mkdocs build --strict` fails.

### Pitfall 5: Banned-word linter false positives on "mount" as generic verb
**What goes wrong:** "mount" is on the P2 ban list — but it's a generic English verb ("You'll mount an argument against this approach"). If we use it generically, the linter false-positives.
**Why it happens:** Context-free word matching can't distinguish technical from conversational.
**How to avoid:** (a) Don't use "mount" generically anywhere above Layer 3 — rephrase. (b) If a false-positive does slip in, use Vale's `[vale-comment] mount [/vale-comment]` inline ignore or structure the prose to avoid ambiguity. **Recommend (a).** This is a discipline check, not a technical problem.
**Warning signs:** Copy review reveals "mount" used in a non-technical sense; linter doesn't know what you meant.

### Pitfall 6: Social cards plugin failing in CI due to missing system fonts
**What goes wrong:** `material[imaging]` requires fonts (often DejaVu, Roboto, or a configurable custom font). GitHub Actions `ubuntu-latest` has these, but if we customize fonts in the mkdocs config and they aren't in the CI image, cards fail silently or use fallbacks.
**Why it happens:** CairoSVG + pillow need font files at runtime.
**How to avoid:** Use default fonts (Roboto ships with material); if customizing, add `sudo apt-get install -y fonts-<your-font>` to the docs workflow. `[CITED: mkdocs-material plugins/requirements/image-processing]`.
**Warning signs:** Social cards look wrong in prod but fine locally.

### Pitfall 7: Deleting `docs/architecture.md` before verifying no cross-links remain
**What goes wrong:** `docs/security.md`, `docs/index.md`, `docs/why.md`, `docs/connectors/guide.md`, `docs/demos/index.md`, and several ADRs all link to `docs/architecture.md#some-anchor`. Deleting it breaks `mkdocs build --strict`.
**Why it happens:** The strict build fails on dangling internal links.
**How to avoid:** Before deletion, grep `docs/` for `architecture.md` references, then decide per-reference: (a) update to new `how-it-works/*.md` anchor, (b) update to `why.md` anchor if content moved there, or (c) delete the reference. Same approach for `docs/security.md`.
**Warning signs:** `mkdocs build --strict` explodes in the last wave.

## Code Examples

### Example 1: `.vale.ini` for progressive-disclosure layer rules

Create at repo root:

```ini
# .vale.ini
StylesPath = .vale-styles
MinAlertLevel = warning

Vocab = Reposix

# Exclude code from prose linting (critical — avoids flagging bash snippets)
IgnoredScopes = code, code_block

# By default, apply the banned-word rule to all markdown
[*.md]
BasedOnStyles = Reposix

# ESCAPE HATCH: how-it-works/, reference/, decisions/, research/ are Layer 3+
# and MAY use the banned terms. Opt-out per-glob.
[docs/how-it-works/**]
Reposix.ProgressiveDisclosure = NO
[docs/reference/**]
Reposix.ProgressiveDisclosure = NO
[docs/decisions/**]
Reposix.ProgressiveDisclosure = NO
[docs/research/**]
Reposix.ProgressiveDisclosure = NO
[docs/development/**]
Reposix.ProgressiveDisclosure = NO

# The hero-ban on "replace" is a different rule, applied EVERYWHERE on the landing
[docs/index.md]
BasedOnStyles = Reposix
Reposix.NoReplace = YES
```

And the rule file `.vale-styles/Reposix/ProgressiveDisclosure.yml`:

```yaml
extends: existence
message: "P2 violation: '%s' is a Layer 3 term — banned on Layer 1/2 pages (index, mental-model, vs-mcp-sdks, tutorial, guides, home-adjacent). Move it into docs/how-it-works/ or rephrase in user-experience language."
level: error
scope: text
ignorecase: true
tokens:
  - FUSE
  - inode
  - daemon
  - \bhelper\b       # boundaries — "Jupyter helper" generic is fine, but flag bare
  - kernel
  - \bmount\b        # noun/verb — flag any bare occurrence; authors rephrase
  - syscall
```

And `.vale-styles/Reposix/NoReplace.yml`:

```yaml
extends: existence
message: "P1 violation: 'replace' is banned in hero/value-prop copy. Use 'complement, absorb, subsume, lift, erase the ceremony' instead."
level: error
scope: text
ignorecase: true
tokens:
  - replace
  - replaces
  - replacing
  - replacement
```

**Source:** Constructed from `[VERIFIED: vale.sh/docs/styles]` + source-of-truth note §P1/P2.

### Example 2: Pre-commit hook (follows existing `scripts/hooks/` pattern)

`scripts/hooks/pre-commit-docs`:

```bash
#!/usr/bin/env bash
# Doc-lint gate — runs Vale on docs/**.md.
# Mirrors scripts/hooks/pre-push pattern (HARD-00).

set -euo pipefail

CHANGED=$(git diff --cached --name-only --diff-filter=ACM | grep -E '^docs/.*\.md$' || true)

if [ -z "$CHANGED" ]; then
    exit 0
fi

if ! command -v vale >/dev/null 2>&1; then
    echo "error: vale not installed. See .planning/phases/30-.../RESEARCH.md §Standard Stack." >&2
    exit 1
fi

echo "==> Vale lint on $CHANGED"
echo "$CHANGED" | xargs vale --config=.vale.ini
```

Install via `scripts/install-hooks.sh` — matches existing pattern for credential pre-push hook (Phase 21 HARD-00).

### Example 3: GitHub Actions step for Vale in docs.yml

Addition to `.github/workflows/docs.yml` before the `mkdocs build --strict` step:

```yaml
      - name: Install Vale
        run: |
          VALE_VERSION=3.10.0  # pin; bump deliberately
          curl -L "https://github.com/errata-ai/vale/releases/download/v${VALE_VERSION}/vale_${VALE_VERSION}_Linux_64-bit.tar.gz" \
              | tar xz -C /usr/local/bin vale
          vale --version

      - name: Lint docs with Vale (banned-word + progressive-disclosure rules)
        run: vale --config=.vale.ini docs/

      - name: Build strict (fail on broken links)
        run: mkdocs build --strict
```

### Example 4: doc-clarity-review invocation for 10-second value-prop validation

The existing skill can be invoked with a custom prompt. Phase 30's "did the value prop land" validation:

```bash
# From phase-gate verification step (post-build, pre-merge)
VERDICT_DIR=$(mktemp -d /tmp/phase-30-value-prop-XXXXXX)

# Render the built home page to plain text (or just use source markdown)
cp docs/index.md "$VERDICT_DIR/"

# Purpose-built prompt (vs default clarity prompt)
cat > "$VERDICT_DIR/_prompt.md" <<'EOF'
You are reading this page for the first time, completely cold. You have NOT
seen the repo, any other documentation, or this project's GitHub.

Your job: after reading the page, tell me in EXACTLY ONE SENTENCE what
reposix is and what problem it solves. Then rate:

- LANDED — you got it from the content alone
- PARTIAL — you got a rough idea but are missing the pivotal point
- MISSED — you'd need to read more to answer

Then: identify the single sentence or image on this page that did the most work.
If you can't identify one, that's itself a verdict.
EOF

claude -p "$(cat $VERDICT_DIR/_prompt.md)" "$VERDICT_DIR/index.md" 2>&1 | tee "$VERDICT_DIR/_feedback.md"
# Parse verdict: grep -i 'LANDED' && declare pass, else fail
```

**Source:** `[VERIFIED: ~/.claude/skills/doc-clarity-review/SKILL.md]` — skill already exists, exactly fits the need.

### Example 5: Playwright screenshot verification (invoked via MCP)

Not literal shell — invoked from the planner / executor via Playwright MCP tools. The pattern:

1. Start `mkdocs serve` in background.
2. Playwright MCP → `browser_navigate` → `http://127.0.0.1:8000/` → `browser_take_screenshot` with viewport 1280×800 and 375×667.
3. Save to `docs/screenshots/phase-30/home-desktop.png` and `home-mobile.png`.
4. Repeat for `/how-it-works/filesystem/`, `/how-it-works/git/`, `/how-it-works/trust-model/`, `/tutorial/`.
5. Visual review for: spaghetti edges, overlapping labels, unreadable node text, contrast, layout, mobile width. Per user global CLAUDE.md OP #1.
6. Commit screenshots; reference from `30-SUMMARY.md`.

### Example 6: Mermaid diagram specs for the three how-it-works pages

**how-it-works/filesystem.md diagram — adapt from existing architecture.md sequence diagram:**

````mermaid
sequenceDiagram
  autonumber
  actor A as You (shell or agent)
  participant K as POSIX
  participant F as reposix
  participant R as Your tracker
  A->>K: cat issues/PROJ-42.md
  K->>F: read("PROJ-42.md")
  F->>R: GET /issue/PROJ-42
  R-->>F: issue JSON
  F-->>K: frontmatter + markdown bytes
  K-->>A: text
````

Keep "FUSE" / "kernel" / "daemon" out of the actor names — use "POSIX" and "reposix" (Layer 3 is where these terms may appear in prose, but diagram LABELS should stay readable to Layer 2 readers who land on the page from the index).

**Source:** derived from current `docs/architecture.md` "Read path" mermaid, simplified for Layer 3 entry-point.

**how-it-works/git.md diagram — the git push round-trip (adapt from architecture.md):**

````mermaid
flowchart LR
  A["Your commit<br/>status: Done"] -->|git push| B["reposix<br/>(diff + dispatch)"]
  B -->|PATCH + If-Match| C["Your tracker"]
  C -->|200 or 409| B
  B -->|success or conflict markers| A
````

**how-it-works/trust-model.md diagram — adapt from architecture.md "Security perimeter":**

````mermaid
flowchart LR
  subgraph Tainted["Tainted (attacker-influenced)"]
    BODY["Issue bodies"]
    TITLE["Titles / comments"]
  end
  subgraph Trusted["Trusted (yours)"]
    CORE["reposix core<br/>sealed HttpClient"]
    AUDIT["append-only<br/>SQLite audit log"]
  end
  subgraph Egress["Egress (allowlisted only)"]
    NET["HTTP → allowlisted origin"]
    DISK["Filesystem writes<br/>(mount point only)"]
  end
  BODY -.->|typed as Tainted&lt;T&gt;| CORE
  TITLE -.->|typed as Tainted&lt;T&gt;| CORE
  CORE -->|sanitize + allowlist| NET
  CORE --> AUDIT
  style Tainted fill:#d32f2f,stroke:#fff,color:#fff
  style Egress fill:#ef6c00,stroke:#fff,color:#fff
  style Trusted fill:#00897b,stroke:#fff,color:#fff
````

**Source:** adapted from current `docs/architecture.md` "Security perimeter" diagram.
