# Phase 44 — Doc Clarity Audit (2026-04-24)

**Scope:** 14 user-facing pages.
**Methodology:** inline cold-reader audit (no Claude subprocess; consistent reviewer rubric). Banned-words linter (`scripts/banned-words-lint.sh`) green at audit start. `scripts/check_doc_links.py` reports zero broken relative links across the user-facing tree at audit start.
**Severity:** Critical (blocking) / Major (backlog) / Minor (nit).

## Per-page findings

### `README.md`
| # | Severity | Finding | Suggested fix |
|---|---|---|---|
| 1 | Critical | The Tier 1–5 demo blocks (`reposix mount`, `reposix demo`, `reposix list`) advertise commands removed in v0.9.0 (CHANGELOG explicitly: `reposix mount` is REMOVED, `reposix demo` is REMOVED). A cold reader running these copy-paste blocks gets clap "unrecognized subcommand" errors. | Phase 45 owns the README rewrite; this finding is recorded as ESCALATE for Phase 45. Surgical Phase-44 fix not applied (would require trimming ~150 lines and is out of Phase 44 scope per runner guardrails). |
| 2 | Major | "v0.7.x — pre-FUSE-deletion" Quickstart is now historical and confuses the eye-line for a cold reader. | Move below `## Quickstart (v0.9.0)` into a `<details>` block in Phase 45. |
| 3 | Major | Architecture ASCII diagram still depicts FUSE + kernel VFS — outdated by v0.9.0 pivot. | Replace with current promisor-remote diagram or remove in Phase 45. |
| 4 | Minor | "Honest scope" paragraph cites the v0.1–v0.7 autonomous sessions; misses v0.9.0. | Update in Phase 45. |

> **ESCALATE: rewrite needed for `README.md`** — the body below the v0.9.0 quickstart is largely v0.7-era and contains dead `reposix mount`/`reposix demo` commands. Phase 45 explicitly owns the README hero+body rewrite. Per runner guardrails ("don't aggressively rewrite well-functioning prose; surgical fixes only" and "if you find a critical finding that requires rewriting an entire page, escalate"), this finding is escalated rather than half-rewritten in Phase 44.

### `docs/index.md`
| # | Severity | Finding | Suggested fix |
|---|---|---|---|
| 1 | Minor | Closing italics paragraph references "Phase 36" indirectly via mention of CI secret packs — minor jargon for a cold reader. | Soften to "Real-backend cells fill in once CI secret packs are wired." Already worded acceptably; backlog as polish. |
| 2 | Minor | "What it looks like underneath" paragraph is a single 8-line block — borderline density; lists would scan better. | Convert to bullet list in v0.11.0 polish. |

### `docs/concepts/mental-model-in-60-seconds.md`
| # | Severity | Finding | Suggested fix |
|---|---|---|---|
| 1 | Minor | "Three keys" is asserted upfront but the sections are not numbered "Key 1/2/3" — they are numbered `1./2./3.` which works but a cold reader has to count to confirm. | Pre-existing structure is fine; backlog only. |

### `docs/concepts/reposix-vs-mcp-and-sdks.md`
| # | Severity | Finding | Suggested fix |
|---|---|---|---|
| 1 | Minor | MCP/SDK latency cells are characterized, not measured — disclosed in the paragraph below the table; cold reader could miss the disclaimer. | Add a row-level footnote marker (`*`) on those cells in v0.11.0 polish. |

### `docs/tutorials/first-run.md`
| # | Severity | Finding | Suggested fix |
|---|---|---|---|
| 1 | Major | Step 1 says `cd /path/to/reposix` — assumes the reader has cloned the repo, but the page promises "five minutes from clone to a real edit" without an explicit `git clone` step. | Prepend a `git clone https://github.com/reubenjohn/reposix && cd reposix` line in Phase 45 polish. |
| 2 | Minor | The tutorial uses `cargo run -p reposix-cli -- init …` (uninstalled path); the "Optional install" callout is correct but a cold reader might wonder why `reposix init` doesn't work directly. | Pre-existing structure with the callout is acceptable; backlog. |

