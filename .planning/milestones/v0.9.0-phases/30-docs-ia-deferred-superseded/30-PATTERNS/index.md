# Phase 30: Docs IA and narrative overhaul — Pattern Map

**Mapped:** 2026-04-17
**Files analyzed:** 26 new + modified + deleted targets (docs pages, mkdocs config, Vale config, scripts, CI)
**Analogs found:** 23 / 26 (3 greenfield with no project analog — see § "No Analog Found")

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `docs/index.md` | docs page (narrative hero) | carved-from-existing + rewrite | `docs/index.md` (current) + `docs/why.md` (voice) | role-match (rewrite over existing) |
| `docs/mental-model.md` | docs page (Explanation, short) | new-from-scratch | `docs/why.md` §"The one-sentence thesis" | role-match |
| `docs/vs-mcp-sdks.md` | docs page (Explanation, comparison) | new-from-scratch | `docs/why.md` + `docs/index.md` (current thesis diagram) | role-match |
| `docs/tutorial.md` | docs page (Tutorial) | carved-from-existing | `docs/demo.md` (steps 3–7) | exact |
| `docs/how-it-works/filesystem.md` | docs page (Explanation) | carved-from-existing | `docs/architecture.md` §"Read path" + §"Write path" + §"The async bridge" | exact |
| `docs/how-it-works/git.md` | docs page (Explanation) | carved-from-existing | `docs/architecture.md` §"git push" + §"Optimistic concurrency as git merge" | exact |
| `docs/how-it-works/trust-model.md` | docs page (Explanation) | carved-from-existing | `docs/security.md` + `docs/architecture.md` §"Security perimeter" | exact |
| `docs/how-it-works/index.md` | section landing | new-from-scratch | None direct — see note | partial |
| `docs/guides/write-your-own-connector.md` | docs page (How-to) | move-unchanged | `docs/connectors/guide.md` (465 lines, preserve verbatim) | exact (file move) |
| `docs/guides/integrate-with-your-agent.md` | docs page (How-to) | new-from-scratch | `docs/connectors/guide.md` (prose voice), `docs/why.md` (agent framing) | role-match (greenfield) |
| `docs/guides/troubleshooting.md` | docs page (How-to stub) | new-from-scratch | `docs/demo.md` §"Limitations / honest scope" (failure patterns) | partial |
| `docs/guides/connect-github.md` | docs page (How-to stub) | new-from-scratch | `docs/reference/confluence.md` + `docs/demos/index.md` | partial |
| `docs/guides/connect-jira.md` | docs page (How-to stub) | new-from-scratch | same as above | partial |
| `docs/guides/connect-confluence.md` | docs page (How-to stub) | new-from-scratch | `docs/reference/confluence.md` | partial |
| `docs/reference/simulator.md` | docs page (Reference) | new-from-scratch + carve | `docs/reference/cli.md` (reference voice) + `docs/reference/http-api.md` (sim REST shape) | exact |
| `docs/architecture.md` | existing — **DELETE** after carve | (carved out) | — | — |
| `docs/security.md` | existing — **DELETE** after carve | (carved out) | — | — |
| `docs/demo.md` | existing — **DELETE / REDIRECT** | (carved out) | — | — |
| `docs/connectors/guide.md` | existing — **DELETE** (moved) | (relocated) | — | — |
| `docs/why.md` | existing — keep (Explanation tier, linked from home) | keep | — | — |
| `mkdocs.yml` | config | modify-in-place | `mkdocs.yml` (current, lines 8–103) | exact (baseline) |
| `.vale.ini` | linter config | new-from-scratch | None in repo — research §Example 1 | no-analog (use research template) |
| `.vale-styles/Reposix/ProgressiveDisclosure.yml` | Vale rule | new-from-scratch | None in repo — research §Example 1 | no-analog |
| `.vale-styles/Reposix/NoReplace.yml` | Vale rule | new-from-scratch | None in repo — research §Example 1 | no-analog |
| `scripts/hooks/pre-commit-docs` | git hook | new-from-scratch | `scripts/hooks/pre-push` (mirror pattern HARD-00) | exact |
| `scripts/hooks/test-pre-commit-docs.sh` | shell test | new-from-scratch | `scripts/hooks/test-pre-push.sh` | exact |
| `scripts/check_phase_30_structure.py` | validation script | new-from-scratch | `scripts/check_fixtures.py` | exact |
| `scripts/test_phase_30_tutorial.sh` | shell test | new-from-scratch | `scripts/hooks/test-pre-push.sh` (bash test pattern) | role-match |
| `scripts/install-hooks.sh` | install script | modify-in-place | `scripts/install-hooks.sh` (current — symlink pattern already scales) | exact (no change needed — loop auto-picks up new hook) |
| `.github/workflows/docs.yml` | CI workflow | modify-in-place | `.github/workflows/docs.yml` (current) + `.github/workflows/ci.yml` (hook test pattern) | exact |
| `README.md` (root) | modify-in-place | modify-in-place | existing | (only link audit) |
| `docs/screenshots/phase-30/*.png` | binary assets | new-from-scratch | `docs/screenshots/*.png` (naming pattern) | exact |

## Chapters

- **[ch-docs-pages.md](./ch-docs-pages.md)** — Pattern assignments for docs narrative pages: `docs/index.md` (hero rewrite), `docs/mental-model.md`, `docs/vs-mcp-sdks.md`, `docs/tutorial.md`, and all four `docs/how-it-works/` pages.
- **[ch-guides-reference.md](./ch-guides-reference.md)** — Pattern assignments for guides pages (`write-your-own-connector`, `integrate-with-your-agent`, `troubleshooting`, `connect-{github,jira,confluence}`) and `docs/reference/simulator.md`.
- **[ch-config-tooling.md](./ch-config-tooling.md)** — Pattern assignments for config and tooling: `mkdocs.yml`, `.vale.ini`, Vale style rules, `scripts/hooks/pre-commit-docs`, hook test, Python validation script, tutorial shell test, GitHub Actions `docs.yml`, screenshots, and `README.md` link audit.
- **[ch-shared-metadata.md](./ch-shared-metadata.md)** — Shared patterns (mermaid, frontmatter, internal links, admonitions, footnotes, hooks, Python scripts, GH Actions), No Analog Found table, and Metadata.
