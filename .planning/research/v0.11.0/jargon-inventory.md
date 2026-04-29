# v0.11.0 Jargon Inventory

**Audit Scope:** `docs/concepts/`, `docs/how-it-works/`, `docs/guides/`, `docs/tutorials/`, `docs/reference/`, `README.md` (root).
**Audit date:** 2026-04-25
**Reviewer:** automated subagent audit (cold read)

## Headline

High jargon density concentrated in the **how-it-works** cluster. **Critical gaps:** git-scm.com external links present for only 3 of 24 terms; many terms lack inline one-line glosses. Terms like `stateless-connect`, `promisor remote`, `fast-import`, and `protocol-v2` appear load-bearing but read as jargon walls to readers unfamiliar with git internals.

## Jargon Density Ranking (by pages affected)

| Jargon Term | First Appearance | Pages Affected | Inline Gloss | External Link | Load-Bearing | Recommended External Link |
|---|---|---|---|---|---|---|
| **sparse-checkout** | `docs/guides/troubleshooting.md:11` | 7 | Yes (line 69) | No | Yes — dark-factory teaching mechanism in blob-limit guardrail | https://git-scm.com/docs/git-sparse-checkout |
| **stateless-connect** | `docs/how-it-works/git-layer.md:20` | 5 | Yes (line 47) | No | Yes — read path, hybrid promisor core | https://git-scm.com/docs/gitremote-helpers#_protocol_for_stateless_connections |
| **partial clone** | `docs/how-it-works/filesystem-layer.md:34` | 6 | Yes (line 38) | No | Yes — architectural foundation v0.9.0 pivot | https://git-scm.com/docs/git-clone#Documentation/git-clone.txt---filterblobslimit |
| **promisor remote** | `docs/reference/cli.md:43` | 3 | Yes (context) | No | Yes — git remote type definition | https://git-scm.com/docs/git-config#Documentation/git-config.txt-extensionspartialClone |
| **fast-import** | `docs/how-it-works/git-layer.md:22` | 4 | Partial (line 48) | Yes — `docs/reference/git-remote.md:54` | Yes — push path serialization | https://git-scm.com/docs/git-fast-import |
| **protocol-v2** | `docs/guides/troubleshooting.md:224` | 4 | Partial (footnote) | No | Yes — tunnel target for stateless-connect reads | https://git-scm.com/docs/protocol-v2 |
| **refspec** | `docs/reference/git-remote.md:20` | 4 | Partial (line 23) | Implicit | Yes — namespace isolation critical to fast-export delta emission | https://git-scm.com/docs/gitrevisions#Documentation/gitrevisions.txt-emrefspecsem |
| **bare repo** | `docs/how-it-works/filesystem-layer.md:46` | 5 | Yes (line 46) | No | Yes — cache architecture substrate | https://git-scm.com/docs/git-init#Documentation/git-init.txt---bare |
| **Tainted&lt;T&gt;** | `docs/reference/crates.md:13` | 7 | Yes (line 13) | No | Yes — essential security boundary | (internal: `crates/reposix-core/src/tainted.rs`) |
| **audit log** | `docs/how-it-works/trust-model.md:56` | 11 | Yes (line 60) | No | Yes — exfiltration mitigation | (internal: `crates/reposix-cache/src/cache_schema.sql`) |
| **SQLite WAL** | `docs/how-it-works/filesystem-layer.md:46` | 5 | Yes (line 46) | No | Partial — append-only mechanism is load-bearing for audit immutability | https://sqlite.org/wal.html |
| **frontmatter** | `docs/concepts/mental-model-in-60-seconds.md:25` | 11 | Yes (line 25) | No | Yes — schema serialization format | https://jekyllrb.com/docs/front-matter/ |
| **YAML** | `docs/concepts/mental-model-in-60-seconds.md:25` | 4 | No inline gloss | No | Partial | https://yaml.org/spec/1.2/spec.html |
| **BackendConnector** | `docs/guides/write-your-own-connector.md:7` | 8 | Yes (line 11) | No | Yes — adapter trait, extension point | (internal: `crates/reposix-core/src/backend.rs`) |
| **gix** | `docs/how-it-works/filesystem-layer.md:46` | 1 | Link only | Yes (line 46) | No — implementation detail | https://github.com/Byron/gitoxide |
| **git remote helper** | `docs/reference/git-remote.md:3` | 3 | Yes (line 3) | Yes (line 3) | Yes — helper protocol definition | https://git-scm.com/docs/gitremote-helpers |
| **extensions.partialClone** | `docs/how-it-works/filesystem-layer.md:47` | 5 | Partial (line 47) | No | Yes — git config knob | https://git-scm.com/docs/git-config#Documentation/git-config.txt-extensionspartialClone |
| **push round-trip** | `docs/how-it-works/git-layer.md:2` | 3 | Yes (title + line 9) | No | Yes — core feature workflow | (reposix-internal) |
| **egress allowlist** | `docs/how-it-works/trust-model.md:35` | 4 | Partial (line 51) | No | Yes — exfiltration cut | (internal: `crates/reposix-core/src/http.rs`) |
| **lethal trifecta** | `docs/how-it-works/trust-model.md:7` | 3 | Yes (line 7) | Implicit (agentic-engineering-reference) | Yes — threat model framing | https://simonwillison.net/2024/Dec/19/prompt-injection/ |
| **fast-export** | `docs/how-it-works/git-layer.md:51` | 2 | Partial (line 51) | No | Yes — why refspec namespace matters (silent bug) | https://git-scm.com/docs/git-fast-export |
| **capability advertisement** | `docs/how-it-works/git-layer.md:19-20` | 3 | Yes (line 43) | No | Yes — helper wire protocol | https://git-scm.com/docs/gitremote-helpers#_protocol_overview |
| **pkt-line** | (mentioned in context, no first definition) | 0 | N/A | N/A | N/A | https://git-scm.com/docs/protocol-v2#Documentation/protocol-v2.txt-pkt-line |
| **libgit2** | (not found) | 0 | N/A | N/A | N/A | https://libgit2.org/ |

