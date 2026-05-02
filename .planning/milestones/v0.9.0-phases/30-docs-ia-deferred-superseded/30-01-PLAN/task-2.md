# Task 2: Create Vale config + P1/P2 rule files + vocabulary

← [back to index](./index.md)

<task type="auto">
  <name>Task 2: Create Vale config + P1/P2 rule files + vocabulary</name>
  <files>.vale.ini, .vale-styles/Reposix/ProgressiveDisclosure.yml, .vale-styles/Reposix/NoReplace.yml, .vale-styles/config/vocabularies/Reposix/accept.txt</files>
  <read_first>
    - `.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-RESEARCH.md` §Example 1 (lines 431-497 — verbatim template for `.vale.ini` + both rule files)
    - `.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-PATTERNS.md` §".vale.ini", §"ProgressiveDisclosure.yml", §"NoReplace.yml" (pattern: use RESEARCH.md templates verbatim — no analog in repo)
    - `.planning/notes/phase-30-narrative-vignettes.md` §"Framing principles" (lines 27-86 — confirms P1 banned word "replace" and P2 banned terms FUSE/inode/daemon/helper/kernel/mount/syscall)
  </read_first>
  <action>
    Create `.vale.ini` at repo root with the following EXACT content (no deviation — the per-glob scoping is load-bearing):

```ini
# .vale.ini — Phase 30 progressive-disclosure + P1 banned-word enforcement.
# See .planning/notes/phase-30-narrative-vignettes.md for the framing principles.

StylesPath = .vale-styles
MinAlertLevel = warning

Vocab = Reposix

# Exclude code from prose linting — bash snippets referencing FUSE, mount commands,
# etc. must never false-positive. Pitfall 1 in RESEARCH.md.
IgnoredScopes = code, code_block

# Default: apply ProgressiveDisclosure rule (P2) on all markdown.
[*.md]
BasedOnStyles = Reposix

# Escape hatch: Layer 3+ pages MAY mention FUSE/inode/daemon/kernel/mount/syscall.
# All pages under these directories are exempt from the P2 rule.
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

[docs/archive/**]
Reposix.ProgressiveDisclosure = NO

# Per-file exception: mental-model.md uses the locked H2 "mount = git working tree"
# per source-of-truth note. This is THE ONLY Layer-1/2 page with a P2 exception.
[docs/mental-model.md]
Reposix.ProgressiveDisclosure = NO

# P1 rule: "replace" banned everywhere in hero copy on the landing page.
[docs/index.md]
Reposix.NoReplace = YES
```

Create `.vale-styles/Reposix/ProgressiveDisclosure.yml` with EXACT content:

```yaml
extends: existence
message: "P2 violation: '%s' is a Layer 3 term and is banned on Layer 1/2 pages (index, mental-model [see per-file exception], vs-mcp-sdks, tutorial, guides, home-adjacent). Move it into docs/how-it-works/ or rephrase in user-experience language."
link: https://github.com/reubenjohn/reposix/blob/main/.planning/notes/phase-30-narrative-vignettes.md
level: error
scope: text
ignorecase: true
tokens:
  - FUSE
  - inode
  - daemon
  - \bhelper\b
  - kernel
  - \bmount\b
  - syscall
```

Create `.vale-styles/Reposix/NoReplace.yml` with EXACT content:

```yaml
extends: existence
message: "P1 violation: '%s' is banned in hero / value-prop copy (docs/index.md). Use 'complement, absorb, subsume, lift, erase the ceremony' instead. See .planning/notes/phase-30-narrative-vignettes.md §P1."
link: https://github.com/reubenjohn/reposix/blob/main/.planning/notes/phase-30-narrative-vignettes.md
level: error
scope: text
ignorecase: true
tokens:
  - replace
  - replaces
  - replacing
  - replacement
```

Create `.vale-styles/config/vocabularies/Reposix/accept.txt` with the following content (one term per line — these are project-specific words Vale should NOT flag as unknown — NOT banned terms):

```
reposix
mkdocs
fusermount
fusermount3
FUSE
jq
curl
YAML
markdown
Jira
Confluence
JIRA
BackendConnector
frontmatter
mcp
MCP
sed
grep
awk
plaintext
mermaid
```

Note: "FUSE" appears in accept.txt so Vale does not flag it as an unknown word in files where it IS allowed (Layer 3+). The ProgressiveDisclosure rule separately enforces the ban — accept.txt is for spell-check style lookups only.

Now run Vale against the current (not-yet-rewritten) `docs/` tree as a calibration pass. Expected behavior: Vale will find existing P1/P2 violations on `docs/index.md` (uses "FUSE", and uses "replaces" in the dark-factory section) — these are EXPECTED because those files will be rewritten in Wave 1. To confirm the rules trigger, run:

```bash
vale --config=.vale.ini docs/index.md; echo "exit=$?"
```

This should exit 1 with error messages mentioning "FUSE" (ProgressiveDisclosure) and/or "replaces" (NoReplace). Document this calibration result in the commit message.

To confirm `docs/how-it-works/` exemption (no pages exist yet, but the rule scoping must parse correctly), also run:

```bash
vale --config=.vale.ini docs/reference/cli.md; echo "exit=$?"
```

This should exit 0 (reference/ is Layer 3+, exempt from ProgressiveDisclosure).
  </action>
  <verify>
    <automated>test -f .vale.ini && test -f .vale-styles/Reposix/ProgressiveDisclosure.yml && test -f .vale-styles/Reposix/NoReplace.yml && test -f .vale-styles/config/vocabularies/Reposix/accept.txt && grep -q "IgnoredScopes = code, code_block" .vale.ini && grep -q "Reposix.ProgressiveDisclosure = NO" .vale.ini && grep -q "^  - FUSE$" .vale-styles/Reposix/ProgressiveDisclosure.yml && grep -q "^  - replace$" .vale-styles/Reposix/NoReplace.yml && ~/.local/bin/vale --config=.vale.ini docs/reference/cli.md; test $? -eq 0</automated>
  </verify>
  <acceptance_criteria>
    - `.vale.ini` exists with `IgnoredScopes = code, code_block` line present (Pitfall 1 mitigation).
    - `.vale.ini` has exactly 6 Layer-3+ directory exemptions: `how-it-works`, `reference`, `decisions`, `research`, `development`, `archive`.
    - `.vale.ini` has exactly one per-file exemption: `docs/mental-model.md`.
    - `.vale-styles/Reposix/ProgressiveDisclosure.yml` contains all 7 banned tokens: FUSE, inode, daemon, `\bhelper\b`, kernel, `\bmount\b`, syscall.
    - `.vale-styles/Reposix/NoReplace.yml` contains all 4 tokens: replace, replaces, replacing, replacement.
    - `~/.local/bin/vale --config=.vale.ini docs/reference/cli.md` exits 0 (reference/ is exempt).
    - `~/.local/bin/vale --config=.vale.ini docs/index.md` exits non-zero (current index.md contains "FUSE" — expected to fail until Wave 1 rewrites it).
    - `.vale-styles/config/vocabularies/Reposix/accept.txt` contains `reposix` on its own line.
  </acceptance_criteria>
  <done>
    Vale config + rules committed. `docs/index.md` fails Vale (as expected — flagged for rewrite in Wave 1). `docs/reference/*.md` pass Vale (correctly exempt). Calibration noted in commit message.
  </done>
</task>