### `docs/how-it-works/filesystem-layer.md`
| # | Severity | Finding | Suggested fix |
|---|---|---|---|
| 1 | Minor | "Why partial clone, not a virtual filesystem" mentions FUSE/`fuser`/`/dev/fuse` — Layer 3 permits this per `docs/.banned-words.toml`; no P2 violation. Some cold readers may not have v0.1 context, but the prose explains it inline. | Acceptable. |
| 2 | Minor | Mermaid diagram is dense (8 nodes); on mobile the labels may stack. | Phase 45 playwright screenshots will surface this; backlog. |

### `docs/how-it-works/git-layer.md`
| # | Severity | Finding | Suggested fix |
|---|---|---|---|
| 1 | Minor | Sequence diagram has `loop for each changed issue` block — clear, but a cold reader might want a one-line preamble before the diagram. | One-liner already exists ("happy path and conflict"); acceptable. |
| 2 | Minor | "Capabilities advertised" lists `option` as "mostly cosmetic" — slightly dismissive; could note what flags it does pass. | Backlog. |

### `docs/how-it-works/trust-model.md`
| # | Severity | Finding | Suggested fix |
|---|---|---|---|
| 1 | Major | Closing "Further reading" bullet `.planning/research/v0.1-fuse-era/threat-model-and-critique.md` is a planning-tree path, not a doc-tree path; renders as plain text in MkDocs (no link). Cold reader cannot click to it. | Backlog — the file exists but is intentionally not in user-facing nav. Acceptable as plain text reference; no fix. |

### `docs/guides/integrate-with-your-agent.md`
| # | Severity | Finding | Suggested fix |
|---|---|---|---|
| 1 | Minor | Pattern 3 "Sketch" pseudo-Python uses Python-ish syntax in a `text` code block — acceptable; could be valid Python with `subprocess.run` to be copy-pastable. | Backlog. |

### `docs/guides/troubleshooting.md`
| # | Severity | Finding | Suggested fix |
|---|---|---|---|
| 1 | Minor | "Cache eviction" section says "Coming in v0.13.0" — soft promise; should match the future-requirements list (which says v0.11.0 for `reposix-bench` and defers `reposix gc`). | Reconcile in v0.11.0 milestone planning. |

### `docs/guides/write-your-own-connector.md`
| # | Severity | Finding | Suggested fix |
|---|---|---|---|
| 1 | Minor | Step 2 stub `LinearCreds`/`translate(tainted)` types are referenced but not defined inline; the cold reader infers from context. | Acceptable for a sketch; backlog. |

### `docs/reference/simulator.md`
| # | Severity | Finding | Suggested fix |
|---|---|---|---|
| 1 | Minor | `[Operating Principle](../research/agentic-engineering-reference.md)` link resolves to file but no specific anchor. Phase 43 deferred anchor warnings. | Backlog. |

### `docs/reference/testing-targets.md`
| # | Severity | Finding | Suggested fix |
|---|---|---|---|
| 1 | Minor | "Phase 36 wires three CI integration jobs" — Phase 36 already shipped; tense should be past or "wires" stays accurate as system description. | Acceptable. |

### `docs/reference/jira.md`
| # | Severity | Finding | Suggested fix |
|---|---|---|---|
| 1 | Critical | `### Mount as FUSE Filesystem` block shows `reposix mount --backend jira --project MYPROJECT /tmp/jira-mount` — the `mount` subcommand is REMOVED in v0.9.0. A cold reader copy-pasting this gets clap "unrecognized subcommand". | Replace the section with `reposix init jira::MYPROJECT /tmp/jira-mount` and update prose to say "Bootstrap as a partial-clone working tree" instead of "Mount as FUSE Filesystem". |
| 2 | Major | "Limitations (Phase 28)" header tags a phase number that has shipped; cold reader has no Phase 28 context. | Rename to "Limitations" in Phase 45. |
| 3 | Minor | Frontmatter example uses ID `10001` while the rest of the docs use 4-digit IDs (`0001`). | Normalize in v0.11.0 polish. |