## Pages Ranked by Jargon Density

1. `docs/how-it-works/git-layer.md` — 9 terms
2. `docs/how-it-works/trust-model.md` — 7 terms
3. `docs/how-it-works/filesystem-layer.md` — 6 terms
4. `docs/reference/crates.md` — 6 terms
5. `docs/guides/troubleshooting.md` — 4 terms
6. `docs/reference/git-remote.md` — 4 terms
7. `docs/guides/write-your-own-connector.md` — 4 terms
8. `README.md` — 5 terms

## Critical Gaps

**Terms with NO inline gloss + NO external link:**
- `stateless-connect` (5 pages) — fundamental to read path
- `protocol-v2` (4 pages) — tunnel target; only footnote comment in troubleshooting
- `promisor remote` (3 pages) — git config concept
- `extensions.partialClone` (5 pages) — config knob; appears only as inline code
- `egress allowlist` (4 pages) — reposix term; inline gloss is scattered across 4 files
- `pkt-line` (used contextually but never glossed)
- `YAML` (4 pages) — no first-appearance gloss

**Terms with partial coverage:**
- `fast-import` — linked in `git-remote.md:54` but not in `git-layer.md:22` or `git-layer.md:48`
- `refspec` — implicit link via gitremote-helpers but no standalone gloss
- `bare repo` — appears 5× but only explained once in `filesystem-layer.md:46`

## Summary Table: Gloss + Link Status

| Status | Count |
|---|---|
| Gloss + External Link | 3 |
| Gloss only | 11 |
| Link only | 1 |
| No gloss, no link | 7 |
| Not found in docs | 2 |

## Highest-Impact Wins

1. Add one-liner glosses + external links for `stateless-connect`, `protocol-v2`, `promisor remote`, and `extensions.partialClone` at first appearance.
2. Consolidate `Tainted<T>` explanation into a single trust-model sidebar instead of repeating across 7 pages.
3. Link `fast-export` and `fast-import` every time they appear, not just in reference pages.
4. Move `egress allowlist` definition earlier — currently scattered intro across 4 guide pages before hitting `trust-model.md` full explanation.
5. Add a glossary page (`docs/reference/glossary.md`) listing every term once with: one-line gloss, external link, internal cross-ref. Every other page links to it on first occurrence.
