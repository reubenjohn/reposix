# Phase 43 Context — Nav restructure (Diátaxis) + theme tuning + banned-words linter

**Milestone:** v0.10.0 "Docs & Narrative Shine"
**Requirements:** DOCS-07, DOCS-08 (linter wiring half), DOCS-09
**Depends on:** Phases 40, 41, 42 (all new pages must exist)

## Goal

1. **Nav restructure (DOCS-07).** `mkdocs.yml` flat → Diátaxis taxonomy: Home / Concepts / Tutorials / How it works / Guides / Reference / Benchmarks / Decisions / Research. Eliminate stale top-level pages (`architecture.md`, `security.md`, `connectors/guide.md`, `why.md`, `demo.md`, `demos/index.md`) — either redirect-stub or delete. `not_in_nav` cleaned of zombies; only `archive/*` excluded.

2. **Theme tuning (DOCS-08 partial).** mkdocs-material palette tasteful (primary indigo or teal — current `deep purple` is the stock-photo default), light + dark toggle preserved, navigation features curated (not blanket-on), `content.code.copy` + `search.suggest` on, repo icon set, social plugin lazy-deferred (requires `mkdocs-material[imaging]` + cairo system libs which are env-fragile; document as Phase 45 follow-up).

3. **Banned-words linter (DOCS-09).** `scripts/banned-words-lint.sh` reads layer rules from `docs/.banned-words.toml` (checked-in config, reviewable diff per OP-4). Default mode scans Layers 1–2 (`docs/index.md`, `docs/concepts/`, `docs/tutorials/`, `docs/guides/`); `--all` scans the whole tree (Layer 3 will legitimately mention `FUSE` historically — that's why default mode skips it). Lines with `<!-- banned-words: ok -->` allowlisted. Pre-commit + CI integration so the rule is mechanical, not a checklist.

## Layer banned-word matrix

(per `.planning/notes/phase-30-narrative-vignettes.md` §"P2 progressive disclosure" + REQUIREMENTS DOCS-07 revised list)

| Layer | Pages | P1 banned (everywhere) | P2 banned (this layer + above) |
|-------|-------|------------------------|-------------------------------|
| 1 (Hero) | `docs/index.md` | `replace` | `FUSE`, `fusermount`, `kernel`, `syscall`, `daemon`, `inode`, `mount`, `helper`, `partial-clone`, `promisor`, `stateless-connect`, `fast-import`, `protocol-v2` |
| 2 (Just below fold) | `docs/concepts/`, `docs/tutorials/`, `docs/guides/` | `replace` | same as Layer 1 |
| 3 (How it works) | `docs/how-it-works/` | `replace` | `partial-clone`, `promisor`, `stateless-connect`, `fast-import`, `protocol-v2` allowed; `FUSE`+`fusermount`+`kernel`+`syscall`+`daemon`+`inode` allowed (historical references during transition) |
| 4 (Reference + research) | `docs/reference/`, `docs/research/`, `docs/decisions/` | none | none |

## Files to redirect or delete

| Path | Disposition | Successor |
|------|-------------|-----------|
| `docs/architecture.md` | redirect stub | `how-it-works/git-layer.md` |
| `docs/security.md` | redirect stub | `how-it-works/trust-model.md` |
| `docs/connectors/guide.md` | redirect stub | `guides/write-your-own-connector.md` |
| `docs/why.md` | redirect stub | `index.md` |
| `docs/demo.md` | redirect stub | `tutorials/first-run.md` |
| `docs/demos/index.md` | keep (Tier-2 walkthrough catalog), move under Reference or delete from nav | TBD — keep but move under nav `Reference > Demos` |
| `docs/development/*` | move to nav `Reference > Development` or keep as-is | keep |

**Cross-link rewires already needed:**
- `docs/how-it-works/trust-model.md` line 87 → currently links `../security.md`; rewire to `../research/threat-model-and-critique.md` (if exists) or remove.
- `docs/reference/http-api.md` line 80 → links `../security.md#whats-still-deferred-v04`; rewire to `../how-it-works/trust-model.md` or drop the anchor.
- `docs/reference/confluence.md` line 189 → links `../connectors/guide.md`; rewire to `../guides/write-your-own-connector.md`.
- `docs/decisions/002-confluence-page-mapping.md` line 187 → same fix.
- `docs/demos/index.md` line 52 → links `../demo.md`; rewire to `../tutorials/first-run.md` or drop (low-value link).

## Plan slice

- **Plan A — nav restructure + theme + redirects.** Update `mkdocs.yml`, write 5 redirect stubs, rewire 4 cross-links. One commit per logical group: `mkdocs.yml`, redirects+rewires.
- **Plan B — banned-words linter + config + hooks + skill.** Write `docs/.banned-words.toml`, `scripts/banned-words-lint.sh`, `.pre-commit-config.yaml`, `.github/workflows/docs.yml` lint job, `.claude/skills/reposix-banned-words/SKILL.md`. Verify against new docs (default mode → exit 0; seeded violation → exit 1).

## Acceptance

1. `mkdocs.yml` nav matches Diátaxis order; `not_in_nav` only contains `archive/*`.
2. `mkdocs build --strict` green (deferrable to Phase 45 if memory tight; document deferral).
3. `scripts/banned-words-lint.sh` exists, executable, exits 0 on default-mode scan of post-Phase-42 docs, exits 1 on seeded `# FUSE` line in `docs/index.md`.
4. Pre-commit + CI both invoke the linter; existing pre-commit chain (if any) untouched.
5. `.claude/skills/reposix-banned-words/SKILL.md` exists with `name:` + `description:` frontmatter and points to `docs/.banned-words.toml`.
6. No nav entry references a deleted file; redirect stubs are 1-line `# → <new path>` markers.

## Out of scope

- README hero rewrite (Phase 45).
- Social card image generation (Phase 45 — needs `mkdocs-material[imaging]` + cairo).
- doc-clarity-review run (Phase 44).
- Layer 3 cleanup of historical FUSE/inode mentions (Phase 41 already enforced this on the trio; default-mode linter skips Layer 3 by design).