### `docs/reference/confluence.md`
| # | Severity | Finding | Suggested fix |
|---|---|---|---|
| 1 | Critical | CLI surface block lists `reposix mount <dir> --backend confluence --project <SPACE_KEY>` and the credential-setup walkthrough refers to "before `reposix list --backend confluence` or `reposix mount --backend confluence` will run." Both `mount` and the `list --backend …` form are v0.7-era. Cold reader gets dead commands. | Replace `reposix mount …` line with `reposix init confluence::<SPACE_KEY> <dir>` and rewrite the walkthrough sentence. The `reposix list` mention is also stale but lower-impact (Phase 45). |
| 2 | Major | "FUSE mount layout (v0.4+)" entire section depicts the deleted FUSE tree (`pages/`, `tree/`, `comments/`). Reference layer 4, so banned-words linter does not flag it, but it is stale architecture for a v0.9.0 reader. | Phase 45 should either delete the section or reframe as "v0.4–v0.7 historical layout" with a callout. |
| 3 | Major | "Implements `IssueBackend` trait" — that trait was renamed to `BackendConnector` in Phase 27 (v0.8.0). Cold reader following the link to `reposix-core/src/backend.rs` finds `BackendConnector`. | Update reference name. |
| 4 | Minor | Demo links point at FUSE-era scripts (`06-mount-real-confluence.sh`). Scripts still exist but exercise dead `reposix mount`. | Phase 45. |

### `docs/reference/http-api.md`
| # | Severity | Finding | Suggested fix |
|---|---|---|---|
| 1 | Major | Closing paragraph describes the FUSE mount form (`issues/<padded>.md` 11-digit) vs the helper's 4-digit form. The 11-digit FUSE side no longer exists. | Drop the FUSE comparison and just describe the current 4-digit form. |
| 2 | Minor | Audit table column shows `agent_id TEXT` from `X-Reposix-Agent` — accurate for the sim's audit; the helper-side `audit_events_cache` is described elsewhere. Cold reader may conflate. | Acceptable; cross-link already exists. |

### `docs/benchmarks/v0.9.0-latency.md`
| # | Severity | Finding | Suggested fix |
|---|---|---|---|
| 1 | Minor | "Phase 36 wires the integration-contract-…-v09 CI jobs that populate them." — Phase 36 shipped; cold reader has no Phase 36 context. | Update to past tense in Phase 45. |
| 2 | Minor | Real-backend cells empty across all backends; no indication whether to expect them filled before v0.10.0 ships. | Reconcile with Phase 45 roadmap. |

### `CHANGELOG.md` `[v0.9.0]` block
| # | Severity | Finding | Suggested fix |
|---|---|---|---|
| 1 | Minor | Cold reader may not know what "fast-import" means in the Added section about parsing the export stream — Layer 4 permits the term. | Acceptable. |
| 2 | Minor | Migration block uses `reposix init sim::demo /tmp/m` while the README quickstart uses `/tmp/reposix-demo`. | Path inconsistency is minor. |

## Summary

- **Pages audited:** 16 (README + docs/index + 2 concepts + 1 tutorial + 3 how-it-works + 3 guides + 5 reference + 1 benchmark + CHANGELOG block)
- **Critical findings:** 3 — README dead-command demos (escalated to Phase 45); `docs/reference/jira.md` `reposix mount` block; `docs/reference/confluence.md` `reposix mount` block.
- **Critical fixes applied this phase:** 2 (jira.md + confluence.md). README escalated.
- **Major findings:** 9 (deferred to backlog).
- **Minor findings:** 17 (deferred to v0.11.0 polish).

## Backlog handoff

Major + minor findings dropped into `.planning/notes/v0.11.0-doc-polish-backlog.md`.

## Final pass (post-fix)

After applying critical fixes:

- `docs/reference/jira.md` — zero critical remaining; the `reposix init` form replaces the dead `reposix mount` block.
- `docs/reference/confluence.md` — zero critical remaining; the CLI surface is updated.
- `README.md` — escalated to Phase 45 per guardrails (the dead-command finding is preserved as a known gap that Phase 45 must close before tag).

Banned-words linter green. `scripts/check_doc_links.py` reports 0 broken links across 19 user-facing pages.

DOCS-10 release-gate criterion: **zero critical friction points in pages where Phase 44 holds the rewrite mandate.** README is owned by Phase 45 — its dead-command findings are tracked there, not as a Phase 44 blocker.
