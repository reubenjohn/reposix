---
name: reposix-banned-words
description: Self-check Markdown changes under docs/ against the reposix progressive-disclosure layer rules before committing. Use whenever you author or edit a file in docs/index.md, docs/concepts/, docs/tutorials/, docs/guides/, or docs/how-it-works/.
---

# reposix-banned-words

This skill encodes the P1 + P2 framing rules from
`.planning/notes/phase-30-narrative-vignettes.md` so an authoring agent can
self-check before commit. The mechanical enforcer is
`scripts/banned-words-lint.sh`; the canonical config is
`docs/.banned-words.toml`.

## When to invoke

Before staging any change under `docs/`, run:

```bash
scripts/banned-words-lint.sh           # default — Layer 1 + Layer 2
scripts/banned-words-lint.sh --all     # QA — also Layer 3
```

Pre-commit and CI both run the default mode automatically. Running it locally
catches violations during authoring instead of after `git commit`.

## The layer rules (TL;DR)

| Layer | Pages | Banned vocabulary |
|-------|-------|-------------------|
| 1 — Hero | `docs/index.md` | the v0.1 FUSE-era plumbing words (`FUSE`, `fusermount`, `inode`, `daemon`, `kernel`, `syscall`) **and** the v0.9.0 git-native plumbing words (`partial-clone`, `promisor`, `stateless-connect`, `fast-import`, `protocol-v2`) |
| 2 — Below the fold | `docs/concepts/`, `docs/tutorials/`, `docs/guides/` | same as Layer 1 |
| 3 — How it works | `docs/how-it-works/` | nothing layer-specific (P1 still applies) — plumbing terms are permitted; this is where the technical reveal happens |
| 4 — Reference + Research | `docs/reference/`, `docs/research/`, `docs/decisions/` | nothing |

P1 (banned everywhere): `replace` — reposix complements REST; it does not replace it.

## Allowlist

If a banned word is genuinely required (a quote, an error message, a
historical note), append the marker `<!-- banned-words: ok -->` to the same
line. Pair the marker with a brief comment explaining why; reviewers will
push back on bare allowlists.

Example:

```markdown
The deleted v0.1 design used a FUSE daemon. <!-- banned-words: ok --> historical
```

## Adding a new banned word

1. Edit `docs/.banned-words.toml`.
2. Mirror the change into the hardcoded arrays in `scripts/banned-words-lint.sh`
   (the script flags this duplication on purpose — drift between the two is a
   review surface).
3. Run the linter against the current docs tree; fix any new violations or
   allowlist them with rationale.
4. Commit `docs/.banned-words.toml`, the script, and the doc fixes together.

## Why this exists

> Above the fold, the only technical words permitted are ones every developer
> already knows — file, folder, edit, commit, push, merge, YAML, markdown.
> — `.planning/notes/phase-30-narrative-vignettes.md` §"P2 progressive disclosure"

The linter is the institutional memory of that rule, not a checklist (per
project CLAUDE.md OP-4: "ad-hoc bash is a missing-tool signal — promote it
to a committed artifact").
